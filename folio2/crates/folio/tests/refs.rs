//! `folio check` の参照 id・逆参照・件数の歯（便 1・docs/design/delivery-1.md §1）。
//! tests/fixtures/refs/ の 3 組（解決に要る欄だけの最小の手書き 4 file に変異 1 つ）で 不合格 1。
//! 各組の違反はちょうど 1 件で、その種類と file まで見る（別の理由で落ちた組を緑にしない）。

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

fn assert_single_violation(name: &str, kind: &str, file: &str, needle: &str) {
    let out = folio_check(&repo_root().join("tests/fixtures/refs").join(name));
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
        v[0].starts_with(&format!("[{kind}] {file}")) && v[0].contains(needle),
        "{name}: {v:?}"
    );
    assert!(stdout(&out).contains("不合格"), "{name}");
}

#[test]
fn refs_dangling_id_fails() {
    assert_single_violation("dangling-id", "参照 id", "srs.yaml", "P-99");
}

#[test]
fn refs_orphan_rule_fails() {
    assert_single_violation("orphan-rule", "逆参照", "rules.yaml", "R-3");
}

#[test]
fn refs_bad_counts_fails() {
    assert_single_violation("bad-counts", "件数", "constitution.yaml", "always");
}
