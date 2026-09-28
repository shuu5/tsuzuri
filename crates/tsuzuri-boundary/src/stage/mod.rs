//! 表示面（要件 FR16・席が端末の一覧から端末の名で表示先を操る）。
//! 傘の行 i の子の行が file を足す（行 i-1 で端末の一覧の読み `terminal` を置く）。
//! 行 i-3 で CDP の口（websocket の client の側 `ws`・小さな JSON の読み書き `json`・命令の列と session の `cdp`）を置く。

pub mod cdp;
pub mod json;
pub mod terminal;
pub mod ws;
