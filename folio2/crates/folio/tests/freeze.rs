//! `folio check --emit-amends` と `--freeze-anchor` の歯（便 9・docs/design/delivery-9.md §1）。
//! 歯の中で凍結した土台（tests/fixtures/floor_base/design-intent/・憲法 v1.0）の写し全部を一時 dir に作り git init と 1 commit を行う。
//! (1) P-1 の title を変え版を v1.1 にして `--emit-amends` → 1・標準出力は見出しと差分 1 行だけ／(2) 変異なしで `--emit-amends` → 0・見出しだけ／
//! (3) 変異なしで `--freeze-anchor` → 1・「新しくない」・anchors/ は不変／(4) 合成した改訂で `--freeze-anchor` → 0・v1.1 の anchor と索引の追記・続けて旗なし → 0／
//! (5) (4) の amends の new_text を 1 字違えて `--freeze-anchor` → 1・「凍結しない」・v1.1 の anchor は作られない。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const OLD_TITLE: &str = "判断する道具を作らない";
const NEW_TITLE: &str = "判断する道具を作らない（改訂）";
const RULING: &str = "f2-648.19 notes 2026-09-17（合成した改訂の承認）";

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

/// 器（scribe2）の導出 file を写しの根へ写す（設計ノートの契約表の節が読む先・便 23）。
fn copy_external_schema(root: &Path) {
    fs::create_dir_all(root.join("contracts")).unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        root.join("contracts/schema.toml"),
    )
    .unwrap();
}

/// git を呼ぶ。環境変数 GIT_* は継承しない。
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

/// 写しの一時 dir（歯の終わりに消す）。
struct Work {
    root: PathBuf,
}

impl Work {
    fn new(case: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-freeze-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        // 土台は凍結した写し（tests/fixtures/floor_base/）。実の置き場は版が上がるので、版を字面で持つ歯の土台にしない。
        copy_tree(
            &repo_root().join("tests/fixtures/floor_base/design-intent"),
            &root.join("design-intent"),
        );
        copy_external_schema(&root);
        git(&root, &["init", "-q"]);
        Work::commit_in(&root);
        Work { root }
    }

    fn commit_in(root: &Path) {
        git(root, &["add", "-A"]);
        git(root, &["commit", "-q", "--allow-empty", "-m", "fixture"]);
    }

    fn commit(&self) {
        Work::commit_in(&self.root);
    }

    fn dir(&self) -> PathBuf {
        self.root.join("design-intent")
    }

    fn check(&self, flags: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("check")
            .arg("--dir")
            .arg(self.dir())
            .args(flags)
            .output()
            .expect("folio を起動できない")
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn edit(path: &Path, f: impl FnOnce(&str) -> String) {
    let before = fs::read_to_string(path).unwrap();
    let after = f(&before);
    assert_ne!(before, after, "変異が当たっていない: {}", path.display());
    fs::write(path, after).unwrap();
}

/// 憲法の P-1 の title を新しい題にし、meta.version を v1.1 にする。
fn bump_title(dir: &Path) {
    edit(&dir.join("constitution.yaml"), |t| {
        let t = t.replacen(
            &format!("  - id: P-1\n    title: {OLD_TITLE}\n"),
            &format!("  - id: P-1\n    title: {NEW_TITLE}\n"),
            1,
        );
        let meta = t.find("\nmeta:\n").expect("meta が無い");
        let at = meta
            + t[meta..]
                .find("\n  version: v1.0\n")
                .expect("meta.version が無い");
        format!(
            "{}\n  version: v1.1\n{}",
            &t[..at],
            &t[at + "\n  version: v1.0\n".len()..]
        )
    });
}

/// 合成した改訂: 版上げ + adr/ADR-5.yaml（amends の new_text は `new_text`）+ P-1 の amended_by。
fn synthetic_amendment(dir: &Path, new_text: &str) {
    bump_title(dir);
    let adr = format!(
        "# 合成した改訂の判断の記録（歯の中で作る）。\n\
id: ADR-5\n\
title: 条 P-1 の見出しを改める\n\
status: accepted\n\
date: 2026-09-17\n\
context: 条 P-1 の見出しを改める必要が生じた。\n\
decision: 条 P-1 の見出しの末尾に「（改訂）」を足す。\n\
options:\n\
  - {{id: a, name: 改める, text: 見出しを改める。, verdict: adopted, reason: 改訂の手順を通すため。}}\n\
  - {{id: b, name: 改めない, text: 見出しをそのままにする。, verdict: rejected, reason: 改訂の手順を通せない。}}\n\
basis: [A-2]\n\
retreat: {{kind: ruling, condition: 持ち主が取り消したら元へ戻す。}}\n\
plain: 条 P-1 の見出しを少しだけ改めます。\n\
approval: {{who: 持ち主, date: 2026-09-17, ruling: \"{RULING}\", verbatim: 承認する, surface: R-8}}\n\
grill: {{when: 2026-09-17, who: 持ち主, where: 対話面, summary: 反対側からの確認をした。}}\n\
amends:\n\
  - {{target: P-1, field: title, version: v1.1, previous_text: {OLD_TITLE}, new_text: {new_text}}}\n"
    );
    fs::write(dir.join("adr/ADR-5.yaml"), adr).unwrap();
    edit(&dir.join("constitution.yaml"), |t| {
        t.replacen(
            &format!("  - id: P-1\n    title: {NEW_TITLE}\n"),
            &format!(
                "  - id: P-1\n    title: {NEW_TITLE}\n    amended_by: [{{adr: ADR-5, date: 2026-09-17, approved_by: 持ち主, ruling: \"{RULING}\", previous_text: {OLD_TITLE}, rationale: 改訂の手順を通すため。}}]\n"
            ),
            1,
        )
    });
}

fn anchors_snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files: Vec<(String, Vec<u8>)> = fs::read_dir(dir.join("anchors"))
        .unwrap()
        .map(|e| {
            let e = e.unwrap();
            (
                e.file_name().to_string_lossy().into_owned(),
                fs::read(e.path()).unwrap(),
            )
        })
        .collect();
    files.sort();
    files
}

/// (1) 版上げだけ（未凍結）で `--emit-amends` → 1・標準出力は見出しと差分 1 行だけ・違反は標準エラー。
#[test]
fn freeze_emit_amends_prints_the_field_diff() {
    let w = Work::new("emit-diff");
    bump_title(&w.dir());
    let out = w.check(&["--emit-amends"]);
    let (so, se) = (text(&out.stdout), text(&out.stderr));
    assert_eq!(out.status.code(), Some(1), "{so}\n{se}");
    assert_eq!(
        so,
        format!(
            "# 比較元 anchor v1.0 → 現行（版 v1.1）の欄単位の差分。amends にそのまま貼る（version は現行 meta.version の値＝上げる版）\n\
- {{\"target\": \"P-1\", \"field\": \"title\", \"version\": \"v1.1\", \"previous_text\": \"{OLD_TITLE}\", \"new_text\": \"{NEW_TITLE}\"}}\n"
        )
    );
    assert!(se.contains("[A-2]"), "{se}");
}

/// (2) 変異なしで `--emit-amends` → 0・標準出力は見出し 1 行だけ。
#[test]
fn freeze_emit_amends_unmutated_is_heading_only() {
    let w = Work::new("emit-none");
    let out = w.check(&["--emit-amends"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        text(&out.stdout),
        "# 比較元 anchor v1.0 → 現行（版 v1.0）の欄単位の差分。amends にそのまま貼る（version は現行 meta.version の値＝上げる版）\n"
    );
}

/// (3) 変異なしで `--freeze-anchor` → 1・「新しくない」・anchors/ は不変。
#[test]
fn freeze_same_version_is_refused() {
    let w = Work::new("same-version");
    let before = anchors_snapshot(&w.dir());
    let out = w.check(&["--freeze-anchor"]);
    let se = text(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{se}");
    assert!(
        se.contains(
            "版 v1.0 は最新 anchor v1.0 より新しくない（同じ版は上書きしない・版を上げてから）"
        ),
        "{se}"
    );
    assert_eq!(anchors_snapshot(&w.dir()), before);
}

/// (4) 合成した改訂を commit して `--freeze-anchor` → 0・v1.1 の anchor と索引の追記・続けて旗なし → 0。
#[test]
fn freeze_synthetic_amendment_is_frozen() {
    let w = Work::new("synthetic");
    synthetic_amendment(&w.dir(), NEW_TITLE);
    w.commit();
    let out = w.check(&["--freeze-anchor"]);
    let se = text(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{}\n{se}", text(&out.stdout));
    assert!(
        se.contains("凍結した: ")
            && se.contains("anchors/constitution-v1.1.yaml（条 27・previous v1.0・承認 1 件）・索引 index.yaml に追記"),
        "{se}"
    );
    let anchor = fs::read_to_string(w.dir().join("anchors/constitution-v1.1.yaml")).unwrap();
    let digest = anchor
        .lines()
        .find_map(|l| l.strip_prefix("\"digest\": \""))
        .and_then(|d| d.strip_suffix('"'))
        .expect("anchor に digest が無い")
        .to_string();
    assert_eq!(digest.len(), 64);
    assert!(anchor.contains("\"adr\": \"ADR-5\""), "{anchor}");
    let index = fs::read_to_string(w.dir().join("anchors/index.yaml")).unwrap();
    assert!(
        index.ends_with(&format!(
            "- \"version\": \"v1.0\"\n    \"previous\": null\n    \"digest\": \"acb52acd04b5d3a1feaf9ad5f0138f7614ce31964144b46ead914bde86e866ed\"\n  - \"version\": \"v1.1\"\n    \"previous\": \"v1.0\"\n    \"digest\": \"{digest}\"\n"
        )),
        "{index}"
    );
    let again = w.check(&[]);
    assert_eq!(
        again.status.code(),
        Some(0),
        "{}\n{}",
        text(&again.stdout),
        text(&again.stderr)
    );
}

/// (5) (4) の amends の new_text を 1 字違えて `--freeze-anchor` → 1・「凍結しない」・v1.1 の anchor は作られない。
#[test]
fn freeze_mismatched_amends_is_not_frozen() {
    let w = Work::new("mismatch");
    synthetic_amendment(&w.dir(), "判断する道具を作らない（改定）");
    w.commit();
    let before = anchors_snapshot(&w.dir());
    let out = w.check(&["--freeze-anchor"]);
    let se = text(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{}\n{se}", text(&out.stdout));
    assert!(
        se.contains("凍結しない — 違反 1 件・まだ分からない 0 件を直してから --freeze-anchor"),
        "{}\n{se}",
        text(&out.stdout)
    );
    assert!(!w.dir().join("anchors/constitution-v1.1.yaml").exists());
    assert_eq!(anchors_snapshot(&w.dir()), before);
}
