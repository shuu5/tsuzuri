//! 群ごとの snapshot の見張りの歯（接頭辞 cfsplit_・設計ノート surface-wave16a 行 t-cform-json）。
//! 群の json（tests/snapshots の群の名の json）は群の module の見本だけを持ち、鍵は群どうしで重ならない。
//! 鍵の和は割る前の snapshot の鍵を含む（型を足す行は一覧を直さず・型を消すか名を替える行だけが直す）。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

/// 群の名と、群が見本を持つ module の列。
const GROUPS: [(&str, &[&str]); 6] = [
    ("surface", &["surface"]),
    ("ledger", &["ledger", "question"]),
    ("graph", &["graph"]),
    ("board", &["board"]),
    ("stats", &["stats"]),
    ("seat", &["seat"]),
];

/// 割る前の snapshot（tests/snapshots/contract_form.json）の鍵（割る前の順）。
const BEFORE_KEYS: [&str; 62] = [
    "surface::RulingId",
    "surface::SeatRole",
    "surface::SeatHealth",
    "surface::SeatView",
    "surface::SurfaceState",
    "surface::SurfaceEvent",
    "surface::QuestionNudge",
    "surface::RulingRequest",
    "surface::RulingResponse",
    "surface::BatchRequest",
    "surface::ItemOutcome",
    "surface::BatchResponse",
    "surface::PolicyRequest",
    "surface::PolicyResponse",
    "surface::Refusal",
    "surface::RefusalResponse",
    "ledger::BeadId",
    "ledger::ChildType",
    "ledger::Effect",
    "ledger::LedgerRow",
    "ledger::LedgerItem",
    "ledger::LedgerList",
    "ledger::LedgerChanged",
    "ledger::LedgerWrite",
    "graph::NodeKind",
    "graph::EdgeType",
    "graph::GraphNode",
    "graph::GraphEdge",
    "graph::GraphSource",
    "graph::BeadAttr",
    "graph::RunAttr",
    "graph::Verdict",
    "graph::InvariantCheck",
    "graph::SkippedEdges",
    "graph::GraphDoc",
    "graph::EdgeEnd",
    "graph::ViewNode",
    "graph::ViewEdge",
    "graph::GraphView",
    "graph::BoxFold",
    "graph::Fold",
    "graph::AroundRow",
    "graph::HubCut",
    "graph::AroundDoc",
    "board::Stage",
    "board::PipelineColumn",
    "board::PipelineCard",
    "board::PipelineBoard",
    "board::NextMove",
    "board::LedgerJudge",
    "board::AccountBoard",
    "stats::LedgerStats",
    "stats::NextStep",
    "stats::UnreflectedRow",
    "stats::UnreflectedList",
    "question::QuestionCard",
    "question::QuestionList",
    "seat::SeatState",
    "seat::QuotaUsed",
    "seat::SeatSpan",
    "seat::AccountMove",
    "seat::SeatCard",
];

fn tests_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

/// 群の json の object。
fn read(group: &str) -> Map<String, Value> {
    let path = tests_dir().join(format!("snapshots/{group}.json"));
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
    match serde_json::from_str::<Value>(&text).expect("群の json は JSON") {
        Value::Object(map) => map,
        other => panic!("{group}.json は object でない: {other}"),
    }
}

#[test]
fn cfsplit_snapshots_by_group() {
    let mut seen = BTreeSet::new();
    for (group, modules) in GROUPS {
        let map = read(group);
        assert!(!map.is_empty(), "{group}.json が空");
        for key in map.keys() {
            let module = key.split("::").next().unwrap_or_default();
            assert!(
                key.contains("::") && modules.contains(&module),
                "{group}.json に群の module でない鍵 {key}"
            );
            assert!(seen.insert(key.clone()), "鍵 {key} が群どうしで重なる");
        }
    }
    assert!(
        !tests_dir().join("snapshots/contract_form.json").exists(),
        "割る前の snapshot が残る"
    );
}

#[test]
fn cfsplit_snapshot_keys_floor() {
    let before: BTreeSet<&str> = BEFORE_KEYS.iter().copied().collect();
    assert_eq!(before.len(), BEFORE_KEYS.len(), "BEFORE_KEYS に重なり");
    let mut keys = BTreeSet::new();
    for (group, _) in GROUPS {
        for key in read(group).keys() {
            assert!(keys.insert(key.clone()), "鍵 {key} が群どうしで重なる");
        }
    }
    let lost: Vec<&str> = before
        .iter()
        .copied()
        .filter(|k| !keys.contains(*k))
        .collect();
    assert!(lost.is_empty(), "割る前の鍵が群の json に無い: {lost:?}");
}
