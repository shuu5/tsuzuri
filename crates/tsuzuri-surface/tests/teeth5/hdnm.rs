//! 受付の断りの名の平易な字の歯（面・接頭辞 hdnm_・設計ノート surface-wave28a 行 g-held-name・判断の記録 ADR-42 決定 (7)）。
//! 器の受付の断りの名 27 語は語の辞書の鍵 rf:<名> の平易な字を持ち、留め置きの吹き出しの理由はその字で始まる。
//! 名の : の後の詳細は外して引き、表に無い名は名のまま、名が無ければまだ分からない。否定の見本は 1 欄だけ替える。
#![cfg(test)]

use tsuzuri_contract::board::{PipelineCard, Reading, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_surface::vocab::{label, vocab};
use tsuzuri_surface::widgets::pop::{
    HELD_KEYS, REFUSAL_HEAD, REFUSALS, Src, UNKNOWN_KEY, Val, held_facts, refusal_text,
};

/// 器の pipe/refuse.rs の REFUSALS の写し（宣言の順・歯が字で持つ）。
const WANT: [&str; 27] = [
    "not-a-repo",
    "duplicate-run",
    "write-set-overlap",
    "write-set-unreadable",
    "write-set-incomplete",
    "write-set-dir-without-slash",
    "contract-table",
    "write-set-item-unresolved",
    "cap-headroom",
    "name-unresolved",
    "write-set-drift",
    "teeth-place-unresolved",
    "also-names-rust",
    "tests-not-a-teeth-file",
    "fn-undeclared",
    "teeth-outside-write-set",
    "hand-written-contract",
    "same-kind-repeated",
    "finding-unaddressed",
    "promised-field-written",
    "promise-symbol-unresolved",
    "max-live",
    "entrance-not-red",
    "ruling-unresolved",
    "index-building",
    "code-facts",
    "code-facts-unmeasured",
];

#[test]
fn hdnm_refusal_names_in_vocab() {
    assert_eq!(REFUSALS, WANT);
    assert_eq!(REFUSAL_HEAD, "rf:");
    let mut seen = Vec::new();
    for name in REFUSALS {
        let key = format!("rf:{name}");
        let t = vocab()
            .term(&key)
            .unwrap_or_else(|| panic!("{key} が語の辞書に無い"));
        assert!(!t.label.is_ascii() && !t.note.is_empty(), "{key}");
        assert!(!seen.contains(&t.label), "{key} の字が重なる");
        seen.push(t.label.clone());
    }
}

#[test]
fn hdnm_refusal_text_plain() {
    let plain = label("rf:teeth-outside-write-set");
    assert_eq!(refusal_text(Some("teeth-outside-write-set")), plain);
    assert_eq!(
        refusal_text(Some("contract-table:verify")),
        label("rf:contract-table"),
        "詳細つきの名"
    );
    assert_eq!(
        refusal_text(Some("teeth-outside-write-sets")),
        "teeth-outside-write-sets",
        "表に無い名は名のまま"
    );
    assert_eq!(refusal_text(None), label(UNKNOWN_KEY));
}

#[test]
fn hdnm_pop_reason_starts_plain() {
    let card = PipelineCard {
        contract: BeadId::new("h-1").unwrap_or_else(|e| panic!("{e:?}")),
        runs: 0,
        stage: Stage::Held,
        reason: Some("cap-headroom".to_string()),
        account: None,
        since: None,
        ci: None,
    };
    let src = Src {
        facts: Reading::Unknown,
        rows: Reading::Unknown,
        cards: std::slice::from_ref(&card),
        parts: Reading::Unknown,
        graph: None,
    };
    let facts = held_facts(&src, &card);
    let why = facts
        .iter()
        .find(|f| f.key == HELD_KEYS[1])
        .unwrap_or_else(|| panic!("理由の欄が無い"));
    let want = format!("{}：{}", label("rf:cap-headroom"), label(UNKNOWN_KEY));
    assert_eq!(why.val, Val::Text(want));
}
