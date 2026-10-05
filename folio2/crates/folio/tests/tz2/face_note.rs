//! 設計ノートの面（`folio face --face note --id <文書 id>`）の歯（便 28・docs/design/delivery-28.md §1 (g)）。binary 経由。
//! - 凍結 fixture（tests/fixtures/face/expected-note.html）との byte 一致（P-10.1）・escape
//! - 実の正本（example）で `folio parts --check` に合格（NFR2）と逐語と件数の census（yaml-rust2 で正本を直に読む）
//! - check の 3 値・id の口 4 形・節の型の閉じた一覧（FR9）・名札の表の外 2 つ・器の導出 file の欄と不在（FR10）・
//!   未解決の参照・発効と廃止・列挙の分割・章の上限・mode
//! - 図の章（便 31・FR15）: 図ごとの figure-panel と型の名札（parts.json の type_ids と突き合わせ）・図の class の母集団・
//!   通らない図で 2 と前の面の保持・道具の不在・型外・図なし・決定的・図の章の外は不変
//!
//! 版管理の下の面は書き換えない（`--out` は必ず一時 dir の中）。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::common::{
    code, copy_dir, cut, cut_line, design_intent, edit, fixture, repo_root, stderr, stdout,
    svg_bodies, vendor,
};
use folio::yaml_rust2::{Yaml, YamlLoader};

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-face-note-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    fs::create_dir_all(&td).unwrap();
    td
}

/// fixture の正本 4 file と design-note/full.yaml を一時 dir の下の src/ へ、器の導出 file を親 dir の
/// contracts/ へ、repo の vendor/archify/（図の道具・便 31 の面は図ごとに撃つ）を親 dir の vendor/archify/ へ写す。
/// 戻り値 = (一時 dir, 写し)。
fn fixture_copy(case: &str) -> (PathBuf, PathBuf) {
    let td = temp_dir(case);
    let work = td.join("src");
    fs::create_dir_all(work.join("design-note")).unwrap();
    fs::create_dir_all(td.join("contracts/field-schema")).unwrap();
    for name in [
        "constitution.yaml",
        "rules.yaml",
        "vocabulary.yaml",
        "srs.yaml",
        "ceiling.yaml",
        "design-note/full.yaml",
    ] {
        fs::copy(fixture().join(name), work.join(name)).unwrap();
    }
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        td.join("contracts/field-schema/schema.toml"),
    )
    .unwrap();
    copy_dir(&vendor(), &td.join("vendor/archify"));
    (td, work)
}

/// `folio face --face <face> [--id <id>] --dir <dir> --out <out> <mode>`。
fn folio_face(face: &str, id: Option<&str>, dir: &Path, out: &Path, mode: &str) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tz"));
    cmd.arg("face").arg("--face").arg(face);
    if let Some(id) = id {
        cmd.arg("--id").arg(id);
    }
    cmd.arg("--dir")
        .arg(dir)
        .arg("--out")
        .arg(out)
        .arg(mode)
        .output()
        .expect("folio を起動できない")
}

/// 5 字の escape（生成側の字面を使わず歯の側で持つ）。
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// 写しに変異を当て、`--write` の結果と面の本文を返す（面が出来ていなければ本文は空）。
fn mutated(case: &str, mutate: impl FnOnce(&str) -> String) -> (Output, String) {
    let (td, work) = fixture_copy(case);
    edit(&work.join("design-note/full.yaml"), mutate);
    let out = td.join("note-full.html");
    let run = folio_face("note", Some("full"), &work, &out, "--write");
    let html = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    (run, html)
}

/// 写しから面を組み、本文を返す（変異なし）。
fn fixture_html(case: &str) -> String {
    let (td, work) = fixture_copy(case);
    let out = td.join("note-full.html");
    let run = folio_face("note", Some("full"), &work, &out, "--write");
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    let html = fs::read_to_string(&out).unwrap();
    let _ = fs::remove_dir_all(&td);
    html
}

/// 変異が導出できない入力なら `--write` = 2 ∧「まだ分からない」∧ 文言。
fn unknown(case: &str, mutate: impl FnOnce(&str) -> String, wording: &str) {
    let (run, html) = mutated(case, mutate);
    assert_eq!(code(&run, "folio face"), 2, "{case}: {}", stderr(&run));
    assert!(
        stderr(&run).contains("まだ分からない"),
        "{case}: {}",
        stderr(&run)
    );
    assert!(
        stderr(&run).contains(wording),
        "{case}: 「{wording}」が無い: {}",
        stderr(&run)
    );
    assert!(html.is_empty(), "{case}: 導出できないのに面を書いた");
}

// ── 凍結 fixture ──

#[test]
fn face_note_write_matches_the_frozen_fixture() {
    let (td, work) = fixture_copy("anchor");
    let out = td.join("note-full.html");
    let run = folio_face("note", Some("full"), &work, &out, "--write");
    let written = fs::read(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert_eq!(
        stdout(&run),
        format!("folio face: 書いた（{} byte）\n", written.len())
    );
    let frozen = fs::read(fixture().join("expected-note.html")).unwrap();
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
            "expected-note.html と違う（{} byte ≠ {} byte・最初の差 {at} byte 目）\n--- folio\n{}\n--- 凍結\n{}",
            written.len(),
            frozen.len(),
            show(&written),
            show(&frozen)
        );
    }
}

#[test]
fn face_note_escapes_values_from_the_source() {
    let (run, html) = mutated("escape", |t| {
        t.replacen(
            "title: 設計ノートの面の凍結 fixture",
            "title: 設計ノートの<b>面</b>の凍結 fixture",
            1,
        )
    });
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert!(
        html.contains("&lt;b&gt;面&lt;/b&gt;"),
        "title の山括弧が escape されていない"
    );
    assert!(
        !html.contains("<b>面</b>"),
        "title の <b> が生のまま出ている"
    );
}

// ── 実の正本（example）──

fn real_example() -> Yaml {
    let text = fs::read_to_string(design_intent().join("design-note/example.yaml")).unwrap();
    YamlLoader::load_from_str(&text).unwrap().remove(0)
}

/// 実の正本 1 本から一時 file へ書く。
fn real_face(td: &Path) -> (PathBuf, String) {
    let out = td.join("note-example.html");
    let run = folio_face("note", Some("example"), &design_intent(), &out, "--write");
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    let html = fs::read_to_string(&out).unwrap();
    (out, html)
}

/// 面の下端の prevnext の（前の href・次の href）。
fn prevnext_hrefs(html: &str) -> (String, String) {
    let line = html
        .lines()
        .find(|l| l.starts_with("<nav class=\"prevnext\">"))
        .expect("prevnext が無い");
    let hrefs: Vec<&str> = line
        .split("<a href=\"")
        .skip(1)
        .map(|p| p.split('"').next().unwrap())
        .collect();
    assert_eq!(hrefs.len(), 2, "prevnext の a が 2 つでない: {line}");
    (hrefs[0].to_string(), hrefs[1].to_string())
}

// ── 前 / 次（便 65・delivery-65.md §1 (e)2）──

#[test]
fn neighbor_note_links_follow_the_shelf_order() {
    // 実の設計ノート（欄の決まり schema.yaml は除く）を id の字の順に = 入口の棚の順
    let mut ids: Vec<String> = fs::read_dir(design_intent().join("design-note"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n != "schema.yaml")
        .filter_map(|n| n.strip_suffix(".yaml").map(str::to_string))
        .collect();
    ids.sort();
    assert_eq!(ids, ["example", "figures"], "実の設計ノートの列");
    let td = temp_dir("neighbor");
    let faces: Vec<(String, String)> = ids
        .iter()
        .map(|id| {
            let out = td.join(format!("note-{id}.html"));
            let run = folio_face("note", Some(id), &design_intent(), &out, "--write");
            assert_eq!(
                code(&run, "folio face --write"),
                0,
                "{id}: {}",
                stderr(&run)
            );
            prevnext_hrefs(&fs::read_to_string(&out).unwrap())
        })
        .collect();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        faces[0],
        ("index.html".to_string(), "note-figures.html".to_string()),
        "1 本目の前は入口・次は 2 本目"
    );
    assert_eq!(
        faces[1],
        ("note-example.html".to_string(), "index.html".to_string()),
        "2 本目の前は 1 本目・次は入口"
    );
}

#[test]
fn face_note_on_the_real_source_passes_parts_check() {
    let td = temp_dir("parts");
    let (out, _) = real_face(&td);
    let check = Command::new(env!("CARGO_BIN_EXE_tz"))
        .arg("parts")
        .arg("--check")
        .arg("--dir")
        .arg(design_intent())
        .arg("--page")
        .arg(format!("note={}", out.display()))
        .output()
        .unwrap();
    let text = stdout(&check);
    let err = stderr(&check);
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&check, "folio parts --check"), 0, "{text}{err}");
    assert!(text.contains("違反 0"), "{text}");
}

#[test]
fn face_note_census_on_the_real_source_counts_and_verbatims() {
    let td = temp_dir("census");
    let (_, html) = real_face(&td);
    let _ = fs::remove_dir_all(&td);
    let d = real_example();
    let count = |needle: &str| html.matches(needle).count();

    // 逐語（h1 は短い名・副題が title の逐語）
    let title = esc(d["meta"]["title"].as_str().unwrap());
    assert!(
        html.contains("<h1>設計ノート example</h1>"),
        "h1 が「設計ノート <id>」でない"
    );
    assert!(
        html.contains(&format!("<p class=\"sub-title\">{title}</p>")),
        "副題が title の逐語でない"
    );

    // 章の帯（節 6 + 図の章 1・承認欄は id=approval なので数えない）。図の数は正本から数える（便 32・1 以上）
    let sections = d["sections"].as_vec().unwrap();
    let figures = d["figures"].as_vec().unwrap();
    assert_eq!(sections.len(), 6, "実の正本の節の数");
    assert!(!figures.is_empty(), "実の正本に図が無い");
    assert_eq!(count("<section id=\"s"), 7, "章の帯の数");

    // item-row（5 つの表の節の行の合計 = 4 + 4 + 4 + 2 + 1）
    let rows: usize = sections
        .iter()
        .filter_map(|s| s["rows"].as_vec().map(Vec::len))
        .sum();
    assert_eq!(rows, 15, "実の正本の表の節の行の合計");
    assert_eq!(count("data-component=\"item-row\""), rows, "item-row の数");

    contract_fields(&html);

    // 図の章は図ごとの図の枠（便 31）＝便 28 の card と「図の生成はまだ無い」の cd は 0
    assert_eq!(
        count("<div class=\"card accent warn\">"),
        0,
        "便 28 の図の card"
    );
    assert_eq!(count("図の生成はまだ無い"), 0, "便 28 の図の cd");
    assert_eq!(
        count("data-component=\"figure-panel\""),
        figures.len(),
        "figure-panel の数"
    );
    assert_eq!(
        count(" data-role=\"diagram\""),
        figures.len(),
        "data-role diagram の数"
    );
    assert_eq!(svg_bodies(&html), figures.len(), "図の本体の数");
    // 生成器が出す「まだ分からない」は 0（行き先の無い参照は 1 つも無い。正本の逐語の中の「まだ分からない」は α なので数えない）
    assert_eq!(
        count("（まだ分からない）"),
        0,
        "行き先の無い参照が在る: {html}"
    );
}

/// 契約表の行の欄の hint（無い欄と空の一覧の欄は出さない・検証は出す）を見る。
fn contract_fields(html: &str) {
    // 契約表の行の欄（write-set は無い・空の一覧の depends は hint を出さない・verify は在る）
    assert!(!html.contains("write-set"), "無い欄の hint が出ている");
    assert!(!html.contains("depends"), "空の一覧の欄の hint が出ている");
    assert!(html.contains(">検証</span>"), "検証の hint が無い: {html}");
}

// ── 図の章（便 31）──

/// 部品目録 parts.json（yaml-rust2 で読む・JSON は YAML の流れの表）。
fn parts_catalog() -> Yaml {
    let text = fs::read_to_string(design_intent().join("preview/parts.json")).unwrap();
    YamlLoader::load_from_str(&text)
        .expect("parts.json を読めない")
        .remove(0)
}

/// 部品目録の図の型 → 名札（figure_body_classes.type_ids・書かれた順）。
fn parts_type_ids() -> Vec<(String, String)> {
    let doc = parts_catalog();
    doc["figure_body_classes"]["type_ids"]
        .as_hash()
        .expect("figure_body_classes.type_ids が表でない")
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().expect("型が文字列でない").to_string(),
                v.as_str().expect("名札が文字列でない").to_string(),
            )
        })
        .collect()
}

/// 部品目録の図の本体の意味 class の全一覧（5 群の和・data 属性の一覧は class でない）。
fn parts_figure_classes() -> Vec<String> {
    let doc = parts_catalog();
    let table = &doc["figure_body_classes"];
    let mut out = Vec::new();
    for group in ["node_kind", "edge_kind", "arrowhead", "text_role", "sigil"] {
        let list = table[group]
            .as_vec()
            .unwrap_or_else(|| panic!("figure_body_classes.{group} が一覧でない"));
        for item in list {
            out.push(item.as_str().expect("class が文字列でない").to_string());
        }
    }
    // 便 32: 道具の 5 型の見本の実測で 39 語に閉じた（便 31 の 33 語 + 6 語）
    assert_eq!(out.len(), 39, "図の class の一覧が 39 語でない: {out:?}");
    for added in [
        "c-region",
        "c-security-group",
        "t-frontend",
        "t-external",
        "t-cloud",
        "t-database",
    ] {
        assert!(
            out.iter().any(|w| w == added),
            "便 32 で足した class「{added}」が parts.json の figure_body_classes に無い"
        );
    }
    out
}

/// 面の class の語（`class="…"` の中身を空白で割る）。
fn classes(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(at) = rest.find("class=\"") {
        let tail = &rest[at + "class=\"".len()..];
        let end = tail.find('"').unwrap_or(tail.len());
        for word in tail[..end].split_whitespace() {
            out.push(word.to_string());
        }
        rest = &tail[end.min(tail.len())..];
    }
    out
}

/// 写しの figures を空の一覧にした面。
fn figureless_html(case: &str) -> String {
    let (run, html) = mutated(case, |t| {
        let at = t.find("\nfigures:\n").expect("figures が無い");
        let end = t.find("\nsources: []").expect("sources が無い");
        format!("{}\nfigures: []{}", &t[..at], &t[end..])
    });
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    html
}

#[test]
fn face_note_on_the_real_source_embeds_each_figure_in_a_figure_panel() {
    let td = temp_dir("figures");
    let (_, html) = real_face(&td);
    let _ = fs::remove_dir_all(&td);
    let d = real_example();
    let figures = d["figures"].as_vec().unwrap();
    assert!(!figures.is_empty(), "実の正本に図が無い");
    let type_ids = parts_type_ids();
    // 型の名札の表は 5 組（字面は契約表 §1 (b) の閉じた表・parts.json の type_ids と同じ）
    let want: Vec<(String, String)> = [
        ("archify-architecture", "構成図（architecture）"),
        ("archify-workflow", "手順図（workflow）"),
        ("archify-sequence", "順序図（sequence）"),
        ("archify-dataflow", "流れ図（dataflow）"),
        ("archify-lifecycle", "状態図（lifecycle）"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    assert_eq!(type_ids, want, "parts.json の type_ids が 5 組の表と違う");

    for (i, fig) in figures.iter().enumerate() {
        let fid = fig["id"].as_str().unwrap();
        let caption = esc(fig["caption"].as_str().unwrap());
        let kind = fig["type"].as_str().unwrap();
        let label = &type_ids
            .iter()
            .find(|(k, _)| k == kind)
            .unwrap_or_else(|| panic!("型「{kind}」が type_ids に無い"))
            .1;
        assert!(
            html.contains(&format!(
                "<figure data-component=\"figure-panel\" data-role=\"diagram\" id=\"{fid}\">"
            )),
            "図 {fid} の figure-panel が無い"
        );
        assert!(
            html.contains(&format!(
                "<div class=\"fig-title\"><span class=\"fn\">図 {}</span>{caption} <span class=\"fig-tools\">",
                i + 1
            )),
            "図 {fid} の fig-title に caption の逐語が無い"
        );
        assert!(
            html.contains(&format!(
                "<figcaption><span class=\"ver\">図 {} · {label} · {fid}",
                i + 1
            )),
            "図 {fid} の figcaption に型の名札と id が無い: {html}"
        );
    }
    assert_eq!(
        svg_bodies(&html),
        figures.len(),
        "図の本体の数が図の数と違う"
    );
}

#[test]
fn face_note_on_the_real_source_uses_only_catalogued_figure_classes() {
    let td = temp_dir("classes");
    let (_, html) = real_face(&td);
    let _ = fs::remove_dir_all(&td);
    let allowed = parts_figure_classes();
    let mut seen = 0;
    for word in classes(&html) {
        let figure_word = ["c-", "a-", "m-", "t-", "s-"]
            .iter()
            .any(|p| word.starts_with(p))
            || word == "semantic-sigil"
            || word == "sigil-fill";
        if !figure_word {
            continue;
        }
        seen += 1;
        assert!(
            allowed.contains(&word),
            "図の class「{word}」が parts.json の figure_body_classes に無い"
        );
    }
    assert!(seen > 0, "面に図の class が 1 つも無い");
}

#[test]
fn face_note_unknown_when_a_figure_fails_the_tool_check_and_keeps_the_previous_face() {
    let (td, work) = fixture_copy("fig-fails");
    // layout を消すと道具の検査（showcase）に落ちる
    edit(&work.join("design-note/full.yaml"), |t| {
        t.replacen(
            "      layout: {mode: grid, cols: 2, gapX: 70, gapY: 110, cellW: 160, cellH: 70}\n",
            "",
            1,
        )
    });
    let out = td.join("note-full.html");
    let before = "<!DOCTYPE html>\n前の面\n".as_bytes().to_vec();
    fs::write(&out, &before).unwrap();
    let run = folio_face("note", Some("full"), &work, &out, "--write");
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
fn face_note_unknown_when_the_figure_tool_is_absent() {
    let (td, work) = fixture_copy("no-tool");
    fs::remove_file(td.join("vendor/archify/bin/archify.mjs")).unwrap();
    let out = td.join("note-full.html");
    let run = folio_face("note", Some("full"), &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --write"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("まだ分からない"), "{}", stderr(&run));
    assert!(stderr(&run).contains("図の道具が無い"), "{}", stderr(&run));
    assert!(!exists, "導出できないのに面を書いた");
}

#[test]
fn face_note_unknown_when_a_figure_type_is_not_a_tool_type() {
    unknown(
        "fig-type",
        |t| t.replacen("type: archify-architecture", "type: pipeline-rail", 1),
        "図の道具の型でない",
    );
}

#[test]
fn face_note_without_figures_has_no_figure_chapter() {
    let html = figureless_html("no-figures");
    assert_eq!(
        html.matches("data-component=\"figure-panel\"").count(),
        0,
        "図が無いのに figure-panel が在る"
    );
    assert!(
        !html.contains("<section id=\"s7\""),
        "図が無いのに図の章が在る"
    );
    assert_eq!(svg_bodies(&html), 0, "図が無いのに図の本体が在る");
    assert!(
        !html.contains("<li><a href=\"#s7\">")
            && !html.contains("<span class=\"k\">図</span><span class=\"t\">"),
        "図が無いのに toc に図の項が在る"
    );
    assert!(
        html.contains("<span class=\"k\">図</span><span class=\"v\">0 枚</span>"),
        "cover-meta の図が 0 枚でない"
    );
}

#[test]
fn face_note_write_twice_yields_the_same_bytes() {
    let (td, work) = fixture_copy("twice");
    let out = td.join("note-full.html");
    let first = folio_face("note", Some("full"), &work, &out, "--write");
    let a = fs::read(&out).unwrap_or_default();
    let second = folio_face("note", Some("full"), &work, &out, "--write");
    let b = fs::read(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&first, "1 回目"), 0, "{}", stderr(&first));
    assert_eq!(code(&second, "2 回目"), 0, "{}", stderr(&second));
    assert!(
        String::from_utf8_lossy(&a).contains("<svg"),
        "面に図の本体が無い"
    );
    assert_eq!(a, b, "2 度撃つと byte が違う（図を含めて決定的でない）");
}

#[test]
fn face_note_figure_chapter_is_the_only_difference_from_the_figureless_face() {
    let frozen = fs::read_to_string(fixture().join("expected-note.html")).unwrap();
    let without = figureless_html("outside");
    // 図の章（s7 の帯から承認欄の帯の直前まで）・cover-meta の図・toc の図の項 を両方から抜く
    let strip = |html: &str| {
        let h = cut(html, "<section id=\"s7\"", "<section id=\"approval\"");
        let h = cut_line(&h, "<span class=\"m\"><span class=\"k\">図</span>");
        cut_line(&h, "<li><a href=\"#s7\">")
    };
    let a = strip(&frozen);
    let b = strip(&without);
    // 章の数から数える字（全 N 章・k/N・機械のための面の figures）だけは正本から数えた数（γ）なので揃える
    let a = a
        .replace("全 8 章", "全 7 章")
        .replace("/8</span>", "/7</span>")
        .replace("<dt>figures</dt><dd>1</dd>", "<dt>figures</dt><dd>0</dd>");
    if a != b {
        let at = a
            .bytes()
            .zip(b.bytes())
            .position(|(x, y)| x != y)
            .unwrap_or(a.len().min(b.len()));
        let show = |s: &str| {
            String::from_utf8_lossy(&s.as_bytes()[at.saturating_sub(120)..(at + 200).min(s.len())])
                .into_owned()
        };
        panic!(
            "図の章の外が違う（最初の差 {at} byte 目）\n--- 凍結\n{}\n--- 図なし\n{}",
            show(&a),
            show(&b)
        );
    }
}

// ── check の 3 値 ──

#[test]
fn face_note_check_has_three_values() {
    let (td, work) = fixture_copy("check");
    let out = td.join("note-full.html");

    let write = folio_face("note", Some("full"), &work, &out, "--write");
    let ok = folio_face("note", Some("full"), &work, &out, "--check");
    let mut bytes = fs::read(&out).unwrap();
    bytes[0] ^= 0x20;
    fs::write(&out, &bytes).unwrap();
    let drift = folio_face("note", Some("full"), &work, &out, "--check");
    fs::remove_file(&out).unwrap();
    let missing = folio_face("note", Some("full"), &work, &out, "--check");
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

// ── id の口 ──

#[test]
fn face_note_id_is_required_and_only_on_the_adr_and_note_faces() {
    let (td, work) = fixture_copy("id");
    let out = td.join("never.html");
    let no_id = folio_face("note", None, &work, &out, "--write");
    let missing = folio_face("note", Some("nope"), &work, &out, "--write");
    let on_srs = folio_face("srs", Some("full"), &work, &out, "--write");
    let bad_shape = folio_face("note", Some("Full"), &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&no_id, "--id 無し"), 2, "{}", stderr(&no_id));
    assert!(stderr(&no_id).contains("--id が無い"), "{}", stderr(&no_id));
    assert_eq!(code(&missing, "無い id"), 2, "{}", stderr(&missing));
    assert_eq!(code(&on_srs, "面 srs に --id"), 2, "{}", stderr(&on_srs));
    assert!(
        stderr(&on_srs).contains("--id は面 adr と note にだけ付く"),
        "{}",
        stderr(&on_srs)
    );
    assert_eq!(
        code(&bad_shape, "id の形でない"),
        2,
        "{}",
        stderr(&bad_shape)
    );
    assert!(
        stderr(&bad_shape).contains("id の形でない"),
        "{}",
        stderr(&bad_shape)
    );
    assert!(!exists, "導出できないのに出力先に書いた");
}

// ── 導出できない入力 ──

#[test]
fn face_note_unknown_when_a_section_type_is_outside_the_list() {
    unknown(
        "type",
        |t| t.replacen("type: prose", "type: recipe", 1),
        "閉じた一覧に無い",
    );
}

#[test]
fn face_note_unknown_when_the_status_is_outside_the_table() {
    unknown(
        "status",
        |t| t.replacen("status: draft", "status: final", 1),
        "状態",
    );
}

#[test]
fn face_note_unknown_when_a_field_need_is_outside_the_table() {
    unknown(
        "need",
        |t| t.replacen("need: required", "need: maybe", 1),
        "要否",
    );
}

#[test]
fn face_note_unknown_when_a_contract_field_is_not_in_the_derived_file() {
    unknown(
        "budget",
        |t| t.replacen("size: S,", "size: S, budget: 3,", 1),
        "器の導出 file に無い",
    );
}

#[test]
fn face_note_unknown_when_the_derived_file_is_missing() {
    let (td, work) = fixture_copy("no-external");
    fs::remove_file(td.join("contracts/field-schema/schema.toml")).unwrap();
    let out = td.join("note-full.html");
    let run = folio_face("note", Some("full"), &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face"), 2, "{}", stderr(&run));
    assert!(
        stderr(&run).contains("器の導出 file が読めない"),
        "{}",
        stderr(&run)
    );
    assert!(!exists, "導出できないのに面を書いた");
}

// ── 行き先の無い参照・状態 ──

#[test]
fn face_note_marks_a_ref_id_without_a_target() {
    let (run, html) = mutated("unresolved", |t| {
        t.replacen("ref: [P-1, FR1]", "ref: [P-1, FR1, FR9]", 1)
    });
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert!(
        html.contains("FR9（まだ分からない）"),
        "行き先の無い id に印が無い"
    );
    assert!(
        !html.contains("srs.html#fr9"),
        "行き先の無い id をリンクにした"
    );
}

#[test]
fn face_note_shows_the_effective_and_retired_states() {
    let (effective, html) = mutated("effective", |t| {
        t.replacen("status: draft", "status: effective", 1).replacen(
            "  note: 手書きの見本。面の骨格だけを測る。",
            "  note: 手書きの見本。面の骨格だけを測る。\n  approval: [{who: 持ち主, date: 2026-09-18, ruling: f2-648.43 notes 2026-09-18, verbatim: 承認する, surface: R-8}]",
            1,
        )
    });
    assert_eq!(code(&effective, "effective"), 0, "{}", stderr(&effective));
    assert!(
        html.contains("発効・拘束力あり（承認 2026-09-18）"),
        "effective の状態の行が無い"
    );
    assert_eq!(
        html.matches("<div class=\"sign\">").count(),
        1,
        "承認欄の sign が 1 つでない"
    );

    let (retired, html) = mutated("retired", |t| {
        t.replacen("status: draft", "status: retired", 1).replacen(
            "  note: 手書きの見本。面の骨格だけを測る。",
            "  note: 手書きの見本。面の骨格だけを測る。\n  superseded_by: full2",
            1,
        )
    });
    assert_eq!(code(&retired, "retired"), 0, "{}", stderr(&retired));
    assert!(html.contains("廃止 → 後継 "), "retired の状態の行が無い");
    assert!(
        html.contains("href=\"note-full2.html\""),
        "後継へのリンクが無い（実在は確かめない）"
    );
}

// ── 散文の段落と列挙の分割 ──

#[test]
fn face_note_splits_the_paragraphs_and_the_head_of_sentence_enumeration() {
    let html = fixture_html("prose");
    assert!(
        html.contains(
            "<p>面の骨格を 1 枚で測る。段落は空行で分かれ、段落の中の改行は空白 1 つになる。</p>"
        ),
        "1 段落目が p 1 つでない: {html}"
    );
    assert_eq!(
        html.matches("<p class=\"intro\">前置き。</p>").count(),
        1,
        "前置きの p が 1 つでない: {html}"
    );
    assert!(
        html.contains("<ol class=\"items\">\n<li>あ。</li>\n<li>い。</li>\n</ol>"),
        "列挙が li 2 つに分かれていない: {html}"
    );
}

// ── 章の上限 ──

#[test]
fn face_note_unknown_when_there_are_too_many_chapters() {
    let extra: String = (7..=13)
        .map(|n| format!("  - {{n: {n}, type: prose, title: 追加 {n}, body: 追加の節。}}\n"))
        .collect();
    unknown(
        "chapters",
        |t| t.replacen("\nfigures:\n", &format!("{extra}\nfigures:\n"), 1),
        "上限 12",
    );
}

// ── 章の上限は置き場の規則の表の欄 key の行から読む（便 179・docs/design/delivery-179.md §1 (c)）──

/// 写しの rules.yaml と full.yaml に変異を当てて `--write` を撃ち、結果と面の本文を返す（面が出来ていなければ本文は空）。
fn with_rules(
    case: &str,
    rules: impl FnOnce(&str) -> String,
    note: impl FnOnce(&str) -> String,
) -> (Output, String) {
    let (td, work) = fixture_copy(case);
    edit(&work.join("rules.yaml"), rules);
    edit(&work.join("design-note/full.yaml"), note);
    let out = td.join("note-full.html");
    let run = folio_face("note", Some("full"), &work, &out, "--write");
    let html = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    (run, html)
}

/// full.yaml（節 6・図 1）に節 7〜`last` を足す（章 = last + 図の章 1）。
fn more_sections(last: usize) -> impl FnOnce(&str) -> String {
    move |t: &str| {
        let extra: String = (7..=last)
            .map(|n| format!("  - {{n: {n}, type: prose, title: 追加 {n}, body: 追加の節。}}\n"))
            .collect();
        t.replacen("\nfigures:\n", &format!("{extra}\nfigures:\n"), 1)
    }
}

#[test]
fn f179_face_reads_the_cap_from_the_keyed_row_whatever_its_id() {
    // 章 14（節 13 + 図の章）: 行の値 12 では 2 で、行の id を R-26 に・値を 14 章 以下にすると導出でき、7 章目から帯の組を繰り返す
    let (run, html) = with_rules("f179-cap-12", |t| t.to_string() + "\n", more_sections(13));
    assert_eq!(code(&run, "folio face"), 2, "{}", stderr(&run));
    // 便 207（ADR-35 決定 (1)(ク)）: 断りの字は床の違反と同じ字で今の数を持たず、今の数は次の # の行
    let err = stderr(&run);
    let refusal = err.lines().find(|l| l.contains("章が多すぎる")).unwrap_or_else(|| panic!("{err}"));
    assert!(refusal.ends_with("design-note/full.yaml: 章が多すぎる（note-chapters の上限 12 章 以下）"), "{err}");
    assert!(!refusal.contains("14"), "{refusal}");
    assert!(err.lines().any(|l| l == "# 今の数: design-note/full.yaml: 章の今の数 14（note-chapters の上限 12 章 以下）"), "{err}");
    assert!(html.is_empty(), "上限を超えたのに面を書いた");
    let (run, html) = with_rules(
        "f179-cap-14",
        |t| t.replacen("  - id: R-2\n", "  - id: R-26\n", 1).replacen("\"12 章 以下\"", "\"14 章 以下\"", 1),
        more_sections(13),
    );
    assert_eq!(code(&run, "folio face"), 0, "{}", stderr(&run));
    for (n, band) in [(1, "band-1"), (6, "band-6"), (7, "band-1"), (12, "band-6"), (13, "band-1"), (14, "band-2")] {
        assert!(html.contains(&format!("<section id=\"s{n}\" ")), "章 {n} が無い");
        let at = html.find(&format!("<section id=\"s{n}\" ")).unwrap();
        let head = &html[at..at + html[at..].find('>').unwrap()];
        assert!(head.contains(&format!("class=\"{band}\"")), "章 {n} の帯が {band} でない: {head}");
    }
    // 章の帯の crumb の分母は帯の章 14 + 承認欄 1
    assert!(html.contains(" 14/15</span>"), "章の数 14 が面に出ていない");
    assert!(!html.contains("<section id=\"s15\" "), "章 15 が在る");
}

/// 行だけを崩した写し（章 7 の今の fixture のまま）で `--write` が 2・まだ分からない・`wording` を出し、面を書かない。
fn unknown_cap(case: &str, rules: impl FnOnce(&str) -> String, wording: &str) {
    let (run, html) = with_rules(case, rules, |t| t.to_string() + "\n");
    assert_eq!(code(&run, "folio face"), 2, "{case}: {}", stderr(&run));
    assert!(stderr(&run).contains("まだ分からない"), "{case}: {}", stderr(&run));
    assert!(stderr(&run).contains("設計ノートの章の上限が読めない"), "{case}: {}", stderr(&run));
    assert!(stderr(&run).contains(wording), "{case}: 「{wording}」が無い: {}", stderr(&run));
    assert!(html.is_empty(), "{case}: 導出できないのに面を書いた");
}

#[test]
fn f179_face_is_unknown_when_the_keyed_row_is_missing_doubled_or_malformed() {
    // どれも既定の値に倒れない（道具は章の上限の値を持たない）
    unknown_cap(
        "f179-missing",
        |t| t.replacen("    key: note-chapters\n", "", 1),
        "note-chapters の閾値の行が無い",
    );
    unknown_cap(
        "f179-doubled",
        |t| t.replacen("\ndiscipline:", "  - {id: R-3, value: \"40 章 以下\", key: note-chapters}\n\ndiscipline:", 1),
        "2 本ある",
    );
    unknown_cap(
        "f179-malformed",
        |t| t.replacen("\"12 章 以下\"", "\"12章以下\"", 1),
        "行 R-2 の value「12章以下」が「<正の整数> 章 以下」の形でない",
    );
    unknown_cap(
        "f179-zero",
        |t| t.replacen("\"12 章 以下\"", "\"0 章 以下\"", 1),
        "の形でない",
    );
}

// ── mode ──

#[test]
fn face_note_mode_is_exactly_one() {
    for args in [&["--check", "--write"][..], &[][..]] {
        let out = Command::new(env!("CARGO_BIN_EXE_tz"))
            .arg("face")
            .arg("--face")
            .arg("note")
            .arg("--id")
            .arg("full")
            .arg("--dir")
            .arg(fixture())
            .arg("--out")
            .arg(std::env::temp_dir().join("folio-face-note-never-written.html"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}: {}", stderr(&out));
    }
}

// ── 名札（便 71・天井の 12 周目の読みやすさ F-6 / F-7 / F-8）──

/// 正本の `id: <rid>` の行の `<key>: ` の値（流れの表の 1 行・次の `, ` か `}` まで）。
fn row_value(yaml: &str, rid: &str, key: &str) -> String {
    let line = yaml
        .lines()
        .find(|l| l.contains(&format!("{{id: {rid},")))
        .unwrap_or_else(|| panic!("行 {rid} が無い"));
    let at = line.find(&format!(" {key}: ")).expect("欄が無い") + key.len() + 3;
    let tail = &line[at..];
    let end = tail.find([',', '}']).unwrap_or(tail.len());
    tail[..end].to_string()
}

#[test]
fn label_fix_red_when_has_a_label() {
    let html = fixture_html("label-red-when");
    let yaml = fs::read_to_string(fixture().join("design-note/full.yaml")).unwrap();
    let red_when = esc(&row_value(&yaml, "anchor", "red_when"));
    // 名札は red_when の文の直前・同じ要素（p.norm）の中
    let want = format!("<p class=\"norm\"><span class=\"pk\">赤くなる条件</span>{red_when}</p>");
    assert!(html.contains(&want), "red_when に名札が無い: {html}");
    assert_eq!(html.matches("赤くなる条件").count(), 1, "名札の数");
}

#[test]
fn label_fix_size_legend_is_present() {
    let html = fixture_html("label-size-legend");
    let legends: Vec<&str> = html
        .lines()
        .filter(|l| l.starts_with("<div class=\"legend-line\">"))
        .collect();
    assert_eq!(legends.len(), 1, "legend-line の数: {legends:?}");
    let legend = legends[0];
    assert!(legend.contains("大きさ: S = "), "S の凡例が無い: {legend}");
    assert!(legend.contains("M = "), "M の凡例が無い: {legend}");
    assert!(
        legend.find("S = ") < legend.find("M = "),
        "値域の順（S / M）でない: {legend}"
    );
    // 置き場は契約表の章（帯 s6 の後・次の帯の前）
    let at = html.find(legend).unwrap();
    let chapter = html.find("<section id=\"s6\"").expect("契約表の章が無い");
    assert!(chapter < at, "凡例が契約表の章より前に在る");
    let between = &html[chapter + 1..at];
    assert!(
        between.contains("<h2>§6 契約表</h2>"),
        "s6 が契約表の章でない"
    );
    assert!(
        !between.contains("<section id="),
        "凡例が契約表の章の後の章に在る"
    );
}

#[test]
fn label_fix_footer_names_the_real_file() {
    let html = fixture_html("label-footer");
    let yaml = real_yaml(&fixture().join("design-note/full.yaml"));
    let id = yaml["meta"]["id"].as_str().unwrap();
    let foot = html
        .lines()
        .find(|l| l.starts_with("<p class=\"ft-plain\">"))
        .expect("脚注が無い");
    assert!(
        !foot.contains("design-note/&lt;文書 id&gt;.yaml") && !foot.contains("<文書 id>"),
        "脚注に差し込みの合図が残る: {foot}"
    );
    assert!(
        foot.contains(&format!("正本 design-note/{id}.yaml から")),
        "脚注に実際の file 名が無い: {foot}"
    );
}

fn real_yaml(path: &Path) -> Yaml {
    let text = fs::read_to_string(path).unwrap();
    YamlLoader::load_from_str(&text).unwrap().remove(0)
}

// ── 用語集への札と読み手の名札（便 74・delivery-74.md §1 (c)(f)）──

#[test]
fn f74_note_foot_links_to_the_glossary() {
    let html = fixture_html("f74-chip");
    let line = html
        .lines()
        .find(|l| l.starts_with("<p class=\"doc-locator\">"))
        .expect("doc-locator の行が無い");
    assert_eq!(
        line.matches("<span class=\"annex-chips\">").count(),
        1,
        "doc-locator の行に札の span が 1 つでない: {line}"
    );
    assert!(
        line.ends_with(
            "<a href=\"index.html\">入口へ戻る</a><span class=\"annex-chips\"><a href=\"constitution.html#s7\">付録 語彙 <span class=\"cnt\">2 語 → 憲法 §7</span></a></span></p>"
        ),
        "doc-locator の行の札の字・行き先が違う: {line}"
    );
}

#[test]
fn f74_note_cover_names_the_reader() {
    let html = fixture_html("f74-reader");
    let start = html
        .find("<header data-component=\"doc-cover-band\">")
        .expect("表紙が無い");
    let cover = &html[start..start + html[start..].find("</header>").unwrap()];
    let reader =
        "<div class=\"summary-card\"><span class=\"ic\">読</span><div><p class=\"lab\">読み手</p>";
    assert_eq!(
        cover.matches(reader).count(),
        1,
        "表紙に読み手の名札が 1 枚でない: {cover}"
    );
    let note = cover
        .find("<p class=\"lab\">注</p>")
        .expect("写しの表紙に注の枠が無い");
    assert!(
        cover.find(reader).unwrap() < note,
        "読み手の名札が注の枠より後に在る"
    );
    let source = fs::read_to_string(fixture().join("design-note/full.yaml")).unwrap();
    assert!(
        !source.contains("読み手"),
        "正本に「読み手」の字が在る（欄を足していないはず）"
    );
}

// ── 便 146: 鮮度の札・版の札・足の行の日付は承認欄の日付（draft と見本は読まない・無ければ生成日・
// docs/design/delivery-146.md §1 (c) の 7）──

#[test]
fn f146_note_stamp_cover_and_foot_follow_the_approval() {
    let note = "  note: 手書きの見本。面の骨格だけを測る。";
    let approval = format!(
        "{note}\n  approval: [{{who: 持ち主, date: 2026-09-21, ruling: f2-648.219 notes, verbatim: 承認する, surface: R-8}}]"
    );
    // (写しの名, 状態, 承認欄を足すか, 日付の名, 日付)
    let cases = [
        ("f146-draft", "draft", false, "生成", "2026-09-18"),
        ("f146-draft-ap", "draft", true, "生成", "2026-09-18"),
        ("f146-effective", "effective", true, "承認", "2026-09-21"),
    ];
    for (case, status, with_approval, dated, date) in cases {
        let (run, html) = mutated(case, |t| {
            let mut t = t.replacen("status: draft", &format!("status: {status}"), 1);
            if with_approval {
                t = t.replacen(note, &approval, 1);
            }
            if !with_approval {
                t.push_str("# 変異なし\n");
            }
            t
        });
        assert_eq!(code(&run, case), 0, "{case}: {}", stderr(&run));
        let once = |want: &str, what: &str| {
            assert_eq!(html.matches(want).count(), 1, "{case} の{what}「{want}」がちょうど 1 つでない");
        };
        once(
            &format!("<span data-component=\"freshness-stamp\">{dated} <b>{date}</b> · <b>full v0.1</b>（"),
            "鮮度の札",
        );
        once(
            &format!("<span class=\"m\"><span class=\"k\">版</span><span class=\"v\">v0.1 / {date}</span></span>"),
            "版の札",
        );
        once(&format!(" · 設計ノート full v0.1（{dated} {date}）· 手で直さない</p>"), "足の行");
        if dated == "承認" {
            assert!(
                !html.contains("<span data-component=\"freshness-stamp\">生成 "),
                "{case}: 承認の日付の在る面に生成日の鮮度の札が在る"
            );
            assert!(!html.contains("2026-09-18"), "{case}: 生成日が面に残る");
        }
    }
}

// ── 計画の設計ノートの 3 つの型（便 184・docs/design/delivery-184.md §1 (c)・判断の記録 ADR-31 決定 (2)(イ)(7)・要件書 FR9 / FR27） ──

const PLAN_BEGIN: &str = "# folio:rows:begin — 生成区間・手で直さない・正本は置き場の契約表（folio derive --write が書く）";

/// 行の索引（行 1 本と、印だけの 0 本）・計画だけの行・判断の表を持つ計画のノート（歯の中の最小の手書き）。
fn plan_note(index_rows: &str) -> String {
    format!(
        "meta:\n  id: plan\n  title: 計画のノート\n  version: v0.1\n  status: example\n  generated: 2026-09-28\n  profile: design-note\nsections:\n  - n: 1\n    type: prose\n    title: 目的\n    body: 計画の見本。\n  - n: 2\n    type: row-index\n    title: 行の索引\n    rows:\n      {PLAN_BEGIN}\n{index_rows}      # folio:rows:end\n  - n: 3\n    type: row-plan\n    title: 計画だけの行\n    rows:\n      - {{id: b, what: 次の <行>, size: S, files: [crates/x.rs, crates/y.rs], depends: [a], ruling: t3-hub.1, note: 注の字}}\n      - {{id: c, what: その次の行}}\n  - n: 4\n    type: decision-table\n    title: 判断\n    rows:\n      - {{id: d1, text: 行を足す, ruling: f2-648 notes 2026-09-28 10:29 JST}}\n"
    )
}

/// 歯 f184_ 1: 3 つの型の章を名札つきで描き、索引の行は所属の設計ノートの面へ、計画だけの行は大きさ・依存・書く file・
/// 拠る裁定・注を、判断の表は裁定の字を出す（値は escape して逐語）。面は部品目録の外の class を持たない（parts --check 0）。
#[test]
fn f184_the_three_plan_types_are_drawn_with_their_fields() {
    let (td, work) = fixture_copy("plan-types");
    fs::write(work.join("design-note/plan.yaml"), plan_note("      - {id: a, doc: full}\n")).unwrap();
    let out = td.join("note-plan.html");
    let run = folio_face("note", Some("plan"), &work, &out, "--write");
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    let html = fs::read_to_string(&out).unwrap();
    for want in [
        "行の索引",
        "計画だけの行",
        "判断の表",
        "<a class=\"xref\" href=\"note-full.html\">full</a>",
        &esc("次の <行>"),
        "<span class=\"pill\">S</span>",
        "<code>crates/x.rs</code><br><code>crates/y.rs</code>",
        "t3-hub.1",
        "注の字",
        "行を足す",
        "f2-648 notes 2026-09-28 10:29 JST",
    ] {
        assert!(html.contains(want), "「{want}」が無い");
    }
    let check = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["parts", "--check", "--dir"])
        .arg(design_intent())
        .arg("--page")
        .arg(format!("note={}", out.display()))
        .output()
        .unwrap();
    assert_eq!(code(&check, "folio parts --check"), 0, "{}{}", stdout(&check), stderr(&check));
    // 行の索引の生成区間が空（rows は印だけ＝null）でも面を描く
    fs::write(work.join("design-note/plan.yaml"), plan_note("")).unwrap();
    let run = folio_face("note", Some("plan"), &work, &out, "--write");
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert!(!fs::read_to_string(&out).unwrap().contains("note-full.html\">full</a>"));
    let _ = fs::remove_dir_all(&td);
}

// ── 便 193（docs/design/delivery-193.md §1 (c)・台帳 f2-648.231）: 設計ノートの散文も判断の記録の面と同じ印の数え ──

#[test]
fn f193_note_prose_splits_after_a_bold_or_quote_closer() {
    let (run, html) = mutated("f193-closers", |t| {
        t.replacen("前置き。(1) あ。(2) い。", "前置き。(1) **あ。** (2) 「い。」 (3) う。", 1)
    });
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert!(
        html.contains("<ol class=\"items\">\n<li>**あ。**</li>\n<li>「い。」</li>\n<li>う。</li>\n</ol>"),
        "閉じの字の後の項が分かれていない: {html}"
    );
}

// ── 便 209（docs/design/delivery-209.md §1 (c)・判断の記録 ADR-35 決定 (3)(オ)・要件書 FR32・AC35）: 廃止の行の後継のリンクは先の種類の面へ ──

/// 写しの見本を廃止にして後継を `to` にし、`adr` が真なら fixture の判断の記録 ADR-2 を写しの adr/ へ置いて面を組む（本文を返す）。
fn retired_face(case: &str, to: &str, adr: bool) -> String {
    let (td, work) = fixture_copy(case);
    if adr {
        fs::create_dir_all(work.join("adr")).unwrap();
        fs::copy(fixture().join("adr/ADR-2.yaml"), work.join("adr/ADR-2.yaml")).unwrap();
    }
    edit(&work.join("design-note/full.yaml"), |t| {
        t.replacen("status: draft", "status: retired", 1).replacen(
            "  note: 手書きの見本。面の骨格だけを測る。",
            &format!("  note: 手書きの見本。面の骨格だけを測る。\n  superseded_by: {to}"),
            1,
        )
    });
    let out = td.join("note-full.html");
    let run = folio_face("note", Some("full"), &work, &out, "--write");
    assert_eq!(code(&run, "folio face --write"), 0, "{to}: {}", stderr(&run));
    let html = fs::read_to_string(&out).unwrap();
    let _ = fs::remove_dir_all(&td);
    html
}

/// 便 209 (c)7（AC35）: 後継が要件・非機能要件なら要件書の面の要件の場所へ、判断の記録なら判断の記録の面へリンクを張る
/// （fixture の要件書に在る FR2・NFR1、写しの adr/ に置いた ADR-2）。設計ノートの先は今のまま設計ノートの面へ。
#[test]
fn f209_retired_successor_links_to_the_srs_and_adr_faces() {
    for (to, adr, href) in [
        ("FR2", false, "srs.html#fr2"),
        ("NFR1", false, "srs.html#nfr1"),
        ("ADR-2", true, "adr-2.html"),
        ("full2", false, "note-full2.html"),
    ] {
        let html = retired_face(&format!("f209-link-{to}"), to, adr);
        let want = format!("廃止 → 後継 <a class=\"xref\" href=\"{href}\">{to}</a>");
        assert!(html.contains(&want), "{to}: 「{want}」が無い");
    }
}

/// 便 209 (c)8（AC35・ADR-35 決定 (3)(オ)）: 要件と判断の記録の先が解けなければリンクを張らず「まだ分からない」を添える
/// （要件書に無い FR9・写しの adr/ に無い ADR-9・adr/ の無い写しの ADR-2）。面は導出できる（0）。
#[test]
fn f209_unresolved_successor_has_no_link_and_is_marked_unknown() {
    for (to, adr, dead) in [("FR9", false, "srs.html#fr9"), ("ADR-9", true, "adr-9.html"), ("ADR-2", false, "adr-2.html")] {
        let html = retired_face(&format!("f209-dead-{to}"), to, adr);
        let want = format!("廃止 → 後継 {to}（まだ分からない）");
        assert!(html.contains(&want), "{to}: 「{want}」が無い");
        assert!(!html.contains(dead) && !html.contains(&format!("note-{to}.html")), "{to}: 解けない先にリンクを張った");
    }
}
