//! 便 g-min の歯: 並べ方（bead 8 本の fixture）・測れていないの判定・状態の印・時刻の字。
//! fixture は server と同じ台帳の読み（tsuzuri_boundary::server::ledger::parse）で読み、
//! 一覧の口と同じ電文（LedgerList）にしてから画面の状態へ渡す。
//! 件数は block の module（ask・ledger）が数える（便 g-frame で見出しの字を vocab へ移した）。
//! 問いの block（ask）は便 g-ask で口 /api/questions の電文を読むので、問いの件数は台帳の画面の状態から数える。

use std::path::Path;

use tsuzuri_boundary::server::ledger;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{LedgerList, LedgerRow};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::{self, ask};
use tsuzuri_surface::view::{self, Board, Fetched, Screen, clock, id_order, mark};

/// 画面の状態の問いの件数（測れていなければ Unknown）。
fn questions(screen: &Screen) -> Reading<usize> {
    match &screen.board {
        Reading::Known(b) => Reading::Known(b.questions.len()),
        Reading::Unknown => Reading::Unknown,
    }
}

fn fixture_rows() -> Vec<LedgerRow> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/ledger/board-8.jsonl");
    let text = std::fs::read_to_string(&path).expect("fixture board-8.jsonl");
    let Reading::Known(items) = ledger::parse(&text) else {
        panic!("fixture が server の読みで Unknown");
    };
    items.into_iter().map(|i| i.row).collect()
}

fn ids(rows: &[LedgerRow]) -> Vec<&str> {
    rows.iter().map(|r| r.id.as_str()).collect()
}

fn body(rows: Reading<Vec<LedgerRow>>) -> Fetched {
    Fetched::Body(wire::encode(&LedgerList { rows }).expect("電文"))
}

#[test]
fn board_min_fixture_has_eight_beads() {
    assert_eq!(fixture_rows().len(), 8);
}

#[test]
fn board_min_questions_open_oldest_first() {
    let rows = fixture_rows();
    // bm.4 は +09:00 の字で bm.5 より後（字の順でなく時刻の順）。閉じた bm.6 は出さない。
    assert_eq!(ids(&view::questions(&rows)), vec!["bm.5", "bm.4"]);
}

#[test]
fn board_min_ledger_epic_then_tasks_then_memos() {
    let groups = view::ledger_groups(&fixture_rows());
    assert_eq!(groups.len(), 1, "{groups:?}");
    let group = &groups[0];
    assert_eq!(group.epic.as_ref().map(|e| e.id.as_str()), Some("bm"));
    // task は id の段の数の順（bm.10 は bm.3 の後）、その後に memo。question は台帳の一覧に入れない。
    assert_eq!(ids(&group.children), vec!["bm.1", "bm.3", "bm.10", "bm.2"]);
}

#[test]
fn board_min_ledger_outside_epic_goes_last() {
    let mut rows = fixture_rows();
    let mut stray = rows[0].clone();
    stray.id = "zz.1".to_string().try_into().expect("id");
    stray.kind = "task".to_string();
    rows.push(stray);
    let mut second = rows[2].clone();
    second.id = "am".to_string().try_into().expect("id");
    rows.push(second);
    let groups = view::ledger_groups(&rows);
    let heads: Vec<Option<&str>> = groups
        .iter()
        .map(|g| g.epic.as_ref().map(|e| e.id.as_str()))
        .collect();
    assert_eq!(heads, vec![Some("am"), Some("bm"), None]);
    assert!(groups[0].children.is_empty());
    assert_eq!(ids(&groups[2].children), vec!["zz.1"]);
}

#[test]
fn board_min_screen_reads_known_ledger() {
    let rows = fixture_rows();
    let screen = Screen::initial().after_read(&body(Reading::Known(rows.clone())), 1_790_495_000);
    assert_eq!(screen.board, Reading::Known(view::board(&rows)));
    assert_eq!(screen.updated_at, Some(1_790_495_000));
    assert_eq!(questions(&screen), Reading::Known(2));
    assert_eq!(project::ledger::count(&screen), Reading::Known(5));
}

#[test]
fn board_min_unmeasured_is_not_zero() {
    let empty = Screen::initial().after_read(&body(Reading::Known(vec![])), 10);
    assert_eq!(
        empty.board,
        Reading::Known(Board {
            questions: vec![],
            groups: vec![],
        })
    );
    assert_eq!(questions(&empty), Reading::Known(0));

    let known = Screen::initial().after_read(&body(Reading::Known(fixture_rows())), 20);
    for fetched in [
        Fetched::NotRead,
        Fetched::Failed,
        body(Reading::Unknown),
        Fetched::Body("not json".to_string()),
    ] {
        let screen = known.after_read(&fetched, 30);
        assert_eq!(screen.board, Reading::Unknown, "{fetched:?}");
        assert_eq!(questions(&screen), Reading::Unknown);
        // 問いの block も同じ読みを 0 件でなく測れていないと数える。
        assert_eq!(ask::count(&fetched), Reading::Unknown);
        assert_eq!(project::ledger::count(&screen), Reading::Unknown);
        // 最終更新は最後に読めた時刻のまま（読めなかった時刻に進めない）。
        assert_eq!(screen.updated_at, Some(20));
    }
    let lost = known.after_lost();
    assert_eq!(lost.board, Reading::Unknown);
    assert_eq!(lost.updated_at, Some(20));

    let start = Screen::initial();
    assert_eq!(start.board, Reading::Unknown);
    assert_eq!(questions(&start), Reading::Unknown);
    assert_eq!(start.updated_at, None);
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
fn board_min_clock_is_utc() {
    assert_eq!(clock(0), "1970-01-01 00:00:00 UTC");
    assert_eq!(clock(1_790_494_740), "2026-09-27 07:39:00 UTC");
    assert_eq!(clock(1_709_208_000), "2024-02-29 12:00:00 UTC");
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
