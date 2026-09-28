//! 極性一覧の口と編集時の止めの下限の歯（便 200・docs/design/delivery-200.md §1 (e)・判断の記録 ADR-33 決定 (5)・要件書 FR29・AC32）。
//! folio は実行 file の crate なので命令を撃つ。写しは床の土台（tests/fixtures/floor_base/design-intent）を一時 dir に design-intent/ として作り、
//! 置き場の親の contracts/ に器の導出 file を写し、git init と 1 commit を行う（tests/mechanism_live.rs と同じ作り方・歯の終わりに消す）。
//! 凍結 anchor（P-10.1）は、土台の正本を歯の側で手で数えた本数と行の字（folio の code から組まない）。
//! 1. 土台の極性一覧の行の数と集計の 1 行／2. 下限の行の値を割る写しは素の床が違反 1・足りる写しは合格／
//! 3. 下限の行が 2 本の写しは まだ分からない／4. 値の形が違う写しは まだ分からない／5. 下限の行が無い写しは数えなかった 1 行／
//! 6. 行の極性は行の条の機構の極性（検証役の N-19x-2）／7. 正本の symlink は床と同じ読み口で断る（検証役の N-19x-4）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FLOOR_BASE: &str = "tests/fixtures/floor_base/design-intent";
/// 土台の手で数えた本数: 憲法の条の機構の種別が reject か build-check の条 24・規則の表の閾値の行 17・床の定数の仕掛け 5（post）。
const ARTICLES: usize = 24;
const ROWS: usize = 17;
const FLOOR_GUARDS: usize = 5;
/// 土台の集計の 1 行（手で書く）。
const SUMMARY: &str = "folio check --polarity: 仕掛け 46（in-loop 0・post 46）・下限の行が無い＝数えない";
/// 下限の行が無くて数えなかった知らせ（delivery-200.md §1 (b)・手で書く）。
const OFF: &str = "# 欄 key が in-loop-min の閾値の行が規則の表に無い＝編集時の止めの本数の下限は数えていない（床の判定の外・条 P-18.4）";
const UNKNOWN: &str = "# まだ分からない: ";
/// 土台の行 R-13 の頭と値（行の欄 key を足し、値を変える所）。
const R13: &str = "{id: R-13, article: P-18, ";
const R13_VALUE: &str = "value: \"1 本以上\", kind: deny";
/// 土台の条 P-11 の機構の極性（fail-open に替える所）。
const OPEN_FROM: &str = "polarity: fail-closed, note: 便・並列実行の再試行回数を R-7";
const OPEN_TO: &str = "polarity: fail-open, note: 便・並列実行の再試行回数を R-7";

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
        .args(["-c", "user.email=fx@example", "-c", "user.name=fx", "-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .expect("git を起動できない");
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

/// 写しの一時 dir（歯の終わりに消す）。
struct Work {
    root: PathBuf,
}

impl Work {
    /// 土台を design-intent/ として写し、規則の表に `edits`（前の字 → 後の字・各 1 度）を当ててから git init と 1 commit。
    fn new(case: &str, edits: &[(&str, &str)]) -> Work {
        Work::at(case, "rules.yaml", edits)
    }

    /// `new` と同じで、`edits` を当てる file を置き場からの相対の字 `file` で選ぶ。
    fn at(case: &str, file: &str, edits: &[(&str, &str)]) -> Work {
        let root = std::env::temp_dir().join(format!("folio-f200-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_tree(&repo_root().join(FLOOR_BASE), &root.join("design-intent"));
        fs::create_dir_all(root.join("contracts")).unwrap();
        fs::copy(repo_root().join("contracts/schema.toml"), root.join("contracts/schema.toml")).unwrap();
        let path = root.join("design-intent").join(file);
        let mut text = fs::read_to_string(&path).unwrap();
        for (from, to) in edits {
            assert_eq!(text.matches(from).count(), 1, "写しの字が 1 度でない: {from}");
            text = text.replacen(from, to, 1);
        }
        fs::write(&path, text).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-q", "-m", "fixture"]);
        Work { root }
    }

    fn check(&self, flags: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("check")
            .arg("--dir")
            .arg(self.root.join("design-intent"))
            .args(flags)
            .output()
            .expect("folio を起動できない")
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn lines(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes).lines().map(str::to_string).collect()
}

fn show(out: &Output) -> String {
    format!("{}\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
}

/// 違反の行（`[名札] …`）。
fn violations(out: &Output) -> Vec<String> {
    lines(&out.stdout).into_iter().filter(|l| l.starts_with('[')).collect()
}

/// まだ分からない の行（床は標準エラーに出す）。
fn unknowns(out: &Output) -> Vec<String> {
    lines(&out.stderr).into_iter().filter(|l| l.starts_with(UNKNOWN)).collect()
}

fn off_count(out: &Output) -> usize {
    lines(&out.stderr).iter().filter(|l| *l == OFF).count()
}

/// 行 R-13 に欄 key の値 in-loop-min を付け、値を `value` にする写しの編集。
fn keyed_r13(value: &'static str) -> [(&'static str, String); 2] {
    [
        (R13, format!("{R13}key: in-loop-min, ")),
        (R13_VALUE, format!("value: \"{value}\", kind: deny")),
    ]
}

fn borrow<'a>(edits: &'a [(&'static str, String)]) -> Vec<(&'static str, &'a str)> {
    edits.iter().map(|(from, to)| (*from, to.as_str())).collect()
}

// ── f200_ 1. 土台の極性一覧の行の数と集計の 1 行 ──

#[test]
fn f200_polarity_lists_each_guard_and_a_summary_on_the_floor_base() {
    let w = Work::new("list", &[]);
    let out = w.check(&["--polarity"]);
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    assert!(out.stderr.is_empty(), "{}", show(&out));
    let got = lines(&out.stdout);
    assert_eq!(got.len(), ARTICLES + ROWS + FLOOR_GUARDS + 1, "{}", show(&out));
    assert_eq!(got.last().map(String::as_str), Some(SUMMARY), "{}", show(&out));
    let from = |suffix: &str| got.iter().filter(|l| l.ends_with(suffix)).count();
    assert_eq!(from(" · 憲法の条の機構"), ARTICLES, "{}", show(&out));
    assert_eq!(from(" · 規則の表の閾値の行"), ROWS, "{}", show(&out));
    assert_eq!(from(" · 床の定数の仕掛けの一覧"), FLOOR_GUARDS, "{}", show(&out));
    // 出所の順（憲法 → 規則の表 → 床の定数）と 1 行の形（名 · 段 · 極性 · 出所）
    assert_eq!(got[0], "P-1 · post · fail-closed · 憲法の条の機構", "{}", show(&out));
    assert_eq!(got[ARTICLES], "R-1 · post · fail-closed · 規則の表の閾値の行", "{}", show(&out));
    assert_eq!(got[ARTICLES + ROWS], "yaml-form · post · fail-closed · 床の定数の仕掛けの一覧", "{}", show(&out));
    // 種別の絞り: 機構の種別が human-review の条（P-16・A-1・A-4）は一覧に出ない（前文の none は条でない）
    for id in ["P-16", "A-1", "A-4"] {
        assert!(!got.iter().any(|l| l.starts_with(&format!("{id} · "))), "{id}: {}", show(&out));
    }
}

// ── f200_ 2. 下限の行の値を割る写しは素の床が違反 1・足りる写しは合格 ──

#[test]
fn f200_floor_fails_when_in_loop_guards_are_below_the_bound() {
    let edits = keyed_r13("99 本以上");
    let w = Work::new("short", &borrow(&edits));
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    assert_eq!(
        violations(&out),
        ["[P-18] 極性一覧の編集時（in-loop）の仕掛けが 0 本で、行 R-13 の下限 99 本以上を割る（P-18.4）"],
        "{}",
        show(&out)
    );
    assert_eq!(off_count(&out), 0, "{}", show(&out));
    let listed = w.check(&["--polarity"]);
    assert_eq!(listed.status.code(), Some(0), "{}", show(&listed));
    assert_eq!(
        lines(&listed.stdout).last().map(String::as_str),
        Some("folio check --polarity: 仕掛け 46（in-loop 0・post 46）・下限 99 本以上（行 R-13）に足りない"),
        "{}",
        show(&listed)
    );

    // 行 R-7 の段を in-loop にし、下限を 1 本以上にすると足りる（段が in-loop の行を数える）
    let mut edits = keyed_r13("1 本以上").to_vec();
    edits.push(("{id: R-7, ", "{id: R-7, stage: in-loop, ".to_string()));
    edits.push((", stage: post, same_failure", ", same_failure".to_string()));
    let w = Work::new("enough", &borrow(&edits));
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    assert!(violations(&out).is_empty(), "{}", show(&out));
    let listed = w.check(&["--polarity"]);
    let got = lines(&listed.stdout);
    assert!(got.iter().any(|l| l == "R-7 · in-loop · fail-closed · 規則の表の閾値の行"), "{}", show(&listed));
    assert_eq!(
        got.last().map(String::as_str),
        Some("folio check --polarity: 仕掛け 46（in-loop 1・post 45）・下限 1 本以上（行 R-13）に足りる"),
        "{}",
        show(&listed)
    );
}

// ── f200_ 3. 下限の行が 2 本の写しは まだ分からない ──

#[test]
fn f200_two_bound_rows_are_unknown() {
    let mut edits = keyed_r13("1 本以上").to_vec();
    edits.push(("{id: R-7, ", "{id: R-7, key: in-loop-min, ".to_string()));
    let w = Work::new("two", &borrow(&edits));
    let listed = w.check(&["--polarity"]);
    assert_eq!(listed.status.code(), Some(2), "{}", show(&listed));
    assert_eq!(
        lines(&listed.stdout),
        [
            "# まだ分からない: 欄 key が in-loop-min の閾値の行が 2 本ある",
            "folio check --polarity: まだ分からない（一覧を組めない）",
        ],
        "{}",
        show(&listed)
    );
    // 素の床: 下限の判定は まだ分からない の 1 行で、床は まだ分からない（2）。同じ key の 2 本目は便 179 の欄の決まりの違反として別に数える
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(2), "{}", show(&out));
    assert_eq!(
        violations(&out),
        ["[schema] rules.yaml: 行 R-13 の key「in-loop-min」を持つ閾値の行が 2 本以上ある"],
        "{}",
        show(&out)
    );
    assert_eq!(
        unknowns(&out),
        ["# まだ分からない: rules.yaml: 欄 key が in-loop-min の閾値の行が 2 本ある"],
        "{}",
        show(&out)
    );
    assert_eq!(off_count(&out), 0, "{}", show(&out));
}

// ── f200_ 4. 値の形が違う写しは まだ分からない ──

#[test]
fn f200_a_bound_value_out_of_form_is_unknown() {
    for (case, value) in [("zero", "0 本以上"), ("unit", "3 本"), ("word", "いくつか 本以上")] {
        let edits = keyed_r13(value);
        let w = Work::new(case, &borrow(&edits));
        let out = w.check(&[]);
        assert_eq!(out.status.code(), Some(2), "{value}: {}", show(&out));
        assert!(violations(&out).is_empty(), "{value}: {}", show(&out));
        assert_eq!(
            unknowns(&out),
            [format!("# まだ分からない: rules.yaml: 行 R-13 の value「{value}」が「<正の整数> 本以上」の形でない")],
            "{value}: {}",
            show(&out)
        );
        assert_eq!(off_count(&out), 0, "{value}: {}", show(&out));
        assert_eq!(w.check(&["--polarity"]).status.code(), Some(2), "{value}");
    }
}

// ── f200_ 5. 下限の行が無い写しは数えなかった 1 行 ──

#[test]
fn f200_no_bound_row_prints_one_line_and_does_not_count() {
    let w = Work::new("off", &[]);
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    assert_eq!(off_count(&out), 1, "{}", show(&out));
    assert!(violations(&out).is_empty() && unknowns(&out).is_empty(), "{}", show(&out));
    // 口（--proposed）は同じ床の関数を撃つが、知らせの行は素の床だけが出す
    let rules = fs::read_to_string(w.root.join("design-intent/rules.yaml")).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(["check", "--dir"])
        .arg(w.root.join("design-intent"))
        .args(["--proposed", "rules.yaml"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("folio を起動できない");
    let short = rules.replacen(R13, &format!("{R13}key: in-loop-min, "), 1).replacen(R13_VALUE, "value: \"99 本以上\", kind: deny", 1);
    std::io::Write::write_all(child.stdin.as_mut().unwrap(), short.as_bytes()).unwrap();
    let proposed = child.wait_with_output().unwrap();
    assert_eq!(proposed.status.code(), Some(1), "{}", show(&proposed));
    assert!(
        lines(&proposed.stdout).iter().any(|l| l.contains("行 R-13 の下限 99 本以上を割る")),
        "{}",
        show(&proposed)
    );
    assert_eq!(off_count(&proposed), 0, "{}", show(&proposed));
}

// ── f200_ 6. 行の極性は行の条の機構の極性（検証役の N-19x-2） ──

#[test]
fn f200_row_polarity_follows_its_article() {
    // 条 P-11（行 R-7 の条）の機構の極性を fail-open にした写し
    let w = Work::at("open", "constitution.yaml", &[(OPEN_FROM, OPEN_TO)]);
    let out = w.check(&["--polarity"]);
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    let got = lines(&out.stdout);
    for line in ["P-11 · post · fail-open · 憲法の条の機構", "R-7 · post · fail-open · 規則の表の閾値の行"] {
        assert!(got.iter().any(|l| l == line), "{line}: {}", show(&out));
    }
    assert_eq!(got.iter().filter(|l| l.contains(" · fail-open · ")).count(), 2, "{}", show(&out));
    assert_eq!(got.last().map(String::as_str), Some(SUMMARY), "{}", show(&out));
}

// ── f200_ 7. 正本の symlink は床と同じ読み口で断る（検証役の N-19x-4） ──

#[test]
fn f200_polarity_refuses_a_symlinked_source_like_the_floor() {
    let w = Work::new("link", &[]);
    let rules = w.root.join("design-intent/rules.yaml");
    let outside = w.root.join("outside-rules.yaml");
    fs::rename(&rules, &outside).unwrap();
    std::os::unix::fs::symlink(&outside, &rules).unwrap();
    let listed = w.check(&["--polarity"]);
    assert_eq!(listed.status.code(), Some(2), "{}", show(&listed));
    assert_eq!(
        lines(&listed.stdout),
        ["# まだ分からない: rules.yaml: symlink は認めない", "folio check --polarity: まだ分からない（一覧を組めない）"],
        "{}",
        show(&listed)
    );
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(2), "{}", show(&out));
    assert_eq!(unknowns(&out), ["# まだ分からない: rules.yaml: symlink は認めない"], "{}", show(&out));
}
