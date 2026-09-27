//! block「記号の見方」（見本の `#legend`）: 状態の記号 5 値の凡例と、一覧の項に付く bd の状態の印の凡例。

use super::STATES;
use crate::frame::Block;
use crate::view::{Mark, mark};

pub const BLOCK: Block = Block {
    id: "legend",
    heading: "status",
    class: "panel",
};

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

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    use crate::vocab::label;

    let states = states()
        .into_iter()
        .map(|e| view! { <span data-term=e.key>{super::state_icon(e.state)}{label(e.key)}</span> })
        .collect_view();
    let marks = marks()
        .into_iter()
        .map(|m| view! { <span><b>{m.glyph}</b>{m.word}</span> })
        .collect_view();
    let body = view! {
        <div class="legend5">{states}</div>
        <div class="lgline">{marks}</div>
    };
    super::section(BLOCK, ().into_any(), body.into_any())
}
