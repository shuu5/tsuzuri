//! 行 g-closed-mark の歯（面）: 台帳で閉じた（着地せず）の札の表の印（Kcard の closed・kcard_view の ✕ と段の字）・
//! fixture の札の値・mod dom の字・歯の名。

use std::path::PathBuf;

use tsuzuri_contract::board::{PipelineBoard, PipelineCard, Reading, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::wire;
use tsuzuri_surface::project::pipeline::{CLOSED_STAGE, CLOSED_TAG, closed_card, kcard};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn card(id: &str, stage: Stage, reason: Option<String>) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(id).expect("bead の id"),
        runs: 1,
        stage,
        reason,
        account: None,
        elapsed_s: Some(60),
    }
}

/// (1) kcard の closed は closed_card の値で、閉じた札も state は None・class は kcard のまま。
#[test]
fn cmark_kcard_closed_follows_rule() {
    let yes = [
        card("px.40", Stage::Landed, Some(format!("{CLOSED_TAG}x"))),
        card("px.41", Stage::Landed, Some(CLOSED_TAG.to_string())),
    ];
    for c in &yes {
        let k = kcard(c, &[]);
        assert!(closed_card(c));
        assert!(k.closed, "{} は閉じた札", k.id);
        assert_eq!(k.state, None);
        assert_eq!(k.class, "kcard");
    }
    let no = [
        card("px.42", Stage::Landed, None),
        card("px.43", Stage::Failed, Some(format!("{CLOSED_TAG}x"))),
        card("px.44", Stage::Running, Some(format!("{CLOSED_TAG}x"))),
        card("px.45", Stage::Landed, Some("Closed:x".to_string())),
    ];
    for c in &no {
        let k = kcard(c, &[]);
        assert!(!closed_card(c));
        assert!(!k.closed, "{} は閉じた札でない", k.id);
    }
}

/// (2) fixture の板の 12 枚の札はどれも閉じた札でない。
#[test]
fn cmark_fixture_cards_not_closed() {
    let board: PipelineBoard = wire::decode(&read("../../tests/fixtures/surface/pipeline-board.json"))
        .expect("fixture の板が電文として読める");
    let Reading::Known(cards) = board.cards else {
        panic!("fixture の札が Unknown");
    };
    assert_eq!(cards.len(), 12);
    let reasoned: Vec<&str> = cards
        .iter()
        .filter(|c| c.reason.is_some())
        .map(|c| c.contract.as_str())
        .collect();
    assert_eq!(reasoned, vec!["px.5", "px.6"]);
    for c in &cards {
        assert!(
            !c.reason.as_deref().is_some_and(|r| r.starts_with(CLOSED_TAG)),
            "{} の理由が CLOSED_TAG で始まる",
            c.contract
        );
        assert!(!kcard(c, &[]).closed, "{} は閉じた札でない", c.contract);
    }
}

/// (3) mod dom の kcard_view は閉じた札に ✕ と段の字を出し、ほかの札の記号と a の字は今のまま。
#[test]
fn cmark_dom_text() {
    let src = read("src/project/pipeline.rs");
    let dom = &src[src.find("mod dom {").expect("mod dom の字")..];
    let view = &dom[dom.find("fn kcard_view(").expect("fn kcard_view の字")..];
    for s in [
        "card.closed",
        "CROSS",
        "CLOSED_STAGE",
        "var(--ink-3)",
        "CHECK",
        "var(--s-land)",
        "<a class=card.class href=",
        "use:attach=card.hover.clone()",
    ] {
        assert!(view.contains(s), "kcard_view の後に {s} が無い");
    }
    assert!(dom.contains("M6 6l12 12M18 6L6 18"), "mod dom に IC.cross の path が無い");
    assert_eq!(CLOSED_STAGE, "閉じた（着地せず）");
}

/// 着地済みの行と後の行と同じ波の行の verify の filter の語。
const FILTERS: &[&str] = &[
    "aaround_",
    "accept_",
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
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadopt_",
    "cgdom_",
    "contract_form_",
    "cround_",
    "csled_",
    "denv_",
    "flight_",
    "fmark_",
    "frame_",
    "fserve_",
    "fstop_",
    "gapspage_",
    "gfresh_",
    "ghb_",
    "glabel_",
    "gnav_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
    "hfig_",
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
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lspark_",
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
    "pgz_",
    "pipe_",
    "plimit_",
    "pmore_",
    "pquest_",
    "project_",
    "ptitle_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "question_",
    "rhold_",
    "runsdoc_",
    "saxis_",
    "seatblock_",
    "seatcard_",
    "server_",
    "sesplit_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "steady_",
    "stlaunch_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "topbar_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
];

/// (5) この file の歯の名はどれも cmark_ で始まり、残りの字は filter の語を含まない。
#[test]
fn cmark_own_names_clean() {
    let text = read("tests/cmark.rs");
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
    assert!(names.len() >= 3, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("cmark_")
            .unwrap_or_else(|| panic!("{name} が cmark_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
