//! home の頁（1 枚の画面・行 g-one-screen-a・判断の記録 ADR-27 決定 (5)）: 台帳 open の一覧 ｜ pipeline の 2 つの面。
//! 席からの知らせ・次の一手・凡例・orchestrator と口座・表示先の block は頁に置かず、帯の印が開く窓（wins の draw）が
//! 同じ中身の関数で描く。面の並べ方（幅の 3 段）は stylesheet が決める。

use crate::frame::{Column, PageDef};
use crate::project::{ledger, pipeline};

/// 頁の class（帯の下の全部の高さを 2 つの面に使う）。
pub const ONE: &str = "one";

/// 面の列の class。
pub const PANE: &str = "pane";

pub const PAGE: PageDef = PageDef {
    heading: "home",
    class: ONE,
    columns,
};

fn columns() -> Vec<Column> {
    vec![
        Column {
            class: PANE,
            blocks: vec![ledger::BLOCK],
        },
        Column {
            class: PANE,
            blocks: vec![pipeline::BLOCK],
        },
    ]
}
