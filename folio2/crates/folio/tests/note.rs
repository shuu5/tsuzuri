//! `folio check` の設計ノートの正本（design-note/*.yaml）の検査の歯（便 23・docs/design/delivery-23.md §1 (d)）。
//! 歯の中で design-intent の写し全部を一時 dir の `design-intent/` に、器（scribe2）の導出 file を写しの根の
//! `contracts/` に作り、git init と 1 commit を行ってから（版管理の無い写しは別の理由で「まだ分からない」になる）、
//! 設計ノートの file を置く・変異を 1 つ当てて `folio check --dir` を回す。
//! 違反の歯は、変異が 1 つなら違反の件数が 1 であること（出力の件数の表示）も確かめる。
//! 凍結 fixture は要件書 AC7 / AC8 / AC13 の red_test が名指す tests/fixtures/design-note/ の 3 組だけ。
//! 便 120（docs/design/delivery-120.md §1 (d)）の歯 f120_ は、同じ dir の手書きの凍結の対 need-conditional.yaml と
//! need-conditional-schema.toml（器の導出 file の verify と done が要否 conditional）を使う。
//! 便 161（docs/design/delivery-161.md §1 (c)）の歯 f161_ は fixture を足さず、見本の承認欄を口 approve と row_with で置く。

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

    /// 便 120 の凍結の対を置く（契約表を持つ設計ノートを対の 1 本にするため実の見本 example.yaml を外す）。
    fn put_need_conditional(&self) {
        let from = repo_root().join("tests/fixtures/design-note");
        fs::remove_file(self.note("example.yaml")).unwrap();
        fs::copy(
            from.join("need-conditional.yaml"),
            self.note("need-conditional.yaml"),
        )
        .unwrap();
        fs::copy(from.join("need-conditional-schema.toml"), self.external()).unwrap();
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

// ── 便 120: 器の導出 file の要否 conditional（docs/design/delivery-120.md §1 (d)） ──

/// 合格・違反 0・まだ分からない 0。
fn assert_clean_pass(out: &Output) {
    assert_passes(out);
    assert!(
        stdout(out).contains("folio check: 合格（違反 0・まだ分からない 0）"),
        "{}",
        stdout(out)
    );
}

#[test]
fn f120_conditional_fields_pass_with_and_without_values() {
    let w = Work::new("f120-pair");
    w.put_need_conditional();
    let schema = fs::read_to_string(w.external()).unwrap();
    assert!(
        schema.contains("need = \"conditional\""),
        "対の導出 file が conditional を持たない"
    );
    let note = fs::read_to_string(w.note("need-conditional.yaml")).unwrap();
    let row_b = note
        .lines()
        .find(|l| l.trim_start().starts_with("- {id: b,"))
        .expect("行 b が無い");
    assert!(
        !row_b.contains("verify:") && !row_b.contains("done:"),
        "行 b が verify か done を持つ: {row_b}"
    );
    assert_clean_pass(&w.check());
}

#[test]
fn f120_needs_outside_the_domain_stay_unknown() {
    for (i, value) in ["sometimes", "Conditional", "required?"].iter().enumerate() {
        let w = Work::new(&format!("f120-need-{i}"));
        w.put_need_conditional();
        w.mutate_file(
            &w.external(),
            "name = \"verify\"\nneed = \"conditional\"\n",
            &format!("name = \"verify\"\nneed = \"{value}\"\n"),
        );
        assert_unknown(&w.check(), &format!("need「{value}」を知らない"));
    }
}

#[test]
fn f120_the_real_copy_has_the_current_shape_and_passes() {
    let text = fs::read_to_string(repo_root().join("contracts/schema.toml")).unwrap();
    assert!(
        text.contains("need = \"conditional\""),
        "repo の写しが conditional を持たない"
    );
    assert!(
        text.lines().any(|l| l == "[[promise-field]]"),
        "repo の写しが約束の行の欄の表を持たない"
    );
    let w = Work::new("f120-real");
    assert_clean_pass(&w.check());
}

// ── 形の違反（見本の写しに変異 1 つずつ） ──

#[test]
fn note_status_not_in_enum_fails() {
    let w = Work::new("status");
    w.mutate("\n  status: example\n", "\n  status: nope\n");
    assert_single_violation(&w.check(), "note", &["meta.status「nope」が一覧に無い"]);
}

/// 便 68（delivery-68.md §1 (d) 3）: 密度 profile の一覧を部品目録へ移しても、一覧に無い値で床が同じに落ちる。
#[test]
fn profiles_gate_the_note_meta() {
    let w = Work::new("profile");
    w.mutate("\n  profile: design-note\n", "\n  profile: design-notex\n");
    let out = w.check();
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}{}",
        stdout(&out),
        stderr(&out)
    );
    let text = stdout(&out);
    assert!(text.contains("meta.profile"), "{text}");
    assert!(text.contains("一覧に無い"), "{text}");
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

// ── 未知の欄（meta・節・図の行・便 61） ──
//
// 欄の決まりは 3 階層とも閉じた欄の集合（required と optional）なので、和集合に無い鍵は違反 1 件（種別「未知の欄」・
// 要件書の図の行の検査と同じ字面）。道（{at}）は同じ節の他の違反と同じ字面（meta / §N / figures の <id>）を使う。

#[test]
fn note_unknown_field_in_meta_fails() {
    let w = Work::new("unknown-meta");
    w.mutate(
        "\n  profile: design-note\n",
        "\n  profile: design-note\n  mystery: x\n",
    );
    assert_single_violation(
        &w.check(),
        "未知の欄",
        &["design-note/example.yaml", "meta の未知の欄「mystery」"],
    );
}

#[test]
fn note_unknown_field_in_section_fails() {
    let w = Work::new("unknown-section");
    w.mutate("  - n: 2\n", "  - n: 2\n    mystery: x\n");
    assert_single_violation(
        &w.check(),
        "未知の欄",
        &["design-note/example.yaml", "§2 の未知の欄「mystery」"],
    );
}

#[test]
fn note_unknown_field_in_figure_fails() {
    let w = Work::new("unknown-figure");
    w.mutate_file(
        &w.note("figures.yaml"),
        "  - id: fig-plain\n",
        "  - id: fig-plain\n    mystery: x\n",
    );
    assert_single_violation(
        &w.check(),
        "未知の欄",
        &[
            "design-note/figures.yaml",
            "figures の fig-plain の未知の欄「mystery」",
        ],
    );
}

/// 変えない写し＝実の設計ノート 3 本に未知の欄は 1 つも無い（式を足した後の床で数え直す）。
#[test]
fn note_unknown_field_none_on_the_canonical_copy() {
    let w = Work::new("unknown-none");
    assert_passes(&w.check());
}

// ── 便 103: 索引の節は索引の欄の決まりを指す・事後の検査に prose-mentions（docs/design/delivery-103.md §1 (g)） ──

const INDEX_SCHEMA: &str = "design-intent/graph.yaml";

/// 実の file の生成区間（begin の次の行から end の行の前まで）。
fn schema_region(rel: &str) -> String {
    let text = fs::read_to_string(repo_root().join(rel)).unwrap();
    let b = text.find("# folio:schema:begin").expect("begin が無い");
    let b = b + text[b..].find('\n').unwrap() + 1;
    let e = text.find("\n# folio:schema:end\n").expect("end が無い") + 1;
    text[b..e].to_string()
}

/// 設計ノートの欄の決まりの生成区間から、欄 key の 1 行を字下げを落として引く。
fn note_schema_line(key: &str) -> String {
    let region = schema_region("design-intent/design-note/schema.yaml");
    region
        .lines()
        .map(str::trim_start)
        .find(|l| l.starts_with(&format!("{key}: ")))
        .unwrap_or_else(|| panic!("欄 {key} の行が無い"))
        .to_string()
}

#[test]
fn f103_the_index_section_points_at_the_index_schema() {
    for line in [
        "node_fields_ref: design-intent/graph.yaml node",
        "node_kinds_ref: design-intent/graph.yaml node_kinds",
        "edge_fields_ref: design-intent/graph.yaml edge",
        "edge_types_ref: design-intent/graph.yaml edge_types",
    ] {
        let key = line.split(": ").next().unwrap();
        assert_eq!(note_schema_line(key), line, "指す欄の逐語");
    }
    let region = schema_region("design-intent/design-note/schema.yaml");
    for key in ["entries", "entry_fields"] {
        assert!(
            !region
                .lines()
                .any(|l| l.trim_start().starts_with(&format!("{key}:"))),
            "写しの一覧の行 {key} が残っている"
        );
    }
}

#[test]
fn f103_the_index_refs_resolve_in_the_index_schema() {
    let region = schema_region("design-intent/design-note/schema.yaml");
    let target = schema_region(INDEX_SCHEMA);
    let mut found = 0;
    for line in region.lines().map(str::trim_start) {
        let Some((name, value)) = line.split_once(": ") else {
            continue;
        };
        let Some(field) = value.strip_prefix(&format!("{INDEX_SCHEMA} ")) else {
            continue;
        };
        assert!(
            name.ends_with("_ref"),
            "指す欄の名が _ref で終わらない: {line}"
        );
        assert!(
            target.lines().any(|l| l
                .strip_prefix("  ")
                .is_some_and(|l| !l.starts_with(' ') && l.starts_with(&format!("{field}:")))),
            "指す先 {field} が {INDEX_SCHEMA} の生成区間に無い"
        );
        found += 1;
    }
    assert_eq!(found, 4, "索引の欄の決まりを指す欄の数");
}

#[test]
fn f103_the_index_note_names_the_landed_port() {
    let line = note_schema_line("index_note");
    for word in ["folio graph --print", "ADR-14", INDEX_SCHEMA, "CON9"] {
        assert!(
            line.contains(word),
            "index_note が {word} を名指さない: {line}"
        );
    }
    assert!(
        !line.contains("未実装"),
        "index_note に 未実装 の字が残っている: {line}"
    );
}

#[test]
fn f103_the_post_guards_carry_the_mentions_tooth() {
    assert_eq!(
        note_schema_line("post"),
        "post: [yaml-form, derived-diff-zero, own-id-space, prose-gate, prose-mentions]",
        "事後の検査の一覧"
    );
    let line = note_schema_line("guards_note");
    for word in [
        "prose-mentions",
        "R-17",
        "crates/folio/src/mentions.rs",
        "design-note/",
    ] {
        assert!(
            line.contains(word),
            "guards_note が {word} を名指さない: {line}"
        );
    }
}

// ── 便 161: 承認欄の形は判断の記録の承認欄と同じ（docs/design/delivery-161.md §1 (c)） ──

/// 良い 1 項（違反 0）の 5 欄。
const GOOD_ROW: [(&str, &str); 5] = [
    ("who", "持ち主"),
    ("date", "2026-09-18"),
    ("ruling", "f2-648.37 notes"),
    ("verbatim", "承認する"),
    ("surface", "R-8"),
];

/// 良い 1 項から `changes` の欄だけを替えた承認欄の 1 項（値は全部引用符で囲む）。
fn row_with(changes: &[(&str, &str)]) -> String {
    let fields: Vec<String> = GOOD_ROW
        .iter()
        .map(|(key, good)| {
            let value = changes
                .iter()
                .find(|(k, _)| k == key)
                .map_or(*good, |(_, v)| v);
            format!("{key}: \"{value}\"")
        })
        .collect();
    format!("{{{}}}", fields.join(", "))
}

/// 5 欄とも印の 1 項。
fn unfilled_row() -> String {
    row_with(&GOOD_ROW.map(|(key, _)| (key, "未記入")))
}

/// 見本 example.yaml の status を替え、承認欄に `rows` を置く。
fn approve(w: &Work, status: &str, rows: &[String]) {
    let items: String = rows.iter().map(|r| format!("    - {r}\n")).collect();
    w.mutate(
        "\n  status: example\n",
        &format!("\n  status: {status}\n  approval:\n{items}"),
    );
}

const VERBATIM_MARK: &str =
    "meta の approval[0] の verbatim が 未記入（init の雛形の印・空と同じ）";

/// 不合格 1・違反はちょうど 5 行で、5 欄とも印の項 n の行が承認者・日付・裁定 id・逐語・対話面の順に並ぶ。
fn assert_five_lines(out: &Output, n: usize) {
    assert_eq!(out.status.code(), Some(1), "{}{}", stdout(out), stderr(out));
    let at = format!("meta の approval[{n}]");
    let want = [
        format!("{at}: who「未記入」が一覧に無い（判断の記録の承認者の値域）"),
        format!("{at}: date「未記入」が年-月-日でない"),
        format!("{at}: ruling「未記入」に台帳 id（"),
        format!("{at} の verbatim が 未記入（init の雛形の印・空と同じ）"),
        format!("{at}: surface「未記入」が一覧に無い"),
    ];
    let v = violations(out);
    assert_eq!(v.len(), want.len(), "違反の行数: {v:?}");
    for (line, w) in v.iter().zip(&want) {
        assert!(
            line.starts_with("[note] design-note/example.yaml: "),
            "{v:?}"
        );
        assert!(line.contains(w.as_str()), "「{w}」が無い: {v:?}");
    }
    assert!(
        stdout(out).contains("folio check: 不合格（違反 5・"),
        "{}",
        stdout(out)
    );
}

#[test]
fn f161_the_verbatim_mark_is_empty() {
    for (i, (status, verbatim)) in [
        ("effective", "未記入"),
        ("effective", "　未記入 "),
        ("draft", "未記入"),
    ]
    .iter()
    .enumerate()
    {
        let w = Work::new(&format!("f161-verbatim-{i}"));
        approve(&w, status, &[row_with(&[("verbatim", verbatim)])]);
        assert_single_violation(&w.check(), "note", &[VERBATIM_MARK]);
    }
    let w = Work::new("f161-verbatim-contains");
    approve(
        &w,
        "effective",
        &[row_with(&[("verbatim", "未記入の欄は無いので承認する")])],
    );
    assert_passes(&w.check());
}

#[test]
fn f161_who_and_ruling_take_the_adr_shape() {
    let w = Work::new("f161-who-mark");
    approve(&w, "effective", &[row_with(&[("who", "未記入")])]);
    assert_single_violation(
        &w.check(),
        "note",
        &["meta の approval[0]: who「未記入」が一覧に無い（判断の記録の承認者の値域）"],
    );

    let w = Work::new("f161-ruling-mark");
    approve(&w, "effective", &[row_with(&[("ruling", "未記入")])]);
    assert_single_violation(
        &w.check(),
        "note",
        &[
            "meta の approval[0]: ruling「未記入」に台帳 id（",
            "）が無い",
        ],
    );

    let w = Work::new("f161-who-blank");
    approve(&w, "effective", &[row_with(&[("who", "")])]);
    assert_single_violation(&w.check(), "欄の非空", &["meta の approval[0] の who が空"]);

    let w = Work::new("f161-five-effective");
    approve(&w, "effective", &[unfilled_row()]);
    assert_five_lines(&w.check(), 0);

    let w = Work::new("f161-five-draft");
    approve(&w, "draft", &[unfilled_row()]);
    assert_five_lines(&w.check(), 0);

    let w = Work::new("f161-five-second");
    approve(&w, "effective", &[row_with(&[]), unfilled_row()]);
    assert_five_lines(&w.check(), 1);
}

/// 判断の記録の欄の決まりの生成区間の approver の値。
fn adr_approvers() -> Vec<String> {
    let region = schema_region("design-intent/adr/schema.yaml");
    let line = region
        .lines()
        .map(str::trim_start)
        .find_map(|l| l.strip_prefix("approver: ["))
        .expect("approver の行が無い");
    line.strip_suffix(']')
        .expect("approver の行が ] で終わらない")
        .split(", ")
        .map(str::to_string)
        .collect()
}

#[test]
fn f161_who_and_ruling_agree_with_the_adr_approval() {
    let approvers = adr_approvers();
    assert_eq!(approvers.len(), 3, "approver の値: {approvers:?}");
    let src = fs::read_to_string(repo_root().join("crates/folio/src/note.rs")).unwrap();
    for a in &approvers {
        assert!(
            !src.contains(&format!("\"{a}\"")),
            "note.rs が承認者の値「{a}」を自分で持つ"
        );
    }
    let rulings_ok = ["f2-648.2 notes", "s2-07l.149", "f2-648 notes 2026-09-27"];
    let whos_bad = ["持ち主（shuu5）", "未記入", "席", " 持ち主 "];
    let rulings_bad = ["口頭", "未記入", "F2-648", "f2-"];
    let mut cases: Vec<(String, &str, bool)> = approvers
        .iter()
        .zip(rulings_ok)
        .map(|(a, r)| (a.clone(), r, true))
        .collect();
    cases.extend(
        whos_bad
            .iter()
            .zip(rulings_bad)
            .map(|(a, r)| ((*a).to_string(), r, false)),
    );
    for (i, (who, ruling, good)) in cases.iter().enumerate() {
        let w = Work::new(&format!("f161-agree-{i}"));
        w.mutate_file(
            &w.dir().join("adr/ADR-2.yaml"),
            "approval: {who: 持ち主, date: 2026-09-13, ruling: f2-648.2 notes 2026-09-13 09:35,",
            &format!("approval: {{who: \"{who}\", date: 2026-09-13, ruling: \"{ruling}\","),
        );
        approve(
            &w,
            "effective",
            &[row_with(&[("who", who), ("ruling", ruling)])],
        );
        let out = w.check();
        let v = violations(&out);
        let has =
            |prefix: &str, word: &str| v.iter().any(|l| l.starts_with(prefix) && l.contains(word));
        let adr_who = has("[N-4]", "ADR-2.approval.who");
        let adr_ruling = has("[N-4]", "ADR-2.approval.ruling");
        let note_who = has("[note] design-note/example.yaml", "who「");
        let note_ruling = has("[note] design-note/example.yaml", "ruling「");
        assert_eq!(adr_who, note_who, "who「{who}」: {v:?}");
        assert_eq!(adr_ruling, note_ruling, "ruling「{ruling}」: {v:?}");
        assert_eq!(note_who, !good, "who「{who}」: {v:?}");
        assert_eq!(note_ruling, !good, "ruling「{ruling}」: {v:?}");
        if *good {
            // 発効した ADR-2 の承認欄を写しの上で変えた＝違反は封の 1 行だけ（便 170・ADR-30 決定 (3)）
            assert_eq!(v.len(), 1, "who「{who}」: {v:?}");
            assert!(
                v[0].starts_with("[adr] ADR-2: 発効した判断の記録の本文が封（anchors/adr-seals.yaml）の行と違う"),
                "who「{who}」: {v:?}"
            );
        }
    }
}

// ── 章の上限（便 179・docs/design/delivery-179.md §1 (c)）──

/// 見本 example.yaml（節 6・図 1）に節 7〜12 と 2 枚目の図を足す（章 13）。
fn thirteen_chapters(w: &Work) {
    let extra: String = (7..=12)
        .map(|n| format!("  - {{n: {n}, type: prose, title: 追加 {n}, body: 追加の節。}}\n"))
        .collect();
    w.mutate("\nfigures:\n", &format!("\n{extra}figures:\n"));
    // 2 枚目の図（図の章は枚数に依らず 1 章＝面と床が同じ数え方でなければ字が割れる）
    let path = w.dir().join("design-note/example.yaml");
    let text = std::fs::read_to_string(&path).unwrap();
    let at = text.find("  - id: fig-1\n").unwrap();
    let end = text[at..].find("\nsources:").map(|e| at + e + 1).unwrap();
    let second = text[at..end].replacen("id: fig-1", "id: fig-2", 1);
    std::fs::write(&path, format!("{}{}{}", &text[..end], second, &text[end..])).unwrap();
}

/// 写しの規則の表の欄 key が note-chapters の行（本流の行 R-19）の値を置き換える（字の当て先は 1 か所）。
fn cap_value(w: &Work, value: &str) {
    w.mutate_file(
        &w.dir().join("rules.yaml"),
        "value: \"12 章 以下\"",
        &format!("value: \"{value}\""),
    );
}

#[test]
fn f179_floor_counts_chapters_with_the_same_cap_and_words_as_the_face() {
    let w = Work::new("f179-floor");
    thirteen_chapters(&w);
    let words = "design-note/example.yaml: 章が 13 本ある＝章が多すぎる（上限 12）";
    assert_single_violation(&w.check(), "note", &[words]);
    // 面の生成器も同じ写しで同じ字を出して止まる（同じ関数・書かない）
    let out = w.root.join("note-example.html");
    let face = Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(["face", "--face", "note", "--id", "example", "--dir"])
        .arg(w.dir())
        .arg("--out")
        .arg(&out)
        .arg("--write")
        .output()
        .expect("folio を起動できない");
    assert_eq!(face.status.code(), Some(2), "{}", stderr(&face));
    assert!(stderr(&face).contains(words), "{}", stderr(&face));
    assert!(!out.exists(), "上限を超えたのに面を書いた");
    // 値を 13 に上げると床は合格（数えるのは行の値）
    cap_value(&w, "13 章 以下");
    assert_passes(&w.check());
}

#[test]
fn f179_floor_is_unknown_without_the_keyed_row() {
    let w = Work::new("f179-no-row");
    let rules = w.dir().join("rules.yaml");
    let text = fs::read_to_string(&rules).unwrap().replace("key: note-chapters, ", "");
    assert!(!text.contains("key: note-chapters"), "欄 key の行が残った");
    fs::write(&rules, text).unwrap();
    assert_unknown(&w.check(), "設計ノートの章の上限が読めない");
}
