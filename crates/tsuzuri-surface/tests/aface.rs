//! 読むだけの server の面の歯（接頭辞 aface_・設計ノート surface-wave19h 行 e-ask-own-only）。
//! 答えを受けるかの読みと語は host で試し、DOM は wasm の target のときだけなので、file の字で付け方を見る。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::question::QuestionList;
use tsuzuri_contract::wire;
use tsuzuri_surface::project::ask::{self, CHAT_KEY};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab;

fn read(rel: &str) -> String {
    std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel))
        .unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// fixture の問いの一覧の欄 answerable を `answerable` にした本文。
fn listed(answerable: bool) -> Fetched {
    let list: QuestionList = wire::decode(&read("../../tests/fixtures/surface/question-list.json"))
        .expect("fixture の問いの一覧が電文として読める");
    Fetched::Body(wire::encode(&QuestionList { answerable, ..list }).expect("電文の字にできる"))
}

/// (3) 電文の answerable が偽なら偽・真と欄の無い電文とまだ読んでいないと読めないは真。
#[test]
fn aface_answerable_from_wire() {
    assert!(!ask::answerable(&listed(false)));
    assert!(ask::answerable(&listed(true)));

    // 欄 answerable の無い電文（前の server）は答えを受ける。
    let bare = r#"{"cards":{"known":[]}}"#;
    let list: QuestionList = wire::decode(bare).expect("欄の無い電文が読める");
    assert!(list.answerable);
    assert_eq!(list.cards, Reading::Known(vec![]));
    assert!(ask::answerable(&Fetched::Body(bare.to_string())));
    assert!(ask::answerable(&Fetched::Body(
        r#"{"cards":"unknown"}"#.to_string()
    )));

    assert!(ask::answerable(&Fetched::NotRead));
    assert!(ask::answerable(&Fetched::Failed));

    // 読むだけの電文でも card の読みは替わらない。
    assert_eq!(ask::cards(&listed(false)), ask::cards(&listed(true)));
}

/// (4) チャットで答える 1 行の語の鍵と見出しの語。
#[test]
fn aface_chat_line_word() {
    assert_eq!(CHAT_KEY, "answer_in_chat");
    let term = vocab::vocab().term(CHAT_KEY).expect("鍵 answer_in_chat");
    assert_eq!(term.label, "チャットで答える");
    assert_eq!(vocab::label(CHAT_KEY), "チャットで答える");
}

/// `start` から末尾までの字。
fn after<'a>(text: &'a str, start: &str) -> &'a str {
    let at = text
        .find(start)
        .unwrap_or_else(|| panic!("字 {start} が無い"));
    &text[at..]
}

/// (5) ask.rs の mod dom は答えを受けるかを context で置いて答えの欄で読み、batch.rs の mod dom は Memo で読む。
#[test]
fn aface_dom_wiring() {
    let ask_text = read("src/project/ask.rs");
    let dom = after(&ask_text, "mod dom {");
    for want in [
        "provide_context(CanAnswer(Memo::new(move |_| fetched.with(answerable))));",
        "move || answerable() && outcome.with(",
        "(!answerable()).then(|| view! { <div class=",
        "data-term=CHAT_KEY>{label(CHAT_KEY)}</div> })",
    ] {
        assert!(dom.contains(want), "ask.rs の mod dom に {want} が無い");
    }

    let batch_text = read("src/project/batch.rs");
    let dom = after(&batch_text, "mod dom {");
    for want in [
        "let can_answer = Memo::new(move |_| fetched.with(ask::answerable));",
        "Body::Filled(()) if !can_answer.get() => {",
    ] {
        assert!(dom.contains(want), "batch.rs の mod dom に {want} が無い");
    }
}
