//! `folio face --face srs`（要件書の面）の本体の歯（便 15・36・64・70）。binary 経由。
//! - 凍結 fixture（tests/fixtures/face/）との byte 一致（P-10.1・回帰の anchor で唯一の判定にしない）・escape
//! - 実の正本で生成した面が `folio parts --check` に合格する・逐語と件数の census（yaml-rust2 で正本を直に読む）
//! - check の 3 値・導出できない入力 8 つ
//! - 用語集の章（便 36・憲法の面の章 07 と同じ形）・受入基準の章の凡例（便 64）・章の見出しの助数詞（便 70）
//!
//! 便 105（docs/design/delivery-105.md §1 (b)）で `face.rs` から字を変えずに移した。図の章の歯は
//! `face_srs_figure.rs` に在る。歯の file どうしは互いに use できないので、helper は `face.rs` の写しを持つ
//! （写しは字を変えない・この file の歯が呼ぶものだけ）。
//! 版管理の `design-intent/` の正本は書き換えない（`--out` と写しは必ず一時 dir の中）。

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
