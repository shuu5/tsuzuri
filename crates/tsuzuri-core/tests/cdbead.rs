//! 台帳の契約の bead の write-set の歯（行 t-code-bead・要件 FR2・接頭辞 cdbead_）。
//! 台帳の字から契約の bead の行の id と write-set の項を読み、契約の bead を名指す memo と要件を名指す memo の繋ぐ門
//! （`memo_needs`）が行 t-graph-bead の req の辺と blocks の辺で契約の bead を拾うことを留める。
#![cfg(test)]

use std::collections::BTreeSet;

use serde_json::{Value, json};
use tsuzuri_core::graph::code::bead_write_sets;
use tsuzuri_core::graph::{Graph, Inputs, Source, build};
use tsuzuri_core::memo_gate::memo_needs;

/// folio の索引の形の 13 行（節点 5・辺 5）。
const IDX: &str = "# 節点（1 行 = id / 種類 / file / 要約値 8 字 / 題 36 字・タブ区切り）
FR1\t要件\tsrs.yaml\t00000000\t面は 2 つ
A-1\t条\tconstitution.yaml\t00000000\t見本の条
nx#a\t設計ノートの行\tdesign-note/nx.yaml\t1a2b3c4d\t便 a — 骨格
nx#b\t設計ノートの行\tdesign-note/nx.yaml\t5e6f7a8b\t便 b — 型
ny#c\t設計ノートの行\tdesign-note/ny.yaml\t9c0d1e2f\t行 c — 札
# 辺（1 行 = 端 / 端 / 型・タブ区切り）
FR1\tA-1\tbasis
nx#a\tFR1\treq
nx#b\tFR1\treq
nx#b\tnx#a\tdepends
ny#c\tnx#b\tdepends
# 節点 5・辺 5・型 3・端が節点でない参照 0
";

/// 契約の行 1 つの acceptance の字（write-set などの値は JSON の字のまま）。
fn contract(id: &str, lines: &[(&str, Value)]) -> String {
    let mut text = format!("[[contract]]\nid = {}", json!(id));
    for (key, value) in lines {
        text.push_str(&format!("\n{key} = {value}"));
    }
    text
}

/// 台帳の bead 1 本（acceptance が None なら欄を持たない）。
fn bead(id: &str, status: &str, acceptance: Option<&str>) -> Value {
    let mut b = json!({"id": id, "title": format!("t {id}"), "status": status, "issue_type": "task"});
    if let Some(text) = acceptance {
        b["acceptance_criteria"] = json!(text);
    }
    b
}

/// 節の見本の台帳（kb.1 と kb.3 が契約の行・kb.2 は pointer の行だけ・kb.4 は acceptance 無し）。
fn sample() -> Vec<Value> {
    vec![
        bead(
            "kb.1",
            "open",
            Some(&contract("t-one", &[("write-set", json!(["b.rs", "+d.rs"]))])),
        ),
        bead("kb.2", "open", Some("design = contracts/nx.toml#a")),
        bead(
            "kb.3",
            "closed",
            Some(&contract("t-three", &[("write-set", json!(["a.rs"]))])),
        ),
        bead("kb.4", "open", None),
    ]
}

fn ledger(beads: Vec<Value>) -> String {
    Value::Array(beads).to_string()
}

fn row(id: &str, items: &[&str]) -> (String, Vec<String>) {
    (id.to_string(), items.iter().map(|s| s.to_string()).collect())
}

#[test]
fn cdbead_bead_write_sets_reads_contract_beads() {
    assert_eq!(
        bead_write_sets(&ledger(sample())),
        Some(vec![
            row("kb.1#t-one", &["b.rs", "+d.rs"]),
            row("kb.3#t-three", &["a.rs"]),
        ])
    );
}

#[test]
fn cdbead_bead_write_sets_unreadable_is_none() {
    for text in ["", "not json"] {
        assert_eq!(bead_write_sets(text), None, "{text:?}");
    }
    let mut broken = sample();
    broken[0] = bead(
        "kb.1",
        "open",
        Some(&contract("t-one", &[("write-set", json!("b.rs"))])),
    );
    assert_eq!(bead_write_sets(&ledger(broken)), None);
    assert_eq!(bead_write_sets("[]"), Some(Vec::new()));
}

/// 節の見本の索引と台帳（epic mb と task の mb.1・mb.2・mb.3・mb.4）で組んだグラフ。
fn memo_graph() -> Graph {
    let task = |id: &str, status: &str, acceptance: String, blocks: Option<&str>| {
        let mut deps = vec![json!({"depends_on_id": "mb", "type": "parent-child"})];
        if let Some(to) = blocks {
            deps.push(json!({"depends_on_id": to, "type": "blocks"}));
        }
        let mut b = bead(id, status, Some(&acceptance));
        b["dependencies"] = Value::Array(deps);
        b
    };
    let one = |id: &str| contract(id, &[("req", json!(["FR1"]))]);
    let ledger = ledger(vec![
        json!({"id": "mb", "title": "根", "status": "open", "issue_type": "epic"}),
        task("mb.1", "open", one("t-one"), None),
        task("mb.2", "open", one("t-two"), Some("mb.1")),
        task("mb.3", "closed", one("t-three"), None),
        task("mb.4", "open", "design = contracts/nx.toml#a".to_string(), None),
    ]);
    build(&Inputs {
        design_index: IDX,
        ledger: &ledger,
        events: "",
    })
}

/// memo_needs の結果を集合にする。
fn needs(g: &Graph, touch: &str) -> BTreeSet<String> {
    memo_needs(g, &[touch.to_string()]).into_iter().collect()
}

fn set(ids: &[&str]) -> BTreeSet<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

#[test]
fn cdbead_memo_needs_reach_bead_contracts() {
    let g = memo_graph();
    assert_eq!(g.unread, vec![Source::Runs], "空の event log だけが読めない");
    assert_eq!(
        needs(&g, "FR1"),
        set(&["A-1", "mb.1", "mb.2", "mb.4", "nx#a", "nx#b"])
    );
    assert_eq!(needs(&g, "mb.2"), set(&["FR1", "mb.1"]));
}
