//! 席が自分で開く問う窓の数え（規則の行 R-38・判断の記録 ADR-29 決定 (10)）。
//! 今日の数は、台帳の相談の開きの行のうち、起こし手が席・形が問う・結果が開いた・撃ち直しの印が無い行の、
//! 時刻の頭 8 字（UTC の日）が今の分の字の頭 8 字と同じ物の数。今の数は、席が問う窓として開いた窓のうち、
//! process の印が生きている窓（呼ぶ側が渡す）の数。持ち主の言葉で開く窓（起こし手が持ち主のチャットか button）と
//! 話す窓は数えず、撃ち直しは今日の数に入れず断りもしない。

use tsuzuri_contract::consult::{Form, SeatQuota, WindowId};

use super::lines::{By, Line};

/// 同時の本数の上限（規則の行 R-38）。
pub const LIVE_MAX: u32 = 1;

/// 1 日（UTC の日）の本数の上限（規則の行 R-38）。
pub const DAY_MAX: u32 = 3;

/// 上限で断る時の標準エラーの 1 行。
pub const REFUSAL: &str = "R-38 の上限（同時 1 本・1 日 3 本）で断った・決定の画面で持ち主に問う";

/// 席が問う窓として開いた行の窓と時刻と撃ち直しの印。
fn seat_ask(line: &Line) -> Option<(WindowId, &str, bool)> {
    match line {
        Line::Open {
            window,
            form: Form::Ask,
            by: By::Seat,
            opened: true,
            again,
            at,
            ..
        } => Some((*window, at.as_str(), *again)),
        _ => None,
    }
}

/// 今日の数と今の数（`now` は今の UTC の分の字・`live` は process の印が生きている窓）。
pub fn count(lines: &[Line], now: &str, live: &[WindowId]) -> SeatQuota {
    let day = now.get(..8);
    let asks: Vec<(WindowId, &str, bool)> = lines.iter().filter_map(seat_ask).collect();
    let today = asks
        .iter()
        .filter(|(_, at, again)| !again && day.is_some() && at.get(..8) == day)
        .count();
    let mut windows: Vec<WindowId> = asks.iter().map(|(w, _, _)| *w).collect();
    windows.sort();
    windows.dedup();
    let alive = windows.iter().filter(|w| live.contains(w)).count();
    SeatQuota {
        today: u32::try_from(today).unwrap_or(u32::MAX),
        live: u32::try_from(alive).unwrap_or(u32::MAX),
    }
}

/// 席が自分で問う窓を開いてよいか（撃ち直しはいつも通す）。
pub fn admit(quota: SeatQuota, again: bool) -> Result<(), &'static str> {
    if again || (quota.live < LIVE_MAX && quota.today < DAY_MAX) {
        Ok(())
    } else {
        Err(REFUSAL)
    }
}
