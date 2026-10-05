//! 事実の口の概要を消す歯（接頭辞 ftrim_・設計ノート surface-wave27b 行 c-fact-trim・判断の記録 ADR-30 決定 (3)）。
//! 吹き出しは概要を 1 本の引きの口から読むので、bead の事実の一覧は本文を読まず、電文の行は概要の鍵を持たない。
#![cfg(test)]

use std::path::Path;

use serde_json::{Value, json};
use tsuzuri_contract::board::Reading;
use tsuzuri_core::ledger::facts::facts;

/// 中核の crate の根からの相対 path の字（読めなければ落ちる）。
fn src(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} が読めない: {e}", path.display()))
}

/// 本文に観測の見出しと 2 つの概要の定型行を持つ bead（ほかの欄は全部読める形）。
fn bead_with_body() -> Value {
    json!({
        "id": "fx-c.1",
        "title": "問い — 色",
        "description": "### 観測\n見た事の行\n概要 = 平の字の概要\n技術 = 技術の字の概要\n",
        "status": "open",
        "issue_type": "task",
        "created_at": "2026-09-27T00:00:00Z",
        "metadata": {"short": "色"},
        "dependencies": [{"depends_on_id": "fx-c.2", "type": "blocks"}],
    })
}

#[test]
fn ftrim_fact_keys_without_summary() {
    let got = facts(&Value::Array(vec![bead_with_body()]).to_string());
    let Reading::Known(rows) = got.rows.clone() else {
        panic!("読める台帳が Unknown");
    };
    assert_eq!(rows.len(), 1, "{rows:?}");
    let row = serde_json::to_value(&rows[0]).expect("事実の行を JSON にする");
    let mut keys: Vec<&str> = row
        .as_object()
        .expect("事実の行は object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        ["blocks", "created_at", "id", "short", "short_set"],
        "事実の行の鍵は 5 つで summary を持たない"
    );
    let wire = serde_json::to_string(&got).expect("事実の一覧を JSON にする");
    for body in ["見た事の行", "平の字の概要", "技術の字の概要"] {
        assert!(
            !wire.contains(body),
            "事実の電文が本文の字 {body} を運ぶ: {wire}"
        );
    }
}

#[test]
fn ftrim_facts_src_without_summary() {
    let contract = src("../tsuzuri-contract/src/ledger.rs");
    let start = contract
        .find("pub struct BeadFact {")
        .expect("契約に BeadFact が在る");
    let block = &contract[start..];
    let block = &block[..block.find("\n}\n").expect("BeadFact の閉じ")];
    assert!(!block.contains("summary"), "BeadFact が欄 summary を持つ");
    let text = src("src/ledger/facts.rs");
    for name in [
        "summary_of",
        "SUMMARY_MAX",
        "OBSERVATION_HEADING",
        "SUMMARY_LABEL",
        "summary",
        "description",
    ] {
        assert!(!text.contains(name), "facts.rs が字 {name} を持つ");
    }
}
