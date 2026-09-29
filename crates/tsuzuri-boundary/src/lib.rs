//! 境界: 面の server と席の hook と問いの起票の門（便 e・f で置く）。
//! 便 e-min で最小の server（台帳の読み・変化の SSE・面の file の配布）と tz の入口を置く。
//! 便 e-acct で account board の読み（`acct`）を置く（口の登録は便 h-wire）。
//! 便 e-acct-hb で停止の切り替えの受付（`accthb`）を置く（口の登録は便 h-wire）。
//! 行 k-graph で tz の口の graph（`cli::graph`・導出グラフと不変条件を端末で撃つ）を置く。
//! 行 f-stop で席の停止の hook（`hook::stop`・tz hook stop）を置く。
//! 行 i-1 で表示面の module（`stage`・端末の一覧の読み `stage::terminal`）を置く。
//! 行 f-gate で問いの起票の門（`hook::question_gate`・tz hook question-gate）を置く。
//! 行 j-count で受入 12 条の数え（`audit`・測りの事実から条ごとの違反の数と report の行）を置く。
//! 行 e-stage-target で表示先の設定と窓を開く頼みの受付（`stagecall`・tz の口を撃つだけ）を置く。

pub mod acct;
pub mod accthb;
pub mod audit;
pub mod cli;
pub mod hook;
pub mod server;
pub mod stage;
pub mod stagecall;
