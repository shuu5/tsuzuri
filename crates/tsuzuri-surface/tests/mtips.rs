//! 行 g-map-tips の歯: グラフの数の行の経験者だけの注釈（見本の data-tip-expert）。
//! 圧縮の面の 0 件の帯の card は行 m-map-compact で、表の行列の数の cell は行 m-map-tree で消した。
//! DOM は wasm の target のときだけなので、file の字で付け方を見る。
#![cfg(test)]

use std::path::PathBuf;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

#[test]
fn mtips_dom_text() {
    let graph = read("src/mapview/graph/dom.rs");
    assert!(graph.contains(
        "<div class=\"cutline num\" tabindex=\"0\" data-term=\"cut\" use:expert_tip=line>{count}</div>"
    ));
    assert!(graph.contains("let line = expert_line(&v);"));
    for w in ["expert_row", "shows_internal"] {
        assert!(!graph.contains(w), "graph/dom.rs に {w} が在る");
    }
}
