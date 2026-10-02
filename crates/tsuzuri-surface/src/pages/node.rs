//! 節点の頁（見本の bead.html・block は頭と概要の node と、つながりの around と、run の時間軸の timeline の 3 つ）。
//! query の page=node と id で開く。

use crate::frame::{Column, PageDef, STACK};
use crate::project::{node, nodearound, timeline};

pub const PAGE: PageDef = PageDef {
    heading: "nb_self",
    class: STACK,
    columns,
};

fn columns() -> Vec<Column> {
    vec![Column {
        class: STACK,
        blocks: vec![node::BLOCK, nodearound::BLOCK, timeline::BLOCK],
    }]
}
