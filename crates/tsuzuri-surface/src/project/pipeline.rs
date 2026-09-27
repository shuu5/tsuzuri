//! block「pipeline」（見本の `#pipe`）。口（/api/pipeline）はまだ server に無いので、測れていないの印と理由の 1 行を出す。
//! 中身（4 列の板・札・「+n」）は後の便がこの module に足す。

use super::{Body, pending};
use crate::frame::Block;
use crate::view::Fetched;

pub const BLOCK: Block = Block {
    id: "pipe",
    heading: "pipeline",
    class: "panel",
};

/// 読みの口（便 g-parts）。
pub const PATH: &str = "/api/pipeline";

/// 口が読めないときの理由。
pub const REASON: &str =
    "run の段を読む口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 中身（この便は 3 値のどれでも測れていない）。
pub fn body(fetched: &Fetched) -> Body<()> {
    pending(fetched, REASON)
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    let fetched = crate::net::read(PATH);
    let content = move || super::body_view(fetched.with(body));
    super::section(BLOCK, ().into_any(), content.into_any())
}
