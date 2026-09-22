//! 要件書の面（`folio face --face srs`）の歯の置き場（便 81・docs/design/delivery-81.md §1 (d)）。binary 経由。
//! 入口・判断の記録・設計ノートの 3 面と同じ 1 面 1 file の形。既存の要件書の面の歯は `face.rs` に残す。
//! - 図の段の名札が読める字（図 2 の n 段目）で、数字が 2 つ続く旧字面が 0 回・対応表の短い字も同じ
//! - 正本から数えた参照の段が全部面に在る（yaml-rust2 で正本を直に読む・生成器の字を写さない）
//! - 受入基準の章に英字の欄名の字面が 0 回で、日本語の名札が行の数だけ
//! - 承認欄の lead が status_note の要旨（最初の「。」まで）だけで、来歴は折りたたみの中に逐語で在る
//!
//! 版管理の下の面は書き換えない（`--out` は必ず一時 dir の中）。

use std::collections::BTreeSet;
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
    let td = std::env::temp_dir().join(format!("folio-face-srs-{case}-{}", std::process::id()));
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

fn code(out: &Output, what: &str) -> i32 {
    out.status.code().unwrap_or_else(|| {
        panic!(
            "{what} が signal で終わった: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

/// 実の置き場 design-intent/ を一時 dir の下の src/ へ、図の道具 vendor/archify/ を親 dir の vendor/archify/ へ写し、
/// 要件書の面を一時 file へ書く。面の本文を返す（一時 dir は消す）。
fn real_srs(case: &str) -> String {
    let td = temp_dir(case);
    let work = td.join("src");
    copy_dir(&design_intent(), &work);
    copy_dir(&vendor(), &td.join("vendor/archify"));
    let out = td.join("srs.html");
    let run = Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("face")
        .arg("--face")
        .arg("srs")
        .arg("--dir")
        .arg(&work)
        .arg("--out")
        .arg(&out)
        .arg("--write")
        .output()
        .expect("folio を起動できない");
    let html = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        code(&run, "folio face --face srs --write"),
        0,
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    html
}

fn srs() -> Yaml {
    let text = fs::read_to_string(design_intent().join("srs.yaml")).unwrap();
    YamlLoader::load_from_str(&text)
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
}

/// 5 字の escape（生成側の字面を使わず歯の側で持つ）。
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// `open` から次の `close` の手前までの字面（`open` を含む）。
fn span<'a>(html: &'a str, open: &str, close: &str) -> &'a str {
    let start = html
        .find(open)
        .unwrap_or_else(|| panic!("「{open}」が無い"));
    let end = html[start..]
        .find(close)
        .unwrap_or_else(|| panic!("「{open}」の後に「{close}」が無い"));
    &html[start..start + end]
}

/// 旧字面（図 2 {n}・数字が 2 つ続く形）の数。n は 1〜rail の段の数。
fn old_labels(text: &str, steps: usize) -> Vec<String> {
    (1..=steps)
        .map(|n| format!("図 2 {n}"))
        .filter(|old| text.contains(old.as_str()))
        .collect()
}

fn rail_len(s: &Yaml) -> usize {
    s["rail"].as_vec().expect("rail が一覧でない").len()
}

/// status_note を最初の「。」で要旨（「。」を含む）と来歴に分ける。
fn split_note(s: &Yaml) -> (String, String) {
    let note = s["meta"]["status_note"]
        .as_str()
        .expect("meta.status_note が無い");
    let cut = note.find('。').expect("status_note に「。」が無い") + '。'.len_utf8();
    (note[..cut].to_string(), note[cut..].to_string())
}

#[test]
fn f81_figure_label_reads_as_a_step() {
    let html = real_srs("f81-label");
    let s = srs();
    assert_eq!(rail_len(&s), 7, "rail の段の数（実測 7）");
    assert!(html.contains("図 2 の "), "読める字の段の名札が無い");
    let old = old_labels(&html, rail_len(&s));
    assert!(old.is_empty(), "旧字面が残っている: {old:?}");
}

#[test]
fn f81_every_referenced_step_has_a_label() {
    let html = real_srs("f81-every");
    let s = srs();
    let mut steps = BTreeSet::new();
    for key in ["requirements", "nonfunctional"] {
        for it in s[key].as_vec().unwrap_or_else(|| panic!("{key} が一覧でない")) {
            for f in it["figures"].as_vec().expect("figures が一覧でない") {
                let f = f.as_str().expect("figures の値が文字列でない");
                if let Some(n) = f.strip_prefix("図2-") {
                    steps.insert(n.parse::<usize>().expect("段の番号が数でない"));
                }
            }
        }
    }
    assert!(!steps.is_empty(), "正本に段の参照が無い");
    let rail: BTreeSet<usize> = (1..=rail_len(&s)).collect();
    assert!(steps.is_subset(&rail), "rail に無い段: {steps:?}");
    for n in &steps {
        assert!(
            html.contains(&format!("図 2 の {n} 段目")),
            "段 {n} の名札が面に無い"
        );
    }
}

#[test]
fn f81_rtm_cell_uses_the_short_label() {
    let html = real_srs("f81-rtm");
    let s = srs();
    let rtm = span(&html, "data-component=\"rtm-grid\"", "</table>");
    assert!(rtm.contains("図 2 の "), "対応表の枡に読める字の名札が無い");
    let old = old_labels(rtm, rail_len(&s));
    assert!(old.is_empty(), "対応表に旧字面が残っている: {old:?}");
}

#[test]
fn f81_acceptance_has_no_english_field_name() {
    let html = real_srs("f81-ac");
    let s = srs();
    let rows = s["acceptance"]
        .as_vec()
        .expect("acceptance が一覧でない")
        .len();
    let ac = span(&html, "<section id=\"s5\"", "<section id=\"s6\"");
    assert!(ac.contains("受入基準"), "章 05 が受入基準でない");
    assert_eq!(ac.matches("／fixture: ").count(), 0, "英字の欄名が残っている");
    assert_eq!(
        ac.matches("／固定の材料: ").count(),
        rows,
        "日本語の名札の数が acceptance の行の数と違う"
    );
}

#[test]
fn f81_approval_lead_is_only_the_summary() {
    let html = real_srs("f81-lead");
    let (gist, _) = split_note(&srs());
    let band = span(&html, "<section id=\"approval\"", "</section>");
    let lead = span(band, "<p class=\"lead\">", "</p>")
        .strip_prefix("<p class=\"lead\">")
        .unwrap();
    let (status, rest) = lead
        .split_once(" — ")
        .unwrap_or_else(|| panic!("lead に「 — 」が無い: {lead}"));
    assert!(!status.is_empty(), "lead に状態の名札が無い");
    assert_eq!(rest, esc(&gist), "lead が要旨だけでない");
    assert!(!lead.contains("v1.0 = "), "lead に来歴が在る");
}

#[test]
fn f81_approval_history_is_folded() {
    let html = real_srs("f81-fold");
    let (_, history) = split_note(&srs());
    assert!(!history.is_empty(), "正本の来歴が空");
    let tail = &html[html.find("<section id=\"approval\"").expect("承認欄が無い")..];
    let body = span(tail, "<div class=\"chapbody\">", "data-component=\"approval-block\"");
    let open = "<details class=\"note\"><summary>版ごとの来歴</summary><div><p>";
    assert_eq!(body.matches(open).count(), 1, "来歴の折りたたみが 1 つでない");
    let inner = span(body, open, "</p></div></details>")
        .strip_prefix(open)
        .unwrap();
    assert_eq!(inner, esc(&history), "折りたたみの中が来歴の逐語でない");
}

// ── 便 84: 用語集の欄の名前の節（docs/design/delivery-84.md §1 (c)） ──

/// `folio face --face <face> --dir <dir> --out <out> --write` の結果と面の本文。
fn write_face(face: &str, dir: &Path, out: &Path) -> (Output, String) {
    let run = Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("face")
        .arg("--face")
        .arg(face)
        .arg("--dir")
        .arg(dir)
        .arg("--out")
        .arg(out)
        .arg("--write")
        .output()
        .expect("folio を起動できない");
    (run, fs::read_to_string(out).unwrap_or_default())
}

/// 実の置き場の写しで憲法の面を書き、本文を返す。
fn real_constitution(case: &str) -> String {
    let td = temp_dir(case);
    let work = td.join("src");
    copy_dir(&design_intent(), &work);
    copy_dir(&vendor(), &td.join("vendor/archify"));
    let (run, html) = write_face("constitution", &work, &td.join("constitution.html"));
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        code(&run, "folio face --face constitution --write"),
        0,
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    html
}

/// 章 `n`（帯 s<n> から次の section の前まで）。
fn chapter(html: &str, n: usize) -> &str {
    let start = html
        .find(&format!("<section id=\"s{n}\""))
        .unwrap_or_else(|| panic!("章 {n} が無い"));
    let rest = &html[start..];
    let end = rest[1..].find("<section ").map_or(rest.len(), |e| e + 1);
    &rest[..end]
}

/// 章の中の部品 glossary-term-table の div の中身の全部。
fn glossary_tables(chapter: &str) -> Vec<&str> {
    let open = "<div data-component=\"glossary-term-table\">";
    chapter
        .split(open)
        .skip(1)
        .map(|b| &b[..b.find("\n</div>").expect("表の終わりが無い")])
        .collect()
}

#[test]
fn f84_srs_glossary_has_the_same_field_terms_section() {
    let srs = real_srs("f84-srs");
    let constitution = real_constitution("f84-srs-c");
    let s = glossary_tables(chapter(&srs, 8));
    let c = glossary_tables(chapter(&constitution, 7));
    assert_eq!(s.len(), 2, "章 08 の glossary-term-table が 2 つでない");
    assert_eq!(
        c.len(),
        2,
        "憲法の面の章 07 の glossary-term-table が 2 つでない"
    );
    assert!(
        s[1].contains("<div class=\"grow\""),
        "欄の名前の表に行が無い"
    );
    assert_eq!(s[1], c[1], "欄の名前の表が 2 面で byte 一致しない");
    let h3 = |ch: &str| span(ch, "<h3>欄の名前", "</h3>").to_string();
    assert_eq!(h3(chapter(&srs, 8)), h3(chapter(&constitution, 7)));
}

#[test]
fn f84_missing_field_terms_leaves_the_face_unchanged() {
    let fixture = repo_root().join("tests/fixtures/face");
    let td = temp_dir("f84-missing");
    let work = td.join("src");
    fs::create_dir_all(&work).unwrap();
    for name in [
        "constitution.yaml",
        "rules.yaml",
        "srs.yaml",
        "ceiling.yaml",
    ] {
        fs::copy(fixture.join(name), work.join(name)).unwrap();
    }
    let vocab = fs::read_to_string(fixture.join("vocabulary.yaml")).unwrap();
    let cut = vocab
        .find("\nfield_terms:\n")
        .expect("写しに field_terms の節が無い");
    fs::write(work.join("vocabulary.yaml"), &vocab[..cut + 1]).unwrap();
    copy_dir(&vendor(), &td.join("vendor/archify"));
    let (run_c, c) = write_face("constitution", &work, &td.join("constitution.html"));
    let (run_s, s) = write_face("srs", &work, &td.join("srs.html"));
    let _ = fs::remove_dir_all(&td);
    for (face, run, html) in [("constitution", &run_c, &c), ("srs", &run_s, &s)] {
        assert_eq!(
            code(run, face),
            0,
            "{face}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert!(
            !html.contains("<h3>欄の名前"),
            "{face} に欄の名前の節が在る"
        );
        assert_eq!(
            html.matches("data-component=\"glossary-term-table\"")
                .count(),
            1,
            "{face} の glossary-term-table が 1 つでない"
        );
    }
    let s1 = span(&c, "<section id=\"s1\"", "</section>");
    assert!(!s1.contains("<p class=\"lead\">"), "章 01 の帯に副題が在る");
}

// ── 便 100: 章 03〜06 を face_srs_items.rs へ切り出す（docs/design/delivery-100.md §1 (f)）──

/// 器の行数の式: 空行を含む全行を数え、字数（Unicode の字の数）が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す。
fn cap_lines(text: &str) -> usize {
    text.lines()
        .map(|l| {
            let n = l.chars().count();
            if n > 120 { n.div_ceil(120) } else { 1 }
        })
        .sum()
}

#[test]
fn f100_face_srs_is_split_and_under_the_cap() {
    let src = repo_root().join("crates/folio/src");
    let read = |name: &str| {
        fs::read_to_string(src.join(name)).unwrap_or_else(|e| panic!("{name} が読めない: {e}"))
    };
    let srs = read("face_srs.rs");
    let items = read("face_srs_items.rs");
    let (n_srs, n_items) = (cap_lines(&srs), cap_lines(&items));
    assert!(n_srs <= 1100, "face_srs.rs が器の式で {n_srs} 行（上限 1100）");
    assert!(n_items <= 600, "face_srs_items.rs が器の式で {n_items} 行（上限 600）");
    for head in [
        "fn legend_line(",
        "fn fr_chapter(",
        "fn nfr_chapter(",
        "fn item_row(",
        "fn ac_legend_line(",
        "fn ac_chapter(",
        "fn con_chapter(",
    ] {
        assert_eq!(
            items.matches(head).count(),
            1,
            "face_srs_items.rs の「{head}」が 1 つでない"
        );
        assert_eq!(
            srs.matches(head).count(),
            0,
            "face_srs.rs に「{head}」が残っている"
        );
    }
}
