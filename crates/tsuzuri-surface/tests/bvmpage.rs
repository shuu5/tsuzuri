//! 行 m-map-page の歯（接頭辞 bvmpage_・判断の記録 ADR-27）: 地図の頁と地図の block と 6 面の切り替え（View）が無く、
//! グラフの口（/api/graph）の読みは kit の下の map に移って crate::project::map の path のまま使える。
#![cfg(test)]

use std::path::{Path, PathBuf};

use tsuzuri_surface::frame::{self, Mode, PageId};
use tsuzuri_surface::project::{Module, map};
use tsuzuri_surface::view::Fetched;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// src の下の .rs の file の全部（path の順）。
fn sources() -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("src を読む").flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let mut paths = Vec::new();
    walk(&crate_dir().join("src"), &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|p| {
            let text = std::fs::read_to_string(&p).expect("src の file");
            (p, text)
        })
        .collect()
}

/// (1) 頁は home・node の 2 つで（質問と抜けの検査の頁は行 g-one-screen-a で消した）、nav は home だけ・地図の頁の file と snapshot が無く、
/// 前の地図の URL は home に落ちる。
#[test]
fn bvmpage_no_map_page() {
    let ids: Vec<&str> = PageId::ALL.iter().map(|p| p.id()).collect();
    assert_eq!(ids, ["home", "node"]);
    assert_eq!(frame::nav_keys(), ["home"]);
    for page in frame::pages() {
        assert!(!page.block_ids().contains(&"map"), "{:?}", page.id);
    }
    for mode in Mode::ALL {
        let q = format!("?page=map&mode={}", mode.key());
        assert_eq!(PageId::from_query(&q), PageId::Home, "{q}");
    }
    for rel in ["src/pages/map.rs", "tests/snapshots/pages/map.json"] {
        assert!(!crate_dir().join(rel).exists(), "{rel} が在る");
    }
    assert!(!read("tests/snapshots/header.json").contains("\"map\""));
}

/// (2) block の列に地図が無く、グラフの口の読み（PATH・REASON・doc）は kit の下の map に在る。
#[test]
fn bvmpage_no_map_block() {
    let names: Vec<&str> = Module::ALL.iter().map(|m| m.name()).collect();
    assert!(!names.contains(&"map"), "{names:?}");
    assert!(!crate_dir().join("src/project/map.rs").exists());
    assert_eq!(map::PATH, "/api/graph");
    assert_eq!(map::doc(&Fetched::Failed), Err(map::REASON));
    let kit = read("src/kit.rs");
    assert!(
        kit.contains("\npub mod map;\n"),
        "kit.rs に pub mod map が無い"
    );
    let text = read("src/kit/map.rs");
    for want in [
        "pub const PATH: &str = \"/api/graph\";",
        "pub const REASON: &str =",
        "pub fn doc(fetched: &Fetched) -> Result<GraphDoc, &'static str> {",
    ] {
        assert!(text.contains(want), "kit/map.rs に {want} が無い");
    }
    for word in [
        "pub const BLOCK",
        "pub const PATHS",
        "pub const FOLDS",
        "pub fn body(",
    ] {
        assert!(!text.contains(word), "kit/map.rs に {word} が在る");
    }
}

/// (3) src に 6 面の切り替え（面の型 View・URL の query の view の鍵・面を切り替える関数・tab の矢印の key）の字が無い。
#[test]
fn bvmpage_no_view_param() {
    for (path, text) in sources() {
        for word in [
            "pub enum View",
            "VIEW_PARAM",
            "fn with_view(",
            "mapview::View",
            "View::",
            "tab_step",
        ] {
            assert!(!text.contains(word), "{} に {word} が在る", path.display());
        }
    }
}
