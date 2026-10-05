//! folio の歯の群 tz4 の共通の手（群の 2 つ以上の module が字で同じ写しを持っていた helper を 1 つだけ置く・畳みの道具 fold2.py が寄せる・判断の記録 ADR-63 の決定 (8) の (c)）。
//! 写しを持っていた module は use crate::common::… で呼ぶ。字が 1 文字でも違う物はここへ寄せない。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

pub(crate) const BEGIN: &str = "# folio:schema:begin — 生成区間・手で直さない・正本は実装の定数（folio schema --write が書く）";

pub(crate) const END: &str = "# folio:schema:end";

pub(crate) const FLOOR_BASE: &str = "tests/fixtures/floor_base/design-intent";

/// 命令が見る file の数（合格の標準出力の行数・判断の記録 → 設計ノート → 天井の正本 → 規則の表 → 入口の正本
/// → 要件書 → 語彙 → 相談窓口 → 索引の欄の決まり）。
pub(crate) const TARGETS: usize = 9;

/// 終了コードと、標準出力（0 のとき）か標準エラー（それ以外）に含む語。合格の標準出力は file ごとの 1 行（`TARGETS` 行）。
pub(crate) fn assert_outcome(out: &Output, code: i32, words: &[&str]) {
    let shown = format!("{}{}", stdout(out), stderr(out));
    assert_eq!(out.status.code(), Some(code), "{shown}");
    let stream = if code == 0 { stdout(out) } else { stderr(out) };
    for word in words {
        assert!(stream.contains(word), "「{word}」が無い: {shown}");
    }
    if code != 0 {
        assert!(stdout(out).is_empty(), "標準出力は空のはず: {shown}");
        assert!(stderr(out).starts_with("folio schema: "), "{shown}");
    }
    assert_eq!(
        stdout(out).lines().count(),
        if code == 0 { TARGETS } else { 0 },
        "{shown}"
    );
}

pub(crate) fn assert_passes(out: &Output) {
    assert_eq!(out.status.code(), Some(0), "{}{}", stdout(out), stderr(out));
    assert!(violations(out).is_empty(), "{:?}", violations(out));
    assert!(
        stdout(out).contains("folio check: 合格（違反 0・"),
        "{}",
        stdout(out)
    );
}

pub(crate) fn both(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

pub(crate) fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

pub(crate) fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &to);
        } else {
            fs::copy(entry.path(), &to).unwrap();
        }
    }
}

pub(crate) fn design_intent() -> PathBuf {
    repo_root().join("design-intent")
}

pub(crate) fn fixture() -> PathBuf {
    repo_root().join("tests/fixtures/face")
}

pub(crate) fn folio(head: &[&str], dir: &Path, tail: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(head)
        .arg(dir)
        .args(tail)
        .output()
        .expect("folio を起動できない")
}

pub(crate) fn folio_check(dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tz"))
        .arg("check")
        .arg("--dir")
        .arg(dir)
        .output()
        .expect("folio を起動できない")
}

/// git を呼ぶ。環境変数 GIT_* は継承しない。
pub(crate) fn git(cwd: &Path, args: &[&str]) {
    let mut cmd = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(key);
        }
    }
    let out = cmd
        .current_dir(cwd)
        .args([
            "-c",
            "user.email=fx@example",
            "-c",
            "user.name=fx",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .expect("git を起動できない");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// file の字面の変異（当て先は 1 か所だけ）。
pub(crate) fn mutate_file(path: &Path, from: &str, to: &str) {
    let before = fs::read_to_string(path).unwrap();
    assert_eq!(
        before.matches(from).count(),
        1,
        "変異の当て先が 1 か所でない: {from:?}"
    );
    fs::write(path, before.replacen(from, to, 1)).unwrap();
}

/// 生成区間（begin の行の次から end の行の手前まで）。
pub(crate) fn region(text: &str) -> &str {
    let b = text.find(&format!("{BEGIN}\n")).expect("begin が無い") + BEGIN.len() + 1;
    let e = text.find(&format!("\n{END}\n")).expect("end が無い") + 1;
    &text[b..e]
}

pub(crate) fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../folio2")
}

/// 要約値（sha256）は命令 `sha256sum` を子の処理で撃って測る（歯は crate の中を読めない・tests/tz1/bundle.rs と同じ形）。
pub(crate) fn sha256_hex(bytes: &[u8]) -> Result<String, String> {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("sha256sum を起動できない: {e}"))?;
    child
        .stdin
        .take()
        .ok_or_else(|| "sha256sum の標準入力が無い".to_string())?
        .write_all(bytes)
        .map_err(|e| format!("sha256sum へ書けない: {e}"))?;
    let out = child
        .wait_with_output()
        .map_err(|e| format!("sha256sum を待てない: {e}"))?;
    if !out.status.success() {
        return Err("sha256sum が失敗した".to_string());
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let hex = text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    if hex.len() != 64 {
        return Err(format!("sha256sum の出力が 16 進 64 字でない: {text}"));
    }
    Ok(hex)
}

pub(crate) fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

pub(crate) fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// 違反の行（`[種類] …`）だけを拾う。
pub(crate) fn violations(out: &Output) -> Vec<String> {
    stdout(out)
        .lines()
        .filter(|l| l.starts_with('['))
        .map(str::to_string)
        .collect()
}
