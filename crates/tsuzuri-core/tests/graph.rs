//! 導出グラフの歯（便 c）: 実物の見本の 3 つの入力・violations.json・hub-619.json・no-state.json・runs-5.json・unruled.json。
//! fixture の file は 3 つの入力（design_index・ledger・events）を持つ。
//! design_index と events は字か行の配列（配列は 1 行ずつ改行で繋ぐ・object の行は JSON にする）、
//! ledger は字か bd の出力の形の配列（配列は JSON にする）。

use std::path::{Path, PathBuf};

use serde_json::Value;
use tsuzuri_contract::graph::{EdgeType, NodeKind};
use tsuzuri_core::graph::build::DESIGN_EDGE_TYPES;
use tsuzuri_core::graph::check::{INVARIANTS, UNMEASURED};
use tsuzuri_core::graph::{Graph, Inputs, Source, Verdict, build, check};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture(name: &str) -> Value {
    let rel = format!("tests/fixtures/graph/{name}");
    serde_json::from_str(&read(&rel)).unwrap_or_else(|e| panic!("{rel} は JSON: {e}"))
}

/// fixture の欄を入力の字にする。
fn input_text(v: &Value, key: &str) -> String {
    match &v[key] {
        Value::String(s) => s.clone(),
        Value::Array(items) if key != "ledger" => items
            .iter()
            .map(|item| match item {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Null => panic!("fixture に欄 {key} が無い"),
        other => other.to_string(),
    }
}

/// fixture の 3 つの欄から組む。
fn build_from(v: &Value) -> Graph {
    let design_index = input_text(v, "design_index");
    let ledger = input_text(v, "ledger");
    let events = input_text(v, "events");
    build(&Inputs {
        design_index: &design_index,
        ledger: &ledger,
        events: &events,
    })
}

/// 違反と名指された不変条件の id（id の順）。
fn violated(g: &Graph) -> Vec<&'static str> {
    check(g)
        .into_iter()
        .filter(|i| matches!(i.verdict, Verdict::Violation(_)))
        .map(|i| i.id)
        .collect()
}

fn verdict_of(g: &Graph, id: &str) -> Verdict {
    check(g)
        .into_iter()
        .find(|i| i.id == id)
        .unwrap_or_else(|| panic!("不変条件 {id}"))
        .verdict
}

const BEAD_KINDS: [NodeKind; 4] = [
    NodeKind::Epic,
    NodeKind::Task,
    NodeKind::Memo,
    NodeKind::Question,
];

/// 実物の見本の 3 つの入力。
fn real_inputs() -> (String, String, String) {
    (
        read("tests/fixtures/graph/real/design-index.tsv"),
        read("tests/fixtures/ledger/bd-list-8.json"),
        read("tests/fixtures/graph/real/events.jsonl"),
    )
}

/// 設計の索引の節点の行（5 列）と辺の行（3 列）の数（`#` の行は数えない）。
fn table_rows(tsv: &str) -> (usize, usize) {
    let cols = |n: usize| {
        tsv.lines()
            .filter(|l| !l.starts_with('#') && l.split('\t').count() == n)
            .count()
    };
    (cols(5), cols(3))
}

#[test]
fn graph_real_inputs_match_row_counts() {
    let (design_index, ledger, events) = real_inputs();
    let g = build(&Inputs {
        design_index: &design_index,
        ledger: &ledger,
        events: &events,
    });
    assert!(g.unread.is_empty(), "読めない出所: {:?}", g.unread);

    let (node_rows, edge_rows) = table_rows(&design_index);
    assert_eq!(
        (node_rows, edge_rows),
        (236, 518),
        "索引の末尾の行の数と同じ"
    );
    let design_nodes: usize = Source::Design
        .kinds()
        .iter()
        .map(|k| g.count_nodes(*k))
        .sum();
    assert_eq!(design_nodes, node_rows);

    let beads: Vec<Value> = serde_json::from_str(&ledger).expect("bd の一覧");
    let bead_nodes: usize = BEAD_KINDS.iter().map(|k| g.count_nodes(*k)).sum();
    assert_eq!(bead_nodes, beads.len());
    assert_eq!(bead_nodes, 8);

    let created = events
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|e| e["kind"] == "RunCreated")
        .count();
    assert_eq!(g.count_nodes(NodeKind::Run), created);
    assert_eq!(created, 10);
    assert_eq!(g.count_edges(EdgeType::RunOf), created);

    let design_types = &EdgeType::ALL[..DESIGN_EDGE_TYPES];
    let design_edges = g
        .edges
        .iter()
        .filter(|e| design_types.contains(&e.edge_type))
        .count();
    assert_eq!(design_edges, edge_rows, "索引の辺は全部 17 型の内側");
    assert_eq!(g.skipped.design_edges, 0);
    assert_eq!(g.count_edges(EdgeType::ParentChild), 7);
    assert_eq!(g.count_edges(EdgeType::Blocks), 1);
}

#[test]
fn graph_real_unknown_edge_type_is_counted_not_built() {
    let (mut design_index, ledger, events) = real_inputs();
    let before = build(&Inputs {
        design_index: &design_index,
        ledger: &ledger,
        events: &events,
    });
    design_index.push_str("A-1\tA-2\tnot-a-type\n");
    let after = build(&Inputs {
        design_index: &design_index,
        ledger: &ledger,
        events: &events,
    });
    assert_eq!(after.skipped.design_edges, 1);
    assert_eq!(after.edges, before.edges);
    assert!(after.unread.is_empty());
}

#[test]
fn graph_real_runs_hold_stage_and_account() {
    let (design_index, ledger, events) = real_inputs();
    let g = build(&Inputs {
        design_index: &design_index,
        ledger: &ledger,
        events: &events,
    });
    let run = &g.runs["t3-hub.2-20260927T072139Z"];
    assert_eq!(run.stage.as_deref(), Some("Landed"));
    assert_eq!(run.account.as_deref(), Some("acct-1"));
    let first = &g.runs["t3-hub.2-20260927T071348Z"];
    assert_eq!(first.stage.as_deref(), Some("Reviewed"));
    assert_eq!(first.account, None);
    assert!(
        g.nodes
            .iter()
            .all(|n| n.kind != NodeKind::NoteRow && n.id != "acct-1"),
        "見本の索引は設計ノートの行を持たず、口座は節点にしない"
    );
}

#[test]
fn graph_violations_match_fixture() {
    let v = fixture("violations.json");
    let cases = v["cases"].as_array().expect("cases の配列");
    let mut hit: Vec<&str> = Vec::new();
    for case in cases {
        let name = case["name"].as_str().expect("組の名");
        let want: Vec<&str> = case["violates"]
            .as_array()
            .expect("violates の配列")
            .iter()
            .map(|x| x.as_str().expect("不変条件の id"))
            .collect();
        let g = build_from(case);
        assert!(g.unread.is_empty(), "{name}: 読めない出所 {:?}", g.unread);
        let got = violated(&g);
        let mut want_sorted: Vec<&str> = want.clone();
        want_sorted.sort_by_key(|id| INVARIANTS.iter().position(|x| x == id));
        assert_eq!(got, want_sorted, "{name}: 違反と名指された不変条件");
        for id in UNMEASURED {
            assert_eq!(verdict_of(&g, id), Verdict::Unknown, "{name}: {id}");
        }
        hit.extend(got);
    }
    for id in INVARIANTS.iter().filter(|id| !UNMEASURED.contains(id)) {
        assert!(hit.contains(id), "{id} を違反にする組が無い");
    }
    assert_eq!(
        INVARIANTS.len() - UNMEASURED.len(),
        9,
        "この便で数えるのは 9 本"
    );
}

#[test]
fn graph_violation_names_the_ids() {
    let v = fixture("violations.json");
    let case = v["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|c| c["violates"] == serde_json::json!(["g-1", "g-11"]))
        .expect("g-1 と g-11 の組");
    let g = build_from(case);
    assert_eq!(
        verdict_of(&g, "g-11"),
        Verdict::Violation(vec!["gone.1-20260927T000000Z".to_string()])
    );
    assert_eq!(
        verdict_of(&g, "g-1"),
        Verdict::Violation(vec!["gone.1".to_string()])
    );
}

#[test]
fn graph_no_state_unknown_only_runs() {
    let v = fixture("no-state.json");
    assert_eq!(v["events"], "", "event log の字が空");
    let g = build_from(&v);
    assert_eq!(g.unread, vec![Source::Runs]);
    assert_eq!(g.unknown_kinds(), vec![NodeKind::Run]);
    assert!(g.count_nodes(NodeKind::Article) > 0, "設計の節点は組める");
    assert!(
        BEAD_KINDS.iter().all(|k| g.count_nodes(*k) > 0),
        "台帳の節点は組める"
    );
    assert_eq!(g.count_nodes(NodeKind::Run), 0);
    for id in ["g-11", "g-12"] {
        assert_eq!(verdict_of(&g, id), Verdict::Unknown, "{id}");
    }
    // memo の source の先の走行は組めないので、片端の無い辺は「まだ分からない」（違反にしない）。
    assert_eq!(verdict_of(&g, "g-1"), Verdict::Unknown);
    assert_eq!(verdict_of(&g, "g-10"), Verdict::Unknown);
    for id in ["g-2", "g-4", "g-5", "g-6", "g-8"] {
        assert_eq!(verdict_of(&g, id), Verdict::Pass, "{id}");
    }
}

#[test]
fn graph_runs_5_are_five_nodes() {
    let g = build_from(&fixture("runs-5.json"));
    assert!(g.unread.is_empty());
    assert_eq!(g.count_nodes(NodeKind::Run), 5);
    let run_of: Vec<&str> = g
        .edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::RunOf)
        .map(|e| e.to.as_str())
        .collect();
    assert_eq!(run_of, vec!["r.1"; 5]);
    assert_eq!(
        verdict_of(&g, "g-12"),
        Verdict::Violation(vec!["r.1-20260927T000400Z".to_string()]),
        "問いの無い Questioned の走行を名指す"
    );
    assert_eq!(verdict_of(&g, "g-11"), Verdict::Pass);
    assert_eq!(
        g.runs["r.1-20260927T000400Z"].account.as_deref(),
        Some("acct-2")
    );
}

#[test]
fn graph_unruled_has_no_ruling() {
    let v = fixture("unruled.json");
    let g = build_from(&v);
    assert!(g.unread.is_empty());
    let closed_questions: Vec<&str> = g
        .beads
        .iter()
        .filter(|(_, b)| b.kind == NodeKind::Question && !b.is_open())
        .map(|(id, _)| id.as_str())
        .collect();
    assert_eq!(closed_questions, vec!["u.1"]);
    assert_eq!(g.count_nodes(NodeKind::Ruling), 0);
    assert_eq!(g.count_edges(EdgeType::Answers), 0);
}

#[test]
fn graph_typed_lines_make_nodes_and_answers() {
    let v = fixture("violations.json");
    let case = v["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|c| c["violates"] == serde_json::json!([]))
        .expect("違反 0 の組");
    let g = build_from(case);
    assert_eq!(g.count_nodes(NodeKind::Ruling), 1);
    assert_eq!(g.count_nodes(NodeKind::Receipt), 1);
    assert_eq!(g.count_nodes(NodeKind::Policy), 1);
    let has = |from: &str, to: &str, t: EdgeType| {
        g.edges
            .iter()
            .any(|e| e.from == from && e.to == to && e.edge_type == t)
    };
    assert!(has("user 2026-09-27T01:00Z", "v.3", EdgeType::Answers));
    assert!(has("v.3", "A-1", EdgeType::Touches));
    assert!(has("v.3", "p-1", EdgeType::Premises));
    assert!(has("v.4", "v.1-20260927T000000Z", EdgeType::Source));
    assert!(has("v.1-20260927T000000Z", "v.3", EdgeType::Raised));
    assert_eq!(g.beads["v.1"].pointers, vec!["design = contracts/x.toml#a"]);
    assert_eq!(g.count_edges(EdgeType::Design), 0, "design の辺は組まない");
    assert_eq!(
        g.count_edges(EdgeType::RuledBy),
        0,
        "ruled_by の辺は組まない"
    );
}

#[test]
fn graph_bead_kind_follows_the_order() {
    let ledger = r#"[
      {"id":"k","title":"epic に memo の label","status":"open","issue_type":"epic","labels":["intake:memo"]},
      {"id":"k.1","title":"memo と問いの label","status":"open","issue_type":"task","labels":["intake:question","intake:memo"]},
      {"id":"k.2","title":"問い","status":"open","issue_type":"task","labels":["intake:question"]},
      {"id":"k.3","title":"契約","status":"open","issue_type":"feature"}
    ]"#;
    let g = build(&Inputs {
        design_index: "A-1\t条\tconstitution.yaml\t00000000\t条",
        ledger,
        events: "{\"kind\":\"RunCreated\",\"run\":\"k.3-20260927T000000Z\"}",
    });
    let kinds: Vec<(String, NodeKind)> =
        g.beads.iter().map(|(id, b)| (id.clone(), b.kind)).collect();
    assert_eq!(
        kinds,
        vec![
            ("k".to_string(), NodeKind::Epic),
            ("k.1".to_string(), NodeKind::Memo),
            ("k.2".to_string(), NodeKind::Question),
            ("k.3".to_string(), NodeKind::Task),
        ]
    );
}

#[test]
fn graph_unreadable_inputs_leave_other_kinds() {
    let g = build(&Inputs {
        design_index: "A-1\t条\tconstitution.yaml\t00000000\t条",
        ledger: "not json",
        events: "{\"kind\":\"RunCreated\",\"run\":\"x.1-20260927T000000Z\"}\n{broken",
    });
    assert_eq!(g.unread, vec![Source::Ledger, Source::Runs]);
    assert_eq!(g.count_nodes(NodeKind::Article), 1);
    let ledger_unknown: Vec<NodeKind> = Source::Ledger.kinds().to_vec();
    assert!(ledger_unknown.iter().all(|k| g.unknown_kinds().contains(k)));
    for id in ["g-2", "g-4", "g-5", "g-6", "g-8", "g-11", "g-12"] {
        assert_eq!(verdict_of(&g, id), Verdict::Unknown, "{id}");
    }

    let bad_kind = build(&Inputs {
        design_index: "A-1\t知らない種類\tconstitution.yaml\t00000000\t条",
        ledger: "[]",
        events: "{\"kind\":\"RunCreated\",\"run\":\"x.1-20260927T000000Z\"}",
    });
    assert_eq!(bad_kind.unread, vec![Source::Design]);
    assert!(bad_kind.nodes.iter().all(|n| n.kind == NodeKind::Run));
}
