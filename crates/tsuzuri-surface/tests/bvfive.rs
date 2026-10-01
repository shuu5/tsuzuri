//! 行 c-pipe-five の歯: pipeline の板の 5 列（Blocked と Queued を分ける・判断の記録 ADR-27 決定 (6)）の順と電文の名と、
//! Blocked の札と Queued の札が別の列に載ることと、列の見出しの語の鍵と class が語の辞書と stylesheet に在ること。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{PipelineCard, PipelineColumn, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::wire;
use tsuzuri_surface::project::pipeline::{LANES, columns, lane};
use tsuzuri_surface::vocab::vocab;

/// 着地済みの pipe_ の歯と同じ今（UTC の日の正午）。
const NOW: EpochSecs = 1_790_510_400;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn card(id: &str, stage: Stage, elapsed: u64) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(id).expect("id"),
        runs: 1,
        stage,
        reason: None,
        account: None,
        since: Some(NOW - elapsed),
        ci: None,
    }
}

/// (1) 列は閉じた 5 で、板の順は Blocked・Queued・Running / Gated・止まり・着地。電文の名と URL の名も 5 つで、
/// Blocked と Queued の列の見出しの語の鍵は col_block と col_queue。
#[test]
fn bvfive_five_columns_in_order() {
    let order: Vec<PipelineColumn> = LANES.iter().map(|l| l.column).collect();
    assert_eq!(
        order,
        vec![
            PipelineColumn::Blocked,
            PipelineColumn::Queued,
            PipelineColumn::RunningGated,
            PipelineColumn::QuestionedFailedStopped,
            PipelineColumn::Landed,
        ]
    );
    let wire: Vec<String> = order
        .iter()
        .map(|c| wire::encode(c).expect("列の電文"))
        .collect();
    assert_eq!(
        wire,
        [
            "\"Blocked\"",
            "\"Queued\"",
            "\"Running/Gated\"",
            "\"Questioned/Failed/Stopped\"",
            "\"Landed\"",
        ]
    );
    let names: Vec<&str> = LANES.iter().map(|l| l.name).collect();
    assert_eq!(names, ["block", "queue", "run", "stop", "land"]);
    assert_eq!([LANES[0].key, LANES[1].key], ["col_block", "col_queue"]);
    assert_eq!(Stage::Blocked.column(), PipelineColumn::Blocked);
    assert_eq!(Stage::Queued.column(), PipelineColumn::Queued);
    for stage in Stage::ALL {
        assert_eq!(lane(stage.column()).column, stage.column(), "{stage:?}");
    }
}

/// (2) Blocked の札は 1 列目、Queued の札は 2 列目に載り、同じ列に混ざらない（class は col c-block と col c-queue）。
#[test]
fn bvfive_blocked_and_queued_apart() {
    let cards = [
        card("bv.1", Stage::Queued, 60),
        card("bv.2", Stage::Blocked, 120),
        card("bv.3", Stage::Queued, 30),
        card("bv.4", Stage::Running, 10),
    ];
    let cols = columns(&cards, &[], NOW);
    assert_eq!(cols.len(), 5);
    let ids: Vec<Vec<&str>> = cols
        .iter()
        .map(|c| c.cards.iter().map(|k| k.id.as_str()).collect())
        .collect();
    assert_eq!(
        ids,
        vec![vec!["bv.2"], vec!["bv.3", "bv.1"], vec!["bv.4"], vec![], vec![]]
    );
    assert_eq!(cols[0].class, "col c-block");
    assert_eq!(cols[1].class, "col c-queue");
    // 両方の列の札は待ちの記号。
    assert_eq!(cols[0].cards[0].state, Some("wait"));
    assert_eq!(cols[1].cards[0].state, Some("wait"));
}

/// (3) 見出しの語の鍵 col_block と col_queue は語の辞書に在り、class は stylesheet に在り、板の grid は 5 列。
/// 口座の板の 4 つの数の語（col_wait）は残る（口座の板の形は変えない・判断の記録 ADR-27 決定 (1)）。
#[test]
fn bvfive_column_words_in_vocab() {
    let v = vocab();
    assert_eq!(v.term("col_block").map(|t| t.label.as_str()), Some("Blocked"));
    assert_eq!(v.term("col_queue").map(|t| t.label.as_str()), Some("Queued"));
    assert_eq!(
        v.term("col_wait").map(|t| t.label.as_str()),
        Some("Queued / Blocked")
    );
    let css = read("style.css");
    for rule in [".col.c-block .dot {", ".col.c-queue .dot {", ".col.c-block, .col.c-queue {"] {
        assert!(css.contains(rule), "stylesheet に {rule} が無い");
    }
    assert!(!css.contains(".col.c-wait"), "列の class c-wait が残る");
    assert!(css.contains(".board { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr));"));
    let cards = read("src/account/cards.rs");
    assert!(cards.contains("[\"col_wait\", \"col_run\", \"col_stop\", \"col_land\"]"));
}
