//! 表示面（要件 FR16・席が端末の一覧から端末の名で表示先を操る）。
//! 傘の行 i の子の行が file を足す（行 i-1 で端末の一覧の読み `terminal` を置く）。
//! 行 i-3 で CDP の口（websocket の client の側 `ws`・小さな JSON の読み書き `json`・命令の列と session の `cdp`）を置く。
//! 行 i-2 で起動の引数 `launch` と、tunnel と窓を 1 回だけ起こすこと `tunnel` を置く。
//! 行 i-4 で席の目の Chrome の pipe の運び手 `pipe`・端末に届かない時の落ちる先 `relay`・board の URL `url` を置く。
//! 行 i-5 で tz の口 `cli`（命令の旗の読み・board の頁の断り・席の中の open の断り・窓を開く錠）を置く。
//! 行 i-7 で表示先の設定 `target`（全体の既定と project ごとの上書きと初めて見せた印・追跡されない 1 つの file）を置く。

pub mod cdp;
pub mod cli;
pub mod json;
pub mod launch;
pub mod pipe;
pub mod relay;
pub mod target;
pub mod terminal;
pub mod tunnel;
pub mod url;
pub mod ws;
