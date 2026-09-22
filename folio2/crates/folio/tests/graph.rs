//! `folio graph --print`（便 94・docs/design/delivery-94.md §1 (g)）の歯。folio は実行 file の crate なので命令を撃つ。
//! 1. 凍結した土台の写しの索引が凍結 anchor tests/fixtures/schema/graph-anchor.txt の 3 つの要約値と一致し、要約の 1 行が anchor の数え。
//! 2. 実の正本の索引の節点の種類が閉じた一覧 11 語の中、辺の型が 17 語の中（数は固定しない）。
//! 3. 辺の端がすべて節点で、要約の 1 行の 3 つの数が実際の表の行数と型の異なり数に一致。
//! 4. 2 度当てて byte 一致し、写しの file を 1 つも変えない。
//! 5. 正本を 1 つ消すと終了コード 2 で表が 1 行も出ない。
//! 6. 改行を含む題が 1 行 36 字以下に畳まれる。

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// 閉じた一覧の写し（正本は crates/folio/src/graph.rs の NODE_KINDS / EDGE_TYPES・歯は crate の中を読めない）。
const NODE_KINDS: [&str; 11] = [
    "条",
    "規範文",
    "規則行",
    "目的",
    "要件",
    "非機能要件",
    "受入基準",
    "制約",
    "登場人物",
    "出力",
    "判断の記録",
];
const EDGE_TYPES: [&str; 17] = [
    "in-article",
    "relations.articles",
    "relations.reqs",
    "relations.rules",
    "relations.sections",
    "amended_by",
    "article",
    "refs",
    "basis",
    "goals",
    "rules",
    "adrs",
    "verify.ac",
    "verifies",
    "figures",
    "produced",
    "amends",
];

const NODES_HEAD: &str = "# 節点（1 行 = id / 種類 / file / 題 36 字・タブ区切り）";
const EDGES_HEAD: &str = "# 辺（1 行 = 端 / 端 / 型・タブ区切り）";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &to);
        } else {
            fs::copy(entry.path(), &to).unwrap();
        }
    }
}

/// 置き場の下の file の相対 path → 中身（写しを変えていないことを見る）。
fn snapshot(dir: &Path, base: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            snapshot(&path, base, out);
        } else {
            out.insert(
                path.strip_prefix(base).unwrap().to_path_buf(),
                fs::read(&path).unwrap(),
            );
        }
    }
}

/// 要約値（sha256）は命令 `sha256sum` を子の処理で撃って測る（歯は crate の中を読めない・tests/schema.rs と同じ形）。
fn sha256_hex(bytes: &[u8]) -> Result<String, String> {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("sha256sum を起動できない: {e}"))?;
    child
        .stdin
        .take()
        .ok_or_else(|| "sha256sum の標準入力が無い".to_string())?
        .write_all(bytes)
        .map_err(|e| format!("sha256sum へ書けない: {e}"))?;
    let out = child
        .wait_with_output()
        .map_err(|e| format!("sha256sum を待てない: {e}"))?;
    if !out.status.success() {
        return Err("sha256sum が失敗した".to_string());
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let hex = text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    if hex.len() != 64 {
        return Err(format!("sha256sum の出力が 16 進 64 字でない: {text}"));
    }
    Ok(hex)
}

/// 正本の写しの一時 dir（歯の終わりに消す）。
struct Work {
    root: PathBuf,
}

impl Work {
    fn new(case: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-graph-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_tree(&repo_root().join("design-intent"), &root.join("design-intent"));
        Work { root }
    }

    fn dir(&self) -> PathBuf {
        self.root.join("design-intent")
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn print(dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(["graph", "--print", "--dir"])
        .arg(dir)
        .output()
        .expect("folio を起動できない")
}

/// 合格で終わった出力の字。
fn printed(dir: &Path) -> String {
    let out = print(dir);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("出力が UTF-8 でない")
}

/// 出力を割ったもの: 節点の行・辺の行・要約の 1 行。
struct Parts<'a> {
    nodes: Vec<&'a str>,
    edges: Vec<&'a str>,
    summary: &'a str,
}

fn split(text: &str) -> Parts<'_> {
    assert!(text.ends_with('\n'), "最後の行が改行で終わらない");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.first(), Some(&NODES_HEAD), "節点の表の見出し");
    let at = lines
        .iter()
        .position(|l| *l == EDGES_HEAD)
        .expect("辺の表の見出しが無い");
    let summary = *lines.last().unwrap();
    assert!(summary.starts_with("# 節点 "), "要約の 1 行: {summary}");
    Parts {
        nodes: lines[1..at].to_vec(),
        edges: lines[at + 1..lines.len() - 1].to_vec(),
        summary,
    }
}

/// 行の連なりを、各行を改行で終えた byte 列に。
fn joined(lines: &[&str]) -> Vec<u8> {
    lines.iter().flat_map(|l| format!("{l}\n").into_bytes()).collect()
}

/// anchor の 1 行「<名> sha256 = <16 進 64 字>…」の値。
fn anchor_hex<'a>(anchor: &'a str, name: &str) -> &'a str {
    let line = anchor
        .lines()
        .find(|l| l.starts_with(&format!("{name} sha256 = ")))
        .unwrap_or_else(|| panic!("anchor に {name} の行が無い"));
    let rest = &line[format!("{name} sha256 = ").len()..];
    &rest[..64]
}

#[test]
fn f94_the_frozen_base_index_matches_the_anchor() {
    let anchor = fs::read_to_string(repo_root().join("tests/fixtures/schema/graph-anchor.txt"))
        .expect("凍結 anchor を読めない");
    let text = printed(&repo_root().join("tests/fixtures/floor_base/design-intent"));
    let parts = split(&text);
    let counts = anchor.lines().nth(1).expect("anchor の 2 行目が無い");
    assert_eq!(counts, "# 節点 189・辺 576・型 10・端が節点でない参照 26");
    assert_eq!(parts.summary, counts, "要約の 1 行");
    assert_eq!(text.lines().count(), 768, "出力の行数");
    assert_eq!(text.len(), 29_558, "出力の byte 数");
    for (name, bytes) in [
        ("節点の表", joined(&parts.nodes)),
        ("辺の表", joined(&parts.edges)),
        ("出力全体", text.clone().into_bytes()),
    ] {
        match sha256_hex(&bytes) {
            Ok(hex) => assert_eq!(hex, anchor_hex(&anchor, name), "{name} の要約値"),
            Err(e) => panic!("{name} の要約値を測れない: {e}"),
        }
    }
}

#[test]
fn f94_the_real_sources_build_the_closed_lists() {
    let text = printed(&repo_root().join("design-intent"));
    let parts = split(&text);
    assert!(!parts.nodes.is_empty(), "節点が 1 つも無い");
    assert!(!parts.edges.is_empty(), "辺が 1 本も無い");
    for line in &parts.nodes {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 4, "節点の行の欄の数: {line}");
        assert!(NODE_KINDS.contains(&cols[1]), "閉じた一覧に無い種類: {line}");
    }
    for line in &parts.edges {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 3, "辺の行の欄の数: {line}");
        assert!(EDGE_TYPES.contains(&cols[2]), "閉じた一覧に無い型: {line}");
    }
}

#[test]
fn f94_the_summary_line_agrees_with_the_tables() {
    let text = printed(&repo_root().join("design-intent"));
    let parts = split(&text);
    let ids: BTreeSet<&str> = parts
        .nodes
        .iter()
        .map(|l| l.split('\t').next().unwrap())
        .collect();
    assert_eq!(ids.len(), parts.nodes.len(), "節点の id が重複している");
    let mut types = BTreeSet::new();
    for line in &parts.edges {
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(ids.contains(cols[0]), "端が節点でない辺: {line}");
        assert!(ids.contains(cols[1]), "端が節点でない辺: {line}");
        types.insert(cols[2]);
    }
    let head = format!(
        "# 節点 {}・辺 {}・型 {}・端が節点でない参照 ",
        parts.nodes.len(),
        parts.edges.len(),
        types.len()
    );
    assert!(
        parts.summary.starts_with(&head),
        "要約の 1 行 {} が表の数え {head} と合わない",
        parts.summary
    );
    let rest = &parts.summary[head.len()..];
    assert!(
        !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()),
        "端が節点でない参照の数: {rest}"
    );
}

#[test]
fn f94_the_output_is_deterministic_and_writes_nothing() {
    let work = Work::new("determinism");
    let mut before = BTreeMap::new();
    snapshot(&work.root, &work.root, &mut before);
    let first = printed(&work.dir());
    let second = printed(&work.dir());
    assert_eq!(first, second, "2 度当てた出力が byte 一致しない");
    let mut after = BTreeMap::new();
    snapshot(&work.root, &work.root, &mut after);
    assert!(before == after, "写しの file が変わった");
}

#[test]
fn f94_a_broken_source_is_inconclusive() {
    let work = Work::new("broken");
    fs::remove_file(work.dir().join("constitution.yaml")).unwrap();
    let out = print(&work.dir());
    assert_eq!(out.status.code(), Some(2), "終了コード");
    assert!(out.stdout.is_empty(), "表が出た: {}", String::from_utf8_lossy(&out.stdout));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("まだ分からない"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn f94_the_title_is_folded_into_one_line() {
    let work = Work::new("fold");
    let path = work.dir().join("constitution.yaml");
    let from = "      - {id: P-1.1, pattern: ubiquitous, strength: must, text: folio は文書を生成し、機械で検査し、結果を知らせるところまでを担う。}\n";
    let to = "      - id: P-1.1\n        pattern: ubiquitous\n        strength: must\n        text: |\n          folio は   文書を\n            生成し、\t機械で  検査し、\n          結果を知らせるところまでを担う。\n";
    let text = fs::read_to_string(&path).unwrap();
    assert_eq!(text.matches(from).count(), 1, "P-1.1 の行が 1 つでない");
    fs::write(&path, text.replacen(from, to, 1)).unwrap();
    let out = printed(&work.dir());
    let line = split(&out)
        .nodes
        .into_iter()
        .find(|l| l.starts_with("P-1.1\t"))
        .expect("P-1.1 の節点の行が無い");
    let title = line.split('\t').nth(3).expect("題の欄が無い");
    assert!(!title.contains('\n') && !title.contains('\t'), "題: {title:?}");
    assert!(!title.contains("  "), "連なった空白が残った: {title:?}");
    assert_eq!(title, title.trim(), "前後の空白が残った: {title:?}");
    assert!(title.chars().count() <= 36, "題が 36 字を超えた: {title:?}");
    assert!(title.starts_with("folio は 文書を 生成し、 機械で 検査し、"), "題: {title:?}");
}
