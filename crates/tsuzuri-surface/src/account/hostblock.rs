//! account board の HOME の host の負荷と書きの block（3 段目の口座 × 窓の後）: project board の帯の host の窓と
//! 同じ中身の関数（hostwin の `content`）を、同じ DOM の本文（hostwin の `body`・口は契約の型の host の PATH）で描く。
//! account board の口は読まないので、host の口が読めなくてもほかの block は今のまま出る（要件 NFR2）。

use crate::frame::Block;
use crate::hostwin::HOST_KEY;

/// host の負荷と書き（HOME の 3 段目・口座 × 窓の後）。見出しの語は帯の host の窓の題と同じ鍵。
pub const BLOCK: Block = Block {
    id: "hostld",
    heading: HOST_KEY,
    class: "panel",
};

/// block の DOM（見出しと、帯の host の窓と同じ本文）。
#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    use leptos::prelude::IntoAny;
    crate::project::section(BLOCK, ().into_any(), crate::hostwin::body())
}
