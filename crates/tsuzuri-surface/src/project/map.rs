//! 地図の頁（見本の map.html）。口（/api/graph）はまだ server に無いので、枠と見出しと測れていないの印と理由の 1 行だけを置く。
//! 中身（4 面の切り替え・帯・札）は後の便がこの module に足す。

use super::{Body, pending};
use crate::frame::Block;
use crate::view::Fetched;

pub const BLOCK: Block = Block {
    id: "map",
    heading: "map",
    class: "panel",
};

/// 読みの口（便 g-parts）。
pub const PATH: &str = "/api/graph";

/// 口が読めないときの理由。
pub const REASON: &str =
    "地図の 7 つの出所を組む口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 中身（この便は 3 値のどれでも測れていない）。
pub fn body(fetched: &Fetched) -> Body<()> {
    pending(fetched, REASON)
}

/// 見出しは頁の題（h1）にする（見本の map.html と同じ）。
#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    use crate::widgets::help::h1;

    let fetched = crate::net::read(PATH);
    let content = move || super::body_view(fetched.with(body));
    view! {
        <section class=BLOCK.class id=BLOCK.id>
            <header>{h1(BLOCK.heading)}</header>
            {content}
        </section>
    }
    .into_any()
}
