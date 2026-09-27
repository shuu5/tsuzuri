//! block「答えを待つ質問」（問いの頁・便 g-ask）: 口 /api/questions の card を電文の順（古い順）に番号つきで出し、
//! card ごとに答えの欄から口 /api/ruling へ答えを送る。見本は docs/design/mock3/ask.html の qcard。
//! 問いの見分けは server の側（label intake:question）が済ませていて、面は bd の種類で選ばず電文を写すだけ。
//! card の部分の並び（配置の表）・card の中身・送る button の状態・鍵の判定・要求の本文・応答から card の状態を決める関数は
//! 純粋な関数にして host で試し、DOM と通信は wasm の target のときだけ組み立てる。
//! 持ち主の字は送る要求の本文の外に書かない（URL にも、画面の外の保存の口にも残さない）。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::question::{QuestionCard, QuestionList};
use tsuzuri_contract::surface::{Refusal, RefusalResponse, RulingId, RulingRequest, RulingResponse};
use tsuzuri_contract::wire;

use super::{Body, NOT_READ};
use crate::frame::Block;
use crate::view::{Fetched, clock};
use crate::widgets::hover::clip;

pub const BLOCK: Block = Block {
    id: "ask",
    heading: "ask_open",
    class: "panel",
};

/// 問いの一覧の口（契約の型の QuestionList）。
pub const PATH: &str = "/api/questions";

/// 答えを送る口（POST・契約の型の RulingRequest・server の便 e-ask）。
pub const RULING_PATH: &str = "/api/ruling";

/// 口が読めないときの理由。
pub const REASON: &str =
    "問いの一覧の口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 本文が電文として読めないときの理由。
pub const UNREADABLE: &str = "問いの一覧の本文が電文として読めない";

/// server が台帳を読めず card が「まだ分からない」ときの理由。
pub const CARDS_UNKNOWN: &str = "server が台帳を読めず、答えを待つ質問が分からない";

/// 測れて 0 件のときの 1 行。
pub const EMPTY: &str = "答えを待つ question は無い";

/// 概要が両方無いときの 1 行。
pub const NO_SUMMARY: &str = "要約なし";

/// 概要の片方が無いときの字。
pub const MISSING: &str = "―";

/// 200 の応答の後に card に出す字（記録した id と時刻の前）。
pub const RECORDED: &str = "記録した";

/// 409 の応答の後に card に出す字（一覧は読み直す）。
pub const STALE: &str = "質問が更新された";

/// 口に届かないときの理由の字。
pub const NOT_REACHED: &str = "口に届かない";

/// 200 の応答の本文が電文として読めないときの理由の字。
pub const BAD_REPLY: &str = "応答が電文として読めない";

/// 断られたときの字（理由の前）。
pub const REFUSED: &str = "送れなかった";

/// card の部分（見本の qcard の順: 題・概要・理由・推奨・答えの欄・つながり）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// 頭: 番号・題・A-1 の印・置かれてからの経過。
    Head,
    Summary,
    Reason,
    Recommend,
    Answer,
    Around,
}

/// 配置の表の 1 行（部分・class・語の鍵・畳める段なら最初に開いているか）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub part: Part,
    pub class: &'static str,
    pub key: Option<&'static str>,
    pub open: Option<bool>,
}

/// card の部分の並び（DOM はこの表を順にたどって組み立てる）。
pub const LAYOUT: [Slot; 6] = [
    Slot {
        part: Part::Head,
        class: "head",
        key: None,
        open: None,
    },
    Slot {
        part: Part::Summary,
        class: "qsum",
        key: None,
        open: None,
    },
    Slot {
        part: Part::Reason,
        class: "qreason",
        key: Some("reason"),
        open: None,
    },
    Slot {
        part: Part::Recommend,
        class: "rec",
        key: Some("recommend"),
        open: None,
    },
    Slot {
        part: Part::Answer,
        class: "answer",
        key: Some("own_words"),
        open: None,
    },
    Slot {
        part: Part::Around,
        class: "nb-d",
        key: Some("around"),
        open: Some(false),
    },
];

/// 概要の 1 行（class・語の鍵・字・エンジニア向けか）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SumLine {
    pub class: &'static str,
    pub key: Option<&'static str>,
    pub text: String,
    pub eng: bool,
}

/// 1 本の card の中身（番号は 1 から・題は 36 字で切る）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub number: usize,
    pub id: BeadId,
    pub title: String,
    pub a1: bool,
    pub posted_at: EpochSecs,
    pub summary: Vec<SumLine>,
    pub reason: String,
    pub recommend: String,
    pub touches: Vec<String>,
    pub digest: String,
}

/// 口の本文を card の列に読む（まだ読んでいない・読めない・電文が読めない・まだ分からないは理由）。
pub fn cards(fetched: &Fetched) -> Result<Vec<QuestionCard>, &'static str> {
    match fetched {
        Fetched::NotRead => Err(NOT_READ),
        Fetched::Failed => Err(REASON),
        Fetched::Body(text) => match wire::decode::<QuestionList>(text) {
            Ok(QuestionList {
                cards: Reading::Known(cards),
            }) => Ok(cards),
            Ok(QuestionList {
                cards: Reading::Unknown,
            }) => Err(CARDS_UNKNOWN),
            Err(_) => Err(UNREADABLE),
        },
    }
}

/// 問いの件数（測れていなければ Unknown・0 件と書かない）。
pub fn count(fetched: &Fetched) -> Reading<usize> {
    match cards(fetched) {
        Ok(cards) => Reading::Known(cards.len()),
        Err(_) => Reading::Unknown,
    }
}

/// block の中身（電文の順のまま番号を 1 から付ける）。
pub fn body(fetched: &Fetched) -> Body<Vec<Card>> {
    match cards(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(cards) if cards.is_empty() => Body::Empty(EMPTY),
        Ok(cards) => Body::Filled(
            cards
                .iter()
                .enumerate()
                .map(|(i, q)| card(i + 1, q))
                .collect(),
        ),
    }
}

/// 電文の 1 本を card の中身にする。
pub fn card(number: usize, q: &QuestionCard) -> Card {
    Card {
        number,
        id: q.id.clone(),
        title: clip(&q.title),
        a1: q.a1,
        posted_at: q.posted_at,
        summary: summary(q.plain.as_deref(), q.eng.as_deref()),
        reason: q.reason.clone().unwrap_or_default(),
        recommend: q.recommend.clone().unwrap_or_default(),
        touches: q.touches.clone(),
        digest: q.digest.clone(),
    }
}

/// 概要の行（2 行・片方だけ無ければ「―」・両方無ければ「要約なし」の 1 行）。
pub fn summary(plain: Option<&str>, eng: Option<&str>) -> Vec<SumLine> {
    if plain.is_none() && eng.is_none() {
        return vec![SumLine {
            class: "ln plain muted",
            key: None,
            text: NO_SUMMARY.to_string(),
            eng: false,
        }];
    }
    vec![
        SumLine {
            class: "ln plain",
            key: Some("summary_plain"),
            text: plain.unwrap_or(MISSING).to_string(),
            eng: false,
        },
        SumLine {
            class: "ln eng",
            key: Some("summary_eng"),
            text: eng.unwrap_or(MISSING).to_string(),
            eng: true,
        },
    ]
}

/// 置かれてからの経過の字（60 分未満は分・48 時間未満は時間と分・それ以上は日・見本の durMs）。
pub fn age(now: EpochSecs, posted_at: EpochSecs) -> String {
    let minutes = now.saturating_sub(posted_at) / 60;
    if minutes < 60 {
        return format!("{minutes}m");
    }
    let hours = minutes / 60;
    if hours < 48 {
        return match minutes % 60 {
            0 => format!("{hours}h"),
            m => format!("{hours}h{m:02}"),
        };
    }
    format!("{}d", hours / 24)
}

/// 送る button を押せるか（答えの欄が空白だけのときと送っている間は押せない）。
pub fn can_send(text: &str, sending: bool) -> bool {
    !sending && !text.trim().is_empty()
}

/// 答えの欄の鍵の判定の結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// 何もしない（字の変換の途中）。
    Nothing,
    /// 送る（Ctrl か Meta と Enter）。
    Send,
    /// 送らない（字の欄のふつうの動き）。
    Hold,
}

/// 鍵の判定（変換の途中か・Ctrl か Meta が押されているか・鍵の名）。
pub fn key_action(composing: bool, ctrl_or_meta: bool, key: &str) -> KeyAction {
    if composing {
        KeyAction::Nothing
    } else if ctrl_or_meta && key == "Enter" {
        KeyAction::Send
    } else {
        KeyAction::Hold
    }
}

/// 要求の本文（見た版の要約値は card の digest の字のまま・逐語は答えの欄の字のまま）。
pub fn request_body(card: &Card, verbatim: &str) -> String {
    wire::encode(&RulingRequest {
        question: card.id.clone(),
        seen_digest: card.digest.clone(),
        verbatim: verbatim.to_string(),
    })
    .expect("字の欄だけの要求は電文の字にできる")
}

/// 送った後の card の状態。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// 200: 記録した id と時刻。
    Recorded { ruling: RulingId, at: EpochSecs },
    /// 409: 質問が更新された（一覧を読み直す）。
    Stale,
    /// ほかの 4xx・5xx・届かない: 理由の字。
    Refused(String),
}

impl Outcome {
    /// card に出す 1 行。
    pub fn line(&self) -> String {
        match self {
            Outcome::Recorded { ruling, at } => format!("{RECORDED} {ruling} · {}", clock(*at)),
            Outcome::Stale => STALE.to_string(),
            Outcome::Refused(reason) => format!("{REFUSED}: {reason}"),
        }
    }

    /// 答えの欄の字を残すか（200 の外は残す）。
    pub fn keeps_text(&self) -> bool {
        !matches!(self, Outcome::Recorded { .. })
    }

    /// 答えの欄を開いたままにするか（200 は閉じる）。
    pub fn answer_open(&self) -> bool {
        self.keeps_text()
    }

    /// 問いの一覧を読み直すか（200 と 409）。
    pub fn reloads(&self) -> bool {
        matches!(self, Outcome::Recorded { .. } | Outcome::Stale)
    }
}

/// 断りの理由の字（閉じた 6 値）。
pub fn refusal_text(reason: Refusal) -> &'static str {
    match reason {
        Refusal::EmptyVerbatim => "言葉が空",
        Refusal::UnknownQuestion => "台帳に無い質問",
        Refusal::StaleVersion => STALE,
        Refusal::A1NeedsOwnVerbatim => "A-1 の質問には個別の言葉が要る",
        Refusal::A1InBatch => "A-1 の質問は束に入れない",
        Refusal::EmptyBatch => "束が空",
    }
}

/// 応答から card の状態を決める（`reply` は状態の数と本文の字・届かなければ None）。
pub fn outcome(reply: Option<(u16, &str)>) -> Outcome {
    match reply {
        None => Outcome::Refused(NOT_REACHED.to_string()),
        Some((200, text)) => match wire::decode::<RulingResponse>(text) {
            Ok(r) => Outcome::Recorded {
                ruling: r.ruling,
                at: r.recorded_at,
            },
            Err(_) => Outcome::Refused(format!("{BAD_REPLY}（200）")),
        },
        Some((409, _)) => Outcome::Stale,
        Some((status, text)) => Outcome::Refused(match wire::decode::<RefusalResponse>(text) {
            Ok(r) => format!("{}（{status}）", refusal_text(r.reason)),
            Err(_) => format!("状態 {status}"),
        }),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

/// 問いの card の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::ev;
    use leptos::prelude::*;
    use leptos::task::spawn_local;
    use tsuzuri_contract::board::Reading;
    use tsuzuri_contract::ledger::BeadId;

    use super::{
        BLOCK, Card, KeyAction, LAYOUT, Outcome, PATH, Part, RULING_PATH, Slot, age, body,
        can_send, count, key_action, outcome, request_body,
    };
    use crate::project::{Body, body_view, section, unmeasured};
    use crate::vocab::label;

    /// 見本の IC.warn・IC.clock・IC.person・IC.code・IC.check・IC.link。
    const WARN: &str = r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><path d="M12 3l10 18H2z"/><path d="M12 10v5"/><circle cx="12" cy="18" r=".8" fill="currentColor"/></svg>"#;
    const CLOCK: &str = r#"<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/></svg>"#;
    const PERSON: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="12" cy="8" r="4"/><path d="M4 21c1-4 4-6 8-6s7 2 8 6"/></svg>"#;
    const CODE: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M8 7l-5 5 5 5M16 7l5 5-5 5M14 4l-4 16"/></svg>"#;
    const CHECK: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" aria-hidden="true"><path d="M5 12l5 5 9-10"/></svg>"#;
    const LINK: &str = r#"<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1"/><path d="M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"/></svg>"#;

    /// 1 本の card の答えの状態（一覧を読み直しても残すので、問いの id ごとに block が持つ）。
    #[derive(Clone)]
    struct Draft {
        text: ArcRwSignal<String>,
        sending: ArcRwSignal<bool>,
        outcome: ArcRwSignal<Option<Outcome>>,
    }

    type Drafts = StoredValue<Vec<(BeadId, Draft)>>;

    /// 問いの id の答えの状態（初めての id は空で作る）。
    fn draft(drafts: Drafts, id: &BeadId) -> Draft {
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

    pub fn view() -> AnyView {
        let fetched = crate::net::read(PATH);
        let drafts: Drafts = StoredValue::new(Vec::new());
        let extra = move || match fetched.with(count) {
            Reading::Known(n) => view! { <span class="chip num">{n}</span> }.into_any(),
            Reading::Unknown => ().into_any(),
        };
        let list = move || match fetched.with(body) {
            Body::Unmeasured(reason) => unmeasured(reason),
            Body::Empty(line) => body_view(Body::Empty(line)),
            Body::Filled(cards) => {
                let now = crate::net::now();
                cards
                    .into_iter()
                    .map(|c| {
                        let d = draft(drafts, &c.id);
                        card_view(c, d, now)
                    })
                    .collect_view()
                    .into_any()
            }
        };
        section(BLOCK, extra.into_any(), list.into_any())
    }

    fn card_view(card: Card, d: Draft, now: u64) -> AnyView {
        let parts = LAYOUT
            .iter()
            .map(|slot| part_view(*slot, &card, &d, now))
            .collect_view();
        view! { <article class="qcard" data-q=card.id.to_string()>{parts}</article> }.into_any()
    }

    fn part_view(slot: Slot, card: &Card, d: &Draft, now: u64) -> AnyView {
        match slot.part {
            Part::Head => {
                let a1 = card.a1.then(|| {
                    view! { <span class="warn" data-term="a1" tabindex="0" aria-label=label("a1") inner_html=WARN></span> }
                });
                view! {
                    <div class=slot.class>
                        <span class="nb">{card.number}</span>
                        <span class="t"><span data-t="">{card.title.clone()}</span></span>
                        {a1}
                        <span class="chip num"><span inner_html=CLOCK></span><span>{age(now, card.posted_at)}</span></span>
                    </div>
                }
                .into_any()
            }
            Part::Summary => {
                let lines = card
                    .summary
                    .iter()
                    .map(|l| {
                        let icon = if l.eng { CODE } else { PERSON };
                        view! {
                            <div class=l.class data-term=l.key>
                                <span inner_html=icon></span>
                                <span data-t="">{l.text.clone()}</span>
                            </div>
                        }
                    })
                    .collect_view();
                view! { <div class=slot.class>{lines}</div> }.into_any()
            }
            Part::Reason => {
                let key = slot.key.unwrap_or_default();
                view! {
                    <div class=slot.class>
                        <b data-term=key tabindex="0">{label(key)}</b>
                        <span data-t="">{card.reason.clone()}</span>
                    </div>
                }
                .into_any()
            }
            Part::Recommend => {
                let key = slot.key.unwrap_or_default();
                view! {
                    <div class=slot.class data-term=key>
                        <span inner_html=CHECK></span>
                        <div><b>{label(key)}</b>" "<span data-t="">{card.recommend.clone()}</span></div>
                    </div>
                }
                .into_any()
            }
            Part::Answer => answer_view(slot, card.clone(), d.clone()),
            Part::Around => {
                let key = slot.key.unwrap_or_default();
                let ids = card
                    .touches
                    .iter()
                    .map(|t| view! { <li><span class="nid">{t.clone()}</span></li> })
                    .collect_view();
                view! {
                    <details class=slot.class open=slot.open.unwrap_or(false)>
                        <summary>
                            <span data-term=key>{label(key)}</span>
                            <span class="chip num" data-term="touches"><span inner_html=LINK></span>{label("touches")}" "{card.touches.len()}</span>
                        </summary>
                        <div class="nb-body"><ul class="items">{ids}</ul></div>
                    </details>
                }
                .into_any()
            }
        }
    }

    /// 答えの欄（字の欄と送る button）と、送った後の 1 行。200 の後は欄を閉じる。
    fn answer_view(slot: Slot, card: Card, d: Draft) -> AnyView {
        let key = slot.key.unwrap_or_default();
        let open = {
            let outcome = d.outcome.clone();
            move || outcome.with(|o| o.as_ref().is_none_or(Outcome::answer_open))
        };
        let form = {
            let d = d.clone();
            move || {
                open().then(|| {
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
                    let keydown = {
                        let (card, d) = (card.clone(), d.clone());
                        move |ev: ev::KeyboardEvent| {
                            let composing = ev.is_composing() || ev.key_code() == 229;
                            let action = key_action(composing, ev.ctrl_key() || ev.meta_key(), &ev.key());
                            if action == KeyAction::Send {
                                ev.prevent_default();
                                submit(&card, &d);
                            }
                        }
                    };
                    let click = {
                        let (card, d) = (card.clone(), d.clone());
                        move |_: ev::MouseEvent| submit(&card, &d)
                    };
                    view! {
                        <div class=slot.class>
                            <textarea rows="2" aria-label=label(key) placeholder=label(key) prop:value=value on:input=input on:keydown=keydown></textarea>
                            <button type="button" class="btn primary send" disabled=disabled on:click=click>{label("ruling")}" ›"</button>
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

    /// 答えを送る（押せないときは何もしない）。応答で card の状態を決め、200 と 409 は一覧を読み直す。
    fn submit(card: &Card, d: &Draft) {
        let text = d.text.get_untracked();
        if !can_send(&text, d.sending.get_untracked()) {
            return;
        }
        d.sending.set(true);
        let body = request_body(card, &text);
        let d = d.clone();
        spawn_local(async move {
            let reply = crate::net::post(RULING_PATH, body).await;
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
