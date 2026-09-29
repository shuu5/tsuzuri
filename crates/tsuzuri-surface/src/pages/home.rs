//! home の頁: 席からの知らせ・次の一手・pipeline・台帳・凡例 ｜ orchestrator と口座（2 列の grid・見本の `.home`）。
//! 席からの知らせは見本に無い block で、持ち主が最初に見る左の列の頭に置く（行 i-11）。
//! 表示先の block も見本に無く、右の列の orchestrator の下に置く（行 i-stage-own）。

use crate::frame::{Column, PageDef, STACK};
use crate::project::{ledger, legend, next, notice, pipeline, seat, stage};

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
            blocks: vec![
                notice::BLOCK,
                next::BLOCK,
                pipeline::BLOCK,
                ledger::BLOCK,
                legend::BLOCK,
            ],
        },
        Column {
            class: STACK,
            blocks: vec![seat::BLOCK, stage::BLOCK],
        },
    ]
}
