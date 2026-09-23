//! `folio face` の憲法の面と命令そのものの共通の歯（便 14・docs/design/delivery-14.md §1 (e)）。binary 経由。
//! - 凍結 fixture（tests/fixtures/face/）との byte 一致（P-10.1・回帰の anchor で唯一の判定にしない）・escape
//! - 実の正本で生成した面が `folio parts --check` に合格する・逐語と件数の census（yaml-rust2 で正本を直に読む）
//! - check の 3 値・旗が 1 つだけ・表に無い面の名・導出できない入力・組み立てた版とのずれ（便 50）
//! - 章の札の凡例（便 63）・数えの字（便 63・70）・§5 の種別の凡例と名札（便 70・85）
//!
//! 入口の面の歯は `face_index.rs`・`face_index_shelf.rs`・`face_index_sheet.rs` へ（便 26・便 105）、要件書の面の
//! 本体の歯は `face_srs_body.rs` へ、要件書の面の図の章の歯は `face_srs_figure.rs` へ移した（便 105・
//! docs/design/delivery-105.md §1 (b)）。歯の file どうしは互いに use できないので、helper は各 file に写しを持つ
//! （写しは字を変えない・この file の歯が呼ぶものだけ）。
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

/// 章の区間（帯 `a` から帯 `b` の直前まで）に出る凡例の行を出た順に。
fn legend_lines<'a>(html: &'a str, a: &str, b: &str) -> Vec<&'a str> {
    between(html, a, b)
        .lines()
        .filter(|l| l.starts_with("<div class=\"legend-line\">"))
        .collect()
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

// ── 便 85（docs/design/delivery-85.md §1 (d) 3）: 章 05 の種別の凡例に deny の新しい意味が届く ──

#[test]
fn f85_constitution_legend_shows_the_new_meaning() {
    let (td, _, html) = real_face("f85-legend");
    let _ = fs::remove_dir_all(&td);
    let five = between(&html, "<section id=\"s5\"", "<section id=\"s6\"");
    assert!(five.contains("値域の外なら落とす"), "章 05 の凡例に新しい deny の意味が無い");
    assert_eq!(five.matches("超過なら落とす").count(), 0, "章 05 に古い字が残っている");
}
