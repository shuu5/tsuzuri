//! 契約の型の外形の歯（接頭辞 contract_form_）のうち board の群の型の見本。
//! snapshot は tests/snapshots の群の名の json（board.json）で、字の比べは標準 library だけで行う。
//! 共通の手は tests/teeth1/common/mod.rs に 1 つだけ在る。
//! 字が違えば今の字を CARGO_TARGET_TMPDIR の群の名の json に書いて落ちる（見て正しければ snapshot へ写す）。
#![cfg(test)]

use crate::common;

use std::collections::BTreeSet;

use common::{AT, Form, bead, distinct, form};
use tsuzuri_contract::board::{
    AccountBoard, Ci, GroupRow, LedgerJudge, NextMove, PipelineBoard, PipelineCard, PipelineColumn,
    ProjectMetrics, QuotaLeft, Reading, SessionRow, Stage,
};
use tsuzuri_contract::surface::{SeatHealth, SeatRole};

fn pipeline_card() -> PipelineCard {
    PipelineCard {
        contract: bead("t3-hub.3"),
        runs: 2,
        stage: Stage::Running,
        reason: Some("verify".into()),
        account: Some("acct-4".into()),
        since: Some(AT - 185),
        ci: None,
    }
}

/// board の群の型の見本（この順が snapshot の file の key の順）。
fn forms() -> Vec<Box<dyn Form>> {
    vec![
        // board
        form("board::Stage", Stage::ALL.to_vec()),
        form(
            "board::PipelineColumn",
            vec![
                PipelineColumn::Blocked,
                PipelineColumn::Queued,
                PipelineColumn::RunningGated,
                PipelineColumn::QuestionedFailedStopped,
                PipelineColumn::Landed,
            ],
        ),
        form("board::Ci", Ci::ALL.to_vec()),
        form(
            "board::PipelineCard",
            vec![
                pipeline_card(),
                PipelineCard {
                    contract: bead("t3-hub.4"),
                    runs: 0,
                    stage: Stage::Queued,
                    reason: None,
                    account: None,
                    since: None,
                    ci: None,
                },
                PipelineCard {
                    contract: bead("px.7"),
                    runs: 1,
                    stage: Stage::Failed,
                    reason: Some("terminal:ci:failure".into()),
                    account: Some("acct-4".into()),
                    since: Some(AT - 420),
                    ci: Some(Ci::Failure),
                },
            ],
        ),
        form(
            "board::PipelineBoard",
            vec![
                PipelineBoard {
                    cards: Reading::Known(vec![pipeline_card()]),
                    misfits: Reading::Known(vec![]),
                },
                PipelineBoard {
                    cards: Reading::Known(vec![]),
                    misfits: Reading::Known(vec![]),
                },
                PipelineBoard {
                    cards: Reading::Unknown,
                    misfits: Reading::Unknown,
                },
            ],
        ),
        form("board::NextMove", NextMove::ALL.to_vec()),
        form("board::LedgerJudge", LedgerJudge::ALL.to_vec()),
    ]
    .into_iter()
    .chain(account_forms())
    .collect()
}

/// board の群の型の見本の後半（口座の板の型）。
fn account_forms() -> Vec<Box<dyn Form>> {
    vec![
        form(
            "board::AccountBoard",
            vec![AccountBoard {
                at: AT,
                groups: vec![GroupRow {
                    group: "Tier1".into(),
                    account: "acct-4".into(),
                    candidates: vec!["acct-1".into(), "acct-3".into(), "acct-5".into()],
                    next_account: Some("acct-5".into()),
                    remaining: vec![
                        QuotaLeft {
                            window: "5h".into(),
                            left_pct: 62,
                        },
                        QuotaLeft {
                            window: "7d".into(),
                            left_pct: 40,
                        },
                    ],
                    park: false,
                }],
                sessions: vec![SessionRow {
                    project: "tsuzuri".into(),
                    role: SeatRole::Pipeline,
                    session: "t3-hub.3-20260927T073916Z".into(),
                    account: "acct-4".into(),
                    stage: Some(Stage::Running),
                    elapsed_s: Some(185),
                    health: SeatHealth::Working,
                }],
                projects: vec![ProjectMetrics {
                    project: "tsuzuri".into(),
                    judge: LedgerJudge::OnTrack,
                    open_tasks: 9,
                    net_drop_24h: 2,
                    net_drop_7d: -1,
                    closed_per_day: 1.5,
                    unreflected: 3,
                    memos: 4,
                    next_move: NextMove::BatchApproval,
                }],
            }],
        ),
    ]
}

#[test]
fn contract_form_board_snapshot_matches() {
    common::snapshot_matches("board", &forms());
}

#[test]
fn contract_form_board_roundtrip_all_types() {
    common::roundtrip_all(&forms());
}

#[test]
fn contract_form_board_closed_lists() {
    assert_eq!(distinct(&NextMove::ALL), 7);
    assert_eq!(distinct(&LedgerJudge::ALL), 5);
    assert_eq!(distinct(&Stage::ALL), 9);
    let labels: BTreeSet<&str> = NextMove::ALL.iter().map(|m| m.label()).collect();
    assert_eq!(labels.len(), 7, "次の一手の名が重なる");
    let mut sorted = NextMove::ALL.to_vec();
    sorted.sort();
    assert_eq!(
        sorted,
        NextMove::ALL.to_vec(),
        "次の一手の宣言の順が優先の順"
    );
    let columns: BTreeSet<String> = Stage::ALL
        .iter()
        .map(|s| serde_json::to_string(&s.column()).expect("列"))
        .collect();
    assert_eq!(columns.len(), 5);
    let ci: Vec<String> = Ci::ALL
        .iter()
        .map(|c| serde_json::to_string(c).expect("CI の読み"))
        .collect();
    assert_eq!(
        ci,
        [
            "\"waiting\"",
            "\"success\"",
            "\"failure\"",
            "\"unmeasurable\"",
            "\"push-failed\"",
            "\"close-failed\"",
            "\"unreadable\"",
        ]
    );
}
