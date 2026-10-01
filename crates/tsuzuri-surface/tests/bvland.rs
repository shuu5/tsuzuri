//! 行 c-landed-12h の歯: pipeline の板の着地の列は、今から 12 時間（規則の行 R-36）の内の着地と、着地の時刻を問わず
//! CI を待つ札を出す（判断の記録 ADR-27 決定 (6)・前の日本の日の今日をやめる）。範囲の秒は規則の行の字と照らす。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{Ci, PipelineCard, PipelineColumn, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_surface::project::pipeline::{LAND_WINDOW_S, columns, landed_recent, lane};
use tsuzuri_surface::view::jst;
use tsuzuri_surface::vocab::vocab;

/// 着地済みの pipe_ の歯と同じ今（UTC の日の正午）。
const NOW: EpochSecs = 1_790_510_400;

/// 日本時間の 0 時 10 分（NOW の日の翌日）。
const AFTER: EpochSecs = 1_790_521_800;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn card(id: &str, stage: Stage, since: Option<EpochSecs>, ci: Option<Ci>) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(id).expect("id"),
        runs: 1,
        stage,
        reason: None,
        account: None,
        since,
        ci,
    }
}

fn land_ids(cards: &[PipelineCard], now: EpochSecs) -> Vec<String> {
    let cols = columns(cards, &[], now);
    let land = cols
        .iter()
        .find(|c| c.lane.column == PipelineColumn::Landed)
        .expect("着地の列");
    land.cards.iter().map(|k| k.id.clone()).collect()
}

/// (1) 範囲の秒 LAND_WINDOW_S は 43200（12 時間）で、規則の行 R-36 の value の字の時間の数と同じ。日本の日の今日の判じは残らない。
#[test]
fn bvland_window_is_rule_r36() {
    assert_eq!(LAND_WINDOW_S, 43_200);
    let rules = read("../../design-intent/rules.yaml");
    let row = rules
        .lines()
        .find(|l| l.contains("id: R-36,"))
        .expect("rules の file に行 R-36 が在る");
    let value = row
        .split_once("value: \"")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map(|(v, _)| v)
        .expect("行 R-36 の value");
    let hours: u64 = value
        .split_once("今から")
        .and_then(|(_, rest)| rest.split_once("時間 以内"))
        .and_then(|(n, _)| n.trim().parse().ok())
        .expect("今から何時間の数");
    assert_eq!(hours * 3_600, LAND_WINDOW_S, "{row}");
    let src = read("src/project/pipeline.rs");
    assert!(src.contains("pub const LAND_WINDOW_S: u64 = 43_200;"));
    assert!(!src.contains("landed_today"), "前の今日の関数が残る");
    assert!(!src.contains("jst("), "pipeline.rs が日本の日で判じる");
}

/// (2) 日本時間の 0 時を越えても、前の日の夜の着地は 12 時間の内なら列に残る（0 時 10 分に 23 時 50 分の着地が在る）。
#[test]
fn bvland_kept_after_midnight() {
    assert_eq!(jst(AFTER).1, 600, "AFTER は日本時間の 0 時 10 分");
    let night = AFTER - 1_200;
    assert_ne!(jst(night).0, jst(AFTER).0, "23 時 50 分は前の日本の日");
    let cards = [
        card("bv.1", Stage::Landed, Some(night), None),
        card("bv.2", Stage::Landed, Some(AFTER - LAND_WINDOW_S), None),
        card("bv.3", Stage::Landed, Some(AFTER - LAND_WINDOW_S - 1), None),
    ];
    assert!(landed_recent(&cards[0], AFTER));
    assert_eq!(land_ids(&cards, AFTER), ["bv.1", "bv.2"]);
}

/// (3) 経過が範囲を 1 秒でも越えた着地と時刻の無い着地は列から落ち、Landed でない段は範囲の内でも偽。CI を待つ札は時刻を問わず残る。
#[test]
fn bvland_dropped_after_window() {
    let at = |elapsed: u64| Some(NOW - elapsed);
    for (since, want) in [
        (at(0), true),
        (at(LAND_WINDOW_S), true),
        (at(LAND_WINDOW_S + 1), false),
        (Some(NOW + 30), true),
        (None, false),
    ] {
        let c = card("bv.1", Stage::Landed, since, None);
        assert_eq!(landed_recent(&c, NOW), want, "{since:?}");
    }
    for stage in Stage::ALL.into_iter().filter(|s| *s != Stage::Landed) {
        assert!(!landed_recent(&card("bv.1", stage, at(60), None), NOW), "{stage:?}");
    }
    let cards = [
        card("bv.4", Stage::Landed, at(60), None),
        card("bv.5", Stage::Landed, at(LAND_WINDOW_S + 1), None),
        card("bv.6", Stage::Landed, at(2 * 86_400), Some(Ci::Waiting)),
        card("bv.7", Stage::Landed, None, None),
    ];
    assert_eq!(land_ids(&cards, NOW), ["bv.4", "bv.6"]);
    // 今が 12 時間進めば bv.4 も落ち、CI を待つ bv.6 だけが残る。
    assert_eq!(land_ids(&cards, NOW + LAND_WINDOW_S), ["bv.6"]);
}

/// (4) 着地の列の見出しの語の鍵は col_land_12h（語は Landed（12 時間））。口座の板の数の語 col_land は今日のまま残る。
#[test]
fn bvland_lane_word_12h() {
    assert_eq!(lane(PipelineColumn::Landed).key, "col_land_12h");
    let v = vocab();
    let term = v.term("col_land_12h").expect("鍵 col_land_12h");
    assert_eq!(term.label, "Landed（12 時間）");
    assert!(term.note.starts_with("今から 12 時間の内に"), "{}", term.note);
    assert_eq!(
        v.term("col_land").map(|t| t.label.as_str()),
        Some("Landed（今日）")
    );
}
