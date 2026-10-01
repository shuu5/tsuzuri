//! 行 m-map-tree の歯（接頭辞 bvmtbl_・判断の記録 ADR-27）: 地図の表の面と 2 つの木の面の file が無く、src がその
//! module を引かず、辺の型の電文の字（edge_name）は近傍の around が今の字のまま持ち、2 つの面だけが引いた項が mod.rs に無い。
#![cfg(test)]

use std::path::{Path, PathBuf};

use tsuzuri_contract::graph::EdgeType;
use tsuzuri_contract::wire;
use tsuzuri_surface::mapview::around::edge_name;

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

/// (1) 表の面と 2 つの木の面の file とその歯の file が無く、mapview の mod.rs はその module を宣言せず、
/// src のどの file もその path を引かない。
#[test]
fn bvmtbl_files_gone() {
    for rel in [
        "src/mapview/table.rs",
        "src/mapview/tree.rs",
        "tests/mtree.rs",
        "tests/gmret.rs",
    ] {
        assert!(!crate_dir().join(rel).exists(), "{rel} が在る");
    }
    let modrs = read("src/mapview/mod.rs");
    for word in ["pub mod table;", "pub mod tree;"] {
        assert!(!modrs.contains(word), "mapview/mod.rs に {word} が在る");
    }
    for (path, text) in sources() {
        for word in [
            "mapview::table",
            "mapview::tree",
            "super::table",
            "super::tree",
        ] {
            assert!(!text.contains(word), "{} に {word} が在る", path.display());
        }
    }
}

/// (2) 辺の型の電文の字は近傍の around の edge_name が返し、型ごとに契約の電文の字と同じで、定義は src に 1 つだけ。
#[test]
fn bvmtbl_edge_name_moved() {
    for t in EdgeType::ALL {
        let want = wire::encode(&t).expect("辺の型の電文");
        let name = edge_name(t);
        assert!(!name.is_empty(), "{t:?}");
        assert_eq!(format!("\"{name}\""), want, "{t:?}");
    }
    assert_eq!(edge_name(EdgeType::InArticle), "in-article");
    assert_eq!(edge_name(EdgeType::RelationsArticles), "relations.articles");
    assert_eq!(edge_name(EdgeType::VerifyAc), "verify.ac");
    assert_eq!(edge_name(EdgeType::ParentChild), "parent-child");
    assert_eq!(edge_name(EdgeType::AmendedBy), "amended_by");

    assert!(read("src/mapview/around.rs").contains("pub fn edge_name(t: EdgeType) -> String {"));
    let defs: Vec<String> = sources()
        .into_iter()
        .filter(|(_, text)| text.contains("fn edge_name("))
        .map(|(p, _)| p.display().to_string())
        .collect();
    assert_eq!(defs.len(), 1, "{defs:?}");
}

/// (3) 表の面と木の面だけが引いた項（読めなかった出所の理由の行・廃止したノートの段・種類の引き）が mapview の mod.rs に無い。
#[test]
fn bvmtbl_orphans_gone() {
    let modrs = read("src/mapview/mod.rs");
    for word in [
        "fn unread_reasons(",
        "pub const RETIRED:",
        "RETIRED_UNREAD",
        "fn retired_notes(",
        "fn retired_unread(",
        "fn kinds_by_id(",
    ] {
        assert!(!modrs.contains(word), "mapview/mod.rs に {word} が在る");
    }
}
