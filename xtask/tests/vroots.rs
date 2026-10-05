//! 行 v-roots の歯（接頭辞 vroots_・設計ノート surface-wave29c・判断の記録 ADR-33 の決定 (12)）: 根の器の宣言 .vessel.toml の任意 key
//! crate-roots が folio2/crates/ と scribe2/crates/ の 2 項目をこの順で持つ 1 行で、repo の追跡する package の manifest の dir が、根の開発の
//! 道具 xtask の 1 つを除いて、器の固定の根 crates/ か宣言した根の直下の dir であること（器の追随の再 gate の面・受付の上限の余地・歯の在りかの
//! 読みが、器の crate を crates/ の直下の crate と同じに読む）。否定の見本は正しい見本から 1 句だけ替える。外の依存を使わない。
#![cfg(test)]

use std::path::PathBuf;
use std::process::Command;

/// 名乗りの 1 行の字（key と値の間は空白 1 つずつ）。
const ROOTS: &str = "crate-roots = [\"folio2/crates/\", \"scribe2/crates/\"]";

/// 器の固定の根（宣言に依らず常に在る）。
const FIXED: &str = "crates/";

/// 根の外に在ってよい package の dir（根の開発の道具）。
const OUTSIDE: [&str; 1] = ["xtask"];

/// repo の根（xtask の manifest の dir の 1 つ上）。
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// repo の根からの相対の path の file の字。
fn read(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// key が crate-roots の行（前後の空白を除いた字・file の順・注の行は key の字が # で始まるので当たらない）。
fn roots_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| line.split('=').next().map(str::trim) == Some("crate-roots"))
        .collect()
}

/// 行の値の配列の項目（二重引用符の中の字・順のまま）。
fn items(line: &str) -> Vec<&str> {
    line.split_once('=')
        .map(|(_, value)| value.split('"').skip(1).step_by(2).collect())
        .unwrap_or_default()
}

/// package の dir のうち、固定の根と宣言した根のどれの直下の dir でもない dir（字の順）。
fn outside(declared: &[&str], dirs: &[String]) -> Vec<String> {
    let under = |dir: &str| {
        std::iter::once(FIXED)
            .chain(declared.iter().copied())
            .any(|root| {
                dir.strip_prefix(root)
                    .is_some_and(|name| !name.is_empty() && !name.contains('/'))
            })
    };
    let mut out: Vec<String> = dirs.iter().filter(|dir| !under(dir)).cloned().collect();
    out.sort();
    out
}

/// repo の追跡する Cargo.toml のうち行 [package] を持つ file の dir（repo の根からの相対・字の順）。
fn package_dirs() -> Vec<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo_root())
        .args(["ls-files", "-z", "--", "*Cargo.toml"])
        .output()
        .expect("git ls-files を撃つ");
    assert!(out.status.success(), "git ls-files の rc");
    let listed = String::from_utf8_lossy(&out.stdout).into_owned();
    let mut dirs: Vec<String> = listed
        .split('\0')
        .filter(|path| {
            !path.is_empty() && read(path).lines().any(|line| line.trim() == "[package]")
        })
        .map(|path| {
            path.trim_end_matches("Cargo.toml")
                .trim_end_matches('/')
                .to_string()
        })
        .collect();
    dirs.sort();
    dirs
}

#[test]
fn vroots_declaration_adds_the_vessel_crates_root() {
    let text = read(".vessel.toml");
    assert_eq!(roots_lines(&text), [ROOTS]);
    assert_eq!(items(ROOTS), ["folio2/crates/", "scribe2/crates/"]);
}

#[test]
fn vroots_every_crate_sits_under_a_root() {
    let text = read(".vessel.toml");
    let lines = roots_lines(&text);
    let declared = lines.first().map(|line| items(line)).unwrap_or_default();
    let dirs = package_dirs();
    assert_eq!(outside(&declared, &dirs), OUTSIDE, "{dirs:?}");
    for root in &declared {
        assert!(
            dirs.iter().any(|dir| dir.starts_with(root)),
            "宣言した根 {root} の下に crate が無い"
        );
    }
    // 否定の見本: 宣言から scribe2/crates/ だけを外すと、器の 3 つの crate の dir が根の外に出る。
    assert_eq!(
        outside(&["folio2/crates/"], &dirs),
        [
            "scribe2/crates/scribe2",
            "scribe2/crates/scribe2-boundary",
            "scribe2/crates/scribe2-xtask",
            "xtask"
        ]
    );
}

#[test]
fn vroots_one_clause_changes_are_refused() {
    let good = format!("schema = 1\n{ROOTS}\n");
    assert_eq!(roots_lines(&good), [ROOTS]);
    let cases: [(String, Vec<&str>); 4] = [
        (
            "schema = 1\ncrate-roots = [\"folio2/crates/\"]\n".to_string(),
            vec!["folio2/crates/"],
        ),
        (
            "schema = 1\ncrate-roots = [\"scribe2/crates/\", \"folio2/crates/\"]\n".to_string(),
            vec!["scribe2/crates/", "folio2/crates/"],
        ),
        (
            "schema = 1\ncrate-roots = [\"folio2/crates/\", \"scribe2/crates\"]\n".to_string(),
            vec!["folio2/crates/", "scribe2/crates"],
        ),
        (format!("schema = 1\n# {ROOTS}\n"), Vec::new()),
    ];
    for (text, want) in cases {
        let lines = roots_lines(&text);
        let got = lines.first().map(|line| items(line)).unwrap_or_default();
        assert_eq!(got, want, "{text}");
        assert_ne!(lines, [ROOTS], "{text}");
    }
    // 同じ行を 2 度書いた宣言は 1 行と読まない。
    let twice = format!("schema = 1\n{ROOTS}\n{ROOTS}\n");
    assert_eq!(roots_lines(&twice), [ROOTS, ROOTS]);
    // 根の直下の dir の 1 段下（crate の dir の下の dir）は根の直下の crate と読まない。
    let nested = vec!["scribe2/crates/scribe2/sub".to_string()];
    assert_eq!(outside(&["scribe2/crates/"], &nested), nested);
}
