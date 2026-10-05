//! 次の一手の束の承認の歯（行 c-next-batch・接頭辞 nbatch_）。
//! 束の承認は、問いの一覧（`open_questions`）の A-1 の印の無い問いが `BATCH_MIN` 本以上なら当たる。
//! 台帳の字は bd の一覧の形の配列を json の macro で組む。fixture: tests/fixtures/pipeline/next.json。
#![cfg(test)]

use std::path::Path;

use crate::common::root;
use serde_json::{Value, json};
use tsuzuri_contract::board::NextMove;
use tsuzuri_contract::stats::{CheckResult, NextCheck, NextStep};
use tsuzuri_core::next_step::{BATCH_MIN, next_step, next_step_seat};

/// 2026-09-27T02:00:00Z。
const NOW: u64 = 1_790_474_400;

fn ask(id: &str, status: &str, labels: &[&str]) -> Value {
    json!({
        "id": id,
        "title": format!("問い {id}"),
        "status": status,
        "issue_type": "task",
        "labels": labels,
        "created_at": "2026-09-26T10:00:00Z",
    })
}

fn task(id: &str) -> Value {
    json!({
        "id": id,
        "title": format!("契約 {id}"),
        "status": "open",
        "issue_type": "task",
        "labels": [],
        "created_at": "2026-09-26T00:00:00Z",
    })
}

fn plain(id: &str) -> Value {
    ask(id, "open", &["intake:question"])
}

fn text(beads: &[Value]) -> String {
    Value::Array(beads.to_vec()).to_string()
}

fn of(step: &NextStep, kind: NextMove) -> &NextCheck {
    step.checks
        .iter()
        .find(|c| c.kind == kind)
        .unwrap_or_else(|| panic!("{kind:?} の check が無い"))
}

/// 並び・lead・発効待ち・席の card の無い next_step_seat を見て、値を返す。
fn judged(ledger: &str, events: &str) -> NextStep {
    let step = next_step(ledger, events, NOW);
    let kinds: Vec<NextMove> = step.checks.iter().map(|c| c.kind).collect();
    assert_eq!(kinds, NextMove::ALL.to_vec(), "並びは NextMove の ALL");
    let first = step
        .checks
        .iter()
        .find(|c| c.result == CheckResult::Hit)
        .map_or(NextMove::Nothing, |c| c.kind);
    assert_eq!(
        step.lead, first,
        "lead は並びで最初に Hit の種類（どれも Hit でなければなし）"
    );
    assert_eq!(
        of(&step, NextMove::AwaitingEffect).result,
        CheckResult::NotJudged
    );
    assert_eq!(next_step_seat(ledger, events, NOW, None), step);
    step
}

fn batch_is(step: &NextStep, result: CheckResult, count: u32) {
    let c = of(step, NextMove::BatchApproval);
    assert_eq!((c.result, c.count, &c.target), (result, count, &None), "{c:?}");
}

#[test]
fn nbatch_two_plain_asks_hit() {
    let ledger = text(&[plain("nb.q1"), plain("nb.q2"), task("nb.1")]);
    let step = judged(&ledger, "");
    batch_is(&step, CheckResult::Hit, 2);
    let q = of(&step, NextMove::Question);
    assert_eq!((q.result, q.count), (CheckResult::Hit, 2));
    assert_eq!(step.lead, NextMove::BatchApproval);
}

#[test]
fn nbatch_a1_left_out() {
    let ledger = text(&[
        plain("nb.q1"),
        ask("nb.q2", "open", &["intake:question", "A-1"]),
        ask("nb.q3", "open", &["A-1:消す", "intake:question"]),
    ]);
    let step = judged(&ledger, "");
    batch_is(&step, CheckResult::Miss, 0);
    let q = of(&step, NextMove::Question);
    assert_eq!((q.result, q.count), (CheckResult::Hit, 3));
    assert_eq!(step.lead, NextMove::Question);
}

#[test]
fn nbatch_closed_and_plain_tasks_left_out() {
    let ledger = text(&[
        plain("nb.q1"),
        plain("nb.q2"),
        ask("nb.q3", "closed", &["intake:question"]),
        ask("nb.q4", "tombstone", &["intake:question"]),
        task("nb.1"),
    ]);
    let step = judged(&ledger, "");
    batch_is(&step, CheckResult::Hit, 2);
    assert_eq!(step.lead, NextMove::BatchApproval);
}

#[test]
fn nbatch_unread_is_not_judged() {
    batch_is(&judged("", ""), CheckResult::NotJudged, 0);
    batch_is(&judged("[]", ""), CheckResult::Miss, 0);
    // 1 本は BATCH_MIN に届かない。
    batch_is(&judged(&text(&[plain("nb.q1")]), ""), CheckResult::Miss, 0);
}

#[test]
fn nbatch_stalled_leads_first() {
    let ledger = text(&[task("nb.1"), plain("nb.q1"), plain("nb.q2")]);
    let events = [
        json!({"schema": 1, "ts": "2026-09-27T00:00:00Z", "kind": "RunCreated", "run": "nb.1-20260927T000000Z", "bead": "nb.1", "stage": "Intake"}),
        json!({"schema": 1, "ts": "2026-09-27T00:10:00Z", "kind": "RunStage", "run": "nb.1-20260927T000000Z", "bead": "nb.1", "stage": "Gated", "detail": "verdict:FAIL"}),
    ]
    .iter()
    .map(Value::to_string)
    .collect::<Vec<_>>()
    .join("\n");
    let step = judged(&ledger, &events);
    assert_eq!(of(&step, NextMove::StalledRun).result, CheckResult::Hit);
    batch_is(&step, CheckResult::Hit, 2);
    assert_eq!(step.lead, NextMove::StalledRun);
}

#[test]
fn nbatch_order_keeps_all() {
    assert_eq!(BATCH_MIN, 2);
    for ledger in [
        String::new(),
        "[]".to_owned(),
        text(&[plain("nb.q1"), plain("nb.q2")]),
    ] {
        judged(&ledger, "");
    }
}

#[test]
fn nbatch_fixture_cases_miss() {
    let rel = "tests/fixtures/pipeline/next.json";
    let raw = std::fs::read_to_string(root().join(rel)).expect("next.json を読む");
    let v: Value = serde_json::from_str(&raw).expect("next.json は JSON");
    let now = v["now"].as_u64().expect("now");
    let cases = v["cases"].as_array().expect("cases");
    assert_eq!(cases.len(), 4);
    let q1 = cases[1]["ledger"]
        .as_array()
        .expect("ledger")
        .iter()
        .find(|b| b["id"] == "nx2.q1")
        .expect("nx2.q1");
    assert!(
        q1["labels"].as_array().expect("labels").contains(&json!("A-1")),
        "nx2.q1 は A-1 の印を持つ"
    );
    for c in cases {
        let ledger = c["ledger"].to_string();
        let events = match &c["events"] {
            Value::String(s) => s.clone(),
            Value::Array(items) => items
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
            other => panic!("events の形: {other}"),
        };
        let step = next_step(&ledger, &events, now);
        batch_is(&step, CheckResult::Miss, 0);
        let want: NextStep = serde_json::from_value(c["want"].clone()).expect("want");
        batch_is(&want, CheckResult::Miss, 0);
    }
}

/// 着地済みの行とこの波の行の verify の語（接頭辞 nbatch_ は並べない）。
const FILTER_WORDS: [&str; 65] = [
    "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
    "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
    "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
    "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_",
    "server_", "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_", "gnav_",
    "mkeys_", "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_", "ticker_", "hfig_",
    "pmore_", "lspark_", "apop_", "brand_", "runsdoc_",
];

#[test]
fn nbatch_own_names_clean() {
    let src = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth2/nbatch.rs"))
        .expect("tests/teeth2/nbatch.rs を読む");
    let lines: Vec<&str> = src.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("#[test] の次の行は fn");
            rest.split('(').next().unwrap_or_default()
        })
        .collect();
    assert_eq!(names.len(), 8, "{names:?}");
    for name in names {
        let tail = name
            .strip_prefix("nbatch_")
            .unwrap_or_else(|| panic!("{name} は nbatch_ で始まる"));
        for w in FILTER_WORDS {
            assert!(!tail.contains(w), "{name} は語 {w} を含む");
        }
    }
}
