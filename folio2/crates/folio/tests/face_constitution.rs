//! 憲法の面（`folio face --face constitution`）の歯（便 79・docs/design/delivery-79.md §1 (c)）。binary 経由。
//! - 数値の表の閾値行の値の枡: 閉じた表の日本語の小見出し（表に無い鍵は「まだ分からない」）・表でない値は素のまま
//! - 裁定の枡: 最新の 1 件だけを出し、過去の裁定は小窓「前の裁定 <N> 件」へ畳む（件数は正本を直に読んで数える）
//!
//! 実の置き場 design-intent/ と図の道具 vendor/archify/ を一時 dir へ写して面を書く（版管理の下の面は書き換えない）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use yaml_rust2::{Yaml, YamlLoader};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn design_intent() -> PathBuf {
    repo_root().join("design-intent")
}

fn vendor() -> PathBuf {
    repo_root().join("vendor/archify")
}

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!(
        "folio-face-constitution-{case}-{}",
        std::process::id()
    ));
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

/// 実の置き場を一時 dir の下の src/ へ、repo の vendor/archify/ を親 dir の vendor/archify/ へ写す。
/// 戻り値 = (一時 dir, 写し)。
fn real_copy(case: &str) -> (PathBuf, PathBuf) {
    let td = temp_dir(case);
    let work = td.join("src");
    copy_dir(&design_intent(), &work);
    copy_dir(&vendor(), &td.join("vendor/archify"));
    (td, work)
}

/// `folio face --face constitution --dir <dir> --out <out> --write`。
fn folio_face(dir: &Path, out: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("face")
        .arg("--face")
        .arg("constitution")
        .arg("--dir")
        .arg(dir)
        .arg("--out")
        .arg(out)
        .arg("--write")
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

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// 5 字の escape（生成側の字面を使わず歯の側で持つ）。
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// 写しの置き場に変異を当て（None なら当てない）、`--write` の結果と面の本文を返す（面が出来ていなければ本文は空）。
fn face_with(case: &str, mutate: Option<fn(&str) -> String>) -> (Output, String) {
    let (td, work) = real_copy(case);
    if let Some(f) = mutate {
        let path = work.join("rules.yaml");
        let before = fs::read_to_string(&path).unwrap();
        let after = f(&before);
        assert_ne!(before, after, "変異が当たっていない");
        fs::write(&path, after).unwrap();
    }
    let out = td.join("constitution.html");
    let run = folio_face(&work, &out);
    let html = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    (run, html)
}

/// 実の置き場の写しから面を組み、本文を返す。
fn real_html(case: &str) -> String {
    let (run, html) = face_with(case, None);
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    html
}

/// 行 `id` の欄 `k`（data-k）の枡の中身。
fn cell(html: &str, id: &str, k: &str) -> String {
    let row_at = html
        .find(&format!("data-k=\"id\">{id}</td>"))
        .unwrap_or_else(|| panic!("行 {id} が面に無い"));
    let row = &html[row_at..];
    let row = &row[..row.find("</tr>").expect("行の終わりが無い")];
    let open = format!("data-k=\"{k}\">");
    let start = row
        .find(&open)
        .unwrap_or_else(|| panic!("行 {id} に欄 {k} が無い"))
        + open.len();
    let body = &row[start..];
    body[..body.find("</td>").expect("枡の終わりが無い")].to_string()
}

/// 裁定の枡のうち小窓より前（直に見える字）。
fn visible(cell: &str) -> &str {
    cell.split("<span class=\"hint\">").next().unwrap()
}

/// 裁定の枡の小窓の（名札, 本体）。小窓が無ければ None。
fn fold(cell: &str) -> Option<(String, String)> {
    let btn = cell.split("<span class=\"hint-btn\">").nth(1)?;
    let label = btn[..btn.find("</span>").unwrap()].to_string();
    let body = cell.split("<span class=\"hint-body\">").nth(1)?;
    let body = body[..body.rfind("</span></span>").unwrap()].to_string();
    Some((label, body))
}

/// 正本 rules.yaml の行 `id` の（ruling, ruled_at）を直に読む。
fn source_ruling(id: &str) -> (String, String) {
    let text = fs::read_to_string(design_intent().join("rules.yaml")).unwrap();
    let doc: Yaml = YamlLoader::load_from_str(&text).unwrap().remove(0);
    for table in ["thresholds", "discipline"] {
        for row in doc[table].as_vec().unwrap() {
            if row["id"].as_str() == Some(id) {
                return (
                    row["ruling"].as_str().unwrap().to_string(),
                    row["ruled_at"].as_str().unwrap().to_string(),
                );
            }
        }
    }
    panic!("正本に行 {id} が無い")
}

/// 正本の裁定の欄の過去の裁定の件数（区切りの出現数）。
fn source_previous(id: &str) -> usize {
    source_ruling(id).0.matches("）・前の裁定 = ").count()
}

// ── (a) 値の枡の小見出し ──

#[test]
fn f79_rules_value_cell_labels_every_key() {
    let html = real_html("labels");
    let v = cell(&html, "R-16", "値");
    let mut at = 0;
    for (key, label) in [
        ("marks", "規範の印"),
        ("prohibition", "「禁止」の扱い"),
        ("word", "語"),
        ("clause_ends", "直後の字"),
        ("units", "単位"),
    ] {
        let head = format!("<b>{label}（{key}）</b>");
        let found = v[at..]
            .find(&head)
            .unwrap_or_else(|| panic!("R-16 の値の枡に「{head}」が順どおりに無い: {v}"));
        at += found + head.len();
    }
}

#[test]
fn f79_rules_value_cell_is_closed() {
    let (run, html) = face_with(
        "closed",
        Some(|t| t.replacen("units: [秒", "extras: 1, units: [秒", 1)),
    );
    assert_eq!(code(&run, "folio face"), 2, "{}", stderr(&run));
    let err = stderr(&run);
    assert!(err.contains("まだ分からない"), "{err}");
    assert!(err.contains("rules 行の値の表に無い鍵"), "{err}");
    assert!(err.contains("extras"), "{err}");
    assert!(html.is_empty(), "導出できないのに面を書いた");
}

#[test]
fn f79_rules_value_cell_leaves_scalars_alone() {
    let html = real_html("scalars");
    let v = cell(&html, "R-1", "値");
    assert!(!v.contains("<b>"), "表でない値に小見出しが付いた: {v}");
}

// ── (b) 裁定の枡の折りたたみ ──

#[test]
fn f79_rules_ruling_shows_only_the_latest() {
    let html = real_html("latest");
    let c = cell(&html, "R-14", "裁定");
    let seen = visible(&c);
    assert!(!seen.contains("前の裁定 = "), "過去の裁定が見えている: {seen}");
    assert!(seen.contains("2026-09-21 22:50 JST"), "{seen}");
    assert!(seen.contains("（2026-09-21）"), "{seen}");
}

#[test]
fn f79_rules_ruling_folds_the_previous_ones() {
    let html = real_html("folds");
    let (label, body) = fold(&cell(&html, "R-14", "裁定")).expect("R-14 に小窓が無い");
    assert_eq!(label, "前の裁定 2 件");
    assert_eq!(body.matches("前の裁定 = ").count(), 2, "{body}");
    for id in ["R-14", "R-1", "D-3", "R-2"] {
        let n = source_previous(id);
        assert!(n > 0, "正本の {id} に過去の裁定が無い");
        let (label, body) = fold(&cell(&html, id, "裁定"))
            .unwrap_or_else(|| panic!("{id} に小窓が無い"));
        assert_eq!(label, format!("前の裁定 {n} 件"), "{id}");
        assert_eq!(body.matches("前の裁定 = ").count(), n, "{id}");
    }
    assert_eq!(source_previous("R-1"), 4);
    assert_eq!(source_previous("D-3"), 3);
    assert_eq!(source_previous("R-2"), 2);
}

#[test]
fn f79_rules_ruling_without_previous_is_unchanged() {
    let html = real_html("unchanged");
    let (ruling, at) = source_ruling("D-1");
    assert_eq!(source_previous("D-1"), 0);
    assert_eq!(
        cell(&html, "D-1", "裁定"),
        format!("{}（{}）", esc(&ruling), esc(&at))
    );
}

// ── 便 80: 条の欠番の行と改訂来歴の小窓（docs/design/delivery-80.md §1 (c)） ──

/// 正本 design-intent/constitution.yaml を直に読む。
fn source_constitution() -> Yaml {
    let text = fs::read_to_string(design_intent().join("constitution.yaml")).unwrap();
    YamlLoader::load_from_str(&text).unwrap().remove(0)
}

/// 歯の側の数え: 条の id を接頭辞ごとに集め、1 から最大までの欠けを「接頭辞-数」で返す（接頭辞は初出の順）。
fn source_gaps() -> Vec<String> {
    let doc = source_constitution();
    let mut prefixes: Vec<String> = Vec::new();
    let mut numbers: Vec<Vec<u32>> = Vec::new();
    for a in doc["articles"].as_vec().unwrap() {
        let id = a["id"].as_str().unwrap();
        let cut = id.rfind('-').unwrap();
        let (p, n) = (&id[..cut], id[cut + 1..].parse::<u32>().unwrap());
        match prefixes.iter().position(|q| q == p) {
            Some(i) => numbers[i].push(n),
            None => {
                prefixes.push(p.to_string());
                numbers.push(vec![n]);
            }
        }
    }
    let mut gaps = Vec::new();
    for (p, ns) in prefixes.iter().zip(&numbers) {
        let max = *ns.iter().max().unwrap();
        gaps.extend((1..=max).filter(|k| !ns.contains(k)).map(|k| format!("{p}-{k}")));
    }
    gaps
}

/// 正本の amended_by の項の全部の（previous_text, rationale）。
fn source_amendments() -> Vec<(String, String)> {
    let doc = source_constitution();
    let mut out = Vec::new();
    for a in doc["articles"].as_vec().unwrap() {
        if let Some(items) = a["amended_by"].as_vec() {
            for am in items {
                out.push((
                    am["previous_text"].as_str().unwrap().to_string(),
                    am["rationale"].as_str().unwrap().to_string(),
                ));
            }
        }
    }
    out
}

/// 章 01（読み方）の本文（帯 s1 から帯 s2 の前まで）。
fn chapter_01(html: &str) -> &str {
    let start = html.find("id=\"s1\"").expect("章 01 が無い");
    let rest = &html[start..];
    &rest[..rest.find("id=\"s2\"").expect("章 02 が無い")]
}

/// 面の欠番の行の列。
fn face_gaps(html: &str) -> Vec<String> {
    let line = html.split("欠番: ").nth(1).expect("欠番の行が無い");
    line[..line.find('（').unwrap()]
        .split('・')
        .map(str::to_string)
        .collect()
}

/// 部品 principle-amendment-history の中の am-row の行の全部。
fn amendment_rows(html: &str) -> Vec<String> {
    let mut rows = Vec::new();
    for block in html
        .split("<div data-component=\"principle-amendment-history\">")
        .skip(1)
    {
        let block = &block[..block.find("</div>").unwrap()];
        rows.extend(
            block
                .split("<span class=\"am-row\">")
                .skip(1)
                .map(str::to_string),
        );
    }
    rows
}

/// 行の中の小窓 `label` の本体の全部。
fn hint_bodies(row: &str, label: &str) -> Vec<String> {
    let open = format!("<span class=\"hint-btn\">{label}</span></label><span class=\"hint-body\">");
    row.split(&open)
        .skip(1)
        .map(|b| b[..b.find("</span>").unwrap()].to_string())
        .collect()
}

#[test]
fn f80_constitution_shows_the_missing_number() {
    let html = real_html("gap-line");
    assert!(
        chapter_01(&html).contains("欠番: P-9"),
        "章 01 に欠番の行が無い"
    );
    assert_eq!(html.matches("欠番:").count(), 1, "欠番の行が 1 回でない");
    assert!(
        chapter_01(&html).contains("<a class=\"xref\" href=\"#p-7\">P-7</a>"),
        "欠番の行に P-7 へのリンクが無い"
    );
}

#[test]
fn f80_missing_numbers_match_an_independent_count() {
    let html = real_html("gap-count");
    let gaps = source_gaps();
    assert_eq!(gaps, vec!["P-9".to_string()], "正本の欠番の実測が変わった");
    assert_eq!(face_gaps(&html), gaps);
}

#[test]
fn f80_no_missing_line_when_the_numbers_are_dense() {
    let html = real_html("gap-prefix");
    let gaps = face_gaps(&html);
    assert!(!gaps.is_empty());
    for g in &gaps {
        assert!(
            !g.starts_with("A-") && !g.starts_with("N-"),
            "欠けの無い接頭辞に欠番が出た: {g}"
        );
    }
}

#[test]
fn f80_amendment_rows_carry_the_previous_text() {
    let html = real_html("am-count");
    let total = source_amendments().len();
    assert_eq!(total, 4, "正本の amended_by の項の実測が変わった");
    let rows = amendment_rows(&html);
    let amended: Vec<&String> = rows.iter().filter(|r| !r.contains("を置換（")).collect();
    assert_eq!(amended.len(), total, "amended_by の行の数が正本と違う");
    for r in &amended {
        assert_eq!(hint_bodies(r, "前の文").len(), 1, "{r}");
        assert_eq!(hint_bodies(r, "理由").len(), 1, "{r}");
    }
    let prev: usize = rows.iter().map(|r| hint_bodies(r, "前の文").len()).sum();
    let why: usize = rows.iter().map(|r| hint_bodies(r, "理由").len()).sum();
    assert_eq!((prev, why), (total, total));
}

#[test]
fn f80_amendment_previous_text_is_verbatim() {
    let html = real_html("am-verbatim");
    let rows = amendment_rows(&html);
    let mut face_prev: Vec<String> = rows.iter().flat_map(|r| hint_bodies(r, "前の文")).collect();
    let mut face_why: Vec<String> = rows.iter().flat_map(|r| hint_bodies(r, "理由")).collect();
    let (mut src_prev, mut src_why): (Vec<String>, Vec<String>) = source_amendments()
        .into_iter()
        .map(|(p, w)| (esc(&p), esc(&w)))
        .unzip();
    for v in [&mut face_prev, &mut face_why, &mut src_prev, &mut src_why] {
        v.sort();
    }
    assert_eq!(face_prev, src_prev);
    assert_eq!(face_why, src_why);
}

// ── 便 84: 機構の小窓のいつから動くかの意味と用語集の欄の名前の節（docs/design/delivery-84.md §1 (c)） ──

/// いつから動くかの（正本の値, 名札）5 つ（歯の側で持つ）。
const LIVE_LABELS: [(&str, &str); 5] = [
    ("now", "いま動く"),
    ("M0", "M0 で動く"),
    ("delivery-0", "便 0 で動く"),
    ("M1", "M1 で動く"),
    ("adr", "判断の記録の欄の決まりの後"),
];

/// 正本 design-intent/constitution.yaml の各条の機構の「いつから動くか」の値（重複を畳まない）。
fn source_live_values() -> Vec<String> {
    let text = fs::read_to_string(design_intent().join("constitution.yaml")).unwrap();
    let doc: Yaml = YamlLoader::load_from_str(&text).unwrap().remove(0);
    doc["articles"]
        .as_vec()
        .unwrap()
        .iter()
        .filter_map(|a| a["mechanism"]["live"].as_str().map(str::to_string))
        .collect()
}

/// 正本 design-intent/vocabulary.yaml の field_terms の（id, term）。
fn source_field_terms() -> Vec<(String, String)> {
    let text = fs::read_to_string(design_intent().join("vocabulary.yaml")).unwrap();
    let doc: Yaml = YamlLoader::load_from_str(&text).unwrap().remove(0);
    doc["field_terms"]
        .as_vec()
        .expect("field_terms が一覧でない")
        .iter()
        .map(|t| {
            (
                t["id"].as_str().unwrap().to_string(),
                t["term"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

/// 数えの字（歯の側で独立に組む: 9 までは「n つの語」・10 以上は「n の語」）。
fn count_words(n: usize) -> String {
    if n <= 9 {
        format!("{n} つの語")
    } else {
        format!("{n} の語")
    }
}

/// 章 07（用語集）の本文（帯 s7 から帯 s8 の前まで）。
fn chapter_07(html: &str) -> &str {
    let start = html.find("<section id=\"s7\"").expect("章 07 が無い");
    let rest = &html[start..];
    &rest[..rest.find("<section id=\"s8\"").expect("章 08 が無い")]
}

/// 章の中の部品 glossary-term-table の div の中身（開きの直後から次の「\n</div>」の手前まで）の全部。
fn glossary_tables(chapter: &str) -> Vec<&str> {
    let open = "<div data-component=\"glossary-term-table\">";
    chapter
        .split(open)
        .skip(1)
        .map(|b| &b[..b.find("\n</div>").expect("表の終わりが無い")])
        .collect()
}

#[test]
fn f84_mechanism_chip_explains_the_stage() {
    let html = real_html("f84-chip");
    let bodies = hint_bodies(&html, "機構");
    assert!(!bodies.is_empty(), "機構の小窓が無い");
    // 正本に在る値の名札だけを数える（値が消えても歯が釘付けにならない・便 43 / 44 と同じ向き）
    let live = source_live_values();
    assert!(!live.is_empty(), "正本の条に機構の live が無い");
    for (value, label) in LIVE_LABELS {
        if !live.iter().any(|v| v == value) {
            continue;
        }
        assert!(
            bodies.iter().any(|b| b.contains(&format!("{label}（"))),
            "正本に在る「{value}」の意味が機構の小窓に無い"
        );
    }
    let mut seen = 0;
    for b in &bodies {
        for (_, label) in LIVE_LABELS {
            for (at, _) in b.match_indices(label) {
                seen += 1;
                assert!(
                    b[at + label.len()..].starts_with('（'),
                    "名札「{label}」の直後が括弧でない: {b}"
                );
            }
        }
    }
    assert!(seen >= bodies.len(), "機構の小窓に名札が無いものがある");
}

#[test]
fn f84_glossary_has_the_field_terms_section() {
    let html = real_html("f84-fields");
    let ch = chapter_07(&html);
    let tables = glossary_tables(ch);
    assert_eq!(
        tables.len(),
        2,
        "章 07 の glossary-term-table が 2 つでない"
    );
    let second_at = ch
        .rfind("<div data-component=\"glossary-term-table\">")
        .unwrap();
    let h3_at = ch.find("<h3>欄の名前").expect("h3 の「欄の名前」が無い");
    assert!(
        h3_at < second_at,
        "h3 の「欄の名前」が 2 つ目の表の前に無い"
    );
    let fields = source_field_terms();
    assert_eq!(fields.len(), 7, "正本の field_terms の実測が変わった");
    for (_, term) in &fields {
        assert!(
            tables[1].contains(&format!("<div class=\"gword\">{}", esc(term))),
            "欄の名前の表に「{term}」が無い"
        );
    }
    assert_eq!(
        tables[1].matches("<div class=\"grow\"").count(),
        fields.len()
    );
    let id = &fields
        .iter()
        .find(|(_, t)| t == "規範文")
        .expect("正本に規範文が無い")
        .0;
    assert!(
        html.contains(&format!("id=\"g-{id}\"")),
        "規範文の行の id が無い"
    );
}

#[test]
fn f84_glossary_heading_counts_from_the_source() {
    let html = real_html("f84-count");
    let n = source_field_terms().len();
    let want = format!("<h3>欄の名前 — {}</h3>", count_words(n));
    assert_eq!(
        chapter_07(&html).matches(&want).count(),
        1,
        "{want} が章 07 に 1 回でない"
    );
}
