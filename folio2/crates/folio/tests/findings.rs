//! `folio ceiling --check`（便 39・docs/design/delivery-39.md §1 (e)(f)）の歯。binary 経由。
//! 束は便 38 の凍結 fixture（tests/fixtures/ceiling/bundle/）から `--write` で一時 dir に組み、
//! 所見 file の凍結 fixture（tests/fixtures/ceiling/findings/・11 本・AC16 の red_test）を <観点>/findings.yaml に写してから撃つ。
//! - 合格（4 観点 pass）・まだ分からない 12 種・不合格 4 種・再判定待ち（便 41・規則 9）2 本・束の側 4 種・混ざり 2 種・旗 2 種・実の正本
//! - 反証の束（便 42・delivery-42.md §1 (e)(f)・`--refute`）: 凍結 anchor（要約値 843ad558…・P-10.1）・対象なし・
//!   result.yaml の読み 3 通り・違反 6 種・2 つの結果・旗 3 通り・実の正本
//!
//! 版管理の下の file は書き換えない（`--out` は必ず一時 dir の中）。

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const VIEWPOINTS: [&str; 4] = ["fidelity", "readability", "coherence", "reality"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// 便 38 の凍結 fixture（source/ = 欠陥を 1 つ仕込んだ最小の正本・faces/ = 面の写しの見本）。
fn bundle_fixture() -> PathBuf {
    repo_root().join("tests/fixtures/ceiling/bundle")
}

/// 所見 file の凍結 fixture。
fn findings_fixture(name: &str) -> String {
    fs::read_to_string(
        repo_root()
            .join("tests/fixtures/ceiling/findings")
            .join(name),
    )
    .unwrap()
}

fn design_intent() -> PathBuf {
    repo_root().join("design-intent")
}

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-findings-{case}-{}", std::process::id()));
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

/// 実の正本の写し（design-intent/ と器の導出 file と図の道具）を一時 dir へ写し、git の 1 commit にする
/// （folio build の --write は最初に構造の床を回す＝便 56・FR5・版管理の無い写しは「まだ分からない」の 2 になる）。
/// 戻り値 = 正本の写し。
fn real_copy(td: &Path) -> PathBuf {
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
    git(td, &["init", "-q"]);
    git(td, &["add", "-A"]);
    git(td, &["commit", "-q", "-m", "fixture"]);
    dir
}

/// `folio ceiling --dir <dir> --faces <faces> --out <out> <flags…>`。
fn folio_ceiling(dir: &Path, faces: &Path, out: &Path, flags: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("ceiling")
        .arg("--dir")
        .arg(dir)
        .arg("--faces")
        .arg(faces)
        .arg("--out")
        .arg(out)
        .args(flags)
        .output()
        .expect("folio を起動できない")
}

/// `folio ceiling --dir <dir> --out <out> <flags…>`（`--faces` なし・便 42 の `--refute` の形）。
fn folio_ceiling_no_faces(dir: &Path, out: &Path, flags: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("ceiling")
        .arg("--dir")
        .arg(dir)
        .arg("--out")
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

/// 組んだ束の置き場。fixture の写し（src・faces）と `--write` で組んだ束（bundle）を持つ。
struct Site {
    td: PathBuf,
    src: PathBuf,
    faces: PathBuf,
    out: PathBuf,
}

impl Site {
    /// fixture を一時 dir へ写し、`--write` で束を組み、4 観点に pass-<観点>.yaml を写す。
    fn passing(case: &str) -> Site {
        let td = temp_dir(case);
        let src = td.join("src");
        let faces = td.join("faces");
        copy_tree(&bundle_fixture().join("source"), &src);
        copy_tree(&bundle_fixture().join("faces"), &faces);
        let out = td.join("bundle");
        let run = folio_ceiling(&src, &faces, &out, &["--write"]);
        assert_eq!(code(&run, "folio ceiling --write"), 0, "{}", stderr(&run));
        let site = Site {
            td,
            src,
            faces,
            out,
        };
        for id in VIEWPOINTS {
            site.put(id, &findings_fixture(&format!("pass-{id}.yaml")));
        }
        site
    }

    /// <観点>/findings.yaml に書く。
    fn put(&self, id: &str, text: &str) {
        fs::write(self.out.join(id).join("findings.yaml"), text).unwrap();
    }

    fn check(&self) -> Output {
        folio_ceiling(&self.src, &self.faces, &self.out, &["--check"])
    }

    /// `--refute`（`--faces` なし）。
    fn refute(&self) -> Output {
        folio_ceiling_no_faces(&self.src, &self.out, &["--refute"])
    }

    /// fidelity の F-1 の反証の束の dir。
    fn f1(&self) -> PathBuf {
        self.out.join("fidelity/refute/F-1")
    }

    /// 4 観点 pass の上に fidelity を stop-unrefuted（止める F-1・refute なし）にして `--refute` を撃つ（凍結 anchor の形）。
    fn stop_unrefuted(case: &str) -> (Site, Output) {
        let site = Site::passing(case);
        site.put("fidelity", &findings_fixture("stop-unrefuted.yaml"));
        let run = site.refute();
        (site, run)
    }

    /// F-1 の反証の束の dir に result.yaml を置く。
    fn put_result(&self, text: &str) {
        fs::write(self.f1().join("result.yaml"), text).unwrap();
    }

    fn done(self) {
        let _ = fs::remove_dir_all(&self.td);
    }
}

/// 反証役の result.yaml（bundle は F-1 の digest.txt の中身）。
fn result_text(site: &Site, refute: &str) -> String {
    let digest = fs::read_to_string(site.f1().join("digest.txt")).unwrap();
    format!(
        "id: F-1\nrefute: {refute}\nmodel: opus\neffort: default\nat: 2026-09-19T08:00:00Z\nbundle: {}\n",
        digest.trim_end_matches('\n')
    )
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

/// 要約値（sha256）は命令 `sha256sum` を子の処理で撃って測る（歯は crate の中を読めない・tests/bundle.rs と同じ形）。
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

/// 反証の束の dir を (e) の測り方で測り直す: digest.txt と result.yaml を除く file を path の byte 順に並べ、中身を連結して
/// sha256sum に流す。戻り値 = (連結の byte 数, 要約値)。
fn remeasure_refute(dir: &Path) -> (usize, Result<String, String>) {
    let mut bytes = Vec::new();
    for (rel, body) in tree(dir) {
        if rel != "digest.txt" && rel != "result.yaml" {
            bytes.extend_from_slice(&body);
        }
    }
    (bytes.len(), sha256_hex(&bytes))
}

/// digest.txt の中身が `hex` で、sha256sum で測り直した要約値も同じ（`concat_len` = Some なら連結の byte 数も）。
fn assert_refute_digest(dir: &Path, hex: &str, concat_len: Option<usize>) {
    let digest = fs::read_to_string(dir.join("digest.txt")).unwrap();
    assert_eq!(digest, format!("sha256-files-1 {hex}\n"), "digest.txt");
    let (len, measured) = remeasure_refute(dir);
    if let Some(want) = concat_len {
        assert_eq!(len, want, "連結の byte 数");
    }
    match measured {
        Ok(m) => assert_eq!(m, hex, "sha256sum で測り直した要約値"),
        // 測る道具が無いときは合格にしない代わりに理由を出す（panic で落とさない・FR5 の形）
        Err(why) => eprintln!("# まだ分からない: 要約値を測れない: {why}"),
    }
}

/// 字面の変異（1 か所だけ）。
fn mutated(text: &str, from: &str, to: &str) -> String {
    assert_eq!(
        text.matches(from).count(),
        1,
        "変異の当て先が 1 か所でない: {from:?}"
    );
    text.replacen(from, to, 1)
}

fn last_line(s: &str) -> &str {
    s.trim_end_matches('\n').rsplit('\n').next().unwrap_or("")
}

fn first_line(s: &str) -> &str {
    s.split('\n').next().unwrap_or("")
}

/// 終了 `want` ∧ 標準エラーに `words` を全部含む（`case` は落ちたときの見出し）。
fn assert_outcome(run: &Output, case: &str, want: i32, words: &[&str]) {
    assert_eq!(
        code(run, case),
        want,
        "{case}: stdout:\n{}stderr:\n{}",
        stdout(run),
        stderr(run)
    );
    let err = stderr(run);
    for w in words {
        assert!(err.contains(w), "{case}: 「{w}」が無い: {err}");
    }
}

// ── 1. 合格 ──

#[test]
fn findings_pass_when_all_four_viewpoints_pass() {
    let site = Site::passing("pass");
    let run = site.check();
    site.done();
    assert_outcome(&run, "4 観点 pass", 0, &[]);
    let out = stdout(&run);
    assert_eq!(
        first_line(&out),
        "fidelity: 合格（所見 1・止める 0）",
        "{out}"
    );
    assert_eq!(
        last_line(&out),
        "folio ceiling: 合格（観点 4・合格 4・不合格 0・まだ分からない 0）",
        "{out}"
    );
    assert_eq!(out.lines().count(), 5, "{out}");
    assert!(stderr(&run).is_empty(), "{}", stderr(&run));
}

// ── 2. まだ分からない（fidelity だけ差し替え）──

/// fidelity の所見 file を `text` にして --check → 2 ∧ 標準エラーに `words` ∧ 最後の行は 3 合格 1 まだ分からない。
fn assert_unknown_fidelity(case: &str, text: Option<&str>, words: &[&str]) {
    let site = Site::passing(case);
    match text {
        Some(t) => site.put("fidelity", t),
        None => fs::remove_file(site.out.join("fidelity/findings.yaml")).unwrap(),
    }
    let run = site.check();
    site.done();
    assert_outcome(&run, case, 2, words);
    let err = stderr(&run);
    assert!(
        err.lines()
            .all(|l| l.starts_with("folio ceiling: fidelity: ")),
        "{case}: 他の観点に理由が出た: {err}"
    );
    assert_eq!(
        last_line(&stdout(&run)),
        "folio ceiling: まだ分からない（観点 4・合格 3・不合格 0・まだ分からない 1）",
        "{case}: {}",
        stdout(&run)
    );
}

#[test]
fn findings_unknown_when_a_record_field_is_missing() {
    assert_unknown_fidelity(
        "missing-field",
        Some(&findings_fixture("missing-field.yaml")),
        &["fidelity: ", "record", "at"],
    );
}

#[test]
fn findings_unknown_when_the_record_digest_differs_from_the_bundle() {
    assert_unknown_fidelity(
        "digest-mismatch",
        Some(&findings_fixture("digest-mismatch.yaml")),
        &["要約値が束と合わない"],
    );
}

#[test]
fn findings_unknown_when_the_evidence_is_not_in_the_sources() {
    assert_unknown_fidelity(
        "fabricated-evidence",
        Some(&findings_fixture("fabricated-evidence.yaml")),
        &["根拠が正本に無い", "F-1"],
    );
}

#[test]
fn findings_unknown_when_a_stop_finding_has_no_refute() {
    assert_unknown_fidelity(
        "stop-unrefuted",
        Some(&findings_fixture("stop-unrefuted.yaml")),
        &["反証が未", "F-1"],
    );
}

#[test]
fn findings_unknown_when_fail_has_no_findings() {
    assert_unknown_fidelity(
        "fail-no-findings",
        Some(&findings_fixture("fail-no-findings.yaml")),
        &["不合格なのに所見が無い"],
    );
}

#[test]
fn findings_unknown_when_the_ai_could_not_decide() {
    let text = mutated(
        &findings_fixture("pass-fidelity.yaml"),
        "verdict: 合格",
        "verdict: まだ分からない",
    );
    assert_unknown_fidelity("verdict-unknown", Some(&text), &["AI が判定できなかった"]);
}

#[test]
fn findings_unknown_when_the_top_level_has_an_unknown_field() {
    let mut text = findings_fixture("pass-fidelity.yaml");
    text.push_str("extra: 1\n");
    assert_unknown_fidelity("extra-field", Some(&text), &["未知の欄", "extra"]);
}

#[test]
fn findings_unknown_when_the_viewpoint_differs() {
    let text = mutated(
        &findings_fixture("pass-fidelity.yaml"),
        "viewpoint: fidelity",
        "viewpoint: readability",
    );
    assert_unknown_fidelity("viewpoint", Some(&text), &["viewpoint", "readability"]);
}

#[test]
fn findings_unknown_when_read_lacks_a_doc_of_reads() {
    let text = mutated(
        &findings_fixture("pass-fidelity.yaml"),
        "read: [constitution, srs, adr, design-note]",
        "read: [constitution, adr, design-note]",
    );
    assert_unknown_fidelity("read-srs", Some(&text), &["read", "srs"]);
}

#[test]
fn findings_unknown_when_the_weight_is_out_of_range() {
    let text = mutated(
        &findings_fixture("pass-fidelity.yaml"),
        "weight: 直す",
        "weight: 大変",
    );
    assert_unknown_fidelity("weight", Some(&text), &["weight", "大変"]);
}

#[test]
fn findings_unknown_when_the_findings_file_is_missing() {
    assert_unknown_fidelity("no-file", None, &["所見 file が無い"]);
}

// ── 3. 不合格 ──

#[test]
fn findings_fail_when_an_upheld_stop_finding_remains() {
    let site = Site::passing("stop-upheld");
    site.put("fidelity", &findings_fixture("stop-upheld.yaml"));
    let run = site.check();
    site.done();
    assert_outcome(
        &run,
        "stop-upheld",
        1,
        &["fidelity: ", "止める所見が残っている", "F-1"],
    );
    let out = stdout(&run);
    assert_eq!(
        first_line(&out),
        "fidelity: 不合格（所見 1・止める 1）",
        "{out}"
    );
    assert_eq!(
        last_line(&out),
        "folio ceiling: 不合格（観点 4・合格 3・不合格 1・まだ分からない 0）",
        "{out}"
    );
}

#[test]
fn findings_fail_when_the_verdict_is_fail_with_a_finding() {
    let site = Site::passing("verdict-fail");
    let text = mutated(
        &findings_fixture("pass-fidelity.yaml"),
        "verdict: 合格",
        "verdict: 不合格",
    );
    site.put("fidelity", &text);
    let run = site.check();
    site.done();
    assert_outcome(&run, "verdict 不合格", 1, &[]);
    assert_eq!(
        first_line(&stdout(&run)),
        "fidelity: 不合格（所見 1・止める 0）"
    );
}

#[test]
fn findings_pass_when_the_stop_finding_was_refuted() {
    let site = Site::passing("stop-refuted");
    let text = mutated(
        &findings_fixture("stop-upheld.yaml"),
        "refute: 支持",
        "refute: 退けた",
    );
    site.put("fidelity", &text);
    let run = site.check();
    site.done();
    assert_outcome(&run, "refute 退けた", 0, &[]);
    assert_eq!(
        first_line(&stdout(&run)),
        "fidelity: 合格（所見 1・止める 0）"
    );
}

#[test]
fn findings_fail_when_the_refute_is_undecided() {
    let site = Site::passing("stop-undecided");
    let text = mutated(
        &findings_fixture("stop-upheld.yaml"),
        "refute: 支持",
        "refute: まだ分からない",
    );
    site.put("fidelity", &text);
    let run = site.check();
    site.done();
    assert_outcome(
        &run,
        "refute まだ分からない",
        1,
        &["止める所見が残っている", "F-1"],
    );
    assert_eq!(
        first_line(&stdout(&run)),
        "fidelity: 不合格（所見 1・止める 1）"
    );
}

// ── 3'. 再判定待ち（便 41・delivery-41.md §1 (a)(c)・規則 9 の場合分け）──

#[test]
fn findings_unknown_when_all_stop_findings_were_refuted_under_a_fail_verdict() {
    let site = Site::passing("stop-refuted-fail");
    site.put("fidelity", &findings_fixture("stop-refuted.yaml"));
    let run = site.check();
    site.done();
    assert_outcome(
        &run,
        "stop-refuted",
        2,
        &[
            "fidelity: ",
            "止める所見が全部退けられた（再判定待ち・F-1）",
        ],
    );
    let out = stdout(&run);
    assert_eq!(
        first_line(&out),
        "fidelity: まだ分からない（所見 1・止める 0）",
        "{out}"
    );
    assert_eq!(
        last_line(&out),
        "folio ceiling: まだ分からない（観点 4・合格 3・不合格 0・まだ分からない 1）",
        "{out}"
    );
}

#[test]
fn findings_rule_nine_leaves_the_other_verdicts_as_before() {
    // 退けた で残る止めるが 0・verdict 合格 → 規則 10 の従来どおり 0
    let site = Site::passing("stop-refuted-pass");
    let text = mutated(
        &findings_fixture("stop-refuted.yaml"),
        "verdict: 不合格",
        "verdict: 合格",
    );
    site.put("fidelity", &text);
    let run = site.check();
    site.done();
    assert_outcome(&run, "stop-refuted + 合格", 0, &[]);
    assert_eq!(
        first_line(&stdout(&run)),
        "fidelity: 合格（所見 1・止める 0）"
    );

    // 止める 0・直す 1 で verdict 不合格 → 審査役の不合格を代行しない 1
    let site = Site::passing("fail-no-stops");
    let text = mutated(
        &findings_fixture("pass-fidelity.yaml"),
        "verdict: 合格",
        "verdict: 不合格",
    );
    site.put("fidelity", &text);
    let run = site.check();
    site.done();
    assert_outcome(&run, "直す だけの 不合格", 1, &[]);
    assert!(!stderr(&run).contains("再判定待ち"), "{}", stderr(&run));
    assert_eq!(
        first_line(&stdout(&run)),
        "fidelity: 不合格（所見 1・止める 0）"
    );
}

// ── 4. 束の側 ──

#[test]
fn findings_unknown_when_the_bundle_dir_is_missing() {
    let site = Site::passing("no-bundle");
    fs::remove_dir_all(site.out.join("fidelity")).unwrap();
    let run = site.check();
    site.done();
    assert_outcome(&run, "束が無い", 2, &["fidelity: 束が無い"]);
    assert_eq!(
        first_line(&stdout(&run)),
        "fidelity: まだ分からない（所見 0・止める 0）"
    );
}

#[test]
fn findings_unknown_when_the_bundle_is_corrupted() {
    let site = Site::passing("corrupted");
    let srs = site.out.join("fidelity/sources/srs.yaml");
    let mut bytes = fs::read(&srs).unwrap();
    assert_eq!(bytes[0], b'#');
    bytes[0] = b'%';
    fs::write(&srs, bytes).unwrap();
    let run = site.check();
    site.done();
    assert_outcome(&run, "束が壊れている", 2, &["fidelity: 束が壊れている"]);
    assert!(!stderr(&run).contains("束が古い"), "{}", stderr(&run));
}

#[test]
fn findings_unknown_when_the_bundle_is_stale() {
    let site = Site::passing("stale");
    let srs = site.src.join("srs.yaml");
    let text = mutated(
        &fs::read_to_string(&srs).unwrap(),
        "    plain: 合格か不合格のどちらかを出します。\n",
        "    plain: 合格・不合格・まだ分からない のどれかを出します。\n",
    );
    fs::write(&srs, text).unwrap();
    let run = site.check();
    site.done();
    assert_outcome(&run, "束が古い", 2, &["fidelity: 束が古い"]);
    assert!(!stderr(&run).contains("束が壊れている"), "{}", stderr(&run));
    // 他の 3 観点も srs を読むので古い
    for id in ["readability", "coherence", "reality"] {
        assert!(
            stderr(&run).contains(&format!("{id}: 束が古い")),
            "{id}: {}",
            stderr(&run)
        );
    }
}

#[test]
fn findings_unknown_when_the_faces_dir_is_missing() {
    let site = Site::passing("no-faces");
    let run = folio_ceiling(&site.src, &site.td.join("nowhere"), &site.out, &["--check"]);
    site.done();
    assert_outcome(
        &run,
        "配信先が無い",
        2,
        &["folio ceiling: まだ分からない: ", "配信先が無い"],
    );
    assert!(stdout(&run).is_empty(), "{}", stdout(&run));
}

// ── 4'. 絞った束（便 98・delivery-98.md §1 (e) 歯 8）──

/// 絞った束に凍結の所見 fixture を当てて 3 値が今と同じ（根拠は絞った写しの中に残る・作り話の根拠は外のまま）。
#[test]
fn f98_the_frozen_findings_still_pass_the_check() {
    let site = Site::passing("f98-frozen");
    let run = site.check();
    assert_outcome(&run, "pass 4 本", 0, &[]);
    let srs = fs::read_to_string(site.out.join("fidelity/sources/srs.yaml")).unwrap();
    assert!(!srs.contains("\nrail:\n"), "srs.yaml が絞られていない");
    for (name, want, word) in [
        ("stop-upheld", 1, "止める所見が残っている"),
        ("stop-refuted", 2, "再判定待ち"),
        ("stop-unrefuted", 2, "反証が未"),
        ("fail-no-findings", 2, "不合格なのに所見が無い"),
        ("missing-field", 2, "record"),
        ("digest-mismatch", 2, "要約値が束と合わない"),
        ("fabricated-evidence", 2, "根拠が正本に無い"),
    ] {
        site.put("fidelity", &findings_fixture(&format!("{name}.yaml")));
        assert_outcome(&site.check(), name, want, &[word]);
    }
    site.done();
}

// ── 5. 混ざり ──

const REALITY_FAIL: &str = "verdict: 不合格
record:
  model: opus
  effort: high
  at: 2026-09-19T05:00:00Z
  read: [srs, adr, design-note]
  bundle: sha256-files-1 d8734c77995da9034593d27e757890ec9133c86444a466b05c85a1a9e1250d89
findings:
  - id: R-1
    viewpoint: reality
    place: {doc: srs, at: requirements.FR2.plain}
    weight: 直す
    evidence: 合格か不合格のどちらかを出します。
";

#[test]
fn findings_fail_when_one_viewpoint_fails_and_the_rest_pass() {
    let site = Site::passing("mixed-fail");
    site.put("reality", REALITY_FAIL);
    let run = site.check();
    site.done();
    assert_outcome(&run, "reality 不合格", 1, &[]);
    let out = stdout(&run);
    assert!(out.contains("reality: 不合格（所見 1・止める 0）"), "{out}");
    assert_eq!(
        last_line(&out),
        "folio ceiling: 不合格（観点 4・合格 3・不合格 1・まだ分からない 0）"
    );
}

#[test]
fn findings_unknown_takes_precedence_over_fail() {
    let site = Site::passing("mixed-unknown");
    site.put("reality", REALITY_FAIL);
    site.put("fidelity", &findings_fixture("missing-field.yaml"));
    let run = site.check();
    site.done();
    assert_outcome(&run, "不合格 + まだ分からない", 2, &["fidelity: "]);
    assert_eq!(
        last_line(&stdout(&run)),
        "folio ceiling: まだ分からない（観点 4・合格 2・不合格 1・まだ分からない 1）"
    );
}

// ── 6. 旗 ──

#[test]
fn findings_flags_require_exactly_one_of_write_and_check() {
    let site = Site::passing("flags");
    let both = folio_ceiling(&site.src, &site.faces, &site.out, &["--write", "--check"]);
    let none = folio_ceiling(&site.src, &site.faces, &site.out, &[]);
    site.done();
    assert_eq!(code(&both, "--write --check"), 2, "{}", stderr(&both));
    assert!(
        stderr(&both).to_lowercase().contains("usage"),
        "{}",
        stderr(&both)
    );
    assert_eq!(code(&none, "旗なし"), 2, "{}", stderr(&none));
}

// ── 7. 実の正本 ──

#[test]
fn findings_check_passes_on_the_real_source_with_empty_findings() {
    let td = temp_dir("real");
    let dir = real_copy(&td);
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
    let write = folio_ceiling(&dir, &site, &out, &["--write"]);
    assert_eq!(
        code(&write, "folio ceiling --write"),
        0,
        "{}",
        stderr(&write)
    );

    for id in VIEWPOINTS {
        let vp_dir = out.join(id);
        let digest = fs::read_to_string(vp_dir.join("digest.txt")).unwrap();
        let reads = fs::read_to_string(vp_dir.join("reads.yaml")).unwrap();
        let docs: Vec<&str> = reads
            .lines()
            .filter(|l| !l.starts_with('#')) // 落とした節の注釈（便 98）
            .map(|l| {
                l.strip_prefix("- {doc: ")
                    .and_then(|r| r.split(',').next())
                    .unwrap_or_else(|| panic!("{id}: reads.yaml の行の形が違う: {l}"))
            })
            .collect();
        let text = format!(
            "verdict: 合格\nrecord:\n  model: opus\n  effort: high\n  at: 2026-09-19T05:00:00Z\n  read: [{}]\n  bundle: {}\nfindings: []\n",
            docs.join(", "),
            digest.trim_end_matches('\n')
        );
        fs::write(vp_dir.join("findings.yaml"), text).unwrap();
    }
    let run = folio_ceiling(&dir, &site, &out, &["--check"]);
    let _ = fs::remove_dir_all(&td);
    assert_outcome(&run, "実の正本", 0, &[]);
    assert_eq!(
        last_line(&stdout(&run)),
        "folio ceiling: 合格（観点 4・合格 4・不合格 0・まだ分からない 0）"
    );
}

// ══ 反証の束（便 42・delivery-42.md §1 (e)(f)・`--refute`）══

/// (e) 凍結 anchor: fidelity の F-1 の反証の束（file の一覧・連結の byte 数・要約値）。
const REFUTE_FILES: [&str; 5] = [
    "finding.yaml",
    "question.yaml",
    "reads.yaml",
    "schema.yaml",
    "sources.txt",
];
const REFUTE_CONCAT_LEN: usize = 2_338;
const REFUTE_DIGEST: &str = "ec8643e3a9942c05096c506eaf6c11a35d26a323c53a4852148f248cbdea0d46";

const REFUTE_FINDING: &str = "id: F-1\nviewpoint: fidelity\nplace: {doc: srs, at: requirements.FR2.plain}\nweight: 止める\nevidence: |\n  合格か不合格のどちらかを出します。\nnote: |\n  規範文の 3 値のうち「まだ分からない」を平易文が落としている。\n";

/// 便 38 の (e) の fidelity の question.yaml の 6 行 + 反証の 4 行。
const REFUTE_QUESTION: &str = "id: fidelity\nname: 忠実さ\nreader: |\n  元の文と平易文の両方を読み、意味の差だけを拾う編集者\nquestion: |\n  人が書いた自由文（やさしく言うと・平易文・平易な説明）は、元の文（規範文・要件の文・決定の文）の意味を保っているか。義務を足していないか、落としていないか。専門語の日本語の言い換えは元の語と同じものを指しているか。図の根拠が指す先は、図の中身と合っているか。判定できない箇所は「まだ分からない」と書く。\nrefute:\n  values: [支持, 退けた, まだ分からない]\n  rule: |\n    所見を出した文脈から独立して中立に検証する。根拠が正本に逐語で在り、主張が正本の文から裏付けられれば 支持。根拠が無い、または主張が正本の文と両立しないと裏付けられれば 退けた。材料だけでは決められなければ まだ分からない（所見は残る）。\n";

/// 親の観点の reads.yaml の写し（便 98 の落とした節の注釈を含む・delivery-98.md §1 (c) の 1 つ目の逐語）。
const REFUTE_READS: &str = "- {doc: constitution, fields: [articles.plain, articles.statements.text]}\n- {doc: srs, fields: [requirements.plain, requirements.shall, acceptance.plain, acceptance.title]}\n- {doc: adr, fields: [plain, decision, options.text, figures.refs]}\n- {doc: design-note, fields: [sections, figures.refs]}\n# 落とした節 adr/ADR-2.yaml: context, basis, retreat, amends, consequences\n# 落とした節 constitution.yaml: north_star, precedence, rules_pointer, amendment, glossary_pointer, sources\n# 落とした節 design-note/full.yaml: sources\n# 落とした節 srs.yaml: goals, scope, scope_m1, actors, outputs, rail, verdicts, nonfunctional, not_frozen, constraints, glossary_pointer, figures\n# 常に残す節: meta, id, title, status, date, schema\n";

const REFUTE_SOURCES: &str =
    "sha256-files-1 d2e153b29a2b46b88295d10f782163636a5613649ceb9efa80dcebfb48473782\n";

const REFUTE_SCHEMA: &str = "# 反証の結果の欄の決まり（folio ceiling --refute が組んだ・結果は同じ dir の result.yaml に書く）\nresult:\n  required: [id, refute, model, effort, at, bundle]\n  values: [支持, 退けた, まだ分からない]\n";

// ── R1. 凍結 anchor ──

#[test]
fn findings_refute_matches_the_frozen_anchor() {
    let (site, run) = Site::stop_unrefuted("refute-anchor");
    assert_outcome(&run, "folio ceiling --refute", 0, &[]);
    // byte 数 = 5 つの連結 + digest.txt の 1 行（「sha256-files-1 」+ 16 進 64 字 + 改行）
    let digest_line_len = "sha256-files-1 ".len() + 64 + 1;
    assert_eq!(
        stdout(&run),
        format!(
            "folio ceiling: 反証の束を組んだ（所見 1・file 6・{} byte）\n",
            REFUTE_CONCAT_LEN + digest_line_len
        )
    );
    assert!(stderr(&run).is_empty(), "{}", stderr(&run));
    let f1 = site.f1();
    let mut want: Vec<String> = REFUTE_FILES.iter().map(|f| f.to_string()).collect();
    want.push("digest.txt".to_string());
    want.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    let first = tree(&f1);
    let got: Vec<String> = first.keys().cloned().collect();
    assert_eq!(got, want, "file の一覧");
    assert_refute_digest(&f1, REFUTE_DIGEST, Some(REFUTE_CONCAT_LEN));
    assert_eq!(
        fs::read_to_string(f1.join("finding.yaml")).unwrap(),
        REFUTE_FINDING
    );
    assert_eq!(
        fs::read_to_string(f1.join("question.yaml")).unwrap(),
        REFUTE_QUESTION
    );
    assert_eq!(
        fs::read_to_string(f1.join("reads.yaml")).unwrap(),
        REFUTE_READS
    );
    assert_eq!(
        fs::read_to_string(f1.join("sources.txt")).unwrap(),
        REFUTE_SOURCES
    );
    assert_eq!(
        fs::read_to_string(f1.join("schema.yaml")).unwrap(),
        REFUTE_SCHEMA
    );
    // 他の観点には refute/ を作らない・所見 file は触らない
    for id in ["readability", "coherence", "reality"] {
        assert!(!site.out.join(id).join("refute").exists(), "{id}: refute/");
    }
    assert_eq!(
        fs::read_to_string(site.out.join("fidelity/findings.yaml")).unwrap(),
        findings_fixture("stop-unrefuted.yaml")
    );

    // 2 度組んで byte 一致（決定性）
    let again = site.refute();
    assert_outcome(&again, "2 回目", 0, &[]);
    assert_eq!(tree(&f1), first, "2 度組むと byte が違う（決定的でない）");
    site.done();
}

// ── R2. 対象なし ──

#[test]
fn findings_refute_writes_nothing_when_no_stop_finding_is_unrefuted() {
    // 4 観点 pass（止める 0）
    let site = Site::passing("refute-none");
    let run = site.refute();
    assert_outcome(&run, "止める 0", 0, &[]);
    assert_eq!(
        stdout(&run),
        "folio ceiling: 反証の束を組んだ（所見 0・file 0・0 byte）\n"
    );
    for id in VIEWPOINTS {
        assert!(!site.out.join(id).join("refute").exists(), "{id}: refute/");
    }
    site.done();

    // stop-upheld（refute 在り）は対象外
    let site = Site::passing("refute-upheld");
    site.put("fidelity", &findings_fixture("stop-upheld.yaml"));
    let run = site.refute();
    assert_outcome(&run, "refute 在り", 0, &[]);
    assert_eq!(
        stdout(&run),
        "folio ceiling: 反証の束を組んだ（所見 0・file 0・0 byte）\n"
    );
    assert!(!site.out.join("fidelity/refute").exists());
    site.done();
}

// ── R3. result.yaml の読み ──

#[test]
fn findings_check_reads_the_result_file_as_the_refute_of_the_stop_finding() {
    // 退けた → 残る止めるが 0・verdict 合格 → 規則 10 で 0
    let (site, run) = Site::stop_unrefuted("result-refuted");
    assert_outcome(&run, "--refute", 0, &[]);
    site.put_result(&result_text(&site, "退けた"));
    let run = site.check();
    assert_outcome(&run, "result 退けた", 0, &[]);
    assert_eq!(
        first_line(&stdout(&run)),
        "fidelity: 合格（所見 1・止める 0）"
    );

    // 支持 → 止めるが残る → 1
    site.put_result(&result_text(&site, "支持"));
    let run = site.check();
    assert_outcome(&run, "result 支持", 1, &["止める所見が残っている（F-1）"]);
    assert_eq!(
        first_line(&stdout(&run)),
        "fidelity: 不合格（所見 1・止める 1）"
    );

    // まだ分からない → 所見は残る → 1
    site.put_result(&result_text(&site, "まだ分からない"));
    let run = site.check();
    assert_outcome(
        &run,
        "result まだ分からない",
        1,
        &["止める所見が残っている（F-1）"],
    );

    // verdict 不合格 + 退けた → 再判定待ち（便 41 の規則 9）→ 2
    site.put(
        "fidelity",
        &mutated(
            &findings_fixture("stop-unrefuted.yaml"),
            "verdict: 合格",
            "verdict: 不合格",
        ),
    );
    site.put_result(&result_text(&site, "退けた"));
    let run = site.check();
    site.done();
    assert_outcome(&run, "不合格 + 退けた", 2, &["再判定待ち・F-1"]);
    assert_eq!(
        first_line(&stdout(&run)),
        "fidelity: まだ分からない（所見 1・止める 0）"
    );
}

// ── R4. result.yaml の違反 ──

/// 歯 1 の後に `text` を result.yaml に置いて --check → 2 ∧ 標準エラーに `words`。`mutate_bundle` = 反証の束の
/// finding.yaml の 1 byte を書き換える。
fn assert_result_unknown(case: &str, text: &str, mutate_bundle: bool, words: &[&str]) {
    let (site, run) = Site::stop_unrefuted(case);
    assert_outcome(&run, "--refute", 0, &[]);
    let text = text.replace(
        "<bundle>",
        fs::read_to_string(site.f1().join("digest.txt"))
            .unwrap()
            .trim_end_matches('\n'),
    );
    site.put_result(&text);
    if mutate_bundle {
        let finding = site.f1().join("finding.yaml");
        let mut bytes = fs::read(&finding).unwrap();
        assert_eq!(bytes[0], b'i');
        bytes[0] = b'I';
        fs::write(&finding, bytes).unwrap();
    }
    let run = site.check();
    site.done();
    assert_outcome(&run, case, 2, words);
    assert_eq!(
        first_line(&stdout(&run)),
        "fidelity: まだ分からない（所見 1・止める 1）",
        "{case}"
    );
}

const RESULT_OK: &str = "id: F-1\nrefute: 退けた\nmodel: opus\neffort: default\nat: 2026-09-19T08:00:00Z\nbundle: <bundle>\n";

#[test]
fn findings_unknown_when_the_result_bundle_differs() {
    assert_result_unknown(
        "result-bundle",
        &RESULT_OK.replace("<bundle>", "sha256-files-1 0000"),
        false,
        &["反証の結果 file: F-1", "bundle"],
    );
}

#[test]
fn findings_unknown_when_the_result_refute_is_out_of_range() {
    assert_result_unknown(
        "result-range",
        &mutated(RESULT_OK, "refute: 退けた", "refute: 大変"),
        false,
        &["反証の結果 file: F-1", "refute", "大変"],
    );
}

#[test]
fn findings_unknown_when_the_result_id_differs() {
    assert_result_unknown(
        "result-id",
        &mutated(RESULT_OK, "id: F-1", "id: F-9"),
        false,
        &["反証の結果 file: F-1", "id", "F-9"],
    );
}

#[test]
fn findings_unknown_when_a_result_field_is_missing() {
    assert_result_unknown(
        "result-missing",
        &mutated(RESULT_OK, "at: 2026-09-19T08:00:00Z\n", ""),
        false,
        &["反証の結果 file: F-1", "at が無い"],
    );
}

#[test]
fn findings_unknown_when_the_result_has_an_extra_field() {
    let mut text = RESULT_OK.to_string();
    text.push_str("extra: 1\n");
    assert_result_unknown(
        "result-extra",
        &text,
        false,
        &["反証の結果 file: F-1", "未知の欄", "extra"],
    );
}

#[test]
fn findings_unknown_when_the_refute_bundle_is_corrupted() {
    assert_result_unknown(
        "result-corrupted",
        RESULT_OK,
        true,
        &["反証の結果 file: F-1", "反証の束が壊れている"],
    );
}

// ── R5. 2 つの結果 ──

#[test]
fn findings_unknown_when_the_findings_file_and_the_result_disagree() {
    let (site, run) = Site::stop_unrefuted("two-results");
    assert_outcome(&run, "--refute", 0, &[]);
    let mut text = findings_fixture("stop-unrefuted.yaml");
    text.push_str("    refute: 支持\n");
    site.put("fidelity", &text);
    site.put_result(&result_text(&site, "退けた"));
    let run = site.check();
    assert_outcome(&run, "支持 と 退けた", 2, &["反証の結果が 2 つ（F-1）"]);

    // 同じ値は可（両方 支持 → 止めるが残る → 1）
    site.put_result(&result_text(&site, "支持"));
    let run = site.check();
    site.done();
    assert_outcome(&run, "支持 と 支持", 1, &["止める所見が残っている（F-1）"]);
    assert!(!stderr(&run).contains("2 つ"), "{}", stderr(&run));
}

// ── R6. 旗 ──

#[test]
fn findings_refute_flags_are_exclusive_and_faces_is_optional_only_for_refute() {
    let site = Site::passing("refute-flags");
    let both = folio_ceiling(&site.src, &site.faces, &site.out, &["--refute", "--write"]);
    let write_no_faces = folio_ceiling_no_faces(&site.src, &site.out, &["--write"]);
    let check_no_faces = folio_ceiling_no_faces(&site.src, &site.out, &["--check"]);
    let refute_no_faces = folio_ceiling_no_faces(&site.src, &site.out, &["--refute"]);
    site.done();
    assert_eq!(code(&both, "--refute --write"), 2, "{}", stderr(&both));
    assert!(
        stderr(&both).to_lowercase().contains("usage"),
        "{}",
        stderr(&both)
    );
    assert_outcome(
        &write_no_faces,
        "--write で --faces なし",
        2,
        &["folio ceiling: まだ分からない: --faces が要る"],
    );
    assert_outcome(
        &check_no_faces,
        "--check で --faces なし",
        2,
        &["folio ceiling: まだ分からない: --faces が要る"],
    );
    assert_outcome(&refute_no_faces, "--refute で --faces なし", 0, &[]);
}

// ── R7. 実の正本 ──

#[test]
fn findings_refute_on_the_real_source_builds_a_bundle_the_check_reads() {
    let td = temp_dir("refute-real");
    let dir = real_copy(&td);
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
    let write = folio_ceiling(&dir, &site, &out, &["--write"]);
    assert_eq!(
        code(&write, "folio ceiling --write"),
        0,
        "{}",
        stderr(&write)
    );

    // 所見 file: fidelity に 止める 1 件（evidence は sources/srs.yaml の逐語・refute なし）・他の 3 観点は所見なし
    for id in VIEWPOINTS {
        let vp_dir = out.join(id);
        let digest = fs::read_to_string(vp_dir.join("digest.txt")).unwrap();
        let reads = fs::read_to_string(vp_dir.join("reads.yaml")).unwrap();
        let docs: Vec<&str> = reads
            .lines()
            .filter(|l| !l.starts_with('#')) // 落とした節の注釈（便 98）
            .map(|l| {
                l.strip_prefix("- {doc: ")
                    .and_then(|r| r.split(',').next())
                    .unwrap_or_else(|| panic!("{id}: reads.yaml の行の形が違う: {l}"))
            })
            .collect();
        let findings = if id == "fidelity" {
            let srs = fs::read_to_string(vp_dir.join("sources/srs.yaml")).unwrap();
            let evidence = srs
                .lines()
                .filter_map(|l| l.strip_prefix("    plain: "))
                .find(|v| !v.contains('"') && !v.contains('\\'))
                .expect("sources/srs.yaml に plain: の行が無い");
            format!(
                "\n  - id: F-1\n    viewpoint: fidelity\n    place: {{doc: srs, at: requirements.plain}}\n    weight: 止める\n    evidence: \"{evidence}\"\n"
            )
        } else {
            " []\n".to_string()
        };
        let text = format!(
            "verdict: 合格\nrecord:\n  model: opus\n  effort: high\n  at: 2026-09-19T05:00:00Z\n  read: [{}]\n  bundle: {}\nfindings:{findings}",
            docs.join(", "),
            digest.trim_end_matches('\n')
        );
        fs::write(vp_dir.join("findings.yaml"), text).unwrap();
    }

    let refute = folio_ceiling_no_faces(&dir, &out, &["--refute"]);
    assert_outcome(&refute, "folio ceiling --refute", 0, &[]);
    assert!(
        stdout(&refute).starts_with("folio ceiling: 反証の束を組んだ（所見 1・file 6・"),
        "{}",
        stdout(&refute)
    );
    let f1 = out.join("fidelity/refute/F-1");
    let digest = fs::read_to_string(f1.join("digest.txt")).unwrap();
    let hex = digest
        .strip_prefix("sha256-files-1 ")
        .and_then(|h| h.strip_suffix('\n'))
        .unwrap_or_else(|| panic!("digest.txt の形が違う: {digest:?}"));
    assert_refute_digest(&f1, hex, None);
    assert_eq!(
        fs::read(f1.join("sources.txt")).unwrap(),
        fs::read(out.join("fidelity/digest.txt")).unwrap(),
        "sources.txt は親の digest.txt の写し"
    );
    assert_eq!(
        fs::read(f1.join("reads.yaml")).unwrap(),
        fs::read(out.join("fidelity/reads.yaml")).unwrap(),
        "reads.yaml は親の写し"
    );

    // 反証役の結果（退けた）を置いて --check → 規則 10 で 0
    fs::write(
        f1.join("result.yaml"),
        format!(
            "id: F-1\nrefute: 退けた\nmodel: opus\neffort: default\nat: 2026-09-19T08:00:00Z\nbundle: {}\n",
            digest.trim_end_matches('\n')
        ),
    )
    .unwrap();
    let run = folio_ceiling(&dir, &site, &out, &["--check"]);
    let _ = fs::remove_dir_all(&td);
    assert_outcome(&run, "実の正本 + 反証", 0, &[]);
    assert_eq!(
        first_line(&stdout(&run)),
        "fidelity: 合格（所見 1・止める 0）"
    );
    assert_eq!(
        last_line(&stdout(&run)),
        "folio ceiling: 合格（観点 4・合格 4・不合格 0・まだ分からない 0）"
    );
}
