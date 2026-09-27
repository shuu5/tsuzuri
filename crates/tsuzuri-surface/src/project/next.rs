//! block「次の一手」（見本の `#next`）。口（/api/next）はまだ server に無いので、測れていないの印と理由の 1 行を出す。
//! 中身（閉じた一覧 7 種・先頭の 1 つを大きく）は後の便がこの module に足す。

use super::{Body, pending};
use crate::frame::Block;
use crate::view::Fetched;

pub const BLOCK: Block = Block {
    id: "next",
    heading: "next",
    class: "panel",
};

/// 読みの口（便 g-parts）。
pub const PATH: &str = "/api/next";

/// 口が読めないときの理由。
pub const REASON: &str =
    "次の一手を判じる口が読めない（server にまだ無い・届かない・知らせが切れた）";

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
