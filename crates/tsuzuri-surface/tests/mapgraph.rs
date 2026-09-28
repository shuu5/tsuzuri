//! 便 g-graph の歯: グラフの面の配置（例のグラフ・9 個の升・節点の無い眺め）・線の形と道の字・子の数の札・
//! 拡大と移動の値と transform の字・光らせ方（hlCompute）・固定の移り方（hlPin）・狭い幅の一覧・数の行・
//! 読めなかった出所の帯と 0 の帯・口の読みの 3 値・凡例に出す帯と型・全部の箱の id・口の path・外の依存。

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::graph::{
    BoxFold, EdgeType, GraphNode, GraphSource, GraphView, NodeKind, ViewEdge, ViewNode,
};
use tsuzuri_contract::wire;
use tsuzuri_surface::mapview::band::{BEADS_LANES, Band, unread_reason};
use tsuzuri_surface::mapview::graph::{
    self, BandFill, LineStyle, PATH, PinAction, Pos, REASON, Side, Zoom, band_fill, border,
    box_height, chain, count_line, curve, degrees, doc, edge_path, edge_term, expert_line,
    highlight, initial_scale, kids_badge, layout, legend, line_style, node_svg, pin_next, svg,
    zero_text,
};
use tsuzuri_surface::project::{NO_CONTENT, NOT_READ};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::{label, vocab};

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

/// (1) 例のグラフの配置は節の表と一致する。
#[test]
fn mapgraph_fixture_layout_matches_table() {
    let v = fixture();
    let l = layout(&v);
    assert_eq!((l.width, l.height), (1408, 658));
    let xs: Vec<(u32, u32, u32)> = l.columns.iter().map(|c| (c.rank, c.x, c.cols)).collect();
    assert_eq!(
        xs,
        vec![
            (0, 116, 1),
            (1, 336, 1),
            (2, 556, 1),
            (3, 776, 1),
            (4, 996, 1),
            (5, 1216, 1)
        ]
    );
    let bands: Vec<(Band, u32, u32)> = l.bands.iter().map(|b| (b.band, b.top, b.height)).collect();
    assert_eq!(
        bands,
        vec![
            (Band::Constitution, 8, 66),
            (Band::Rules, 74, 66),
            (Band::Adr, 140, 66),
            (Band::Srs, 206, 66),
            (Band::DesignNote, 272, 44),
            (Band::Beads, 316, 272),
            (Band::Pipeline, 588, 66),
        ]
    );
    let lanes: Vec<(NodeKind, u32, u32)> = l
        .band(Band::Beads)
        .expect("beads の帯")
        .lanes
        .iter()
        .map(|x| (x.kind, x.top, x.height))
        .collect();
    assert_eq!(
        lanes,
        vec![
            (NodeKind::Epic, 316, 66),
            (NodeKind::Task, 382, 66),
            (NodeKind::Memo, 448, 28),
            (NodeKind::Question, 476, 28),
            (NodeKind::Ruling, 504, 28),
            (NodeKind::Receipt, 532, 28),
            (NodeKind::Policy, 560, 28),
        ]
    );
    assert_eq!(
        lanes.iter().map(|x| x.0).collect::<Vec<_>>(),
        BEADS_LANES.to_vec()
    );
    for band in Band::ALL {
        if band != Band::Beads {
            assert!(l.band(band).expect("帯").lanes.is_empty(), "{band:?}");
        }
    }
    let boxes: [(&str, u32, u32); 8] = [
        ("P-1", 116, 18),
        ("R-1", 336, 84),
        ("ADR-1", 336, 150),
        ("FR1", 556, 216),
        ("AC1", 776, 216),
        ("e", 776, 326),
        ("e.1", 996, 392),
        (RUN, 1216, 598),
    ];
    for (id, x, y) in boxes {
        assert_eq!(l.pos(id), Some(Pos { x, y }), "{id} の箱");
    }
    assert_eq!(l.boxes.len(), 8);
}

/// (2) 1 つの升に 9 個: 9 個目は 2 列目の先頭・その段は 2 列ぶん・次の段は 184 右へずれる。
#[test]
fn mapgraph_nine_in_one_cell_wrap_to_second_column() {
    let mut nodes: Vec<ViewNode> = (1..=9)
        .map(|i| node(&format!("FR{i}"), NodeKind::Req, 0, 1))
        .collect();
    nodes.push(node("AC1", NodeKind::Ac, 1, 1));
    let one = layout(&view_of(
        vec![
            node("FR1", NodeKind::Req, 0, 1),
            node("AC1", NodeKind::Ac, 1, 1),
        ],
        vec![],
    ));
    let l = layout(&view_of(nodes, vec![]));
    let first = l.pos("FR1").expect("FR1");
    let ninth = l.pos("FR9").expect("FR9");
    assert_eq!(
        first,
        Pos {
            x: 116,
            y: 8 + 44 * 3 + 10
        }
    );
    assert_eq!(
        ninth,
        Pos {
            x: first.x + 184,
            y: first.y
        }
    );
    // 升の中は id の自然な順（FR2 は FR1 の下）。
    assert_eq!(
        l.pos("FR2"),
        Some(Pos {
            x: 116,
            y: first.y + 46
        })
    );
    assert_eq!(
        l.pos("FR8"),
        Some(Pos {
            x: 116,
            y: first.y + 7 * 46
        })
    );
    assert_eq!(l.column(0).map(|c| c.cols), Some(2));
    let next = l.column(1).expect("段 1").x;
    assert_eq!(next, one.column(1).expect("段 1").x + 184);
    assert_eq!(next, 116 + 2 * 184 + 36);
    assert_eq!(l.width, next + 184 + 8);
    // 行の高さは 8 個まで（8 × 46 + 20）。
    assert_eq!(l.band(Band::Srs).map(|b| b.height), Some(8 * 46 + 20));
}

/// (3) 節点が 1 つも無い眺め: 7 つの帯は全部 44・幅 300・高さ 320。
#[test]
fn mapgraph_empty_view_layout() {
    let l = layout(&view_of(vec![], vec![]));
    assert!(l.bands.iter().all(|b| b.height == 44 && b.lanes.is_empty()));
    assert_eq!(l.bands.len(), 7);
    assert_eq!((l.width, l.height), (300, 320));
    assert!(l.columns.is_empty());
    assert!(l.boxes.is_empty());
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
            EdgeType::RanBy
        ]
    );
    assert_eq!(by["arrow"], vec![EdgeType::Amends]);
    assert_eq!(by["solid"].len(), 25);
    assert_eq!(by.values().map(Vec::len).sum::<usize>(), 32);
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

/// (5) 線の道の字（FR1 から P-1 の辺）。
#[test]
fn mapgraph_edge_path_text() {
    let v = fixture();
    let l = layout(&v);
    let e = v
        .edges
        .iter()
        .find(|e| e.from == "FR1" && e.to == "P-1")
        .expect("FR1 → P-1");
    assert_eq!(
        edge_path(&l, e).as_deref(),
        Some("M556 236 C 424 236, 424 38, 292 38")
    );
    assert_eq!(curve(1.0, 2.0, 4.0, 6.0), "M1 2 C 2.5 2, 2.5 6, 4 6");
    // 全部の辺が図の道を持つ（両端の箱が在る）。
    assert!(v.edges.iter().all(|e| edge_path(&l, e).is_some()));
    let picture = svg(&v, &l);
    assert!(picture.contains(r#"d="M556 236 C 424 236, 424 38, 292 38""#));
    assert_eq!(picture.matches(r#"<g class="e""#).count(), 10);
}

/// (6) 子を持つ節点だけが子の数の札を持つ・(16) 全部の箱に id が出る。
#[test]
fn mapgraph_kids_badge_and_ids_in_every_box() {
    let v = fixture();
    let l = layout(&v);
    let children = label("children");
    for n in &v.nodes {
        let badge = kids_badge(n);
        match n.node.id.as_str() {
            "P-1" | "e" => assert_eq!(badge, Some(format!("{children} 2")), "{}", n.node.id),
            _ => assert_eq!(badge, None, "{}", n.node.id),
        }
        let p = l.pos(&n.node.id).expect("箱");
        let s = node_svg(n, p);
        assert!(
            s.contains(&format!(">{}</text>", n.node.id)),
            "{} の箱に id が無い",
            n.node.id
        );
        assert_eq!(s.contains(&format!("{children} ")), badge.is_some());
        // 箱は link にしない。
        assert!(!s.contains("href") && !s.contains(r#"role="link""#));
    }
    let picture = svg(&v, &l);
    assert_eq!(picture.matches(r#"<g class="node""#).count(), 8);
    for n in &v.nodes {
        assert!(picture.contains(&format!(r#"data-key="{}""#, n.node.id)));
    }
    // 形は 2 種（file に書かれた行は四角・台帳と走行は丸）。
    let p1 = node_svg(&v.nodes[0], l.pos("P-1").expect("P-1"));
    assert!(!p1.contains("<circle"));
    let run = v.nodes.iter().find(|n| n.node.id == RUN).expect("走行");
    assert!(node_svg(run, l.pos(RUN).expect("走行")).contains("<circle"));
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
    let s = node_svg(&q, Pos { x: 116, y: 18 });
    assert!(s.contains(r#"fill="none" stroke="var(--s-stop)""#));
    assert!(s.contains(r#"stroke="var(--s-stop)" stroke-width="2""#));
    let mut c = node("c", NodeKind::Task, 0, 1);
    c.status = Some("closed".to_string());
    assert!(graph::closed(&c));
    assert!(node_svg(&c, Pos { x: 116, y: 18 }).contains(r#"fill="var(--ink-3)" data-t="""#));
    let mut r = node("r", NodeKind::Rule, 0, 1);
    r.status = Some("closed".to_string());
    assert!(!graph::closed(&r));
}

/// (7) 拡大と移動の値と transform の字。
#[test]
fn mapgraph_zoom_values() {
    let s0 = initial_scale(1000.0, 1408);
    assert!((s0 - 884.0 / 1292.0).abs() < 1e-12);
    assert_eq!((s0 * 10000.0).round() / 10000.0, 0.6842);
    assert_eq!(box_height(658, s0), 458.0);
    assert_eq!(initial_scale(2000.0, 1408), 1.0);
    assert_eq!(box_height(100, 1.0), 360.0);
    assert_eq!(box_height(2000, 1.0), 720.0);
    assert_eq!(
        Zoom {
            s: 0.5,
            tx: 10.0,
            ty: -4.0
        }
        .transform(),
        "translate(126.00 -4.00) scale(0.5000) translate(-116 0)"
    );
    assert_eq!(
        Zoom::initial(s0).transform(),
        "translate(116.00 0.00) scale(0.6842) translate(-116 0)"
    );

    // wheel: 倍率は e の（縦の量 × −0.0015）乗を掛け、pointer の下の点は動かない。
    let z = Zoom {
        s: s0,
        tx: 30.0,
        ty: -12.0,
    };
    let (cx, cy) = (400.0, 200.0);
    let before = z.to_graph(cx, cy);
    let w = z.wheel(cx, cy, -100.0);
    assert!((w.s - s0 * 0.15_f64.exp()).abs() < 1e-12);
    let after = w.to_graph(cx, cy);
    assert!((before.0 - after.0).abs() < 1e-9 && (before.1 - after.1).abs() < 1e-9);
    // 0.2 から 4 に収まる（収めても pointer の下の点は動かない）。
    let big = z.wheel(cx, cy, -100_000.0);
    assert_eq!(big.s, 4.0);
    let small = z.wheel(cx, cy, 100_000.0);
    assert_eq!(small.s, 0.2);
    for zz in [big, small] {
        let p = zz.to_graph(cx, cy);
        assert!((p.0 - before.0).abs() < 1e-9 && (p.1 - before.1).abs() < 1e-9);
    }
    // drag は移動の量に足す・元に戻すは最初の倍率と移動 0。
    assert_eq!(
        z.drag(5.0, -3.0),
        Zoom {
            s: s0,
            tx: 35.0,
            ty: -15.0
        }
    );
    assert_eq!(
        Zoom::initial(s0),
        Zoom {
            s: s0,
            tx: 0.0,
            ty: 0.0
        }
    );
    assert!(!graph::dragged(3.0, 0.0));
    assert!(!graph::dragged(0.0, 4.0));
    assert!(graph::dragged(3.0, 3.0));
    // 帯の名は左端の欄に・拡大に合わせて縦の位置と高さだけが変わる。
    let l = layout(&fixture());
    let labels = graph::band_labels(&l, Zoom::initial(0.5), 458.0);
    assert!(labels.contains(r#"<g class="blab" data-band="rules""#));
    assert!(labels.contains(r#"y="37.0" width="116" height="33.0""#));
    let moved = graph::band_labels(
        &l,
        Zoom {
            s: 0.5,
            tx: 50.0,
            ty: 10.0,
        },
        458.0,
    );
    assert!(moved.contains(r#"y="47.0" width="116" height="33.0""#));
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

/// (10) 狭い幅の一覧。
#[test]
fn mapgraph_narrow_chain_order() {
    let mut v = fixture();
    v.nodes.push(node("FR10", NodeKind::Req, 2, 1));
    v.nodes.push(node("FR2", NodeKind::Req, 2, 1));
    v.nodes.push(node("G-1", NodeKind::Goal, 1, 1));
    let c = chain(&v);
    let bands: Vec<Band> = c.iter().map(|b| b.band).collect();
    assert_eq!(
        bands,
        vec![
            Band::Constitution,
            Band::Rules,
            Band::Adr,
            Band::Srs,
            Band::Beads,
            Band::Pipeline
        ]
    );
    let srs: Vec<&str> = c[3].rows.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(srs, vec!["G-1", "FR1", "FR2", "FR10", "AC1"]);
    let beads: Vec<(&str, Option<u32>)> =
        c[4].rows.iter().map(|r| (r.id.as_str(), r.kids)).collect();
    assert_eq!(beads, vec![("e", Some(2)), ("e.1", None)]);
    assert_eq!(c[0].rows[0].kids, Some(2));
    assert_eq!(c[4].rows[0].shape, "shape band-beads");
    assert_eq!(c[0].rows[0].shape, "shape band-constitution fill");
    assert_eq!(c[0].rows[0].title, "根拠の条");
}

/// (11) 数の行と経験者向けの 1 行。
#[test]
fn mapgraph_count_lines() {
    let v = fixture();
    assert_eq!(
        count_line(&v),
        format!("8 / 13 ・ {} 4 ・ ✂ 1", label("children"))
    );
    assert_eq!(count_line(&v), "8 / 13 ・ 子 4 ・ ✂ 1");
    let line = expert_line(&v);
    assert_eq!(line, "shown=8 folded=4 cut=1 total=13");
    assert!(line.chars().count() <= 60);
}

/// (12) 読めなかった出所の帯は測れていない・読めて節点の無い帯は 0。
#[test]
fn mapgraph_unread_band_and_zero_band() {
    let mut v = fixture();
    assert_eq!(band_fill(&v, Band::DesignNote), BandFill::Zero);
    assert_eq!(band_fill(&v, Band::Pipeline), BandFill::Nodes(1));
    assert_eq!(
        zero_text(Band::DesignNote),
        format!("{} 0", label("b:design-note"))
    );
    let l = layout(&v);
    assert!(svg(&v, &l).contains(&format!(">{}</text>", zero_text(Band::DesignNote))));

    v.nodes.retain(|n| n.node.kind != NodeKind::Run);
    v.unread = vec![GraphSource::Runs];
    let reason = unread_reason(GraphSource::Runs);
    assert_eq!(band_fill(&v, Band::Pipeline), BandFill::Unread(reason));
    let picture = svg(&v, &layout(&v));
    assert!(picture.contains(reason));
    assert!(picture.contains(&label("st_unknown")));
    assert!(!picture.contains(&zero_text(Band::Pipeline)));

    v.unread = vec![GraphSource::Design];
    for b in &Band::ALL[..5] {
        assert_eq!(
            band_fill(&v, *b),
            BandFill::Unread(unread_reason(GraphSource::Design))
        );
    }
    assert_eq!(band_fill(&v, Band::Beads), BandFill::Nodes(2));
    v.unread = vec![GraphSource::Ledger];
    assert_eq!(
        band_fill(&v, Band::Beads),
        BandFill::Unread(unread_reason(GraphSource::Ledger))
    );
}

/// (13) 口が読めない・まだ読んでいない・本文が電文の型として読めないは、面の全体が測れていない。
#[test]
fn mapgraph_read_three_values() {
    assert_eq!(doc(&Fetched::NotRead), Err(NOT_READ));
    assert_eq!(doc(&Fetched::Failed), Err(REASON));
    for text in ["{}", "not json", "[]", r#"{"nodes": []}"#] {
        assert_eq!(
            doc(&Fetched::Body(text.to_string())),
            Err(NO_CONTENT),
            "{text}"
        );
    }
    assert_eq!(doc(&Fetched::Body(fixture_text())), Ok(fixture()));
    assert!(!REASON.trim().is_empty() && !REASON.contains('\n'));
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
    // 注釈の属性は語の辞書に鍵の在る型だけ（20 個）。
    let with_term: Vec<EdgeType> = EdgeType::ALL
        .into_iter()
        .filter(|t| edge_term(*t).is_some())
        .collect();
    assert_eq!(with_term.len(), 20);
    assert_eq!(edge_term(EdgeType::Basis).as_deref(), Some("e:basis"));
    assert_eq!(edge_term(EdgeType::VerifyAc), None);
    for t in with_term {
        let key = edge_term(t).expect("鍵");
        assert!(vocab().term(&key).is_some());
    }
    assert_eq!(legend(&view_of(vec![], vec![])).bands, Vec::<Band>::new());
    for key in [
        "lines",
        "zoom_reset",
        "zoom_hint",
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
        "view_graph",
        "st_unknown",
    ] {
        assert!(vocab().term(key).is_some(), "鍵 {key} が vocab に無い");
    }
}

/// (17) 口の path・(18) 外の依存は足さない。
#[test]
fn mapgraph_path_and_no_new_dependency() {
    assert_eq!(PATH, "/api/graph/view");
    let text = read("src/mapview/graph.rs");
    assert_eq!(text.matches("\"/api/graph/view\"").count(), 1);
    let manifest = read("Cargo.toml");
    let mut inside = false;
    let mut deps = Vec::new();
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line == "[dependencies]";
        } else if inside
            && !line.starts_with('#')
            && let Some((k, _)) = line.split_once('=')
        {
            deps.push(k.trim().to_string());
        }
    }
    assert_eq!(deps, vec!["tsuzuri-contract".to_string()]);
    // 画面の code に節点を選ぶ分岐と数え直しを書かない（電文の数の欄をそのまま出す）。
    for word in [".len() as", "retain(", "shown =", "total ="] {
        assert!(!text.contains(word), "グラフの module に {word} が在る");
    }
}
