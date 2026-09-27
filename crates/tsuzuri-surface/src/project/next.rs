//! block「次の一手」（見本の `#next`）。判じる data の口がまだ無いので、測れていないの印と理由の 1 行を出す。
//! 中身（閉じた一覧 7 種・先頭の 1 つを大きく）は後の便がこの module に足す。

use super::Body;
use crate::frame::Block;

pub const BLOCK: Block = Block {
    id: "next",
    heading: "next",
    class: "panel",
};

/// 測れていない理由。
pub const REASON: &str = "次の一手を判じる data の口（器の記録・質問の台帳）がまだ無い";

pub fn body() -> Body<()> {
    Body::Unmeasured(REASON)
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    use leptos::prelude::IntoAny;
    super::section(BLOCK, ().into_any(), super::unmeasured(REASON))
}
