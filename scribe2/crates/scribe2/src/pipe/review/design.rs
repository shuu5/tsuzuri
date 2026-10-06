//! 設計の節の本文の読み手（`pipe/review.rs` からの純移動）。審査の材料の `{design}` の本文を base の設計 doc から組む。

use super::table;
use std::path::Path;

/// 契約の `design` が設計 pointer（`<doc>#<id>`）なら、base の設計 doc からその行の `section` の節の本文を読む
/// （§4「順序」: 生成 (b) の前後で穴の出所は変わらない）。pointer でない周・解けない周は欠けの印の 1 行（`Err`）。
pub(super) fn design_read(repo: &Path, design: &str) -> Result<String, String> {
    let pointer = match table::parse_pointer(design) {
        Ok(found) => found,
        Err(error) => return Err(format!("（設計の節なし: design={design} は設計 pointer でない・{}）", error.reason())),
    };
    let text = match table::read(repo, &pointer.path) {
        Ok(found) => found,
        Err(reason) => return Err(format!("（設計の節を読めない: {reason}）")),
    };
    let row = match table::find_row(&pointer.path, &text, &pointer.id) {
        Ok(found) => found,
        Err(errors) => {
            let reasons: Vec<String> = errors.iter().map(|error| error.reason()).collect();
            return Err(format!("（契約表の行 {} を読めない: {}）", pointer.id, reasons.join(" / ")));
        }
    };
    // goal を持つ行（導出物の行）は goal が節の本文（出所の 1 行は不変・設計 contract-source.md §47 の 6）。
    let body = if row.goal.is_empty() { section_text(&text, &row.section) } else { row.goal };
    if body.trim().is_empty() {
        return Err(format!("（設計 doc {} の節 {} が無いか空）", pointer.path, row.section));
    }
    Ok(format!("{}#{} §{}\n{body}", pointer.path, pointer.id, row.section))
}

/// [`design_read`] の本文か欠けの印の 1 行をそのまま `{design}` の字にする。
pub(super) fn design_text(repo: &Path, design: &str) -> String {
    design_read(repo, design).unwrap_or_else(|missing| missing)
}

/// 節 `number` の本文（`## N.` の見出しの次の行から次の `## ` 見出しの前まで・契約表の区間と fence の中の
/// `## ` は見出しに数えない・`pipe::table` の節の読みと同じ形）。無ければ空。
pub(in crate::pipe) fn section_text(doc: &str, number: &str) -> String {
    let mut found: Vec<&str> = Vec::new();
    let (mut fenced, mut inside, mut open) = (false, false, false);
    for line in doc.lines() {
        let trimmed = line.trim();
        if trimmed == table::BEGIN {
            inside = true;
            continue;
        }
        if trimmed == table::END {
            inside = false;
            continue;
        }
        if inside {
            continue;
        }
        if trimmed.starts_with("```") {
            fenced = !fenced;
        }
        match line.strip_prefix("## ").filter(|_| !fenced) {
            Some(title) => open = section_number(title).as_deref() == Some(number),
            None if open => found.push(line),
            None => {}
        }
    }
    found.join("\n")
}

/// `## N. …` の N（数字の列だけ・それ以外は `None`）。
fn section_number(title: &str) -> Option<String> {
    let (head, _) = title.split_once('.')?;
    (!head.is_empty() && head.chars().all(|found| found.is_ascii_digit())).then(|| head.to_owned())
}
