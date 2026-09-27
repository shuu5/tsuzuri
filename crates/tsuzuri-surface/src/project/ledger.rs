//! block「台帳」（見本の `#ledger`）: 指標の段（見本の 4 数）と台帳の一覧（便 g-min の中身を見本の class で描き直す）。
//! 指標の段は組む口がまだ無いので測れていないと出す（中身は後の便が足す）。一覧は epic の下に task・memo の順。

use tsuzuri_contract::board::Reading;

use super::{Body, Item, LEDGER_UNREAD, item};
use crate::frame::Block;
use crate::view::Screen;

pub const BLOCK: Block = Block {
    id: "ledger",
    heading: "ledger_block",
    class: "panel",
};

/// 指標の段の語の鍵（見本の 4 数 = open task・memo・未反映・純減 24h）。
pub const METRICS: [&str; 4] = ["l_task", "l_memo", "l_unref", "l_net24"];

/// 指標の段が測れていない理由。
pub const METRICS_REASON: &str = "台帳の指標（積みと速度）を組む口がまだ無い";

/// 測れて 0 件のときの 1 行。
pub const EMPTY: &str = "台帳に bead は無い";

/// epic の外の組の見出しの字（epic の項の代わり）。
pub const OUTSIDE: &str = "epic の外";

/// 一覧の 1 組（epic の項・その下の項）。epic の外の組は `head` が None。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub head: Option<Item>,
    pub children: Vec<Item>,
}

/// 指標の段（この便はいつも測れていない）。
pub fn metrics() -> Body<()> {
    Body::Unmeasured(METRICS_REASON)
}

/// 台帳の件数（epic も数える・測れていなければ Unknown）。
pub fn count(screen: &Screen) -> Reading<usize> {
    match &screen.board {
        Reading::Known(b) => Reading::Known(
            b.groups
                .iter()
                .map(|g| g.children.len() + usize::from(g.epic.is_some()))
                .sum(),
        ),
        Reading::Unknown => Reading::Unknown,
    }
}

/// 一覧の中身。
pub fn body(screen: &Screen) -> Body<Vec<Group>> {
    match &screen.board {
        Reading::Unknown => Body::Unmeasured(LEDGER_UNREAD),
        Reading::Known(b) if b.groups.is_empty() => Body::Empty(EMPTY),
        Reading::Known(b) => Body::Filled(
            b.groups
                .iter()
                .map(|g| Group {
                    head: g.epic.as_ref().map(item),
                    children: g.children.iter().map(item).collect(),
                })
                .collect(),
        ),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn view(screen: leptos::prelude::RwSignal<Screen>) -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    use crate::widgets::help::hs;

    let extra = move || match screen.with(count) {
        Reading::Known(n) => view! { <span class="chip num">{n}</span> }.into_any(),
        Reading::Unknown => ().into_any(),
    };
    let boxes = METRICS
        .iter()
        .map(|k| {
            view! {
                <div>
                    <span class="small muted">{hs(k)}</span>
                    <span class="big">{super::state_icon(super::UNKNOWN)}</span>
                </div>
            }
        })
        .collect_view();
    let reason = match metrics() {
        Body::Unmeasured(reason) => super::unmeasured(reason),
        _ => ().into_any(),
    };
    let list = move || match screen.with(body) {
        Body::Unmeasured(reason) => super::unmeasured(reason),
        Body::Empty(line) => view! { <div class="empty"><span>{line}</span></div> }.into_any(),
        Body::Filled(groups) => {
            let rows = groups.iter().map(group_view).collect_view();
            view! { <ul class="items">{rows}</ul> }.into_any()
        }
    };
    let body = view! {
        <div class="l4">{boxes}</div>
        {reason}
        <div class="lmid">{list}</div>
    };
    super::section(BLOCK, extra.into_any(), body.into_any())
}

/// 1 組（epic の項と、その下の項を入れ子の一覧に）。
#[cfg(target_arch = "wasm32")]
fn group_view(group: &Group) -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    let head = match &group.head {
        Some(epic) => super::item_view(epic, None),
        None => view! {
            <li>
                <span class="shape band-beads" aria-hidden="true"></span>
                <span class="ttl muted">{OUTSIDE}</span>
            </li>
        }
        .into_any(),
    };
    let children = group
        .children
        .iter()
        .map(|c| super::item_view(c, None))
        .collect_view();
    view! {
        {head}
        <li class="nest"><ul class="items">{children}</ul></li>
    }
    .into_any()
}
