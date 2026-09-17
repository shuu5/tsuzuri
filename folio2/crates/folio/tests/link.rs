//! `folio check` の判断の記録と正本 4 file・凍結 anchor の列の突き合わせの歯（便 6・docs/design/delivery-6.md §1）。
//! tests/fixtures/link/ の 3 組（違反 0 の最小の手書き 4 file + adr/schema.yaml + adr/ADR-1.yaml に変異 1 つ）で 不合格 1。
//! 各組の違反はちょうど 1 件で、その種類と文言まで見る（別の理由で落ちた組を緑にしない）。

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

fn assert_single_violation(name: &str, kind: &str, needle: &str) {
    let out = folio_check(&repo_root().join("tests/fixtures/link").join(name));
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
        v[0].starts_with(&format!("[{kind}] ")) && v[0].contains(needle),
        "{name}: {v:?}"
    );
    assert!(stdout(&out).contains("不合格"), "{name}");
}

#[test]
fn link_retreat_kind_drift_fails() {
    assert_single_violation("retreat-kind-drift", "adr", "retreat_kind");
}

#[test]
fn link_adr_id_missing_fails() {
    assert_single_violation("adr-id-missing", "adr", "ADR-9");
}

#[test]
fn link_amended_by_orphan_fails() {
    assert_single_violation("amended-by-orphan", "N-4", "発効していない");
}
