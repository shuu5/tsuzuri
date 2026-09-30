//! 台帳の bead の概要と、要約の無い節点を名指す床の歯（行 c-summary-ledger・要件 FR15）。
//! build は bead の description の行頭「概要 = 」「技術 = 」の行を plain と eng に写し、
//! 中核の check の `unsummarized` が要約の無い節点を数えて名指す。節の字は歯の中で組み、
//! workspace の根の fixture は読むだけで書かない。
#![cfg(test)]

use std::path::{Path, PathBuf};

use serde_json::json;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::{Fold, GraphNode, NodeKind};
use tsuzuri_core::graph::build::add_summary;
use tsuzuri_core::graph::check::{SUMMARY_KINDS, unsummarized};
use tsuzuri_core::graph::{Graph, Inputs, around, build};
use tsuzuri_core::question;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

fn read_src(name: &str) -> String {
    read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/graph").join(name))
}

/// 節の索引の字 IDX1（タブ区切りの 5 列の 1 行）。
const IDX1: &str = "FR1\t要件\tsrs.yaml\t00000000\t面は 2 つ\n";

/// 節の event log の字 EV1。
const EV1: &str = r#"{"kind":"RunCreated","run":"t-2-20260928T010000Z"}"#;

const POINTER: &str = "design = contracts/surface-wave6.toml#c-summary-wire";

const Q1_NOTES: &str = "裁定 id = q-1:20260928T0100Z-1・問い = q-1・逐語 = はい\n受け id = r-1・受けた";

/// 節の台帳の字 LED（bead 8 本・時刻の鍵は持たない）。
fn led() -> String {
    json!([
        {
            "id": "q-1", "title": "題", "issue_type": "task", "status": "open",
            "labels": ["intake:question"],
            "description": "前置きの字\n概要 = やさしい説明\n技術 = かたい説明\n概要 = 2 つ目の概要（取らない）",
            "notes": Q1_NOTES
        },
        {
            "id": "t-2", "title": "題", "issue_type": "task", "status": "open", "labels": [],
            "description": "技術 = 技術だけの行",
            "acceptance_criteria": POINTER
        },
        {
            "id": "m-3", "title": "方針", "issue_type": "task", "status": "open",
            "labels": ["intake:memo"], "parent": "e-4",
            "description": "概要 =   memo の説明  \r\n残り",
            "notes": "方針 id = p-1・方針の行"
        },
        {
            "id": "e-4", "title": "題", "issue_type": "epic", "status": "open", "labels": [],
            "description": "概要 = epic の説明"
        },
        {
            "id": "t-5", "title": "題", "issue_type": "task", "status": "closed", "labels": [],
            "description": "概要 = 閉じた bead の説明"
        },
        {
            "id": "t-6", "title": "題", "issue_type": "task", "status": "open", "labels": [],
            "description": "第 8 波。契約 = contracts/surface-wave6.toml#c-summary-wire。",
            "acceptance_criteria": POINTER
        },
        {
            "id": "t-7", "title": "題", "issue_type": "task", "status": "open", "labels": [],
            "description": " 概要 = 行頭に空白\n技術 = "
        },
        {
            "id": "t-8", "title": "題", "issue_type": "task", "status": "open", "labels": []
        }
    ])
    .to_string()
}

/// 節のグラフ G1。
fn g1() -> Graph {
    build(&Inputs {
        design_index: IDX1,
        ledger: &led(),
        events: EV1,
    })
}

/// 要約の 1 行（鍵は id・kind・file・line・title・plain・eng の順）。
#[expect(
    clippy::too_many_arguments,
    reason = "引数が規則の行 R-4 の 5 を越える・R-4 の歯の行が直してこの属性を外す"
)]
fn sum_line(
    id: &str,
    kind: &str,
    file: &str,
    line: u32,
    title: &str,
    plain: Option<&str>,
    eng: Option<&str>,
) -> String {
    format!(
        "{{\"id\":{},\"kind\":{},\"file\":{},\"line\":{line},\"title\":{},\"plain\":{},\"eng\":{}}}",
        json!(id),
        json!(kind),
        json!(file),
        json!(title),
        json!(plain),
        json!(eng),
    )
}

/// 節の要約の字 SUM1（2 行）。
fn sum1() -> String {
    [
        sum_line(
            "FR1",
            "要件",
            "srs.yaml",
            2,
            "面は 2 つ",
            Some("画面は 2 つです。"),
            None,
        ),
        sum_line("q-1", "question", "x.yaml", 5, "x", Some("上書き"), Some("上書き")),
    ]
    .join("\n")
}

/// 節の索引の字 IDX2（2 行）。
const IDX2: &str = "FR1\t要件\tsrs.yaml\t00000000\t面は 2 つ\nFR2\t要件\tsrs.yaml\t00000000\t地図の面\n";

/// 節の要約の字 SUM2（2 行）。
fn sum2() -> String {
    [
        sum_line(
            "FR1",
            "要件",
            "srs.yaml",
            2,
            "面は 2 つ",
            Some("画面は 2 つです。"),
            None,
        ),
        sum_line(
            "FR2",
            "要件",
            "srs.yaml",
            9,
            "地図の面",
            None,
            Some("地図は 4 面とする"),
        ),
    ]
    .join("\n")
}

/// 節の台帳の字 LED2（bead 3 本）。
fn led2() -> String {
    json!([
        {
            "id": "q-1", "title": "題", "issue_type": "task", "status": "open",
            "labels": ["intake:question"],
            "description": "概要 = やさしい説明",
            "notes": Q1_NOTES
        },
        {
            "id": "t-2", "title": "題", "issue_type": "task", "status": "open", "labels": [],
            "description": "技術 = 技術だけの行"
        },
        {
            "id": "x-3", "title": "題", "issue_type": "task", "status": "open", "labels": [],
            "description": "第 8 波。",
            "acceptance_criteria": POINTER
        }
    ])
    .to_string()
}

/// 節の event log の字 EV2。
const EV2: &str = r#"{"kind":"RunCreated","run":"x-3-20260928T010000Z"}"#;

/// 3 つの字で build し、SUM2 で add_summary を撃つ（真を返すことも見る）。
fn g2_from(index: &str, ledger: &str, events: &str) -> Graph {
    let mut g = build(&Inputs {
        design_index: index,
        ledger,
        events,
    });
    assert!(add_summary(&mut g, &sum2()), "SUM2 は読める");
    g
}

/// 節のグラフ G2。
fn g2() -> Graph {
    g2_from(IDX2, &led2(), EV2)
}

type Fields = (Option<u32>, Option<String>, Option<String>);

fn fields(n: &GraphNode) -> Fields {
    (n.line, n.plain.clone(), n.eng.clone())
}

fn node<'a>(g: &'a Graph, id: &str) -> &'a GraphNode {
    g.node(id).unwrap_or_else(|| panic!("節点 {id} が在る"))
}

fn node_mut<'a>(g: &'a mut Graph, id: &str) -> &'a mut GraphNode {
    g.nodes
        .iter_mut()
        .find(|n| n.id == id)
        .unwrap_or_else(|| panic!("節点 {id} が在る"))
}

fn f(plain: Option<&str>, eng: Option<&str>) -> Fields {
    (None, plain.map(str::to_string), eng.map(str::to_string))
}

fn known(ids: &[&str]) -> Reading<Vec<String>> {
    Reading::Known(ids.iter().map(|s| s.to_string()).collect())
}

fn bare(id: &str, kind: NodeKind) -> GraphNode {
    GraphNode {
        id: id.into(),
        kind,
        file: None,
        digest: None,
        title: id.into(),
        line: None,
        plain: None,
        eng: None,
        updated: None,
    }
}

#[test]
fn csled_typed_lines_on_beads() {
    let g = g1();
    let got: Vec<(String, Fields)> = g.nodes.iter().map(|n| (n.id.clone(), fields(n))).collect();
    let want: Vec<(String, Fields)> = [
        ("FR1", f(None, None)),
        ("q-1", f(Some("やさしい説明"), Some("かたい説明"))),
        ("q-1:20260928T0100Z-1", f(None, None)),
        ("r-1", f(None, None)),
        ("t-2", f(None, Some("技術だけの行"))),
        ("m-3", f(Some("memo の説明"), None)),
        ("p-1", f(None, None)),
        ("e-4", f(Some("epic の説明"), None)),
        ("t-5", f(Some("閉じた bead の説明"), None)),
        ("t-6", f(None, None)),
        ("t-7", f(None, None)),
        ("t-8", f(None, None)),
        ("t-2-20260928T010000Z", f(None, None)),
    ]
    .into_iter()
    .map(|(id, v)| (id.to_string(), v))
    .collect();
    let mut got_sorted = got.clone();
    let mut want_sorted = want.clone();
    got_sorted.sort();
    want_sorted.sort();
    assert_eq!(got_sorted, want_sorted, "G1 の 13 の節点の 3 つの欄は表 T1");
    assert_eq!(g.nodes.len(), 13);
    assert_eq!(node(&g, "q-1:20260928T0100Z-1").kind, NodeKind::Ruling);
    assert_eq!(node(&g, "r-1").kind, NodeKind::Receipt);
    assert_eq!(node(&g, "p-1").kind, NodeKind::Policy);
    assert_eq!(node(&g, "t-2-20260928T010000Z").kind, NodeKind::Run);
    assert_eq!(node(&g, "e-4").kind, NodeKind::Epic);
    assert_eq!(node(&g, "m-3").kind, NodeKind::Memo);
    assert_eq!(node(&g, "q-1").kind, NodeKind::Question);
}

#[test]
fn csled_same_as_cards() {
    let text = read(&root().join("tests/fixtures/surface/question-2.json"));
    let g = build(&Inputs {
        design_index: "",
        ledger: &text,
        events: "",
    });
    let Reading::Known(cards) = question::list(&text).cards else {
        panic!("問いの一覧が読める");
    };
    let mut ids: Vec<&str> = cards.iter().map(|c| c.id.as_str()).collect();
    ids.sort();
    assert_eq!(ids, vec!["fx-ask.2", "fx-ask.3"]);
    for card in &cards {
        let n = node(&g, card.id.as_str());
        assert_eq!(n.plain, card.plain, "{} の plain", card.id.as_str());
        assert_eq!(n.eng, card.eng, "{} の eng", card.id.as_str());
    }
    assert_eq!(
        fields(node(&g, "fx-ask.3")),
        f(
            Some("画面の色を 2 つに減らしてよいか"),
            Some("style.css の変数を 5 から 2 へ")
        )
    );
    assert_eq!(fields(node(&g, "fx-ask.2")), f(None, None));
    assert_eq!(
        fields(node(&g, "fx-ask.1")),
        f(Some("閉じた問いの概要"), None)
    );
    assert_eq!(fields(node(&g, "fx-ask.1:20260924T0200Z-1")), f(None, None));
}

#[test]
fn csled_later_steps_keep_fields() {
    let mut g = g1();
    let doc = around(&g, "q-1", 1, Fold::None).expect("q-1 の近傍");
    let q1 = node(&g, "q-1").clone();
    let centers: Vec<&GraphNode> = doc
        .rows
        .iter()
        .filter(|r| r.col == 0)
        .map(|r| &r.node)
        .collect();
    assert!(!centers.is_empty(), "列 0 の行が在る");
    for n in centers {
        assert_eq!((&n.plain, &n.eng), (&q1.plain, &q1.eng));
    }
    let before: Vec<(String, Fields)> = g
        .nodes
        .iter()
        .filter(|n| g.beads.contains_key(&n.id))
        .map(|n| (n.id.clone(), fields(n)))
        .collect();
    assert_eq!(before.len(), 8);
    assert!(add_summary(&mut g, &sum1()), "SUM1 は読める");
    assert_eq!(
        fields(node(&g, "FR1")),
        (Some(2), Some("画面は 2 つです。".to_string()), None)
    );
    let after: Vec<(String, Fields)> = g
        .nodes
        .iter()
        .filter(|n| g.beads.contains_key(&n.id))
        .map(|n| (n.id.clone(), fields(n)))
        .collect();
    assert_eq!(after, before, "台帳の bead の節点は撃つ前と同じ");
}

#[test]
fn csled_floor_shape_and_pure() {
    let _: fn(&Graph, bool) -> Reading<Vec<String>> = tsuzuri_core::graph::check::unsummarized;
    let mut want: Vec<NodeKind> = NodeKind::ALL[..11].to_vec();
    want.extend([NodeKind::Epic, NodeKind::Task, NodeKind::Memo, NodeKind::Question]);
    assert_eq!(SUMMARY_KINDS.to_vec(), want);
    for name in ["check.rs", "build.rs"] {
        let text = read_src(name);
        for word in ["std::fs", "std::process", "SystemTime"] {
            assert!(!text.contains(word), "{name} に {word} が無い");
        }
    }
}

#[test]
fn csled_floor_names_one() {
    let g = g2();
    assert_eq!(unsummarized(&g, true), known(&["x-3"]));
}

#[test]
fn csled_floor_rules() {
    assert_eq!(unsummarized(&g2(), false), Reading::Unknown);
    assert_eq!(
        unsummarized(&g2_from("", &led2(), EV2), true),
        Reading::Unknown
    );
    assert_eq!(unsummarized(&g2_from(IDX2, "", EV2), true), Reading::Unknown);
    assert_eq!(
        unsummarized(&g2_from(IDX2, &led2(), ""), true),
        known(&["x-3"])
    );

    let mut g = g2();
    node_mut(&mut g, "FR1").plain = Some(String::new());
    assert_eq!(unsummarized(&g, true), known(&["FR1", "x-3"]));

    let mut g = g2();
    let copy = node(&g, "x-3").clone();
    g.nodes.push(copy);
    assert_eq!(unsummarized(&g, true), known(&["x-3"]));

    let mut g = g2();
    g.nodes
        .push(bare("surface-wave6#c-summary-wire", NodeKind::NoteRow));
    g.nodes
        .push(bare("policy:20260928T0100Z-1", NodeKind::Policy));
    assert_eq!(unsummarized(&g, true), known(&["x-3"]));

    let mut g = g2();
    node_mut(&mut g, "x-3").plain = Some("あ".into());
    assert_eq!(unsummarized(&g, true), known(&[]));

    assert_eq!(unsummarized(&Graph::default(), true), known(&[]));
}

/// 着地済みの行と第 3 波から第 8 波の行の verify の filter の語（106 語）。
const FILTER_WORDS: [&str; 106] = [
    "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
    "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
    "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
    "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_",
    "server_", "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_", "gnav_",
    "mkeys_", "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_", "ticker_", "hfig_",
    "pmore_", "lspark_", "apop_", "brand_", "runsdoc_", "nbatch_", "kcli_", "qgate_", "hcard_",
    "ntime_", "ptitle_", "ncard_", "hsym_", "smore_", "lcard_", "aaround_", "hruling_",
    "sxaxis_", "plimit_", "bport_", "fstop_", "nsum_", "nsumw_", "fmark_", "fserve_", "cadopt_",
    "tipx_", "hcsess_", "hcproj_", "sesplit_", "mstore_", "athr_", "qblock_", "wsteady_",
    "gsum_", "nstall_", "pfold_", "uword_", "cround_", "cgdom_", "lhome_", "shb_", "ghb_",
    "nact_", "aord_", "mtree_",
];

#[test]
fn csled_own_names_clean() {
    let text = read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/csled.rs"));
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0] == "#[test]")
        .filter_map(|w| w[1].strip_prefix("fn "))
        .map(|rest| rest.split('(').next().unwrap_or(rest))
        .collect();
    assert_eq!(names.len(), 7, "test の fn は 7 つ: {names:?}");
    for name in names {
        let rest = name
            .strip_prefix("csled_")
            .unwrap_or_else(|| panic!("{name} は csled_ で始まる"));
        for word in FILTER_WORDS {
            assert!(!rest.contains(word), "{name} は {word} を含まない");
        }
    }
}
