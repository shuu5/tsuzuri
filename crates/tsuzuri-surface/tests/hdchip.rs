//! 行 h-dormant の歯: account board の header の休止中の chip と card（見本の account/index.html の dormant:all の枝）と、
//! board が chip を最終の記録の chip の直後・読みの脈の前に置く字の並び（拠る要件 FR12・持ち主の裁定 t3-hub.53.10）。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::account::{AccountDoc, DormantSeat};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::{
    DORMANT_CARD, DORMANT_CLASS, DORMANT_KEY, DORMANT_KIND, DORMANT_NAME_CHARS, DORMANT_SRC,
    DORMANT_STATE, DORMANT_VALUE_CHARS, dormant_card, dormant_chip,
};
use tsuzuri_surface::project::seat::hmd;
use tsuzuri_surface::vocab::label;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture() -> AccountDoc {
    wire::decode(&read("../../tests/fixtures/account/acct-doc.json"))
        .expect("fixture が AccountDoc として読める")
}

fn seat(
    target: &str,
    account: Option<&str>,
    last: EpochSecs,
    tick: Reading<bool>,
    hb: Reading<bool>,
) -> DormantSeat {
    DormantSeat {
        project: String::new(),
        target: target.to_string(),
        account: account.map(str::to_string),
        last,
        tick_healthy: tick,
        heartbeat: hb,
    }
}

/// 関数の本文（`fn name(` から次の行頭の `fn `・`pub fn ` まで）。
fn function<'a>(text: &'a str, name: &str) -> &'a str {
    let start = text
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("fn {name} が在る"));
    let rest = &text[start..];
    let end = ["\nfn ", "\npub fn "]
        .iter()
        .filter_map(|m| rest[1..].find(m).map(|i| i + 1))
        .min()
        .unwrap_or(rest.len());
    &rest[..end]
}

/// `text` で最初に出る `word` の位置。
fn at(text: &str, word: &str) -> usize {
    text.find(word)
        .unwrap_or_else(|| panic!("{word} が無い: {text}"))
}

/// (1) 休止中の席が無い電文（fixture の dormant は空の列）では chip も card も出さない。
#[test]
fn hdchip_none_without_resting_seats() {
    let doc = fixture();
    assert!(doc.dormant.is_empty(), "fixture の dormant が空でない");
    assert_eq!(dormant_chip(&doc), None);
    assert_eq!(dormant_card(&doc), None);
}

/// (2) 3 つの席の chip と card の字（見本の dormant:all の枝）と、1 つの席なら値は切らない。
#[test]
fn hdchip_chip_and_card_follow_mock() {
    assert_eq!(DORMANT_KEY, "dormant");
    assert_eq!(DORMANT_CLASS, "chip num dormant");
    assert_eq!(DORMANT_CARD, "dormant:all");
    assert_eq!(DORMANT_STATE, "wait");
    assert_eq!(DORMANT_KIND, "12 時間以上 動きなし");
    assert_eq!(DORMANT_SRC, "seat/<seat>/state.jsonl の最後");
    assert_eq!(DORMANT_VALUE_CHARS, 34);
    assert_eq!(DORMANT_NAME_CHARS, 18);
    assert_eq!(label("dormant"), "休止中の session");

    three_seats_chip_and_card();
}

/// 3 つの席の chip と card の字・1 つの席なら値は切らない。
fn three_seats_chip_and_card() {
    let mut doc = fixture();
    assert_eq!(doc.at, 1790510400);
    doc.dormant = vec![
        seat(
            "proj-a:1.1",
            Some("acct-2"),
            1790467199,
            Reading::Known(false),
            Reading::Known(false),
        ),
        seat(
            "a-very-long-seat-name-x:0.1",
            None,
            1790467300,
            Reading::Unknown,
            Reading::Unknown,
        ),
        seat(
            "proj-z:0.1",
            Some("acct-1"),
            1790400000,
            Reading::Known(true),
            Reading::Known(true),
        ),
    ];
    assert_eq!(dormant_chip(&doc).as_deref(), Some("休止中の session 3"));
    let card = dormant_card(&doc).expect("休止中の席が在れば card が在る");
    assert_eq!(card.title, "休止中の session 3");
    assert_eq!(card.kind, "12 時間以上 動きなし");
    assert_eq!(card.value, "proj-a:1.1 / a-very-long-seat-nam…");
    assert_eq!(card.value.chars().count(), 34);
    assert_eq!(card.src, "seat/<seat>/state.jsonl の最後");
    let when = |last: EpochSecs| hmd(last, doc.at);
    assert_eq!(
        card.more,
        vec![
            format!("proj-a:1.1 acct-2 ◷ {} · tick stale · hb off", when(1790467199)),
            format!("a-very-long-seat-… ? ◷ {} · tick ? · hb ?", when(1790467300)),
            format!("proj-z:0.1 acct-1 ◷ {} · tick healthy · hb on", when(1790400000)),
        ]
    );

    doc.dormant.drain(..2);
    assert_eq!(dormant_chip(&doc).as_deref(), Some("休止中の session 1"));
    let card = dormant_card(&doc).expect("1 つの席でも card が在る");
    assert_eq!(card.title, "休止中の session 1");
    assert_eq!(card.value, "proj-z:0.1");
}

/// (3) board の top が休止中の chip を最終の記録の chip の直後・読みの脈の前に 1 度だけ置き、
/// dormant_view が電文の読めるときだけ class と card の鍵と card と状態の記号を持つ span を出す。
#[test]
fn hdchip_board_places_chip_after_updated() {
    let board = read("src/account/board.rs");
    let words = [
        "{updated_chip()}",
        "{move || dormant_view(read)}",
        "{fresh::pulse()}",
    ];
    for w in words {
        assert_eq!(board.matches(w).count(), 1, "{w} が 1 度だけ在らない");
    }
    let places: Vec<usize> = words.iter().map(|w| at(&board, w)).collect();
    assert!(places.windows(2).all(|p| p[0] < p[1]), "{places:?}");

    let view = function(&board, "dormant_view");
    for w in [
        "dormant_chip(d)?",
        "dormant_card(d)?",
        "class=DORMANT_CLASS",
        "data-card=DORMANT_CARD",
        "tabindex",
        "use:attach=card",
        "project::state_icon(DORMANT_STATE)",
    ] {
        assert!(view.contains(w), "dormant_view に {w} が無い: {view}");
    }
    assert!(!board.contains("休止中の chip は出さない"));
}
