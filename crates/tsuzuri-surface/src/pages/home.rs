//! home の頁: 次の一手・pipeline・台帳・凡例 ｜ orchestrator と口座（2 列の grid・見本の `.home`）。

use crate::frame::{Column, PageDef, STACK};
use crate::project::{ledger, legend, next, pipeline, seat};

pub const PAGE: PageDef = PageDef {
    heading: "home",
    class: "home",
    nav: Some(1),
    badge: false,
    icon: r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M3 11l9-7 9 7v9a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1z"/></svg>"#,
    columns,
};

fn columns() -> Vec<Column> {
    vec![
        Column {
            class: STACK,
            blocks: vec![next::BLOCK, pipeline::BLOCK, ledger::BLOCK, legend::BLOCK],
        },
        Column {
            class: STACK,
            blocks: vec![seat::BLOCK],
        },
    ]
}
