//! `folio face --face srs`（要件書の面）の図の歯（便 34・35・74・FR15）。binary 経由。
//! - 図 3 は verdicts の節が在るときだけ出る・無ければ章 09 の番号が 1 つ繰り上がる（便 35）
//! - 図の章: 写し（srs.yaml・図 1 枚）の toc の 09 と figure-panel と型の名札と根拠のリンク・図なしの面は図の章の外が
//!   byte で同じ・章 09 の番号は自前の図の続き・通らない図で 2 と前の面の保持・型外・道具の不在
//! - 表紙の図の列が面の figure-panel の列と同じ・行き先は正本の id（便 74）
//!
//! 便 105（docs/design/delivery-105.md §1 (b)）で `face.rs` から字を変えずに移した。要件書の面の本体の歯は
//! `face_srs_body.rs` に在る。歯の file どうしは互いに use できないので、helper は `face.rs` の写しを持つ
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

fn load_yaml_at(dir: &Path, name: &str) -> Yaml {
    let text = fs::read_to_string(dir.join(name)).unwrap();
    YamlLoader::load_from_str(&text).unwrap().remove(0)
}

fn load_yaml(name: &str) -> Yaml {
    load_yaml_at(&design_intent(), name)
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
