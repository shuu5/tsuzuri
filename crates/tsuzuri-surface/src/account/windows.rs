//! account board の開いている窓の block（便 h-frame は枠だけ）: 見本の `#winsp`（HOME の 1 段目の右）。
//! 中身は後の便（h-win）が描く。この便は口の読みの結果から中身の有無だけを出す。

use crate::frame::Block;
use crate::project::Body;
use crate::view::Fetched;

pub const BLOCK: Block = Block {
    id: "winsp",
    heading: "open_windows",
    class: "panel",
};

/// block の中身（中身の関数がまだ無い: 測れていない）。
pub fn body(fetched: &Fetched) -> Body<()> {
    super::pending(fetched)
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    super::section_view(BLOCK)
}
