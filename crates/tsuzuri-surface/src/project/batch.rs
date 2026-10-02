//! block「まとめて承認」（問いの頁の右の列・便 g-batch）: 問いの一覧（ask の module の口）の card を 1 行ずつ出し、
//! 選んだ card を 1 つの束にして口 /api/batch へ送る。見本は docs/design/mock3/ask.html の side の id batch。
//! A-1 の印を持つ card は選べない（選ぶ box の代わりに印を出す）。ほかの行は初めは選ばれている。
//! 行の一覧・関わる所の数と重なりの数・送る button の判定・要求の本文・応答の出し方は純粋な関数にして host で試し、
//! DOM と通信は wasm の target のときだけ組み立てる。
//! 持ち主の字は送る要求の本文の外に書かない（URL にも、画面の外の保存の口にも残さない）。
//! 問いの一覧の電文の answerable が偽（読むだけの server）なら、欄と button の代わりにチャットで答える 1 行を出す
//! （行 e-ask-own-only）。

use std::collections::{BTreeMap, BTreeSet};

use tsuzuri_contract::graph::title36;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::question::QuestionCard;
use tsuzuri_contract::surface::{
    BatchItem, BatchRequest, BatchResponse, ItemOutcome, RefusalResponse, RulingId,
};
use tsuzuri_contract::wire;

use super::Body;
use super::ask::{self, BAD_REPLY, NOT_REACHED, RECORDED, REFUSED, STALE};
use crate::frame::Block;
use crate::view::Fetched;

pub const BLOCK: Block = Block {
    id: "batch",
    heading: "batch",
    class: "panel",
};

/// 束を送る口（POST・契約の型の BatchRequest・server の便 e-batch）。
pub const PATH: &str = "/api/batch";

/// この file が字を持つ口の path（ほかの module の口を読む所は数えない・行 hs-derived）。
pub const PATHS: &[&str] = &[PATH];

/// この file の畳める段の開き閉じの鍵の形（無い・行 hs-derived）。
pub const FOLDS: &[&str] = &[];

/// 書いた行の数の前の字。
pub const WRITTEN: &str = "書いた行";

/// 502 で残った行の数の前の字。
pub const LEFT: &str = "残り";

/// 追記は済み閉じる書きが落ちた行の題の後に括弧で包む字。
pub const UNCLOSED: &str = "記録したが閉じていない";

/// 重なりの chip の経験者だけの注釈（見本の id bo の chip の `data-tip-expert` の字）。
pub const OVERLAP_TIP: &str = "選んだ質問の touches の重なり（衝突の兆し）";

/// 1 行（一覧の順の 1 から始まる番号・36 字に切った題・A-1 の印・関わる所・見た版の要約値）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Row {
    pub number: usize,
    pub id: BeadId,
    pub title: String,
    pub a1: bool,
    pub touches: Vec<String>,
    pub digest: String,
}

impl Row {
    /// 選ぶ box を持つか（A-1 の印を持つ行は持たない）。
    pub fn selectable(&self) -> bool {
        !self.a1
    }
}

/// 問いの一覧の順に 1 行ずつ。
pub fn rows(cards: &[QuestionCard]) -> Vec<Row> {
    cards
        .iter()
        .enumerate()
        .map(|(i, q)| Row {
            number: i + 1,
            id: q.id.clone(),
            title: title36(&q.title),
            a1: q.a1,
            touches: q.touches.clone(),
            digest: q.digest.clone(),
        })
        .collect()
}

/// block の中身（口が読めない・まだ読んでいない・電文が読めないは測れていない・0 本は 0 件の 1 行）。
pub fn body(fetched: &Fetched) -> Body<Vec<Row>> {
    match ask::cards(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(cards) if cards.is_empty() => Body::Empty(ask::EMPTY),
        Ok(cards) => Body::Filled(rows(&cards)),
    }
}

/// 鍵つきの一覧の鍵（番号を 0 にした行・前の行が答えられて番号が詰まっても変わらない）。
pub fn row_key(row: &Row) -> Row {
    Row {
        number: 0,
        ..row.clone()
    }
}

/// block の形（Filled の中身を捨てた値・形が同じなら外枠を組み直さない）。
pub fn outline(fetched: &Fetched) -> Body<()> {
    match body(fetched) {
        Body::Unmeasured(reason) => Body::Unmeasured(reason),
        Body::Empty(line) => Body::Empty(line),
        Body::Filled(_) => Body::Filled(()),
    }
}

/// 行の列（Filled でなければ空の列）。
pub fn listed(fetched: &Fetched) -> Vec<Row> {
    match body(fetched) {
        Body::Filled(rows) => rows,
        _ => Vec::new(),
    }
}

/// id の同じ行の番号（無ければ None）。
pub fn number_of(rows: &[Row], id: &BeadId) -> Option<usize> {
    rows.iter().find(|r| r.id == *id).map(|r| r.number)
}

/// 選んだ行（一覧の順・A-1 の印を持つ行と、選ぶ box を外した行 `off` は入らない）。
/// 外した id だけを持つので、読み直しで増えた行は初めのとおり選ばれている。
pub fn selected<'a>(rows: &'a [Row], off: &BTreeSet<BeadId>) -> Vec<&'a Row> {
    rows.iter()
        .filter(|r| r.selectable() && !off.contains(&r.id))
        .collect()
}

/// 関わる所の数（選んだ card の touches の和集合の数）と重なりの数（2 つ以上の選んだ card が持つ touches の数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub touches: usize,
    pub overlap: usize,
}

pub fn counts(chosen: &[&Row]) -> Counts {
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    for row in chosen {
        let own: BTreeSet<&str> = row.touches.iter().map(String::as_str).collect();
        for t in own {
            *seen.entry(t).or_default() += 1;
        }
    }
    Counts {
        touches: seen.len(),
        overlap: seen.values().filter(|n| **n > 1).count(),
    }
}

/// 送る button を押せるか（逐語が空白だけ・選んだ card が 0 本・送っている間は押せない）。
pub fn can_send(text: &str, chosen: usize, sending: bool) -> bool {
    chosen > 0 && ask::can_send(text, sending)
}

/// 要求の本文（選んだ card を一覧の順に・見た版の要約値は card の digest の字のまま・個別の逐語は無し・
/// 束の逐語は欄の字のまま）。A-1 の印を持つ行は入れない。
pub fn request_body(chosen: &[&Row], verbatim: &str) -> String {
    let items = chosen
        .iter()
        .filter(|r| r.selectable())
        .map(|r| BatchItem {
            question: r.id.clone(),
            seen_digest: r.digest.clone(),
            verbatim: None,
        })
        .collect();
    wire::encode(&BatchRequest {
        items,
        verbatim: verbatim.to_string(),
    })
    .unwrap_or_default()
}

/// 502 で残った 1 行（問いの id・送った行の題〔無ければ問いの id の字〕・閉じていない行の裁定の id）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Left {
    pub question: BeadId,
    pub title: String,
    /// 閉じていない行は notes に残った裁定の id・書いていない行は None。
    pub ruling: Option<RulingId>,
}

impl Left {
    /// 残りの名（閉じていない行は題の後に括弧で包んだ字）。
    fn name(&self) -> String {
        match self.ruling {
            Some(_) => format!("{}（{UNCLOSED}）", self.title),
            None => self.title.clone(),
        }
    }
}

/// 送った後の block の状態。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// 200: 束の id と書いた行の数。
    Recorded { batch: RulingId, written: usize },
    /// 409: 質問が更新された（一覧を読み直す）。
    Stale,
    /// 502 で本文が束の応答として読めた: 送れなかったと書いた行の数と残った行（一覧を読み直す）。
    Partial { written: usize, left: Vec<Left> },
    /// ほかの 4xx・5xx・届かない: 理由の字。
    Refused(String),
}

impl Outcome {
    /// block に出す 1 行。
    pub fn line(&self) -> String {
        match self {
            Outcome::Recorded { batch, written } => {
                format!("{RECORDED} {batch} · {WRITTEN} {written}")
            }
            Outcome::Stale => STALE.to_string(),
            Outcome::Partial { written, left } if left.is_empty() => {
                format!("{REFUSED}（502） · {WRITTEN} {written}")
            }
            Outcome::Partial { written, left } => {
                let names: Vec<String> = left.iter().map(Left::name).collect();
                format!(
                    "{REFUSED}（502） · {WRITTEN} {written} · {LEFT} {} · {}",
                    left.len(),
                    names.join("、")
                )
            }
            Outcome::Refused(reason) => format!("{REFUSED}: {reason}"),
        }
    }

    /// 欄の字を残すか（200 の外は残す）。
    pub fn keeps_text(&self) -> bool {
        !matches!(self, Outcome::Recorded { .. })
    }

    /// 口を全部読み直すか（200・409・502 の束の応答）。
    pub fn reloads(&self) -> bool {
        !matches!(self, Outcome::Refused(_))
    }
}

/// 書いた行の数（行ごとの結果が書いたの行）。
pub fn written(reply: &BatchResponse) -> usize {
    reply
        .items
        .iter()
        .filter(|i| matches!(i.outcome, ItemOutcome::Written { .. }))
        .count()
}

/// ほかの 4xx と 5xx の理由の字（本文が断りの応答なら理由の字・text なら状態の数と本文の字）。
pub fn refused_text(status: u16, text: &str) -> String {
    match wire::decode::<RefusalResponse>(text) {
        Ok(r) => format!("{}（{status}）", ask::refusal_text(r.reason)),
        Err(_) => format!("状態 {status}: {}", text.trim()),
    }
}

/// 502 の束の応答の残った行（閉じていないと書いていないの行を応答の順に・題は `sent` の同じ id の行の題）。
fn left(reply: &BatchResponse, sent: &[Row]) -> Vec<Left> {
    reply
        .items
        .iter()
        .filter_map(|i| {
            let ruling = match &i.outcome {
                ItemOutcome::Unclosed { ruling } => Some(ruling.clone()),
                ItemOutcome::Unwritten => None,
                _ => return None,
            };
            let title = sent
                .iter()
                .find(|r| r.id == i.question)
                .map_or_else(|| i.question.to_string(), |r| r.title.clone());
            Some(Left {
                question: i.question.clone(),
                title,
                ruling,
            })
        })
        .collect()
}

/// 応答から block の状態を決める（`reply` は状態の数と本文の字・届かなければ None・`sent` は送った行）。
pub fn outcome(reply: Option<(u16, &str)>, sent: &[Row]) -> Outcome {
    match reply {
        None => Outcome::Refused(NOT_REACHED.to_string()),
        Some((200, text)) => match wire::decode::<BatchResponse>(text) {
            Ok(r) => Outcome::Recorded {
                written: written(&r),
                batch: r.batch,
            },
            Err(_) => Outcome::Refused(format!("{BAD_REPLY}（200）")),
        },
        Some((409, _)) => Outcome::Stale,
        Some((502, text)) => match wire::decode::<BatchResponse>(text) {
            Ok(r) => Outcome::Partial {
                written: written(&r),
                left: left(&r, sent),
            },
            Err(_) => Outcome::Refused(refused_text(502, text)),
        },
        Some((status, text)) => Outcome::Refused(refused_text(status, text)),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

/// 質問の窓の下の段の中身（見出しの無い本文・行 g-ask-win）。
#[cfg(target_arch = "wasm32")]
pub fn inner() -> leptos::prelude::AnyView {
    dom::inner()
}

/// まとめて承認の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use std::collections::BTreeSet;

    use leptos::ev;
    use leptos::prelude::*;
    use leptos::task::spawn_local;
    use tsuzuri_contract::ledger::BeadId;

    use super::{
        BLOCK, OVERLAP_TIP, Outcome, PATH, Row, can_send, counts, listed, number_of, outcome,
        outline, request_body, row_key, selected,
    };
    use crate::project::ask::{self, KeyAction, key_action};
    use crate::project::{Body, body_view, section, unmeasured};
    use crate::vocab::label;
    use crate::widgets::help::expert_tip;

    /// 見本の IC.warn・IC.link。
    const WARN: &str = r#"<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><path d="M12 3l10 18H2z"/><path d="M12 10v5"/><circle cx="12" cy="18" r=".8" fill="currentColor"/></svg>"#;
    const LINK: &str = r#"<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1"/><path d="M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"/></svg>"#;

    /// block の状態（選ぶ box を外した id・欄の字・送っている間か・送った後の 1 行）。
    #[derive(Clone, Copy)]
    struct State {
        off: RwSignal<BTreeSet<BeadId>>,
        text: RwSignal<String>,
        sending: RwSignal<bool>,
        outcome: RwSignal<Option<Outcome>>,
    }

    pub fn view() -> AnyView {
        section(BLOCK, ().into_any(), inner())
    }

    /// block の本文（見出しの無い中身・質問の窓の下の段も使う）。
    pub fn inner() -> AnyView {
        let fetched = crate::net::read(ask::PATH);
        let s = State {
            off: RwSignal::new(BTreeSet::new()),
            text: RwSignal::new(String::new()),
            sending: RwSignal::new(false),
            outcome: RwSignal::new(None),
        };
        // 形と行の列は値が前と同じなら知らせない（形が Filled のまま替わらなければ欄と button を作り直さない）。
        let shape = Memo::new(move |_| fetched.with(outline));
        let rows = Memo::new(move |_| fetched.with(listed));
        // 読むだけの server なら欄と button の代わりにチャットで答える 1 行を出す（行 e-ask-own-only）。
        let can_answer = Memo::new(move |_| fetched.with(ask::answerable));
        let content = move || match shape.get() {
            Body::Unmeasured(reason) => unmeasured(reason),
            Body::Empty(line) => body_view(Body::Empty(line)),
            Body::Filled(()) if !can_answer.get() => {
                view! { <div class="small muted" data-term=ask::CHAT_KEY>{label(ask::CHAT_KEY)}</div> }
                    .into_any()
            }
            Body::Filled(()) => filled(rows, s),
        };
        let note = move || {
            s.outcome
                .get()
                .map(|o| view! { <div class="small" role="status">{o.line()}</div> })
        };
        view! { {content}{note} }.into_any()
    }

    fn filled(rows: Memo<Vec<Row>>, s: State) -> AnyView {
        let chosen = move || rows.with(|rows| s.off.with(|off| counts(&selected(rows, off))));
        let n = move || rows.with(|rows| s.off.with(|off| selected(rows, off).len()));
        let disabled = move || !can_send(&s.text.get(), n(), s.sending.get());
        let keydown = move |ev: ev::KeyboardEvent| {
            let composing = ev.is_composing() || ev.key_code() == 229;
            if key_action(composing, ev.ctrl_key() || ev.meta_key(), &ev.key()) == KeyAction::Send {
                ev.prevent_default();
                submit(rows, s);
            }
        };
        let own = label("own_words");
        view! {
            <ul class="items">
                <For each=move || rows.get() key=row_key children=move |r: Row| row_view(&r, rows, s)/>
            </ul>
            <div class="metas row">
                <span class="chip num" data-term="touches"><span inner_html=LINK></span>{label("touches")}" "{move || chosen().touches}</span>
                <span class="chip num" use:expert_tip=OVERLAP_TIP.to_string()><span inner_html=WARN></span>" "{move || chosen().overlap}</span>
            </div>
            <textarea rows="2" aria-label=own.clone() placeholder=own prop:value=move || s.text.get() on:input=move |ev: ev::Event| s.text.set(event_target_value(&ev)) on:keydown=keydown></textarea>
            <button type="button" class="btn primary" disabled=disabled on:click=move |_| submit(rows, s)>{label("batch")}" ›"</button>
        }
        .into_any()
    }

    /// 1 行（A-1 の印を持つ行は印と注の記号・ほかは選ぶ box）。番号は行の列から読み直す。
    fn row_view(r: &Row, rows: Memo<Vec<Row>>, s: State) -> AnyView {
        let number = {
            let id = r.id.clone();
            Memo::new(move |_| rows.with(|v| number_of(v, &id)).unwrap_or(0))
        };
        let lead = if r.selectable() {
            let id = r.id.clone();
            let checked = {
                let id = id.clone();
                move || s.off.with(|off| !off.contains(&id))
            };
            let change = move |ev: ev::Event| {
                let on = event_target_checked(&ev);
                s.off.update(|off| {
                    if on {
                        off.remove(&id);
                    } else {
                        off.insert(id.clone());
                    }
                });
            };
            view! { <input type="checkbox" aria-label=move || number.get().to_string()prop:checked=checked on:change=change/> }
                .into_any()
        } else {
            view! { <span class="gi ng" aria-hidden="true">"!"</span> }.into_any()
        };
        let a1 = r.a1.then(|| {
            view! { <span class="warn" data-term="a1" tabindex="0" aria-label=label("a1") inner_html=WARN></span> }
        });
        view! {
            <li>
                {lead}
                <span class="nb">{move || number.get()}</span>
                <span class="ttl" data-t="">{r.title.clone()}</span>
                {a1}
            </li>
        }
        .into_any()
    }

    /// 束を送る（押せないときは何もしない）。送る前に選んだ行を写し、応答をその写しと一緒に読んで block の状態を決め、
    /// 読み直す応答は口を全部読み直す。
    fn submit(rows: Memo<Vec<Row>>, s: State) {
        let text = s.text.get_untracked();
        let body = rows.with_untracked(|rows| {
            s.off.with_untracked(|off| {
                let chosen = selected(rows, off);
                can_send(&text, chosen.len(), s.sending.get_untracked()).then(|| {
                    let sent: Vec<Row> = chosen.iter().map(|r| (*r).clone()).collect();
                    (request_body(&chosen, &text), sent)
                })
            })
        });
        let Some((body, sent)) = body else {
            return;
        };
        s.sending.set(true);
        spawn_local(async move {
            let reply = crate::net::post(PATH, body).await;
            let out = outcome(reply.as_ref().map(|(st, t)| (*st, t.as_str())), &sent);
            if !out.keeps_text() {
                s.text.set(String::new());
            }
            let reload = out.reloads();
            s.outcome.set(Some(out));
            s.sending.set(false);
            if reload {
                crate::net::reload_all();
            }
        });
    }
}
