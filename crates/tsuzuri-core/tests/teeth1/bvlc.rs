//! 行 c-ledger-lc の歯（中核・接頭辞 bvlc_）: 局面の出力が読めれば、走行の在る札の段も契約の部品とその最新の便の部品の
//! 局面で決める（便の局面は閉じた 14 語の表・契約の待ちは部品の理由と since・表に無い語は札を作らず unmapped に数える）。
//! 部品が無いか契約が閉じた局面か便の部品が無ければ event log の段のまま。台帳で閉じた bead の札と CI の読みは今までどおり。
//! 節の組は歯の中で組む（台帳の一覧・event log・局面の出力の字）。局面の語の閉じた列は fixture の phases の欄から読む。
#![cfg(test)]

use std::path::Path;

use serde_json::{Value, json};
use tsuzuri_contract::board::{Ci, PipelineCard, Reading, Stage};
use tsuzuri_contract::case::CaseDoc;
use tsuzuri_core::case::cases_of;
use tsuzuri_core::pipeline::{Board, CLOSED_TAG, RUN_PHASES, board, board_with_cases};

/// 節の今（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

/// 段を決めた event の時刻（11:10）と部品の since（11:20 と 11:30）の epoch 秒。
const AT: u64 = NOW - 3000;
const RUN_SINCE: u64 = NOW - 2400;
const WAIT_SINCE: u64 = NOW - 1800;

/// 便の局面の語と板の段（器の case-lifecycle §2.1 の写し・歯が字で持つ）。
const WANT: [(&str, Stage); 14] = [
    ("run-intake", Stage::Running),
    ("run-reviewed", Stage::Running),
    ("run-review-failed", Stage::Failed),
    ("run-blocked", Stage::Blocked),
    ("run-implementing", Stage::Running),
    ("run-asking", Stage::Questioned),
    ("run-rate-limited", Stage::Running),
    ("run-gating", Stage::Running),
    ("run-landing", Stage::Gated),
    ("run-gate-failed", Stage::Failed),
    ("run-ci-waiting", Stage::Landed),
    ("run-landed-open", Stage::Landed),
    ("run-stopped", Stage::Stopped),
    ("run-failed", Stage::Failed),
];

fn run_id(bead: &str) -> String {
    format!("{bead}-20260927T110000Z")
}

/// 節の台帳（lc.1〜lc.6 の配れる task・lc.5 は閉じた）。
fn ledger() -> String {
    let beads: Vec<Value> = (1..=6)
        .map(|n| {
            let id = format!("lc.{n}");
            let status = if n == 5 { "closed" } else { "open" };
            json!({"id": id, "title": format!("t {id}"), "status": status, "issue_type": "task",
                "close_reason": "済んだ", "acceptance_criteria": format!("design = contracts/x.toml#{id}")})
        })
        .collect();
    Value::Array(beads).to_string()
}

/// 走行 1 つの event log（bead ごとに RunCreated と、段を決めた最後の event 1 つ）。
fn events(runs: &[(&str, &str, Option<&str>, &str)]) -> String {
    runs.iter()
        .flat_map(|&(bead, kind, stage, detail)| {
            let run = run_id(bead);
            [
                json!({"schema": 1, "ts": "2026-09-27T11:00:00Z", "kind": "RunCreated", "run": run, "bead": bead, "stage": "Intake"}),
                json!({"schema": 1, "ts": "2026-09-27T11:10:00Z", "kind": kind, "run": run, "bead": bead, "stage": stage, "detail": detail}),
            ]
        })
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// 契約の部品（局面・理由・結び runs の便）。
fn contract(id: &str, phase: &str, reason: Option<&str>, runs: &[String]) -> Value {
    json!({"part": "contract", "id": id, "phase": phase, "turn": "vessel", "since": "2026-09-27T11:30:00Z",
        "reason": reason, "closed": false, "overdue": null, "links": {"on": [], "runs": runs}})
}

/// 便の部品（局面・理由）。
fn run_part(bead: &str, phase: &str, reason: &str) -> Value {
    json!({"part": "run", "id": run_id(bead), "phase": phase, "turn": "vessel", "since": "2026-09-27T11:20:00Z",
        "reason": reason, "closed": false, "overdue": null, "links": {}, "bead": bead})
}

/// 走る契約 `bead` とその便の部品の組。
fn running(bead: &str, phase: &str) -> Vec<Value> {
    vec![
        contract(bead, "contract-running", Some(phase), &[run_id(bead)]),
        run_part(bead, phase, "R"),
    ]
}

fn doc(parts: Vec<Value>) -> CaseDoc {
    cases_of(&json!({"version": 1, "parts": parts}).to_string(), None)
}

fn card<'b>(b: &'b Board, id: &str) -> Option<&'b PipelineCard> {
    let Reading::Known(cards) = &b.board.cards else {
        panic!("札が Unknown");
    };
    cards.iter().find(|c| c.contract.as_str() == id)
}

/// 札の段と理由と since（札が無ければ落ちる）。
fn row(b: &Board, id: &str) -> (Stage, Option<String>, Option<u64>) {
    let c = card(b, id).unwrap_or_else(|| panic!("{id} の札が無い"));
    (c.stage, c.reason.clone(), c.since)
}

fn manifest(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn bvlc_run_phases_cover_vessel_words() {
    assert_eq!(RUN_PHASES, WANT);
    let fixture: Value =
        serde_json::from_str(&manifest("../../tests/fixtures/case/lifecycle.json"))
            .expect("fixture");
    let words: Vec<&str> = fixture["phases"]
        .as_array()
        .expect("phases の欄")
        .iter()
        .filter_map(Value::as_str)
        .filter(|w| w.starts_with("run-"))
        .collect();
    assert_eq!(words, WANT.map(|(w, _)| w), "器の便の局面の閉じた列");
}

#[test]
fn bvlc_run_stage_from_run_part() {
    let log = events(&[("lc.1", "RunStage", Some("Spawned"), "")]);
    for (word, stage) in WANT {
        let b = board_with_cases(&ledger(), &log, &doc(running("lc.1", word)), NOW);
        assert_eq!(row(&b, "lc.1"), (stage, None, Some(AT)), "{word}");
        assert_eq!(b.unmapped, 0, "{word}");
    }
}

#[test]
fn bvlc_event_unmapped_carded_from_part() {
    let log = events(&[("lc.2", "RunStage", Some("RateLimited"), "")]);
    let plain = board(&ledger(), &log, NOW);
    assert!(card(&plain, "lc.2").is_none());
    assert_eq!(plain.unmapped, 1);
    let mut parts = running("lc.2", "run-rate-limited");
    parts[1]["reason"] = json!("RateLimited");
    let b = board_with_cases(&ledger(), &log, &doc(parts), NOW);
    assert_eq!(
        row(&b, "lc.2"),
        (Stage::Running, Some("RateLimited".into()), Some(RUN_SINCE))
    );
    assert_eq!(b.unmapped, 0);
}

#[test]
fn bvlc_unknown_phase_is_unmapped() {
    let log = events(&[("lc.1", "RunStage", Some("Spawned"), "")]);
    for parts in [
        running("lc.1", "run-new-word"),
        vec![contract("lc.1", "contract-parked", None, &[run_id("lc.1")])],
    ] {
        let b = board_with_cases(&ledger(), &log, &doc(parts), NOW);
        assert!(card(&b, "lc.1").is_none(), "まだ分からない語の札は作らない");
        assert_eq!(b.unmapped, 1);
    }
    let parked = doc(vec![contract("lc.4", "contract-parked", Some("x"), &[])]);
    let b = board_with_cases(&ledger(), "", &parked, NOW);
    assert_eq!(b.board.cards, Reading::Unknown, "event log が読めない");
    let b = board_with_cases(&ledger(), &log, &parked, NOW);
    assert_eq!(
        row(&b, "lc.4"),
        (Stage::Queued, None, None),
        "走行の無い札は台帳の割り"
    );
}

#[test]
fn bvlc_waiting_contract_with_runs() {
    let log = events(&[("lc.3", "RunStage", Some("Gated"), "verdict:FAIL")]);
    let runs = [run_id("lc.3")];
    let wait = |phase: &str, reason: &str| {
        let parts = vec![
            contract("lc.3", phase, Some(reason), &runs),
            run_part("lc.3", "run-gate-failed", "verdict:FAIL kind:x"),
        ];
        row(&board_with_cases(&ledger(), &log, &doc(parts), NOW), "lc.3")
    };
    assert_eq!(
        wait("contract-queued", "dependency"),
        (Stage::Blocked, Some("dependency".into()), Some(WAIT_SINCE))
    );
    assert_eq!(
        wait("contract-queued", "host-busy"),
        (Stage::Queued, Some("host-busy".into()), Some(WAIT_SINCE))
    );
    assert_eq!(
        wait("contract-refused", "no-room"),
        (Stage::Queued, Some("no-room".into()), Some(WAIT_SINCE))
    );
    assert_eq!(
        wait("contract-queued", "settled"),
        (Stage::Failed, Some("verdict:FAIL".into()), Some(AT)),
        "settled は便の部品の局面"
    );
}

#[test]
fn bvlc_runless_from_parts() {
    let log = events(&[("lc.1", "RunStage", Some("Spawned"), "")]);
    let refused = doc(vec![contract(
        "lc.4",
        "contract-refused",
        Some("admission"),
        &[],
    )]);
    assert_eq!(
        row(&board_with_cases(&ledger(), &log, &refused, NOW), "lc.4"),
        (Stage::Queued, Some("admission".into()), Some(WAIT_SINCE))
    );
    let mut parts = running("lc.4", "run-implementing");
    parts[1]["reason"] = json!("Spawned");
    let b = board_with_cases(&ledger(), &log, &doc(parts), NOW);
    assert_eq!(
        row(&b, "lc.4"),
        (Stage::Running, Some("Spawned".into()), Some(RUN_SINCE))
    );
    assert_eq!(card(&b, "lc.4").map(|c| c.runs), Some(0));
}

#[test]
fn bvlc_fallback_event_stage() {
    let log = events(&[
        ("lc.1", "RunStage", Some("Spawned"), ""),
        ("lc.3", "RunStage", Some("Gated"), "verdict:FAIL"),
    ]);
    let plain = board(&ledger(), &log, NOW);
    assert_eq!(
        board_with_cases(&ledger(), &log, &cases_of("", None), NOW),
        plain
    );
    for parts in [
        vec![],
        vec![contract("lc.1", "contract-closed", None, &[run_id("lc.1")])],
        vec![contract(
            "lc.1",
            "contract-running",
            Some("run-gating"),
            &[run_id("lc.1")],
        )],
        vec![
            contract(
                "lc.1",
                "contract-running",
                Some("run-gating"),
                &["lc.1-x".into()],
            ),
            run_part("lc.1", "run-failed", "x"),
        ],
    ] {
        let b = board_with_cases(&ledger(), &log, &doc(parts), NOW);
        assert_eq!(b, plain);
    }
    assert_eq!(row(&plain, "lc.1"), (Stage::Running, None, Some(AT)));
}

#[test]
fn bvlc_closed_ledger_and_ci_stay() {
    let log = events(&[
        ("lc.5", "RunStage", Some("Spawned"), ""),
        ("lc.6", "RunDone", Some("Landed"), "terminal:ci:success"),
    ]);
    let mut parts = running("lc.5", "run-implementing");
    parts.extend(running("lc.6", "run-landed-open"));
    let b = board_with_cases(&ledger(), &log, &doc(parts), NOW);
    let (stage, reason, _) = row(&b, "lc.5");
    assert_eq!(stage, Stage::Landed);
    assert!(
        reason.is_some_and(|r| r.starts_with(CLOSED_TAG)),
        "台帳の閉じ"
    );
    let landed = card(&b, "lc.6").expect("lc.6 の札");
    assert_eq!(
        (landed.stage, landed.ci),
        (Stage::Landed, Some(Ci::Success))
    );
    let gating = board_with_cases(&ledger(), &log, &doc(running("lc.6", "run-gating")), NOW);
    let c = card(&gating, "lc.6").expect("lc.6 の札");
    assert_eq!(
        (c.stage, c.ci),
        (Stage::Running, None),
        "Landed でない段に CI を付けない"
    );
    let mut shut: Vec<Value> = serde_json::from_str(&ledger()).expect("台帳の字");
    for at in [3, 5] {
        shut[at]["status"] = json!("closed");
    }
    let shut = Value::Array(shut).to_string();
    let log = events(&[
        ("lc.4", "RunDone", Some("Landed"), "terminal:push:ok"),
        ("lc.6", "RunDone", Some("Landed"), "terminal:ci:success"),
    ]);
    let mut parts = running("lc.6", "run-landed-open");
    parts.extend(running("lc.4", "run-ci-waiting"));
    let b = board_with_cases(&shut, &log, &doc(parts), NOW);
    let c = card(&b, "lc.6").expect("閉じた lc.6 の札");
    assert_eq!(
        (c.stage, c.reason.as_deref(), c.ci),
        (Stage::Landed, None, Some(Ci::Success)),
        "Landed の段の閉じた札の理由は event log の読みのまま"
    );
    let c = card(&b, "lc.4").expect("閉じた lc.4 の札");
    assert_eq!(
        (c.stage, c.reason.as_deref(), c.ci),
        (Stage::Landed, None, None),
        "閉じた札の CI の読み Waiting は ci None"
    );
}
