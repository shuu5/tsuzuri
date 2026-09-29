//! 行 g-map-retired の歯（境界）: 廃止した設計ノートの文書 id を中核の関数で挙げ、電文の欄 retired に写すこと・
//! この file の歯の名。設計の索引と要約の字を直に渡し、子 process も file も使わない。

use std::path::Path;

use tsuzuri_boundary::server::board::{Texts, built, doc, graph};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::GraphDoc;
use tsuzuri_contract::wire;
use tsuzuri_core::graph::check::retired_notes;

/// 設計の索引（設計ノートの行 5 と判断の記録 1・辺は無い）。
const INDEX: &str = "# 節点（1 行 = id / 種類 / file / 要約値 8 字 / 題 36 字・タブ区切り）\n\
ra#r1\t設計ノートの行\tdesign-note/ra.yaml\t00000001\t見本の行\n\
ra#r2\t設計ノートの行\tdesign-note/ra.yaml\t00000002\t見本の行\n\
rb#r1\t設計ノートの行\tdesign-note/rb.yaml\t00000003\t見本の行\n\
rc#r1\t設計ノートの行\tdesign-note/rc.yaml\t00000004\t見本の行\n\
rd#r1\t設計ノートの行\tdesign-note/rd.yaml\t00000005\t見本の行\n\
ADR-1\t判断の記録\tdesign-note/adr.yaml\t00000006\t見本の記録\n\
# 辺（1 行 = 端 / 端 / 型・タブ区切り）\n";

/// 設計ノートの行の無い設計の索引（判断の記録 1 つ）。
const NO_ROWS: &str = "# 節点（1 行 = id / 種類 / file / 要約値 8 字 / 題 36 字・タブ区切り）\n\
ADR-1\t判断の記録\tdesign-note/adr.yaml\t00000006\t見本の記録\n\
# 辺（1 行 = 端 / 端 / 型・タブ区切り）\n";

/// 設計ノートの行 5 つの id と状態の字（ra と rd が retired・rb が effective・rc が draft）。
const STATES: [(&str, Option<&str>); 5] = [
    ("ra#r1", Some("retired")),
    ("ra#r2", Some("retired")),
    ("rb#r1", Some("effective")),
    ("rc#r1", Some("draft")),
    ("rd#r1", Some("retired")),
];

/// 要約の行 1 つ（種類・欄 status は字か null）。
fn row(id: &str, kind: &str, status: Option<&str>) -> String {
    let status = status.map_or("null".to_string(), |s| format!("\"{s}\""));
    format!(
        "{{\"id\":\"{id}\",\"kind\":\"{kind}\",\"file\":\"design-note/x.yaml\",\"status\":{status}}}\n"
    )
}

/// 要約の字（判断の記録の行 1 つと、設計ノートの行 5 つ）。
fn summary(states: &[(&str, Option<&str>)]) -> String {
    let mut text = row("ADR-1", "判断の記録", Some("accepted"));
    for (id, status) in states {
        text.push_str(&row(id, "設計ノートの行", *status));
    }
    text
}

/// 節の要約の字。
fn sum() -> String {
    summary(&STATES)
}

/// 節の要約の字の 1 つの行の状態だけを替えた字。
fn sum_with(id: &str, status: Option<&str>) -> String {
    let states: Vec<(&str, Option<&str>)> = STATES
        .into_iter()
        .map(|(row, s)| if row == id { (row, status) } else { (row, s) })
        .collect();
    summary(&states)
}

/// 節の要約の字から 1 つの行を除いた字。
fn sum_without(id: &str) -> String {
    let states: Vec<(&str, Option<&str>)> =
        STATES.into_iter().filter(|(row, _)| *row != id).collect();
    summary(&states)
}

fn owned(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

fn texts(design: &str, summary: &str) -> Texts {
    Texts {
        design: design.to_string(),
        summary: summary.to_string(),
        ..Texts::default()
    }
}

/// 設計の索引の字と要約の字を渡した挙げ方。
fn listed(design: &str, summary: &str) -> Reading<Vec<String>> {
    retired_notes(&built(&texts(design, "")), summary)
}

/// (1) 中核: 文書 id ごとの行の状態の字がちょうど retired の 1 つなら挙げ、読めない・揃わないは全体をまだ分からない。
#[test]
fn gmretw_core_lists_retired() {
    assert_eq!(listed(INDEX, &sum()), Reading::Known(owned(&["ra", "rd"])));
    let effective = summary(&STATES.map(|(id, _)| (id, Some("effective"))));
    assert_eq!(listed(INDEX, &effective), Reading::Known(Vec::new()));

    for (name, unread) in [
        ("空の要約", String::new()),
        ("JSON でない行を足した要約", format!("{}これは JSON でない\n", sum())),
        ("ra#r2 の行の無い要約", sum_without("ra#r2")),
        ("ra#r2 が effective の要約", sum_with("ra#r2", Some("effective"))),
        ("rb#r1 が null の要約", sum_with("rb#r1", None)),
        ("判断の記録の行だけの要約", row("ADR-1", "判断の記録", Some("accepted"))),
    ] {
        assert_eq!(listed(INDEX, &unread), Reading::Unknown, "{name}");
    }

    for unread in [String::new(), sum()] {
        assert_eq!(listed(NO_ROWS, &unread), Reading::Known(Vec::new()));
    }
    for summary in [String::new(), sum()] {
        assert_eq!(listed("", &summary), Reading::Unknown);
    }
}

/// (2) 境界: board::graph と board::doc は欄 retired に挙げた列を写し、読めなければ書かない。
#[test]
fn gmretw_wire_carries_retired() {
    let read = texts(INDEX, &sum());
    let wired = graph(&read);
    assert_eq!(wired.retired, Some(owned(&["ra", "rd"])));
    let text = wire::encode(&wired).expect("電文にできる");
    assert!(text.contains("\"retired\":[\"ra\",\"rd\"]"), "{text}");
    assert_eq!(wire::decode::<GraphDoc>(&text).expect("読み戻せる"), wired);

    let g = built(&read);
    let invariants = tsuzuri_core::graph::check(&g);
    assert_eq!(doc(&g, &invariants, &sum()).retired, wired.retired);
    assert_eq!(doc(&g, &invariants, "").retired, None);

    let bare = graph(&texts(INDEX, ""));
    assert_eq!(bare.retired, None);
    let text = wire::encode(&bare).expect("電文にできる");
    assert!(!text.contains("\"retired\""), "{text}");
    assert_eq!(wire::decode::<GraphDoc>(&text).expect("読み戻せる"), bare);

    for summary in [String::new(), sum()] {
        assert_eq!(graph(&texts(NO_ROWS, &summary)).retired, Some(Vec::new()));
    }
}

/// 計画の verify の filter の語を、ほかの語を部分の字として含まない語に畳んだ字（歯の名が含んではならない部分の字）。
const FILTERS: [&str; 222] = [
    "aaround_",
    "abss_",
    "abst_",
    "accept_",
    "acchold_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "aface_",
    "afocus_",
    "alean_",
    "aord_",
    "aown_",
    "apark_",
    "apop_",
    "areread_",
    "askcard_",
    "athr_",
    "batchpanel_",
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadl_",
    "cadopt_",
    "cadq_",
    "cdorm_",
    "cexcl_",
    "cfsplit_",
    "cg9_",
    "cgdom_",
    "cmark_",
    "cnote_",
    "cnret_",
    "contract_form_",
    "cround_",
    "csled_",
    "cspk_",
    "ctick_",
    "cupd_",
    "cupdlist_",
    "denv_",
    "dnedge_",
    "dngrp_",
    "dnrow_",
    "dnskip_",
    "dretry_",
    "ecache_",
    "epolq_",
    "eretry_",
    "esig_",
    "evkind_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "fserve_",
    "fstop_",
    "fxpre_",
    "g3g7_",
    "gacct_",
    "gapspage_",
    "gatt_",
    "gbnote_",
    "gchip_",
    "gcoach_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gins_",
    "gjst_",
    "glabel_",
    "gnav_",
    "gpface_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "gwv_",
    "hacols_",
    "harest_",
    "hasplit_",
    "hbconf_",
    "hbmark_",
    "hbon_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
    "hdchip_",
    "hfig_",
    "hnunk_",
    "hook_",
    "hruling_",
    "hsblock_",
    "hsderive_",
    "hshist_",
    "hspage_",
    "hspk_",
    "hsym_",
    "hthr_",
    "http_",
    "hwstore_",
    "iclose_",
    "ilink_",
    "jcount_",
    "jrun_",
    "kcli_",
    "kg9_",
    "kindlab_",
    "klink_",
    "ksum_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lgrp_",
    "lhome_",
    "lidle_",
    "lkind_",
    "lresume_",
    "lsnap_",
    "lspark_",
    "lstg_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
    "mqask_",
    "mqface_",
    "mstore_",
    "mtips_",
    "mtree_",
    "nact_",
    "nbatch_",
    "ncard_",
    "nextstep_",
    "nodepage_",
    "nstall_",
    "nsum_",
    "nsumw_",
    "ntc_",
    "ntime_",
    "nxact_",
    "nxorg_",
    "parts_",
    "pci_",
    "pclosed_",
    "pfold_",
    "pgsw_",
    "pgz_",
    "pipe_",
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
    "ptitle_",
    "pubfp_",
    "pubscan_",
    "punmap_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "qsig_",
    "question_",
    "rbusy_",
    "relay_",
    "rhold_",
    "runsdoc_",
    "rvk_",
    "saxis_",
    "sclosed_",
    "seatblock_",
    "seatcard_",
    "server::events::",
    "server_",
    "sesplit_",
    "sgrace_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "stbp_",
    "stcli_",
    "steady_",
    "sthr_",
    "stnfy_",
    "stskill_",
    "sttgt_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "tkad_",
    "tlic_",
    "topbar_",
    "topfit_",
    "tz_",
    "udacct_",
    "udash_",
    "unow_",
    "urpanel_",
    "uword_",
    "wstrip_",
];

/// (3) この file の歯の名。
#[test]
fn gmretw_names_clean() {
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/gmretw.rs"))
        .expect("この file");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("fn の行");
            rest.split('(').next().expect("fn の名")
        })
        .collect();
    assert_eq!(names.len(), 3, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("gmretw_")
            .unwrap_or_else(|| panic!("{name} は gmretw_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} は {word} を含む");
        }
    }
    for (i, a) in FILTERS.iter().enumerate() {
        for b in &FILTERS[i + 1..] {
            assert_ne!(a, b, "filter の語が重なる");
        }
    }
}
