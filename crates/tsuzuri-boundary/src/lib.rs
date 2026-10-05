//! 境界: 面の server と席の hook と問いの起票の門。
//! 最小の server（台帳の読み・変化の SSE・面の file の配布）と tz の入口を置く。
//! account board の読み（`acct`）を置く（口の登録は `server`）。
//! 停止の切り替えの受付（`accthb`）を置く（口の登録は `server`）。
//! tz の口の graph（`cli::graph`・導出グラフと不変条件を端末で撃つ）を置く。
//! tz の口の folio の 11 subcommand と索引の形の graph（`cli::folio`・folio の lib の入口を撃つ）を置く。
//! 席の停止の hook（`hook::stop`・tz hook stop）を置く。
//! 表示面の module（`stage`・端末の一覧の読み `stage::terminal`）を置く。
//! 問いの起票の門（`hook::question_gate`・tz hook question-gate）を置く。
//! 受入 12 条の数え（`audit`・測りの事実から条ごとの違反の数と report の行）を置く。
//! 表示先の設定と窓を開く頼みの受付（`stagecall`・tz の口を撃つだけ）を置く。
//! 出力の手（`out`・標準出力と標準エラーへ書く 2 つの関数）を置く。
//! 相談の窓の命令（`consult`・tz consult の口の振り分けと共通の手・判断の記録 ADR-29）を置く。

pub mod acct;
pub mod accthb;
pub mod audit;
pub mod cli;
pub mod consult;
pub mod hook;
pub mod out;
pub mod server;
pub mod stage;
pub mod stagecall;
