//! `folio ceiling --write`（便 38・docs/design/delivery-38.md §1 (a)〜(d)）。天井（AI による意味の検査・ADR-8 決定 (2)・FR17）の
//! 材料の束を観点ごとに 1 つの置き場へ組む。folio は AI を起動しない。束を読んで AI を回すのは席か器で、所見と起動の記録を
//! 同じ置き場へ書く（便 39 がそれを数える）。
//! 束の中身 = 天井の正本 `bundle.contents` の 5 つ（sources/・faces/・question.yaml・finding.yaml・reads.yaml）と要約値 digest.txt。
//! 要約値の規則 = 5 つの下の file を観点の dir からの相対 path の byte 順に並べ、中身を区切りなしに連結した byte 列の sha256
//! （rules 行 R-15 の写しの要約値と同じ規則・`sha256::hex`）。
//! 決定性: 同じ入力から byte まで同じ束が組める（時刻・絶対 path・環境の値をどの file にも書かない）。
//! 全部か無しか: 4 観点の全 file を memory の上で先に用意し、1 本でも用意できなければ何も書かず 2（P-4.1）。
//! 書くときは各観点の sources/ と faces/ を消してから作り直す（古い写しが要約値に混ざらないため）。
//! 席や器が書く所見 file・起動の記録は触らない。判定を持たないので 1（不合格）は返さない（FR5 の 3 値のうち 2 つ）。
//! 便 39（`findings.rs`・`--check`）は正本の読み `load`・観点 1 つの組み立て `build_one`・要約値 `digest_text` を
//! crate の中から呼ぶ（--write の振る舞いと文言は不変）。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::face::R;
use crate::sha256;
use crate::verdict::Verdict;
use crate::yaml::{self, Node};

/// 天井の正本の file。
const FILE: &str = "ceiling.yaml";

/// 床が組める束の中身（天井の正本 `bundle.contents` の逐語・順も同じ）。
pub const CONTENTS: [&str; 5] = ["sources", "faces", "question", "finding", "reads"];

/// 要約値の規則の名（天井の正本 `bundle.digest` の逐語）。
pub const DIGEST: &str = "sha256-files-1";

/// 束の要約値の file（要約値の計算には数えない）。
pub const DIGEST_FILE: &str = "digest.txt";

/// 文書の id → 面の file の名の形（`site.rs` の配信先の名の写し・床の定数）。`None` = 面は無い。
pub const FACE_NAMES: [(&str, Option<FaceName>); 9] = [
    ("index", Some(FaceName::Exact("index.html"))),
    ("constitution", Some(FaceName::Exact("constitution.html"))),
    ("srs", Some(FaceName::Exact("srs.html"))),
    ("adr", Some(FaceName::Affix("adr-", ".html"))),
    ("design-note", Some(FaceName::Affix("note-", ".html"))),
    ("rules", None),
    ("vocabulary", None),
    ("intake", None),
    ("ceiling", None),
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
    /// 所見の欄の決まり（finding・weights・verdicts・record の値）
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

/// 所見の欄の決まり（天井の正本 finding・weights・verdicts・record の写し・finding.yaml の材料と `--check` の値域）。
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

// ── 命令の口 ──

/// `--faces` と `--out` は相対なら `--dir` からの相対・絶対ならそのまま（`folio build` と同じ読み）。
pub fn run(dir: &Path, faces: &Path, out: &Path) -> Outcome {
    let faces_dir = dir.join(faces);
    let out_dir = dir.join(out);
    let ceiling = match load(dir) {
        Ok(c) => c,
        Err(e) => return Outcome::unknown(e),
    };
    let bundles = match build_all(dir, &faces_dir, &ceiling) {
        Ok(b) => b,
        Err(e) => return Outcome::unknown(e),
    };
    write_all(&out_dir, &bundles)
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
    let bundle = table_of(&root, "bundle", "")?;
    if list_of(bundle, "contents", "bundle")? != CONTENTS
        || text_of(bundle, "digest", "bundle")? != DIGEST
    {
        return Err(format!("{FILE}: bundle: 床が組める形でない"));
    }
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

/// 所見の欄の決まり（天井の正本の finding・weights・verdicts・record）。
fn load_rules(root: &Node) -> R<Rules> {
    let finding = table_of(root, "finding", "")?;
    let place = table_of(finding, "place", "finding")?;
    let refute = table_of(finding, "refute", "finding")?;
    let weights = table_of(root, "weights", "")?;
    let verdicts = table_of(root, "verdicts", "")?;
    let record = table_of(root, "record", "")?;
    Ok(Rules {
        finding_required: list_of(finding, "required", "finding")?,
        finding_optional: list_of(finding, "optional", "finding")?,
        place_required: list_of(place, "required", "finding.place")?,
        refute_values: list_of(refute, "values", "finding.refute")?,
        weight_values: list_of(weights, "values", "weights")?,
        weight_refute: list_of(weights, "refute", "weights")?,
        verdict_values: list_of(verdicts, "values", "verdicts")?,
        record_required: list_of(record, "required", "record")?,
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

/// 4 観点の束を全部 memory の上で用意する（1 本でも用意できなければ Err）。戻り値 = (観点の id, file) の列（viewpoints の順）。
fn build_all(dir: &Path, faces_dir: &Path, ceiling: &Ceiling) -> R<Vec<(String, Files)>> {
    if !faces_dir.is_dir() {
        return Err(format!("{}: 配信先が無い", faces_dir.display()));
    }
    let face_files = read_dir_names(faces_dir)?;
    let mut bundles = Vec::with_capacity(ceiling.viewpoints.len());
    for vp in &ceiling.viewpoints {
        let files = build_one(dir, faces_dir, &face_files, ceiling, vp)?;
        bundles.push((vp.id.clone(), files));
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
    let mut files = Files::new();
    let mut seen: Vec<&str> = Vec::new();
    for (doc, _) in &vp.reads {
        if seen.contains(&doc.as_str()) {
            continue;
        }
        seen.push(doc);
        let file = &ceiling
            .documents
            .iter()
            .find(|(id, _)| id == doc)
            .expect("reads の doc は load で documents に解いてある")
            .1;
        copy_sources(dir, file, &mut files)?;
        copy_faces(faces_dir, face_files, doc, &mut files)?;
    }
    files.insert("question.yaml".to_string(), question_text(vp).into_bytes());
    files.insert(
        "finding.yaml".to_string(),
        ceiling.finding.clone().into_bytes(),
    );
    files.insert("reads.yaml".to_string(), reads_text(vp).into_bytes());
    let digest = digest_text(&files);
    files.insert(DIGEST_FILE.to_string(), digest.into_bytes());
    Ok(files)
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

/// question.yaml（6 行・reader と question は区間の文で 2 字下げ）。
fn question_text(vp: &Viewpoint) -> String {
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
    format!("{DIGEST} {}\n", sha256::hex(&bytes))
}

// ── 置き場へ書く ──

fn write_all(out_dir: &Path, bundles: &[(String, Files)]) -> Outcome {
    if !out_dir.is_dir() {
        if !out_dir.parent().is_some_and(Path::is_dir) {
            return Outcome::unknown(format!("{}: 置き場の親 dir が無い", out_dir.display()));
        }
        if let Err(e) = fs::create_dir(out_dir) {
            return Outcome::unknown(format!("{}: 置き場を作れない: {e}", out_dir.display()));
        }
    }
    let mut count = 0;
    let mut total = 0;
    for (id, files) in bundles {
        let vp_dir = out_dir.join(id);
        // 古い写しが要約値に混ざらないよう sources/ と faces/ は消してから作り直す。他の file は触らない
        for sub in ["sources", "faces"] {
            let path = vp_dir.join(sub);
            if path.exists()
                && let Err(e) = fs::remove_dir_all(&path)
            {
                return Outcome::unknown(format!("{}: 消せない: {e}", path.display()));
            }
        }
        for (rel, bytes) in files {
            let path: PathBuf = vp_dir.join(rel);
            if let Some(parent) = path.parent()
                && let Err(e) = fs::create_dir_all(parent)
            {
                return Outcome::unknown(format!("{}: 作れない: {e}", parent.display()));
            }
            if let Err(e) = fs::write(&path, bytes) {
                return Outcome::unknown(format!("{}: 書けない: {e}", path.display()));
            }
            count += 1;
            total += bytes.len();
        }
    }
    Outcome {
        verdict: Verdict::Pass,
        stdout: Some(format!(
            "folio ceiling: 束を組んだ（観点 {}・file {count}・{total} byte）",
            bundles.len()
        )),
        stderr: None,
    }
}

#[cfg(test)]
mod bundle_tests {
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
        for doc in ["rules", "vocabulary", "intake", "ceiling"] {
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
        assert_eq!(line, format!("{DIGEST} {}\n", sha256::hex(b"abcd")));
    }
}
