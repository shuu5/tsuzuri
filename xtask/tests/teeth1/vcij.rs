//! 行 v-ci の歯: 歯の写しの孤児の照らし task insta-refs（xtask の src/snaprefs.rs）の判じと走り。daily の段 insta-refs が撃つ
//! （check が環境変数で読んだ写しの path を file に残し、段が和で照らす・行 t-daily-deny）。
//! 行 t-ci-stop の歯: push ごとの CI を退けたこと（持ち主の決め D3）。根の .github/workflows/ に file が無く、字は .github/retired/ci.yml に在り、
//! xtask の src と tests の .rs のどれも、この file の外で ci.yml の path を読まない。
#![cfg(test)]

#[path = "../../src/snaprefs.rs"]
mod snaprefs;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

fn read_root(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

#[test]
fn vcij_refs_wiring() {
    let main = read_root("xtask/src/main.rs");
    assert_eq!(
        main.matches("\nmod snaprefs;\n").count(),
        1,
        "main.rs の mod の行"
    );
    assert_eq!(
        main.matches("Some(\"insta-refs\") => exit_code(insta_refs(")
            .count(),
        1,
        "main.rs の task insta-refs の腕"
    );
}

fn strings(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

#[test]
fn vcij_refs_judge() {
    let tracked = strings(&[
        "scribe2/a/snapshots/x.snap",
        "scribe2/b/snapshots/y.snap",
        "crates/c/snapshots/z.snap",
        "scribe2/a/x.rs",
    ]);
    let lines = strings(&[
        "/r/scribe2/a/snapshots/x.snap",
        "",
        "/r/scribe2/b/tests/./../snapshots/y.snap",
        "/r/scribe2/a/snapshots/x.snap",
    ]);
    let refs = snaprefs::referenced("/r", &lines).expect("根の下の行");
    assert_eq!(refs.len(), 2, "重なる行は 1 つ");
    assert_eq!(
        snaprefs::judge(&tracked, &refs),
        Ok("写し 2 本・読まれた path 2 本・読まれない写し 0".to_string()),
        "scribe2/ の外の写しと .rs は見ない"
    );
    let one: BTreeSet<String> = refs.iter().take(1).cloned().collect();
    assert!(
        snaprefs::judge(&tracked, &one).is_err(),
        "読まれない写しを通す"
    );
    for extra in [
        "scribe2/a/snapshots/x.snap.new",
        "scribe2/a/.x.pending-snap",
    ] {
        let mut more = tracked.clone();
        more.push(extra.to_string());
        assert!(snaprefs::judge(&more, &refs).is_err(), "{extra} を通す");
    }
    let none = strings(&["crates/c/snapshots/z.snap"]);
    assert!(snaprefs::judge(&none, &refs).is_err(), "写しの無い木を通す");
    assert!(
        snaprefs::referenced("/r", &strings(&["/other/scribe2/a/snapshots/x.snap"])).is_err(),
        "根の下でない行を通す"
    );
    assert!(
        snaprefs::referenced("/r", &strings(&["/rx/scribe2/a/snapshots/x.snap"])).is_err(),
        "根の名を頭に持つだけの行を通す"
    );
    assert!(
        snaprefs::referenced("/r", &strings(&["/r/../r/scribe2/a/snapshots/x.snap"])).is_ok(),
        "節 .. を解いて根の下"
    );
    assert!(
        snaprefs::referenced("/r", &strings(&["/r/../../r/scribe2/a/snapshots/x.snap"])).is_err(),
        "根より上へ打ち消す行を通す"
    );
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .is_ok_and(|o| o.status.success());
    assert!(ok, "git {args:?}");
}

fn put(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().expect("親の dir")).expect("dir を作る");
    std::fs::write(path, text).expect("file を書く");
}

#[test]
fn vcij_refs_run() {
    let root = std::env::temp_dir().join(format!("vcij-refs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for rel in ["scribe2/a/snapshots/x.snap", "scribe2/b/snapshots/y.snap"] {
        put(&root.join(rel), "---\n");
    }
    git(&root, &["init", "-q"]);
    git(&root, &["add", "-A"]);
    let refs = root.join("refs");
    let top = root.to_string_lossy().to_string();
    assert_eq!(snaprefs::run(&root, &refs).0, 1, "下ろした dir が無い");
    put(
        &refs.join("p1/insta-refs.txt"),
        &format!("{top}/scribe2/a/snapshots/x.snap\n"),
    );
    let (rc, lines) = snaprefs::run(&root, &refs);
    assert_eq!(rc, 1, "役 1 つの和は y を読まない: {lines:?}");
    assert_eq!(lines, ["どの歯も読まない写し: scribe2/b/snapshots/y.snap"]);
    put(
        &refs.join("p2/insta-refs.txt"),
        &format!("{top}/scribe2/b/snapshots/y.snap\n"),
    );
    let (rc, lines) = snaprefs::run(&root, &refs);
    assert_eq!(rc, 0, "2 つの役の和: {lines:?}");
    assert_eq!(
        lines,
        ["役の file 2 本・写し 2 本・読まれた path 2 本・読まれない写し 0"]
    );
    put(&root.join("scribe2/c/snapshots/w.snap"), "---\n");
    assert_eq!(snaprefs::run(&root, &refs).0, 0, "追跡されない写しは見ない");
    git(&root, &["add", "-A"]);
    assert_eq!(
        snaprefs::run(&root, &refs).0,
        1,
        "追跡された孤児の写しを通す"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// dir の下の file の path を全部（dir が無ければ空）。
fn files_under(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries {
        let path = entry.expect("dir の項目").path();
        if path.is_dir() {
            out.extend(files_under(&path));
        } else {
            out.push(path);
        }
    }
    out
}

/// 根の .vessel.toml の行のうち、字のとおり want の行の数。
fn vessel_lines(want: &str) -> usize {
    read_root(".vessel.toml")
        .lines()
        .filter(|l| *l == want)
        .count()
}

/// 根の .vessel.toml の行のうち、鍵 key の行（字 key = で始まる行）の数。
fn vessel_keys(key: &str) -> usize {
    let head = format!("{key} = ");
    read_root(".vessel.toml")
        .lines()
        .filter(|l| l.starts_with(&head))
        .count()
}

#[test]
fn cistop_push_ci_is_retired() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let workflows: Vec<PathBuf> = files_under(&root.join(".github/workflows"))
        .into_iter()
        .filter(|p| {
            p.to_string_lossy().ends_with(".yml") || p.to_string_lossy().ends_with(".yaml")
        })
        .collect();
    assert!(workflows.is_empty(), "根の .github/workflows/ の下: {workflows:?}");
    assert!(
        root.join(".github/retired/ci.yml").is_file(),
        ".github/retired/ci.yml は file"
    );
    let retired = read_root(".github/retired/ci.yml");
    for want in [
        "cargo run -q -p xtask -- check",
        "cargo deny check -D unmatched-skip -D advisory-not-detected",
    ] {
        assert!(retired.contains(want), "退けた ci.yml に字 {want} が無い");
    }
    assert_eq!(vessel_lines("ci-watch = false"), 1, "鍵 ci-watch の行");
    assert_eq!(vessel_keys("ci-watch"), 1, "鍵 ci-watch の行は 1 つだけ");
    assert_eq!(vessel_lines("remote = \"origin\""), 1, "鍵 remote の行");
    assert_eq!(vessel_keys("remote"), 1, "鍵 remote の行は 1 つだけ");
}

#[test]
fn cistop_no_file_reads_the_workflow() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let needle = concat!("\"", ".github/workflows/ci.yml");
    let mut readers: Vec<PathBuf> = Vec::new();
    for dir in ["src", "tests"] {
        for path in files_under(&root.join(dir)) {
            let is_rs = path.extension().is_some_and(|e| e == "rs");
            let is_self = path.file_name().is_some_and(|n| n == "vcij.rs");
            if !is_rs || is_self {
                continue;
            }
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
            if text.contains(needle) {
                readers.push(path);
            }
        }
    }
    assert!(readers.is_empty(), "ci.yml の path を読む file: {readers:?}");
}
