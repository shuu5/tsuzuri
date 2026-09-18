//! `folio build`（便 17・docs/design/delivery-17.md §1 (a)）。3 面（入口・憲法・要件書）と判断の記録の面
//! （記録 1 本につき 1 枚・便 26・delivery-26.md §1 (b)）と設計ノートの面（設計ノート 1 本につき 1 枚・便 29・
//! delivery-29.md §1 (b)）と様式 2 本を 1 つの配信先 dir へまとめて出す
//! （--write）・配信先と正本の一致を検査する（--check）。
//! 面の生成は便 14〜16・便 25・便 28 の生成器（`face_index` / `face_constitution` / `face_srs` / `face_adr` /
//! `face_note` の derive）をそのまま呼ぶ。判断の記録の並びは入口の面と同じ読み（`face_index::records`）で
//! id の数の昇順・設計ノートの並びも入口の面と同じ読み（`face_index::notes`）で id の字の昇順。
//! 全部か無しか: 出す file を先に全部 memory の上で用意し、1 つでも導出できなければ 2 で終わり、配信先に 1 byte も
//! 書かない（配信先の dir も作らない・P-4.1）。配信先に在る他の file は消さない（N-1.1）。

use std::fs;
use std::path::Path;

use crate::face::R;
use crate::verdict::Verdict;
use crate::{face_adr, face_constitution, face_index, face_note, face_srs};

/// 配信先へ出す 1 本の出どころ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// 面の生成器（面の名）
    Face(&'static str),
    /// `<dir>/preview/<file の名>` の byte の写し
    Style,
}

/// 配信先へ出す固定の file（この表が閉じた一覧・順もこのとおり）。判断の記録の面はこの 5 本の後に、
/// 正本 `adr/ADR-n.yaml` の数だけ続き（名は `adr-<数>.html`）、その後に設計ノートの面が
/// 正本 `design-note/<文書 id>.yaml` の数だけ続く（名は `note-<文書 id>.html`）。
pub const OUTPUTS: [(&str, Source); 5] = [
    ("index.html", Source::Face("index")),
    ("constitution.html", Source::Face("constitution")),
    ("srs.html", Source::Face("srs")),
    ("folio.css", Source::Style),
    ("folio-ui.js", Source::Style),
];

pub enum Mode {
    Write,
    Check,
}

/// 1 回の実行の結果。`stdout` / `stderr` は 1 行ずつ。
pub struct Outcome {
    pub verdict: Verdict,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl Outcome {
    fn unknown(reason: impl Into<String>) -> Self {
        Outcome {
            verdict: Verdict::Unknown,
            stdout: None,
            stderr: Some(format!("folio build: まだ分からない: {}", reason.into())),
        }
    }
}

// ── 命令の口 ──

pub fn run(dir: &Path, out: &Path, mode: Mode) -> Outcome {
    // --out が相対なら --dir からの相対・絶対ならそのまま
    let out_dir = dir.join(out);
    let built = match build_all(dir) {
        Ok(b) => b,
        Err(e) => return Outcome::unknown(e),
    };
    let total: usize = built.iter().map(|(_, bytes)| bytes.len()).sum();
    match mode {
        Mode::Write => write_all(&out_dir, &built, total),
        Mode::Check => check_all(&out_dir, &built, total),
    }
}

/// 出す file を全部 memory の上で用意する（1 本でも用意できなければ Err）。
fn build_all(dir: &Path) -> R<Vec<(String, Vec<u8>)>> {
    let mut built = Vec::with_capacity(OUTPUTS.len());
    for (name, source) in OUTPUTS {
        let bytes = match source {
            Source::Face(face) => derive(face, dir)?.into_bytes(),
            Source::Style => {
                let path = dir.join("preview").join(name);
                fs::read(&path).map_err(|e| format!("{}: 読めない: {e}", path.display()))?
            }
        };
        built.push((name.to_string(), bytes));
    }
    for record in face_index::records(dir)? {
        let html = face_adr::derive(dir, record.id())?;
        built.push((record.file(), html.into_bytes()));
    }
    for note in face_index::notes(dir)? {
        let html = face_note::derive(dir, note.id())?;
        built.push((note.file(), html.into_bytes()));
    }
    Ok(built)
}

/// 面の名 → 便 14〜16 の生成器（`face.rs` の run と同じ選び方）。
fn derive(face: &str, dir: &Path) -> R<String> {
    match face {
        "index" => face_index::derive(dir),
        "constitution" => face_constitution::derive(dir),
        "srs" => face_srs::derive(dir),
        f => Err(format!(
            "面の名「{f}」は index・constitution・srs のどれでもない"
        )),
    }
}

fn write_all(out_dir: &Path, built: &[(String, Vec<u8>)], total: usize) -> Outcome {
    if !out_dir.is_dir() {
        if !out_dir.parent().is_some_and(Path::is_dir) {
            return Outcome::unknown(format!("{}: 配信先の親 dir が無い", out_dir.display()));
        }
        if let Err(e) = fs::create_dir(out_dir) {
            return Outcome::unknown(format!("{}: 配信先を作れない: {e}", out_dir.display()));
        }
    }
    for (name, bytes) in built {
        let path = out_dir.join(name);
        if let Err(e) = fs::write(&path, bytes) {
            return Outcome::unknown(format!("{}: 書けない: {e}", path.display()));
        }
    }
    Outcome {
        verdict: Verdict::Pass,
        stdout: Some(format!(
            "folio build: 書いた（{} file・{total} byte）",
            built.len()
        )),
        stderr: None,
    }
}

fn check_all(out_dir: &Path, built: &[(String, Vec<u8>)], total: usize) -> Outcome {
    let mut missing: Vec<&str> = Vec::new();
    let mut drift: Vec<&str> = Vec::new();
    for (name, bytes) in built {
        // 比較は byte 列の一致（改行の読み替えをしない）
        match fs::read(out_dir.join(name)) {
            Ok(cur) if cur == *bytes => {}
            Ok(_) => drift.push(name),
            Err(_) => missing.push(name),
        }
    }
    if !missing.is_empty() {
        return Outcome {
            verdict: Verdict::Unknown,
            stdout: None,
            stderr: Some(format!("folio build: 配信先に無い: {}", missing.join("・"))),
        };
    }
    if !drift.is_empty() {
        return Outcome {
            verdict: Verdict::Fail,
            stdout: None,
            stderr: Some(format!(
                "folio build: DRIFT — {}（手で直したか正本が変わった）",
                drift.join("・")
            )),
        };
    }
    Outcome {
        verdict: Verdict::Pass,
        stdout: Some(format!(
            "folio build: OK — 配信先は正本と一致（{} file・{total} byte）",
            built.len()
        )),
        stderr: None,
    }
}

#[cfg(test)]
mod site_tests {
    use super::*;

    #[test]
    fn site_outputs_are_five_files_in_this_order() {
        assert_eq!(OUTPUTS.len(), 5);
        assert_eq!(
            OUTPUTS.map(|(name, _)| name),
            [
                "index.html",
                "constitution.html",
                "srs.html",
                "folio.css",
                "folio-ui.js"
            ]
        );
        assert_eq!(
            OUTPUTS.map(|(_, source)| source),
            [
                Source::Face("index"),
                Source::Face("constitution"),
                Source::Face("srs"),
                Source::Style,
                Source::Style
            ]
        );
    }

    #[test]
    fn site_derive_takes_only_the_three_face_names() {
        assert!(derive("figure", Path::new("design-intent")).is_err());
    }
}
