//! 検証役の型の本文の決定はしごの行の歯（接頭辞 agldr_・設計ノート surface-v4a 行 ag-ladder-verifier）。
//! plugin/agents/verifier.md の本文がはしごの行を 1 度持つことを見る。型の定義は Claude Code が係を起こす時に読む設定の字で、歯から係を起こして所見を測れないので字を照らす。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

/// 検証役の型の file（workspace の根から）。
const VERIFIER: &str = "plugin/agents/verifier.md";

/// 本文の末に在るはしごの行（改行を含まない）。
const LINE: &str = "足す提案（走査・記録・規則の表の行・仕掛け・道具）を攻める時は、所見に、決定はしご（条 P-27.1）の段 1〜4 のどこで止まれたかと、消す物の名指し（条 P-27.2）の有無を書く。";

/// 否定の見本で行を動かし込む所（型の頭に 1 度在る行）。
const MODEL: &str = "model: opus\n";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// 検証役の型の今の字。
fn current() -> String {
    fs::read_to_string(root().join(VERIFIER))
        .unwrap_or_else(|e| panic!("{VERIFIER} を読む: {e}"))
}

/// 頭の --- の後の本文の中の LINE の数。頭が読めなければ None。
fn ladder_count(text: &str) -> Option<usize> {
    let rest = text.strip_prefix("---\n")?;
    let (_, body) = rest.split_once("\n---\n")?;
    Some(body.matches(LINE).count())
}

/// (1) 検証役の型の頭の後の本文は、はしごの行をちょうど 1 度持つ。
#[test]
fn agldr_verifier_body_holds_the_ladder_line() {
    assert_eq!(ladder_count(&current()), Some(1));
}

/// (2) 行を外した見本は 0 を、2 度にした見本は 2 を、外して頭の欄 model の行の直後へ動かした見本は 0 を数える。
#[test]
fn agldr_line_removed_doubled_or_moved_is_counted() {
    let now = current();
    let removed = now.replacen(LINE, "", 1);
    assert_eq!(ladder_count(&removed), Some(0));
    let doubled = now.replacen(LINE, &format!("{LINE}\n{LINE}"), 1);
    assert_eq!(ladder_count(&doubled), Some(2));
    let moved = removed.replacen(MODEL, &format!("{MODEL}{LINE}\n"), 1);
    assert_eq!(moved.matches(LINE).count(), 1);
    assert_eq!(ladder_count(&moved), Some(0));
}
