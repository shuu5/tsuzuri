//! 便 128（docs/design/delivery-128.md §1 (c) 歯 1〜5・FR5・憲法 N-3.1）: 憲法の meta・前文・前文の mechanism・条・条の mechanism・
//! 規範文と、規則の表の閾値行・開発規律行の欄を holder ごとの閉じた一覧で数え、一覧に無い欄を 1 つにつき種別 未知の欄 の違反 1 件に
//! すること、meta・前文・前文の mechanism・条の mechanism が表でない形と mechanism の必須の欄 kind・live の欠けを種別 schema の
//! 違反にすることの歯。憲法の 5 部位の一覧は道具を組み立てた憲法の正本から導出したもの、規則の表の行の一覧は床の定数で、
//! 置き場の schema 節と生成区間は読まない。
//! 土台は design-intent の写し（git の 1 commit・tests/constitution_range.rs の Work と同じ形）で、写しの字面に変異を当てて
//! `folio check` を撃つ。歯は標準出力の違反の行（`[` で始まる行）を全部数える。変異で足す欄の値は id の形を含まない字にする。

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

const CONSTITUTION: &str = "constitution.yaml";
const RULES: &str = "rules.yaml";

/// meta の頭（写しの中で 1 か所）。
const META_HEAD: &str = "\nmeta:\n  id: folio2-constitution\n";
/// 前文の頭。
const PRECEDENCE_HEAD: &str = "\nprecedence:\n";
/// 前文の mechanism の行。
const PRECEDENCE_MECHANISM: &str =
    "  mechanism: {kind: none, live: now, note: 判断の規則であって機械では検査できない。生成区間（P-14）の先頭に写す。}\n";
/// 条 N-3 の頭。
const N3_HEAD: &str = "  - id: N-3\n";
/// 条 N-3 の mechanism の行。
const N3_MECHANISM: &str = "    mechanism: {kind: build-check, live: now, stage: post, polarity: fail-closed, note: 正本と rules の全節（top_level・meta・前文・条・規範文・mechanism・rules 行）で未知の欄を schema 検査が落とす（床は folio check）。}\n";
/// 条 N-3 の mechanism の表の頭。
const N3_MECHANISM_MAP: &str = "    mechanism: {kind: build-check, live: now, stage: post, polarity: fail-closed, note: 正本と rules";
/// 規範文 N-3.1 の頭。
const N31_HEAD: &str = "{id: N-3.1, ";
/// 憲法の schema.meta.optional の行。
const SCHEMA_META_OPTIONAL: &str = "    optional: [changes_from_v0_1, changes_from_v0_2]\n";
/// 規則の表の生成区間の threshold_row.optional の行。
const THRESHOLD_ROW_OPTIONAL: &str =
    "    optional: [basis, projection, same_failure, population, note, refs]\n";

/// design-intent の写しの一時 dir（歯の終わりに消す）。器（scribe2）の導出 file は写しの根の contracts/ に置く。
struct Work {
    root: PathBuf,
}

impl Work {
    fn new(case: &str) -> Work {
        let root = std::env::temp_dir().join(format!(
            "folio-unknown-fields-{case}-{}",
            std::process::id()
        ));
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

    /// 写しの file の字面の変異（当て先はちょうど 1 か所）。
    fn mutate(&self, file: &str, from: &str, to: &str) {
        let path = self.root.join("design-intent").join(file);
        let before = fs::read_to_string(&path).unwrap();
        assert_eq!(
            before.matches(from).count(),
            1,
            "変異の当て先が 1 か所でない: {file}: {from:?}"
        );
        fs::write(&path, before.replacen(from, to, 1)).unwrap();
    }

    fn check(&self) -> Run {
        let out = Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("check")
            .arg("--dir")
            .arg(self.root.join("design-intent"))
            .output()
            .expect("folio を起動できない");
        Run::of(&out)
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// `folio check` の 1 回の結果を数えたもの。
struct Run {
    code: Option<i32>,
    /// 改憲の違反（種別 N-4）の行。
    n4: Vec<String>,
    /// N-4 を除いた違反の行。
    others: Vec<String>,
    /// 判定の行。
    verdict: String,
    all: String,
}

impl Run {
    fn of(out: &Output) -> Run {
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        let violations: Vec<String> = stdout
            .lines()
            .filter(|l| l.starts_with('['))
            .map(str::to_string)
            .collect();
        Run {
            code: out.status.code(),
            n4: violations
                .iter()
                .filter(|l| l.starts_with("[N-4] "))
                .cloned()
                .collect(),
            others: violations
                .iter()
                .filter(|l| !l.starts_with("[N-4] "))
                .cloned()
                .collect(),
            verdict: stdout
                .lines()
                .find(|l| l.starts_with("folio check: "))
                .unwrap_or("")
                .to_string(),
            all: format!("{stdout}{stderr}"),
        }
    }

    /// 違反の行がちょうど `lines`（N-4 は無い）で、終了コード 1。
    fn only(&self, case: &str, lines: &[String]) {
        assert!(self.n4.is_empty(), "{case}: N-4 は無いはず: {}", self.all);
        let mut got = self.others.clone();
        got.sort();
        let mut want = lines.to_vec();
        want.sort();
        assert_eq!(got, want, "{case}: {}", self.all);
        assert_eq!(self.code, Some(1), "{case}: {}", self.all);
    }

    /// 違反の行がちょうど 2 行 = 改憲の違反 1 行と `line`、終了コード 1。
    fn amendment_and(&self, case: &str, line: &str) {
        assert_eq!(self.n4.len(), 1, "{case}: N-4 がちょうど 1 件のはず: {}", self.all);
        assert_eq!(self.others, [line], "{case}: {}", self.all);
        assert_eq!(self.code, Some(1), "{case}: {}", self.all);
    }
}

fn unknown(file: &str, at: &str, key: &str) -> String {
    format!("[未知の欄] {file}: {at} の未知の欄「{key}」")
}

/// 歯 1: 憲法の 5 部位に足した未知の欄は、それぞれ違反 1 行（前文と前文の mechanism は改憲の違反と 2 行）。
#[test]
fn f128_constitution_unknown_fields_are_violations() {
    let w = Work::new("control");
    let r = w.check();
    assert_eq!(r.code, Some(0), "{}", r.all);
    assert_eq!(
        r.verdict, "folio check: 合格（違反 0・まだ分からない 0）",
        "{}",
        r.all
    );

    let n3_mech = N3_MECHANISM_MAP.replacen("{kind:", "{bogus_m: x, kind:", 1);
    let cases: [(&str, &str, String, String); 4] = [
        (
            "meta",
            META_HEAD,
            META_HEAD.replacen("\n  id:", "\n  bogus_meta: x\n  id:", 1),
            unknown(CONSTITUTION, "meta", "bogus_meta"),
        ),
        (
            "article",
            N3_HEAD,
            format!("{N3_HEAD}    bogus_a: x\n"),
            unknown(CONSTITUTION, "条 N-3", "bogus_a"),
        ),
        (
            "article-mechanism",
            N3_MECHANISM_MAP,
            n3_mech,
            unknown(CONSTITUTION, "条 N-3 の mechanism", "bogus_m"),
        ),
        (
            "statement",
            N31_HEAD,
            format!("{N31_HEAD}bogus_s: x, "),
            unknown(CONSTITUTION, "条 N-3 の規範文 N-3.1", "bogus_s"),
        ),
    ];
    for (case, from, to, line) in cases {
        let w = Work::new(case);
        w.mutate(CONSTITUTION, from, &to);
        w.check().only(case, &[line]);
    }

    let w = Work::new("precedence");
    w.mutate(
        CONSTITUTION,
        PRECEDENCE_HEAD,
        &format!("{PRECEDENCE_HEAD}  bogus_p: x\n"),
    );
    w.check().amendment_and(
        "precedence",
        &unknown(CONSTITUTION, "前文（precedence）", "bogus_p"),
    );

    let w = Work::new("precedence-mechanism");
    w.mutate(
        CONSTITUTION,
        PRECEDENCE_MECHANISM,
        &PRECEDENCE_MECHANISM.replacen("{kind:", "{bogus_pm: x, kind:", 1),
    );
    w.check().amendment_and(
        "precedence-mechanism",
        &unknown(CONSTITUTION, "前文（precedence）の mechanism", "bogus_pm"),
    );
}

/// 歯 2: 規則の表の閾値行・開発規律行に足した未知の欄は、それぞれ違反 1 行。
#[test]
fn f128_rule_row_unknown_fields_are_violations() {
    for (case, head, add, at, key) in [
        ("threshold", "{id: R-2, ", "bogus_r: x, ", "行 R-2", "bogus_r"),
        ("discipline-flag", "{id: D-5, ", "disabled: true, ", "行 D-5", "disabled"),
        ("discipline-stage", "{id: D-14, ", "stage: post, ", "行 D-14", "stage"),
    ] {
        let w = Work::new(case);
        w.mutate(RULES, head, &format!("{head}{add}"));
        w.check().only(case, &[unknown(RULES, at, key)]);
    }
}

/// 歯 3: 置き場の憲法の schema 節と規則の表の生成区間で一覧を広げても、未知の欄の違反は消えない。
#[test]
fn f128_place_schema_cannot_widen_the_lists() {
    let w = Work::new("widen-meta");
    w.mutate(
        CONSTITUTION,
        SCHEMA_META_OPTIONAL,
        "    optional: [changes_from_v0_1, changes_from_v0_2, bogus_meta]\n",
    );
    w.mutate(
        CONSTITUTION,
        META_HEAD,
        &META_HEAD.replacen("\n  id:", "\n  bogus_meta: x\n  id:", 1),
    );
    w.check()
        .amendment_and("widen-meta", &unknown(CONSTITUTION, "meta", "bogus_meta"));

    let w = Work::new("widen-threshold");
    w.mutate(
        RULES,
        THRESHOLD_ROW_OPTIONAL,
        "    optional: [basis, projection, same_failure, population, note, refs, bogus_r]\n",
    );
    w.mutate(RULES, "{id: R-2, ", "{id: R-2, bogus_r: x, ");
    w.check()
        .only("widen-threshold", &[unknown(RULES, "行 R-2", "bogus_r")]);
}

/// 歯 4: 一覧は holder ごとで和集合でない（条で在ってよい欄は規範文と mechanism で、閾値行で在ってよい欄は開発規律行で未知）。
#[test]
fn f128_lists_are_per_holder() {
    let w = Work::new("per-holder");
    w.mutate(CONSTITUTION, N31_HEAD, &format!("{N31_HEAD}note: x, "));
    w.mutate(
        CONSTITUTION,
        N3_MECHANISM_MAP,
        &N3_MECHANISM_MAP.replacen("{kind:", "{relations: x, kind:", 1),
    );
    w.mutate(
        CONSTITUTION,
        N3_HEAD,
        &format!("{N3_HEAD}    bogus_a1: x\n    bogus_a2: x\n"),
    );
    w.mutate(RULES, "{id: D-5, ", "{id: D-5, value: x, ");
    w.check().only(
        "per-holder",
        &[
            unknown(CONSTITUTION, "条 N-3 の規範文 N-3.1", "note"),
            unknown(CONSTITUTION, "条 N-3 の mechanism", "relations"),
            unknown(CONSTITUTION, "条 N-3", "bogus_a1"),
            unknown(CONSTITUTION, "条 N-3", "bogus_a2"),
            unknown(RULES, "行 D-5", "value"),
        ],
    );
}

/// 歯 5: mechanism の形の崩れ（表でない・必須の欄 kind と live の欠け）は種別 schema の違反。
#[test]
fn f128_mechanism_shape_is_a_violation() {
    let not_a_map = "[schema] constitution.yaml: 条 N-3 の mechanism が表でない".to_string();
    let missing =
        |key: &str| format!("[schema] constitution.yaml: 条 N-3 の mechanism の必須の欄 {key} が無い");
    let cases: [(&str, &str, Vec<String>); 5] = [
        ("string", "無効", vec![not_a_map.clone()]),
        ("list", "[disabled]", vec![not_a_map.clone()]),
        ("null", "null", vec![not_a_map]),
        ("empty", "{}", vec![missing("kind"), missing("live")]),
        (
            "no-kind",
            "{live: now, stage: post, polarity: fail-closed, note: x}",
            vec![missing("kind")],
        ),
    ];
    for (case, value, lines) in cases {
        let w = Work::new(&format!("shape-{case}"));
        w.mutate(
            CONSTITUTION,
            N3_MECHANISM,
            &format!("    mechanism: {value}\n"),
        );
        w.check().only(case, &lines);
    }

    let w = Work::new("shape-precedence");
    w.mutate(CONSTITUTION, PRECEDENCE_MECHANISM, "  mechanism: 無効\n");
    w.check().amendment_and(
        "shape-precedence",
        "[schema] constitution.yaml: 前文（precedence）の mechanism が表でない",
    );
}
