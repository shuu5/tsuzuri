//! `folio derive`（便 119・docs/design/delivery-119.md §1 (b)・判断の記録 ADR-16 決定 (5)・要件書 FR11・責務の層 3 導出する）。
//! 規則の表に欄 key が seat-bytes と seat-role-bytes の行が在る置き場の、憲法の正本から席の手元の写し（`seat.rs`）を置き場の下の
//! dir seat へ書く（--write）・置き場の写しとの byte 一致を数える（--check）。配信の組み立て（`folio build`）から切り離した
//! 独立の命令で、面の生成器も様式の file も呼ばない。
//! 行 t-seatcopy（判断の記録 ADR-38 決定 (3)(4)）の 2 つの写しと、行 t-seatcap の役割の行の上限の file（seat-role-bytes の値）を
//! 同じ回に書き・比べる（条 P-2.3）。要の写しの file 全体の byte が 2 つの値の差を越えれば、どちらの命令も何も書かずに 1（違反）。
//! 全部か無しか: どれか 1 file でも導出できなければ（読めない・欄が合わない）どの file も書かずに 2（P-4.1）。
//! 行 t-derive-off（判断の記録 ADR-74 決定 (6)(8)）から、設計ノートの契約表の導出物と計画のノートの行の索引は書かない・読まない
//! （置き場の直下の .toml は読まず名も出さない）。

use std::fs;
use std::path::{Path, PathBuf};

use crate::floor_note::DERIVED_SUBCOMMAND;
use crate::note;
use crate::rules;
use crate::seat;
use crate::verdict::Verdict;

pub enum Mode {
    Write,
    Check,
}

/// 1 回の実行の結果。`stdout` は行の列（要約の行）、`stderr` は 1 行。
pub struct Outcome {
    pub verdict: Verdict,
    pub stdout: Vec<String>,
    pub stderr: Option<String>,
}

impl Outcome {
    fn unknown(reason: impl Into<String>) -> Self {
        Outcome {
            verdict: Verdict::Unknown,
            stdout: Vec::new(),
            stderr: Some(format!(
                "folio {DERIVED_SUBCOMMAND}: まだ分からない: {}",
                reason.into()
            )),
        }
    }
}

/// 導出した 1 file（file 名と中身）。
struct Derived {
    name: String,
    text: String,
}

// ── 命令の口 ──

pub fn run(dir: &Path, out: &Path, from_root: bool, mode: Mode) -> Outcome {
    // --out が相対なら --dir からの相対（--from-root なら置き場の根 `note::root_of` からの相対・便 192）・絶対ならそのまま（folio build と同じ）
    let base = if from_root {
        match note::root_of(dir) {
            Ok(root) => root,
            Err(e) => return Outcome::unknown(e),
        }
    } else {
        dir.to_path_buf()
    };
    let out_dir = base.join(out);
    let derived = match seat::derive(dir, &out_dir) {
        Ok(None) => Vec::new(),
        Ok(Some(c)) if c.brief.len() > c.cap => return over_cap(&c),
        Ok(Some(c)) => seat_files(c).into(),
        Err(e) => return Outcome::unknown(e),
    };
    match mode {
        Mode::Write => write_all(&out_dir, &derived),
        Mode::Check => check_all(&out_dir, &derived),
    }
}

/// 席の手元の 2 つの写しと役割の行の上限の file を置き場の下の dir seat の導出物にする（行 t-seatcopy・t-seatcap）。
fn seat_files(c: seat::Copies) -> [Derived; 3] {
    [
        Derived {
            name: format!("{}/{}", seat::DIR, seat::BRIEF),
            text: c.brief,
        },
        Derived {
            name: format!("{}/{}", seat::DIR, seat::FULL),
            text: c.full,
        },
        Derived {
            name: format!("{}/{}", seat::DIR, seat::ROLE_MAX),
            text: c.role_max,
        },
    ]
}

/// 要の写しが上限を越える（違反 1・どの file も書かない・判断の記録 ADR-38 決定 (4)）。
fn over_cap(c: &seat::Copies) -> Outcome {
    Outcome {
        verdict: Verdict::Fail,
        stdout: vec![format!(
            "folio {DERIVED_SUBCOMMAND}: 要の写し {}/{} が {} byte で上限 {} byte（欄 key {} の値から {} の値を引いた数）を越える",
            seat::DIR,
            seat::BRIEF,
            c.brief.len(),
            c.cap,
            rules::SEAT_BYTES,
            rules::SEAT_ROLE_BYTES
        )],
        stderr: None,
    }
}

// ── 置き場 ──

/// 導出物の path が file として書けない形か（symlink・symlink の dir の下・dir）。
fn not_plain(path: &Path) -> bool {
    path.is_symlink() || path.parent().is_some_and(Path::is_symlink) || (path.exists() && !path.is_file())
}

/// 置き場が dir として在るか（symlink は認めない）。
fn out_is_dir(out_dir: &Path) -> bool {
    !out_dir.is_symlink() && out_dir.is_dir()
}

/// --write: 違う file だけを書く。置き場が無ければ作る（親 dir は作らない）。ほかの file は消さない（N-1.1）。
fn write_all(out_dir: &Path, derived: &[Derived]) -> Outcome {
    if !out_is_dir(out_dir) {
        if out_dir.exists() || out_dir.is_symlink() {
            return Outcome::unknown(format!(
                "置き場が無い（dir として無い・symlink を含む）: {}",
                out_dir.display()
            ));
        }
        let parent_ok = out_dir
            .parent()
            .is_some_and(|p| p.as_os_str().is_empty() || p.is_dir());
        if !parent_ok {
            return Outcome::unknown(format!("置き場の親 dir が無い: {}", out_dir.display()));
        }
    }
    // 書く前に書く先を全部確かめる（全部か無しか）
    let mut pending = Vec::new();
    for d in derived {
        let path = out_dir.join(&d.name);
        if not_plain(&path) {
            return Outcome::unknown(format!("{}: file でない（symlink・dir）", path.display()));
        }
        if fs::read(&path).ok().as_deref() != Some(d.text.as_bytes()) {
            pending.push((path, d));
        }
    }
    if let Err(e) = make_dirs(out_dir, &pending) {
        return Outcome::unknown(e);
    }
    let written = pending.len();
    for (path, d) in pending {
        if let Err(e) = fs::write(&path, &d.text) {
            return Outcome::unknown(format!("{}: 書けない: {e}", path.display()));
        }
    }
    Outcome {
        verdict: Verdict::Pass,
        stdout: vec![format!(
            "folio {DERIVED_SUBCOMMAND}: 書いた {written} file・変わらない {} file（{}）",
            derived.len() - written,
            out_dir.display()
        )],
        stderr: None,
    }
}

/// 置き場と、書く file の親の dir（置き場の下の dir seat）が無ければ作る（親 dir の親は作らない）。dir でない物が在れば何も作らない。
fn make_dirs(out_dir: &Path, pending: &[(PathBuf, &Derived)]) -> Result<(), String> {
    let parents = pending.iter().filter_map(|(p, _)| p.parent());
    let dirs: Vec<&Path> = std::iter::once(out_dir).chain(parents).collect();
    if let Some(d) = dirs.iter().find(|d| d.exists() && !d.is_dir()) {
        return Err(format!("{}: dir でない", d.display()));
    }
    for dir in dirs {
        if !dir.exists()
            && let Err(e) = fs::create_dir(dir)
        {
            return Err(format!("{}: 置き場を作れない: {e}", dir.display()));
        }
    }
    Ok(())
}

/// --check: 導出物だけを置き場の file と byte 比較する（違う・置き場に無い = 1）。置き場の直下の .toml は読まない。
fn check_all(out_dir: &Path, derived: &[Derived]) -> Outcome {
    if !out_is_dir(out_dir) {
        return Outcome::unknown(format!(
            "置き場が無い（dir として無い・symlink を含む）: {}",
            out_dir.display()
        ));
    }
    let mut stdout = Vec::new();
    for d in derived {
        let path = out_dir.join(&d.name);
        if path.is_symlink() || path.parent().is_some_and(Path::is_symlink) {
            return Outcome::unknown(format!("{}: symlink は認めない", path.display()));
        }
        match fs::read(&path) {
            Ok(bytes) if bytes == d.text.as_bytes() => {}
            Ok(_) => stdout.push(format!(
                "folio {DERIVED_SUBCOMMAND}: DRIFT: {}（導出と byte で違う）",
                d.name
            )),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => stdout.push(format!(
                "folio {DERIVED_SUBCOMMAND}: DRIFT: {}（置き場に無い）",
                d.name
            )),
            Err(e) => return Outcome::unknown(format!("{}: 読めない: {e}", path.display())),
        }
    }
    let drift = stdout.len();
    let verdict = if drift == 0 {
        Verdict::Pass
    } else {
        Verdict::Fail
    };
    stdout.push(format!(
        "folio {DERIVED_SUBCOMMAND}: {}（一致 {}・差分 {drift}・{}）",
        if drift == 0 { "一致" } else { "差分あり" },
        derived.len() - drift,
        out_dir.display()
    ));
    Outcome {
        verdict,
        stdout,
        stderr: None,
    }
}
