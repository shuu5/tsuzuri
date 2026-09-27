//! 面: 画面を持つ側。便 g-min で最小の project board（問いの一覧・台帳の一覧・最終更新）を置く。
//! 並べ方と状態の印と測れていないの判定（view）は host でも組み立てて試す。
//! DOM（board）と通信（net）は wasm の target のときだけ組み立てる（trunk が xtask の surface-build から呼ばれる）。

pub mod view;

#[cfg(target_arch = "wasm32")]
pub mod board;
#[cfg(target_arch = "wasm32")]
pub mod net;
