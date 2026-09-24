//! `folio check` の判断の記録と正本 4 file・凍結 anchor の列の突き合わせの歯（便 6・docs/design/delivery-6.md §1）。
//! tests/fixtures/link/ の 3 組（違反 0 の最小の手書き 4 file + adr/schema.yaml + adr/ADR-1.yaml に変異 1 つ）で 不合格 1。
//! 各組の違反はちょうど 1 件で、その種類と文言まで見る（別の理由で落ちた組を緑にしない）。
//! ただし `amended-by-orphan/` は便 7 の凍結 anchor の列の検査で「anchor が消された」が足されて 2 件。
//! `retreat-kind-drift/` は便 122 から違反 0・まだ分からない（撤退条件の種類は部分集合で数える）。

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

/// 撤退条件の種類は部分集合で数える（便 122・FR25）。床の定数に無い値 drift を持つ値域は違反でなく「まだ分からない」で、
/// 合格にならない。部分集合なら黙る側は tests/constitution_range.rs の歯 5 が実の design-intent の写しで見る。
#[test]
fn link_retreat_kind_drift_is_unknown_and_not_pass() {
    let name = "retreat-kind-drift";
    let out = folio_check(&repo_root().join("tests/fixtures/link").join(name));
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{name}: {}{err}", stdout(&out));
    assert!(violations(&out).is_empty(), "{name}: {:?}", violations(&out));
    assert!(
        err.lines().any(|l| l
            == "# まだ分からない: constitution.yaml: schema.enums.retreat_kind の「drift」が判断の記録の床の撤退条件の種類 [spike, measure, ruling] に無い＝判断の記録の撤退条件を置き場の値域で数えられない（FR25）"),
        "{name}: {err}"
    );
}

#[test]
fn link_adr_id_missing_fails() {
    assert_single_violation("adr-id-missing", "adr", "ADR-9");
}

/// 改訂の記録（amended_by）が在るのに anchors/ が無いので、便 7 の列の検査が「anchor が消された」を足す＝違反 2 件。
#[test]
fn link_amended_by_orphan_fails() {
    let name = "amended-by-orphan";
    let out = folio_check(&repo_root().join("tests/fixtures/link").join(name));
    assert_eq!(
        out.status.code(),
        Some(1),
        "{name}: {}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    let v = violations(&out);
    assert_eq!(
        v.len(),
        2,
        "{name}: 発効していない + anchor が消された の 2 件のはず: {v:?}"
    );
    assert!(
        v.iter()
            .any(|l| l.starts_with("[N-4] ") && l.contains("発効していない")),
        "{name}: {v:?}"
    );
    assert!(
        v.iter()
            .any(|l| l.starts_with("[N-4] ") && l.contains("anchor が消された")),
        "{name}: {v:?}"
    );
    assert!(stdout(&out).contains("不合格"), "{name}");
}
