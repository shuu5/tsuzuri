//! block「これまでの決定」（問いの頁）: 答え済みの問いを畳める段に並べる（見本の ask.html の `#hist`）。
//! 中は台帳の一覧（口 /api/ledger・定数は ledger の module に 1 本）から、label intake:question を持つ closed の bead を
//! id の自然な順（数は数として比べる）に出す。方針の問い（範囲の札の label を持つ）は承認でない（要件 FR8）ので
//! 出さず数えない。段は最初は閉じていて、見出しに件数を出す。
//! URL の `?id=` で名指された問いが答え済みなら、段を開いてその行に背景を置き画面の上端へ寄せる。
//! 選んで並べる関数と件数は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。
//! 行の題と答えた決定の link には、グラフの口の電文から引いた節点の hover の card を付ける。
//! 段は質問の窓の下の畳みの末に置く（見本の qModal の「これまでの決定 N ›」）: 行は台帳の更新の時刻の
//! 新しい順で、時刻・短い題（bead の事実の口）・答えた決定・問いの id を出す（`decisions`・`inner`）。頁には置かない。

use std::collections::BTreeMap;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::{EdgeType, GraphDoc};
use tsuzuri_contract::ledger::{LedgerList, LedgerRow};
use tsuzuri_contract::wire;

use super::{Body, Item, LEDGER_UNREAD, NOT_READ, item};
use crate::frame::{Block, Mode};
use crate::ledgerlist::{facts, short_of};
use crate::view::{Fetched, JST, hhmm, id_order};
use crate::vocab::label;
use crate::widgets::hover::Card;
use crate::widgets::nodecard::card_of_in;

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

/// 行の問いの id と答えた決定の id ごとの節点の hover の card（初心者の表示の型の `hist_cards_in`）。
pub fn hist_cards(entries: &[HistEntry], graph: &Fetched) -> BTreeMap<String, Card> {
    hist_cards_in(entries, graph, Mode::Beginner)
}

/// 行の問いの id と答えた決定の id ごとの節点の表示の型の hover の card（グラフの口が読めなければ空・電文に無い id は持たない）。
pub fn hist_cards_in(entries: &[HistEntry], graph: &Fetched, mode: Mode) -> BTreeMap<String, Card> {
    let Ok(doc) = super::map::doc(graph) else {
        return BTreeMap::new();
    };
    entries
        .iter()
        .flat_map(|e| std::iter::once(e.item.id.as_str()).chain(e.ruling.as_deref()))
        .filter_map(|id| card_of_in(&doc, id, mode).map(|c| (id.to_string(), c)))
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
    let digits = |r: std::ops::Range<usize>| {
        b.get(r)
            .is_some_and(|s| s.iter().all(u8::is_ascii_digit))
    };
    let server = !head.is_empty()
        && b.len() >= 16
        && digits(0..8)
        && b.get(8) == Some(&b'T')
        && digits(9..13)
        && b.get(13) == Some(&b'Z')
        && b.get(14) == Some(&b'-')
        && digits(15..b.len());
    if server {
        let num = |s: &str| s.parse::<u64>().unwrap_or(0);
        let secs = num(&tail[9..11]) * 3600 + num(&tail[11..13]) * 60;
        format!("{} {JST}", hhmm(secs))
    } else {
        id.to_string()
    }
}

/// 質問の窓の畳みの 1 行（行 g-ask-hist・見本の qModal の「これまでの決定」の行）: 台帳の更新の時刻（閉じた時刻）・
/// 短い題（bead の事実の short・事実が無ければ台帳の題）・問いの項と答えた決定の id。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub at: EpochSecs,
    pub short: String,
    pub entry: HistEntry,
}

/// 質問の窓の畳みの中身（行 g-ask-hist）: これまでの決定（`rulings`）を台帳の更新の時刻の新しい順（同じなら id の
/// 自然な順の逆）に並べ、短い題と答えた決定の id（`hist_rows`）を添える。短い題は一覧の行と同じく bead の事実の short
/// （metadata の short か題から機械で作った字・判断の記録 ADR-27 決定 (9)）で、事実が無ければ台帳の題。
/// 台帳が読めない時と 0 件の時は `body` と同じ理由と 1 行。bead の事実の口とグラフの口が読めなければ、題は台帳の題で決定の id はどれも None。
pub fn decisions(ledger: &Fetched, graph: &Fetched, beads: &Fetched) -> Body<Vec<Decision>> {
    let rows = match rows(ledger) {
        Ok(rows) => rows,
        Err(reason) => return Body::Unmeasured(reason),
    };
    let mut done = rulings(&rows);
    if done.is_empty() {
        return Body::Empty(EMPTY);
    }
    done.sort_by(|a, b| {
        b.updated_at
            .cmp(&a.updated_at)
            .then_with(|| id_order(b.id.as_str(), a.id.as_str()))
    });
    let facts = facts(beads);
    let items: Vec<Item> = done.iter().map(item).collect();
    Body::Filled(
        hist_rows(&items, graph)
            .into_iter()
            .zip(&done)
            .map(|(entry, row)| Decision {
                at: row.updated_at,
                short: short_of(row, &facts),
                entry,
            })
            .collect(),
    )
}

/// 畳みの見出しの字（見本の「これまでの決定 N ›」・測れていなければ数を出さない）。
pub fn fold_title(count: Reading<usize>) -> String {
    match count {
        Reading::Known(n) => format!("{} {n} ›", label(BLOCK.heading)),
        Reading::Unknown => format!("{} ›", label(BLOCK.heading)),
    }
}

/// block の中身（質問の窓の下の畳みと同じ DOM・頁には置かない・URL の `?id=` の問いを名指す）。
#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    let search = leptos::prelude::window()
        .location()
        .search()
        .unwrap_or_default();
    inner(super::ask::focus(&search))
}

/// 質問の窓の下の畳みの段（行 g-ask-hist・見本の qModal の folds の末の「これまでの決定 N ›」）: 台帳の一覧の口と
/// グラフの口と bead の事実の口を読み、新しい順の決定を時刻・短い題・答えた決定・問いの id の行に出す。
/// 名指された問い（`target`）が答え済みの項に在れば、段を開いてその行に背景を置き窓の中へ寄せる（便 g-ask-focus）。
#[cfg(target_arch = "wasm32")]
pub fn inner(target: Option<String>) -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    use crate::frame::Mode;
    use crate::widgets::help::HelpCtx;

    let graph = crate::net::read(super::map::PATH);
    let beads = crate::net::read(crate::ledgerlist::FACTS_PATH);
    let fetched = crate::net::read(super::ledger::PATH);
    let mode = {
        let ctx = use_context::<HelpCtx>();
        let url = Mode::from_query(&window().location().search().unwrap_or_default());
        move || ctx.map_or(url, |c| c.mode.get())
    };
    let lit = StoredValue::new(false);
    let title = move || fold_title(fetched.with(count));
    let initial = {
        let target = target.clone();
        move || fetched.with(|f| opens(f, target.as_deref()))
    };
    let list = move || match fetched.with(|l| graph.with(|g| beads.with(|b| decisions(l, g, b)))) {
        Body::Unmeasured(reason) => super::unmeasured(reason),
        Body::Empty(line) => super::body_view(Body::Empty(line)),
        Body::Filled(ds) => {
            let items: Vec<Item> = ds.iter().map(|d| d.entry.item.clone()).collect();
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
            let entries: Vec<HistEntry> = ds.iter().map(|d| d.entry.clone()).collect();
            let cards = graph.with(|g| hist_cards_in(&entries, g, mode()));
            let now = crate::net::now();
            let rows = ds
                .into_iter()
                .map(|d| hist_li(d, &cards, now, mode))
                .collect_view();
            view! { <ul class="items">{rows}</ul> }.into_any()
        }
    };
    let (open, toggle) = super::fold("ask:hist".to_string(), initial);
    view! {
        <details class="fold" id=BLOCK.id prop:open=open on:toggle=toggle>
            <summary>{title}</summary>
            {list}
        </details>
    }
    .into_any()
}

/// 畳みの 1 行（時刻・短い題・答えた決定の頁への link か状態の字・問いの頁への link の id・見本の `.mini li`）。
#[cfg(target_arch = "wasm32")]
fn hist_li(
    d: Decision,
    cards: &BTreeMap<String, Card>,
    now: EpochSecs,
    mode: impl Fn() -> crate::frame::Mode + Copy + Send + Sync + 'static,
) -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    use crate::frame::node_href;
    use crate::view::clock_short;
    use crate::widgets::hover::attach_some;

    let Decision { at, short, entry } = d;
    let HistEntry { item, ruling } = entry;
    let id = item.id.clone();
    let card = cards.get(&id).cloned();
    let href = move || node_href(&id, mode());
    view! {
        <li>
            <time>{clock_short(at, now)}</time>
            <span class="ttl" title=item.title.clone() data-t="" data-ledger-text="">{crate::widgets::hover::clip(&short)}</span>
            <b class="aside">{match ruling {
                Some(rid) => view! {
                    <a href={let rid = rid.clone(); move || node_href(&rid, mode())} use:attach_some=cards.get(&rid).cloned()>{ruling_text(&rid)}</a>
                }
                .into_any(),
                None => item.aside.clone().into_any(),
            }}</b>
            <a class="lk" href=href use:attach_some=card><code>{item.id.clone()}</code></a>
        </li>
    }
    .into_any()
}
