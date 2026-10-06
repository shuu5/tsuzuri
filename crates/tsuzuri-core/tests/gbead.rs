//! 台帳の bead の契約の行の歯（行 t-graph-bead・要件 FR2・接頭辞 gbead_）。
//! acceptance の `[[contract]]` の行を持つ bead は、種類 task の節点で、塊の req から req の辺を組み、design の辺は組まない。
//! 不変条件 g-2 と g-8 は pointer の行と契約の行の塊を併せて数え、pipeline の板は契約の行の bead に札を作る。
#![cfg(test)]

use serde_json::{Value, json};
use tsuzuri_contract::board::{PipelineCard, Reading, Stage};
use tsuzuri_contract::graph::{BeadAttr, EdgeType, NodeKind};
use tsuzuri_core::graph::build::{CONTRACT_HEAD, contract_rows};
use tsuzuri_core::graph::{Graph, Inputs, Verdict, build, check};
use tsuzuri_core::pipeline::board;

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

/// 器の event log の 1 行（台帳に無い bead zz.1 の RunCreated）。
const EV: &str = r#"{"schema":1,"ts":"2026-10-06T01:00:00Z","kind":"RunCreated","run":"zz.1-20261006T010000Z","bead":"zz.1","stage":"Intake"}"#;

/// 板の今の時刻。
const NOW: u64 = 1_791_000_000;

/// 契約の行の塊 1 つの acceptance の字。
fn block(id: &str, reqs: &[&str]) -> String {
    format!(
        "{CONTRACT_HEAD}\nid = {}\nreq = {}",
        json!(id),
        json!(reqs)
    )
}

/// 台帳の task の bead（親 gb・種類 task・open）。
fn task(id: &str, acceptance: &str, blocks: Option<&str>) -> Value {
    let mut deps = vec![json!({"depends_on_id": "gb", "type": "parent-child"})];
    if let Some(to) = blocks {
        deps.push(json!({"depends_on_id": to, "type": "blocks"}));
    }
    json!({
        "id": id,
        "title": format!("t {id}"),
        "status": "open",
        "issue_type": "task",
        "acceptance_criteria": acceptance,
        "dependencies": deps,
    })
}

/// 節の台帳の bead（epic gb と task の gb.1・gb.2・gb.3）。
fn beads() -> Vec<Value> {
    vec![
        json!({"id": "gb", "title": "根", "status": "open", "issue_type": "epic"}),
        task("gb.1", &block("t-one", &["FR1"]), None),
        task("gb.2", "design = contracts/nx.toml#a", None),
        task("gb.3", &block("t-three", &["FR1", "FR1"]), Some("gb.1")),
    ]
}

/// 節の台帳に足す bead を加えた台帳の字。
fn ledger_with(extra: Vec<Value>) -> String {
    let mut all = beads();
    all.extend(extra);
    Value::Array(all).to_string()
}

fn graph_of(ledger: &str) -> Graph {
    build(&Inputs {
        design_index: IDX,
        ledger,
        events: EV,
    })
}

/// 節の台帳に両方の形の gb.4 と 2 塊の gb.5 を足した台帳の字。
fn g2_ledger() -> String {
    let both = format!("design = contracts/nx.toml#b\n{}", block("t-four", &[]));
    let two = format!("{}\n{}", block("t-five", &[]), block("t-six", &[]));
    ledger_with(vec![task("gb.4", &both, None), task("gb.5", &two, None)])
}

/// 節の台帳に契約の行を持つ memo の gb.6 を足した台帳の字。
fn g8_ledger() -> String {
    let mut memo = task("gb.6", &block("t-seven", &["FR1"]), None);
    memo["labels"] = json!(["intake:memo"]);
    ledger_with(vec![memo])
}

/// 不変条件 id の判定。
fn verdict(g: &Graph, id: &str) -> Verdict {
    check(g)
        .into_iter()
        .find(|i| i.id == id)
        .unwrap_or_else(|| panic!("不変条件 {id} が無い"))
        .verdict
}

/// from から出る辺の (to, 型)。
fn out(g: &Graph, from: &str) -> Vec<(String, EdgeType)> {
    g.edges
        .iter()
        .filter(|e| e.from == from)
        .map(|e| (e.to.clone(), e.edge_type))
        .collect()
}

fn of_type(edges: &[(String, EdgeType)], t: EdgeType) -> Vec<&str> {
    edges
        .iter()
        .filter(|(_, ty)| *ty == t)
        .map(|(to, _)| to.as_str())
        .collect()
}

/// 板の札。
fn card(cards: &[PipelineCard], id: &str) -> Option<Stage> {
    cards
        .iter()
        .find(|c| c.contract.to_string() == id)
        .map(|c| c.stage)
}

fn cards_of(ledger: &str) -> Vec<PipelineCard> {
    match board(ledger, EV, NOW).board.cards {
        Reading::Known(c) => c,
        Reading::Unknown => panic!("札が Unknown"),
    }
}

/// (1) 定数 CONTRACT_HEAD は字 `[[contract]]` である。
#[test]
fn gbead_contract_head_word() {
    assert_eq!(CONTRACT_HEAD, "[[contract]]");
}

/// (2) 2 塊の見本の字から、id が t-one で req が FR1 の塊と、id が読めず req が空の塊を、この順に読む。
#[test]
fn gbead_contract_rows_read_id_and_req() {
    let text = "前の行は読まない\nid = \"x\"\n[[contract]]\r\nid = \"t-one\"\r\nreq = [\"FR1\"]\r\n[[contract]]  \nid = bare\n";
    assert_eq!(
        contract_rows(text),
        vec![
            (Some("t-one".to_string()), vec!["FR1".to_string()]),
            (None, vec![]),
        ]
    );
}

/// (3) 契約の行の bead gb.1 は種類 task の節点で、属性の contracts は t-one の 1 つ、pointers は空。
#[test]
fn gbead_contract_bead_is_a_node() {
    let g = graph_of(&ledger_with(vec![]));
    assert!(g.unread.is_empty(), "読めない出所: {:?}", g.unread);
    let node = g.nodes.iter().find(|n| n.id == "gb.1").expect("gb.1");
    assert_eq!(node.kind, NodeKind::Task);
    let attr = &g.beads["gb.1"];
    assert_eq!(attr.kind, NodeKind::Task);
    assert_eq!(attr.contracts, vec!["t-one".to_string()]);
    assert!(attr.pointers.is_empty());
}

/// (4)(5) req の辺は塊の req から 1 本ずつ（同じ先は 1 度）・design の辺は組まない・blocks の辺は台帳のまま。
#[test]
fn gbead_req_edges_and_no_design_edge() {
    let g = graph_of(&ledger_with(vec![]));
    let one = out(&g, "gb.1");
    assert_eq!(of_type(&one, EdgeType::Req), vec!["FR1"]);
    assert_eq!(one.len(), 2, "req の辺と親への parent-child の辺: {one:?}");
    assert_eq!(of_type(&one, EdgeType::ParentChild), vec!["gb"]);
    assert!(of_type(&one, EdgeType::Design).is_empty());
    let three = out(&g, "gb.3");
    assert_eq!(of_type(&three, EdgeType::Req), vec!["FR1"], "FR1 を 2 度書いても 1 本");
    assert_eq!(of_type(&three, EdgeType::Blocks), vec!["gb.1"]);
    assert!(of_type(&three, EdgeType::Design).is_empty());
}

/// (6) pointer の形の gb.2 は nx#a への design の辺を組み、contracts は空。
#[test]
fn gbead_pointer_form_keeps_design_edge() {
    let g = graph_of(&ledger_with(vec![]));
    let two = out(&g, "gb.2");
    assert_eq!(of_type(&two, EdgeType::Design), vec!["nx#a"]);
    assert!(of_type(&two, EdgeType::Req).is_empty());
    assert!(g.beads["gb.2"].contracts.is_empty());
    assert_eq!(
        g.beads["gb.2"].pointers,
        vec!["design = contracts/nx.toml#a".to_string()]
    );
}

/// (7) g-2 は節の台帳で合格・両方の形の gb.4 と 2 塊の gb.5 を足すと両方の違反。
#[test]
fn gbead_g2_counts_both_forms() {
    assert_eq!(verdict(&graph_of(&ledger_with(vec![])), "g-2"), Verdict::Pass);
    assert_eq!(
        verdict(&graph_of(&g2_ledger()), "g-2"),
        Verdict::Violation(vec!["gb.4".to_string(), "gb.5".to_string()])
    );
}

/// (8) g-8 は節の台帳で合格・契約の行を持つ memo の gb.6 を足すと違反。
#[test]
fn gbead_g8_counts_contract_memo() {
    assert_eq!(verdict(&graph_of(&ledger_with(vec![])), "g-8"), Verdict::Pass);
    assert_eq!(
        verdict(&graph_of(&g8_ledger()), "g-8"),
        Verdict::Violation(vec!["gb.6".to_string()])
    );
}

/// (9) 板は契約の行の bead に札を作る（gb.1 は Queued・blocks の先が open の gb.3 は Blocked）・両方の形の gb.4 は札を作らない。
#[test]
fn gbead_pipeline_cards_bead_contracts() {
    let cards = cards_of(&ledger_with(vec![]));
    assert_eq!(card(&cards, "gb.1"), Some(Stage::Queued));
    assert_eq!(card(&cards, "gb.3"), Some(Stage::Blocked));
    assert_eq!(card(&cards, "gb.2"), Some(Stage::Queued));
    let cards = cards_of(&g2_ledger());
    assert_eq!(card(&cards, "gb.4"), None);
    assert_eq!(card(&cards, "gb.5"), Some(Stage::Queued));
}

/// (10) 電文の BeadAttr は contracts が空なら字 contracts を持たず、1 つなら配列を持ち、どちらも同じ値に読み戻る。
#[test]
fn gbead_wire_contracts_skip_empty() {
    let attr = |contracts: Vec<String>| BeadAttr {
        kind: NodeKind::Task,
        status: "open".into(),
        labels: vec![],
        pointers: vec![],
        contracts,
        touches: vec![],
    };
    for (contracts, wire) in [(vec![], None), (vec!["t-one".to_string()], Some(json!(["t-one"])))] {
        let a = attr(contracts);
        let text = serde_json::to_string(&a).expect("字にする");
        let value: Value = serde_json::from_str(&text).expect("JSON");
        assert_eq!(value.get("contracts"), wire.as_ref(), "{text}");
        assert_eq!(serde_json::from_str::<BeadAttr>(&text).expect("読み戻す"), a);
    }
}
