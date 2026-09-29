//! 便 123（docs/design/delivery-123.md §1 (e)・判断の記録 ADR-16 決定 (2)(キ)(オ)）: 器の導出 file の解決先と生成区間の注の歯。
//! 器の導出 file `contracts/schema.toml` は、置き場を含む版管理の根が在ればその下、無ければ置き場の親の下で解く
//! （床の読み手と面の生成器が同じ式を共有する・根の下に無くても親へは倒さない）。凍結 anchor は手書きの設計ノート
//! tests/fixtures/design-note/derive-anchor.yaml と repo の contracts/schema.toml の写しで、生成器の出力を写さない（P-10.1）。
//! 土台: 一時 dir の下の置き場（`<一時 dir>/<段>/design-intent`）に実の design-intent の写しと上の設計ノートを置き、
//! 器の導出 file の写しの置き場と git の init / commit の位置を歯ごとに変えて `folio check` と `folio face --face note` を撃つ。
//! 1. 版管理の根が置き場の 2 段上なら、床は根の写しを読み、親の壊れた写しを読まない（床 0）。
//! 2. 同じ形で面の生成器も根の写しを読んで面を書く（面 0）。
//! 3. 版管理の根に写しが無ければ、親に在っても床と面は読めない（2）・面は書かない。
//! 4. 版管理の無い写しは置き場の親を読む（床の まだ分からない は版管理の不在だけ・面 0）。
//! 5. 置き場そのものが版管理の根なら置き場の下を読み親を読まない（床は照合の違反と読めないの両方・面は書かない）。
//! 6. folio2 の版管理の根は design-intent の親。
//! 7. 5 file の生成区間の注 top_level_note は条 id N-3 を名指さない。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ANCHOR_NOTE: &str = "tests/fixtures/design-note/derive-anchor.yaml";
const EXTERNAL: &str = "contracts/schema.toml";
const BROKEN: &str = "schema = 2\n";
const UNREADABLE: &str = "器の導出 file が読めない";
const NO_GIT: &str = "版管理（git）が無いか読めない";
const NESTED: &str = "design-intent 自体が版管理の根";

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

/// git を呼ぶ。環境変数 GIT_* は継承しない。
fn git(cwd: &Path, args: &[&str]) -> Output {
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
    out
}

fn git_commit_all(cwd: &Path) {
    git(cwd, &["init", "-q"]);
    git(cwd, &["add", "-A"]);
    git(cwd, &["commit", "-q", "-m", "fixture"]);
}

/// 一時 dir（`<root>/<段…>/design-intent`）。落ちても消す。
struct Work {
    root: PathBuf,
    dir: PathBuf,
}

impl Work {
    /// `steps` は根と置き場の間の段（空なら置き場は根の直下）。
    fn new(case: &str, steps: &[&str]) -> Work {
        let root = std::env::temp_dir().join(format!("folio-resolve-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let mut dir = root.clone();
        for s in steps {
            dir = dir.join(s);
        }
        let dir = dir.join("design-intent");
        copy_tree(&repo_root().join("design-intent"), &dir);
        fs::copy(
            repo_root().join(ANCHOR_NOTE),
            dir.join("design-note/derive-anchor.yaml"),
        )
        .unwrap();
        Work { root, dir }
    }

    fn parent(&self) -> PathBuf {
        self.dir.parent().unwrap().to_path_buf()
    }

    fn out(&self) -> PathBuf {
        self.root.join("out/note-derive-anchor.html")
    }

    /// `<at>/contracts/schema.toml` に repo の写しを置く。
    fn put_external(&self, at: &Path) {
        fs::create_dir_all(at.join("contracts")).unwrap();
        fs::copy(repo_root().join(EXTERNAL), at.join(EXTERNAL)).unwrap();
    }

    /// `<at>/contracts/schema.toml` に先頭の字の違う壊れた写しを置く。
    fn put_broken(&self, at: &Path) {
        fs::create_dir_all(at.join("contracts")).unwrap();
        fs::write(at.join(EXTERNAL), BROKEN).unwrap();
    }

    fn check(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("check")
            .arg("--dir")
            .arg(&self.dir)
            .output()
            .expect("folio を起動できない")
    }

    fn face(&self) -> Output {
        fs::create_dir_all(self.root.join("out")).unwrap();
        Command::new(env!("CARGO_BIN_EXE_folio"))
            .args(["face", "--face", "note", "--id", "derive-anchor", "--dir"])
            .arg(&self.dir)
            .arg("--out")
            .arg(self.out())
            .arg("--write")
            .output()
            .expect("folio を起動できない")
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn assert_rc(out: &Output, want: i32, what: &str) {
    assert_eq!(out.status.code(), Some(want), "{what}: {}", text(out));
}

fn assert_has(out: &Output, word: &str, what: &str) {
    assert!(text(out).contains(word), "{what} に「{word}」が無い: {}", text(out));
}

fn assert_lacks(out: &Output, word: &str, what: &str) {
    assert!(!text(out).contains(word), "{what} に「{word}」が在る: {}", text(out));
}

/// 根で git・根に repo の写し・置き場の親（根の 1 段下）に壊れた写し。
fn root_two_up(case: &str) -> Work {
    let w = Work::new(case, &["stage"]);
    w.put_external(&w.root);
    w.put_broken(&w.parent());
    git_commit_all(&w.root);
    w
}

// ── 1. 版管理の根が 2 段上なら床は根の写しを読む ──

#[test]
fn f123_floor_reads_the_copy_at_the_version_root_not_the_broken_parent() {
    let w = root_two_up("floor-root");
    let out = w.check();
    assert_lacks(&out, UNREADABLE, "folio check");
    assert_rc(&out, 0, "folio check");
}

// ── 2. 同じ形で面の生成器も根の写しを読む ──

#[test]
fn f123_face_reads_the_copy_at_the_version_root_and_writes() {
    let w = root_two_up("face-root");
    let out = w.face();
    assert_rc(&out, 0, "folio face");
    assert!(w.out().is_file(), "面を書いていない");
}

// ── 3. 根に写しが無ければ親に在っても読まない ──

#[test]
fn f123_no_copy_at_the_root_does_not_fall_back_to_the_parent() {
    let w = Work::new("no-root-copy", &["stage"]);
    w.put_external(&w.parent());
    git_commit_all(&w.root);
    let floor = w.check();
    assert_rc(&floor, 2, "folio check");
    assert_has(&floor, UNREADABLE, "folio check");
    let face = w.face();
    assert_rc(&face, 2, "folio face");
    assert_has(&face, UNREADABLE, "folio face");
    assert!(!w.out().exists(), "読めないのに面を書いた");
}

// ── 4. 版管理の無い写しは置き場の親を読む ──

#[test]
fn f123_without_version_control_the_parent_is_read() {
    let w = Work::new("no-git", &[]);
    w.put_external(&w.root);
    let floor = w.check();
    assert_rc(&floor, 2, "folio check");
    assert_has(&floor, NO_GIT, "folio check");
    assert_lacks(&floor, UNREADABLE, "folio check");
    let face = w.face();
    assert_rc(&face, 0, "folio face");
    assert!(w.out().is_file(), "面を書いていない");
}

// ── 5. 置き場そのものが版管理の根なら置き場の下を読む ──

#[test]
fn f123_when_the_place_is_the_root_it_reads_under_the_place_not_the_parent() {
    let w = Work::new("nested", &["stage"]);
    w.put_external(&w.parent());
    git_commit_all(&w.dir);
    let floor = w.check();
    assert_rc(&floor, 2, "folio check");
    assert_has(&floor, UNREADABLE, "folio check");
    assert_has(&floor, NESTED, "folio check");
    let face = w.face();
    assert_rc(&face, 2, "folio face");
    assert_has(&face, UNREADABLE, "folio face");
    assert!(!w.out().exists(), "読めないのに面を書いた");
    w.put_external(&w.dir);
    let face = w.face();
    assert_rc(&face, 0, "folio face（置き場の下に写し）");
    assert!(w.out().is_file(), "面を書いていない");
}

// ── 6. folio2 の版管理の根は design-intent の親 ──

#[test]
fn f123_folio2_version_root_is_the_parent_of_design_intent() {
    let out = git(
        &repo_root().join("design-intent"),
        &["rev-parse", "--show-toplevel"],
    );
    let top = String::from_utf8_lossy(&out.stdout).trim_end_matches(['\n', '\r']).to_string();
    let top = fs::canonicalize(top).unwrap();
    let want = fs::canonicalize(repo_root()).unwrap();
    assert_eq!(top, want, "版管理の根が design-intent の親でない");
    let parent = fs::canonicalize(repo_root().join("design-intent"))
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    assert_eq!(top, parent);
}

// ── 7. 5 file の生成区間の注は条 id を名指さない ──

#[test]
fn f123_region_top_level_notes_do_not_name_an_article_id() {
    const HEAD: &str = "最上位の節の閉じた一覧（ほかの節は床が落とす）。";
    for name in [
        "srs.yaml",
        "vocabulary.yaml",
        "rules.yaml",
        "intake.yaml",
        "graph.yaml",
    ] {
        let text = fs::read_to_string(repo_root().join("design-intent").join(name)).unwrap();
        let begin = text
            .find("# folio:schema:begin")
            .unwrap_or_else(|| panic!("{name}: 生成区間の begin が無い"));
        let end = text[begin..]
            .find("# folio:schema:end")
            .unwrap_or_else(|| panic!("{name}: 生成区間の end が無い"));
        let region = &text[begin..begin + end];
        let notes: Vec<&str> = region
            .lines()
            .filter(|l| l.trim_start().starts_with("top_level_note:"))
            .collect();
        assert_eq!(notes.len(), 1, "{name}: 生成区間の top_level_note が 1 行でない");
        let line = notes[0];
        assert!(line.contains(HEAD), "{name}: 「{HEAD}」が無い: {line}");
        assert!(!line.contains("N-3"), "{name}: 条 id N-3 を名指す: {line}");
    }
}
