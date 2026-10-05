//! session の表の orchestrator の行の停止の切り替え（要件 FR12）: 見本は ui.js の tkhbHTML・openHB・hbCmd。
//! button の字は今の heartbeat の逆へ向ける語の鍵（on なら hb_to_off・off なら hb_to_on）。押すと行の下に確かめの段を開き
//! （dialog の要素は使わない）、撃つと契約の型の HeartbeatRequest を HEARTBEAT_PATH へ送る。
//! heartbeat は電文の projects の同じ名の行の席の card から引き、読めない行・席の無い行・pipeline の行は button を持たない（要件 NFR2）。
//! button の字・段の字・要求の本文・応答の出し方・送る間の状態の移り方は純粋な関数にして host で試し、
//! DOM と送りは wasm の target のときだけ組み立てる。
//! project board の席の block も同じ button と確かめの段を使う:
//! 切り替えは席の card から `seat_toggle` で引き（席の名が在り heartbeat が読めるときだけ）、段は `below_to` に
//! 送り先 `Dest::Seat` を渡して組む。送りの本文は向きだけの SeatHeartbeatRequest で、SEAT_HEARTBEAT_PATH へ送る
//! （server が自分の --repo の project の席を引く）。account board の行は `below`（送り先 `Dest::Account`）のまま。

use tsuzuri_contract::account::{AccountDoc, Heartbeat, HeartbeatRequest, SessionLine};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::SeatCard;
use tsuzuri_contract::seathb::SeatHeartbeatRequest;
use tsuzuri_contract::surface::SeatRole;
use tsuzuri_contract::wire;

/// 停止の切り替えの口の path（契約の型の crate の定数・面の code に字を直に書かない）。
pub use tsuzuri_contract::account::HEARTBEAT_PATH;

/// project board の停止の切り替えの口の path（契約の型の crate の定数・行 g-seat-hb）。
pub use tsuzuri_contract::seathb::PATH as SEAT_HEARTBEAT_PATH;

/// button の class（見本の `.btn.hbbtn`）。
pub const BUTTON: &str = "btn hbbtn";

/// 撃つ字の class（見本の `pre.cmd`）。
pub const CMD: &str = "cmd";

/// 段の語の鍵（前置きと、撃つと何が変わるか）。
pub const LEAD_KEY: &str = "hb_dlg_lead";
pub const SCOPE_KEY: &str = "hb_dlg_scope";

/// 段の 2 つの button の字（語の辞書に鍵が無い）。
pub const CANCEL: &str = "やめる";
pub const FIRE: &str = "撃つ";

/// 応答の字。
pub const STOPPED: &str = "止めた";
pub const RESUMED: &str = "戻した";
pub const NO_PROJECT: &str = "project が群の宣言に無い";
pub const NO_SEAT: &str = "orchestrator の席が見つからない";
pub const BAD_BODY: &str = "要求の本文が読めない";
pub const VESSEL_FAILED: &str = "器の CLI が失敗した（5 秒で返らないか rc が 0 でない）";
pub const REFUSED: &str = "口が断った";
pub const NOT_REACHED: &str = "口に届かない";

/// 向きの字（撃つ字と見本の `hb.v`）。
pub fn word(h: Heartbeat) -> &'static str {
    match h {
        Heartbeat::On => "on",
        Heartbeat::Off => "off",
    }
}

/// 今の向きの逆（撃つ向き）。
pub fn opposite(h: Heartbeat) -> Heartbeat {
    match h {
        Heartbeat::On => Heartbeat::Off,
        Heartbeat::Off => Heartbeat::On,
    }
}

/// button の語の鍵（今が on なら止める・off なら戻す）。
pub fn button_key(now: Heartbeat) -> &'static str {
    match now {
        Heartbeat::On => "hb_to_off",
        Heartbeat::Off => "hb_to_on",
    }
}

/// 撃つ字（state dir は出さない）。
pub fn command(to: Heartbeat, target: &str) -> String {
    format!("scribe2 seat heartbeat {} --target {target}", word(to))
}

/// 要求の本文（project と向きだけ・電文の字にできなければ空の本文で、server が断る）。
pub fn request_body(project: &str, to: Heartbeat) -> String {
    wire::encode(&HeartbeatRequest {
        project: project.to_string(),
        to,
    })
    .unwrap_or_default()
}

/// project の今の heartbeat（電文の projects の同じ名の行の席の card・行か card か heartbeat が読めなければ測れていない）。
pub fn heartbeat_of(doc: &AccountDoc, project: &str) -> Reading<Heartbeat> {
    match doc
        .projects
        .iter()
        .find(|p| p.name == project)
        .map(|p| &p.seat)
    {
        Some(Reading::Known(card)) => match card.heartbeat {
            Reading::Known(true) => Reading::Known(Heartbeat::On),
            Reading::Known(false) => Reading::Known(Heartbeat::Off),
            Reading::Unknown => Reading::Unknown,
        },
        _ => Reading::Unknown,
    }
}

/// 1 行の切り替え（project・席の名・今の向き・撃つ向き）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toggle {
    pub project: String,
    pub target: String,
    pub now: Heartbeat,
    pub to: Heartbeat,
}

impl Toggle {
    /// button の語の鍵。
    pub fn button_key(&self) -> &'static str {
        button_key(self.now)
    }

    /// 撃つ字。
    pub fn command(&self) -> String {
        command(self.to, &self.target)
    }

    /// 要求の本文。
    pub fn request_body(&self) -> String {
        request_body(&self.project, self.to)
    }
}

/// 行の切り替え（orchestrator の行で席の名が在り heartbeat が読めるときだけ）。
pub fn toggle(doc: &AccountDoc, line: &SessionLine) -> Option<Toggle> {
    if line.role != SeatRole::Orchestrator || line.name.is_empty() {
        return None;
    }
    let Reading::Known(now) = heartbeat_of(doc, &line.project) else {
        return None;
    };
    Some(Toggle {
        project: line.project.clone(),
        target: line.name.clone(),
        now,
        to: opposite(now),
    })
}

/// 席の card の切り替え（席の名が在り heartbeat が読めるときだけ・行 g-seat-hb）。
/// project は空の字（席の card は project の名を持たず、口は server が自分の project を引く・States の鍵にだけ使う）。
pub fn seat_toggle(card: &SeatCard) -> Option<Toggle> {
    if card.target.is_empty() {
        return None;
    }
    let now = match card.heartbeat {
        Reading::Known(true) => Heartbeat::On,
        Reading::Known(false) => Heartbeat::Off,
        Reading::Unknown => return None,
    };
    Some(Toggle {
        project: String::new(),
        target: card.target.clone(),
        now,
        to: opposite(now),
    })
}

/// 送り先の口（account board の口か、project board の席の口か）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dest {
    Account,
    Seat,
}

impl Dest {
    /// 送りの本文（Account は project と向き・Seat は向きだけ・電文の字にできなければ空の本文）。
    pub fn body(self, t: &Toggle) -> String {
        match self {
            Dest::Account => t.request_body(),
            Dest::Seat => wire::encode(&SeatHeartbeatRequest { to: t.to }).unwrap_or_default(),
        }
    }
}

/// 確かめの段の中身（語の鍵 2 つ・撃つ字・2 つの button の字）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Panel {
    pub lead_key: &'static str,
    pub scope_key: &'static str,
    pub command: String,
    pub cancel: &'static str,
    pub fire: &'static str,
}

pub fn panel(t: &Toggle) -> Panel {
    Panel {
        lead_key: LEAD_KEY,
        scope_key: SCOPE_KEY,
        command: t.command(),
        cancel: CANCEL,
        fire: FIRE,
    }
}

/// 送った後の応答。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// 200: 撃った向き。
    Done(Heartbeat),
    /// 決まった断り（404 の no-project と no-seat・400 の bad-body・502 の vessel-failed）の字。
    Known(&'static str),
    /// ほかの状態の数。
    Refused(u16),
    /// 口に届かない。
    NotReached,
}

impl Outcome {
    /// 行の下に出す 1 行。
    pub fn line(&self) -> String {
        match self {
            Outcome::Done(Heartbeat::Off) => STOPPED.to_string(),
            Outcome::Done(Heartbeat::On) => RESUMED.to_string(),
            Outcome::Known(text) => (*text).to_string(),
            Outcome::Refused(status) => format!("{REFUSED}（{status}）"),
            Outcome::NotReached => NOT_REACHED.to_string(),
        }
    }

    /// 口を全部読み直すか（200 だけ）。
    pub fn reloads(&self) -> bool {
        matches!(self, Outcome::Done(_))
    }
}

/// 応答から出し方を決める（`to` は撃った向き・`reply` は状態の数と本文の字・届かなければ None・断りの本文は字だけ）。
pub fn outcome(to: Heartbeat, reply: Option<(u16, &str)>) -> Outcome {
    let Some((status, text)) = reply else {
        return Outcome::NotReached;
    };
    match (status, text.trim()) {
        (200, _) => Outcome::Done(to),
        (404, "no-project") => Outcome::Known(NO_PROJECT),
        (404, "no-seat") => Outcome::Known(NO_SEAT),
        (400, "bad-body") => Outcome::Known(BAD_BODY),
        (502, "vessel-failed") => Outcome::Known(VESSEL_FAILED),
        _ => Outcome::Refused(status),
    }
}

/// 送る状態（送っていない・送っている・応答）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SendState {
    #[default]
    Idle,
    Sending,
    Replied(Outcome),
}

/// 行の状態（確かめの段が開いているか・送る状態）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RowState {
    pub open: bool,
    pub send: SendState,
}

impl RowState {
    /// button を押せるか（送っている間は押せない）。
    pub fn can_press(&self) -> bool {
        self.send != SendState::Sending
    }

    /// 行の下に出す応答の 1 行（応答の後だけ）。
    pub fn reply_line(&self) -> Option<String> {
        match &self.send {
            SendState::Replied(o) => Some(o.line()),
            _ => None,
        }
    }
}

/// 行の状態を動かす出来事。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// 切り替えの button を押した（段を開く）。
    Open,
    /// やめるを押した（段を閉じる）。
    Cancel,
    /// 撃つを押した（送り始める）。
    Fire,
    /// 応答が来た（段を閉じて応答を出す）。
    Reply(Outcome),
}

/// 状態の移り方（送っている間は応答のほかを受けない・段が閉じていれば撃たない・送っていなければ応答を受けない）。
pub fn step(state: &RowState, event: Event) -> RowState {
    let sending = state.send == SendState::Sending;
    match event {
        Event::Open if !sending => RowState {
            open: true,
            send: state.send.clone(),
        },
        Event::Cancel if !sending => RowState {
            open: false,
            send: state.send.clone(),
        },
        Event::Fire if !sending && state.open => RowState {
            open: true,
            send: SendState::Sending,
        },
        Event::Reply(o) if sending => RowState {
            open: false,
            send: SendState::Replied(o),
        },
        _ => state.clone(),
    }
}

/// 撃つを押したとき送り始めるか（移る前が送っていない・移った後が送っている）。
pub fn starts(before: &RowState, after: &RowState) -> bool {
    before.send != SendState::Sending && after.send == SendState::Sending
}

#[cfg(target_arch = "wasm32")]
pub use dom::{States, below, below_to, button};

/// 切り替えの DOM と送り（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use std::collections::BTreeMap;

    use leptos::prelude::*;
    use leptos::task::spawn_local;

    use super::{
        BUTTON, CMD, Dest, Event, HEARTBEAT_PATH, RowState, SEAT_HEARTBEAT_PATH, Toggle, outcome,
        panel, starts, step,
    };
    use crate::vocab::label;

    /// 行ごとの状態（project の名で引く・表を描き直しても残る）。
    pub type States = RwSignal<BTreeMap<String, RowState>>;

    fn state(states: States, project: &str) -> RowState {
        states.with(|m| m.get(project).cloned().unwrap_or_default())
    }

    /// 出来事で行の状態を動かし、移る前と後を返す。
    fn apply(states: States, project: &str, event: Event) -> (RowState, RowState) {
        let before = states.with_untracked(|m| m.get(project).cloned().unwrap_or_default());
        let after = step(&before, event);
        states.update(|m| {
            m.insert(project.to_string(), after.clone());
        });
        (before, after)
    }

    /// 切り替えの button（送っている間は押せない）。
    pub fn button(t: Toggle, states: States) -> AnyView {
        let project = t.project.clone();
        let disabled = {
            let project = project.clone();
            move || !state(states, &project).can_press()
        };
        let open = move |_| {
            apply(states, &project, Event::Open);
        };
        view! {
            <button type="button" class=BUTTON data-hb=t.target.clone() disabled=disabled on:click=open>
                {label(t.button_key())}
            </button>
        }
        .into_any()
    }

    /// 行の下の段（開いていれば確かめの段・応答の後は応答の 1 行）。
    pub fn below(t: Toggle, states: States) -> AnyView {
        below_to(Dest::Account, t, states)
    }

    /// 送り先を選んだ行の下の段（project board の席の block は `Dest::Seat`）。
    pub fn below_to(dest: Dest, t: Toggle, states: States) -> AnyView {
        let project = t.project.clone();
        let content = move || {
            let s = state(states, &project);
            let dlg = s.open.then(|| confirm(dest, t.clone(), states));
            let reply = s
                .reply_line()
                .map(|l| view! { <div class="small" role="status">{l}</div> });
            view! { {dlg}{reply} }
        };
        content.into_any()
    }

    fn confirm(dest: Dest, t: Toggle, states: States) -> AnyView {
        let p = panel(&t);
        let project = t.project.clone();
        let disabled = {
            let project = project.clone();
            move || !state(states, &project).can_press()
        };
        let cancel = {
            let project = project.clone();
            move |_| {
                apply(states, &project, Event::Cancel);
            }
        };
        let fire = move |_| submit(dest, t.clone(), states);
        view! {
            <div class="stack" role="group">
                <p class="small">{label(p.lead_key)}</p>
                <pre class=CMD>{p.command}</pre>
                <p class="small muted">{label(p.scope_key)}</p>
                <div class="row">
                    <button type="button" class="btn" disabled=disabled.clone() on:click=cancel>{p.cancel}</button>
                    <button type="button" class="btn primary" disabled=disabled on:click=fire>{p.fire}</button>
                </div>
            </div>
        }
        .into_any()
    }

    /// 撃つ（送っている間と段が閉じているときは何もしない）。応答を出し、200 なら口を全部読み直す。
    fn submit(dest: Dest, t: Toggle, states: States) {
        let (before, after) = apply(states, &t.project, Event::Fire);
        if !starts(&before, &after) {
            return;
        }
        let body = dest.body(&t);
        spawn_local(async move {
            let reply = match dest {
                Dest::Account => crate::net::post(HEARTBEAT_PATH, body).await,
                Dest::Seat => crate::net::post(SEAT_HEARTBEAT_PATH, body).await,
            };
            let out = outcome(t.to, reply.as_ref().map(|(st, s)| (*st, s.as_str())));
            let reload = out.reloads();
            apply(states, &t.project, Event::Reply(out));
            if reload {
                crate::net::reload_all();
            }
        });
    }
}
