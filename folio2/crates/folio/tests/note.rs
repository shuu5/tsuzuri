//! `folio check` の設計ノートの正本（design-note/*.yaml）の検査の歯（便 23・docs/design/delivery-23.md §1 (d)）。
//! 歯の中で design-intent の写し全部を一時 dir の `design-intent/` に、器（scribe2）の導出 file を写しの根の
//! `contracts/` に作り、git init と 1 commit を行ってから（版管理の無い写しは別の理由で「まだ分からない」になる）、
//! 設計ノートの file を置く・変異を 1 つ当てて `folio check --dir` を回す。
//! 違反の歯は、変異が 1 つなら違反の件数が 1 であること（出力の件数の表示）も確かめる。
//! 凍結 fixture は要件書 AC7 / AC8 / AC13 の red_test が名指す tests/fixtures/design-note/ の 3 組だけ。
//! 便 120（docs/design/delivery-120.md §1 (d)）の歯 f120_ は、同じ dir の手書きの凍結の対 need-conditional.yaml と
//! need-conditional-schema.toml（器の導出 file の verify と done が要否 conditional）を使う。
//! 便 161（docs/design/delivery-161.md §1 (c)）の歯 f161_ は fixture を足さず、見本の承認欄を口 approve と row_with で置く。
//! 便 181（docs/design/delivery-181.md §1 (c)）から承認欄の裁定の欄は決定の欄の床（種別 裁定 id）が数える＝f161_ の 2 本の字を合わせた。
//! 便 207（docs/design/delivery-207.md §1 (c)）の歯 f207_ は、写しの土台を床の土台（tests/fixtures/floor_base/design-intent/・数の上限の
//! 値の行 3 本は 99）にして数の上限の値を 1 に下げ、生きたノートを歯の中の最小の手書きで足す（folio2 の本流の値に依らない）。
//! 便 209（docs/design/delivery-209.md §1 (c)）の歯 f209_ は、同じ床の土台の写し（要件 FR1〜FR19・非機能要件 NFR1〜NFR3・
//! 判断の記録 ADR-1〜ADR-10）に廃止のノートを手書きで足し、後継の先を要件・非機能要件・判断の記録の id にする。無い先には folio2 の本流に
//! 在って土台に無い id（FR32・ADR-35）を使い、土台の置き場を読むことを確かめる。

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

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
        "      - decision-table\n",
        "      - decision-table\n      - extra-type\n",
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

/// 不合格 1・違反はちょうど 5 行で、5 欄とも印の項 n の行が承認者・日付・逐語・対話面の順に並び、裁定 id の行
/// （便 181 から決定の欄の床・種別 裁定 id）が後に続く。
fn assert_five_lines(out: &Output, n: usize) {
    assert_eq!(out.status.code(), Some(1), "{}{}", stdout(out), stderr(out));
    let at = format!("meta の approval[{n}]");
    let want = [
        ("note", format!("{at}: who「未記入」が一覧に無い（判断の記録の承認者の値域）")),
        ("note", format!("{at}: date「未記入」が年-月-日でない")),
        ("note", format!("{at} の verbatim が 未記入（init の雛形の印・空と同じ）")),
        ("note", format!("{at}: surface「未記入」が一覧に無い")),
        ("裁定 id", format!("meta.approval[{n}].ruling「未記入」に台帳 id が無い")),
    ];
    let v = violations(out);
    assert_eq!(v.len(), want.len(), "違反の行数: {v:?}");
    for (line, (kind, w)) in v.iter().zip(&want) {
        assert!(
            line.starts_with(&format!("[{kind}] design-note/example.yaml: ")),
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

    // 裁定の欄は便 181 から決定の欄の床（種別 裁定 id・判断の記録の承認欄と同じ関数と同じ字）が数える
    let w = Work::new("f161-ruling-mark");
    approve(&w, "effective", &[row_with(&[("ruling", "未記入")])]);
    assert_single_violation(
        &w.check(),
        "裁定 id",
        &["meta.approval[0].ruling「未記入」に台帳 id が無い"],
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
        let adr_ruling = has("[裁定 id] adr/ADR-2.yaml", "approval.ruling「");
        let note_who = has("[note] design-note/example.yaml", "who「");
        let note_ruling = has("[裁定 id] design-note/example.yaml", "ruling「");
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
    let words = "design-note/example.yaml: 章が多すぎる（note-chapters の上限 12 章 以下）";
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

// ── 便 207: 数の上限（生きたノートの本数・1 本の契約表の行の数・計画だけの行の合計・ADR-35 決定 (1)・FR30・AC33） ──

/// 床の土台の写し（値の行 3 本は 99）を作り、数の上限の 3 行の値を 1 に下げて git の 1 commit にする。
fn growth_base(case: &str) -> Work {
    let root = std::env::temp_dir().join(format!("folio-note-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    copy_tree(
        &repo_root().join("tests/fixtures/floor_base/design-intent"),
        &root.join("design-intent"),
    );
    fs::create_dir_all(root.join("contracts")).unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        root.join("contracts/schema.toml"),
    )
    .unwrap();
    let w = Work { root };
    let rules = w.dir().join("rules.yaml");
    w.mutate_file(&rules, "value: \"99 本 以下\"", "value: \"1 本 以下\"");
    let text = fs::read_to_string(&rules).unwrap();
    assert_eq!(text.matches("value: \"99 行 以下\"").count(), 2, "土台の行の値が 99 でない");
    fs::write(&rules, text.replace("value: \"99 行 以下\"", "value: \"1 行 以下\"")).unwrap();
    w.commit();
    w
}

/// 生きたノートの手書き（節 = 目的の散文・契約表〔行 `rows` 本・行 id は id の後ろに番号〕・散文の節 `extra` 個）。
/// 発効と廃止は承認欄を、廃止は後継（土台の見本 example）を持つ。
fn growth_note(id: &str, status: &str, rows: usize, extra: usize) -> String {
    let mut meta = format!(
        "meta:\n  id: {id}\n  title: 数えの見本 {id}\n  version: v0.1\n  status: {status}\n  generated: 2026-09-29\n  profile: design-note\n"
    );
    if status == "retired" {
        meta.push_str("  superseded_by: example\n");
    }
    if status == "effective" || status == "retired" {
        meta.push_str("  approval:\n    - {who: 持ち主, date: 2026-09-29, ruling: f2-648 notes 2026-09-29 13:19 JST, verbatim: 承認する, surface: R-8}\n");
    }
    let rows: String = (1..=rows)
        .map(|i| format!("      - {{id: {id}{i}, title: 行 {i}, req: [FR15], section: \"1\", verify: [cargo nextest run -p folio x], size: S, done: 緑, depends: []}}\n"))
        .collect();
    let extra: String = (3..3 + extra)
        .map(|n| format!("  - {{n: {n}, type: prose, title: 追加 {n}, body: 追加の節。}}\n"))
        .collect();
    format!(
        "{meta}sections:\n  - n: 1\n    type: prose\n    title: 目的\n    body: 数えの見本。\n  - n: 2\n    type: contract-table\n    title: 契約表\n    rows:\n{rows}{extra}"
    )
}

impl Work {
    fn put_note(&self, id: &str, text: &str) {
        fs::write(self.note(&format!("{id}.yaml")), text).unwrap();
    }

    fn commit(&self) {
        git(&self.root, &["init", "-q"]);
        git(&self.root, &["add", "-A"]);
        git(&self.root, &["commit", "-q", "-m", "fixture"]);
    }

    /// 編集時の口（`rel` に `text` を書こうとしている・置き場は書かない）。
    fn propose(&self, rel: &str, text: &str) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_folio"))
            .args(["check", "--dir"])
            .arg(self.dir())
            .args(["--proposed", rel])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("folio を起動できない");
        child.stdin.take().unwrap().write_all(text.as_bytes()).unwrap();
        child.wait_with_output().unwrap()
    }
}

/// 標準エラーの今の数の行（`# 今の数: ` の後ろ）。
fn counts(out: &Output) -> Vec<String> {
    stderr(out).lines().filter_map(|l| l.strip_prefix("# 今の数: ")).map(str::to_string).collect()
}

const LIVE_OVER: &str = "[note] design-note/: 生きたノートが多すぎる（live-notes の上限 1 本 以下）";

/// 便 207 (c)4（AC33）: 値 1 の写しで、見本 2 本だけなら合格（見本は数えない）、生きたノート 1 本（ちょうど上限）も合格、下書きと発効の
/// 2 本で違反ちょうど 1。字は置き場と欄 key と上限の値だけで今の数を持たず、今の数は標準エラーの # の行。
#[test]
fn f207_live_notes_over_the_cap_is_one_violation_without_the_count() {
    let w = growth_base("f207-live");
    assert_passes(&w.check());
    w.put_note("ga", &growth_note("ga", "draft", 1, 0));
    assert_passes(&w.check());
    w.put_note("gb", &growth_note("gb", "effective", 1, 0));
    let out = w.check();
    assert_single_violation(&out, "note", &[]);
    assert_eq!(violations(&out), [LIVE_OVER]);
    assert_eq!(counts(&out), ["design-note/: 生きたノートの今の数 2（live-notes の上限 1 本 以下）"]);
}

/// 便 207 (c)5（AC33）: 足したノートを廃止（承認欄と後継つき）か見本にすると数えず、違反 0 に戻る。
#[test]
fn f207_retired_and_example_notes_are_not_counted() {
    let w = growth_base("f207-retire");
    w.put_note("ga", &growth_note("ga", "draft", 1, 0));
    w.put_note("gb", &growth_note("gb", "draft", 1, 0));
    assert_eq!(violations(&w.check()), [LIVE_OVER]);
    w.put_note("gb", &growth_note("gb", "retired", 1, 0));
    assert_passes(&w.check());
    w.put_note("gb", &growth_note("gb", "example", 1, 0));
    assert_passes(&w.check());
}

/// 便 207 (c)6（AC33）: 契約表の行は生きたノート 1 本ごとに値と比べ（1 行ずつの 2 本は合格）、1 本が 2 行なら違反 1 でそのノートを名指す。
/// 廃止にしたノートの行は数えない。
#[test]
fn f207_note_rows_are_counted_per_live_note() {
    let w = growth_base("f207-rows");
    w.mutate_file(&w.dir().join("rules.yaml"), "value: \"1 本 以下\"", "value: \"9 本 以下\"");
    w.put_note("ga", &growth_note("ga", "draft", 1, 0));
    w.put_note("gb", &growth_note("gb", "draft", 1, 0));
    assert_passes(&w.check());
    w.put_note("ga", &growth_note("ga", "draft", 2, 0));
    let out = w.check();
    assert_single_violation(&out, "note", &[]);
    assert_eq!(violations(&out), ["[note] design-note/ga.yaml: 契約表の行が多すぎる（note-rows の上限 1 行 以下）"]);
    assert_eq!(counts(&out), ["design-note/ga.yaml: 契約表の行の今の数 2（note-rows の上限 1 行 以下）"]);
    w.put_note("ga", &growth_note("ga", "retired", 2, 0));
    assert_passes(&w.check());
}

/// 便 207 (c)7（AC33・ADR-35 決定 (1)(エ)）: 設計ノートが見本だけでも 3 つの行を読み、欄 key の行が無い・2 本・値の形が違う（「1 行以上」・
/// 単位の取り違え）は、その数えが まだ分からない（合格にしない）。読めた設計ノートが 0 本なら読まない。
#[test]
fn f207_missing_twice_or_misshaped_cap_rows_are_unknown() {
    for (key, bad) in [("live-notes", "1 行 以下"), ("note-rows", "1 行以上"), ("plan-rows", "1 本 以下")] {
        let unreadable = format!("設計ノートの数の上限 {key} が読めない");
        let rules = |w: &Work| w.dir().join("rules.yaml");
        let w = growth_base(&format!("f207-none-{key}"));
        w.mutate_file(&rules(&w), &format!(", key: {key}}}"), "}");
        assert_unknown(&w.check(), &unreadable);
        fs::remove_file(w.note("example.yaml")).unwrap();
        fs::remove_file(w.note("figures.yaml")).unwrap();
        let none = w.check();
        assert!(!stderr(&none).contains(&unreadable), "0 本なのに行を読んだ: {}", stderr(&none));
        let w = growth_base(&format!("f207-two-{key}"));
        w.mutate_file(&rules(&w), "{id: R-2, article: P-14, ", &format!("{{id: R-2, article: P-14, key: {key}, "));
        let two = w.check();
        assert_eq!(two.status.code(), Some(2), "{}", stdout(&two));
        assert!(stderr(&two).lines().any(|l| l.starts_with("# まだ分からない: ") && l.contains(&unreadable) && l.contains("2 本ある")), "{}", stderr(&two));
        let w = growth_base(&format!("f207-shape-{key}"));
        let text = fs::read_to_string(rules(&w)).unwrap();
        let line = text.lines().find(|l| l.ends_with(&format!("key: {key}}}"))).unwrap();
        let from = line.split("value: ").nth(1).unwrap().split(',').next().unwrap();
        w.mutate_file(&rules(&w), line, &line.replacen(from, &format!("\"{bad}\""), 1));
        assert_unknown(&w.check(), &unreadable);
    }
}

/// 便 207 (c)8（AC33・ADR-35 決定 (1)(オ)・ADR-33 決定 (2)）: 上限を 2 超えた写し（生きたノート 3 本・上限 1）で、1 本を廃止にする中身も、
/// 超えたまま 1 本足す中身も、編集時の口は止めない（違反の字が書く前と同じ）。超えていない別のノートが契約表の行の上限を跨ぐ中身は、
/// そのノートを名指す新しい違反で止め（1・口の標準出力に今の数の行は出さない）、同じ中身を書いた置き場の素の床にも同じ行が在る（P-15.2）。
#[test]
fn f207_proposed_does_not_stop_retiring_or_adding_over_the_cap_but_stops_a_note_crossing_it() {
    let w = growth_base("f207-proposed");
    for id in ["ga", "gb", "gc"] {
        w.put_note(id, &growth_note(id, "draft", 1, 0));
    }
    git(&w.root, &["add", "-A"]);
    git(&w.root, &["commit", "-q", "-m", "three"]);
    assert_eq!(violations(&w.check()), [LIVE_OVER]);
    for (rel, text) in [
        ("design-note/gc.yaml", growth_note("gc", "retired", 1, 0)),
        ("design-note/gd.yaml", growth_note("gd", "draft", 1, 0)),
    ] {
        let r = w.propose(rel, &text);
        assert_eq!(r.status.code(), Some(0), "{rel}: {}", stdout(&r));
        assert!(stdout(&r).contains("folio check --proposed: 通す（新しい違反 0・"), "{}", stdout(&r));
    }
    let cross = growth_note("gb", "draft", 2, 0);
    let r = w.propose("design-note/gb.yaml", &cross);
    let stop = "[note] design-note/gb.yaml: 契約表の行が多すぎる（note-rows の上限 1 行 以下）";
    assert_eq!(r.status.code(), Some(1), "{}", stdout(&r));
    assert_eq!(stdout(&r).lines().next(), Some(stop), "{}", stdout(&r));
    // 口の標準出力は要件 FR28 の 4 種の行だけ（今の数の行は素の床の標準エラーに出す・席の裁定 2026-09-29）
    assert!(!stdout(&r).contains("今の数"), "{}", stdout(&r));
    w.put_note("gb", &cross);
    assert!(violations(&w.check()).iter().any(|l| l == stop));
}

/// 便 207 (c)9（ADR-35 決定 (1)(ク)）: 章の上限も同じ形。章が上限を 2 超えたノートの章を 1 つ減らす中身は止めず（字に章の今の数が無い）、
/// 上限の内のノートが上限を跨ぐ中身は、そのノートを名指す新しい違反で止める。
#[test]
fn f207_chapter_cap_does_not_stop_reducing_over_the_cap_but_stops_crossing() {
    let w = growth_base("f207-chapters");
    w.mutate_file(&w.dir().join("rules.yaml"), "value: \"12 章 以下\"", "value: \"2 章 以下\"");
    w.put_note("ga", &growth_note("ga", "draft", 1, 2));
    w.put_note("gb", &growth_note("gb", "draft", 1, 0));
    w.mutate_file(&w.dir().join("rules.yaml"), "value: \"1 本 以下\"", "value: \"9 本 以下\"");
    git(&w.root, &["add", "-A"]);
    git(&w.root, &["commit", "-q", "-m", "chapters"]);
    let over = |id: &str| format!("[note] design-note/{id}.yaml: 章が多すぎる（note-chapters の上限 2 章 以下）");
    let out = w.check();
    assert!(violations(&out).contains(&over("ga")), "{:?}", violations(&out));
    assert!(counts(&out).contains(&"design-note/ga.yaml: 章の今の数 4（note-chapters の上限 2 章 以下）".to_string()), "{:?}", counts(&out));
    let r = w.propose("design-note/ga.yaml", &growth_note("ga", "draft", 1, 1));
    assert_eq!(r.status.code(), Some(0), "{}", stdout(&r));
    let r = w.propose("design-note/gb.yaml", &growth_note("gb", "draft", 1, 1));
    assert_eq!(r.status.code(), Some(1), "{}", stdout(&r));
    assert_eq!(stdout(&r).lines().next(), Some(over("gb").as_str()), "{}", stdout(&r));
}

// ── 便 209: 後継の欄の先（要件・非機能要件・判断の記録・ADR-35 決定 (3)・FR32・AC35） ──

/// 廃止のノート `id`（承認欄つき・契約表 1 行）の後継を `to` にした手書き。
fn retired_to(id: &str, to: &str) -> String {
    growth_note(id, "retired", 1, 0).replacen("  superseded_by: example\n", &format!("  superseded_by: {to}\n"), 1)
}

/// 便 209 (c)2（AC35）: 床の土台の写しで、廃止のノートの後継を実在する要件・非機能要件・判断の記録の id にすると違反 0（設計ノートの先も
/// 今のまま・先が廃止のノートでも先の状態は数えない）。
#[test]
fn f209_successor_naming_an_existing_requirement_nonfunctional_or_adr_passes() {
    let w = growth_base("f209-exists");
    for to in ["FR19", "NFR3", "ADR-10", "ADR-1", "example"] {
        w.put_note("gone", &retired_to("gone", to));
        assert_passes(&w.check());
    }
    w.put_note("gone", &retired_to("gone", "example"));
    w.put_note("older", &retired_to("older", "gone"));
    assert_passes(&w.check());
}

/// 便 209 (c)3（AC35・ADR-35 決定 (3)(ウ)）: 無い要件の id・無い非機能要件の id・無い判断の記録の id（本流に在って土台に無い）は、先の種類を
/// 名指すつながりの違反ちょうど 1。受入基準の id は後継の先の種類でない（設計ノートの id として読む）。編集時の口は止めない（ADR-33 決定 (2)）。
#[test]
fn f209_missing_requirement_or_adr_successor_is_one_link_violation() {
    for (to, what) in [("FR32", "要件"), ("NFR4", "要件"), ("ADR-35", "判断の記録"), ("AC1", "設計ノート")] {
        let w = growth_base(&format!("f209-missing-{to}"));
        let text = retired_to("gone", to);
        let want = format!("[note] design-note/gone.yaml: meta.superseded_by「{to}」の{what}が実在しない");
        let r = w.propose("design-note/gone.yaml", &text);
        assert_eq!(r.status.code(), Some(0), "{to}: {}", stdout(&r));
        assert!(stdout(&r).lines().all(|l| !l.starts_with('[')), "{to}: 口が止めた: {}", stdout(&r));
        let link = format!("# つながり（編集は止めない・事後の床が数える）: {want}");
        assert!(stdout(&r).lines().any(|l| l == link), "{to}: つながりの行が無い: {}", stdout(&r));
        w.put_note("gone", &text);
        let out = w.check();
        assert_single_violation(&out, "note", &[]);
        assert_eq!(violations(&out), [want], "{to}");
    }
}

/// 便 209 (c)3 の続き（ADR-35 決定 (3)(ウ)）: 要件の先は要件書の要件と非機能要件の節の id だけで、目的の節に要件の形の id（FR99）の行を
/// 足した写しでも、その id を後継にすると要件の不在のつながりの違反ちょうど 1。
#[test]
fn f209_requirement_successor_reads_only_the_requirement_sections() {
    let w = growth_base("f209-goal");
    let srs = w.dir().join("srs.yaml");
    let text = fs::read_to_string(&srs).unwrap();
    let at = text.find("  - {id: GOAL4,").unwrap();
    let end = at + text[at..].find('\n').unwrap() + 1;
    let row = "  - {id: FR99, title: 要件の形の目的, text: 要件の形の id を目的の節に置いた写し。}\n";
    fs::write(&srs, format!("{}{row}{}", &text[..end], &text[end..])).unwrap();
    w.put_note("gone", &retired_to("gone", "FR99"));
    let out = w.check();
    assert_single_violation(&out, "note", &[]);
    assert_eq!(violations(&out), ["[note] design-note/gone.yaml: meta.superseded_by「FR99」の要件が実在しない"]);
}

/// 便 209 (c)4（AC35）: 後継が自分自身の id は今と同じ 1 つの file の形の違反ちょうど 1 で、編集時の口も止める。
#[test]
fn f209_successor_pointing_at_itself_stays_a_shape_violation() {
    let w = growth_base("f209-self");
    let text = retired_to("gone", "gone");
    let want = "[note] design-note/gone.yaml: meta.superseded_by「gone」の設計ノートが実在しない";
    let r = w.propose("design-note/gone.yaml", &text);
    assert_eq!(r.status.code(), Some(1), "{}", stdout(&r));
    assert_eq!(stdout(&r).lines().next(), Some(want), "{}", stdout(&r));
    w.put_note("gone", &text);
    let out = w.check();
    assert_single_violation(&out, "note", &[]);
    assert_eq!(violations(&out), [want]);
}

/// 便 209 (c)5（AC35・ADR-35 決定 (3)(ア)）: 前（supersedes）は今のまま設計ノートの id だけで、在る要件と判断の記録の id を書いても
/// 設計ノートの不在のつながりの違反ちょうど 1（編集時の口は通し、つながりの行に同じ字）。
#[test]
fn f209_supersedes_still_names_design_notes_only() {
    for to in ["FR19", "ADR-10"] {
        let w = growth_base(&format!("f209-supersedes-{to}"));
        let text = growth_note("ga", "draft", 1, 0).replacen("  profile: design-note\n", &format!("  profile: design-note\n  supersedes: {to}\n"), 1);
        let r = w.propose("design-note/ga.yaml", &text);
        assert_eq!(r.status.code(), Some(0), "{to}: {}", stdout(&r));
        assert!(stdout(&r).lines().any(|l| l == format!("# つながり（編集は止めない・事後の床が数える）: [note] design-note/ga.yaml: meta.supersedes「{to}」の設計ノートが実在しない")), "{to}: {}", stdout(&r));
        w.put_note("ga", &text);
        let out = w.check();
        assert_single_violation(&out, "note", &[]);
        assert_eq!(violations(&out), [format!("[note] design-note/ga.yaml: meta.supersedes「{to}」の設計ノートが実在しない")], "{to}");
    }
}

/// 便 209 (c)6（ADR-35 決定 (3)(ウ)・P-4.2）: 判断の記録の置き場が読めない写しでは、判断の記録を指す後継の先を数えられない＝まだ分からない の
/// 行が在り、後継の違反は出さない（要件の先は数える）。
#[test]
fn f209_adr_successor_without_the_adr_place_is_unknown() {
    let w = growth_base("f209-noadr");
    fs::remove_dir_all(w.dir().join("adr")).unwrap();
    w.put_note("gone", &retired_to("gone", "ADR-10"));
    let out = w.check();
    assert_ne!(out.status.code(), Some(0), "{}", stdout(&out));
    assert!(
        stderr(&out).lines().any(|l| l == "# まだ分からない: design-note/gone.yaml: meta.superseded_by「ADR-10」の判断の記録を数えられない（adr/ が読めない）"),
        "{}",
        stderr(&out)
    );
    assert!(violations(&out).iter().all(|l| !l.contains("superseded_by")), "{:?}", violations(&out));
}
