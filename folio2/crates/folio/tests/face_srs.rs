//! 要件書の面（`folio face --face srs`）の歯の置き場（便 81・docs/design/delivery-81.md §1 (d)）。binary 経由。
//! 入口・判断の記録・設計ノートの 3 面と同じ 1 面 1 file の形。既存の要件書の面の歯は `face.rs` に残す。
//! - 図の段の名札が読める字（図 2 の n 段目）で、数字が 2 つ続く旧字面が 0 回・対応表の短い字も同じ
//! - 正本から数えた参照の段が全部面に在る（yaml-rust2 で正本を直に読む・生成器の字を写さない）
//! - 受入基準の章に英字の欄名の字面が 0 回で、日本語の名札が行の数だけ
//! - 承認欄の lead が status_note の要旨（最初の「。」まで）だけで、来歴は折りたたみの中に逐語で在る
//! - 便 118: M3 の範囲の節 scope_m3 は、在れば章 02 の段の範囲の塊を M1 の直後に 1 つ足し（名札 M3・注の小窓も同じ形）、
//!   無い・null なら面は凍結 fixture と byte 一致する（scope と scope_m1 は必須のまま・docs/design/delivery-118.md §1 (d)）
//! - 便 135: 本文の判断の記録の番号は正本の在る番号だけ adr-n.html へのリンク・範囲の節の番号は図の根拠と同じ行き先へ
//!   （docs/design/delivery-135.md §1 (c)）
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
    assert_eq!(unlink_adr(inner), esc(&history), "折りたたみの中が来歴の逐語でない");
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

// ── 便 118: M3 の範囲の節 scope_m3 を段の範囲の塊に描く（docs/design/delivery-118.md §1 (d)）──

/// 見本の scope_m3 の字（build・not_build・note）。escape を見るため「&」「<」「>」を含む。
const M3_BUILD: &str = "束 & 門の生成";
const M3_NOT_BUILD: &str = "他の repo の設計文書の移送";
const M3_NOTE: &str = "版 B で <中身> を決める。";
/// 段の範囲の塊（部品 section-lead-callout）の開き。
const CALLOUT: &str = "<div data-component=\"section-lead-callout\" style=\"--band-n:2\">";

/// 手書きの scope_m3 の節（値は全部二重引用符付き）。
fn m3_section() -> String {
    format!(
        "scope_m3:\n  build:\n    - \"{M3_BUILD}\"\n  not_build:\n    - \"{M3_NOT_BUILD}\"\n  note: \"{M3_NOTE}\"\n\n"
    )
}

/// 面の fixture の 5 file を一時 dir の下の src/ へ写し（要件書だけ `edit` で書き換える）、要件書の面を書く。
/// 結果・面の本文・面の file が書かれたかを返す（一時 dir は消す）。
fn fixture_srs(case: &str, edit: impl FnOnce(String) -> String) -> (Output, String, bool) {
    fixture_srs_with(case, &[], edit)
}

/// `fixture_srs` と同じ・面の fixture の判断の記録 `adrs`（adr/ の下の file 名）も写しの adr/ へ添える（便 135）。
fn fixture_srs_with(
    case: &str,
    adrs: &[&str],
    edit: impl FnOnce(String) -> String,
) -> (Output, String, bool) {
    let fixture = repo_root().join("tests/fixtures/face");
    let td = temp_dir(case);
    let work = td.join("src");
    fs::create_dir_all(&work).unwrap();
    if !adrs.is_empty() {
        fs::create_dir_all(work.join("adr")).unwrap();
    }
    for name in adrs {
        fs::copy(fixture.join("adr").join(name), work.join("adr").join(name)).unwrap();
    }
    for name in [
        "constitution.yaml",
        "rules.yaml",
        "vocabulary.yaml",
        "ceiling.yaml",
    ] {
        fs::copy(fixture.join(name), work.join(name)).unwrap();
    }
    let srs = fs::read_to_string(fixture.join("srs.yaml")).unwrap();
    fs::write(work.join("srs.yaml"), edit(srs)).unwrap();
    copy_dir(&vendor(), &td.join("vendor/archify"));
    let out = td.join("srs.html");
    let (run, html) = write_face("srs", &work, &out);
    let written = out.exists();
    let _ = fs::remove_dir_all(&td);
    (run, html, written)
}

/// 最上位の節 actors の直前に `section` を足す。
fn add_before_actors(srs: &str, section: &str) -> String {
    let at = srs.find("\nactors:\n").expect("写しに actors の節が無い") + 1;
    format!("{}{section}{}", &srs[..at], &srs[at..])
}

/// 最上位の節 `key` を（次の最上位の鍵の手前まで）落とす。
fn drop_section(srs: &str, key: &str) -> String {
    let start = srs
        .find(&format!("\n{key}:\n"))
        .unwrap_or_else(|| panic!("写しに {key} の節が無い"))
        + 1;
    let rest = &srs[start..];
    let mut end = rest.find('\n').unwrap() + 1;
    for line in rest[end..].split_inclusive('\n') {
        if !line.trim().is_empty() && !line.starts_with(' ') {
            break;
        }
        end += line.len();
    }
    format!("{}{}", &srs[..start], &rest[end..])
}

/// 章 02 の段の範囲の塊（開きから次の行頭の閉じの div まで・閉じを含む）の全部。
fn callouts(html: &str) -> Vec<&str> {
    let ch = chapter(html, 2);
    ch.match_indices(CALLOUT)
        .map(|(i, _)| {
            let end = ch[i..].find("\n</div>\n").expect("塊の閉じが無い") + "\n</div>\n".len();
            &ch[i..i + end]
        })
        .collect()
}

fn ok(run: &Output) {
    assert_eq!(
        code(run, "folio face --face srs --write"),
        0,
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
}

fn frozen_srs() -> String {
    fs::read_to_string(repo_root().join("tests/fixtures/face/expected-srs.html")).unwrap()
}

#[test]
fn f118_scope_m3_copied_from_scope_m1_has_the_same_block() {
    let (run, html, _) = fixture_srs("f118-copy", |srs| {
        let start = srs.find("\nscope_m1:\n").expect("写しに scope_m1 が無い") + 1;
        let end = srs.find("\nactors:\n").expect("写しに actors が無い") + 1;
        let m3 = srs[start..end].replacen("scope_m1:", "scope_m3:", 1);
        add_before_actors(&srs, &m3)
    });
    ok(&run);
    let b = callouts(&html);
    assert_eq!(b.len(), 3, "章 02 の段の範囲の塊が 3 つでない");
    for (block, label) in b.iter().zip(["M0", "M1", "M3"]) {
        assert!(
            block.contains(&format!("<div class=\"cid\">{label} で作る</div>")),
            "塊の名札が {label} でない: {block}"
        );
    }
    let want = b[1].replace("<div class=\"cid\">M1 ", "<div class=\"cid\">M3 ");
    assert_eq!(b[2], want, "3 つ目の塊が 2 つ目の名札を M3 に替えた字でない");
    assert!(b[2].contains("<span class=\"hint-body\">"), "3 つ目の塊に注の小窓が無い");
}

#[test]
fn f118_handwritten_scope_m3_adds_one_escaped_block() {
    let (run, html, _) = fixture_srs("f118-hand", |srs| add_before_actors(&srs, &m3_section()));
    ok(&run);
    let b = callouts(&html);
    assert_eq!(b.len(), 3, "章 02 の段の範囲の塊が 3 つでない");
    let m3 = b[2];
    assert!(
        m3.contains(&format!(
            "<div class=\"cid\">M3 で作る</div><p class=\"cd\">{}</p>",
            esc(M3_BUILD)
        )),
        "M3 で作るの card に build の値が無い: {m3}"
    );
    assert!(
        m3.contains(&format!(
            "<div class=\"cid\">M3 では作らない <span class=\"pill\">対象外</span></div><p class=\"cd\">{} ",
            esc(M3_NOT_BUILD)
        )),
        "M3 では作らないの card に not_build の値が無い: {m3}"
    );
    assert!(
        m3.contains(&format!("<span class=\"hint-body\">{}</span>", esc(M3_NOTE))),
        "注の小窓に note の値が無い: {m3}"
    );
    assert!(!html.contains("<中身>"), "note の生の「<」「>」が面に在る");
    let rest = html.replacen(m3, "", 1);
    assert!(rest == frozen_srs(), "M3 の塊を除いた面が凍結 fixture と一致しない");
}

#[test]
fn f118_null_scope_m3_is_unchanged_and_scope_m1_stays_required() {
    let (run, html, _) = fixture_srs("f118-null", |srs| add_before_actors(&srs, "scope_m3:\n\n"));
    ok(&run);
    assert!(html == frozen_srs(), "scope_m3 が null の面が凍結 fixture と一致しない");
    for key in ["scope", "scope_m1"] {
        let (run, _, written) =
            fixture_srs(&format!("f118-drop-{key}"), |srs| drop_section(&srs, key));
        let err = String::from_utf8_lossy(&run.stderr);
        assert_eq!(code(&run, key), 2, "{key} を落とした写しが 2 で終わらない: {err}");
        assert!(err.contains(&format!("欄 {key} が無い")), "{key}: {err}");
        assert!(!written, "{key} を落とした写しで面を書いた");
    }
}

// ── 便 135: 判断の記録の番号と範囲の節の番号を行き先へのリンクに（docs/design/delivery-135.md §1 (c)(e)）──

/// 判断の記録の面へのリンクの包みを外す（歯の側の手書きの式・生成側の式を写さない）。`<a class="xref" href="adr-` で
/// 始まるリンクごとに、中の字が判断の記録の番号（ADR- に数字列）で行き先がその番号の面（adr-<数>.html）であることを
/// 確かめてから、開きと閉じを外して字だけを残す。
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

/// 面の本文の判断の記録の番号が全部 adr-<数>.html へのリンクで、行き先の正本が在り、`<a` の入れ子が無いことを数える
/// （1 つ以上在ることも）。歯の側の数え。
fn assert_every_adr_mention_is_a_link(html: &str, face: &str) {
    let body = body_outside_svg(html);
    let mentions = adr_mentions(&body);
    assert!(!mentions.is_empty(), "{face}: 判断の記録の番号が 1 つも無い");
    for (at, n) in &mentions {
        let open = format!("<a class=\"xref\" href=\"adr-{n}.html\">");
        let end = at + "ADR-".len() + n.len();
        assert!(
            body[..*at].ends_with(&open) && body[end..].starts_with("</a>"),
            "{face}: ADR-{n} がリンクでない: {}",
            &body[at.saturating_sub(120)..(end + 40).min(body.len())]
        );
        assert!(
            design_intent().join("adr").join(format!("ADR-{n}.yaml")).is_file(),
            "{face}: ADR-{n} の正本が無い"
        );
    }
    let mut depth = 0i32;
    for (at, _) in html.match_indices('<') {
        let t = &html[at..];
        if t.starts_with("<a ") || t.starts_with("<a>") {
            depth += 1;
            assert!(depth <= 1, "{face}: 入れ子のリンクが在る");
        } else if t.starts_with("</a>") {
            depth -= 1;
        }
    }
    assert_eq!(depth, 0, "{face}: a の開きと閉じが揃わない");
}

#[test]
fn f135_scope_m3_numbers_link_to_their_pages() {
    let html = real_srs("f135-m3");
    let b = callouts(&html);
    assert_eq!(b.len(), 3, "章 02 の段の範囲の塊が 3 つでない");
    let build = span(b[2], "<div class=\"cid\">M3 で作る</div>", "</p>");
    for link in [
        "<a class=\"xref\" href=\"adr-16.html\">ADR-16</a>",
        "<a class=\"xref\" href=\"adr-21.html\">ADR-21</a>",
        "<a class=\"xref\" href=\"constitution.html#p-6\">P-6.3</a>",
        "<a class=\"xref\" href=\"#ac23\">AC23</a>",
        "<a class=\"xref\" href=\"#ac24\">AC24</a>",
        "<a class=\"xref\" href=\"#ac25\">AC25</a>",
        "<a class=\"xref\" href=\"#fr22\">FR22</a>",
        "<a class=\"xref\" href=\"#fr23\">FR23</a>",
        "<a class=\"xref\" href=\"#fr24\">FR24</a>",
        "<a class=\"xref\" href=\"#fr25\">FR25</a>",
        "<a class=\"xref\" href=\"#fr11\">FR11</a>",
    ] {
        assert_eq!(build.matches(link).count(), 1, "M3 で作るの枠に「{link}」が 1 つでない: {build}");
    }
    assert_eq!(build.matches("<a ").count(), 11, "M3 で作るの枠のリンクが 11 本でない: {build}");
}

#[test]
fn f135_every_adr_mention_on_the_real_srs_is_a_link() {
    assert_every_adr_mention_is_a_link(&real_srs("f135-adr"), "要件書の面");
}

#[test]
fn f135_numbers_without_a_page_and_ids_outside_the_scope_stay_plain() {
    let m3 = "scope_m3:\n  build:\n    - \"判断の記録 ADR-2 と ADR-99・要件 FR1 と FR99・条 P-1\"\n  not_build:\n    - \"なし\"\n\n";
    let (run, html, _) = fixture_srs_with("f135-plain", &["ADR-2.yaml"], |srs| {
        let old = "not_frozen: 時間と token の数値はここに書かない。";
        assert!(srs.contains(old), "写しに not_frozen の行が無い");
        add_before_actors(&srs.replacen(old, "not_frozen: \"ADR-2 と FR2 を読む。\"", 1), m3)
    });
    ok(&run);
    let b = callouts(&html);
    assert_eq!(b.len(), 3, "章 02 の段の範囲の塊が 3 つでない");
    for want in [
        "<a class=\"xref\" href=\"adr-2.html\">ADR-2</a> と ADR-99・",
        "要件 <a class=\"xref\" href=\"#fr1\">FR1</a> と FR99・",
        "条 <a class=\"xref\" href=\"constitution.html#p-1\">P-1</a>",
    ] {
        assert!(b[2].contains(want), "M3 の塊に「{want}」が無い: {}", b[2]);
    }
    assert!(!html.contains("adr-99.html"), "正本の無い ADR-99 がリンクになった");
    assert!(!html.contains("href=\"#fr99\""), "要件に無い FR99 がリンクになった");
    assert!(
        html.contains("<span class=\"ck\">凍結しないもの</span><p><a class=\"xref\" href=\"adr-2.html\">ADR-2</a> と FR2 を読む。</p>"),
        "範囲の節の外の凍結しないものの枠が ADR-2 だけのリンクでない"
    );
}

#[test]
fn f118_real_srs_has_an_m3_block_only_with_scope_m3() {
    let text = fs::read_to_string(design_intent().join("srs.yaml")).unwrap();
    let has = text.lines().any(|l| l.starts_with("scope_m3:"));
    let html = real_srs("f118-real");
    assert_eq!(
        html.matches("<div class=\"cid\">M3 で作る</div>").count(),
        usize::from(has),
        "実の要件書の面の M3 の塊の数が scope_m3 の有無と違う"
    );
}
