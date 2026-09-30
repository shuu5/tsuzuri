//! `folio check` の判断の記録と正本 4 file・凍結 anchor の列の突き合わせの歯（便 6・docs/design/delivery-6.md §1）。
//! tests/fixtures/link/ の 3 組（違反 0 の最小の手書き 4 file + adr/schema.yaml + adr/ADR-1.yaml に変異 1 つ）で 不合格 1。
//! 各組の違反はちょうど 1 件で、その種類と文言まで見る（別の理由で落ちた組を緑にしない）。
//! ただし `amended-by-orphan/` は便 7 の凍結 anchor の列の検査で「anchor が消された」が足されて 2 件。
//! `retreat-kind-drift/` は便 122 から違反 0・まだ分からない（撤退条件の種類は部分集合で数える）。
//! 便 203: 写しの憲法の名（fixture-constitution）は folio2 の置き場の名でなく条 N-4 を持たないので、名札は検査の名（改訂の承認）で、
//! まだ分からない の行の要件の id（FR25）は落ちる。
//! 編集時の口のつながりの行（網の中の A-2・N-4）も同じ名札で出す（歯 f203_）。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../folio2")
}

fn folio_check(dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tz"))
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
            == "# まだ分からない: constitution.yaml: schema.enums.retreat_kind の「drift」が判断の記録の床の撤退条件の種類 [spike, measure, ruling] に無い＝判断の記録の撤退条件を置き場の値域で数えられない"),
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
            .any(|l| l.starts_with("[改訂の承認] ") && l.contains("発効していない")),
        "{name}: {v:?}"
    );
    assert!(
        v.iter()
            .any(|l| l.starts_with("[改訂の承認] ") && l.contains("anchor が消された")),
        "{name}: {v:?}"
    );
    assert!(stdout(&out).contains("不合格"), "{name}");
}

/// 便 203: 外の置き場（写しの名 fixture-constitution）の編集時の口は、つながりに数える違反の行も素の床と同じ検査の名で出す。
/// 要件の平易文に実在しない判断の記録の id を書こうとすると、つながりの行は [改訂と判断の記録] の 1 行だけ（置き場は書かない）。
#[test]
fn f203_abroad_proposed_link_lines_name_the_check() {
    let dir = repo_root().join("tests/fixtures/link/adr-id-missing");
    let srs = fs::read_to_string(dir.join("srs.yaml")).unwrap();
    let text = srs.replacen("    plain: 書類を作ります。\n", "    plain: 書類を作ります（ADR-99）。\n", 1);
    assert_ne!(srs, text, "置き換える字が無い（前提が崩れた）");
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["check", "--dir"])
        .arg(&dir)
        .args(["--proposed", "srs.yaml"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("folio を起動できない");
    child.stdin.take().unwrap().write_all(text.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    let all = stdout(&out);
    assert_eq!(out.status.code(), Some(0), "{all}{}", String::from_utf8_lossy(&out.stderr));
    assert!(violations(&out).is_empty(), "{all}");
    assert!(
        all.lines()
            .any(|l| l.starts_with("folio check --proposed: 通す（新しい違反 0・つながり 1・まだ分からない 0・")),
        "{all}"
    );
    let links: Vec<&str> = all.lines().filter(|l| l.starts_with("# つながり")).collect();
    assert_eq!(
        links,
        ["# つながり（編集は止めない・事後の床が数える）: [改訂と判断の記録] srs.yaml: requirements[0].plain: 判断の記録 ADR-99 が実在しない"],
        "{all}"
    );
}
