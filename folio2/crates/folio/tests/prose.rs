//! 設計ノートの散文の門（FR12・rules 行 R-16）の歯（便 24・docs/design/delivery-24.md §1 (g)）。
//! 入力は便 23 の歯と同じ作り＝design-intent の写し全部を一時 dir へ、器（scribe2）の導出 file を写しの根の
//! `contracts/` へ置き、git init と 1 commit を行ってから、設計ノートを置く・変異を 1 つ当てて
//! `folio check --dir` を回す。
//! 凍結 anchor は要件書 AC10 の red_test が名指す `tests/fixtures/prose-gate/` の 2 本（手書き）。
//! 式の枝は `clean.yaml` の写しの散文 1 行（「枝の当て先」の文）に変異を 1 つ当てて数える。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// `clean.yaml` の散文の中の、式の枝が変異を当てる行（前後の行は変えない）。
const ANCHOR: &str = "      枝の当て先（この文は記述の文）。\n";

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
        let root = std::env::temp_dir().join(format!("folio-prose-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_tree(
            &repo_root().join("design-intent"),
            &root.join("design-intent"),
        );
        fs::create_dir_all(root.join("contracts")).unwrap();
        fs::copy(
            repo_root().join("contracts/schema.toml"),
            root.join("contracts/schema.toml"),
        )
        .unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-q", "-m", "fixture"]);
        Work { root }
    }

    fn dir(&self) -> PathBuf {
        self.root.join("design-intent")
    }

    /// 設計ノートの置き場の file。
    fn note(&self, name: &str) -> PathBuf {
        self.dir().join("design-note").join(name)
    }

    fn rules(&self) -> PathBuf {
        self.dir().join("rules.yaml")
    }

    /// 凍結 fixture を設計ノートの置き場へ置く。
    fn put(&self, file: &str) {
        let from = repo_root().join("tests/fixtures/prose-gate").join(file);
        fs::copy(&from, self.note(file))
            .unwrap_or_else(|e| panic!("{}: 置けない: {e}", from.display()));
    }

    /// file の字面の変異（1 か所だけ）。
    fn mutate_file(&self, path: &Path, from: &str, to: &str) {
        let before = fs::read_to_string(path).unwrap();
        assert_eq!(
            before.matches(from).count(),
            1,
            "変異の当て先が 1 か所でない: {from:?}"
        );
        fs::write(path, before.replacen(from, to, 1)).unwrap();
    }

    /// rules 行 R-16 の value（型付きデータ）を文字列に替える＝門の一覧が読めない。
    fn blunt_r16(&self) {
        let path = self.rules();
        let text = fs::read_to_string(&path).unwrap();
        let start = text.find(", value: {marks:").expect("R-16 の value が無い");
        let end = start + text[start..].find(", kind:").expect("R-16 の kind が無い");
        fs::write(
            &path,
            format!("{}, value: 型付きデータ{}", &text[..start], &text[end..]),
        )
        .unwrap();
    }

    fn check(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("check")
            .arg("--dir")
            .arg(self.dir())
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

/// 不合格 1・違反はちょうど 1 件（件数の表示も 1）・その 1 件が種別 prose-gate で `words` を全部含む。
fn assert_single_gate_violation(out: &Output, words: &[&str]) {
    assert_eq!(out.status.code(), Some(1), "{}{}", stdout(out), stderr(out));
    let v = violations(out);
    assert_eq!(v.len(), 1, "違反は変異の 1 件だけのはず: {v:?}");
    assert!(
        v[0].starts_with("[prose-gate] design-note/clean.yaml: 節 1 行 "),
        "{v:?}"
    );
    for w in words {
        assert!(v[0].contains(w), "「{w}」が無い: {v:?}");
    }
    assert!(
        stdout(out).contains("folio check: 不合格（違反 1・"),
        "{}",
        stdout(out)
    );
}

/// `clean.yaml` の写しの散文 1 行に変異を当てて回す。
fn branch(case: &str, to: &str) -> Output {
    let w = Work::new(case);
    w.put("clean.yaml");
    w.mutate_file(&w.note("clean.yaml"), ANCHOR, to);
    w.check()
}

// ── AC10（凍結 anchor） ──

#[test]
fn prose_gate_violations_fixture_counts_two() {
    let w = Work::new("violations");
    w.put("violations.yaml");
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
            .all(|l| l.starts_with("[prose-gate] design-note/violations.yaml: 節 1 行 ")),
        "{v:?}"
    );
    assert!(
        v.iter()
            .any(|l| l
                .contains("行 2: no-pointer 生成器は検査を通らなかった図を上書きしてはならない。")),
        "{v:?}"
    );
    assert!(
        v.iter().any(|l| l.contains(
            "行 3: number-with-unit 検査は 30 秒 のうちに完了しなければならない（FR12）。"
        )),
        "{v:?}"
    );
    // 記述の文（1 行目・4 行目）は 1 件も出ない
    assert!(!stdout(&out).contains("散文の門は規範の印"), "{v:?}");
    assert!(
        !stdout(&out).contains("記述の文は数と単位を自由に"),
        "{v:?}"
    );
}

#[test]
fn prose_gate_clean_fixture_passes() {
    let w = Work::new("clean");
    w.put("clean.yaml");
    assert_passes(&w.check());
}

// ── 式の枝（clean.yaml の写しに変異 1 つずつ） ──

#[test]
fn prose_gate_counts_a_prohibition_before_a_clause_end() {
    let out = branch("prohibition", "      複製は禁止。\n");
    assert_single_gate_violation(&out, &["no-pointer", "複製は禁止。"]);
}

#[test]
fn prose_gate_ignores_a_prohibition_inside_a_word() {
    assert_passes(&branch(
        "prohibition-word",
        "      禁止事項の一覧を持つ。\n",
    ));
}

#[test]
fn prose_gate_accepts_an_english_mark_with_a_pointer() {
    assert_passes(&branch(
        "shall-not",
        "      The tool SHALL NOT overwrite the figure（FR12）。\n",
    ));
}

#[test]
fn prose_gate_counts_an_english_mark_without_a_pointer() {
    let out = branch(
        "shall-not-bare",
        "      The tool SHALL NOT overwrite the figure。\n",
    );
    assert_single_gate_violation(&out, &["no-pointer", "The tool SHALL NOT overwrite"]);
}

#[test]
fn prose_gate_ignores_an_adverb_without_a_mark() {
    assert_passes(&branch("adverb", "      生成器は必ず検査を通す。\n"));
}

#[test]
fn prose_gate_ignores_a_number_before_a_longer_english_word() {
    assert_passes(&branch(
        "skills",
        "      道具は 4 skills を一覧してはならない（FR12）。\n",
    ));
}

#[test]
fn prose_gate_counts_a_number_before_an_english_unit() {
    let out = branch(
        "seconds",
        "      道具は 4 s を一覧してはならない（FR12）。\n",
    );
    assert_single_gate_violation(
        &out,
        &["number-with-unit", "道具は 4 s を一覧してはならない"],
    );
}

#[test]
fn prose_gate_counts_a_percent_without_a_space() {
    let out = branch(
        "percent",
        "      道具は 100% を超えて一覧してはならない（FR12）。\n",
    );
    assert_single_gate_violation(&out, &["number-with-unit", "道具は 100% を超えて"]);
}

#[test]
fn prose_gate_breaks_a_sentence_at_the_line_end() {
    let out = branch(
        "line-end",
        "      生成器は前の生成物を上書きしてはならない\n      （FR12）。\n",
    );
    assert_single_gate_violation(
        &out,
        &[
            "行 2: no-pointer",
            "生成器は前の生成物を上書きしてはならない",
        ],
    );
}

#[test]
fn prose_gate_ignores_a_pointer_inside_an_identifier() {
    let out = branch(
        "identifier",
        "      生成器は xFR12y を書き出してはならない。\n",
    );
    assert_single_gate_violation(&out, &["no-pointer", "生成器は xFR12y を"]);
}

#[test]
fn prose_gate_accepts_a_contract_id_of_the_same_document() {
    assert_passes(&branch(
        "contract-id",
        "      契約 clean#a のとおり生成器は図を書き出してはならない。\n",
    ));
}

// ── まだ分からない（R-16 の value が読めない） ──

#[test]
fn prose_gate_without_a_readable_rules_row_is_unknown() {
    let w = Work::new("blunt");
    w.put("clean.yaml");
    w.blunt_r16();
    let out = w.check();
    assert_eq!(
        out.status.code(),
        Some(2),
        "{}{}",
        stdout(&out),
        stderr(&out)
    );
    assert!(violations(&out).is_empty(), "{:?}", violations(&out));
    assert!(
        stderr(&out)
            .lines()
            .any(|l| l.starts_with("# まだ分からない: ") && l.contains("R-16 の value が読めない")),
        "{}",
        stderr(&out)
    );
    assert!(
        !stdout(&out).contains("folio check: 合格"),
        "{}",
        stdout(&out)
    );
}
