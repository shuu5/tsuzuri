//! `folio intake` の歯（便 19・docs/design/delivery-19.md §1 (c)）。binary 経由。
//! - AC1（凍結 anchor・P-10.1）: 固定の回答 5 つ／回答なし の支度表が tests/fixtures/intake/ の期待と byte 一致
//! - FR8（差分）: 既に在る支度表の answered の回答は引き継ぎ、問うのは答えの無い質問だけ（推奨で進めた行は引き継がない）
//! - `--print` の 3 形（支度表なし・差分・全部答え済み）と、file を書かないこと
//! - 導出できない 6 つ（どれも 2・支度表は出来ない／変わらない）・出力先の親 dir が無い・旗の使い方の誤り
//! - 便 160（docs/design/delivery-160.md §1 (c)）: 別の process の同じ歯が、この process の写しを消さない
//! - 便 197（docs/design/delivery-197.md §1 (c)）: 回答の値は intake.yaml の answers の values から読む（1 つ目 = yes・2 つ目 = no・
//!   2 つでなければ支度表を書かない）・実の生成区間はその写しを持つ・src は回答の値の字を持たない。
//!   同乗（台帳 f2-648.227）: 実の rules.yaml の生成区間の除外は人の作業の時間と AI の費用だけ（歯を置く file を増やさないためここに置く）
//!
//! 入力は版管理の `design-intent/` を丸ごと一時 dir へ写したもの（支度表は写しの中にだけ生まれる）。
#![cfg(test)]

use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::common::{copy_tree, repo_root, stderr};
use folio::yaml_rust2::YamlLoader;

fn fixture(name: &str) -> PathBuf {
    repo_root().join("tests/fixtures/intake").join(name)
}

const SHEET: &str = "intake-sheet.yaml";

/// `design-intent/` の写しを持つ一時 dir（歯の終わりに消す）。
struct Work {
    root: PathBuf,
}

impl Deref for Work {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.root
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// `design-intent/` の写しを持つ一時 dir を作る（名に process の id・前の回の残りは消す）。
fn work(case: &str) -> Work {
    let td = std::env::temp_dir().join(format!("folio-sheet-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    copy_tree(&repo_root().join("design-intent"), &td);
    // 実の正本の支度表（持ち主の裁定 2026-09-18「aで」）は写しから外し、歯は支度表なしから始める
    let _ = fs::remove_file(td.join(SHEET));
    Work { root: td }
}

fn folio_intake(dir: &Path, answers: Option<&Path>, mode: &str) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tz"));
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
    let both = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["intake", "--print", "--write"])
        .output()
        .expect("folio を起動できない");
    assert_eq!(both.status.code(), Some(2), "{}", show(&both));
    let neither = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["intake", "--dir"])
        .arg(&*dir)
        .output()
        .expect("folio を起動できない");
    assert_eq!(neither.status.code(), Some(2), "{}", show(&neither));
    assert!(!dir.join(SHEET).exists());
}

// ── 便 160（別の process の写し）──

/// 子の役の印。値が親の process の id のときだけ子として走る。
const F160_CHILD: &str = "FOLIO_F160_CHILD";
const F160_NAME: &str = "sheet::f160_another_process_leaves_this_copy_alone";
/// 子が標準エラーに出す path の行の頭。
const F160_LINE: &str = "f160-child-path: ";

#[test]
fn f160_another_process_leaves_this_copy_alone() {
    let parent = std::os::unix::process::parent_id().to_string();
    if std::env::var(F160_CHILD).ok().as_deref() == Some(parent.as_str()) {
        // 子: 同じ host の別の写しで同じ歯が走るのと同じ形で、同じ case の写しを作る
        let dir = work("f160-same");
        eprintln!("{F160_LINE}{}", dir.display());
        return;
    }

    let dir = work("f160-same");
    let mark = dir.join("f160-mark");
    fs::write(&mark, "parent").unwrap();
    let other = work("f160-other");
    assert_ne!(&*dir, &*other, "同じ process の別の case と同じ path");

    let out = Command::new(std::env::current_exe().unwrap())
        .args([F160_NAME, "--exact", "--nocapture"])
        .env(F160_CHILD, std::process::id().to_string())
        .output()
        .expect("test binary を撃ち直せない");
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    let err = stderr(&out);
    let paths: Vec<&str> = err
        .lines()
        .filter_map(|line| line.strip_prefix(F160_LINE))
        .collect();
    assert_eq!(paths.len(), 1, "子が path を 1 行出さない: {}", show(&out));
    let child = Path::new(paths[0]);
    assert_ne!(child, &*dir, "子の写しが親と同じ path");
    assert!(mark.exists(), "子が親の写しを消した");
    assert!(!child.exists(), "子の写しが子の終わりに残った: {}", child.display());
}

// ── 便 197（回答の値は file から読む）──

/// 写しの正本の字を全部書き換える（当たる数が `count` でなければ歯の側の誤り）。
fn f197_replace_all(dir: &Path, name: &str, from: &str, to: &str, count: usize) {
    let path = dir.join(name);
    let text = fs::read_to_string(&path).unwrap();
    assert_eq!(text.matches(from).count(), count, "{name}: 変異「{from}」の数");
    fs::write(&path, text.replace(from, to)).unwrap();
}

/// 歯 1: values を ["yes", "no"] にした写しでも、1 つ目の値の回答は yes の行き先を・2 つ目の値の回答は no の行き先を選ぶ。
#[test]
fn f197_answer_words_come_from_the_values_in_the_file() {
    let dir = work("f197-words");
    mutate(&dir, "intake.yaml", "values: [はい, いいえ]", "values: [\"yes\", \"no\"]");
    f197_replace_all(&dir, "intake.yaml", "recommend: はい,", "recommend: \"yes\",", 5);
    // q1 だけ 2 つ目の値で答える（q1 の no の行き先は空＝憲法を持たない）・残り 4 つは推奨（1 つ目の値）で進む
    let answers = put_answers(&dir, "a.yaml", "answers:\n  q1: \"no\"\n");
    let out = write_sheet(&dir, Some(&answers));
    let line = &lines(&out)[0];
    assert!(line.contains("持つ文書 4・推奨で進めた項目 4"), "{line}");
    let sheet = fs::read_to_string(dir.join(SHEET)).unwrap();
    for id in ["srs", "adr", "design-note", "inject"] {
        assert!(sheet.contains(&format!("\"id\": \"{id}\"")), "{id}: {sheet}");
    }
    assert!(!sheet.contains("\"id\": \"constitution\""), "{sheet}");
    assert!(sheet.contains("\"value\": \"no\""), "{sheet}");
    assert!(!sheet.contains("はい") && !sheet.contains("いいえ"), "{sheet}");
}

/// 歯 2: values が 2 つでない写し（3 つ・1 つ）で `folio intake --write` は 2 に倒れ、支度表を 1 byte も書かない
/// （無ければ出来ない・在れば変わらない）。
#[test]
fn f197_values_other_than_two_make_no_sheet() {
    for (case, values, n, prior) in [
        ("f197-three", "values: [はい, いいえ, たぶん]", 3, false),
        ("f197-three-prior", "values: [はい, いいえ, たぶん]", 3, true),
        ("f197-one", "values: [はい]", 1, false),
        ("f197-one-prior", "values: [はい]", 1, true),
    ] {
        let dir = work(case);
        if prior {
            write_sheet(&dir, None);
        }
        let before = fs::read(dir.join(SHEET)).ok();
        mutate(&dir, "intake.yaml", "values: [はい, いいえ]", values);
        let out = folio_intake(&dir, None, "--write");
        assert_eq!(out.status.code(), Some(2), "{case}: {}", show(&out));
        let err = stderr(&out);
        assert!(err.starts_with("folio intake: まだ分からない: "), "{case}: {err}");
        assert!(err.contains(&format!("answers の values が {n} つ")), "{case}: {err}");
        assert!(out.stdout.is_empty(), "{case}: {}", show(&out));
        assert_eq!(fs::read(dir.join(SHEET)).ok(), before, "{case}: 支度表が出来た／変わった");
    }
}

/// 生成区間の印の間の字（実の正本の file から）。
fn f197_region(file: &str) -> String {
    let text = fs::read_to_string(repo_root().join("design-intent").join(file)).unwrap();
    text.lines()
        .skip_while(|l| !l.starts_with("# folio:schema:begin"))
        .skip(1)
        .take_while(|l| !l.starts_with("# folio:schema:end"))
        .map(|l| format!("{l}\n"))
        .collect()
}

/// 字の一覧（一覧でない・字でない項は落とす）。
fn f197_strs(y: &folio::yaml_rust2::Yaml) -> Vec<String> {
    y.as_vec()
        .map(|v| v.iter().filter_map(|x| x.as_str()).map(str::to_string).collect())
        .unwrap_or_default()
}

/// 歯 3: 実の intake.yaml の生成区間は、回答の値の読み方（値の数・行き先の欄の順・default の固定の値）と行き先の固定の値の
/// 写しを持ち、人が書く欄（values の数・default・各質問の行の欄）と食い違わない。期待の値は歯の中の手書き。
#[test]
fn f197_real_region_copies_the_answer_rule() {
    let text = fs::read_to_string(repo_root().join("design-intent/intake.yaml")).unwrap();
    let region = f197_region("intake.yaml");
    let docs = YamlLoader::load_from_str(&region).unwrap();
    let schema = &docs[0]["schema"];
    let strs = f197_strs;
    let answers = &schema["answers"];
    assert_eq!(answers["values_count"].as_i64(), Some(2), "{region}");
    assert_eq!(strs(&answers["branches"]), ["yes", "no"], "{region}");
    assert_eq!(answers["default_fixed"].as_str(), Some("recommend"), "{region}");
    assert_eq!(strs(&schema["targets"]["fixed"]), ["inject"], "{region}");
    for note in ["answers_note", "targets_note"] {
        let s = schema[note].as_str().unwrap_or_default();
        assert!(s.contains("この節はその写しである"), "{note}: {region}");
    }

    let whole = &YamlLoader::load_from_str(&text).unwrap()[0];
    assert_eq!(strs(&whole["answers"]["values"]).len(), 2);
    assert_eq!(whole["answers"]["default"].as_str(), Some("recommend"));
    let rows = whole["questions"].as_vec().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        for branch in ["yes", "no"] {
            assert!(row[branch].as_vec().is_some(), "質問の行に欄 {branch} が無い: {row:?}");
        }
    }
}

/// 歯 4: crates/folio/src/ の各 file の `#[cfg(test)]` より前に、引用符付きの回答の値の字（"はい"・"いいえ"）が無い
/// （値の言葉の正本は intake.yaml の answers の values・実装は持たない）。
#[test]
fn f197_no_source_file_spells_the_answer_words() {
    let src = repo_root().join("crates/folio/src");
    let mut scanned = Vec::new();
    let mut hits = Vec::new();
    for entry in fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let text = fs::read_to_string(&path).unwrap();
        let head = text.split("#[cfg(test)]").next().unwrap_or_default();
        for word in ["\"はい\"", "\"いいえ\""] {
            if head.contains(word) {
                hits.push(format!("{name}: {word}"));
            }
        }
        scanned.push(name);
    }
    assert!(scanned.iter().any(|n| n == "sheet.rs"), "{scanned:?}");
    assert!(hits.is_empty(), "回答の値の字を持つ: {hits:?}");
}

/// 歯 6（検証役の非 blocking N1・変異 V3）: 相談窓口の生成区間の写しの 4 欄は実装の定数を引く。crates/folio/src/ の各 file の
/// `#[cfg(test)]` より前で、行き先の固定の値の字 "inject" と回答の欄の一覧の字 ["yes", "no"] は intake.rs に 1 回ずつだけ在り
/// （空白を除いて数える＝定数の宣言だけ）、床の木 INTAKE_FLOOR の 4 欄は定数の名を引く（"recommend" は質問の行の欄の名と同じ字
/// なので数えず、名を引くことを見る）。
#[test]
fn f197_only_intake_spells_the_copied_values() {
    let src = repo_root().join("crates/folio/src");
    let mut at = Vec::new();
    let mut intake = String::new();
    for entry in fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let text = fs::read_to_string(&path).unwrap();
        let head = text.split("#[cfg(test)]").next().unwrap_or_default();
        let flat: String = head.chars().filter(|c| !c.is_whitespace()).collect();
        for word in ["\"inject\"", "[\"yes\",\"no\"]"] {
            at.extend(std::iter::repeat_n(format!("{name}: {word}"), flat.matches(word).count()));
        }
        if name == "intake.rs" {
            intake = flat;
        }
    }
    at.sort();
    assert_eq!(at, ["intake.rs: \"inject\"", "intake.rs: [\"yes\",\"no\"]"], "写しの値の字を持つ file");
    for want in [
        "(\"values_count\",Floor::Num(ANSWER_BRANCHES.len()))",
        "(\"branches\",Floor::Strs(&ANSWER_BRANCHES))",
        "(\"default_fixed\",Floor::Val(DEFAULT_RECOMMEND))",
        "(\"fixed\",Floor::Strs(&[INJECT_TARGET]))",
    ] {
        assert!(intake.contains(want), "INTAKE_FLOOR が定数を引かない: {want}");
    }
}

/// 歯 5（便 197 に同乗・台帳 f2-648.227）: 実の rules.yaml の生成区間の除外（excluded）は、what が人の作業の時間と AI の費用の
/// 2 語で、why が道具の測れる機械の待ち時間を除外に入れない字を持つ。期待は歯の中の手書き。
#[test]
fn f197_rules_excluded_names_only_human_time_and_ai_cost() {
    let region = f197_region("rules.yaml");
    let docs = YamlLoader::load_from_str(&region).unwrap();
    let excluded = &docs[0]["schema"]["excluded"];
    assert_eq!(
        f197_strs(&excluded["what"]),
        ["人の作業の時間（「60 分以内」）", "AI の費用（「300k token 以下」）"],
        "{region}"
    );
    let why = excluded["why"].as_str().unwrap_or_default();
    assert!(why.starts_with("人の作業の時間と AI の費用は測る仕組みが別"), "{why}");
    assert!(why.contains("機械の待ち時間は除外に当たらず"), "{why}");
}
