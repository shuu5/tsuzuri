//! 便 g-min の歯: bead 8 本の fixture・状態の印・時刻の字・id の順。
//! fixture は server と同じ台帳の読み（tsuzuri_boundary::server::ledger::parse）で読む。
//! 画面の状態（Screen）と問いの一覧と epic の組の並べ方の歯は、読み手が無くなった型と関数とともに行 g-dead-sweep-b で消した。
#![cfg(test)]

use std::path::Path;

use tsuzuri_boundary::server::ledger;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::LedgerRow;
use tsuzuri_surface::view::{clock, id_order, mark};

fn fixture_rows() -> Vec<LedgerRow> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/ledger/board-8.jsonl");
    let text = std::fs::read_to_string(&path).expect("fixture board-8.jsonl");
    let Reading::Known(items) = ledger::parse(&text) else {
        panic!("fixture が server の読みで Unknown");
    };
    items.into_iter().map(|i| i.row).collect()
}

#[test]
fn board_min_fixture_has_eight_beads() {
    assert_eq!(fixture_rows().len(), 8);
}

#[test]
fn board_min_status_marks() {
    let marks: Vec<&str> = [
        "open",
        "in_progress",
        "blocked",
        "deferred",
        "closed",
        "pinned",
    ]
    .iter()
    .map(|s| mark(s).glyph)
    .collect();
    assert_eq!(marks, vec!["○", "◐", "⊘", "◌", "●", "?"]);
    assert_eq!(mark("in_progress").class, "st-in-progress");
    assert_eq!(mark("closed").word, "閉じた");
}

#[test]
fn board_min_clock_is_jst() {
    assert_eq!(clock(0), "1970-01-01 09:00:00 JST");
    assert_eq!(clock(1_790_494_740), "2026-09-27 16:39:00 JST");
    assert_eq!(clock(1_709_208_000), "2024-02-29 21:00:00 JST");
}

#[test]
fn board_min_id_order_by_segments() {
    use std::cmp::Ordering::{Equal, Greater, Less};
    assert_eq!(id_order("a.2", "a.10"), Less);
    assert_eq!(id_order("a.10", "a.2"), Greater);
    assert_eq!(id_order("a", "a.1"), Less);
    assert_eq!(id_order("a.1", "a.1"), Equal);
    assert_eq!(id_order("a.01", "a.1"), Less);
    assert_eq!(id_order("b", "a.9"), Greater);
}
