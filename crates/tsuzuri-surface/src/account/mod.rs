//! account board の頁の枠（便 h-frame）: 入口の振り分け・3 つの tab と URL・header の部品・tab ごとの block の並び・
//! 数の印・読みの結果から block の中身の有無を決める関数。純粋な値と関数だけを持ち、DOM は board（wasm の target のときだけ）。
//! 見本は docs/design/mock3/account/index.html と ui.js の topHTML（PAGE_KIND が account の枝）と acctTabHref。
//! block ごとに 1 つの module（home・windows・session・ledger・projects）が描く。この便は中身を描かず、
//! 読めたら中身はまだ無いの字を、読めない・まだ読んでいない・本文が電文として読めないときは測れていないと理由の 1 行を出す。
//! 口は 1 つ（契約の型の crate の account の module の PATH）で、全部の block が net の同じ signal を分け合う。

use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::board::{NextMove, Reading};
use tsuzuri_contract::seat::SeatState;
use tsuzuri_contract::wire;

use crate::frame::{Block, Mode, STACK, param};
use crate::project::{Body, NO_CONTENT, NOT_READ};
use crate::view::Fetched;

pub mod heartbeat;
pub mod home;
pub mod ledger;
pub mod projects;
pub mod session;
pub mod windows;

#[cfg(target_arch = "wasm32")]
pub mod board;

/// 読みの口の path（契約の型の crate の定数・面の code に字を直に書かない）。
pub use tsuzuri_contract::account::PATH;

/// URL の query の board が account のときだけ account board の入口を選ぶ（ほかは project board のまま）。
pub fn selects(search: &str) -> bool {
    param(search, "board") == Some("account")
}

/// account board の tab（見本の ACCT_TABS の順）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Home,
    Session,
    Projects,
}

impl Tab {
    pub const ALL: [Tab; 3] = [Tab::Home, Tab::Session, Tab::Projects];

    /// URL の `tab=` の字。
    pub fn id(self) -> &'static str {
        match self {
            Tab::Home => "home",
            Tab::Session => "session",
            Tab::Projects => "projects",
        }
    }

    /// tab の link の語の鍵。
    pub fn key(self) -> &'static str {
        match self {
            Tab::Home => "tab_home",
            Tab::Session => "tab_session",
            Tab::Projects => "tab_projects",
        }
    }

    /// URL の query の `tab=` から決める（無い・知らない値は home）。
    pub fn from_query(search: &str) -> Self {
        match param(search, "tab") {
            Some("session") => Tab::Session,
            Some("projects") => Tab::Projects,
            _ => Tab::Home,
        }
    }
}

/// tab への link（account board の入口と tab と mode を URL に残す）。
pub fn tab_href(tab: Tab, mode: Mode) -> String {
    format!("?board=account&tab={}&mode={}", tab.id(), mode.key())
}

/// header の 1 つの部品（役・語の鍵・class・中の語の鍵・数の印の class）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderPart {
    pub part: &'static str,
    pub key: &'static str,
    pub class: &'static str,
    pub items: &'static [&'static str],
    pub badge: &'static str,
}

/// header の class（見本の `header.top.vessel`）。
pub const TOP: &str = "top vessel";

/// tab の link の数の印の class（見本の `.badge.num`）。
pub const BADGE: &str = "badge num";

/// header の部品（題・tab の link・mode の切り替え・この順）。
pub const HEADER: [HeaderPart; 3] = [
    HeaderPart {
        part: "brand",
        key: "acct_board",
        class: "brand",
        items: &[],
        badge: "",
    },
    HeaderPart {
        part: "nav",
        key: "dashboard",
        class: "nav",
        items: &["tab_home", "tab_session", "tab_projects"],
        badge: BADGE,
    },
    HeaderPart {
        part: "mode",
        key: "mode",
        class: "seg",
        items: &["beginner", "expert"],
        badge: "",
    },
];

/// 題の字（見本の `.name`）。
pub const BRAND: &str = "器";

/// 最終の記録の chip（header の部品に数えず、tab の link の後に置く・語の鍵と class）。
pub const UPDATED_KEY: &str = "last_record";
pub const UPDATED_CLASS: &str = "chip num";

/// tab の 1 つの link（行き先の tab・語の鍵・今の tab なら `on`・数の印の class）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabLink {
    pub tab: Tab,
    pub key: &'static str,
    pub class: &'static str,
    pub badge: &'static str,
}

/// tab の link（header の nav の部品の順）。
pub fn tab_links(current: Tab) -> Vec<TabLink> {
    Tab::ALL
        .into_iter()
        .map(|tab| TabLink {
            tab,
            key: tab.key(),
            class: if tab == current { "on" } else { "" },
            badge: BADGE,
        })
        .collect()
}

/// block を並べる 1 段（段の class と block の並び）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub class: &'static str,
    pub blocks: Vec<Block>,
}

/// 1 つの tab の枠（tab と段）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabPage {
    pub tab: Tab,
    pub rows: Vec<Row>,
}

impl TabPage {
    /// block の id の並び（段の順・段の中の順）。
    pub fn block_ids(&self) -> Vec<&'static str> {
        self.rows
            .iter()
            .flat_map(|r| r.blocks.iter().map(|b| b.id))
            .collect()
    }
}

/// 1 段目の class（見本の `.hgrid`・次の一手と開いている窓を横に並べる）。
pub const HGRID: &str = "hgrid";

/// tab の枠（home: 次の一手と開いている窓 ｜ 群の枠・口座 × 窓・移動、session: session と台帳、projects: 各 project）。
pub fn page(tab: Tab) -> TabPage {
    let rows = match tab {
        Tab::Home => vec![
            Row {
                class: HGRID,
                blocks: vec![home::NXALL, windows::BLOCK],
            },
            Row {
                class: STACK,
                blocks: vec![home::GROUPS, home::ALLOWANCE, home::MOVES],
            },
        ],
        Tab::Session => vec![Row {
            class: STACK,
            blocks: vec![session::BLOCK, ledger::BLOCK],
        }],
        Tab::Projects => vec![Row {
            class: STACK,
            blocks: vec![projects::BLOCK],
        }],
    };
    TabPage { tab, rows }
}

/// tab の枠の全部（tab の順）。
pub fn pages() -> [TabPage; 3] {
    Tab::ALL.map(page)
}

/// 口が読めないときの理由。
pub const UNREAD: &str = "account board の口が読めない（server にまだ無い・届かない・知らせが切れた）ので今の中身が正しいと言えない";

/// 本文が電文として読めないときの理由。
pub const BAD_BODY: &str = "account board の口の本文が電文（AccountDoc）として読めない";

/// 口の本文を電文に読む（まだ読んでいない・読めない・電文が読めないは理由）。
pub fn doc(fetched: &Fetched) -> Result<AccountDoc, &'static str> {
    match fetched {
        Fetched::NotRead => Err(NOT_READ),
        Fetched::Failed => Err(UNREAD),
        Fetched::Body(text) => wire::decode::<AccountDoc>(text).map_err(|_| BAD_BODY),
    }
}

/// 中身の関数がまだ無い block の中身: 読めないときは理由、読めたら中身はまだ無い（どちらも測れていない）。
pub fn pending(fetched: &Fetched) -> Body<()> {
    Body::Unmeasured(doc(fetched).err().unwrap_or(NO_CONTENT))
}

/// 要対応の project の数（次の一手が読めて lead が なし でない・読めない project は数えない）。
pub fn need_count(doc: &AccountDoc) -> usize {
    doc.projects
        .iter()
        .filter(|p| matches!(&p.next, Reading::Known(s) if s.lead != NextMove::Nothing))
        .count()
}

/// 止まっている session の数（状態が限度か無応答の行）。
pub fn stuck_count(doc: &AccountDoc) -> usize {
    doc.sessions
        .iter()
        .filter(|s| matches!(s.state, SeatState::Limit | SeatState::Silent))
        .count()
}

/// 各 project の tab の印の字（見本の BAD の `!`）。
pub const NEED_MARK: &str = "!";

/// tab の数の印の字（0 は出さない・各 project は home の数が 1 以上なら `!`）。
pub fn badge(tab: Tab, doc: &AccountDoc) -> Option<String> {
    let count = |n: usize| (n > 0).then(|| n.to_string());
    match tab {
        Tab::Home => count(need_count(doc)),
        Tab::Session => count(stuck_count(doc)),
        Tab::Projects => (need_count(doc) > 0).then(|| NEED_MARK.to_string()),
    }
}

/// 枠の値の JSON の字（snapshot の file と 1 字も違わないことを歯が見る）。
pub fn snapshot() -> String {
    let header: Vec<String> = HEADER
        .iter()
        .map(|h| {
            format!(
                "    {{\"part\": {}, \"key\": {}, \"class\": {}, \"items\": [{}], \"badge\": {}}}",
                quote(h.part),
                quote(h.key),
                quote(h.class),
                list(h.items),
                quote(h.badge)
            )
        })
        .collect();
    let tabs: Vec<String> = pages().iter().map(tab_json).collect();
    format!(
        "{{\n  \"class\": {},\n  \"brand\": {},\n  \"header\": [\n{}\n  ],\n  \"tabs\": [\n{}\n  ]\n}}\n",
        quote(TOP),
        quote(BRAND),
        header.join(",\n"),
        tabs.join(",\n")
    )
}

fn tab_json(page: &TabPage) -> String {
    let rows: Vec<String> = page
        .rows
        .iter()
        .map(|r| {
            let blocks: Vec<String> = r
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
                quote(r.class),
                blocks.join(",\n")
            )
        })
        .collect();
    format!(
        "    {{\n      \"id\": {},\n      \"key\": {},\n      \"rows\": [\n{}\n      ]\n    }}",
        quote(page.tab.id()),
        quote(page.tab.key()),
        rows.join(",\n")
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

#[cfg(target_arch = "wasm32")]
pub use dom::{fold_view, section_view};

/// block に共通の DOM（wasm の target のときだけ）: 口を読み、中身の有無を出す。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;

    use super::{PATH, pending};
    use crate::frame::Block;
    use crate::project::{body_view, section};
    use crate::widgets::help::h2;

    /// 中身の無い block の section（口は頁に 1 本・同じ path の read は同じ signal を返す）。
    pub fn section_view(block: Block) -> AnyView {
        let fetched = crate::net::read(PATH);
        let body = move || body_view(fetched.with(pending));
        section(block, ().into_any(), body.into_any())
    }

    /// 畳める段の block（見本の `details.panel.fold.mvp`・見出しは summary に置く）。
    pub fn fold_view(block: Block) -> AnyView {
        let fetched = crate::net::read(PATH);
        let body = move || body_view(fetched.with(pending));
        view! {
            <details class=block.class id=block.id>
                <summary>{h2(block.heading)}</summary>
                {body}
            </details>
        }
        .into_any()
    }
}
