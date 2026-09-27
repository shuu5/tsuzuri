//! block「これまでの決定」（問いの頁・便 g-ask）: 答え済みの問いを畳める段に並べる（見本の ask.html の `#hist`）。
//! 中は台帳の一覧（口 /api/ledger・定数は ledger の module に 1 本）から、label intake:question を持つ closed の bead を
//! id の自然な順（数は数として比べる）に出す。段は最初は閉じていて、見出しに件数を出す。
//! 選んで並べる関数と件数は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{LedgerList, LedgerRow};
use tsuzuri_contract::wire;

use super::{Body, Item, LEDGER_UNREAD, NOT_READ, item};
use crate::frame::Block;
use crate::view::{Fetched, id_order};

pub const BLOCK: Block = Block {
    id: "hist",
    heading: "rulings",
    class: "fold panel",
};

/// この file が字を持つ口の path（無い・台帳の口は ledger の module が持つ・行 hs-derived）。
pub const PATHS: &[&str] = &[];

/// この file の畳める段の開き閉じの鍵の形（行 hs-derived）。
pub const FOLDS: &[&str] = &["ask:hist"];

/// 段が最初に開いているか（閉じている）。
pub const OPEN: bool = false;

/// 測れて 0 件のときの 1 行。
pub const EMPTY: &str = "答え済みの question は無い";

/// 本文が電文として読めないときの理由。
pub const UNREADABLE: &str = "台帳の一覧の本文が電文として読めない";

/// server が台帳を読めず行が「まだ分からない」ときの理由。
pub const ROWS_UNKNOWN: &str = "server が台帳を読めず、これまでの決定が分からない";

/// 口の本文を台帳の行に読む（まだ読んでいない・読めない・電文が読めない・まだ分からないは理由）。
pub fn rows(fetched: &Fetched) -> Result<Vec<LedgerRow>, &'static str> {
    match fetched {
        Fetched::NotRead => Err(NOT_READ),
        Fetched::Failed => Err(LEDGER_UNREAD),
        Fetched::Body(text) => match wire::decode::<LedgerList>(text) {
            Ok(LedgerList {
                rows: Reading::Known(rows),
            }) => Ok(rows),
            Ok(LedgerList {
                rows: Reading::Unknown,
            }) => Err(ROWS_UNKNOWN),
            Err(_) => Err(UNREADABLE),
        },
    }
}

/// これまでの決定: label intake:question を持つ closed の bead を id の自然な順に。
pub fn rulings(rows: &[LedgerRow]) -> Vec<LedgerRow> {
    let mut out: Vec<LedgerRow> = rows
        .iter()
        .filter(|r| r.is_question() && r.status == "closed")
        .cloned()
        .collect();
    out.sort_by(|a, b| id_order(a.id.as_str(), b.id.as_str()));
    out
}

/// 見出しの件数（測れていなければ Unknown・0 件と書かない）。
pub fn count(fetched: &Fetched) -> Reading<usize> {
    match rows(fetched) {
        Ok(rows) => Reading::Known(rulings(&rows).len()),
        Err(_) => Reading::Unknown,
    }
}

/// 段の中身（一覧の項）。
pub fn body(fetched: &Fetched) -> Body<Vec<Item>> {
    match rows(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(rows) => {
            let done = rulings(&rows);
            if done.is_empty() {
                Body::Empty(EMPTY)
            } else {
                Body::Filled(done.iter().map(item).collect())
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    use crate::widgets::help::h2;

    let fetched = crate::net::read(super::ledger::PATH);
    let chip = move || match fetched.with(count) {
        Reading::Known(n) => view! { <span class="chip num">{n}</span> }.into_any(),
        Reading::Unknown => ().into_any(),
    };
    let list = move || match fetched.with(body) {
        Body::Unmeasured(reason) => super::unmeasured(reason),
        Body::Empty(line) => super::body_view(Body::Empty(line)),
        Body::Filled(items) => {
            let rows = items
                .iter()
                .map(|it| super::item_view(it, None))
                .collect_view();
            view! { <ul class="items">{rows}</ul> }.into_any()
        }
    };
    let (open, toggle) = super::fold("ask:hist".to_string(), || OPEN);
    view! {
        <details class=BLOCK.class id=BLOCK.id prop:open=open on:toggle=toggle>
            <summary>{h2(BLOCK.heading)}{chip}</summary>
            {list}
        </details>
    }
    .into_any()
}
