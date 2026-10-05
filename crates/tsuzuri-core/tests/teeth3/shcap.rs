//! 短い題の字数の上限の歯（接頭辞 shcap_・設計ノート surface-wave27b 行 c-short-cap・判断の記録 ADR-30 決定 (5)・規則の行 R-19）。
//! 台帳の字は歯の中の JSON の配列（bd の読み取りの口の形）。字数は Unicode のスカラー値で数える（全角の字は 3 byte）。
#![cfg(test)]

use serde_json::{Value, json};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadFact;
use tsuzuri_core::ledger::facts::facts;

/// 全角の字を n 字（字は 1 から 9 の全角の数字を繰り返す）。
fn wide(n: usize) -> String {
    "１２３４５６７８９".chars().cycle().take(n).collect()
}

fn bead(id: &str, title: &str, metadata: Value) -> Value {
    json!({"id": id, "title": title, "status": "open", "issue_type": "task", "metadata": metadata})
}

fn rows(beads: Vec<Value>) -> Vec<BeadFact> {
    match facts(&Value::Array(beads).to_string()).rows {
        Reading::Known(rows) => rows,
        Reading::Unknown => panic!("読める台帳が Unknown"),
    }
}

fn short_of(rows: &[BeadFact], id: &str) -> (String, bool) {
    let f = rows
        .iter()
        .find(|f| f.id.as_str() == id)
        .unwrap_or_else(|| panic!("{id} が無い"));
    (f.short.clone(), f.short_set)
}

#[test]
fn shcap_machine_cut() {
    let rows = rows(vec![
        bead("fx-c.1", &format!("{} — 残りの題", wide(30)), json!({})),
        bead("fx-c.2", &format!("便 {} の行", "a".repeat(25)), json!({})),
    ]);
    let cut = format!("{}…", wide(19));
    assert_eq!(cut.chars().count(), 20);
    assert_eq!(short_of(&rows, "fx-c.1"), (cut, false));
    assert_eq!(
        short_of(&rows, "fx-c.2"),
        (format!("{}…", "a".repeat(19)), false)
    );
}

#[test]
fn shcap_twenty_kept() {
    let rows = rows(vec![
        bead("fx-c.1", &format!("{} — 残り", wide(20)), json!({})),
        bead("fx-c.2", &format!("{} — 残り", wide(21)), json!({})),
        bead("fx-c.3", &"a".repeat(20), json!({})),
    ]);
    assert!(wide(20).len() > 20, "byte でなく字で数える見本");
    assert_eq!(short_of(&rows, "fx-c.1"), (wide(20), false));
    assert_eq!(short_of(&rows, "fx-c.2"), (format!("{}…", wide(19)), false));
    assert_eq!(short_of(&rows, "fx-c.3"), ("a".repeat(20), false));
}

#[test]
fn shcap_set_cut() {
    let rows = rows(vec![
        bead("fx-c.1", "題 — 残り", json!({"short": wide(25)})),
        bead("fx-c.2", "題 — 残り", json!({"short": wide(20)})),
    ]);
    assert_eq!(short_of(&rows, "fx-c.1"), (format!("{}…", wide(19)), true));
    assert_eq!(short_of(&rows, "fx-c.2"), (wide(20), true));
}
