//! 個別の頁の留め置きの段の歯（面・接頭辞 hdpage_・設計ノート surface-wave28a 行 g-held-page・判断の記録 ADR-42 決定 (7)）。
//! 中心の bead の札が段 Held の時だけ、頭と概要の下に吹き出しと同じ 4 つの欄を 1 段で出す（欄は pop::held_facts の 1 本）。
//! 札が無いかほかの段の bead は出さない。否定の見本は 1 欄だけ替える。
#![cfg(test)]

use crate::common::{AT, NOW, WHY, read};
use tsuzuri_contract::board::{PipelineCard, Reading, Stage};
use tsuzuri_contract::case::{CaseLinks, CasePart};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_surface::project::node::{HELD_CLASS, HELD_HEAD, held_rows, held_text};
use tsuzuri_surface::vocab::{label, vocab};
use tsuzuri_surface::widgets::keyline::HOLD;
use tsuzuri_surface::widgets::pop::{
    HELD_KEYS, Src, UNKNOWN_KEY, Val, at_text, held_facts, stage_facts,
};

fn card(stage: Stage) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new("h-1").unwrap_or_else(|e| panic!("{e:?}")),
        runs: 0,
        stage,
        reason: Some(HOLD.to_string()),
        account: None,
        since: Some(AT),
        ci: None,
    }
}

fn parts() -> Vec<CasePart> {
    vec![CasePart {
        part: "contract".to_string(),
        id: "h-1".to_string(),
        phase: "contract-queued".to_string(),
        turn: "seat".to_string(),
        since: Some(AT),
        reason: Some(HOLD.to_string()),
        why: Some(WHY.to_string()),
        closed: false,
        links: CaseLinks::default(),
    }]
}

fn src<'a>(cards: &'a [PipelineCard], parts: &'a [CasePart]) -> Src<'a> {
    Src {
        facts: Reading::Unknown,
        rows: Reading::Unknown,
        cards,
        parts: Reading::Known(parts),
        graph: None,
    }
}

#[test]
fn hdpage_held_rows_same_as_pop() {
    let cards = [card(Stage::Held)];
    let parts = parts();
    let s = src(&cards, &parts);
    let rows = held_rows("h-1", &s).unwrap_or_else(|| panic!("留め置きの段が無い"));
    assert_eq!(rows, held_facts(&s, &cards[0]));
    assert_eq!(rows, stage_facts(&s, &cards[0]), "吹き出しと同じ欄");
    let keys: Vec<&str> = rows.iter().map(|f| f.key).collect();
    assert_eq!(keys, HELD_KEYS);
    assert_eq!(rows[1].val, Val::Text(WHY.to_string()));
}

#[test]
fn hdpage_not_held_no_section() {
    let parts = parts();
    for stage in [Stage::Queued, Stage::Blocked] {
        let cards = [card(stage)];
        assert_eq!(
            held_rows("h-1", &src(&cards, &parts)),
            None,
            "段だけ替えた見本 {stage:?}"
        );
    }
    let cards = [card(Stage::Held)];
    assert_eq!(
        held_rows("h-2", &src(&cards, &parts)),
        None,
        "札の無い bead"
    );
}

#[test]
fn hdpage_held_text_values() {
    assert_eq!(held_text(&Val::Text(WHY.to_string()), NOW), WHY);
    assert_eq!(held_text(&Val::At(AT), NOW), at_text(AT, NOW));
    assert_eq!(held_text(&Val::Unknown, NOW), label(UNKNOWN_KEY));
}

#[test]
fn hdpage_dom_and_words() {
    assert_eq!((HELD_HEAD, HELD_CLASS), ("nb_held", "nheld"));
    let t = vocab()
        .term(HELD_HEAD)
        .unwrap_or_else(|| panic!("nb_held が語の辞書に無い"));
    assert_eq!(t.label, "留め置き");
    assert!(read("style.css").contains(".nheld {"));
    let src = read("src/project/node.rs");
    assert!(src.contains("let held = held_of(pipe, cases, &h.id);"));
    assert!(src.contains("        held_rows(id, &src).map(held_view)\n"));
    assert!(
        src.contains(
            "view! { {head_view(h, mode, revoke)}<div class=\"nsum\">{boxes}</div>{held} }"
        )
    );
    assert!(src.contains("<td>{held_text(&f.val, now)}</td>"));
}
