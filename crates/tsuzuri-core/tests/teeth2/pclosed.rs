//! 行 c-pipe-closed の歯（中核）: 台帳で閉じた bead の走行の札は閉じた（着地せず）として段 Landed に置き、
//! account board の run の 4 列も同じ読み替えで数える。節の組は歯の中で組む（fixture の file は使わない）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::common::{NOW, cards, host};
use serde_json::{Value, json};
use tsuzuri_contract::account::RunCounts;
use tsuzuri_contract::board::{NextMove, PipelineCard, Reading, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::stats::CheckResult;
use tsuzuri_core::account::project::{ProjectTexts, project_rows, run_counts, run_counts_of};
use tsuzuri_core::next_step::next_step;
use tsuzuri_core::pipeline::{CLOSED_CHARS, CLOSED_TAG, board};

/// cs.1 の閉じた理由（改行と空白 2 つを間に持つ）。
const CS1_REASON: &str = "行を取り下げた。\n新しい行 cs.6 が  同じ直しを持つので、この行の走行は着地させずに閉じる（席の決め・2026-09-27 の点検で重なりを見つけた）";

/// 節の cs.1 の段の理由。
const CS1_HEAD: &str = "closed:行を取り下げた。 新しい行 cs.6 が 同じ直しを持つので、この行の走行は着地させずに閉じる（席の決め・2026-09";

fn bead(id: &str, n: u32, status: &str, closed_at: Option<&str>, reason: Option<&str>) -> Value {
    let mut b = json!({"id": id, "title": format!("t{n}"), "status": status, "issue_type": "task"});
    if let Some(at) = closed_at {
        b["closed_at"] = json!(at);
    }
    if let Some(r) = reason {
        b["close_reason"] = json!(r);
    }
    b
}

/// 節の台帳の一覧（10 本）。
fn ledger() -> String {
    Value::Array(vec![
        bead("cs.1", 1, "closed", Some("2026-09-27T10:00:00Z"), Some(CS1_REASON)),
        bead("cs.2", 2, "closed", Some("2026-09-27T09:00:00Z"), Some("superseded by cs.6")),
        bead("cs.3", 3, "closed", None, None),
        bead("cs.4", 4, "closed", Some("2026-09-26T20:00:00Z"), Some("withdrawn")),
        bead("cs.5", 5, "closed", Some("2026-09-27T11:00:00Z"), Some("landed 0a ci=success")),
        bead("cs.6", 6, "open", None, None),
        bead("cs.7", 7, "in_progress", None, None),
        bead("cs.9", 9, "closed", Some("2026-09-27T06:00:00Z"), Some("x")),
        bead("cs.11", 11, "closed", Some("2026-09-27T06:00:00Z"), Some("no runs")),
        {
            // 走行の無い open の task は、器の読める設計 pointer を持つときだけ札になる（行 c-pipe-queue）。
            let mut b = bead("cs.12", 12, "open", None, None);
            b["acceptance_criteria"] = json!("design = contracts/pc.toml#cs-12");
            b
        },
    ])
    .to_string()
}

/// event の 1 行（run は bead とハイフンと RunCreated の時刻の札）。
fn ev(ts: &str, kind: &str, run: &str, stage: Option<&str>, detail: Option<&str>) -> String {
    let (bead, _) = run.rsplit_once('-').expect("run の id");
    let mut e = json!({"schema": 1, "ts": ts, "kind": kind, "run": run, "bead": bead});
    if let Some(s) = stage {
        e["stage"] = json!(s);
    }
    if let Some(d) = detail {
        e["detail"] = json!(d);
    }
    e.to_string()
}

/// 節の event log（20 行）。
fn events() -> String {
    let d = |t: &str| format!("2026-09-27T{t}:00Z");
    let created = |t: &str, run: &str| ev(&d(t), "RunCreated", run, Some("Intake"), None);
    let stage = |t: &str, run: &str, s: &str, detail: Option<&str>| {
        ev(&d(t), "RunStage", run, Some(s), detail)
    };
    [
        created("08:00", "cs.1-20260927T080000Z"),
        stage(
            "08:02",
            "cs.1-20260927T080000Z",
            "Reviewed",
            Some("verdict:FAIL kind:other"),
        ),
        created("07:00", "cs.2-20260927T070000Z"),
        stage(
            "07:01",
            "cs.2-20260927T070000Z",
            "Spawned",
            Some("base:0a,account:acct-1"),
        ),
        created("06:00", "cs.3-20260927T060000Z"),
        stage("06:30", "cs.3-20260927T060000Z", "Gated", Some("verdict:PASS")),
        events_cs4(),
        created("09:00", "cs.5-20260927T090000Z"),
        stage("09:10", "cs.5-20260927T090000Z", "Gated", Some("verdict:FAIL")),
        created("10:00", "cs.5-20260927T100000Z"),
        ev(
            &d("10:30"),
            "RunDone",
            "cs.5-20260927T100000Z",
            Some("Landed"),
            Some("sha:0a main:0a"),
        ),
        created("08:30", "cs.6-20260927T083000Z"),
        stage(
            "08:40",
            "cs.6-20260927T083000Z",
            "Gated",
            Some("verdict:FAIL,account:acct-2"),
        ),
        created("11:00", "cs.7-20260927T110000Z"),
        stage("11:30", "cs.7-20260927T110000Z", "Implemented", None),
        created("07:30", "cs.8-20260927T073000Z"),
        stage(
            "07:31",
            "cs.8-20260927T073000Z",
            "Reviewed",
            Some("verdict:INCONCLUSIVE"),
        ),
        created("05:00", "cs.9-20260927T050000Z"),
        stage("05:10", "cs.9-20260927T050000Z", "Merging", None),
    ]
    .join("\n")
}

/// 節の event log の cs.4 の 2 行（前の日の RunCreated と QuestionRaised）。
fn events_cs4() -> String {
    [
        ev(
            "2026-09-26T18:00:00Z",
            "RunCreated",
            "cs.4-20260926T180000Z",
            Some("Intake"),
            None,
        ),
        ev(
            "2026-09-26T18:05:00Z",
            "QuestionRaised",
            "cs.4-20260926T180000Z",
            None,
            None,
        ),
    ]
    .join("\n")
}

fn card(
    id: &str,
    runs: u32,
    (stage, reason): (Stage, Option<&str>),
    account: Option<&str>,
    elapsed: Option<u64>,
) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(id).expect("bead の id"),
        runs,
        stage,
        reason: reason.map(str::to_string),
        account: account.map(str::to_string),
        since: elapsed.map(|e| NOW - e),
        ci: None,
    }
}

/// 台帳が読めるときも読めないときも同じ札（cs.5・cs.6・cs.7・cs.8）。
fn shared() -> [PipelineCard; 4] {
    [
        card("cs.5", 2, (Stage::Landed, None), None, Some(5400)),
        card(
            "cs.6",
            1,
            (Stage::Failed, Some("verdict:FAIL,account:acct-2")),
            Some("acct-2"),
            Some(12000),
        ),
        card("cs.7", 1, (Stage::Running, None), None, Some(1800)),
        card(
            "cs.8",
            1,
            (Stage::Failed, Some("verdict:INCONCLUSIVE")),
            None,
            Some(16140),
        ),
    ]
}

/// (1) 節の組の板は節の表の 10 枚で、台帳で閉じた bead の札は段 Landed。
#[test]
fn pclosed_board_settles_closed_beads() {
    assert_eq!(CLOSED_TAG, "closed:");
    assert_eq!(CLOSED_CHARS, 60);
    let b = board(&ledger(), &events(), NOW);
    let [cs5, cs6, cs7, cs8] = shared();
    let want = vec![
        card(
            "cs.1",
            1,
            (Stage::Landed, Some(CS1_HEAD)),
            None,
            Some(14280),
        ),
        card(
            "cs.2",
            1,
            (Stage::Landed, Some("closed:superseded by cs.6")),
            Some("acct-1"),
            Some(17940),
        ),
        card(
            "cs.3",
            1,
            (Stage::Landed, Some("closed:")),
            None,
            Some(19800),
        ),
        card(
            "cs.4",
            1,
            (Stage::Landed, Some("closed:withdrawn")),
            None,
            Some(64500),
        ),
        cs5,
        cs6,
        cs7,
        cs8,
        card(
            "cs.9",
            1,
            (Stage::Landed, Some("closed:x")),
            None,
            Some(24600),
        ),
        card("cs.12", 0, (Stage::Queued, None), None, None),
    ];
    assert_eq!(cards(&b), want.as_slice());
    assert_eq!(b.unmapped, 0);
    for c in cards(&b) {
        if ["cs.1", "cs.2", "cs.3", "cs.4", "cs.5", "cs.9"].contains(&c.contract.as_str()) {
            assert_eq!(c.stage, Stage::Landed, "{}", c.contract);
        }
    }
}

/// 閉じた契約 1 本と Failed の走行 1 つの板の、札の段の理由。
fn one_reason(reason: Option<&str>) -> Option<String> {
    let ledger = Value::Array(vec![bead(
        "rh.1",
        1,
        "closed",
        Some("2026-09-27T10:00:00Z"),
        reason,
    )])
    .to_string();
    let run = "rh.1-20260927T080000Z";
    let events = [
        ev("2026-09-27T08:00:00Z", "RunCreated", run, Some("Intake"), None),
        ev(
            "2026-09-27T08:05:00Z",
            "RunStage",
            run,
            Some("Reviewed"),
            Some("verdict:FAIL"),
        ),
    ]
    .join("\n");
    let b = board(&ledger, &events, NOW);
    let [c] = cards(&b) else {
        panic!("札が 1 枚でない: {b:?}");
    };
    assert_eq!(c.stage, Stage::Landed);
    c.reason.clone()
}

/// (2) 段の理由の CLOSED_TAG の後は、空白を畳み前後を除いた閉じた理由の頭の 60 字（印は足さない）。
#[test]
fn pclosed_reason_head_60() {
    let a60 = "あ".repeat(60);
    assert_eq!(
        one_reason(Some(&a60)),
        Some(format!("{CLOSED_TAG}{a60}"))
    );
    assert_eq!(
        one_reason(Some(&"い".repeat(61))),
        Some(format!("{CLOSED_TAG}{}", "い".repeat(60)))
    );
    assert_eq!(
        one_reason(Some("  a \n\t b  ")),
        Some(format!("{CLOSED_TAG}a b"))
    );
    for r in [Some(""), Some("   "), None] {
        assert_eq!(one_reason(r), Some(CLOSED_TAG.to_string()), "{r:?}");
    }
    let b = board(&ledger(), &events(), NOW);
    let cs1 = cards(&b)
        .iter()
        .find(|c| c.contract.as_str() == "cs.1")
        .expect("cs.1 の札");
    assert_eq!(
        cs1.reason.as_deref().map(|r| r.chars().count()),
        Some(67)
    );
}

/// (3) 台帳が読めない板は閉じた bead の札も最後の event の段のまま。
#[test]
fn pclosed_board_without_ledger() {
    let b = board("", &events(), NOW);
    let [cs5, cs6, cs7, cs8] = shared();
    let want = vec![
        card(
            "cs.1",
            1,
            (Stage::Failed, Some("verdict:FAIL kind:other")),
            None,
            Some(14280),
        ),
        card(
            "cs.2",
            1,
            (Stage::Running, None),
            Some("acct-1"),
            Some(17940),
        ),
        card("cs.3", 1, (Stage::Gated, None), None, Some(19800)),
        card("cs.4", 1, (Stage::Questioned, None), None, Some(64500)),
        cs5,
        cs6,
        cs7,
        cs8,
    ];
    assert_eq!(cards(&b), want.as_slice());
    assert_eq!(b.unmapped, 1);
}

/// (4) 止まっている走行は open の bead の札だけを数え、止まり・動いている段の札の bead は open か台帳に無い。
#[test]
fn pclosed_next_step_counts_open_only() {
    let ledger = ledger();
    let step = next_step(&ledger, &events(), NOW);
    let stalled = step
        .checks
        .iter()
        .find(|c| c.kind == NextMove::StalledRun)
        .expect("止まっている走行の結果");
    assert_eq!(
        (stalled.result, stalled.count, stalled.target.as_ref().map(BeadId::as_str)),
        (CheckResult::Hit, 1, Some("cs.6"))
    );
    assert_eq!(step.lead, NextMove::StalledRun);

    let rows: Vec<Value> = serde_json::from_str(&ledger).expect("台帳の字");
    let status = |id: &str| {
        rows.iter()
            .find(|r| r["id"] == id)
            .map(|r| r["status"].as_str().unwrap_or_default().to_string())
    };
    let b = board(&ledger, &events(), NOW);
    let mut seen = Vec::new();
    for c in cards(&b) {
        if matches!(
            c.stage,
            Stage::Failed | Stage::Stopped | Stage::Running | Stage::Gated | Stage::Questioned
        ) {
            let s = status(c.contract.as_str());
            assert!(
                s.as_deref().is_none_or(|s| s != "closed"),
                "{} は閉じている",
                c.contract
            );
            seen.push(c.contract.to_string());
        }
    }
    assert_eq!(seen, ["cs.6", "cs.7", "cs.8"]);
}

fn counts(wait: u32, run: u32, stop: u32, land: u32) -> Reading<RunCounts> {
    Reading::Known(RunCounts {
        wait,
        run,
        stop,
        land,
    })
}

/// (5) run の 4 列は台帳で閉じた bead の最新の run を着地と読み替え、台帳が無いか読めなければ今のまま。
#[test]
fn pclosed_run_counts_with_ledger() {
    let (ev, led) = (events(), ledger());
    assert_eq!(run_counts_of(Some(&ev), Some(&led), NOW), counts(0, 1, 2, 5));
    assert_eq!(
        run_counts_of(Some(&ev), Some(&led), NOW + 86_400),
        counts(0, 1, 2, 0)
    );
    for now in [NOW, NOW + 86_400] {
        let plain = run_counts(Some(&ev), now);
        assert_eq!(run_counts_of(Some(&ev), None, now), plain);
        assert_eq!(run_counts_of(Some(&ev), Some("not json"), now), plain);
    }
    assert_eq!(run_counts(Some(&ev), NOW), counts(1, 3, 4, 1));
    for e in [None, Some("")] {
        assert_eq!(run_counts_of(e, Some(&led), NOW), Reading::Unknown);
    }
}

fn texts(ledger: Option<String>) -> BTreeMap<String, ProjectTexts> {
    BTreeMap::from([(
        "/w/p".to_string(),
        ProjectTexts {
            state_dir_known: true,
            events: Some(events().into()),
            ledger: ledger.map(Into::into),
            ..ProjectTexts::default()
        },
    )])
}

/// (6) project の行の runs は project の event log と台帳の字で数える。
#[test]
fn pclosed_rows_pass_ledger() {
    for (ledger, want) in [
        (Some(ledger()), counts(0, 1, 2, 5)),
        (None, counts(1, 3, 4, 1)),
    ] {
        let rows = project_rows(&host(), &texts(ledger), NOW);
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert_eq!(rows[0].runs, want);
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
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/teeth2/pclosed.rs");
    let text = std::fs::read_to_string(&path).expect("tests/teeth2/pclosed.rs を読む");
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
    assert!(names.len() >= 7, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("pclosed_")
            .unwrap_or_else(|| panic!("{name} が pclosed_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
