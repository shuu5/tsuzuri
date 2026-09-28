//! 行 c-pipe-misfit の歯（中核と電文）: 板は器の doctor の台帳の形の行を写した形の崩れた open の bead の一覧を持つ
//! （tsuzuri は形を判じない・判断の記録 ADR-16 の決定 (6)）。
//! 節の組は歯の中で組む（fixture の file は使わない）。

use std::path::PathBuf;

use serde_json::{Value, json};
use tsuzuri_contract::board::{Misfit, MisfitBead, PipelineBoard, PipelineCard, Reading, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::wire;
use tsuzuri_core::pipeline::{
    Board, FORM_FIELDS, FORM_PREFIX, board, board_with_doctor, form_ids,
};

/// 節の今（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

/// 節の 37 字の題。
const LONG_TITLE: &str = "題の頭  0123456789 0123456789 0123456789";

/// 節の doctor の字（台帳の形の行の前後にほかの行・mf.6 は台帳で閉じ、mf.9 は台帳に無い）。
const DOCTOR: &str = "scribe2 0.1.0\n\
ledger: open=7 contracts=1\n\
ledger-form: open=7 memos=2 no-source=0 no-observation=0 no-candidate=0 no-promotion=0 shaped=6 both=1:mf.4 neither=5:mf.1,mf.5,mf.6,mf.7,mf.9 contracts=1 undiscovered=0 unlanded=0 drift=0 settled=0\n\
host-guard: on\n";

/// 器の e2e の歯の行（器の repo の crates/scribe2-boundary/tests/e2e/ledger_form.rs の 205 行から 207 行の字）。
const E2E: &str = "ledger-form: open=18 memos=10 no-source=1:s2-f.1 no-observation=2:s2-f.2,s2-f.3 no-candidate=0 no-promotion=3:s2-f.4,s2-f.5,s2-f.6 shaped=16 both=1:s2-f.b1 neither=2:s2-f.n1,s2-f.n2 contracts=4 undiscovered=2:s2-f.c1,s2-f.c2 unlanded=2 drift=1:toy#b settled=1:s2-f.8";

/// この repo の 2026-09-28 の台帳の行（崩れ 0）。
const ZERO: &str = "ledger-form: open=15 memos=4 no-source=0 no-observation=0 no-candidate=0 no-promotion=0 shaped=10 both=0 neither=0 contracts=6 undiscovered=0 unlanded=0 drift=0 settled=0";

/// 台帳の bead の 1 本（題は t と空白と id）。
fn bead(id: &str, status: &str, labels: &[&str], design: Option<&str>) -> Value {
    let mut b = json!({"id": id, "title": format!("t {id}"), "status": status, "issue_type": "task"});
    if !labels.is_empty() {
        b["labels"] = json!(labels);
    }
    if let Some(d) = design {
        b["acceptance_criteria"] = json!(d);
    }
    b
}

/// 節の台帳の一覧（mf.1 から mf.8）。
fn ledger() -> String {
    let mut mf7 = bead("mf.7", "open", &[], None);
    mf7["title"] = json!(LONG_TITLE);
    let mut mf8 = bead("mf.8", "open", &[], None);
    mf8["issue_type"] = json!("epic");
    Value::Array(vec![
        bead("mf.1", "open", &[], None),
        bead("mf.2", "open", &[], Some("design = contracts/x.toml#a")),
        bead("mf.3", "open", &["intake:memo"], None),
        bead(
            "mf.4",
            "open",
            &["intake:memo"],
            Some("design = contracts/x.toml#b"),
        ),
        bead("mf.5", "open", &["intake:question"], None),
        bead("mf.6", "closed", &[], None),
        mf7,
        mf8,
    ])
    .to_string()
}

/// 節の event log（mf.1 の走行 1 つ・2 行）。
fn events() -> String {
    let run = "mf.1-20260927T110000Z";
    [
        json!({"schema": 1, "ts": "2026-09-27T11:00:00Z", "kind": "RunCreated", "run": run, "bead": "mf.1", "stage": "Intake"}),
        json!({"schema": 1, "ts": "2026-09-27T11:01:00Z", "kind": "RunStage", "run": run, "bead": "mf.1", "stage": "Spawned"}),
    ]
    .iter()
    .map(Value::to_string)
    .collect::<Vec<_>>()
    .join("\n")
}

fn id(s: &str) -> BeadId {
    BeadId::new(s).expect("bead の id")
}

fn misfit(bead: &str, title: &str, misfit: Misfit) -> MisfitBead {
    MisfitBead {
        bead: id(bead),
        title: title.to_string(),
        misfit,
    }
}

/// 節の期待の一覧（台帳の順）。
fn want_misfits() -> Vec<MisfitBead> {
    vec![
        misfit("mf.1", "t mf.1", Misfit::Neither),
        misfit("mf.4", "t mf.4", Misfit::Both),
        misfit("mf.5", "t mf.5", Misfit::Neither),
        misfit("mf.7", LONG_TITLE, Misfit::Neither),
    ]
}

/// 節の期待の札。
fn want_cards() -> Vec<PipelineCard> {
    vec![
        PipelineCard {
            contract: id("mf.1"),
            runs: 1,
            stage: Stage::Running,
            reason: None,
            account: None,
            elapsed_s: Some(3540),
        },
        PipelineCard {
            contract: id("mf.2"),
            runs: 0,
            stage: Stage::Queued,
            reason: None,
            account: None,
            elapsed_s: None,
        },
    ]
}

fn pairs(list: &[(&str, Misfit)]) -> Vec<(String, Misfit)> {
    list.iter().map(|(i, m)| (i.to_string(), *m)).collect()
}

/// (1) 電文の語と形と読み戻し、欄 cards だけの字は読めない。
#[test]
fn pmisfit_wire_words() {
    let words: Vec<String> = Misfit::ALL
        .iter()
        .map(|m| wire::encode(m).expect("電文"))
        .collect();
    assert_eq!(words, ["\"neither\"", "\"both\""]);
    assert_eq!(
        wire::encode(&misfit("mf.1", "t", Misfit::Both)).expect("電文"),
        r#"{"bead":"mf.1","title":"t","misfit":"both"}"#
    );
    let b = PipelineBoard {
        cards: Reading::Unknown,
        misfits: Reading::Known(vec![misfit("mf.1", "t", Misfit::Neither)]),
    };
    let text = wire::encode(&b).expect("電文");
    assert_eq!(
        text,
        r#"{"cards":"unknown","misfits":{"known":[{"bead":"mf.1","title":"t","misfit":"neither"}]}}"#
    );
    assert_eq!(wire::decode::<PipelineBoard>(&text).expect("読み戻し"), b);
    assert!(wire::decode::<PipelineBoard>(r#"{"cards":"unknown"}"#).is_err());
}

/// (2) 台帳の形の行の both と neither の id を欄の順に写す。
#[test]
fn pmisfit_form_line_ids() {
    assert_eq!(FORM_PREFIX, "ledger-form:");
    assert_eq!(
        FORM_FIELDS,
        [("both", Misfit::Both), ("neither", Misfit::Neither)]
    );
    assert_eq!(
        form_ids(E2E),
        Some(pairs(&[
            ("s2-f.b1", Misfit::Both),
            ("s2-f.n1", Misfit::Neither),
            ("s2-f.n2", Misfit::Neither),
        ]))
    );
    assert_eq!(form_ids(ZERO), Some(vec![]));
    assert_eq!(
        form_ids(DOCTOR),
        Some(pairs(&[
            ("mf.4", Misfit::Both),
            ("mf.1", Misfit::Neither),
            ("mf.5", Misfit::Neither),
            ("mf.6", Misfit::Neither),
            ("mf.7", Misfit::Neither),
            ("mf.9", Misfit::Neither),
        ]))
    );
    assert_eq!(
        form_ids("x\nledger-form: neither=1:a.1 open=2 both=1:b.1"),
        Some(pairs(&[("b.1", Misfit::Both), ("a.1", Misfit::Neither)]))
    );
    let ids: Vec<String> = (1..=30).map(|n| format!("m.{n}")).collect();
    let line = format!("ledger-form: both=0 neither=30:{}", ids.join(","));
    assert_eq!(
        form_ids(&line),
        Some(ids.into_iter().map(|i| (i, Misfit::Neither)).collect())
    );
}

/// (3) 読めない行の表の 15 の字はどれも None。
#[test]
fn pmisfit_form_line_broken() {
    for text in [
        "",
        "scribe2 0.1.0\nledger: open=7",
        "ledger-form: unreadable reason=ledger-unreadable",
        "ledger-form: unreadable reason=ledger-timeout",
        "ledger-form: both=0 neither=0\nledger-form: both=0 neither=0",
        " ledger-form: both=0 neither=0",
        "ledger-form: neither=0",
        "ledger-form: both=0",
        "ledger-form: both=0 both=0 neither=0",
        "ledger-form: both=2:a.1 neither=0",
        "ledger-form: both=1 neither=0",
        "ledger-form: both=x neither=0",
        "ledger-form: both=0 neither=2:a.1,",
        "ledger-form: both=1:a.1 neither=1:a.1",
        "ledger-form: both= neither=0",
    ] {
        assert_eq!(form_ids(text), None, "{text:?}");
    }
}

fn parts(b: &Board) -> (&Reading<Vec<PipelineCard>>, u32) {
    (&b.board.cards, b.unmapped)
}

/// (4) 節の組の一覧は台帳の順の open の 4 本で、札は board と同じ。
#[test]
fn pmisfit_board_joins_titles() {
    let (ledger, events) = (ledger(), events());
    let got = board_with_doctor(&ledger, &events, Some(DOCTOR), NOW);
    assert_eq!(got.board.misfits, Reading::Known(want_misfits()));
    assert_eq!(got.board.cards, Reading::Known(want_cards()));
    assert_eq!(got.unmapped, 0);
    let plain = board(&ledger, &events, NOW);
    assert_eq!(parts(&plain), parts(&got));
    assert_eq!(plain.board.misfits, Reading::Unknown);
    let zero = board_with_doctor(&ledger, &events, Some(ZERO), NOW);
    assert_eq!(zero.board.misfits, Reading::Known(vec![]));
    assert_eq!(parts(&zero), parts(&got));
}

/// (5) doctor の字か台帳が読めなければ一覧は Unknown、event log が読めなくても一覧は Known。
#[test]
fn pmisfit_unknown_sources() {
    let (ledger, events) = (ledger(), events());
    let plain = board(&ledger, &events, NOW);
    for doctor in [
        None,
        Some("scribe2 0.1.0\nledger: open=7"),
        Some("ledger-form: unreadable reason=ledger-unreadable"),
    ] {
        let got = board_with_doctor(&ledger, &events, doctor, NOW);
        assert_eq!(got.board.misfits, Reading::Unknown, "{doctor:?}");
        assert_eq!(parts(&got), parts(&plain), "{doctor:?}");
    }
    let no_ledger = board_with_doctor("", &events, Some(DOCTOR), NOW);
    assert_eq!(no_ledger.board.misfits, Reading::Unknown);
    assert_eq!(
        no_ledger.board.cards,
        Reading::Known(want_cards()[..1].to_vec())
    );
    let no_events = board_with_doctor(&ledger, "", Some(DOCTOR), NOW);
    assert_eq!(no_events.board.cards, Reading::Unknown);
    assert_eq!(no_events.unmapped, 0);
    assert_eq!(no_events.board.misfits, Reading::Known(want_misfits()));
}

/// 着地済みの verify の filter の語を畳んだ 137 語と、後の行 g-gz・g-pipe-misfit の接頭辞。
const FILTERS: [&str; 139] = [
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
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadopt_",
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "denv_",
    "ecache_",
    "epolq_",
    "flight_",
    "fmark_",
    "frame_",
    "fserve_",
    "fstop_",
    "gapspage_",
    "gbnote_",
    "gfresh_",
    "ghb_",
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
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
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
    "gfix_",
];

/// file の字の test の属性の付いた fn の名の全部。
fn names(path: &PathBuf) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path:?} を読む: {e}"));
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == "#[test]")
        .map(|(i, _)| {
            lines[i + 1..]
                .iter()
                .find_map(|l| l.strip_prefix("fn "))
                .and_then(|rest| rest.split('(').next())
                .expect("test の属性の後の fn")
                .to_string()
        })
        .collect()
}

/// (12) 2 つの file の歯の名はどれも pmisfit_ で始まり、残りの字は filter の語を含まない。
#[test]
fn pmisfit_own_names_clean() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let core = names(&dir.join("tests/pmisfit.rs"));
    let boundary = names(&dir.join("../tsuzuri-boundary/tests/pmisroute.rs"));
    assert!(core.len() >= 6, "中核の歯の数 {}", core.len());
    assert!(boundary.len() >= 4, "境界の歯の数 {}", boundary.len());
    for name in core.iter().chain(&boundary) {
        let rest = name
            .strip_prefix("pmisfit_")
            .unwrap_or_else(|| panic!("{name} が pmisfit_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
