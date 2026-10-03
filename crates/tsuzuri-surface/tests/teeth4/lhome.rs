//! 行 g-ledger-home の歯: 一覧に出さない状態の字と、閉じていない行が無いときの 1 行と、台帳の block の DOM の字。
//! 閉じた bead を出さない一覧（持ち主の裁定 t3-hub.52.31）は module ledgerlist が描き、歯 bvlist が見る。
//! 前の一覧の組と件数の関数（`listed`・`body`・`count`）の歯は行 g-list-sweep で外した。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::project::ledger::{CLOSED, EMPTY, FOLDS, NO_OPEN};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// (5) 一覧に出さない状態の字と閉じていない行が無いときの 1 行は今のまま・畳みは今のまま・block の中身は module ledgerlist の
/// view を描き（行 g-list-groups）、見出しの件数の chip は無い（配置の表と件数の chip は行 g-ledger-trim で外した）。
#[test]
fn lhome_list_draws_body() {
    assert_eq!(CLOSED, "closed");
    assert_eq!(NO_OPEN, "閉じていない bead は無い");
    assert_ne!(NO_OPEN, EMPTY);
    assert!(!NO_OPEN.trim().is_empty() && !NO_OPEN.contains('\n'));
    assert_eq!(FOLDS, &["ledger:unref"]);

    let src = read("src/project/ledger.rs");
    let at = src.find("mod dom {").expect("mod dom の字");
    let dom = &src[at..];
    assert!(dom.contains("section(BLOCK, extra.into_any(), crate::ledgerlist::view())"));
    assert!(!dom.contains("screen.with(count)"));
}

/// 着地済みの行と第 3 波から第 8 波の行の verify の filter の語。
const FILTERS: &[&str] = &[
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
    "gsum_", "nstall_", "pfold_", "uword_", "cround_", "cgdom_", "csled_", "shb_", "ghb_",
    "nact_", "aord_", "mtree_",
];

/// (8) この file の歯の名はどれも lhome_ で始まり、残りの字は filter の語を含まない。
#[test]
fn lhome_own_names_clean() {
    assert_eq!(FILTERS.len(), 106);
    let src = read("tests/teeth4/lhome.rs");
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
    assert_eq!(names.len(), 2, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("lhome_")
            .unwrap_or_else(|| panic!("{name} が lhome_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
