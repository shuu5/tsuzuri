//! block「全体への指示」（問いの頁の右の列・便 g-batch）: 方針の逐語を、つねに範囲 全体（字 all）で口 /api/policy へ送る。
//! 見本は docs/design/mock3/ask.html の side の id policy（範囲の切り替えは持ち主の指示で外した・1 つの問いへの答えは問いの card の欄で書く）。
//! 問いの一覧は読まないので、字の欄と送る button は 1 度だけ組み、書きかけの欄の focus と caret を保つ。
//! 送る button の判定・要求の本文・応答の出し方は純粋な関数にして host で試し、
//! DOM と通信は wasm の target のときだけ組み立てる。
//! 持ち主の字は送る要求の本文の外に書かない（URL にも、画面の外の保存の口にも残さない）。
//! server の書きの断り（根が無い・作れない・途中で落ちた・閉じる書きだけ落ちた）は、何が起きたかと送り直してよいかを持ち主の語の 1 行にする（行 g-policy-face）。

use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::surface::{PolicyRequest, PolicyResponse, RulingId};
use tsuzuri_contract::wire;

use super::ask::{self, BAD_REPLY, NOT_REACHED, RECORDED, REFUSED, STALE};
use super::batch::refused_text;
use crate::frame::Block;

pub const BLOCK: Block = Block {
    id: "policy",
    heading: "policy",
    class: "panel",
};

/// 方針を送る口（POST・契約の型の PolicyRequest・server の便 e-batch）。
pub const PATH: &str = "/api/policy";

/// この file が字を持つ口の path（ほかの module の口を読む所は数えない・行 hs-derived）。
pub const PATHS: &[&str] = &[PATH];

/// この file の畳める段の開き閉じの鍵の形（無い・行 hs-derived）。
pub const FOLDS: &[&str] = &[];

/// 全体の範囲の scope の字（この block が送る範囲はつねにこれ）。
pub const ALL_SCOPE: &str = "all";

/// 400 の scope のときの字。
pub const NOT_CURRENT: &str = "範囲が今の問いでない";

/// 503 の no-root のときの字。
pub const NO_ROOT: &str = "指示を残す台帳の根が無い（何も書いていない）";

/// 502 の ledger-create のときの字。
pub const NOT_CREATED: &str = "指示を台帳に書けたかが分からない（書きが落ちた）";

/// 502 の ledger-append と 500 の policy-id-shape のときの字（後に残った質問の id を付ける）。
pub const HALF_WRITTEN: &str = "指示は記録できていない（送り直してよい）・途中で残った質問";

/// 502 の ledger-close のときの字（前に記録した指示の id を付ける）。
pub const NOT_CLOSED: &str = "質問を閉じる書きだけ落ちた（送り直さない）";

/// 送る button を押せるか（逐語が空白だけのときと送っている間は押せない）。
pub fn can_send(text: &str, sending: bool) -> bool {
    ask::can_send(text, sending)
}

/// 要求の本文（scope は ALL_SCOPE の字 all・逐語は欄の字のまま・電文の字にできなければ空の本文で、server が断る）。
pub fn request_body(verbatim: &str) -> String {
    wire::encode(&PolicyRequest {
        scope: ALL_SCOPE.to_string(),
        verbatim: verbatim.to_string(),
    })
    .unwrap_or_default()
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
    /// 503 の no-root: 根が無く何も書いていない。
    NoRoot,
    /// 502 の ledger-create: 方針の問いを作れたかが分からない。
    NotCreated,
    /// 502 の ledger-append・500 の policy-id-shape: 方針は記録できておらず、作った問いが open で残る。
    HalfWritten { question: BeadId },
    /// 502 の ledger-close: 方針は記録し、問いを閉じる書きだけ落ちた。
    NotClosed { policy: RulingId },
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
            Outcome::NoRoot => NO_ROOT.to_string(),
            Outcome::NotCreated => NOT_CREATED.to_string(),
            Outcome::HalfWritten { question } => format!("{HALF_WRITTEN} {question}"),
            Outcome::NotClosed { policy } => format!("{RECORDED} {policy}・{NOT_CLOSED}"),
            Outcome::Refused(reason) => format!("{REFUSED}: {reason}"),
        }
    }

    /// 欄の字を残すか（方針を記録した 200 と ledger-close の外は残す・記録した方針は送り直させない）。
    pub fn keeps_text(&self) -> bool {
        !matches!(self, Outcome::Recorded { .. } | Outcome::NotClosed { .. })
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
        Some((status, text)) => written_part(status, text)
            .unwrap_or_else(|| Outcome::Refused(refused_text(status, text))),
    }
}

/// server の書きの断りの本文（語と、語によって id）を読む。読めない本文は None（読んだ振りをしない）。
fn written_part(status: u16, text: &str) -> Option<Outcome> {
    let text = text.trim();
    let (word, id) = text.split_once(char::is_whitespace).unwrap_or((text, ""));
    match (status, word) {
        (503, "no-root") if id.is_empty() => Some(Outcome::NoRoot),
        (502, "ledger-create") if id.is_empty() => Some(Outcome::NotCreated),
        (502, "ledger-append") | (500, "policy-id-shape") => BeadId::new(id)
            .ok()
            .map(|question| Outcome::HalfWritten { question }),
        (502, "ledger-close") => RulingId::new(id)
            .ok()
            .map(|policy| Outcome::NotClosed { policy }),
        _ => None,
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

    use super::{BLOCK, Outcome, PATH, can_send, outcome, request_body};
    use crate::project::ask::{KeyAction, key_action};
    use crate::project::section;
    use crate::vocab::label;

    /// block の状態（欄の字・送っている間か・送った後の 1 行）。
    #[derive(Clone, Copy)]
    struct State {
        text: RwSignal<String>,
        sending: RwSignal<bool>,
        outcome: RwSignal<Option<Outcome>>,
    }

    pub fn view() -> AnyView {
        let s = State {
            text: RwSignal::new(String::new()),
            sending: RwSignal::new(false),
            outcome: RwSignal::new(None),
        };
        let note = move || {
            s.outcome
                .get()
                .map(|o| view! { <div class="small" role="status">{o.line()}</div> })
        };
        section(BLOCK, ().into_any(), view! { {form(s)}{note} }.into_any())
    }

    fn form(s: State) -> AnyView {
        let disabled = move || !can_send(&s.text.get(), s.sending.get());
        let keydown = move |ev: ev::KeyboardEvent| {
            let composing = ev.is_composing() || ev.key_code() == 229;
            if key_action(composing, ev.ctrl_key() || ev.meta_key(), &ev.key()) == KeyAction::Send {
                ev.prevent_default();
                submit(s);
            }
        };
        view! {
            <textarea rows="2" aria-label=label("policy") prop:value=move || s.text.get() on:input=move |ev: ev::Event| s.text.set(event_target_value(&ev)) on:keydown=keydown></textarea>
            <button type="button" class="btn" disabled=disabled on:click=move |_| submit(s)>{label("policy")}" ›"</button>
        }
        .into_any()
    }

    /// 指示を送る（押せないときは何もしない）。応答で block の状態を決め、読み直す応答は口を全部読み直す。
    fn submit(s: State) {
        let text = s.text.get_untracked();
        if !can_send(&text, s.sending.get_untracked()) {
            return;
        }
        s.sending.set(true);
        let body = request_body(&text);
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
