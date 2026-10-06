//! 行 v-skip の歯: 入れ子の段を省くかの判じ（xtask の src/nested/skip.rs・判断の記録 ADR-34）を一時の git の repo の fixture で撃ち、
//! 入力の path の一覧と環境変数の名と行の字を固定し、nested の段の呼びを字で読む。
#![cfg(test)]

#[path = "../../src/nested/skip.rs"]
mod skip;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::common::{read_root, strings};
use skip::{BASE_REF, FORCE_ENV, SHARED, Verdict};

/// fixture の base の木（入れ子の dir s2 と s3・入力の外の crate と xtask の歯・.gitignore）。
const BASE_FILES: &[(&str, &str)] = &[
    ("Cargo.toml", "[workspace]\n"),
    ("s2/Cargo.toml", "[workspace]\n"),
    ("s2/src/lib.rs", "// s2\n"),
    ("s3/Cargo.toml", "[workspace]\n"),
    ("s3/src/lib.rs", "// s3\n"),
    ("crates/a/src/lib.rs", "// a\n"),
    ("rust-toolchain.toml", "[toolchain]\n"),
    ("xtask/Cargo.toml", "[package]\n"),
    ("xtask/src/main.rs", "// x\n"),
    ("xtask/tests/t.rs", "// t\n"),
    (".gitignore", "/s2/ignored/\n"),
];

/// 一時の dir（名に pid と字 tag）を作り直す。
fn fresh(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tsuzuri-vskip-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("一時の dir");
    dir
}

fn put(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().expect("親の dir")).expect(rel);
    std::fs::write(&path, body).expect(rel);
}

/// fixture の repo で git を撃ち、標準出力の字を返す（落ちれば歯が落ちる・家と system の git の設定を読まない）。
fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_AUTHOR_NAME", "fx")
        .env("GIT_AUTHOR_EMAIL", "fx@example.invalid")
        .env("GIT_COMMITTER_NAME", "fx")
        .env("GIT_COMMITTER_EMAIL", "fx@example.invalid")
        .output()
        .expect("git を起動する");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn commit(dir: &Path, msg: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", msg]);
}

/// base の木を 1 commit 置いて BASE_REF をそこへ向けた repo（HEAD は base のまま）。
fn repo(tag: &str) -> PathBuf {
    let dir = fresh(tag);
    git(&dir, &["init", "-q", "-b", "main"]);
    for (rel, body) in BASE_FILES {
        put(&dir, rel, body);
    }
    commit(&dir, "base");
    git(&dir, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    dir
}

/// base の上に入力の外の commit を 1 つ置いた repo（判じは省く側）。
fn ahead(tag: &str) -> PathBuf {
    let dir = repo(tag);
    put(&dir, "crates/a/src/lib.rs", "// a2\n");
    commit(&dir, "outside");
    dir
}

fn base(dir: &Path) -> String {
    git(dir, &["rev-parse", BASE_REF])
}

fn touches(dir: &Path) -> Verdict {
    Verdict::Full(format!(
        "base {} からの差が入れ子の入力に触れる",
        &base(dir)[..12]
    ))
}

#[test]
fn vskip_fixed_inputs_and_env() {
    assert_eq!(
        SHARED,
        [
            "rust-toolchain",
            "rust-toolchain.toml",
            ".cargo/",
            "xtask/src/",
            "xtask/Cargo.toml"
        ]
    );
    assert_eq!(
        skip::inputs("scribe2"),
        strings(&[
            "scribe2/",
            "rust-toolchain",
            "rust-toolchain.toml",
            ".cargo/",
            "xtask/src/",
            "xtask/Cargo.toml"
        ])
    );
    assert_eq!(FORCE_ENV, "TSUZURI_CHECK_NESTED_ALL");
    assert_eq!(BASE_REF, "origin/main");
    let sha = "0123456789abcdef0123456789abcdef01234567";
    assert_eq!(
        skip::line("scribe2", &Verdict::Skip(sha.to_string())),
        "nested scribe2: 省いた（base 0123456789ab からの差が入れ子の入力に触れない）"
    );
    assert_eq!(
        skip::line("scribe2", &Verdict::Full("理由".to_string())),
        "nested scribe2: 全部を撃つ（理由）"
    );
}

#[test]
fn vskip_outside_change_skips() {
    let dir = ahead("out");
    put(&dir, "xtask/tests/t.rs", "// t2\n");
    put(&dir, "s2x/lib.rs", "// 名の頭だけが同じ dir\n");
    put(&dir, "rust-toolchain.toml.bak", "x\n");
    commit(&dir, "tests and look-alikes");
    put(&dir, "docs/new.md", "x\n");
    put(&dir, "s2/ignored/out.txt", "x\n");
    assert_eq!(skip::decide(&dir, "s2", false), Verdict::Skip(base(&dir)));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn vskip_inputs_touched_run_all() {
    let cases = [
        "s2/src/lib.rs",
        "s2/new.rs",
        "rust-toolchain",
        "rust-toolchain.toml",
        ".cargo/config.toml",
        "xtask/src/main.rs",
        "xtask/src/new.rs",
        "xtask/Cargo.toml",
    ];
    for (n, rel) in cases.iter().enumerate() {
        let dir = ahead(&format!("in{n}"));
        put(&dir, rel, "// 直した\n");
        commit(&dir, rel);
        assert_eq!(skip::decide(&dir, "s2", false), touches(&dir), "{rel}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn vskip_renames_run_all() {
    let cases = [
        ("s2/src/lib.rs", "crates/a/src/moved.rs", true),
        ("crates/a/src/lib.rs", "s2/src/moved.rs", true),
        ("rust-toolchain.toml", "docs/toolchain.toml", true),
        ("crates/a/src/lib.rs", "crates/a/src/moved.rs", false),
    ];
    for (n, (from, to, full)) in cases.iter().enumerate() {
        let dir = ahead(&format!("mv{n}"));
        std::fs::create_dir_all(dir.join(to).parent().expect("親の dir")).expect(to);
        git(&dir, &["mv", from, to]);
        commit(&dir, "move");
        let want = if *full {
            touches(&dir)
        } else {
            Verdict::Skip(base(&dir))
        };
        assert_eq!(skip::decide(&dir, "s2", false), want, "{from} → {to}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn vskip_untracked_and_worktree_run_all() {
    let untracked = ["s2/new.rs", ".cargo/config.toml", "xtask/src/new.rs"];
    for (n, rel) in untracked.iter().enumerate() {
        let dir = ahead(&format!("un{n}"));
        put(&dir, rel, "// 未追跡\n");
        assert_eq!(skip::decide(&dir, "s2", false), touches(&dir), "{rel}");
        let _ = std::fs::remove_dir_all(&dir);
    }
    let dir = ahead("wt");
    put(&dir, "s2/src/lib.rs", "// 作業木だけの直し\n");
    assert_eq!(skip::decide(&dir, "s2", false), touches(&dir), "作業木");
    git(&dir, &["add", "s2/src/lib.rs"]);
    assert_eq!(skip::decide(&dir, "s2", false), touches(&dir), "index");
    git(&dir, &["reset", "-q", "--", "s2/src/lib.rs"]);
    git(&dir, &["checkout", "-q", "--", "s2/src/lib.rs"]);
    assert_eq!(skip::decide(&dir, "s2", false), Verdict::Skip(base(&dir)));
    std::fs::remove_file(dir.join("rust-toolchain.toml")).expect("消す");
    assert_eq!(skip::decide(&dir, "s2", false), touches(&dir), "消した");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn vskip_no_own_commit_runs_all() {
    let dir = repo("own0");
    put(&dir, "crates/a/src/lib.rs", "// 未 commit の外の直し\n");
    let b = base(&dir);
    let want = format!("base {} から HEAD までに自分の commit が無い", &b[..12]);
    assert_eq!(skip::decide(&dir, "s2", false), Verdict::Full(want));
    git(&dir, &["checkout", "-q", "--", "crates/a/src/lib.rs"]);
    put(&dir, "docs/x.md", "x\n");
    commit(&dir, "pushed");
    git(&dir, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    git(&dir, &["checkout", "-q", "--detach", "HEAD~1"]);
    let older = git(&dir, &["rev-parse", "HEAD"]);
    let want = format!("base {} から HEAD までに自分の commit が無い", &older[..12]);
    assert_eq!(
        skip::decide(&dir, "s2", false),
        Verdict::Full(want),
        "後ろの HEAD"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn vskip_unreadable_base_runs_all() {
    let dir = ahead("nobase");
    git(&dir, &["update-ref", "-d", "refs/remotes/origin/main"]);
    let want = Verdict::Full(format!("base を読めない（merge-base HEAD {BASE_REF}）"));
    assert_eq!(skip::decide(&dir, "s2", false), want);
    git(&dir, &["checkout", "-q", "--orphan", "other"]);
    git(&dir, &["commit", "-q", "-m", "orphan"]);
    git(&dir, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    git(&dir, &["checkout", "-q", "main"]);
    assert_eq!(skip::decide(&dir, "s2", false), want, "共通の祖先が無い");
    assert_eq!(
        skip::decide(&dir.join("crates"), "a", false),
        Verdict::Full("根が git の木の根でない".to_string())
    );
    let _ = std::fs::remove_dir_all(&dir);
    let plain = fresh("plain");
    put(&plain, "s2/Cargo.toml", "[workspace]\n");
    assert_eq!(
        skip::decide(&plain, "s2", false),
        Verdict::Full("根が git の木でない".to_string())
    );
    let _ = std::fs::remove_dir_all(&plain);
}

#[test]
fn vskip_force_env_runs_all() {
    let dir = ahead("force");
    assert_eq!(skip::decide(&dir, "s2", false), Verdict::Skip(base(&dir)));
    let want = Verdict::Full(format!("{FORCE_ENV} が在る"));
    assert_eq!(skip::decide(&dir, "s2", true), want);
    let _ = std::fs::remove_dir_all(&dir);
    let plain = fresh("forceplain");
    assert_eq!(skip::decide(&plain, "s2", true), want, "git を読まない");
    let _ = std::fs::remove_dir_all(&plain);
}

#[test]
fn vskip_per_dir_and_zero_dirs() {
    let dir = ahead("dirs");
    put(&dir, "s3/src/lib.rs", "// s3 を直した\n");
    commit(&dir, "s3");
    let b = base(&dir);
    let (keep, lines) = skip::select(&strings(&["s2", "s3"]), |d| skip::decide(&dir, d, false));
    assert_eq!(keep, strings(&["s3"]));
    assert_eq!(
        lines,
        [
            format!(
                "nested s2: 省いた（base {} からの差が入れ子の入力に触れない）",
                &b[..12]
            ),
            format!(
                "nested s3: 全部を撃つ（base {} からの差が入れ子の入力に触れる）",
                &b[..12]
            ),
        ]
    );
    let _ = std::fs::remove_dir_all(&dir);
    let (keep, lines) = skip::select(&[], |d| panic!("dir が無ければ判じない: {d}"));
    assert!(keep.is_empty() && lines.is_empty());
    let seen = RefCell::new(Vec::new());
    let (keep, lines) = skip::select(&strings(&["a", "b", "c"]), |d| {
        seen.borrow_mut().push(d.to_string());
        if d == "b" {
            Verdict::Skip("b".to_string())
        } else {
            Verdict::Full("x".to_string())
        }
    });
    assert_eq!(seen.into_inner(), strings(&["a", "b", "c"]));
    assert_eq!(keep, strings(&["a", "c"]));
    assert_eq!(lines.len(), 3);
}

#[test]
fn vskip_run_wiring() {
    let src = read_root("xtask/src/nested.rs");
    assert_eq!(src.lines().filter(|l| *l == "mod skip;").count(), 1);
    let (_, run) = src.split_once("pub fn run(").expect("fn run");
    let (run, _) = run.split_once("\n}\n").expect("fn run の終わり");
    let order = [
        "emit_err(&format!(\"xtask check: {}\", summary(&dirs)));",
        "let forced = std::env::var_os(skip::FORCE_ENV).is_some();",
        "let (dirs, lines) = skip::select(&dirs, |d| skip::decide(root, d, forced));",
        "for line in lines {",
        "for shot in plan(root, &base, &dirs, part) {",
    ];
    let at: Vec<usize> = order
        .iter()
        .map(|s| {
            assert_eq!(run.matches(s).count(), 1, "{s}");
            run.find(s).expect(s)
        })
        .collect();
    assert!(at.windows(2).all(|w| w[0] < w[1]), "字の順: {at:?}");
    assert!(
        run.contains(
            "for line in lines {\n        emit_err(&format!(\"xtask check: {line}\"));\n    }\n"
        ),
        "判じの行を 1 つずつ出す"
    );
    let main = read_root("xtask/src/main.rs");
    assert!(
        !main.contains(FORCE_ENV),
        "main.rs は {FORCE_ENV} を持たない"
    );
}
