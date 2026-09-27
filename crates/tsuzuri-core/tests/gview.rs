//! グラフの眺めと近傍の歯（便 c-view・接頭辞 gview_）: 辺の向き・id の自然な順・次数・眺め・近傍。
//! 例のグラフは歯の中で直に組む（13 節点・16 辺）。hub-619.json は便 c の着地済みの fixture を読むだけ。

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tsuzuri_contract::graph::{
    AroundDoc, EdgeEnd, EdgeType, Fold, GraphEdge, GraphNode, GraphView, NodeKind, basis_end,
    natural_cmp,
};
use tsuzuri_core::graph::around::{AROUND_STEPS, HUB_DEGREE};
use tsuzuri_core::graph::view::VIEW_CAP;
use tsuzuri_core::graph::{BeadAttr, Graph, Inputs, RunAttr, around, build, view};

const RULING: &str = "e.2:20260927T0000Z-1";
const RUN: &str = "e.1-20260927T000000Z";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// fixture の欄を入力の字にする（歯の file graph.rs と同じ読み方）。
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

/// 子を 619 本持つ epic の h を含むグラフ。
fn hub_619() -> Graph {
    let rel = "tests/fixtures/graph/hub-619.json";
    let v: Value =
        serde_json::from_str(&read(rel)).unwrap_or_else(|e| panic!("{rel} は JSON: {e}"));
    let design_index = input_text(&v, "design_index");
    let ledger = input_text(&v, "ledger");
    let events = input_text(&v, "events");
    let g = build(&Inputs {
        design_index: &design_index,
        ledger: &ledger,
        events: &events,
    });
    assert!(g.unread.is_empty());
    g
}

fn node(id: &str, kind: NodeKind) -> GraphNode {
    GraphNode {
        id: id.into(),
        kind,
        file: None,
        digest: None,
        title: id.into(),
    }
}

fn edge(from: &str, to: &str, edge_type: EdgeType) -> GraphEdge {
    GraphEdge {
        from: from.into(),
        to: to.into(),
        edge_type,
    }
}

fn bead(kind: NodeKind, status: &str) -> BeadAttr {
    BeadAttr {
        kind,
        status: status.into(),
        labels: vec![],
        pointers: vec![],
        touches: vec![],
    }
}

/// 例のグラフ（13 節点・16 辺）。
fn example() -> Graph {
    use EdgeType::*;
    let nodes = vec![
        node("P-1", NodeKind::Article),
        node("P-1.1", NodeKind::Norm),
        node("P-1.2", NodeKind::Norm),
        node("R-1", NodeKind::Rule),
        node("FR1", NodeKind::Req),
        node("AC1", NodeKind::Ac),
        node("ACT1", NodeKind::Actor),
        node("ADR-1", NodeKind::Adr),
        node("e", NodeKind::Epic),
        node("e.1", NodeKind::Task),
        node("e.2", NodeKind::Question),
        node(RULING, NodeKind::Ruling),
        node(RUN, NodeKind::Run),
    ];
    let edges = vec![
        edge("P-1", "P-1.1", InArticle),
        edge("P-1.1", "P-1", InArticle),
        edge("P-1", "P-1.2", InArticle),
        edge("P-1.2", "P-1", InArticle),
        edge("R-1", "P-1", ArticleRef),
        edge("P-1", "R-1", RelationsRules),
        edge("FR1", "P-1", Basis),
        edge("ADR-1", "P-1", Basis),
        edge("FR1", "AC1", VerifyAc),
        edge("AC1", "FR1", Verifies),
        edge("FR1", "ADR-1", Adrs),
        edge("e.1", "e", ParentChild),
        edge("e.2", "e", ParentChild),
        edge(RULING, "e.2", Answers),
        edge(RUN, "e.1", RunOf),
        edge("e.2", "FR1", Touches),
    ];
    Graph {
        nodes,
        edges,
        beads: [
            ("e".to_string(), bead(NodeKind::Epic, "open")),
            ("e.1".to_string(), bead(NodeKind::Task, "in_progress")),
            ("e.2".to_string(), bead(NodeKind::Question, "closed")),
        ]
        .into(),
        runs: [(
            RUN.to_string(),
            RunAttr {
                stage: Some("Landed".into()),
                account: None,
                unanswered: 0,
            },
        )]
        .into(),
        ..Graph::default()
    }
}

/// 眺めの節点の (id・段・子の数・次数)。
fn view_nodes(v: &GraphView) -> Vec<(&str, u32, u32, u32)> {
    v.nodes
        .iter()
        .map(|n| (n.node.id.as_str(), n.rank, n.kids, n.degree))
        .collect()
}

/// 眺めの辺の (from・to・型・本数)。
fn view_edges(v: &GraphView) -> Vec<(&str, &str, EdgeType, u32)> {
    v.edges
        .iter()
        .map(|e| (e.from.as_str(), e.to.as_str(), e.edge_type, e.count))
        .collect()
}

/// 近傍の行の (id・列・1 つ前の節点・辺の型)。
fn around_rows(a: &AroundDoc) -> Vec<(&str, i8, Option<&str>, Option<EdgeType>)> {
    a.rows
        .iter()
        .map(|r| (r.node.id.as_str(), r.col, r.via.as_deref(), r.edge_type))
        .collect()
}

#[test]
fn gview_basis_end_examples() {
    use NodeKind::*;
    let cases = [
        (EdgeType::Basis, Req, Article, EdgeEnd::To),
        (EdgeType::InArticle, Article, Norm, EdgeEnd::From),
        (EdgeType::InArticle, Norm, Article, EdgeEnd::To),
        (EdgeType::VerifyAc, Req, Ac, EdgeEnd::From),
        (EdgeType::Verifies, Ac, Req, EdgeEnd::To),
        (EdgeType::ParentChild, Task, Epic, EdgeEnd::To),
    ];
    for (t, from, to, want) in cases {
        assert_eq!(basis_end(t, from, to), want, "{t:?} {from:?} → {to:?}");
    }
}

#[test]
fn gview_basis_end_all_types() {
    let from_types: Vec<EdgeType> = EdgeType::ALL
        .into_iter()
        .filter(|t| basis_end(*t, NodeKind::Req, NodeKind::Task) == EdgeEnd::From)
        .collect();
    assert_eq!(from_types, vec![EdgeType::AmendedBy, EdgeType::VerifyAc]);
    // 対の型は、どちらの向きの辺でも同じ節点が根拠の側（節点の種類は見本の組）。
    let up = |t: EdgeType, from: (&'static str, NodeKind), to: (&'static str, NodeKind)| {
        match basis_end(t, from.1, to.1) {
            EdgeEnd::From => from.0,
            EdgeEnd::To => to.0,
        }
    };
    let p = ("P-1", NodeKind::Article);
    let s = ("P-1.1", NodeKind::Norm);
    let r = ("R-1", NodeKind::Rule);
    let fr = ("FR1", NodeKind::Req);
    let ac = ("AC1", NodeKind::Ac);
    let old = ("ADR-1", NodeKind::Adr);
    let new = ("ADR-2", NodeKind::Adr);
    let pairs = [
        (EdgeType::InArticle, p, s, EdgeType::InArticle),
        (EdgeType::ArticleRef, r, p, EdgeType::RelationsRules),
        (EdgeType::VerifyAc, fr, ac, EdgeType::Verifies),
        (EdgeType::AmendedBy, old, new, EdgeType::Amends),
    ];
    for (t, a, b, back) in pairs {
        assert_eq!(up(t, a, b), up(back, b, a), "{t:?} と {back:?}");
    }
}

#[test]
fn gview_natural_cmp_orders_numbers() {
    assert_eq!(natural_cmp("R-2", "R-10"), Ordering::Less);
    assert_eq!(natural_cmp("R-10", "R-2"), Ordering::Greater);
    assert_eq!(natural_cmp("t3-hub.9", "t3-hub.10"), Ordering::Less);
    for id in ["R-2", "t3-hub.10", "", "e.2:20260927T0000Z-1"] {
        assert_eq!(natural_cmp(id, id), Ordering::Equal, "{id}");
    }
    assert_ne!(natural_cmp("R-01", "R-1"), Ordering::Equal);
    let mut ids = vec!["e.1", "e", "e.10", "e.2", "AC1", "ADR-1"];
    ids.sort_by(|a, b| natural_cmp(a, b));
    assert_eq!(ids, vec!["AC1", "ADR-1", "e", "e.1", "e.2", "e.10"]);
}

#[test]
fn gview_degree_counts_neighbours() {
    let g = example();
    let d = g.degrees();
    assert_eq!(d["P-1"], 5, "in-article の両向きの 2 本は隣 1");
    assert_eq!(d["FR1"], 4);
    assert!(!d.contains_key("ACT1"));
    let mut loose = example();
    loose.edges.push(edge("P-1", "P-1", EdgeType::Refs));
    loose.edges.push(edge("P-1", "no-such-id", EdgeType::Refs));
    assert_eq!(loose.degrees(), d, "自分へ戻る辺と端の無い辺は数えない");
}

#[test]
fn gview_example_view() {
    let v = view(&example());
    assert_eq!(
        view_nodes(&v),
        vec![
            ("P-1", 0, 2, 5),
            ("R-1", 1, 0, 1),
            ("FR1", 2, 0, 4),
            ("AC1", 3, 0, 1),
            ("ADR-1", 1, 0, 2),
            ("e", 3, 2, 2),
            ("e.1", 4, 0, 2),
            (RUN, 5, 0, 1),
        ]
    );
    use EdgeType::*;
    assert_eq!(
        view_edges(&v),
        vec![
            ("AC1", "FR1", VerifyAc, 1),
            ("AC1", "FR1", Verifies, 1),
            ("ADR-1", "P-1", Basis, 1),
            ("FR1", "ADR-1", Adrs, 1),
            ("FR1", "P-1", Basis, 1),
            ("R-1", "P-1", RelationsRules, 1),
            ("R-1", "P-1", ArticleRef, 1),
            ("e", "FR1", Touches, 1),
            ("e.1", "e", ParentChild, 1),
            (RUN, "e.1", RunOf, 1),
        ]
    );
    assert_eq!((v.shown, v.folded, v.cut, v.total), (8, 4, 1, 13));
    let status: Vec<Option<&str>> = v.nodes.iter().map(|n| n.status.as_deref()).collect();
    assert_eq!(
        status,
        vec![
            None,
            None,
            None,
            None,
            None,
            Some("open"),
            Some("in_progress"),
            Some("Landed")
        ]
    );
    assert!(v.unread.is_empty());
}

#[test]
fn gview_hub_619_view_is_capped() {
    let v = view(&hub_619());
    assert_eq!((v.shown, v.folded, v.cut, v.total), (40, 0, 582, 622));
    assert_eq!(v.nodes.len(), VIEW_CAP);
    let top = v.nodes.iter().max_by_key(|n| n.degree).expect("出す節点");
    assert_eq!((top.node.id.as_str(), top.degree), ("h", 619));
    // 出す節点の先頭（種類の順）も h。
    assert_eq!(v.nodes[0].node.id, "h");
    assert_eq!(v.edges.len(), 39);
    let shown: BTreeSet<&str> = v.nodes.iter().map(|n| n.node.id.as_str()).collect();
    assert!(shown.contains("h.1") && shown.contains("h.1-20260927T000000Z"));
    assert!(
        v.edges
            .iter()
            .all(|e| shown.contains(e.from.as_str()) && shown.contains(e.to.as_str()))
    );
}

#[test]
fn gview_cycle_view_returns() {
    let g = Graph {
        nodes: vec![
            node("A-1", NodeKind::Article),
            node("A-2", NodeKind::Article),
        ],
        edges: vec![
            edge("A-1", "A-2", EdgeType::RelationsArticles),
            edge("A-2", "A-1", EdgeType::RelationsArticles),
        ],
        ..Graph::default()
    };
    let v = view(&g);
    assert_eq!(v.edges.len(), 2);
    let ranks: Vec<(&str, u32)> = v
        .nodes
        .iter()
        .map(|n| (n.node.id.as_str(), n.rank))
        .collect();
    assert_eq!(ranks, vec![("A-1", 2), ("A-2", 1)]);
}

#[test]
fn gview_example_around_each_fold() {
    use EdgeType::*;
    let g = example();
    let p1 = ("P-1", -1, Some("FR1"), Some(Basis));
    let adr = ("ADR-1", -1, Some("FR1"), Some(Adrs));
    let fr1 = ("FR1", 0, None, None);
    let ac1 = ("AC1", 1, Some("FR1"), Some(Verifies));
    let e2 = ("e.2", 1, Some("FR1"), Some(Touches));
    let ruling = (RULING, 2, Some("e.2"), Some(Answers));
    let cases = [
        (Fold::None, vec![p1, adr, fr1, ac1, e2, ruling]),
        (Fold::Up, vec![fr1, ac1, e2, ruling]),
        (Fold::Down, vec![p1, adr, fr1]),
        (Fold::Both, vec![fr1]),
    ];
    for (fold, want) in cases {
        let a = around(&g, "FR1", AROUND_STEPS, fold).expect("FR1 の近傍");
        assert_eq!(around_rows(&a), want, "{fold:?}");
        let n = u32::try_from(want.len()).expect("数");
        assert_eq!((a.shown, a.total), (n, n), "{fold:?}");
        assert_eq!((a.basis, a.impact), (2, 3), "{fold:?}");
        assert_eq!((a.cut_hub, a.cut_cap), (0, 0), "{fold:?}");
        assert!(a.hubs.is_empty(), "{fold:?}");
        assert_eq!((a.center.as_str(), a.steps, a.fold), ("FR1", 2, fold));
    }
}

#[test]
fn gview_around_steps_range() {
    let g = example();
    let a = around(&g, "FR1", 1, Fold::None).expect("段数 1");
    let ids: Vec<&str> = a.rows.iter().map(|r| r.node.id.as_str()).collect();
    assert_eq!(ids, vec!["P-1", "ADR-1", "FR1", "AC1", "e.2"]);
    assert!(a.rows.iter().all(|r| r.col.abs() <= 1));
    assert_eq!((a.basis, a.impact), (2, 2));
    for steps in [0, 4] {
        assert!(
            around(&g, "FR1", steps, Fold::None).is_none(),
            "段数 {steps}"
        );
    }
    assert!(around(&g, "no-such-id", AROUND_STEPS, Fold::None).is_none());
    let three = around(&g, "FR1", 3, Fold::None).expect("段数 3");
    assert!(three.rows.iter().all(|r| r.col.abs() <= 3));
}

#[test]
fn gview_hub_619_around() {
    let g = hub_619();
    let a = around(&g, "h", AROUND_STEPS, Fold::None).expect("epic の近傍");
    assert_eq!((a.shown, a.total, a.cut_cap, a.cut_hub), (12, 621, 609, 0));
    assert!(a.hubs.is_empty(), "中心は hub でも広げる");

    // 子から見ても、hub の親から先（兄弟）へは広げない。
    let run = "h.1-20260927T000000Z";
    let c = around(&g, "h.1", AROUND_STEPS, Fold::None).expect("子の近傍");
    assert_eq!(
        around_rows(&c),
        vec![
            ("h", -1, Some("h.1"), Some(EdgeType::ParentChild)),
            ("h.1", 0, None, None),
            (run, 1, Some("h.1"), Some(EdgeType::RunOf)),
        ]
    );
    assert_eq!(c.cut_hub, 618);
    let hubs: Vec<(&str, u32)> = c.hubs.iter().map(|h| (h.id.as_str(), h.degree)).collect();
    assert_eq!(hubs, vec![("h", 619)]);
    assert_eq!(c.total, 3 + 618);
}

/// 実物の見本の近傍: 列は段数の幅の中・同じ節点は 1 度・1 つ前の節点は出す行に在り、辺の向きが列の符号と合う。
#[test]
fn gview_around_real_rows_follow_edges() {
    let g = build(&Inputs {
        design_index: &read("tests/fixtures/graph/real/design-index.tsv"),
        ledger: &read("tests/fixtures/ledger/bd-list-8.json"),
        events: &read("tests/fixtures/graph/real/events.jsonl"),
    });
    let index = g.index();
    // R-2 は A-3・P-26・ADR-3 に拠り、NFR3 と R-25 が R-2 に拠る。
    let a = around(&g, "R-2", AROUND_STEPS, Fold::None).expect("R-2 の近傍");
    let steps = i8::try_from(AROUND_STEPS).expect("段数");
    let ids: BTreeSet<&str> = a.rows.iter().map(|r| r.node.id.as_str()).collect();
    assert_eq!(ids.len(), a.rows.len(), "同じ節点を 2 度出さない");
    assert_eq!(a.shown as usize, a.rows.len());
    let at = |col: i8| -> BTreeSet<&str> {
        a.rows
            .iter()
            .filter(|r| r.col == col)
            .map(|r| r.node.id.as_str())
            .collect()
    };
    assert_eq!(at(-1), BTreeSet::from(["A-3", "ADR-3", "P-26"]));
    assert_eq!(at(1), BTreeSet::from(["NFR3", "R-25"]));
    assert!(!at(-2).is_empty(), "根拠の側の 2 段目");
    let cols: Vec<i8> = a.rows.iter().map(|r| r.col).collect();
    assert!(cols.is_sorted(), "列の小さい順");
    for r in &a.rows {
        assert!((-steps..=steps).contains(&r.col), "{r:?}");
        let Some(via) = r.via.as_deref() else {
            assert_eq!((r.node.id.as_str(), r.col), ("R-2", 0));
            continue;
        };
        assert!(ids.contains(via), "{r:?}: 1 つ前の節点が出す行に無い");
        let t = r.edge_type.expect("辺の型");
        let id = r.node.id.as_str();
        let found = g.edges.iter().any(|e| {
            let ends = (e.from == via && e.to == id) || (e.from == id && e.to == via);
            let (Some(from), Some(to)) = (index.get(e.from.as_str()), index.get(e.to.as_str()))
            else {
                return false;
            };
            let up = match basis_end(e.edge_type, from.kind, to.kind) {
                EdgeEnd::From => e.from.as_str(),
                EdgeEnd::To => e.to.as_str(),
            };
            e.edge_type == t && ends && ((r.col < 0) == (up == id))
        });
        assert!(found, "{r:?}: 列の符号と向きの合う辺が無い");
    }
}

/// hub の閾値と既定の段数の定数が rules の file の行 R-20 の字（hub の閾値（次数）N 超・近傍は各 N 段）と同じ。
#[test]
fn gview_hub_threshold_matches_rule_r20() {
    let rules = read("design-intent/rules.yaml");
    let row = rules
        .lines()
        .find(|l| l.contains("id: R-20,"))
        .expect("rules の file に行 R-20 が在る");
    let value = row
        .split_once("value: \"")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map(|(v, _)| v)
        .expect("行 R-20 の value");
    let n: usize = value
        .split_once("hub の閾値（次数）")
        .and_then(|(_, rest)| rest.split_once('超'))
        .and_then(|(n, _)| n.trim().parse().ok())
        .expect("hub の閾値の数");
    assert_eq!(n, HUB_DEGREE, "{row}");
    assert!(
        value.contains(&format!("近傍は各 {AROUND_STEPS} 段")),
        "{row}"
    );
}
