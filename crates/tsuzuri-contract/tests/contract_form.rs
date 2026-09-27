//! 契約の型の外形の歯（接頭辞 contract_form_）。
//! snapshot は全型の見本の値を JSON にした 1 つの file で、字の比べは標準 library だけで行う。
//! 字が違えば今の字を CARGO_TARGET_TMPDIR の contract_form.json に書いて落ちる（見て正しければ snapshot へ写す）。

use std::collections::BTreeSet;
use std::fmt::Debug;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use tsuzuri_contract::board::{
    AccountBoard, GroupRow, LedgerJudge, NextMove, PipelineBoard, PipelineCard, PipelineColumn,
    ProjectMetrics, QuotaLeft, Reading, SessionRow, Stage,
};
use tsuzuri_contract::graph::{EdgeType, GraphEdge, GraphNode, NodeKind, title36};
use tsuzuri_contract::ledger::{
    BDW, BdLine, BeadId, ChildType, LedgerChanged, LedgerItem, LedgerList, LedgerRow, LedgerWrite,
    MEMO_LABEL, NOTES_REPLACE_FLAG, PARENT_FLAG, QUESTION_LABEL,
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
        account: Some("black4".into()),
        move_left_s: Some(1200),
    }
}

fn pipeline_card() -> PipelineCard {
    PipelineCard {
        contract: bead("t3-hub.3"),
        runs: 2,
        stage: Stage::Running,
        reason: Some("verify".into()),
        account: Some("black4".into()),
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
                RulingId::for_policy("20260926T1437Z", 2).unwrap(),
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
                policy: ruling("policy:20260926T1437Z-1"),
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
        form("ledger::LedgerRow", vec![ledger_row(), root_row()]),
        form(
            "ledger::LedgerItem",
            vec![LedgerItem {
                row: ledger_row(),
                description: "本文".into(),
                notes: "裁定 t3-hub.5:20260926T1437Z-1".into(),
            }],
        ),
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
                },
                GraphNode {
                    id: "t3-hub.3".into(),
                    kind: NodeKind::Task,
                    file: None,
                    digest: None,
                    title: "契約の型".into(),
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
                },
                PipelineBoard {
                    cards: Reading::Known(vec![]),
                },
                PipelineBoard {
                    cards: Reading::Unknown,
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
                    account: "black4".into(),
                    candidates: vec!["black1".into(), "black3".into(), "black5".into()],
                    next_account: Some("black5".into()),
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
                    account: "black4".into(),
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

/// snapshot の字（型の名を key にした 1 つの JSON の object・key は `forms` の順・値は電文の欄の順のまま）。
fn snapshot_text() -> String {
    let forms = forms();
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

#[test]
fn contract_form_snapshot_matches() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/contract_form.json");
    let want = std::fs::read_to_string(&path).unwrap_or_default();
    let got = snapshot_text();
    if got != want {
        let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("contract_form.json");
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
    assert_eq!(variants.len(), 3, "全 variant の見本が要る: {variants:?}");
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
fn contract_form_closed_lists() {
    fn distinct<T: Serialize>(all: &[T]) -> usize {
        all.iter()
            .map(|v| serde_json::to_string(v).expect("語"))
            .collect::<BTreeSet<_>>()
            .len()
    }
    assert_eq!(distinct(&NodeKind::ALL), 20);
    assert_eq!(distinct(&EdgeType::ALL), 30);
    assert_eq!(distinct(&NextMove::ALL), 7);
    assert_eq!(distinct(&LedgerJudge::ALL), 5);
    assert_eq!(distinct(&Stage::ALL), 8);
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
