//! block「全体への指示」（問いの頁の右の列・便 g-batch）: 範囲（この論点・全体）を選び、方針の逐語を口 /api/policy へ送る。
//! 見本は docs/design/mock3/ask.html の side の id policy。問いの一覧は ask の module の口から読む。
//! 範囲の初めの値・問いの選びの項・scope の字・送る button の判定・要求の本文・応答の出し方は純粋な関数にして host で試し、
//! DOM と通信は wasm の target のときだけ組み立てる。
//! 持ち主の字は送る要求の本文の外に書かない（URL にも、画面の外の保存の口にも残さない）。

use tsuzuri_contract::graph::title36;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::question::QuestionCard;
use tsuzuri_contract::surface::{PolicyRequest, PolicyResponse, RulingId};
use tsuzuri_contract::wire;

use super::ask::{self, BAD_REPLY, NOT_REACHED, RECORDED, REFUSED, STALE};
use super::batch::refused_text;
use crate::frame::Block;
use crate::view::Fetched;

pub const BLOCK: Block = Block {
    id: "policy",
    heading: "policy",
    class: "panel",
};

/// 方針を送る口（POST・契約の型の PolicyRequest・server の便 e-batch）。
pub const PATH: &str = "/api/policy";

/// 範囲の切り替えの aria-label の字。
pub const SCOPE: &str = "範囲";

/// 範囲の button の字（この論点）。
pub const TOPIC: &str = "この論点";

/// 範囲の button の字（全体）。
pub const ALL: &str = "全体";

/// 問いの選びの aria-label の字。
pub const TOPIC_PICK: &str = "論点";

/// 全体の範囲の scope の字。
pub const ALL_SCOPE: &str = "all";

/// 503 の no-policy-memo のときの字。
pub const NO_MEMO: &str = "方針の memo が台帳に無い";

/// 400 の scope のときの字。
pub const NOT_CURRENT: &str = "範囲が今の問いでない";

/// 範囲。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// この論点（選んだ問いの id）。
    Topic,
    /// 全体。
    All,
}

impl Scope {
    /// 切り替えの button の順（この論点・全体）。
    pub const ALL: [Scope; 2] = [Scope::Topic, Scope::All];

    /// button の字。
    pub fn text(self) -> &'static str {
        match self {
            Scope::Topic => TOPIC,
            Scope::All => ALL,
        }
    }
}

/// 問いの選びの 1 項（問いの id・一覧の順の番号と 36 字に切った題）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topic {
    pub id: BeadId,
    pub text: String,
}

/// 問いの選びの項（問いの一覧の順）。
pub fn topics(cards: &[QuestionCard]) -> Vec<Topic> {
    cards
        .iter()
        .enumerate()
        .map(|(i, q)| Topic {
            id: q.id.clone(),
            text: format!("{} {}", i + 1, title36(&q.title)),
        })
        .collect()
}

/// block の中身（口が読めない・まだ読んでいない・電文が読めないは測れていないの理由・0 本は項の無い一覧）。
pub fn body(fetched: &Fetched) -> Result<Vec<Topic>, &'static str> {
    ask::cards(fetched).map(|cards| topics(&cards))
}

/// この論点を押せるか（問いが 1 本以上在るとき）。
pub fn topic_enabled(topics: usize) -> bool {
    topics > 0
}

/// 今の範囲（`pressed` は押した button・まだ押していなければ None）。
/// 問いが 1 本以上在れば初めはこの論点・0 本ならこの論点は押せず全体。
pub fn scope(topics: usize, pressed: Option<Scope>) -> Scope {
    match pressed {
        _ if !topic_enabled(topics) => Scope::All,
        Some(s) => s,
        None => Scope::Topic,
    }
}

/// 今の問いの選び（選んだ id が一覧に在ればそれ・無ければ一覧の 1 本目）。
pub fn pick<'a>(topics: &'a [Topic], chosen: Option<&BeadId>) -> Option<&'a Topic> {
    chosen
        .and_then(|id| topics.iter().find(|t| t.id == *id))
        .or_else(|| topics.first())
}

/// 要求の scope の字（全体は all・この論点は選んだ問いの id・問いが無ければ None）。
pub fn scope_text(scope: Scope, topic: Option<&Topic>) -> Option<String> {
    match scope {
        Scope::All => Some(ALL_SCOPE.to_string()),
        Scope::Topic => topic.map(|t| t.id.to_string()),
    }
}

/// 送る button を押せるか（逐語が空白だけのときと送っている間は押せない）。
pub fn can_send(text: &str, sending: bool) -> bool {
    ask::can_send(text, sending)
}

/// 要求の本文（scope の字・逐語は欄の字のまま）。
pub fn request_body(scope: &str, verbatim: &str) -> String {
    wire::encode(&PolicyRequest {
        scope: scope.to_string(),
        verbatim: verbatim.to_string(),
    })
    .expect("字の欄だけの要求は電文の字にできる")
}

/// 送った後の block の状態。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// 200: 方針の id。
    Recorded { policy: RulingId },
    /// 409: 質問が更新された（一覧を読み直す）。
    Stale,
    /// 400 の scope: 範囲が今の問いでない（一覧を読み直す）。
    NotCurrent,
    /// 503 の no-policy-memo: 方針の memo が台帳に無い。
    NoMemo,
    /// ほかの 4xx・5xx・届かない: 理由の字。
    Refused(String),
}

impl Outcome {
    /// block に出す 1 行。
    pub fn line(&self) -> String {
        match self {
            Outcome::Recorded { policy } => format!("{RECORDED} {policy}"),
            Outcome::Stale => STALE.to_string(),
            Outcome::NotCurrent => NOT_CURRENT.to_string(),
            Outcome::NoMemo => NO_MEMO.to_string(),
            Outcome::Refused(reason) => format!("{REFUSED}: {reason}"),
        }
    }

    /// 欄の字を残すか（200 の外は残す）。
    pub fn keeps_text(&self) -> bool {
        !matches!(self, Outcome::Recorded { .. })
    }

    /// 口を全部読み直すか（409 と 400 の scope・200 は問いを閉じないので読み直さない）。
    pub fn reloads(&self) -> bool {
        matches!(self, Outcome::Stale | Outcome::NotCurrent)
    }
}

/// 応答から block の状態を決める（`reply` は状態の数と本文の字・届かなければ None）。
pub fn outcome(reply: Option<(u16, &str)>) -> Outcome {
    match reply {
        None => Outcome::Refused(NOT_REACHED.to_string()),
        Some((200, text)) => match wire::decode::<PolicyResponse>(text) {
            Ok(r) => Outcome::Recorded { policy: r.policy },
            Err(_) => Outcome::Refused(format!("{BAD_REPLY}（200）")),
        },
        Some((409, _)) => Outcome::Stale,
        Some((400, text)) if text.trim() == "scope" => Outcome::NotCurrent,
        Some((503, text)) if text.trim() == "no-policy-memo" => Outcome::NoMemo,
        Some((status, text)) => Outcome::Refused(refused_text(status, text)),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

/// 全体への指示の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::ev;
    use leptos::prelude::*;
    use leptos::task::spawn_local;
    use tsuzuri_contract::ledger::BeadId;

    use super::{
        BLOCK, Outcome, PATH, SCOPE, Scope, TOPIC_PICK, Topic, body, can_send, outcome, pick,
        request_body, scope, scope_text, topic_enabled,
    };
    use crate::project::ask::{self, KeyAction, key_action};
    use crate::project::{section, unmeasured};
    use crate::vocab::label;

    /// block の状態（押した範囲・選んだ問い・欄の字・送っている間か・送った後の 1 行）。
    #[derive(Clone, Copy)]
    struct State {
        pressed: RwSignal<Option<Scope>>,
        chosen: RwSignal<Option<BeadId>>,
        text: RwSignal<String>,
        sending: RwSignal<bool>,
        outcome: RwSignal<Option<Outcome>>,
    }

    pub fn view() -> AnyView {
        let fetched = crate::net::read(ask::PATH);
        let s = State {
            pressed: RwSignal::new(None),
            chosen: RwSignal::new(None),
            text: RwSignal::new(String::new()),
            sending: RwSignal::new(false),
            outcome: RwSignal::new(None),
        };
        let content = move || match fetched.with(body) {
            Err(reason) => unmeasured(reason),
            Ok(topics) => form(topics, s),
        };
        let note = move || {
            s.outcome
                .get()
                .map(|o| view! { <div class="small" role="status">{o.line()}</div> })
        };
        section(BLOCK, ().into_any(), view! { {content}{note} }.into_any())
    }

    fn form(topics: Vec<Topic>, s: State) -> AnyView {
        let n = topics.len();
        let topics = StoredValue::new(topics);
        let now = move || scope(n, s.pressed.get());
        let seg = Scope::ALL
            .into_iter()
            .map(|sc| {
                let pressed = move || (now() == sc).to_string();
                let off = sc == Scope::Topic && !topic_enabled(n);
                view! { <button type="button" aria-pressed=pressed disabled=off on:click=move |_| s.pressed.set(Some(sc))>{sc.text()}</button> }
            })
            .collect_view();
        let select = move || {
            (now() == Scope::Topic).then(|| {
                let current = move || {
                    topics.with_value(|t| pick(t, s.chosen.get().as_ref()).map(|p| p.id.clone()))
                };
                let options = topics.with_value(|t| {
                    t.iter()
                        .map(|p| {
                            let id = p.id.clone();
                            let on = move || current().as_ref() == Some(&id);
                            view! { <option value=p.id.to_string() selected=on>{p.text.clone()}</option> }
                        })
                        .collect_view()
                });
                let change = move |ev: ev::Event| s.chosen.set(BeadId::new(event_target_value(&ev)).ok());
                view! { <select aria-label=TOPIC_PICK on:change=change>{options}</select> }
            })
        };
        let disabled = move || !can_send(&s.text.get(), s.sending.get());
        let keydown = move |ev: ev::KeyboardEvent| {
            let composing = ev.is_composing() || ev.key_code() == 229;
            if key_action(composing, ev.ctrl_key() || ev.meta_key(), &ev.key()) == KeyAction::Send {
                ev.prevent_default();
                submit(topics, s);
            }
        };
        view! {
            <div class="seg" role="group" aria-label=SCOPE>{seg}</div>
            {select}
            <textarea rows="2" aria-label=label("policy") prop:value=move || s.text.get() on:input=move |ev: ev::Event| s.text.set(event_target_value(&ev)) on:keydown=keydown></textarea>
            <button type="button" class="btn" disabled=disabled on:click=move |_| submit(topics, s)>{label("policy")}" ›"</button>
        }
        .into_any()
    }

    /// 指示を送る（押せないときは何もしない）。応答で block の状態を決め、読み直す応答は口を全部読み直す。
    fn submit(topics: StoredValue<Vec<Topic>>, s: State) {
        let text = s.text.get_untracked();
        if !can_send(&text, s.sending.get_untracked()) {
            return;
        }
        let target = topics.with_value(|t| {
            let sc = scope(t.len(), s.pressed.get_untracked());
            scope_text(sc, pick(t, s.chosen.get_untracked().as_ref()))
        });
        let Some(target) = target else {
            return;
        };
        s.sending.set(true);
        let body = request_body(&target, &text);
        spawn_local(async move {
            let reply = crate::net::post(PATH, body).await;
            let out = outcome(reply.as_ref().map(|(st, t)| (*st, t.as_str())));
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
