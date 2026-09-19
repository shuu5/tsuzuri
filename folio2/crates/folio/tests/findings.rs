//! `folio ceiling --check`（便 39・docs/design/delivery-39.md §1 (e)(f)）の歯。binary 経由。
//! 束は便 38 の凍結 fixture（tests/fixtures/ceiling/bundle/）から `--write` で一時 dir に組み、
//! 所見 file の凍結 fixture（tests/fixtures/ceiling/findings/・10 本・AC16 の red_test）を <観点>/findings.yaml に写してから撃つ。
//! - 合格（4 観点 pass）・まだ分からない 11 種・不合格 4 種・束の側 4 種・混ざり 2 種・旗 2 種・実の正本
//!
//! 版管理の下の file は書き換えない（`--out` は必ず一時 dir の中）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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

    fn done(self) {
        let _ = fs::remove_dir_all(&self.td);
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

// ── 5. 混ざり ──

const REALITY_FAIL: &str = "verdict: 不合格
record:
  model: opus
  effort: high
  at: 2026-09-19T05:00:00Z
  read: [srs, adr, design-note]
  bundle: sha256-files-1 24ce1b872fa08dd128b39ac42e4ba72096bfe263df3bb463c9616899c96786d7
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
