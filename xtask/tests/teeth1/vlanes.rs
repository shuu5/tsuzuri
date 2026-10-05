//! 行 r-lanes-on の歯（接頭辞 vlanes_・設計ノート surface-wave29b・判断の記録 ADR-35 の決定 (1)(7) と ADR-37 の決定 (6)）: 根の器の宣言
//! .vessel.toml が任意 key build-lanes を値 true の 1 行だけで持つこと（器は名指した sha の宣言が値 true を持つ周だけ、便の初回の木を
//! 並び .worktrees/scribe2/lane/<n> で使い回す）。注の行を除いて key の字の行を数える。否定の見本は正しい見本から 1 句だけ替える。外の依存を使わない。
//! 同じ宣言が任意 key teeth-check を値 true の 1 行だけで持つこと（判断の記録 ADR-63 の決定 (5) の名乗り）も同じ数え方で見る。
#![cfg(test)]

use std::path::PathBuf;

/// 名乗りの行の字（key と値の間は空白 1 つずつ・値は TOML の真偽の true）。
const ON: &str = "build-lanes = true";

/// 歯の検査の名乗りの行の字（同じ書き方）。
const TEETH_ON: &str = "teeth-check = true";

/// 根の器の宣言の字。
fn declaration() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    std::fs::read_to_string(root.join(".vessel.toml")).expect("根の .vessel.toml を読む")
}

/// 注でない行のうち key が build-lanes の行（前後の空白を除いた字・file の順）。
fn lanes_lines(text: &str) -> Vec<&str> {
    key_lines(text, "build-lanes")
}

/// 注でない行のうち key が key の行（前後の空白を除いた字・file の順）。
fn key_lines<'a>(text: &'a str, key: &str) -> Vec<&'a str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter(|line| line.split('=').next().map(str::trim) == Some(key))
        .collect()
}

#[test]
fn vlanes_root_declaration_names_teeth_check_true_once() {
    assert_eq!(key_lines(&declaration(), "teeth-check"), vec![TEETH_ON]);
    assert_eq!(key_lines("schema = 1\nteeth-check = true\n", "teeth-check"), vec![TEETH_ON]);
    let cases: [(&str, Vec<&str>); 6] = [
        ("schema = 1\nteeth-check = false\n", vec!["teeth-check = false"]),
        ("schema = 1\nteeth-check = \"true\"\n", vec!["teeth-check = \"true\""]),
        ("schema = 1\n# teeth-check = true\n", vec![]),
        ("schema = 1\nteeth_check = true\n", vec![]),
        ("schema = 1\nteeth-checks = true\n", vec![]),
        ("schema = 1\nteeth-check = true\nteeth-check = true\n", vec![TEETH_ON, TEETH_ON]),
    ];
    for (text, want) in cases {
        let got = key_lines(text, "teeth-check");
        assert_eq!(got, want, "{text}");
        assert_ne!(got, vec![TEETH_ON], "{text}");
    }
}

#[test]
fn vlanes_root_declaration_names_build_lanes_true_once() {
    assert_eq!(lanes_lines(&declaration()), vec![ON]);
}

#[test]
fn vlanes_one_clause_changes_are_refused() {
    assert_eq!(lanes_lines("schema = 1\nbuild-lanes = true\n"), vec![ON]);
    let cases: [(&str, Vec<&str>); 5] = [
        ("schema = 1\nbuild-lanes = false\n", vec!["build-lanes = false"]),
        ("schema = 1\nbuild-lanes = \"true\"\n", vec!["build-lanes = \"true\""]),
        ("schema = 1\n# build-lanes = true\n", vec![]),
        ("schema = 1\n", vec![]),
        ("schema = 1\nbuild-lanes = true\nbuild-lanes = true\n", vec![ON, ON]),
    ];
    for (text, want) in cases {
        let got = lanes_lines(text);
        assert_eq!(got, want, "{text}");
        assert_ne!(got, vec![ON], "{text}");
    }
}
