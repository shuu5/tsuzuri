//! 行 c-pipe-closed の歯（面）: 台帳で閉じた bead の着地しなかった札（段 Landed・段の理由の頭が `CLOSED_TAG`）の
//! 見分け・段の字・hover の詳しく・Landed の列の今日の札・中核の字の写し・歯の名。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::board::{PipelineCard, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_surface::project::pipeline::{
    CLOSED_STAGE, CLOSED_TAG, Lead, closed_card, columns, kcard,
};
use tsuzuri_surface::vocab::vocab;

/// 着地済みの pipe_ の歯と同じ今（UTC の日の正午）。
const NOW: u64 = 1_790_510_400;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn card(
    id: &str,
    runs: u32,
    (stage, reason): (Stage, Option<String>),
    account: Option<&str>,
    elapsed: Option<u64>,
) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(id).expect("bead の id"),
        runs,
        stage,
        reason,
        account: account.map(str::to_string),
        since: elapsed.map(|e| NOW - e),
        ci: None,
    }
}

fn tagged(rest: &str) -> Option<String> {
    Some(format!("{CLOSED_TAG}{rest}"))
}

fn px30() -> PipelineCard {
    card(
        "px.30",
        2,
        (Stage::Landed, tagged(&"a".repeat(40))),
        Some("acct-1"),
        Some(3600),
    )
}

fn px31() -> PipelineCard {
    card("px.31", 1, (Stage::Landed, tagged("x")), None, Some(60))
}

fn px32() -> PipelineCard {
    card("px.32", 1, (Stage::Landed, None), None, Some(120))
}

fn px33() -> PipelineCard {
    card("px.33", 1, (Stage::Failed, tagged("x")), None, Some(30))
}

fn px34() -> PipelineCard {
    card("px.34", 1, (Stage::Landed, tagged("y")), None, Some(1_468_800))
}

/// (7) closed_card は段の列が Landed で段の理由が CLOSED_TAG で始まるときだけ真。
#[test]
fn pclosed_closed_card_rule() {
    let one = |stage, reason: Option<String>| card("px.1", 1, (stage, reason), None, None);
    assert!(closed_card(&one(Stage::Landed, tagged("x"))));
    assert!(closed_card(&one(Stage::Landed, tagged(""))));
    assert!(!closed_card(&one(Stage::Landed, None)));
    assert!(!closed_card(&one(
        Stage::Landed,
        Some(format!("x {CLOSED_TAG}"))
    )));
    assert!(!closed_card(&one(
        Stage::Landed,
        Some("Closed:x".to_string())
    )));
    assert!(!closed_card(&one(Stage::Failed, tagged("x"))));
    assert!(!closed_card(&one(Stage::Running, tagged("x"))));
}

/// (8) 閉じた（着地せず）の札は閉じた印を持ち、回数の lead で状態の記号の値が無く、ほかの札は今のまま
/// （札の hover の card と値の行は行 g-dead-sweep-a で消した）。
#[test]
fn pclosed_kcard_hover_and_lines() {
    let k = kcard(&px30(), &[], NOW);
    assert!(k.closed);
    assert_eq!(k.lead, Lead::Runs(2));
    assert_eq!(k.state, None);
    assert_eq!(k.class, "kcard");
    other_cards();
}

/// px.31 から px.33 の札の閉じた印と lead。
fn other_cards() {
    assert!(kcard(&px31(), &[], NOW).closed);
    assert!(!kcard(&px32(), &[], NOW).closed);
    let k = kcard(&px33(), &[], NOW);
    assert!(!k.closed);
    assert_eq!(k.lead, Lead::Why(format!("{CLOSED_TAG}x")));
    assert_eq!(k.class, "kcard why-stop");
}

/// (9) 閉じた（着地せず）の札も Landed の列は今日の札だけ。
#[test]
fn pclosed_land_lane_today_only() {
    let cols = columns(&[px30(), px31(), px32(), px33(), px34()], &[], NOW);
    let ids: Vec<Vec<&str>> = cols
        .iter()
        .map(|c| c.cards.iter().map(|k| k.id.as_str()).collect())
        .collect();
    assert_eq!(
        ids,
        vec![
            vec![],
            vec![],
            vec![],
            vec!["px.33"],
            vec!["px.31", "px.32", "px.30"],
        ]
    );
}

/// (10) 面の CLOSED_TAG は中核の字の写しで、CLOSED_STAGE は語の辞書の見出しの語でない。
#[test]
fn pclosed_tag_copies_core() {
    let core = read("../tsuzuri-core/src/pipeline.rs");
    assert!(
        core.lines()
            .any(|l| l.trim() == "pub const CLOSED_TAG: &str = \"closed:\";"),
        "中核に CLOSED_TAG の宣言の行が無い"
    );
    assert_eq!(CLOSED_TAG, "closed:");
    assert_eq!(CLOSED_STAGE, "閉じた（着地せず）");
    for key in vocab().keys() {
        let term = vocab().term(key).expect("鍵の語");
        assert_ne!(term.label, CLOSED_STAGE, "鍵 {key} の見出しの語");
    }
}

/// 着地済みの行と第 3 波から第 9 波の行と表示面の行の verify の filter の語（114 語）。
const FILTERS: [&str; 114] = [
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
    "pwhole_",
];

/// (12) この file の歯の名はどれも pclosed_ で始まり、残りの字は filter の語を含まない。
#[test]
fn pclosed_own_names_clean() {
    let text = read("tests/teeth4/pclosed.rs");
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
    assert!(names.len() >= 5, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("pclosed_")
            .unwrap_or_else(|| panic!("{name} が pclosed_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
