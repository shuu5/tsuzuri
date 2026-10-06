//! 端末の口（tz の入口が surface serve のほかに撃つ subcommand）。
//! 読む部品は server の module と同じものを使い、何も書かない。
//! folio の 11 の口と索引の形の tz graph は `folio`（folio の lib の入口を撃つ）。
//! 設計ノートの行の近い仕様の tz context は `context`（folio の lib の入口 context を撃つ）。
//! 行の触る定義と file の定義の tz code は `code`（git と構文で探す道具を撃ち、中核の code の層で組む）。

pub mod code;
pub mod context;
pub mod folio;
pub mod graph;
