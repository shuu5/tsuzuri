//! 地図の頁（block は map の 1 つ）。

use crate::frame::{Column, PageDef, STACK};
use crate::project::map;

pub const PAGE: PageDef = PageDef {
    heading: "map",
    class: STACK,
    nav: Some(3),
    badge: false,
    icon: r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><rect x="3" y="4" width="18" height="4" rx="1"/><rect x="3" y="10" width="18" height="4" rx="1"/><rect x="3" y="16" width="18" height="4" rx="1"/></svg>"#,
    columns,
};

fn columns() -> Vec<Column> {
    vec![Column {
        class: STACK,
        blocks: vec![map::BLOCK],
    }]
}
