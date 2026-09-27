//! 問いの一覧の歯（便 e-ask・接頭辞 question_）。
//! fixture: tests/fixtures/surface/question-2.json（bd の一覧の形・open の問い 2 本と closed の問い 1 本）。
//! 要約値は契約の型の読み（`BdLine`）から組んだ `LedgerItem::digest` の値と比べる。

use std::path::Path;

use serde_json::{Value, json};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BdLine;
use tsuzuri_contract::question::QuestionCard;
use tsuzuri_contract::wire;
use tsuzuri_core::question::{list, open_questions, touches, typed};

const FIXTURE: &str = "tests/fixtures/surface/question-2.json";

fn ledger() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(FIXTURE);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{FIXTURE} を読む: {e}"))
}

fn cards(ledger: &str) -> Vec<QuestionCard> {
    match list(ledger).cards {
        Reading::Known(cards) => cards,
        Reading::Unknown => panic!("読める台帳が Unknown"),
    }
}

#[test]
fn question_open_two_oldest_first() {
    let cards = cards(&ledger());
    let ids: Vec<&str> = cards.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        ids,
        ["fx-ask.2", "fx-ask.3"],
        "closed を除き作られた時刻の古い順"
    );
    assert_eq!(cards[0].posted_at, 1_790_298_000, "2026-09-25T01:00:00Z");
    assert_eq!(
        cards[1].posted_at, 1_790_388_000,
        "作られた時刻（更新時刻でない）"
    );
    assert_eq!(cards[1].title, "問い — 定型行を持つ新しい方の問い");
}

#[test]
fn question_typed_lines_to_fields() {
    let cards = cards(&ledger());
    let with = &cards[1];
    assert_eq!(
        (
            with.plain.as_deref(),
            with.eng.as_deref(),
            with.reason.as_deref(),
            with.recommend.as_deref()
        ),
        (
            Some("画面の色を 2 つに減らしてよいか"),
            Some("style.css の変数を 5 から 2 へ"),
            Some("見分けの負担を減らす"),
            Some("減らす")
        ),
        "定型行の値（同じ定型の 2 行目は取らない）"
    );
    let without = &cards[0];
    assert_eq!(
        (
            &without.plain,
            &without.eng,
            &without.reason,
            &without.recommend
        ),
        (&None, &None, &None, &None),
        "定型行の無い欄は無し"
    );
}

#[test]
fn question_a1_and_touches() {
    let cards = cards(&ledger());
    assert!(cards[1].a1, "label が A-1 で始まる");
    assert!(!cards[0].a1);
    assert_eq!(cards[1].touches, ["FR6", "ADR-7"]);
    assert!(cards[0].touches.is_empty());
}

#[test]
fn question_digest_matches_contract() {
    let text = ledger();
    let lines: Vec<BdLine> = wire::decode(&text).expect("契約の型で読める");
    for card in cards(&text) {
        let line = lines
            .iter()
            .find(|l| l.id == card.id)
            .expect("台帳の bead")
            .clone();
        assert_eq!(card.digest, line.into_item(0).digest(), "{}", card.id);
        assert_eq!(card.digest.len(), 16);
    }
}

#[test]
fn question_notes_ride_along() {
    let Reading::Known(qs) = open_questions(&ledger()) else {
        panic!("読める台帳が Unknown");
    };
    let notes: Vec<&str> = qs.iter().map(|q| q.notes.as_str()).collect();
    assert_eq!(notes, ["", "見本の notes の 1 行"]);
}

#[test]
fn question_unreadable_is_unknown() {
    for bad in ["", "  ", "{", "not json", "{\"id\":\"x\"}"] {
        assert_eq!(list(bad).cards, Reading::Unknown, "{bad:?}");
    }
    // open の問いの id が bead の id の形でない台帳も Unknown。
    let bad_id = json!([{"id":"x y","status":"open","labels":["intake:question"]}]).to_string();
    assert_eq!(list(&bad_id).cards, Reading::Unknown);
    assert_eq!(list("[]").cards, Reading::Known(vec![]));
}

#[test]
fn question_same_time_by_id_and_kinds() {
    let at = "2026-09-27T00:00:00Z";
    let bead = |id: &str, status: &str, labels: Value| json!({"id": id, "title": id, "status": status, "created_at": at, "updated_at": at, "labels": labels});
    let ledger = Value::Array(vec![
        bead("q.b", "open", json!(["intake:question"])),
        bead("q.a", "blocked", json!(["intake:question"])),
        bead("q.c", "tombstone", json!(["intake:question"])),
        bead("m.1", "open", json!(["intake:memo"])),
        bead("t.1", "open", json!(null)),
    ])
    .to_string();
    let ids: Vec<String> = cards(&ledger)
        .into_iter()
        .map(|c| c.id.to_string())
        .collect();
    assert_eq!(
        ids,
        ["q.a", "q.b"],
        "同じ時刻は id の順・問いでない bead と tombstone は出さない"
    );
}

#[test]
fn question_typed_and_touches_shapes() {
    assert_eq!(
        typed("概要 = a\r\n概要 = b", "概要 = ").as_deref(),
        Some("a")
    );
    assert_eq!(typed(" 概要 = 行頭でない", "概要 = "), None);
    assert_eq!(typed("概要 = ", "概要 = "), None);
    assert_eq!(touches(&json!("{\"touches\":\"FR6\"}")), ["FR6"]);
    assert_eq!(touches(&json!({"touches": ["", "A-1"]})), ["A-1"]);
    assert!(touches(&json!(null)).is_empty());
}
