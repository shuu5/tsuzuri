//! `folio ceiling --write`（便 38・docs/design/delivery-38.md §1 (e)(f)）の歯。binary 経由。
//! - 凍結 anchor（tests/fixtures/ceiling/bundle/・P-10.1）: 4 観点の file の一覧と要約値が planner の手組みの実測と一致し、
//!   sha256sum で測り直した要約値も同じ（folio の code に依らない・P-10.2）
//! - 写しの byte（正本と面は byte のまま・folio.css は写さない・観点ごとの面の有無・question / reads / finding の逐語）
//! - 決定性と全部か無しか（2 度組んで byte 一致・sources/ と faces/ は作り直し・所見 file と起動の記録は触らない）
//! - まだ分からない 6 種（正本が無い・読む正本が無い・面が無い・配信先が無い・親 dir が無い・行き先が一覧に無い）
//! - 読み手は一覧を床の定数から取る（便 47・delivery-47.md §1 (e)5）: fixture の天井の正本から旧 4 節を外し凍結 anchor
//!   tests/fixtures/schema/ceiling-region.txt の中身を足しても、凍結 anchor と同じ束が組める
//! - 実の正本（design-intent の写し・前段は folio build）
//! - 絞り（便 98・docs/design/delivery-98.md §1 (e)）: 凍結 anchor の 6 種別・reads.yaml の注釈の逐語・骨格と頭・行の byte と順・
//!   列 0 の `- `・生成区間
//!
//! 版管理の下の file は書き換えない（`--out` は必ず一時 dir の中）。

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// 凍結 fixture の置き場（source/ = 欠陥を 1 つ仕込んだ最小の正本・faces/ = 面の写しの見本 7 本）。
fn fixture() -> PathBuf {
    repo_root().join("tests/fixtures/ceiling/bundle")
}

fn design_intent() -> PathBuf {
    repo_root().join("design-intent")
}

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-bundle-{case}-{}", std::process::id()));
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

/// fixture の source/ と faces/ を一時 dir へ写す。戻り値 = (一時 dir, 正本の写し, 面の写し)。
fn fixture_copy(case: &str) -> (PathBuf, PathBuf, PathBuf) {
    let td = temp_dir(case);
    let src = td.join("src");
    let faces = td.join("faces");
    copy_tree(&fixture().join("source"), &src);
    copy_tree(&fixture().join("faces"), &faces);
    (td, src, faces)
}

/// `folio ceiling --dir <dir> --faces <faces> --out <out> --write`。
fn folio_ceiling(dir: &Path, faces: &Path, out: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("ceiling")
        .arg("--dir")
        .arg(dir)
        .arg("--faces")
        .arg(faces)
        .arg("--out")
        .arg(out)
        .arg("--write")
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

/// dir の下の全 file（dir からの相対 path・byte 順）と中身。
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
    if dir.is_dir() {
        walk(dir, dir, &mut out);
    }
    out
}

/// 要約値（sha256）は命令 `sha256sum` を子の処理で撃って測る（歯は crate の中を読めない・tests/figure.rs と同じ形）。
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

/// 観点の dir の束を (e) の測り方で測り直す: sources と faces の下の全 file と question.yaml・finding.yaml・reads.yaml を
/// LC_ALL=C sort の順（path の byte 順）に並べ、中身を連結して sha256sum に流す。戻り値 = (連結の byte 数, 要約値)。
fn remeasure(vp_dir: &Path) -> (usize, Result<String, String>) {
    let mut bytes = Vec::new();
    for (rel, body) in tree(vp_dir) {
        if rel != "digest.txt" {
            bytes.extend_from_slice(&body);
        }
    }
    (bytes.len(), sha256_hex(&bytes))
}

/// digest.txt の中身が (e) の値で、測り直した要約値も同じ。
fn assert_digest(vp_dir: &Path, hex: &str, concat_len: usize) {
    let id = vp_dir.file_name().unwrap().to_string_lossy().into_owned();
    let digest = fs::read_to_string(vp_dir.join("digest.txt")).unwrap();
    assert_eq!(
        digest,
        format!("sha256-files-1 {hex}\n"),
        "{id}: digest.txt"
    );
    let (len, measured) = remeasure(vp_dir);
    assert_eq!(len, concat_len, "{id}: 連結の byte 数");
    match measured {
        Ok(m) => assert_eq!(m, hex, "{id}: sha256sum で測り直した要約値"),
        // 測る道具が無いときは合格にしない代わりに理由を出す（panic で落とさない・FR5 の形）
        Err(why) => eprintln!("# まだ分からない: {id}: 要約値を測れない: {why}"),
    }
}

// ── (e) 凍結 anchor の期待 ──

const FIDELITY_FILES: [&str; 13] = [
    "faces/adr-1.html",
    "faces/adr-2.html",
    "faces/constitution.html",
    "faces/note-full.html",
    "faces/srs.html",
    "finding.yaml",
    "question.yaml",
    "reads.yaml",
    "sources/adr/ADR-1.yaml",
    "sources/adr/ADR-2.yaml",
    "sources/constitution.yaml",
    "sources/design-note/full.yaml",
    "sources/srs.yaml",
];

const REALITY_FILES: [&str; 11] = [
    "faces/adr-1.html",
    "faces/adr-2.html",
    "faces/note-full.html",
    "faces/srs.html",
    "finding.yaml",
    "question.yaml",
    "reads.yaml",
    "sources/adr/ADR-1.yaml",
    "sources/adr/ADR-2.yaml",
    "sources/design-note/full.yaml",
    "sources/srs.yaml",
];

/// 観点の id・file の一覧（digest.txt を除く）・連結の byte 数・要約値（便 98 の絞った束・delivery-98.md §1 (d)）。
fn expected() -> Vec<(&'static str, Vec<&'static str>, usize, &'static str)> {
    let fidelity = FIDELITY_FILES.to_vec();
    let mut readability = fidelity.clone();
    readability.extend(["faces/index.html", "sources/index.yaml"]);
    let mut coherence = fidelity.clone();
    coherence.push("sources/rules.yaml");
    vec![
        (
            "fidelity",
            fidelity,
            11_738,
            "d2e153b29a2b46b88295d10f782163636a5613649ceb9efa80dcebfb48473782",
        ),
        (
            "readability",
            readability,
            11_867,
            "877fcb5b03de9be5f6d8c4c1f61602e0f578a15e054cff9109eeb37834d08c68",
        ),
        (
            "coherence",
            coherence,
            11_489,
            "b0030a3fa1e1dc4d4e4710bbba07ec65f80bf1fa14bd2b4c75eea186b26f281d",
        ),
        (
            "reality",
            REALITY_FILES.to_vec(),
            7_560,
            "d8734c77995da9034593d27e757890ec9133c86444a466b05c85a1a9e1250d89",
        ),
    ]
}

/// 観点ごとの 落とした節の数・正本に無い節の数（節の名の総数・delivery-98.md §1 (d)）。
const EXPECTED_NOTED: [(usize, usize); 4] = [(24, 0), (30, 1), (24, 1), (20, 0)];

const FIDELITY_QUESTION: &str = "id: fidelity\nname: 忠実さ\nreader: |\n  元の文と平易文の両方を読み、意味の差だけを拾う編集者\nquestion: |\n  人が書いた自由文（やさしく言うと・平易文・平易な説明）は、元の文（規範文・要件の文・決定の文）の意味を保っているか。義務を足していないか、落としていないか。専門語の日本語の言い換えは元の語と同じものを指しているか。図の根拠が指す先は、図の中身と合っているか。判定できない箇所は「まだ分からない」と書く。\n";

/// delivery-98.md §1 (c) の 1 つ目の逐語（746 byte）。
const FIDELITY_READS: &str = "- {doc: constitution, fields: [articles.plain, articles.statements.text]}\n- {doc: srs, fields: [requirements.plain, requirements.shall, acceptance.plain, acceptance.title]}\n- {doc: adr, fields: [plain, decision, options.text, figures.refs]}\n- {doc: design-note, fields: [sections, figures.refs]}\n# 落とした節 adr/ADR-2.yaml: context, basis, retreat, amends, consequences\n# 落とした節 constitution.yaml: north_star, precedence, rules_pointer, amendment, glossary_pointer, sources\n# 落とした節 design-note/full.yaml: sources\n# 落とした節 srs.yaml: goals, scope, scope_m1, actors, outputs, rail, verdicts, nonfunctional, not_frozen, constraints, glossary_pointer, figures\n# 常に残す節: meta, id, title, status, date, schema\n";

/// delivery-98.md §1 (c) の 2 つ目の逐語（860 byte）。
const READABILITY_READS: &str = "- {doc: index, fields: [shelf, sections]}\n- {doc: constitution, fields: [articles.title, articles.plain]}\n- {doc: srs, fields: [goals, scope, requirements.title, requirements.plain]}\n- {doc: adr, fields: [title, plain]}\n- {doc: design-note, fields: [sections]}\n# 落とした節 adr/ADR-2.yaml: context, decision, options, basis, retreat, amends, consequences, figures\n# 落とした節 constitution.yaml: north_star, precedence, rules_pointer, amendment, glossary_pointer, sources\n# 落とした節 design-note/full.yaml: figures, sources\n# 落とした節 index.yaml: audience, lanes, intake\n# 落とした節 srs.yaml: scope_m1, actors, outputs, rail, verdicts, nonfunctional, acceptance, not_frozen, constraints, glossary_pointer, figures\n# 宣言に在るが正本に無い節 index.yaml: sections\n# 常に残す節: meta, id, title, status, date, schema\n";

const SKELETON: [&str; 6] = ["meta", "id", "title", "status", "date", "schema"];

const FINDING: &str = "# 所見の欄の決まり（天井の正本 ceiling.yaml の finding・weights・verdicts・record の写し・folio ceiling が組んだ）\nfinding:\n  required: [id, viewpoint, place, weight, evidence]\n  optional: [refute, note]\n  place: {required: [doc, at]}\n  refute: {values: [支持, 退けた, まだ分からない]}\nweights:\n  values: [止める, 直す, 参考]\n  refute: [止める]\nverdicts:\n  values: [合格, 不合格, まだ分からない]\nrecord:\n  required: [model, effort, at, read, bundle]\n";

// ── 1. 凍結 anchor ──

#[test]
fn bundle_write_matches_the_frozen_anchor() {
    let (td, src, faces) = fixture_copy("anchor");
    let out = td.join("bundle");
    let run = folio_ceiling(&src, &faces, &out);
    assert_eq!(code(&run, "folio ceiling --write"), 0, "{}", stderr(&run));
    assert!(stdout(&run).contains("観点 4"), "{}", stdout(&run));
    for (id, files, concat_len, hex) in expected() {
        let vp_dir = out.join(id);
        let mut want: Vec<String> = files.iter().map(|f| f.to_string()).collect();
        want.push("digest.txt".to_string());
        want.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        let got: Vec<String> = tree(&vp_dir).into_keys().collect();
        assert_eq!(got, want, "{id}: file の一覧");
        assert_digest(&vp_dir, hex, concat_len);
    }
    let _ = fs::remove_dir_all(&td);
}

// ── 1b. 独立の script と凍結 anchor file（便 97・docs/design/delivery-97.md §1 (c)(g)・P-10.1 / P-10.2 / P-10.3） ──

fn anchor_script() -> PathBuf {
    repo_root().join("tests/fixtures/ceiling/bundle-anchor.py")
}

fn anchor_file() -> PathBuf {
    repo_root().join("tests/fixtures/ceiling/bundle-anchor.txt")
}

/// anchor file の観点 1 つ分（便 98 で 6 種別）。
#[derive(Debug, PartialEq)]
struct AnchorRow {
    id: String,
    files: usize,
    bytes: usize,
    hex: String,
    dropped: usize,
    absent: usize,
}

/// anchor file を読む（file の順）。
fn anchor_rows() -> Vec<AnchorRow> {
    let text = fs::read_to_string(anchor_file()).unwrap();
    let mut rows: Vec<AnchorRow> = Vec::new();
    for line in text.lines().filter(|l| !l.starts_with('#')) {
        let (kind, val) = line
            .split_once('\t')
            .unwrap_or_else(|| panic!("anchor の行にタブが無い: {line:?}"));
        if kind == "観点" {
            rows.push(AnchorRow {
                id: val.to_string(),
                files: 0,
                bytes: 0,
                hex: String::new(),
                dropped: 0,
                absent: 0,
            });
            continue;
        }
        let row = rows.last_mut().unwrap();
        match kind {
            "file数" => row.files = val.parse().unwrap(),
            "byte" => row.bytes = val.parse().unwrap(),
            "要約値" => row.hex = val.to_string(),
            "落とした節の数" => row.dropped = val.parse().unwrap(),
            "正本に無い節の数" => row.absent = val.parse().unwrap(),
            _ => panic!("anchor の種別が違う: {line:?}"),
        }
    }
    rows
}

/// reads.yaml の注釈の行 `<head><file>: <名>, <名>…` の名の総数（行の数ではない）。
fn noted(reads: &str, head: &str) -> usize {
    reads
        .lines()
        .filter_map(|l| l.strip_prefix(head))
        .map(|rest| rest.split_once(": ").unwrap().1.split(", ").count())
        .sum()
}

const DROPPED_HEAD: &str = "# 落とした節 ";
const ABSENT_HEAD: &str = "# 宣言に在るが正本に無い節 ";

#[test]
fn f97_the_anchor_file_agrees_with_the_teeth() {
    let got = anchor_rows();
    let want: Vec<AnchorRow> = expected()
        .into_iter()
        .zip(EXPECTED_NOTED)
        .map(|((id, files, len, hex), (dropped, absent))| AnchorRow {
            id: id.to_string(),
            files: files.len(),
            bytes: len,
            hex: hex.to_string(),
            dropped,
            absent,
        })
        .collect();
    assert_eq!(got, want, "anchor file と凍結の 4 対");
}

/// 独立の script の出力が凍結 anchor と byte 一致し、凍結の土台から組んだ束が 6 種別とも anchor と一致する（便 98 §1 (d)）。
#[test]
fn f98_the_cut_bundle_matches_the_rebuilt_anchor() {
    match Command::new("python3").arg(anchor_script()).output() {
        Ok(run) => {
            assert_eq!(code(&run, "bundle-anchor.py"), 0, "{}", stderr(&run));
            assert_eq!(
                stdout(&run),
                fs::read_to_string(anchor_file()).unwrap(),
                "script の出力が凍結 anchor と違う"
            );
        }
        // 道具が無いときは合格にしない代わりに理由を出す（assert_digest と同じ形・P-10.3）
        Err(e) => eprintln!("# まだ分からない: bundle-anchor.py: python3 を起動できない: {e}"),
    }
    let (td, src, faces) = fixture_copy("f98-anchor");
    let out = td.join("bundle");
    let run = folio_ceiling(&src, &faces, &out);
    assert_eq!(code(&run, "folio ceiling --write"), 0, "{}", stderr(&run));
    let rows = anchor_rows();
    assert_eq!(rows.len(), 4, "anchor の観点の数");
    let (mut dropped, mut absent) = (0, 0);
    for row in rows {
        let vp_dir = out.join(&row.id);
        let files = tree(&vp_dir).into_keys().filter(|k| k != "digest.txt").count();
        assert_eq!(files, row.files, "{}: file の数", row.id);
        assert_digest(&vp_dir, &row.hex, row.bytes);
        let reads = fs::read_to_string(vp_dir.join("reads.yaml")).unwrap();
        assert_eq!(noted(&reads, DROPPED_HEAD), row.dropped, "{}: 落とした節の数", row.id);
        assert_eq!(noted(&reads, ABSENT_HEAD), row.absent, "{}: 正本に無い節の数", row.id);
        dropped += row.dropped;
        absent += row.absent;
    }
    assert!(
        stdout(&run).contains(&format!("落とした節 {dropped}・正本に無い節 {absent}")),
        "{}",
        stdout(&run)
    );
    let _ = fs::remove_dir_all(&td);
}

/// 凍結の土台の 忠実さ の reads.yaml が §1 (c) の 1 つ目の逐語と byte 一致する。
#[test]
fn f98_the_dropped_sections_are_named_in_reads() {
    let (td, src, faces) = fixture_copy("f98-dropped");
    let out = td.join("bundle");
    let run = folio_ceiling(&src, &faces, &out);
    assert_eq!(code(&run, "folio ceiling --write"), 0, "{}", stderr(&run));
    let reads = fs::read_to_string(out.join("fidelity/reads.yaml")).unwrap();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(reads.len(), 746);
    assert_eq!(reads, FIDELITY_READS);
}

/// 読みやすさ の reads.yaml が §1 (c) の 2 つ目の逐語と byte 一致し、整合 には rules.yaml の rows が同じ形で出る。
#[test]
fn f98_a_section_absent_from_every_source_is_named() {
    let (td, src, faces) = fixture_copy("f98-absent");
    let out = td.join("bundle");
    let run = folio_ceiling(&src, &faces, &out);
    assert_eq!(code(&run, "folio ceiling --write"), 0, "{}", stderr(&run));
    let readability = fs::read_to_string(out.join("readability/reads.yaml")).unwrap();
    let coherence = fs::read_to_string(out.join("coherence/reads.yaml")).unwrap();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(readability.len(), 860);
    assert_eq!(readability, READABILITY_READS);
    let lines: Vec<&str> = coherence.lines().collect();
    let at = lines
        .iter()
        .position(|l| *l == "# 宣言に在るが正本に無い節 rules.yaml: rows")
        .unwrap_or_else(|| panic!("整合 に rules.yaml の rows が無い: {coherence}"));
    assert_eq!(at + 2, lines.len(), "{coherence}");
    assert!(lines[at + 1].starts_with("# 常に残す節: "), "{coherence}");
    assert!(
        lines[..at].iter().all(|l| !l.starts_with("# 常に残す節")),
        "{coherence}"
    );
    assert!(
        lines[at + 1..].iter().all(|l| !l.starts_with(DROPPED_HEAD)),
        "{coherence}"
    );
}

// ── 2. 写しの byte ──

#[test]
fn bundle_copies_sources_and_faces_byte_for_byte() {
    let (td, src, faces) = fixture_copy("bytes");
    let out = td.join("bundle");
    let run = folio_ceiling(&src, &faces, &out);
    assert_eq!(code(&run, "folio ceiling --write"), 0, "{}", stderr(&run));
    let fidelity = out.join("fidelity");
    // 正本の写しは読む節まで絞る（便 98）＝ byte のままの行と順は歯 f98_every_kept_line_is_verbatim_and_in_order が見る
    let srs = fs::read(fidelity.join("sources/srs.yaml")).unwrap();
    assert!(
        String::from_utf8_lossy(&srs).contains("    plain: 合格か不合格のどちらかを出します。\n"),
        "欠陥の行が写しに無い"
    );
    assert_eq!(
        fs::read(fidelity.join("faces/srs.html")).unwrap(),
        fs::read(fixture().join("faces/srs.html")).unwrap()
    );
    assert!(
        !fidelity.join("faces/folio.css").exists(),
        "folio.css を写した"
    );
    assert!(out.join("readability/faces/index.html").is_file());
    assert!(!out.join("reality/faces/index.html").exists());
    assert_eq!(
        fs::read_to_string(fidelity.join("question.yaml")).unwrap(),
        FIDELITY_QUESTION
    );
    assert_eq!(
        fs::read_to_string(fidelity.join("reads.yaml")).unwrap(),
        FIDELITY_READS
    );
    for id in ["fidelity", "readability", "coherence", "reality"] {
        assert_eq!(
            fs::read_to_string(out.join(id).join("finding.yaml")).unwrap(),
            FINDING,
            "{id}: finding.yaml"
        );
    }
    let _ = fs::remove_dir_all(&td);
}

// ── 3. 決定性と全部か無しか ──

#[test]
fn bundle_is_deterministic_and_rebuilds_sources_and_faces_only() {
    let (td, src, faces) = fixture_copy("twice");
    let out = td.join("bundle");
    let first = folio_ceiling(&src, &faces, &out);
    assert_eq!(code(&first, "1 回目"), 0, "{}", stderr(&first));
    let a = tree(&out);
    let second = folio_ceiling(&src, &faces, &out);
    assert_eq!(code(&second, "2 回目"), 0, "{}", stderr(&second));
    let b = tree(&out);
    assert!(!a.is_empty());
    assert_eq!(a, b, "2 度組むと byte が違う（決定的でない）");

    // 席や器が書く所見 file と起動の記録は残し、古い写しは消える
    let fidelity = out.join("fidelity");
    let digest_before = fs::read_to_string(fidelity.join("digest.txt")).unwrap();
    fs::write(fidelity.join("findings.yaml"), "findings: []\n").unwrap();
    fs::write(fidelity.join("record.yaml"), "model: x\n").unwrap();
    fs::write(fidelity.join("sources/zzz.yaml"), "zzz: 1\n").unwrap();
    let third = folio_ceiling(&src, &faces, &out);
    assert_eq!(code(&third, "3 回目"), 0, "{}", stderr(&third));
    assert!(
        fidelity.join("findings.yaml").is_file(),
        "所見 file を消した"
    );
    assert!(fidelity.join("record.yaml").is_file(), "起動の記録を消した");
    assert!(
        !fidelity.join("sources/zzz.yaml").exists(),
        "古い写しが残った"
    );
    assert_eq!(
        fs::read_to_string(fidelity.join("digest.txt")).unwrap(),
        digest_before
    );
    let _ = fs::remove_dir_all(&td);
}

// ── 4. まだ分からない ──

/// 2 ∧ 標準エラーに「まだ分からない」と `words` を全部含む。
fn assert_unknown(run: &Output, case: &str, words: &[&str]) {
    assert_eq!(code(run, case), 2, "{case}: {}", stderr(run));
    let err = stderr(run);
    assert!(
        err.starts_with("folio ceiling: まだ分からない: "),
        "{case}: {err}"
    );
    for w in words {
        assert!(err.contains(w), "{case}: 「{w}」が無い: {err}");
    }
}

/// 写しの ceiling.yaml の字面の変異（1 か所だけ）。
fn mutate(path: &Path, from: &str, to: &str) {
    let before = fs::read_to_string(path).unwrap();
    assert_eq!(
        before.matches(from).count(),
        1,
        "変異の当て先が 1 か所でない: {from:?}"
    );
    fs::write(path, before.replacen(from, to, 1)).unwrap();
}

#[test]
fn bundle_unknown_when_the_ceiling_source_is_missing() {
    let (td, src, faces) = fixture_copy("no-ceiling");
    fs::remove_file(src.join("ceiling.yaml")).unwrap();
    let run = folio_ceiling(&src, &faces, &td.join("bundle"));
    let _ = fs::remove_dir_all(&td);
    assert_unknown(&run, "ceiling.yaml が無い", &["ceiling.yaml: 正本が無い"]);
}

#[test]
fn bundle_unknown_when_a_read_source_is_missing_and_writes_nothing() {
    let (td, src, faces) = fixture_copy("no-srs");
    fs::remove_file(src.join("srs.yaml")).unwrap();
    let out = td.join("bundle");
    let run = folio_ceiling(&src, &faces, &out);
    let written = tree(&out);
    let _ = fs::remove_dir_all(&td);
    assert_unknown(&run, "srs.yaml が無い", &["srs.yaml: 正本が無い"]);
    assert!(
        written.is_empty(),
        "組めないのに置き場に書いた: {:?}",
        written.keys()
    );
}

#[test]
fn bundle_unknown_when_a_face_is_missing() {
    let (td, src, faces) = fixture_copy("no-face");
    fs::remove_file(faces.join("srs.html")).unwrap();
    let run = folio_ceiling(&src, &faces, &td.join("bundle"));
    let _ = fs::remove_dir_all(&td);
    assert_unknown(&run, "srs.html が無い", &["srs: 面が無い"]);
}

#[test]
fn bundle_unknown_when_the_faces_dir_is_missing() {
    let (td, src, _faces) = fixture_copy("no-faces-dir");
    let run = folio_ceiling(&src, &td.join("nowhere"), &td.join("bundle"));
    let _ = fs::remove_dir_all(&td);
    assert_unknown(&run, "配信先が無い", &["配信先が無い"]);
}

#[test]
fn bundle_unknown_when_the_out_parent_is_missing() {
    let (td, src, faces) = fixture_copy("no-parent");
    let run = folio_ceiling(&src, &faces, &td.join("nowhere/bundle"));
    let _ = fs::remove_dir_all(&td);
    assert_unknown(&run, "親 dir が無い", &["親 dir が無い"]);
}

/// 最上位の節 `name` の行（列 0 の `name:`）から字下げの続く行までと、直前の注の行を外す（tests/ceiling.rs と同じ形）。
fn drop_section(lines: &mut Vec<String>, name: &str) {
    let Some(start) = lines.iter().position(|l| *l == format!("{name}:")) else {
        return;
    };
    let mut end = start + 1;
    while end < lines.len() && lines[end].starts_with(' ') {
        end += 1;
    }
    let mut from = start;
    while from > 0 && lines[from - 1].starts_with('#') {
        from -= 1;
    }
    lines.drain(from..end);
}

/// 第 3 版の形: 旧 4 節（verdicts・finding・record・bundle）を外し、末尾に凍結 anchor（生成区間の本文）を足す。
fn third_edition(text: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    for name in ["verdicts", "finding", "record", "bundle"] {
        drop_section(&mut lines, name);
    }
    let mut out: String = lines.iter().map(|l| format!("{l}\n")).collect();
    out.push('\n');
    out.push_str(
        &fs::read_to_string(repo_root().join("tests/fixtures/schema/ceiling-region.txt")).unwrap(),
    );
    out
}

/// 読み手が一覧を file でなく床の定数から取ること（便 47 §1 (d)(e)5）: fixture の天井の正本から旧 4 節を外し凍結 anchor の
/// 中身を足しても、凍結 anchor と同じ束（file の一覧・要約値 4 本）が組める。この fixture の 4 観点の reads は天井の正本
/// （doc の id は ceiling）を名指さないので、正本の file の byte を書き換えても束には写されない。
#[test]
fn bundle_reads_the_lists_from_the_floor_not_the_source_file() {
    let (td, src, faces) = fixture_copy("floor-lists");
    let ceiling = src.join("ceiling.yaml");
    let before = fs::read_to_string(&ceiling).unwrap();
    assert!(
        !before.contains("{doc: ceiling,"),
        "fixture の観点が天井の正本を読む＝要約値が正本の byte に依る"
    );
    let after = third_edition(&before);
    assert!(!after.contains("\nbundle:\n"), "{after}");
    assert!(!after.contains("\nfinding:\n"), "{after}");
    assert!(after.contains("\nschema:\n  top_level: ["), "{after}");
    fs::write(&ceiling, after).unwrap();
    let out = td.join("bundle");
    let run = folio_ceiling(&src, &faces, &out);
    assert_eq!(code(&run, "folio ceiling --write"), 0, "{}", stderr(&run));
    for (id, files, concat_len, hex) in expected() {
        let vp_dir = out.join(id);
        let mut want: Vec<String> = files.iter().map(|f| f.to_string()).collect();
        want.push("digest.txt".to_string());
        want.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        let got: Vec<String> = tree(&vp_dir).into_keys().collect();
        assert_eq!(got, want, "{id}: file の一覧");
        assert_digest(&vp_dir, hex, concat_len);
        assert_eq!(
            fs::read_to_string(vp_dir.join("finding.yaml")).unwrap(),
            FINDING,
            "{id}: finding.yaml"
        );
    }
    let _ = fs::remove_dir_all(&td);
}

#[test]
fn bundle_unknown_when_a_read_doc_is_not_a_document() {
    let (td, src, faces) = fixture_copy("read-doc");
    mutate(
        &src.join("ceiling.yaml"),
        "- {doc: index, fields: [shelf, sections]}",
        "- {doc: mystery, fields: [shelf, sections]}",
    );
    let run = folio_ceiling(&src, &faces, &td.join("bundle"));
    let _ = fs::remove_dir_all(&td);
    assert_unknown(&run, "reads の doc", &["行き先", "mystery"]);
}

// ── 5. 実の正本 ──

/// git を呼ぶ。環境変数 GIT_* は継承しない（tests/schema.rs と同じ形）。
fn git(cwd: &Path, args: &[&str]) {
    let mut cmd = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(key);
        }
    }
    let out = cmd
        .current_dir(cwd)
        .args([
            "-c",
            "user.email=fx@example",
            "-c",
            "user.name=fx",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .expect("git を起動できない");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// dir の直下の file の名（byte 順）。
fn names(dir: &Path) -> Vec<String> {
    let mut out: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| {
            let e = e.unwrap();
            e.path()
                .is_file()
                .then(|| e.file_name().to_string_lossy().into_owned())
        })
        .collect();
    out.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    out
}

/// 実の正本（design-intent の写し）から folio build で面を組み、束を組む。戻り値 = (一時 dir, 正本の写し, 面, 束)。
fn real_bundle(case: &str) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let td = temp_dir(case);
    let dir = td.join("design-intent");
    copy_tree(&design_intent(), &dir);
    fs::create_dir_all(td.join("contracts")).unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        td.join("contracts/schema.toml"),
    )
    .unwrap();
    copy_tree(
        &repo_root().join("vendor/archify"),
        &td.join("vendor/archify"),
    );
    // folio build の --write は最初に構造の床を回す（便 56・FR5）。写しを git の 1 commit にして床を合格させる
    git(&td, &["init", "-q"]);
    git(&td, &["add", "-A"]);
    git(&td, &["commit", "-q", "-m", "fixture"]);
    let site = td.join("site");
    let build = Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("build")
        .arg("--dir")
        .arg(&dir)
        .arg("--out")
        .arg(&site)
        .arg("--write")
        .output()
        .expect("folio を起動できない");
    assert_eq!(code(&build, "folio build --write"), 0, "{}", stderr(&build));

    let out = td.join("bundle");
    let run = folio_ceiling(&dir, &site, &out);
    assert_eq!(code(&run, "folio ceiling --write"), 0, "{}", stderr(&run));
    (td, dir, site, out)
}

const VIEWPOINTS: [&str; 4] = ["fidelity", "readability", "coherence", "reality"];

#[test]
fn bundle_on_the_real_source_builds_four_bundles() {
    let (td, dir, _site, out) = real_bundle("real");
    let dirs = VIEWPOINTS;
    for id in dirs {
        let vp_dir = out.join(id);
        assert!(vp_dir.is_dir(), "{id}: 観点の dir が無い");
        let digest = fs::read_to_string(vp_dir.join("digest.txt")).unwrap();
        let hex = digest
            .strip_prefix("sha256-files-1 ")
            .and_then(|h| h.strip_suffix('\n'))
            .unwrap_or_else(|| panic!("{id}: digest.txt の形が違う: {digest:?}"));
        match remeasure(&vp_dir).1 {
            Ok(m) => assert_eq!(m, hex, "{id}: sha256sum で測り直した要約値"),
            Err(why) => eprintln!("# まだ分からない: {id}: 要約値を測れない: {why}"),
        }
    }
    let mut top: Vec<String> = fs::read_dir(&out)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    top.sort();
    let mut want: Vec<String> = dirs.iter().map(|d| d.to_string()).collect();
    want.sort();
    assert_eq!(top, want);

    let fidelity = out.join("fidelity");
    // 固定の本数でなく、写した正本の adr/ の一覧と等しいことを言う（判断の記録が増えても歯は変わらない）。
    let source_adrs = names(&dir.join("adr"));
    let adrs = names(&fidelity.join("sources/adr"));
    assert_eq!(adrs, source_adrs, "sources/adr/ は写しの adr/ と同じ一覧");
    assert!(adrs.iter().all(|n| n.ends_with(".yaml")), "{adrs:?}");
    assert!(adrs.contains(&"schema.yaml".to_string()), "{adrs:?}");
    assert!(adrs.contains(&"ADR-8.yaml".to_string()), "{adrs:?}");
    let notes = names(&fidelity.join("sources/design-note"));
    assert_eq!(notes.len(), 3, "sources/design-note/: {notes:?}");
    let faces = names(&fidelity.join("faces"));
    let adr_faces = faces.iter().filter(|n| n.starts_with("adr-")).count();
    let note_faces: Vec<&String> = faces.iter().filter(|n| n.starts_with("note-")).collect();
    let source_adr_faces = source_adrs
        .iter()
        .filter(|n| n.starts_with("ADR-") && n.ends_with(".yaml"))
        .count();
    assert_eq!(adr_faces, source_adr_faces, "faces/: {faces:?}");
    assert_eq!(note_faces, ["note-example.html", "note-figures.html"]);
    assert!(!faces.contains(&"folio.css".to_string()), "{faces:?}");
    let _ = fs::remove_dir_all(&td);
}

// ── 6. 絞り（便 98・docs/design/delivery-98.md §1 (b)(e)）──

/// 列 0 の `名:` の行なら名（歯の側の読み・列 0 の `- ` と注釈は節でない）。
fn top_key(line: &str) -> Option<&str> {
    let line = line.trim_end_matches('\n');
    if line.starts_with([' ', '\t', '#', '-']) {
        return None;
    }
    let (name, rest) = line.split_once(':')?;
    (!name.is_empty() && (rest.is_empty() || rest.starts_with(' '))).then_some(name)
}

/// 束の 4 観点の sources/ の下の全 file について `f(観点, sources からの相対 path, 正本, 写し)`。
fn each_copy(dir: &Path, out: &Path, mut f: impl FnMut(&str, &str, &str, &str)) {
    for id in VIEWPOINTS {
        for (rel, body) in tree(&out.join(id)) {
            let Some(rel) = rel.strip_prefix("sources/") else {
                continue;
            };
            let source = fs::read_to_string(dir.join(rel)).unwrap();
            f(id, rel, &source, &String::from_utf8(body).unwrap());
        }
    }
}

/// 骨格 6 節の見出しの行と file の頭の行が 1 行も落ちない。
fn assert_skeleton_and_head(dir: &Path, out: &Path) {
    let mut seen = 0;
    each_copy(dir, out, |id, rel, source, copy| {
        let lines: Vec<&str> = source.split_inclusive('\n').collect();
        let first = lines.iter().position(|l| top_key(l).is_some()).unwrap_or(lines.len());
        let head: String = lines[..first].concat();
        assert!(copy.starts_with(&head), "{id}: {rel}: file の頭が落ちた");
        for line in &lines[first..] {
            if top_key(line).is_some_and(|k| SKELETON.contains(&k)) {
                assert!(
                    copy.split_inclusive('\n').any(|l| l == *line),
                    "{id}: {rel}: 骨格の行が落ちた: {line:?}"
                );
                seen += 1;
            }
        }
    });
    assert!(seen > 0, "骨格の行が 1 つも無い");
}

/// 写しの各行が正本の同じ file の行として byte のまま在り、順序も同じ（写しの行の列は正本の行の列の部分列）。
fn assert_verbatim_in_order(dir: &Path, out: &Path) {
    each_copy(dir, out, |id, rel, source, copy| {
        let mut rest = source.split_inclusive('\n');
        for line in copy.split_inclusive('\n') {
            assert!(
                rest.any(|l| l == line),
                "{id}: {rel}: 正本に無いか順が違う行: {line:?}"
            );
        }
    });
}

#[test]
fn f98_the_skeleton_and_the_head_survive_every_cut() {
    let (td, src, faces) = fixture_copy("f98-skeleton");
    let out = td.join("bundle");
    let run = folio_ceiling(&src, &faces, &out);
    assert_eq!(code(&run, "folio ceiling --write"), 0, "{}", stderr(&run));
    assert_skeleton_and_head(&src, &out);
    let _ = fs::remove_dir_all(&td);
    let (td, dir, _site, out) = real_bundle("f98-skeleton-real");
    assert_skeleton_and_head(&dir, &out);
    let _ = fs::remove_dir_all(&td);
}

#[test]
fn f98_every_kept_line_is_verbatim_and_in_order() {
    let (td, src, faces) = fixture_copy("f98-verbatim");
    let out = td.join("bundle");
    let run = folio_ceiling(&src, &faces, &out);
    assert_eq!(code(&run, "folio ceiling --write"), 0, "{}", stderr(&run));
    assert_verbatim_in_order(&src, &out);
    let _ = fs::remove_dir_all(&td);
    let (td, dir, _site, out) = real_bundle("f98-verbatim-real");
    assert_verbatim_in_order(&dir, &out);
    let _ = fs::remove_dir_all(&td);
}

/// 列 0 の `- ` の連なりは直前の節（figures）の続き（規則 2）。figures を宣言しない観点の写しに figures の行が残らない。
#[test]
fn f98_a_column_zero_sequence_is_not_a_new_section() {
    let (td, dir, _site, out) = real_bundle("f98-column-zero");
    let (mut kept, mut cut) = (0, 0);
    for id in VIEWPOINTS {
        let reads = fs::read_to_string(out.join(id).join("reads.yaml")).unwrap();
        let declares = |doc: &str| {
            reads.lines().any(|l| {
                l.strip_prefix(&format!("- {{doc: {doc}, fields: ["))
                    .is_some_and(|r| r.split([',', ' ', ']', '.']).any(|f| f == "figures"))
            })
        };
        for (doc, rel) in [
            ("srs", "srs.yaml"),
            ("adr", "adr/ADR-4.yaml"),
            ("adr", "adr/ADR-5.yaml"),
            ("adr", "adr/ADR-7.yaml"),
        ] {
            let path = out.join(id).join("sources").join(rel);
            if !path.exists() {
                continue;
            }
            let copy = fs::read_to_string(path).unwrap();
            let source = fs::read_to_string(dir.join(rel)).unwrap();
            let lines: Vec<&str> = source.split_inclusive('\n').collect();
            let start = lines.iter().position(|l| *l == "figures:\n").unwrap();
            let len = lines[start + 1..]
                .iter()
                .position(|l| top_key(l).is_some())
                .unwrap_or(lines.len() - start - 1);
            let mut block = &lines[start..=start + len];
            while block.last().is_some_and(|l| l.trim().is_empty() || l.trim_start().starts_with('#')) {
                block = &block[..block.len() - 1];
            }
            let dashes: Vec<&&str> = block.iter().filter(|l| l.starts_with("- ")).collect();
            assert!(!dashes.is_empty(), "{rel}: 列 0 の - の連なりが無い");
            if declares(doc) {
                assert!(copy.contains(&block.concat()), "{id}: {rel}: figures の節が残らない");
                kept += 1;
            } else {
                let copied: Vec<&str> = copy.split_inclusive('\n').collect();
                assert!(!copied.contains(&"figures:\n"), "{id}: {rel}: figures の見出しが残った");
                for d in dashes {
                    assert!(!copied.contains(d), "{id}: {rel}: figures の行が残った: {d:?}");
                }
                cut += 1;
            }
        }
    }
    let _ = fs::remove_dir_all(&td);
    assert!(kept > 0 && cut > 0, "残す側 {kept}・落とす側 {cut}");
}

/// 生成区間を持つ写しで開きの印と閉じの印がどちらも 1 つずつ残る（規則 3・規則 4）。
#[test]
fn f98_the_generated_region_stays_whole() {
    let (td, dir, _site, out) = real_bundle("f98-region");
    let mut pairs = 0;
    each_copy(&dir, &out, |id, rel, source, copy| {
        if !source.lines().any(|l| l.starts_with("# folio:schema:begin")) {
            return;
        }
        let count = |mark: &str| copy.lines().filter(|l| l.starts_with(mark)).count();
        assert_eq!(count("# folio:schema:begin"), 1, "{id}: {rel}: 開きの印");
        assert_eq!(count("# folio:schema:end"), 1, "{id}: {rel}: 閉じの印");
        pairs += 1;
    });
    let _ = fs::remove_dir_all(&td);
    assert!(pairs > 0, "生成区間を持つ写しが無い");
}
