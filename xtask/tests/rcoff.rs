//! 行 t-run-cap-off の歯（接頭辞 rcoff_・設計ノート surface-wave29d・判断の記録 ADR-64）: 根の器の宣言 .vessel.toml が、器の任意 key
//! run-cap と run-cap-paths の行も、その key を名指す注の行も持たないこと（器を書く便を同時に 1 本までにする床を外した・判断の記録
//! ADR-63 の決定 (13) の値を外す行）。注の行も数え、字 run-cap を含む行を拾う。否定の見本は正しい見本に 1 句だけ足す。外の依存を使わない。
#![cfg(test)]

use std::path::PathBuf;

/// 外した 2 つの key の共通の頭（run-cap-paths も含む）。
const KEY: &str = "run-cap";

/// 根の器の宣言の字。
fn declaration() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    std::fs::read_to_string(root.join(".vessel.toml")).expect("根の .vessel.toml を読む")
}

/// 字 run-cap を含む行（key の行と注の行・前後の空白を除いた字・file の順）。
fn cap_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| line.contains(KEY))
        .collect()
}

#[test]
fn rcoff_root_declaration_names_no_vessel_cap() {
    let text = declaration();
    assert_eq!(cap_lines(&text), Vec::<&str>::new());
    assert_eq!(
        text.lines()
            .filter(|line| line.trim() == "schema = 1")
            .count(),
        1
    );
}

#[test]
fn rcoff_one_clause_additions_are_found() {
    let good = "schema = 1\nbuild-lanes = true\n";
    assert_eq!(cap_lines(good), Vec::<&str>::new());
    let cases: [(&str, &str); 5] = [
        (
            "schema = 1\nbuild-lanes = true\nrun-cap = 1\n",
            "run-cap = 1",
        ),
        (
            "schema = 1\nbuild-lanes = true\nrun-cap-paths = [\"scribe2/\"]\n",
            "run-cap-paths = [\"scribe2/\"]",
        ),
        (
            "schema = 1\nbuild-lanes = true\n# 器の任意の key run-cap の注\n",
            "# 器の任意の key run-cap の注",
        ),
        ("schema = 1\nbuild-lanes = true\n  run-cap=2\n", "run-cap=2"),
        (
            "schema = 1\nrun-cap = 1\nbuild-lanes = true\n",
            "run-cap = 1",
        ),
    ];
    for (text, want) in cases {
        assert_eq!(cap_lines(text), [want], "{text}");
    }
}
