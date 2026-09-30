//! 設計ノートの行の辺の型 req と depends の歯（行 c-dn-edges・要件 FR2）。
//! folio の索引の設計ノートの行の欄 req と depends の辺の行を、型 Req（行 → 要件）と Depends（行 → 行）の辺に組む。
#![cfg(test)]

use std::path::Path;

use serde_json::json;
use tsuzuri_contract::graph::{AroundDoc, EdgeType, Fold};
use tsuzuri_contract::wire;
use tsuzuri_core::graph::build::DESIGN_EDGE_TYPES;
use tsuzuri_core::graph::{Graph, Inputs, Verdict, around, build, check, view};

/// 節の索引（節点 5・辺 6・型は req と depends）。
const IDX: &str = "# 節点（1 行 = id / 種類 / file / 要約値 8 字 / 題 36 字・タブ区切り）
FR1\t要件\tsrs.yaml\t00000000\t面は 2 つ
FR2\t要件\tsrs.yaml\t00000000\t導出グラフ
ex#a\t設計ノートの行\tdesign-note/ex.yaml\t1a2b3c4d\t便 a — 骨格
ex#b\t設計ノートの行\tdesign-note/ex.yaml\t5e6f7a8b\t便 b — 型
ey#c\t設計ノートの行\tdesign-note/ey.yaml\t9c0d1e2f\t行 c — 札
# 辺（1 行 = 端 / 端 / 型・タブ区切り）
ex#a\tFR1\treq
ex#b\tFR1\treq
ex#b\tFR2\treq
ex#b\tex#a\tdepends
ey#c\tex#b\tdepends
ey#c\tFR2\treq
";

/// 器の event log の 1 行。
const EV: &str = r#"{"schema":1,"ts":"2026-09-28T03:00:00Z","kind":"RunCreated","run":"de.1-20260928T030000Z","bead":"de.1","stage":"Intake"}"#;

/// 節の台帳（epic de と task de.1）。
fn led() -> String {
    json!([
        {"id": "de", "title": "根", "status": "open", "issue_type": "epic"},
        {
            "id": "de.1",
            "title": "t de.1",
            "status": "open",
            "issue_type": "task",
            "acceptance_criteria": "design = contracts/ey.toml#c",
            "dependencies": [{"depends_on_id": "de", "type": "parent-child"}],
        },
    ])
    .to_string()
}

/// 節の索引と台帳と event log のグラフ G。
fn g() -> Graph {
    build(&Inputs {
        design_index: IDX,
        ledger: &led(),
        events: EV,
    })
}

fn triple(from: &str, to: &str, t: EdgeType) -> (String, String, EdgeType) {
    (from.to_string(), to.to_string(), t)
}

#[test]
fn dnedge_types_closed() {
    assert_eq!(EdgeType::ALL.len(), 32);
    assert_eq!(DESIGN_EDGE_TYPES, 19);
    let words: Vec<String> = EdgeType::ALL[..DESIGN_EDGE_TYPES]
        .iter()
        .map(|t| wire::encode(t).expect("型の電文"))
        .collect();
    let want: Vec<String> = [
        "in-article",
        "relations.articles",
        "relations.reqs",
        "relations.rules",
        "relations.sections",
        "amended_by",
        "article",
        "refs",
        "basis",
        "goals",
        "rules",
        "adrs",
        "verify.ac",
        "verifies",
        "figures",
        "produced",
        "amends",
        "req",
        "depends",
    ]
    .iter()
    .map(|w| format!("\"{w}\""))
    .collect();
    assert_eq!(words, want);
    assert_eq!(EdgeType::ALL[17], EdgeType::Req);
    assert_eq!(EdgeType::ALL[18], EdgeType::Depends);
    assert_eq!(EdgeType::ALL[19], EdgeType::ParentChild);
    assert_eq!(wire::decode::<EdgeType>("\"req\"").expect("req"), EdgeType::Req);
    assert_eq!(
        wire::decode::<EdgeType>("\"depends\"").expect("depends"),
        EdgeType::Depends
    );
}

#[test]
fn dnedge_index_builds() {
    let g = g();
    assert!(g.unread.is_empty(), "読めない出所: {:?}", g.unread);
    let design: Vec<(String, String, EdgeType)> = g
        .edges
        .iter()
        .filter(|e| EdgeType::ALL[..DESIGN_EDGE_TYPES].contains(&e.edge_type))
        .map(|e| (e.from.clone(), e.to.clone(), e.edge_type))
        .collect();
    assert_eq!(
        design,
        vec![
            triple("ex#a", "FR1", EdgeType::Req),
            triple("ex#b", "FR1", EdgeType::Req),
            triple("ex#b", "FR2", EdgeType::Req),
            triple("ex#b", "ex#a", EdgeType::Depends),
            triple("ey#c", "ex#b", EdgeType::Depends),
            triple("ey#c", "FR2", EdgeType::Req),
        ]
    );
    assert_eq!(g.skipped.design_edges, 0);
    assert_eq!(g.count_edges(EdgeType::Design), 1);

    let got: Vec<(&str, Verdict)> = check(&g).into_iter().map(|i| (i.id, i.verdict)).collect();
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
fn dnedge_view_and_around() {
    let g = g();
    let v = view(&g);
    let edges: Vec<(&str, &str, EdgeType, u32)> = v
        .edges
        .iter()
        .filter(|e| matches!(e.edge_type, EdgeType::Req | EdgeType::Depends))
        .map(|e| (e.from.as_str(), e.to.as_str(), e.edge_type, e.count))
        .collect();
    assert_eq!(
        edges,
        vec![
            ("~note:ex", "~srs:req", EdgeType::Req, 3),
            ("~note:ey", "~note:ex", EdgeType::Depends, 1),
            ("~note:ey", "~srs:req", EdgeType::Req, 1),
        ]
    );
    let rank = |id: &str| {
        v.nodes
            .iter()
            .find(|n| n.node.id == id)
            .map(|n| n.rank)
            .unwrap_or_else(|| panic!("{id} の箱が無い"))
    };
    assert_eq!(
        [rank("~srs:req"), rank("~note:ex"), rank("~note:ey")],
        [0, 1, 2]
    );

    let doc: AroundDoc = around(&g, "ex#b", 1, Fold::None).expect("ex#b の近傍");
    let rows: Vec<(&str, i8, Option<EdgeType>)> = doc
        .rows
        .iter()
        .map(|r| (r.node.id.as_str(), r.col, r.edge_type))
        .collect();
    assert_eq!(
        rows,
        vec![
            ("FR1", -1, Some(EdgeType::Req)),
            ("FR2", -1, Some(EdgeType::Req)),
            ("ex#a", -1, Some(EdgeType::Depends)),
            ("ex#b", 0, None),
            ("ey#c", 1, Some(EdgeType::Depends)),
        ]
    );
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

#[test]
fn dnedge_own_names_clean() {
    assert_eq!(FILTER_WORDS.len(), 152, "畳んだ 151 語と pgz_");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/dnedge.rs");
    let text = std::fs::read_to_string(&path).expect("tests/dnedge.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .filter_map(|w| {
            let rest = w[1].trim().strip_prefix("fn ")?;
            rest.split('(').next()
        })
        .collect();
    assert!(names.len() >= 4, "歯の名: {names:?}");
    for name in names {
        let rest = name
            .strip_prefix("dnedge_")
            .unwrap_or_else(|| panic!("{name} は dnedge_ で始まる"));
        for word in FILTER_WORDS {
            assert!(!rest.contains(word), "{name} は {word} を含む");
        }
    }
}
