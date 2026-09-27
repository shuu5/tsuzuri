//! 節点の頁（見本の bead.html・block は頭と概要の node と、つながりの around の 2 つ）。
//! query の page=node と id で開き、nav には出さない（印の字も空）。

use crate::frame::{Column, PageDef, STACK};
use crate::project::{node, nodearound};

pub const PAGE: PageDef = PageDef {
    heading: "nb_self",
    class: STACK,
    nav: None,
    badge: false,
    icon: "",
    columns,
};

fn columns() -> Vec<Column> {
    vec![Column {
        class: STACK,
        blocks: vec![node::BLOCK, nodearound::BLOCK],
    }]
}
