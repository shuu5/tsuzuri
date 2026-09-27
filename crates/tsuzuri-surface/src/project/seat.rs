//! block「orchestrator と口座」（見本の `#orch`）。席の状態と口座を読む口がまだ無いので、測れていないの印と理由の 1 行を出す。
//! 中身（状態の 5 値・から・稼働の記録・登録の口座・利用枠）は後の便がこの module に足す。

use super::Body;
use crate::frame::Block;

pub const BLOCK: Block = Block {
    id: "orch",
    heading: "orch_acct",
    class: "panel seatcard",
};

/// 測れていない理由。
pub const REASON: &str = "orchestrator の状態と口座を読む口（器の state dir）がまだ無い";

pub fn body() -> Body<()> {
    Body::Unmeasured(REASON)
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    use leptos::prelude::IntoAny;
    super::section(BLOCK, ().into_any(), super::unmeasured(REASON))
}
