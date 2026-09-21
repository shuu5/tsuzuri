//! `folio face` の歯（便 14・docs/design/delivery-14.md §1 (e)）。binary 経由。
//! - 凍結 fixture（tests/fixtures/face/）との byte 一致（P-10.1・回帰の anchor で唯一の判定にしない）
//! - 実の正本で生成した面が `folio parts --check` に合格する（AC2 の機構・生成側から独立した物差し 1）
//! - 逐語と件数の census（yaml-rust2 で正本を直に読む・生成側から独立した物差し 2）
//! - check の 3 値・表に無い面の名・導出できない入力 8 つ・escape
//! - 組み立てた版と読んでいる版のずれ（便 50・docs/design/delivery-50.md §1 (d)(e) 3）: 実の置き場の写しで憲法の
//!   値域に値を足す・知らない鍵を足す → 2 ∧ 標準エラーに鍵の名と「組み立て時の憲法の値域と違う」
//!
//! 入口の面（便 16・20〜22）の歯は `face_index.rs` へ移した（便 26・docs/design/delivery-26.md §1 (c)）。
//! 図の章（便 34・FR15）: 写し（srs.yaml・図 1 枚）の toc の 09 と figure-panel と型の名札と根拠のリンク・図なしの面は
//! 図の章の外が byte で同じ・通らない図で 2 と前の面の保持・型外・道具の不在・実の正本の figure-panel の数。
//! 版管理の `design-intent/preview/constitution.html`（手書きの見本）は書き換えない（`--out` は必ず一時 dir の中）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use yaml_rust2::{Yaml, YamlLoader};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture() -> PathBuf {
    repo_root().join("tests/fixtures/face")
}

fn design_intent() -> PathBuf {
    repo_root().join("design-intent")
}

fn vendor() -> PathBuf {
    repo_root().join("vendor/archify")
}

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-face-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    fs::create_dir_all(&td).unwrap();
    td
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

/// fixture の正本 4 file を一時 dir の下の src/ へ、repo の vendor/archify/（図の道具・便 34 の要件書の面は図ごとに
/// 撃つ）を親 dir の vendor/archify/ へ写す（expected.html は写さない）。戻り値 = (一時 dir, 写し)。
fn fixture_copy(case: &str) -> (PathBuf, PathBuf) {
    let td = temp_dir(case);
    let work = td.join("src");
    fs::create_dir_all(&work).unwrap();
    for name in [
        "constitution.yaml",
        "rules.yaml",
        "vocabulary.yaml",
        "srs.yaml",
        "ceiling.yaml",
    ] {
        fs::copy(fixture().join(name), work.join(name)).unwrap();
    }
    copy_dir(&vendor(), &td.join("vendor/archify"));
    (td, work)
}

fn folio_face(face: &str, dir: &Path, out: &Path, mode: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("face")
        .arg("--face")
        .arg(face)
        .arg("--dir")
        .arg(dir)
        .arg("--out")
        .arg(out)
        .arg(mode)
        .output()
        .expect("folio を起動できない")
}

fn code(out: &Output, what: &str) -> i32 {
    out.status.code().unwrap_or_else(|| {
        panic!(
            "{what} が signal で終わった: {}",
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

fn edit(path: &Path, f: impl FnOnce(&str) -> String) {
    let before = fs::read_to_string(path).unwrap();
    let after = f(&before);
    assert_ne!(before, after, "変異が当たっていない: {}", path.display());
    fs::write(path, after).unwrap();
}

/// 5 字の escape（生成側の字面を使わず歯の側で持つ）。
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

// ── 凍結 fixture ──

#[test]
fn face_write_matches_the_frozen_fixture() {
    let td = temp_dir("anchor");
    let out = td.join("constitution.html");
    let run = folio_face("constitution", &fixture(), &out, "--write");
    let written = fs::read(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert_eq!(
        stdout(&run),
        format!("folio face: 書いた（{} byte）\n", written.len())
    );
    let frozen = fs::read(fixture().join("expected.html")).unwrap();
    if written != frozen {
        let at = written
            .iter()
            .zip(&frozen)
            .position(|(a, b)| a != b)
            .unwrap_or(written.len().min(frozen.len()));
        let show = |b: &[u8]| {
            String::from_utf8_lossy(&b[at.saturating_sub(120)..(at + 200).min(b.len())])
                .into_owned()
        };
        panic!(
            "expected.html と違う（{} byte ≠ {} byte・最初の差 {at} byte 目）\n--- folio\n{}\n--- 凍結\n{}",
            written.len(),
            frozen.len(),
            show(&written),
            show(&frozen)
        );
    }
}

#[test]
fn face_escapes_values_from_the_sources() {
    let td = temp_dir("escape");
    let out = td.join("constitution.html");
    let run = folio_face("constitution", &fixture(), &out, "--write");
    let html = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    for want in ["&lt;b&gt;", "&amp;", "&quot;"] {
        assert!(html.contains(want), "{want} が無い");
    }
    // 語彙の def の生の字面（<b>太字</b>・二重引用符）は出ない
    assert!(
        !html.contains("<b>太字</b>"),
        "def の <b> が escape されていない"
    );
    assert!(
        !html.contains("\"引用\""),
        "def の二重引用符が escape されていない"
    );
    assert!(!html.contains("A & B"), "def の & が escape されていない");
}

// ── 実の正本 ──

/// 実の正本から一時 file へ書く。戻り値 = (一時 dir, 面の本文)。
fn real_face(case: &str) -> (PathBuf, PathBuf, String) {
    let td = temp_dir(case);
    let out = td.join("constitution.html");
    let run = folio_face("constitution", &design_intent(), &out, "--write");
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert!(stdout(&run).contains("書いた"), "{}", stdout(&run));
    let html = fs::read_to_string(&out).unwrap();
    (td, out, html)
}

#[test]
fn face_on_the_real_sources_passes_parts_check() {
    let (td, out, _) = real_face("parts");
    let check = Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("parts")
        .arg("--check")
        .arg("--dir")
        .arg(design_intent())
        .arg("--page")
        .arg(format!("constitution={}", out.display()))
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        code(&check, "folio parts --check"),
        0,
        "{}{}",
        stdout(&check),
        stderr(&check)
    );
    assert!(stdout(&check).contains("違反 0"), "{}", stdout(&check));
}

fn load_yaml_at(dir: &Path, name: &str) -> Yaml {
    let text = fs::read_to_string(dir.join(name)).unwrap();
    YamlLoader::load_from_str(&text).unwrap().remove(0)
}

fn load_yaml(name: &str) -> Yaml {
    load_yaml_at(&design_intent(), name)
}

fn seq<'a>(y: &'a Yaml, what: &str) -> &'a Vec<Yaml> {
    y.as_vec().unwrap_or_else(|| panic!("{what} が一覧でない"))
}

fn text<'a>(y: &'a Yaml, key: &str) -> &'a str {
    y[key]
        .as_str()
        .unwrap_or_else(|| panic!("欄 {key} が文字列でない: {y:?}"))
}

/// 属性 data-component の値を出た順に。
fn components(html: &str) -> Vec<&str> {
    let needle = "data-component=\"";
    html.match_indices(needle)
        .map(|(i, _)| {
            let rest = &html[i + needle.len()..];
            &rest[..rest.find('"').unwrap()]
        })
        .collect()
}

#[test]
fn face_census_on_the_real_sources_counts_and_verbatims() {
    let (td, _, html) = real_face("census");
    let _ = fs::remove_dir_all(&td);
    let c = load_yaml("constitution.yaml");
    let r = load_yaml("rules.yaml");
    let v = load_yaml("vocabulary.yaml");

    // 逐語（title・plain・全規範文の text）
    let articles = seq(&c["articles"], "articles");
    for a in articles {
        for key in ["title", "plain"] {
            let want = esc(text(a, key));
            assert!(html.contains(&want), "{key} が面に無い: {want}");
        }
        for st in seq(&a["statements"], "statements") {
            let want = esc(text(st, "text"));
            assert!(html.contains(&want), "規範文が面に無い: {want}");
        }
    }

    // 件数
    let count = |needle: &str| html.matches(needle).count();
    let parts = components(&html);
    let parts_of = |name: &str| parts.iter().filter(|p| **p == name).count();
    assert_eq!(parts_of("item-row"), articles.len(), "item-row の数");
    assert_eq!(
        parts_of("rail-node"),
        seq(&c["amendment"]["steps"], "steps").len(),
        "rail-node の数"
    );
    // 語の行 = terms の数 + 欄の名前の節（field_terms・便 84）の数
    assert_eq!(
        count("class=\"grow\""),
        seq(&v["terms"], "terms").len() + v["field_terms"].as_vec().map_or(0, Vec::len),
        "grow の数"
    );
    assert_eq!(
        parts_of("glossary-term-table"),
        1 + usize::from(v["field_terms"].as_vec().is_some_and(|f| !f.is_empty())),
        "glossary-term-table の数"
    );
    assert_eq!(
        count("<tr id=\"r-"),
        seq(&r["thresholds"], "thresholds").len(),
        "閾値行の tr の数"
    );
    assert_eq!(
        count("<tr id=\"d-"),
        seq(&r["discipline"], "discipline").len(),
        "開発規律行の tr の数"
    );

    // 部品の名札は 15 種の中だけ・lane-chip を含まない
    const ALLOWED: [&str; 15] = [
        "freshness-stamp",
        "ceiling-stamp",
        "font-size-control",
        "doc-cover-band",
        "chapter-deck-band",
        "section-lead-callout",
        "item-row",
        "principle-amendment-history",
        "figure-panel",
        "pipeline-rail",
        "rail-node",
        "stepper",
        "amendment-example",
        "glossary-term-table",
        "approval-block",
    ];
    assert!(!parts.is_empty());
    for p in &parts {
        assert!(ALLOWED.contains(p), "15 種に無い部品「{p}」");
    }
    assert!(!html.contains("lane-chip"));
}

// ── check の 3 値 ──

#[test]
fn face_check_has_three_values() {
    let (td, work) = fixture_copy("check");
    let out = td.join("constitution.html");

    let write = folio_face("constitution", &work, &out, "--write");
    let ok = folio_face("constitution", &work, &out, "--check");
    let mut bytes = fs::read(&out).unwrap();
    bytes[0] ^= 0x20;
    fs::write(&out, &bytes).unwrap();
    let drift = folio_face("constitution", &work, &out, "--check");
    fs::remove_file(&out).unwrap();
    let missing = folio_face("constitution", &work, &out, "--check");
    let rewrite = folio_face("constitution", &work, &out, "--write");
    let after = folio_face("constitution", &work, &out, "--check");
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&write, "write"), 0, "{}", stderr(&write));
    assert_eq!(code(&ok, "check（一致）"), 0, "{}", stderr(&ok));
    assert!(stdout(&ok).contains("folio face: OK"), "{}", stdout(&ok));
    assert_eq!(code(&drift, "check（不一致）"), 1, "{}", stderr(&drift));
    assert!(stderr(&drift).contains("DRIFT"), "{}", stderr(&drift));
    assert_eq!(code(&missing, "check（無い）"), 2, "{}", stderr(&missing));
    assert!(
        stderr(&missing).contains("面が無い"),
        "{}",
        stderr(&missing)
    );
    assert_eq!(code(&rewrite, "write（再）"), 0, "{}", stderr(&rewrite));
    assert_eq!(code(&after, "check（再生成後）"), 0, "{}", stderr(&after));
}

#[test]
fn face_mode_is_exactly_one() {
    for args in [&["--check", "--write"][..], &[][..]] {
        let out = Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("face")
            .arg("--face")
            .arg("constitution")
            .arg("--dir")
            .arg(fixture())
            .arg("--out")
            .arg(std::env::temp_dir().join("folio-face-never-written.html"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}: {}", stderr(&out));
    }
}

// ── 面の名 ──

#[test]
fn face_name_outside_the_three_is_unknown() {
    let td = temp_dir("foo");
    let out = td.join("foo.html");
    let run = folio_face("foo", &fixture(), &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --face foo"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("どれでもない"), "{}", stderr(&run));
    assert!(!exists);
}

// ── 導出できない入力 ──

/// fixture の写しに変異を 1 つ当て、`--write` = 2 ∧「まだ分からない」∧ 出力先が出来ていない。
fn unknown(case: &str, mutate: impl FnOnce(&Path), out_of: impl FnOnce(&Path) -> PathBuf) {
    let (td, work) = fixture_copy(&format!("unknown-{case}"));
    mutate(&work);
    let out = out_of(&td);
    let run = folio_face("constitution", &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face"), 2, "{case}: {}", stderr(&run));
    assert!(
        stderr(&run).contains("まだ分からない"),
        "{case}: {}",
        stderr(&run)
    );
    assert!(!exists, "{case}: 導出できないのに出力先に書いた");
}

fn plain_out(td: &Path) -> PathBuf {
    td.join("never.html")
}

#[test]
fn face_unknown_when_a_tier_is_outside_the_table() {
    unknown(
        "tier",
        |w| {
            edit(&w.join("constitution.yaml"), |t| {
                t.replacen("    tier: never\n", "    tier: sometimes\n", 1)
            })
        },
        plain_out,
    );
}

/// 実の設計文書の置き場の写し（面が読む 5 file）に変異を 1 つ当て、`--write` = 2 ∧「まだ分からない」∧ 標準エラーが
/// 各 `wants` を含む ∧ 出力先が出来ていない（便 50 §1 (e) 3）。
fn real_unknown(case: &str, from: &str, to: &str, wants: &[&str]) {
    let td = temp_dir(&format!("real-unknown-{case}"));
    let work = td.join("src");
    fs::create_dir_all(&work).unwrap();
    for name in [
        "constitution.yaml",
        "rules.yaml",
        "vocabulary.yaml",
        "srs.yaml",
        "ceiling.yaml",
    ] {
        fs::copy(design_intent().join(name), work.join(name)).unwrap();
    }
    edit(&work.join("constitution.yaml"), |t| t.replacen(from, to, 1));
    let out = td.join("never.html");
    let run = folio_face("constitution", &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face"), 2, "{case}: {}", stderr(&run));
    assert!(
        stderr(&run).contains("まだ分からない"),
        "{case}: {}",
        stderr(&run)
    );
    for want in wants {
        assert!(
            stderr(&run).contains(want),
            "{case}: 「{want}」が無い: {}",
            stderr(&run)
        );
    }
    assert!(!exists, "{case}: 導出できないのに出力先に書いた");
}

/// 読んでいる置き場の憲法の値域に値が足された = 組み立てた版とずれる（便 50 §1 (d)）。
#[test]
fn face_unknown_when_an_enum_value_is_added_to_the_sources() {
    real_unknown(
        "enum-value-added",
        "    pattern: [ubiquitous, event, state, unwanted, optional]\n",
        "    pattern: [ubiquitous, event, state, unwanted, optional, sometimes]\n",
        &["schema.enums.pattern", "組み立て時の憲法の値域と違う"],
    );
}

/// 読んでいる置き場の schema.enums に組み立てた版に無い鍵が在る（便 50 §1 (d)）。
#[test]
fn face_unknown_when_an_enum_key_is_not_in_the_built_constitution() {
    real_unknown(
        "enum-key-unknown",
        "    retreat_kind: [spike, measure, ruling]\n",
        "    retreat_kind: [spike, measure, ruling]\n    mood: [calm, tense]\n",
        &["schema.enums.mood", "組み立て時の憲法の値域と違う"],
    );
}

#[test]
fn face_unknown_when_counts_differ_from_the_articles() {
    unknown(
        "counts",
        |w| {
            edit(&w.join("constitution.yaml"), |t| {
                t.replacen(
                    "{always: 1, ask-first: 1, never: 1}",
                    "{always: 2, ask-first: 1, never: 1}",
                    1,
                )
            })
        },
        plain_out,
    );
}

#[test]
fn face_unknown_when_relations_name_a_missing_article() {
    unknown(
        "relations-article",
        |w| {
            edit(&w.join("constitution.yaml"), |t| {
                t.replacen(
                    "articles: [A-1], sections",
                    "articles: [A-1, X-9], sections",
                    1,
                )
            })
        },
        plain_out,
    );
}

#[test]
fn face_unknown_when_a_step_names_a_missing_article() {
    unknown(
        "step-article",
        |w| {
            edit(&w.join("constitution.yaml"), |t| {
                t.replacen("article: [A-1, N-1]}", "article: [A-1, N-1, X-9]}", 1)
            })
        },
        plain_out,
    );
}

#[test]
fn face_unknown_when_a_rules_status_is_outside_the_table() {
    unknown(
        "rules-status",
        |w| {
            edit(&w.join("rules.yaml"), |t| {
                t.replacen("    status: 仮\n", "    status: 保留\n", 1)
            })
        },
        plain_out,
    );
}

#[test]
fn face_unknown_when_srs_is_missing() {
    unknown(
        "srs-missing",
        |w| fs::remove_file(w.join("srs.yaml")).unwrap(),
        plain_out,
    );
}

#[test]
fn face_unknown_when_constitution_has_a_duplicate_key() {
    unknown(
        "duplicate-key",
        |w| {
            edit(&w.join("constitution.yaml"), |t| {
                format!("{t}rules_pointer: もう 1 度\n")
            })
        },
        plain_out,
    );
}

#[test]
fn face_unknown_when_the_out_parent_dir_is_missing() {
    unknown("out-parent", |_| {}, |td| td.join("no-such-dir/never.html"));
}

// ── 要件書の面（便 15・docs/design/delivery-15.md §1 (e)）──

/// 2 つの byte 列が同じでなければ最初の差の前後を見せて落ちる。
fn assert_same_bytes(written: &[u8], frozen: &[u8], what: &str) {
    if written == frozen {
        return;
    }
    let at = written
        .iter()
        .zip(frozen)
        .position(|(a, b)| a != b)
        .unwrap_or(written.len().min(frozen.len()));
    let show = |b: &[u8]| {
        String::from_utf8_lossy(&b[at.saturating_sub(120).min(b.len())..(at + 200).min(b.len())])
            .into_owned()
    };
    panic!(
        "{what} と違う（{} byte ≠ {} byte・最初の差 {at} byte 目）\n--- folio\n{}\n--- 期待\n{}",
        written.len(),
        frozen.len(),
        show(written),
        show(frozen)
    );
}

#[test]
fn face_srs_write_matches_the_frozen_fixture() {
    let (td, work) = fixture_copy("srs-anchor");
    let out = td.join("srs.html");
    let run = folio_face("srs", &work, &out, "--write");
    let written = fs::read(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        code(&run, "folio face --face srs --write"),
        0,
        "{}",
        stderr(&run)
    );
    assert_eq!(
        stdout(&run),
        format!("folio face: 書いた（{} byte）\n", written.len())
    );
    let frozen = fs::read(fixture().join("expected-srs.html")).unwrap();
    assert_same_bytes(&written, &frozen, "expected-srs.html");
}

#[test]
fn face_srs_escapes_values_from_the_sources() {
    let (td, work) = fixture_copy("srs-escape");
    let out = td.join("srs.html");
    let run = folio_face("srs", &work, &out, "--write");
    let html = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --face srs"), 0, "{}", stderr(&run));
    assert!(html.contains("&lt;b&gt;太字&lt;/b&gt; と A &amp; B と &quot;引用&quot;"));
    assert!(
        !html.contains("<b>太字</b>"),
        "promise の <b> が escape されていない"
    );
}

/// 実の正本から要件書の面を一時 file へ書く。戻り値 = (一時 dir, 出力先, 面の本文)。
fn real_srs(case: &str) -> (PathBuf, PathBuf, String) {
    let td = temp_dir(case);
    let out = td.join("srs.html");
    let run = folio_face("srs", &design_intent(), &out, "--write");
    assert_eq!(
        code(&run, "folio face --face srs --write"),
        0,
        "{}",
        stderr(&run)
    );
    assert!(stdout(&run).contains("書いた"), "{}", stdout(&run));
    let html = fs::read_to_string(&out).unwrap();
    (td, out, html)
}

#[test]
fn face_srs_on_the_real_sources_passes_parts_check() {
    let (td, out, html) = real_srs("srs-parts");
    let check = Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("parts")
        .arg("--check")
        .arg("--dir")
        .arg(design_intent())
        .arg("--page")
        .arg(format!("srs={}", out.display()))
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        code(&check, "folio parts --check"),
        0,
        "{}{}",
        stdout(&check),
        stderr(&check)
    );
    assert!(stdout(&check).contains("違反 0"), "{}", stdout(&check));
    // 図の枠の数 = 図 1〜3 の分（図 3 は verdicts の節が在るときだけ）+ 正本の図の節の数（便 34・実測 0）
    let s = load_yaml("srs.yaml");
    let builtin = 2 + usize::from(s["verdicts"].as_vec().is_some());
    let figures = s["figures"].as_vec().map_or(0, Vec::len);
    assert_eq!(
        html.matches("data-component=\"figure-panel\"").count(),
        builtin + figures,
        "figure-panel の数が 図 1〜3 の分 + figures の数と違う"
    );
}

/// `open` の後の最初の `close` までの字面。
fn between<'a>(html: &'a str, open: &str, close: &str) -> &'a str {
    let start = html
        .find(open)
        .unwrap_or_else(|| panic!("「{open}」が無い"))
        + open.len();
    let end = html[start..]
        .find(close)
        .unwrap_or_else(|| panic!("「{open}」の後に「{close}」が無い"));
    &html[start..start + end]
}

#[test]
fn face_srs_census_on_the_real_sources_counts_and_verbatims() {
    let (td, _, html) = real_srs("srs-census");
    let _ = fs::remove_dir_all(&td);
    let s = load_yaml("srs.yaml");
    let v = load_yaml("vocabulary.yaml");

    // 逐語
    let fr = seq(&s["requirements"], "requirements");
    let nfr = seq(&s["nonfunctional"], "nonfunctional");
    for x in fr.iter().chain(nfr) {
        for key in ["title", "shall", "plain"] {
            let want = esc(text(x, key));
            assert!(html.contains(&want), "{key} が面に無い: {want}");
        }
    }
    let acs = seq(&s["acceptance"], "acceptance");
    for x in acs {
        for key in ["title", "plain"] {
            let want = esc(text(x, key));
            assert!(html.contains(&want), "受入の {key} が面に無い: {want}");
        }
    }
    for x in seq(&s["constraints"], "constraints") {
        let want = esc(text(x, "text"));
        assert!(html.contains(&want), "制約の text が面に無い: {want}");
    }

    // 件数
    let parts = components(&html);
    let parts_of = |name: &str| parts.iter().filter(|p| **p == name).count();
    assert_eq!(parts_of("item-row"), fr.len() + nfr.len(), "item-row の数");
    assert_eq!(
        parts_of("band-node"),
        seq(&s["actors"], "actors").len() + seq(&s["outputs"], "outputs").len(),
        "band-node の数"
    );
    assert_eq!(
        parts_of("rail-node"),
        seq(&s["rail"], "rail").len(),
        "rail-node の数"
    );
    let rtm = between(&html, "<table class=\"rtm\">", "</tbody>");
    assert_eq!(
        rtm.matches("<tr>").count() - 1,
        fr.len() + nfr.len(),
        "対応表の tbody の tr の数（thead の 1 行を除く）"
    );
    // 章 08 は憲法の面の章 07 と同じ形（便 36）: 語の行 = terms の数・目次へ = terms の数・id g-<語の id> が terms の順
    let glossary = between(&html, "data-component=\"glossary-term-table\">", "\n</div>");
    let terms = seq(&v["terms"], "terms");
    assert_eq!(
        glossary.matches("<div class=\"grow\"").count(),
        terms.len(),
        "用語の行（div.grow）の数"
    );
    assert_eq!(
        glossary
            .matches("<a class=\"back\" href=\"#toc\">目次へ</a>")
            .count(),
        terms.len(),
        "用語の行の目次へ（a.back）の数"
    );
    let ids = terms
        .iter()
        .map(|t| {
            let needle = format!("<div class=\"grow\" id=\"g-{}\">", text(t, "id"));
            glossary
                .find(&needle)
                .unwrap_or_else(|| panic!("語の行が無い: {needle}"))
        })
        .collect::<Vec<_>>();
    assert!(
        ids.windows(2).all(|w| w[0] < w[1]),
        "語の行の id が terms の順でない"
    );
    assert_eq!(parts_of("ac-state-chip"), acs.len(), "ac-state-chip の数");
    // 章の帯は 8 章 + 図の章（正本の figures が 1 枚以上のときだけ・便 34）+ 承認欄
    let figures = s["figures"].as_vec().map_or(0, Vec::len);
    assert_eq!(
        parts_of("chapter-deck-band"),
        8 + usize::from(figures > 0) + 1,
        "chapter-deck-band の数"
    );

    // 部品の名札は 18 種の中だけ・lane-chip を含まない
    const ALLOWED: [&str; 18] = [
        "freshness-stamp",
        "ceiling-stamp",
        "font-size-control",
        "doc-cover-band",
        "chapter-deck-band",
        "section-lead-callout",
        "figure-panel",
        "context-band",
        "band-node",
        "pipeline-rail",
        "rail-node",
        "state-strip",
        "state-node",
        "item-row",
        "ac-state-chip",
        "rtm-grid",
        "glossary-term-table",
        "approval-block",
    ];
    assert!(!parts.is_empty());
    for p in &parts {
        assert!(ALLOWED.contains(p), "18 種に無い部品「{p}」");
    }
    assert!(!html.contains("lane-chip"));
    assert!(
        !html.contains("glossary-links"),
        "退役した glossary-links が面に在る"
    );

    // 図 3: verdicts の節が在れば答えの数だけ・無ければ 0 で「図 3」は字面だけ（FR5）
    match s["verdicts"].as_vec() {
        Some(verdicts) => {
            assert_eq!(parts_of("state-strip"), 1);
            assert_eq!(parts_of("state-node"), verdicts.len());
        }
        None => {
            assert_eq!(parts_of("state-strip"), 0);
            assert!(
                !html.contains("#fig-verdicts"),
                "verdicts が無いのに図 3 へのリンク"
            );
            let fr5 = between(
                &html,
                "<article data-component=\"item-row\" id=\"fr5\">",
                "</article>",
            );
            assert_eq!(fr5.matches("図 3").count(), 1, "FR5 の「図 3」の字面");
        }
    }
}

// ── 要件書の面の用語集（便 36・章 08 は憲法の面の章 07 と同じ形）──

#[test]
fn face_srs_glossary_rows_are_byte_identical_to_the_constitution_face() {
    // 実の正本: 2 面の glossary-term-table の中身（語の行の区間）が byte で同じ（2 面とも vocabulary.yaml から導出）
    let (td_c, _, constitution) = real_face("glossary-c");
    let _ = fs::remove_dir_all(&td_c);
    let (td_s, _, srs) = real_srs("glossary-s");
    let _ = fs::remove_dir_all(&td_s);
    let open = "data-component=\"glossary-term-table\">";
    let rows_c = between(&constitution, open, "\n</div>");
    let rows_s = between(&srs, open, "\n</div>");
    assert!(
        rows_c.contains("<div class=\"grow\""),
        "憲法の面に語の行が無い"
    );
    assert_same_bytes(
        rows_s.as_bytes(),
        rows_c.as_bytes(),
        "要件書の面の語の行（憲法の面の語の行）",
    );
    // 憲法の面の語彙へのリンク（constitution.html#g-…）は要件書の面から消えた
    assert!(
        !srs.contains("constitution.html#g-"),
        "要件書の面に憲法の面の語彙へのリンクが残っている"
    );
}

#[test]
fn face_srs_glossary_chapter_has_the_constitution_h2_and_no_glossary_links() {
    let (td, work) = fixture_copy("srs-glossary-h2");
    let out = td.join("srs.html");
    let run = folio_face("srs", &work, &out, "--write");
    let html = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --face srs"), 0, "{}", stderr(&run));
    assert!(
        html.contains("<h2>本文に出てくる専門語のやさしい説明</h2>"),
        "章 08 の h2 が憲法の面の章 07 と同じ字でない"
    );
    assert!(
        html.contains("<span class=\"t\">本文に出てくる専門語のやさしい説明</span>"),
        "toc の 08 の h2 が憲法の面の章 07 と同じ字でない"
    );
    assert!(
        !html.contains("glossary-links"),
        "glossary-links の字が面に在る"
    );
    assert!(
        !html.contains("説明は憲法 §7 で"),
        "便 35 までの章 08 の h2 の字が面に残っている"
    );
    // 語の表 1 つ + 写しの語彙が field_terms を持つので欄の名前の表 1 つ（便 84）
    let vocab = fs::read_to_string(fixture().join("vocabulary.yaml")).unwrap();
    let vocab = YamlLoader::load_from_str(&vocab).unwrap().remove(0);
    let fields = vocab["field_terms"].as_vec().map_or(0, Vec::len);
    assert!(fields > 0, "写しの語彙に field_terms が無い");
    assert_eq!(
        html.matches("data-component=\"glossary-term-table\"")
            .count(),
        2,
        "glossary-term-table が 2 つでない"
    );
}

#[test]
fn face_srs_figure_3_appears_only_with_verdicts() {
    let (td, work) = fixture_copy("srs-verdicts");
    let out = td.join("srs.html");
    let with = folio_face("srs", &work, &out, "--write");
    let html_with = fs::read_to_string(&out).unwrap_or_default();
    edit(&work.join("srs.yaml"), |t| {
        let start = t.find("\nverdicts:\n").expect("verdicts の節が無い");
        let end = t
            .find("\nrequirements:\n")
            .expect("requirements の節が無い");
        format!("{}{}", &t[..start], &t[end..])
    });
    let without = folio_face("srs", &work, &out, "--write");
    let html_without = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&with, "verdicts 在り"), 0, "{}", stderr(&with));
    assert_eq!(code(&without, "verdicts 無し"), 0, "{}", stderr(&without));

    // 在る側: state-node 3・図 3 へのリンク
    let with_parts = components(&html_with);
    assert_eq!(with_parts.iter().filter(|p| **p == "state-node").count(), 3);
    assert!(html_with.contains("<a class=\"rq-where\" href=\"#fig-verdicts\">図 3</a>"));

    // 無い側: state-strip 0・「図3」の参照は字面だけ・章 09 の番号は 1 つ繰り上がる（図 4 → 図 3・便 35）・それ以外は同じ
    assert!(!components(&html_without).contains(&"state-strip"));
    assert!(!html_without.contains("#fig-verdicts"));
    let figure_open =
        "<figure data-component=\"figure-panel\" data-role=\"diagram\" id=\"fig-verdicts\">";
    let start = html_with.find(figure_open).unwrap();
    let end = start + html_with[start..].find("</figure>\n").unwrap() + "</figure>\n".len();
    let expected = format!("{}{}", &html_with[..start], &html_with[end..])
        .replacen(" · <a href=\"#fig-verdicts\">図 3</a>", "", 1)
        .replace(
            "<a class=\"rq-where\" href=\"#fig-verdicts\">図 3</a>",
            "図 3",
        )
        .replace("<a class=\"fig\" href=\"#fig-verdicts\">図 3</a>", "図 3")
        .replacen(
            "<a href=\"#fig-1\">図 4</a>",
            "<a href=\"#fig-1\">図 3</a>",
            1,
        )
        .replacen(
            "<span class=\"fn\">図 4</span>",
            "<span class=\"fn\">図 3</span>",
            1,
        )
        .replacen(
            "<span class=\"ver\">図 4 · ",
            "<span class=\"ver\">図 3 · ",
            1,
        );
    assert_same_bytes(
        html_without.as_bytes(),
        expected.as_bytes(),
        "verdicts を消した面の期待",
    );
}

#[test]
fn face_srs_check_has_three_values() {
    let (td, work) = fixture_copy("srs-check");
    let out = td.join("srs.html");

    let write = folio_face("srs", &work, &out, "--write");
    let ok = folio_face("srs", &work, &out, "--check");
    let mut bytes = fs::read(&out).unwrap();
    bytes[0] ^= 0x20;
    fs::write(&out, &bytes).unwrap();
    let drift = folio_face("srs", &work, &out, "--check");
    fs::remove_file(&out).unwrap();
    let missing = folio_face("srs", &work, &out, "--check");
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&write, "write"), 0, "{}", stderr(&write));
    assert_eq!(code(&ok, "check（一致）"), 0, "{}", stderr(&ok));
    assert!(stdout(&ok).contains("folio face: OK"), "{}", stdout(&ok));
    assert_eq!(code(&drift, "check（不一致）"), 1, "{}", stderr(&drift));
    assert!(stderr(&drift).contains("DRIFT"), "{}", stderr(&drift));
    assert_eq!(code(&missing, "check（無い）"), 2, "{}", stderr(&missing));
    assert!(
        stderr(&missing).contains("面が無い"),
        "{}",
        stderr(&missing)
    );
}

/// 要件書の面で、fixture の写しの srs.yaml に変異を 1 つ当て、`--write` = 2 ∧「まだ分からない」∧ 出力先が出来ていない。
fn srs_unknown(case: &str, from: &str, to: &str) {
    let (td, work) = fixture_copy(&format!("srs-unknown-{case}"));
    edit(&work.join("srs.yaml"), |t| t.replacen(from, to, 1));
    let out = td.join("never.html");
    let run = folio_face("srs", &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        code(&run, "folio face --face srs"),
        2,
        "{case}: {}",
        stderr(&run)
    );
    assert!(
        stderr(&run).contains("まだ分からない"),
        "{case}: {}",
        stderr(&run)
    );
    assert!(!exists, "{case}: 導出できないのに出力先に書いた");
}

#[test]
fn face_srs_unknown_when_counts_differ_from_the_rows() {
    srs_unknown("counts", "counts: {fr: 2,", "counts: {fr: 3,");
}

#[test]
fn face_srs_unknown_when_a_figure_names_a_missing_step() {
    srs_unknown(
        "figure-step",
        "figures: [図2-1, 図1]",
        "figures: [図2-9, 図1]",
    );
}

#[test]
fn face_srs_unknown_when_a_figure_is_outside_the_forms() {
    srs_unknown(
        "figure-form",
        "figures: [全段, 図3]",
        "figures: [全段, 図4]",
    );
}

#[test]
fn face_srs_unknown_when_a_step_names_a_missing_requirement() {
    srs_unknown("step-req", "reqs: [FR1], note", "reqs: [FR9], note");
}

#[test]
fn face_srs_unknown_when_basis_names_a_missing_article() {
    srs_unknown("basis", "    basis: [P-1]\n", "    basis: [P-9]\n");
}

#[test]
fn face_srs_unknown_when_a_verdict_tone_is_outside_the_table() {
    srs_unknown("tone", "tone: neutral", "tone: purple");
}

#[test]
fn face_srs_unknown_when_no_actor_is_the_tool() {
    srs_unknown("tool-actor", "role: 道具}", "role: 作る}");
}

#[test]
fn face_srs_unknown_when_a_pattern_is_outside_the_table() {
    srs_unknown("pattern", "pattern: event", "pattern: sometimes");
}

// ── 要件書の面の図の章（便 34・FR15）──

/// 写しの srs.yaml に変異を当て、`--write` の結果と面の本文を返す（面が出来ていなければ本文は空）。
fn srs_mutated(case: &str, mutate: impl FnOnce(&str) -> String) -> (Output, String) {
    let (td, work) = fixture_copy(case);
    edit(&work.join("srs.yaml"), mutate);
    let out = td.join("srs.html");
    let run = folio_face("srs", &work, &out, "--write");
    let html = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    (run, html)
}

/// 写しの srs.yaml から図の節（figures 以降・末尾まで）を消した面（便 33 までの形）。
fn srs_figureless_html(case: &str) -> String {
    let (run, html) = srs_mutated(case, |t| {
        let at = t.find("\nfigures:\n").expect("figures が無い");
        format!("{}\n", &t[..at])
    });
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    html
}

/// `a` から `b` の直前までを切り取る（a が無ければそのまま・a の後の最初の b・b が無ければ末尾まで）。
fn cut(html: &str, a: &str, b: &str) -> String {
    let Some(start) = html.find(a) else {
        return html.to_string();
    };
    let end = html[start..].find(b).map_or(html.len(), |e| start + e);
    format!("{}{}", &html[..start], &html[end..])
}

/// `a` で始まる行を改行ごと切り取る（a が無ければそのまま）。
fn cut_line(html: &str, a: &str) -> String {
    let Some(start) = html.find(a) else {
        return html.to_string();
    };
    let end = html[start..]
        .find('\n')
        .map_or(html.len(), |e| start + e + 1);
    format!("{}{}", &html[..start], &html[end..])
}

/// 図の本体の数（章の帯の kicker の絵記号 `<svg class="ico"` は数えない）。
fn svg_bodies(html: &str) -> usize {
    html.matches("<svg").count() - html.matches("<svg class=\"ico\"").count()
}

#[test]
fn face_srs_embeds_the_figure_in_a_figure_panel_with_label_and_refs() {
    let html = fs::read_to_string(fixture().join("expected-srs.html")).unwrap();
    assert!(
        html.contains(
            "<li><a href=\"#s9\"><span class=\"n\">09</span><span class=\"k\">図</span><span class=\"t\">1 枚</span></a></li>"
        ),
        "toc に 09「図」が無い: {html}"
    );
    // 図の枠は 図 1〜3 の分（写しは verdicts を持つ）+ 図の節の 1 枚
    assert_eq!(
        html.matches("data-component=\"figure-panel\"").count(),
        3 + 1,
        "figure-panel が 図 1〜3 + 1 でない: {html}"
    );
    assert_eq!(
        html.matches("<figure data-component=\"figure-panel\" data-role=\"diagram\" id=\"fig-1\">")
            .count(),
        1,
        "図の節の figure-panel が 1 つでない: {html}"
    );
    assert_eq!(svg_bodies(&html), 1, "図の本体が 1 つでない");
    // 章 09 の番号は自前の図（図 1〜3・写しは verdicts を持つ）の続き = 図 4（便 35）
    assert!(
        html.contains("<div class=\"fig-title\"><span class=\"fn\">図 4</span>見本の図 <span class=\"fig-tools\">"),
        "fig-title に caption の逐語が無い: {html}"
    );
    // 根拠の要件書の id は同じ面の anchor（href が # で始まる）
    assert!(
        html.contains(
            "<figcaption><span class=\"ver\">図 4 · 構成図（architecture） · fig-1 · 根拠: <a class=\"xref\" href=\"#fr1\">FR1</a></span></figcaption>"
        ),
        "figcaption に型の名札・id・根拠の同じ面へのリンクが無い: {html}"
    );
    // 章 09 の帯は用語集（08）の後・承認欄の前
    let s8 = html.find("<section id=\"s8\"").expect("章 08 が無い");
    let s9 = html.find("<section id=\"s9\"").expect("章 09 が無い");
    let ap = html.find("<section id=\"approval\"").expect("承認欄が無い");
    assert!(s8 < s9 && s9 < ap, "章 09 の置き場が違う");
    assert!(
        html.contains("<section id=\"s9\" data-component=\"chapter-deck-band\" class=\"band-3\">"),
        "章 09 の帯が band-3 でない: {html}"
    );
    assert!(
        html.contains("<h2>図 1 枚</h2>"),
        "章 09 の h2 が「図 1 枚」でない: {html}"
    );
    assert!(
        html.contains("<dt>figures</dt><dd>1</dd>"),
        "機械のための面に figures が無い: {html}"
    );
}

#[test]
fn face_srs_without_figures_has_no_figure_chapter() {
    let html = srs_figureless_html("srs-no-figures");
    assert_eq!(
        html.matches("data-component=\"figure-panel\"").count(),
        3,
        "図が無いのに 図 1〜3 の外に figure-panel が在る"
    );
    assert!(
        !html.contains("<section id=\"s9\""),
        "図が無いのに図の章が在る"
    );
    assert_eq!(svg_bodies(&html), 0, "図が無いのに図の本体が在る");
    assert!(
        !html.contains("<li><a href=\"#s9\">"),
        "図が無いのに toc に 09 が在る"
    );
    assert!(
        !html.contains("<dt>figures</dt>"),
        "図が無いのに機械のための面に figures が在る"
    );
    assert!(html.contains("全 9 章"), "図なしの面が全 9 章でない");
}

#[test]
fn face_srs_figure_chapter_is_the_only_difference_from_the_figureless_face() {
    let frozen = fs::read_to_string(fixture().join("expected-srs.html")).unwrap();
    let without = srs_figureless_html("srs-outside");
    // 図の章（s9 の帯から承認欄の帯の直前まで）・toc の 09・foot の figures を図ありの面から抜く
    let a = cut(&frozen, "<section id=\"s9\"", "<section id=\"approval\"");
    let a = cut_line(&a, "<li><a href=\"#s9\">");
    let a = a.replace("<dt>figures</dt><dd>1</dd>", "");
    // 表紙の図の列の章 09 の図（便 74）
    let a = a.replacen(" · <a href=\"#fig-1\">図 4</a>", "", 1);
    // 章の数から数える字（全 N 章・k/N）だけは正本から数えた数（γ）なので揃える
    let a = a
        .replace("全 10 章", "全 9 章")
        .replace("/10</span>", "/9</span>");
    assert_same_bytes(
        without.as_bytes(),
        a.as_bytes(),
        "図の章を抜いた凍結（図なしの面の期待）",
    );
}

/// fig-title の番号の札（`<span class="fn">図 N</span>`）の数。
fn fig_labels(html: &str, n: usize) -> usize {
    html.matches(&format!("<span class=\"fn\">図 {n}</span>"))
        .count()
}

#[test]
fn face_srs_figure_chapter_numbers_continue_from_the_own_figures() {
    // 写しから verdicts の節を消す → 自前の図は 2 枚 → 章 09 は 図 3 から
    let (run, html) = srs_mutated("srs-fig-numbering", |t| {
        let start = t.find("\nverdicts:\n").expect("verdicts の節が無い");
        let end = t
            .find("\nrequirements:\n")
            .expect("requirements の節が無い");
        format!("{}{}", &t[..start], &t[end..])
    });
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert_eq!(
        html.matches("data-component=\"figure-panel\"").count(),
        2 + 1,
        "figure-panel が 図 1〜2 + 1 でない: {html}"
    );
    assert!(
        html.contains("<div class=\"fig-title\"><span class=\"fn\">図 3</span>見本の図 <span class=\"fig-tools\">"),
        "verdicts 無しの章 09 の fig-title が「図 3」でない: {html}"
    );
    assert!(
        html.contains(
            "<figcaption><span class=\"ver\">図 3 · 構成図（architecture） · fig-1 · 根拠: "
        ),
        "verdicts 無しの章 09 の figcaption が「図 3 ·」でない: {html}"
    );
    // 帯の h2・toc・foot の数は章 09 の図の数のまま
    assert!(html.contains("<h2>図 1 枚</h2>"), "章 09 の h2 が変わった");
    assert!(
        html.contains("<dt>figures</dt><dd>1</dd>"),
        "foot の figures が変わった"
    );

    // 実の正本（図の節 2 枚・verdicts あり）: 章 09 は 図 4・図 5 が 1 つずつ・「図 1」の札は自前の図の 1 つだけ
    let (td, _, real) = real_srs("srs-fig-numbering-real");
    let _ = fs::remove_dir_all(&td);
    let s = load_yaml("srs.yaml");
    assert!(s["verdicts"].as_vec().is_some(), "正本に verdicts が無い");
    assert_eq!(
        s["figures"].as_vec().map_or(0, Vec::len),
        2,
        "正本の図の節が 2 枚でない"
    );
    assert_eq!(
        fig_labels(&real, 1),
        1,
        "「図 1」の札が自前の図の 1 つだけでない"
    );
    assert_eq!(fig_labels(&real, 4), 1, "「図 4」の札が 1 つでない");
    assert_eq!(fig_labels(&real, 5), 1, "「図 5」の札が 1 つでない");
    assert_eq!(fig_labels(&real, 6), 0, "「図 6」の札が在る");
    assert!(
        real.contains("<span class=\"ver\">図 4 · ")
            && real.contains("<span class=\"ver\">図 5 · "),
        "実の正本の figcaption の番号が 図 4・図 5 でない"
    );
}

#[test]
fn face_srs_unknown_when_a_figure_fails_the_tool_check_and_keeps_the_previous_face() {
    let (td, work) = fixture_copy("srs-fig-fails");
    // layout を消すと道具の検査（showcase）に落ちる
    edit(&work.join("srs.yaml"), |t| {
        t.replacen(
            "      layout: {mode: grid, cols: 2, gapX: 70, gapY: 110, cellW: 160, cellH: 70}\n",
            "",
            1,
        )
    });
    let out = td.join("srs.html");
    let before = "<!DOCTYPE html>\n前の面\n".as_bytes().to_vec();
    fs::write(&out, &before).unwrap();
    let run = folio_face("srs", &work, &out, "--write");
    let after = fs::read(&out).unwrap();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --write"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("まだ分からない"), "{}", stderr(&run));
    assert!(
        stderr(&run).contains("図の道具の検査を通らない"),
        "{}",
        stderr(&run)
    );
    assert_eq!(after, before, "図が通らないのに前の面を上書きした");
}

#[test]
fn face_srs_unknown_when_a_figure_type_is_not_a_tool_type() {
    let (run, html) = srs_mutated("srs-fig-type", |t| {
        t.replacen("type: archify-architecture", "type: pipeline-rail", 1)
    });
    assert_eq!(code(&run, "folio face"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("まだ分からない"), "{}", stderr(&run));
    assert!(
        stderr(&run).contains("図の道具の型でない"),
        "{}",
        stderr(&run)
    );
    assert!(html.is_empty(), "導出できないのに面を書いた");
}

// ── 受入基準の章の凡例（便 64・docs/design/delivery-64.md §1 (c)）──

/// 受入基準の札「まだ分からない」の凡例が言う字。
const AC_LEGEND: &str = "合否を folio はまだ数えていません";

/// 章の区間（帯 `a` から帯 `b` の直前まで）に出る凡例の行を出た順に。
fn legend_lines<'a>(html: &'a str, a: &str, b: &str) -> Vec<&'a str> {
    between(html, a, b)
        .lines()
        .filter(|l| l.starts_with("<div class=\"legend-line\">"))
        .collect()
}

#[test]
fn ac_legend_appears_once_in_the_acceptance_chapter() {
    let (td, _, html) = real_srs("srs-ac-legend");
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        html.matches(AC_LEGEND).count(),
        1,
        "凡例の字が面に 1 回でない"
    );
    let chapter = between(&html, "<section id=\"s5\"", "<section id=\"s6\"");
    let lines = legend_lines(&html, "<section id=\"s5\"", "<section id=\"s6\"");
    assert_eq!(
        lines.len(),
        1,
        "受入基準の章の凡例の行が 1 行でない: {lines:?}"
    );
    assert!(
        lines[0].contains(&format!("まだ分からない = この基準の{AC_LEGEND}")),
        "凡例が札の字と意味を並べていない: {}",
        lines[0]
    );
    // 帯の直後・基準の一覧の前（最初の札より先）
    let at = chapter.find(lines[0]).unwrap();
    let first_chip = chapter
        .find("data-component=\"ac-state-chip\"")
        .expect("受入基準の章に札が無い");
    assert!(at < first_chip, "凡例が基準の一覧より後に在る");
}

#[test]
fn ac_legend_uses_the_same_parts_as_chapter_three() {
    let (td, _, html) = real_srs("srs-ac-legend-parts");
    let _ = fs::remove_dir_all(&td);
    let fr = legend_lines(&html, "<section id=\"s3\"", "<section id=\"s4\"");
    let ac = legend_lines(&html, "<section id=\"s5\"", "<section id=\"s6\"");
    assert_eq!(fr.len(), 1, "§3 の凡例の行が 1 行でない: {fr:?}");
    assert_eq!(ac.len(), 1, "受入基準の章の凡例の行が 1 行でない: {ac:?}");
    assert_eq!(
        components(ac[0]),
        components(fr[0]),
        "凡例の部品の名札が §3 の凡例と違う"
    );
    let head = "<div class=\"legend-line\"><span>凡例:</span>";
    assert!(
        fr[0].starts_with(head),
        "§3 の凡例の形が変わった: {}",
        fr[0]
    );
    assert!(
        ac[0].starts_with(head) && ac[0].ends_with("</div>"),
        "受入基準の章の凡例が §3 と同じ class で包まれていない: {}",
        ac[0]
    );
}

#[test]
fn face_srs_unknown_when_the_figure_tool_is_absent() {
    let (td, work) = fixture_copy("srs-no-tool");
    fs::remove_file(td.join("vendor/archify/bin/archify.mjs")).unwrap();
    let out = td.join("srs.html");
    let run = folio_face("srs", &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --write"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("まだ分からない"), "{}", stderr(&run));
    assert!(stderr(&run).contains("図の道具が無い"), "{}", stderr(&run));
    assert!(!exists, "導出できないのに面を書いた");
}

// ── 憲法の面の数えの字と札の凡例（便 63・docs/design/delivery-63.md §1 (a)(b)(d)）──

/// 写しの憲法へ足す「いつも守る」の条 2〜`n`（fixture の A-1 と同じ最小の形・id と字面だけ変える）。
fn always_articles(n: usize) -> String {
    (2..=n)
        .map(|i| {
            format!(
                "  - id: P-{i}\n    title: 足した条 {i}\n    tier: always\n    binds: practice\n    \
                 statements:\n      - {{id: P-{i}.1, pattern: ubiquitous, strength: must, text: 足した条 {i} の規範文。}}\n    \
                 plain: 足した条 {i} のやさしい言い方。\n    rationale:\n      - {{kind: folio2-ruling, ref: 裁定 F-1}}\n    \
                 mechanism: {{kind: human-review, live: now}}\n\n"
            )
        })
        .collect()
}

#[test]
fn count_word_uses_tsu_only_up_to_nine() {
    // 9 以下の段（fixture のまま・3 段とも 1 条）は「つ」が付く
    let td = temp_dir("count-word-nine");
    let out = td.join("constitution.html");
    let run = folio_face("constitution", &fixture(), &out, "--write");
    let nine = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    // 目次と章の帯の 2 か所 × 3 段
    assert_eq!(
        nine.matches("1 つの原則").count(),
        6,
        "9 以下の段の見出しが 3 段 × 2 か所で「つ」付きでない"
    );
    assert!(!nine.contains("1 の原則"), "9 以下の段から「つ」が落ちた");

    // いつも守るを 10 条にした写し = その段だけ「つ」が落ちる
    let (td, work) = fixture_copy("count-word-ten");
    edit(&work.join("constitution.yaml"), |t| {
        t.replacen("{always: 1,", "{always: 10,", 1).replacen(
            "\nrules_pointer:",
            &format!("\n{}rules_pointer:", always_articles(10)),
            1,
        )
    });
    let out = td.join("constitution.html");
    let run = folio_face("constitution", &work, &out, "--write");
    let ten = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert_eq!(
        ten.matches("10 の原則").count(),
        2,
        "「10 の原則」が目次と章の帯の 2 か所に出ていない"
    );
    assert!(!ten.contains("10 つの原則"), "10 以上なのに「つ」が付いた");
    // 1 条のままの 2 段（確認してから・絶対にやらない）は「つ」が付いたまま
    assert_eq!(
        ten.matches("1 つの原則").count(),
        4,
        "9 以下の段の見出しが 2 段 × 2 か所で「つ」付きでない"
    );
}

/// 札の凡例の 1 つ目の対（要件書の面 §3 の凡例と同じ字）。
const TIER_LEGEND_HEAD: &str = "MUST = 必ず守る";

/// 章 s3（確認してから）と s4（絶対にやらない）の区間。
const LATER_TIER_CHAPTERS: [(&str, &str); 2] = [
    ("<section id=\"s3\"", "<section id=\"s4\""),
    ("<section id=\"s4\"", "<section id=\"s5\""),
];

#[test]
fn tier_legend_appears_once_in_the_constitution_face() {
    let (td, _, html) = real_face("tier-legend");
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        html.matches(TIER_LEGEND_HEAD).count(),
        1,
        "凡例の字が面に 1 回でない"
    );
    let first = legend_lines(&html, "<section id=\"s2\"", "<section id=\"s3\"");
    assert_eq!(
        first.len(),
        1,
        "いつも守るの章の凡例の行が 1 行でない: {first:?}"
    );
    assert!(
        first[0].contains(TIER_LEGEND_HEAD),
        "凡例が札の字と意味を並べていない: {}",
        first[0]
    );
    for (a, b) in LATER_TIER_CHAPTERS {
        let later = legend_lines(&html, a, b);
        assert!(later.is_empty(), "後の段の章にも凡例が出ている: {later:?}");
    }
    // 帯の直後・条の一覧の前
    let chapter = between(&html, "<section id=\"s2\"", "<section id=\"s3\"");
    let at = chapter.find(first[0]).unwrap();
    let first_row = chapter
        .find("data-component=\"item-row\"")
        .expect("いつも守るの章に条が無い");
    assert!(at < first_row, "凡例が条の一覧より後に在る");
    // 要件書の面 §3 の凡例と同じ class・同じ部品の名札
    let (td, _, srs) = real_srs("tier-legend-srs");
    let _ = fs::remove_dir_all(&td);
    let fr = legend_lines(&srs, "<section id=\"s3\"", "<section id=\"s4\"");
    assert_eq!(fr.len(), 1, "要件書の面 §3 の凡例の行が 1 行でない: {fr:?}");
    let head = "<div class=\"legend-line\"><span>凡例:</span>";
    assert!(
        fr[0].starts_with(head),
        "要件書の面 §3 の凡例の形が変わった: {}",
        fr[0]
    );
    assert!(
        first[0].starts_with(head) && first[0].ends_with("</div>"),
        "憲法の面の凡例が §3 と同じ class で包まれていない: {}",
        first[0]
    );
    assert_eq!(
        components(first[0]),
        components(fr[0]),
        "凡例の部品の名札が §3 の凡例と違う"
    );
}

#[test]
fn tier_legend_words_come_from_the_shared_table() {
    let (td, _, html) = real_face("tier-legend-words");
    let _ = fs::remove_dir_all(&td);
    let (td2, _, srs) = real_srs("tier-legend-words-srs");
    let _ = fs::remove_dir_all(&td2);
    let c = legend_lines(&html, "<section id=\"s2\"", "<section id=\"s3\"");
    let fr = legend_lines(&srs, "<section id=\"s3\"", "<section id=\"s4\"");
    assert_eq!(c.len(), 1, "憲法の面の凡例の行が 1 行でない: {c:?}");
    assert_eq!(fr.len(), 1, "要件書の面 §3 の凡例の行が 1 行でない: {fr:?}");
    // 3 つの語は歯の側で持つ（生成側の表を呼ばない・2 面が同じ字を出す）
    for (kw, meaning) in [
        ("MUST", "必ず守る"),
        ("MUST NOT", "決してしない"),
        ("SHOULD", "強い推奨（外すなら理由が要る）"),
    ] {
        let pair = format!("{kw} = {meaning}");
        assert!(
            c[0].contains(&pair),
            "憲法の面の凡例に「{pair}」が無い: {}",
            c[0]
        );
        assert!(
            fr[0].contains(&pair),
            "要件書の面 §3 の凡例に「{pair}」が無い: {}",
            fr[0]
        );
    }
}

// ── 章の見出しの助数詞と §5 の名札（便 70・docs/design/delivery-70.md §1 (e)）──

/// 便 63 の数えの規則で組んだ字（生成側の count_word を呼ばず歯の側で組む・9 までは「つ」・10 以上は「N の」）。
fn counted(n: usize, noun: &str) -> String {
    if n <= 9 {
        format!("{n} つの{noun}")
    } else {
        format!("{n} の{noun}")
    }
}

/// 規則を外した字（9 以下で「つ」が落ちた形・10 以上で「つ」が付いた形）。
fn miscounted(n: usize, noun: &str) -> String {
    if n <= 9 {
        format!("{n} の{noun}")
    } else {
        format!("{n} つの{noun}")
    }
}

/// 数えの字が目次と章の帯の 2 か所に出ていて、規則を外した形が無い（同じ数の節が重なっても数が合う）。
fn assert_counted(html: &str, counts: &[(usize, &str)], what: &str) {
    let wants: Vec<String> = counts.iter().map(|(n, noun)| counted(*n, noun)).collect();
    for ((n, noun), want) in counts.iter().zip(&wants) {
        assert!(html.contains(want), "{what}: 「{want}」が面に無い");
        let bad = miscounted(*n, noun);
        assert!(
            !html.contains(&bad),
            "{what}: 助数詞の規則を外れた「{bad}」が在る"
        );
    }
    let mut distinct: Vec<&String> = wants.iter().collect();
    distinct.sort();
    distinct.dedup();
    let hits: usize = distinct
        .iter()
        .map(|w| html.matches(w.as_str()).count())
        .sum();
    assert_eq!(
        hits,
        2 * wants.len(),
        "{what}: 数えの字が目次と章の帯の 2 か所 × {} 章で出ていない",
        wants.len()
    );
}

/// 要件書の正本の 3 節の数（歯が自分で数える・面の側の数え方に依らない）。
fn srs_counts(dir: &Path) -> Vec<(usize, &'static str)> {
    let s = load_yaml_at(dir, "srs.yaml");
    [
        ("requirements", "機能要件"),
        ("nonfunctional", "非機能要件"),
        ("acceptance", "受入基準"),
    ]
    .iter()
    .map(|(sec, noun)| (seq(&s[*sec], sec).len(), *noun))
    .collect()
}

#[test]
fn noun_count_srs_uses_the_counter_word() {
    // 凍結の正本（3 節とも 9 以下 = 「つ」が付く）
    let (td, work) = fixture_copy("noun-count-srs");
    let out = td.join("srs.html");
    let run = folio_face("srs", &work, &out, "--write");
    let frozen = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        code(&run, "folio face --face srs --write"),
        0,
        "{}",
        stderr(&run)
    );
    assert_counted(&frozen, &srs_counts(&fixture()), "凍結の正本の要件書の面");

    // 実の正本（機能要件と受入基準は 10 以上・非機能要件は 3 = 「3 の非機能要件」にならない）
    let (td, _, real) = real_srs("noun-count-srs-real");
    let _ = fs::remove_dir_all(&td);
    let counts = srs_counts(&design_intent());
    assert_counted(&real, &counts, "実の正本の要件書の面");
    assert!(
        counts.iter().any(|(n, _)| *n >= 10) && counts.iter().any(|(n, _)| *n <= 9),
        "実の正本が 10 以上と 9 以下の両方の節を持たない（歯の前提）: {counts:?}"
    );
}

#[test]
fn noun_count_constitution_is_unchanged_by_the_move() {
    for (dir, case, what) in [
        (fixture(), "noun-count-const", "凍結の正本の憲法の面"),
        (
            design_intent(),
            "noun-count-const-real",
            "実の正本の憲法の面",
        ),
    ] {
        let td = temp_dir(case);
        let out = td.join("constitution.html");
        let run = folio_face("constitution", &dir, &out, "--write");
        let html = fs::read_to_string(&out).unwrap_or_default();
        let _ = fs::remove_dir_all(&td);
        assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
        // 段ごとの数は正本の meta.counts から歯が自分で読む（再生成する写しに依らない oracle）
        let c = load_yaml_at(&dir, "constitution.yaml");
        let counts: Vec<(usize, &str)> = ["always", "ask-first", "never"]
            .iter()
            .map(|key| {
                let n = c["meta"]["counts"][*key]
                    .as_i64()
                    .unwrap_or_else(|| panic!("meta.counts.{key} が数でない"));
                (usize::try_from(n).expect("meta.counts が負"), "原則")
            })
            .collect();
        assert_counted(&html, &counts, what);
    }
}

/// 種別の鍵 → 表の種別の欄と同じ名札（歯の側で持つ・生成側の表を呼ばない）。
const RULE_KIND_LABELS: [(&str, &str); 4] = [
    ("deny", "測って落とす"),
    ("build-check", "生成時の検査"),
    ("detect", "記録のみ"),
    ("human-review", "人が守る作法"),
];

#[test]
fn noun_count_rule_kind_legend_uses_the_labels() {
    let (td, _, html) = real_face("noun-count-legend");
    let _ = fs::remove_dir_all(&td);
    let r = load_yaml("rules.yaml");
    let meaning = &r["schema"]["kind_meaning"];
    for (key, label) in RULE_KIND_LABELS {
        let m = meaning[key]
            .as_str()
            .unwrap_or_else(|| panic!("rules.yaml の schema.kind_meaning.{key} が無い"));
        let want = format!("{label}（{key}）: {}", esc(m));
        assert!(html.contains(&want), "§5 の凡例に「{want}」が無い");
        assert!(
            !html.contains(&format!("{key}: ")),
            "§5 の凡例に英語の鍵だけの形「{key}: 」が残っている"
        );
    }
}

#[test]
fn noun_count_projection_label_is_plain() {
    let (td, _, html) = real_face("noun-count-projection");
    let _ = fs::remove_dir_all(&td);
    let five = between(&html, "<section id=\"s5\"", "<section id=\"s6\"");
    assert!(
        five.contains("写す範囲"),
        "§5 に「写す範囲」の名札が無い: {five}"
    );
    assert!(!html.contains("射影"), "面に数学の語「射影」が残っている");
}

// ── 要件書の面の表紙の図の列（便 74・delivery-74.md §1 (d)(f)）──

/// 表紙の「図」の欄の a の列（行き先・番号の字）。
fn cover_figure_links(html: &str) -> Vec<(String, String)> {
    let cell = between(
        html,
        "<span class=\"m\"><span class=\"k\">図</span><span class=\"v\">",
        "</span></span>",
    );
    cell.split(" · ")
        .map(|a| {
            let href = between(a, "<a href=\"#", "\">").to_string();
            let n = between(a, "\">", "</a>").to_string();
            (href, n)
        })
        .collect()
}

/// 面の中の figure-panel の id の列（出る順）。
fn figure_panel_ids(html: &str) -> Vec<String> {
    let open = "<figure data-component=\"figure-panel\" data-role=\"diagram\" id=\"";
    html.split(open)
        .skip(1)
        .map(|rest| rest[..rest.find('"').unwrap()].to_string())
        .collect()
}

fn f74_srs_html(case: &str, mutate: Option<fn(&str) -> String>) -> String {
    let (td, work) = fixture_copy(case);
    if let Some(m) = mutate {
        edit(&work.join("srs.yaml"), m);
    }
    let out = td.join("srs.html");
    let run = folio_face("srs", &work, &out, "--write");
    let html = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    html
}

#[test]
fn f74_srs_cover_lists_every_figure_panel() {
    let html = f74_srs_html("f74-cover", None);
    let links = cover_figure_links(&html);
    let panels = figure_panel_ids(&html);
    let hrefs: Vec<&str> = links.iter().map(|(h, _)| h.as_str()).collect();
    assert_eq!(
        hrefs,
        panels.iter().map(String::as_str).collect::<Vec<_>>(),
        "表紙の図の列と面の figure-panel の列が違う"
    );
    for (i, (_, n)) in links.iter().enumerate() {
        assert_eq!(
            n,
            &format!("図 {}", i + 1),
            "表紙の図の番号が 1 からの連番でない"
        );
    }
    assert_eq!(
        hrefs,
        ["fig-context", "fig-rail", "fig-verdicts", "fig-1"],
        "凍結 fixture の表紙の図の列が 4 つでない"
    );
}

#[test]
fn f74_srs_cover_figure_href_comes_from_the_source() {
    let html = f74_srs_html(
        "f74-href",
        Some(|t| t.replacen("  - id: fig-1\n", "  - id: fig-x\n", 1)),
    );
    let links = cover_figure_links(&html);
    assert_eq!(links.len(), 4, "{links:?}");
    assert_eq!(links[3], ("fig-x".to_string(), "図 4".to_string()));
    assert_eq!(figure_panel_ids(&html)[3], "fig-x");
}

// ── 便 85（docs/design/delivery-85.md §1 (d) 3）: 章 05 の種別の凡例に deny の新しい意味が届く ──

#[test]
fn f85_constitution_legend_shows_the_new_meaning() {
    let (td, _, html) = real_face("f85-legend");
    let _ = fs::remove_dir_all(&td);
    let five = between(&html, "<section id=\"s5\"", "<section id=\"s6\"");
    assert!(five.contains("値域の外なら落とす"), "章 05 の凡例に新しい deny の意味が無い");
    assert_eq!(five.matches("超過なら落とす").count(), 0, "章 05 に古い字が残っている");
}
