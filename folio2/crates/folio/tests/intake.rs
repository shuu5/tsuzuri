//! `folio check` の相談窓口の正本（intake.yaml）の検査の歯（便 18・docs/design/delivery-18.md §1 (d)）。
//! fixture は増やさず、歯の中で design-intent の写し全部を一時 dir に作り git init と 1 commit を行い
//! （版管理の無い写しは別の理由で「まだ分からない」になる）、intake.yaml に変異を 1 つ当てて `folio check --dir` を回す。
//! 違反の歯は、変異が 1 つなら違反の件数が 1 であること（出力の件数の表示）も確かめる（重複 id の歯だけ 2）。

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
        let root = std::env::temp_dir().join(format!("folio-intake-{case}-{}", std::process::id()));
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

    fn intake(&self) -> PathBuf {
        self.root.join("design-intent/intake.yaml")
    }

    /// intake.yaml の字面の変異（1 か所だけ）。
    fn mutate(&self, from: &str, to: &str) {
        let before = fs::read_to_string(self.intake()).unwrap();
        assert_eq!(
            before.matches(from).count(),
            1,
            "変異の当て先が 1 か所でない: {from:?}"
        );
        fs::write(self.intake(), before.replacen(from, to, 1)).unwrap();
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
    assert!(v[0].starts_with(&format!("[{kind}] intake.yaml")), "{v:?}");
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
fn intake_canonical_copy_passes() {
    let w = Work::new("canonical");
    assert_passes(&w.check());
}

#[test]
fn intake_missing_intake_is_unknown() {
    let w = Work::new("missing");
    fs::remove_file(w.intake()).unwrap();
    assert_unknown(&w.check(), "intake.yaml: 正本が無い");
}

#[test]
fn intake_unknown_section_fails() {
    let w = Work::new("unknown-section");
    let mut text = fs::read_to_string(w.intake()).unwrap();
    text.push_str("\nextra:\n  x: 1\n");
    fs::write(w.intake(), text).unwrap();
    assert_single_violation(&w.check(), "未知の節", &["未知の節「extra」"]);
}

#[test]
fn intake_empty_ask_fails() {
    let w = Work::new("empty-field");
    w.mutate(
        "ask: 何があっても守る線（憲法）を決めておきますか,",
        "ask: \"\",",
    );
    assert_single_violation(&w.check(), "欄の非空", &["questions の行 q1 の ask が空"]);
}

#[test]
fn intake_question_yes_not_a_target_fails() {
    let w = Work::new("yes-target");
    w.mutate("yes: [constitution]", "yes: [nowhere]");
    assert_single_violation(
        &w.check(),
        "intake",
        &[
            "questions の行 q1 の yes[0]",
            "行き先「nowhere」が一覧に無い",
        ],
    );
}

/// targets の id は 棚の documents か注入。棚に無い id の行を 1 つ足す（今在る行の id を書き替えると
/// その行を指す質問の yes も同時に解けなくなる＝変異 1 つで違反 2 件になる）。
#[test]
fn intake_target_id_not_on_the_shelf_fails() {
    let w = Work::new("target-id");
    w.mutate(
        "  - {id: inject, type:",
        "  - {id: nowhere, type: 注入, with: []}\n  - {id: inject, type:",
    );
    assert_single_violation(
        &w.check(),
        "intake",
        &[
            "targets の行 nowhere の id",
            "行き先「nowhere」が一覧に無い",
        ],
    );
}

#[test]
fn intake_target_with_not_an_annex_fails() {
    let w = Work::new("target-with");
    w.mutate("with: [vocabulary, rules]", "with: [vocabulary, nowhere]");
    assert_single_violation(
        &w.check(),
        "intake",
        &[
            "targets の行 constitution の with[1]",
            "行き先「nowhere」が一覧に無い",
        ],
    );
}

#[test]
fn intake_recommend_not_in_values_fails() {
    let w = Work::new("recommend");
    w.mutate(
        "を決めておきますか, recommend: はい,",
        "を決めておきますか, recommend: たぶん,",
    );
    assert_single_violation(
        &w.check(),
        "intake",
        &[
            "questions の行 q1 の recommend",
            "行き先「たぶん」が一覧に無い",
        ],
    );
}

#[test]
fn intake_over_the_question_limit_fails() {
    let w = Work::new("limit");
    w.mutate(
        "yes: [inject], no: []}\n",
        "yes: [inject], no: []}\n  - {id: q6, ask: 余った質問ですか, recommend: はい, why: 上限を超える。, yes: [], no: []}\n",
    );
    assert_single_violation(&w.check(), "intake", &["questions が FR1 の上限を超える"]);
}

/// 行 id の重複。6 本目の行なので上限にも触れる＝この歯だけ違反 2 件。
#[test]
fn intake_duplicate_question_id_fails() {
    let w = Work::new("dup-id");
    w.mutate(
        "yes: [inject], no: []}\n",
        "yes: [inject], no: []}\n  - {id: q1, ask: 同じ id の質問ですか, recommend: はい, why: 重複する。, yes: [], no: []}\n",
    );
    let out = w.check();
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}{}",
        stdout(&out),
        stderr(&out)
    );
    let v = violations(&out);
    assert_eq!(v.len(), 2, "{v:?}");
    assert!(
        v.iter()
            .any(|l| l.starts_with("[重複キー] intake.yaml") && l.contains("行 id「q1」が重複")),
        "{v:?}"
    );
    assert!(
        v.iter()
            .any(|l| l.starts_with("[intake] intake.yaml") && l.contains("上限")),
        "{v:?}"
    );
    assert!(
        stdout(&out).contains("folio check: 不合格（違反 2・"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn intake_unknown_word_fails() {
    let w = Work::new("unknown-word");
    w.mutate("why: 守る線が無いと、", "why: 守る線が無いと zzqx、");
    assert_single_violation(
        &w.check(),
        "intake",
        &["questions の行 q1 の why", "語彙に無い英字の語「zzqx」"],
    );
}

#[test]
fn intake_known_word_passes() {
    let w = Work::new("known-word");
    w.mutate("why: 守る線が無いと、", "why: 守る線が無いと folio、");
    assert_passes(&w.check());
}

#[test]
fn intake_questions_not_a_list_of_rows_is_unknown() {
    let w = Work::new("questions-type");
    let before = fs::read_to_string(w.intake()).unwrap();
    let start = before.find("\nquestions:\n").expect("questions の節が無い");
    let end = before.find("\nsheet:\n").expect("sheet の節が無い");
    let after = format!("{}\nquestions: 質問{}", &before[..start], &before[end..]);
    fs::write(w.intake(), after).unwrap();
    assert_unknown(&w.check(), "intake.yaml: questions が表の一覧でない");
}

#[test]
fn intake_duplicate_key_fails() {
    let w = Work::new("dup-key");
    w.mutate("  version: v0.1\n", "  version: v0.1\n  version: v0.1\n");
    assert_single_violation(
        &w.check(),
        "重複キー",
        &["同じ表にキー「version」を 2 度書いている"],
    );
}
