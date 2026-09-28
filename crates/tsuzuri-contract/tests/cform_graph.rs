//! 契約の型の外形の歯（接頭辞 contract_form_）のうち graph の群の型の見本。
//! snapshot は tests/snapshots の群の名の json（graph.json）で、字の比べは標準 library だけで行う。
//! 共通の手は tests/common/mod.rs に 1 つだけ在る。
//! 字が違えば今の字を CARGO_TARGET_TMPDIR の群の名の json に書いて落ちる（見て正しければ snapshot へ写す）。

mod common;

use common::{Form, distinct, form};
use tsuzuri_contract::graph::{
    AroundDoc, AroundRow, BeadAttr, BoxFold, EdgeEnd, EdgeType, Fold, GraphDoc, GraphEdge, GraphNode,
    GraphSource, GraphView, HubCut, InvariantCheck, NodeKind, RunAttr, SkippedEdges, Verdict,
    ViewEdge, ViewNode, title36,
};
use tsuzuri_contract::wire;

fn bead_attr() -> BeadAttr {
    BeadAttr {
        kind: NodeKind::Task,
        status: "open".into(),
        labels: vec!["surface".into()],
        pointers: vec!["design = contracts/surface-base.toml#b".into()],
        touches: vec![],
    }
}

fn run_attr() -> RunAttr {
    RunAttr {
        stage: Some("Gated".into()),
        account: Some("acct-4".into()),
    }
}

/// 3 値の判定の見本（合格・違反・まだ分からない）。
fn invariant_checks() -> Vec<InvariantCheck> {
    vec![
        InvariantCheck {
            id: "g-1".into(),
            verdict: Verdict::Pass,
            violations: 0,
            ids: vec![],
        },
        InvariantCheck {
            id: "g-2".into(),
            verdict: Verdict::Violation,
            violations: 2,
            ids: vec!["t3-hub.3".into(), "t3-hub.4".into()],
        },
        InvariantCheck {
            id: "g-3".into(),
            verdict: Verdict::Unknown,
            violations: 0,
            ids: vec![],
        },
    ]
}

fn task_node() -> GraphNode {
    GraphNode {
        id: "t3-hub.3".into(),
        kind: NodeKind::Task,
        file: None,
        digest: None,
        title: "契約の型".into(),
        line: None,
        plain: None,
        eng: None,
    }
}

fn run_node() -> GraphNode {
    GraphNode {
        id: "t3-hub.3-20260927T073916Z".into(),
        kind: NodeKind::Run,
        file: None,
        digest: None,
        title: "t3-hub.3-20260927T073916Z".into(),
        line: None,
        plain: None,
        eng: None,
    }
}

/// 状態を持つ bead の節点と、状態の無い走行の節点。
fn view_nodes() -> Vec<ViewNode> {
    vec![
        ViewNode {
            node: task_node(),
            status: Some("open".into()),
            rank: 0,
            kids: 2,
            degree: 5,
            group: false,
            fold: BoxFold::Folded,
        },
        ViewNode {
            node: run_node(),
            status: None,
            rank: 1,
            kids: 0,
            degree: 1,
            group: false,
            fold: BoxFold::Leaf,
        },
    ]
}

fn view_edge() -> ViewEdge {
    ViewEdge {
        from: "t3-hub.3-20260927T073916Z".into(),
        to: "t3-hub.3".into(),
        edge_type: EdgeType::RunOf,
        count: 1,
    }
}

/// 中心の行と、影響の側の 1 段目の行。
fn around_rows() -> Vec<AroundRow> {
    vec![
        AroundRow {
            node: task_node(),
            status: Some("open".into()),
            col: 0,
            via: None,
            edge_type: None,
            degree: 5,
        },
        AroundRow {
            node: run_node(),
            status: Some("Landed".into()),
            col: 1,
            via: Some("t3-hub.3".into()),
            edge_type: Some(EdgeType::RunOf),
            degree: 1,
        },
    ]
}

fn hub_cut() -> HubCut {
    HubCut {
        id: "t3-hub".into(),
        degree: 31,
    }
}

/// graph の群の型の見本（この順が snapshot の file の key の順）。
fn forms() -> Vec<Box<dyn Form>> {
    vec![
        // graph
        form("graph::NodeKind", NodeKind::ALL.to_vec()),
        form("graph::EdgeType", EdgeType::ALL.to_vec()),
        form(
            "graph::GraphNode",
            vec![
                GraphNode {
                    id: "ADR-7".into(),
                    kind: NodeKind::Adr,
                    file: Some("adr/ADR-7.yaml".into()),
                    digest: Some("9c1e0a4b".into()),
                    title: title36(
                        "裁定面は project board と account board の 2 面とし、台帳と設計文書と走行を 1 つの導出グラフに結ぶ",
                    ),
                    line: Some(3),
                    plain: Some("決定の画面の形を決めます。".into()),
                    eng: Some("面は 2 つとする。".into()),
                },
                GraphNode {
                    id: "t3-hub.3".into(),
                    kind: NodeKind::Task,
                    file: None,
                    digest: None,
                    title: "契約の型".into(),
                    line: None,
                    plain: None,
                    eng: None,
                },
            ],
        ),
        form(
            "graph::GraphEdge",
            vec![GraphEdge {
                from: "t3-hub.3".into(),
                to: "surface-base#b".into(),
                edge_type: EdgeType::Design,
            }],
        ),
        form("graph::GraphSource", GraphSource::ALL.to_vec()),
        form("graph::BeadAttr", vec![bead_attr()]),
        form(
            "graph::RunAttr",
            vec![
                run_attr(),
                RunAttr {
                    stage: None,
                    account: None,
                },
            ],
        ),
        form("graph::Verdict", Verdict::ALL.to_vec()),
        form("graph::InvariantCheck", invariant_checks()),
        form(
            "graph::SkippedEdges",
            vec![SkippedEdges {
                design: 3,
                ledger: 0,
                design_nodes: 2,
            }],
        ),
        form(
            "graph::GraphDoc",
            vec![
                GraphDoc {
                    nodes: vec![
                        GraphNode {
                            id: "t3-hub.3".into(),
                            kind: NodeKind::Task,
                            file: None,
                            digest: None,
                            title: "契約の型".into(),
                            line: None,
                            plain: None,
                            eng: None,
                        },
                        GraphNode {
                            id: "t3-hub.3-20260927T073916Z".into(),
                            kind: NodeKind::Run,
                            file: None,
                            digest: None,
                            title: "t3-hub.3-20260927T073916Z".into(),
                            line: None,
                            plain: None,
                            eng: None,
                        },
                    ],
                    edges: vec![GraphEdge {
                        from: "t3-hub.3-20260927T073916Z".into(),
                        to: "t3-hub.3".into(),
                        edge_type: EdgeType::RunOf,
                    }],
                    unread: vec![GraphSource::Design],
                    beads: [("t3-hub.3".to_string(), bead_attr())].into(),
                    runs: [("t3-hub.3-20260927T073916Z".to_string(), run_attr())].into(),
                    invariants: invariant_checks(),
                    skipped: SkippedEdges {
                        design: 0,
                        ledger: 1,
                        design_nodes: 0,
                    },
                },
                // どの出所も読めない。
                GraphDoc {
                    nodes: vec![],
                    edges: vec![],
                    unread: GraphSource::ALL.to_vec(),
                    beads: Default::default(),
                    runs: Default::default(),
                    invariants: vec![InvariantCheck {
                        id: "g-1".into(),
                        verdict: Verdict::Unknown,
                        violations: 0,
                        ids: vec![],
                    }],
                    skipped: SkippedEdges {
                        design: 0,
                        ledger: 0,
                        design_nodes: 0,
                    },
                },
            ],
        ),
        form("graph::EdgeEnd", EdgeEnd::ALL.to_vec()),
        form("graph::ViewNode", view_nodes()),
        form("graph::ViewEdge", vec![view_edge()]),
        form(
            "graph::GraphView",
            vec![
                GraphView {
                    nodes: view_nodes(),
                    edges: vec![view_edge()],
                    shown: 2,
                    folded: 2,
                    cut: 1,
                    total: 5,
                    unread: vec![GraphSource::Design],
                    open: vec!["t3-hub.3".into()],
                    refused: vec![],
                },
                // 節点の無いグラフ。
                GraphView {
                    nodes: vec![],
                    edges: vec![],
                    shown: 0,
                    folded: 0,
                    cut: 0,
                    total: 0,
                    unread: GraphSource::ALL.to_vec(),
                    open: vec![],
                    refused: vec![],
                },
            ],
        ),
        form("graph::BoxFold", BoxFold::ALL.to_vec()),
        form("graph::Fold", Fold::ALL.to_vec()),
        form("graph::AroundRow", around_rows()),
        form("graph::HubCut", vec![hub_cut()]),
        form(
            "graph::AroundDoc",
            vec![
                AroundDoc {
                    center: "t3-hub.3".into(),
                    steps: 2,
                    fold: Fold::Up,
                    rows: around_rows(),
                    basis: 1,
                    impact: 1,
                    shown: 2,
                    total: 32,
                    cut_hub: 30,
                    cut_cap: 0,
                    hubs: vec![hub_cut()],
                    unread: vec![],
                },
                // 中心だけの近傍。
                AroundDoc {
                    center: "t3-hub.3".into(),
                    steps: 1,
                    fold: Fold::Both,
                    rows: vec![around_rows().remove(0)],
                    basis: 0,
                    impact: 0,
                    shown: 1,
                    total: 1,
                    cut_hub: 0,
                    cut_cap: 0,
                    hubs: vec![],
                    unread: vec![GraphSource::Runs],
                },
            ],
        ),
    ]
}

#[test]
fn contract_form_snapshot_matches() {
    common::snapshot_matches("graph", &forms());
}

#[test]
fn contract_form_roundtrip_all_types() {
    common::roundtrip_all(&forms());
}

#[test]
fn contract_form_closed_lists() {
    assert_eq!(distinct(&NodeKind::ALL), 20);
    assert_eq!(distinct(&EdgeType::ALL), 30);
    assert_eq!(distinct(&GraphSource::ALL), 3);
    assert_eq!(distinct(&EdgeEnd::ALL), 2);
    let fold_words: Vec<String> = Fold::ALL
        .iter()
        .map(|f| wire::encode(f).expect("語"))
        .collect();
    assert_eq!(
        fold_words,
        ["\"none\"", "\"up\"", "\"down\"", "\"both\""],
        "畳みの ALL の順と字"
    );
    assert!(wire::decode::<Fold>("\"all\"").is_err());
    let box_words: Vec<String> = BoxFold::ALL
        .iter()
        .map(|f| wire::encode(f).expect("語"))
        .collect();
    assert_eq!(
        box_words,
        ["\"leaf\"", "\"folded\"", "\"open\""],
        "箱の開き閉じの ALL の順と字"
    );
    assert_eq!(distinct(&Verdict::ALL), 3);
}
