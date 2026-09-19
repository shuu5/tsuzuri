//! `folio face` の歯（便 14・docs/design/delivery-14.md §1 (e)）。binary 経由。
//! - 凍結 fixture（tests/fixtures/face/）との byte 一致（P-10.1・回帰の anchor で唯一の判定にしない）
//! - 実の正本で生成した面が `folio parts --check` に合格する（AC2 の機構・生成側から独立した物差し 1）
//! - 逐語と件数の census（yaml-rust2 で正本を直に読む・生成側から独立した物差し 2）
//! - check の 3 値・表に無い面の名・導出できない入力 8 つ・escape
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
    assert_eq!(
        count("class=\"grow\""),
        seq(&v["terms"], "terms").len(),
        "grow の数"
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

    // 部品の名札は 14 種の中だけ・lane-chip を含まない
    const ALLOWED: [&str; 14] = [
        "freshness-stamp",
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
        assert!(ALLOWED.contains(p), "14 種に無い部品「{p}」");
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

    // 部品の名札は 17 種の中だけ・lane-chip を含まない
    const ALLOWED: [&str; 17] = [
        "freshness-stamp",
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
        assert!(ALLOWED.contains(p), "17 種に無い部品「{p}」");
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
    assert_eq!(
        html.matches("data-component=\"glossary-term-table\"")
            .count(),
        1,
        "glossary-term-table が 1 つでない"
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
