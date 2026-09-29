//! `folio ceiling --stamp`（便 72・docs/design/delivery-72.md §1 (c)(d)）の歯。binary 経由。
//! 周（stamp-case）は「4 観点が同じ文書を読む周」として組む: 便 38 の凍結 fixture の束（tests/fixtures/ceiling/bundle/）を
//! 一時 dir へ写し、写しの source/ceiling.yaml の 4 観点の reads を pass-coherence.yaml の record.read と同じ 5 文書
//! （constitution・rules・srs・adr・design-note）に揃えてから `--write` で組み、pass-coherence.yaml を 4 観点の findings.yaml に
//! 写す（record.bundle は観点ごとの digest.txt に合わせる）。凍結 anchor stamp-expected.yaml は触らない（P-10.1）。
//! 便 169（docs/design/delivery-169.md §1 (c) の 5〜7）: 印の refutes の行の場所（at・file）と観点の行の wait を門が読む。
//! 門の古さを縛った歯（f126_ の門の 1 本）は外した。
//! 便 176（docs/design/delivery-176.md §1 (c)）: 場所の頭が file 名でなく設計ノートの欄の決まりの meta.id のとき（tsuzuri の形）。
//! 便 175（docs/design/delivery-175.md §1 (c) の 3）: 印は周の引き金の要約値 trigger を書かない（便 126 の 1 本を f175_ の 1 本に置き換えた）。
//! 便 177（docs/design/delivery-177.md §1 (c) の 2）: 印は節点の表 rest・nodes を書かない（凍結 anchor stamp-expected.yaml から nodes の表を
//! 落とした・便 99 の 3 本を f177_ の 1 本に置き換えた）。
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

/// 印から要約値 3 種（sources / faces / 各行の bundle）と at を落とす（anchor の形・§1 (c)）。
fn without_digests(stamp: &str) -> String {
    stamp
        .split_inclusive('\n')
        .filter(|line| {
            !line.starts_with("at: ")
                && !line.starts_with("sources: ")
                && !line.starts_with("faces: ")
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

/// `folio ceiling --dir src --gate --write-set <paths…>`（今の dir = 周の一時 dir）。
fn folio_gate(round: &Round, write_set: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .current_dir(&round.td)
        .args(["ceiling", "--dir", "src", "--gate", "--write-set"])
        .args(write_set)
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
        stamp.contains(concat!(
            "refutes:\n",
            "  - {viewpoint: fidelity, finding: F-1, refute: 支持, at: requirements.FR2.plain, file: srs.yaml}\n",
            "reads: "
        )),
        "{stamp}"
    );
    assert!(
        stamp.contains("  - {id: fidelity, verdict: 不合格, findings: 1, stops: 1, "),
        "{stamp}"
    );
}

// ── 印の欄の並び（便 99 の道具・便 177 で便 99 の歯を f177_ に置き換えた） ──

/// 印の最上位の欄の名（字下げの無い `key:` の行・注釈を除く）の並び。
fn top_keys(stamp: &str) -> Vec<&str> {
    stamp
        .lines()
        .filter(|l| !l.starts_with([' ', '#']))
        .filter_map(|l| l.split_once(':').map(|(k, _)| k))
        .collect()
}

// ── 便 175: 印は周の引き金の要約値を書かない（docs/design/delivery-175.md §1 (c) の 3） ──

/// 印の最上位の欄に trigger が無く、sources の直後が faces（ADR-30 決定 (5)(6)・門も名札も読まない）。
#[test]
fn f175_the_stamp_has_no_trigger() {
    let round = Round::passing("f175-keys");
    let run = round.stamp();
    let stamp = fs::read_to_string(round.stamp_path());
    round.done();
    assert_eq!(code(&run, "--stamp"), 0, "{}{}", stdout(&run), stderr(&run));
    let stamp = stamp.expect("印が無い");
    let keys = top_keys(&stamp);
    assert!(!keys.contains(&"trigger"), "印に trigger が在る: {keys:?}");
    let at = keys.iter().position(|k| *k == "sources").expect("印に sources が無い");
    assert_eq!(keys.get(at + 1), Some(&"faces"), "sources の直後が faces でない: {keys:?}");
}

// ── 便 177: 印は節点の表を書かない（docs/design/delivery-177.md §1 (c) の 2） ──

/// 印の最上位の欄はこの 8 つでこの順（節点の表 rest・nodes を書かない・ADR-30 決定 (5)(6)）。要約値を落とした印は凍結 anchor と
/// byte で同じ。
#[test]
fn f177_the_stamp_has_no_node_table() {
    let round = Round::passing("f177-keys");
    let run = round.stamp();
    let stamp = fs::read_to_string(round.stamp_path());
    round.done();
    assert_eq!(code(&run, "--stamp"), 0, "{}{}", stdout(&run), stderr(&run));
    let stamp = stamp.expect("印が無い");
    assert_eq!(
        top_keys(&stamp),
        ["round", "at", "verdict", "sources", "faces", "viewpoints", "refutes", "reads"],
        "印の欄の並び"
    );
    let anchor = findings_fixture("stamp-expected.yaml");
    assert_eq!((anchor.lines().count(), anchor.len()), (10, 560), "凍結 anchor の行数と byte 数");
    assert_eq!(without_digests(&stamp), anchor, "印:\n{stamp}");
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

// ── 写しの字の置き換え（便 126 で入った道具・便 175 で便 126 の歯を外した） ──

/// 写しの file の字 `from` を 1 か所だけ `to` に置き換える（1 か所でなければ歯を落とす）。
fn edit_once(path: &Path, from: &str, to: &str) {
    let text = fs::read_to_string(path).unwrap();
    assert_eq!(text.matches(from).count(), 1, "{}: 「{from}」が 1 か所でない", path.display());
    fs::write(path, text.replacen(from, to, 1)).unwrap();
}

// ── 便 154: 名の無い置き場の印（docs/design/delivery-154.md §1 (c) の 2 の 5） ──

#[test]
fn f154_an_unnamed_place_stamps_without_a_name() {
    let round = Round::passing("f154-unnamed");
    edit_once(
        &round.src.join("constitution.yaml"),
        "  id: fixture-constitution\n",
        "  id: 未記入\n",
    );
    let run = round.stamp();
    let stamp = fs::read_to_string(round.stamp_path());
    round.done();
    assert_eq!(code(&run, "--stamp"), 0, "{}{}", stdout(&run), stderr(&run));
    let stamp = stamp.expect("印が無い");
    assert!(stamp.starts_with("# 天井の印 — 生成物（"), "印:\n{stamp}");
    assert!(!stamp.contains("folio2"), "印:\n{stamp}");
}

#[test]
fn f104_the_gate_passes_the_stamp_it_just_wrote() {
    let round = Round::split("f104-gate");
    let run = round.stamp();
    let gate = folio_gate(&round, &["src/srs.yaml"]);
    round.done();
    assert_eq!(code(&run, "--stamp"), 0, "{}{}", stdout(&run), stderr(&run));
    assert_eq!(code(&gate, "--gate"), 0, "{}{}", stdout(&gate), stderr(&gate));
    assert!(stdout(&gate).contains("印の後の変更は審査していない"), "{}", stdout(&gate));
}

// ── 便 169: 印の refutes の行は 止める の場所を持ち、門がそれを読む（docs/design/delivery-169.md §1 (c) の 5〜7） ──

/// 所見 file の fixture `name` の字を `edits`（元の字・替える字）で 1 か所ずつ替える。
fn finding_with(name: &str, edits: &[(&str, &str)]) -> String {
    let mut text = findings_fixture(name);
    for (from, to) in edits {
        assert_eq!(text.matches(from).count(), 1, "{name}: 「{from}」が 1 か所でない");
        text = text.replacen(from, to, 1);
    }
    text
}

const FR2_PLACE: &str = "place: {doc: srs, at: requirements.FR2.plain}";

#[test]
fn f169_the_stamp_writes_the_stop_places_and_the_gate_reads_them() {
    let round = Round::passing("f169-places");
    round.put("fidelity", &findings_fixture("stop-upheld.yaml"));
    round.put(
        "coherence",
        &finding_with(
            "stop-unrefuted.yaml",
            &[("viewpoint: fidelity", "viewpoint: coherence"), (FR2_PLACE, "place: {doc: adr, at: ADR-1.decision}")],
        ),
    );
    round.put(
        "reality",
        &finding_with(
            "stop-refuted.yaml",
            &[
                ("verdict: 不合格", "verdict: 合格"),
                ("viewpoint: fidelity", "viewpoint: reality"),
                (FR2_PLACE, "place: {doc: design-note, at: sections.x}"),
            ],
        ),
    );
    let run = round.stamp();
    let stamp = fs::read_to_string(round.stamp_path());
    let gates: Vec<(&str, Output)> = ["src/srs.yaml", "src/adr/ADR-1.yaml", "src/design-note/full.yaml", "src/rules.yaml"]
        .into_iter()
        .map(|p| (p, folio_gate(&round, &[p])))
        .collect();
    round.done();
    assert_eq!(code(&run, "--stamp"), 0, "{}{}", stdout(&run), stderr(&run));
    let stamp = stamp.expect("印が無い");
    assert!(
        stamp.contains(concat!(
            "refutes:\n",
            "  - {viewpoint: fidelity, finding: F-1, refute: 支持, at: requirements.FR2.plain, file: srs.yaml}\n",
            "  - {viewpoint: coherence, finding: F-1, at: ADR-1.decision, file: adr/ADR-1.yaml}\n",
            "  - {viewpoint: reality, finding: F-1, refute: 退けた, at: sections.x, file: design-note/}\n",
            "reads: "
        )),
        "{stamp}"
    );
    let waiting: Vec<&str> = stamp.lines().filter(|l| l.contains("wait: ")).collect();
    assert_eq!(waiting.len(), 1, "{stamp}");
    assert!(
        waiting[0].starts_with("  - {id: coherence, verdict: まだ分からない, ") && waiting[0].ends_with(", wait: 反証}"),
        "{stamp}"
    );
    for ((path, run), want) in gates.iter().zip([1, 2, 0, 0]) {
        assert_eq!(code(run, "--gate"), want, "{path}: {}", stdout(run));
    }

    // 文書の一覧に無い doc の 止める の周は印を組まない（全部か無しか）
    let round = Round::passing("f169-nowhere");
    round.put(
        "fidelity",
        &finding_with("stop-upheld.yaml", &[(FR2_PLACE, "place: {doc: nowhere, at: x.y}")]),
    );
    let run = round.stamp();
    let written = round.stamp_path().exists();
    round.done();
    assert_eq!(code(&run, "--stamp"), 2, "{}", stdout(&run));
    assert!(stdout(&run).contains("文書の一覧に無い"), "{}", stdout(&run));
    assert!(!written, "印が書かれた");
}

#[test]
fn f169_a_failed_round_is_stamped_with_its_upheld_stop() {
    let round = Round::passing("f169-failed");
    let first = round.stamp();
    assert_eq!(code(&first, "--stamp 1"), 0, "{}", stdout(&first));
    round.put("fidelity", &findings_fixture("stop-upheld.yaml"));
    let run = round.stamp();
    let stamp = fs::read_to_string(round.stamp_path()).unwrap();
    let gate = folio_gate(&round, &["src/srs.yaml"]);
    round.done();
    assert_eq!(code(&run, "--stamp 2"), 0, "{}", stdout(&run));
    assert!(stdout(&run).contains("印を書いた"), "{}", stdout(&run));
    assert_eq!(value(&stamp, "verdict"), "不合格", "{stamp}");
    assert!(
        stamp.contains(concat!(
            "refutes:\n",
            "  - {viewpoint: fidelity, finding: F-1, refute: 支持, at: requirements.FR2.plain, file: srs.yaml}\n",
            "reads: "
        )),
        "{stamp}"
    );
    assert_eq!(code(&gate, "--gate"), 1, "{}", stdout(&gate));
}

#[test]
fn f169_a_viewpoint_unknown_for_another_reason_does_not_wait() {
    let round = Round::passing("f169-other");
    // AI が判定できない（所見 file の verdict が まだ分からない）と反証の済んでいない 止める
    round.put(
        "readability",
        &finding_with(
            "stop-unrefuted.yaml",
            &[("verdict: 合格", "verdict: まだ分からない"), ("viewpoint: fidelity", "viewpoint: readability")],
        ),
    );
    // 所見の欄の違反（根拠が正本に無い）と反証の済んでいない 止める
    round.put(
        "coherence",
        &finding_with(
            "stop-unrefuted.yaml",
            &[
                ("viewpoint: fidelity", "viewpoint: coherence"),
                ("evidence: 合格か不合格のどちらかを出します。", "evidence: 正本に無い字の根拠。"),
            ],
        ),
    );
    // 止める が全部退けられた不合格（再判定待ち）
    round.put(
        "reality",
        &finding_with("stop-refuted.yaml", &[("viewpoint: fidelity", "viewpoint: reality")]),
    );
    let run = round.stamp();
    let stamp = fs::read_to_string(round.stamp_path());
    let gate = folio_gate(&round, &["src/rules.yaml"]);
    round.done();
    assert_eq!(code(&run, "--stamp"), 0, "{}{}", stdout(&run), stderr(&run));
    let stamp = stamp.expect("印が無い");
    for id in ["readability", "coherence", "reality"] {
        let head = format!("  - {{id: {id}, verdict: まだ分からない, ");
        assert!(stamp.lines().any(|l| l.starts_with(&head)), "{id}: {stamp}");
    }
    assert!(!stamp.contains("wait:"), "{stamp}");
    assert_eq!(code(&gate, "--gate"), 2, "{}", stdout(&gate));
    assert!(
        stdout(&gate).contains(
            "印の観点の結果が欠けている: readability（まだ分からない）・coherence（まだ分からない）・reality（まだ分からない）"
        ),
        "{}",
        stdout(&gate)
    );
}

// ── 便 176: 止める の場所の頭が file 名でなく meta.id のとき（docs/design/delivery-176.md §1 (c) の 1・2） ──

/// 設計ノートの欄の決まり（tsuzuri の形・file 名は schema.yaml・meta.id は design-note-schema）の最小の字。
const NOTE_SCHEMA: &str = "meta: {id: design-note-schema, version: 1}\n";
/// 場所の頭がその meta.id の 止める（tsuzuri の周の 整合 F-4 の場所）。
const SCHEMA_PLACE: &str = "place: {doc: design-note, at: design-note-schema.schema.figures.retry_rules_row}";

/// 周の写しの design-note/ の下へ `files`（design-note/ からの相対 path・字）を足し、観点 fidelity を支持の 止める 1 件
/// （場所 SCHEMA_PLACE）にして印を書く。印の場所の file と、`write_set` の 1 本ずつに撃った門の終了コードを返す。
fn schema_stop(case: &str, files: &[(&str, &str)], write_set: &[&str]) -> (String, Vec<i32>) {
    let round = Round::passing(case);
    for (rel, text) in files {
        let path = round.src.join("design-note").join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
    }
    round.put("fidelity", &finding_with("stop-upheld.yaml", &[(FR2_PLACE, SCHEMA_PLACE)]));
    let run = round.stamp();
    let stamp = fs::read_to_string(round.stamp_path());
    let gates: Vec<i32> = write_set.iter().map(|p| code(&folio_gate(&round, &[p]), p)).collect();
    round.done();
    assert_eq!(code(&run, "--stamp"), 0, "{case}: {}{}", stdout(&run), stderr(&run));
    let stamp = stamp.expect("印が無い");
    let head = "  - {viewpoint: fidelity, finding: F-1, refute: 支持, at: design-note-schema.schema.figures.retry_rules_row, file: ";
    let rows: Vec<&str> = stamp.lines().filter_map(|l| l.strip_prefix(head)).collect();
    assert_eq!(rows.len(), 1, "{case}: {stamp}");
    (rows[0].trim_end_matches('}').to_string(), gates)
}

#[test]
fn f176_a_stop_on_the_schema_meta_id_is_its_file() {
    // 下の dir（retired/）の同じ meta.id は数えない
    let retired = "meta: {id: design-note-schema, version: 0}\n";
    let (file, gates) = schema_stop(
        "f176-schema",
        &[("schema.yaml", NOTE_SCHEMA), ("retired/schema-v0.yaml", retired)],
        &["src/design-note/full.yaml", "src/design-note/schema.yaml", "src/design-note/", "src/srs.yaml"],
    );
    assert_eq!(file, "design-note/schema.yaml");
    assert_eq!(gates, [0, 1, 1, 0], "full.yaml・schema.yaml・design-note/・srs.yaml");
}

#[test]
fn f176_the_meta_id_narrows_to_exactly_one_readable_file() {
    let gates = ["src/design-note/full.yaml", "src/design-note/schema.yaml"];
    // 対照: .yaml でない file は読まない
    let control = schema_stop("f176-control", &[("schema.yaml", NOTE_SCHEMA), ("notes.txt", "meta: [\n")], &gates);
    assert_eq!(control, ("design-note/schema.yaml".to_string(), vec![0, 1]), "対照");
    let twin = "meta: {id: design-note-schema, version: 2}\n";
    for (case, files) in [
        ("f176-none", vec![]),
        ("f176-twin", vec![("schema.yaml", NOTE_SCHEMA), ("twin.yaml", twin)]),
        ("f176-broken", vec![("schema.yaml", NOTE_SCHEMA), ("broken.yaml", "meta: [\n")]),
    ] {
        let (file, got) = schema_stop(case, &files, &gates);
        assert_eq!(file, "design-note/", "{case}");
        assert_eq!(got, [1, 1], "{case}");
    }
}
