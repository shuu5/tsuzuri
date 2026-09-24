//! `folio check` 自身の歯（便 0・docs/design/delivery-0.md §1 / §4）。
//! 正本 4 file で合格、tests/fixtures/check/ の 4 組で 不合格 1 / 不合格 1 / 不合格 1 / まだ分からない 2。
//! 各組は変異 1 つだけを持つ＝違反はちょうど 1 件で、その種類まで見る（別の理由で落ちた組を緑にしない）。
//! 要件書の図の節（便 34・FR15）は design-intent の写し（copy_tree + git init・tests/adr.rs の Work と同じ形）の
//! srs.yaml に図を 1 枚足して合格を見、その図に変異 1 つずつ（型・caption・refs・図の id の重複・未知の欄）で 不合格 1 を見る。
//! 憲法の条の値域を持つ欄（便 55・床の穴 f2-648.82）は同じ写しの constitution.yaml の条 P-1 に変異 1 つずつを当てて見る
//! （改訂の差分の範囲の外の欄 3 つは違反ちょうど 1 件・範囲の内の欄 2 つは値域の違反を含む・欄が無ければ黙る）。
//! 便 117（docs/design/delivery-117.md §1 (d)）: 要件書の最上位の節の閉じた一覧に M3 の範囲の節 scope_m3 を足した。f117_ の 3 本は
//! 写しに足した scope_m3 の節の合格・ほかの段の名の節の未知の節・実の生成区間の一覧の並びを見る（どれも数を pin しない）。

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

// ── 要件書・語彙・相談窓口の最上位の節は床の定数から（便 77 §1 (e) 6） ──

/// 実の置き場（生成区間を置いた 3 file）に folio check → 合格・未知の節 0 件。続けて写しの 3 file それぞれで、
/// file の側の schema.top_level に extras を足し、最上位に節 extras を足しても、床は定数の一覧で数える＝
/// 不合格 1・未知の節はちょうど 1 件でその file 名と extras を含む。
#[test]
fn f77_top_level_is_closed_on_the_three_files() {
    let out = folio_check(&repo_root().join("design-intent"));
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    let unknown: Vec<String> = violations(&out)
        .into_iter()
        .filter(|v| v.starts_with("[未知の節]"))
        .collect();
    assert!(unknown.is_empty(), "{unknown:?}");

    for (file, from, to) in [
        ("srs.yaml", "\n    - figures\n    - schema\n", "\n    - figures\n    - schema\n    - extras\n"),
        (
            "vocabulary.yaml",
            "\n  top_level: [terms, field_terms, identifiers, schema]\n",
            "\n  top_level: [terms, field_terms, identifiers, schema, extras]\n",
        ),
        (
            "intake.yaml",
            "\n  top_level: [meta, answers, targets, questions, sheet, schema]\n",
            "\n  top_level: [meta, answers, targets, questions, sheet, schema, extras]\n",
        ),
    ] {
        let w = Work::new(&format!("f77-extras-{file}"));
        let path = w.dir().join(file);
        mutate_file(&path, from, to);
        let before = fs::read_to_string(&path).unwrap();
        let sep = if before.ends_with('\n') { "" } else { "\n" };
        fs::write(&path, format!("{before}{sep}extras: 余分\n")).unwrap();
        let out = w.check();
        assert_eq!(
            out.status.code(),
            Some(1),
            "{file}: {}{}",
            stdout(&out),
            String::from_utf8_lossy(&out.stderr)
        );
        let unknown: Vec<String> = violations(&out)
            .into_iter()
            .filter(|v| v.starts_with("[未知の節]"))
            .collect();
        assert_eq!(unknown.len(), 1, "{file}: 未知の節はちょうど 1 件: {unknown:?}");
        assert!(unknown[0].starts_with(&format!("[未知の節] {file}")), "{unknown:?}");
        assert!(unknown[0].contains("extras"), "{unknown:?}");
        assert!(stdout(&out).contains("不合格"), "{}", stdout(&out));
    }
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
    "{kind: build-check, live: now, stage: post, polarity: fail-closed, note: 公開する命令";

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
        "{kind: mystery, live: now, stage: post, polarity: fail-closed, note: 公開する命令",
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
        "{kind: build-check, live: now, polarity: fail-closed, note: 公開する命令",
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

/// folio schema --help の --dir の説明が生成区間を持つ file 8 本の名を出す（便 77 §1 (d)）。
#[test]
fn r11_schema_help_names_the_schema_files() {
    let help = folio_help("schema");
    assert!(
        help.contains(
            "adr/schema.yaml・design-note/schema.yaml・ceiling.yaml・rules.yaml・index.yaml・srs.yaml・vocabulary.yaml・intake.yaml"
        ),
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
    const CLOSED: [&str; 14] = [
        "check", "inject", "parts", "face", "figure", "build", "derive", "intake", "hello", "ceiling",
        "schema", "graph", "init", "serve",
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

// ── 要件の行の欄の閉じた一覧は実装の定数が持つ（便 86・生成区間は写しで file の側から緩められない） ──

/// 便 86 (c) の生成区間の optional の行（写しの側の変異の当て先）。
const F86_REGION_OPTIONAL: &str = "\n    optional: [milestone, rules, adrs, note]\n";

#[test]
fn f86_unknown_field_cannot_be_loosened_from_the_file() {
    let w = Work::new("f86-loosen");
    let out = w.check();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(violations(&out).is_empty(), "{:?}", violations(&out));
    w.mutate(FR1_FIGURES, "\n    figures: [図2-1, 図2-2, 図1]\n    extras: 1\n");
    assert_srs_item_violation(&w, &["未知の欄", "extras"]);
    // 生成区間の optional に extras を足しても閉じた一覧は緩まない（N-3.1）
    w.mutate(
        F86_REGION_OPTIONAL,
        "\n    optional: [milestone, rules, adrs, note, extras]\n",
    );
    assert_srs_item_violation(&w, &["未知の欄", "extras"]);
}

#[test]
fn f86_verify_inner_list_is_still_checked() {
    let w = Work::new("f86-no-ac");
    w.mutate(
        FR1_VERIFY,
        "    verify: {method: test, how: 決まった回答 5 つを入れ、支度表が期待どおりか比較する}\n",
    );
    assert_srs_item_violation(&w, &["FR1 の verify の ac が無い（一覧・空でよい）"]);
}

// ── 要件の行の判断の記録の欄 adrs（便 90・ADR-13 決定 (3-b)（ア）） ──

/// 便 90 (g) の変異の当て先 = 写しの FR19 の行の adrs（実の要件書で ADR-9 を持つ行は 1 本だけ）。
/// 変異は ADR-9 を残す＝FR19 の散文が指す ADR-9 を外して行 R-17 の違反を足さない（便 93）。
// 変異の当て先は要件書で 1 か所しかない adrs の行（一括 12 で FR19 の行に ADR-13 が足されたため FR17 側へ移した）
const F90_FR19_ADRS: &str = "\n    adrs: [ADR-5]\n";

/// 写しの YAML の木を歩き、鍵 adrs を持つ表の（持ち主の id, adrs の値）を出てきた順に集める。
/// 持ち主の id が無い表（要件の行でない所）に adrs が在れば歯が落ちる。
fn f90_adrs_rows(node: &yaml_rust2::Yaml, out: &mut Vec<(String, yaml_rust2::Yaml)>) {
    use yaml_rust2::Yaml;
    match node {
        Yaml::Hash(h) => {
            if let Some(adrs) = h.get(&Yaml::String("adrs".into())) {
                let owner = h
                    .get(&Yaml::String("id".into()))
                    .and_then(Yaml::as_str)
                    .unwrap_or_else(|| panic!("id の無い表に adrs が在る: {adrs:?}"));
                out.push((owner.to_string(), adrs.clone()));
            }
            for v in h.values() {
                f90_adrs_rows(v, out);
            }
        }
        Yaml::Array(a) => a.iter().for_each(|v| f90_adrs_rows(v, out)),
        _ => {}
    }
}

#[test]
fn f90_the_real_srs_carries_the_adrs_field() {
    let w = Work::new("f90-real");
    let out = w.check();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(violations(&out).is_empty(), "{:?}", violations(&out));
    // adrs を持つ行の集合は写しの要件書そのものから読む（行の数は固定しない＝要件を足しても落ちない）。
    // 歯が見るのは性質＝欄を持つ行が 1 つ以上在り、各行の値が一覧で、各項が判断の記録の id の形で、
    // その正本 adr/<id>.yaml が写しに実在すること。
    let text = fs::read_to_string(w.srs()).unwrap();
    let doc = yaml_rust2::YamlLoader::load_from_str(&text).unwrap().remove(0);
    let mut rows = Vec::new();
    f90_adrs_rows(&doc, &mut rows);
    assert!(!rows.is_empty(), "実の要件書に adrs を持つ行が 1 つも無い");
    // 字面の走査で見つかる行の数と木の上の数が一致する（字面の形の違う adrs が見落とされない）。
    let lines = text.lines().filter(|l| l.trim_start().starts_with("adrs:")).count();
    assert_eq!(lines, rows.len(), "字面の adrs の行と木の上の adrs の行が食い違う: {rows:?}");
    for (owner, adrs) in &rows {
        let items = adrs
            .as_vec()
            .unwrap_or_else(|| panic!("{owner} の adrs が一覧でない: {adrs:?}"));
        assert!(!items.is_empty(), "{owner} の adrs が空");
        for item in items {
            let id = item
                .as_str()
                .unwrap_or_else(|| panic!("{owner} の adrs の項が字でない: {item:?}"));
            let num = id
                .strip_prefix("ADR-")
                .unwrap_or_else(|| panic!("{owner}: ADR- で始まらない: {id}"));
            assert!(
                num.starts_with(|c: char| matches!(c, '1'..='9'))
                    && num.chars().all(|c| c.is_ascii_digit()),
                "{owner}: 判断の記録の id の形でない: {id}"
            );
            assert!(
                w.dir().join(format!("adr/{id}.yaml")).is_file(),
                "{owner}: {id} の正本が無い"
            );
        }
    }
}

#[test]
fn f90_an_id_that_is_not_an_adr_is_a_violation() {
    let w = Work::new("f90-not-adr");
    w.mutate(F90_FR19_ADRS, "\n    adrs: [ADR-5, FR5]\n");
    assert_srs_item_violation(&w, &["FR16", "adrs", "FR5", "判断の記録の id の形でない"]);
}

#[test]
fn f90_an_adr_that_does_not_exist_is_a_violation() {
    let w = Work::new("f90-missing-adr");
    w.mutate(F90_FR19_ADRS, "\n    adrs: [ADR-5, ADR-99]\n");
    assert_srs_item_violation(&w, &["adrs", "ADR-99", "が実在しない"]);
}

#[test]
fn f90_adrs_that_is_not_a_list_is_a_violation() {
    let w = Work::new("f90-not-list");
    w.mutate(F90_FR19_ADRS, "\n    adrs: ADR-5\n");
    assert_srs_item_violation(&w, &["FR16", "adrs が一覧でない"]);
}

// ── 規則の表の行の条以外を指す欄 refs（便 91・ADR-13 決定 (3-b)（イ）） ──

/// 便 91 (g) の変異の当て先 = 写しの行 R-4 の refs（実の rules.yaml でちょうど 1 か所）。
/// 変異は AC6 を残す＝R-4 の散文が指す AC6 を外して行 R-17 の違反を足さない（便 93）。
const F91_R4_REFS: &str = ", refs: [AC6]}";

/// 便 91 (b) の書き写した 27 対（行 → refs）と、便 93 (d) が R-16 に足した 2 対。
const F91_ROWS: [(&str, &[&str]); 15] = [
    ("R-1", &["P-4.2", "P-6.3", "D-3"]),
    ("R-3", &["P-4.2", "AC2"]),
    ("R-4", &["AC6"]),
    ("R-5", &["P-2.4"]),
    // R-12 の R-9 は一括 17（天井の 30 周目 実態 F-2）で足した（この行を数える口は R-9 の床）
    ("R-12", &["P-4.2", "R-2", "R-9"]),
    ("R-13", &["P-4.2"]),
    ("R-14", &["P-4.1", "P-11.1", "N-3.1", "R-7"]),
    ("R-15", &["P-10.3", "A-3.1", "CON2", "ADR-4"]),
    // R-9 / R-12 は便 93 (d) の書き写し（母集団の文が名指す境界）
    ("R-16", &["P-6.3", "P-10.1", "R-9", "R-12", "ADR-3"]),
    ("R-17", &["P-4.2", "P-5.6", "ADR-13"]),
    ("D-3", &["R-1"]),
    ("D-10", &["P-5.2", "R-8"]),
    ("D-11", &["P-5.6"]),
    ("D-12", &["ADR-8", "ADR-13", "ADR-18", "D-14", "ADR-19"]),
    // D-14 は判断の記録 ADR-19 の発効で新設（2026-09-24）
    (
        "D-14",
        &["P-3", "P-12", "R-7", "D-12", "ADR-8", "ADR-13", "ADR-18", "ADR-19"],
    ),
];

/// 写しの rules.yaml に変異を当てた結果が 不合格 1・違反はちょうど 1 件（rules.yaml の場所）で `words` を全部含む。
fn assert_rules_violation(w: &Work, words: &[&str]) {
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
    assert!(v[0].contains("rules.yaml"), "{v:?}");
    for word in words {
        assert!(v[0].contains(word), "「{word}」が無い: {v:?}");
    }
}

#[test]
fn f91_the_real_rules_carry_the_refs_field() {
    let w = Work::new("f91-real");
    let out = w.check();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(violations(&out).is_empty(), "{:?}", violations(&out));
    let text = fs::read_to_string(w.rules()).unwrap();
    assert_eq!(text.matches("refs: [").count(), 15, "refs の行の数");
    let mut total = 0;
    for line in text.lines().filter(|l| l.contains("refs: [")) {
        let id = line
            .strip_prefix("  - {id: ")
            .and_then(|l| l.split(',').next())
            .unwrap_or_else(|| panic!("行の形でない: {line}"));
        let (_, want) = F91_ROWS
            .iter()
            .find(|(row, _)| *row == id)
            .unwrap_or_else(|| panic!("表に無い行: {id}"));
        let body = line
            .rsplit_once(", refs: [")
            .and_then(|(_, b)| b.strip_suffix("]}"))
            .unwrap_or_else(|| panic!("行の末尾の一覧でない: {id}"));
        let got: Vec<&str> = body.split(", ").collect();
        assert_eq!(got, *want, "{id} の refs");
        total += got.len();
    }
    assert_eq!(total, 44, "refs の id の合計");
}

#[test]
fn f91_a_value_that_is_not_an_id_is_a_violation() {
    let w = Work::new("f91-not-id");
    w.mutate_rules(F91_R4_REFS, ", refs: [AC6, xyz]}");
    assert_rules_violation(&w, &["R-4", "refs", "xyz", "id の形でない"]);
}

#[test]
fn f91_the_article_of_the_row_is_a_violation() {
    let w = Work::new("f91-article");
    w.mutate_rules(F91_R4_REFS, ", refs: [AC6, P-5]}");
    assert_rules_violation(&w, &["R-4", "refs", "P-5", "自分の id か article の条である"]);
}

#[test]
fn f91_refs_that_is_not_a_list_is_a_violation() {
    let w = Work::new("f91-not-list");
    w.mutate_rules(F91_R4_REFS, ", refs: AC6}");
    assert_rules_violation(&w, &["R-4", "refs が一覧でない"]);
}

#[test]
fn f91_an_id_that_does_not_exist_is_a_violation() {
    let w = Work::new("f91-missing");
    w.mutate_rules(F91_R4_REFS, ", refs: [AC6, AC99]}");
    assert_rules_violation(&w, &["thresholds[3].refs[1]", "AC99", "実在しない"]);
}

// ── 規則の表の行 R-17 の床の歯（便 93・散文の言及は型付きの欄にも在ること） ──

/// 便 93 (h) の変異の当て先 = 写しの行 R-16 の refs（実の rules.yaml でちょうど 1 か所）。
const F93_R16_REFS: &str = "refs: [P-6.3, P-10.1, R-9, R-12, ADR-3]";
/// 当て先から R-9 を外した字。
const F93_R16_WITHOUT_R9: &str = "refs: [P-6.3, P-10.1, R-12, ADR-3]";

/// 写しの判断の記録の basis の一覧（`basis:` の次の行から一覧の終わりまで）。
fn f93_basis(w: &Work, id: &str) -> Vec<String> {
    let text = fs::read_to_string(w.dir().join(format!("adr/{id}.yaml"))).unwrap();
    let (_, rest) = text
        .split_once("\nbasis:\n")
        .unwrap_or_else(|| panic!("{id} に basis が無い"));
    rest.lines()
        .map_while(|l| l.strip_prefix("  - "))
        .map(str::to_string)
        .collect()
}

#[test]
fn f93_the_real_sources_leave_no_prose_edge() {
    let w = Work::new("f93-real");
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
    let basis = f93_basis(&w, "ADR-13");
    for id in ["P-11", "P-12", "P-14", "P-17", "A-2"] {
        assert!(basis.iter().any(|b| b == id), "ADR-13 の basis に {id} が無い: {basis:?}");
    }
    let rules = fs::read_to_string(w.rules()).unwrap();
    let row = rules
        .lines()
        .find(|l| l.starts_with("  - {id: R-16,"))
        .expect("R-16 の行が無い");
    assert!(row.ends_with(&format!(", {F93_R16_REFS}}}")), "R-16 の refs: {row}");
}

#[test]
fn f93_a_prose_id_outside_the_typed_fields_is_a_violation() {
    let w = Work::new("f93-violation");
    w.mutate_rules(F93_R16_REFS, F93_R16_WITHOUT_R9);
    let out = w.check();
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    let v = violations(&out);
    assert_eq!(v.len(), 1, "違反は変異の 1 件だけのはず: {v:?}");
    assert!(v[0].starts_with("[R-17] "), "{v:?}");
    for word in ["R-16", "R-9", "型付きの欄"] {
        assert!(v[0].contains(word), "「{word}」が無い: {v:?}");
    }
}

#[test]
fn f93_an_excluded_phrase_is_not_counted() {
    let w = Work::new("f93-excluded");
    w.mutate_rules(F93_R16_REFS, F93_R16_WITHOUT_R9);
    w.mutate_rules("（R-9 / R-12 の領分）", "（R-9 / R-12 の領分・対象外）");
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
fn f93_a_kind_without_a_receptacle_is_not_counted() {
    let w = Work::new("f93-receptacle");
    let out = w.check();
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    assert!(violations(&out).is_empty(), "{:?}", violations(&out));
    let text = fs::read_to_string(w.srs()).unwrap();
    // 要件 → 要件 は受け皿の表に無い組なので、散文が指していても数えない。
    // 当て先は実の正本で互いの注が相手を指す 2 組（一括 12 で FR12 の注の二重の記述を外したので移した）。
    for (owner, id) in [("FR3", "FR21"), ("FR21", "FR3")] {
        let (_, rest) = text
            .split_once(&format!("\n  - id: {owner}\n"))
            .unwrap_or_else(|| panic!("{owner} の行が無い"));
        let row = rest.split("\n  - id: ").next().unwrap();
        let note = row
            .lines()
            .find(|l| l.starts_with("    note: "))
            .unwrap_or_else(|| panic!("{owner} の note が無い"));
        assert!(note.contains(id), "{owner} の note に {id} が無い");
        for key in ["basis", "verify", "goals", "figures"] {
            let typed = row
                .lines()
                .find(|l| l.starts_with(&format!("    {key}: ")))
                .unwrap_or_else(|| panic!("{owner} の {key} が無い"));
            assert!(!typed.contains(id), "{owner} の {key} に {id} が在る: {typed}");
        }
    }
}

#[test]
fn f93_deleting_the_rule_row_is_not_a_silent_escape() {
    let w = Work::new("f93-row-deleted");
    w.mutate_rules(F93_R16_REFS, F93_R16_WITHOUT_R9);
    let before = fs::read_to_string(w.rules()).unwrap();
    let kept: Vec<&str> = before
        .lines()
        .filter(|l| !l.starts_with("  - {id: R-17,"))
        .collect();
    assert_eq!(before.lines().count(), kept.len() + 1, "R-17 の行が 1 行でない");
    fs::write(w.rules(), format!("{}\n", kept.join("\n"))).unwrap();
    let out = w.check();
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    // 違反は全部 R-17 の未解決で、種別は 参照 id（憲法・要件書・語彙が R-17 を名指す所）か adr（判断の記録の
    // 型付きの欄が R-17 を名指す所）だけ。件数は固定しない（R-17 を名指す記録が増えれば増える）。
    // 代わりに、違反が指す所が写しの記録に実在し、その値が R-17 を名指していることを 1 件ずつ照合する。
    // 歯そのものは黙る
    let v = violations(&out);
    assert!(
        v.iter().all(|l| l.contains("id R-17 が実在しない")),
        "違反は R-17 の未解決だけのはず: {v:?}"
    );
    assert!(
        v.iter()
            .all(|l| l.starts_with("[参照 id] ") || l.starts_with("[adr] ADR-")),
        "違反の種別は 参照 id か adr のはず: {v:?}"
    );
    assert!(
        v.iter().any(|l| l.starts_with("[参照 id] ")),
        "参照 id の違反が 1 件も無い（行を消すことが黙って通った）: {v:?}"
    );
    for l in &v {
        let (file, path) = f93_located(l);
        let text = fs::read_to_string(w.dir().join(&file))
            .unwrap_or_else(|e| panic!("違反が指す file {file} が写しに無い（{e}）: {l}"));
        let doc = yaml_rust2::YamlLoader::load_from_str(&text).unwrap().remove(0);
        let node = f93_node(&doc, &path).unwrap_or_else(|| panic!("違反が指す所 {path} が {file} に無い: {l}"));
        assert!(f93_text(node).contains("R-17"), "違反が指す所の値が R-17 を名指さない: {l}");
    }
    // 憲法の条の関係の欄（relations.rules・型付き）が R-17 を名指す項は、写しの憲法から数えた数と同じだけ違反になる。
    let constitution = fs::read_to_string(w.constitution()).unwrap();
    let constitution = yaml_rust2::YamlLoader::load_from_str(&constitution).unwrap().remove(0);
    let named = constitution["articles"]
        .as_vec()
        .expect("憲法の articles が一覧でない")
        .iter()
        .filter_map(|a| a["relations"]["rules"].as_vec())
        .flatten()
        .filter(|r| r.as_str() == Some("R-17"))
        .count();
    let relation = |l: &&String| l.starts_with("[参照 id] constitution.yaml: ") && l.contains(".relations.rules");
    assert_eq!(v.iter().filter(relation).count(), named, "条の関係の欄の違反: {v:?}");
    assert!(!v.iter().any(|l| l.starts_with("[R-17]")), "{v:?}");
}

/// 違反の行が指す（写しの置き場からの file, file の中の道筋）。
/// `[参照 id] <file>: <道筋>: <文>` と `[adr] ADR-<n>.<道筋>: <文>` の 2 つの形を読む。
fn f93_located(line: &str) -> (String, String) {
    if let Some(rest) = line.strip_prefix("[参照 id] ") {
        let (file, rest) = rest.split_once(": ").unwrap_or_else(|| panic!("形が違う: {line}"));
        let (path, _) = rest.split_once(": ").unwrap_or_else(|| panic!("形が違う: {line}"));
        (file.to_string(), path.to_string())
    } else if let Some(rest) = line.strip_prefix("[adr] ") {
        let (at, _) = rest.split_once(": ").unwrap_or_else(|| panic!("形が違う: {line}"));
        let (id, path) = at.split_once('.').unwrap_or_else(|| panic!("形が違う: {line}"));
        (format!("adr/{id}.yaml"), path.to_string())
    } else {
        panic!("種別が違う: {line}")
    }
}

/// YAML の木で道筋 `a.b[3].c` の先の節（無ければ None）。
fn f93_node<'a>(doc: &'a yaml_rust2::Yaml, path: &str) -> Option<&'a yaml_rust2::Yaml> {
    let mut node = doc;
    for seg in path.split('.') {
        let (key, mut rest) = seg.split_once('[').map_or((seg, ""), |(k, r)| (k, r));
        if !key.is_empty() {
            node = node.as_hash()?.get(&yaml_rust2::Yaml::String(key.into()))?;
        }
        while !rest.is_empty() {
            let (idx, after) = rest.split_once(']')?;
            node = node.as_vec()?.get(idx.parse::<usize>().ok()?)?;
            rest = after.strip_prefix('[').unwrap_or(after);
        }
    }
    Some(node)
}

/// 節の字面（字ならそのまま・それ以外は YAML に書き出す）。
fn f93_text(node: &yaml_rust2::Yaml) -> String {
    if let Some(s) = node.as_str() {
        return s.to_string();
    }
    let mut out = String::new();
    yaml_rust2::YamlEmitter::new(&mut out).dump(node).unwrap();
    out
}

// ── 要件書の最上位の節の閉じた一覧に M3 の範囲の節 scope_m3（便 117・ADR-16 決定 (1)(7)①） ──

/// 見本の節の中身（scope_m1 と同じ形＝build と not_build の一覧に 1 行ずつ）。名は歯が前に付ける。
const F117_SECTION_BODY: &str = ":\n  build:\n    - 見本の範囲\n  not_build:\n    - 見本の範囲の外\n";

/// 写しの srs.yaml の最上位の節 actors の直前に、名 name の見本の節を足す。
fn f117_with_section(w: &Work, name: &str) {
    w.mutate("\nactors:\n", &format!("\n{name}{F117_SECTION_BODY}actors:\n"));
}

/// 要件書の生成区間（印 2 本の間）の schema.top_level の一覧の項。
fn f117_region_top_level(text: &str) -> Vec<String> {
    let begin = text.find("# folio:schema:begin").expect("生成区間の begin が無い");
    let end = text.find("# folio:schema:end").expect("生成区間の end が無い");
    let region = &text[begin..end];
    let doc = yaml_rust2::YamlLoader::load_from_str(region).unwrap().remove(0);
    doc["schema"]["top_level"]
        .as_vec()
        .expect("生成区間の top_level が一覧でない")
        .iter()
        .map(|v| v.as_str().expect("top_level の項が字でない").to_string())
        .collect()
}

/// scope_m3 の節を持つ要件書の写しは床を通る（実の要件書が既に節を持つときは足さずに撃つ）。
#[test]
fn f117_srs_with_the_scope_m3_section_passes() {
    let w = Work::new("f117-scope-m3");
    let before = fs::read_to_string(w.srs()).unwrap();
    if !before.lines().any(|l| l.starts_with("scope_m3:")) {
        f117_with_section(&w, "scope_m3");
    }
    let after = fs::read_to_string(w.srs()).unwrap();
    assert_eq!(
        after.lines().filter(|l| l.starts_with("scope_m3:")).count(),
        1,
        "最上位の scope_m3 の鍵はちょうど 1 つ"
    );
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

/// ほかの段の名の節（scope_m2・scope_m4・scope_m）は 1 つずつ未知の節のまま落ちる。
#[test]
fn f117_other_stage_sections_are_still_unknown() {
    for name in ["scope_m2", "scope_m4", "scope_m"] {
        let w = Work::new(&format!("f117-{name}"));
        f117_with_section(&w, name);
        let out = w.check();
        assert_eq!(
            out.status.code(),
            Some(1),
            "{name}: {}{}",
            stdout(&out),
            String::from_utf8_lossy(&out.stderr)
        );
        let unknown: Vec<String> = violations(&out)
            .into_iter()
            .filter(|v| v.starts_with("[未知の節]"))
            .collect();
        assert_eq!(unknown.len(), 1, "{name}: 未知の節はちょうど 1 件: {unknown:?}");
        assert!(unknown[0].starts_with("[未知の節] srs.yaml"), "{unknown:?}");
        assert!(unknown[0].contains(&format!("「{name}」")), "{unknown:?}");
        assert!(stdout(&out).contains("不合格"), "{}", stdout(&out));
    }
}

/// 実の要件書の生成区間の一覧は scope_m3 をちょうど 1 回 scope_m1 の直後に持ち、実の最上位の節はどれも一覧に在る。
#[test]
fn f117_the_real_region_lists_scope_m3_after_scope_m1() {
    let text = fs::read_to_string(repo_root().join("design-intent/srs.yaml")).unwrap();
    let listed = f117_region_top_level(&text);
    assert_eq!(
        listed.iter().filter(|k| *k == "scope_m3").count(),
        1,
        "scope_m3 はちょうど 1 回: {listed:?}"
    );
    let at = listed.iter().position(|k| k == "scope_m1").expect("一覧に scope_m1 が無い");
    assert_eq!(listed.get(at + 1).map(String::as_str), Some("scope_m3"), "{listed:?}");
    let doc = yaml_rust2::YamlLoader::load_from_str(&text).unwrap().remove(0);
    let real: Vec<String> = doc
        .as_hash()
        .expect("要件書の最上位が表でない")
        .keys()
        .map(|k| k.as_str().expect("最上位の鍵が字でない").to_string())
        .collect();
    assert!(!real.is_empty(), "要件書の最上位の節が無い");
    for key in &real {
        assert!(listed.contains(key), "最上位の節「{key}」が一覧に無い: {listed:?}");
    }
}
