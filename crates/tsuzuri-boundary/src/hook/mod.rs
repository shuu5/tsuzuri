//! 席の hook（Claude Code が撃つ tz の口）。停止の hook（`stop`）を置く。
//! 係の終える前の門（`subagent_stop`・SubagentStop の hook）を置く。
//! 問いの起票の門（`question_gate`・PreToolUse の hook）を置く。
//! 問いの合図の送り手（`question_signal`・PostToolUse の async の hook）を置く。
//! 配達の hook（`deliver`・UserPromptSubmit の hook）を置く。
//! 考え中の席への配達の hook（`deliver_tool`・PostToolBatch の hook）を置く。
//! 停止と入力の時と道具の周の hook が見張りの居ない間に足す相談の拾い（`consult`）を置く。
//! 係の起こしの門（`agent_spawn`・PreToolUse の hook）と結びの口（`agent_bind`・PostToolUse の hook）を置く。
//! 係の測り（`agent_meter`・PostToolUse の hook）を置く。
//! 係の門（`agent_guard`・PreToolUse の hook）を置く。

pub mod agent_bind;
pub mod agent_guard;
pub mod agent_meter;
pub mod agent_spawn;
pub mod consult;
pub mod deliver;
pub mod deliver_tool;
pub mod question_gate;
pub mod question_signal;
pub mod stop;
pub mod subagent_stop;
