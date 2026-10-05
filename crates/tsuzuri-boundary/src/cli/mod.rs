//! 端末の口（tz の入口が surface serve のほかに撃つ subcommand・行 k-graph）。
//! 読む部品は server の module と同じものを使い、何も書かない。
//! folio の 11 の口と索引の形の tz graph は `folio`（folio の lib の入口を撃つ・行 k-tz-entry）。
//! 設計ノートの行の近い仕様の tz context は `context`（folio の lib の入口 context を撃つ・行 k-ctx-cli）。

pub mod context;
pub mod folio;
pub mod graph;
