//! block「記号の見方」（見本の `#legend`）: 状態の記号 5 値の凡例と、一覧の項に付く bd の状態の印の凡例と、
//! 画面の見方（1 枚の画面・上の帯・窓・吹き出し・幅の 3 段・段の tile の語と「?」の注釈・行 g-help-sweep）。

use super::STATES;
use crate::frame::Block;
use crate::view::{Mark, mark};

pub const BLOCK: Block = Block {
    id: "legend",
    heading: "status",
    class: "panel",
};

/// この file が字を持つ口の path（無い・行 hs-derived）。
pub const PATHS: &[&str] = &[];

/// この file の畳める段の開き閉じの鍵の形（無い・行 hs-derived）。
pub const FOLDS: &[&str] = &[];

/// 凡例の 1 項（記号の値・語の鍵）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    pub state: &'static str,
    pub key: &'static str,
}

/// 状態の記号 5 値の凡例（見本と同じ順）。
pub fn states() -> Vec<Entry> {
    STATES
        .iter()
        .map(|&(state, key)| Entry { state, key })
        .collect()
}

/// bd の状態の語（一覧の項の右に出る印の並び）。
pub const BD_STATUSES: [&str; 5] = ["open", "in_progress", "blocked", "deferred", "closed"];

/// bd の状態の印の凡例。
pub fn marks() -> Vec<Mark> {
    BD_STATUSES.iter().map(|s| mark(s)).collect()
}

/// 画面の見方の段の見出しの語の鍵。
pub const HOW_KEY: &str = "how";

/// 画面の見方の語の鍵（並べる順・注釈が部品と開き方を説明する）。
pub const HOW: [&str; 6] = [
    "how_one", "how_bar", "how_win", "how_pop", "how_lay", "how_tile",
];

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    super::section(BLOCK, ().into_any(), inner())
}

/// 記号の見方の窓の中身（見出しの無い本文・帯の設定の中の口が開く・行 g-win-parts）。
#[cfg(target_arch = "wasm32")]
pub fn inner() -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    use crate::vocab::label;
    use crate::widgets::help::hs;

    let states = states()
        .into_iter()
        .map(|e| view! { <span data-term=e.key>{super::state_icon(e.state)}{label(e.key)}</span> })
        .collect_view();
    let marks = marks()
        .into_iter()
        .map(|m| view! { <span><b>{m.glyph}</b>{m.word}</span> })
        .collect_view();
    view! {
        <div class="legend5">{states}</div>
        <div class="lgline">{marks}</div>
        <div class="lghow">{hs(HOW_KEY)}{HOW.map(hs).into_iter().collect_view()}</div>
    }
    .into_any()
}
