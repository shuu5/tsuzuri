//! 質問の窓の card の概要を表示の型の 1 行にする歯（行 g-ask-mode・接頭辞 qmode_・判断の記録 ADR-30 決定 (2)）。
//! card の 1 行は host の純な関数で撃ち、card の DOM（wasm の枝）と stylesheet は字を読む。
#![cfg(test)]

use crate::common::read;
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::ask::{self, SumLine};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::widgets::sumpick::{self, ENG_KEY, PLAIN_KEY};

const P: &str = "P-字 答えの欄を 1 つ置く";
const E: &str = "E-字 qcard ごとに textarea";

fn line(class: &'static str, key: &'static str, text: &str, marked: bool) -> SumLine {
    SumLine {
        class,
        key: Some(key),
        text: text.to_string(),
        eng: key == ENG_KEY,
        marked,
    }
}

/// (1) card は電文の 2 つの概要を字のまま持ち、概要の行は 1 つで、初心者は非エンジニア向けの字・経験者はエンジニア向けの字を印なしで出す。
#[test]
fn qmode_card_one_line() {
    let cards = ask::cards(&Fetched::Body(read(
        "../../tests/fixtures/surface/question-list.json",
    )))
    .expect("問いの一覧の fixture");
    let mut q = cards[0].clone();
    q.plain = Some(P.to_string());
    q.eng = Some(E.to_string());
    let card = ask::card(1, &q);
    assert_eq!(
        (card.plain.as_deref(), card.eng.as_deref()),
        (Some(P), Some(E))
    );
    let (plain, eng) = (card.plain.as_deref(), card.eng.as_deref());
    let b = ask::summary(plain, eng, Mode::Beginner);
    assert_eq!(b, line("ln plain", PLAIN_KEY, P, false));
    let e = ask::summary(plain, eng, Mode::Expert);
    assert_eq!(e, line("ln eng", ENG_KEY, E, false));
    assert!(!b.text.contains("E-字") && !e.text.contains("P-字"));
}

/// (2) 表示の型の側が無い（None か空）時はもう一方の 1 行を class の印 marked つきで、部品 sumpick の pick と同じ字と鍵で出し、
/// 両方無ければ語の鍵の無い 要約なし の 1 行（class ln plain muted）。
#[test]
fn qmode_card_fallback() {
    assert_eq!(
        ask::summary(None, Some(E), Mode::Beginner),
        line("ln eng marked", ENG_KEY, E, true)
    );
    assert_eq!(
        ask::summary(Some(P), Some(""), Mode::Expert),
        line("ln plain marked", PLAIN_KEY, P, true)
    );
    for mode in Mode::ALL {
        for (plain, eng) in [(Some(P), Some(E)), (None, Some(E)), (Some(P), None)] {
            let got = ask::summary(plain, eng, mode);
            let want = sumpick::pick(mode, plain, eng, None).expect("どちらかが在る");
            assert_eq!(
                (got.key, got.text, got.marked),
                (Some(want.key), want.text, want.marked)
            );
        }
        let none = ask::summary(None, None, mode);
        assert_eq!(
            none,
            SumLine {
                class: "ln plain muted",
                key: None,
                text: ask::NO_SUMMARY.to_string(),
                eng: false,
                marked: false,
            }
        );
    }
}

/// (3) card の DOM は概要の段で表示の型を渡して 1 行を組み、印の時だけ語の鍵の語を small で前に置き、
/// stylesheet は初心者の表示の型でエンジニア向けの行を隠す規則を持たない。
#[test]
fn qmode_dom_one_line() {
    let src = read("src/project/ask.rs");
    let dom = &src[src.find("mod dom {").expect("字 mod dom { が在る")..];
    for want in [
        "Part::Summary => summary_view(slot, card, mode),",
        "let one = summary(plain.as_deref(), eng.as_deref(), mode());",
        "l.key.filter(|_| l.marked).map(|k| view! { <small>{label(k)}</small> })",
    ] {
        assert!(dom.contains(want), "ask.rs の DOM に {want} が無い");
    }
    assert!(
        !src.contains(".summary\n"),
        "ask.rs が card の 2 行の列を読む"
    );
    let css = read("style.css");
    assert!(
        !css.contains("body.mode-beginner .qsum .ln.eng"),
        "初心者でエンジニア向けの行を隠す規則が在る"
    );
    assert!(css.contains(".qsum .ln.marked small {"));
}
