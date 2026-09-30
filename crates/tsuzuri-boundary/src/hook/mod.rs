//! 席の hook（Claude Code が撃つ tz の口）。行 f-stop で停止の hook（`stop`）を置く。
//! 行 f-gate で問いの起票の門（`question_gate`・PreToolUse の hook）を置く。
//! 行 e-signal-send で問いの合図の送り手（`question_signal`・PostToolUse の async の hook）を置く。
//! 行 f-deliver で配達の hook（`deliver`・UserPromptSubmit の hook）を置く。
//! 行 f-deliver-tool で考え中の席への配達の hook（`deliver_tool`・PostToolBatch の hook）を置く。

pub mod deliver;
pub mod deliver_tool;
pub mod question_gate;
pub mod question_signal;
pub mod stop;
