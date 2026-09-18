//! `folio face` の歯（便 14・docs/design/delivery-14.md §1 (e)）。binary 経由。
//! - 凍結 fixture（tests/fixtures/face/）との byte 一致（P-10.1・回帰の anchor で唯一の判定にしない）
//! - 実の正本で生成した面が `folio parts --check` に合格する（AC2 の機構・生成側から独立した物差し 1）
//! - 逐語と件数の census（yaml-rust2 で正本を直に読む・生成側から独立した物差し 2）
//! - check の 3 値・面の名 2 つ・導出できない入力 8 つ・escape
//!
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

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-face-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    fs::create_dir_all(&td).unwrap();
    td
}

/// fixture の正本 4 file を一時 dir の下の src/ へ写す（expected.html は写さない）。戻り値 = (一時 dir, 写し)。
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
fn face_index_face_is_writable() {
    let td = temp_dir("index");
    let out = td.join("index.html");
    let run = folio_face("index", &fixture(), &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --face index"), 0, "{}", stderr(&run));
    assert!(stdout(&run).contains("書いた"), "{}", stdout(&run));
    assert!(
        !stderr(&run).contains("生成器はまだ無い"),
        "{}",
        stderr(&run)
    );
    assert!(exists, "入口の面を出力先に書いていない");
}

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
    let td = temp_dir("srs-anchor");
    let out = td.join("srs.html");
    let run = folio_face("srs", &fixture(), &out, "--write");
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
    let td = temp_dir("srs-escape");
    let out = td.join("srs.html");
    let run = folio_face("srs", &fixture(), &out, "--write");
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
    let (td, out, _) = real_srs("srs-parts");
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
    let glossary = between(&html, "data-component=\"glossary-links\">", "\n</div>");
    assert_eq!(
        glossary.matches("<a href=").count(),
        seq(&v["terms"], "terms").len() * 2,
        "用語の一覧の a の数"
    );
    assert_eq!(parts_of("ac-state-chip"), acs.len(), "ac-state-chip の数");

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
        "glossary-links",
        "approval-block",
    ];
    assert!(!parts.is_empty());
    for p in &parts {
        assert!(ALLOWED.contains(p), "17 種に無い部品「{p}」");
    }
    assert!(!html.contains("lane-chip"));

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

    // 無い側: state-strip 0・「図3」の参照は字面だけ・それ以外は同じ
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
        .replace("<a class=\"fig\" href=\"#fig-verdicts\">図 3</a>", "図 3");
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

// ── 入口の面（便 16・docs/design/delivery-16.md §1 (d)）──

/// fixture の正本 4 file と index.yaml・intake.yaml・adr/ADR-1.yaml を一時 dir の下の src/ へ写す
/// （支度表 intake-sheet.yaml と期待の面は写さない）。
fn index_fixture_copy(case: &str) -> (PathBuf, PathBuf) {
    let (td, work) = fixture_copy(case);
    for name in ["index.yaml", "intake.yaml"] {
        fs::copy(fixture().join(name), work.join(name)).unwrap();
    }
    fs::create_dir_all(work.join("adr")).unwrap();
    fs::copy(
        fixture().join("adr/ADR-1.yaml"),
        work.join("adr/ADR-1.yaml"),
    )
    .unwrap();
    (td, work)
}

/// 入口の写しに支度表（fixture の intake-sheet.yaml）も置く（便 20）。
fn index_sheet_copy(case: &str) -> (PathBuf, PathBuf) {
    let (td, work) = index_fixture_copy(case);
    fs::copy(
        fixture().join("intake-sheet.yaml"),
        work.join("intake-sheet.yaml"),
    )
    .unwrap();
    (td, work)
}

/// 写しから入口の面を一時 dir へ書く。戻り値 = (出力先, 面の本文)。
fn index_from(case: &str, work: &Path, td: &Path) -> (PathBuf, String) {
    let out = td.join("index.html");
    let run = folio_face("index", work, &out, "--write");
    assert_eq!(
        code(&run, "folio face --face index --write"),
        0,
        "{case}: {}",
        stderr(&run)
    );
    let html = fs::read_to_string(&out).unwrap();
    assert_eq!(
        stdout(&run),
        format!("folio face: 書いた（{} byte）\n", html.len())
    );
    (out, html)
}

/// 写しから入口の面を書き、凍結した期待の面と byte で比べる。
fn index_frozen(case: &str, work: &Path, td: &Path, expected: &str) {
    let (out, _) = index_from(case, work, td);
    let written = fs::read(&out).unwrap();
    let frozen = fs::read(fixture().join(expected)).unwrap();
    assert_same_bytes(&written, &frozen, expected);
}

#[test]
fn face_index_write_matches_the_frozen_fixture() {
    let (td, work) = index_fixture_copy("index-anchor");
    index_frozen("支度表なし", &work, &td, "expected-index.html");
    let _ = fs::remove_dir_all(&td);
}

#[test]
fn face_index_write_with_a_sheet_matches_the_frozen_fixture() {
    let (td, work) = index_sheet_copy("index-anchor-sheet");
    index_frozen("支度表あり", &work, &td, "expected-index-sheet.html");
    let _ = fs::remove_dir_all(&td);
}

/// 実の正本から入口の面を一時 file へ書く。戻り値 = (一時 dir, 出力先, 面の本文)。
fn real_index(case: &str) -> (PathBuf, PathBuf, String) {
    let td = temp_dir(case);
    let out = td.join("index.html");
    let run = folio_face("index", &design_intent(), &out, "--write");
    assert_eq!(
        code(&run, "folio face --face index --write"),
        0,
        "{}",
        stderr(&run)
    );
    assert!(stdout(&run).contains("書いた"), "{}", stdout(&run));
    let html = fs::read_to_string(&out).unwrap();
    (td, out, html)
}

fn parts_check(pages: &[String]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_folio"));
    cmd.arg("parts")
        .arg("--check")
        .arg("--dir")
        .arg(design_intent());
    for page in pages {
        cmd.arg("--page").arg(page);
    }
    cmd.output().unwrap()
}

#[test]
fn face_index_on_the_real_sources_passes_parts_check() {
    let (td, out, _) = real_index("index-parts");
    let check = parts_check(&[format!("index={}", out.display())]);
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

#[test]
fn face_all_three_faces_on_the_real_sources_pass_parts_check_together() {
    let (td_i, index, _) = real_index("three-index");
    let (td_c, constitution, _) = real_face("three-constitution");
    let (td_s, srs, _) = real_srs("three-srs");
    let check = parts_check(&[
        format!("index={}", index.display()),
        format!("constitution={}", constitution.display()),
        format!("srs={}", srs.display()),
    ]);
    for td in [td_i, td_c, td_s] {
        let _ = fs::remove_dir_all(&td);
    }
    assert_eq!(
        code(&check, "folio parts --check（3 面）"),
        0,
        "{}{}",
        stdout(&check),
        stderr(&check)
    );
    assert!(stdout(&check).contains("違反 0"), "{}", stdout(&check));
}

#[test]
fn face_index_census_on_the_real_sources_counts_and_verbatims() {
    let (td, _, html) = real_index("index-census");
    let _ = fs::remove_dir_all(&td);
    let i = load_yaml("index.yaml");
    let c = load_yaml("constitution.yaml");
    let s = load_yaml("srs.yaml");
    let v = load_yaml("vocabulary.yaml");
    let r = load_yaml("rules.yaml");

    // 逐語
    let mut wants = vec![text(&i["audience"], "text").to_string()];
    let shelf = &i["shelf"];
    for d in seq(&shelf["documents"], "documents") {
        wants.push(text(d, "use").to_string());
    }
    for rel in seq(&shelf["relations"], "relations") {
        wants.push(text(rel, "label").to_string());
    }
    let rows = seq(&i["lanes"]["rows"], "rows");
    let mut stops = 0;
    for row in rows {
        wants.push(text(row, "who").to_string());
        wants.push(text(row, "why").to_string());
        for st in seq(&row["stops"], "stops") {
            wants.push(text(st, "label").to_string());
            let file = match text(st, "doc") {
                "constitution" => "constitution.html",
                "srs" => "srs.html",
                other => panic!("面の無い行き先「{other}」"),
            };
            let href = format!("<li><a href=\"{file}#{}\">", text(st, "at"));
            assert!(html.contains(&href), "行き先の href が無い: {href}");
            stops += 1;
        }
    }
    wants.push(text(&i["intake"], "text").to_string());
    for step in seq(&i["intake"]["steps"], "steps") {
        wants.push(step.as_str().unwrap().to_string());
    }
    for want in wants {
        let want = esc(&want);
        assert!(html.contains(&want), "面に無い: {want}");
    }
    let lanes = between(&html, "<div class=\"lane-grid\"", "\n</div>\n</div>");
    assert_eq!(lanes.matches("<li><a href=\"").count(), stops, "行き先の数");

    // 件数
    let parts = components(&html);
    let parts_of = |name: &str| parts.iter().filter(|p| **p == name).count();
    assert_eq!(parts_of("shelf-card"), 4, "shelf-card の数");
    assert_eq!(parts_of("shelf-link"), 4, "shelf-link の数");
    assert_eq!(parts_of("reader-lane"), rows.len(), "reader-lane の数");
    let articles = seq(&c["articles"], "articles").len();
    let fr = seq(&s["requirements"], "requirements").len();
    let terms = seq(&v["terms"], "terms").len();
    let rules =
        seq(&r["thresholds"], "thresholds").len() + seq(&r["discipline"], "discipline").len();
    let adr = fs::read_dir(design_intent().join("adr"))
        .unwrap()
        .filter(|e| {
            let name = e.as_ref().unwrap().file_name();
            let name = name.to_string_lossy();
            name.starts_with("ADR-") && name.ends_with(".yaml")
        })
        .count();
    for want in [
        format!("{articles} 条"),
        format!("機能 {fr}"),
        format!("{terms} 語"),
        format!("{rules} 行"),
        format!("{adr} 本"),
    ] {
        assert!(html.contains(&want), "数の字面が無い: {want}");
    }

    // 部品の名札は 12 種の中だけ
    const ALLOWED: [&str; 12] = [
        "freshness-stamp",
        "font-size-control",
        "hub-cover",
        "figure-panel",
        "doc-shelf",
        "shelf-card",
        "shelf-link",
        "status-line",
        "intake-line",
        "chapter-deck-band",
        "reader-lane",
        "intake-callout",
    ];
    assert!(!parts.is_empty());
    for p in &parts {
        assert!(ALLOWED.contains(p), "12 種に無い部品「{p}」");
    }
    assert!(!html.contains("layer-line"));
}

#[test]
fn face_index_check_has_three_values() {
    let (td, work) = index_fixture_copy("index-check");
    let out = td.join("index.html");

    let write = folio_face("index", &work, &out, "--write");
    let ok = folio_face("index", &work, &out, "--check");
    let mut bytes = fs::read(&out).unwrap();
    bytes[0] ^= 0x20;
    fs::write(&out, &bytes).unwrap();
    let drift = folio_face("index", &work, &out, "--check");
    fs::remove_file(&out).unwrap();
    let missing = folio_face("index", &work, &out, "--check");
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

/// 入口の面で、fixture の写しに変異を 1 つ当て、`--write` = 2 ∧「まだ分からない」∧ 出力先が出来ていない。
fn index_unknown(case: &str, mutate: impl FnOnce(&Path)) {
    index_unknown_from(
        index_fixture_copy(&format!("index-unknown-{case}")),
        case,
        mutate,
    );
}

/// 同じ形を支度表の在る写しで（便 20）。
fn index_unknown_with_a_sheet(case: &str, mutate: impl FnOnce(&Path)) {
    index_unknown_from(
        index_sheet_copy(&format!("index-unknown-{case}")),
        case,
        mutate,
    );
}

fn index_unknown_from((td, work): (PathBuf, PathBuf), case: &str, mutate: impl FnOnce(&Path)) {
    mutate(&work);
    let out = td.join("never.html");
    let run = folio_face("index", &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        code(&run, "folio face --face index"),
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

fn index_edit(from: &'static str, to: &'static str) -> impl FnOnce(&Path) {
    move |w| edit(&w.join("index.yaml"), |t| t.replacen(from, to, 1))
}

#[test]
fn face_index_unknown_when_a_document_is_missing() {
    index_unknown(
        "documents",
        index_edit(
            "    - {id: adr, type: 判断の記録, use: 戻せない判断の記録。, absent: まだ無い判断}\n",
            "",
        ),
    );
}

#[test]
fn face_index_unknown_when_a_relation_id_is_outside_the_table() {
    index_unknown(
        "relations",
        index_edit("{id: before-build,", "{id: before,"),
    );
}

#[test]
fn face_index_unknown_when_a_third_annex_is_added() {
    index_unknown(
        "annexes",
        index_edit(
            "    - {id: rules, type: 数値の表, inside: constitution}\n",
            "    - {id: rules, type: 数値の表, inside: constitution}\n    - {id: glossary, type: 用語, inside: constitution}\n",
        ),
    );
}

#[test]
fn face_index_unknown_when_a_stop_anchor_is_outside_the_face() {
    index_unknown(
        "stop-at",
        index_edit("at: s0, label: 目指すこと}", "at: s9, label: 目指すこと}"),
    );
}

#[test]
fn face_index_unknown_when_a_stop_names_a_document_without_a_face() {
    index_unknown(
        "stop-doc",
        index_edit(
            "{doc: srs, at: fig-rail,",
            "{doc: design-note, at: fig-rail,",
        ),
    );
}

#[test]
fn face_index_unknown_when_a_legend_id_is_outside_the_table() {
    index_unknown(
        "legend",
        index_edit("{id: binds, text: 矢印}", "{id: arrow, text: 矢印}"),
    );
}

#[test]
fn face_index_unknown_when_constitution_counts_differ() {
    index_unknown("counts", |w| {
        edit(&w.join("constitution.yaml"), |t| {
            t.replacen(
                "{always: 1, ask-first: 1, never: 1}",
                "{always: 2, ask-first: 1, never: 1}",
                1,
            )
        })
    });
}

#[test]
fn face_index_unknown_when_the_adr_dir_is_missing() {
    index_unknown("adr-dir", |w| fs::remove_dir_all(w.join("adr")).unwrap());
}

// ── 入口の面の節「支度表」（便 20・docs/design/delivery-20.md §1 (d)）──

/// 節「支度表」の字面（帯の始まりから chapbody の終わりまで）。
fn sheet_section(html: &str) -> &str {
    between(html, "<section id=\"s3\"", "\n</div>")
}

#[test]
fn face_index_with_a_sheet_passes_parts_check() {
    let (td, work) = index_sheet_copy("index-sheet-parts");
    let (out, _) = index_from("支度表あり", &work, &td);
    let check = parts_check(&[format!("index={}", out.display())]);
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

#[test]
fn face_index_census_of_the_sheet_section_without_a_sheet() {
    let (td, _, html) = real_index("index-sheet-census");
    let _ = fs::remove_dir_all(&td);
    let n = load_yaml("intake.yaml");
    let i = load_yaml("index.yaml");
    assert!(
        !design_intent().join(text(&n["sheet"], "file")).exists(),
        "実の正本に支度表が在る（この歯は支度表なしの形を測る）"
    );
    let section = sheet_section(&html);
    for want in [
        esc(text(&n["sheet"], "title")),
        esc(text(&n["sheet"], "explain")),
        "まだ無い".to_string(),
        format!(
            "「{}」と AI に頼むと作られる",
            esc(text(&i["intake"], "command"))
        ),
    ] {
        assert!(section.contains(&want), "節「支度表」に無い: {want}");
    }
    // 部品は 2 つだけ（slim の帯と status-line）・行は 1 行
    assert_eq!(
        components(section),
        ["chapter-deck-band", "status-line"],
        "節「支度表」の部品"
    );
    assert_eq!(section.matches("<p class=\"st").count(), 1, "行の数");
}

#[test]
fn face_index_census_of_the_sheet_section_with_a_sheet() {
    let (td, work) = index_sheet_copy("index-sheet-census-with");
    let (_, html) = index_from("支度表あり", &work, &td);
    let _ = fs::remove_dir_all(&td);
    let n = load_yaml_at(&fixture(), "intake.yaml");
    let sheet = load_yaml_at(&fixture(), "intake-sheet.yaml");
    let section = sheet_section(&html);

    let documents = seq(&sheet["documents"], "documents");
    let recommended = seq(&sheet["recommended"], "recommended");
    for d in documents {
        assert!(
            section.contains(&esc(text(d, "type"))),
            "持つ文書の型が節に無い: {}",
            text(d, "type")
        );
    }
    for r in recommended {
        assert!(
            section.contains(&esc(text(r, "ask"))),
            "推奨で進めた項目の文が節に無い: {}",
            text(r, "ask")
        );
    }
    // 承認はまだ（支度表の approval が空）・正本の file 名は intake.yaml の sheet.file
    assert!(seq(&sheet["approval"], "approval").is_empty());
    assert!(section.contains("<span class=\"k\">承認</span><span class=\"v\">まだ（"));
    assert!(section.contains(text(&n["sheet"], "file")));
    assert_eq!(
        components(section),
        ["chapter-deck-band", "status-line"],
        "節「支度表」の部品"
    );
    assert_eq!(
        section.matches("<p class=\"st").count(),
        4,
        "行の数（持つ文書・推奨で進めた項目・承認・正本）"
    );
}

// 導出できない 4 つ（どれも 2・出力先が出来ていない）

#[test]
fn face_index_unknown_when_the_intake_source_is_missing() {
    index_unknown("intake-missing", |w| {
        fs::remove_file(w.join("intake.yaml")).unwrap()
    });
}

#[test]
fn face_index_unknown_when_the_sheet_title_is_empty() {
    index_unknown("sheet-title", |w| {
        edit(&w.join("intake.yaml"), |t| {
            t.replacen(
                "  title: 見本の支度表 & <b>1 枚</b>\n",
                "  title: \"\"\n",
                1,
            )
        })
    });
}

#[test]
fn face_index_unknown_when_a_sheet_document_is_outside_the_targets() {
    index_unknown_with_a_sheet("sheet-document", |w| {
        edit(&w.join("intake-sheet.yaml"), |t| {
            t.replacen("{id: constitution, type: 憲法", "{id: memo, type: 憲法", 1)
        })
    });
}

#[test]
fn face_index_unknown_when_the_sheet_is_not_a_table() {
    index_unknown_with_a_sheet("sheet-seq", |w| {
        fs::write(w.join("intake-sheet.yaml"), "- 支度表\n- 一覧\n").unwrap()
    });
}

// 字面の 2 case（binary 経由）

#[test]
fn face_index_sheet_document_without_an_annex_has_no_parentheses() {
    let (td, work) = index_sheet_copy("index-sheet-with");
    edit(&work.join("intake-sheet.yaml"), |t| {
        t.replacen("with: [vocabulary]", "with: []", 1)
    });
    let (_, html) = index_from("付録なし", &work, &td);
    let _ = fs::remove_dir_all(&td);
    let sheet = load_yaml_at(&fixture(), "intake-sheet.yaml");
    let ty = esc(text(&seq(&sheet["documents"], "documents")[0], "type"));
    assert!(
        sheet_section(&html).contains(&format!(
            "<span class=\"k\">持つ文書</span><span class=\"v\">{ty}</span>"
        )),
        "{}",
        sheet_section(&html)
    );
}

#[test]
fn face_index_sheet_approval_is_the_first_row_when_it_is_stamped() {
    let (td, work) = index_sheet_copy("index-sheet-approval");
    edit(&work.join("intake-sheet.yaml"), |t| {
        t.replacen(
            "approval: []\n",
            "approval:\n  - {when: 2026-09-03, verbatim: 承認する}\n",
            1,
        )
    });
    let (_, html) = index_from("承認あり", &work, &td);
    let _ = fs::remove_dir_all(&td);
    assert!(
        sheet_section(&html)
            .contains("<span class=\"k\">承認</span><span class=\"v\">2026-09-03 承認する</span>"),
        "{}",
        sheet_section(&html)
    );
}

// ── 支度表の付録は index.yaml の annexes に解く（便 22・docs/design/delivery-22.md §1 (c)）──

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

/// 実の正本の写しで `folio intake --write`（`answers` が None なら全部おすすめ）→ 入口の面を書く。
/// 戻り値 = (一時 dir, 出力先, 面の本文)。
fn real_round_trip(case: &str, answers: Option<PathBuf>) -> (PathBuf, PathBuf, String) {
    let td = temp_dir(case);
    let work = td.join("design-intent");
    copy_tree(&design_intent(), &work);
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_folio"));
    cmd.arg("intake").arg("--dir").arg(&work);
    if let Some(path) = &answers {
        cmd.arg("--answers").arg(path);
    }
    let intake = cmd.arg("--write").output().expect("folio を起動できない");
    assert_eq!(
        code(&intake, "folio intake --write"),
        0,
        "{case}: {}",
        stderr(&intake)
    );
    let n = load_yaml("intake.yaml");
    assert!(
        work.join(text(&n["sheet"], "file")).exists(),
        "{case}: 支度表が書かれていない"
    );
    let (out, html) = index_from(case, &work, &td);
    (td, out, html)
}

/// 実の正本で憲法の行に出る付録の型（intake.yaml の constitution の with を index.yaml の annexes に解いたもの）。
fn real_constitution_annex_types() -> Vec<String> {
    let i = load_yaml("index.yaml");
    let n = load_yaml("intake.yaml");
    let annexes = seq(&i["shelf"]["annexes"], "annexes");
    let target = seq(&n["targets"], "targets")
        .iter()
        .find(|t| text(t, "id") == "constitution")
        .expect("intake.yaml の targets に constitution が無い");
    let with = seq(&target["with"], "with");
    assert!(!with.is_empty(), "憲法の行に付録が無い");
    with.iter()
        .map(|w| {
            let id = w.as_str().expect("付録の id が文字列でない");
            let a = annexes
                .iter()
                .find(|a| text(a, "id") == id)
                .unwrap_or_else(|| panic!("付録「{id}」が index.yaml の annexes に無い"));
            esc(text(a, "type"))
        })
        .collect()
}

/// 持つ文書の行の憲法の字面（付録は annexes の型を「・」で繋ぐ）。
fn real_constitution_line() -> String {
    let n = load_yaml("intake.yaml");
    let ty = seq(&n["targets"], "targets")
        .iter()
        .find(|t| text(t, "id") == "constitution")
        .map(|t| esc(text(t, "type")))
        .expect("intake.yaml の targets に constitution が無い");
    format!(
        "{ty}（付録の{}）",
        real_constitution_annex_types().join("・")
    )
}

#[test]
fn face_index_sheet_on_the_real_sources_round_trips_intake_and_parts_check() {
    let answers = repo_root().join("tests/fixtures/intake/answers-5.yaml");
    let (td, out, html) = real_round_trip("index-sheet-real", Some(answers));
    let check = parts_check(&[format!("index={}", out.display())]);
    let want = real_constitution_line();
    let section = sheet_section(&html).to_string();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        code(&check, "folio parts --check"),
        0,
        "{}{}",
        stdout(&check),
        stderr(&check)
    );
    assert!(stdout(&check).contains("違反 0"), "{}", stdout(&check));
    assert!(
        section.contains(&want),
        "節「支度表」に無い: {want}\n{section}"
    );
}

#[test]
fn face_index_sheet_on_the_real_sources_round_trips_with_the_recommended_answers() {
    let (td, _, html) = real_round_trip("index-sheet-real-recommended", None);
    let want = real_constitution_line();
    let section = sheet_section(&html).to_string();
    let _ = fs::remove_dir_all(&td);
    let n = load_yaml("intake.yaml");
    let questions = seq(&n["questions"], "questions");
    assert_eq!(questions.len(), 5, "質問の数");
    for q in questions {
        let ask = esc(text(q, "ask"));
        assert!(
            section.contains(&ask),
            "推奨で進めた項目に無い: {ask}\n{section}"
        );
    }
    assert!(
        section.contains(&want),
        "節「支度表」に無い: {want}\n{section}"
    );
}

#[test]
fn face_index_unknown_when_a_sheet_annex_is_outside_the_annexes() {
    let (td, work) = index_sheet_copy("index-unknown-sheet-annex");
    edit(&work.join("intake-sheet.yaml"), |t| {
        t.replacen("with: [vocabulary]", "with: [nowhere]", 1)
    });
    let out = td.join("never.html");
    let run = folio_face("index", &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --face index"), 2, "{}", stderr(&run));
    for want in [
        "まだ分からない",
        "intake-sheet.yaml.documents[0].with[0]",
        "付録の id「nowhere」が index.yaml の annexes に無い",
    ] {
        assert!(
            stderr(&run).contains(want),
            "{want} が無い: {}",
            stderr(&run)
        );
    }
    assert!(!exists, "導出できないのに出力先に書いた");
}

#[test]
fn face_index_sheet_annex_type_comes_from_the_index_annexes() {
    let (td, work) = index_sheet_copy("index-sheet-annex-type");
    edit(&work.join("intake-sheet.yaml"), |t| {
        t.replacen("with: [vocabulary]", "with: [rules]", 1)
    });
    let (_, html) = index_from("付録 rules", &work, &td);
    let section = sheet_section(&html).to_string();
    let _ = fs::remove_dir_all(&td);
    let n = load_yaml_at(&fixture(), "intake.yaml");
    assert!(
        seq(&n["targets"], "targets")
            .iter()
            .all(|t| text(t, "id") != "rules"),
        "付録 rules は intake.yaml の targets に無いはず（型は annexes から来る）"
    );
    let i = load_yaml_at(&fixture(), "index.yaml");
    let ty = seq(&i["shelf"]["annexes"], "annexes")
        .iter()
        .find(|a| text(a, "id") == "rules")
        .map(|a| esc(text(a, "type")))
        .expect("index.yaml の annexes に rules が無い");
    let sheet = load_yaml_at(&fixture(), "intake-sheet.yaml");
    let doc_ty = esc(text(&seq(&sheet["documents"], "documents")[0], "type"));
    let want =
        format!("<span class=\"k\">持つ文書</span><span class=\"v\">{doc_ty}（付録の{ty}）</span>");
    assert!(section.contains(&want), "{want} が無い: {section}");
}
