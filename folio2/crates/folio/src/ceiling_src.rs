//! 天井の正本の読み手と材料の束の組み直し・所見と束の閉じた一覧・印の file 名の置き場（便 110・docs/design/delivery-110.md
//! §1 (b)・ADR-15・層 1 読む）。読み手・組み直し・要約値・面の名の形・結果の型は `bundle.rs` から、閉じた一覧 8 本は
//! `ceiling.rs` から、印の file 名は `stamp.rs` から字を変えずに降ろした（移した注の中の file 名は移す前の置き場を指す）。
//! 読むのは束を組む命令（`bundle.rs`）・天井の床（`ceiling.rs`）・所見の検査（`findings.rs`）・門（`gate.rs`）・印・索引・面・入口。

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::cursor::R;
use crate::sha256;
use crate::verdict::Verdict;
use crate::yaml::{self, Node};

/// 観点ごとの 3 値（要件書 FR5 の 3 値・順も固定）。
pub const VERDICT_VALUES: [&str; 3] = ["合格", "不合格", "まだ分からない"];

/// 所見 1 件が必ず持つ欄。
pub const FINDING_REQUIRED: [&str; 5] = ["id", "viewpoint", "place", "weight", "evidence"];

/// 所見 1 件が持ってよい欄。
pub const FINDING_OPTIONAL: [&str; 2] = ["refute", "note"];

/// 所見の場所が必ず持つ欄。
pub const PLACE_REQUIRED: [&str; 2] = ["doc", "at"];

/// 反証の結果の値域。
pub const REFUTE_VALUES: [&str; 3] = ["支持", "退けた", "まだ分からない"];

/// 起動の記録が必ず持つ欄。
pub const RECORD_REQUIRED: [&str; 5] = ["model", "effort", "at", "read", "bundle"];

/// 束の sources の写しで常に残す最上位の節（順も同じ・`bundle.rs` が組む・便 102）。
pub const BUNDLE_SKELETON: [&str; 6] = ["meta", "id", "title", "status", "date", "schema"];

/// 束の要約値の規則の名（digest.txt の頭）。
pub const BUNDLE_DIGEST: &str = "sha256-files-1";

/// 印の置き場（`--dir` からの相対・床の定数）。
pub const STAMP_FILE: &str = "preview/ceiling-stamp.yaml";

/// 天井の正本の file。
const FILE: &str = "ceiling.yaml";

/// 束の要約値の file（要約値の計算には数えない）。
pub const DIGEST_FILE: &str = "digest.txt";

/// 文書の id → 面の file の名の形（`site.rs` の配信先の名の写し・床の定数）。`None` = 面は無い。
pub const FACE_NAMES: [(&str, Option<FaceName>); 10] = [
    ("index", Some(FaceName::Exact("index.html"))),
    ("constitution", Some(FaceName::Exact("constitution.html"))),
    ("srs", Some(FaceName::Exact("srs.html"))),
    ("adr", Some(FaceName::Affix("adr-", ".html"))),
    ("design-note", Some(FaceName::Affix("note-", ".html"))),
    ("rules", None),
    ("vocabulary", None),
    ("intake", None),
    ("ceiling", None),
    ("graph", None),
];

/// 面の file の名の形。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceName {
    /// 名の全体一致
    Exact(&'static str),
    /// 頭と尾（`adr-<数>.html`・`note-<文書 id>.html`）
    Affix(&'static str, &'static str),
}

impl FaceName {
    fn matches(self, name: &str) -> bool {
        match self {
            FaceName::Exact(want) => name == want,
            FaceName::Affix(head, tail) => {
                name.len() > head.len() + tail.len()
                    && name.starts_with(head)
                    && name.ends_with(tail)
            }
        }
    }
}

/// 1 回の実行の結果。`stdout` / `stderr` は 1 行ずつ。
pub struct Outcome {
    pub verdict: Verdict,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl Outcome {
    pub fn unknown(reason: impl Into<String>) -> Self {
        Outcome {
            verdict: Verdict::Unknown,
            stdout: None,
            stderr: Some(format!("folio ceiling: まだ分からない: {}", reason.into())),
        }
    }
}

/// 観点 1 つの束（観点の dir からの相対 path → 中身・path の byte 順）。
pub type Files = BTreeMap<String, Vec<u8>>;

/// 天井の正本から読む欄（束を組むのに要る分と、所見 file を数える値域・欄の名・形の検査は床 `ceiling.rs` の領分）。
pub struct Ceiling {
    /// documents の各行（id・file）
    documents: Vec<(String, String)>,
    pub viewpoints: Vec<Viewpoint>,
    /// 所見の欄の決まり（weights は正本の値・残りは床の定数）
    pub rules: Rules,
    /// finding.yaml の本文（4 観点とも同じ）
    finding: String,
}

pub struct Viewpoint {
    pub id: String,
    name: String,
    reader: String,
    question: String,
    /// reads の各行（doc・fields）
    pub reads: Vec<(String, Vec<String>)>,
}

/// 所見の欄の決まり（finding.yaml の材料と `--check` の値域）。weights の 2 本は天井の正本の写し（file が正本のまま・
/// ADR-11 決定 (3)(ア)）、残り 6 本は床の定数 `ceiling.rs` の写し（便 47 §1 (d)）。
pub struct Rules {
    pub finding_required: Vec<String>,
    pub finding_optional: Vec<String>,
    pub place_required: Vec<String>,
    pub refute_values: Vec<String>,
    pub weight_values: Vec<String>,
    pub weight_refute: Vec<String>,
    pub verdict_values: Vec<String>,
    pub record_required: Vec<String>,
}

// ── 天井の正本を読む ──

/// `<dir>/ceiling.yaml` を `check.rs` の正本の読み手と同じ文言で読む。
pub fn load(dir: &Path) -> R<Ceiling> {
    let path = dir.join(FILE);
    if path.is_symlink() {
        return Err(format!("{FILE}: symlink は認めない"));
    }
    if !path.exists() {
        return Err(format!("{FILE}: 正本が無い"));
    }
    if !path.is_file() {
        return Err(format!("{FILE}: file でない"));
    }
    let text = fs::read_to_string(&path).map_err(|e| format!("{FILE}: 読めない: {e}"))?;
    let doc = yaml::parse(&text).map_err(|e| format!("{FILE}: parse できない: {e}"))?;
    let root = doc.root;
    if root.as_map().is_none() {
        return Err(format!("{FILE}: 最上位が欄の表でない"));
    }
    let documents = rows(&root, "documents")?
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let at = format!("documents[{i}]");
            Ok((text_of(row, "id", &at)?, text_of(row, "file", &at)?))
        })
        .collect::<R<Vec<_>>>()?;
    let mut viewpoints = Vec::new();
    for (i, row) in rows(&root, "viewpoints")?.iter().enumerate() {
        let at = format!("viewpoints[{i}]");
        let id = text_of(row, "id", &at)?;
        let at = format!("viewpoints[{id}]");
        let reads = rows(row, "reads")
            .map_err(|_| format!("{FILE}: {at}.reads: 読めない"))?
            .iter()
            .enumerate()
            .map(|(j, read)| {
                let here = format!("{at}.reads[{j}]");
                Ok((
                    text_of(read, "doc", &here)?,
                    list_of(read, "fields", &here)?,
                ))
            })
            .collect::<R<Vec<_>>>()?;
        for (doc, _) in &reads {
            if !documents.iter().any(|(id, _)| id == doc) {
                return Err(format!("{FILE}: {at}.reads: 行き先「{doc}」が一覧に無い"));
            }
        }
        viewpoints.push(Viewpoint {
            name: text_of(row, "name", &at)?,
            reader: text_of(row, "reader", &at)?,
            question: text_of(row, "question", &at)?,
            reads,
            id,
        });
    }
    let rules = load_rules(&root)?;
    let finding = finding_text(&rules);
    Ok(Ceiling {
        documents,
        viewpoints,
        rules,
        finding,
    })
}

/// 行の一覧の節（各行は表）。
fn rows<'a>(node: &'a Node, key: &str) -> R<Vec<&'a Node>> {
    match node.get(key) {
        Some(Node::Seq(items)) if items.iter().all(|n| n.as_map().is_some()) => {
            Ok(items.iter().collect())
        }
        _ => Err(format!("{FILE}: {key}: 読めない")),
    }
}

/// 表の欄。
fn table_of<'a>(node: &'a Node, key: &str, at: &str) -> R<&'a Node> {
    match node.get(key) {
        Some(t) if t.as_map().is_some() => Ok(t),
        _ => Err(format!("{FILE}: {}{key}: 読めない", prefix(at))),
    }
}

/// 文の欄。
fn text_of(node: &Node, key: &str, at: &str) -> R<String> {
    node.get(key)
        .and_then(Node::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("{FILE}: {}{key}: 読めない", prefix(at)))
}

/// 文の一覧の欄。
fn list_of(node: &Node, key: &str, at: &str) -> R<Vec<String>> {
    node.get(key)
        .and_then(Node::as_seq)
        .and_then(|items| {
            items
                .iter()
                .map(|n| n.as_str().map(str::to_string))
                .collect::<Option<Vec<_>>>()
        })
        .ok_or_else(|| format!("{FILE}: {}{key}: 読めない", prefix(at)))
}

fn prefix(at: &str) -> String {
    if at.is_empty() {
        String::new()
    } else {
        format!("{at}.")
    }
}

/// 所見の欄の決まり。天井の正本から読むのは weights の 2 本（values・refute）だけで、残りは床の定数から埋める。
fn load_rules(root: &Node) -> R<Rules> {
    let weights = table_of(root, "weights", "")?;
    let floor = |items: &[&str]| items.iter().map(|s| s.to_string()).collect();
    Ok(Rules {
        finding_required: floor(&FINDING_REQUIRED),
        finding_optional: floor(&FINDING_OPTIONAL),
        place_required: floor(&PLACE_REQUIRED),
        refute_values: floor(&REFUTE_VALUES),
        weight_values: list_of(weights, "values", "weights")?,
        weight_refute: list_of(weights, "refute", "weights")?,
        verdict_values: floor(&VERDICT_VALUES),
        record_required: floor(&RECORD_REQUIRED),
    })
}

/// finding.yaml の本文（所見の欄の決まりを固定の型に流し込む）。
fn finding_text(rules: &Rules) -> String {
    let join = |items: &[String]| items.join(", ");
    format!(
        "# 所見の欄の決まり（天井の正本 ceiling.yaml の finding・weights・verdicts・record の写し・folio ceiling が組んだ）\n\
         finding:\n  required: [{}]\n  optional: [{}]\n  place: {{required: [{}]}}\n  refute: {{values: [{}]}}\n\
         weights:\n  values: [{}]\n  refute: [{}]\n\
         verdicts:\n  values: [{}]\n\
         record:\n  required: [{}]\n",
        join(&rules.finding_required),
        join(&rules.finding_optional),
        join(&rules.place_required),
        join(&rules.refute_values),
        join(&rules.weight_values),
        join(&rules.weight_refute),
        join(&rules.verdict_values),
        join(&rules.record_required),
    )
}

// ── 束を memory の上で組む ──

/// 観点 1 つの束と、落とした節の名の総数・宣言に在るが正本に無い節の名の総数。
pub(crate) struct Built {
    pub(crate) files: Files,
    pub(crate) dropped: usize,
    pub(crate) absent: usize,
}

/// 4 観点の束を全部 memory の上で用意する（1 本でも用意できなければ Err）。戻り値 = (観点の id, 束) の列（viewpoints の順）。
pub(crate) fn build_all(dir: &Path, faces_dir: &Path, ceiling: &Ceiling) -> R<Vec<(String, Built)>> {
    if !faces_dir.is_dir() {
        return Err(format!("{}: 配信先が無い", faces_dir.display()));
    }
    let face_files = read_dir_names(faces_dir)?;
    let mut bundles = Vec::with_capacity(ceiling.viewpoints.len());
    for vp in &ceiling.viewpoints {
        let built = build_counted(dir, faces_dir, &face_files, ceiling, vp)?;
        bundles.push((vp.id.clone(), built));
    }
    Ok(bundles)
}

/// 観点 1 つの束を memory の上で用意する（digest.txt を含む）。`face_files` = `--faces` の直下の名（`read_dir_names`）。
pub fn build_one(
    dir: &Path,
    faces_dir: &Path,
    face_files: &[(String, bool)],
    ceiling: &Ceiling,
    vp: &Viewpoint,
) -> R<Files> {
    build_counted(dir, faces_dir, face_files, ceiling, vp).map(|b| b.files)
}

fn build_counted(
    dir: &Path,
    faces_dir: &Path,
    face_files: &[(String, bool)],
    ceiling: &Ceiling,
    vp: &Viewpoint,
) -> R<Built> {
    let mut files = Files::new();
    // doc ごとの読む最上位の節（宣言の順・重複なし）
    let mut declared: Vec<(&str, Vec<&str>)> = Vec::new();
    for (doc, fields) in &vp.reads {
        let at = match declared.iter().position(|(d, _)| d == doc) {
            Some(at) => at,
            None => {
                declared.push((doc, Vec::new()));
                declared.len() - 1
            }
        };
        for field in fields {
            let top = field.split('.').next().unwrap_or(field);
            if !declared[at].1.contains(&top) {
                declared[at].1.push(top);
            }
        }
    }
    let mut dropped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut notes_absent = String::new();
    let mut absent = 0;
    for (doc, tops) in &declared {
        let file = &ceiling
            .documents
            .iter()
            .find(|(id, _)| id == doc)
            .expect("reads の doc は load で documents に解いてある")
            .1;
        let mut copied = Files::new();
        copy_sources(dir, file, &mut copied)?;
        let mut present: Vec<String> = Vec::new();
        for (rel, bytes) in copied {
            let text = String::from_utf8(bytes)
                .map_err(|_| format!("{}: UTF-8 として読めない", &rel["sources/".len()..]))?;
            let cut = cut_sections(&text, tops);
            present.extend(cut.sections);
            if !cut.dropped.is_empty() {
                dropped.insert(rel["sources/".len()..].to_string(), cut.dropped);
            }
            files.insert(rel, cut.kept.into_bytes());
        }
        let missing: Vec<&str> = tops
            .iter()
            .copied()
            .filter(|t| !present.iter().any(|p| p == t))
            .collect();
        if !missing.is_empty() {
            absent += missing.len();
            notes_absent.push_str(&format!(
                "# 宣言に在るが正本に無い節 {file}: {}\n",
                missing.join(", ")
            ));
        }
        copy_faces(faces_dir, face_files, doc, &mut files)?;
    }
    let mut reads = reads_text(vp);
    for (rel, names) in &dropped {
        reads.push_str(&format!("# 落とした節 {rel}: {}\n", names.join(", ")));
    }
    reads.push_str(&notes_absent);
    reads.push_str(&format!("# 常に残す節: {}\n", BUNDLE_SKELETON.join(", ")));
    files.insert("question.yaml".to_string(), question_text(vp).into_bytes());
    files.insert(
        "finding.yaml".to_string(),
        ceiling.finding.clone().into_bytes(),
    );
    files.insert("reads.yaml".to_string(), reads.into_bytes());
    let digest = digest_text(&files);
    files.insert(DIGEST_FILE.to_string(), digest.into_bytes());
    Ok(Built {
        files,
        dropped: dropped.values().map(Vec::len).sum(),
        absent,
    })
}

// ── 最上位の節で切る（便 98・docs/design/delivery-98.md §1 (b)） ──

// 骨格の閉じた一覧（どの観点の写しにも常に残す最上位の節）は床の定数 `ceiling::BUNDLE_SKELETON`（便 102 で移した・値は不変）。

/// 生成区間の開きと閉じの印（行の頭）。
const REGION_BEGIN: &str = "# folio:schema:begin";
const REGION_END: &str = "# folio:schema:end";

/// 1 file を切った結果。
struct Cut {
    /// 残した行（正本の byte のまま・正本の順のまま）
    kept: String,
    /// 最上位の節の名（正本に出る順）
    sections: Vec<String>,
    /// 落とした節の名（正本に出る順）
    dropped: Vec<String>,
}

/// 列 0 の `名:`（`:` の後が行末か空白）なら名。列 0 の `- ` は節の続き・注釈と空行は節でない（規則 2）。
fn section_name(line: &str) -> Option<&str> {
    let body = line.trim_end_matches(['\n', '\r']);
    if body.is_empty() || body.starts_with([' ', '\t', '#', '-']) {
        return None;
    }
    body.match_indices(':')
        .map(|(i, _)| i)
        .find(|&i| i > 0 && matches!(body[i + 1..].chars().next(), None | Some(' ' | '\t')))
        .map(|i| &body[..i])
}

fn is_trivia(line: &str) -> bool {
    let t = line.trim();
    t.is_empty() || t.starts_with('#')
}

/// 最上位の節の単位で切る（規則 1〜4）。残すのは `tops` の節・骨格・file の頭（最初の節より前の行）。
fn cut_sections(text: &str, tops: &[&str]) -> Cut {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let names: Vec<Option<&str>> = lines.iter().map(|l| section_name(l)).collect();
    // 行の持ち主の節の番号（None = file の頭）
    let mut owner: Vec<Option<usize>> = vec![None; lines.len()];
    let mut sections: Vec<String> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if let Some(name) = names[i] {
            sections.push(name.to_string());
            owner[i] = Some(sections.len() - 1);
            i += 1;
            continue;
        }
        let cur = sections.len().checked_sub(1);
        if cur.is_some() && is_trivia(lines[i]) {
            // 注釈と空行の連なりは直後の節へ寄せ、直後が節でなければ直前の節へ（規則 3）
            let mut j = i;
            while j < lines.len() && is_trivia(lines[j]) {
                j += 1;
            }
            let to = if j < lines.len() && names[j].is_some() {
                Some(sections.len())
            } else {
                cur
            };
            owner[i..j].fill(to);
            i = j;
            continue;
        }
        owner[i] = cur;
        i += 1;
    }
    let keep: Vec<bool> = sections
        .iter()
        .map(|s| tops.contains(&s.as_str()) || BUNDLE_SKELETON.contains(&s.as_str()))
        .collect();
    // 生成区間は中の節と一緒に 1 つの塊（規則 4）
    let begin = lines.iter().position(|l| l.starts_with(REGION_BEGIN));
    let end = lines.iter().position(|l| l.starts_with(REGION_END));
    let region = match (begin, end) {
        (Some(b), Some(e)) if b < e => {
            let live = (b..=e).any(|k| names[k].is_some() && owner[k].is_some_and(|s| keep[s]));
            Some((b, e, live))
        }
        _ => None,
    };
    let mut kept = String::new();
    for (k, line) in lines.iter().enumerate() {
        let live = match region {
            Some((b, e, live)) if (b..=e).contains(&k) => live,
            _ => owner[k].is_none_or(|s| keep.get(s).copied().unwrap_or(false)),
        };
        if live {
            kept.push_str(line);
        }
    }
    let dropped = sections
        .iter()
        .zip(&keep)
        .filter(|(_, k)| !**k)
        .map(|(s, _)| s.clone())
        .collect();
    Cut {
        kept,
        sections,
        dropped,
    }
}

/// 正本の写し。file 形（末尾が / でない）は `<dir>/<file>` を `sources/<file>` へ、dir 形（末尾が /）は
/// `<dir>/<file>` の直下の .yaml を名の byte 順に全部 `sources/<file><名>` へ、byte のまま写す（下の dir は見ない）。
fn copy_sources(dir: &Path, file: &str, files: &mut Files) -> R<()> {
    let path = dir.join(file);
    if path.is_symlink() {
        return Err(format!("{file}: symlink は認めない"));
    }
    if !path.exists() {
        return Err(format!("{file}: 正本が無い"));
    }
    if file.ends_with('/') {
        if !path.is_dir() {
            return Err(format!("{file}: dir でない"));
        }
        let names: Vec<String> = read_dir_names(&path)?
            .into_iter()
            .filter(|(name, is_file)| *is_file && name.ends_with(".yaml"))
            .map(|(name, _)| name)
            .collect();
        if names.is_empty() {
            return Err(format!("{file}: 正本が無い"));
        }
        for name in names {
            let entry = path.join(&name);
            if entry.is_symlink() {
                return Err(format!("{file}{name}: symlink は認めない"));
            }
            let bytes = fs::read(&entry).map_err(|e| format!("{file}{name}: 読めない: {e}"))?;
            files.insert(format!("sources/{file}{name}"), bytes);
        }
        return Ok(());
    }
    if !path.is_file() {
        return Err(format!("{file}: file でない"));
    }
    let bytes = fs::read(&path).map_err(|e| format!("{file}: 読めない: {e}"))?;
    files.insert(format!("sources/{file}"), bytes);
    Ok(())
}

/// 生成した面の写し。doc の id から面の file の名の形を床の定数の表で引き、当たる file を `--faces` の直下から
/// 名の byte 順に `faces/<名>` へ byte のまま写す（folio.css・folio-ui.js は写さない）。
fn copy_faces(
    faces_dir: &Path,
    face_files: &[(String, bool)],
    doc: &str,
    files: &mut Files,
) -> R<()> {
    let Some((_, form)) = FACE_NAMES.iter().find(|(id, _)| *id == doc) else {
        return Err(format!("{doc}: 面の名の形が床の定数に無い"));
    };
    let Some(form) = form else {
        return Ok(());
    };
    let mut hit = 0;
    for (name, is_file) in face_files {
        if !*is_file || !form.matches(name) {
            continue;
        }
        let path = faces_dir.join(name);
        let bytes = fs::read(&path).map_err(|e| format!("{}: 読めない: {e}", path.display()))?;
        files.insert(format!("faces/{name}"), bytes);
        hit += 1;
    }
    if hit == 0 {
        return Err(format!("{doc}: 面が無い（{}）", faces_dir.display()));
    }
    Ok(())
}

/// dir の直下の名（UTF-8 に読めるものだけ・byte 順）と file かどうか。
pub fn read_dir_names(dir: &Path) -> R<Vec<(String, bool)>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: 読めない: {e}", dir.display()))? {
        let entry = entry.map_err(|e| format!("{}: 読めない: {e}", dir.display()))?;
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        let is_file = entry.path().is_file();
        names.push((name, is_file));
    }
    names.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    Ok(names)
}

/// question.yaml（6 行・reader と question は区間の文で 2 字下げ）。便 42（`findings::refute`）は反証の束の question.yaml の
/// 頭 6 行としてこれを呼ぶ（振る舞い不変）。
pub fn question_text(vp: &Viewpoint) -> String {
    let block = |text: &str| -> String {
        text.trim_end_matches('\n')
            .split('\n')
            .map(|line| format!("  {line}\n"))
            .collect()
    };
    format!(
        "id: {}\nname: {}\nreader: |\n{}question: |\n{}",
        vp.id,
        vp.name,
        block(&vp.reader),
        block(&vp.question)
    )
}

/// reads.yaml（reads の各行を天井の正本の順に 1 行ずつ）。
fn reads_text(vp: &Viewpoint) -> String {
    vp.reads
        .iter()
        .map(|(doc, fields)| format!("- {{doc: {doc}, fields: [{}]}}\n", fields.join(", ")))
        .collect()
}

/// digest.txt の 1 行。束の file（digest.txt を除く）を相対 path の byte 順に並べ、中身を連結した sha256。
pub fn digest_text(files: &Files) -> String {
    let mut bytes = Vec::new();
    for (path, body) in files {
        if path != DIGEST_FILE {
            bytes.extend_from_slice(body);
        }
    }
    format!("{BUNDLE_DIGEST} {}\n", sha256::hex(&bytes))
}

#[cfg(test)]
mod ceiling_src_tests {
    use super::*;

    #[test]
    fn bundle_face_name_forms_follow_the_site_outputs() {
        let form = |doc: &str| FACE_NAMES.iter().find(|(id, _)| *id == doc).unwrap().1;
        assert!(form("index").unwrap().matches("index.html"));
        assert!(!form("index").unwrap().matches("index.htm"));
        assert!(form("adr").unwrap().matches("adr-12.html"));
        assert!(!form("adr").unwrap().matches("adr-.html"));
        assert!(form("design-note").unwrap().matches("note-full.html"));
        assert!(!form("design-note").unwrap().matches("folio.css"));
        for doc in ["rules", "vocabulary", "intake", "ceiling", "graph"] {
            assert!(form(doc).is_none(), "{doc}");
        }
    }

    #[test]
    fn bundle_digest_concatenates_in_path_byte_order_without_itself() {
        let mut files = Files::new();
        files.insert("sources/a.yaml".into(), b"cd".to_vec());
        files.insert("faces/x.html".into(), b"ab".to_vec());
        files.insert(DIGEST_FILE.into(), b"junk".to_vec());
        let line = digest_text(&files);
        assert_eq!(line, format!("{BUNDLE_DIGEST} {}\n", sha256::hex(b"abcd")));
    }
}
