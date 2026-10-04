//! block「節点」（見本の bead.html の頭と概要・便 g-node）: 節点の頁の 1 つ目の block。
//! 近傍の口の電文（block「つながり」と同じ 1 つの読み・nodearound の module が持つ）の中心の行（列 0）から頭と概要を組む。
//! 頭・概要・質問の頁への link は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。
//! 概要は表示の型の 1 つを切らずに 1 つの箱に出し、もう一方は箱の下に畳んで置く（行 g-sum-pick・判断の記録 ADR-30 決定 (2)）。
//! 2 つの概要が両方無い bead は、本文の頭の 1 行を「本文から」の印つきで箱に出す。本文は block「本文と記録」と同じ
//! 1 本の引きの読み（nodebody の module の `item_source`）から取り、同じ口を 2 度読まない（行 g-node-excerpt）。
//! 決定の頁（中心が あなたの決定）の頭には、理由の欄と 取り消す の button を置く（行 e-revoke）。
//! 出すのは問いの 1 本の引きを読み、その決定が閉じた問いの効いている最後の決定のときだけ（server の受付と同じ関数で判じる）。
//! 取り消しで戻るのは台帳だけで、問いは未回答に戻る。開き直しだけが落ちた後も button は残り、
//! 撃ち直しは同じ button をもう一度押す（server は行を足さず開き直しだけを撃つ）。

use tsuzuri_contract::graph::{AroundDoc, AroundRow, NodeKind, title36};
use tsuzuri_contract::ledger::{BeadId, ITEM_PATH, LedgerItem};
use tsuzuri_contract::summary::excerpt;
use tsuzuri_contract::surface::{RevokeRequest, RulingId, revocable};
use tsuzuri_contract::wire;

use crate::frame::{Block, Mode};
use crate::mapview::band::{Band, band_of, kind_key};
use crate::mapview::{encode, is_open};
use crate::project::nodearound::PageState;
use crate::topbar::{Win, win_href};
use crate::view::Fetched;
use crate::vocab::label;
use crate::widgets::nodecard::full_src;
use crate::widgets::pop::UNKNOWN_KEY;
use crate::widgets::sumpick::{self, Picked};

pub const BLOCK: Block = Block {
    id: "node",
    heading: "nb_summary",
    class: "stack",
};

/// この file が字を持つ口の path（無い・近傍の口は nodearound の module が持つ・行 hs-derived）。
pub const PATHS: &[&str] = &[];

/// この file の畳める段の開き閉じの鍵の形（概要の箱の下のもう一方の概要・行 hs-derived・行 g-sum-pick）。
pub const FOLDS: &[&str] = &["node:other"];

/// 電文の節点に概要の字が無いときの概要の字。
pub const NO_SUMMARY: &str = "要約なし";

/// 出所の file を持たない節点（台帳と走行）の出所の字。
pub const NO_SRC: &str = "出所なし";

/// 要約の無い概要の箱の class。
pub const SUMMARY_NONE: &str = "sumbox none";

/// 表示の型の側でない概要を出す箱の class（見出しの語がどちら向けかの印）。
pub const SUMMARY_MARKED: &str = "sumbox marked";

/// 箱の下に畳むもう一方の概要の class。
pub const SUMBOX_OTHER: &str = "fold sumother";

/// 節点の頁の頭（印・種類の見出し・帯・状態・id・題・出所・質問の頁への link）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Head {
    /// 印の class（帯の色・動いている節点は塗らない・大きい印）。
    pub shape: String,
    /// open の問いの印は赤。
    pub alert: bool,
    /// 種類の見出しの語の鍵（「k:」に種類の名）。
    pub kind_key: &'static str,
    pub band: Band,
    /// bead は状態の字・走行は段の字・ほかは None。
    pub state: Option<String>,
    pub id: String,
    /// 題（36 字で切る）。
    pub title: String,
    /// 出所の file とコロンと行（行が無ければ file と「（行は測れていない）」・file が無ければ「出所なし」）。
    pub src: String,
    /// 中心が open の問いなら質問の頁への link を出す。
    pub answer: bool,
}

/// 電文の中心の行（列 0）。
pub fn center(doc: &AroundDoc) -> Option<&AroundRow> {
    doc.rows.iter().find(|r| r.col == 0)
}

/// 頭（中心の行から組む・中心の行が無ければ None）。
pub fn head(doc: &AroundDoc) -> Option<Head> {
    let row = center(doc)?;
    let band = band_of(row.node.kind);
    let state = matches!(band, Band::Beads | Band::Pipeline)
        .then(|| row.status.clone())
        .flatten();
    let alert = row.node.kind == NodeKind::Question && state.as_deref() == Some("open");
    let open = alert || is_open(state.as_deref());
    Some(Head {
        shape: format!(
            "shape {} big{}",
            band.class_name(),
            if open { "" } else { " fill" }
        ),
        alert,
        kind_key: kind_key(row.node.kind),
        band,
        state,
        id: row.node.id.clone(),
        title: title36(&row.node.title),
        src: match row.node.file {
            Some(_) => full_src(&row.node),
            None => NO_SRC.to_string(),
        },
        answer: alert,
    })
}

/// 概要の箱（見出しの語の鍵・class・字・箱の下に畳むもう一方の概要）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SumBox {
    pub key: &'static str,
    pub class: &'static str,
    pub text: String,
    pub other: Option<Picked>,
}

/// 概要の箱に渡す本文の頭の 1 行（行 g-node-excerpt）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Excerpt {
    /// 中心が台帳の bead でない（1 本の引きを読まない・設計の節点と走行と決定）。
    Never,
    /// 1 本の引きをまだ読んでいない・口が読めない・電文が読めない・ほかの bead の電文の間。
    Unread,
    /// 読んだ本文の頭の 1 行（本文に行が無ければ None）。
    Read(Option<String>),
}

/// 中心の bead の 1 本の引きの電文から本文の頭の 1 行（bead が None なら Never・電文の行の id が bead と同じ時だけ読む）。
pub fn excerpt_of(item: &Fetched, bead: Option<&BeadId>) -> Excerpt {
    let Some(bead) = bead else {
        return Excerpt::Never;
    };
    match item {
        Fetched::Body(text) => match wire::decode::<LedgerItem>(text) {
            Ok(it) if it.row.id == *bead => Excerpt::Read(excerpt(&it.description)),
            _ => Excerpt::Unread,
        },
        Fetched::NotRead | Fetched::Failed => Excerpt::Unread,
    }
}

/// 概要の箱（本文の頭の 1 行を渡さない形・`summary_in` に `Excerpt::Never` を渡す）。
pub fn summary(center: &AroundRow, mode: Mode) -> SumBox {
    summary_in(center, mode, &Excerpt::Never)
}

/// 概要の箱（中心の節点の plain と eng から表示の型の 1 つを切らずに写す・両方無いか空なら本文の頭の 1 行を本文からの
/// 印つきで・それも無ければ表示の型の側の見出しで要約なし・本文をまだ読めていない間は要約なしでなくまだ分からない）。
pub fn summary_in(center: &AroundRow, mode: Mode, body: &Excerpt) -> SumBox {
    let (plain, eng) = (center.node.plain.as_deref(), center.node.eng.as_deref());
    let other = sumpick::other(mode, plain, eng);
    let line = match body {
        Excerpt::Read(line) => line.as_deref(),
        Excerpt::Never | Excerpt::Unread => None,
    };
    match sumpick::pick(mode, plain, eng, line) {
        Some(p) => SumBox {
            key: p.key,
            class: if p.marked { SUMMARY_MARKED } else { "sumbox" },
            text: p.text,
            other,
        },
        None => SumBox {
            key: match mode {
                Mode::Beginner => sumpick::PLAIN_KEY,
                Mode::Expert => sumpick::ENG_KEY,
            },
            class: SUMMARY_NONE,
            text: match body {
                Excerpt::Unread => label(UNKNOWN_KEY),
                Excerpt::Never | Excerpt::Read(_) => NO_SUMMARY.to_string(),
            },
            other,
        },
    }
}

/// 質問の窓への link（home の頁を質問の窓を開いて読み直す・問いの id を `%XX` にして残す・mode を URL に残す・
/// 行 g-one-screen-a）。
pub fn answer_href(id: &str, mode: Mode) -> String {
    format!("{}&id={}", win_href(Win::Ask, mode), encode(id))
}

/// 頁の題の語に替える字（行 g-title）: 電文なら中心の行の題（空白だけ・中心の行が無ければ None）、
/// 見つからないなら None、まだ読んでいない・読めないなら前の値（読み直しの間も題を保つ）。
pub fn kept_subject(before: Option<String>, state: &PageState) -> Option<String> {
    match state {
        PageState::Doc(doc) => center(doc)
            .map(|row| row.node.title.clone())
            .filter(|t| !t.trim().is_empty()),
        PageState::NotFound => None,
        PageState::NotRead | PageState::Unread(_) => before,
    }
}

/// 取り消しの的（問いの id と取り消す決定の id）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Revoke {
    pub question: BeadId,
    pub ruling: RulingId,
}

/// 決定の頁の取り消しの的（中心の行が決定でなければ None・問いの id は決定の id の最初のコロンの前から読む・
/// 近傍の行の問いの行には頼らない）。
pub fn revoke_target(doc: &AroundDoc) -> Option<Revoke> {
    let row = center(doc)?;
    if row.node.kind != NodeKind::Ruling {
        return None;
    }
    let ruling = RulingId::new(row.node.id.clone()).ok()?;
    let (question, _) = row.node.id.split_once(':')?;
    Some(Revoke {
        question: BeadId::new(question).ok()?,
        ruling,
    })
}

/// 問いの 1 本の引きの口の path。
pub fn item_path(question: &BeadId) -> String {
    format!("{ITEM_PATH}{question}")
}

/// 取り消しの要求の本文（逐語は理由の欄の字のまま・電文の字にできなければ空の本文で、server が断る）。
pub fn revoke_body(t: &Revoke, verbatim: &str) -> String {
    wire::encode(&RevokeRequest {
        question: t.question.clone(),
        ruling: t.ruling.clone(),
        verbatim: verbatim.to_string(),
    })
    .unwrap_or_default()
}

/// 取り消しの欄を出すか（問いの 1 本の引きの電文で、その決定を取り消せるときだけ）。
pub fn shows_revoke(item: &Fetched, ruling: &RulingId) -> bool {
    match item {
        Fetched::Body(text) => {
            wire::decode::<LedgerItem>(text).is_ok_and(|item| revocable(&item, ruling))
        }
        Fetched::NotRead | Fetched::Failed => false,
    }
}

#[cfg(target_arch = "wasm32")]
pub use dom::view;

/// 節点の block の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::ev;
    use leptos::prelude::*;
    use leptos::task::spawn_local;
    use tsuzuri_contract::surface::{REVOKE_PATH, RulingId};

    use super::{
        BLOCK, Excerpt, Head, PageState, Revoke, SUMBOX_OTHER, SUMMARY_NONE, SumBox, answer_href,
        center, excerpt_of, head, item_path, kept_subject, revoke_body, revoke_target,
        shows_revoke, summary_in,
    };
    use crate::mapview::band_chip;
    use crate::project::ask::{Outcome, can_send, outcome};
    use crate::project::nodearound::{id_of, mode_of, source, state, unmeasured_reason};
    use crate::project::nodebody::item_source;
    use crate::project::{ALERT_STYLE, NO_CONTENT, UNKNOWN, fold, state_icon, unmeasured};
    use crate::view::{Fetched, PageSubject};
    use crate::vocab::label;
    use crate::widgets::help::{h1, h2};
    use crate::widgets::sumpick::{ENG_KEY, PLAIN_KEY};

    /// 出所の印（見本の IC.file）。
    const FILE_ICON: &str = r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M6 3h8l4 4v14H6z"/><path d="M14 3v4h4"/></svg>"#;
    /// 非エンジニア向けの概要の印（見本の IC.person）。
    const PERSON: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="12" cy="8" r="4"/><path d="M4 21c1-4 4-6 8-6s7 2 8 6"/></svg>"#;
    /// エンジニア向けの概要の印（見本の IC.code）。
    const CODE: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M8 7l-5 5 5 5M16 7l5 5-5 5M14 4l-4 16"/></svg>"#;

    /// 1 つの決定の取り消しの状態（頁を読み直しても残すので、決定の id ごとに block が持つ）。
    #[derive(Clone)]
    struct Draft {
        text: ArcRwSignal<String>,
        sending: ArcRwSignal<bool>,
        outcome: ArcRwSignal<Option<Outcome>>,
    }

    type Drafts = StoredValue<Vec<(RulingId, Draft)>>;

    /// 決定の id の取り消しの状態（初めての id は空で作る）。
    fn draft(drafts: Drafts, id: &RulingId) -> Draft {
        let found = drafts.with_value(|v| v.iter().find(|(k, _)| k == id).map(|(_, d)| d.clone()));
        found.unwrap_or_else(|| {
            let d = Draft {
                text: ArcRwSignal::new(String::new()),
                sending: ArcRwSignal::new(false),
                outcome: ArcRwSignal::new(None),
            };
            drafts.update_value(|v| v.push((id.clone(), d.clone())));
            d
        })
    }

    /// 節点の block（見つからないときは見出しと id・読めないときは測れていないと理由の 1 行）。
    pub fn view() -> AnyView {
        let src = source();
        let search = src.search;
        let id = search.with_untracked(|s| id_of(s));
        let Some(read) = src.read else {
            return not_found(id);
        };
        // 頁の題の語に節点の題を置く（context に PageSubject が無ければ置かない・行 g-title）。
        if let Some(subject) = use_context::<RwSignal<PageSubject>>() {
            Effect::new(move |_| {
                let st = read.with(|(f, s)| state(f, *s));
                let next = subject.with_untracked(|PageSubject(s)| kept_subject(s.clone(), &st));
                if subject.with_untracked(|PageSubject(s)| *s != next) {
                    subject.set(PageSubject(next));
                }
            });
        }
        let mode = mode_of(search);
        // 中心の bead の本文の頭の 1 行は、block「本文と記録」と同じ 1 本の引きの読みから取る（同じ口を 2 度読まない・行 g-node-excerpt）。
        let body = item_source();
        // 決定の頁の取り消しの的と、その問いの 1 本の引き（的が無ければ空の path で読まない）。
        let target = Memo::new(move |_| match read.with(|(f, s)| state(f, *s)) {
            PageState::Doc(doc) => revoke_target(&doc),
            _ => None,
        });
        let item = crate::net::read_path(Signal::derive(move || {
            target.with(|t| {
                t.as_ref()
                    .map(|t| item_path(&t.question))
                    .unwrap_or_default()
            })
        }));
        let drafts: Drafts = StoredValue::new(Vec::new());
        let content = move || {
            let st = read.with(|(f, s)| state(f, *s));
            if let Some(reason) = unmeasured_reason(&st) {
                return unmeasured(reason);
            }
            match st {
                PageState::Doc(doc) => match (head(&doc), center(&doc)) {
                    (Some(h), Some(c)) => {
                        let excerpt = body.map_or(Excerpt::Never, |s| {
                            s.bead
                                .with(|b| s.item.with(|(f, _)| excerpt_of(f, b.as_ref())))
                        });
                        let boxes = sum_view(summary_in(c, mode(), &excerpt));
                        let revoke = target.get().map(|t| {
                            let d = draft(drafts, &t.ruling);
                            revoke_view(t, item, d)
                        });
                        view! { {head_view(h, mode, revoke)}<div class="nsum">{boxes}</div> }
                            .into_any()
                    }
                    _ => unmeasured(NO_CONTENT),
                },
                _ => not_found(id.clone()),
            }
        };
        view! { <section class=BLOCK.class id=BLOCK.id>{content}</section> }.into_any()
    }

    /// 見出しの語の鍵の印（非エンジニア向けは人・エンジニア向けは code・本文からは file）。
    fn icon_of(key: &str) -> &'static str {
        match key {
            PLAIN_KEY => PERSON,
            ENG_KEY => CODE,
            _ => FILE_ICON,
        }
    }

    /// 概要の箱と、その下に畳んだもう一方の概要（押すと開く）。
    fn sum_view(b: SumBox) -> AnyView {
        let t = (b.class != SUMMARY_NONE).then_some("");
        let other = b.other.map(|o| {
            let (open, toggle) = fold("node:other".to_string(), || false);
            view! {
                <details class=SUMBOX_OTHER prop:open=open on:toggle=toggle><summary><span inner_html=icon_of(o.key)></span>{h2(o.key)}</summary><p data-t="">{o.text}</p></details>
            }
        });
        view! {
            <section class=b.class><header><span inner_html=icon_of(b.key)></span>{h2(b.key)}</header><p data-t=t>{b.text}</p></section>
            {other}
        }
        .into_any()
    }

    /// 見つからない（見出しと、id が在れば id の字）。
    fn not_found(id: Option<String>) -> AnyView {
        let line = id.map(|id| {
            view! { <div class="empty">{state_icon(UNKNOWN)}<span data-t="">{id}</span></div> }
        });
        view! { {h1("not_found")}{line} }.into_any()
    }

    /// 頭（印・種類の見出し・帯の chip・状態・id・題・出所・質問の頁への link・決定の頁の取り消しの欄）。
    fn head_view(
        h: Head,
        mode: impl Fn() -> crate::frame::Mode + Send + Sync + 'static,
        revoke: Option<AnyView>,
    ) -> AnyView {
        let style = if h.alert { ALERT_STYLE } else { "" };
        let state = h.state.map(|s| view! { <span>{s}</span> });
        let answer = h.answer.then(|| {
            let id = h.id.clone();
            view! {
                <a class="btn sm primary" href=move || answer_href(&id, mode())>{label("answer_here")}" ›"</a>
            }
        });
        view! {
            <div class="nhead">
                <span class=h.shape style=style aria-hidden="true"></span>
                <div style="min-width:0">
                    <div class="kind">{h1(h.kind_key)}{band_chip(h.band)}{state}</div>
                    <div class="cid">{h.id}</div>
                    <div class="t" data-t="">{h.title}</div>
                    <div class="srcline" data-term="src" tabindex="0"><span inner_html=FILE_ICON></span><span>{h.src}</span></div>
                    {answer}
                    {revoke}
                </div>
            </div>
        }
        .into_any()
    }

    /// 取り消しの欄（理由の欄と 取り消す の button）と、送った後の 1 行。
    /// 欄は取り消せる決定のときだけ出し、200 の後は閉じる（開き直しだけが落ちた 502 は欄の字を残す）。
    fn revoke_view(t: Revoke, item: ReadSignal<(Fetched, Option<u16>)>, d: Draft) -> AnyView {
        let form = {
            let d = d.clone();
            move || {
                let shown = item.with(|(f, _)| shows_revoke(f, &t.ruling))
                    && d.outcome
                        .with(|o| o.as_ref().is_none_or(Outcome::answer_open));
                shown.then(|| {
                    let (text, sending) = (d.text.clone(), d.sending.clone());
                    let disabled = move || !can_send(&text.get(), sending.get());
                    let value = {
                        let text = d.text.clone();
                        move || text.get()
                    };
                    let input = {
                        let text = d.text.clone();
                        move |ev: ev::Event| text.set(event_target_value(&ev))
                    };
                    let click = {
                        let (t, d) = (t.clone(), d.clone());
                        move |_: ev::MouseEvent| submit(&t, &d)
                    };
                    view! {
                        <div class="answer">
                            <textarea rows="2" aria-label=label("own_words") placeholder=label("own_words") prop:value=value on:input=input></textarea>
                            <button type="button" class="btn" data-term="revoke" disabled=disabled on:click=click>{label("revoke")}</button>
                        </div>
                    }
                })
            }
        };
        let note = {
            let outcome = d.outcome.clone();
            move || {
                outcome
                    .get()
                    .map(|o| view! { <div class="small" role="status">{o.line()}</div> })
            }
        };
        view! { {form}{note} }.into_any()
    }

    /// 取り消しを送る（押せないときは何もしない）。応答で欄の状態を決め、200 と 409 は頁を読み直す。
    fn submit(t: &Revoke, d: &Draft) {
        let text = d.text.get_untracked();
        if !can_send(&text, d.sending.get_untracked()) {
            return;
        }
        d.sending.set(true);
        let body = revoke_body(t, &text);
        let d = d.clone();
        spawn_local(async move {
            let reply = crate::net::post(REVOKE_PATH, body).await;
            let out = outcome(reply.as_ref().map(|(s, t)| (*s, t.as_str())));
            if !out.keeps_text() {
                d.text.set(String::new());
            }
            let reload = out.reloads();
            d.outcome.set(Some(out));
            d.sending.set(false);
            if reload {
                crate::net::reload_all();
            }
        });
    }
}
