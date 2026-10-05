//! 行 g-dead-sweep-b の歯（接頭辞 bvshed_）: 頁の枠の nav と header の部品の並びと頁の切り替え、画面の状態（Screen）の鎖、
//! 問いの頁の一覧の DOM と DOM だけが使った純粋な関数を歯ごと消した後の形（src の字と歯の file の有無で見る）。
#![cfg(test)]

use crate::common::{crate_dir, read};
use tsuzuri_surface::frame::{HeaderPart, UPDATED};

/// `text` の中の `decl` から、その後の最初の行頭の閉じ括弧までの字。
fn item<'a>(text: &'a str, decl: &str) -> &'a str {
    let at = text
        .find(decl)
        .unwrap_or_else(|| panic!("字 {decl} が無い"));
    let rest = &text[at..];
    let end = rest.find("\n}").map_or(rest.len(), |i| i + 2);
    &rest[..end]
}

/// 構造体の本文の欄の名の列: 注と属性の行を除き、`::` でない 1 つのコロンの前の名を拾う（`pub(crate)` などの可視の字や、
/// 1 行に並べた欄や、コロンの前の空白を問わない）。
fn fields(def: &str) -> Vec<&str> {
    let mut out = Vec::new();
    for line in def.lines().skip(1).map(str::trim) {
        if line.starts_with("//") || line.starts_with("#[") {
            continue;
        }
        let b = line.as_bytes();
        for (i, _) in line.match_indices(':') {
            let near = |j: Option<usize>| j.and_then(|j| b.get(j)) == Some(&b':');
            if near(i.checked_sub(1)) || near(Some(i + 1)) {
                continue;
            }
            let head = line[..i].trim_end();
            let start = head
                .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .map_or(0, |j| j + 1);
            out.push(&head[start..]);
        }
    }
    out
}

/// 名に使える字（英数字と _）の続きが `word` ちょうどである所の数（束縛の使い道を数える）。
fn uses(text: &str, word: &str) -> usize {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|w| *w == word)
        .count()
}

/// frame.rs は nav と header の部品の並びと頁の切り替え（nav・pages・nav_keys・HEADER・BACK・BACK_WRAP・NavLink・BADGE・
/// nav_links・badge・switch_url・header_snapshot・list）を可視の字によらず宣言せず、頁の定義 PageDef の欄は（可視の字によらず
/// 数えて）heading・class・columns の 3 つで、2 つの頁の定義は nav・badge・icon の欄を書かず、最終の記録の部品 UPDATED を
/// board.rs が引いてその鍵を帯の時刻の chip の語に渡し、header の snapshot の file が無い。
#[test]
fn bvshed_header_bits_gone() {
    let src = read("src/frame.rs");
    for gone in [
        "fn nav(",
        "fn pages(",
        "fn nav_keys(",
        "const HEADER:",
        "const BACK:",
        "BACK_WRAP",
        "struct NavLink",
        "const BADGE:",
        "fn nav_links(",
        "fn badge(",
        "fn switch_url(",
        "fn header_snapshot(",
        "fn list(",
    ] {
        assert!(!src.contains(gone), "frame.rs に {gone} が在る");
    }
    let def = item(&src, "pub struct PageDef {");
    assert_eq!(fields(def), ["heading", "class", "columns"], "{def}");
    for rel in ["src/pages/home.rs", "src/pages/node.rs"] {
        let page = read(rel);
        for gone in ["nav:", "badge:", "icon:"] {
            assert!(!page.contains(gone), "{rel} に {gone} が在る");
        }
    }
    assert_eq!(
        UPDATED,
        HeaderPart {
            part: "updated",
            key: "last_record",
            class: "chip num",
            items: &[],
        }
    );
    let board = read("src/board.rs");
    let chip = item(&board, "    let key = frame::UPDATED.key;");
    assert!(
        chip.contains("        Some(t) => term(key, clock_short(t, net::now())),"),
        "{chip}"
    );
    assert_eq!(
        uses(chip, "key"),
        3,
        "鍵の束縛 key は帯の時刻の chip の語の 1 所だけに渡す"
    );
    assert!(
        chip.contains("<span class=\"chip num\" title=title>{at}</span>"),
        "{chip}"
    );
    assert!(!board.contains("HEADER"));
    assert!(!crate_dir().join("tests/snapshots/header.json").exists());
}

/// view.rs は画面の状態の鎖（Screen・Board・EpicGroup・board・questions・ledger_groups と助けの ancestors・parent・
/// sort_children・kind_rank）を可視の字によらず宣言せず、board.rs は画面の状態を作らず台帳の一覧の口（ledger の PATH）を
/// 読まず、歯の file lgrp.rs が無い。
#[test]
fn bvshed_screen_chain_gone() {
    let view = read("src/view.rs");
    for gone in [
        "struct Screen",
        "struct Board",
        "struct EpicGroup",
        "fn board(",
        "fn questions(",
        "fn ledger_groups(",
        "fn ancestors(",
        "fn parent(",
        "fn sort_children(",
        "fn kind_rank(",
    ] {
        assert!(!view.contains(gone), "view.rs に {gone} が在る");
    }
    let board = read("src/board.rs");
    for gone in ["Screen", "net::read(ledger::PATH)", "ledger::PATH"] {
        assert!(!board.contains(gone), "board.rs に {gone} が在る");
    }
    assert!(
        !crate_dir().join("tests/lgrp.rs").exists(),
        "tests/lgrp.rs が在る"
    );
}

/// ask.rs は問いの頁の一覧の DOM（mod dom の view と scroll_to）と、それだけが使った関数（count・total・card_key・outline・
/// node_cards）を可視の字によらず宣言せず（mod dom の中には view の宣言が無い）、block の view の本文は use と注の行のほかは
/// 空の中身を返す式 ().into_any() だけで字 dom を持たず、質問の窓が使う one_view・listed・unknown_projects・OTHER_UNKNOWN と
/// 届いていない裁定の 1 行の late_view は残る。
#[test]
fn bvshed_qlist_dom_gone() {
    let src = read("src/project/ask.rs");
    for gone in [
        "fn count(",
        "fn total(",
        "fn card_key(",
        "fn outline(",
        "fn node_cards(",
        "fn scroll_to(",
        "    pub fn view() -> AnyView {",
    ] {
        assert!(!src.contains(gone), "ask.rs に {gone} が在る");
    }
    let dom = item(&src, "\nmod dom {");
    assert!(!dom.contains("fn view("), "mod dom の中の view の宣言");
    for kept in [
        "pub fn one_view(",
        "pub fn listed(",
        "pub fn unknown_projects(",
        "pub const OTHER_UNKNOWN",
        "pub fn late_view() -> AnyView {",
    ] {
        assert!(src.contains(kept), "ask.rs に {kept} が無い");
    }
    let view = item(&src, "pub fn view() -> leptos::prelude::AnyView {");
    let body: Vec<&str> = view
        .lines()
        .skip(1)
        .map(str::trim)
        .filter(|l| !l.is_empty() && *l != "}" && !l.starts_with("use ") && !l.starts_with("//"))
        .collect();
    assert_eq!(body, ["().into_any()"], "{view}");
    assert!(!view.contains("dom"), "{view}");
}
