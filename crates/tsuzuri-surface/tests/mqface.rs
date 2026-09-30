//! ほかの project の問いを問いの頁の一覧に混ぜる面の歯（接頭辞 mqface_・設計ノート surface-wave22c 行 e-multi-ask）。
//! 自分の問いは見本の surface/question-list.json（qa.2 の投稿の時刻 1790488800・qa.10 は 1790494200）で、qa.2 の card の
//! id と時刻を替えた fx-oth.2（1790490000）と fx-oth.4（1790494200）を札 proj-x の組に、Unknown を札 proj-y の組にする。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::question::{AllQuestions, ProjectQuestions, QuestionCard, QuestionList};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::{Body, ask, batch};
use tsuzuri_surface::view::Fetched;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn bare_text() -> String {
    read("../../tests/fixtures/surface/question-list.json")
}

fn own() -> QuestionList {
    wire::decode(&bare_text()).expect("見本の問いの一覧")
}

/// 見本の qa.2 の card の id と投稿の時刻を替えた card。
fn other_card(id: &str, posted_at: u64) -> QuestionCard {
    let Reading::Known(cards) = own().cards else {
        panic!("見本の card が Unknown");
    };
    let base = cards
        .into_iter()
        .find(|c| c.id.as_str() == "qa.2")
        .expect("qa.2 の card");
    QuestionCard {
        id: BeadId::new(id).expect("bead id"),
        posted_at,
        ..base
    }
}

/// 自分の問いに proj-x（fx-oth.2・fx-oth.4）と proj-y（Unknown）を足した電文。
fn mixed() -> Fetched {
    let all = AllQuestions {
        own: own(),
        others: vec![
            ProjectQuestions {
                project: "proj-x".into(),
                answerable: false,
                cards: Reading::Known(vec![
                    other_card("fx-oth.2", 1_790_490_000),
                    other_card("fx-oth.4", 1_790_494_200),
                ]),
            },
            ProjectQuestions {
                project: "proj-y".into(),
                answerable: false,
                cards: Reading::Unknown,
            },
        ],
    };
    Fetched::Body(wire::encode(&all).expect("全部の問いの一覧の電文"))
}

/// (6) body は番号 1 から 4 に qa.2・fx-oth.2・qa.10・fx-oth.4 の順で札と answerable を置き、unknown_projects は proj-y、
/// total は Known の 4、count は Known の 2、cards の id は qa.2・qa.10、batch::body は 2 行。
#[test]
fn mqface_mixes_by_posted_time() {
    let fetched = mixed();
    let Body::Filled(cards) = ask::body(&fetched) else {
        panic!("混ぜた一覧が中身にならない");
    };
    let got: Vec<(usize, &str, Option<&str>, bool)> = cards
        .iter()
        .map(|c| (c.number, c.id.as_str(), c.project.as_deref(), c.answerable))
        .collect();
    assert_eq!(
        got,
        [
            (1, "qa.2", None, true),
            (2, "fx-oth.2", Some("proj-x"), false),
            (3, "qa.10", None, true),
            (4, "fx-oth.4", Some("proj-x"), false),
        ]
    );
    assert_eq!(ask::unknown_projects(&fetched), ["proj-y"]);
    assert_eq!(ask::total(&fetched), Reading::Known(4));
    assert_eq!(ask::count(&fetched), Reading::Known(2));
    let own_ids: Vec<String> = ask::cards(&fetched)
        .expect("自分の問い")
        .iter()
        .map(|c| c.id.to_string())
        .collect();
    assert_eq!(own_ids, ["qa.2", "qa.10"]);
    let Body::Filled(rows) = batch::body(&fetched) else {
        panic!("束の block が中身にならない");
    };
    assert_eq!(rows.len(), 2);
}

/// (7) 鍵 others の無い見本の body は fn card で組んだ札なし・answerable 真の card の列と一致し、unknown_projects は空で、
/// total は count と一致する。
#[test]
fn mqface_bare_list_unchanged() {
    let fetched = Fetched::Body(bare_text());
    let Reading::Known(cards) = own().cards else {
        panic!("見本の card が Unknown");
    };
    let want: Vec<ask::Card> = cards
        .iter()
        .enumerate()
        .map(|(i, q)| ask::card(i + 1, q))
        .collect();
    assert!(want.iter().all(|c| c.project.is_none() && c.answerable));
    assert_eq!(ask::body(&fetched), Body::Filled(want));
    assert!(ask::unknown_projects(&fetched).is_empty());
    assert_eq!(ask::total(&fetched), ask::count(&fetched));
    for fetched in [Fetched::NotRead, Fetched::Failed] {
        assert!(ask::unknown_projects(&fetched).is_empty());
        assert_eq!(ask::total(&fetched), ask::count(&fetched));
    }
}

/// (8) ask.rs の mod dom は札・題の字・つながりの段・答えの欄・読めない組の行・数の chip の 8 つの字を持つ。
#[test]
fn mqface_dom_wiring() {
    let text = read("src/project/ask.rs");
    let (_, dom) = text.split_once("mod dom {").expect("ask.rs に mod dom が無い");
    for want in [
        "card.project.clone().map(|p|",
        "if other {",
        "return view! { <span data-t=",
        "Part::Around if card.project.is_some() => ().into_any(),",
        "let own_ok = card.answerable;",
        "let answerable = move || own_ok && can_answer.is_none_or(|m| m.get());",
        ".with(unknown_projects)",
        "let extra = move || match fetched.with(total) {",
    ] {
        assert!(dom.contains(want), "ask.rs の mod dom に {want} が無い");
    }
}
