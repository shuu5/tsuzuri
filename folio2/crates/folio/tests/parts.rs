//! `folio parts` の歯（便 13・docs/design/delivery-13.md §1 (c)）。
//! 生成した 3 面（folio build を一時 dir へ・手書きの見本は退役済み 2026-09-18）で合格・目録外の class / 部品 / 面に置けない部品 / 許されない行内の様式で不合格（変異は写しに 1 つずつ）・
//! 読めない入力 6 つで終了 2・独立した凍結 fixture（tests/fixtures/floor/）の合格と不合格と --print の byte 一致。
//! 便 52（delivery-52.md §1 (d) 3・4）: build.rs を path で取り込み、純粋な関数 `parts_catalog`（部品目録の文字列 →
//! 導出した Rust の source か理由の文）を直に呼ぶ = 実の部品目録で Ok・変異 3 つがそれぞれ Err。実行時の一致 = 部品目録の
//! 上限か図の型の名札を変えた写しで --check が「まだ分からない」。

#[allow(dead_code)]
#[path = "../build.rs"]
mod build;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn design_intent() -> PathBuf {
    repo_root().join("design-intent")
}

fn floor(name: &str) -> PathBuf {
    repo_root().join("tests/fixtures/floor").join(name)
}

/// `folio parts --check --dir <dir>` に引数を足して撃つ。
fn parts_check(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("parts")
        .arg("--check")
        .arg("--dir")
        .arg(dir)
        .args(args)
        .output()
        .expect("folio を起動できない")
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or_else(|| {
        panic!(
            "folio parts が signal で終わった: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-parts-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    fs::create_dir_all(&td).unwrap();
    td
}

/// 実の正本から入口の面を一時 dir へ生成する（手書きの見本は退役済み・生成器の出力が唯一の面）。戻り値 = 面の path。
fn generated_index(td: &Path) -> PathBuf {
    let page = td.join("generated-index.html");
    let out = Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(["face", "--face", "index", "--dir"])
        .arg(design_intent())
        .arg("--out")
        .arg(&page)
        .arg("--write")
        .output()
        .expect("folio を起動できない");
    assert_eq!(
        code(&out),
        0,
        "入口の面を生成できない: {}{}",
        stdout(&out),
        stderr(&out)
    );
    page
}

/// 入口の面の写しに変異を 1 つ当て、`--page index=<写し>` で撃つ。戻り値は folio の出力。
fn mutated_index(case: &str, mutate: impl FnOnce(&str) -> String) -> Output {
    let td = temp_dir(case);
    let page = td.join("index.html");
    let before = fs::read_to_string(generated_index(&td)).unwrap();
    let after = mutate(&before);
    assert_ne!(before, after, "{case}: 変異が当たっていない");
    fs::write(&page, after).unwrap();
    let out = parts_check(
        &design_intent(),
        &["--page", &format!("index={}", page.display())],
    );
    let _ = fs::remove_dir_all(&td);
    out
}

fn assert_fail_with(out: &Output, case: &str, wording: &str) {
    let text = stdout(out);
    assert_eq!(code(out), 1, "{case}: {text}{}", stderr(out));
    assert!(
        text.contains(wording),
        "{case}: 「{wording}」が無い: {text}"
    );
    assert!(text.contains("違反 1・"), "{case}: 違反が 1 でない: {text}");
}

// ── 見本 3 面 ──

#[test]
fn parts_check_passes_on_the_three_generated_faces() {
    let td = temp_dir("three-faces");
    let site = td.join("site");
    let built = Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(["build", "--dir"])
        .arg(design_intent())
        .arg("--out")
        .arg(&site)
        .arg("--write")
        .output()
        .expect("folio を起動できない");
    assert_eq!(code(&built), 0, "{}{}", stdout(&built), stderr(&built));
    let pages: Vec<String> = ["index", "constitution", "srs"]
        .iter()
        .map(|f| format!("{f}={}", site.join(format!("{f}.html")).display()))
        .collect();
    let mut args = Vec::new();
    for p in &pages {
        args.push("--page");
        args.push(p.as_str());
    }
    let out = parts_check(&design_intent(), &args);
    let _ = fs::remove_dir_all(&td);
    let text = stdout(&out);
    assert_eq!(code(&out), 0, "{text}{}", stderr(&out));
    assert!(text.contains("違反 0"), "{text}");
}

#[test]
fn parts_check_is_unknown_without_pages_now_that_the_samples_are_retired() {
    let out = parts_check(&design_intent(), &[]);
    assert_unknown(&out, "no-pages", "読めない");
}

// ── 変異 1 つで不合格 ──

#[test]
fn parts_check_fails_on_a_class_outside_the_catalog() {
    let out = mutated_index("extra-class", |t| {
        t.replacen(
            "class=\"skip-link\"",
            "class=\"skip-link not-in-folio-css\"",
            1,
        )
    });
    assert_fail_with(
        &out,
        "extra-class",
        "部品目録に無い class「not-in-folio-css」",
    );
    assert!(
        stdout(&out).contains("[R-3] index.html: "),
        "{}",
        stdout(&out)
    );
}

#[test]
fn parts_check_fails_on_a_component_outside_the_catalog() {
    let out = mutated_index("extra-part", |t| {
        t.replacen(
            "data-component=\"hub-cover\"",
            "data-component=\"no-such-part\"",
            1,
        )
    });
    assert_fail_with(&out, "extra-part", "部品目録に無い部品「no-such-part」");
    assert!(
        stdout(&out).contains("[parts] index.html: "),
        "{}",
        stdout(&out)
    );
}

#[test]
fn parts_check_fails_on_a_component_not_allowed_on_the_face() {
    // doc-cover-band は憲法と要件書だけに置ける
    let out = mutated_index("wrong-face", |t| {
        t.replacen(
            "<h1>folio2 — 設計文書の入口</h1>",
            "<h1 data-component=\"doc-cover-band\">folio2 — 設計文書の入口</h1>",
            1,
        )
    });
    assert_fail_with(
        &out,
        "wrong-face",
        "部品「doc-cover-band」はこの面に置けない",
    );
}

#[test]
fn parts_check_fails_on_a_disallowed_inline_style_prop() {
    let out = mutated_index("bad-style", |t| {
        t.replacen(
            "style=\"--shelf-n:3\"",
            "style=\"--shelf-n:3; color: red\"",
            1,
        )
    });
    assert_fail_with(&out, "bad-style", "許されない行内の様式「color」");
}

#[test]
fn parts_check_fails_on_an_inline_style_piece_without_a_colon() {
    // 「:」の無い片は、許す性質と同じ字面 --shelf-n でも違反 1
    let out = mutated_index("no-colon-style", |t| {
        t.replacen(
            "style=\"--shelf-n:3\"",
            "style=\"--shelf-n:3; --shelf-n\"",
            1,
        )
    });
    assert_fail_with(&out, "no-colon-style", "許されない行内の様式「--shelf-n」");
}

// ── 読めない入力 = 終了 2 ──

fn assert_unknown(out: &Output, case: &str, wording: &str) {
    let err = stderr(out);
    assert_eq!(code(out), 2, "{case}: {}{err}", stdout(out));
    assert!(
        err.contains("# まだ分からない: ") && err.contains(wording),
        "{case}: 「{wording}」が無い: {err}"
    );
    assert!(
        stdout(out).contains("folio parts: まだ分からない（"),
        "{case}: {}",
        stdout(out)
    );
}

#[test]
fn parts_check_is_unknown_when_a_page_is_missing() {
    let td = temp_dir("missing-page");
    let missing = td.join("index.html");
    let out = parts_check(
        &design_intent(),
        &["--page", &format!("index={}", missing.display())],
    );
    let _ = fs::remove_dir_all(&td);
    assert_unknown(&out, "missing-page", "読めない");
}

#[test]
fn parts_check_is_unknown_when_a_class_value_is_unquoted() {
    let out = mutated_index("unquoted", |t| {
        t.replacen("class=\"skip-link\"", "class=skip-link", 1)
    });
    assert_unknown(&out, "unquoted", "引用符で囲まれていない");
}

#[test]
fn parts_check_is_unknown_when_a_comment_is_unclosed() {
    let out = mutated_index("unclosed-comment", |t| format!("{t}<!-- 閉じない注釈\n"));
    assert_unknown(&out, "unclosed-comment", "閉じない注釈");
}

#[test]
fn parts_check_is_unknown_when_css_is_missing() {
    let td = temp_dir("missing-css");
    let missing = td.join("folio.css");
    let out = parts_check(&design_intent(), &["--css", &missing.display().to_string()]);
    let _ = fs::remove_dir_all(&td);
    assert_unknown(&out, "missing-css", "様式の定義");
}

#[test]
fn parts_check_is_unknown_when_the_face_name_is_not_one_of_three() {
    let td = temp_dir("bad-face");
    let page = generated_index(&td);
    let out = parts_check(
        &design_intent(),
        &["--page", &format!("hub={}", page.display())],
    );
    let _ = fs::remove_dir_all(&td);
    assert_unknown(&out, "bad-face", "面の名「hub」");
}

#[test]
fn parts_check_is_unknown_when_the_catalog_differs_from_build_time() {
    let td = temp_dir("catalog-drift");
    fs::create_dir_all(td.join("preview")).unwrap();
    let before = fs::read_to_string(design_intent().join("preview/parts.json")).unwrap();
    let removed = "    \"hub-cover\": {\n      \"role\": \"cover\",\n      \"faces\": [\n        \"index\"\n      ]\n    },\n";
    let after = before.replacen(removed, "", 1);
    assert_ne!(before, after, "catalog-drift: 変異が当たっていない");
    fs::write(td.join("preview/parts.json"), after).unwrap();
    let out = parts_check(&td, &[]);
    let _ = fs::remove_dir_all(&td);
    assert_unknown(&out, "catalog-drift", "組み立て時の部品目録と違う");
    assert!(
        stdout(&out).contains("まだ分からない 1）"),
        "2〜4 を数えている: {}",
        stdout(&out)
    );
}

// ── 部品目録の上限と図の型の名札（便 52）──

fn catalog_text() -> String {
    fs::read_to_string(design_intent().join("preview/parts.json")).expect("部品目録が読めない")
}

/// 部品目録の 1 か所を書き換えた変異（置き換えが当たらなければ歯の側の誤り）。
fn mutate(text: &str, from: &str, to: &str) -> String {
    assert_eq!(
        text.matches(from).count(),
        1,
        "変異の的「{from}」が部品目録に 1 つでない"
    );
    text.replace(from, to)
}

/// 変異した部品目録の写しを一時 dir の preview/ に置き、`--check --dir <一時 dir>` で撃つ。
fn check_with_mutated_catalog(case: &str, from: &str, to: &str) -> Output {
    let td = temp_dir(case);
    fs::create_dir_all(td.join("preview")).unwrap();
    fs::write(
        td.join("preview/parts.json"),
        mutate(&catalog_text(), from, to),
    )
    .unwrap();
    let out = parts_check(&td, &[]);
    let _ = fs::remove_dir_all(&td);
    out
}

const TYPE_IDS_TABLE: &str = "    \"type_ids\": {\n      \"archify-architecture\": \"構成図（architecture）\",\n      \"archify-workflow\": \"手順図（workflow）\",\n      \"archify-sequence\": \"順序図（sequence）\",\n      \"archify-dataflow\": \"流れ図（dataflow）\",\n      \"archify-lifecycle\": \"状態図（lifecycle）\"\n    },\n";

#[test]
fn parts_catalog_derives_limits_and_figure_type_labels_from_the_real_catalog() {
    let source = build::parts_catalog(&catalog_text()).expect("実の部品目録から導出できない");
    assert!(
        source.contains("pub const PIPELINE_RAIL_MAX_NODES: usize = "),
        "PIPELINE_RAIL_MAX_NODES が無い"
    );
    assert!(
        source.contains("pub const STATE_STRIP_MAX_NODES: usize = "),
        "STATE_STRIP_MAX_NODES が無い"
    );
    assert!(
        source.contains("pub const CONTEXT_BAND_MAX_PER_BAND: usize = "),
        "CONTEXT_BAND_MAX_PER_BAND が無い"
    );
    assert!(
        source.contains("pub const LIMITS: [(&str, &str, usize); "),
        "LIMITS が無い"
    );
    assert!(
        source.contains("pub const FIGURE_TYPE_LABELS: [(&str, &str); "),
        "FIGURE_TYPE_LABELS が無い"
    );
    assert!(
        source.contains("pub enum Component {"),
        "型 4 つの出力が無い"
    );
}

#[test]
fn parts_catalog_rejects_a_limit_that_is_a_string() {
    let text = mutate(&catalog_text(), "\"max_nodes\": 7", "\"max_nodes\": \"7\"");
    let err = build::parts_catalog(&text).expect_err("文字列の上限なのに Ok");
    assert!(
        err.contains("pipeline-rail") && err.contains("max_nodes"),
        "{err}"
    );
}

#[test]
fn parts_catalog_rejects_a_type_id_outside_the_figure_type_enum() {
    let text = mutate(
        &catalog_text(),
        "\"archify-architecture\": \"構成図",
        "\"archify-tower\": \"構成図",
    );
    let err = build::parts_catalog(&text).expect_err("figure_type_enum に無い鍵なのに Ok");
    assert!(
        err.contains("archify-tower") && err.contains("figure_type_enum"),
        "{err}"
    );
}

#[test]
fn parts_catalog_rejects_type_ids_written_as_a_list() {
    let text = mutate(
        &catalog_text(),
        TYPE_IDS_TABLE,
        "    \"type_ids\": [\n      \"archify-architecture\"\n    ],\n",
    );
    let err = build::parts_catalog(&text).expect_err("type_ids が一覧なのに Ok");
    assert!(err.contains("type_ids"), "{err}");
}

#[test]
fn parts_check_is_unknown_when_a_catalog_limit_differs_from_build_time() {
    let out = check_with_mutated_catalog("limit-drift", "\"max_nodes\": 7", "\"max_nodes\": 8");
    assert_unknown(&out, "limit-drift", "組み立て時の部品目録と違う");
}

#[test]
fn parts_check_is_unknown_when_a_figure_type_label_differs_from_build_time() {
    let out = check_with_mutated_catalog("label-drift", "手順図（workflow）", "手續図（workflow）");
    assert_unknown(&out, "label-drift", "組み立て時の部品目録と違う");
}

// ── 独立した凍結 fixture ──

#[test]
fn parts_check_passes_on_the_floor_fixture() {
    let out = parts_check(
        &design_intent(),
        &[
            "--css",
            &floor("mini.css").display().to_string(),
            "--page",
            &format!("index={}", floor("mini-ok.html").display()),
        ],
    );
    let text = stdout(&out);
    assert_eq!(code(&out), 0, "{text}{}", stderr(&out));
    assert!(text.contains("違反 0"), "{text}");
}

#[test]
fn parts_check_fails_on_the_floor_fixture_with_an_extra_class() {
    let out = parts_check(
        &design_intent(),
        &[
            "--css",
            &floor("mini.css").display().to_string(),
            "--page",
            &format!("index={}", floor("mini-extra.html").display()),
        ],
    );
    assert_fail_with(&out, "floor-extra", "部品目録に無い class「mini-extra」");
}

#[test]
fn parts_print_matches_the_floor_catalog_byte_for_byte() {
    let out = Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("parts")
        .arg("--print")
        .output()
        .unwrap();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(out.stdout, fs::read(floor("parts-catalog.json")).unwrap());
}

// ── 密度 profile の閉じた一覧（便 68・delivery-68.md §1 (d) 1・2）──

fn parts_print() -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("parts")
        .arg("--print")
        .output()
        .unwrap();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    stdout(&out)
}

/// `open` の直後から次の `]` までの文字列の一覧（区切りは「,」・前後の空白と引用符を外す）。
fn string_list_after(text: &str, open: &str) -> Vec<String> {
    let start = text
        .find(open)
        .unwrap_or_else(|| panic!("「{open}」が無い"))
        + open.len();
    let end = start + text[start..].find(']').expect("一覧が閉じていない");
    text[start..end]
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[test]
fn profiles_are_in_the_catalog_print() {
    let text = parts_print();
    assert!(text.contains("\"profiles\":[\"design-note\"]"), "{text}");
}

#[test]
fn profiles_print_matches_the_parts_json() {
    let printed = string_list_after(&parts_print(), "\"profiles\":[");
    let catalog = string_list_after(&catalog_text(), "\"profile_enum\": [");
    assert!(!catalog.is_empty(), "parts.json の profile_enum が空");
    assert_eq!(printed, catalog);
}

#[test]
fn parts_mode_is_exactly_one() {
    for args in [&["--check", "--print"][..], &[][..]] {
        let out = Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("parts")
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}: {}", stderr(&out));
    }
}
