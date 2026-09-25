//! 機構がまだ無い条の 1 行の歯（便 131・docs/design/delivery-131.md §1 (c)・ADR-23 決定 (3)・FR5）。folio は実行 file の crate なので
//! 命令を撃つ。写しは一時 dir に design-intent/ として作り、置き場の親の contracts/ に器の導出 file を写し、git init と 1 commit を行う
//! （tests/freeze_root.rs と同じ作り方・歯の終わりに消す）。
//! 凍結 anchor（P-10.1）は土台の一覧の定数 FLOOR_BASE_LIST（手で書く・folio の code から組まない）。folio2 自身の憲法の一覧の定数
//! FOLIO2_LIST は、条の機構の段の欄の変更を審査に引き出すための意図した 2 つ目の写しである（実の正本に依らない歯の向きの例外・ADR-23 決定 (3)）。
//! 1. 土台の 14 本の行と要約の直前と種別の絞り／2. 0 本の写しと骨格で行が無い／3. 断りの道で行が無い／4. folio2 自身の 6 本。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FLOOR_BASE: &str = "tests/fixtures/floor_base/design-intent";
const HEAD: &str = "# 機構がまだ無い条（床の判定の外・憲法 schema.mechanism_live_rule）: ";
const PASS: &str = "folio check: 合格（違反 0・まだ分からない 0）";
/// 床の凍結の土台（第 1.0 版）の一覧（delivery-131.md §1 (a) の 4）。
const FLOOR_BASE_LIST: &str = "# 機構がまだ無い条（床の判定の外・憲法 schema.mechanism_live_rule）: P-1（M0）・P-3（M0）・P-4（M0）・P-7（delivery-0）・P-10（M0）・P-11（delivery-0）・P-12（M0）・P-13（M0）・P-15（M1）・P-17（delivery-0）・P-18（delivery-0）・A-3（M1）・N-1（M0）・N-5（M0）";
/// folio2 自身の憲法（第 1.4 版）の一覧（delivery-131.md §1 (b) の 3）。
const FOLIO2_LIST: &str = "# 機構がまだ無い条（床の判定の外・憲法 schema.mechanism_live_rule）: P-11（M1）・P-13（M1）・P-15（M1）・P-17（M1）・P-18（M1）・A-3（M1）";

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
    fn root(case: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("folio-f131-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        root
    }

    /// 土台を design-intent/ として写し、器の導出 file を写し、`prep` を当ててから git init と 1 commit。
    fn floor_base(case: &str, prep: impl FnOnce(&Path)) -> Work {
        let root = Work::root(case);
        copy_tree(&repo_root().join(FLOOR_BASE), &root.join("design-intent"));
        fs::create_dir_all(root.join("contracts")).unwrap();
        fs::copy(
            repo_root().join("contracts/schema.toml"),
            root.join("contracts/schema.toml"),
        )
        .unwrap();
        prep(&root.join("design-intent"));
        let w = Work { root };
        w.commit();
        w
    }

    /// folio init の骨格を design-intent/ に作り、git init と 1 commit。
    fn skeleton(case: &str) -> Work {
        let root = Work::root(case);
        fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        let w = Work { root };
        let out = folio(&["init"], &w.dir(), &[]);
        assert_eq!(out.status.code(), Some(0), "{}", show(&out));
        w.commit();
        w
    }

    fn commit(&self) {
        git(&self.root, &["init", "-q"]);
        git(&self.root, &["add", "-A"]);
        git(&self.root, &["commit", "-q", "--allow-empty", "-m", "fixture"]);
    }

    fn dir(&self) -> PathBuf {
        self.root.join("design-intent")
    }

    fn check(&self, flags: &[&str]) -> Output {
        folio(&["check"], &self.dir(), flags)
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn folio(head: &[&str], dir: &Path, tail: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(head)
        .arg("--dir")
        .arg(dir)
        .args(tail)
        .output()
        .expect("folio を起動できない")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn show(out: &Output) -> String {
    format!("{}\n{}", text(&out.stdout), text(&out.stderr))
}

fn lines(bytes: &[u8]) -> Vec<String> {
    text(bytes).lines().map(str::to_string).collect()
}

/// 標準エラーの一覧の行（頭の字で始まる）。
fn listed(out: &Output) -> Vec<String> {
    lines(&out.stderr)
        .into_iter()
        .filter(|l| l.starts_with(HEAD))
        .collect()
}

fn edit(path: &Path, f: impl FnOnce(&str) -> String) {
    let before = fs::read_to_string(path).unwrap();
    let after = f(&before);
    assert_ne!(before, after, "置き換える字が無い（前提が崩れた）: {}", path.display());
    fs::write(path, after).unwrap();
}

#[test]
fn f131_floor_base_lists_the_articles_before_the_summary() {
    let w = Work::floor_base("base", |_| {});
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    assert_eq!(lines(&out.stdout), [PASS], "{}", show(&out));
    assert_eq!(lines(&out.stderr), [FLOOR_BASE_LIST], "{}", show(&out));

    let amends = w.check(&["--emit-amends"]);
    assert_eq!(amends.status.code(), Some(0), "{}", show(&amends));
    assert_eq!(lines(&amends.stderr), [FLOOR_BASE_LIST, PASS], "{}", show(&amends));
    assert!(!text(&amends.stdout).contains(HEAD), "{}", show(&amends));

    // 種別の絞り: human-review の最初の条（P-16）の live を M1 にしても一覧は 14 本のまま
    let review = Work::floor_base("review", |dir| {
        edit(&dir.join("constitution.yaml"), |s| {
            s.replacen("{kind: human-review, live: now,", "{kind: human-review, live: M1,", 1)
        });
    });
    let out = review.check(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    assert_eq!(lines(&out.stdout), [PASS], "{}", show(&out));
    assert_eq!(listed(&out), [FLOOR_BASE_LIST], "{}", show(&out));
}

#[test]
fn f131_no_line_when_no_article_waits_for_its_mechanism() {
    let w = Work::floor_base("now", |dir| {
        edit(&dir.join("constitution.yaml"), |s| {
            let mut t = s.to_string();
            for live in ["M0", "delivery-0", "M1", "adr"] {
                t = t.replace(&format!(", live: {live},"), ", live: now,");
            }
            t
        });
    });
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    assert_eq!(lines(&out.stdout), [PASS], "{}", show(&out));
    assert!(out.stderr.is_empty(), "{}", show(&out));

    let sk = Work::skeleton("skeleton");
    let out = sk.check(&[]);
    assert_eq!(out.status.code(), Some(2), "{}", show(&out));
    assert_eq!(
        lines(&out.stdout),
        ["folio check: まだ分からない（違反 0・まだ分からない 2）"],
        "{}",
        show(&out)
    );
    assert!(listed(&out).is_empty(), "{}", show(&out));
}

#[test]
fn f131_a_refused_freeze_prints_no_line() {
    let w = Work::floor_base("refused", |_| {});
    let out = w.check(&["--freeze-anchor"]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    assert!(out.stdout.is_empty(), "{}", show(&out));
    assert!(listed(&out).is_empty(), "{}", show(&out));
    assert!(text(&out.stderr).contains("同じ版は上書きしない"), "{}", show(&out));
}

#[test]
fn f131_folio2_constitution_lists_the_six_articles() {
    let out = folio(&["check"], &repo_root().join("design-intent"), &[]);
    assert_eq!(listed(&out), [FOLIO2_LIST], "{}", show(&out));
}
