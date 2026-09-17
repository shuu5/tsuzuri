//! `folio check` の判断の記録（adr/）の欄の決まりの歯（便 5・docs/design/delivery-5.md §1）。
//! tests/fixtures/adr/ の 3 組（違反 0 の最小の手書き 4 file + adr/schema.yaml + adr/ADR-1.yaml に変異 1 つ）で 不合格 1。
//! 各組の違反はちょうど 1 件で、その種類と場所と文言まで見る（別の理由で落ちた組を緑にしない）。

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

fn assert_single_violation(name: &str, kind: &str, at: &str, needle: &str) {
    let out = folio_check(&repo_root().join("tests/fixtures/adr").join(name));
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
        v[0].starts_with(&format!("[{kind}] {at}")) && v[0].contains(needle),
        "{name}: {v:?}"
    );
    assert!(stdout(&out).contains("不合格"), "{name}");
}

#[test]
fn adr_schema_drift_fails() {
    assert_single_violation("schema-drift", "adr", "adr/schema.yaml", "options_rule.min");
}

#[test]
fn adr_two_adopted_fails() {
    assert_single_violation("two-adopted", "adr", "ADR-1", "採用の案");
}

#[test]
fn adr_effective_without_approval_fails() {
    assert_single_violation("effective-no-approval", "N-4", "ADR-1", "approval");
}
