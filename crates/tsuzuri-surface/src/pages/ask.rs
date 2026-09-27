//! 問いの頁: 問いの card の列・これまでの決定 ｜ まとめて承認・全体への指示（見本の ask.html の `.ask` の 2 列）。
//! nav の link に open の問いの数の印を付ける。

use crate::frame::{Column, PageDef, SIDE, STACK};
use crate::project::{ask, askpage, batch, policy};

pub const PAGE: PageDef = PageDef {
    heading: "questions",
    class: "ask",
    nav: Some(2),
    badge: true,
    icon: r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M4 5h16v11H9l-5 4z"/><path d="M10 9.5a2 2 0 1 1 2.8 1.8c-.5.3-.8.7-.8 1.2"/><circle cx="12" cy="14.5" r=".6" fill="currentColor"/></svg>"#,
    columns,
};

fn columns() -> Vec<Column> {
    vec![
        Column {
            class: STACK,
            blocks: vec![ask::BLOCK, askpage::BLOCK],
        },
        Column {
            class: SIDE,
            blocks: vec![batch::BLOCK, policy::BLOCK],
        },
    ]
}
