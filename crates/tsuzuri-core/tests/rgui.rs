//! 配達の読みの歯（接頭辞 rgui_）。
//! 配達の読みが器の経路 gui の裁定の行（欄の区切り ` | ` の 5 欄）を読み、経路 chat の行を読まない事を見る。
#![cfg(test)]

use serde_json::{Value, json};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::surface::RulingId;
use tsuzuri_core::delivery::{GUI_ROUTE, Pending, Route, Said, mark_line, said, undelivered};

fn rid(id: &str) -> RulingId {
    RulingId::new(id).expect("裁定 id の形")
}

fn qid(id: &str) -> BeadId {
    BeadId::new(id).expect("bead id の形")
}

fn bead(id: &str, labels: &[&str], notes: &[String]) -> Value {
    json!({"id": id, "status": "open", "labels": labels, "notes": notes.join("\n")})
}

/// 器の結びの口の裁定の行（`<裁定 id> | <問い id> | <発話の ts> | <経路> | <逐語>`）。
fn bind(id: &str, question: &str, ts: &str, route: &str, verbatim: &str) -> String {
    format!("{id} | {question} | {ts} | {route} | {verbatim}")
}

/// 節の fixture の台帳（問い a1-g.1 と label の無い a1-g.2）。
fn ledger() -> String {
    let mark = mark_line(
        &rid("a1-g.1:20261006T0400Z-1"),
        Route::Deliver,
        "20261006T0401Z",
    );
    let notes = [
        "裁定 id = a1-g.1:20261006T0100Z-1・問い = a1-g.1・逐語 = はい".to_string(),
        bind(
            "a1-g.1:20261006T0200Z-1",
            "a1-g.1",
            "2026-10-06T02:00:00.000Z",
            "gui",
            "\"よい | それで\\nはい\"",
        ),
        bind(
            "a1-g.1:20261006T0300Z-1",
            "a1-g.1",
            "2026-10-06T03:00:00.000Z",
            "chat",
            "\"会話\"",
        ),
        bind(
            "a1-g.1:20261006T0400Z-1",
            "a1-g.1",
            "2026-10-06T04:00:00.000Z",
            "gui",
            "\"済み\"",
        ),
        mark,
        bind(
            "a1-g.9:20261006T0500Z-1",
            "a1-g.9",
            "2026-10-06T05:00:00.000Z",
            "gui",
            "\"他の問い\"",
        ),
    ];
    let other = [bind(
        "a1-g.2:20261006T0600Z-1",
        "a1-g.2",
        "2026-10-06T06:00:00.000Z",
        "gui",
        "\"問いでない\"",
    )];
    Value::Array(vec![
        bead("a1-g.1", &["intake:question"], &notes),
        bead("a1-g.2", &[], &other),
    ])
    .to_string()
}

#[test]
fn rgui_gui_route_word() {
    assert_eq!(GUI_ROUTE, "gui");
}

#[test]
fn rgui_undelivered_reads_gui_bind_lines() {
    let want = vec![
        Pending {
            question: qid("a1-g.1"),
            ruling: rid("a1-g.1:20261006T0100Z-1"),
        },
        Pending {
            question: qid("a1-g.1"),
            ruling: rid("a1-g.1:20261006T0200Z-1"),
        },
    ];
    match undelivered(&ledger()) {
        Reading::Known(got) => assert_eq!(got, want),
        Reading::Unknown => panic!("台帳が読めない"),
    }
}

#[test]
fn rgui_said_unquotes_the_gui_verbatim() {
    let ids = [
        rid("a1-g.1:20261006T0200Z-1"),
        rid("a1-g.1:20261006T0300Z-1"),
    ];
    let want = vec![Said {
        question: qid("a1-g.1"),
        ruling: rid("a1-g.1:20261006T0200Z-1"),
        verbatim: "よい | それで\nはい".to_string(),
    }];
    match said(&ledger(), &ids) {
        Reading::Known(got) => assert_eq!(got, want),
        Reading::Unknown => panic!("台帳が読めない"),
    }
    match said(&ledger(), &[rid("a1-g.1:20261006T0300Z-1")]) {
        Reading::Known(got) => assert!(got.is_empty(), "経路 chat の行は読まない: {got:?}"),
        Reading::Unknown => panic!("台帳が読めない"),
    }
}
