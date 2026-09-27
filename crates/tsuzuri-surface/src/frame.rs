//! 頁の枠（便 g-frame）: header の部品・頁ごとの列と block の並び・見出しの語の鍵・class の名・mode と頁の URL。
//! 純粋な値だけを持ち、block の中身は持たない（中身は project の下の module ごとに置く）。
//! 並びと鍵と class は見本（docs/design/mock3 の index.html・ask.html・map.html・ui.css）に揃える。
//! 問いの頁（便 g-ask）: 頁の順は home・ask・map、header の質問の link に open の問いの数の印を付ける。
//! 抜けの検査の頁（便 g-gaps）: 頁の順は home・ask・map・gaps（見本の header と同じ 4 つ）。
//! 節点の頁（便 g-node）: query の page=node と id で開き、nav には出さない（nav の順と頁の一覧は 4 つのまま）。
//! 問いの頁の右の列（便 g-batch）: class side stack の列に batch と policy の 2 つの block。
//! header の「戻る」（行 h-wire）: HEADER の前の部品 BACK と、account board の窓へ戻る段の列（`back_steps`）。

use crate::mapview::encode;
use crate::project::{
    ask, askpage, batch, gaps, ledger, legend, map, next, node as node_block, nodearound, pipeline,
    policy, seat,
};

/// 1 つの block の枠（DOM の id・見出しの語の鍵・section の class）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    pub id: &'static str,
    pub heading: &'static str,
    pub class: &'static str,
}

/// block を縦に積む列（見本の `.stack`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub class: &'static str,
    pub blocks: Vec<Block>,
}

/// 頁（home・問い・地図・抜けの検査・節点）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageId {
    Home,
    Ask,
    Map,
    Gaps,
    /// 節点の頁（nav には出さない）。
    Node,
}

impl PageId {
    /// nav の頁の全部（nav の順・節点の頁は入れない）。
    pub const ALL: [PageId; 4] = [PageId::Home, PageId::Ask, PageId::Map, PageId::Gaps];

    pub fn id(self) -> &'static str {
        match self {
            PageId::Home => "home",
            PageId::Ask => "ask",
            PageId::Map => "map",
            PageId::Gaps => "gaps",
            PageId::Node => "node",
        }
    }

    /// URL の query の `page=` から頁を決める（無い・知らない値は home）。
    pub fn from_query(search: &str) -> Self {
        match param(search, "page") {
            Some("ask") => PageId::Ask,
            Some("map") => PageId::Map,
            Some("gaps") => PageId::Gaps,
            Some("node") => PageId::Node,
            _ => PageId::Home,
        }
    }
}

/// 1 つの頁の枠（頁の id・nav の語の鍵・列を包む class・列）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub id: PageId,
    pub heading: &'static str,
    pub class: &'static str,
    pub columns: Vec<Column>,
}

impl Page {
    /// block の id の並び（列の順・列の中の順）。
    pub fn block_ids(&self) -> Vec<&'static str> {
        self.columns
            .iter()
            .flat_map(|c| c.blocks.iter().map(|b| b.id))
            .collect()
    }
}

/// 列の class（見本の `.stack`）。
pub const STACK: &str = "stack";

/// 右の列の class（見本の ask.html の `aside.side.stack`）。
pub const SIDE: &str = "side stack";

/// home の頁: 次の一手・pipeline・台帳・凡例 ｜ orchestrator と口座（2 列の grid・見本の `.home`）。
pub fn home() -> Page {
    Page {
        id: PageId::Home,
        heading: "home",
        class: "home",
        columns: vec![
            Column {
                class: STACK,
                blocks: vec![next::BLOCK, pipeline::BLOCK, ledger::BLOCK, legend::BLOCK],
            },
            Column {
                class: STACK,
                blocks: vec![seat::BLOCK],
            },
        ],
    }
}

/// 問いの頁: 問いの card の列・これまでの決定 ｜ まとめて承認・全体への指示（見本の ask.html の `.ask` の 2 列）。
pub fn ask() -> Page {
    Page {
        id: PageId::Ask,
        heading: "questions",
        class: "ask",
        columns: vec![
            Column {
                class: STACK,
                blocks: vec![ask::BLOCK, askpage::BLOCK],
            },
            Column {
                class: SIDE,
                blocks: vec![batch::BLOCK, policy::BLOCK],
            },
        ],
    }
}

/// 地図の頁（枠と見出しだけ・中身は後の便）。
pub fn map() -> Page {
    Page {
        id: PageId::Map,
        heading: "map",
        class: STACK,
        columns: vec![Column {
            class: STACK,
            blocks: vec![map::BLOCK],
        }],
    }
}

/// 抜けの検査の頁（見本の gaps.html・block は gaps の 1 つ）。
pub fn gaps() -> Page {
    Page {
        id: PageId::Gaps,
        heading: "gaps",
        class: STACK,
        columns: vec![Column {
            class: STACK,
            blocks: vec![gaps::BLOCK],
        }],
    }
}

/// 節点の頁（見本の bead.html・block は頭と概要の node と、つながりの around の 2 つ）。
pub fn node() -> Page {
    Page {
        id: PageId::Node,
        heading: "nb_self",
        class: STACK,
        columns: vec![Column {
            class: STACK,
            blocks: vec![node_block::BLOCK, nodearound::BLOCK],
        }],
    }
}

pub fn page(id: PageId) -> Page {
    match id {
        PageId::Home => home(),
        PageId::Ask => ask(),
        PageId::Map => map(),
        PageId::Gaps => gaps(),
        PageId::Node => node(),
    }
}

/// 頁の全部（nav の順）。
pub fn pages() -> [Page; 4] {
    PageId::ALL.map(page)
}

/// header の 1 つの部品（役・語の鍵・class・中の語の鍵）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderPart {
    pub part: &'static str,
    pub key: &'static str,
    pub class: &'static str,
    pub items: &'static [&'static str],
}

/// header の部品（題・頁の link・最終更新・mode の切り替え・この順）。
pub const HEADER: [HeaderPart; 4] = [
    HeaderPart {
        part: "brand",
        key: "project",
        class: "brand",
        items: &[],
    },
    HeaderPart {
        part: "nav",
        key: "dashboard",
        class: "nav",
        items: &["home", "questions", "map", "gaps"],
    },
    HeaderPart {
        part: "updated",
        key: "last_record",
        class: "chip num",
        items: &[],
    },
    HeaderPart {
        part: "mode",
        key: "mode",
        class: "seg",
        items: &["beginner", "expert"],
    },
];

/// header の「戻る」の部品（HEADER の前に描く・行 h-wire・見本の ui.js の topHTML の `.backwrap` の中の `.upto`）。
pub const BACK: HeaderPart = HeaderPart {
    part: "back",
    key: "acct_back",
    class: "upto",
    items: &[],
};

/// 「戻る」の部品を包む span の class（見本の `.backwrap`）。
pub const BACK_WRAP: &str = "backwrap";

/// account board の窓を新しく開くときの URL（同じ index.html の query の board が account）。
pub const ACCOUNT_URL: &str = "?board=account";

/// 「戻る」の 1 段（見本の ui.js の backToBoard）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackStep {
    /// account board の窓（`tz-account`）を前面へ。
    Front,
    /// account board を新しい窓で開く（URL の字）。
    OpenNew(&'static str),
    /// 自分の窓を閉じる。
    CloseSelf,
}

/// 「戻る」の段の列: account board の窓が在れば前面へ、無ければ `ACCOUNT_URL` を新しい窓で開き、
/// どちらの後も自分の窓を閉じる。
pub fn back_steps(account_open: bool) -> Vec<BackStep> {
    let first = if account_open {
        BackStep::Front
    } else {
        BackStep::OpenNew(ACCOUNT_URL)
    };
    vec![first, BackStep::CloseSelf]
}

/// 題の字（project の名）。
pub const BRAND: &str = "tsuzuri";

/// 表示の型（初心者 = 説明の「?」を出す・経験者 = 型の名を出し「?」を出さない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Beginner,
    Expert,
}

impl Mode {
    pub const ALL: [Mode; 2] = [Mode::Beginner, Mode::Expert];

    /// URL の query の `mode=` から決める（無い・知らない値は初心者）。
    pub fn from_query(search: &str) -> Self {
        match param(search, "mode") {
            Some("expert") => Mode::Expert,
            _ => Mode::Beginner,
        }
    }

    /// URL と語彙の鍵に使う名（`beginner` / `expert`）。
    pub fn key(self) -> &'static str {
        match self {
            Mode::Beginner => "beginner",
            Mode::Expert => "expert",
        }
    }

    /// body の class（見本の ui.css は `body.mode-beginner .q` で「?」を出す）。
    pub fn body_class(self) -> &'static str {
        match self {
            Mode::Beginner => "mode-beginner",
            Mode::Expert => "mode-expert",
        }
    }
}

/// nav の 1 つの link（語の鍵・行き先の頁・class）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavLink {
    pub key: &'static str,
    pub page: PageId,
    /// 今の頁なら `on`（見本の `.nav a.on`）。
    pub class: &'static str,
    /// 数の印の class（問いの頁の link だけ `badge`・ほかは空）。
    pub badge: &'static str,
}

/// header の質問の link の数の印の class（見本の `.nav .badge`）。
pub const BADGE: &str = "badge";

/// nav の link（header の nav の部品の順）。
pub fn nav_links(current: PageId) -> Vec<NavLink> {
    pages()
        .iter()
        .map(|p| NavLink {
            key: p.heading,
            page: p.id,
            class: if p.id == current { "on" } else { "" },
            badge: if p.id == PageId::Ask { BADGE } else { "" },
        })
        .collect()
}

/// 質問の link の数の印の字（open の問いの数・数が読めないときと 0 のときは出さない）。
pub fn badge(open: Option<usize>) -> Option<String> {
    open.filter(|n| *n > 0).map(|n| n.to_string())
}

/// 頁への link（同じ path の query だけ・mode を URL に残す）。
pub fn href(page: PageId, mode: Mode) -> String {
    match page {
        PageId::Home => format!("?mode={}", mode.key()),
        PageId::Ask => format!("?page=ask&mode={}", mode.key()),
        PageId::Map => format!("?page=map&mode={}", mode.key()),
        PageId::Gaps => format!("?page=gaps&mode={}", mode.key()),
        PageId::Node => format!("?page=node&mode={}", mode.key()),
    }
}

/// 節点の頁への link（節点の id は `%XX` にする・mode を URL に残す）。
pub fn node_href(id: &str, mode: Mode) -> String {
    format!("?page=node&id={}&mode={}", encode(id), mode.key())
}

/// query の 1 つの値（`?a=1&b=2` の形・無ければ None）。
pub fn param<'a>(search: &'a str, key: &str) -> Option<&'a str> {
    search
        .trim_start_matches('?')
        .split('&')
        .filter_map(|kv| kv.split_once('=').or(Some((kv, ""))))
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v)
}

/// query の 1 つの値を置き換える（無ければ末尾に足す・ほかの値と順はそのまま）。
pub fn with_param(search: &str, key: &str, value: &str) -> String {
    let mut found = false;
    let mut parts: Vec<String> = search
        .trim_start_matches('?')
        .split('&')
        .filter(|kv| !kv.is_empty())
        .map(|kv| {
            let k = kv.split_once('=').map_or(kv, |(k, _)| k);
            if k == key && !found {
                found = true;
                format!("{key}={value}")
            } else {
                kv.to_string()
            }
        })
        .collect();
    if !found {
        parts.push(format!("{key}={value}"));
    }
    format!("?{}", parts.join("&"))
}

/// 枠の値の JSON の字（snapshot の file と 1 字も違わないことを歯が見る）。
pub fn snapshot() -> String {
    let mut out = String::from("{\n  \"header\": [\n");
    // 「戻る」の部品は HEADER の前。
    let header: Vec<String> = [&BACK]
        .into_iter()
        .chain(HEADER.iter())
        .map(|h| {
            format!(
                "    {{\"part\": {}, \"key\": {}, \"class\": {}, \"items\": [{}]}}",
                quote(h.part),
                quote(h.key),
                quote(h.class),
                list(h.items)
            )
        })
        .collect();
    out.push_str(&header.join(",\n"));
    out.push_str("\n  ],\n  \"pages\": [\n");
    // nav の 4 つの頁の後に、nav に出さない節点の頁。
    let pages: Vec<String> = pages().iter().chain([&node()]).map(page_json).collect();
    out.push_str(&pages.join(",\n"));
    out.push_str("\n  ]\n}\n");
    out
}

fn page_json(page: &Page) -> String {
    let columns: Vec<String> = page
        .columns
        .iter()
        .map(|c| {
            let blocks: Vec<String> = c
                .blocks
                .iter()
                .map(|b| {
                    format!(
                        "            {{\"id\": {}, \"heading\": {}, \"class\": {}}}",
                        quote(b.id),
                        quote(b.heading),
                        quote(b.class)
                    )
                })
                .collect();
            format!(
                "        {{\n          \"class\": {},\n          \"blocks\": [\n{}\n          ]\n        }}",
                quote(c.class),
                blocks.join(",\n")
            )
        })
        .collect();
    format!(
        "    {{\n      \"id\": {},\n      \"heading\": {},\n      \"class\": {},\n      \"columns\": [\n{}\n      ]\n    }}",
        quote(page.id.id()),
        quote(page.heading),
        quote(page.class),
        columns.join(",\n")
    )
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn list(items: &[&str]) -> String {
    items
        .iter()
        .map(|s| quote(s))
        .collect::<Vec<_>>()
        .join(", ")
}
