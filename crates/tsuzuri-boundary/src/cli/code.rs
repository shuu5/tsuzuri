//! tz code（判断の記録 ADR-46 の決定 (1)〜(4)・要件 FR2）: code の層を組み直し、数か行の触る定義か file の定義を
//! 標準出力へ出す。書かない。席と係が行を書く前に、write-set の file の関数と型を引く口。
//! tz code [--repo <dir>] [--sg <program>] [--row <ノート>#<行> | --file <path>]
//! - 旗なし — 数の行（`files`・`defs`・種類ごとの `kind`・`rows`・`writes`・`unbound`・`skipped`）
//! - --row — 行の辺ごとの `file` の行とその file の `def` の行、file に当たらない項の `none` の行
//! - --file — file の `def` の行
//!
//! 組みは repo の git ls-files -z と、構文で探す道具（--sg・既定 ast-grep）の scan --rule <repo>/.config/code-defs.yml
//! --json=stream と、repo の contracts/ の直下の .toml の字から `tsuzuri_core::graph::code` が組む。
//! 終了 code は 組めた 0・使い方の誤りと名指しの誤り（規則の file・行・file が無い）1・字が読めない 2（まだ分からない）。

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tsuzuri_core::graph::code::{self, CodeGraph, Def, DefKind, Defs};

use crate::out::{emit, emit_err};
use crate::server::proc;

pub const USAGE: &str =
    "usage: tz code [--repo <dir>] [--sg <program>] [--row <ノート>#<行> | --file <path>]";

/// 規則の file（repo の根からの path）。
pub const RULES: &str = ".config/code-defs.yml";

/// 構文で探す道具の既定の program。
pub const SG: &str = "ast-grep";

/// 契約表の導出物の dir（repo の根からの path）。
pub const CONTRACTS: &str = "contracts";

/// 子 process の時間の上限。
const TIMEOUT: Duration = Duration::from_secs(120);

/// 不合格（使い方の誤りと名指しの誤り）。
const FAIL: u8 = 1;

/// まだ分からない（字が読めない）。
const UNKNOWN: u8 = 2;

/// 出す物。
enum Ask<'a> {
    Counts,
    Row(&'a str),
    File(&'a str),
}

/// 読んだ引数。
struct Args<'a> {
    repo: PathBuf,
    sg: &'a str,
    ask: Ask<'a>,
}

pub fn run(rest: &[&str]) -> u8 {
    let args = match parse(rest) {
        Ok(args) => args,
        Err(e) => {
            emit_err(&format!("tz code: {e}\n{USAGE}"));
            return FAIL;
        }
    };
    let rules = args.repo.join(RULES);
    if !rules.is_file() {
        emit_err(&format!("tz code: 規則の file {} が無い", rules.display()));
        return FAIL;
    }
    let g = match gather(&args.repo, args.sg) {
        Ok(g) => g,
        Err(why) => {
            emit_err(&format!("tz code: {why}"));
            return UNKNOWN;
        }
    };
    match args.ask {
        Ask::Counts => counts(&g),
        Ask::Row(row) => row_lines(&g, row),
        Ask::File(file) => file_lines(&g, file),
    }
}

/// file の一覧と定義の stream の 2 つの撃ちを読む（子は repo で撃つ・file の一覧の字と読んだ定義・読めない字はその訳）。
pub(crate) fn read_layer(repo: &Path, sg: &str) -> Result<(String, Defs), String> {
    let files = proc::run(OsStr::new("git"), ["ls-files", "-z"], repo, TIMEOUT)
        .map_err(|f| format!("git ls-files が読めない（{}）", f.word()))?;
    let args = ["scan", "--rule", RULES, "--json=stream"];
    let stream = proc::run(OsStr::new(sg), args, repo, TIMEOUT)
        .map_err(|f| format!("{sg} の scan が読めない（{}）", f.word()))?;
    let defs = code::read_defs(&String::from_utf8_lossy(&stream))
        .ok_or_else(|| format!("{sg} の scan の stream に読めない行が在る"))?;
    Ok((String::from_utf8_lossy(&files).into_owned(), defs))
}

/// 3 つの字を読んで code の層を組む（file の一覧と定義は `read_layer`・読めない字はその訳）。
fn gather(repo: &Path, sg: &str) -> Result<CodeGraph, String> {
    let (files, defs) = read_layer(repo, sg)?;
    let rows = write_sets(&repo.join(CONTRACTS))?;
    Ok(code::build(&files, defs, &rows))
}

/// contracts/ の直下の .toml の行の id（`<file の名の .toml の前>#<行>`）と write-set（file の名の順）。
fn write_sets(dir: &Path) -> Result<Vec<(String, Vec<String>)>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("{} が読めない（{e}）", dir.display()))?;
    let mut notes: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    notes.sort();
    let mut rows = Vec::new();
    for path in notes {
        let doc = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let read = fs::read_to_string(&path)
            .ok()
            .and_then(|t| code::read_write_sets(&t));
        let Some(read) = read else {
            return Err(format!("{} の行が読めない", path.display()));
        };
        rows.extend(
            read.into_iter()
                .map(|(id, items)| (format!("{doc}#{id}"), items)),
        );
    }
    Ok(rows)
}

/// 定義の行（`def`・名前・範囲を `,` で繋いだ字）。
fn def_line(d: &Def) -> String {
    let spans: Vec<String> = d.spans.iter().map(|(a, b)| format!("{a}-{b}")).collect();
    format!("def\t{}\t{}", d.id, spans.join(","))
}

/// 数の行を出す。
fn counts(g: &CodeGraph) -> u8 {
    let rows: std::collections::BTreeSet<&str> = g.writes.iter().map(|w| w.row.as_str()).collect();
    emit(&format!("files\t{}", g.files.len()));
    emit(&format!("defs\t{}", g.defs.len()));
    for kind in DefKind::ALL {
        let n = g.defs.iter().filter(|d| d.kind == kind).count();
        emit(&format!("kind\t{}\t{n}", kind.word()));
    }
    emit(&format!("rows\t{}", rows.len()));
    emit(&format!("writes\t{}", g.writes.len()));
    emit(&format!("unbound\t{}", g.unbound.len()));
    emit(&format!("skipped\t{}", g.skipped));
    0
}

/// 行の辺と定義と file に当たらない項の行を出す（辺も項も無い行は名指しの誤り）。
fn row_lines(g: &CodeGraph, row: &str) -> u8 {
    let writes = g.row_writes(row);
    let none: Vec<&String> = g
        .unbound
        .iter()
        .filter(|(r, _)| r == row)
        .map(|(_, i)| i)
        .collect();
    if writes.is_empty() && none.is_empty() {
        emit_err(&format!("tz code: 行 {row} の辺も項も無い"));
        return FAIL;
    }
    for w in writes {
        let defs = g.file_defs(&w.file);
        emit(&format!("file\t{}\t{}\t{}", w.file, w.item, defs.len()));
        for d in defs {
            emit(&def_line(d));
        }
    }
    for item in none {
        emit(&format!("none\t{item}"));
    }
    0
}

/// file の定義の行を出す（一覧に無い file は名指しの誤り）。
fn file_lines(g: &CodeGraph, file: &str) -> u8 {
    if !g.files.contains(file) {
        emit_err(&format!("tz code: file {file} が git の一覧に無い"));
        return FAIL;
    }
    for d in g.file_defs(file) {
        emit(&def_line(d));
    }
    0
}

/// `--名 値` か `--名=値` の --repo・--sg・--row・--file を読む（空の値と 2 度の引数と知らない引数と、
/// --row と --file の両方を断る）。
fn parse<'a>(rest: &[&'a str]) -> Result<Args<'a>, String> {
    let (mut repo, mut sg, mut row, mut file) = (None, None, None, None);
    let mut it = rest.iter();
    while let Some(arg) = it.next() {
        let (name, value) = match arg.split_once('=') {
            Some((n, v)) => (n, Some(v)),
            None => (*arg, None),
        };
        let slot = match name {
            "--repo" => &mut repo,
            "--sg" => &mut sg,
            "--row" => &mut row,
            "--file" => &mut file,
            _ => return Err(format!("知らない引数 {arg}")),
        };
        let value = match value {
            Some(v) => v,
            None => *it.next().ok_or_else(|| format!("{name} の値が無い"))?,
        };
        if value.is_empty() {
            return Err(format!("{name} の値が空"));
        }
        if slot.replace(value).is_some() {
            return Err(format!("{name} が 2 度ある"));
        }
    }
    let ask = match (row, file) {
        (Some(_), Some(_)) => return Err("--row と --file は 1 つだけ".into()),
        (Some(r), None) => Ask::Row(r),
        (None, Some(f)) => Ask::File(f),
        (None, None) => Ask::Counts,
    };
    Ok(Args {
        repo: PathBuf::from(repo.unwrap_or(".")),
        sg: sg.unwrap_or(SG),
        ask,
    })
}
