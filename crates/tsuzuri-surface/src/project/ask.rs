//! block「問いの一覧」（便 g-min の中身を見本の class で描き直す）: open の question を古い順に番号つきで並べる。
//! 並べ方は view（便 g-min）が決め、ここは一覧の項と件数にする。答えの欄は後の便が足す。

use tsuzuri_contract::board::Reading;

use super::{Body, Item, LEDGER_UNREAD, item};
use crate::frame::Block;
use crate::view::{Screen, clock};

pub const BLOCK: Block = Block {
    id: "ask",
    heading: "ask_open",
    class: "panel",
};

/// 測れて 0 件のときの 1 行。
pub const EMPTY: &str = "答えを待つ question は無い";

/// 問いの件数（測れていなければ Unknown・0 件と書かない）。
pub fn count(screen: &Screen) -> Reading<usize> {
    match &screen.board {
        Reading::Known(b) => Reading::Known(b.questions.len()),
        Reading::Unknown => Reading::Unknown,
    }
}

/// 一覧の中身（古い順・右は更新の時刻）。
pub fn body(screen: &Screen) -> Body<Vec<Item>> {
    match &screen.board {
        Reading::Unknown => Body::Unmeasured(LEDGER_UNREAD),
        Reading::Known(b) if b.questions.is_empty() => Body::Empty(EMPTY),
        Reading::Known(b) => Body::Filled(
            b.questions
                .iter()
                .map(|q| Item {
                    aside: format!("◷ {}", clock(q.updated_at)),
                    ..item(q)
                })
                .collect(),
        ),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn view(screen: leptos::prelude::RwSignal<crate::view::Screen>) -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    let extra = move || match screen.with(count) {
        Reading::Known(n) => view! { <span class="chip num">{n}</span> }.into_any(),
        Reading::Unknown => ().into_any(),
    };
    let body = move || match screen.with(body) {
        Body::Unmeasured(reason) => super::unmeasured(reason),
        Body::Empty(line) => view! { <div class="empty"><span>{line}</span></div> }.into_any(),
        Body::Filled(items) => {
            let rows = items
                .iter()
                .enumerate()
                .map(|(i, it)| super::item_view(it, Some(i + 1)))
                .collect_view();
            view! { <ul class="items">{rows}</ul> }.into_any()
        }
    };
    super::section(BLOCK, extra.into_any(), body.into_any())
}
