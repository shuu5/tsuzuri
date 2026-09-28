//! グラフの眺めを組の箱で畳む歯（行 c-graph-fold・接頭辞 gtuck_）: 電文の既定・例のグラフの段・
//! 古い開きの畳み直し・帯の畳み直し・断りと捨てる id・塊の id・実物の見本の不変の値・開く列の字・純さ・自分の名。
//! グラフは歯の中で直に組む（実物の見本だけ fixture の字を build に渡す）。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tsuzuri_contract::graph::{
    BoxFold, EdgeType, GraphEdge, GraphNode, GraphView, NodeKind, ViewNode,
};
use tsuzuri_contract::wire;
use tsuzuri_core::graph::fold::{CHUNK, open_list};
use tsuzuri_core::graph::view::VIEW_CAP;
use tsuzuri_core::graph::{BeadAttr, Graph, Inputs, RunAttr, build, view, view_open};

const RULING: &str = "e.2:20260927T0000Z-1";
const RUN: &str = "e.1-20260927T000000Z";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn node(id: &str, kind: NodeKind) -> GraphNode {
    GraphNode {
        id: id.into(),
        kind,
        file: None,
        digest: None,
        title: id.into(),
        line: None,
        plain: None,
        eng: None,
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

fn ids(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// 例のグラフ（13 節点・16 辺・歯の file gview.rs と同じ）。
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

/// epic `epic` と、その子の task `<epic>.1`〜`<epic>.<n>` を足す。
fn add_epic(g: &mut Graph, epic: &str, n: usize) {
    g.nodes.push(node(epic, NodeKind::Epic));
    for i in 1..=n {
        let task = format!("{epic}.{i}");
        g.nodes.push(node(&task, NodeKind::Task));
        g.edges.push(edge(&task, epic, EdgeType::ParentChild));
    }
}

/// 子 619 の epic の見本（h と task h.1〜h.619 と条 A-1 と h.1 の走行 1 つ）。
fn hub() -> Graph {
    let mut g = Graph {
        nodes: vec![node("A-1", NodeKind::Article)],
        ..Graph::default()
    };
    add_epic(&mut g, "h", 619);
    g.nodes.push(node("h.1-r1", NodeKind::Run));
    g.edges.push(edge("h.1-r1", "h.1", EdgeType::RunOf));
    g
}

/// 眺めの箱の (id・畳み・子の数)。
fn boxes(v: &GraphView) -> Vec<(&str, BoxFold, u32)> {
    v.nodes
        .iter()
        .map(|n| (n.node.id.as_str(), n.fold, n.kids))
        .collect()
}

fn find<'v>(v: &'v GraphView, id: &str) -> &'v ViewNode {
    v.nodes
        .iter()
        .find(|n| n.node.id == id)
        .unwrap_or_else(|| panic!("箱 {id} が無い: {:?}", boxes(v)))
}

#[test]
fn gtuck_wire_defaults() {
    let node = r#"{"node":{"id":"x","kind":"条","file":null,"digest":null,"title":"x"},"status":null,"rank":0,"kids":0,"degree":0}"#;
    let n: ViewNode = wire::decode(node).expect("欄 group と fold の無い節点の字");
    assert_eq!((n.group, n.fold), (false, BoxFold::Leaf));
    let text = format!(
        r#"{{"nodes":[{node}],"edges":[],"shown":1,"folded":0,"cut":0,"total":1,"unread":[]}}"#
    );
    let v: GraphView = wire::decode(&text).expect("欄 open と refused の無い眺めの字");
    assert!(v.open.is_empty() && v.refused.is_empty());
    assert_eq!(BoxFold::default(), BoxFold::Leaf);
    let words: Vec<String> = BoxFold::ALL
        .iter()
        .map(|f| wire::encode(f).expect("語"))
        .collect();
    assert_eq!(words, ["\"leaf\"", "\"folded\"", "\"open\""]);
}

#[test]
fn gtuck_levels_example() {
    let asks = ids(&[
        "~art:P",
        "P-1",
        "~srs:req",
        "FR1",
        "e",
        "e.1",
        "~rule",
        "~adr",
        "~srs:actor",
    ]);
    let v = view_open(&example(), &asks);
    use BoxFold::*;
    let got: Vec<(&str, u32, u32, u32, BoxFold)> = v
        .nodes
        .iter()
        .map(|n| (n.node.id.as_str(), n.rank, n.kids, n.degree, n.fold))
        .collect();
    assert_eq!(
        got,
        vec![
            ("P-1", 0, 0, 5, Open),
            ("~art:P", 0, 0, 0, Open),
            ("P-1.1", 1, 0, 1, Leaf),
            ("P-1.2", 1, 0, 1, Leaf),
            ("R-1", 1, 0, 1, Leaf),
            ("~rule", 0, 0, 0, Open),
            ("FR1", 2, 0, 4, Open),
            ("~srs:req", 0, 0, 0, Open),
            ("AC1", 3, 0, 1, Leaf),
            ("ACT1", 0, 0, 0, Leaf),
            ("~srs:actor", 0, 0, 0, Open),
            ("ADR-1", 1, 0, 2, Leaf),
            ("~adr", 0, 0, 0, Open),
            ("e", 3, 2, 2, Open),
            ("e.1", 4, 0, 2, Open),
            (RUN, 5, 0, 1, Leaf),
        ]
    );
    use EdgeType::*;
    let edges: Vec<(&str, &str, EdgeType, u32)> = v
        .edges
        .iter()
        .map(|e| (e.from.as_str(), e.to.as_str(), e.edge_type, e.count))
        .collect();
    assert_eq!(
        edges,
        vec![
            ("AC1", "FR1", VerifyAc, 1),
            ("AC1", "FR1", Verifies, 1),
            ("ADR-1", "P-1", Basis, 1),
            ("FR1", "ADR-1", Adrs, 1),
            ("FR1", "P-1", Basis, 1),
            ("P-1.1", "P-1", InArticle, 2),
            ("P-1.2", "P-1", InArticle, 2),
            ("R-1", "P-1", RelationsRules, 1),
            ("R-1", "P-1", ArticleRef, 1),
            ("e", "FR1", Touches, 1),
            ("e.1", "e", ParentChild, 1),
            (RUN, "e.1", RunOf, 1),
        ]
    );
    assert_eq!((v.shown, v.folded, v.cut, v.total), (16, 2, 0, 13));
    assert_eq!(v.open, asks);
    assert!(v.refused.is_empty());
}

#[test]
fn gtuck_refold_oldest() {
    let mut g = Graph::default();
    for i in 1..=30 {
        g.nodes.push(node(&format!("R-{i}"), NodeKind::Rule));
        g.nodes.push(node(&format!("ADR-{i}"), NodeKind::Adr));
        g.nodes.push(node(&format!("FR{i}"), NodeKind::Req));
    }
    let asks = ids(&[
        "~rule",
        "~rule~1-12",
        "~adr",
        "~adr~1-12",
        "~srs:req",
        "~srs:req~1-12",
    ]);
    let v = view_open(&g, &asks);
    assert_eq!(
        v.open,
        ids(&["~adr", "~adr~1-12", "~srs:req", "~srs:req~1-12"])
    );
    let rule = find(&v, "~rule");
    assert_eq!((rule.fold, rule.kids), (BoxFold::Folded, 30));
    assert_eq!(v.shown, 33);
    assert!(v.refused.is_empty());
}

#[test]
fn gtuck_bands_make_room() {
    let mut g = Graph::default();
    for (id, kind) in [
        ("A-1", NodeKind::Article),
        ("N-1", NodeKind::Article),
        ("P-1", NodeKind::Article),
        ("G-1", NodeKind::Goal),
        ("FR1", NodeKind::Req),
        ("NFR1", NodeKind::Nfr),
        ("AC1", NodeKind::Ac),
        ("CON1", NodeKind::Constraint),
        ("ACT1", NodeKind::Actor),
        ("OUT1", NodeKind::Output),
    ] {
        g.nodes.push(node(id, kind));
    }
    add_epic(&mut g, "E", 144);
    for i in 1..=12 {
        let run = format!("E.1-r{i}");
        g.nodes.push(node(&run, NodeKind::Run));
        g.edges.push(edge(&run, "E.1", EdgeType::RunOf));
    }
    assert_eq!(g.nodes.len(), 167);

    let first = view(&g);
    let shown: Vec<&str> = first.nodes.iter().map(|n| n.node.id.as_str()).collect();
    assert_eq!(
        shown,
        vec![
            "~art:A",
            "~art:N",
            "~art:P",
            "~srs:goal",
            "~srs:req",
            "~srs:nfr",
            "~srs:ac",
            "~srs:constraint",
            "~srs:actor",
            "~srs:output",
            "E",
        ]
    );

    let mut asks = ids(&["E", "E~1-12", "E.1"]);
    let v = view_open(&g, &asks);
    assert_eq!(v.open, asks);
    assert_eq!(v.shown, 39);
    for (band, kids) in [("~b:constitution", 3), ("~b:SRS", 7)] {
        let b = find(&v, band);
        assert_eq!((b.group, b.fold, b.kids), (true, BoxFold::Folded, kids), "{band}");
    }
    assert!(v.shown as usize <= VIEW_CAP);

    asks.push("~b:SRS".into());
    let v = view_open(&g, &asks);
    assert_eq!(v.open, ids(&["~b:SRS"]));
    assert_eq!(v.shown, 9);
    let e = find(&v, "E");
    assert_eq!((e.fold, e.kids), (BoxFold::Folded, 156));
    assert_eq!(find(&v, "~b:constitution").fold, BoxFold::Folded);
    assert!(v.refused.is_empty());
}

#[test]
fn gtuck_refuse_and_ignore() {
    let mut g = Graph::default();
    for i in 2..=30 {
        g.nodes.push(node(&format!("X{i}"), NodeKind::Epic));
    }
    add_epic(&mut g, "X1", 144);
    let v = view_open(&g, &ids(&["X1", "X2", "no-such", "X1"]));
    assert_eq!(v.refused, ids(&["X1"]));
    assert!(v.open.is_empty());
    assert_eq!(boxes(&v), boxes(&view(&g)));
    assert!(v.shown as usize <= VIEW_CAP);

    let v = view_open(&hub(), &ids(&["h~1-12", "h.1", "A-1", "", "~b:nope"]));
    assert!(v.open.is_empty() && v.refused.is_empty());
    assert_eq!(v.shown, 2);
}

#[test]
fn gtuck_chunk_ids() {
    let v = view_open(&hub(), &ids(&["h", "h~577-619"]));
    let shown: Vec<&str> = v.nodes.iter().map(|n| n.node.id.as_str()).collect();
    assert_eq!(
        shown,
        vec![
            "~art:A",
            "h",
            "h~1-144",
            "h~145-288",
            "h~289-432",
            "h~433-576",
            "h~577-588",
            "h~577-619",
            "h~589-600",
            "h~601-612",
            "h~613-619",
        ]
    );
    assert_eq!(find(&v, "h~1-144").kids, 145);
    assert_eq!(find(&v, "h~577-588").kids, 12);
    assert_eq!(find(&v, "h~613-619").kids, 7);
    assert_eq!(find(&v, "h~577-588").node.title, "h.577 … h.588");
    assert!(
        v.edges
            .iter()
            .all(|e| e.edge_type == EdgeType::ParentChild && e.to == "h")
    );
    let counts: Vec<u32> = v.edges.iter().map(|e| e.count).collect();
    let mut sorted = counts.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, vec![7, 12, 12, 12, 144, 144, 144, 144], "{counts:?}");
    assert_eq!(v.open, ids(&["h", "h~577-619"]));
}

/// 眺めの不変の値（落とさない・上限・箱の id・数の和・辺の両端）。
fn assert_whole(v: &GraphView, what: &str) {
    assert_eq!(v.cut, 0, "{what}");
    assert!(v.shown as usize <= VIEW_CAP, "{what}: {}", v.shown);
    assert_eq!(v.shown as usize, v.nodes.len(), "{what}");
    let names: BTreeSet<&str> = v.nodes.iter().map(|n| n.node.id.as_str()).collect();
    assert_eq!(names.len(), v.nodes.len(), "{what}: 箱の id が重なる");
    let leaves = v.nodes.iter().filter(|n| !n.group).count() as u32;
    assert_eq!(leaves + v.folded, v.total, "{what}");
    for e in &v.edges {
        assert!(
            names.contains(e.from.as_str()) && names.contains(e.to.as_str()),
            "{what}: {e:?}"
        );
        assert_ne!(e.from, e.to, "{what}");
    }
}

#[test]
fn gtuck_real_every_node_boxed() {
    let g = build(&Inputs {
        design_index: &read("tests/fixtures/graph/real/design-index.tsv"),
        ledger: &read("tests/fixtures/ledger/bd-list-8.json"),
        events: &read("tests/fixtures/graph/real/events.jsonl"),
    });
    let first = view(&g);
    assert!(first.shown <= 20, "{}", first.shown);
    assert_whole(&first, "最初の図");
    let folded: Vec<&str> = first
        .nodes
        .iter()
        .filter(|n| n.fold == BoxFold::Folded)
        .map(|n| n.node.id.as_str())
        .collect();
    assert!(!folded.is_empty());
    for id in folded {
        let v = view_open(&g, &ids(&[id]));
        assert_whole(&v, id);
        assert_eq!(v.open, ids(&[id]), "{id}");
        assert!(v.refused.is_empty(), "{id}");
    }
}

#[test]
fn gtuck_open_list_split() {
    assert_eq!(open_list("a,,b"), ids(&["a", "b"]));
    assert!(open_list("").is_empty());
    assert!(open_list(",,").is_empty());
    assert_eq!(open_list(" a , b"), ids(&[" a ", " b"]));
    assert_eq!(open_list("~art:P,e.1"), ids(&["~art:P", "e.1"]));
    assert_eq!(CHUNK, 12);
    assert_eq!(VIEW_CAP, 40);
}

#[test]
fn gtuck_pure_text() {
    for rel in [
        "crates/tsuzuri-core/src/graph/fold.rs",
        "crates/tsuzuri-core/src/graph/view.rs",
    ] {
        let text = read(rel);
        for word in ["std::fs", "std::process", "SystemTime"] {
            assert!(!text.contains(word), "{rel} に {word}");
        }
    }
    let manifest = read("crates/tsuzuri-core/Cargo.toml");
    let mut names = Vec::new();
    let mut inside = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line == "[dependencies]";
            continue;
        }
        if inside
            && !line.is_empty()
            && !line.starts_with('#')
            && let Some((name, _)) = line.split_once(['=', '.'])
        {
            names.push(name.trim().to_string());
        }
    }
    assert_eq!(names, ids(&["serde", "serde_json", "tsuzuri-contract"]));
}

/// 着地済みの行と流れている行の verify の filter の語（接頭辞 gtuck_ は並べない）。
const FILTER_WORDS: &[&str] = &[
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "acctframe_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "askcard_",
    "batchpanel_",
    "board_min_",
    "contract_form_",
    "frame_",
    "gapspage_",
    "gquestion_",
    "graph_",
    "gview_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hook_",
    "hsblock_",
    "hsderive_",
    "hspage_",
    "ledgerblock_",
    "mapgraph_",
    "mapview_",
    "nextstep_",
    "nodepage_",
    "parts_",
    "pipe_",
    "project_",
    "question_",
    "seatblock_",
    "seatcard_",
    "server_",
    "skeleton_",
    "stage_",
    "stats_",
    "steady_",
    "topbar_",
    "tz_",
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
    "ilink_",
    "afocus_",
    "nxact_",
    "urpanel_",
    "saxis_",
    "ticker_",
    "hfig_",
    "pmore_",
    "lspark_",
    "apop_",
    "brand_",
    "runsdoc_",
    "nbatch_",
    "kcli_",
    "qgate_",
    "nsum_",
    "hcard_",
    "ntime_",
    "ptitle_",
    "ncard_",
    "hsym_",
    "smore_",
    "lcard_",
    "aaround_",
    "hruling_",
    "sxaxis_",
    "plimit_",
    "bport_",
    "fstop_",
    "nsumw_",
    "fmark_",
    "fserve_",
    "cadopt_",
    "tipx_",
    "hcsess_",
    "hcproj_",
    "sesplit_",
    "mstore_",
    "athr_",
    "qblock_",
    "wsteady_",
    "gsum_",
    "nstall_",
    "pfold_",
    "uword_",
    "cround_",
    "cgdom_",
    "csled_",
    "lhome_",
    "shb_",
    "ghb_",
    "nact_",
    "aord_",
    "mtree_",
    "rhold_",
    "wstrip_",
    "flight_",
    "qkey_",
    "pwhole_",
    "stage_term_",
    "stage_cdp_",
    "pclosed_",
    "btuck_",
];

#[test]
fn gtuck_own_names_clean() {
    let text = read("crates/tsuzuri-core/tests/gtuck.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0] == "#[test]")
        .map(|w| {
            w[1].strip_prefix("fn ")
                .and_then(|rest| rest.split_once('('))
                .map(|(name, _)| name)
                .unwrap_or_else(|| panic!("test の属性の次が fn でない: {}", w[1]))
        })
        .collect();
    assert!(names.len() >= 10, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("gtuck_")
            .unwrap_or_else(|| panic!("{name} が gtuck_ で始まらない"));
        for word in FILTER_WORDS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
