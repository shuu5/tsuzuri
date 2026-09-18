//! `folio intake` の歯（便 19・docs/design/delivery-19.md §1 (c)）。binary 経由。
//! - AC1（凍結 anchor・P-10.1）: 固定の回答 5 つ／回答なし の支度表が tests/fixtures/intake/ の期待と byte 一致
//! - FR8（差分）: 既に在る支度表の answered の回答は引き継ぎ、問うのは答えの無い質問だけ（推奨で進めた行は引き継がない）
//! - `--print` の 3 形（支度表なし・差分・全部答え済み）と、file を書かないこと
//! - 導出できない 6 つ（どれも 2・支度表は出来ない／変わらない）・出力先の親 dir が無い・旗の使い方の誤り
//!
//! 入力は版管理の `design-intent/` を丸ごと一時 dir へ写したもの（支度表は写しの中にだけ生まれる）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(name: &str) -> PathBuf {
    repo_root().join("tests/fixtures/intake").join(name)
}

const SHEET: &str = "intake-sheet.yaml";

/// `design-intent/` の写しを持つ一時 dir を作る（前の回の残りは消す）。
fn work(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-sheet-{case}"));
    let _ = fs::remove_dir_all(&td);
    copy_tree(&repo_root().join("design-intent"), &td);
    // 実の正本の支度表（持ち主の裁定 2026-09-18「aで」）は写しから外し、歯は支度表なしから始める
    let _ = fs::remove_file(td.join(SHEET));
    td
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

fn folio_intake(dir: &Path, answers: Option<&Path>, mode: &str) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_folio"));
    cmd.arg("intake").arg("--dir").arg(dir);
    if let Some(path) = answers {
        cmd.arg("--answers").arg(path);
    }
    cmd.arg(mode).output().expect("folio を起動できない")
}

fn show(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn lines(out: &Output) -> Vec<String> {
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// 支度表を書き、`code` で終わったことを確かめる。
fn write_sheet(dir: &Path, answers: Option<&Path>) -> Output {
    let out = folio_intake(dir, answers, "--write");
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    out
}

/// 書いた支度表が凍結 anchor と byte 一致すること（P-10.1）。
fn assert_frozen(got: &Path, expected: &str) {
    let got = fs::read(got).unwrap();
    let expected = fixture(expected);
    assert_eq!(
        got,
        fs::read(&expected).unwrap(),
        "支度表が凍結 anchor {} と違う",
        expected.display()
    );
}

/// 一時 dir に回答の file を置く（file 名を返す＝`--dir` からの相対）。
fn put_answers(dir: &Path, name: &str, body: &str) -> PathBuf {
    fs::write(dir.join(name), body).unwrap();
    PathBuf::from(name)
}

/// 正本の 1 か所を書き換える（変異が当たらなければ歯の側の誤り）。
fn mutate(dir: &Path, name: &str, from: &str, to: &str) {
    let path = dir.join(name);
    let text = fs::read_to_string(&path).unwrap();
    let after = text.replacen(from, to, 1);
    assert_ne!(text, after, "{name}: 変異「{from}」が当たらない");
    fs::write(&path, after).unwrap();
}

// ── AC1（凍結 anchor）──

#[test]
fn sheet_write_matches_the_frozen_anchor() {
    let dir = work("anchor-answered");
    let out = write_sheet(&dir, Some(&fixture("answers-5.yaml")));
    let line = &lines(&out)[0];
    assert!(line.contains("持つ文書 3・推奨で進めた項目 0"), "{line}");
    assert_frozen(&dir.join(SHEET), "expected-sheet.yaml");
}

#[test]
fn sheet_write_without_answers_proceeds_on_the_recommendation() {
    let dir = work("anchor-recommended");
    let out = write_sheet(&dir, None);
    let line = &lines(&out)[0];
    assert!(line.contains("持つ文書 5・推奨で進めた項目 5"), "{line}");
    assert_frozen(&dir.join(SHEET), "expected-sheet-recommended.yaml");
}

// ── FR8（差分と引き継ぎ）──

#[test]
fn sheet_carries_the_answered_rows_and_asks_only_the_rest() {
    let dir = work("follow-up");
    write_sheet(&dir, Some(&fixture("answers-3.yaml")));

    // 推奨で進めた q2・q5 だけをまた問う（q1 は答え済みなので出さない）
    let out = folio_intake(&dir, None, "--print");
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    let l = lines(&out);
    assert_eq!(l.len(), 3, "{l:?}");
    assert!(l[0].contains("質問 2 つ"), "{l:?}");
    assert!(l[1].starts_with("q2. "), "{l:?}");
    assert!(l[2].starts_with("q5. "), "{l:?}");
    assert!(
        !l.iter().any(|line| line.contains("何があっても守る線")),
        "答え済みの q1 を問うている: {l:?}"
    );

    // 残りの 2 つに答えると、5 つ全部に答えた支度表と同じものが出来る（引き継ぎ + 上書き）
    let rest = put_answers(&dir, "rest.yaml", "answers:\n  q2: いいえ\n  q5: いいえ\n");
    write_sheet(&dir, Some(&rest));
    assert_frozen(&dir.join(SHEET), "expected-sheet.yaml");
}

// ── --print の 3 形 ──

#[test]
fn sheet_print_lists_every_question_and_writes_nothing() {
    let dir = work("print-all");
    let out = folio_intake(&dir, None, "--print");
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    let l = lines(&out);
    assert_eq!(l.len(), 6, "{l:?}");
    assert!(l[0].contains("質問 5 つ"), "{l:?}");
    for (i, line) in l[1..].iter().enumerate() {
        assert!(line.starts_with(&format!("q{}. ", i + 1)), "{line}");
        assert!(line.contains("（おすすめ: はい）— "), "{line}");
    }
    assert!(
        l[1].contains("何があっても守る線（憲法）を決めておきますか"),
        "{l:?}"
    );
    assert!(!dir.join(SHEET).exists(), "--print が支度表を書いた");
}

#[test]
fn sheet_print_has_nothing_to_ask_when_every_question_is_answered() {
    let dir = work("print-none");
    write_sheet(&dir, Some(&fixture("answers-5.yaml")));
    let before = fs::read(dir.join(SHEET)).unwrap();
    let out = folio_intake(&dir, None, "--print");
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    let l = lines(&out);
    assert_eq!(l.len(), 1, "{l:?}");
    assert!(l[0].contains("問う質問は無い"), "{l:?}");
    assert_eq!(fs::read(dir.join(SHEET)).unwrap(), before);
}

// ── 導出できない（2・支度表は出来ない／変わらない）──

/// 支度表の無い写しで `--write` が 2 に倒れ、支度表が 1 byte も出来ないこと。
fn assert_sheet_unknown(case: &str, prepare: impl FnOnce(&Path) -> Option<PathBuf>) {
    let dir = work(case);
    let answers = prepare(&dir);
    let out = folio_intake(&dir, answers.as_deref(), "--write");
    assert_eq!(out.status.code(), Some(2), "{case}: {}", show(&out));
    assert!(
        stderr(&out).starts_with("folio intake: まだ分からない: "),
        "{case}: {}",
        stderr(&out)
    );
    assert!(!dir.join(SHEET).exists(), "{case}: 支度表が出来ている");
}

#[test]
fn sheet_unknown_inputs_do_not_make_a_sheet() {
    assert_sheet_unknown("bad-id", |dir| {
        Some(put_answers(dir, "a.yaml", "answers:\n  q9: はい\n"))
    });
    assert_sheet_unknown("bad-value", |dir| {
        Some(put_answers(dir, "a.yaml", "answers:\n  q1: たぶん\n"))
    });
    assert_sheet_unknown("bad-shape", |dir| {
        Some(put_answers(dir, "a.yaml", "- q1: はい\n"))
    });
    assert_sheet_unknown("bad-recommend", |dir| {
        mutate(
            dir,
            "intake.yaml",
            "recommend: はい, why: 守る線が無い",
            "recommend: たぶん, why: 守る線が無い",
        );
        None
    });
    assert_sheet_unknown("bad-target", |dir| {
        mutate(dir, "intake.yaml", "yes: [constitution]", "yes: [nope]");
        None
    });
}

/// 既に在る支度表が読めない形なら 2 で、その支度表は書き換えない。
#[test]
fn sheet_unknown_existing_sheet_is_left_alone() {
    let dir = work("bad-existing-sheet");
    write_sheet(&dir, Some(&fixture("answers-5.yaml")));
    mutate(&dir, SHEET, "\"value\": \"はい\"", "\"value\": \"たぶん\"");
    let before = fs::read(dir.join(SHEET)).unwrap();
    let out = folio_intake(&dir, None, "--write");
    assert_eq!(out.status.code(), Some(2), "{}", show(&out));
    assert!(stderr(&out).contains("まだ分からない"), "{}", stderr(&out));
    assert_eq!(
        fs::read(dir.join(SHEET)).unwrap(),
        before,
        "支度表が変わった"
    );
}

// ── 出力先と旗 ──

#[test]
fn sheet_write_needs_the_parent_dir_of_the_output() {
    // 正本の置き場そのものが無い
    let missing = std::env::temp_dir().join("folio-sheet-no-such-dir");
    let _ = fs::remove_dir_all(&missing);
    let out = folio_intake(&missing, None, "--write");
    assert_eq!(out.status.code(), Some(2), "{}", show(&out));

    // 支度表の置き場（sheet の file の親）が無い
    let dir = work("no-parent");
    mutate(
        &dir,
        "intake.yaml",
        "file: intake-sheet.yaml",
        "file: nodir/intake-sheet.yaml",
    );
    let out = folio_intake(&dir, None, "--write");
    assert_eq!(out.status.code(), Some(2), "{}", show(&out));
    assert!(
        stderr(&out).contains("出力先の親 dir が無い"),
        "{}",
        stderr(&out)
    );
    assert!(!dir.join("nodir").exists());
}

#[test]
fn sheet_needs_exactly_one_of_print_and_write() {
    let dir = work("flags");
    let both = Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(["intake", "--print", "--write"])
        .output()
        .expect("folio を起動できない");
    assert_eq!(both.status.code(), Some(2), "{}", show(&both));
    let neither = Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(["intake", "--dir"])
        .arg(&dir)
        .output()
        .expect("folio を起動できない");
    assert_eq!(neither.status.code(), Some(2), "{}", show(&neither));
    assert!(!dir.join(SHEET).exists());
}
