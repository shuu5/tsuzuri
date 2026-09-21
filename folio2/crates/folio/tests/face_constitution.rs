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
    assert_eq!(source_previous("R-1"), 3);
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
