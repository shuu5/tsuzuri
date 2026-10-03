//! folio の build.rs の純粋な関数 `parts_catalog`（部品目録の文字列 → 導出した Rust の source か理由の文）の歯（便 52・
//! delivery-52.md §1 (d) 3）: 実の部品目録で Ok・変異 3 つがそれぞれ Err。build.rs を path で取り込むので folio の package に
//! 置く（行 k-tz-tests が tests/tz4/parts.rs から分けた・binary を撃たない）。
#![cfg(test)]

use crate::build;

use std::fs;
use std::path::{Path, PathBuf};

fn design_intent() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../design-intent")
}

fn catalog_text() -> String {
    fs::read_to_string(design_intent().join("preview/parts.json")).expect("部品目録が読めない")
}

/// 部品目録の 1 か所を書き換えた変異（置き換えが当たらなければ歯の側の誤り）。
fn mutate(text: &str, from: &str, to: &str) -> String {
    assert_eq!(
        text.matches(from).count(),
        1,
        "変異の的「{from}」が部品目録に 1 つでない"
    );
    text.replace(from, to)
}

const TYPE_IDS_TABLE: &str = "    \"type_ids\": {\n      \"archify-architecture\": \"構成図（architecture）\",\n      \"archify-workflow\": \"手順図（workflow）\",\n      \"archify-sequence\": \"順序図（sequence）\",\n      \"archify-dataflow\": \"流れ図（dataflow）\",\n      \"archify-lifecycle\": \"状態図（lifecycle）\"\n    },\n";

#[test]
fn parts_catalog_derives_limits_and_figure_type_labels_from_the_real_catalog() {
    let source = build::parts_catalog(&catalog_text()).expect("実の部品目録から導出できない");
    assert!(
        source.contains("pub const PIPELINE_RAIL_MAX_NODES: usize = "),
        "PIPELINE_RAIL_MAX_NODES が無い"
    );
    assert!(
        source.contains("pub const STATE_STRIP_MAX_NODES: usize = "),
        "STATE_STRIP_MAX_NODES が無い"
    );
    assert!(
        source.contains("pub const CONTEXT_BAND_MAX_PER_BAND: usize = "),
        "CONTEXT_BAND_MAX_PER_BAND が無い"
    );
    assert!(
        source.contains("pub const LIMITS: [(&str, &str, usize); "),
        "LIMITS が無い"
    );
    assert!(
        source.contains("pub const FIGURE_TYPE_LABELS: [(&str, &str); "),
        "FIGURE_TYPE_LABELS が無い"
    );
    assert!(
        source.contains("pub enum Component {"),
        "型 4 つの出力が無い"
    );
}

#[test]
fn parts_catalog_rejects_a_limit_that_is_a_string() {
    let text = mutate(&catalog_text(), "\"max_nodes\": 7", "\"max_nodes\": \"7\"");
    let err = build::parts_catalog(&text).expect_err("文字列の上限なのに Ok");
    assert!(
        err.contains("pipeline-rail") && err.contains("max_nodes"),
        "{err}"
    );
}

#[test]
fn parts_catalog_rejects_a_type_id_outside_the_figure_type_enum() {
    let text = mutate(
        &catalog_text(),
        "\"archify-architecture\": \"構成図",
        "\"archify-tower\": \"構成図",
    );
    let err = build::parts_catalog(&text).expect_err("figure_type_enum に無い鍵なのに Ok");
    assert!(
        err.contains("archify-tower") && err.contains("figure_type_enum"),
        "{err}"
    );
}

#[test]
fn parts_catalog_rejects_type_ids_written_as_a_list() {
    let text = mutate(
        &catalog_text(),
        TYPE_IDS_TABLE,
        "    \"type_ids\": [\n      \"archify-architecture\"\n    ],\n",
    );
    let err = build::parts_catalog(&text).expect_err("type_ids が一覧なのに Ok");
    assert!(err.contains("type_ids"), "{err}");
}
