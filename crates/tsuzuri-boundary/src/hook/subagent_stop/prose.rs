//! 終える前の門の契約の本文の散文の門（判断の記録 ADR-72 の決定 (2)）。
//! 出力の dir の子 `contract` の直下の名の末が .md の file（設計係が bead の本文として bd update --body-file に渡す file）を名の順に、
//! 起票の門と同じ入口（folio の check --prose）で撃ち、落ちる file と まだ分からない file ごとに欠けの 1 行を返す。

use std::fs;
use std::path::{Path, PathBuf};

use tsuzuri_core::contract_gate::lines_of;

use crate::hook::question_gate::shoot;

/// 出力の dir の子の名（係が契約の file を置く dir）。
const DIR: &str = "contract";

/// 本文の file の名の末。
const SUFFIX: &str = ".md";

/// 出力の dir `out` の子 contract の直下の .md の file（名の順・読めなければ空・子の dir は見ない）。
fn files(out: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(out.join(DIR)) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .is_some_and(|n| n.to_string_lossy().ends_with(SUFFIX))
        })
        .collect();
    found.sort();
    found
}

/// 標準出力の字を 1 行にする（空でない行を「 / 」で繋ぐ）。
fn one_line(stdout: &str) -> String {
    let lines: Vec<&str> = stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    lines.join(" / ")
}

/// 本文の file の散文の門の欠け（.md の file が無ければ規則の表を読まない・通る file は欠けに数えない）。
pub fn holes(out: &Path, repo: &Path) -> Vec<String> {
    files(out)
        .into_iter()
        .filter_map(|file| {
            let (rc, stdout) = shoot(repo, file.as_os_str());
            let shown = file.display();
            match rc {
                0 => None,
                1 => Some(format!(
                    "契約の本文 {shown} が散文の門で落ちる（{}）",
                    lines_of(&stdout).join(" / ")
                )),
                _ => Some(format!(
                    "契約の本文 {shown} の散文の門が まだ分からない（{}）",
                    one_line(&stdout)
                )),
            }
        })
        .collect()
}
