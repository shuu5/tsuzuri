//! `folio ceiling --gate`（便 73・docs/design/delivery-73.md §1 (a)〜(c)・FR20 / AC18・設計ノート ceiling-gate.md §2）。
//! 天井の印 `<dir>/preview/ceiling-stamp.yaml`（便 72 の欄の決まり）と便の書き換える file の一覧（`--write-set`）から
//! 3 値を返す: 設計文書の正本を書き換えない便は 通す（0）・印が 4 観点とも合格で正本の要約値が同じなら 通す（0）・
//! 不合格 が在れば 止める（1）・印が無い / 古い / まだ分からない が在れば まだ分からない（2）。
//! 設計文書の正本 = `<dir>` の下に在り、`<dir>/preview/` の下でなく、path のどの要素も retired でないもの。
//! 正本の要約値は観点の reads が指す文書の file（file 形はその file・dir 形は直下の .yaml）の全文を `<dir>` からの
//! 相対 path の byte 順に連結した sha256。印（`stamp.rs`）も欄 sources をこの関数で測る（便 104・2 面に実装しない）。
//! 何も書かない。標準出力は 1 行「folio ceiling: <3 値>（<理由>）」。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::bundle;
use crate::face::R;
use crate::sha256;
use crate::stamp::STAMP_FILE;
use crate::verdict::Verdict;
use crate::yaml::{self, Node};

/// 1 回の実行の結果。`stdout` は 1 行。
pub struct Outcome {
    pub verdict: Verdict,
    pub stdout: String,
}

impl Outcome {
    fn new(verdict: Verdict, reason: impl Into<String>) -> Self {
        let word = match verdict {
            Verdict::Pass => "通す",
            Verdict::Fail => "止める",
            Verdict::Unknown => "まだ分からない",
        };
        Outcome {
            verdict,
            stdout: format!("folio ceiling: {word}（{}）", reason.into()),
        }
    }
}

// ── 命令の口 ──

/// `write_set` の各 path は repo の根からの相対（接頭辞 + / - / ~ は剥がす）。`dir` は同じ根からの `--dir`。
pub fn run(dir: &Path, write_set: &[String]) -> Outcome {
    let root = dir_parts(dir);
    if !write_set.iter().any(|p| is_design_source(&root, p)) {
        return Outcome::new(Verdict::Pass, "設計文書の正本を書き換えない便");
    }
    let stamp = match read_stamp(dir) {
        Ok(Some(s)) => s,
        Ok(None) => return Outcome::new(Verdict::Unknown, "印が無い"),
        Err(e) => return Outcome::new(Verdict::Unknown, format!("印が読めない: {e}")),
    };
    let ceiling = match bundle::load(dir) {
        Ok(c) => c,
        Err(e) => return Outcome::new(Verdict::Unknown, e),
    };
    match sources_digest(dir, &ceiling) {
        Ok(now) if now == stamp.sources => {}
        Ok(_) => return Outcome::new(Verdict::Unknown, "印が古い"),
        Err(e) => return Outcome::new(Verdict::Unknown, e),
    }
    let mut unknown = Vec::new();
    let mut failed = Vec::new();
    for vp in &ceiling.viewpoints {
        match stamp.viewpoints.iter().find(|(id, _)| *id == vp.id) {
            Some((_, v)) if v == "合格" => {}
            Some((_, v)) if v == "不合格" => failed.push(vp.id.as_str()),
            _ => unknown.push(vp.id.as_str()),
        }
    }
    if !unknown.is_empty() {
        return Outcome::new(
            Verdict::Unknown,
            format!("まだ分からない観点: {}", unknown.join("・")),
        );
    }
    if !failed.is_empty() {
        return Outcome::new(Verdict::Fail, format!("不合格の観点: {}", failed.join("・")));
    }
    Outcome::new(Verdict::Pass, "印が 4 観点とも合格・正本の要約値が同じ")
}

// ── 設計文書の判定（§1 (b)）──

/// path を要素に分ける（`.` と空の要素は落とす）。
fn parts(path: &str) -> Vec<&str> {
    path.split('/').filter(|s| !s.is_empty() && *s != ".").collect()
}

/// `--dir` の要素。絶対 path なら今の dir からの相対に直せるときだけ直す。
fn dir_parts(dir: &Path) -> Vec<String> {
    let rel: PathBuf = if dir.is_absolute() {
        std::env::current_dir()
            .ok()
            .and_then(|cwd| dir.strip_prefix(cwd).ok().map(Path::to_path_buf))
            .unwrap_or_else(|| dir.to_path_buf())
    } else {
        dir.to_path_buf()
    };
    rel.components()
        .filter_map(|c| match c {
            Component::Normal(s) => s.to_str().map(str::to_string),
            _ => None,
        })
        .collect()
}

/// 設計文書の正本か: `<dir>` の下・`<dir>/preview/` の下でない・どの要素も retired でない。
fn is_design_source(root: &[String], path: &str) -> bool {
    let path = path.trim_start_matches(['+', '-', '~']);
    let parts = parts(path);
    if parts.len() <= root.len() || parts.iter().zip(root).any(|(a, b)| *a != b) {
        return false;
    }
    let rest = &parts[root.len()..];
    rest[0] != "preview" && !parts.contains(&"retired")
}

// ── 印の判定（§1 (c)）──

/// 印から読む欄（sources と観点ごとの 3 値）。
struct Stamp {
    sources: String,
    viewpoints: Vec<(String, String)>,
}

/// 印を読む。無ければ None。
fn read_stamp(dir: &Path) -> R<Option<Stamp>> {
    let path = dir.join(STAMP_FILE);
    if path.is_symlink() {
        return Err("symlink は認めない".to_string());
    }
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).map_err(|e| format!("読めない: {e}"))?;
    let root = yaml::parse(&text)
        .map_err(|e| format!("parse できない: {e}"))?
        .root;
    let sources = root
        .get("sources")
        .and_then(Node::as_str)
        .ok_or_else(|| "sources が読めない".to_string())?
        .to_string();
    let viewpoints = root
        .get("viewpoints")
        .and_then(Node::as_seq)
        .and_then(|rows| {
            rows.iter()
                .map(|row| {
                    let id = row.get("id").and_then(Node::as_str)?;
                    let verdict = row.get("verdict").and_then(Node::as_str)?;
                    Some((id.to_string(), verdict.to_string()))
                })
                .collect::<Option<Vec<_>>>()
        })
        .ok_or_else(|| "viewpoints が読めない".to_string())?;
    Ok(Some(Stamp {
        sources,
        viewpoints,
    }))
}

/// 今の正本の要約値（「sha256 <16 進>」）。観点の reads が指す文書だけを全文で集める（印の sources も同じ関数・便 104）。
pub(crate) fn sources_digest(dir: &Path, ceiling: &bundle::Ceiling) -> R<String> {
    let documents = documents(dir)?;
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut seen: Vec<&str> = Vec::new();
    for vp in &ceiling.viewpoints {
        for (doc, _) in &vp.reads {
            if seen.contains(&doc.as_str()) {
                continue;
            }
            seen.push(doc);
            let file = documents
                .iter()
                .find(|(id, _)| id == doc)
                .map(|(_, f)| f.as_str())
                .ok_or_else(|| format!("{doc}: 文書の一覧に無い"))?;
            collect(dir, file, &mut files)?;
        }
    }
    let bytes: Vec<u8> = files.into_values().flatten().collect();
    Ok(format!("sha256 {}", sha256::hex(&bytes)))
}

/// 天井の正本の documents（id・file）。
pub(crate) fn documents(dir: &Path) -> R<Vec<(String, String)>> {
    let text = fs::read_to_string(dir.join("ceiling.yaml"))
        .map_err(|e| format!("ceiling.yaml: 読めない: {e}"))?;
    let root = yaml::parse(&text)
        .map_err(|e| format!("ceiling.yaml: parse できない: {e}"))?
        .root;
    root.get("documents")
        .and_then(Node::as_seq)
        .and_then(|rows| {
            rows.iter()
                .map(|row| {
                    let id = row.get("id").and_then(Node::as_str)?;
                    let file = row.get("file").and_then(Node::as_str)?;
                    Some((id.to_string(), file.to_string()))
                })
                .collect::<Option<Vec<_>>>()
        })
        .ok_or_else(|| "ceiling.yaml: documents: 読めない".to_string())
}

/// 文書 1 つの file。file 形はその file、dir 形（末尾が /）は直下の .yaml（`<dir>` からの相対 path で持つ）。
pub(crate) fn collect(dir: &Path, file: &str, files: &mut BTreeMap<String, Vec<u8>>) -> R<()> {
    let path = dir.join(file);
    if path.is_symlink() {
        return Err(format!("{file}: symlink は認めない"));
    }
    if !file.ends_with('/') {
        let bytes = fs::read(&path).map_err(|e| format!("{file}: 読めない: {e}"))?;
        files.insert(file.to_string(), bytes);
        return Ok(());
    }
    for (name, is_file) in bundle::read_dir_names(&path)? {
        if !is_file || !name.ends_with(".yaml") {
            continue;
        }
        let entry = path.join(&name);
        if entry.is_symlink() {
            return Err(format!("{file}{name}: symlink は認めない"));
        }
        let bytes = fs::read(&entry).map_err(|e| format!("{file}{name}: 読めない: {e}"))?;
        files.insert(format!("{file}{name}"), bytes);
    }
    Ok(())
}

#[cfg(test)]
mod gate_tests {
    use super::*;

    #[test]
    fn gate_classifies_design_sources() {
        let root = vec!["design-intent".to_string()];
        let yes = |p: &str| is_design_source(&root, p);
        assert!(yes("design-intent/srs.yaml"));
        assert!(yes("+design-intent/adr/ADR-9.yaml"));
        assert!(yes("~./design-intent/design-note/x.yaml"));
        assert!(!yes("design-intent"));
        assert!(!yes("design-intent/preview/ceiling-stamp.yaml"));
        assert!(!yes("design-intent/adr/retired/ADR-0.yaml"));
        assert!(!yes("crates/folio/src/gate.rs"));
        assert!(!yes("docs/design/delivery-73.md"));
        assert!(!yes("design-intent-x/srs.yaml"));
    }
}
