//! 節点の頁（見本の bead.html・block は頭と概要の node と、つながりの around と、本文と記録の body と、
//! run の時間軸の timeline の 4 つ・詳細の表示の順は判断の記録 ADR-30 決定 (4)・行 g-node-body）。
//! query の page=node と id で開く。

use crate::frame::{Column, PageDef, STACK};
use crate::project::{node, nodearound, nodebody, timeline};

pub const PAGE: PageDef = PageDef {
    heading: "nb_self",
    class: STACK,
    columns,
};

fn columns() -> Vec<Column> {
    vec![Column {
        class: STACK,
        blocks: vec![
            node::BLOCK,
            nodearound::BLOCK,
            nodebody::BLOCK,
            timeline::BLOCK,
        ],
    }]
}
