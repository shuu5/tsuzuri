//! 便 h-frame の歯: account board の入口の振り分け・3 つの tab と URL・header と tab の枠の snapshot・
//! block ごとに 1 つの module・数の印・読めないときは測れていない・口の path は契約の型の定数・保存の口の字が無い。
#![cfg(test)]

use std::path::{Path, PathBuf};

use tsuzuri_contract::account::{self as contract_account, AccountDoc};
use tsuzuri_contract::board::{NextMove, Reading};
use tsuzuri_contract::seat::SeatState;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::{
    self, BADGE, HEADER, HGRID, NEED_MARK, Tab, badge, home, ledger, need_count, notices, projects,
    session, stage, stuck_count, tab_href, tab_links, windows,
};
use tsuzuri_surface::frame::{Mode, PageId, STACK};
use tsuzuri_surface::project::{Body, NO_CONTENT, NOT_READ};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture_text() -> String {
    read("../../tests/fixtures/account/acct-doc.json")
}

fn fixture() -> AccountDoc {
    wire::decode(&fixture_text()).expect("fixture が AccountDoc として読める")
}

/// src の account の下の .rs の file（名の順）。
fn account_sources() -> Vec<(PathBuf, String)> {
    let dir = crate_dir().join("src/account");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("src/account")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|p| {
            let text = std::fs::read_to_string(&p).expect("src の file");
            (p, text)
        })
        .collect()
}

fn file_name(p: &Path) -> String {
    p.file_name()
        .and_then(|n| n.to_str())
        .expect("file の名")
        .to_string()
}

/// (1) 入口は query の board が account のときだけ account board・ほかは project board の決め方のまま。
#[test]
fn acctframe_entry_selects_board_account_only() {
    for q in [
        "?board=account",
        "board=account",
        "?board=account&tab=session&mode=expert",
        "?mode=expert&board=account",
    ] {
        assert!(account::selects(q), "{q}");
    }
    for q in [
        "",
        "?",
        "?board",
        "?board=",
        "?board=project",
        "?board=accounts",
        "?board=Account",
        "?page=ask",
        "?tab=home&mode=expert",
    ] {
        assert!(!account::selects(q), "{q}");
    }
    // 偽のときの project board の頁と mode の決め方は変わらない。
    for (q, page) in [
        ("", PageId::Home),
        ("?page=ask", PageId::Ask),
        ("?board=project&page=gaps", PageId::Gaps),
        ("?page=node&id=FR1", PageId::Node),
    ] {
        assert!(!account::selects(q), "{q}");
        assert_eq!(PageId::from_query(q), page, "{q}");
    }
    assert_eq!(Mode::from_query("?board=project&mode=expert"), Mode::Expert);
    // 入口は振り分けの関数で account board か project board の mount を呼ぶ。
    let main = read("src/main.rs");
    for call in [
        "account::selects(&search)",
        "account::board::mount()",
        "tsuzuri_surface::board::mount()",
    ] {
        assert!(main.contains(call), "main.rs に {call} が無い");
    }
}

/// (2) tab は 3 つで、query の tab から決め（既定は home）、link は board と tab と mode を持つ。
#[test]
fn acctframe_tabs_and_links() {
    let ids: Vec<&str> = Tab::ALL.iter().map(|t| t.id()).collect();
    assert_eq!(ids, vec!["home", "session", "projects"]);
    let keys: Vec<&str> = Tab::ALL.iter().map(|t| t.key()).collect();
    assert_eq!(keys, vec!["tab_home", "tab_session", "tab_projects"]);
    assert_eq!(Tab::from_query("?tab=session"), Tab::Session);
    assert_eq!(
        Tab::from_query("?board=account&tab=projects&mode=expert"),
        Tab::Projects
    );
    for q in [
        "",
        "?board=account",
        "?tab=home",
        "?tab=",
        "?tab=Session",
        "?tab=bogus",
        "?tab=relation",
    ] {
        assert_eq!(Tab::from_query(q), Tab::Home, "{q}");
    }
    hrefs_and_links();
}

/// link の href は board と tab と mode を持ち、今の tab の link だけが on。
fn hrefs_and_links() {
    assert_eq!(
        tab_href(Tab::Home, Mode::Beginner),
        "?board=account&tab=home&mode=beginner"
    );
    assert_eq!(
        tab_href(Tab::Projects, Mode::Expert),
        "?board=account&tab=projects&mode=expert"
    );
    for tab in Tab::ALL {
        for mode in Mode::ALL {
            let href = tab_href(tab, mode);
            assert!(account::selects(&href), "{href}");
            assert_eq!(Tab::from_query(&href), tab, "{href}");
            assert_eq!(Mode::from_query(&href), mode, "{href}");
        }
    }
    let links = tab_links(Tab::Session);
    let got: Vec<(&str, &str, &str)> = links.iter().map(|l| (l.key, l.class, l.badge)).collect();
    assert_eq!(
        got,
        vec![
            ("tab_home", "", BADGE),
            ("tab_session", "on", BADGE),
            ("tab_projects", "", BADGE)
        ]
    );
    assert_eq!(BADGE, "badge num");
}

/// (3) header の部品は 題・tab の link・mode の順で、枠の値の字が snapshot の file と 1 字も違わない。
#[test]
fn acctframe_snapshot_matches_file() {
    let want = read("tests/snapshots/acctframe.json");
    let got = account::snapshot();
    assert!(
        got == want,
        "account board の枠の値が snapshot と違う。今の値:\n{got}"
    );
    let parts: Vec<(&str, &str)> = HEADER.iter().map(|h| (h.part, h.key)).collect();
    assert_eq!(
        parts,
        vec![
            ("brand", "acct_board"),
            ("nav", "dashboard"),
            ("mode", "mode")
        ]
    );
    assert_eq!(HEADER[1].items, ["tab_home", "tab_session", "tab_projects"]);
    assert_eq!(HEADER[1].badge, "badge num");
    assert_eq!(HEADER[2].class, "seg");
    assert_eq!(account::TOP, "top vessel");
}

/// (4) tab ごとの block と段の class・block ごとに 1 つの module。
#[test]
fn acctframe_blocks_per_tab_and_module() {
    let home_page = account::page(Tab::Home);
    let rows: Vec<(&str, Vec<&str>)> = home_page
        .rows
        .iter()
        .map(|r| (r.class, r.blocks.iter().map(|b| b.id).collect()))
        .collect();
    assert_eq!(
        rows,
        vec![
            (STACK, vec!["notices"]),
            (HGRID, vec!["nxall", "winsp"]),
            (STACK, vec!["groups", "allowance", "moves", "stage"])
        ]
    );
    assert_eq!(HGRID, "hgrid");
    assert_eq!(
        account::page(Tab::Session).block_ids(),
        vec!["sessions", "ledger"]
    );
    assert_eq!(account::page(Tab::Projects).block_ids(), vec!["ptab"]);
    for tab in Tab::ALL {
        assert_eq!(account::page(tab).tab, tab);
    }

    // block（id・見出しの語の鍵・class・描く module）。
    let table = [
        (notices::BLOCK, "notices", "notices", "panel", "notices"),
        (home::NXALL, "nxall", "next_all", "panel", "home"),
        (windows::BLOCK, "winsp", "open_windows", "panel", "windows"),
        (home::GROUPS, "groups", "group", "gtop", "home"),
        (home::ALLOWANCE, "allowance", "allowance", "panel", "home"),
        (home::MOVES, "moves", "moves", "panel fold mvp", "home"),
        (
            stage::BLOCK,
            "stage",
            "stage_target",
            "panel fold mvp",
            "stage",
        ),
        (session::BLOCK, "sessions", "sessions", "panel", "session"),
        (ledger::BLOCK, "ledger", "ledger_state", "panel", "ledger"),
        (projects::BLOCK, "ptab", "dashboards", "panel", "projects"),
    ];
    let all: Vec<_> = account::pages()
        .iter()
        .flat_map(|p| p.rows.iter().flat_map(|r| r.blocks.clone()))
        .collect();
    assert_eq!(all, table.iter().map(|t| t.0).collect::<Vec<_>>());
    for (block, id, heading, class, module) in table {
        assert_eq!((block.id, block.heading, block.class), (id, heading, class));
        let text = read(&format!("src/account/{module}.rs"));
        assert!(
            text.contains(&format!("id: \"{id}\"")),
            "{module}.rs が block {id} の枠を持たない"
        );
    }
    assert_eq!(
        home::BLOCKS.map(|b| b.id),
        ["nxall", "groups", "allowance", "moves"]
    );
    files_and_pure_mod();
}

/// account の下の file の並びと、枠の値（mod.rs）の純粋な部分が DOM を持たないこと。
fn files_and_pure_mod() {
    let files: Vec<String> = account_sources()
        .iter()
        .map(|(p, _)| file_name(p))
        .collect();
    assert_eq!(
        files,
        vec![
            "board.rs",
            "cards.rs",
            "heartbeat.rs",
            "home.rs",
            "ledger.rs",
            "mod.rs",
            "notices.rs",
            "projects.rs",
            "session.rs",
            "stage.rs",
            "windows.rs"
        ]
    );
    // 枠の値（mod.rs）は中身を持たない・DOM は board と各 module の wasm の枝だけ。
    let mod_src = read("src/account/mod.rs");
    let pure = mod_src
        .split("mod dom {")
        .next()
        .expect("mod.rs の純粋な部分");
    assert!(!pure.contains("view!"), "枠の値の部分が DOM を持つ");
}

/// 語の鍵は全部 vocab に在り、class は stylesheet に在る。
#[test]
fn acctframe_keys_in_vocab_and_classes_in_stylesheet() {
    let mut keys: Vec<&str> = vec![account::UPDATED_KEY, "not_yet", "st_unknown"];
    for part in HEADER {
        keys.push(part.key);
        keys.extend(part.items);
    }
    for page in account::pages() {
        keys.push(page.tab.key());
        for r in &page.rows {
            keys.extend(r.blocks.iter().map(|b| b.heading));
        }
    }
    for key in keys {
        let term = vocab()
            .term(key)
            .unwrap_or_else(|| panic!("鍵 {key} が vocab に無い"));
        assert!(!term.label.is_empty(), "鍵 {key} の語が空");
    }
    assert_eq!(vocab().label("tab_home"), "HOME");

    let css = read("style.css");
    let sources = account_sources();
    let mut classes: Vec<&str> = vec![account::TOP, account::UPDATED_CLASS, BADGE, "on"];
    for part in HEADER {
        classes.push(part.class);
    }
    for page in account::pages() {
        for r in &page.rows {
            classes.push(r.class);
            classes.extend(r.blocks.iter().map(|b| b.class));
        }
    }
    for (_, text) in &sources {
        let mut rest = text.as_str();
        while let Some(i) = rest.find("class=\"") {
            rest = &rest[i + 7..];
            let end = rest.find('"').expect("class の字の終わり");
            classes.push(&rest[..end]);
            rest = &rest[end..];
        }
    }
    for class in classes.iter().flat_map(|c| c.split_whitespace()) {
        assert!(
            css.contains(&format!(".{class}")),
            "stylesheet に class {class} が無い"
        );
    }
}

/// (5) 数の印: fixture では home が 1・session は出さない・各 project は「!」。
#[test]
fn acctframe_badges_on_fixture() {
    let doc = fixture();
    let names: Vec<&str> = doc.projects.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["proj-a", "proj-b", "proj-c"]);
    assert!(matches!(&doc.projects[0].next, Reading::Known(s) if s.lead == NextMove::Question));
    assert_eq!(doc.projects[1].next, Reading::Unknown);
    assert_eq!(doc.projects[2].next, Reading::Unknown);
    let states: Vec<SeatState> = doc.sessions.iter().map(|s| s.state).collect();
    assert_eq!(
        states,
        vec![
            SeatState::Run,
            SeatState::Run,
            SeatState::Wait,
            SeatState::Unknown
        ]
    );

    assert_eq!(need_count(&doc), 1);
    assert_eq!(stuck_count(&doc), 0);
    assert_eq!(badge(Tab::Home, &doc), Some("1".to_string()));
    assert_eq!(badge(Tab::Session, &doc), None);
    assert_eq!(badge(Tab::Projects, &doc), Some("!".to_string()));
    assert_eq!(NEED_MARK, "!");
}

/// (5) 数の印の決まり: lead が なし と Unknown は数えない・limit と silent を数える・0 は出さない。
#[test]
fn acctframe_badges_rules() {
    let base = fixture();

    // proj-a の lead を なし にすると要対応は 0（home と各 project の印を出さない）。
    let mut none = base.clone();
    if let Reading::Known(s) = &mut none.projects[0].next {
        s.lead = NextMove::Nothing;
    }
    assert_eq!(need_count(&none), 0);
    assert_eq!(badge(Tab::Home, &none), None);
    assert_eq!(badge(Tab::Projects, &none), None);

    // Unknown の project に読める次の一手を足すと数に入る（なし 以外の 6 種はどれも数える）。
    for lead in NextMove::ALL {
        let mut more = base.clone();
        more.projects[1].next = base.projects[0].next.clone();
        if let Reading::Known(s) = &mut more.projects[1].next {
            s.lead = lead;
        }
        let want = if lead == NextMove::Nothing { 1 } else { 2 };
        assert_eq!(need_count(&more), want, "{lead:?}");
        assert_eq!(badge(Tab::Home, &more), Some(want.to_string()));
        assert_eq!(badge(Tab::Projects, &more), Some("!".to_string()));
    }
    stuck_and_empty(base);
}

/// session の数の印と、project も session も無いときの印。
fn stuck_and_empty(base: AccountDoc) {
    // session の数は状態が limit か silent の行だけ。
    for (state, counted) in [
        (SeatState::Run, false),
        (SeatState::Wait, false),
        (SeatState::Limit, true),
        (SeatState::Silent, true),
        (SeatState::Unknown, false),
    ] {
        let mut d = base.clone();
        d.sessions[0].state = state;
        d.sessions[2].state = state;
        let n = if counted { 2 } else { 0 };
        assert_eq!(stuck_count(&d), n, "{state:?}");
        assert_eq!(
            badge(Tab::Session, &d),
            (n > 0).then(|| n.to_string()),
            "{state:?}"
        );
        // session の数は home と各 project の印に入らない。
        assert_eq!(badge(Tab::Home, &d), Some("1".to_string()));
    }

    let mut empty = base.clone();
    empty.projects.clear();
    empty.sessions.clear();
    for tab in Tab::ALL {
        assert_eq!(badge(tab, &empty), None, "{tab:?}");
    }
}

/// block の中身の関数。
type BodyFn = fn(&Fetched) -> Body<()>;

/// (6) 口が読めない・まだ読んでいない・本文が電文として読めないときは、どの block も測れていないと理由の 1 行。
/// 読めたら中身はまだ無いの字（この便は中身を描かない）。
#[test]
fn acctframe_blocks_unmeasured_until_read() {
    let bodies: [(&str, BodyFn); 8] = [
        ("nxall", home::body),
        ("winsp", windows::body),
        ("groups", home::body),
        ("allowance", home::body),
        ("moves", home::body),
        ("sessions", session::body),
        ("ledger", ledger::body),
        ("ptab", projects::body),
    ];
    let cases = [
        (Fetched::NotRead, NOT_READ),
        (Fetched::Failed, account::UNREAD),
        (Fetched::Body("{}".to_string()), account::BAD_BODY),
        (Fetched::Body("not json".to_string()), account::BAD_BODY),
        (Fetched::Body(String::new()), account::BAD_BODY),
        (Fetched::Body(fixture_text()), NO_CONTENT),
    ];
    for (fetched, want) in &cases {
        for (id, body) in bodies {
            let got = body(fetched);
            let Body::Unmeasured(reason) = got else {
                panic!("{id} が {fetched:?} で測れていないでない: {got:?}");
            };
            assert_eq!(reason, *want, "{id} が {fetched:?} で返す理由");
            assert!(!reason.trim().is_empty() && !reason.contains('\n'));
        }
    }
    assert!(account::doc(&Fetched::Body(fixture_text())).is_ok());
    let reasons = [NOT_READ, account::UNREAD, account::BAD_BODY, NO_CONTENT];
    for (i, a) in reasons.iter().enumerate() {
        for b in &reasons[i + 1..] {
            assert_ne!(a, b, "理由が重なる");
        }
    }
}

/// (7) 口の path は契約の型の crate の定数で、account の下の file に `/api/` の字が無い。
/// (8) account の下の file に保存の口（localStorage・sessionStorage・cookie）の字が無い。
#[test]
fn acctframe_path_from_contract_and_no_storage() {
    assert_eq!(account::PATH, contract_account::PATH);
    assert_eq!(account::PATH, "/api/account");
    let sources = account_sources();
    assert!(!sources.is_empty());
    for (path, text) in &sources {
        for word in ["/api/", "localStorage", "sessionStorage", "cookie"] {
            assert!(
                !text.contains(word),
                "{} に {word} の字が在る",
                path.display()
            );
        }
    }
    // 読みは block ごとに同じ口を net の read に渡す（同じ path は 1 つの signal を分ける）。
    let mod_src = read("src/account/mod.rs");
    assert!(mod_src.contains("crate::net::read(PATH)"));
    assert!(mod_src.contains("pub use tsuzuri_contract::account::PATH;"));
}
