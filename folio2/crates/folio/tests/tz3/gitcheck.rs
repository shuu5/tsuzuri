//! `folio check` の版管理（git）との照合の歯（便 8・docs/design/delivery-8.md §1）。
//! fixture は増やさず、tests/fixtures/anchor/ の組を一時 dir の `design-intent/` に写し、
//! 版管理の根はその 1 つ上（写しの design-intent 自体を根にしない）に作る。
#![cfg(test)]

use crate::common::{copy_tree, repo_root, stderr, stdout, violations};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// git を呼ぶ。環境変数 GIT_* は継承しない（外の repo へ照合先をすげ替えない）。標準出力の前後の空白を除いて返す。
fn git(cwd: &Path, args: &[&str]) -> String {
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
    String::from_utf8_lossy(&out.stdout).trim().to_string()
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

/// 床を撃つ（写しは残す）。
fn run_check(td: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tz"))
        .arg("check")
        .arg("--dir")
        .arg(td.join("design-intent"))
        .output()
        .expect("folio を起動できない")
}

fn folio_check(td: &Path) -> Output {
    let out = run_check(td);
    let _ = fs::remove_dir_all(td);
    out
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

// ── 便 190（判断の記録 ADR-34）: 照合するのは HEAD の祖先と根の無い枝の履歴だけ ──

const LOST: &str = "は版管理の履歴に在ったが作業ツリーに無い";
const SWAPPED: &str = "が版管理の履歴の同じ形式の anchor と中身が違う";
const V10: &str = "design-intent/anchors/constitution-v1.0.yaml";

/// 違反の行のうち `needle` を含む行の数。
fn count(out: &Output, needle: &str) -> usize {
    violations(out)
        .iter()
        .filter(|l| l.contains(needle))
        .count()
}

/// 取り込んでいない枝 w を切って `edit` の後に commit し（その前に根の file だけの commit を 1 つ挟む）、元の枝へ戻る。
fn side_branch(td: &Path, edit: impl FnOnce(&Path)) {
    git(td, &["checkout", "-q", "-b", "w"]);
    fs::write(td.join("side.txt"), "w\n").unwrap();
    git(td, &["add", "-A"]);
    git(td, &["commit", "-q", "-m", "w1"]);
    edit(td);
    git(td, &["add", "-A"]);
    git(td, &["commit", "-q", "-m", "w2"]);
    git(td, &["checkout", "-q", "-"]);
}

/// v1.0 の anchor の版の字を v1.1 に替えた anchor を足す。
fn add_v11(td: &Path) {
    let v10 = fs::read_to_string(td.join(V10)).unwrap();
    fs::write(
        td.join("design-intent/anchors/constitution-v1.1.yaml"),
        v10.replace("v1.0", "v1.1"),
    )
    .unwrap();
}

/// v1.0 の anchor の題を 1 字変える（形式は同じ・digest は合わない）。
fn rewrite_v10(td: &Path) {
    let path = td.join(V10);
    let before = fs::read_to_string(&path).unwrap();
    let after = before.replacen("title: 床は数える", "title: 床は数えた", 1);
    assert_ne!(before, after, "変異が当たっていない");
    fs::write(&path, after).unwrap();
}

#[test]
fn f190_side_branch_added_anchor_is_not_lost() {
    let td = copy_fixture("f190-side-adds", "root-digest-drift", true);
    side_branch(&td, add_v11);
    let out = folio_check(&td);
    let v = violations(&out);
    assert_eq!(count(&out, LOST), 0, "{v:?}");
    // 土台の違反（列の根が床の定数の表に無い）は今のまま 1 本だけ
    assert_eq!(v.len(), 1, "{v:?}");
    assert!(v[0].contains("列の根の表に無い"), "{v:?}");
}

#[test]
fn f190_side_branch_rewrite_is_not_a_swap_on_head() {
    let td = copy_fixture("f190-side-rewrites", "root-digest-drift", true);
    side_branch(&td, rewrite_v10);
    let out = folio_check(&td);
    let v = violations(&out);
    assert_eq!(count(&out, SWAPPED), 0, "{v:?}");
    assert_eq!(v.len(), 1, "{v:?}");
}

#[test]
fn f190_side_branch_first_anchors_leave_head_unknown() {
    let td = copy_fixture("f190-side-first", "no-anchor", true);
    side_branch(&td, |td| {
        copy_tree(
            &repo_root().join("tests/fixtures/anchor/root-digest-drift/anchors"),
            &td.join("design-intent/anchors"),
        );
    });
    let out = folio_check(&td);
    assert_eq!(
        out.status.code(),
        Some(2),
        "{}{}",
        stdout(&out),
        stderr(&out)
    );
    assert!(violations(&out).is_empty(), "{:?}", violations(&out));
}

#[test]
fn f190_deletion_committed_on_head_still_fails() {
    let td = copy_fixture("f190-head-deletes", "root-digest-drift", true);
    git(&td, &["rm", "-q", V10]);
    git(&td, &["commit", "-q", "-m", "del"]);
    let out = folio_check(&td);
    let v = violations(&out);
    assert_eq!(out.status.code(), Some(1), "{v:?}");
    assert_eq!(count(&out, LOST), 1, "{v:?}");
    assert!(
        v.iter()
            .any(|l| l.contains("anchors/constitution-v1.0.yaml") && l.contains(LOST)),
        "{v:?}"
    );
}

#[test]
fn f190_side_branch_merged_then_deleted_fails() {
    let td = copy_fixture("f190-merged-deletes", "root-digest-drift", true);
    side_branch(&td, add_v11);
    git(&td, &["merge", "-q", "--no-ff", "--no-edit", "w"]);
    git(
        &td,
        &["rm", "-q", "design-intent/anchors/constitution-v1.1.yaml"],
    );
    git(&td, &["commit", "-q", "-m", "del"]);
    let out = folio_check(&td);
    let v = violations(&out);
    assert_eq!(count(&out, LOST), 1, "{v:?}");
    assert!(
        v.iter()
            .any(|l| l.contains("anchors/constitution-v1.1.yaml")),
        "{v:?}"
    );
}

#[test]
fn f190_rootless_branch_still_sees_the_other_history() {
    let td = copy_fixture("f190-orphan", "root-digest-drift", true);
    git(&td, &["checkout", "-q", "--orphan", "clean"]);
    git(
        &td,
        &["rm", "-q", "-r", "--cached", "design-intent/anchors"],
    );
    fs::remove_dir_all(td.join("design-intent/anchors")).unwrap();
    git(&td, &["commit", "-q", "-m", "orphan"]);
    let out = folio_check(&td);
    let v = violations(&out);
    assert_eq!(out.status.code(), Some(1), "{v:?}");
    assert_eq!(count(&out, LOST), 2, "{v:?}");
}

#[test]
fn f190_rewrite_committed_on_head_still_fails() {
    let td = copy_fixture("f190-head-rewrites", "root-digest-drift", true);
    rewrite_v10(&td);
    git(&td, &["commit", "-q", "-am", "rw"]);
    let out = folio_check(&td);
    let v = violations(&out);
    assert_eq!(out.status.code(), Some(1), "{v:?}");
    assert_eq!(count(&out, SWAPPED), 1, "{v:?}");
}

#[test]
fn f190_schema_notes_name_the_counted_history() {
    let text = fs::read_to_string(repo_root().join("design-intent/adr/schema.yaml")).unwrap();
    for want in [
        "照合するのは先頭（HEAD）の祖先の履歴と、HEAD と共通の祖先を持たない根の無い枝の履歴だけ（HEAD と共通の祖先を持つ取り込んでいない枝の履歴は数えない・取り込みの commit では両方の親の履歴を辿る〔--full-history〕・判断の記録 ADR-34）",
        "作業の一時置き場 refs/stash を除く",
        "本流へ取り込んだ時点で本流の床が落とし、本流の床を通さない参照の付け替えは床の外",
        "床が応じるのは環境変数の遮断・根の無い枝の履歴の照合・",
    ] {
        assert_eq!(text.matches(want).count(), 1, "{want}");
    }
    for gone in ["全ての参照（--all）の履歴を見る", "全ての参照の照合"] {
        assert!(!text.contains(gone), "{gone}");
    }
}

#[test]
fn f190_side_refreeze_fails_once_merged_into_head() {
    let td = copy_fixture("f190-side-refreeze", "root-digest-drift", true);
    // 共通の祖先を持つ枝 w で v1.0 を消す commit の後に、中身の違う v1.0 を凍結し直す commit を置く
    git(&td, &["checkout", "-q", "-b", "w"]);
    git(&td, &["rm", "-q", V10]);
    git(&td, &["commit", "-q", "-m", "drop"]);
    git(&td, &["checkout", "-q", "HEAD~1", "--", V10]);
    rewrite_v10(&td);
    git(&td, &["add", "-A"]);
    git(&td, &["commit", "-q", "-m", "refreeze"]);
    git(&td, &["checkout", "-q", "-"]);
    // 取り込む前の本流の床に w の履歴は出ない（土台の違反 1 行だけ）
    let before = run_check(&td);
    assert_eq!(violations(&before).len(), 1, "{:?}", violations(&before));
    // 取り込むと w の commit は先頭の祖先になり、本流の元の v1.0 と中身が違う
    git(&td, &["merge", "-q", "--no-ff", "--no-edit", "w"]);
    let out = folio_check(&td);
    let v = violations(&out);
    assert_eq!(out.status.code(), Some(1), "{v:?}");
    assert_eq!(count(&out, SWAPPED), 1, "{v:?}");
    assert_eq!(count(&out, LOST), 0, "{v:?}");
}

#[test]
fn f190_foreign_rootless_ref_is_not_a_violation() {
    let td = copy_fixture("f190-foreign-ref", "root-digest-drift", true);
    // 台帳の置き場のような git 以外の用途の ref（refs/dolt/data）: anchors/ を触らない根の無い 2 commit
    let empty = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
    let c1 = git(&td, &["commit-tree", empty, "-m", "dolt 1"]);
    let c2 = git(&td, &["commit-tree", empty, "-p", &c1, "-m", "dolt 2"]);
    git(&td, &["update-ref", "refs/dolt/data", &c2]);
    let out = folio_check(&td);
    let v = violations(&out);
    // 土台の違反（列の根が床の定数の表に無い）だけ
    assert_eq!(v.len(), 1, "{v:?}");
    assert!(v[0].contains("列の根の表に無い"), "{v:?}");
}

#[test]
fn f190_cut_before_anchors_refreeze_fails_once_merged() {
    let td = copy_fixture("f190-cut-refreeze", "no-anchor", true);
    let drift = repo_root().join("tests/fixtures/anchor/root-digest-drift/anchors");
    let anchors = td.join("design-intent/anchors");
    // 本流: anchor の無い頃の commit P の上に anchor を足す commit A
    copy_tree(&drift, &anchors);
    git(&td, &["add", "-A"]);
    git(&td, &["commit", "-q", "-m", "anchors"]);
    // P から切った枝 cut で、中身の違う同じ版を凍結し直す commit B
    git(&td, &["checkout", "-q", "-b", "cut", "HEAD~1"]);
    copy_tree(&drift, &anchors);
    rewrite_v10(&td);
    git(&td, &["add", "-A"]);
    git(&td, &["commit", "-q", "-m", "refreeze"]);
    // 範囲の外（ADR-34 決定 (4)）: その枝の上の床は本流の A を数えない
    assert_eq!(count(&run_check(&td), SWAPPED), 0);
    // 衝突を枝の側で解いて本流へ取り込むと、本流の床が A と中身の違いを落とす
    git(&td, &["checkout", "-q", "-"]);
    git(&td, &["merge", "-q", "--no-edit", "-X", "theirs", "cut"]);
    let out = folio_check(&td);
    let v = violations(&out);
    assert_eq!(out.status.code(), Some(1), "{v:?}");
    assert_eq!(count(&out, SWAPPED), 1, "{v:?}");
}

#[test]
fn f190_remote_tracking_side_branch_is_not_lost() {
    // 台帳 .258 の形: 取り込んでいない枝を指すのは remote-tracking の参照だけ
    let td = copy_fixture("f190-remote-side", "root-digest-drift", true);
    side_branch(&td, add_v11);
    git(
        &td,
        &["update-ref", "refs/remotes/origin/w", "refs/heads/w"],
    );
    git(&td, &["update-ref", "-d", "refs/heads/w"]);
    let out = folio_check(&td);
    let v = violations(&out);
    assert_eq!(count(&out, LOST), 0, "{v:?}");
    assert_eq!(v.len(), 1, "{v:?}");
}

#[test]
fn f190_rev_list_failure_is_unknown() {
    let td = copy_fixture("f190-rev-list-fails", "root-digest-drift", true);
    // rev-list だけを失敗させ、ほかの命令は本物の git へ渡す git を PATH の先頭に置く
    let path = std::env::var_os("PATH").unwrap_or_default();
    let real = std::env::split_paths(&path)
        .map(|d| d.join("git"))
        .find(|g| g.is_file())
        .expect("git が PATH に無い");
    let bin = td.join("fakebin");
    fs::create_dir_all(&bin).unwrap();
    let fake = bin.join("git");
    fs::write(
        &fake,
        format!(
            "#!/bin/sh\nfor a in \"$@\"; do [ \"$a\" = rev-list ] && exit 128; done\nexec '{}' \"$@\"\n",
            real.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    let mut paths = vec![bin];
    paths.extend(std::env::split_paths(&path));
    let out = Command::new(env!("CARGO_BIN_EXE_tz"))
        .arg("check")
        .arg("--dir")
        .arg(td.join("design-intent"))
        .env("PATH", std::env::join_paths(paths).unwrap())
        .output()
        .expect("folio を起動できない");
    let _ = fs::remove_dir_all(&td);
    assert!(
        stderr(&out)
            .lines()
            .any(|l| l.starts_with("# まだ分からない: ")
                && l.contains("版管理を読めない（ls-tree / log / rev-list が失敗）")),
        "{}",
        stderr(&out)
    );
    assert_eq!(count(&out, LOST) + count(&out, SWAPPED), 0);
}

#[test]
fn f190_stash_is_not_counted() {
    let td = copy_fixture("f190-stash", "root-digest-drift", true);
    // 作業の一時置き場（refs/stash）: 未追跡の anchor を stash -u で退ける（未追跡の commit は根の無い commit）
    add_v11(&td);
    git(&td, &["stash", "-q", "-u"]);
    let out = folio_check(&td);
    let v = violations(&out);
    assert_eq!(count(&out, LOST), 0, "{v:?}");
    assert_eq!(v.len(), 1, "{v:?}");
}
