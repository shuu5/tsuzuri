//! block「これまでの決定」（問いの頁・便 g-ask）: 答え済みの問いを畳める段に並べる（見本の ask.html の `#hist`）。
//! 中は台帳の一覧（口 /api/ledger・定数は ledger の module に 1 本）から、label intake:question を持つ closed の bead を
//! id の自然な順（数は数として比べる）に出す。方針の問い（範囲の札の label を持つ）は承認でない（要件 FR8）ので
//! 出さず数えない。段は最初は閉じていて、見出しに件数を出す。
//! URL の `?id=` で名指された問いが答え済みなら、段を開いてその行に背景を置き画面の上端へ寄せる（便 g-ask-focus）。
//! 選んで並べる関数と件数は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use std::collections::BTreeMap;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::{EdgeType, GraphDoc};
use tsuzuri_contract::ledger::{LedgerList, LedgerRow};
use tsuzuri_contract::wire;

use super::{Body, Item, LEDGER_UNREAD, NOT_READ, item};
use crate::frame::Block;
use crate::view::{Fetched, JST, hhmm, id_order};

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
/// 方針の問い（範囲の札の label を持つ）は承認でない（要件 FR8）ので出さず数えない。
pub fn rulings(rows: &[LedgerRow]) -> Vec<LedgerRow> {
    let mut out: Vec<LedgerRow> = rows
        .iter()
        .filter(|r| r.is_question() && r.status == "closed" && !r.is_policy())
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

/// 段が最初に開くか（OPEN が真か、URL で名指された問いが答え済みの項に在れば開く）。
pub fn opens(fetched: &Fetched, focus: Option<&str>) -> bool {
    OPEN || focus.is_some_and(|id| match body(fetched) {
        Body::Filled(items) => focus_index(&items, id).is_some(),
        _ => false,
    })
}

/// 名指された問いの項の位置（0 から数える・無ければ None）。
pub fn focus_index(items: &[Item], id: &str) -> Option<usize> {
    items.iter().position(|it| it.id == id)
}

/// 段の一覧の位置の行を選ぶ字（CSS の nth-child は 1 から数える）。
pub fn row_selector(index: usize) -> String {
    format!("#{} ul.items > li:nth-child({})", BLOCK.id, index + 1)
}

/// 名指された行に置く背景（見本の script が li に置く色と同じ）。
pub const HIGHLIGHT: &str = "background:var(--panel-2)";

/// グラフの口の本文から、問いの id ごとにそれに答えた決定の id（型 answers の辺の to から from・便 g-hist-ruling）。
/// 同じ問いへの辺が 2 本以上なら電文の順の最後（答え直した今の決定）。読めなければ空。
pub fn ruling_links(fetched: &Fetched) -> BTreeMap<String, String> {
    let Fetched::Body(text) = fetched else {
        return BTreeMap::new();
    };
    let Ok(doc) = wire::decode::<GraphDoc>(text) else {
        return BTreeMap::new();
    };
    doc.edges
        .into_iter()
        .filter(|e| e.edge_type == EdgeType::Answers)
        .map(|e| (e.to, e.from))
        .collect()
}

/// 段の 1 行（台帳の項と、その問いに答えた決定の id・無ければ None）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistEntry {
    pub item: Item,
    pub ruling: Option<String>,
}

/// 項の順のまま、項ごとに答えた決定の id を添える（グラフが読めなければどれも None）。
pub fn hist_rows(items: &[Item], graph: &Fetched) -> Vec<HistEntry> {
    let links = ruling_links(graph);
    items
        .iter()
        .map(|it| HistEntry {
            item: it.clone(),
            ruling: links.get(&it.id).cloned(),
        })
        .collect()
}

/// 決定の link の字: server の形の id（問いの id・字 :・UTC の分・字 -・数）なら、その UTC の時分を
/// 日本時間の時分と空白と JST にした字（例 T1300Z は 22:00 JST・日を越えても時分だけで T1500Z は 00:00 JST）、
/// ほかの形（手書きの古い id）は id のまま。年月日と時分の値の範囲は見ない。id の字は記録の鍵なので変えない。
pub fn ruling_text(id: &str) -> String {
    let Some((head, tail)) = id.rsplit_once(':') else {
        return id.to_string();
    };
    let b = tail.as_bytes();
    let digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
    let server = !head.is_empty()
        && b.len() >= 16
        && digits(0..8)
        && b[8] == b'T'
        && digits(9..13)
        && b[13] == b'Z'
        && b[14] == b'-'
        && digits(15..b.len());
    if server {
        let num = |s: &str| s.parse::<u64>().unwrap_or(0);
        let secs = num(&tail[9..11]) * 3600 + num(&tail[11..13]) * 60;
        format!("{} {JST}", hhmm(secs))
    } else {
        id.to_string()
    }
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    use crate::frame::Mode;
    use crate::widgets::help::{HelpCtx, h2};

    let graph = crate::net::read(super::map::PATH);
    let search = window().location().search().unwrap_or_default();
    let mode = {
        let ctx = use_context::<HelpCtx>();
        let url = Mode::from_query(&search);
        move || ctx.map_or(url, |c| c.mode.get())
    };
    let target = super::ask::focus(&search);
    let lit = StoredValue::new(false);
    let fetched = crate::net::read(super::ledger::PATH);
    let chip = move || match fetched.with(count) {
        Reading::Known(n) => view! { <span class="chip num">{n}</span> }.into_any(),
        Reading::Unknown => ().into_any(),
    };
    let initial = {
        let target = target.clone();
        move || fetched.with(|f| opens(f, target.as_deref()))
    };
    let list = move || match fetched.with(body) {
        Body::Unmeasured(reason) => super::unmeasured(reason),
        Body::Empty(line) => super::body_view(Body::Empty(line)),
        Body::Filled(items) => {
            if let Some(id) = target.as_deref()
                && !lit.get_value()
                && let Some(i) = focus_index(&items, id)
            {
                lit.set_value(true);
                let selector = row_selector(i);
                request_animation_frame(move || {
                    if let Ok(Some(el)) = document().query_selector(&selector) {
                        let _ = el.set_attribute("style", HIGHLIGHT);
                        el.scroll_into_view_with_bool(true);
                    }
                });
            }
            let rows = graph
                .with(|g| hist_rows(&items, g))
                .into_iter()
                .map(|entry| hist_li(entry, mode))
                .collect_view();
            view! { <ul class="items">{rows}</ul> }.into_any()
        }
    };
    let (open, toggle) = super::fold("ask:hist".to_string(), initial);
    view! {
        <details class=BLOCK.class id=BLOCK.id prop:open=open on:toggle=toggle>
            <summary>{h2(BLOCK.heading)}{chip}</summary>
            {list}
        </details>
    }
    .into_any()
}

/// 段の 1 行（印・題は問いの頁への link・右の小さい字は答えた決定の頁への link、無ければ状態の字）。
#[cfg(target_arch = "wasm32")]
fn hist_li(
    entry: HistEntry,
    mode: impl Fn() -> crate::frame::Mode + Copy + Send + Sync + 'static,
) -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    use crate::frame::node_href;

    let HistEntry { item, ruling } = entry;
    let style = if item.alert { super::ALERT_STYLE } else { "" };
    let id = item.id.clone();
    let href = move || node_href(&id, mode());
    view! {
        <li>
            <span class=item.shape.clone() style=style aria-hidden="true"></span>
            <a class="ttl" href=href><span class="nid">{item.id.clone()}</span>" "<span data-t="">{item.title.clone()}</span></a>
            <span class="aside">{match ruling {
                Some(rid) => view! {
                    <a href={let rid = rid.clone(); move || node_href(&rid, mode())}>{ruling_text(&rid)}</a>
                }
                .into_any(),
                None => item.aside.clone().into_any(),
            }}</span>
        </li>
    }
    .into_any()
}
