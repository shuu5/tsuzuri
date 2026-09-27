//! 頁の枠（便 g-frame）: header の部品・頁ごとの列と block の並び・見出しの語の鍵・class の名・mode と頁の URL。
//! 純粋な値だけを持ち、block の中身は持たない（中身は project の下の module ごとに置く）。
//! 並びと鍵と class は見本（docs/design/mock3 の index.html・map.html・ui.css）に揃える。

use crate::project::{ask, ledger, legend, map, next, pipeline, seat};

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

/// 頁（home と地図）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageId {
    Home,
    Map,
}

impl PageId {
    pub fn id(self) -> &'static str {
        match self {
            PageId::Home => "home",
            PageId::Map => "map",
        }
    }

    /// URL の query の `page=` から頁を決める（無い・知らない値は home）。
    pub fn from_query(search: &str) -> Self {
        match param(search, "page") {
            Some("map") => PageId::Map,
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

/// home の頁: 次の一手・問いの一覧・pipeline ｜ orchestrator と口座・台帳・凡例（2 列の grid・見本の `.home`）。
pub fn home() -> Page {
    Page {
        id: PageId::Home,
        heading: "home",
        class: "home",
        columns: vec![
            Column {
                class: STACK,
                blocks: vec![next::BLOCK, ask::BLOCK, pipeline::BLOCK],
            },
            Column {
                class: STACK,
                blocks: vec![seat::BLOCK, ledger::BLOCK, legend::BLOCK],
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

pub fn page(id: PageId) -> Page {
    match id {
        PageId::Home => home(),
        PageId::Map => map(),
    }
}

/// 頁の全部（nav の順）。
pub fn pages() -> [Page; 2] {
    [home(), map()]
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
        items: &["home", "map"],
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
}

/// nav の link（header の nav の部品の順）。
pub fn nav_links(current: PageId) -> Vec<NavLink> {
    pages()
        .iter()
        .map(|p| NavLink {
            key: p.heading,
            page: p.id,
            class: if p.id == current { "on" } else { "" },
        })
        .collect()
}

/// 頁への link（同じ path の query だけ・mode を URL に残す）。
pub fn href(page: PageId, mode: Mode) -> String {
    match page {
        PageId::Home => format!("?mode={}", mode.key()),
        PageId::Map => format!("?page=map&mode={}", mode.key()),
    }
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
    let header: Vec<String> = HEADER
        .iter()
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
    let pages: Vec<String> = pages().iter().map(page_json).collect();
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
