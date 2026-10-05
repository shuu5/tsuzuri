//! 留め置きの札と列の見出しの歯（面・接頭辞 hdcol_・設計ノート surface-wave28a 行 g-held-col・判断の記録 ADR-42 決定 (3)(7)）。
//! 段 Held の札は Blocked の列に出て、理由の語 hold は席の止め・ほかは受付の断りの印を持つ（Blocked の札は印なし）。
//! Blocked の列の見出しの数は Blocked と Held の内訳、札の要の 1 行は印の語と止めてからの経過。否定の見本は 1 欄だけ替える。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{PipelineCard, PipelineColumn, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_surface::project::pipeline::{NO_AGE, columns, count_text, kcard};
use tsuzuri_surface::vocab::{label, vocab};
use tsuzuri_surface::widgets::keyline::{
    HELD_MARKS, HOLD, HeldBy, LINE_KEYS, LineSrc, held_by, key_line,
};

const NOW: EpochSecs = 1_790_000_000;

/// 受付の断りの名。
const NAME: &str = "teeth-outside-write-set";

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn card(x: &str, stage: Stage, reason: Option<&str>, since: Option<EpochSecs>) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(x).unwrap_or_else(|e| panic!("{x}: {e:?}")),
        runs: 0,
        stage,
        reason: reason.map(str::to_string),
        account: None,
        since,
        ci: None,
    }
}

fn line(c: &PipelineCard) -> String {
    let src = LineSrc {
        card: c.clone(),
        wait: None,
        created: None,
    };
    key_line(&src, NOW).text
}

#[test]
fn hdcol_held_by_reason() {
    let seat = card("h-1", Stage::Held, Some(HOLD), Some(NOW));
    let intake = card("h-2", Stage::Held, Some(NAME), Some(NOW));
    assert_eq!(HOLD, "hold");
    assert_eq!(held_by(&seat), Some(HeldBy::Seat));
    assert_eq!(held_by(&intake), Some(HeldBy::Intake));
    for stage in [Stage::Queued, Stage::Blocked] {
        let other = PipelineCard {
            stage,
            ..seat.clone()
        };
        assert_eq!(held_by(&other), None, "段だけ替えた見本 {stage:?}");
    }
}

#[test]
fn hdcol_card_mark_in_blocked_column() {
    let cards = [
        card("h-1", Stage::Held, Some(HOLD), Some(NOW - 60)),
        card("h-2", Stage::Held, Some(NAME), Some(NOW - 120)),
        card("b-1", Stage::Blocked, Some("dependency"), Some(NOW - 180)),
    ];
    let cols = columns(&cards, &[], NOW);
    let block = cols
        .iter()
        .find(|c| c.lane.column == PipelineColumn::Blocked)
        .unwrap_or_else(|| panic!("Blocked の列が無い"));
    let marks: Vec<(&str, Option<HeldBy>)> = block
        .cards
        .iter()
        .map(|k| (k.id.as_str(), k.held))
        .collect();
    assert_eq!(
        marks,
        [
            ("h-1", Some(HeldBy::Seat)),
            ("h-2", Some(HeldBy::Intake)),
            ("b-1", None)
        ]
    );
    assert_eq!(kcard(&cards[0], &[], NOW).held, Some(HeldBy::Seat));
    assert_eq!(
        kcard(&cards[2], &[], NOW).held,
        None,
        "Blocked の札は印なし"
    );
}

#[test]
fn hdcol_count_breakdown() {
    let cards = [
        card("h-1", Stage::Held, Some(HOLD), Some(NOW - 60)),
        card("b-1", Stage::Blocked, Some("dependency"), Some(NOW - 120)),
        card("b-2", Stage::Blocked, Some("overlap"), Some(NOW - 180)),
        card("q-1", Stage::Queued, Some(HOLD), Some(NOW - 240)),
    ];
    let cols = columns(&cards, &[], NOW);
    let text = |column: PipelineColumn| {
        let col = cols
            .iter()
            .find(|c| c.lane.column == column)
            .unwrap_or_else(|| panic!("{column:?} の列が無い"));
        count_text(col)
    };
    assert_eq!(
        text(PipelineColumn::Blocked),
        "2·1",
        "内訳は中黒でつなぎ空白を挟まない"
    );
    assert_eq!(text(PipelineColumn::Queued), "1", "ほかの列は札の数");
    let none = columns(&cards[1..], &[], NOW);
    assert_eq!(count_text(&none[0]), "2·0", "Held の札の無い Blocked の列");
}

#[test]
fn hdcol_key_line_mark_and_age() {
    let seat = card("h-1", Stage::Held, Some(HOLD), Some(NOW - 7_200));
    let intake = card("h-2", Stage::Held, Some(NAME), Some(NOW - 300));
    assert_eq!(line(&seat), format!("{} · 2h", label("hm:seat")));
    assert_eq!(line(&intake), format!("{} · 5m", label("hm:intake")));
    let unknown = PipelineCard {
        since: None,
        ..seat.clone()
    };
    assert_eq!(line(&unknown), format!("{} · {NO_AGE}", label("hm:seat")));
    let queued = PipelineCard {
        stage: Stage::Queued,
        ..seat
    };
    assert!(
        line(&queued).contains(&label(LINE_KEYS[2])),
        "段だけ替えた見本: {}",
        line(&queued)
    );
}

#[test]
fn hdcol_vocab_and_marks() {
    let v = vocab();
    let term = |k: &str| v.term(k).unwrap_or_else(|| panic!("{k} が語の辞書に無い"));
    assert_eq!(term("col_block").label, "Blocked / Held");
    assert!(term("col_block").note.contains("Held = 留め置き"));
    assert_eq!(
        HELD_MARKS,
        [
            (HeldBy::Seat, "hm:seat", "hdm hd-seat"),
            (HeldBy::Intake, "hm:intake", "hdm hd-intake")
        ]
    );
    for (by, key, class) in HELD_MARKS {
        assert_eq!((by.key(), by.class()), (key, class));
        let t = term(key);
        assert!(!t.label.is_ascii() && !t.note.is_empty(), "{key}");
    }
    assert_ne!(term("hm:seat").label, term("hm:intake").label);
}

#[test]
fn hdcol_css_and_dom_text() {
    let css = read("style.css");
    for rule in [
        ".kcard .m .hdm {",
        ".kcard .m .hd-seat {",
        ".kcard .m .hd-intake {",
    ] {
        assert!(css.contains(rule), "stylesheet に {rule} が無い");
    }
    let src = read("src/project/pipeline.rs");
    assert!(src.contains("let count = count_text(&col);"));
    assert!(src.contains(".map(|h| view! { <span class=h.class()>{label(h.key())}</span> });"));
    assert!(src.contains("{lead}\n                    {held}\n                    {closed}"));
}
