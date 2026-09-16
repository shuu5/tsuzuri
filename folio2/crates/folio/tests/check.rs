//! `folio check` 自身の歯（便 0・docs/design/delivery-0.md §1 / §4）。
//! 正本 4 file で合格、tests/fixtures/check/ の 4 組で 不合格 1 / 不合格 1 / 不合格 1 / まだ分からない 2。
//! 各組は変異 1 つだけを持つ＝違反はちょうど 1 件で、その種類まで見る（別の理由で落ちた組を緑にしない）。

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

fn fixture(name: &str) -> PathBuf {
    repo_root().join("tests/fixtures/check").join(name)
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

fn assert_single_violation(name: &str, kind: &str, file: &str) {
    let out = folio_check(&fixture(name));
    assert_eq!(out.status.code(), Some(1), "{name}: {}", stdout(&out));
    let v = violations(&out);
    assert_eq!(v.len(), 1, "{name}: 違反は変異の 1 件だけのはず: {v:?}");
    assert!(
        v[0].starts_with(&format!("[{kind}] {file}")),
        "{name}: {v:?}"
    );
    assert!(stdout(&out).contains("不合格"), "{name}");
}

#[test]
fn check_canonical_design_intent_passes() {
    let out = folio_check(&repo_root().join("design-intent"));
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(violations(&out).is_empty());
    assert!(stdout(&out).contains("合格"));
}

#[test]
fn check_dup_key_fails() {
    assert_single_violation("dup-key", "重複キー", "rules.yaml");
}

#[test]
fn check_unknown_section_fails() {
    assert_single_violation("unknown-section", "未知の節", "srs.yaml");
}

#[test]
fn check_empty_field_fails() {
    assert_single_violation("empty-field", "欄の非空", "vocabulary.yaml");
}

#[test]
fn check_missing_file_is_unknown_not_pass() {
    let out = folio_check(&fixture("missing-file"));
    assert_eq!(out.status.code(), Some(2), "{}", stdout(&out));
    assert!(String::from_utf8_lossy(&out.stderr).contains("constitution.yaml: 正本が無い"));
    let s = stdout(&out);
    assert!(
        s.contains("まだ分からない") && !s.contains("folio check: 合格"),
        "{s}"
    );
}
