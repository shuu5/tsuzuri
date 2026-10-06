//! 面: 画面を持つ側。project board の頁の枠（frame）と block（project の下に 1 つずつ）と部品（widgets）を置く。
//! 見た目と頁の枠は見本（docs/design/mock3）に揃える: stylesheet は見本の ui.css の写し・語は vocab.json の写し。
//! 並べ方と状態の印と測れていないの判定（view）・枠の値（frame）・block の中身の関数は host でも組み立てて試す。
//! 地図の面の部品（帯・圧縮・一覧・表・グラフと近傍）は mapview に置く（地図の頁と block は消した）。
//! DOM（board と各 module の view）と通信（net）は wasm の target のときだけ組み立てる（trunk が xtask の surface-build から呼ばれる）。
//! account board は account に置く: 入口は URL の query の board が account のときだけ選ぶ（同じ index.html と wasm）。
//! block に共通の部品は kit に置く（project が再公開する）。

pub mod account;
/// 質問の窓（1 問ずつの選び方と局面は host でも組む・窓の DOM は wasm の target だけ・行 g-ask-win）。
pub mod askwin;
/// 相談の窓（頼みの form と一覧の段は host でも組む・窓の DOM は wasm の target だけ・行 cs-bar）。
pub mod consultwin;
/// 口ごとの読みの印の決め方（net が使い host でも組む・行 g-reads）。
pub mod flight;
pub mod frame;
/// host の負荷と書きの印と窓の中身（字と段は host でも組む・DOM は wasm の target だけ）。
pub mod hostwin;
/// 中身の古さと読み込み不良の印（決め方は host でも組む・印の DOM は wasm の target だけ・行 g-fresh）。
pub mod fresh;
pub mod kit;
/// 台帳 open の一覧（epic ごとの組・組の中身は host でも組む・DOM は wasm の target だけ・行 g-list-groups）。
pub mod ledgerlist;
pub mod mapview;
pub mod pages;
pub mod project;
/// header の席の pill（pill と card の値は host でも組む・DOM は wasm の target だけ・行 g-seatpill）。
pub mod seatpill;
/// browser の保存（mode などの便利の写し・決め方は host でも組む・行 g-mode-store）。
pub mod store;
/// stylesheet の部品（style の dir の file を名の順につなぎ、wasm が基の後に入れる・行 g-style-parts・判断の記録 ADR-58）。
pub mod style;
/// 幅の 3 段とスマホの段の tile（tile の組みと開く段は host でも組む・DOM は pipeline の block の DOM・行 g-layout）。
pub mod tiles;
/// 上の固定の帯（帯の中身の字と並びは host でも組む・DOM は wasm の target だけ・行 g-topbar）。
pub mod topbar;
pub mod view;
pub mod vocab;
pub mod widgets;
/// 帯の印が開く窓の中身（止まった run の並びは host でも組む・窓の DOM は wasm の target だけ・行 g-win-parts）。
pub mod wins;

#[cfg(target_arch = "wasm32")]
pub mod board;
#[cfg(target_arch = "wasm32")]
pub mod net;
