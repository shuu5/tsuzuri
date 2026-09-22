//! `folio ceiling --stamp`（便 72・docs/design/delivery-72.md §1 (c)(d)）の歯。binary 経由。
//! 周（stamp-case）は「4 観点が同じ文書を読む周」として組む: 便 38 の凍結 fixture の束（tests/fixtures/ceiling/bundle/）を
//! 一時 dir へ写し、写しの source/ceiling.yaml の 4 観点の reads を pass-coherence.yaml の record.read と同じ 5 文書
//! （constitution・rules・srs・adr・design-note）に揃えてから `--write` で組み、pass-coherence.yaml を 4 観点の findings.yaml に
//! 写す（record.bundle は観点ごとの digest.txt に合わせる）。凍結 anchor stamp-expected.yaml は触らない（P-10.1）。
//!
//! 版管理の下の file は書き換えない（`--dir` と `--out` は必ず一時 dir の中）。

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const VIEWPOINTS: [&str; 4] = ["fidelity", "readability", "coherence", "reality"];

/// 4 観点が読む 5 文書（pass-coherence.yaml の record.read と同じ・天井の正本の reads の行の形）。
const SAME_READS: &str = concat!(
    "      - {doc: constitution, fields: [articles, precedence]}\n",
    "      - {doc: rules, fields: [rows]}\n",
    "      - {doc: srs, fields: [requirements, nonfunctional, acceptance, constraints]}\n",
    "      - {doc: adr, fields: [decision, basis, consequences, retreat]}\n",
    "      - {doc: design-note, fields: [sections]}\n",
);

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn bundle_fixture() -> PathBuf {
    repo_root().join("tests/fixtures/ceiling/bundle")
}

fn findings_fixture(name: &str) -> String {
    fs::read_to_string(
        repo_root()
            .join("tests/fixtures/ceiling/findings")
            .join(name),
    )
    .unwrap()
}

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-stamp-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    fs::create_dir_all(&td).unwrap();
    td
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

/// `folio ceiling --dir <dir> [--faces <faces>] --out <out> <flags…>`。
fn folio_ceiling(dir: &Path, faces: Option<&Path>, out: &Path, flags: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_folio"));
    cmd.arg("ceiling").arg("--dir").arg(dir);
    if let Some(f) = faces {
        cmd.arg("--faces").arg(f);
    }
    cmd.arg("--out")
        .arg(out)
        .args(flags)
        .output()
        .expect("folio を起動できない")
}

fn code(out: &Output, what: &str) -> i32 {
    out.status.code().unwrap_or_else(|| {
        panic!(
            "{what} が signal で終わった: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// 天井の正本の viewpoints の各 reads を、観点の id から引いた行の塊に差し替える（一時 dir の写しの上だけ）。
fn put_reads(ceiling: &str, reads: &dyn Fn(&str) -> String) -> String {
    let mut out = String::new();
    let mut id = String::new();
    let mut in_reads = false;
    for line in ceiling.split_inclusive('\n') {
        if in_reads && line.starts_with("      - {doc: ") {
            continue;
        }
        in_reads = false;
        if let Some(rest) = line.strip_prefix("  - id: ") {
            id = rest.trim_end().to_string();
        }
        out.push_str(line);
        if line == "    reads:\n" {
            out.push_str(&reads(&id));
            in_reads = true;
        }
    }
    out
}

/// 便 104 の土台 split-reads.yaml の観点ごとの行の塊（観点の id → 字下げ 6 の reads の行）。
fn split_reads() -> BTreeMap<String, String> {
    let text = fs::read_to_string(repo_root().join("tests/fixtures/ceiling/split-reads.yaml")).unwrap();
    let mut blocks: BTreeMap<String, String> = BTreeMap::new();
    let mut id = String::new();
    for line in text.split_inclusive('\n') {
        if line.starts_with('#') {
            continue;
        }
        if let Some(key) = line.strip_suffix(":\n") {
            id = key.to_string();
            blocks.insert(id.clone(), String::new());
        } else {
            assert!(line.starts_with("      - {doc: "), "split-reads.yaml の行の形でない: {line}");
            blocks.get_mut(&id).expect("観点の id の前に行が在る").push_str(line);
        }
    }
    blocks
}

/// 所見 file の record の read と bundle を、この周の 5 文書と観点の digest.txt に合わせる。
fn fit_record(text: &str, digest: &str) -> String {
    text.split_inclusive('\n')
        .map(|line| {
            if line.starts_with("  read: ") {
                "  read: [constitution, rules, srs, adr, design-note]\n".to_string()
            } else if line.starts_with("  bundle: ") {
                format!("  bundle: {digest}")
            } else {
                line.to_string()
            }
        })
        .collect()
}

/// 組んだ周。src = 正本の写し（印は src/preview/ceiling-stamp.yaml）・out = 束の置き場（名 stamp-case）。
struct Round {
    td: PathBuf,
    src: PathBuf,
    out: PathBuf,
}

impl Round {
    /// 凍結 fixture の束を写し、reads を揃えて `--write` で組み、pass-coherence.yaml を 4 観点に写す。
    fn passing(case: &str) -> Round {
        Round::build(case, &|_| SAME_READS.to_string())
    }

    /// 便 104: 4 観点が同じ 5 文書の違う欄を読む周（split-reads.yaml）。
    fn split(case: &str) -> Round {
        let blocks = split_reads();
        Round::build(case, &|id| blocks.get(id).cloned().unwrap_or_default())
    }

    /// 凍結 fixture の束を写し、観点ごとの reads を差し替えて `--write` で組み、pass-coherence.yaml を 4 観点に写す。
    fn build(case: &str, reads: &dyn Fn(&str) -> String) -> Round {
        let td = temp_dir(case);
        let src = td.join("src");
        let faces = td.join("faces");
        copy_tree(&bundle_fixture().join("source"), &src);
        copy_tree(&bundle_fixture().join("faces"), &faces);
        let ceiling = src.join("ceiling.yaml");
        let replaced = put_reads(&fs::read_to_string(&ceiling).unwrap(), reads);
        let mut lines = 0;
        for id in VIEWPOINTS {
            let block = reads(id);
            assert!(!block.is_empty() && replaced.contains(&block), "{id}: reads を差し込めない");
            lines += block.lines().count();
        }
        assert_eq!(
            replaced.matches("      - {doc: ").count(),
            lines,
            "reads を 4 観点に差し込めない"
        );
        fs::write(&ceiling, replaced).unwrap();
        let out = td.join("stamp-case");
        let run = folio_ceiling(&src, Some(&faces), &out, &["--write"]);
        assert_eq!(code(&run, "folio ceiling --write"), 0, "{}", stderr(&run));
        let round = Round { td, src, out };
        for id in VIEWPOINTS {
            round.put(id, &findings_fixture("pass-coherence.yaml"));
        }
        round
    }

    /// <観点>/findings.yaml に書く（record は周に合わせる）。
    fn put(&self, id: &str, text: &str) {
        let digest = fs::read_to_string(self.out.join(id).join("digest.txt")).unwrap();
        fs::write(
            self.out.join(id).join("findings.yaml"),
            fit_record(text, &digest),
        )
        .unwrap();
    }

    fn stamp(&self) -> Output {
        folio_ceiling(&self.src, None, &self.out, &["--stamp"])
    }

    fn stamp_path(&self) -> PathBuf {
        self.src.join("preview/ceiling-stamp.yaml")
    }

    fn done(self) {
        let _ = fs::remove_dir_all(&self.td);
    }
}

/// 印から要約値 4 種（sources / faces / rest / 各行の bundle）と at を落とす（anchor の形・§1 (c)）。rest は写しの
/// reads を書き替えると母集団が動くので凍結しない（便 99・delivery-99.md §1 (e)）。nodes の表は落とさない。
fn without_digests(stamp: &str) -> String {
    stamp
        .split_inclusive('\n')
        .filter(|line| {
            !line.starts_with("at: ")
                && !line.starts_with("sources: ")
                && !line.starts_with("faces: ")
                && !line.starts_with("rest: ")
        })
        .map(|line| {
            let mut line = line.to_string();
            for key in [", bundle: ", ", at: "] {
                if let Some(start) = line.find(key) {
                    let rest = &line[start + key.len()..];
                    let end = rest
                        .find([',', '}'])
                        .map_or(line.len(), |i| start + key.len() + i);
                    line.replace_range(start..end, "");
                }
            }
            line
        })
        .collect()
}

/// 印の鍵 `key` の値（1 行）。
fn value<'a>(stamp: &'a str, key: &str) -> &'a str {
    let head = format!("{key}: ");
    stamp
        .lines()
        .find_map(|l| l.strip_prefix(head.as_str()))
        .unwrap_or_else(|| panic!("印に {key} が無い: {stamp}"))
}

/// 要約値（sha256）は命令 `sha256sum` を子の処理で撃って測る（tests/schema.rs と同じ形）。
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

/// dir の下の全 file（dir からの相対 path）と中身。
fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                walk(root, &path, out);
            } else {
                let rel = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                out.insert(rel, fs::read(&path).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

/// `<out>/*/<sub>/` の和集合を相対 path の byte 順に連結して sha256sum で測る。
fn union_hex(out: &Path, sub: &str) -> Result<String, String> {
    let mut union: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for entry in fs::read_dir(out).unwrap() {
        let dir = entry.unwrap().path().join(sub);
        if !dir.is_dir() {
            continue;
        }
        for (rel, body) in tree(&dir) {
            if let Some(have) = union.get(&rel) {
                assert_eq!(*have, body, "{sub}/{rel}: 観点で中身が違う");
            }
            union.insert(rel, body);
        }
    }
    let bytes: Vec<u8> = union.into_values().flatten().collect();
    sha256_hex(&bytes)
}

/// 流れの形の行 `{key: 値, …}` から欄 `key` の値を取る。
fn flow_field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let head = format!("{key}: ");
    let at = line
        .find(&format!("{{{head}"))
        .map(|i| i + 1)
        .or_else(|| line.find(&format!(", {head}")).map(|i| i + 2))?;
    let rest = &line[at + head.len()..];
    Some(rest[..rest.find([',', '}'])?].trim())
}

/// folio を呼ばずに測った正本の要約値（便 104・§1 (e) の 2）: 天井の正本の documents と viewpoints の reads から文書の
/// file（dir 形は直下の .yaml）を集め、`<dir>` からの相対 path の byte 順に連結して外の命令 sha256sum で測る。
/// 集められないときは歯を落とす（P-4.1）。Err は sha256sum の側の失敗だけ。
fn canonical_hex(dir: &Path) -> Result<String, String> {
    let ceiling = fs::read_to_string(dir.join("ceiling.yaml")).expect("天井の正本が読めない");
    let mut documents: BTreeMap<String, String> = BTreeMap::new();
    let mut reads: Vec<String> = Vec::new();
    let mut in_documents = false;
    for line in ceiling.lines() {
        if !line.starts_with(' ') {
            in_documents = line == "documents:";
            continue;
        }
        if in_documents && let (Some(id), Some(file)) = (flow_field(line, "id"), flow_field(line, "file")) {
            documents.insert(id.to_string(), file.to_string());
        }
        if line.starts_with("      - {doc: ") {
            let doc = flow_field(line, "doc").expect("reads の行に doc が無い").to_string();
            if !reads.contains(&doc) {
                reads.push(doc);
            }
        }
    }
    assert!(!documents.is_empty(), "天井の正本の documents が読めない");
    assert!(!reads.is_empty(), "天井の正本の reads が読めない");
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for doc in &reads {
        let file = documents.get(doc).unwrap_or_else(|| panic!("{doc}: 文書の一覧に無い"));
        if let Some(sub) = file.strip_suffix('/') {
            for entry in fs::read_dir(dir.join(sub)).unwrap() {
                let entry = entry.unwrap();
                let name = entry.file_name().into_string().unwrap();
                if entry.file_type().unwrap().is_file() && name.ends_with(".yaml") {
                    files.insert(format!("{file}{name}"), fs::read(entry.path()).unwrap());
                }
            }
        } else {
            files.insert(file.clone(), fs::read(dir.join(file)).unwrap());
        }
    }
    assert!(files.len() >= reads.len(), "文書の file が集められない: {files:?}", files = files.keys());
    let bytes: Vec<u8> = files.into_values().flatten().collect();
    assert!(!bytes.is_empty(), "正本の byte 列が空");
    sha256_hex(&bytes)
}

/// sha256sum を起動できないときだけ まだ分からない の 1 行で済ませる。それ以外の Err は歯を落とす。
fn canonical_or_unknown(dir: &Path) -> Option<String> {
    match canonical_hex(dir) {
        Ok(hex) => Some(hex),
        Err(why) if why.starts_with("sha256sum を起動できない") => {
            eprintln!("# まだ分からない: 正本の要約値を測れない: {why}");
            None
        }
        Err(why) => panic!("正本の要約値を測れない: {why}"),
    }
}

/// `folio ceiling --dir src --gate --write-set src/srs.yaml`（今の dir = 周の一時 dir）。
fn folio_gate(round: &Round) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .current_dir(&round.td)
        .args(["ceiling", "--dir", "src", "--gate", "--write-set", "src/srs.yaml"])
        .output()
        .expect("folio を起動できない")
}

// ── 1. 凍結の形 ──

#[test]
fn stamp_writes_the_frozen_shape() {
    let round = Round::passing("frozen");
    let run = round.stamp();
    let stamp = fs::read_to_string(round.stamp_path());
    round.done();
    assert_eq!(code(&run, "--stamp"), 0, "{}{}", stdout(&run), stderr(&run));
    assert!(stdout(&run).contains("印を書いた"), "{}", stdout(&run));
    let stamp = stamp.expect("印が無い");
    let anchor = findings_fixture("stamp-expected.yaml");
    assert_eq!(without_digests(&stamp), anchor, "印:\n{stamp}");
    assert_eq!(value(&stamp, "at"), "2026-09-19T05:00:00Z");
}

// ── 2. 要約値の再計算 ──

#[test]
fn stamp_sources_digest_is_recomputable() {
    let round = Round::passing("digest");
    let run = round.stamp();
    assert_eq!(code(&run, "--stamp"), 0, "{}", stdout(&run));
    let stamp = fs::read_to_string(round.stamp_path()).unwrap();
    // sources は正本から測り直す（便 104）・faces は束の faces/ の和集合を測り直す
    let sources = canonical_or_unknown(&round.src);
    let faces = union_hex(&round.out, "faces")
        .map_err(|why| eprintln!("# まだ分からない: 要約値を測れない: {why}"))
        .ok();
    round.done();
    for (key, hex) in [("sources", sources), ("faces", faces)] {
        let got = value(&stamp, key);
        let got = got.strip_prefix("sha256 ").expect("sha256 の形でない");
        assert_eq!(got.len(), 64, "{key}: {got}");
        if let Some(h) = hex {
            assert_eq!(got, h, "{key}");
        }
    }
}

// ── 3. 冪等 ──

#[test]
fn stamp_is_idempotent() {
    let round = Round::passing("idem");
    let first = round.stamp();
    assert_eq!(code(&first, "--stamp 1"), 0, "{}", stdout(&first));
    let before = fs::read(round.stamp_path()).unwrap();
    let second = round.stamp();
    let after = fs::read(round.stamp_path()).unwrap();
    round.done();
    assert_eq!(code(&second, "--stamp 2"), 0, "{}", stdout(&second));
    assert!(stdout(&second).contains("印は同じ"), "{}", stdout(&second));
    assert_eq!(before, after);
}

// ── 4. 欠けた周は書かない ──

#[test]
fn stamp_refuses_an_incomplete_round() {
    // 印が無いところで欠けた周 → 書かない
    let round = Round::passing("incomplete");
    fs::remove_file(round.out.join("readability/findings.yaml")).unwrap();
    let run = round.stamp();
    let written = round.stamp_path().exists();
    round.done();
    assert_eq!(code(&run, "--stamp"), 2, "{}", stdout(&run));
    let out = stdout(&run);
    assert!(
        out.contains("まだ分からない") && out.contains("readability"),
        "{out}"
    );
    assert!(!written, "印が書かれた");

    // 在った印は変わらない
    let round = Round::passing("incomplete-kept");
    let first = round.stamp();
    assert_eq!(code(&first, "--stamp 1"), 0, "{}", stdout(&first));
    let before = fs::read(round.stamp_path()).unwrap();
    fs::remove_file(round.out.join("reality/findings.yaml")).unwrap();
    let run = round.stamp();
    let after = fs::read(round.stamp_path()).unwrap();
    round.done();
    assert_eq!(code(&run, "--stamp 2"), 2, "{}", stdout(&run));
    assert!(stdout(&run).contains("reality"), "{}", stdout(&run));
    assert_eq!(before, after, "在った印が変わった");
}

// ── 5. 反証の結果を運ぶ ──

#[test]
fn stamp_carries_the_refute_results() {
    let round = Round::passing("refute");
    round.put("fidelity", &findings_fixture("stop-unrefuted.yaml"));
    let refute = folio_ceiling(&round.src, None, &round.out, &["--refute"]);
    assert_eq!(code(&refute, "--refute"), 0, "{}", stderr(&refute));
    let f1 = round.out.join("fidelity/refute/F-1");
    let digest = fs::read_to_string(f1.join("digest.txt")).unwrap();
    fs::write(
        f1.join("result.yaml"),
        format!(
            "id: F-1\nrefute: 支持\nmodel: opus\neffort: default\nat: 2026-09-19T08:00:00Z\nbundle: {}\n",
            digest.trim_end_matches('\n')
        ),
    )
    .unwrap();
    let run = round.stamp();
    let stamp = fs::read_to_string(round.stamp_path());
    round.done();
    assert_eq!(code(&run, "--stamp"), 0, "{}", stdout(&run));
    let stamp = stamp.expect("印が無い");
    assert_eq!(value(&stamp, "verdict"), "不合格", "{stamp}");
    assert!(
        stamp.contains("refutes:\n  - {viewpoint: fidelity, finding: F-1, refute: 支持}\nreads: "),
        "{stamp}"
    );
    assert!(
        stamp.contains("  - {id: fidelity, verdict: 不合格, findings: 1, stops: 1, "),
        "{stamp}"
    );
}

// ── 便 99: 印の rest と nodes（docs/design/delivery-99.md §1 (h) の 7〜9） ──

/// 印の最上位の欄の名（字下げの無い `key:` の行・注釈を除く）の並び。
fn top_keys(stamp: &str) -> Vec<&str> {
    stamp
        .lines()
        .filter(|l| !l.starts_with([' ', '#']))
        .filter_map(|l| l.split_once(':').map(|(k, _)| k))
        .collect()
}

/// 印の nodes の行（id・要約値）。
fn node_rows(stamp: &str) -> Vec<(String, String)> {
    let at = stamp.find("\nnodes:\n").expect("印に nodes が無い") + "\nnodes:\n".len();
    stamp[at..]
        .lines()
        .map(|l| {
            let inner = l
                .strip_prefix("  - {id: ")
                .and_then(|r| r.strip_suffix('}'))
                .unwrap_or_else(|| panic!("nodes の行の形でない: {l}"));
            let (id, digest) = inner.split_once(", digest: ").expect("digest の対が無い");
            (id.to_string(), digest.to_string())
        })
        .collect()
}

#[test]
fn f99_the_stamp_carries_the_node_table() {
    let round = Round::passing("f99-table");
    let run = round.stamp();
    let stamp = fs::read_to_string(round.stamp_path());
    round.done();
    assert_eq!(code(&run, "--stamp"), 0, "{}{}", stdout(&run), stderr(&run));
    let stamp = stamp.expect("印が無い");
    assert_eq!(
        top_keys(&stamp),
        ["round", "at", "verdict", "sources", "faces", "viewpoints", "refutes", "reads", "rest", "nodes"],
        "印の欄の並び"
    );
    let anchor = findings_fixture("stamp-expected.yaml");
    assert_eq!((anchor.lines().count(), anchor.len()), (34, 1_342), "凍結 anchor の行数と byte 数");
    assert_eq!(without_digests(&stamp), anchor, "印:\n{stamp}");
}

#[test]
fn f99_the_stamp_rest_is_recomputable() {
    let round = Round::passing("f99-rest");
    let run = round.stamp();
    assert_eq!(code(&run, "--stamp"), 0, "{}", stdout(&run));
    let stamp = fs::read_to_string(round.stamp_path()).unwrap();
    let script = repo_root().join("tests/fixtures/schema/node-digest.py");
    let independent = Command::new("python3").arg(script).arg(&round.src).output();
    round.done();
    let rest = value(&stamp, "rest").strip_prefix("sha256 ").expect("rest が sha256 の形でない");
    assert!(
        rest.len() == 64 && rest.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')),
        "rest: {rest}"
    );
    match independent {
        Ok(out) => {
            assert_eq!(code(&out, "node-digest.py"), 0, "{}", stderr(&out));
            let text = stdout(&out);
            let want = text
                .lines()
                .find_map(|l| l.strip_prefix("# 残差 sha256 "))
                .expect("独立の実装の出力に残差の行が無い");
            assert_eq!(rest, want, "印の rest が独立の実装の残差の要約値と違う");
        }
        Err(e) => eprintln!("# まだ分からない: node-digest.py: python3 を起動できない: {e}"),
    }
}

#[test]
fn f99_the_stamp_node_rows_are_the_index() {
    let round = Round::passing("f99-index");
    let run = round.stamp();
    assert_eq!(code(&run, "--stamp"), 0, "{}", stdout(&run));
    let stamp = fs::read_to_string(round.stamp_path()).unwrap();
    let print = Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(["graph", "--print", "--dir"])
        .arg(&round.src)
        .output()
        .expect("folio を起動できない");
    round.done();
    assert_eq!(code(&print, "folio graph --print"), 0, "{}", stderr(&print));
    let index: Vec<(String, String)> = stdout(&print)
        .lines()
        .take_while(|l| !l.starts_with("# 辺"))
        .filter(|l| !l.starts_with('#'))
        .map(|l| {
            let cols: Vec<&str> = l.split('\t').collect();
            assert_eq!(cols.len(), 5, "節点の行の欄の数: {l}");
            (cols[0].to_string(), cols[3].to_string())
        })
        .collect();
    let rows = node_rows(&stamp);
    assert_eq!(rows.len(), 23, "nodes の行の数");
    assert_eq!(rows, index, "印の nodes が索引の節点の行と同じ数・同じ順で一致しない");
}

// ── 便 104: 観点ごとに読む欄が違う周（docs/design/delivery-104.md §1 (e)） ──

#[test]
fn f104_the_stamp_survives_viewpoints_reading_different_fields() {
    let blocks = split_reads();
    assert_eq!(
        blocks.keys().map(String::as_str).collect::<Vec<_>>(),
        ["coherence", "fidelity", "readability", "reality"],
        "土台の観点の id"
    );
    let distinct: std::collections::BTreeSet<&String> = blocks.values().collect();
    assert_eq!(distinct.len(), 4, "土台の 4 つの塊が互いに違わない");
    let round = Round::split("f104-split");
    let fidelity = fs::read(round.out.join("fidelity/sources/srs.yaml")).unwrap();
    let readability = fs::read(round.out.join("readability/sources/srs.yaml")).unwrap();
    let run = round.stamp();
    let stamp = fs::read_to_string(round.stamp_path());
    round.done();
    assert_ne!(fidelity, readability, "srs.yaml の写しが観点で同じ byte（読む欄の違いが周に無い）");
    assert_eq!(code(&run, "--stamp"), 0, "{}{}", stdout(&run), stderr(&run));
    assert!(stdout(&run).contains("印を書いた"), "{}", stdout(&run));
    let stamp = stamp.expect("印が無い");
    let anchor = findings_fixture("stamp-expected.yaml");
    assert_eq!(without_digests(&stamp), anchor, "印:\n{stamp}");
}

#[test]
fn f104_the_stamp_sources_is_the_canonical_digest() {
    let split = Round::split("f104-canon-split");
    let run = split.stamp();
    assert_eq!(code(&run, "--stamp"), 0, "{}{}", stdout(&run), stderr(&run));
    let stamp = fs::read_to_string(split.stamp_path()).unwrap();
    let canonical = canonical_or_unknown(&split.src);
    split.done();
    let same = Round::passing("f104-canon-same");
    let run = same.stamp();
    assert_eq!(code(&run, "--stamp"), 0, "{}{}", stdout(&run), stderr(&run));
    let aligned = fs::read_to_string(same.stamp_path()).unwrap();
    same.done();
    let got = value(&stamp, "sources");
    let hex = got.strip_prefix("sha256 ").expect("sources が sha256 の形でない");
    assert!(
        hex.len() == 64 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')),
        "sources: {got}"
    );
    if let Some(want) = canonical {
        assert_eq!(hex, want, "印の sources が正本の要約値と違う");
    }
    assert_eq!(got, value(&aligned, "sources"), "読む欄を揃えた周と sources が違う");
}

#[test]
fn f104_the_gate_passes_the_stamp_it_just_wrote() {
    let round = Round::split("f104-gate");
    let run = round.stamp();
    let gate = folio_gate(&round);
    round.done();
    assert_eq!(code(&run, "--stamp"), 0, "{}{}", stdout(&run), stderr(&run));
    assert_eq!(code(&gate, "--gate"), 0, "{}{}", stdout(&gate), stderr(&gate));
    assert!(stdout(&gate).contains("正本の要約値が同じ"), "{}", stdout(&gate));
}
