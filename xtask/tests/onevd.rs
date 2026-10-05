//! 行 t-run-cap-on の歯（接頭辞 onevd_・設計ノート surface-wave29c・判断の記録 ADR-63 の決定 (13)）: 根の器の宣言 .vessel.toml が
//! 任意 key run-cap を値 1 の 1 行で、run-cap-paths を値 ["scribe2/"] の 1 行で、この順に持つこと（器は write-set に scribe2/ の下の項目を持つ
//! 便を、同じ dir の下を書く live な便と同じ周に起こした便を合わせて 1 本までしか起こさない）。注の行を除いて key の字の行を数える。
//! 否定の見本は正しい見本から 1 句だけ替える。外の依存を使わない。
#![cfg(test)]

use std::path::PathBuf;

/// 名乗りの 2 行の字（key と値の間は空白 1 つずつ・file の順）。
const ON: [&str; 2] = ["run-cap = 1", "run-cap-paths = [\"scribe2/\"]"];

/// 根の器の宣言の字。
fn declaration() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    std::fs::read_to_string(root.join(".vessel.toml")).expect("根の .vessel.toml を読む")
}

/// 注でない行のうち key が run-cap か run-cap-paths の行（前後の空白を除いた字・file の順）。
fn cap_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter(|line| matches!(line.split('=').next().map(str::trim), Some("run-cap" | "run-cap-paths")))
        .collect()
}

#[test]
fn onevd_root_declaration_names_the_run_cap_of_the_vessel_dir() {
    assert_eq!(cap_lines(&declaration()), ON);
}

#[test]
fn onevd_one_clause_changes_are_refused() {
    assert_eq!(cap_lines("schema = 1\nrun-cap = 1\nrun-cap-paths = [\"scribe2/\"]\n"), ON);
    let cases: [(&str, Vec<&str>); 5] = [
        ("schema = 1\nrun-cap = 2\nrun-cap-paths = [\"scribe2/\"]\n", vec!["run-cap = 2", ON[1]]),
        ("schema = 1\nrun-cap = 1\nrun-cap-paths = [\"scribe2/crates/\"]\n", vec![ON[0], "run-cap-paths = [\"scribe2/crates/\"]"]),
        ("schema = 1\n# run-cap = 1\nrun-cap-paths = [\"scribe2/\"]\n", vec![ON[1]]),
        ("schema = 1\nrun-cap-paths = [\"scribe2/\"]\nrun-cap = 1\n", vec![ON[1], ON[0]]),
        ("schema = 1\nrun-cap = 1\nrun-cap-paths = [\"scribe2/\"]\nrun-cap = 1\n", vec![ON[0], ON[1], ON[0]]),
    ];
    for (text, want) in cases {
        let got = cap_lines(text);
        assert_eq!(got, want, "{text}");
        assert_ne!(got, ON, "{text}");
    }
}
