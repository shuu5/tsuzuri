//! `folio check` の設計ノートの正本（design-note/*.yaml）の検査の歯（便 23・docs/design/delivery-23.md §1 (d)）。
//! 歯の中で design-intent の写し全部を一時 dir の `design-intent/` に、器（scribe2）の導出 file を写しの根の
//! `contracts/` に作り、git init と 1 commit を行ってから（版管理の無い写しは別の理由で「まだ分からない」になる）、
//! 設計ノートの file を置く・変異を 1 つ当てて `folio check --dir` を回す。
//! 違反の歯は、変異が 1 つなら違反の件数が 1 であること（出力の件数の表示）も確かめる。
//! 凍結 fixture は要件書 AC7 / AC8 / AC13 の red_test が名指す tests/fixtures/design-note/ の 3 組だけ。

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

/// 写しの一時 dir（歯の終わりに消す）。
struct Work {
    root: PathBuf,
}

impl Work {
    fn new(case: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-note-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_tree(
            &repo_root().join("design-intent"),
            &root.join("design-intent"),
        );
        // 器（scribe2）の導出 file は正本の置き場の親 dir の contracts/ に在る
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

    fn external(&self) -> PathBuf {
        self.root.join("contracts/schema.toml")
    }

    /// 凍結 fixture の file を設計ノートの置き場（か写しの根の contracts/）へ置く。
    fn put(&self, group: &str, file: &str, to: &Path) {
        let from = repo_root()
            .join("tests/fixtures/design-note")
            .join(group)
            .join(file);
        fs::copy(&from, to).unwrap_or_else(|e| panic!("{}: 置けない: {e}", from.display()));
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

    /// 見本 example.yaml の字面の変異（1 か所だけ）。
    fn mutate(&self, from: &str, to: &str) {
        self.mutate_file(&self.note("example.yaml"), from, to);
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

/// 不合格 1・違反はちょうど 1 件（件数の表示も 1）・その 1 件が種別 `kind` で `words` を全部含む。
fn assert_single_violation(out: &Output, kind: &str, words: &[&str]) {
    assert_eq!(out.status.code(), Some(1), "{}{}", stdout(out), stderr(out));
    let v = violations(out);
    assert_eq!(v.len(), 1, "違反は変異の 1 件だけのはず: {v:?}");
    assert!(v[0].starts_with(&format!("[{kind}] design-note/")), "{v:?}");
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

// ── 写しそのまま ──

#[test]
fn note_canonical_copy_passes() {
    let w = Work::new("canonical");
    assert_passes(&w.check());
}

// ── AC7（節の型の閉じた一覧） ──

#[test]
fn note_section_types_ok_fixture_passes() {
    let w = Work::new("types-ok");
    w.put("section-types", "ok.yaml", &w.note("ok.yaml"));
    assert_passes(&w.check());
}

#[test]
fn note_section_types_unknown_fixture_fails() {
    let w = Work::new("types-unknown");
    w.put(
        "section-types",
        "unknown-type.yaml",
        &w.note("unknown-type.yaml"),
    );
    assert_single_violation(&w.check(), "note", &["節の型「mystery」が一覧に無い"]);
}

// ── AC8（器の導出 file が無い） ──

#[test]
fn note_missing_external_schema_is_unknown() {
    let w = Work::new("schema-missing");
    w.put("schema-missing", "note.yaml", &w.note("note.yaml"));
    fs::remove_file(w.external()).unwrap();
    assert_unknown(&w.check(), "器の導出 file が読めない");
}

/// 契約表の節を持つ設計ノートが 1 本も無ければ導出 file は読まない（無くても合格）。
#[test]
fn note_missing_external_schema_without_contract_table_passes() {
    let w = Work::new("schema-missing-no-table");
    fs::remove_file(w.note("example.yaml")).unwrap();
    fs::remove_file(w.external()).unwrap();
    assert_passes(&w.check());
}

// ── AC13（器の導出 file の欄 1 つ） ──

#[test]
fn note_external_schema_plus_one_field_passes() {
    let w = Work::new("plus-one");
    w.put("schema-plus-one", "note.yaml", &w.note("note.yaml"));
    w.put("schema-plus-one", "schema.toml", &w.external());
    assert_passes(&w.check());
}

#[test]
fn note_extra_contract_field_without_derived_field_fails() {
    let w = Work::new("plus-one-missing");
    w.put("schema-plus-one", "note.yaml", &w.note("note.yaml"));
    assert_single_violation(
        &w.check(),
        "note",
        &["契約表の欄「extra」が器の導出 file に無い"],
    );
}

// ── 形の違反（見本の写しに変異 1 つずつ） ──

#[test]
fn note_status_not_in_enum_fails() {
    let w = Work::new("status");
    w.mutate("\n  status: example\n", "\n  status: nope\n");
    assert_single_violation(&w.check(), "note", &["meta.status「nope」が一覧に無い"]);
}

#[test]
fn note_id_not_matching_the_file_stem_fails() {
    let w = Work::new("id-stem");
    w.mutate("\n  id: example\n", "\n  id: sample\n");
    assert_single_violation(
        &w.check(),
        "note",
        &["meta.id「sample」が file 名の stem「example」と違う"],
    );
}

#[test]
fn note_section_number_not_ascending_fails() {
    let w = Work::new("section-n");
    w.mutate("  - n: 2\n", "  - n: 1\n");
    assert_single_violation(&w.check(), "note", &["§1", "n が前の節（1）より大きくない"]);
}

#[test]
fn note_prose_section_with_rows_fails() {
    let w = Work::new("prose-rows");
    w.mutate("  - n: 2\n", "    rows: []\n  - n: 2\n");
    assert_single_violation(
        &w.check(),
        "note",
        &["§1", "節の型 prose に置けない欄「rows」が在る"],
    );
}

#[test]
fn note_parts_row_without_role_fails() {
    let w = Work::new("parts-role");
    w.mutate(
        "{id: tool, name: 図の道具（archify）, role: 検査と描画を掛ける組み立て器（子の処理）, ref:",
        "{id: tool, name: 図の道具（archify）, ref:",
    );
    assert_single_violation(&w.check(), "欄の非空", &["§2 の行 tool の role が空"]);
}

#[test]
fn note_fields_row_need_not_in_enum_fails() {
    let w = Work::new("fields-need");
    w.mutate("name: 図の型, need: required", "name: 図の型, need: maybe");
    assert_single_violation(
        &w.check(),
        "note",
        &["§4 の行 type", "need「maybe」が一覧に無い"],
    );
}

#[test]
fn note_contract_row_section_not_prose_fails() {
    let w = Work::new("contract-section");
    w.mutate("section: \"1\"", "section: \"2\"");
    assert_single_violation(
        &w.check(),
        "note",
        &[
            "§6 の行 a",
            "section「2」が同じ文書の prose の節の n でない",
        ],
    );
}

#[test]
fn note_contract_row_req_not_a_requirement_fails() {
    let w = Work::new("contract-req");
    w.mutate("req: [FR15], section:", "req: [FR99], section:");
    assert_single_violation(
        &w.check(),
        "note",
        &["§6 の行 a の req[0]", "id「FR99」が実在しない"],
    );
}

#[test]
fn note_row_ref_not_a_known_id_fails() {
    let w = Work::new("row-ref");
    w.mutate("ref: [P-10, R-15]", "ref: [P-10, ADR-99]");
    assert_single_violation(
        &w.check(),
        "note",
        &["§2 の行 anchor の ref[1]", "id「ADR-99」が実在しない"],
    );
}

#[test]
fn note_figure_type_not_in_the_catalog_fails() {
    let w = Work::new("figure-type");
    w.mutate(
        "\n    type: archify-architecture\n",
        "\n    type: mystery\n",
    );
    assert_single_violation(
        &w.check(),
        "note",
        &[
            "figures の fig-1",
            "図の型「mystery」が部品目録の一覧に無い",
        ],
    );
}

// ── まだ分からない ──

#[test]
fn note_sections_not_a_list_is_unknown() {
    let w = Work::new("sections-type");
    let path = w.note("example.yaml");
    let before = fs::read_to_string(&path).unwrap();
    let start = before.find("\nsections:\n").expect("sections の節が無い");
    let end = before.find("\nfigures:\n").expect("figures の節が無い");
    let after = format!("{}\nsections: 節{}", &before[..start], &before[end..]);
    fs::write(&path, after).unwrap();
    assert_unknown(&w.check(), "sections が表の一覧でない");
}

#[test]
fn note_external_schema_with_another_head_is_unknown() {
    let w = Work::new("external-head");
    w.mutate_file(&w.external(), "\nschema = 1\n", "\nschema = 2\n");
    assert_unknown(&w.check(), "先頭が「schema = 1」でない");
}

#[test]
fn note_symlinked_note_is_unknown() {
    let w = Work::new("symlink");
    let path = w.note("example.yaml");
    let outside = w.root.join("outside-example.yaml");
    fs::rename(&path, &outside).unwrap();
    std::os::unix::fs::symlink(&outside, &path).unwrap();
    assert_unknown(&w.check(), "design-note/example.yaml: symlink は認めない");
}

// ── 承認欄 ──

#[test]
fn note_effective_without_approval_fails() {
    let w = Work::new("effective");
    w.mutate("\n  status: example\n", "\n  status: effective\n");
    assert_single_violation(
        &w.check(),
        "note",
        &["status effective なのに approval（承認欄）が無い"],
    );
}

#[test]
fn note_effective_with_approval_passes() {
    let w = Work::new("approval");
    w.mutate(
        "\n  status: example\n",
        "\n  status: effective\n  approval:\n    - {who: 持ち主, date: 2026-09-18, ruling: f2-648.37 notes, verbatim: 承認する, surface: R-8}\n",
    );
    assert_passes(&w.check());
}

#[test]
fn note_example_with_approval_fails() {
    let w = Work::new("example-approval");
    w.mutate(
        "\n  status: example\n",
        "\n  status: example\n  approval:\n    - {who: 持ち主, date: 2026-09-18, ruling: f2-648.37 notes, verbatim: 承認する, surface: R-8}\n",
    );
    assert_single_violation(
        &w.check(),
        "note",
        &["status example は approval（承認欄）を持たない"],
    );
}

// ── 欄の決まりの写し ──

#[test]
fn note_schema_copy_drift_fails() {
    let w = Work::new("schema-drift");
    w.mutate_file(
        &w.note("schema.yaml"),
        "teeth-table, contract-table]",
        "teeth-table, contract-table, extra-type]",
    );
    assert_single_violation(
        &w.check(),
        "note",
        &["design-note/schema.yaml", "床の定数と違う", "type_enum"],
    );
}
