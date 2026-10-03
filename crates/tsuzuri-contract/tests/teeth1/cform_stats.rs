//! 契約の型の外形の歯（接頭辞 contract_form_）のうち stats の群の型の見本。
//! snapshot は tests/snapshots の群の名の json（stats.json）で、字の比べは標準 library だけで行う。
//! 共通の手は tests/teeth1/common/mod.rs に 1 つだけ在る。
//! 字が違えば今の字を CARGO_TARGET_TMPDIR の群の名の json に書いて落ちる（見て正しければ snapshot へ写す）。
#![cfg(test)]

use crate::common;

use common::{AT, Form, bead, distinct, form};
use tsuzuri_contract::board::{LedgerJudge, NextMove, Reading};
use tsuzuri_contract::stats::{
    CheckResult, DayCount, EpicProgress, LeadDays, LedgerStats, MemoStats, NextCheck, NextStep,
    OpenCounts, UnreflectedCount, UnreflectedKind, UnreflectedList, UnreflectedRow,
};

/// 作った時刻の在る memo と、作った時刻の読めない memo。
fn unreflected_rows() -> Vec<UnreflectedRow> {
    vec![
        UnreflectedRow {
            id: "t3-hub.9".into(),
            title: "[memo] 控え".into(),
            created: Some(AT - 604_800),
        },
        UnreflectedRow {
            id: "t3-hub.10".into(),
            title: "[memo] 時刻の無い控え".into(),
            created: None,
        },
    ]
}

/// stats の群の型の見本（この順が snapshot の file の key の順）。
fn forms() -> Vec<Box<dyn Form>> {
    vec![
        // stats
        form(
            "stats::LedgerStats",
            vec![
                ledger_stats(),
                // 閉じた task も memo も無い台帳。
                LedgerStats {
                    at: AT,
                    judge: LedgerJudge::Stalled,
                    open: OpenCounts {
                        task: 0,
                        memo: 0,
                        question: 0,
                        epic: 0,
                    },
                    blocked: 0,
                    ready: 0,
                    stale: 0,
                    net_drop_24h: 0,
                    net_drop_7d: 0,
                    closed_7d: 0,
                    closed_per_day: 0.0,
                    lead: None,
                    days: vec![],
                    epics: vec![],
                    memo: MemoStats {
                        open: 0,
                        awaiting_promotion: 0,
                        closed_7d: 0,
                        created_p50: None,
                    },
                    unreflected: 0,
                    unreflected_kinds: vec![],
                    unreflected_unknown: UnreflectedKind::ALL.to_vec(),
                },
            ],
        ),
    ]
    .into_iter()
    .chain(step_forms())
    .collect()
}

/// stats の群の型の見本の後半（次の一手と未反映の一覧の型）。
fn step_forms() -> Vec<Box<dyn Form>> {
    vec![
        form(
            "stats::NextStep",
            vec![NextStep {
                checks: NextMove::ALL
                    .iter()
                    .map(|&kind| match kind {
                        NextMove::StalledRun => NextCheck {
                            kind,
                            result: CheckResult::Hit,
                            count: 2,
                            target: Some(bead("t3-hub.3")),
                        },
                        NextMove::Question => NextCheck {
                            kind,
                            result: CheckResult::Hit,
                            count: 1,
                            target: Some(bead("t3-hub.7")),
                        },
                        NextMove::Nothing => NextCheck {
                            kind,
                            result: CheckResult::Miss,
                            count: 0,
                            target: None,
                        },
                        _ => NextCheck {
                            kind,
                            result: CheckResult::NotJudged,
                            count: 0,
                            target: None,
                        },
                    })
                    .collect(),
                lead: NextMove::StalledRun,
            }],
        ),
        form("stats::UnreflectedRow", unreflected_rows()),
        form("stats::UnreflectedList", unreflected_lists()),
    ]
}

fn unreflected_lists() -> Vec<UnreflectedList> {
    vec![
        // 古さの印の在る出力。
        UnreflectedList {
            memos: Reading::Known(unreflected_rows()),
            rulings: Reading::Unknown,
            utterances: Reading::Unknown,
            stale: vec!["ledger-gate".to_string()],
        },
        UnreflectedList {
            memos: Reading::Known(vec![]),
            rulings: Reading::Known(vec![]),
            utterances: Reading::Known(vec![]),
            stale: vec![],
        },
        // 局面の出力が読めない。
        UnreflectedList {
            memos: Reading::Unknown,
            rulings: Reading::Unknown,
            utterances: Reading::Unknown,
            stale: vec![],
        },
    ]
}

/// 閉じた task も memo も在る台帳の統計の見本。
fn ledger_stats() -> LedgerStats {
    LedgerStats {
        at: AT,
        judge: LedgerJudge::PilingUp,
        open: OpenCounts {
            task: 9,
            memo: 3,
            question: 2,
            epic: 2,
        },
        blocked: 3,
        ready: 5,
        stale: 3,
        net_drop_24h: 0,
        net_drop_7d: 2,
        closed_7d: 5,
        closed_per_day: 0.5,
        lead: Some(LeadDays {
            p50: 4.0,
            p90: 11.5,
        }),
        days: vec![
            DayCount {
                end: AT - 86_400,
                created: 1,
                closed: 0,
                open: 9,
            },
            DayCount {
                end: AT,
                created: 1,
                closed: 1,
                open: 9,
            },
        ],
        epics: vec![EpicProgress {
            epic: bead("t3-hub"),
            closed: 6,
            total: 9,
        }],
        memo: MemoStats {
            open: 3,
            awaiting_promotion: 2,
            closed_7d: 1,
            created_p50: Some(AT - 820_800),
        },
        unreflected: 3,
        unreflected_kinds: vec![UnreflectedCount {
            kind: UnreflectedKind::Memo,
            count: 3,
        }],
        unreflected_unknown: vec![UnreflectedKind::Ruling, UnreflectedKind::Utterance],
    }
}

#[test]
fn contract_form_stats_snapshot_matches() {
    common::snapshot_matches("stats", &forms());
}

#[test]
fn contract_form_stats_roundtrip_all_types() {
    common::roundtrip_all(&forms());
}

#[test]
fn contract_form_stats_closed_lists() {
    assert_eq!(distinct(&CheckResult::ALL), 3);
    assert_eq!(distinct(&UnreflectedKind::ALL), 3);
}
