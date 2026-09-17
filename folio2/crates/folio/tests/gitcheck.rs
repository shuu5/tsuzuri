//! `folio check` の版管理（git）との照合の歯（便 8・docs/design/delivery-8.md §1）。
//! fixture は増やさず、tests/fixtures/anchor/ の組を一時 dir の `design-intent/` に写し、
//! 版管理の根はその 1 つ上（写しの design-intent 自体を根にしない）に作る。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy_tree(src: &Path, dst: &Path) {
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

/// git を呼ぶ。環境変数 GIT_* は継承しない（外の repo へ照合先をすげ替えない）。
fn git(cwd: &Path, args: &[&str]) {
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

/// 写しを作る（`commit` なら版管理の根で git init + add + 1 commit）。一時 dir の根を返す。
fn copy_fixture(case: &str, fixture: &str, commit: bool) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-gitcheck-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    copy_tree(
        &repo_root().join("tests/fixtures/anchor").join(fixture),
        &td.join("design-intent"),
    );
    if commit {
        git(&td, &["init", "-q"]);
        git(&td, &["add", "-A"]);
        git(&td, &["commit", "-q", "-m", "fixture"]);
    }
    td
}

fn folio_check(td: &Path) -> Output {
    let out = Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("check")
        .arg("--dir")
        .arg(td.join("design-intent"))
        .output()
        .expect("folio を起動できない");
    let _ = fs::remove_dir_all(td);
    out
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// 違反の行（`[種類] …`）だけを拾う。
fn violations(out: &Output) -> Vec<String> {
    stdout(out)
        .lines()
        .filter(|l| l.starts_with('['))
        .map(str::to_string)
        .collect()
}

#[test]
fn gitcheck_without_git_is_unknown() {
    let td = copy_fixture("no-git", "no-anchor", false);
    let out = folio_check(&td);
    assert_eq!(
        out.status.code(),
        Some(2),
        "{}{}",
        stdout(&out),
        stderr(&out)
    );
    assert!(violations(&out).is_empty(), "{:?}", violations(&out));
    assert!(
        stderr(&out)
            .lines()
            .any(|l| l.starts_with("# まだ分からない: ")
                && l.contains("版管理（git）が無いか読めない")),
        "{}",
        stderr(&out)
    );
}

#[test]
fn gitcheck_anchor_removed_from_worktree_fails() {
    let td = copy_fixture("removed", "root-digest-drift", true);
    fs::remove_file(td.join("design-intent/anchors/constitution-v1.0.yaml")).unwrap();
    let out = folio_check(&td);
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}{}",
        stdout(&out),
        stderr(&out)
    );
    let v = violations(&out);
    assert!(v.len() >= 2, "{v:?}");
    assert_eq!(
        v.iter()
            .filter(|l| l.contains("にあるが作業ツリーに無い"))
            .count(),
        1,
        "{v:?}"
    );
    assert!(
        v.iter()
            .any(|l| l.starts_with("[anchor] ") && l.contains("列の根") && l.contains("床の定数")),
        "{v:?}"
    );
}

#[test]
fn gitcheck_ignored_anchors_fail() {
    let td = copy_fixture("ignored", "root-digest-drift", true);
    fs::write(td.join(".gitignore"), "anchors/\n").unwrap();
    let out = folio_check(&td);
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}{}",
        stdout(&out),
        stderr(&out)
    );
    let v = violations(&out);
    assert!(v.iter().any(|l| l.contains("版管理から除外")), "{v:?}");
}

#[test]
fn gitcheck_rewritten_anchor_fails() {
    let td = copy_fixture("rewritten", "root-digest-drift", true);
    let path = td.join("design-intent/anchors/constitution-v1.0.yaml");
    let before = fs::read_to_string(&path).unwrap();
    let after = before.replacen("title: 床は数える", "title: 床は数えた", 1);
    assert_ne!(before, after, "変異が当たっていない");
    fs::write(&path, after).unwrap();
    let out = folio_check(&td);
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}{}",
        stdout(&out),
        stderr(&out)
    );
    let v = violations(&out);
    assert!(
        v.iter()
            .any(|l| l.contains("履歴の同じ形式の anchor と中身が違う")),
        "{v:?}"
    );
}
