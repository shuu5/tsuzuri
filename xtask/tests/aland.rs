//! 行 t-after-land-decl の歯（接頭辞 aland_・設計ノート surface-v4b・器の行 v-after-land）: 根の器の宣言 .vessel.toml が、器の任意 key
//! after-land の行をちょうど 1 つ（着地のたびに anchor で境界の binary を組み直す 1 項）持つこと。頭が after-land の行を拾い、
//! 否定の見本は正しい見本から 1 句だけ替える。外の依存を使わない。
#![cfg(test)]

use std::path::PathBuf;

/// 根の宣言が持つ after-land の行（項を二重引用符で囲んだ形）。
const LINE: &str = "after-land = [\"cargo build -q -p tsuzuri-boundary --bin tz\"]";

/// 根の器の宣言の字。
fn declaration() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    std::fs::read_to_string(root.join(".vessel.toml")).expect("根の .vessel.toml を読む")
}

/// 頭が after-land の行（前後の空白を除いた字・file の順）。
fn key_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("after-land"))
        .collect()
}

#[test]
fn aland_root_declaration_builds_the_boundary_binary() {
    let text = declaration();
    assert_eq!(key_lines(&text), [LINE]);
    assert_eq!(
        text.lines()
            .filter(|line| line.trim() == "schema = 1")
            .count(),
        1
    );
}

#[test]
fn aland_one_clause_changes_are_found() {
    let good = format!("schema = 1\n{LINE}\n");
    let want = key_lines(&good);
    assert_eq!(want, [LINE]);
    let cases: [String; 4] = [
        format!(
            "schema = 1\n{}\n",
            LINE.replace("--bin tz\"]", "--bin tz\", \"cargo build -q -p xtask\"]")
        ),
        format!("schema = 1\n{}\n", LINE.replace(" -q", "")),
        format!("schema = 1\n{LINE}\n{LINE}\n"),
        "schema = 1\n".to_string(),
    ];
    for text in &cases {
        assert_ne!(key_lines(text), want, "{text}");
    }
}
