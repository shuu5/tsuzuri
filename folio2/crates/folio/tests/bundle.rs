//! `folio ceiling --write`（便 38・docs/design/delivery-38.md §1 (e)(f)）の歯。binary 経由。
//! - 凍結 anchor（tests/fixtures/ceiling/bundle/・P-10.1）: 4 観点の file の一覧と要約値が planner の手組みの実測と一致し、
//!   sha256sum で測り直した要約値も同じ（folio の code に依らない・P-10.2）
//! - 写しの byte（正本と面は byte のまま・folio.css は写さない・観点ごとの面の有無・question / reads / finding の逐語）
//! - 決定性と全部か無しか（2 度組んで byte 一致・sources/ と faces/ は作り直し・所見 file と起動の記録は触らない）
//! - まだ分からない 6 種（正本が無い・読む正本が無い・面が無い・配信先が無い・親 dir が無い・行き先が一覧に無い）
//! - 読み手は一覧を床の定数から取る（便 47・delivery-47.md §1 (e)5）: fixture の天井の正本から旧 4 節を外し凍結 anchor
//!   tests/fixtures/schema/ceiling-region.txt の中身を足しても、凍結 anchor と同じ束が組める
//! - 実の正本（design-intent の写し・前段は folio build）
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

/// 観点の id・file の一覧（digest.txt を除く）・連結の byte 数・要約値。
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
            15_055,
            "2bd676377b9d6e75d752e645b11cd50088de5288b1de897bf12aa6f5e7cb5a32",
        ),
        (
            "readability",
            readability,
            17_317,
            "ded1548e1b6c5185b5b2ea0083d6ee68ed66dd5ca998bdd7d5d9a52ed2065576",
        ),
        (
            "coherence",
            coherence,
            15_770,
            "91ca924dceef12d5b2df3946e6b56de69b1feeef13ea66f78d950af54c28476c",
        ),
        (
            "reality",
            REALITY_FILES.to_vec(),
            11_385,
            "24ce1b872fa08dd128b39ac42e4ba72096bfe263df3bb463c9616899c96786d7",
        ),
    ]
}

const FIDELITY_QUESTION: &str = "id: fidelity\nname: 忠実さ\nreader: |\n  元の文と平易文の両方を読み、意味の差だけを拾う編集者\nquestion: |\n  人が書いた自由文（やさしく言うと・平易文・平易な説明）は、元の文（規範文・要件の文・決定の文）の意味を保っているか。義務を足していないか、落としていないか。専門語の日本語の言い換えは元の語と同じものを指しているか。図の根拠が指す先は、図の中身と合っているか。判定できない箇所は「まだ分からない」と書く。\n";

const FIDELITY_READS: &str = "- {doc: constitution, fields: [articles.plain, articles.statements.text]}\n- {doc: srs, fields: [requirements.plain, requirements.shall, acceptance.plain, acceptance.title]}\n- {doc: adr, fields: [plain, decision, options.text, figures.refs]}\n- {doc: design-note, fields: [sections, figures.refs]}\n";

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

// ── 2. 写しの byte ──

#[test]
fn bundle_copies_sources_and_faces_byte_for_byte() {
    let (td, src, faces) = fixture_copy("bytes");
    let out = td.join("bundle");
    let run = folio_ceiling(&src, &faces, &out);
    assert_eq!(code(&run, "folio ceiling --write"), 0, "{}", stderr(&run));
    let fidelity = out.join("fidelity");
    let srs = fs::read(fidelity.join("sources/srs.yaml")).unwrap();
    assert_eq!(srs, fs::read(fixture().join("source/srs.yaml")).unwrap());
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

#[test]
fn bundle_on_the_real_source_builds_four_bundles() {
    let td = temp_dir("real");
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
    let dirs = ["fidelity", "readability", "coherence", "reality"];
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
