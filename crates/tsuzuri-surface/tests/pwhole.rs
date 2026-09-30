//! 行 g-policy-all の歯: 全体への指示の block は範囲の切り替えと問いの選びを持たず、指示の本文の範囲はつねに all・
//! 字の欄と送る button は問いの一覧を読まずに 1 度だけ組む・自分の歯の名は着地済みの verify の filter の語を含まない。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::surface::PolicyRequest;
use tsuzuri_contract::wire;
use tsuzuri_surface::project::policy;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 着地済みの行と第 3 波から第 9 波の行と表示面の行の verify の filter の語（この行の接頭辞は並べない）。
const FILTERS: [&str; 113] = [
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
    "stage_term_",
    "stage_cdp_",
];

/// (1) 指示の本文は逐語だけを受け、範囲はつねに all・逐語は渡した字のまま。
#[test]
fn pwhole_request_always_all() {
    assert_eq!(policy::ALL_SCOPE, "all");
    for words in ["  全体に小さく刻む\n  ", "qa.2", "x"] {
        let req: PolicyRequest =
            wire::decode(&policy::request_body(words)).expect("要求の電文");
        assert_eq!(req.scope, "all", "{words:?}");
        assert_eq!(req.scope, policy::ALL_SCOPE);
        assert_eq!(req.verbatim, words, "{words:?}");
    }
}

/// (2) policy.rs に範囲の切り替えと問いの選びの字が無く、batchpanel.rs に Scope とその歯が無い。
#[test]
fn pwhole_toggle_gone() {
    let src = read("src/project/policy.rs");
    for word in [
        "この論点",
        "Scope",
        "aria-pressed",
        "<select",
        "net::read",
        "\"seg\"",
        "\"全体\"",
        "\"範囲\"",
    ] {
        assert!(!src.contains(word), "policy.rs に {word} が在る");
    }
    assert!(
        !src.to_lowercase().contains("topic"),
        "policy.rs に問いの選びの字が在る"
    );
    let teeth = read("tests/batchpanel.rs");
    for word in ["Scope", "fn batchpanel_policy_scope"] {
        assert!(!teeth.contains(word), "batchpanel.rs に {word} が在る");
    }
}

/// (3) DOM は字の欄と送る button を 1 度だけ組み、範囲を選ばず要求の本文を組まない。
#[test]
fn pwhole_dom_form_once() {
    let src = read("src/project/policy.rs");
    assert_eq!(src.matches("mod dom {").count(), 1, "mod dom の数");
    let (_, dom) = src.split_once("mod dom {").expect("mod dom");
    for word in [
        "{form(s)}",
        "request_body(",
        "crate::net::post(PATH, body)",
        "key_action(composing,",
    ] {
        assert!(dom.contains(word), "DOM に {word} が無い");
    }
    for word in ["move || form(", "scope", "PolicyRequest"] {
        assert!(!dom.contains(word), "DOM に {word} が在る");
    }
}

/// (5) この file の歯の名は pwhole_ で始まり、残りの字は filter の語を含まない。
#[test]
fn pwhole_own_names_clean() {
    let text = read("tests/pwhole.rs");
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
    assert!(names.len() >= 4, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("pwhole_")
            .unwrap_or_else(|| panic!("{name} が pwhole_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
