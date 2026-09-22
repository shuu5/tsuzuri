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

/// 天井の正本の viewpoints の各 reads を SAME_READS に揃える（一時 dir の写しの上だけ）。
fn same_reads(ceiling: &str) -> String {
    let mut out = String::new();
    let mut in_reads = false;
    for line in ceiling.split_inclusive('\n') {
        if in_reads && line.starts_with("      - {doc: ") {
            continue;
        }
        in_reads = false;
        out.push_str(line);
        if line == "    reads:\n" {
            out.push_str(SAME_READS);
            in_reads = true;
        }
    }
    out
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
        let td = temp_dir(case);
        let src = td.join("src");
        let faces = td.join("faces");
        copy_tree(&bundle_fixture().join("source"), &src);
        copy_tree(&bundle_fixture().join("faces"), &faces);
        let ceiling = src.join("ceiling.yaml");
        let aligned = same_reads(&fs::read_to_string(&ceiling).unwrap());
        assert_eq!(
            aligned.matches(SAME_READS).count(),
            4,
            "reads を 4 観点に揃えられない"
        );
        fs::write(&ceiling, aligned).unwrap();
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
    let measured = [
        ("sources", union_hex(&round.out, "sources")),
        ("faces", union_hex(&round.out, "faces")),
    ];
    round.done();
    for (key, hex) in measured {
        let got = value(&stamp, key);
        let got = got.strip_prefix("sha256 ").expect("sha256 の形でない");
        assert_eq!(got.len(), 64, "{key}: {got}");
        match hex {
            Ok(h) => assert_eq!(got, h, "{key}"),
            Err(why) => eprintln!("# まだ分からない: 要約値を測れない: {why}"),
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
