//! 地図の頁（見本の map.html）。7 つの出所を読む口がまだ無いので、枠と見出しと測れていないの印と理由の 1 行だけを置く。
//! 中身（4 面の切り替え・帯・札）は後の便がこの module に足す。

use super::Body;
use crate::frame::Block;

pub const BLOCK: Block = Block {
    id: "map",
    heading: "map",
    class: "panel",
};

/// 測れていない理由。
pub const REASON: &str = "地図の 7 つの出所（design-intent・台帳・器の記録）を読む口がまだ無い";

pub fn body() -> Body<()> {
    Body::Unmeasured(REASON)
}

/// 見出しは頁の題（h1）にする（見本の map.html と同じ）。
#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    use crate::widgets::help::h1;
    view! {
        <section class=BLOCK.class id=BLOCK.id>
            <header>{h1(BLOCK.heading)}</header>
            {super::unmeasured(REASON)}
        </section>
    }
    .into_any()
}
