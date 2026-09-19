//! `folio check` の天井の正本（ceiling.yaml）の検査の歯（便 37・docs/design/delivery-37.md §1 (e)）。
//! fixture は増やさず、歯の中で design-intent の写し全部を一時 dir に作り git init と 1 commit を行い
//! （版管理の無い写しは別の理由で「まだ分からない」になる）、ceiling.yaml に変異を 1 つ当てて `folio check --dir` を回す。
//! 違反の歯は、変異が 1 つなら違反の件数が 1 であること（出力の件数の表示）も確かめる。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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

/// 器（scribe2）の導出 file を写しの根へ写す（設計ノートの契約表の節が読む先・便 23）。
fn copy_external_schema(root: &Path) {
    fs::create_dir_all(root.join("contracts")).unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        root.join("contracts/schema.toml"),
    )
    .unwrap();
}

/// git を呼ぶ。環境変数 GIT_* は継承しない。
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

/// 写しの一時 dir（歯の終わりに消す）。
struct Work {
    root: PathBuf,
}

impl Work {
    fn new(case: &str) -> Work {
        let root =
            std::env::temp_dir().join(format!("folio-ceiling-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_tree(
            &repo_root().join("design-intent"),
            &root.join("design-intent"),
        );
        copy_external_schema(&root);
        git(&root, &["init", "-q"]);
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-q", "-m", "fixture"]);
        Work { root }
    }

    fn ceiling(&self) -> PathBuf {
        self.root.join("design-intent/ceiling.yaml")
    }

    /// ceiling.yaml の字面の変異（1 か所だけ）。
    fn mutate(&self, from: &str, to: &str) {
        let before = fs::read_to_string(self.ceiling()).unwrap();
        assert_eq!(
            before.matches(from).count(),
            1,
            "変異の当て先が 1 か所でない: {from:?}"
        );
        fs::write(self.ceiling(), before.replacen(from, to, 1)).unwrap();
    }

    fn check(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("check")
            .arg("--dir")
            .arg(self.root.join("design-intent"))
            .output()
            .expect("folio を起動できない")
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// 違反の行（`[種類] …`）だけを拾う。
fn violations(out: &Output) -> Vec<String> {
    stdout(out)
        .lines()
        .filter(|l| l.starts_with('['))
        .map(str::to_string)
        .collect()
}

fn assert_passes(out: &Output) {
    assert_eq!(out.status.code(), Some(0), "{}{}", stdout(out), stderr(out));
    assert!(violations(out).is_empty(), "{:?}", violations(out));
    assert!(
        stdout(out).contains("folio check: 合格（違反 0・"),
        "{}",
        stdout(out)
    );
}

/// 不合格 1・違反はちょうど 1 件（件数の表示も 1）・その 1 件が種別 `kind` で `words` を全部含む。
fn assert_single_violation(out: &Output, kind: &str, words: &[&str]) {
    assert_eq!(out.status.code(), Some(1), "{}{}", stdout(out), stderr(out));
    let v = violations(out);
    assert_eq!(v.len(), 1, "違反は変異の 1 件だけのはず: {v:?}");
    assert!(v[0].starts_with(&format!("[{kind}] ceiling.yaml")), "{v:?}");
    for w in words {
        assert!(v[0].contains(w), "「{w}」が無い: {v:?}");
    }
    assert!(
        stdout(out).contains("folio check: 不合格（違反 1・"),
        "{}",
        stdout(out)
    );
}

/// まだ分からない 2・違反 0・標準エラーに `word` を含む「まだ分からない」の行が在る。
fn assert_unknown(out: &Output, word: &str) {
    assert_eq!(out.status.code(), Some(2), "{}{}", stdout(out), stderr(out));
    assert!(violations(out).is_empty(), "{:?}", violations(out));
    assert!(
        stderr(out)
            .lines()
            .any(|l| l.starts_with("# まだ分からない: ") && l.contains(word)),
        "{}",
        stderr(out)
    );
    assert!(
        !stdout(out).contains("folio check: 合格"),
        "{}",
        stdout(out)
    );
}

#[test]
fn ceiling_canonical_copy_passes() {
    let w = Work::new("canonical");
    assert_passes(&w.check());
}

#[test]
fn ceiling_missing_ceiling_is_unknown() {
    let w = Work::new("missing");
    fs::remove_file(w.ceiling()).unwrap();
    assert_unknown(&w.check(), "ceiling.yaml: 正本が無い");
}

#[test]
fn ceiling_unknown_section_fails() {
    let w = Work::new("unknown-section");
    let mut text = fs::read_to_string(w.ceiling()).unwrap();
    text.push_str("\nextra:\n  x: 1\n");
    fs::write(w.ceiling(), text).unwrap();
    assert_single_violation(&w.check(), "未知の節", &["未知の節「extra」"]);
}

#[test]
fn ceiling_empty_question_fails() {
    let w = Work::new("empty-field");
    w.mutate("    question: 入口から歩いて", "    question: \"\" #");
    assert_single_violation(
        &w.check(),
        "欄の非空",
        &["viewpoints の行 readability の question が空"],
    );
}

#[test]
fn ceiling_viewpoint_id_off_the_list_fails() {
    let w = Work::new("viewpoint-id");
    w.mutate("  - id: readability\n", "  - id: mystery\n");
    assert_single_violation(
        &w.check(),
        "ceiling",
        &["viewpoints の id", "一覧", "mystery"],
    );
}

#[test]
fn ceiling_three_viewpoints_fails() {
    let w = Work::new("viewpoint-count");
    let before = fs::read_to_string(w.ceiling()).unwrap();
    let start = before
        .find("  - id: reality\n")
        .expect("観点 reality が無い");
    let end = before
        .find("\n# 所見 1 件の欄の決まり")
        .expect("finding の節の前の註が無い");
    let after = format!("{}{}", &before[..start], &before[end + 1..]);
    fs::write(w.ceiling(), after).unwrap();
    assert_single_violation(
        &w.check(),
        "ceiling",
        &["viewpoints の id", "一覧", "（無い: reality）"],
    );
}

#[test]
fn ceiling_verdict_order_fails() {
    let w = Work::new("verdict-order");
    w.mutate(
        "values: [合格, 不合格, まだ分からない]",
        "values: [不合格, 合格, まだ分からない]",
    );
    assert_single_violation(
        &w.check(),
        "ceiling",
        &["verdicts の values", "一覧「不合格, 合格, まだ分からない」"],
    );
}

#[test]
fn ceiling_missing_document_fails() {
    let w = Work::new("document-missing");
    w.mutate(
        "  - {id: ceiling, file: ceiling.yaml, note: この正本}\n",
        "",
    );
    assert_single_violation(
        &w.check(),
        "ceiling",
        &["documents の id", "一覧", "（無い: ceiling）"],
    );
}

#[test]
fn ceiling_read_doc_not_a_document_fails() {
    let w = Work::new("read-doc");
    w.mutate(
        "      - {doc: index, fields: [shelf, sections]}",
        "      - {doc: nowhere, fields: [shelf, sections]}",
    );
    assert_single_violation(
        &w.check(),
        "ceiling",
        &[
            "viewpoints の行 readability の reads[0] の doc",
            "行き先「nowhere」が一覧に無い",
        ],
    );
}

#[test]
fn ceiling_refute_weight_not_a_value_fails() {
    let w = Work::new("refute-weight");
    w.mutate("  refute: [止める]\n", "  refute: [止める, 消す]\n");
    assert_single_violation(
        &w.check(),
        "ceiling",
        &["weights の refute[1]", "行き先「消す」が一覧に無い"],
    );
}

#[test]
fn ceiling_finding_required_without_evidence_fails() {
    let w = Work::new("finding-required");
    w.mutate(
        "  required: [id, viewpoint, place, weight, evidence]",
        "  required: [id, viewpoint, place, weight]",
    );
    assert_single_violation(
        &w.check(),
        "ceiling",
        &["finding の required", "一覧", "（無い: evidence）"],
    );
}

#[test]
fn ceiling_record_required_without_bundle_fails() {
    let w = Work::new("record-required");
    w.mutate(
        "  required: [model, effort, at, read, bundle]",
        "  required: [model, effort, at, read]",
    );
    assert_single_violation(
        &w.check(),
        "ceiling",
        &["record の required", "一覧", "（無い: bundle）"],
    );
}

/// 行 id の重複。同じ id の行を足す（今在る行の id を書き替えると文書の集合も同時に変わる）。
#[test]
fn ceiling_duplicate_document_id_fails() {
    let w = Work::new("dup-id");
    w.mutate(
        "  - {id: ceiling, file: ceiling.yaml, note: この正本}\n",
        "  - {id: ceiling, file: ceiling.yaml, note: この正本}\n  - {id: ceiling, file: ceiling.yaml, note: 同じ正本}\n",
    );
    assert_single_violation(&w.check(), "重複キー", &["行 id「ceiling」が重複"]);
}

#[test]
fn ceiling_unknown_word_fails() {
    let w = Work::new("unknown-word");
    w.mutate(
        "    question: 入口から歩いて",
        "    question: 入口から zork 歩いて",
    );
    assert_single_violation(
        &w.check(),
        "ceiling",
        &[
            "viewpoints の行 readability の question",
            "語彙に無い英字の語「zork」",
        ],
    );
}

#[test]
fn ceiling_known_word_passes() {
    let w = Work::new("known-word");
    w.mutate(
        "    question: 入口から歩いて",
        "    question: 入口から yaml 歩いて",
    );
    assert_passes(&w.check());
}

#[test]
fn ceiling_viewpoints_not_a_list_of_rows_is_unknown() {
    let w = Work::new("viewpoints-type");
    let before = fs::read_to_string(w.ceiling()).unwrap();
    let start = before
        .find("\nviewpoints:\n")
        .expect("viewpoints の節が無い");
    let end = before.find("\nfinding:\n").expect("finding の節が無い");
    let after = format!("{}\nviewpoints: 観点{}", &before[..start], &before[end..]);
    fs::write(w.ceiling(), after).unwrap();
    assert_unknown(&w.check(), "ceiling.yaml: viewpoints が表の一覧でない");
}

#[test]
fn ceiling_duplicate_key_fails() {
    let w = Work::new("dup-key");
    w.mutate("  version: v0.1\n", "  version: v0.1\n  version: v0.1\n");
    assert_single_violation(
        &w.check(),
        "重複キー",
        &["同じ表にキー「version」を 2 度書いている"],
    );
}
