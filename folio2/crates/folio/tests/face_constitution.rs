//! 憲法の面（`folio face --face constitution`）の歯（便 79・docs/design/delivery-79.md §1 (c)）。binary 経由。
//! - 数値の表の閾値行の値の枡: 閉じた表の日本語の小見出し（表に無い鍵は「まだ分からない」）・表でない値は素のまま
//! - 裁定の枡: 最新の 1 件だけを出し、過去の裁定は小窓「前の裁定 <N> 件」へ畳む（件数は正本を直に読んで数える）
//! - 便 135: 本文の判断の記録の番号は正本の在る番号だけ adr-n.html へのリンク（逐語を比べる歯は包みを外して比べる）
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

/// 判断の記録の面へのリンクの包みを外す（便 135・歯の側の手書きの式）。`<a class="xref" href="adr-` で始まるリンクごとに、
/// 中の字が判断の記録の番号（ADR- に数字列）で行き先がその番号の面（adr-<数>.html）であることを確かめてから、字だけを残す。
fn unlink_adr(html: &str) -> String {
    const OPEN: &str = "<a class=\"xref\" href=\"adr-";
    let mut out = String::new();
    let mut rest = html;
    while let Some(at) = rest.find(OPEN) {
        out.push_str(&rest[..at]);
        let (n, tail) = rest[at + OPEN.len()..]
            .split_once(".html\">")
            .expect("リンクの行き先の閉じが無い");
        let (text, tail) = tail.split_once("</a>").expect("リンクの閉じが無い");
        assert!(
            !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()),
            "行き先が判断の記録の面でない: adr-{n}"
        );
        assert_eq!(text, format!("ADR-{n}"), "リンクの中の字が行き先の番号でない");
        out.push_str(text);
        rest = tail;
    }
    out.push_str(rest);
    out
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

/// 正本 rules.yaml の全部の行（閾値の表と規律の表・正本の順）の id。
fn source_row_ids() -> Vec<String> {
    let text = fs::read_to_string(design_intent().join("rules.yaml")).unwrap();
    let doc: Yaml = YamlLoader::load_from_str(&text).unwrap().remove(0);
    ["thresholds", "discipline"]
        .iter()
        .flat_map(|t| doc[*t].as_vec().unwrap().clone())
        .map(|row| row["id"].as_str().unwrap().to_string())
        .collect()
}

/// 正本の裁定の欄を（最新の 1 件, 過去の裁定の字の並び）に分ける（歯の側の数え・区切りは「）・前の裁定 = 」）。
fn source_split(id: &str) -> (String, Vec<String>) {
    let (ruling, _) = source_ruling(id);
    let mut pieces = ruling.split("）・前の裁定 = ").map(str::to_string);
    let latest = pieces.next().unwrap();
    (latest, pieces.collect())
}

/// 前の裁定を畳んだ行（裁定の欄に「前の裁定 = 」を持つ行）と持たない行に、正本の全行を分ける。
/// どちらの側も 1 行以上在ることを確かめる（片側が空なら、その側の性質を当てる歯が黙って空回りする）。
fn source_rows_by_previous() -> (Vec<String>, Vec<String>) {
    let (folded, plain): (Vec<String>, Vec<String>) =
        source_row_ids().into_iter().partition(|id| source_previous(id) > 0);
    assert!(!folded.is_empty(), "正本に前の裁定を持つ行が 1 つも無い");
    assert!(!plain.is_empty(), "正本に前の裁定を持たない行が 1 つも無い");
    (folded, plain)
}

#[test]
fn f79_rules_ruling_shows_only_the_latest() {
    // 前の裁定を持つ行の全部で、小窓より前に見える字は最新の裁定と ruled_at だけ（行の id と数は正本から読む）。
    let html = unlink_adr(&real_html("latest"));
    let (folded, _) = source_rows_by_previous();
    for id in &folded {
        let c = cell(&html, id, "裁定");
        let seen = visible(&c);
        assert!(!seen.contains("前の裁定 = "), "{id}: 過去の裁定が見えている: {seen}");
        let (latest, _) = source_split(id);
        let (_, at) = source_ruling(id);
        assert!(seen.starts_with(&esc(&latest)), "{id}: 最新の裁定で始まらない: {seen}");
        assert!(seen.contains(&format!("（{}）", esc(&at))), "{id}: ruled_at が無い: {seen}");
    }
}

#[test]
fn f79_rules_ruling_folds_the_previous_ones() {
    // 前の裁定を持つ行の全部で、小窓の名札の件数と本体の「前の裁定 = 」の数が、歯の側で正本を直に数えた数と一致し、
    // 過去の裁定の字が 1 件ずつ本体に在る（生成器の数えを写さない）。
    let html = unlink_adr(&real_html("folds"));
    let (folded, _) = source_rows_by_previous();
    for id in &folded {
        let n = source_previous(id);
        let (label, body) = fold(&cell(&html, id, "裁定"))
            .unwrap_or_else(|| panic!("{id} に小窓が無い"));
        assert_eq!(label, format!("前の裁定 {n} 件"), "{id}");
        assert_eq!(body.matches("前の裁定 = ").count(), n, "{id}: {body}");
        let (_, previous) = source_split(id);
        assert_eq!(previous.len(), n, "{id}");
        for p in &previous {
            assert!(body.contains(&esc(p)), "{id}: 小窓に過去の裁定「{p}」が無い: {body}");
        }
    }
}

#[test]
fn f79_rules_ruling_without_previous_is_unchanged() {
    // 前の裁定を持たない行の全部で、裁定の枡に小窓が無く、字面が「{ruling}（{ruled_at}）」のまま（歯の側で正本から組む）。
    let html = unlink_adr(&real_html("unchanged"));
    let (_, plain) = source_rows_by_previous();
    for id in &plain {
        let (ruling, at) = source_ruling(id);
        let c = cell(&html, id, "裁定");
        assert!(fold(&c).is_none(), "{id}: 前の裁定の無い行に小窓が在る: {c}");
        assert_eq!(c, format!("{}（{}）", esc(&ruling), esc(&at)), "{id}");
    }
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
    assert_eq!(total, 5, "正本の amended_by の項の実測が変わった");
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
    let html = unlink_adr(&real_html("am-verbatim"));
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

/// いつから動くかの（正本の値, 名札）5 つ（歯の側で持つ・now でない 4 値は 1 つの名札・便 132）。
const LIVE_LABELS: [(&str, &str); 5] = [
    ("now", "いま動く"),
    ("M0", "機構がまだ無い"),
    ("delivery-0", "機構がまだ無い"),
    ("M1", "機構がまだ無い"),
    ("adr", "機構がまだ無い"),
];

/// 機構の小窓の本文のうち名札の部分（機構の注「 — 」の前）。
fn chip_head(body: &str) -> &str {
    body.split(" — ").next().unwrap()
}

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
    // 名札の数え上げは機構の注の前だけ（注が「機構がまだ無い条」の字を持つ条が在る・便 132）
    let mut labels: Vec<&str> = LIVE_LABELS.iter().map(|(_, l)| *l).collect();
    labels.dedup();
    let mut seen = 0;
    for b in &bodies {
        let b = chip_head(b);
        for label in &labels {
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

// ── 便 132: 機構がまだ無い条の名札（docs/design/delivery-132.md §1 (c)） ──

#[test]
fn f132_articles_without_a_mechanism_say_so() {
    let html = real_html("f132-none");
    let heads: Vec<String> = hint_bodies(&html, "機構")
        .iter()
        .map(|b| chip_head(b).to_string())
        .collect();
    assert!(!heads.is_empty(), "機構の小窓が無い");
    // 本数は正本から数える（歯に数を書かない・便 43 / 44 と同じ向き）
    let live = source_live_values();
    let mut values: Vec<&String> = live.iter().filter(|v| *v != "now").collect();
    assert!(!values.is_empty(), "正本に live が now でない条が無い（前提が崩れた）");
    values.sort();
    values.dedup();
    for v in values {
        let want = format!("機構がまだ無い（床は判定しない・憲法が書く実在の予定は {v}）");
        let n = live.iter().filter(|w| *w == v).count();
        let got = heads.iter().filter(|h| h.contains(&want)).count();
        assert_eq!(got, n, "「{want}」の小窓の数が正本の {v} の条の数と違う");
    }
    for h in &heads {
        for old in [
            "まだ分からない",
            "M0 で動く",
            "便 0 で動く",
            "M1 で動く",
            "判断の記録の欄の決まりの後",
        ] {
            assert!(!h.contains(old), "機構の小窓の名札に「{old}」が在る: {h}");
        }
    }
}

// ── 便 141: 機構がまだ無い条の名札から「段」を外す（docs/design/delivery-141.md §1 (c)） ──

/// 機構がまだ無い条の名札（歯の側で手で写した字・値は正本の live の字のまま続ける）。
fn plan_chip(v: &str) -> String {
    format!("機構がまだ無い（床は判定しない・憲法が書く実在の予定は {v}）")
}

#[test]
fn f141_each_live_value_names_the_plan_in_the_chip() {
    // 実の置き場の写しの live: M1 の先頭 3 つを M0・delivery-0・adr に替え、4 つの値を 1 枚の面に並べる。
    let (td, work) = real_copy("f141-values");
    let path = work.join("constitution.yaml");
    let mut text = fs::read_to_string(&path).unwrap();
    let m1 = text.matches("live: M1,").count();
    assert!(m1 >= 4, "正本に live: M1 が 4 つ以上無い（前提が崩れた）: {m1}");
    for v in ["M0", "delivery-0", "adr"] {
        text = text.replacen("live: M1,", &format!("live: {v},"), 1);
    }
    fs::write(&path, &text).unwrap();
    let out = td.join("constitution.html");
    let run = folio_face(&work, &out);
    let html = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    let heads: Vec<String> = hint_bodies(&html, "機構")
        .iter()
        .map(|b| chip_head(b).to_string())
        .collect();
    assert!(!heads.is_empty(), "機構の小窓が無い");
    for (v, n) in [("M0", 1), ("delivery-0", 1), ("adr", 1), ("M1", m1 - 3)] {
        let want = plan_chip(v);
        let got = heads.iter().filter(|h| h.contains(&want)).count();
        assert_eq!(got, n, "「{want}」の小窓の数が写しの {v} の条の数と違う");
    }
    for h in &heads {
        assert!(!h.contains('段'), "機構の小窓の名札に「段」が在る: {h}");
    }
}

#[test]
fn f141_no_chip_head_names_the_tier() {
    let real = real_html("f141-tier");
    let anchor =
        fs::read_to_string(repo_root().join("tests/fixtures/face/expected.html")).unwrap();
    for (what, html) in [("実の憲法の面", &real), ("凍結 anchor", &anchor)] {
        assert!(!html.contains("憲法の段の値"), "{what} に「憲法の段の値」が在る");
        let bodies = hint_bodies(html, "機構");
        assert!(!bodies.is_empty(), "{what} に機構の小窓が無い");
        for b in &bodies {
            let h = chip_head(b);
            assert!(!h.contains('段'), "{what} の機構の小窓の名札に「段」が在る: {h}");
        }
    }
    for v in ["M0", "delivery-0"] {
        assert_eq!(
            anchor.matches(&plan_chip(v)).count(),
            1,
            "凍結 anchor に {v} の新しい名札が 1 つでない"
        );
    }
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

// ── 便 135: 本文の判断の記録の番号を判断の記録の面へのリンクに（docs/design/delivery-135.md §1 (c)）──

/// 面の本文（`<body>` から後）のうち `<svg>` の中を除いた字。
fn body_outside_svg(html: &str) -> String {
    let mut rest = &html[html.find("<body>").expect("body が無い")..];
    let mut out = String::new();
    while let Some(at) = rest.find("<svg") {
        out.push_str(&rest[..at]);
        let end = rest[at..].find("</svg>").expect("svg の閉じが無い");
        rest = &rest[at + end + "</svg>".len()..];
    }
    out.push_str(rest);
    out
}

/// 判断の記録の番号の出現（byte の位置・番号の数字列）。床の形（前が英数字でも「-」でもない ADR- に 1〜9 で始まる
/// 数字列が続き、後ろが英数字でない）を歯の側で手で写す。
fn adr_mentions(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (at, _) in text.match_indices("ADR-") {
        let before = text[..at].chars().next_back();
        if before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '-') {
            continue;
        }
        let n: String = text[at + 4..].chars().take_while(char::is_ascii_digit).collect();
        let after = text[at + 4 + n.len()..].chars().next();
        if n.is_empty() || n.starts_with('0') || after.is_some_and(|c| c.is_ascii_alphanumeric()) {
            continue;
        }
        out.push((at, n));
    }
    out
}

#[test]
fn f135_every_adr_mention_on_the_real_constitution_is_a_link() {
    let html = real_html("f135-adr");
    let body = body_outside_svg(&html);
    let mentions = adr_mentions(&body);
    assert!(!mentions.is_empty(), "判断の記録の番号が 1 つも無い");
    for (at, n) in &mentions {
        let open = format!("<a class=\"xref\" href=\"adr-{n}.html\">");
        let end = at + "ADR-".len() + n.len();
        assert!(
            body[..*at].ends_with(&open) && body[end..].starts_with("</a>"),
            "ADR-{n} がリンクでない: {}",
            &body[at.saturating_sub(120)..(end + 40).min(body.len())]
        );
        assert!(
            design_intent().join("adr").join(format!("ADR-{n}.yaml")).is_file(),
            "ADR-{n} の正本が無い"
        );
    }
    let mut depth = 0i32;
    for (at, _) in html.match_indices('<') {
        let t = &html[at..];
        if t.starts_with("<a ") || t.starts_with("<a>") {
            depth += 1;
            assert!(depth <= 1, "入れ子のリンクが在る");
        } else if t.starts_with("</a>") {
            depth -= 1;
        }
    }
    assert_eq!(depth, 0, "a の開きと閉じが揃わない");
    assert!(
        html.contains("<dt>判断の記録</dt><dd><a class=\"xref\" href=\"adr-11.html\">ADR-11</a></dd>"),
        "改訂の例の判断の記録の欄が ADR-11 へのリンクでない"
    );
}

#[test]
fn f135_number_without_a_page_stays_plain_on_the_constitution() {
    let (run, html) = face_with(
        "f135-plain",
        Some(|t| t.replacen("ruling: \"f2-648 notes", "ruling: \"ADR-99・ADR-11・f2-648 notes", 1)),
    );
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    let c = cell(&html, "R-1", "裁定");
    assert!(
        c.starts_with("ADR-99・<a class=\"xref\" href=\"adr-11.html\">ADR-11</a>・f2-648 notes"),
        "R-1 の裁定の枡が ADR-99 の字と ADR-11 のリンクで始まらない: {c}"
    );
    assert!(!html.contains("adr-99.html"), "正本の無い ADR-99 がリンクになった");
}

// ── 便 144: 表紙の状態に今の版の承認・承認欄に版ごとの承認の行（docs/design/delivery-144.md §1 (c) の 2〜5）──

/// 発効した判断の状態（欄の決まりの effective_status を歯の側で手で写す）。
const EFFECTIVE: [&str; 2] = ["accepted", "retired"];

/// 版を上げた発効した判断の承認 1 行（歯の側の手書きの読み・字は escape 済み）。
struct Row {
    id: String,
    version: String,
    who: String,
    date: String,
    verbatim: String,
    ruling: String,
    surface: String,
}

fn yaml_at(path: &Path) -> Yaml {
    YamlLoader::load_from_str(&fs::read_to_string(path).unwrap())
        .unwrap()
        .remove(0)
}

fn s<'a>(y: &'a Yaml, key: &str) -> &'a str {
    y[key]
        .as_str()
        .unwrap_or_else(|| panic!("欄 {key} が文字列でない: {y:?}"))
}

/// 置き場 `dir` の adr/ の ADR-<数>.yaml を番号の順に直に読み、発効した判断（状態が EFFECTIVE で承認欄が表）の amends の
/// 各項の版を出た順に重ねずに 1 行ずつ返す。
fn source_rows(dir: &Path) -> Vec<Row> {
    let mut files: Vec<(u64, PathBuf)> = fs::read_dir(dir.join("adr"))
        .unwrap()
        .filter_map(|e| {
            let path = e.unwrap().path();
            let name = path.file_name()?.to_str()?.to_string();
            let n = name.strip_prefix("ADR-")?.strip_suffix(".yaml")?;
            if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            Some((n.parse().ok()?, path))
        })
        .collect();
    files.sort();
    let mut rows = Vec::new();
    for (_, path) in files {
        let y = yaml_at(&path);
        let ap = &y["approval"];
        if !EFFECTIVE.contains(&s(&y, "status")) || ap.as_hash().is_none() {
            continue;
        }
        let mut versions: Vec<String> = Vec::new();
        for item in y["amends"].as_vec().into_iter().flatten() {
            if item.as_hash().is_none() {
                continue;
            }
            let v = esc(s(item, "version"));
            if !versions.contains(&v) {
                versions.push(v);
            }
        }
        for version in versions {
            rows.push(Row {
                id: esc(s(&y, "id")),
                version,
                who: esc(s(ap, "who")),
                date: esc(s(ap, "date")),
                verbatim: esc(s(ap, "verbatim")),
                ruling: esc(s(ap, "ruling")),
                surface: esc(s(ap, "surface")),
            });
        }
    }
    rows
}

/// 置き場 `dir` の憲法の（版・生成日・初回の承認の日付）。
fn source_meta(dir: &Path) -> (String, String, String) {
    let c = yaml_at(&dir.join("constitution.yaml"));
    let m = &c["meta"];
    (
        esc(s(m, "version")),
        esc(s(m, "generated")),
        esc(s(&m["approval"], "date")),
    )
}

/// 版ごとの承認の行の字（§1 (b) の 2 の 5 つの枡を歯の側で手で写す）。
fn sign_of(r: &Row) -> String {
    format!(
        "<div class=\"sign\"><span class=\"role\">承認</span><span class=\"who\">{}</span><span class=\"when\">{} · 版 {} · 逐語「{}」</span><span class=\"when\">判断の記録: {}／裁定: {}／対話面: {}</span><span class=\"stamp\">発効</span></div>",
        r.who, r.date, r.version, r.verbatim, r.id, r.ruling, r.surface
    )
}

/// 承認欄の sign の行（出た順）。
fn signs(html: &str) -> Vec<&str> {
    let at = html
        .find("<section id=\"approval\"")
        .expect("承認欄が無い");
    let rest = &html[at..];
    let block = rest
        .find("\"approval-block\"")
        .expect("承認欄の部品が無い");
    rest[block..]
        .lines()
        .skip(1)
        .take_while(|l| l.starts_with("<div class=\"sign\">"))
        .collect()
}

/// 写しの置き場から面を組み（code 0 を確かめる）、判断の記録のリンクの包みを外した本文を返す。
fn face_of(case: &str, td: &Path, work: &Path) -> String {
    let out = td.join("constitution.html");
    let run = folio_face(work, &out);
    assert_eq!(code(&run, "folio face --write"), 0, "{case}: {}", stderr(&run));
    unlink_adr(&fs::read_to_string(&out).unwrap())
}

/// 鮮度の札・表紙の状態・承認欄のリードの字。
fn stamp_of(generated: &str, version: &str, label: &str) -> String {
    format!("生成 <b>{generated}</b> · <b>{version}</b>（{label}）</span>")
}

fn cover_of(state: &str) -> String {
    format!(
        "<p class=\"cover-status\"><span class=\"k\">状態</span><span>{state}（<a href=\"#approval\">承認欄へ</a>）</span></p>"
    )
}

fn lead_of(lead: &str) -> String {
    format!("<h2>作成 / 承認</h2>\n<p class=\"lead\">{lead}</p>")
}

fn once(html: &str, want: &str) {
    assert_eq!(html.matches(want).count(), 1, "「{want}」がちょうど 1 つでない");
}

/// 写しの判断の記録 `ids` の状態を提案中に戻す。
fn to_proposed(work: &Path, ids: &[String]) {
    for id in ids {
        let path = work.join("adr").join(format!("{id}.yaml"));
        let before = fs::read_to_string(&path).unwrap();
        let after = before.replacen("\nstatus: accepted\n", "\nstatus: proposed\n", 1);
        assert_ne!(before, after, "{id} を提案中に戻せていない");
        fs::write(&path, after).unwrap();
    }
}

const EFFECTIVE_LABEL: &str = "発効・拘束力あり";
const UNKNOWN: &str = "効く版はまだ分からない";

#[test]
fn f144_cover_names_the_approval_of_the_current_version() {
    let (td, work) = real_copy("f144-cover");
    let (version, generated, first) = source_meta(&work);
    let rows = source_rows(&work);
    let naming: Vec<&Row> = rows.iter().filter(|r| r.version == version).collect();
    assert!(!naming.is_empty(), "今の版 {version} を名指す発効した判断が無い");
    let date = naming.iter().map(|r| r.date.clone()).max().unwrap();
    let mut ids: Vec<String> = Vec::new();
    for r in &naming {
        if !ids.contains(&r.id) {
            ids.push(r.id.clone());
        }
    }
    let html = face_of("f144-cover", &td, &work);
    let _ = fs::remove_dir_all(&td);
    once(
        &html,
        &cover_of(&format!(
            "{EFFECTIVE_LABEL}（承認 {date}・判断の記録 {}）",
            ids.join("・")
        )),
    );
    assert_ne!(date, first, "今の版の承認が初回と同じ日で歯が見分けられない");
    assert!(
        !html.contains(&format!("{EFFECTIVE_LABEL}（承認 {first}）")),
        "表紙の状態が初回の承認の日付を出す"
    );
    once(&html, &stamp_of(&generated, &version, EFFECTIVE_LABEL));
    once(&html, &lead_of(EFFECTIVE_LABEL));
}

#[test]
fn f144_approval_lists_every_amending_version() {
    let (td, work) = real_copy("f144-rows");
    let (_, generated, first) = source_meta(&work);
    let rows = source_rows(&work);
    assert!(!rows.is_empty(), "版を上げた発効した判断が無い");
    let html = face_of("f144-rows", &td, &work);
    let _ = fs::remove_dir_all(&td);
    let got = signs(&html);
    assert_eq!(got.len(), 2 + rows.len(), "{got:#?}");
    assert!(
        got[0].starts_with("<div class=\"sign\"><span class=\"role\">作成</span>")
            && got[0].contains(&format!("<span class=\"when\">{generated}</span>")),
        "{}",
        got[0]
    );
    assert!(
        got[1].starts_with("<div class=\"sign\"><span class=\"role\">承認</span>")
            && got[1].contains(&format!("<span class=\"when\">{first} · 逐語「")),
        "{}",
        got[1]
    );
    for (i, r) in rows.iter().enumerate() {
        assert_eq!(got[2 + i], sign_of(r), "版ごとの承認の行 {} 本目", i + 1);
    }
}

#[test]
fn f144_unknown_when_no_decision_names_the_version() {
    // 版の欄を先に進めた写し
    let (td, work) = real_copy("f144-unknown-next");
    let (version, generated, first) = source_meta(&work);
    let rows = source_rows(&work);
    let path = work.join("constitution.yaml");
    let before = fs::read_to_string(&path).unwrap();
    let after = before.replacen(
        &format!("\n  version: {version}\n"),
        &format!("\n  version: {version}-next\n"),
        1,
    );
    assert_ne!(before, after, "版の欄を進められていない");
    fs::write(&path, after).unwrap();
    let html = face_of("f144-unknown-next", &td, &work);
    let _ = fs::remove_dir_all(&td);
    let latest = rows
        .iter()
        .map(|r| r.date.clone())
        .chain([first.clone()])
        .max()
        .unwrap();
    let next = format!("{version}-next");
    once(&html, &stamp_of(&generated, &next, UNKNOWN));
    once(
        &html,
        &cover_of(&format!("{EFFECTIVE_LABEL}（承認 {latest}）・{UNKNOWN}")),
    );
    once(&html, &lead_of(&format!("{EFFECTIVE_LABEL}（{UNKNOWN}）")));
    assert!(
        !html.contains(&format!("<b>{next}</b>（{EFFECTIVE_LABEL}）")),
        "今の版を発効と出す札が在る"
    );
    assert_eq!(signs(&html).len(), 2 + rows.len());

    // 今の版を名指す判断を提案中に戻した写し
    let (td, work) = real_copy("f144-unknown-proposed");
    let naming: Vec<String> = rows
        .iter()
        .filter(|r| r.version == version)
        .map(|r| r.id.clone())
        .collect();
    assert!(!naming.is_empty(), "今の版を名指す発効した判断が無い");
    to_proposed(&work, &naming);
    let left = source_rows(&work);
    let html = face_of("f144-unknown-proposed", &td, &work);
    let _ = fs::remove_dir_all(&td);
    let latest = left
        .iter()
        .map(|r| r.date.clone())
        .chain([first.clone()])
        .max()
        .unwrap();
    once(&html, &stamp_of(&generated, &version, UNKNOWN));
    once(
        &html,
        &cover_of(&format!("{EFFECTIVE_LABEL}（承認 {latest}）・{UNKNOWN}")),
    );
    once(&html, &lead_of(&format!("{EFFECTIVE_LABEL}（{UNKNOWN}）")));
    assert!(
        !html.contains(&format!("<b>{version}</b>（{EFFECTIVE_LABEL}）")),
        "今の版を発効と出す札が在る"
    );
    let got = signs(&html);
    assert_eq!(got.len(), 2 + left.len(), "{got:#?}");
    for id in &naming {
        assert!(
            !got.iter().any(|l| l.contains(&format!("判断の記録: {id}／"))),
            "提案中に戻した {id} の承認の行が在る"
        );
    }
}

#[test]
fn f144_without_amending_decisions_the_first_approval_stays() {
    // 版を上げた判断を全部提案中に戻した写し
    let (td, work) = real_copy("f144-first");
    let (version, generated, first) = source_meta(&work);
    let mut ids: Vec<String> = source_rows(&work).into_iter().map(|r| r.id).collect();
    ids.dedup();
    to_proposed(&work, &ids);
    assert!(source_rows(&work).is_empty(), "版を上げた発効した判断が残る");
    let html = face_of("f144-first", &td, &work);
    let _ = fs::remove_dir_all(&td);
    once(&html, &cover_of(&format!("{EFFECTIVE_LABEL}（承認 {first}）")));
    once(&html, &stamp_of(&generated, &version, EFFECTIVE_LABEL));
    once(&html, &lead_of(EFFECTIVE_LABEL));
    assert_eq!(signs(&html).len(), 2);

    // adr/ の無い面の fixture の写し
    let td = temp_dir("f144-no-adr");
    let work = td.join("src");
    fs::create_dir_all(&work).unwrap();
    let fixture = repo_root().join("tests/fixtures/face");
    for name in [
        "constitution.yaml",
        "rules.yaml",
        "vocabulary.yaml",
        "srs.yaml",
        "ceiling.yaml",
    ] {
        fs::copy(fixture.join(name), work.join(name)).unwrap();
    }
    assert!(!work.join("adr").exists());
    let html = face_of("f144-no-adr", &td, &work);
    let _ = fs::remove_dir_all(&td);
    assert_eq!(signs(&html).len(), 2);
}
