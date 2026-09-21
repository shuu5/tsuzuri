//! 便 87（docs/design/delivery-87.md §1 (e)）: face.rs の名札と名札の表を face_labels.rs へ切り出した歯。
//! 器の行数の式で face.rs が上限の内に収まり、移した定義が新しい module に在って face.rs に無く、
//! face.rs が新しい module を丸ごと再輸出する行を 1 本だけ持つことを見る。振る舞いの不変は既存の歯が見る。

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    let p = repo_root().join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} を読めない: {e}", p.display()))
}

/// 器の行数の式: 空行を含む全行を数え、字数（Unicode の字の数）が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す。
fn vessel_lines(text: &str) -> usize {
    text.lines()
        .map(|l| {
            let n = l.chars().count();
            if n > 120 { n.div_ceil(120) } else { 1 }
        })
        .sum()
}

/// 移した 8 本の定義の頭。
const MOVED_HEADS: &[&str] = &[
    "pub fn tier_label(",
    "pub fn rule_kind_label(",
    "pub fn method_label(",
    "pub fn stop_anchor(",
    "pub const SHELF_DOCS:",
    "pub const ANNEXES:",
    "pub struct Tier {",
    "pub struct Shelf {",
];

const REEXPORT: &str = "pub use crate::face_labels::*;";

#[test]
fn f87_face_is_split_and_under_the_cap() {
    let face = read("crates/folio/src/face.rs");
    let labels = read("crates/folio/src/face_labels.rs");

    let n = vessel_lines(&face);
    assert!(n <= 1250, "face.rs は器の式で {n} 行（上限 1250）");

    for head in MOVED_HEADS {
        assert!(
            labels.lines().any(|l| l.starts_with(head)),
            "face_labels.rs に「{head}」の定義が無い"
        );
        assert!(
            !face.lines().any(|l| l.trim_start().starts_with(head)),
            "face.rs に「{head}」の定義が残っている"
        );
    }

    let reexports = face.lines().filter(|l| l.trim() == REEXPORT).count();
    assert_eq!(reexports, 1, "face.rs の再輸出の行「{REEXPORT}」は 1 本だけ");
}
