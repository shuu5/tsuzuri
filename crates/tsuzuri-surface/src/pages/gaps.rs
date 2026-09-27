//! 抜けの検査の頁（見本の gaps.html・block は gaps の 1 つ）。

use crate::frame::{Column, PageDef, STACK};
use crate::project::gaps;

pub const PAGE: PageDef = PageDef {
    heading: "gaps",
    class: STACK,
    nav: Some(4),
    badge: false,
    icon: r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="11" cy="11" r="6"/><path d="M20 20l-4.5-4.5"/><path d="M8.5 11h5"/></svg>"#,
    columns,
};

fn columns() -> Vec<Column> {
    vec![Column {
        class: STACK,
        blocks: vec![gaps::BLOCK],
    }]
}
