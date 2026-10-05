//! 行 t-scope-on の歯（接頭辞 scopeon_・設計ノート surface-wave29c・判断の記録 ADR-64 の決定 (5)）: 根の器の宣言 .vessel.toml の任意 key
//! scope-paths が、器の dir scribe2/ の 1 項目だけを持つ 1 行で、その項目が repo の追跡する path を名指すこと（器の追随の再 gate・着地の後の
//! 検出線・差の当たりの面が、器の crate の外に在る器の組みと lint の入力の全部を読む）。器は宣言の項目が repo に在るかを知れないので、
//! 名指しの外れはこの歯が見る。否定の見本は正しい見本から 1 句だけ替える。外の依存を使わない。
#![cfg(test)]

use std::path::PathBuf;
use std::process::Command;

/// 名乗りの 1 行の字（key と値の間は空白 1 つずつ）。
const SCOPE: &str = "scope-paths = [\"scribe2/\"]";

/// repo の根（xtask の manifest の dir の 1 つ上）。
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// 注でない行のうち key が scope-paths の行（前後の空白を除いた字・file の順）。
fn scope_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter(|line| line.split('=').next().map(str::trim) == Some("scope-paths"))
        .collect()
}

/// 行の値の配列の項目（二重引用符の中の字・順のまま）。
fn items(line: &str) -> Vec<&str> {
    line.split_once('=')
        .map(|(_, value)| value.split('"').skip(1).step_by(2).collect())
        .unwrap_or_default()
}

/// 項目のうち、追跡する path を名指さない物（末尾 / の項目はその下に追跡する file が無い・ほかは同じ字の追跡する file が無い）。
fn untracked<'a>(declared: &[&'a str], tracked: &[String]) -> Vec<&'a str> {
    let named = |item: &str| {
        if item.ends_with('/') {
            tracked.iter().any(|path| path.starts_with(item))
        } else {
            tracked.iter().any(|path| path == item)
        }
    };
    declared
        .iter()
        .copied()
        .filter(|item| !named(item))
        .collect()
}

/// repo の追跡する file（repo の根からの相対）。
fn tracked() -> Vec<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo_root())
        .args(["ls-files", "-z"])
        .output()
        .expect("git ls-files を撃つ");
    assert!(out.status.success(), "git ls-files の rc");
    String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect()
}

#[test]
fn scopeon_root_declaration_names_the_vessel_scope_paths() {
    let text = std::fs::read_to_string(repo_root().join(".vessel.toml"))
        .expect("根の .vessel.toml を読む");
    assert_eq!(scope_lines(&text), [SCOPE]);
    assert_eq!(items(SCOPE), ["scribe2/"]);
}

#[test]
fn scopeon_declared_paths_are_tracked_in_the_repo() {
    let files = tracked();
    assert_eq!(untracked(&items(SCOPE), &files), Vec::<&str>::new());
    // file の項目の照らしの対照: 根の manifest は同じ字の追跡する file。
    assert_eq!(untracked(&["Cargo.toml"], &files), Vec::<&str>::new());
    // 否定の見本: 1 字外れた dir、dir の印の無い dir、字の続く隣の dir、1 字外れた file は、どれも名指しの外れ。
    let off = ["scribe/", "scribe2", "scribe2x/", "Cargo.tom"];
    for item in off {
        assert_eq!(untracked(&[item], &files), [item], "{item}");
    }
}

#[test]
fn scopeon_one_clause_changes_are_refused() {
    assert_eq!(scope_lines(&format!("schema = 1\n{SCOPE}\n")), [SCOPE]);
    let cases: [(String, Vec<&str>); 5] = [
        (
            "schema = 1\nscope-paths = [\"scribe2\"]\n".to_owned(),
            vec!["scribe2"],
        ),
        (
            "schema = 1\nscope-paths = [\"scribe2/rules/\"]\n".to_owned(),
            vec!["scribe2/rules/"],
        ),
        (
            "schema = 1\nscope-paths = [\"scribe2/\", \"scribe2/rules/\"]\n".to_owned(),
            vec!["scribe2/", "scribe2/rules/"],
        ),
        (format!("schema = 1\n# {SCOPE}\n"), Vec::new()),
        (
            "schema = 1\nscope_paths = [\"scribe2/\"]\n".to_owned(),
            Vec::new(),
        ),
    ];
    for (text, want) in cases {
        let lines = scope_lines(&text);
        let got = lines.first().map(|line| items(line)).unwrap_or_default();
        assert_eq!(got, want, "{text}");
        assert_ne!(lines, [SCOPE], "{text}");
    }
    // 同じ行を 2 度書いた宣言は 1 行と読まない。
    assert_eq!(
        scope_lines(&format!("schema = 1\n{SCOPE}\n{SCOPE}\n")),
        [SCOPE, SCOPE]
    );
}
