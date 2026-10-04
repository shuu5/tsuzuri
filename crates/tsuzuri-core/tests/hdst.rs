//! 段 Held（留め置き）の歯（中核・接頭辞 hdst_・設計ノート surface-wave28a 行 c-held-stage・判断の記録 ADR-42 決定 (1)(2)(6)）。
//! 局面の出力の契約の部品が contract-queued で理由 hold（席の止め）か contract-refused（受付の断り）の札は段 Held で列は Blocked。
//! 理由と since は部品の字。依存・重なり・予約は Blocked のまま、自動で解ける待ちは Queued のまま。出力が読めない間は Held を判じない。
//! 部品の任意の欄 why は欠けても部品を落とさない。否定の見本は正しい見本から 1 欄だけ替える。節の組は歯の中で組む。
#![cfg(test)]

use serde_json::{Value, json};
use tsuzuri_contract::board::{PipelineColumn, Reading, Stage};
use tsuzuri_contract::case::CasePart;
use tsuzuri_core::case::cases_of;
use tsuzuri_core::pipeline::{
    HOLD, PARTNER_REASONS, QUEUED_PHASE, REFUSED_PHASE, board_with_cases, queued_of,
};

/// 節の今（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

/// 部品の since の字（2026-09-27T11:30:00Z）と epoch 秒。
const SINCE: &str = "2026-09-27T11:30:00Z";
const SINCE_SECS: u64 = NOW - 1800;

/// 受付の断りの名と、止めの理由の字。
const NAME: &str = "teeth-outside-write-set";
const WHY: &str = "書く file の重なりを席が確かめる";

/// 節の台帳（配れる open の task hd.1 と、hd.1 が blocks で待つ開いた hd.2）。
fn ledger(blocked: bool) -> String {
    let task = |id: &str| {
        json!({"id": id, "title": format!("t {id}"), "status": "open", "issue_type": "task",
            "acceptance_criteria": format!("design = contracts/x.toml#{id}")})
    };
    let mut one = task("hd.1");
    if blocked {
        one["dependencies"] =
            json!([{"issue_id": "hd.1", "depends_on_id": "hd.2", "type": "blocks"}]);
    }
    json!([one, task("hd.2")]).to_string()
}

/// hd.1 の走行 1 つの event log（走行の無い札は段を決めない event 1 行だけの読める log）。
fn events(run: bool) -> String {
    let id = "hd.1-20260927T110000Z";
    let mut rows = vec![json!({"schema": 1, "ts": "2026-09-27T10:00:00Z", "kind": "SeatSpawned"})];
    if run {
        rows.push(json!({"schema": 1, "ts": "2026-09-27T11:00:00Z", "kind": "RunCreated", "run": id, "bead": "hd.1", "stage": "Intake"}));
        rows.push(json!({"schema": 1, "ts": "2026-09-27T11:10:00Z", "kind": "RunStage", "run": id, "bead": "hd.1", "stage": "Spawned"}));
    }
    rows.iter()
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

/// hd.1 の契約の部品（局面・理由・why）。
fn part(phase: &str, reason: &str, why: Option<&str>) -> Value {
    json!({"part": "contract", "id": "hd.1", "phase": phase, "turn": "seat", "since": SINCE,
        "reason": reason, "why": why, "closed": false, "overdue": null, "links": {"on": []}})
}

/// 局面の出力の字（版を選ぶ）。
fn output(version: u64, parts: Vec<Value>) -> String {
    json!({"version": version, "generated_at": "2026-09-27T11:59:00Z", "parts": parts}).to_string()
}

/// 部品 1 つの出力で組んだ hd.1 の札の段と理由と since（`run` は走行の在る札・`blocked` は開いた blocker を持つ台帳）。
fn card(text: &str, run: bool, blocked: bool) -> (Stage, Option<String>, Option<u64>) {
    let b = board_with_cases(&ledger(blocked), &events(run), &cases_of(text, None), NOW);
    let Reading::Known(cards) = b.board.cards else {
        panic!("札が Unknown");
    };
    let c = cards
        .into_iter()
        .find(|c| c.contract.as_str() == "hd.1")
        .unwrap_or_else(|| panic!("hd.1 の札が無い"));
    (c.stage, c.reason, c.since)
}

fn one(p: Value, run: bool) -> (Stage, Option<String>, Option<u64>) {
    card(&output(1, vec![p]), run, false)
}

#[test]
fn hdst_held_in_closed_stages() {
    assert_eq!(Stage::ALL.len(), 9);
    assert!(Stage::ALL.contains(&Stage::Held));
    assert_eq!(Stage::Held.column(), PipelineColumn::Blocked);
    assert_eq!(serde_json::to_string(&Stage::Held).unwrap(), "\"Held\"");
    assert_eq!(
        serde_json::from_str::<Stage>("\"Held\"").unwrap(),
        Stage::Held
    );
    assert_eq!(HOLD, "hold");
    assert_eq!(queued_of(Some(HOLD)), Stage::Held);
}

#[test]
fn hdst_seat_hold_is_held() {
    for run in [false, true] {
        let want = (Stage::Held, Some(HOLD.to_string()), Some(SINCE_SECS));
        assert_eq!(
            one(part(QUEUED_PHASE, HOLD, Some(WHY)), run),
            want,
            "run={run}"
        );
        let other = one(part(QUEUED_PHASE, "host-busy", Some(WHY)), run);
        assert_eq!(other.0, Stage::Queued, "理由だけ替えた見本 run={run}");
    }
}

#[test]
fn hdst_intake_refusal_is_held() {
    for run in [false, true] {
        let want = (Stage::Held, Some(NAME.to_string()), Some(SINCE_SECS));
        assert_eq!(
            one(part(REFUSED_PHASE, NAME, Some(WHY)), run),
            want,
            "run={run}"
        );
        let other = one(part(QUEUED_PHASE, NAME, Some(WHY)), run);
        assert_eq!(other.0, Stage::Queued, "局面だけ替えた見本 run={run}");
    }
}

#[test]
fn hdst_partner_reasons_stay_blocked() {
    assert_eq!(PARTNER_REASONS, ["dependency", "overlap", "reserved"]);
    for word in PARTNER_REASONS {
        for run in [false, true] {
            let got = one(part(QUEUED_PHASE, word, None), run);
            assert_eq!(got.0, Stage::Blocked, "{word} run={run}");
        }
    }
}

#[test]
fn hdst_self_clearing_reasons_stay_queued() {
    let words = [
        "host-busy",
        "launched",
        "floor",
        "unreflected-ruling",
        "sibling",
    ];
    for word in words {
        for run in [false, true] {
            let got = one(part(QUEUED_PHASE, word, None), run);
            assert_eq!(got.0, Stage::Queued, "{word} run={run}");
        }
    }
}

#[test]
fn hdst_unreadable_cases_no_held() {
    for p in [
        part(QUEUED_PHASE, HOLD, Some(WHY)),
        part(REFUSED_PHASE, NAME, Some(WHY)),
    ] {
        let held = card(&output(1, vec![p.clone()]), false, false);
        assert_eq!(held.0, Stage::Held, "読める出力");
        let text = output(2, vec![p]);
        assert_eq!(cases_of(&text, None).parts, Reading::Unknown);
        assert_eq!(card(&text, false, false), (Stage::Queued, None, None));
        assert_eq!(card(&text, false, true), (Stage::Blocked, None, None));
    }
}

#[test]
fn hdst_why_optional_field() {
    let parts = |p: Value| match cases_of(&output(1, vec![p]), None).parts {
        Reading::Known(parts) => parts,
        Reading::Unknown => panic!("部品が Unknown"),
    };
    let with = parts(part(QUEUED_PHASE, HOLD, Some(WHY)));
    assert_eq!(with.len(), 1);
    assert_eq!(with[0].why.as_deref(), Some(WHY));
    let null = parts(part(QUEUED_PHASE, HOLD, None));
    assert_eq!(null.len(), 1, "null の why");
    assert_eq!(null[0].why, None);
    let mut gone = part(QUEUED_PHASE, HOLD, Some(WHY));
    gone.as_object_mut().unwrap().remove("why");
    let gone = parts(gone);
    assert_eq!(gone.len(), 1, "鍵の無い why");
    assert_eq!(gone[0].why, None);
    let mut typed = part(QUEUED_PHASE, HOLD, Some(WHY));
    typed["why"] = json!(7);
    assert_eq!(parts(typed).len(), 0, "型の違う why の部品は落とす");
    let mut wire = serde_json::to_value(&with[0]).unwrap();
    assert_eq!(wire["why"], json!(WHY));
    wire.as_object_mut().unwrap().remove("why");
    let back: CasePart = serde_json::from_value(wire).unwrap();
    assert_eq!(back.why, None, "鍵の無い電文");
    let none = serde_json::to_value(&back).unwrap();
    assert!(none.get("why").is_none(), "None は鍵を書かない: {none}");
}
