//! `folio check` の入口の面の正本（index.yaml）の検査の歯（便 12・docs/design/delivery-12.md §1 (d)）。
//! fixture は増やさず、歯の中で design-intent の写し全部を一時 dir に作り git init と 1 commit を行い
//! （版管理の無い写しは別の理由で「まだ分からない」になる）、index.yaml に変異を 1 つ当てて `folio check --dir` を回す。
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
            std::env::temp_dir().join(format!("folio-entrance-{case}-{}", std::process::id()));
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

    fn index(&self) -> PathBuf {
        self.root.join("design-intent/index.yaml")
    }

    /// index.yaml の字面の変異（1 か所だけ）。
    fn mutate(&self, from: &str, to: &str) {
        let before = fs::read_to_string(self.index()).unwrap();
        assert_eq!(
            before.matches(from).count(),
            1,
            "変異の当て先が 1 か所でない: {from:?}"
        );
        fs::write(self.index(), before.replacen(from, to, 1)).unwrap();
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
    assert!(v[0].starts_with(&format!("[{kind}] index.yaml")), "{v:?}");
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
fn entrance_canonical_copy_passes() {
    let w = Work::new("canonical");
    assert_passes(&w.check());
}

#[test]
fn entrance_missing_index_is_unknown() {
    let w = Work::new("missing");
    fs::remove_file(w.index()).unwrap();
    assert_unknown(&w.check(), "index.yaml: 正本が無い");
}

#[test]
fn entrance_unknown_section_fails() {
    let w = Work::new("unknown-section");
    let mut text = fs::read_to_string(w.index()).unwrap();
    text.push_str("\nextra:\n  x: 1\n");
    fs::write(w.index(), text).unwrap();
    assert_single_violation(&w.check(), "未知の節", &["未知の節「extra」"]);
}

#[test]
fn entrance_empty_field_fails() {
    let w = Work::new("empty-field");
    w.mutate(
        "  text: 設計文書の持ち主（非エンジニア）と、その文書を読んで作業する AI。両方が同じページを読む。\n",
        "  text: \"\"\n",
    );
    assert_single_violation(&w.check(), "欄の非空", &["audience の text が空"]);
}

#[test]
fn entrance_stop_doc_not_on_shelf_fails() {
    let w = Work::new("stop-doc");
    w.mutate(
        "        - {doc: constitution, at: s0, label: 目指すことと誰のため}",
        "        - {doc: nowhere, at: s0, label: 目指すことと誰のため}",
    );
    assert_single_violation(
        &w.check(),
        "index",
        &["lanes の行 first", "行き先「nowhere」が棚に無い"],
    );
}

#[test]
fn entrance_relation_to_not_on_shelf_fails() {
    let w = Work::new("relation-to");
    w.mutate(
        "    - {id: binds, from: constitution, to: srs,",
        "    - {id: binds, from: constitution, to: nowhere,",
    );
    assert_single_violation(
        &w.check(),
        "index",
        &["shelf.relations の行 binds", "行き先「nowhere」が棚に無い"],
    );
}

#[test]
fn entrance_duplicate_document_id_fails() {
    let w = Work::new("dup-id");
    w.mutate(
        "  annexes:\n",
        "    - {id: srs, type: 要件書, use: 同じ行をもう 1 つ。, absent: null}\n  annexes:\n",
    );
    assert_single_violation(&w.check(), "重複キー", &["行 id「srs」が重複"]);
}

#[test]
fn entrance_unknown_word_fails() {
    let w = Work::new("unknown-word");
    w.mutate("  explain: この棚には、", "  explain: この棚には zzqx、");
    assert_single_violation(
        &w.check(),
        "index",
        &["shelf の explain", "語彙に無い英字の語「zzqx」"],
    );
}

#[test]
fn entrance_known_word_passes() {
    let w = Work::new("known-word");
    w.mutate("  explain: この棚には、", "  explain: この棚には folio、");
    assert_passes(&w.check());
}

#[test]
fn entrance_minutes_not_integer_fails() {
    let w = Work::new("minutes");
    w.mutate("      minutes: 10\n", "      minutes: 十\n");
    assert_single_violation(
        &w.check(),
        "index",
        &["lanes の行 first", "minutes が 1 以上の整数でない"],
    );
}

#[test]
fn entrance_lanes_not_a_map_is_unknown() {
    let w = Work::new("lanes-type");
    let before = fs::read_to_string(w.index()).unwrap();
    let start = before.find("\nlanes:\n").expect("lanes の節が無い");
    let end = before.find("\nintake:\n").expect("intake の節が無い");
    let after = format!("{}\nlanes: 読み方{}", &before[..start], &before[end..]);
    fs::write(w.index(), after).unwrap();
    assert_unknown(&w.check(), "index.yaml: lanes が欄の表でない");
}

#[test]
fn entrance_duplicate_key_fails() {
    let w = Work::new("dup-key");
    w.mutate(
        "  title: このプロジェクトの文書\n",
        "  title: このプロジェクトの文書\n  title: このプロジェクトの文書\n",
    );
    assert_single_violation(
        &w.check(),
        "重複キー",
        &["同じ表にキー「title」を 2 度書いている"],
    );
}

/// 便 76 §1 (d)(h)7: 生成区間の節 schema は未知の節にならない。床は生成区間の中身を床の木と突き合わせないので、
/// 写しから生成区間の節だけを消しても床の結果は同じ。
#[test]
fn f76_entrance_accepts_the_generated_region() {
    let w = Work::new("generated-region");
    let text = fs::read_to_string(w.index()).unwrap();
    assert!(
        text.contains("\nschema:\n"),
        "実の入口の正本に生成区間の節が無い"
    );
    let with_region = w.check();
    assert_passes(&with_region);

    let begin = text.find("\nschema:\n").unwrap() + 1;
    let end = text.find("\n# folio:schema:end\n").unwrap() + 1;
    fs::write(w.index(), format!("{}{}", &text[..begin], &text[end..])).unwrap();
    assert!(!fs::read_to_string(w.index()).unwrap().contains("\nschema:"));
    let without_region = w.check();
    assert_passes(&without_region);
    assert_eq!(without_region.status.code(), with_region.status.code());
    assert_eq!(stdout(&without_region), stdout(&with_region));
}
