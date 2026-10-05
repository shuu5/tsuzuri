//! 話す窓を群の口座の移動に付いて来させる判じ（設計ノート surface-wave29b 行 cs-follow・判断の記録 ADR-55 決定 (3)）。
//! 窓の process の印の口座と席の今の口座のずれ（`drifted`）、会話の印の最後の行が手すきか（`idle`）、tmux の pane の字の
//! 入力欄が空か（`input_clear`・器の席の入力欄の門と同じ読み）を判じる。

use tsuzuri_contract::consult::{Stamp, StampEvent};

/// Claude Code の入力欄の頭の字（器の席の入力欄の門と同じ字）。
pub const PROMPT: char = '❯';

/// 印の口座 `mark` が在り、席の今の口座 `now` と違うか（どちらかが無ければ判じない＝偽）。
pub fn drifted(mark: Option<&str>, now: Option<&str>) -> bool {
    matches!((mark, now), (Some(m), Some(n)) if m != n)
}

/// 会話の印の最後の行が会話の始まりか turn の終わり（持ち主の入力の後の turn の中でない）か。印が無ければ偽。
pub fn idle(stamps: &[Stamp]) -> bool {
    stamps.last().is_some_and(|s| s.event != StampEvent::Prompt)
}

/// pane の字の、`PROMPT` を含む最後の行の `PROMPT` の右（前後の空白を除く）が空か（その行が無ければ None＝照らせない）。
pub fn input_clear(pane: &str) -> Option<bool> {
    let line = pane.lines().rfind(|l| l.contains(PROMPT))?;
    line.split_once(PROMPT)
        .map(|(_, rest)| rest.trim().is_empty())
}
