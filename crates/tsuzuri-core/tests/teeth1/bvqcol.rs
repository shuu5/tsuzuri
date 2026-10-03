//! 行 c-case-columns の歯（中核・接頭辞 bvqcol_）: 走行の無い札の段を、器の局面の出力が読める時は契約の部品で決める
//! （局面 contract-queued の理由が dependency・overlap・reserved なら Blocked・ほかは Queued・since と理由は部品の字）。
//! 読めないか部品が無いか局面が違う札は台帳の blocks の割りのまま。器の RunStage の段 Blocked は段 Blocked の札。
//! 節の組は歯の中で組む（台帳の一覧と局面の出力の字）。
#![cfg(test)]

use serde_json::{Value, json};
use tsuzuri_contract::board::{PipelineCard, PipelineColumn, Reading, Stage};
use tsuzuri_contract::case::CaseDoc;
use tsuzuri_core::case::cases_of;
use tsuzuri_core::pipeline::{
    Board, PARTNER_REASONS, QUEUED_PHASE, board, board_with_cases, queued_of,
};

/// 節の今（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

/// 部品の since の字（2026-09-27T11:30:00Z）と epoch 秒。
const SINCE: &str = "2026-09-27T11:30:00Z";
const SINCE_SECS: u64 = NOW - 1800;

/// 配れる open の task（acceptance に設計 pointer の行）。
fn task(id: &str) -> Value {
    json!({"id": id, "title": format!("t {id}"), "status": "open", "issue_type": "task",
        "acceptance_criteria": format!("design = contracts/x.toml#{id}")})
}

/// 節の台帳（qc.1〜qc.11・qc.6 と qc.8 は開いた qc.9 を blocks で待つ）。
fn ledger() -> String {
    let mut beads: Vec<Value> = (1..=11).map(|n| task(&format!("qc.{n}"))).collect();
    for at in [5, 7] {
        let id = format!("qc.{}", at + 1);
        beads[at]["dependencies"] =
            json!([{"issue_id": id, "depends_on_id": "qc.9", "type": "blocks"}]);
    }
    Value::Array(beads).to_string()
}

/// 契約の部品 1 つ（局面と理由と since を選ぶ）。
fn part(id: &str, phase: &str, reason: Option<&str>, since: Option<&str>) -> Value {
    json!({"part": "contract", "id": id, "phase": phase, "turn": "vessel", "since": since,
        "reason": reason, "closed": false, "overdue": null, "links": {"on": []}})
}

/// 節の局面の出力（qc.8 と qc.11 の部品は無い・qc.9 は局面が違う）。
fn lifecycle() -> String {
    let q = QUEUED_PHASE;
    json!({"version": 1, "generated_at": "2026-09-27T11:59:00Z", "parts": [
        part("qc.1", q, Some("dependency"), Some(SINCE)),
        part("qc.2", q, Some("overlap"), Some(SINCE)),
        part("qc.3", q, Some("reserved"), Some(SINCE)),
        part("qc.4", q, Some("host-busy"), Some(SINCE)),
        part("qc.5", q, Some("hold"), None),
        part("qc.6", q, Some("unreflected-ruling"), Some(SINCE)),
        part("qc.7", q, Some("new-word"), Some(SINCE)),
        part("qc.9", "contract-closed", None, Some(SINCE)),
        part("qc.10", q, None, Some(SINCE)),
    ]})
    .to_string()
}

/// 節の event log（qc.11 の走行 1 つ・器の承認待ちの段 Blocked で終わる）。
fn events() -> String {
    let run = "qc.11-20260927T110000Z";
    [
        json!({"schema": 1, "ts": "2026-09-27T11:00:00Z", "kind": "RunCreated", "run": run, "bead": "qc.11", "stage": "Intake"}),
        json!({"schema": 1, "ts": "2026-09-27T11:10:00Z", "kind": "RunStage", "run": run, "bead": "qc.11", "stage": "Blocked", "detail": "C9"}),
    ]
    .iter()
    .map(Value::to_string)
    .collect::<Vec<_>>()
    .join("\n")
}

fn cased(cases: &CaseDoc) -> Board {
    board_with_cases(&ledger(), &events(), cases, NOW)
}

fn known() -> CaseDoc {
    cases_of(&lifecycle(), None)
}

fn card<'b>(b: &'b Board, id: &str) -> &'b PipelineCard {
    let Reading::Known(cards) = &b.board.cards else {
        panic!("札が Unknown");
    };
    cards
        .iter()
        .find(|c| c.contract.as_str() == id)
        .unwrap_or_else(|| panic!("{id} の札が無い"))
}

/// 札の段と理由と since。
fn row(b: &Board, id: &str) -> (Stage, Option<String>, Option<u64>) {
    let c = card(b, id);
    (c.stage, c.reason.clone(), c.since)
}

#[test]
fn bvqcol_dependency_overlap_reserved_blocked() {
    assert_eq!(PARTNER_REASONS, ["dependency", "overlap", "reserved"]);
    let b = cased(&known());
    for (id, word) in [
        ("qc.1", "dependency"),
        ("qc.2", "overlap"),
        ("qc.3", "reserved"),
    ] {
        assert_eq!(
            row(&b, id),
            (Stage::Blocked, Some(word.to_string()), Some(SINCE_SECS)),
            "{id}"
        );
        assert_eq!(card(&b, id).stage.column(), PipelineColumn::Blocked);
    }
}

#[test]
fn bvqcol_other_reasons_queued() {
    let b = cased(&known());
    for (id, word) in [
        ("qc.4", "host-busy"),
        ("qc.6", "unreflected-ruling"),
        ("qc.7", "new-word"),
    ] {
        assert_eq!(
            row(&b, id),
            (Stage::Queued, Some(word.to_string()), Some(SINCE_SECS)),
            "{id}"
        );
    }
    assert_eq!(row(&b, "qc.10"), (Stage::Queued, None, Some(SINCE_SECS)));
    assert_eq!(queued_of(None), Stage::Queued);
    assert_eq!(queued_of(Some("settled")), Stage::Queued);
    assert_eq!(queued_of(Some("overlap")), Stage::Blocked);
}

#[test]
fn bvqcol_since_and_reason_from_case() {
    let b = cased(&known());
    assert_eq!(QUEUED_PHASE, "contract-queued");
    assert_eq!(card(&b, "qc.1").since, Some(SINCE_SECS));
    assert_eq!(card(&b, "qc.1").reason.as_deref(), Some("dependency"));
    assert_eq!(
        row(&b, "qc.5"),
        (Stage::Queued, Some("hold".to_string()), None),
        "since の無い部品"
    );
    let c = card(&b, "qc.4");
    assert_eq!((c.runs, c.account.as_deref(), c.ci), (0, None, None));
}

#[test]
fn bvqcol_fallback_ledger_blocks_when_unknown() {
    let unknown = cases_of("", None);
    assert_eq!(unknown.parts, Reading::Unknown);
    let fell = cased(&unknown);
    assert_eq!(
        fell,
        board(&ledger(), &events(), NOW),
        "読めない出力は今の板"
    );
    for n in 1..=10 {
        let id = format!("qc.{n}");
        let want = if n == 6 || n == 8 {
            Stage::Blocked
        } else {
            Stage::Queued
        };
        assert_eq!(row(&fell, &id), (want, None, None), "{id}");
    }
    let b = cased(&known());
    assert_eq!(
        row(&b, "qc.8"),
        (Stage::Blocked, None, None),
        "部品の無い札"
    );
    assert_eq!(
        row(&b, "qc.9"),
        (Stage::Queued, None, None),
        "局面の違う部品"
    );
}

#[test]
fn bvqcol_run_blocked_stage() {
    for b in [cased(&known()), board(&ledger(), &events(), NOW)] {
        let c = card(&b, "qc.11");
        assert_eq!(
            (c.runs, c.stage, c.reason.as_deref()),
            (1, Stage::Blocked, Some("C9"))
        );
        assert_eq!(c.stage.column(), PipelineColumn::Blocked);
        assert_eq!(b.unmapped, 0);
    }
}
