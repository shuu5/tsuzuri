//! folio の歯の群 tz3 の共通の手（群の 2 つ以上の module が字で同じ写しを持っていた helper を 1 つだけ置く・畳みの道具 fold2.py が寄せる・判断の記録 ADR-63 の決定 (8) の (c)）。
//! 写しを持っていた module は use crate::common::… で呼ぶ。字が 1 文字でも違う物はここへ寄せない。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub(crate) const EDGES_HEAD: &str = "# 辺（1 行 = 端 / 端 / 型・タブ区切り）";

pub(crate) const FLOOR_BASE: &str = "tests/fixtures/floor_base/design-intent";

/// 器（scribe2）の導出 file を写しの根へ写す（設計ノートの契約表の節が読む先・便 23）。
pub(crate) fn copy_external_schema(root: &Path) {
    fs::create_dir_all(root.join("contracts/field-schema")).unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        root.join("contracts/field-schema/schema.toml"),
    )
    .unwrap();
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

pub(crate) fn edit(path: &Path, f: impl FnOnce(&str) -> String) {
    let before = fs::read_to_string(path).unwrap();
    let after = f(&before);
    assert_ne!(before, after, "変異が当たっていない: {}", path.display());
    fs::write(path, after).unwrap();
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

pub(crate) fn passed(out: Output) -> String {
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).expect("出力が UTF-8 でない")
}

pub(crate) fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../folio2")
}

pub(crate) fn show(out: &Output) -> String {
    format!("{}\n{}", text(&out.stdout), text(&out.stderr))
}

pub(crate) fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

pub(crate) fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub(crate) fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// 違反の行（`[種類] …`）だけを拾う。
pub(crate) fn violations(out: &Output) -> Vec<String> {
    stdout(out)
        .lines()
        .filter(|l| l.starts_with('['))
        .map(str::to_string)
        .collect()
}
