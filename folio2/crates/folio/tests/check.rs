//! `folio check` 自身の歯（便 0・docs/design/delivery-0.md §1 / §4）。
//! 正本 4 file で合格、tests/fixtures/check/ の 4 組で 不合格 1 / 不合格 1 / 不合格 1 / まだ分からない 2。
//! 各組は変異 1 つだけを持つ＝違反はちょうど 1 件で、その種類まで見る（別の理由で落ちた組を緑にしない）。
//! 要件書の図の節（便 34・FR15）は design-intent の写し（copy_tree + git init・tests/adr.rs の Work と同じ形）の
//! srs.yaml に図を 1 枚足して合格を見、その図に変異 1 つずつ（型・caption・refs・図の id の重複・未知の欄）で 不合格 1 を見る。
//! 憲法の条の値域を持つ欄（便 55・床の穴 f2-648.82）は同じ写しの constitution.yaml の条 P-1 に変異 1 つずつを当てて見る
//! （改訂の差分の範囲の外の欄 3 つは違反ちょうど 1 件・範囲の内の欄 2 つは値域の違反を含む・欄が無ければ黙る）。

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

/// 見本の図 1 枚（型 archify-architecture・欄そろい・refs は要件の id）。実の srs.yaml の末尾に足す。
const FIGURE: &str = "figures:
  - id: fig-1
    type: archify-architecture
    caption: 見本の図
    refs: [FR1]
    spec:
      schema_version: 1
      diagram_type: architecture
      meta: {title: 面の組み立て, quality_profile: showcase}
      components:
        - {id: src, type: external, label: 正本, row: 0, col: 0}
        - {id: face, type: frontend, label: 面, row: 0, col: 1}
      connections:
        - {from: src, to: face, label: 導出}
      layout: {mode: grid, cols: 2, gapX: 70, gapY: 110, cellW: 160, cellH: 70}
";

/// design-intent の写しの一時 dir（歯の終わりに消す）。器（scribe2）の導出 file は写しの根の contracts/ に置く。
struct Work {
    root: PathBuf,
}

impl Work {
    fn new(case: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-check-{case}-{}", std::process::id()));
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

    /// 実の srs.yaml の写し。
    fn srs(&self) -> PathBuf {
        self.dir().join("srs.yaml")
    }

    /// 写しの srs.yaml の末尾に図 1 枚を足す。実の要件書が図の節を持つときは、その節（`figures:` 以降・末尾まで）を
    /// 消してから足す（見る図は見本の 1 枚だけ）。
    fn with_figure(&self) {
        let before = fs::read_to_string(self.srs()).unwrap();
        let before = match before.find("\nfigures:\n") {
            Some(at) => format!("{}\n", &before[..at]),
            None => before,
        };
        let sep = if before.ends_with('\n') { "" } else { "\n" };
        fs::write(self.srs(), format!("{before}{sep}{FIGURE}")).unwrap();
    }

    /// 実の rules.yaml の写し。
    fn rules(&self) -> PathBuf {
        self.dir().join("rules.yaml")
    }

    /// 写しの srs.yaml の字面の変異（1 か所だけ）。
    fn mutate(&self, from: &str, to: &str) {
        mutate_file(&self.srs(), from, to);
    }

    /// 写しの rules.yaml の字面の変異（1 か所だけ）。
    fn mutate_rules(&self, from: &str, to: &str) {
        mutate_file(&self.rules(), from, to);
    }

    /// 実の constitution.yaml の写し。
    fn constitution(&self) -> PathBuf {
        self.dir().join("constitution.yaml")
    }

    /// 写しの constitution.yaml の字面の変異（1 か所だけ）。
    fn mutate_constitution(&self, from: &str, to: &str) {
        mutate_file(&self.constitution(), from, to);
    }

    fn check(&self) -> Output {
        folio_check(&self.dir())
    }
}

/// file の字面の変異（当て先はちょうど 1 か所）。
fn mutate_file(path: &Path, from: &str, to: &str) {
    let before = fs::read_to_string(path).unwrap();
    assert_eq!(
        before.matches(from).count(),
        1,
        "変異の当て先が 1 か所でない: {from:?}"
    );
    fs::write(path, before.replacen(from, to, 1)).unwrap();
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn folio_check(dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("check")
        .arg("--dir")
        .arg(dir)
        .output()
        .expect("folio を起動できない")
}

fn fixture(name: &str) -> PathBuf {
    repo_root().join("tests/fixtures/check").join(name)
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// 違反の行（`[種類] …`）だけを拾う。
fn violations(out: &Output) -> Vec<String> {
    stdout(out)
        .lines()
        .filter(|l| l.starts_with('['))
        .map(str::to_string)
        .collect()
}

fn assert_single_violation(name: &str, kind: &str, file: &str) {
    let out = folio_check(&fixture(name));
    assert_eq!(out.status.code(), Some(1), "{name}: {}", stdout(&out));
    let v = violations(&out);
    assert_eq!(v.len(), 1, "{name}: 違反は変異の 1 件だけのはず: {v:?}");
    assert!(
        v[0].starts_with(&format!("[{kind}] {file}")),
        "{name}: {v:?}"
    );
    assert!(stdout(&out).contains("不合格"), "{name}");
}

#[test]
fn check_canonical_design_intent_passes() {
    let out = folio_check(&repo_root().join("design-intent"));
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(violations(&out).is_empty());
    assert!(stdout(&out).contains("合格"));
}

#[test]
fn check_dup_key_fails() {
    assert_single_violation("dup-key", "重複キー", "rules.yaml");
}

#[test]
fn check_unknown_section_fails() {
    assert_single_violation("unknown-section", "未知の節", "srs.yaml");
}

#[test]
fn check_empty_field_fails() {
    assert_single_violation("empty-field", "欄の非空", "vocabulary.yaml");
}

#[test]
fn check_missing_file_is_unknown_not_pass() {
    let out = folio_check(&fixture("missing-file"));
    assert_eq!(out.status.code(), Some(2), "{}", stdout(&out));
    assert!(String::from_utf8_lossy(&out.stderr).contains("constitution.yaml: 正本が無い"));
    let s = stdout(&out);
    assert!(
        s.contains("まだ分からない") && !s.contains("folio check: 合格"),
        "{s}"
    );
}

// ── 要件書の図の節（便 34） ──

/// 写しの srs.yaml に変異を当てた結果が 不合格 1・違反はちょうど 1 件（srs.yaml の場所）で `words` を全部含む。
fn assert_srs_figure_violation(w: &Work, words: &[&str]) {
    let out = w.check();
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    let v = violations(&out);
    assert_eq!(v.len(), 1, "違反は変異の 1 件だけのはず: {v:?}");
    assert!(v[0].contains("srs.yaml"), "{v:?}");
    for word in words {
        assert!(v[0].contains(word), "「{word}」が無い: {v:?}");
    }
    assert!(
        stdout(&out).contains("folio check: 不合格（違反 1・"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn check_srs_figure_with_all_fields_passes() {
    let w = Work::new("figure-ok");
    w.with_figure();
    let out = w.check();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(violations(&out).is_empty(), "{:?}", violations(&out));
    assert!(
        stdout(&out).contains("folio check: 合格（違反 0・まだ分からない 0）"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn check_srs_figure_type_outside_the_catalog_fails() {
    let w = Work::new("figure-type");
    w.with_figure();
    w.mutate(
        "\n    type: archify-architecture\n",
        "\n    type: mystery\n",
    );
    assert_srs_figure_violation(&w, &["図の型「mystery」が部品目録の一覧に無い"]);
}

#[test]
fn check_srs_figure_without_caption_fails() {
    let w = Work::new("figure-caption");
    w.with_figure();
    w.mutate("\n    caption: 見本の図\n", "\n");
    assert_srs_figure_violation(&w, &["caption"]);
}

#[test]
fn check_srs_figure_ref_not_an_id_fails() {
    let w = Work::new("figure-refs");
    w.with_figure();
    w.mutate("\n    refs: [FR1]\n", "\n    refs: [FR]\n");
    assert_srs_figure_violation(&w, &["refs「FR」が id の形"]);
}

#[test]
fn check_srs_figure_id_shared_with_a_requirement_fails() {
    let w = Work::new("figure-dup-id");
    w.with_figure();
    w.mutate("\n  - id: fig-1\n", "\n  - id: FR1\n");
    assert_srs_figure_violation(&w, &["行 id「FR1」が重複"]);
}

// ── 規則の表の最上位の節は床の定数から（便 51） ──

/// 実の rules.yaml の写しの schema.top_level の行（変異の当て先）。行末の注釈の有無に依らない形
/// （実の正本の schema の節は生成区間になり、注釈は欄 top_level_note へ移った）。
const RULES_TOP_LEVEL_LINE: &str = "\n  top_level: [schema, thresholds, discipline]";

/// file の側で schema.top_level に extras を足し、最上位に節 extras を足しても、床は定数の一覧で数える＝
/// 不合格 1・違反は「未知の節」の 1 件だけで extras を含む（本便の前の main では合格してしまう歯）。
#[test]
fn check_rules_section_added_via_the_file_top_level_is_still_unknown() {
    let w = Work::new("rules-extras");
    w.mutate_rules(
        RULES_TOP_LEVEL_LINE,
        "\n  top_level: [schema, thresholds, discipline, extras]",
    );
    let before = fs::read_to_string(w.rules()).unwrap();
    let sep = if before.ends_with('\n') { "" } else { "\n" };
    fs::write(w.rules(), format!("{before}{sep}extras: 余分\n")).unwrap();
    let out = w.check();
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    let v = violations(&out);
    assert_eq!(v.len(), 1, "違反は変異の 1 件だけのはず: {v:?}");
    assert!(v[0].starts_with("[未知の節] rules.yaml"), "{v:?}");
    assert!(v[0].contains("extras"), "{v:?}");
    assert!(stdout(&out).contains("不合格"), "{}", stdout(&out));
}

/// 写しの rules.yaml から schema.top_level の行を消しても、規則の表についての「まだ分からない」は出ない＝
/// 床の結果は消す前（実の正本 = 合格）と同じ。
#[test]
fn check_rules_without_the_file_top_level_line_is_not_unknown() {
    let w = Work::new("rules-no-top-level");
    let before = w.check();
    assert_eq!(before.status.code(), Some(0), "{}", stdout(&before));
    w.mutate_rules(RULES_TOP_LEVEL_LINE, "");
    let out = w.check();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(stdout(&out), stdout(&before));
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains("rules.yaml"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout(&out).contains("folio check: 合格（違反 0・まだ分からない 0）"),
        "{}",
        stdout(&out)
    );
}

// ── 憲法の条の値域を持つ欄（便 55） ──

/// 実の constitution.yaml の写しの条 P-1 の機構の行（変異の当て先・note の字面で 1 か所に絞る）。
const P1_MECHANISM: &str =
    "{kind: build-check, live: M0, stage: post, polarity: fail-closed, note: 公開する命令";

/// 写しの constitution.yaml に変異を当てた結果が 不合格 1・違反はちょうど 1 件（種別 schema・constitution.yaml の場所）で、
/// 値域の外の値 `value` と鍵の名 `schema.enums.<key>` を含む（改訂の差分の範囲の外の欄）。
fn assert_constitution_enum_violation(w: &Work, key: &str, value: &str) {
    let out = w.check();
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    let v = violations(&out);
    assert_eq!(v.len(), 1, "違反は変異の 1 件だけのはず: {v:?}");
    assert!(v[0].starts_with("[schema] constitution.yaml"), "{v:?}");
    assert!(v[0].contains("条 P-1"), "{v:?}");
    assert!(v[0].contains(&format!("「{value}」")), "{v:?}");
    assert!(
        v[0].contains(&format!("schema.enums.{key} に無い")),
        "{v:?}"
    );
    assert!(
        stdout(&out).contains("folio check: 不合格（違反 1・"),
        "{}",
        stdout(&out)
    );
}

/// 改訂の差分の範囲の内の欄（条文 5 欄）の変異 = 改訂の差分の違反も出るので件数は固定せず、値域の違反 1 件が
/// 在ることだけを見る（種別 schema・条 P-1・値・鍵の名）。
fn assert_constitution_enum_violation_among(w: &Work, key: &str, value: &str) {
    let out = w.check();
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    let v = violations(&out);
    let hits: Vec<&String> = v
        .iter()
        .filter(|l| {
            l.starts_with("[schema] constitution.yaml")
                && l.contains("条 P-1")
                && l.contains(&format!("「{value}」"))
                && l.contains(&format!("schema.enums.{key} に無い"))
        })
        .collect();
    assert_eq!(hits.len(), 1, "値域の違反がちょうど 1 件のはず: {v:?}");
    assert!(stdout(&out).contains("不合格"), "{}", stdout(&out));
}

#[test]
fn check_constitution_enum_mechanism_kind_outside_the_range_fails() {
    let w = Work::new("enum-mechanism-kind");
    w.mutate_constitution(
        P1_MECHANISM,
        "{kind: mystery, live: M0, stage: post, polarity: fail-closed, note: 公開する命令",
    );
    assert_constitution_enum_violation(&w, "mechanism_kind", "mystery");
}

#[test]
fn check_constitution_enum_mechanism_live_outside_the_range_fails() {
    let w = Work::new("enum-mechanism-live");
    w.mutate_constitution(
        P1_MECHANISM,
        "{kind: build-check, live: someday, stage: post, polarity: fail-closed, note: 公開する命令",
    );
    assert_constitution_enum_violation(&w, "mechanism_live", "someday");
}

#[test]
fn check_constitution_enum_rationale_kind_outside_the_range_fails() {
    let w = Work::new("enum-rationale-kind");
    w.mutate_constitution(
        "{kind: folio2-ruling, ref: \"裁定 #1（推奨で進み、支度表で覆す）\"}",
        "{kind: hearsay, ref: \"裁定 #1（推奨で進み、支度表で覆す）\"}",
    );
    assert_constitution_enum_violation(&w, "rationale_kind", "hearsay");
}

#[test]
fn check_constitution_enum_tier_outside_the_range_is_among_the_violations() {
    let w = Work::new("enum-tier");
    w.mutate_constitution(
        "\n  - id: P-1\n    title: 判断する道具を作らない\n    tier: always\n",
        "\n  - id: P-1\n    title: 判断する道具を作らない\n    tier: sometimes\n",
    );
    assert_constitution_enum_violation_among(&w, "tier", "sometimes");
}

#[test]
fn check_constitution_enum_strength_outside_the_range_is_among_the_violations() {
    let w = Work::new("enum-strength");
    w.mutate_constitution(
        "{id: P-1.2, pattern: ubiquitous, strength: must-not, text:",
        "{id: P-1.2, pattern: ubiquitous, strength: may, text:",
    );
    assert_constitution_enum_violation_among(&w, "strength", "may");
}

/// 欄が無ければ黙る = 条 P-1 の mechanism から stage の欄を消しても、本便の違反（schema.enums.stage を含む行）は出ない。
#[test]
fn check_constitution_enum_missing_stage_field_is_silent() {
    let w = Work::new("enum-no-stage");
    w.mutate_constitution(
        P1_MECHANISM,
        "{kind: build-check, live: M0, polarity: fail-closed, note: 公開する命令",
    );
    let out = w.check();
    let s = stdout(&out);
    assert!(!s.lines().any(|l| l.contains("schema.enums.stage")), "{s}");
    assert_eq!(
        out.status.code(),
        Some(0),
        "{s}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(violations(&out).is_empty(), "{:?}", violations(&out));
}

// ── 規則 R-11（強度と規範文の文末の一致・便 59） ──

/// 実の constitution.yaml の写しの条 P-1 の規範文 P-1.1（strength must・文末は「担う。」）の行の頭（変異の当て先）。
const P1_1_HEAD: &str = "{id: P-1.1, pattern: ubiquitous, strength: must, text:";

/// 写しの constitution.yaml に変異を当てた結果が 不合格 1 で、種別 R-11 の違反がちょうど 1 件在り、その文言が
/// 「P-1: P-1.1: strength <strength> と文末が合わない」を含む（改訂の差分の違反も出るので件数は固定しない）。
fn assert_r11_violation(w: &Work, strength: &str) {
    let out = w.check();
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    let v = violations(&out);
    let hits: Vec<&String> = v.iter().filter(|l| l.starts_with("[R-11] ")).collect();
    assert_eq!(hits.len(), 1, "R-11 の違反がちょうど 1 件のはず: {v:?}");
    assert!(
        hits[0].contains(&format!("P-1: P-1.1: strength {strength} と文末が合わない")),
        "{v:?}"
    );
    assert!(hits[0].contains("must-not ⇔ 〜ない。"), "{v:?}");
    assert!(stdout(&out).contains("不合格"), "{}", stdout(&out));
}

/// 分岐 1 = must-not なのに文末が「〜する。」（P-1.1 の strength だけを must-not に）。
#[test]
fn r11_must_not_with_affirmative_ending_fails() {
    let w = Work::new("r11-must-not");
    w.mutate_constitution(
        P1_1_HEAD,
        "{id: P-1.1, pattern: ubiquitous, strength: must-not, text:",
    );
    assert_r11_violation(&w, "must-not");
}

/// 分岐 2 = must なのに文末が「〜ない。」（P-1.1 の text の末尾だけを「担わない。」に・strength は must のまま）。
#[test]
fn r11_must_with_negative_ending_fails() {
    let w = Work::new("r11-must");
    w.mutate_constitution(
        "結果を知らせるところまでを担う。}",
        "結果を知らせるところまでを担わない。}",
    );
    assert_r11_violation(&w, "must");
}

/// 変えない写しは合格 = 実の正本の規範文に strength と文末の不一致が無い（式を足した後の床で確かめる）。
#[test]
fn r11_unchanged_copy_passes() {
    let w = Work::new("r11-unchanged");
    let out = w.check();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(violations(&out).is_empty(), "{:?}", violations(&out));
    assert!(
        stdout(&out).contains("folio check: 合格（違反 0・まだ分からない 0）"),
        "{}",
        stdout(&out)
    );
}

/// 規則の表 R-11 の行の注が実態（便 59 で Rust の床に戻した）を言う。
#[test]
fn r11_rules_note_says_the_floor_counts_it_again() {
    let text = fs::read_to_string(repo_root().join("design-intent/rules.yaml")).unwrap();
    let lines: Vec<&str> = text.lines().filter(|l| l.contains("{id: R-11,")).collect();
    assert_eq!(lines.len(), 1, "R-11 の行が 1 本でない: {lines:?}");
    let line = lines[0];
    assert!(
        line.contains("便 59 で Rust の床に戻した（folio check の種別 R-11）"),
        "{line}"
    );
    assert!(!line.contains("Rust の床へ写していない"), "{line}");
}

fn folio_help(subcommand: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg(subcommand)
        .arg("--help")
        .output()
        .expect("folio を起動できない");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    stdout(&out)
}

/// folio schema --help の --dir の説明が欄の決まりの file 4 本の名を出す。
#[test]
fn r11_schema_help_names_the_four_schema_files() {
    let help = folio_help("schema");
    assert!(
        help.contains("adr/schema.yaml・design-note/schema.yaml・ceiling.yaml・rules.yaml"),
        "{help}"
    );
}

/// folio parts --help の --page の説明が既定の面 5 つの名を出す。
#[test]
fn r11_parts_help_names_the_five_default_faces() {
    let help = folio_help("parts");
    assert!(
        help.contains("index / constitution / srs / adr / note"),
        "{help}"
    );
}

#[test]
fn check_srs_figure_with_an_unknown_field_fails() {
    let w = Work::new("figure-extra");
    w.with_figure();
    w.mutate(
        "\n    caption: 見本の図\n",
        "\n    caption: 見本の図\n    extra: 余分\n",
    );
    assert_srs_figure_violation(&w, &["未知の欄", "extra"]);
}

/// 退役した命令 folio render は使い方の誤り（終了 2）で止まり、標準エラーに render の字を出す。
#[test]
fn retired_render_subcommand_is_unrecognized() {
    let out = Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(["render", "--check"])
        .output()
        .expect("folio を起動できない");
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{err}");
    assert!(err.contains("render"), "{err}");
}

/// folio --help の命令の一覧に render が無く、build は在る。
#[test]
fn retired_render_help_does_not_list_render() {
    let out = Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("--help")
        .output()
        .expect("folio を起動できない");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let help = stdout(&out);
    assert!(!help.lines().any(|l| l.starts_with("  render")), "{help}");
    assert!(help.lines().any(|l| l.starts_with("  build")), "{help}");
}

/// 憲法 P-1 の機構: 公開する命令（subcommand）の一覧は閉じた一覧と全数で一致し、採否を決める口はその一覧に無い。
/// 一覧を足すときはこの歯と憲法 P-1 の機構の注を同じ便で直す（P-1.2・天井の 12 周目の実態 F-1）。
#[test]
fn p1_commands_closed_list() {
    const CLOSED: [&str; 11] = [
        "check", "inject", "parts", "face", "figure", "build", "intake", "hello", "ceiling", "schema",
        "serve",
    ];
    let out = Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("--help")
        .output()
        .expect("folio を起動できない");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let help = stdout(&out);
    let body = help.split("Commands:").nth(1).expect("Commands: の節が無い");
    let body = body.split("\n\n").next().unwrap_or(body);
    let listed: Vec<&str> = body
        .lines()
        .filter_map(|l| l.strip_prefix("  ").and_then(|l| l.split_whitespace().next()))
        .filter(|name| *name != "help")
        .collect();
    assert_eq!(listed, CLOSED, "{help}");
    for forbidden in ["approve", "decide", "accept", "reject", "judge"] {
        assert!(!CLOSED.contains(&forbidden), "採否を決める口「{forbidden}」が一覧に在る");
    }
}

// ── 要件の行の必須欄（便 75・面の生成器 face_srs.rs が読む欄から導いた床の定数） ──

const FR1_FIGURES: &str = "\n    figures: [図2-1, 図2-2, 図1]\n";
const FR1_VERIFY: &str = "    verify: {method: test, how: 決まった回答 5 つを入れ、支度表が期待どおりか比較する, ac: [AC1]}\n";

/// 写しの srs.yaml に変異を当てた結果が 不合格 1・違反はちょうど 1 件（srs.yaml の場所）で `words` を全部含む。
fn assert_srs_item_violation(w: &Work, words: &[&str]) {
    assert_srs_figure_violation(w, words);
}

#[test]
fn f75_srs_requirement_without_figures_fails() {
    let w = Work::new("f75-no-figures");
    w.mutate(FR1_FIGURES, "\n");
    assert_srs_item_violation(&w, &["requirements の FR1", "figures", "が無い"]);
}

#[test]
fn f75_srs_requirement_with_an_empty_figures_list_passes() {
    let w = Work::new("f75-empty-figures");
    w.mutate(FR1_FIGURES, "\n    figures: []\n");
    let out = w.check();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(violations(&out).is_empty(), "{:?}", violations(&out));
}

#[test]
fn f75_srs_requirement_without_verify_fails() {
    let w = Work::new("f75-no-verify");
    w.mutate(FR1_VERIFY, "");
    assert_srs_item_violation(&w, &["requirements の FR1", "verify", "が無い"]);
}

#[test]
fn f75_srs_requirement_with_an_empty_verify_how_fails() {
    let w = Work::new("f75-empty-how");
    w.mutate(
        FR1_VERIFY,
        "    verify: {method: test, how: \"\", ac: [AC1]}\n",
    );
    assert_srs_item_violation(&w, &["FR1 の verify", "how", "が空"]);
}

#[test]
fn f75_srs_requirement_with_an_unknown_field_fails() {
    let w = Work::new("f75-extra");
    w.mutate(FR1_FIGURES, "\n    figures: []\n    extra: 1\n");
    assert_srs_item_violation(&w, &["未知の欄", "extra"]);
}

#[test]
fn f75_srs_nonfunctional_without_figures_fails() {
    let w = Work::new("f75-nfr-no-figures");
    w.mutate("\n    figures: [全段]\n", "\n");
    assert_srs_item_violation(&w, &["nonfunctional の NFR3", "figures"]);
}
