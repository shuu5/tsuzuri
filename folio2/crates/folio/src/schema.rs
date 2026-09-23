//! `folio schema`（便 45・docs/design/delivery-45.md §1・ADR-9・FR19）。欄の決まりの file の schema 節（生成区間）を
//! 床の定数から導出して書く（--write）・検査する（--check）。憲法の生成区間（`inject.rs`）と同じ型 = 印 2 本・区間を取る
//! 関数・導出 1 関数を --write / --check が共有し、終了コードは 合格 0 / 不合格 1 / まだ分からない 2。
//!
//! 床の機械（床の木の型 `Floor`・マクロ `keys_floor`・`strip_notes`・`floor_diff`・導出 `derive` と導出の体裁）は
//! `floor.rs`（便 106・ADR-15 決定 (2)(3)・責務の層 1 読む）に置く。ここには命令の口だけが残る。
//! `folio check` の置き場ごとの検査は印を見ない（fixture の写しは印を持たない）。

use std::fs;
use std::path::Path;

use crate::floor::{Floor, derive};
use crate::verdict::Verdict;

/// 生成区間の印（行の全部がこの字面・前後に空白なし）。
pub const BEGIN: &str = "# folio:schema:begin — 生成区間・手で直さない・正本は実装の定数（folio schema --write が書く）";
pub const END: &str = "# folio:schema:end";

/// 命令が扱う file と、その schema 節の正本（床の定数）。この順に見て、最初に合格でない file で返す。
/// 3 本目は天井の正本（便 48・ADR-11 決定 (4)①・生成区間は file の末尾に 1 対）。
/// 4 本目は規則の表（便 53・ADR-11 決定 (4)②・生成区間は file の先頭の注釈の次に 1 対）。
/// 5 本目は入口の正本（便 76・ADR-11 決定 (4)④・生成区間は file の末尾に 1 対）。
/// 6〜8 本目は要件書・語彙・相談窓口（便 77・ADR-11 決定 (4)⑤・生成区間は file の末尾に 1 対）。
/// 9 本目は索引の欄の決まり（便 95・ADR-13 決定 (1)・生成区間は file の末尾に 1 対）。
const TARGETS: &[(&str, &Floor)] = &[
    ("adr/schema.yaml", &crate::floor_adr::FLOOR),
    ("design-note/schema.yaml", &crate::floor_note::FLOOR),
    ("ceiling.yaml", &crate::ceiling::FLOOR),
    ("rules.yaml", &crate::rules::FLOOR),
    ("index.yaml", &crate::entrance::FLOOR),
    ("srs.yaml", &crate::check::SRS_FLOOR),
    ("vocabulary.yaml", &crate::check::VOCABULARY_FLOOR),
    ("intake.yaml", &crate::intake::INTAKE_FLOOR),
    ("graph.yaml", &crate::graph::FLOOR),
];

// ── 区間と命令 ──

/// 生成区間（begin の行の改行の次の byte から end の行の先頭まで）。begin と end の行がそれぞれちょうど 1 本で
/// begin が先のときだけ定まる。印は行の全部が字面のとおりの行だけを数える。
pub fn region_of(text: &str) -> Option<(usize, usize)> {
    let mut begins = Vec::new();
    let mut ends = Vec::new();
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let body = line.strip_suffix('\n').unwrap_or(line);
        if body == BEGIN {
            begins.push(at + line.len());
        }
        if body == END {
            ends.push(at);
        }
        at += line.len();
    }
    match (begins.as_slice(), ends.as_slice()) {
        ([b], [e]) if b <= e => Some((*b, *e)),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Write,
    Check,
}

/// 1 回の実行の結果。`stdout` は file ごとの 1 行、`stderr` は理由（「folio schema: 」は口が付ける）。
#[derive(Debug)]
pub struct Outcome {
    pub verdict: Verdict,
    pub stdout: Vec<String>,
    pub stderr: Option<String>,
}

impl Outcome {
    fn refused(verdict: Verdict, msg: impl Into<String>) -> Self {
        Outcome {
            verdict,
            stdout: Vec::new(),
            stderr: Some(msg.into()),
        }
    }
}

/// 対象の file 1 本を --write / --check に掛ける。合格なら標準出力の 1 行を返す。
fn run_one(dir: &Path, file: &str, floor: &Floor, mode: Mode) -> Result<String, Outcome> {
    let path = dir.join(file);
    if path.is_symlink() || !path.is_file() {
        return Err(Outcome::refused(
            Verdict::Unknown,
            format!("{file}: 読めない"),
        ));
    }
    let text = fs::read_to_string(&path)
        .map_err(|e| Outcome::refused(Verdict::Unknown, format!("{file}: 読めない: {e}")))?;
    let Some((start, end)) = region_of(&text) else {
        return Err(Outcome::refused(
            Verdict::Unknown,
            format!("{file}: 印が 1 対でない"),
        ));
    };
    let want = derive(floor);
    let cur = &text[start..end];
    let size = want.len();
    if cur == want {
        return Ok(match mode {
            Mode::Check => format!("folio schema: 一致（{file}・{size} byte）"),
            Mode::Write => format!("folio schema: 変わらない（{file}・{size} byte）"),
        });
    }
    if mode == Mode::Check {
        return Err(Outcome::refused(
            Verdict::Fail,
            format!("{file}: 生成区間 {} byte ≠ 導出 {size} byte", cur.len()),
        ));
    }
    let new = format!("{}{want}{}", &text[..start], &text[end..]);
    fs::write(&path, new)
        .map_err(|e| Outcome::refused(Verdict::Unknown, format!("{file}: 書けない: {e}")))?;
    Ok(format!("folio schema: 書いた（{file}・{size} byte）"))
}

pub fn run(dir: &Path, mode: Mode) -> Outcome {
    let mut stdout = Vec::new();
    for (file, floor) in TARGETS {
        match run_one(dir, file, floor, mode) {
            Ok(line) => stdout.push(line),
            Err(outcome) => return outcome,
        }
    }
    Outcome {
        verdict: Verdict::Pass,
        stdout,
        stderr: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_region_needs_one_pair_of_marker_lines_in_order() {
        let text = format!("meta: x\n{BEGIN}\nschema:\n  a: 1\n{END}\nplain: y\n");
        let (s, e) = region_of(&text).unwrap();
        assert_eq!(&text[s..e], "schema:\n  a: 1\n");
        assert_eq!(region_of(&format!("{END}\n{BEGIN}\n")), None);
        assert_eq!(region_of(&format!("{BEGIN}\n{END}\n{END}\n")), None);
        assert_eq!(region_of(&format!(" {BEGIN}\n{END}\n")), None);
        assert_eq!(region_of(&format!("{BEGIN}\n")), None);
        assert_eq!(
            region_of(&format!("{BEGIN}\n{END}")),
            Some((BEGIN.len() + 1, BEGIN.len() + 1))
        );
    }
}
