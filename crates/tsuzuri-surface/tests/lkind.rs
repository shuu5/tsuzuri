//! 行 g-ledger-kind の歯: 台帳の一覧の項の種類の語と状態の語と印の赤を、地図の節点の種類と同じ読み
//! （契約の crate の `bead_kind`）にする。台帳の行は歯の中で組む（fixture の file は使わない・bead の id の接頭辞は fx-k）。

use std::path::PathBuf;

use tsuzuri_contract::graph::NodeKind;
use tsuzuri_contract::ledger::{BeadId, LedgerRow, MEMO_LABEL, QUESTION_LABEL, bead_kind};
use tsuzuri_surface::project::{Staged, item, kind_word, staged_item};
use tsuzuri_surface::view::{MEMO_OPEN_KEY, mark, row_mark};
use tsuzuri_surface::vocab::vocab;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 台帳の 1 行（題は「題」・時刻は 0・親は無し）。
fn row(id: &str, kind: &str, status: &str, labels: &[&str]) -> LedgerRow {
    LedgerRow {
        id: BeadId::new(id).expect("id"),
        kind: kind.to_string(),
        title: "題".to_string(),
        status: status.to_string(),
        updated_at: 0,
        parent: None,
        labels: labels.iter().map(|l| l.to_string()).collect(),
    }
}

/// (1) bead_kind と LedgerRow の node_kind は epic・memo・問い・契約の順に決める。
#[test]
fn lkind_bead_kind_order() {
    let cases: [(&str, &[&str], NodeKind); 4] = [
        ("epic", &[MEMO_LABEL], NodeKind::Epic),
        ("task", &[QUESTION_LABEL, MEMO_LABEL], NodeKind::Memo),
        ("task", &[QUESTION_LABEL], NodeKind::Question),
        ("feature", &[], NodeKind::Task),
    ];
    for (issue_type, labels, want) in cases {
        let owned: Vec<String> = labels.iter().map(|l| l.to_string()).collect();
        assert_eq!(bead_kind(issue_type, &owned), want, "{issue_type} {labels:?}");
        let r = row("fx-k.1", issue_type, "open", labels);
        assert_eq!(r.node_kind(), want, "{issue_type} {labels:?}");
    }
}

/// (3) open の問いは「◷ 答え待ち」、open の memo は語の辞書の鍵 memo_unknown の語、ほかは status の mark。
#[test]
fn lkind_row_mark() {
    assert_eq!(MEMO_OPEN_KEY, "memo_unknown");

    let q = row_mark(&row("fx-k.1", "task", "open", &[QUESTION_LABEL]));
    assert_eq!((q.glyph, q.word, q.class), ("◷", "答え待ち", "st-open"));

    let memo_word = &vocab().term(MEMO_OPEN_KEY).expect("鍵 memo_unknown").label;
    assert_eq!(memo_word, "まだ分からない");
    let m = row_mark(&row("fx-k.2", "task", "open", &[MEMO_LABEL]));
    assert_eq!((m.glyph, m.word, m.class), ("?", memo_word.as_str(), "st-open"));

    for r in [
        row("fx-k.3", "task", "closed", &[QUESTION_LABEL]),
        row("fx-k.4", "task", "in_progress", &[MEMO_LABEL]),
        row("fx-k.5", "task", "open", &[]),
        row("fx-k", "epic", "open", &[MEMO_LABEL]),
    ] {
        assert_eq!(row_mark(&r), mark(&r.status), "{}", r.id);
    }
}

/// (4) 項の右の字と赤と種類の語は地図の節点と同じ読み（段の在る項は段の字と種類の語）。
#[test]
fn lkind_items() {
    let question = row("fx-k.1", "task", "open", &[QUESTION_LABEL]);
    let memo = row("fx-k.2", "task", "open", &[MEMO_LABEL]);
    let feature = row("fx-k.3", "feature", "open", &[]);
    let epic = row("fx-k", "epic", "in_progress", &[]);
    let closed_q = row("fx-k.4", "task", "closed", &[QUESTION_LABEL]);

    let want = [
        (&question, "◷ 答え待ち · question", "question"),
        (&memo, "? まだ分からない · memo", "memo"),
        (&feature, "○ 未着手 · task", "task"),
        (&epic, "◐ 作業中 · epic", "epic"),
    ];
    for (r, aside, word) in want {
        assert_eq!(item(r).aside, aside, "{}", r.id);
        assert_eq!(kind_word(r), word, "{}", r.id);
    }

    assert!(item(&question).alert);
    assert!(!item(&memo).alert);
    assert!(!item(&closed_q).alert);

    let run = Staged {
        state: Some("run"),
        closed: false,
        word: "Running".to_string(),
    };
    let staged = staged_item(&memo, Some(&run));
    assert_eq!(staged.aside, "Running · memo");
    assert_eq!(staged.stage, Some(run));
}

/// verify の filter の語（main の 264 語を畳んだ 190 語と同じノートのもう 1 つの行の接頭辞・191 語）。
const FILTERS: [&str; 191] = [
    "aaround_",
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
    "apop_",
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
    "cfsplit_",
    "cg9_",
    "cgdom_",
    "cmark_",
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
    "ecache_",
    "epolq_",
    "eretry_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "fserve_",
    "fstop_",
    "fxpre_",
    "g3g7_",
    "gapspage_",
    "gatt_",
    "gbnote_",
    "gcoach_",
    "gfix_",
    "gfresh_",
    "ghb_",
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
    "kindlab_",
    "klink_",
    "ksum_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lidle_",
    "lresume_",
    "lsnap_",
    "lspark_",
    "lstg_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
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
    "ntime_",
    "nxact_",
    "nxorg_",
    "parts_",
    "pci_",
    "pclosed_",
    "pfold_",
    "pgz_",
    "pipe_",
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
    "ptitle_",
    "punmap_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
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
    "server_",
    "sesplit_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "stbp_",
    "stcli_",
    "steady_",
    "sttgt_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "tkad_",
    "tlic_",
    "topbar_",
    "topfit_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
    "unow_",
];

/// (8) この file の歯の名は 4 つで、どれも lkind_ で始まり、filter の語を含まない。
#[test]
fn lkind_names_clean() {
    let text = read("tests/lkind.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == "#[test]")
        .map(|(i, _)| {
            lines[i + 1..]
                .iter()
                .find_map(|l| l.strip_prefix("fn "))
                .and_then(|rest| rest.split('(').next())
                .expect("test の属性の後の fn")
        })
        .collect();
    assert_eq!(names.len(), 4, "{names:?}");
    for name in names {
        assert!(name.starts_with("lkind_"), "{name} が lkind_ で始まらない");
        for word in FILTERS {
            assert!(!name.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
