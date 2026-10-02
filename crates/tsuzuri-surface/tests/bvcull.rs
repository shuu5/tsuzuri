//! 行 g-dead-sweep-a の歯（接頭辞 bvcull_）: pipeline の札の hover の card の値の組みと、次の一手の block の DOM と
//! DOM だけが使った純粋な関数を歯ごと消した後の形（src の字と歯の file の有無で見る）。
#![cfg(test)]

use std::path::PathBuf;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// `text` の中の `decl` から、その後の最初の行頭の閉じ括弧までの字。
fn item<'a>(text: &'a str, decl: &str) -> &'a str {
    let at = text
        .find(decl)
        .unwrap_or_else(|| panic!("字 {decl} が無い"));
    let rest = &text[at..];
    let end = rest.find("\n}").map_or(rest.len(), |i| i + 2);
    &rest[..end]
}

/// pipeline.rs は札の hover の card の値の組み（欄 hover・run_line・run_more と関数 node_hover・with_nodes・card_href・chunk と
/// 定数 SOURCE・MORE_CHARS と hover の Card の use）を持たず、札の型 Kcard の欄は 11 でこの順、歯の file cadopt.rs が無い。
#[test]
fn bvcull_card_hover_values_gone() {
    let src = read("src/project/pipeline.rs");
    for gone in [
        "pub fn node_hover(",
        "pub fn with_nodes(",
        "pub fn card_href(",
        "fn chunk(",
        "pub const SOURCE",
        "const MORE_CHARS",
        "run_line",
        "run_more",
        "use crate::widgets::hover::Card;",
    ] {
        assert!(!src.contains(gone), "pipeline.rs に {gone} が在る");
    }
    let kcard = item(&src, "pub struct Kcard {");
    let fields: Vec<&str> = kcard
        .lines()
        .filter_map(|l| l.trim().strip_prefix("pub "))
        .filter(|l| !l.starts_with("struct "))
        .filter_map(|l| l.split(':').next())
        .collect();
    assert_eq!(
        fields,
        [
            "id", "title", "state", "lead", "age", "since", "class", "closed", "ci", "short",
            "line"
        ]
    );
    assert!(
        !crate_dir().join("tests/cadopt.rs").exists(),
        "tests/cadopt.rs が在る"
    );
}

/// next.rs は block の DOM（mod dom）と、DOM だけが使った関数と型（content・Next・next・row・Row・Mark・MISS・big_title・BigTitle・
/// action・window_of・Jump・jump・row_link・waited と link の字の定数 5 つ）を持たず、view は空の中身を返し、帯と account board が
/// 使う step・key・unjudged・big と、口の path と body は残り、歯の file nextact.rs・nact.rs・nxorg.rs が無い。
#[test]
fn bvcull_nx_block_dom_gone() {
    let src = read("src/project/next.rs");
    for gone in [
        "mod dom",
        "pub fn content(",
        "pub struct Next ",
        "pub fn next(",
        "pub fn row(",
        "pub struct Row ",
        "pub enum Mark ",
        "pub const MISS",
        "pub struct BigTitle",
        "pub fn big_title(",
        "pub fn action(",
        "pub fn window_of(",
        "pub enum Jump",
        "pub fn jump(",
        "pub fn row_link(",
        "pub fn waited(",
        "ACCOUNT_LINK",
        "SESSION_LINK",
        "BATCH_LINK",
        "ANSWER_LINK",
        "GAPS_LINK",
    ] {
        assert!(!src.contains(gone), "next.rs に {gone} が在る");
    }
    for kept in [
        "pub fn step(",
        "pub fn key(",
        "pub fn unjudged(",
        "pub fn big(",
        "pub fn body(",
        "pub const PATH: &str = \"/api/next\";",
    ] {
        assert!(src.contains(kept), "next.rs に {kept} が無い");
    }
    let view = item(&src, "pub fn view() -> leptos::prelude::AnyView {");
    assert!(view.contains("().into_any()"), "{view}");
    assert!(!view.contains("dom::"), "{view}");
    for rel in ["tests/nextact.rs", "tests/nact.rs", "tests/nxorg.rs"] {
        assert!(!crate_dir().join(rel).exists(), "{rel} が在る");
    }
}
