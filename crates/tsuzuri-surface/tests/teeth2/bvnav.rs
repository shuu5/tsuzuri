//! 行 g-one-screen-a の歯（接頭辞 bvnav_）: project board を 1 枚の画面にする（判断の記録 ADR-27 決定 (3)(4)(5)）。
//! 頁は home と節点の頁の 2 つで、home は台帳 open の一覧と pipeline の 2 つの面。頁の上は帯だけで tab と頁の切り替えを持たず、
//! 帯の印が開く窓の層を頁に 1 つ置く。消した質問の頁と抜けの検査の頁への link は、窓を開く home の頁の URL（query の win）。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::frame::{self, Mode, PageId};
use tsuzuri_surface::project::{ledger, pipeline};
use tsuzuri_surface::topbar::{WIN_PARAM, Win, win_href};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 窓の名は 7 つで query の値は重ならず、窓を開く URL は home の頁で mode と窓の名に読み戻る・無い値と知らない値は None。
#[test]
fn bvnav_win_query_round_trip() {
    let keys: Vec<&str> = Win::ALL.iter().map(|w| w.key()).collect();
    assert_eq!(
        keys,
        [
            "ask", "stalled", "notices", "seat", "gaps", "legend", "dest"
        ]
    );
    assert_eq!(WIN_PARAM, "win");
    for mode in Mode::ALL {
        for w in Win::ALL {
            let href = win_href(w, mode);
            assert_eq!(href, format!("?mode={}&win={}", mode.key(), w.key()));
            assert_eq!(Win::from_query(&href), Some(w), "{href}");
            assert_eq!(PageId::from_query(&href), PageId::Home, "{href}");
            assert_eq!(Mode::from_query(&href), mode, "{href}");
        }
    }
    assert_eq!(Win::from_query("?mode=expert"), None);
    assert_eq!(Win::from_query("?win=bogus"), None);
    assert_eq!(Win::from_query("?page=node&win=ask"), Some(Win::Ask));
}

/// 頁は home と節点の頁の 2 つ（質問の頁と抜けの検査の頁の file と snapshot は無く、前の URL は home に落ちる）・
/// home は台帳 open の一覧の面と pipeline の面の 2 列。
#[test]
fn bvnav_pages_home_and_node() {
    let ids: Vec<&str> = PageId::ALL.iter().map(|p| p.id()).collect();
    assert_eq!(ids, ["home", "node"]);
    for rel in [
        "src/pages/ask.rs",
        "src/pages/gaps.rs",
        "tests/snapshots/pages/ask.json",
        "tests/snapshots/pages/gaps.json",
    ] {
        assert!(!crate_dir().join(rel).exists(), "{rel} が在る");
    }
    for q in ["?page=ask", "?page=gaps&mode=expert"] {
        assert_eq!(PageId::from_query(q), PageId::Home, "{q}");
    }
    let home = frame::page(PageId::Home);
    assert_eq!(home.class, "one");
    let columns: Vec<(&str, Vec<&str>)> = home
        .columns
        .iter()
        .map(|c| (c.class, c.blocks.iter().map(|b| b.id).collect()))
        .collect();
    assert_eq!(
        columns,
        vec![
            ("pane", vec![ledger::BLOCK.id]),
            ("pane", vec![pipeline::BLOCK.id])
        ]
    );
}

/// App は帯（topbar の view）と窓の層を頁に 1 つずつ置き、URL の win の窓を開き、窓を開くと吹き出しを閉じ、
/// tab と頁の切り替えと mode の段を持たない。stylesheet は 2 つの面を帯の下の全部の高さに置く。
#[test]
fn bvnav_one_screen_wiring_text() {
    let board = read("src/board.rs");
    for want in [
        "{top(page, subject, mode, win)}",
        "<main class=\"page\">{page_view(page)}</main>",
        "{layer(win, move |w| wins::draw(w, win))}",
        "if let Some((w, id)) = win_of_href(&query) {",
        "win.open(w, false);",
        "if win.top().is_some()",
        "p.close();",
        "{topbar::view(bar)}",
        "pick: Callback::new(move |m: Mode| {",
    ] {
        assert!(board.contains(want), "board.rs に {want} が無い");
    }
    for gone in [
        "fn switch(",
        "fn go(",
        "popstate",
        "nav_links(",
        "fn mode_seg(",
        "fn nav_badge(",
        "<header class=\"top\">",
    ] {
        assert!(!board.contains(gone), "board.rs に {gone} が残る");
    }
    assert_eq!(board.matches("<PopLayer/>").count(), 1);
    let css = read("style.css");
    for rule in [
        "body:has(#bar) { padding-top: 52px; }",
        ".page:has(> .one) {",
        ".one {",
        ".one > .pane {",
        ".one > .pane > .panel { flex: 1 1 auto; min-height: 0; overflow: auto; }",
        ".freshw {",
    ] {
        assert!(css.contains(rule), "stylesheet に {rule} が無い");
    }
}
