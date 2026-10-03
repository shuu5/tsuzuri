//! 契約の型の外形の歯（接頭辞 contract_form_）のうち surface の群の型の見本。
//! snapshot は tests/snapshots の群の名の json（surface.json）で、字の比べは標準 library だけで行う。
//! 共通の手は tests/teeth1/common/mod.rs に 1 つだけ在る。
//! 字が違えば今の字を CARGO_TARGET_TMPDIR の群の名の json に書いて落ちる（見て正しければ snapshot へ写す）。
#![cfg(test)]

use crate::common;

use common::{AT, Form, bead, form, ruling};
use tsuzuri_contract::board::{NextMove, Stage};
use tsuzuri_contract::surface::{
    BatchItem, BatchItemResult, BatchRequest, BatchResponse, BoardChanged, ChangeKind,
    ItemOutcome, PolicyRequest, PolicyResponse, QuestionNudge, Refusal, RefusalResponse, RulingId, RulingRequest,
    RulingResponse, SeatHealth, SeatRole, SeatView, SurfaceEvent, SurfaceState,
};

fn seat_view() -> SeatView {
    SeatView {
        seat: "t3:orchestrator".into(),
        role: SeatRole::Orchestrator,
        health: SeatHealth::Waiting,
        account: Some("acct-4".into()),
        move_left_s: Some(1200),
    }
}

/// surface の群の型の見本（この順が snapshot の file の key の順）。
fn forms() -> Vec<Box<dyn Form>> {
    let refusals = vec![
        Refusal::EmptyVerbatim,
        Refusal::UnknownQuestion,
        Refusal::StaleVersion,
        Refusal::A1NeedsOwnVerbatim,
        Refusal::A1InBatch,
        Refusal::EmptyBatch,
    ];
    vec![
        // surface
        form(
            "surface::RulingId",
            vec![
                RulingId::for_question(&bead("fx-c.5"),"20260926T1437Z", 1).unwrap(),
                RulingId::for_batch("20260926T1437Z", 1).unwrap(),
            ],
        ),
        form(
            "surface::SeatRole",
            vec![SeatRole::Orchestrator, SeatRole::Pipeline],
        ),
        form(
            "surface::SeatHealth",
            vec![
                SeatHealth::Waiting,
                SeatHealth::Working,
                SeatHealth::Unresponsive,
                SeatHealth::Stopped,
                SeatHealth::Unknown,
            ],
        ),
        form(
            "surface::SeatView",
            vec![
                seat_view(),
                SeatView {
                    seat: "t3:pipeline".into(),
                    role: SeatRole::Pipeline,
                    health: SeatHealth::Unknown,
                    account: None,
                    move_left_s: None,
                },
            ],
        ),
        form(
            "surface::SurfaceState",
            vec![SurfaceState {
                seats: vec![seat_view()],
                next_move: NextMove::Question,
                updated_at: AT,
                undelivered: vec![ruling("fx-c.5:20260926T1437Z-1")],
            }],
        ),
    ]
    .into_iter()
    .chain(event_forms())
    .chain(batch_forms())
    .chain(policy_forms(refusals))
    .collect()
}

/// surface の群の型の見本の続き（事象と問いの督促と裁定の要求と応答の型）。
fn event_forms() -> Vec<Box<dyn Form>> {
    vec![
        form(
            "surface::SurfaceEvent",
            vec![
                SurfaceEvent::SeatState {
                    seat: seat_view(),
                    at: AT,
                },
                SurfaceEvent::QuestionPosted {
                    question: bead("t3-hub.7"),
                    at: AT,
                },
                SurfaceEvent::RulingRead {
                    ruling: ruling("fx-c.7:20260926T1437Z-1"),
                    seat: "t3:orchestrator".into(),
                    at: AT,
                },
                SurfaceEvent::RunStage {
                    run: "t3-hub.3-20260927T073916Z".into(),
                    contract: bead("t3-hub.3"),
                    stage: Stage::Gated,
                    reason: None,
                    at: AT,
                },
            ],
        ),
        form(
            "surface::QuestionNudge",
            vec![QuestionNudge {
                question: bead("t3-hub.7"),
            }],
        ),
        form(
            "surface::RulingRequest",
            vec![RulingRequest {
                question: bead("t3-hub.7"),
                seen_digest: "3fa2b91c".into(),
                verbatim: "よい".into(),
            }],
        ),
        form(
            "surface::RulingResponse",
            vec![RulingResponse {
                ruling: ruling("fx-c.7:20260926T1437Z-1"),
                recorded_at: AT,
            }],
        ),
    ]
}

/// surface の群の型の見本の続き（まとめての裁定の型）。
fn batch_forms() -> Vec<Box<dyn Form>> {
    vec![
        form(
            "surface::BatchRequest",
            vec![BatchRequest {
                items: vec![
                    BatchItem {
                        question: bead("t3-hub.7"),
                        seen_digest: "3fa2b91c".into(),
                        verbatim: None,
                    },
                    BatchItem {
                        question: bead("t3-hub.8"),
                        seen_digest: "0b1c2d3e".into(),
                        verbatim: Some("推奨で".into()),
                    },
                ],
                verbatim: "全部よい".into(),
            }],
        ),
        form(
            "surface::ItemOutcome",
            vec![
                ItemOutcome::Written {
                    ruling: ruling("fx-c.7:20260926T1437Z-1"),
                },
                ItemOutcome::Skipped {
                    ruling: ruling("fx-c.8:20260926T1437Z-1"),
                },
                ItemOutcome::Refused {
                    reason: Refusal::StaleVersion,
                },
            ],
        ),
        form(
            "surface::BatchResponse",
            vec![BatchResponse {
                batch: ruling("batch:20260926T1437Z-1"),
                items: vec![
                    BatchItemResult {
                        question: bead("t3-hub.7"),
                        outcome: ItemOutcome::Written {
                            ruling: ruling("fx-c.7:20260926T1437Z-1"),
                        },
                    },
                    BatchItemResult {
                        question: bead("t3-hub.9"),
                        outcome: ItemOutcome::Refused {
                            reason: Refusal::A1InBatch,
                        },
                    },
                ],
            }],
        ),
    ]
}

/// surface の群の型の見本の後半（方針と断りと板の変化の型・断りの列は `refusals`）。
fn policy_forms(refusals: Vec<Refusal>) -> Vec<Box<dyn Form>> {
    vec![
        form(
            "surface::PolicyRequest",
            vec![PolicyRequest {
                scope: "t3-hub".into(),
                verbatim: "画面を早く出す".into(),
            }],
        ),
        form(
            "surface::PolicyResponse",
            vec![PolicyResponse {
                policy: RulingId::for_question(&bead("fx-p.1"), "20260926T1437Z", 1).unwrap(),
                recorded_at: AT,
            }],
        ),
        form("surface::Refusal", refusals.clone()),
        form(
            "surface::RefusalResponse",
            vec![RefusalResponse {
                reason: Refusal::StaleVersion,
            }],
        ),
        form("surface::ChangeKind", ChangeKind::ALL.to_vec()),
        form(
            "surface::BoardChanged",
            vec![
                BoardChanged {
                    at: AT,
                    kinds: vec![ChangeKind::Seat, ChangeKind::Account],
                },
                BoardChanged {
                    at: AT,
                    kinds: vec![],
                },
            ],
        ),
    ]
}

#[test]
fn contract_form_surface_snapshot_matches() {
    common::snapshot_matches("surface", &forms());
}

#[test]
fn contract_form_surface_roundtrip_all_types() {
    common::roundtrip_all(&forms());
}
