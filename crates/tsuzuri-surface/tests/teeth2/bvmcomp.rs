//! 行 m-map-compact の歯（接頭辞 bvmcomp_・判断の記録 ADR-27）: 地図の圧縮の面と一覧の面の file が無く、src がその
//! module を引かず、節点の card の要約なしと状態なしの字は widgets の nodecard が今の字のまま持つ。
#![cfg(test)]

use std::path::{Path, PathBuf};

use tsuzuri_surface::widgets::nodecard::{NO_GIST, NO_STATE};

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

/// (1) 圧縮の面と一覧の面の file が無く、mapview の mod.rs はその module を宣言せず、src のどの file もその path を引かない。
#[test]
fn bvmcomp_files_gone() {
    for rel in ["src/mapview/compact.rs", "src/mapview/list.rs"] {
        assert!(!crate_dir().join(rel).exists(), "{rel} が在る");
    }
    let modrs = read("src/mapview/mod.rs");
    for word in ["pub mod compact;", "pub mod list;"] {
        assert!(!modrs.contains(word), "mapview/mod.rs に {word} が在る");
    }
    for (path, text) in sources() {
        for word in [
            "mapview::compact",
            "mapview::list",
            "super::compact",
            "super::list",
        ] {
            assert!(!text.contains(word), "{} に {word} が在る", path.display());
        }
    }
}

/// (2) 節点の card の要約なしと状態なしの字は widgets の nodecard の定数で、今の字のまま。
#[test]
fn bvmcomp_nodecard_keeps_words() {
    assert_eq!(NO_GIST, "要約なし");
    assert_eq!(NO_STATE, "状態なし");
    let text = read("src/widgets/nodecard.rs");
    for want in [
        "pub const NO_STATE: &str = \"状態なし\";",
        "pub const NO_GIST: &str = \"要約なし\";",
    ] {
        assert!(text.contains(want), "nodecard.rs に {want} が無い");
    }
}
