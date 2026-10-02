//! 行 hs-pages の歯（接頭辞 hspage_・判断の記録 ADR-13）: 頁は src/pages の下に 1 頁 1 file・列挙 PageId は
//! 組み立ての script が生成・snapshot は頁ごとの file・
//! href と from_query の往復・frame.rs と board.rs に頁の変種の名が無い（header の file と nav の歯は行 g-dead-sweep-b で消した）。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use tsuzuri_surface::frame::{self, Mode, PageId};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// dir の下の file の名で、拡張子 `ext` を持つものの名（拡張子を除く・名の順）。`skip` の名は除く。
fn dir_names(rel: &str, ext: &str, skip: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(crate_dir().join(rel))
        .unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n != skip)
        .filter_map(|n| n.strip_suffix(ext).map(str::to_string))
        .collect();
    names.sort();
    names
}

/// 着地済みの 2 つの頁（変種と id・地図の頁は行 m-map-page で、質問の頁と抜けの検査の頁は行 g-one-screen-a で消した）。
const LANDED: [(PageId, &str); 2] = [(PageId::Home, "home"), (PageId::Node, "node")];

/// (1) 生成した PageId の ALL の id の列は src/pages の下の mod.rs でない .rs の file の名（名の順）と同じ・
/// 着地済みの 4 つが全部在り、変種の名は Home・Ask・Gaps・Node のまま。
#[test]
fn hspage_ids_follow_dir() {
    let files = dir_names("src/pages", ".rs", "mod.rs");
    let ids: Vec<String> = PageId::ALL.iter().map(|p| p.id().to_string()).collect();
    assert_eq!(ids, files, "ALL の id の列と src/pages の file の名");
    for (page, id) in LANDED {
        assert_eq!(page.id(), id);
        assert!(PageId::ALL.contains(&page), "{id} が ALL に無い");
    }
    // tsuzuri_surface::pages の PageId と同じ型（frame は再公開するだけ）。
    let same: tsuzuri_surface::pages::PageId = PageId::Home;
    assert_eq!(same, tsuzuri_surface::frame::PageId::Home);
}

/// (2) 各頁の page_snapshot は tests/snapshots/pages の同じ id の file と 1 字も違わず、file の名の集合は ALL の id の集合。
#[test]
fn hspage_snaps_match_files() {
    let files: BTreeSet<String> = dir_names("tests/snapshots/pages", ".json", "")
        .into_iter()
        .collect();
    let ids: BTreeSet<String> = PageId::ALL.iter().map(|p| p.id().to_string()).collect();
    assert_eq!(files, ids, "snapshot の file の名と ALL の id");
    for page in PageId::ALL {
        let got = frame::page_snapshot(page);
        let want = read(&format!("tests/snapshots/pages/{}.json", page.id()));
        assert!(got == want, "{} の頁の値が snapshot と違う。今の値:\n{got}", page.id());
    }
}

/// (5) 全部の頁と 2 つの mode で href を from_query に渡すと同じ頁と mode に戻り、home の href は mode だけ・
/// 知らない page の字と page の無い字は home。
#[test]
fn hspage_href_round_trip() {
    for mode in Mode::ALL {
        for page in PageId::ALL {
            let href = frame::href(page, mode);
            assert_eq!(PageId::from_query(&href), page, "{href}");
            assert_eq!(Mode::from_query(&href), mode, "{href}");
        }
        assert_eq!(frame::href(PageId::Home, mode), format!("?mode={}", mode.key()));
    }
    assert_eq!(frame::href(PageId::Node, Mode::Expert), "?page=node&mode=expert");
    assert_eq!(PageId::from_query("?page=bogus&mode=expert"), PageId::Home);
    assert_eq!(PageId::from_query("?mode=expert"), PageId::Home);
    assert_eq!(PageId::from_query(""), PageId::Home);
}

/// (6) frame.rs と board.rs の字に Ask・Map・Gaps・Node の変種の名と fn nav_icon が無い。
#[test]
fn hspage_no_variant_words() {
    for file in ["src/frame.rs", "src/board.rs"] {
        let text = read(file);
        for word in [
            "PageId::Ask",
            "PageId::Map",
            "PageId::Gaps",
            "PageId::Node",
            "fn nav_icon",
        ] {
            assert!(!text.contains(word), "{file} に {word} が在る");
        }
    }
}

/// 着地済みの行とこの文書の行の verify の filter の語。
const FILTERS: [&str; 49] = [
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "acctframe_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "askcard_",
    "batchpanel_",
    "board_min_",
    "contract_form_",
    "frame_",
    "gapspage_",
    "gquestion_",
    "graph_",
    "gview_",
    "hook_",
    "ledgerblock_",
    "mapgraph_",
    "mapview_",
    "nextstep_",
    "nodepage_",
    "parts_",
    "pipe_",
    "project_",
    "question_",
    "seatblock_",
    "seatcard_",
    "server_",
    "skeleton_",
    "stage_",
    "stats_",
    "steady_",
    "topbar_",
    "tz_",
    "hbproc_",
    "hbconf_",
    "hbroute_",
    "hbpost_",
    "hsblock_",
    "hspage_",
    "hsderive_",
];

/// (9) この file の歯の名はどれも hspage_ で始まり、残りの字は filter の語を含まない。
#[test]
fn hspage_own_names_clean() {
    let text = read("tests/hspage.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == "#[test]")
        .map(|(i, _)| {
            lines[i + 1..]
                .iter()
                .find_map(|l| l.strip_prefix("fn "))
                .and_then(|rest| rest.split('(').next())
                .expect("test の属性の後の fn")
        })
        .collect();
    assert!(names.len() >= 5, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("hspage_")
            .unwrap_or_else(|| panic!("{name} が hspage_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
