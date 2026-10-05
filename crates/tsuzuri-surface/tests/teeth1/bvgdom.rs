//! 行 m-map-graph の歯（接頭辞 bvgdom_・判断の記録 ADR-27）: 地図のグラフの面の DOM の file とその歯の file が無く、
//! 面の src が眺めの口の字を持たず、グラフの module には近傍の図と節点の頁と節点の card が引く項だけが残る。
#![cfg(test)]

use crate::common::{crate_dir, read, sources};

/// file の頭の段の pub の項の名（行の頭が `pub const`・`pub fn`・`pub struct`・`pub enum`・`pub mod` の行・file の順）。
fn pub_items(rel: &str) -> Vec<String> {
    read(rel)
        .lines()
        .filter_map(|l| {
            [
                "pub const ",
                "pub fn ",
                "pub struct ",
                "pub enum ",
                "pub mod ",
            ]
            .iter()
            .find_map(|head| l.strip_prefix(head))
        })
        .map(|rest| {
            rest.chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect()
        })
        .collect()
}

/// (1) グラフの面の DOM の file とその歯の file が無く、graph.rs は DOM の module を宣言も再公開もしない。
#[test]
fn bvgdom_file_gone() {
    for rel in [
        "src/mapview/graph/dom.rs",
        "tests/graphnav.rs",
        "tests/mtips.rs",
    ] {
        assert!(!crate_dir().join(rel).exists(), "{rel} が在る");
    }
    let host = read("src/mapview/graph.rs");
    for word in ["mod dom", "dom::view"] {
        assert!(!host.contains(word), "graph.rs に {word} が在る");
    }
}

/// (2) 面の src のどの file も眺めの口の字を持たず、graph.rs は口の読み（PATH・REASON・doc）を持たない。
#[test]
fn bvgdom_no_view_path() {
    for (path, text) in sources() {
        assert!(
            !text.contains("/api/graph/view"),
            "{} に /api/graph/view が在る",
            path.display()
        );
    }
    let host = read("src/mapview/graph.rs");
    for word in ["pub const PATH", "pub const REASON", "pub fn doc("] {
        assert!(!host.contains(word), "graph.rs に {word} が在る");
    }
}

/// (3) graph.rs の pub の項は近傍の図と節点の頁と節点の card が引く 33 の名だけ・fold.rs は 2 つの名だけ・
/// band.rs は読めなかった出所の帯の理由の関数を持たない。
#[test]
fn bvgdom_items_exact() {
    let want = [
        "NODE_H",
        "Pos",
        "Lane",
        "BandRow",
        "LineStyle",
        "line_style",
        "curve",
        "straight",
        "line_svg",
        "legend_line",
        "esc",
        "cut",
        "round_mark",
        "open_question",
        "closed",
        "border",
        "LEGEND_SHAPES",
        "LEGEND_BORDERS",
        "LEGEND_HOVER",
        "Legend",
        "legend",
        "edge_term",
        "HUB_DEGREE",
        "LIT_MAX",
        "DEPTH",
        "Side",
        "Highlight",
        "degrees",
        "highlight",
        "PinBar",
        "PinAction",
        "pin_next",
        "fold",
    ];
    assert_eq!(want.len(), 33);
    assert_eq!(pub_items("src/mapview/graph.rs"), want);
    assert_eq!(
        pub_items("src/mapview/graph/fold.rs"),
        ["fold_key", "plain_title"]
    );
    assert!(!read("src/mapview/band.rs").contains("fn unread_reason("));
}
