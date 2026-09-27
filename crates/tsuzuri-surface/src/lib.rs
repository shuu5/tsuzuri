//! 面: 画面を持つ側。project board の頁の枠（frame）と block（project の下に 1 つずつ）と部品（widgets）を置く。
//! 見た目と頁の枠は見本（docs/design/mock3）に揃える（便 g-frame）: stylesheet は見本の ui.css の写し・語は vocab.json の写し。
//! 並べ方と状態の印と測れていないの判定（view）・枠の値（frame）・block の中身の関数は host でも組み立てて試す。
//! 地図の頁の面の部品（帯・圧縮・一覧・表・グラフと近傍）は mapview に置く（便 g-map・block の module は project の下の map）。
//! DOM（board と各 module の view）と通信（net）は wasm の target のときだけ組み立てる（trunk が xtask の surface-build から呼ばれる）。

pub mod frame;
pub mod mapview;
pub mod project;
pub mod view;
pub mod vocab;
pub mod widgets;

#[cfg(target_arch = "wasm32")]
pub mod board;
#[cfg(target_arch = "wasm32")]
pub mod net;
