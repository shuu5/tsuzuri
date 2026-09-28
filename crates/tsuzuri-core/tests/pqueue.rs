//! 行 c-pipe-queue の歯（中核）: 走行を 1 つも持たない open の契約は、acceptance に器の読める設計 pointer の行を持つ
//! task の bead だけを Blocked か Queued の札にする（読みは器の列の受付と同じ）。
//! 節の組は歯の中で組む（fixture の file は pqueue_fixture_runless だけが読む）。

use std::path::PathBuf;

use serde_json::{Value, json};
use tsuzuri_contract::board::{PipelineCard, Reading, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_core::pipeline::{Board, board};

/// 節の今（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

/// 台帳の bead の 1 本（題は t と空白と id・欄 acceptance_criteria は design の字が在るときだけ）。
fn bead(id: &str, status: &str, design: Option<&str>) -> Value {
    let mut b = json!({"id": id, "title": format!("t {id}"), "status": status, "issue_type": "task"});
    if let Some(d) = design {
        b["acceptance_criteria"] = json!(d);
    }
    b
}

/// 節の台帳の一覧（pq.1 から pq.10）。
fn ledger() -> String {
    let mut pq3 = bead("pq.3", "open", None);
    pq3["acceptance_criteria"] = json!("受け入れの字");
    let mut pq4 = bead("pq.4", "open", Some("design = contracts/x.toml#b"));
    pq4["dependencies"] = json!([{"issue_id": "pq.4", "depends_on_id": "pq.1", "type": "blocks"}]);
    let mut pq6 = bead("pq.6", "open", Some("design = contracts/x.toml#d"));
    pq6["labels"] = json!(["intake:memo"]);
    let mut pq7 = bead("pq.7", "open", Some("design = contracts/x.toml#e"));
    pq7["labels"] = json!(["intake:question"]);
    let mut pq8 = bead("pq.8", "open", Some("design = contracts/x.toml#f"));
    pq8["issue_type"] = json!("epic");
    Value::Array(vec![
        bead("pq.1", "open", Some("design = contracts/x.toml#a")),
        bead("pq.2", "open", None),
        pq3,
        pq4,
        bead("pq.5", "in_progress", Some("design = docs/design/x.md#c")),
        pq6,
        pq7,
        pq8,
        bead("pq.9", "closed", Some("design = contracts/x.toml#g")),
        bead("pq.10", "open", None),
    ])
    .to_string()
}

/// 節の event log（pq.10 の走行 1 つ・2 行）。
fn events() -> String {
    let run = "pq.10-20260927T110000Z";
    [
        json!({"schema": 1, "ts": "2026-09-27T11:00:00Z", "kind": "RunCreated", "run": run, "bead": "pq.10", "stage": "Intake"}),
        json!({"schema": 1, "ts": "2026-09-27T11:01:00Z", "kind": "RunStage", "run": run, "bead": "pq.10", "stage": "Spawned"}),
    ]
    .iter()
    .map(Value::to_string)
    .collect::<Vec<_>>()
    .join("\n")
}

fn card(id: &str, runs: u32, stage: Stage, elapsed: Option<u64>) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(id).expect("bead の id"),
        runs,
        stage,
        reason: None,
        account: None,
        elapsed_s: elapsed,
    }
}

fn cards(b: &Board) -> &[PipelineCard] {
    match &b.board.cards {
        Reading::Known(c) => c,
        Reading::Unknown => panic!("札が Unknown"),
    }
}

/// (1) 節の組の板は pq.10・pq.1・pq.4・pq.5 の 4 枚でこの順、unmapped 0。台帳の字が空の板は pq.10 の 1 枚だけ。
#[test]
fn pqueue_runless_cards() {
    let events = events();
    let b = board(&ledger(), &events, NOW);
    assert_eq!(
        cards(&b),
        [
            card("pq.10", 1, Stage::Running, Some(3540)),
            card("pq.1", 0, Stage::Queued, None),
            card("pq.4", 0, Stage::Blocked, None),
            card("pq.5", 0, Stage::Queued, None),
        ]
        .as_slice()
    );
    assert_eq!(b.unmapped, 0);

    let b = board("", &events, NOW);
    assert_eq!(
        cards(&b),
        [card("pq.10", 1, Stage::Running, Some(3540))].as_slice()
    );
    assert_eq!(b.unmapped, 0);
}

/// open の task 1 本（欄 key に字 text）の台帳の板の札の数。
fn count(key: &str, text: &str) -> usize {
    let mut b = json!({"id": "pf.1", "title": "t pf.1", "status": "open", "issue_type": "task"});
    b[key] = json!(text);
    let events = json!({"schema": 1, "ts": "2026-09-27T11:00:00Z", "kind": "SeatRegistered"}).to_string();
    cards(&board(&Value::Array(vec![b]).to_string(), &events, NOW)).len()
}

/// (2) 器の受付と同じ読みで、pointer の字の表の 1 枚の字は 1 枚、0 枚の字は 0 枚。
#[test]
fn pqueue_pointer_forms() {
    for text in [
        "design = contracts/x.toml#a",
        "  design = contracts/x.toml#a  ",
        "受け入れの字\ndesign = docs/design/x.md#z\n本文",
    ] {
        assert_eq!(count("acceptance_criteria", text), 1, "{text:?}");
    }
    assert_eq!(count("acceptance", "design = contracts/x.toml#a"), 1, "別名 acceptance");
    for text in [
        "",
        "design=contracts/x.toml#a",
        "design = contracts/x.toml",
        "design = contracts/x.toml#a#b",
        "design = contracts/x.toml#",
        "design = contracts/x.toml#a b",
        "design = contracts/x.yaml#a",
        "design = contracts/x#a",
        "design = x.txt#a\ndesign = contracts/x.toml#a",
        "research = contracts/x.toml#a",
    ] {
        assert_eq!(count("acceptance_criteria", text), 0, "{text:?}");
    }
}

/// (3) fixture の fx-pl.13 と fx-pl.14 は pointer を持ち、fx-pl.17 は走行も pointer も無く、runs 0 の期待の札は 2 枚だけ。
#[test]
fn pqueue_fixture_runless() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/pipeline/pipeline.json");
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("fixture を読む"))
        .expect("fixture の JSON");
    let row = |id: &str| {
        v["ledger"]
            .as_array()
            .expect("ledger")
            .iter()
            .find(|b| b["id"] == id)
            .unwrap_or_else(|| panic!("{id} の行"))
            .clone()
    };
    assert_eq!(row("fx-pl.13")["acceptance_criteria"], "design = contracts/fx.toml#pl-13");
    assert_eq!(row("fx-pl.14")["acceptance_criteria"], "design = contracts/fx.toml#pl-14");
    let pl17 = row("fx-pl.17");
    assert_eq!((&pl17["status"], &pl17["issue_type"]), (&json!("open"), &json!("task")));
    assert!(pl17.get("acceptance_criteria").is_none());
    let events = v["events"].as_array().expect("events");
    assert!(!events.iter().any(|e| e["bead"] == "fx-pl.17"));

    let want: Vec<PipelineCard> = serde_json::from_value(v["cards"].clone()).expect("期待の札");
    let runless: Vec<(&str, Stage)> = want
        .iter()
        .filter(|c| c.runs == 0)
        .map(|c| (c.contract.as_str(), c.stage))
        .collect();
    assert_eq!(
        runless,
        [("fx-pl.13", Stage::Blocked), ("fx-pl.14", Stage::Queued)]
    );

    let got = board(
        &v["ledger"].to_string(),
        &events.iter().map(Value::to_string).collect::<Vec<_>>().join("\n"),
        v["now"].as_u64().expect("now"),
    );
    assert!(!cards(&got).iter().any(|c| c.contract.as_str() == "fx-pl.17"));
    assert_eq!(got.board.cards, Reading::Known(want));
}

/// 着地済みの verify の filter の語を畳んだ 136 語と、後の行 g-gz・c-pipe-misfit・g-pipe-misfit の接頭辞。
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
    "pmisfit_",
    "gfix_",
];

/// (6) この file の歯の名はどれも pqueue_ で始まり、残りの字は filter の語を含まない。
#[test]
fn pqueue_own_names_clean() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/pqueue.rs");
    let text = std::fs::read_to_string(&path).expect("tests/pqueue.rs を読む");
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
            .strip_prefix("pqueue_")
            .unwrap_or_else(|| panic!("{name} が pqueue_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
