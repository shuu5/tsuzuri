//! header の席の pill（行 g-seatpill・見本の ui.js の topHTML の `.seatpill` と seatPillHTML と seatCard）:
//! 状態の記号と語 orchestrator と、応答なしなら応答なしになった時からの経過を出し、指を置くと席の card を出す。
//! 中身は口 /api/seat（契約の型の SeatCard・読み手は project の seat）から読み、電文を決めずに写す。
//! 電文に無い見本の値（管理の tick の最後の時刻からの経過）は出さない。
//! 限度の再開の時刻は電文の欄 reopens（器の判定の写し）を、限度のときだけ pill に時刻を・card に ↻ の行を出す。
//! pill と card の値は host でも組んで試し、DOM（`view`）は wasm の target のときだけ組む。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::{Reopens, SeatState};

use crate::frame::SEAT;
use crate::project::ask::age;
use crate::project::seat::{self, LIMIT_LINE, band, hmd, state_value};
use crate::project::{UNKNOWN, state_key};
use crate::view::Fetched;
use crate::vocab::label;
use crate::widgets::hover::Card;

/// 経過の字の style（見本の seatPillHTML の応答なしの eta の色）。
pub const ETA_STYLE: &str = "color:var(--st-silent)";

/// 電文に無い値の字。
const ASK: &str = "?";

/// 再開の行の頭の記号（見本の seatCard の ↻）。
pub const RESUME: &str = "↻";

/// 限度に当たっているが再開の時刻が無い（器の reopens=unknown）ときの字。
pub const RESUME_UNKNOWN: &str = "時刻が分からない";

/// pill の値（状態の値と、応答なしの経過か限度の再開の時刻の字）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pill {
    /// 状態の値（run・wait・limit・silent・unknown）。
    pub state: &'static str,
    /// 応答なしになった時からの経過（応答なしで since が在るとき）か、限度の再開の時刻（限度で時刻が在るとき）。
    pub eta: Option<String>,
}

/// 口の本文と今の時刻から pill を組む（読めなければ測れていない・経過と時刻は今と電文の at の大きい方から数える）。
pub fn pill(fetched: &Fetched, now: EpochSecs) -> Pill {
    match seat::card(fetched) {
        Err(_) => Pill {
            state: UNKNOWN,
            eta: None,
        },
        Ok(c) => Pill {
            state: state_value(c.state),
            eta: match (c.state, c.since, c.reopens) {
                (SeatState::Silent, Some(since), _) => Some(age(now.max(c.at), since)),
                (SeatState::Limit, _, Reopens::At(t)) => Some(hmd(t, now.max(c.at))),
                _ => None,
            },
        },
    }
}

/// card の ↻ の行（当たっていなければ無し・測れていないと時刻が分からないは語で出す）。
fn resume_line(reopens: Reopens, at: EpochSecs) -> Option<String> {
    let text = match reopens {
        Reopens::At(t) => hmd(t, at),
        Reopens::Clear => return None,
        Reopens::Unmeasured => label(state_key(UNKNOWN)),
        Reopens::Unknown => RESUME_UNKNOWN.to_string(),
    };
    Some(format!("{RESUME} {text}"))
}

/// 席の card（見本の seatCard・account board の orch_card と同じ字の並び・読めなければ理由の字・描く今の card で組む）。
pub fn card(fetched: &Fetched, now: EpochSecs) -> Card {
    let c = match seat::card(fetched) {
        Err(reason) => {
            return Card {
                title: label(SEAT.key),
                kind: label(state_key(UNKNOWN)),
                value: reason.to_string(),
                src: seat::PATH.to_string(),
                more: Vec::new(),
            };
        }
        Ok(c) => seat::drawn(c, now),
    };
    let state = label(state_key(state_value(c.state)));
    let kind = match c.since {
        Some(s) => format!("{state} · ◷ {} から", hmd(s, c.at)),
        None => state,
    };
    let tick = match c.tick_healthy {
        Reading::Known(true) => "healthy",
        Reading::Known(false) => "stale",
        Reading::Unknown => ASK,
    };
    let hb = match c.heartbeat {
        Reading::Known(true) => "on",
        Reading::Known(false) => "off",
        Reading::Unknown => ASK,
    };
    let mut more = vec![format!("model {}", c.model.as_deref().unwrap_or(ASK))];
    if let Some(b) = band(&c) {
        for line in b.l1 {
            let limit = line == LIMIT_LINE;
            more.push(line);
            if limit {
                more.extend(resume_line(c.reopens, c.at));
            }
        }
        more.extend(b.l2);
    }
    Card {
        title: format!("{} · {}", label(SEAT.key), c.target),
        kind,
        value: format!(
            "口座 {} · tick {tick} · hb {hb}",
            c.account.as_deref().unwrap_or(ASK)
        ),
        src: format!("seat/{}/state.jsonl ほか", c.target),
        more,
    }
}

#[cfg(target_arch = "wasm32")]
pub use dom::view;

#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::ev;
    use leptos::prelude::*;
    use tsuzuri_contract::seat::SeatState;

    use super::{ETA_STYLE, card, pill};
    use crate::frame::SEAT;
    use crate::project::seat::state_value;
    use crate::project::{seat, state_icon};
    use crate::vocab::label;
    use crate::widgets::hover::delegate;

    /// 席の pill（状態の記号・語・応答なしの経過か限度の再開の時刻・指を置くと card を出す）。
    pub fn view() -> AnyView {
        let fetched = crate::net::read(seat::PATH);
        let tick = crate::net::ticker();
        // 毎秒に電文を読み直す（電文は数 KB）。
        let shown = Memo::new(move |_| fetched.with(|f| pill(f, tick.get())));
        let state = Memo::new(move |_| shown.with(|p| p.state));
        let hc = delegate();
        let over = move |ev: ev::MouseEvent| hc.show(&ev, fetched.with_untracked(|f| card(f, crate::net::now())));
        let out = move |ev: ev::MouseEvent| hc.leave(&ev);
        let icon = move || state_icon(state.get());
        // 応答なしの経過だけに応答なしの色を付け、限度の時刻は stylesheet の色のまま。
        let silent = state_value(SeatState::Silent);
        let eta = move || {
            shown.with(|p| p.eta.clone()).map(|e| {
                if state.get() == silent {
                    view! { <span class="eta num" style=ETA_STYLE>{e}</span> }.into_any()
                } else {
                    view! { <span class="eta num">{e}</span> }.into_any()
                }
            })
        };
        view! {
            <span class=SEAT.class tabindex="0" data-seat="1" on:mouseenter=over on:mouseleave=out>
                {icon}
                <span class="who">{label(SEAT.key)}</span>
                {eta}
            </span>
        }
        .into_any()
    }
}
