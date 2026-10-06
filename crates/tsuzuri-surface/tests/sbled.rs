//! 概要の箱の台帳の字の印の歯（接頭辞 sbled_）。
//! 本文の頭の 1 行の箱だけが台帳の字の置き場の印を持つこと（真・偽・偽）と、node.rs の描く側の配線の字を測る。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::project::node::{SumBox, ledger_box};
use tsuzuri_surface::widgets::sumpick::{BODY_KEY, ENG_KEY, PLAIN_KEY};

fn read(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn boxed(key: &'static str) -> SumBox {
    SumBox {
        key,
        class: "sumbox",
        text: "在りか。".to_string(),
        other: None,
    }
}

#[test]
fn sbled_only_body_line() {
    assert!(ledger_box(&boxed(BODY_KEY)));
    assert!(!ledger_box(&boxed(PLAIN_KEY)));
    assert!(!ledger_box(&boxed(ENG_KEY)));
}

#[test]
fn sbled_box_wiring() {
    let src = read("src/project/node.rs");
    for needle in [
        "<p data-t=t data-ledger-text=led>{b.text}</p>",
        "let led = ledger_box(&b)",
    ] {
        assert_eq!(src.matches(needle).count(), 1, "node.rs の字 {needle}");
    }
}
