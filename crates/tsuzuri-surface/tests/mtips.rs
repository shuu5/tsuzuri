//! 行 g-map-tips の歯: グラフの数の行と表の行列の数の cell の経験者だけの注釈（見本の data-tip-expert）。
//! 圧縮の面の 0 件の帯の card は行 m-map-compact で消した。
//! DOM は wasm の target のときだけなので、2 つの file の字で付け方を見る。
#![cfg(test)]

use std::path::PathBuf;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// file の字 `mod dom {` から後の字（wasm の target のときだけの DOM）。
fn dom_part(rel: &str) -> String {
    let src = read(rel);
    let at = src
        .find("mod dom {")
        .unwrap_or_else(|| panic!("{rel} に mod dom が無い"));
    src[at..].to_string()
}

#[test]
fn mtips_dom_text() {
    let table = dom_part("src/mapview/table.rs");
    assert!(
        table.contains(
            "<td use:expert_tip=c.types_text()>{pair_button(c, search, \"\")}</td>"
        ),
        "table.rs の行列の cell に経験者の注釈が無い"
    );
    assert!(!table.contains("title="), "table.rs の mod dom に title= が在る");

    let graph = read("src/mapview/graph/dom.rs");
    assert!(graph.contains(
        "<div class=\"cutline num\" tabindex=\"0\" data-term=\"cut\" use:expert_tip=line>{count}</div>"
    ));
    assert!(graph.contains("let line = expert_line(&v);"));
    for w in ["expert_row", "shows_internal"] {
        assert!(!graph.contains(w), "graph/dom.rs に {w} が在る");
    }
}
