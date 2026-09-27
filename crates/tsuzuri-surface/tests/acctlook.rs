//! 便 h-look の歯: account board の頁の題・群の枠の段は見出しを持たない・群の card の使った割合の見出しの語。
//! DOM は wasm の target のときだけなので、mount と groups_view は src の字を読んで見る。

use std::path::PathBuf;

use tsuzuri_surface::account::{BRAND, page_title};
use tsuzuri_surface::vocab::label;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// `start` の字から次の `fn ` の手前までの本文。
fn fn_body<'a>(text: &'a str, start: &str) -> &'a str {
    let at = text
        .find(start)
        .unwrap_or_else(|| panic!("{start} が在る"));
    let rest = &text[at + start.len()..];
    let end = rest.find("fn ").unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn acctlook_page_title_is_brand_and_board() {
    assert_eq!(BRAND, "器");
    assert_eq!(label("acct_board"), "account board");
    assert_eq!(page_title(), "器 — account board");
}

#[test]
fn acctlook_mount_sets_page_title() {
    let board = read("src/account/board.rs");
    let body = fn_body(&board, "pub fn mount");
    assert!(body.contains("set_title("), "mount が document の title を置く: {body}");
    assert!(body.contains("page_title()"), "mount が page_title の字を渡す: {body}");
    assert!(body.contains("mount_to_body"), "本文は mount のもの: {body}");
}

#[test]
fn acctlook_groups_view_has_no_heading_row() {
    let home = read("src/account/home.rs");
    let body = fn_body(&home, "pub fn groups_view");
    assert!(!body.contains("section("), "groups_view は section を呼ばない: {body}");
    assert!(body.contains("<section"), "groups_view は section の要素に中身を置く: {body}");
    assert!(body.contains("block.class") && body.contains("block.id"), "要素の class と id は block のもの: {body}");
}

#[test]
fn acctlook_used_heading_in_group_card() {
    let home = read("src/account/home.rs");
    assert!(!home.contains("hs(\"pressure\")"), "使った割合の見出しに閾値の語を置かない");
    assert!(home.contains("hs(\"used\")"), "使った割合の見出しは語の鍵 used");
    assert_eq!(label("used"), "使った割合");
}
