//! 設計ノートの行の節点と design の辺の歯（行 c-dn-rows・要件 FR2）。
//! folio の索引の種類の語「設計ノートの行」の行を節点にし、台帳の pointer の行が指す行の節点へ design の辺を組む。
#![cfg(test)]

use std::path::Path;

use serde_json::{Value, json};
use tsuzuri_contract::graph::{EdgeType, GraphNode, NodeKind};
use tsuzuri_core::graph::build::{DESIGN_KINDS, NOTE_ROW_KIND, add_summary};
use tsuzuri_core::graph::{Graph, Inputs, Source, Verdict, build, check};

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

/// 器の event log の 1 行。
const EV: &str = r#"{"schema":1,"ts":"2026-09-28T01:00:00Z","kind":"RunCreated","run":"dn.1-20260928T010000Z","bead":"dn.1","stage":"Intake"}"#;

/// 節の台帳（epic dn と 3 本の task）。
fn led() -> String {
    let task = |id: &str, status: &str, acceptance: &str| {
        json!({
            "id": id,
            "title": format!("t {id}"),
            "status": status,
            "issue_type": "task",
            "acceptance_criteria": acceptance,
            "dependencies": [{"depends_on_id": "dn", "type": "parent-child"}],
        })
    };
    json!([
        {"id": "dn", "title": "根", "status": "open", "issue_type": "epic"},
        task("dn.1", "open", "design = contracts/nx.toml#a"),
        task(
            "dn.2",
            "closed",
            "受け入れの字\ndesign = contracts/nx.toml#b\ndesign = contracts/ny.toml#c",
        ),
        task("dn.3", "open", "design = contracts/nx.toml#zz"),
    ])
    .to_string()
}

fn graph_of(design_index: &str, ledger: &str, events: &str) -> Graph {
    build(&Inputs {
        design_index,
        ledger,
        events,
    })
}

/// 節の索引と台帳と event log のグラフ G。
fn g() -> Graph {
    graph_of(IDX, &led(), EV)
}

/// 型 design の辺（辺の順・from と to）。
fn design_edges(g: &Graph) -> Vec<(String, String)> {
    g.edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::Design)
        .map(|e| (e.from.clone(), e.to.clone()))
        .collect()
}

fn row(id: &str, file: &str, digest: &str, title: &str) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        kind: NodeKind::NoteRow,
        file: Some(file.to_string()),
        digest: Some(digest.to_string()),
        title: title.to_string(),
        line: None,
        plain: None,
        eng: None,
        updated: None,
    }
}

fn pair(from: &str, to: &str) -> (String, String) {
    (from.to_string(), to.to_string())
}

#[test]
fn dnrow_index_reads_rows() {
    let g = g();
    assert!(g.unread.is_empty(), "読めない出所: {:?}", g.unread);
    let rows: Vec<&GraphNode> = g
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::NoteRow)
        .collect();
    assert_eq!(
        rows,
        vec![
            &row("nx#a", "design-note/nx.yaml", "1a2b3c4d", "便 a — 骨格"),
            &row("nx#b", "design-note/nx.yaml", "5e6f7a8b", "便 b — 型"),
            &row("ny#c", "design-note/ny.yaml", "9c0d1e2f", "行 c — 札"),
        ]
    );
    let counts: Vec<(NodeKind, usize)> = NodeKind::ALL
        .iter()
        .map(|k| (*k, g.count_nodes(*k)))
        .filter(|(_, n)| *n > 0)
        .collect();
    assert_eq!(
        counts,
        vec![
            (NodeKind::Article, 1),
            (NodeKind::Req, 1),
            (NodeKind::NoteRow, 3),
            (NodeKind::Epic, 1),
            (NodeKind::Task, 3),
            (NodeKind::Run, 1),
        ]
    );
    let basis: Vec<(String, String)> = g
        .edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::Basis)
        .map(|e| (e.from.clone(), e.to.clone()))
        .collect();
    assert_eq!(basis, vec![pair("FR1", "A-1")]);
    assert_eq!(g.skipped.design_edges, 0, "req の 2 行と depends の 2 行も組む");
    assert_eq!(g.skipped.ledger_edges, 0);
}

#[test]
fn dnrow_kind_words() {
    assert_eq!(NOTE_ROW_KIND, "設計ノートの行");
    assert_eq!(DESIGN_KINDS, 12);
    assert_eq!(Source::Design.kinds(), &NodeKind::ALL[..12]);
    assert_eq!(Source::Design.kinds().last(), Some(&NodeKind::NoteRow));
    assert_eq!(Source::of(NodeKind::NoteRow), Some(Source::Design));
    for k in NodeKind::ALL {
        assert!(Source::of(k).is_some(), "{k:?} の出所");
    }

    let serde_word = IDX.replace(
        "ny#c\t設計ノートの行\t",
        "ny#c\tnote-row\t",
    );
    assert_ne!(serde_word, IDX);
    let g = graph_of(&serde_word, &led(), EV);
    assert!(g.unread.is_empty(), "読めない出所: {:?}", g.unread);
    assert_eq!(g.skipped.design_nodes, 1);
    assert_eq!(g.count_nodes(NodeKind::NoteRow), 2);
    assert_eq!(g.count_nodes(NodeKind::Req), 1);
    assert_eq!(
        design_edges(&g),
        vec![pair("dn.1", "nx#a"), pair("dn.2", "nx#b")]
    );

    empty_index_graph();
}

/// 空の索引と台帳と event log のグラフ（設計ノートの行の出所が読めない）。
fn empty_index_graph() {
    let g = graph_of("", &led(), EV);
    assert_eq!(g.unread, vec![Source::Design], "空の索引");
    assert!(g.unknown_kinds().contains(&NodeKind::NoteRow), "空の索引");
    assert_eq!(g.count_nodes(NodeKind::NoteRow), 0, "空の索引");
    assert_eq!(g.count_nodes(NodeKind::Req), 0, "空の索引");
    assert_eq!(g.count_edges(EdgeType::Design), 0, "空の索引");
    assert_eq!(g.count_nodes(NodeKind::Task), 3, "空の索引");
}

#[test]
fn dnrow_design_edges() {
    let g = g();
    assert_eq!(
        design_edges(&g),
        vec![
            pair("dn.1", "nx#a"),
            pair("dn.2", "nx#b"),
            pair("dn.2", "ny#c"),
        ]
    );
    let counts: Vec<(EdgeType, usize)> = EdgeType::ALL
        .iter()
        .map(|t| (*t, g.count_edges(*t)))
        .filter(|(_, n)| *n > 0)
        .collect();
    assert_eq!(
        counts,
        vec![
            (EdgeType::Basis, 1),
            (EdgeType::Req, 2),
            (EdgeType::Depends, 2),
            (EdgeType::ParentChild, 3),
            (EdgeType::Design, 3),
            (EdgeType::RunOf, 1),
        ]
    );
    assert_eq!(
        g.beads["dn.3"].pointers,
        vec!["design = contracts/nx.toml#zz".to_string()]
    );

    let bare = graph_of(IDX, "", EV);
    assert_eq!(bare.unread, vec![Source::Ledger]);
    assert_eq!(bare.count_nodes(NodeKind::NoteRow), 3);
    assert_eq!(bare.count_edges(EdgeType::Design), 0);
}

/// open の task fm.1 の 1 本の台帳（acceptance_criteria に字を置く）のグラフの design の辺。
fn fm_edges(acceptance: &str) -> Vec<(String, String)> {
    let ledger = json!([{
        "id": "fm.1",
        "title": "t fm.1",
        "status": "open",
        "issue_type": "task",
        "acceptance_criteria": acceptance,
    }])
    .to_string();
    let g = graph_of(IDX, &ledger, EV);
    assert!(g.unread.is_empty(), "{acceptance:?}: 読めない出所 {:?}", g.unread);
    let edges = design_edges(&g);
    assert!(edges.iter().all(|(from, _)| from == "fm.1"), "{acceptance:?}");
    edges
}

#[test]
fn dnrow_pointer_forms() {
    let one = [
        "design = contracts/nx.toml#a",
        "design =   contracts/nx.toml#a  ",
        "design = nx.toml#a",
        "design = a/b/nx.toml#a",
        "受け入れの字\r\ndesign = contracts/nx.toml#a\r\n本文",
        "design = contracts/nx.toml#a\ndesign = contracts/nx.toml#a",
    ];
    for text in one {
        assert_eq!(fm_edges(text), vec![pair("fm.1", "nx#a")], "{text:?}");
    }
    let none = [
        "",
        "design = contracts/nx.toml#zz",
        "design = contracts/nx.md#a",
        "design = contracts/nx.yaml#a",
        "design = contracts/nx#a",
        "design = contracts/nx.toml#a#b",
        "design = contracts/nx.toml#",
        "design = contracts/nx.toml",
        "design = contracts/.toml#a",
        "design=contracts/nx.toml#a",
        " design = contracts/nx.toml#a",
        "research = contracts/nx.toml#a",
    ];
    for text in none {
        assert!(fm_edges(text).is_empty(), "{text:?}");
    }
    assert_eq!(
        fm_edges("design = contracts/ny.toml#c\ndesign = contracts/nx.toml#b"),
        vec![pair("fm.1", "ny#c"), pair("fm.1", "nx#b")]
    );

    let pointer = "design = contracts/nx.toml#a";
    let ledger = json!([
        {"id": "fk.1", "title": "t", "status": "open", "issue_type": "epic", "acceptance": pointer},
        {"id": "fk.2", "title": "t", "status": "open", "issue_type": "task",
         "labels": ["intake:memo"], "acceptance": pointer},
        {"id": "fk.3", "title": "t", "status": "closed", "issue_type": "task",
         "labels": ["intake:question"], "acceptance": pointer},
    ])
    .to_string();
    let g = graph_of(IDX, &ledger, EV);
    assert!(g.unread.is_empty(), "読めない出所: {:?}", g.unread);
    assert_eq!(
        design_edges(&g),
        vec![
            pair("fk.1", "nx#a"),
            pair("fk.2", "nx#a"),
            pair("fk.3", "nx#a"),
        ]
    );
}

#[test]
fn dnrow_invariants_hold() {
    let got: Vec<(&str, Verdict)> = check(&g()).into_iter().map(|i| (i.id, i.verdict)).collect();
    let want: Vec<(&str, Verdict)> = [
        "g-1", "g-2", "g-3", "g-4", "g-5", "g-6", "g-7", "g-8", "g-9", "g-10", "g-11", "g-12",
    ]
    .into_iter()
    .map(|id| {
        let v = if ["g-3", "g-7", "g-9"].contains(&id) {
            Verdict::Unknown
        } else {
            Verdict::Pass
        };
        (id, v)
    })
    .collect();
    assert_eq!(got, want);
}

#[test]
fn dnrow_summary_fills_rows() {
    let mut g = g();
    let summary = [
        json!({"id": "nx#a", "kind": "設計ノートの行", "file": "design-note/nx.yaml", "line": 30,
               "title": "便 a — 骨格", "plain": null, "eng": "行 a の題"}),
        json!({"id": "ny#c", "kind": "設計ノートの行", "file": "design-note/nz.yaml", "line": 9,
               "title": "行 c — 札", "plain": null, "eng": "file の違う行"}),
    ]
    .iter()
    .map(Value::to_string)
    .collect::<Vec<_>>()
    .join("\n");
    assert!(add_summary(&mut g, &summary));
    let a = g.node("nx#a").expect("nx#a");
    assert_eq!(a.line, Some(30));
    assert_eq!(a.plain, None);
    assert_eq!(a.eng.as_deref(), Some("行 a の題"));
    for id in ["nx#b", "ny#c"] {
        let n = g.node(id).expect(id);
        assert_eq!((n.line, &n.plain, &n.eng), (None, &None, &None), "{id}");
    }
}

/// 計画と contracts の verify の filter の語（ほかの語を部分の字として含まない語に畳んだ語と後の行の接頭辞）。
const FILTER_WORDS: [&str; 140] = [
    "aaround_", "accept_", "account_", "acctcore_", "acctdoc_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "afocus_", "aord_", "apop_", "askcard_", "athr_", "batchpanel_", "bhalf_", "board_min_",
    "bport_", "brand_", "btuck_", "cadopt_", "cgdom_", "cmark_", "contract_form_", "cround_",
    "csled_", "denv_", "ecache_", "epolq_", "flight_", "fmark_", "frame_", "fserve_", "fstop_",
    "gapspage_", "gbnote_", "gfresh_", "ghb_", "glabel_", "gnav_", "gpill_", "gpulse_",
    "graph_", "gsum_", "gtuck_", "gview_", "hacols_", "hbconf_", "hbpost_", "hbproc_",
    "hbroute_", "hcard_", "hcled_", "hcnx_", "hcproj_", "hcsess_", "hfig_", "hnunk_", "hook_",
    "hruling_", "hsblock_", "hsderive_", "hspage_", "hsym_", "iclose_", "ilink_", "kcli_",
    "klink_", "launch_", "lcard_", "ledgerblock_", "lhome_", "lidle_", "lsnap_", "lspark_",
    "lstore_", "mapview_", "mkeys_", "mlink_", "mstore_", "mtree_", "nact_", "nbatch_",
    "ncard_", "nextstep_", "nodepage_", "nstall_", "nsum_", "nsumw_", "ntime_", "nxact_",
    "parts_", "pclosed_", "pfold_", "pipe_", "plimit_", "pmore_", "pquest_", "pqueue_",
    "project_", "ptitle_", "punmap_", "pwhole_", "qblock_", "qgate_", "qkey_", "question_",
    "relay_", "rhold_", "runsdoc_", "rvk_", "saxis_", "sclosed_", "seatblock_", "seatcard_",
    "server_", "sesplit_", "shb_", "skeleton_", "smore_", "stage_", "stats_", "stcli_",
    "steady_", "sxaxis_", "ticker_", "tipx_", "topbar_", "tz_", "urpanel_", "uword_",
    "wstrip_", "pgz_", "pmisfit_", "gfix_",
];

#[test]
fn dnrow_own_names_clean() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/dnrow.rs");
    let text = std::fs::read_to_string(&path).expect("tests/dnrow.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .filter_map(|w| {
            let rest = w[1].trim().strip_prefix("fn ")?;
            rest.split('(').next()
        })
        .collect();
    assert!(names.len() >= 7, "歯の名: {names:?}");
    for name in names {
        let rest = name
            .strip_prefix("dnrow_")
            .unwrap_or_else(|| panic!("{name} は dnrow_ で始まる"));
        for word in FILTER_WORDS {
            assert!(!rest.contains(word), "{name} は {word} を含む");
        }
    }
}
