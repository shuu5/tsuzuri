//! 便 g-graph の歯: 線の形の表・箱の縁と薄い字・光らせ方（hlCompute）・固定の移り方（hlPin）・凡例に出す帯と型。
//! グラフの面の配置・線の道の字・子の数の札・拡大と移動・狭い幅の一覧・数の行・読めなかった出所の帯と 0 の帯・
//! 口の読みの 3 値・全部の箱の id・口の path と外の依存の歯は行 m-map-graph で消した（近傍の図の歯は nodepage.rs）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::graph::{
    BoxFold, EdgeType, GraphNode, GraphView, NodeKind, ViewEdge, ViewNode,
};
use tsuzuri_contract::wire;
use tsuzuri_surface::mapview::band::Band;
use tsuzuri_surface::mapview::graph::{
    self, LineStyle, PinAction, Side, border, degrees, edge_term, highlight, legend, line_style,
    pin_next,
};
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture_text() -> String {
    read("../../tests/fixtures/surface/graph-view.json")
}

/// 例のグラフの眺め（契約の型の GraphView として読めなければ落ちる）。
fn fixture() -> GraphView {
    wire::decode(&fixture_text()).expect("fixture が眺めの電文として読める")
}

const RUN: &str = "e.1-20260927T000000Z";

fn node(id: &str, kind: NodeKind, rank: u32, degree: u32) -> ViewNode {
    ViewNode {
        node: GraphNode {
            id: id.to_string(),
            kind,
            file: None,
            digest: None,
            title: format!("{id} の題"),
            line: None,
            plain: None,
            eng: None,
            updated: None,
        },
        status: None,
        rank,
        kids: 0,
        degree,
        group: false,
        fold: BoxFold::Leaf,
    }
}

fn edge(from: &str, to: &str, t: EdgeType) -> ViewEdge {
    ViewEdge {
        from: from.to_string(),
        to: to.to_string(),
        edge_type: t,
        count: 1,
    }
}

fn view_of(nodes: Vec<ViewNode>, edges: Vec<ViewEdge>) -> GraphView {
    let n = u32::try_from(nodes.len()).expect("数");
    GraphView {
        nodes,
        edges,
        shown: n,
        folded: 0,
        cut: 0,
        total: n,
        unread: vec![],
        open: vec![],
        refused: vec![],
    }
}

#[test]
fn mapgraph_fixture_is_small_and_reads() {
    let text = fixture_text();
    assert!(text.len() <= 5000, "fixture が {} byte", text.len());
    let v = fixture();
    assert_eq!(v.nodes.len(), 8);
    assert_eq!(v.edges.len(), 10);
    assert!(v.edges.iter().all(|e| e.count == 1));
    assert_eq!((v.shown, v.folded, v.cut, v.total), (8, 4, 1, 13));
    assert!(v.unread.is_empty());
    let want: [(&str, NodeKind, u32, u32, u32); 8] = [
        ("P-1", NodeKind::Article, 0, 2, 5),
        ("R-1", NodeKind::Rule, 1, 0, 1),
        ("FR1", NodeKind::Req, 2, 0, 4),
        ("AC1", NodeKind::Ac, 3, 0, 1),
        ("ADR-1", NodeKind::Adr, 1, 0, 2),
        ("e", NodeKind::Epic, 3, 2, 2),
        ("e.1", NodeKind::Task, 4, 0, 2),
        (RUN, NodeKind::Run, 5, 0, 1),
    ];
    for (id, kind, rank, kids, degree) in want {
        let n = v.nodes.iter().find(|n| n.node.id == id).expect(id);
        assert_eq!(
            (n.node.kind, n.rank, n.kids, n.degree),
            (kind, rank, kids, degree),
            "{id}"
        );
        let status = match kind {
            NodeKind::Epic | NodeKind::Task => Some("open"),
            NodeKind::Run => Some("Landed"),
            _ => None,
        };
        assert_eq!(n.status.as_deref(), status, "{id} の状態");
    }
}

/// (4) 線の形は辺の型から決まる閉じた表。
#[test]
fn mapgraph_line_style_table() {
    let mut by: BTreeMap<&str, Vec<EdgeType>> = BTreeMap::new();
    for t in EdgeType::ALL {
        let s = line_style(t);
        assert!(LineStyle::ALL.contains(&s));
        let name = match s {
            LineStyle::Solid => "solid",
            LineStyle::Bold => "bold",
            LineStyle::Dotted => "dotted",
            LineStyle::Arrow => "arrow",
        };
        by.entry(name).or_default().push(t);
    }
    assert_eq!(by["bold"], vec![EdgeType::Blocks]);
    assert_eq!(
        by["dotted"],
        vec![
            EdgeType::InArticle,
            EdgeType::ArticleRef,
            EdgeType::ParentChild,
            EdgeType::RunOf,
            EdgeType::RanBy,
            EdgeType::Landed
        ]
    );
    assert_eq!(by["arrow"], vec![EdgeType::Amends]);
    assert_eq!(by["solid"].len(), 25);
    assert_eq!(by.values().map(Vec::len).sum::<usize>(), 33);
    // 見た目の値は見本の edgeSVG と同じ。
    assert_eq!(
        LineStyle::Bold.stroke(),
        r#"stroke="var(--s-stop)" stroke-width="3""#
    );
    assert_eq!(
        LineStyle::Dotted.stroke(),
        r#"stroke="var(--ink-3)" stroke-width="1.4" stroke-dasharray="2 3""#
    );
    assert_eq!(
        LineStyle::Arrow.stroke(),
        r#"stroke="var(--st-limit)" stroke-width="2""#
    );
    assert_eq!(
        LineStyle::Solid.stroke(),
        r#"stroke="var(--ink-3)" stroke-width="1.2""#
    );
}

/// 縁: open の問いは止まりの色・固定は太い縁・閉じた bead は薄い字。
#[test]
fn mapgraph_node_border_and_faded() {
    assert_eq!(border(false, false), ("var(--line-2)", 1));
    assert_eq!(border(true, false), ("var(--s-stop)", 2));
    assert_eq!(border(false, true), ("var(--ink)", 3));
    assert_eq!(border(true, true), ("var(--s-stop)", 3));
    let mut q = node("q", NodeKind::Question, 0, 1);
    q.status = Some("open".to_string());
    assert!(graph::open_question(&q));
    let mut c = node("c", NodeKind::Task, 0, 1);
    c.status = Some("closed".to_string());
    assert!(graph::closed(&c));
    let mut r = node("r", NodeKind::Rule, 0, 1);
    r.status = Some("closed".to_string());
    assert!(!graph::closed(&r));
}

fn fr1() -> graph::Highlight {
    let v = fixture();
    highlight(&v.edges, &degrees(&v), "FR1")
}

/// (8) 光らせ方（例のグラフの FR1・往復の辺・hub・20 を超える入力）。
#[test]
fn mapgraph_highlight_fixture_fr1() {
    let v = fixture();
    let h = fr1();
    assert_eq!(h.basis(), vec!["ADR-1", "P-1"]);
    assert_eq!(h.impact(), vec!["AC1", "e", "e.1"]);
    assert_eq!(h.depth, 2);
    assert_eq!(h.side.get("FR1"), Some(&Side::Center));
    assert_eq!(h.lit.len(), 7);
    let dark: Vec<(&str, &str)> = v
        .edges
        .iter()
        .enumerate()
        .filter(|(i, _)| !h.lit.contains(i))
        .map(|(_, e)| (e.from.as_str(), e.to.as_str()))
        .collect();
    assert_eq!(dark, vec![("R-1", "P-1"), ("R-1", "P-1"), (RUN, "e.1")]);
    assert!(!h.side.contains_key("R-1") && !h.side.contains_key(RUN));
}

#[test]
fn mapgraph_highlight_round_trip_lights_first_only() {
    let edges = vec![
        edge("FR1", "ADR-1", EdgeType::Adrs),
        edge("ADR-1", "FR1", EdgeType::Basis),
    ];
    let degree = BTreeMap::from([("FR1".to_string(), 2), ("ADR-1".to_string(), 2)]);
    let h = highlight(&edges, &degree, "FR1");
    assert_eq!(h.side.get("ADR-1"), Some(&Side::Basis));
    assert_eq!(h.basis(), vec!["ADR-1"]);
    assert!(h.impact().is_empty());
    assert_eq!(h.lit, vec![0]);
}

#[test]
fn mapgraph_highlight_hub_and_cap() {
    // hub（次数 9）は光るが、その先へは広げない。
    let edges = vec![
        edge("c", "hub", EdgeType::Basis),
        edge("hub", "far", EdgeType::Basis),
        edge("c", "b", EdgeType::Basis),
        edge("b", "far2", EdgeType::Basis),
    ];
    let degree = BTreeMap::from([
        ("c".to_string(), 2),
        ("hub".to_string(), 9),
        ("b".to_string(), 8),
        ("far".to_string(), 1),
        ("far2".to_string(), 1),
    ]);
    let h = highlight(&edges, &degree, "c");
    assert_eq!(h.basis(), vec!["hub", "b", "far2"]);
    assert!(!h.side.contains_key("far"));
    // 中心は次数が多くても広げる。
    let mut hub_center = degree.clone();
    hub_center.insert("c".to_string(), 50);
    assert_eq!(highlight(&edges, &hub_center, "c").basis().len(), 3);

    // 2 段で 25 個（5 × (1 + 4)）→ 段を 1 に落として 5 個。
    let mut edges = Vec::new();
    for i in 0..5 {
        edges.push(edge("c", &format!("a{i}"), EdgeType::Basis));
        for j in 0..4 {
            edges.push(edge(
                &format!("a{i}"),
                &format!("a{i}-{j}"),
                EdgeType::Basis,
            ));
        }
    }
    let h = highlight(&edges, &BTreeMap::new(), "c");
    assert_eq!(h.depth, 1);
    assert_eq!(h.order.len(), 5);
    assert_eq!(h.lit.len(), 5);

    cap_at_twenty();
}

/// 1 段でも 20 を超えれば着いた順の先頭の 20 個で切る。
fn cap_at_twenty() {
    // 1 段でも 21 個 → 着いた順の先頭の 20 個で切る。
    let edges: Vec<ViewEdge> = (0..21)
        .map(|i| edge(&format!("d{i}"), "c", EdgeType::Touches))
        .collect();
    let h = highlight(&edges, &BTreeMap::new(), "c");
    assert_eq!(h.depth, 1);
    assert_eq!(h.order.len(), 20);
    assert_eq!(h.side.len(), 21, "中心と 20 個");
    assert_eq!(h.order.last().map(String::as_str), Some("d19"));
    assert!(!h.side.contains_key("d20"));
    assert_eq!(h.lit, (0..20).collect::<Vec<_>>());
    assert_eq!(graph::LIT_MAX, 20);
    assert_eq!(graph::HUB_DEGREE, 8);
}

/// (9) 固定の帯の値と固定の移り方。
#[test]
fn mapgraph_pin_transitions_and_bar() {
    let bar = fr1().bar();
    assert_eq!(bar.id, "FR1");
    assert_eq!((bar.basis, bar.impact, bar.depth), (2, 3, 2));
    let press = |id: &str| PinAction::Press(id.to_string());
    let pinned = pin_next(None, &press("FR1"));
    assert_eq!(pinned.as_deref(), Some("FR1"));
    assert_eq!(pin_next(pinned.as_deref(), &press("FR1")), None);
    assert_eq!(pin_next(pinned.as_deref(), &PinAction::Escape), None);
    assert_eq!(pin_next(pinned.as_deref(), &PinAction::Unpin), None);
    assert_eq!(
        pin_next(pinned.as_deref(), &press("P-1")).as_deref(),
        Some("P-1")
    );
    assert_eq!(pin_next(None, &PinAction::Escape), None);
}

/// (14) 凡例は図に出ている帯（6 つ）と辺の型（9 つ）だけ。
#[test]
fn mapgraph_legend_bands_and_types() {
    let lg = legend(&fixture());
    assert_eq!(
        lg.bands,
        vec![
            Band::Constitution,
            Band::Rules,
            Band::Adr,
            Band::Srs,
            Band::Beads,
            Band::Pipeline
        ]
    );
    assert_eq!(lg.types.len(), 9);
    assert_eq!(
        lg.types,
        vec![
            EdgeType::RelationsRules,
            EdgeType::ArticleRef,
            EdgeType::Basis,
            EdgeType::Adrs,
            EdgeType::VerifyAc,
            EdgeType::Verifies,
            EdgeType::ParentChild,
            EdgeType::Touches,
            EdgeType::RunOf
        ]
    );
    // 注釈の属性は語の辞書に鍵の在る型だけ（21 個）。
    let with_term: Vec<EdgeType> = EdgeType::ALL
        .into_iter()
        .filter(|t| edge_term(*t).is_some())
        .collect();
    assert_eq!(with_term.len(), 21);
    assert_eq!(edge_term(EdgeType::Basis).as_deref(), Some("e:basis"));
    assert_eq!(edge_term(EdgeType::VerifyAc), None);
    for t in with_term {
        let key = edge_term(t).expect("鍵");
        assert!(vocab().term(&key).is_some());
    }
    assert_eq!(legend(&view_of(vec![], vec![])).bands, Vec::<Band>::new());
    for key in [
        "children",
        "cut",
        "pinned",
        "nb_up",
        "nb_down",
        "unpin",
        "lg_shape",
        "lg_color",
        "lg_border",
        "lg_hover",
        "st_unknown",
    ] {
        assert!(vocab().term(key).is_some(), "鍵 {key} が vocab に無い");
    }
}
