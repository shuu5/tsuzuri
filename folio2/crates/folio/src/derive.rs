//! `folio derive`（便 119・docs/design/delivery-119.md §1 (b)・判断の記録 ADR-16 決定 (5)・要件書 FR11・責務の層 3 導出する）。
//! 契約表を持つ設計ノートから、器（scribe2）の形の導出物（1 文書 1 file の `<文書 id>.toml`）を組んで置き場へ書く（--write）・
//! 置き場の導出物との byte 一致を数える（--check）。配信の組み立て（`folio build`）から切り離した独立の命令で、
//! 面の生成器も様式の file も呼ばない。
//! 読みは床と同じ読み手（`note.rs` の load_notes・has_contract_table・load_external）を共有し、形は床の定数
//! （`floor_note.rs` の derived の節）のとおり: 先頭 schema = 1・行ごとに空行 1 つと見出し [[contract]]・欄は器の導出 file の
//! 宣言の順・値の無い欄と空の一覧は欄名ごと省く・行の末尾に section が指す散文の節の body を単一行にした goal。
//! 全部か無しか: 1 行でも導出できなければ（読めない・欄が合わない・escape の要る字）どの file も書かずに 2（P-4.1）。
//! 置き場の導出元の無い .toml は差分に数えず、消さず、名を 1 行ずつ出す（N-1.1・P-4.1）。
//! 便 183（判断の記録 ADR-31 決定 (2)(ウ)・要件書 FR27）から、規則の表に計画の名札の行が在れば、計画のノートの行の索引の
//! 生成区間も同じ回に書き（--write・全部か無しか）、床と同じ関数 `plan::drift` で比べる（--check）。名札の行が無い・規則の表が
//! 無い置き場は今のまま（契約表の導出物だけ）。
//! 行 t-seatcopy（判断の記録 ADR-38 決定 (3)(4)）から、規則の表に欄 key が seat-bytes と seat-role-bytes の行が在れば、憲法の
//! 正本から席の手元の 2 つの写し（`seat.rs`）も置き場の下の dir seat に同じ回に書き（--write）・byte で比べる（--check）。
//! 要の写しの file 全体の byte が 2 つの値の差を越えれば、どちらの命令も何も書かずに 1（違反）。

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::floor_note::{
    CONTRACT_TABLE, DERIVED_ARRAY, DERIVED_EXTENSION, DERIVED_SUBCOMMAND, EXTERNAL_HEAD, PROSE,
};
use crate::note::{self, Field, NoteDoc};
use crate::plan::{self, IndexRow};
use crate::rules;
use crate::seat;
use crate::verdict::{Report, Verdict};
use crate::yaml::{self, Node};

/// 設計ノートの置き場（正本の dir の直下）。
const NOTE_DIR: &str = "design-note";

/// escape しない形では書けない字（器の値の読み手は escape を解かず複数行も扱わない・床の定数の derived_note）。
const UNWRITABLE: &[char] = &['"', '\\', '\n', '\r'];

pub enum Mode {
    Write,
    Check,
}

/// 1 回の実行の結果。`stdout` は行の列（導出元の無い file の行と要約の行）、`stderr` は 1 行。
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

/// 計画のノートの行の索引（便 183）: 計画のノートの path と置き場からの名・今の字・導出した行。
struct PlanIndex {
    path: PathBuf,
    name: String,
    text: String,
    rows: Vec<IndexRow>,
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
    let (mut derived, index) = match derive_all(dir).and_then(|(d, notes)| Ok((d, plan_index(dir, &notes)?))) {
        Ok(d) => d,
        Err(e) => return Outcome::unknown(e),
    };
    match seat::derive(dir, &out_dir) {
        Ok(None) => {}
        Ok(Some(c)) if c.brief.len() > c.cap => return over_cap(&c),
        Ok(Some(c)) => derived.extend(seat_files(c)),
        Err(e) => return Outcome::unknown(e),
    }
    match mode {
        Mode::Write => write_all(&out_dir, &derived, index.as_ref()),
        Mode::Check => check_all(&out_dir, &derived, index.as_ref()),
    }
}

/// 計画の名札の行が在れば、計画のノートの行の索引を導く（無ければ None・規則の表の無い置き場も None）。
/// 名札の行が読めない・計画のノートが無い・行が id の形でない・読めないは Err（まだ分からない）。
fn plan_index(dir: &Path, notes: &[NoteDoc]) -> Result<Option<PlanIndex>, String> {
    let Some(rules) = plan::load_rules(dir)? else {
        return Ok(None);
    };
    let Some(id) = rules::plan_note(&rules)
        .map_err(|e| format!("rules.yaml: 計画のノートの名札の行が読めない: {e}"))?
    else {
        return Ok(None);
    };
    let note = notes.iter().find(|n| n.id == id).ok_or_else(|| {
        format!("rules.yaml: 計画の名札の行が名指す計画のノート {NOTE_DIR}/{id}.yaml が無い")
    })?;
    let rows = plan::index_rows(notes)?;
    let name = format!("{NOTE_DIR}/{}", note.file);
    let path = dir.join(&name);
    let text = fs::read_to_string(&path).map_err(|e| format!("{name}: 読めない: {e}"))?;
    Ok(Some(PlanIndex { path, name, text, rows }))
}

/// 席の手元の 2 つの写しを置き場の下の dir seat の導出物にする（行 t-seatcopy）。
fn seat_files(c: seat::Copies) -> [Derived; 2] {
    [
        Derived {
            name: format!("{}/{}", seat::DIR, seat::BRIEF),
            text: c.brief,
        },
        Derived {
            name: format!("{}/{}", seat::DIR, seat::FULL),
            text: c.full,
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

/// 契約表を持つ設計ノートを全部導出する（1 本でも導出できなければ Err）。読めた設計ノートも返す（行の索引の母集団・便 183）。
fn derive_all(dir: &Path) -> Result<(Vec<Derived>, Vec<NoteDoc>), String> {
    let nd = dir.join(NOTE_DIR);
    if nd.is_symlink() || !nd.is_dir() {
        return Err(format!(
            "{NOTE_DIR}/ が dir として無い（symlink を含む）: {}",
            nd.display()
        ));
    }
    let mut report = Report::default();
    let notes = note::load_notes(&nd, &mut report);
    first_reason(&report)?;
    let with_table: Vec<&NoteDoc> = notes
        .iter()
        .filter(|n| note::has_contract_table(&n.root))
        .collect();
    if with_table.is_empty() {
        return Ok((Vec::new(), notes));
    }
    let Some(fields) = note::load_external(dir, &mut report) else {
        first_reason(&report)?;
        return Err("器の導出 file が読めない".to_string());
    };
    let derived = with_table
        .into_iter()
        .map(|n| {
            derive_doc(n, &fields)
                .map(|text| Derived {
                    name: format!("{}{DERIVED_EXTENSION}", n.id),
                    text,
                })
                .map_err(|e| format!("{NOTE_DIR}/{}: {e}", n.file))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((derived, notes))
}

/// 読み手の違反か「まだ分からない」の最初の 1 つ（無ければ Ok）。
fn first_reason(report: &Report) -> Result<(), String> {
    if let Some(msg) = report.unknowns.first() {
        return Err(msg.clone());
    }
    if let Some((kind, msg)) = report.violations.first() {
        return Err(format!("[{kind}] {msg}"));
    }
    Ok(())
}

/// 設計ノート 1 本の導出物（契約表の節が 2 つ以上なら節の順に行を続ける）。
fn derive_doc(doc: &NoteDoc, fields: &[Field]) -> Result<String, String> {
    let sections = doc
        .root
        .get("sections")
        .and_then(Node::as_seq)
        .ok_or("sections が一覧でない")?;
    let mut text = format!("{EXTERNAL_HEAD}\n");
    let mut seen = HashSet::new();
    for section in sections {
        if section.get("type").and_then(Node::as_str) != Some(CONTRACT_TABLE) {
            continue;
        }
        let at = format!("§{}", section.get("n").and_then(Node::as_str).unwrap_or("?"));
        let rows = section
            .get("rows")
            .and_then(Node::as_seq)
            .ok_or_else(|| format!("{at} の rows が一覧でない"))?;
        for row in rows {
            if row.as_map().is_none() {
                return Err(format!("{at}: 行が欄の表でない"));
            }
            let id = row.get("id").and_then(Node::as_str).unwrap_or("?");
            let rat = format!("{at} の行 {id}");
            if !seen.insert(id.to_string()) {
                return Err(format!("{rat}: 同じ行 id が 2 度在る"));
            }
            text.push_str(&derive_row(row, sections, fields).map_err(|e| format!("{rat}: {e}"))?);
        }
    }
    Ok(text)
}

/// 行 1 つ（空行・見出し・器の宣言の順の欄・goal）。
fn derive_row(row: &Node, sections: &[Node], fields: &[Field]) -> Result<String, String> {
    for (key, _) in row.as_map().unwrap_or_default() {
        if !fields.iter().any(|f| &f.name == key) {
            return Err(format!("欄「{key}」が器の導出 file に無い"));
        }
    }
    let mut text = format!("\n[[{DERIVED_ARRAY}]]\n");
    for f in fields {
        let value = row.get(&f.name).unwrap_or(&Node::Null);
        if f.need == "required" && value.is_blank() {
            return Err(format!("必須の欄「{}」が無いか空", f.name));
        }
        if let Some(line) = field_line(f, value)? {
            text.push_str(&line);
        }
    }
    let section = row
        .get("section")
        .and_then(Node::as_str)
        .ok_or("section が無い")?;
    let goal = goal(sections, section)?;
    text.push_str(&format!("goal = {}\n", quote(&goal, "goal")?));
    Ok(text)
}

/// 欄 1 つの行（値が無い・空白だけ・空の一覧は None＝欄名ごと省く）。
fn field_line(f: &Field, value: &Node) -> Result<Option<String>, String> {
    let name = &f.name;
    let rendered = match (f.shape.as_str(), value) {
        (_, Node::Null) => return Ok(None),
        ("text", Node::Scalar(s)) if s.trim().is_empty() => return Ok(None),
        ("text", Node::Scalar(s)) => quote(s, name)?,
        ("text", _) => return Err(format!("欄「{name}」が text の形でない（文字列でない）")),
        (_, Node::Seq(items)) if items.is_empty() => return Ok(None),
        (_, Node::Seq(items)) => {
            let mut quoted = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                let s = item
                    .as_str()
                    .ok_or_else(|| format!("欄「{name}」の {i} 番目の要素が文字列でない"))?;
                quoted.push(quote(s, name)?);
            }
            format!("[{}]", quoted.join(", "))
        }
        _ => return Err(format!("欄「{name}」が list の形でない（文字列の一覧でない）")),
    };
    Ok(Some(format!("{name} = {rendered}\n")))
}

/// 二重引用符で囲む（escape しない・書けない字が在れば Err）。
fn quote(s: &str, name: &str) -> Result<String, String> {
    if s.contains(UNWRITABLE) {
        return Err(format!(
            "欄「{name}」の値に書けない字（二重引用符・逆斜線・改行）が在る（escape しない形では書けない）"
        ));
    }
    Ok(format!("\"{s}\""))
}

/// section の値と n が等しい散文の節の body を単一行に（各行を trim・空行を落とす・空白 1 つで繋ぐ）。
fn goal(sections: &[Node], section: &str) -> Result<String, String> {
    let body = sections
        .iter()
        .filter(|s| s.get("type").and_then(Node::as_str) == Some(PROSE))
        .find(|s| s.get("n").and_then(Node::as_str) == Some(section))
        .and_then(|s| s.get("body"))
        .and_then(Node::as_str)
        .filter(|b| !b.trim().is_empty())
        .ok_or_else(|| {
            format!("section「{section}」の指す節が同じ文書の本文を持つ prose の節でない")
        })?;
    Ok(body
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" "))
}

// ── 置き場 ──

/// 置き場の直下の導出元の無い .toml（名の昇順・ほかの拡張子は数えない）。
fn orphans(out_dir: &Path, derived: &[Derived]) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(out_dir).map_err(|e| format!("{}: 読めない: {e}", out_dir.display()))?;
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(DERIVED_EXTENSION) && !derived.iter().any(|d| &d.name == n))
        .collect();
    names.sort();
    Ok(names
        .into_iter()
        .map(|n| format!("folio {DERIVED_SUBCOMMAND}: 数えない: {n}（導出元なし）"))
        .collect())
}

/// 導出物の path が file として書けない形か（symlink・symlink の dir の下・dir）。
fn not_plain(path: &Path) -> bool {
    path.is_symlink() || path.parent().is_some_and(Path::is_symlink) || (path.exists() && !path.is_file())
}

/// 置き場が dir として在るか（symlink は認めない）。
fn out_is_dir(out_dir: &Path) -> bool {
    !out_dir.is_symlink() && out_dir.is_dir()
}

/// --write: 違う file だけを書く。置き場が無ければ作る（親 dir は作らない）。ほかの file は消さない（N-1.1）。
/// 計画の行の索引は、書き直した字が床と同じ関数で導出と一致するときだけ、導出物の後に書く（全部か無しか・便 183）。
fn write_all(out_dir: &Path, derived: &[Derived], index: Option<&PlanIndex>) -> Outcome {
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
    let plan_text = match index.map(rewritten).transpose() {
        Ok(t) => t,
        Err(e) => return Outcome::unknown(e),
    };
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
    let mut written = pending.len();
    for (path, d) in pending {
        if let Err(e) = fs::write(&path, &d.text) {
            return Outcome::unknown(format!("{}: 書けない: {e}", path.display()));
        }
    }
    if let (Some(p), Some(text)) = (index, plan_text)
        && text != p.text
    {
        if let Err(e) = fs::write(&p.path, text) {
            return Outcome::unknown(format!("{}: 書けない: {e}", p.path.display()));
        }
        written += 1;
    }
    write_summary(out_dir, derived, index, written)
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

/// --write の結果: 置き場に残る導出元の無い .toml の行と、書いた file の数の行を標準出力に並べる。
fn write_summary(
    out_dir: &Path,
    derived: &[Derived],
    index: Option<&PlanIndex>,
    written: usize,
) -> Outcome {
    let mut stdout = match orphans(out_dir, derived) {
        Ok(lines) => lines,
        Err(e) => return Outcome::unknown(e),
    };
    stdout.push(format!(
        "folio {DERIVED_SUBCOMMAND}: 書いた {written} file・変わらない {} file（{}）",
        derived.len() + usize::from(index.is_some()) - written,
        out_dir.display()
    ));
    Outcome {
        verdict: Verdict::Pass,
        stdout,
        stderr: None,
    }
}

/// 計画のノートの生成区間を書き直した字（印が 1 対でない・書き直しても床の関数で導出と合わないは Err）。
fn rewritten(p: &PlanIndex) -> Result<String, String> {
    let text = plan::rewrite(&p.text, &p.rows).map_err(|e| format!("{}: {e}", p.name))?;
    let root = yaml::parse(&text).map_err(|e| format!("{}: 書き直すと parse できない: {e}", p.name))?.root;
    match plan::drift(&root, &text, &p.rows) {
        None => Ok(text),
        Some(why) => Err(format!("{}: 書き直しても行の索引が合わない: {why}", p.name)),
    }
}

/// --check: 導出元を持つ導出物だけを置き場の file と byte 比較する（違う・置き場に無い = 1）。
/// 計画の行の索引は床と同じ関数で比べ、食い違えば差分 1 に数える（便 183）。
fn check_all(out_dir: &Path, derived: &[Derived], index: Option<&PlanIndex>) -> Outcome {
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
    if let Some(p) = index {
        let why = match yaml::parse(&p.text) {
            Ok(doc) => plan::drift(&doc.root, &p.text, &p.rows),
            Err(e) => return Outcome::unknown(format!("{}: parse できない: {e}", p.name)),
        };
        if let Some(why) = why {
            stdout.push(format!("folio {DERIVED_SUBCOMMAND}: DRIFT: {}（{why}）", p.name));
        }
    }
    let drift = stdout.len();
    match orphans(out_dir, derived) {
        Ok(lines) => stdout.extend(lines),
        Err(e) => return Outcome::unknown(e),
    }
    let verdict = if drift == 0 {
        Verdict::Pass
    } else {
        Verdict::Fail
    };
    stdout.push(format!(
        "folio {DERIVED_SUBCOMMAND}: {}（一致 {}・差分 {drift}・{}）",
        if drift == 0 { "一致" } else { "差分あり" },
        derived.len() + usize::from(index.is_some()) - drift,
        out_dir.display()
    ));
    Outcome {
        verdict,
        stdout,
        stderr: None,
    }
}
