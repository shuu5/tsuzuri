//! `folio check` 自身の歯（便 0・docs/design/delivery-0.md §1 / §4）。
//! 正本 4 file で合格、tests/fixtures/check/ の 4 組で 不合格 1 / 不合格 1 / 不合格 1 / まだ分からない 2。
//! 各組は変異 1 つだけを持つ＝違反はちょうど 1 件で、その種類まで見る（別の理由で落ちた組を緑にしない）。
//! 要件書の図の節（便 34・FR15）は design-intent の写し（copy_tree + git init・tests/adr.rs の Work と同じ形）の
//! srs.yaml に図を 1 枚足して合格を見、その図に変異 1 つずつ（型・caption・refs・図の id の重複・未知の欄）で 不合格 1 を見る。

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

/// 実の rules.yaml の写しの schema.top_level の行（変異の当て先）。
const RULES_TOP_LEVEL_LINE: &str =
    "\n  top_level: [schema, thresholds, discipline]   # 未知の節は検査で落とす（N-3）";

/// file の側で schema.top_level に extras を足し、最上位に節 extras を足しても、床は定数の一覧で数える＝
/// 不合格 1・違反は「未知の節」の 1 件だけで extras を含む（本便の前の main では合格してしまう歯）。
#[test]
fn check_rules_section_added_via_the_file_top_level_is_still_unknown() {
    let w = Work::new("rules-extras");
    w.mutate_rules(
        RULES_TOP_LEVEL_LINE,
        "\n  top_level: [schema, thresholds, discipline, extras]   # 未知の節は検査で落とす（N-3）",
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
