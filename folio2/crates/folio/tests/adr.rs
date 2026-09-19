//! `folio check` の判断の記録（adr/）の欄の決まりの歯（便 5・docs/design/delivery-5.md §1）。
//! tests/fixtures/adr/ の 3 組（違反 0 の最小の手書き 4 file + adr/schema.yaml + adr/ADR-1.yaml に変異 1 つ）で 不合格 1。
//! 各組の違反はちょうど 1 件で、その種類と場所と文言まで見る（別の理由で落ちた組を緑にしない）。
//! 図の節（便 33）は design-intent の写し（copy_tree + git init・tests/note.rs の Work と同じ形）の ADR-1.yaml に
//! 図を 1 枚足して合格を見、その図に変異 1 つずつ（型・caption・refs・図の id の重複）で 不合格 1 を見る。

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

/// 見本の図 1 枚（型 archify-architecture・欄そろい・refs は要件の id）。実の ADR-1.yaml の末尾に足す。
const FIGURE: &str = "figures:
  - id: fig-1
    type: archify-architecture
    caption: 見本の図
    refs: [FR15]
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
        let root = std::env::temp_dir().join(format!("folio-adr-{case}-{}", std::process::id()));
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

    /// 実の ADR-1.yaml の写し。
    fn adr1(&self) -> PathBuf {
        self.dir().join("adr/ADR-1.yaml")
    }

    /// 写しの ADR-1.yaml の末尾に図 1 枚を足す。
    fn with_figure(&self) {
        let before = fs::read_to_string(self.adr1()).unwrap();
        assert!(!before.contains("\nfigures:"), "実の ADR-1 が既に図を持つ");
        fs::write(self.adr1(), format!("{before}{FIGURE}")).unwrap();
    }

    /// 写しの ADR-1.yaml の字面の変異（1 か所だけ）。
    fn mutate(&self, from: &str, to: &str) {
        let before = fs::read_to_string(self.adr1()).unwrap();
        assert_eq!(
            before.matches(from).count(),
            1,
            "変異の当て先が 1 か所でない: {from:?}"
        );
        fs::write(self.adr1(), before.replacen(from, to, 1)).unwrap();
    }

    fn check(&self) -> Output {
        folio_check(&self.dir())
    }
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

fn assert_single_violation(name: &str, kind: &str, at: &str, needle: &str) {
    let out = folio_check(&repo_root().join("tests/fixtures/adr").join(name));
    assert_eq!(
        out.status.code(),
        Some(1),
        "{name}: {}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    let v = violations(&out);
    assert_eq!(v.len(), 1, "{name}: 違反は変異の 1 件だけのはず: {v:?}");
    assert!(
        v[0].starts_with(&format!("[{kind}] {at}")) && v[0].contains(needle),
        "{name}: {v:?}"
    );
    assert!(stdout(&out).contains("不合格"), "{name}");
}

#[test]
fn adr_schema_drift_fails() {
    assert_single_violation("schema-drift", "adr", "adr/schema.yaml", "options_rule.min");
}

#[test]
fn adr_two_adopted_fails() {
    assert_single_violation("two-adopted", "adr", "ADR-1", "採用の案");
}

#[test]
fn adr_effective_without_approval_fails() {
    assert_single_violation("effective-no-approval", "N-4", "ADR-1", "approval");
}

// ── 図の節（便 33） ──

/// 写しの ADR-1 に変異を当てた結果が 不合格 1・違反はちょうど 1 件（種別 adr・ADR-1 の場所）で `words` を全部含む。
fn assert_figure_violation(w: &Work, words: &[&str]) {
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
    assert!(v[0].starts_with("[adr] ADR-1"), "{v:?}");
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
fn adr_figure_with_all_fields_passes() {
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

/// 部品目録の図の型の一覧は 8 型（pipeline-rail 等の既存 3 型を含む）なので、床は一覧の外の字だけを落とす。
#[test]
fn adr_figure_type_outside_the_catalog_fails() {
    let w = Work::new("figure-type");
    w.with_figure();
    w.mutate(
        "\n    type: archify-architecture\n",
        "\n    type: mystery\n",
    );
    assert_figure_violation(&w, &["図の型「mystery」が部品目録の一覧に無い"]);
}

#[test]
fn adr_figure_without_caption_fails() {
    let w = Work::new("figure-caption");
    w.with_figure();
    w.mutate("\n    caption: 見本の図\n", "\n");
    assert_figure_violation(&w, &["必須欄が無い", "caption"]);
}

#[test]
fn adr_figure_ref_not_an_id_fails() {
    let w = Work::new("figure-refs");
    w.with_figure();
    w.mutate("\n    refs: [FR15]\n", "\n    refs: [FR]\n");
    assert_figure_violation(&w, &["refs「FR」が id の形"]);
}

#[test]
fn adr_figure_ids_must_be_unique_fails() {
    let w = Work::new("figure-dup-id");
    w.with_figure();
    // 同じ図をもう 1 枚（id も同じ）
    let second = FIGURE.strip_prefix("figures:\n").unwrap().to_string();
    let before = fs::read_to_string(w.adr1()).unwrap();
    fs::write(w.adr1(), format!("{before}{second}")).unwrap();
    assert_figure_violation(&w, &["図の id「fig-1」が重複"]);
}
