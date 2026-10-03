//! 行 h-acct-split の歯: account の projects の表の hover の card を新しい module の account の cards へ移す。
//! 移した item は cards の path だけに在り、使う側の src の file は cards から use する。
//! card の値は着地済みの歯（hcproj・hcnx・hcsess・hcled・hacols・hnunk）が cards の path で見る。
#![cfg(test)]

use std::path::PathBuf;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// projects.rs から移した 8 つの関数。
const FNS: [&str; 8] = [
    "nx_card",
    "led_card",
    "pcnt_card",
    "orch_card",
    "gproj_card",
    "thr_line",
    "grp_card",
    "row_cards",
];

/// projects.rs から移した 6 つの字。
const STRS: [&str; 6] = [
    "NX_SRC",
    "NX_MISS",
    "LED_SRC",
    "RUNS_UNKNOWN",
    "WAIT_NOTE",
    "GPROJ_SRC",
];

/// 行の字がちょうどその字の行の数。
fn line_count(text: &str, line: &str) -> usize {
    text.lines().filter(|l| l.trim() == line).count()
}

/// (1) cards.rs は移した 8 つの関数と RowCards と 6 つの字を 1 度ずつ持ち、DOM を持たず、mod.rs は pub mod cards; を 1 つ持つ。
#[test]
fn hasplit_cards_file() {
    let cards = read("src/account/cards.rs");
    for f in FNS {
        assert_eq!(
            cards.matches(&format!("pub fn {f}(")).count(),
            1,
            "cards.rs の pub fn {f}"
        );
    }
    assert_eq!(cards.matches("pub struct RowCards").count(), 1);
    for s in STRS {
        assert_eq!(
            cards.matches(&format!("pub const {s}: &str")).count(),
            1,
            "cards.rs の pub const {s}"
        );
    }
    for w in ["mod dom", "view!", "target_arch", "leptos"] {
        assert!(!cards.contains(w), "cards.rs が {w} を含む");
    }
    let module = read("src/account/mod.rs");
    assert_eq!(line_count(&module, "pub mod cards;"), 1);
}

/// (2) projects.rs は移した item の宣言と pub use を持たず、使う側の 4 つの file は cards から use する。
#[test]
fn hasplit_callers_use_cards() {
    let projects = read("src/account/projects.rs");
    for f in FNS {
        assert!(
            !projects.contains(&format!("fn {f}(")),
            "projects.rs が fn {f} を宣言する"
        );
    }
    assert!(!projects.contains("struct RowCards"));
    for s in STRS.iter().chain(&["ASK"]) {
        assert!(
            !projects.contains(&format!("const {s}:")),
            "projects.rs が const {s} を宣言する"
        );
    }
    assert!(!projects.contains("pub use"));

    let callers = [
        (
            "projects",
            "use super::cards::{RowCards, grp_card, row_cards};",
        ),
        ("ledger", "use super::cards::led_card;"),
        ("session", "use super::cards::orch_card;"),
        ("home", "use super::cards::{gproj_card, nx_card};"),
    ];
    for (module, line) in callers {
        let text = read(&format!("src/account/{module}.rs"));
        assert_eq!(line_count(&text, line), 1, "{module}.rs の {line}");
        for l in text
            .lines()
            .filter(|l| l.trim_start().starts_with("use super::projects"))
        {
            for f in FNS {
                assert!(!l.contains(f), "{module}.rs の {l} が {f} を含む");
            }
        }
    }
}

/// filter の語（main の verify の filter の語を畳んだ語と、並行の起草の行と計画の後の行の接頭辞）。
const FILTERS: &[&str] = &[
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
    "afocus_",
    "aord_",
    "apop_",
    "askcard_",
    "athr_",
    "batchpanel_",
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadopt_",
    "cdorm_",
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "denv_",
    "dnrow_",
    "ecache_",
    "epolq_",
    "flight_",
    "fmark_",
    "frame_",
    "fserve_",
    "fstop_",
    "gapspage_",
    "gbnote_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gjst_",
    "glabel_",
    "gnav_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "hacols_",
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
    "hspage_",
    "hsym_",
    "iclose_",
    "ilink_",
    "kcli_",
    "klink_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lidle_",
    "lsnap_",
    "lspark_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
    "mstore_",
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
    "parts_",
    "pclosed_",
    "pfold_",
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
    "stcli_",
    "steady_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "topbar_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
    "pgz_",
    "fprem_",
    "gpface_",
    "dnkind_",
];

/// (5) この file の歯の名は 3 つで hasplit_ で始まり、名の全体は filter の語を含まない。
#[test]
fn hasplit_own_names_clean() {
    assert_eq!(FILTERS.len(), 149);
    let text = read("tests/teeth3/hasplit.rs");
    let lines: Vec<&str> = text.lines().collect();
    let mut names = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() != "#[test]" {
            continue;
        }
        let decl = lines[i + 1..]
            .iter()
            .find(|l| l.trim_start().starts_with("fn "))
            .expect("test の属性の後に fn が在る");
        let name = decl.trim_start()["fn ".len()..]
            .split('(')
            .next()
            .unwrap_or_default()
            .to_string();
        names.push(name);
    }
    assert_eq!(names.len(), 3, "{names:?}");
    for name in names {
        assert!(
            name.starts_with("hasplit_"),
            "歯の名 {name} が hasplit_ で始まらない"
        );
        for w in FILTERS {
            assert!(!name.contains(w), "歯の名 {name} が filter の語 {w} を含む");
        }
    }
}
