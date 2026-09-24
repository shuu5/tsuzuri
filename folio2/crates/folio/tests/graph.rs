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

const NODES_HEAD: &str = "# 節点（1 行 = id / 種類 / file / 要約値 8 字 / 題 36 字・タブ区切り）";
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
        Work::from(case, "design-intent")
    }

    /// 凍結した土台 tests/fixtures/floor_base/design-intent の写し（便 99）。
    fn base(case: &str) -> Work {
        Work::from(case, "tests/fixtures/floor_base/design-intent")
    }

    fn from(case: &str, src: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-graph-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_tree(&repo_root().join(src), &root.join("design-intent"));
        Work { root }
    }

    /// 写しの file の中の `from`（ちょうど 1 か所）を `to` に替える。
    fn replace(&self, file: &str, from: &str, to: &str) {
        let path = self.dir().join(file);
        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(text.matches(from).count(), 1, "{file}: 「{from}」が 1 か所でない");
        fs::write(&path, text.replacen(from, to, 1)).unwrap();
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

fn graph(dir: &Path, flag: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(["graph", flag, "--dir"])
        .arg(dir)
        .output()
        .expect("folio を起動できない")
}

fn print(dir: &Path) -> Output {
    graph(dir, "--print")
}

/// 合格で終わった出力の字。
fn printed(dir: &Path) -> String {
    passed(print(dir))
}

/// 合格で終わった短い出力の字（便 96）。
fn digested(dir: &Path) -> String {
    passed(graph(dir, "--digest"))
}

fn passed(out: Output) -> String {
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
    assert_eq!(text.len(), 31_277, "出力の byte 数（便 99 で要約値の欄を足した）");
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
        assert_eq!(cols.len(), 5, "節点の行の欄の数: {line}");
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
    let title = line.split('\t').nth(4).expect("題の欄が無い");
    assert!(!title.contains('\n') && !title.contains('\t'), "題: {title:?}");
    assert!(!title.contains("  "), "連なった空白が残った: {title:?}");
    assert_eq!(title, title.trim(), "前後の空白が残った: {title:?}");
    assert!(title.chars().count() <= 36, "題が 36 字を超えた: {title:?}");
    assert!(title.starts_with("folio は 文書を 生成し、 機械で 検査し、"), "題: {title:?}");
}

/// 短い出力（便 96）の 3 表の見出しと最後の 1 行。
const KINDS_HEAD: &str = "# 種類ごとの節点（1 行 = 種類 / 数・タブ区切り・閉じた一覧の順）";
const TYPES_HEAD: &str =
    "# 型ごとの辺（1 行 = 型 / 表に出た数 / 端が節点でない数・タブ区切り・閉じた一覧の順）";
const FILES_HEAD: &str = "# file ごとの節点（1 行 = file / 数・タブ区切り・file の名の byte 順）";
const NEXT_LINE: &str = "# 索引そのもの（節点と辺の全行）は folio graph --print";

/// 短い出力を割ったもの: 3 表の行と要約の 1 行。
struct Digest<'a> {
    kinds: Vec<Vec<&'a str>>,
    types: Vec<Vec<&'a str>>,
    files: Vec<Vec<&'a str>>,
    summary: &'a str,
}

fn split_digest<'a>(text: &'a str) -> Digest<'a> {
    assert!(text.ends_with('\n'), "最後の行が改行で終わらない");
    let lines: Vec<&str> = text.lines().collect();
    let at = |head: &str| {
        lines
            .iter()
            .position(|l| *l == head)
            .unwrap_or_else(|| panic!("見出しが無い: {head}"))
    };
    let (k, t, f) = (at(KINDS_HEAD), at(TYPES_HEAD), at(FILES_HEAD));
    assert_eq!(k, 0, "1 表の見出しが先頭に無い");
    assert!(t < f, "表の順");
    let n = lines.len();
    assert!(n >= f + 3, "要約の 2 行が無い");
    assert_eq!(lines[n - 1], NEXT_LINE, "最後の行");
    let cols = |range: &[&'a str]| -> Vec<Vec<&'a str>> {
        range.iter().map(|l| l.split('\t').collect()).collect()
    };
    Digest {
        kinds: cols(&lines[k + 1..t]),
        types: cols(&lines[t + 1..f]),
        files: cols(&lines[f + 1..n - 2]),
        summary: lines[n - 2],
    }
}

fn num(s: &str) -> usize {
    s.parse().unwrap_or_else(|_| panic!("数でない: {s:?}"))
}

/// 要約の 1 行の 4 つの数（節点・辺・型・端が節点でない参照）。
fn summary_counts(summary: &str) -> Vec<usize> {
    summary
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .map(num)
        .collect()
}

#[test]
fn f96_the_digest_of_the_frozen_base_matches_the_anchor() {
    let anchor = fs::read(repo_root().join("tests/fixtures/schema/graph-digest-anchor.txt"))
        .expect("凍結 anchor を読めない");
    assert_eq!(anchor.len(), 1_048, "anchor の byte 数");
    assert_eq!(anchor.iter().filter(|b| **b == b'\n').count(), 46, "anchor の行数");
    let text = digested(&repo_root().join("tests/fixtures/floor_base/design-intent"));
    assert!(text.as_bytes() == anchor.as_slice(), "短い出力が anchor と byte 一致しない:\n{text}");
}

#[test]
fn f96_the_digest_agrees_with_the_print() {
    let dir = repo_root().join("design-intent");
    let index = printed(&dir);
    let text = digested(&dir);
    let d = split_digest(&text);
    assert_eq!(d.summary, split(&index).summary, "要約の 1 行が索引の最後の行と違う");
    let c = summary_counts(d.summary);
    assert_eq!(c.len(), 4, "要約の 1 行の数: {}", d.summary);
    let sum = |rows: &[Vec<&str>], col: usize| rows.iter().map(|r| num(r[col])).sum::<usize>();
    assert_eq!(sum(&d.kinds, 1), c[0], "種類ごとの合計");
    assert_eq!(sum(&d.types, 1), c[1], "型ごとの表に出た数の合計");
    assert_eq!(sum(&d.types, 2), c[3], "型ごとの端が節点でない数の合計");
    let used = d.types.iter().filter(|r| num(r[1]) > 0).count();
    assert_eq!(used, c[2], "表に出た型の数");
    assert_eq!(sum(&d.files, 1), c[0], "file ごとの合計");
}

#[test]
fn f96_the_digest_rows_are_the_closed_lists() {
    let text = digested(&repo_root().join("design-intent"));
    let d = split_digest(&text);
    let kinds: Vec<&str> = d.kinds.iter().map(|r| r[0]).collect();
    assert_eq!(kinds, NODE_KINDS, "1 表の語と順");
    assert!(d.kinds.iter().all(|r| r.len() == 2), "1 表の欄の数");
    let types: Vec<&str> = d.types.iter().map(|r| r[0]).collect();
    assert_eq!(types, EDGE_TYPES, "2 表の語と順");
    assert!(d.types.iter().all(|r| r.len() == 3), "2 表の欄の数");
    assert!(!d.files.is_empty(), "3 表が空");
    assert!(d.files.iter().all(|r| r.len() == 2), "3 表の欄の数");
    let names: Vec<&[u8]> = d.files.iter().map(|r| r[0].as_bytes()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(names, sorted, "3 表が file の名の byte 順でない");
}

#[test]
fn f96_a_broken_source_makes_the_digest_inconclusive() {
    let work = Work::new("digest-broken");
    fs::remove_file(work.dir().join("constitution.yaml")).unwrap();
    let mut before = BTreeMap::new();
    snapshot(&work.root, &work.root, &mut before);
    let first = graph(&work.dir(), "--digest");
    assert_eq!(first.status.code(), Some(2), "終了コード");
    assert!(first.stdout.is_empty(), "表が出た: {}", String::from_utf8_lossy(&first.stdout));
    let second = graph(&work.dir(), "--digest");
    assert_eq!(second.status.code(), Some(2), "2 度目の終了コード");
    assert_eq!(first.stdout, second.stdout, "2 度当てた標準出力が byte 一致しない");
    assert_eq!(first.stderr, second.stderr, "2 度当てた標準エラーが byte 一致しない");
    let mut after = BTreeMap::new();
    snapshot(&work.root, &work.root, &mut after);
    assert!(before == after, "写しの file が変わった");
}

// ── 便 99: 節点の要約値（docs/design/delivery-99.md §1 (h) の 1〜6） ──

const F99_SCRIPT: &str = "tests/fixtures/schema/node-digest.py";
const F99_ANCHOR: &str = "tests/fixtures/schema/node-digest-anchor.txt";
/// 便 119: 床の凍結の土台の導出物の検査の命令の名（folio build --check → folio derive --check）で残差の 2 行だけが動いた値。
const F99_ANCHOR_SHA256: &str = "45741553abe272237d1997dbd2c77d549a956b8253dc5eca8fced6fc67ec125d";
const FLOOR_BASE: &str = "tests/fixtures/floor_base/design-intent";

/// 独立の実装の出力を置き場に当てる。python3 を起動できなければ None（歯は理由を出して落とさない・P-10.3）。
fn independent(dir: &Path) -> Option<String> {
    match Command::new("python3")
        .arg(repo_root().join(F99_SCRIPT))
        .arg(dir)
        .output()
    {
        Ok(out) => Some(passed(out)),
        Err(e) => {
            eprintln!("# まだ分からない: node-digest.py: python3 を起動できない: {e}");
            None
        }
    }
}

/// 節点の行の id → 4 列目（要約値）。
fn digest_column(text: &str) -> BTreeMap<String, String> {
    split(text)
        .nodes
        .iter()
        .map(|line| {
            let cols: Vec<&str> = line.split('\t').collect();
            assert_eq!(cols.len(), 5, "節点の行の欄の数: {line}");
            (cols[0].to_string(), cols[3].to_string())
        })
        .collect()
}

#[test]
fn f99_the_independent_script_matches_the_anchor() {
    let anchor = fs::read(repo_root().join(F99_ANCHOR)).expect("凍結 anchor を読めない");
    assert_eq!(anchor.iter().filter(|b| **b == b'\n').count(), 192, "anchor の行数");
    assert_eq!(anchor.len(), 3_007, "anchor の byte 数");
    let hex = sha256_hex(&anchor).unwrap_or_else(|e| panic!("anchor の要約値を測れない: {e}"));
    assert_eq!(hex, F99_ANCHOR_SHA256, "sha256sum で測った anchor の要約値");
    if let Some(out) = independent(&repo_root().join(FLOOR_BASE)) {
        assert!(out.as_bytes() == anchor.as_slice(), "独立の実装の出力が anchor と違う:\n{out}");
    }
}

#[test]
fn f99_the_index_carries_the_digest_column() {
    let anchor = fs::read_to_string(repo_root().join("tests/fixtures/schema/graph-anchor.txt")).unwrap();
    let text = printed(&repo_root().join(FLOOR_BASE));
    let parts = split(&text);
    assert_eq!(Some(parts.summary), anchor.lines().nth(1), "要約の 1 行");
    assert_eq!((text.lines().count(), text.len()), (768, 31_277), "出力の行数と byte 数");
    for line in &parts.nodes {
        let digest = line.split('\t').nth(3).unwrap_or_default();
        assert!(
            digest.len() == 8 && digest.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')),
            "4 列目が 16 進の小文字 8 字でない: {line}"
        );
    }
    assert_eq!(digest_column(&text).len(), 189, "節点の数");
    let edges = sha256_hex(&joined(&parts.edges)).unwrap();
    assert_eq!(edges, anchor_hex(&anchor, "辺の表"), "辺の表の要約値");
    assert!(edges.starts_with("a09c3f55"), "辺の表の要約値が便 94 の値から動いた: {edges}");
    let nodes = sha256_hex(&joined(&parts.nodes)).unwrap();
    assert_eq!(nodes, anchor_hex(&anchor, "節点の表"), "節点の表の要約値");
    let whole = sha256_hex(text.as_bytes()).unwrap();
    assert_eq!(whole, anchor_hex(&anchor, "出力全体"), "出力全体の要約値");
}

#[test]
fn f99_the_digest_column_is_the_independent_value() {
    let anchor = fs::read_to_string(repo_root().join(F99_ANCHOR)).unwrap();
    let want: BTreeMap<String, String> = anchor
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| {
            let (id, hex) = l.split_once('\t').expect("anchor の行にタブが無い");
            (id.to_string(), hex.to_string())
        })
        .collect();
    assert_eq!(want.len(), 189, "anchor の節点の数");
    let got = digest_column(&printed(&repo_root().join(FLOOR_BASE)));
    assert_eq!(got, want, "索引の 4 列目が独立の実装の値と違う");
}

#[test]
fn f99_an_edge_only_change_moves_no_digest() {
    let work = Work::base("f99-edges");
    let before = digest_column(&printed(&work.dir()));
    work.replace(
        "constitution.yaml",
        "    relations: {reqs: [FR1, FR2, AC1]}\n",
        "    relations: {reqs: [FR1, FR2, FR3, AC1]}\n",
    );
    work.replace("rules.yaml", "  - {id: R-1, article: N-5, ", "  - {id: R-1, article: N-5, refs: [P-4], ");
    work.replace(
        "srs.yaml",
        "    basis: [P-1]\n    verify: {method: test, how: 決まった回答 5 つを入れ、支度表が期待どおりか比較する, ac: [AC1]}\n",
        "    basis: [P-1, P-4]\n    verify: {method: test, how: 決まった回答 5 つを入れ、支度表が期待どおりか比較する, ac: [AC1, AC2]}\n",
    );
    work.replace("adr/ADR-1.yaml", "basis: [P-8, ", "basis: [P-1, P-8, ");
    let after = digest_column(&printed(&work.dir()));
    assert_eq!(after.len(), 189, "節点の数");
    assert_eq!(before, after, "辺の欄だけの変更で要約値が動いた");
}

#[test]
fn f99_a_body_change_moves_exactly_one_digest() {
    let work = Work::base("f99-body");
    let before = digest_column(&printed(&work.dir()));
    work.replace("srs.yaml", "    shall: folio は 5 問以下", "    shall: folio は 6 問以下");
    let after = digest_column(&printed(&work.dir()));
    let moved: Vec<&String> = before.keys().filter(|id| before[*id] != after[*id]).collect();
    assert_eq!(moved, ["FR1"], "動いた要約値");
}

#[test]
fn f99_a_scan_that_disagrees_is_inconclusive() {
    let work = Work::base("f99-disagree");
    // 規範文の頭の行の id の欄を残したまま、行の逐語が節点の頭と読めない形に崩す（索引は YAML として読める）
    work.replace("constitution.yaml", "      - {id: P-1.1, ", "      -  {id: P-1.1, ");
    let mut before = BTreeMap::new();
    snapshot(&work.root, &work.root, &mut before);
    let out = print(&work.dir());
    let mut after = BTreeMap::new();
    snapshot(&work.root, &work.root, &mut after);
    assert_eq!(out.status.code(), Some(2), "終了コード");
    assert!(out.stdout.is_empty(), "表が出た: {}", String::from_utf8_lossy(&out.stdout));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("まだ分からない") && err.contains("食い違う") && err.contains("P-1.1"), "{err}");
    assert!(before == after, "写しの file が変わった");
}
