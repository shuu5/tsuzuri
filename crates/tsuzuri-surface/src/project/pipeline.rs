//! block「pipeline」（見本の `#pipe`）。run の段を読む口がまだ無いので、測れていないの印と理由の 1 行を出す。
//! 中身（4 列の板・札・「+n」）は後の便がこの module に足す。

use super::Body;
use crate::frame::Block;

pub const BLOCK: Block = Block {
    id: "pipe",
    heading: "pipeline",
    class: "panel",
};

/// 測れていない理由。
pub const REASON: &str = "run の段を読む口（器の event の記録）がまだ無い";

pub fn body() -> Body<()> {
    Body::Unmeasured(REASON)
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    use leptos::prelude::IntoAny;
    super::section(BLOCK, ().into_any(), super::unmeasured(REASON))
}
