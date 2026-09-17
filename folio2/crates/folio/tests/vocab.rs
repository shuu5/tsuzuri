//! `folio check` の語彙の検査の歯（便 4・docs/design/delivery-4.md §1）。
//! tests/fixtures/vocab/ の 2 組（最小の手書き 4 file に変異 1 つ）で 不合格 1。
//! 各組の違反はちょうど 1 件で、種別 R-9 と語・場所まで見る（別の理由で落ちた組・免除の形を数えた組を緑にしない）。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn folio_check(dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("check")
        .arg("--dir")
        .arg(dir)
        .output()
        .expect("folio を起動できない")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// 違反の行（`[種類] …`）だけを拾う。
fn violations(out: &Output) -> Vec<String> {
    stdout(out)
        .lines()
        .filter(|l| l.starts_with('['))
        .map(str::to_string)
        .collect()
}

fn assert_single_unknown_word(name: &str, at: &str, word: &str) {
    let out = folio_check(&repo_root().join("tests/fixtures/vocab").join(name));
    assert_eq!(
        out.status.code(),
        Some(1),
        "{name}: {}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    let v = violations(&out);
    assert_eq!(v.len(), 1, "{name}: 違反は変異の 1 件だけのはず: {v:?}");
    assert!(
        v[0].starts_with(&format!("[R-9] {at}")) && v[0].contains(&format!("「{word}」")),
        "{name}: {v:?}"
    );
    assert!(stdout(&out).contains("不合格"), "{name}");
}

#[test]
fn vocab_unknown_word_fails() {
    assert_single_unknown_word("unknown-word", "P-1 plain", "widget");
}

#[test]
fn vocab_exemptions_count_only_the_unknown_word() {
    assert_single_unknown_word("exemptions", "P-1 plain", "widget");
}
