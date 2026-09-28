//! 契約の型の外形の歯（接頭辞 contract_form_）。
//! snapshot は型の見本の値を JSON にした群ごとの file（tests/snapshots の群の名の json・群は型の名の module の頭で決まる）で、字の比べは標準 library だけで行う。
//! 字が違えば今の字を CARGO_TARGET_TMPDIR の群の名の json に書いて落ちる（見て正しければ snapshot へ写す）。

use std::collections::BTreeSet;
use std::fmt::Debug;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use tsuzuri_contract::board::{
    AccountBoard, GroupRow, LedgerJudge, NextMove, PipelineBoard, PipelineCard, PipelineColumn,
    ProjectMetrics, QuotaLeft, Reading, SessionRow, Stage,
};
use tsuzuri_contract::graph::{
    AroundDoc, AroundRow, BeadAttr, BoxFold, EdgeEnd, EdgeType, Fold, GraphDoc, GraphEdge, GraphNode,
    GraphSource, GraphView, HubCut, InvariantCheck, NodeKind, RunAttr, SkippedEdges, Verdict,
    ViewEdge, ViewNode, title36,
};
use tsuzuri_contract::ledger::{
    BDW, BdLine, BeadId, ChildType, Effect, LedgerChanged, LedgerItem, LedgerList, LedgerRow, LedgerWrite,
    MEMO_LABEL, NOTES_REPLACE_FLAG, PARENT_FLAG, QUESTION_LABEL, fnv1a64,
};
use tsuzuri_contract::question::{QuestionCard, QuestionList};
use tsuzuri_contract::seat::{AccountMove, QuotaUsed, SeatCard, SeatSpan, SeatState};
use tsuzuri_contract::stats::{
    CheckResult, DayCount, EpicProgress, LeadDays, LedgerStats, MemoStats, NextCheck, NextStep,
    OpenCounts, UnreflectedCount, UnreflectedKind, UnreflectedList, UnreflectedRow,
};
use tsuzuri_contract::surface::{
    BatchItem, BatchItemResult, BatchRequest, BatchResponse, ItemOutcome, PolicyRequest,
    PolicyResponse, QuestionNudge, Refusal, RefusalResponse, RulingId, RulingRequest,
    RulingResponse, SeatHealth, SeatRole, SeatView, SurfaceEvent, SurfaceState,
};
use tsuzuri_contract::wire;

const AT: u64 = 1_790_494_620;

fn bead(s: &str) -> BeadId {
    BeadId::new(s).expect("見本の bead id")
}

fn ruling(s: &str) -> RulingId {
    RulingId::new(s).expect("見本の記帳 id")
}

/// 1 つの型の見本の値の列。
struct Samples<T> {
    name: &'static str,
    values: Vec<T>,
}

trait Form {
    fn name(&self) -> &'static str;
    fn json(&self) -> String;
    fn roundtrip(&self) -> Result<(), String>;
}

impl<T: Serialize + DeserializeOwned + PartialEq + Debug> Form for Samples<T> {
    fn name(&self) -> &'static str {
        self.name
    }

    fn json(&self) -> String {
        serde_json::to_string_pretty(&self.values).expect("見本を JSON にする")
    }

    fn roundtrip(&self) -> Result<(), String> {
        for v in &self.values {
            let text = wire::encode(v).map_err(|e| format!("{}: encode: {e}", self.name))?;
            let back: T =
                wire::decode(&text).map_err(|e| format!("{}: decode {text}: {e}", self.name))?;
            if &back != v {
                return Err(format!("{}: 値が戻らない {v:?} → {back:?}", self.name));
            }
            let again = wire::encode(&back).map_err(|e| format!("{}: encode: {e}", self.name))?;
            if again != text {
                return Err(format!("{}: 字が戻らない {text} → {again}", self.name));
            }
        }
        Ok(())
    }
}

fn form<T: Serialize + DeserializeOwned + PartialEq + Debug + 'static>(
    name: &'static str,
    values: Vec<T>,
) -> Box<dyn Form> {
    Box::new(Samples { name, values })
}

fn seat_view() -> SeatView {
    SeatView {
        seat: "t3:orchestrator".into(),
        role: SeatRole::Orchestrator,
        health: SeatHealth::Waiting,
        account: Some("acct-4".into()),
        move_left_s: Some(1200),
    }
}

fn pipeline_card() -> PipelineCard {
    PipelineCard {
        contract: bead("t3-hub.3"),
        runs: 2,
        stage: Stage::Running,
        reason: Some("verify".into()),
        account: Some("acct-4".into()),
        elapsed_s: Some(185),
    }
}

fn ledger_row() -> LedgerRow {
    LedgerRow {
        id: bead("t3-hub.5"),
        kind: "task".into(),
        title: "契約の型".into(),
        status: "open".into(),
        updated_at: AT,
        parent: Some(bead("t3-hub")),
        labels: vec![QUESTION_LABEL.into()],
    }
}

/// 親も label も無い行（根の bead）。
fn root_row() -> LedgerRow {
    LedgerRow {
        id: bead("t3-hub"),
        kind: "epic".into(),
        title: "根".into(),
        status: "closed".into(),
        updated_at: AT,
        parent: None,
        labels: vec![],
    }
}

fn ledger_writes() -> Vec<LedgerWrite> {
    vec![
        LedgerWrite::AppendNotes {
            id: bead("t3-hub.5"),
            line: "裁定 t3-hub.5:20260926T1437Z-1・逐語 = よい".into(),
        },
        LedgerWrite::CloseItem {
            id: bead("t3-hub.5"),
            reason: "裁定 t3-hub.5:20260926T1437Z-1".into(),
        },
        LedgerWrite::CreateChild {
            parent: bead("t3-hub"),
            title: "方針".into(),
            child_type: ChildType::Task,
            description: "全体への指示の控え".into(),
            labels: vec![QUESTION_LABEL.into(), "policy-scope:all".into()],
            effect: Some(Effect::Operation),
        },
        // 人の字が旗の形でも旗に化けない見本。
        LedgerWrite::AppendNotes {
            id: bead("t3-hub.5"),
            line: "--notes".into(),
        },
        LedgerWrite::CreateChild {
            parent: bead("t3-hub"),
            title: "--notes=x".into(),
            child_type: ChildType::Epic,
            description: "--parent=".into(),
            labels: vec![NOTES_REPLACE_FLAG.into()],
            effect: None,
        },
        LedgerWrite::ReopenItem {
            id: bead("t3-hub.5"),
            reason: "裁定 t3-hub.5:20260926T1440Z-1・取り消す = t3-hub.5:20260926T1437Z-1".into(),
        },
    ]
}

fn bead_attr() -> BeadAttr {
    BeadAttr {
        kind: NodeKind::Task,
        status: "open".into(),
        labels: vec!["surface".into()],
        pointers: vec!["design = contracts/surface-base.toml#b".into()],
        touches: vec![],
    }
}

fn run_attr() -> RunAttr {
    RunAttr {
        stage: Some("Gated".into()),
        account: Some("acct-4".into()),
    }
}

/// 3 値の判定の見本（合格・違反・まだ分からない）。
fn invariant_checks() -> Vec<InvariantCheck> {
    vec![
        InvariantCheck {
            id: "g-1".into(),
            verdict: Verdict::Pass,
            violations: 0,
            ids: vec![],
        },
        InvariantCheck {
            id: "g-2".into(),
            verdict: Verdict::Violation,
            violations: 2,
            ids: vec!["t3-hub.3".into(), "t3-hub.4".into()],
        },
        InvariantCheck {
            id: "g-3".into(),
            verdict: Verdict::Unknown,
            violations: 0,
            ids: vec![],
        },
    ]
}

fn task_node() -> GraphNode {
    GraphNode {
        id: "t3-hub.3".into(),
        kind: NodeKind::Task,
        file: None,
        digest: None,
        title: "契約の型".into(),
        line: None,
        plain: None,
        eng: None,
    }
}

fn run_node() -> GraphNode {
    GraphNode {
        id: "t3-hub.3-20260927T073916Z".into(),
        kind: NodeKind::Run,
        file: None,
        digest: None,
        title: "t3-hub.3-20260927T073916Z".into(),
        line: None,
        plain: None,
        eng: None,
    }
}

/// 状態を持つ bead の節点と、状態の無い走行の節点。
fn view_nodes() -> Vec<ViewNode> {
    vec![
        ViewNode {
            node: task_node(),
            status: Some("open".into()),
            rank: 0,
            kids: 2,
            degree: 5,
            group: false,
            fold: BoxFold::Folded,
        },
        ViewNode {
            node: run_node(),
            status: None,
            rank: 1,
            kids: 0,
            degree: 1,
            group: false,
            fold: BoxFold::Leaf,
        },
    ]
}

fn view_edge() -> ViewEdge {
    ViewEdge {
        from: "t3-hub.3-20260927T073916Z".into(),
        to: "t3-hub.3".into(),
        edge_type: EdgeType::RunOf,
        count: 1,
    }
}

/// 中心の行と、影響の側の 1 段目の行。
fn around_rows() -> Vec<AroundRow> {
    vec![
        AroundRow {
            node: task_node(),
            status: Some("open".into()),
            col: 0,
            via: None,
            edge_type: None,
            degree: 5,
        },
        AroundRow {
            node: run_node(),
            status: Some("Landed".into()),
            col: 1,
            via: Some("t3-hub.3".into()),
            edge_type: Some(EdgeType::RunOf),
            degree: 1,
        },
    ]
}

fn hub_cut() -> HubCut {
    HubCut {
        id: "t3-hub".into(),
        degree: 31,
    }
}

/// 年齢の在る memo と、作った時刻の読めない memo。
fn unreflected_rows() -> Vec<UnreflectedRow> {
    vec![
        UnreflectedRow {
            id: "t3-hub.9".into(),
            title: "[memo] 控え".into(),
            age_s: Some(604_800),
        },
        UnreflectedRow {
            id: "t3-hub.10".into(),
            title: "[memo] 時刻の無い控え".into(),
            age_s: None,
        },
    ]
}

fn ledger_item() -> LedgerItem {
    LedgerItem {
        row: ledger_row(),
        description: "本文".into(),
        notes: "裁定 t3-hub.5:20260926T1437Z-1".into(),
    }
}

/// 定型行を全部持つ問いの card。
fn question_card() -> QuestionCard {
    QuestionCard {
        id: bead("t3-hub.7"),
        title: "画面の色".into(),
        posted_at: AT,
        plain: Some("画面の色を決める".into()),
        eng: Some("CSS の変数を 2 組持つ".into()),
        reason: Some("夜に読む".into()),
        recommend: Some("暗い色を既定にする".into()),
        a1: true,
        touches: vec!["surface-base#b-cards".into(), "ADR-7".into()],
        blocking: vec!["t3-hub.30".into(), "t3-hub.31".into()],
        digest: ledger_item().digest(),
    }
}

fn group_row() -> GroupRow {
    GroupRow {
        group: "Tier1".into(),
        account: "acct-4".into(),
        candidates: vec!["acct-1".into(), "acct-3".into(), "acct-5".into()],
        next_account: Some("acct-5".into()),
        remaining: vec![QuotaLeft {
            window: "5h".into(),
            left_pct: 62,
        }],
    }
}

fn quota_used() -> Vec<QuotaUsed> {
    vec![
        QuotaUsed {
            window: "5h".into(),
            used_pct: 38,
            resets_at: Some(AT + 3_600),
            counted: true,
        },
        QuotaUsed {
            window: "7d-opus".into(),
            used_pct: 90,
            resets_at: None,
            counted: false,
        },
    ]
}

fn seat_spans() -> Vec<SeatSpan> {
    vec![
        SeatSpan {
            from: AT - 7_200,
            to: AT - 3_600,
            state: SeatState::Run,
        },
        SeatSpan {
            from: AT - 3_600,
            to: AT,
            state: SeatState::Wait,
        },
    ]
}

fn account_moves() -> Vec<AccountMove> {
    vec![
        AccountMove {
            at: AT - 86_400,
            from: None,
            to: "acct-3".into(),
        },
        AccountMove {
            at: AT - 3_600,
            from: Some("acct-3".into()),
            to: "acct-4".into(),
        },
    ]
}

/// 全型の見本（この順が snapshot の file の key の順）。
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
                RulingId::for_question(&bead("t3-hub.5"), "20260926T1437Z", 1).unwrap(),
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
                undelivered: vec![ruling("t3-hub.5:20260926T1437Z-1")],
            }],
        ),
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
                    ruling: ruling("t3-hub.7:20260926T1437Z-1"),
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
                ruling: ruling("t3-hub.7:20260926T1437Z-1"),
                recorded_at: AT,
            }],
        ),
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
                    ruling: ruling("t3-hub.7:20260926T1437Z-1"),
                },
                ItemOutcome::Skipped {
                    ruling: ruling("t3-hub.8:20260926T1437Z-1"),
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
                            ruling: ruling("t3-hub.7:20260926T1437Z-1"),
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
        // ledger
        form("ledger::BeadId", vec![bead("t3-hub"), bead("t3-hub.5")]),
        form("ledger::ChildType", vec![ChildType::Task, ChildType::Epic]),
        form("ledger::Effect", vec![Effect::Document, Effect::Operation]),
        form("ledger::LedgerRow", vec![ledger_row(), root_row()]),
        form("ledger::LedgerItem", vec![ledger_item()]),
        form(
            "ledger::LedgerList",
            vec![
                LedgerList {
                    rows: Reading::Known(vec![ledger_row()]),
                },
                LedgerList {
                    rows: Reading::Known(vec![]),
                },
                LedgerList {
                    rows: Reading::Unknown,
                },
            ],
        ),
        form("ledger::LedgerChanged", vec![LedgerChanged { at: AT }]),
        form("ledger::LedgerWrite", ledger_writes()),
        // graph
        form("graph::NodeKind", NodeKind::ALL.to_vec()),
        form("graph::EdgeType", EdgeType::ALL.to_vec()),
        form(
            "graph::GraphNode",
            vec![
                GraphNode {
                    id: "ADR-7".into(),
                    kind: NodeKind::Adr,
                    file: Some("adr/ADR-7.yaml".into()),
                    digest: Some("9c1e0a4b".into()),
                    title: title36(
                        "裁定面は project board と account board の 2 面とし、台帳と設計文書と走行を 1 つの導出グラフに結ぶ",
                    ),
                    line: Some(3),
                    plain: Some("決定の画面の形を決めます。".into()),
                    eng: Some("面は 2 つとする。".into()),
                },
                GraphNode {
                    id: "t3-hub.3".into(),
                    kind: NodeKind::Task,
                    file: None,
                    digest: None,
                    title: "契約の型".into(),
                    line: None,
                    plain: None,
                    eng: None,
                },
            ],
        ),
        form(
            "graph::GraphEdge",
            vec![GraphEdge {
                from: "t3-hub.3".into(),
                to: "surface-base#b".into(),
                edge_type: EdgeType::Design,
            }],
        ),
        form("graph::GraphSource", GraphSource::ALL.to_vec()),
        form("graph::BeadAttr", vec![bead_attr()]),
        form(
            "graph::RunAttr",
            vec![
                run_attr(),
                RunAttr {
                    stage: None,
                    account: None,
                },
            ],
        ),
        form("graph::Verdict", Verdict::ALL.to_vec()),
        form("graph::InvariantCheck", invariant_checks()),
        form(
            "graph::SkippedEdges",
            vec![SkippedEdges {
                design: 3,
                ledger: 0,
                design_nodes: 2,
            }],
        ),
        form(
            "graph::GraphDoc",
            vec![
                GraphDoc {
                    nodes: vec![
                        GraphNode {
                            id: "t3-hub.3".into(),
                            kind: NodeKind::Task,
                            file: None,
                            digest: None,
                            title: "契約の型".into(),
                            line: None,
                            plain: None,
                            eng: None,
                        },
                        GraphNode {
                            id: "t3-hub.3-20260927T073916Z".into(),
                            kind: NodeKind::Run,
                            file: None,
                            digest: None,
                            title: "t3-hub.3-20260927T073916Z".into(),
                            line: None,
                            plain: None,
                            eng: None,
                        },
                    ],
                    edges: vec![GraphEdge {
                        from: "t3-hub.3-20260927T073916Z".into(),
                        to: "t3-hub.3".into(),
                        edge_type: EdgeType::RunOf,
                    }],
                    unread: vec![GraphSource::Design],
                    beads: [("t3-hub.3".to_string(), bead_attr())].into(),
                    runs: [("t3-hub.3-20260927T073916Z".to_string(), run_attr())].into(),
                    invariants: invariant_checks(),
                    skipped: SkippedEdges {
                        design: 0,
                        ledger: 1,
                        design_nodes: 0,
                    },
                },
                // どの出所も読めない。
                GraphDoc {
                    nodes: vec![],
                    edges: vec![],
                    unread: GraphSource::ALL.to_vec(),
                    beads: Default::default(),
                    runs: Default::default(),
                    invariants: vec![InvariantCheck {
                        id: "g-1".into(),
                        verdict: Verdict::Unknown,
                        violations: 0,
                        ids: vec![],
                    }],
                    skipped: SkippedEdges {
                        design: 0,
                        ledger: 0,
                        design_nodes: 0,
                    },
                },
            ],
        ),
        form("graph::EdgeEnd", EdgeEnd::ALL.to_vec()),
        form("graph::ViewNode", view_nodes()),
        form("graph::ViewEdge", vec![view_edge()]),
        form(
            "graph::GraphView",
            vec![
                GraphView {
                    nodes: view_nodes(),
                    edges: vec![view_edge()],
                    shown: 2,
                    folded: 2,
                    cut: 1,
                    total: 5,
                    unread: vec![GraphSource::Design],
                    open: vec!["t3-hub.3".into()],
                    refused: vec![],
                },
                // 節点の無いグラフ。
                GraphView {
                    nodes: vec![],
                    edges: vec![],
                    shown: 0,
                    folded: 0,
                    cut: 0,
                    total: 0,
                    unread: GraphSource::ALL.to_vec(),
                    open: vec![],
                    refused: vec![],
                },
            ],
        ),
        form("graph::BoxFold", BoxFold::ALL.to_vec()),
        form("graph::Fold", Fold::ALL.to_vec()),
        form("graph::AroundRow", around_rows()),
        form("graph::HubCut", vec![hub_cut()]),
        form(
            "graph::AroundDoc",
            vec![
                AroundDoc {
                    center: "t3-hub.3".into(),
                    steps: 2,
                    fold: Fold::Up,
                    rows: around_rows(),
                    basis: 1,
                    impact: 1,
                    shown: 2,
                    total: 32,
                    cut_hub: 30,
                    cut_cap: 0,
                    hubs: vec![hub_cut()],
                    unread: vec![],
                },
                // 中心だけの近傍。
                AroundDoc {
                    center: "t3-hub.3".into(),
                    steps: 1,
                    fold: Fold::Both,
                    rows: vec![around_rows().remove(0)],
                    basis: 0,
                    impact: 0,
                    shown: 1,
                    total: 1,
                    cut_hub: 0,
                    cut_cap: 0,
                    hubs: vec![],
                    unread: vec![GraphSource::Runs],
                },
            ],
        ),
        // board
        form("board::Stage", Stage::ALL.to_vec()),
        form(
            "board::PipelineColumn",
            vec![
                PipelineColumn::QueuedBlocked,
                PipelineColumn::RunningGated,
                PipelineColumn::QuestionedFailedStopped,
                PipelineColumn::Landed,
            ],
        ),
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
                    elapsed_s: None,
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
        // stats
        form(
            "stats::LedgerStats",
            vec![
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
                        age_p50_days: Some(9.5),
                    },
                    unreflected: 3,
                    unreflected_kinds: vec![UnreflectedCount {
                        kind: UnreflectedKind::Memo,
                        count: 3,
                    }],
                    unreflected_unknown: vec![UnreflectedKind::Ruling, UnreflectedKind::Request],
                },
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
                        age_p50_days: None,
                    },
                    unreflected: 0,
                    unreflected_kinds: vec![],
                    unreflected_unknown: UnreflectedKind::ALL.to_vec(),
                },
            ],
        ),
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
        form(
            "stats::UnreflectedList",
            vec![
                UnreflectedList {
                    memos: Reading::Known(unreflected_rows()),
                    rulings: Reading::Unknown,
                    requests: Reading::Unknown,
                },
                UnreflectedList {
                    memos: Reading::Known(vec![]),
                    rulings: Reading::Known(vec![]),
                    requests: Reading::Known(vec![]),
                },
                // 台帳が読めない。
                UnreflectedList {
                    memos: Reading::Unknown,
                    rulings: Reading::Unknown,
                    requests: Reading::Unknown,
                },
            ],
        ),
        // question
        form(
            "question::QuestionCard",
            vec![
                question_card(),
                // 定型行も A-1 の印も名指しも無い問い。
                QuestionCard {
                    id: bead("t3-hub.8"),
                    title: "素の問い".into(),
                    posted_at: AT + 60,
                    plain: None,
                    eng: None,
                    reason: None,
                    recommend: None,
                    a1: false,
                    touches: vec![],
                    blocking: vec![],
                    digest: "cbf29ce484222325".into(),
                },
            ],
        ),
        form(
            "question::QuestionList",
            vec![
                QuestionList {
                    cards: Reading::Known(vec![question_card()]),
                },
                QuestionList {
                    cards: Reading::Known(vec![]),
                },
                QuestionList {
                    cards: Reading::Unknown,
                },
            ],
        ),
        // seat
        form("seat::SeatState", SeatState::ALL.to_vec()),
        form("seat::QuotaUsed", quota_used()),
        form("seat::SeatSpan", seat_spans()),
        form("seat::AccountMove", account_moves()),
        form(
            "seat::SeatCard",
            vec![
                SeatCard {
                    at: AT,
                    target: "t3:orchestrator".into(),
                    state: SeatState::Wait,
                    since: Some(AT - 3_600),
                    tick_healthy: Reading::Known(true),
                    heartbeat: Reading::Known(false),
                    account: Some("acct-4".into()),
                    model: Some("opus".into()),
                    group: Reading::Known(group_row()),
                    usage: Reading::Known(quota_used()),
                    spans: Reading::Known(seat_spans()),
                    moves: Reading::Known(account_moves()),
                },
                // 器も口座も読めない席。
                SeatCard {
                    at: AT,
                    target: "t3:orchestrator".into(),
                    state: SeatState::Unknown,
                    since: None,
                    tick_healthy: Reading::Unknown,
                    heartbeat: Reading::Unknown,
                    account: None,
                    model: None,
                    group: Reading::Unknown,
                    usage: Reading::Unknown,
                    spans: Reading::Unknown,
                    moves: Reading::Unknown,
                },
            ],
        ),
    ]
}

/// snapshot の群（群の名と、群が見本を持つ module の列・見本の群は型の名の module の頭で決まる）。
const GROUPS: [(&str, &[&str]); 6] = [
    ("surface", &["surface"]),
    ("ledger", &["ledger", "question"]),
    ("graph", &["graph"]),
    ("board", &["board"]),
    ("stats", &["stats"]),
    ("seat", &["seat"]),
];

/// snapshot の字（型の名を key にした 1 つの JSON の object・key は `forms` の順・値は電文の欄の順のまま）。
fn snapshot_text(forms: &[Box<dyn Form>]) -> String {
    let names: BTreeSet<&str> = forms.iter().map(|f| f.name()).collect();
    assert_eq!(names.len(), forms.len(), "型の名が重なる");
    let entries: Vec<String> = forms
        .iter()
        .map(|f| {
            let body = f.json().replace('\n', "\n  ");
            format!("  {}: {body}", serde_json::to_string(f.name()).expect("名"))
        })
        .collect();
    let text = format!("{{\n{}\n}}\n", entries.join(",\n"));
    serde_json::from_str::<serde_json::Value>(&text).expect("snapshot の字が JSON である");
    text
}

/// 群の snapshot（tests/snapshots の群の名の json）の字が群の見本の字と同じことを見る。
fn snapshot_matches(group: &str, forms: &[Box<dyn Form>]) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/snapshots/{group}.json"));
    let want = std::fs::read_to_string(&path).unwrap_or_default();
    let got = snapshot_text(forms);
    if got != want {
        let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{group}.json"));
        std::fs::write(&out, &got).expect("今の字を書く");
        let line = got
            .lines()
            .zip(want.lines())
            .position(|(g, w)| g != w)
            .map_or_else(
                || got.lines().count().min(want.lines().count()) + 1,
                |i| i + 1,
            );
        panic!(
            "snapshot の字が {} 行目で違う（今の字 = {}）",
            line,
            out.display()
        );
    }
}

#[test]
fn contract_form_snapshot_matches() {
    let mut groups: Vec<Vec<Box<dyn Form>>> = GROUPS.iter().map(|_| Vec::new()).collect();
    for f in forms() {
        let module = f.name().split("::").next().unwrap_or_default();
        let i = GROUPS
            .iter()
            .position(|(_, modules)| modules.contains(&module))
            .unwrap_or_else(|| panic!("{} はどの群の module でもない", f.name()));
        groups[i].push(f);
    }
    for ((group, _), forms) in GROUPS.iter().zip(&groups) {
        snapshot_matches(group, forms);
    }
}

#[test]
fn contract_form_roundtrip_all_types() {
    let fails: Vec<String> = forms().iter().filter_map(|f| f.roundtrip().err()).collect();
    assert!(fails.is_empty(), "往復が一致しない: {fails:#?}");
}

#[test]
fn contract_form_ledger_write_argv() {
    let writes = ledger_writes();
    let mut variants = BTreeSet::new();
    for w in &writes {
        // 全 variant を数える（variant が増えれば match が網羅でなくなり組めない）。
        variants.insert(match w {
            LedgerWrite::AppendNotes { .. } => "append-notes",
            LedgerWrite::CloseItem { .. } => "close-item",
            LedgerWrite::CreateChild { .. } => "create-child",
            LedgerWrite::ReopenItem { .. } => "reopen-item",
        });
        let argv = w.argv();
        assert!(!argv.is_empty(), "{w:?}: argv が空");
        assert_ne!(argv[0], BDW, "{w:?}: argv は program の名を含めない");
        // 旗として読まれるのは `--` の前の語だけ。
        let flags: Vec<&String> = argv.iter().take_while(|a| a.as_str() != "--").collect();
        assert!(
            !flags.iter().any(|a| {
                a.as_str() == NOTES_REPLACE_FLAG
                    || a.starts_with(&format!("{NOTES_REPLACE_FLAG}="))
                    || a.as_str() == "-n"
            }),
            "{w:?}: notes を置き換える旗を持つ {argv:?}"
        );
        let parents: Vec<&str> = flags
            .iter()
            .filter_map(|a| a.strip_prefix(&format!("{PARENT_FLAG}=")))
            .collect();
        if w.creates_child() {
            assert_eq!(argv[0], "create", "{w:?}");
            assert_eq!(
                parents.len(),
                1,
                "{w:?}: 子を作る argv は親を 1 つ持つ {argv:?}"
            );
            assert!(
                BeadId::new(parents[0]).is_ok(),
                "{w:?}: 親が bead id でない {argv:?}"
            );
        } else {
            assert_ne!(argv[0], "create", "{w:?}: 子を作らない書きが create を撃つ");
        }
        // 人の字は旗の値か `--` の後にだけ在り、旗の位置に素で立たない。
        for a in &flags[1..] {
            assert!(
                !a.starts_with('-') || a.starts_with("--") && a.contains('='),
                "{w:?}: 値の無い旗 {a} {argv:?}"
            );
        }
    }
    assert_eq!(variants.len(), 4,"全 variant の見本が要る: {variants:?}");
}

#[test]
fn contract_form_ids_refuse_bad_shape() {
    for bad in ["\"\"", "\"-x\"", "\"a b\"", "\"t3:hub\""] {
        assert!(
            wire::decode::<BeadId>(bad).is_err(),
            "bead id が {bad} を通す"
        );
    }
    for bad in ["\"\"", "\"-x\"", "\"a@b#1\""] {
        assert!(
            wire::decode::<RulingId>(bad).is_err(),
            "記帳 id が {bad} を通す"
        );
    }
    let long = format!("\"{}\"", "a".repeat(65));
    assert!(wire::decode::<RulingId>(&long).is_err());
    // 親の無い子の作成は電文でも組めない。
    let orphan = r#"{"op":"create-child","title":"x","child_type":"task","description":"y"}"#;
    assert!(wire::decode::<LedgerWrite>(orphan).is_err());
    // 閉じた enum は知らない語を断る。
    assert!(
        wire::decode::<LedgerWrite>(r#"{"op":"replace-notes","id":"t3-hub.5","notes":"x"}"#)
            .is_err()
    );
    assert!(wire::decode::<NodeKind>("\"file\"").is_err());
}

#[test]
fn contract_form_bd_line_reads() {
    // 知らない欄は読み捨て、省かれた種類・本文・notes・親・label は空で読む。
    let line = r#"{"id":"t3-hub.5","title":"契約の型","status":"open","priority":2,"updated_at":"2026-09-27T07:39:00Z","labels":["x"]}"#;
    let bd: BdLine = wire::decode(line).expect("bd の行");
    assert!(!bd.is_tombstone());
    let item = bd.into_item(AT);
    assert_eq!(item.row.id, bead("t3-hub.5"));
    assert_eq!(item.row.kind, "");
    assert_eq!(item.row.updated_at, AT);
    assert_eq!(item.row.parent, None);
    assert_eq!(item.row.labels, vec!["x".to_string()]);
    assert_eq!(item.description, "");
    // 親と label は行へ写る。
    let child = r#"{"id":"t3-hub.7","title":"問い","status":"closed","updated_at":"2026-09-27T07:39:00Z","parent":"t3-hub","labels":["intake:question"],"dependencies":[{"issue_id":"t3-hub.7","depends_on_id":"t3-hub","type":"parent-child"}]}"#;
    let row = wire::decode::<BdLine>(child)
        .expect("bd の行")
        .into_item(AT)
        .row;
    assert_eq!(row.parent, Some(bead("t3-hub")));
    assert!(row.is_question() && !row.is_memo(), "{row:?}");
    // 形の悪い id と親と欠けた更新時刻は読まない。
    for bad in [
        r#"{"id":"-x","title":"t","status":"open","updated_at":"2026-09-27T07:39:00Z"}"#,
        r#"{"id":"t3-hub.5","title":"t","status":"open"}"#,
        r#"{"id":"t3-hub.5","title":"t","status":"open","updated_at":"2026-09-27T07:39:00Z","parent":"a b"}"#,
    ] {
        assert!(wire::decode::<BdLine>(bad).is_err(), "{bad} を読む");
    }
    let gone =
        r#"{"id":"t3-hub.6","title":"t","status":"tombstone","updated_at":"2026-09-27T07:39:00Z"}"#;
    assert!(
        wire::decode::<BdLine>(gone)
            .expect("bd の行")
            .is_tombstone()
    );
}

#[test]
fn contract_form_ledger_row_labels() {
    assert_eq!(QUESTION_LABEL, "intake:question");
    assert_eq!(MEMO_LABEL, "intake:memo");
    let question = ledger_row();
    assert!(question.is_question() && !question.is_memo());
    let memo = LedgerRow {
        labels: vec!["surface".into(), MEMO_LABEL.into()],
        ..ledger_row()
    };
    assert!(memo.is_memo() && !memo.is_question());
    let plain = root_row();
    assert!(!plain.is_memo() && !plain.is_question());
    // label の語の一部だけでは見分けない。
    let near = LedgerRow {
        labels: vec!["intake:questions".into(), "intake".into()],
        ..ledger_row()
    };
    assert!(!near.is_question() && !near.is_memo());
}

#[test]
fn contract_form_ledger_item_digest() {
    // FNV-1a 64 bit の公開の見本。
    assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
    let item = ledger_item();
    let digest = item.digest();
    assert_eq!(digest, "0874e9bb3b87f545");
    assert_eq!(
        digest,
        format!(
            "{:016x}",
            fnv1a64("t3-hub.5\n契約の型\nopen\n本文\n裁定 t3-hub.5:20260926T1437Z-1".as_bytes())
        )
    );
    // 同じ中身なら同じ値（数えない欄は値を動かさない）。
    assert_eq!(item.clone().digest(), digest);
    let other_row = LedgerItem {
        row: LedgerRow {
            kind: "epic".into(),
            updated_at: AT + 1,
            parent: None,
            labels: vec![],
            ..ledger_row()
        },
        ..ledger_item()
    };
    assert_eq!(other_row.digest(), digest);
    // どの欄の 1 字が変わっても値が変わる（区切りの位置が動くだけでも変わる）。
    let changed = [
        LedgerItem {
            row: LedgerRow {
                id: bead("t3-hub.6"),
                ..ledger_row()
            },
            ..ledger_item()
        },
        LedgerItem {
            row: LedgerRow {
                title: "契約の形".into(),
                ..ledger_row()
            },
            ..ledger_item()
        },
        LedgerItem {
            row: LedgerRow {
                status: "opem".into(),
                ..ledger_row()
            },
            ..ledger_item()
        },
        LedgerItem {
            description: "本分".into(),
            ..ledger_item()
        },
        LedgerItem {
            notes: "裁定 t3-hub.5:20260926T1437Z-2".into(),
            ..ledger_item()
        },
        LedgerItem {
            description: "本文\n裁定".into(),
            notes: " t3-hub.5:20260926T1437Z-1".into(),
            ..ledger_item()
        },
    ];
    let mut seen = BTreeSet::from([digest.clone()]);
    for c in &changed {
        let d = c.digest();
        assert_eq!(d.len(), 16, "{d}");
        assert!(
            d.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
            "16 進の小文字でない {d}"
        );
        assert!(seen.insert(d.clone()), "{c:?}: 値が変わらない {d}");
    }
}

#[test]
fn contract_form_closed_lists() {
    fn distinct<T: Serialize>(all: &[T]) -> usize {
        all.iter()
            .map(|v| serde_json::to_string(v).expect("語"))
            .collect::<BTreeSet<_>>()
            .len()
    }
    assert_eq!(distinct(&NodeKind::ALL), 20);
    assert_eq!(distinct(&EdgeType::ALL), 30);
    assert_eq!(distinct(&GraphSource::ALL), 3);
    assert_eq!(distinct(&EdgeEnd::ALL), 2);
    let fold_words: Vec<String> = Fold::ALL
        .iter()
        .map(|f| wire::encode(f).expect("語"))
        .collect();
    assert_eq!(
        fold_words,
        ["\"none\"", "\"up\"", "\"down\"", "\"both\""],
        "畳みの ALL の順と字"
    );
    assert!(wire::decode::<Fold>("\"all\"").is_err());
    let box_words: Vec<String> = BoxFold::ALL
        .iter()
        .map(|f| wire::encode(f).expect("語"))
        .collect();
    assert_eq!(
        box_words,
        ["\"leaf\"", "\"folded\"", "\"open\""],
        "箱の開き閉じの ALL の順と字"
    );
    assert_eq!(distinct(&Verdict::ALL), 3);
    assert_eq!(distinct(&NextMove::ALL), 7);
    assert_eq!(distinct(&LedgerJudge::ALL), 5);
    assert_eq!(distinct(&Stage::ALL), 8);
    assert_eq!(distinct(&CheckResult::ALL), 3);
    assert_eq!(distinct(&UnreflectedKind::ALL), 3);
    assert_eq!(distinct(&SeatState::ALL), 5);
    let seat_words: Vec<String> = SeatState::ALL
        .iter()
        .map(|s| wire::encode(s).expect("語"))
        .collect();
    assert_eq!(
        seat_words,
        [
            "\"run\"",
            "\"wait\"",
            "\"limit\"",
            "\"silent\"",
            "\"unknown\""
        ],
        "席の状態の ALL の順と字"
    );
    assert!(wire::decode::<SeatState>("\"stopped\"").is_err());
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
    assert_eq!(columns.len(), 4);
}
