//! 種類の読めない節点の行を組まずに数える歯（行 c-dn-unknown・要件 FR2）。
//! 索引の節点の行の種類の語が設計の 12 種の外なら、その行と、その行の id を端に持つ辺の行だけを組まずに数え、
//! 設計の出所のほかの行は組む。列の数の合わない行と空の字は今のまま設計の出所を読めないとする。

use std::path::Path;

use serde_json::json;
use tsuzuri_contract::graph::{EdgeType, GraphEdge, GraphNode, NodeKind};
use tsuzuri_core::graph::{Graph, Inputs, Skipped, Source, Verdict, build, check};

/// 節の索引（節点 5・辺 6・F-1 と F-2 と nx#a の種類の語は設計の 12 種の外）。
const IDX: &str = "# 節点（1 行 = id / 種類 / file / 要約値 8 字 / 題 36 字・タブ区切り）
FR1\t要件\tsrs.yaml\t00000000\t面は 2 つ
A-1\t条\tconstitution.yaml\t00000000\t見本の条
F-1\t図\tfigures.yaml\t11111111\t見本の図
F-2\t図\tfigures.yaml\t22222222\t次の図
nx#a\tnote-row\tdesign-note/nx.yaml\t1a2b3c4d\t便 a — 骨格
# 辺（1 行 = 端 / 端 / 型・タブ区切り）
FR1\tA-1\tbasis
F-1\tFR1\tbasis
FR1\tF-2\trefs
F-1\tF-2\tbasis
A-1\tFR1\tnot-a-type
F-1\tA-1\tnot-a-type
";

/// IDX から飛ばす行を除いた索引。
const CLEAN: &str = "FR1\t要件\tsrs.yaml\t00000000\t面は 2 つ
A-1\t条\tconstitution.yaml\t00000000\t見本の条
FR1\tA-1\tbasis
";

/// 器の event log の 1 行。
const EV: &str = r#"{"schema":1,"ts":"2026-09-28T02:00:00Z","kind":"RunCreated","run":"sk.1-20260928T020000Z","bead":"sk.1","stage":"Intake"}"#;

/// 節の台帳（task sk.1 の 1 本）。
fn led() -> String {
    json!([{
        "id": "sk.1",
        "title": "t sk.1",
        "status": "open",
        "issue_type": "task",
        "acceptance_criteria": "design = contracts/nx.toml#a",
    }])
    .to_string()
}

fn graph_of(design_index: &str) -> Graph {
    build(&Inputs {
        design_index,
        ledger: &led(),
        events: EV,
    })
}

fn node(id: &str, kind: NodeKind, file: &str, title: &str) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        kind,
        file: Some(file.to_string()),
        digest: Some("00000000".to_string()),
        title: title.to_string(),
        line: None,
        plain: None,
        eng: None,
    }
}

fn edge(from: &str, to: &str, edge_type: EdgeType) -> GraphEdge {
    GraphEdge {
        from: from.to_string(),
        to: to.to_string(),
        edge_type,
    }
}

fn verdict_of(g: &Graph, id: &str) -> Verdict {
    check(g)
        .into_iter()
        .find(|i| i.id == id)
        .map(|i| i.verdict)
        .unwrap_or_else(|| panic!("{id} が無い"))
}

#[test]
fn dnskip_rows_counted() {
    let g = graph_of(IDX);
    assert!(g.unread.is_empty(), "読めない出所: {:?}", g.unread);
    let design: Vec<&GraphNode> = g
        .nodes
        .iter()
        .filter(|n| Source::of(n.kind) == Some(Source::Design))
        .collect();
    assert_eq!(
        design,
        vec![
            &node("FR1", NodeKind::Req, "srs.yaml", "面は 2 つ"),
            &node("A-1", NodeKind::Article, "constitution.yaml", "見本の条"),
        ]
    );
    assert_eq!(
        g.edges,
        vec![
            edge("FR1", "A-1", EdgeType::Basis),
            edge("sk.1-20260928T020000Z", "sk.1", EdgeType::RunOf),
        ]
    );
    assert_eq!(
        g.skipped,
        Skipped {
            design_edges: 5,
            ledger_edges: 0,
            design_nodes: 3,
        }
    );
    assert_eq!(g.count_nodes(NodeKind::NoteRow), 0);
    assert_eq!(g.count_edges(EdgeType::Design), 0);
    assert_eq!(verdict_of(&g, "g-1"), Verdict::Pass);
}

#[test]
fn dnskip_same_as_clean() {
    let skip = graph_of(IDX);
    let clean = graph_of(CLEAN);
    assert_eq!(skip.nodes, clean.nodes);
    assert_eq!(skip.edges, clean.edges);
    assert_eq!(skip.beads, clean.beads);
    let skip_checks = check(&skip);
    assert_eq!(skip_checks.len(), 12);
    assert_eq!(skip_checks, check(&clean));
    assert_eq!(clean.skipped, Skipped::default());
}

#[test]
fn dnskip_shape_still_unread() {
    let four = format!("{CLEAN}FR1\tA-1\tbasis\t余り\n");
    let six = format!("{CLEAN}F-3\t図\tfigures.yaml\t33333333\t題\t余り\n");
    let empty = String::new();
    for (name, idx) in [("4 列の行", &four), ("6 列の行", &six), ("空の索引", &empty)] {
        let g = graph_of(idx);
        assert_eq!(g.unread, vec![Source::Design], "{name}");
        assert_eq!(g.skipped.design_nodes, 0, "{name}");
        let design = g
            .nodes
            .iter()
            .filter(|n| Source::of(n.kind) == Some(Source::Design))
            .count();
        assert_eq!(design, 0, "{name}");
        assert_eq!(g.count_nodes(NodeKind::Task), 1, "{name}");
    }

    let g = graph_of(IDX);
    let dropped = ["F-1", "F-2", "nx#a"];
    for e in &g.edges {
        assert!(
            !dropped.contains(&e.from.as_str()) && !dropped.contains(&e.to.as_str()),
            "{e:?}"
        );
    }
}

/// 計画と contracts の verify の filter の語（ほかの語を部分の字として含まない語に畳んだ語と後の行の接頭辞）。
const FILTER_WORDS: &[&str] = &[
    "aaround_", "accept_", "acchold_", "account_", "acctcore_", "acctdoc_", "accthb_",
    "accthome_", "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_",
    "acctwire_", "afocus_", "aord_", "apop_", "askcard_", "athr_", "batchpanel_", "bhalf_",
    "board_min_", "bport_", "brand_", "btuck_", "cadopt_", "cdorm_", "cgdom_", "cmark_",
    "contract_form_", "cround_", "csled_", "denv_", "dnrow_", "ecache_", "epolq_", "flight_",
    "fmark_", "fprem_", "frame_", "fserve_", "fstop_", "gapspage_", "gbnote_", "gfix_",
    "gfresh_", "ghb_", "gjst_", "glabel_", "gnav_", "gpface_", "gpill_", "gpulse_", "graph_",
    "gsum_", "gtuck_", "gview_", "hacols_", "harest_", "hasplit_", "hbconf_", "hbmark_",
    "hbpost_", "hbproc_", "hbroute_", "hcard_", "hcled_", "hcnx_", "hcproj_", "hcsess_",
    "hdchip_", "hfig_", "hnunk_", "hook_", "hruling_", "hsblock_", "hsderive_", "hshist_",
    "hspage_", "hsym_", "iclose_", "ilink_", "kcli_", "klink_", "launch_", "lcard_",
    "ledgerblock_", "lhome_", "lidle_", "lresume_", "lsnap_", "lspark_", "lstore_", "mapview_",
    "mkeys_", "mlink_", "mstore_", "mtree_", "nact_", "nbatch_", "ncard_", "nextstep_",
    "nodepage_", "nstall_", "nsum_", "nsumw_", "ntime_", "nxact_", "parts_", "pclosed_",
    "pfold_", "pipe_", "plimit_", "pmisfit_", "pmore_", "pquest_", "pqueue_", "project_",
    "ptitle_", "punmap_", "pwhole_", "qblock_", "qgate_", "qkey_", "question_", "relay_",
    "rhold_", "runsdoc_", "rvk_", "saxis_", "sclosed_", "seatblock_", "seatcard_", "server_",
    "sesplit_", "shb_", "skeleton_", "smore_", "stage_", "stats_", "stcli_", "steady_",
    "sxaxis_", "ticker_", "tipx_", "topbar_", "tz_", "urpanel_", "uword_", "wstrip_", "pgz_",
];

/// file の字のうち test の属性の行の次の fn の名。
fn test_names(path: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
    let lines: Vec<&str> = text.lines().collect();
    lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .filter_map(|w| {
            let rest = w[1].trim().strip_prefix("fn ")?;
            rest.split('(').next().map(str::to_string)
        })
        .collect()
}

#[test]
fn dnskip_own_names_clean() {
    assert_eq!(FILTER_WORDS.len(), 152, "畳んだ 151 語と pgz_");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut names = test_names(&root.join("tests/dnskip.rs"));
    names.extend(test_names(&root.join("../tsuzuri-boundary/tests/dnskipw.rs")));
    assert!(names.len() >= 6, "歯の名: {names:?}");
    for name in names {
        let rest = name
            .strip_prefix("dnskip_")
            .unwrap_or_else(|| panic!("{name} は dnskip_ で始まる"));
        for word in FILTER_WORDS {
            assert!(!rest.contains(word), "{name} は {word} を含む");
        }
    }
}
