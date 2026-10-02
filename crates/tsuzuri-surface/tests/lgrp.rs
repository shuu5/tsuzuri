//! 行 g-ledger-group-kind の歯: 問いの一覧と台帳の一覧の組と並べは、欄 kind の字でなく地図の節点と同じ種類の読み
//! （LedgerRow の node_kind・issue_type と label intake:question・intake:memo）で決まる。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadId, LedgerList, LedgerRow, MEMO_LABEL, QUESTION_LABEL};
use tsuzuri_contract::wire;
use tsuzuri_surface::view::{self, Fetched, Screen};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// 台帳の 1 行（題は「題」・親は無し）。
fn row(id: &str, kind: &str, status: &str, labels: &[&str], at: u64) -> LedgerRow {
    LedgerRow {
        id: BeadId::new(id).expect("id"),
        kind: kind.to_string(),
        title: "題".to_string(),
        status: status.to_string(),
        updated_at: at,
        parent: None,
        labels: labels.iter().map(|l| l.to_string()).collect(),
    }
}

fn ids(rows: &[LedgerRow]) -> Vec<&str> {
    rows.iter().map(|r| r.id.as_str()).collect()
}

const Q: &str = QUESTION_LABEL;
const M: &str = MEMO_LABEL;

/// 節の台帳 LQ（5 行）。
fn lq() -> Vec<LedgerRow> {
    vec![
        row("fx-g.1", "task", "open", &[Q], 20),
        row("fx-g.2", "task", "open", &[Q], 10),
        row("fx-g.3", "task", "closed", &[Q], 5),
        row("fx-g.4", "question", "open", &[], 1),
        row("fx-g.5", "task", "open", &[M, Q], 2),
    ]
}

/// 節の台帳 LG（13 行・更新時刻はどれも 0）。
fn lg() -> Vec<LedgerRow> {
    vec![
        row("fx-e", "epic", "open", &[], 0),
        row("fx-e.1", "task", "open", &[M], 0),
        row("fx-e.2", "feature", "open", &[], 0),
        row("fx-e.3", "task", "open", &[Q], 0),
        row("fx-e.4", "memo", "open", &[], 0),
        row("fx-e.5", "task", "closed", &[], 0),
        row("fx-e.6", "question", "open", &[], 0),
        row("fx-e.10", "bug", "open", &[], 0),
        row("fx-m", "epic", "open", &[M], 0),
        row("fx-m.1", "task", "open", &[M], 0),
        row("fx-z.1", "task", "open", &[Q], 0),
        row("fx-z.2", "task", "open", &[M], 0),
        row("fx-z.3", "chore", "open", &[], 0),
    ]
}

/// 節の台帳 LS（6 行・更新時刻はどれも 0）。
fn ls() -> Vec<LedgerRow> {
    vec![
        row("fx-s", "epic", "open", &[], 0),
        row("fx-s.1", "task", "open", &[], 0),
        row("fx-s.2", "task", "open", &[M], 0),
        row("fx-s.3", "task", "open", &[Q], 0),
        row("fx-s.4", "task", "closed", &[Q], 0),
        row("fx-s.5", "task", "closed", &[], 0),
    ]
}

/// (1) 問いの一覧は label intake:question の open の行を古い順（label の無い issue_type question と memo の行は入れない）。
#[test]
fn lgrp_questions_follow_labels() {
    assert_eq!(ids(&view::questions(&lq())), vec!["fx-g.2", "fx-g.1"]);
}

/// (2) 組の頭は epic・問いは入れない・組の下は契約（feature・bug・chore と label の無い字を含む）・memo の順。
#[test]
fn lgrp_groups_follow_node_kind() {
    let groups = view::ledger_groups(&lg());
    let heads: Vec<Option<&str>> = groups
        .iter()
        .map(|g| g.epic.as_ref().map(|e| e.id.as_str()))
        .collect();
    assert_eq!(heads, vec![Some("fx-e"), Some("fx-m"), None]);
    assert_eq!(
        ids(&groups[0].children),
        vec!["fx-e.2", "fx-e.4", "fx-e.5", "fx-e.6", "fx-e.10", "fx-e.1"]
    );
    assert_eq!(ids(&groups[1].children), vec!["fx-m.1"]);
    assert_eq!(ids(&groups[2].children), vec!["fx-z.3", "fx-z.2"]);
}

/// (3) 本物の台帳の形の電文を読んだ画面: 問いの一覧は open の問いだけ（台帳の件数と一覧の中身の関数は行 g-list-sweep で外した）。
#[test]
fn lgrp_screen_real_ledger() {
    let body = wire::encode(&LedgerList {
        rows: Reading::Known(ls()),
    })
    .expect("電文");
    let screen = Screen::initial().after_read(&Fetched::Body(body), 100);
    let Reading::Known(board) = &screen.board else {
        panic!("画面の台帳が読めていない");
    };
    assert_eq!(ids(&board.questions), vec!["fx-s.3"]);
}

/// main b5c24c8 の contracts の verify の filter の語 272 を、ほかの語を部分の字として含まない語に畳んだ語。
const FILTERS: &[&str] = &[
    "aaround_", "accept_", "acchold_", "account_", "acctcore_", "acctdoc_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "aface_", "afocus_", "alean_", "aord_", "aown_", "apop_", "askcard_", "athr_", "batchpanel_",
    "bhalf_", "board_min_", "bport_", "brand_", "btuck_", "cadl_", "cadopt_", "cadq_", "cdorm_",
    "cfsplit_", "cg9_", "cgdom_", "cmark_", "cnote_", "contract_form_", "cround_", "csled_",
    "cspk_", "ctick_", "cupd_", "cupdlist_", "denv_", "dnedge_", "dngrp_", "dnrow_", "dnskip_",
    "ecache_", "epolq_", "eretry_", "evkind_", "flight_", "fmark_", "fprem_", "frame_", "fserve_",
    "fstop_", "fxpre_", "g3g7_", "gapspage_", "gatt_", "gbnote_", "gcoach_", "gfix_", "gfresh_",
    "ghb_", "gjst_", "glabel_", "gnav_", "gpface_", "gpill_", "gpulse_", "graph_", "gsum_",
    "gtuck_", "gview_", "gwv_", "hacols_", "harest_", "hasplit_", "hbconf_", "hbmark_", "hbpost_",
    "hbproc_", "hbroute_", "hcard_", "hcled_", "hcnx_", "hcproj_", "hcsess_", "hdchip_", "hfig_",
    "hnunk_", "hook_", "hruling_", "hsblock_", "hsderive_", "hshist_", "hspage_", "hspk_", "hsym_",
    "hthr_", "http_", "hwstore_", "iclose_", "ilink_", "jcount_", "jrun_", "kcli_", "kg9_",
    "kindlab_", "klink_", "ksum_", "launch_", "lcard_", "ledgerblock_", "lhome_", "lidle_",
    "lkind_", "lresume_", "lsnap_", "lspark_", "lstg_", "lstore_", "mapview_", "mkeys_", "mlink_",
    "mstore_", "mtips_", "mtree_", "nact_", "nbatch_", "ncard_", "nextstep_", "nodepage_",
    "nstall_", "nsum_", "nsumw_", "ntime_", "nxact_", "nxorg_", "parts_", "pci_", "pclosed_",
    "pfold_", "pgsw_", "pgz_", "pipe_", "plimit_", "pmisfit_", "pmore_", "pquest_", "pqueue_",
    "project_", "ptitle_", "pubscan_", "punmap_", "pwhole_", "qblock_", "qgate_", "qkey_",
    "question_", "rbusy_", "relay_", "rhold_", "runsdoc_", "rvk_", "saxis_", "sclosed_",
    "seatblock_", "seatcard_", "server_", "sesplit_", "shb_", "skeleton_", "smore_", "stage_",
    "stats_", "stbp_", "stcli_", "steady_", "sttgt_", "sxaxis_", "ticker_", "tipx_", "tkad_",
    "tlic_", "topbar_", "topfit_", "tz_", "unow_", "urpanel_", "uword_", "wstrip_",
];

/// (6) この file の歯の名は 4 つで lgrp_ で始まり、名は filter の語を含まない。
#[test]
fn lgrp_names_clean() {
    assert_eq!(FILTERS.len(), 197);
    let src = std::fs::read_to_string(crate_dir().join("tests/lgrp.rs")).expect("tests/lgrp.rs");
    let mut names = Vec::new();
    let mut lines = src.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "#[test]" {
            continue;
        }
        let next = lines.next().expect("属性の次の行");
        let name = next
            .trim()
            .strip_prefix("fn ")
            .and_then(|s| s.split('(').next())
            .unwrap_or_else(|| panic!("属性の次が fn でない: {next}"));
        names.push(name.to_string());
    }
    assert_eq!(names.len(), 4, "{names:?}");
    for name in &names {
        assert!(name.starts_with("lgrp_"), "{name} が lgrp_ で始まらない");
        for word in FILTERS {
            assert!(!name.contains(word), "{name} が {word} を含む");
        }
    }
}
