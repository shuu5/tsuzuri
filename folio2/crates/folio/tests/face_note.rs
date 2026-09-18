//! 設計ノートの面（`folio face --face note --id <文書 id>`）の歯（便 28・docs/design/delivery-28.md §1 (g)）。binary 経由。
//! - 凍結 fixture（tests/fixtures/face/expected-note.html）との byte 一致（P-10.1）・escape
//! - 実の正本（example）で `folio parts --check` に合格（NFR2）と逐語と件数の census（yaml-rust2 で正本を直に読む）
//! - check の 3 値・id の口 4 形・節の型の閉じた一覧（FR9）・名札の表の外 2 つ・器の導出 file の欄と不在（FR10）・
//!   未解決の参照・発効と廃止・列挙の分割・章の上限・mode
//!
//! 版管理の下の面は書き換えない（`--out` は必ず一時 dir の中）。

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
    let td = std::env::temp_dir().join(format!("folio-face-note-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    fs::create_dir_all(&td).unwrap();
    td
}

/// fixture の正本 4 file と design-note/full.yaml を一時 dir の下の src/ へ、器の導出 file を親 dir の
/// contracts/ へ写す。戻り値 = (一時 dir, 写し)。
fn fixture_copy(case: &str) -> (PathBuf, PathBuf) {
    let td = temp_dir(case);
    let work = td.join("src");
    fs::create_dir_all(work.join("design-note")).unwrap();
    fs::create_dir_all(td.join("contracts")).unwrap();
    for name in [
        "constitution.yaml",
        "rules.yaml",
        "vocabulary.yaml",
        "srs.yaml",
        "design-note/full.yaml",
    ] {
        fs::copy(fixture().join(name), work.join(name)).unwrap();
    }
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        td.join("contracts/schema.toml"),
    )
    .unwrap();
    (td, work)
}

/// `folio face --face <face> [--id <id>] --dir <dir> --out <out> <mode>`。
fn folio_face(face: &str, id: Option<&str>, dir: &Path, out: &Path, mode: &str) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_folio"));
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

#[test]
fn face_note_on_the_real_source_passes_parts_check() {
    let td = temp_dir("parts");
    let (out, _) = real_face(&td);
    let check = Command::new(env!("CARGO_BIN_EXE_folio"))
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

    // 章の帯（節 6 + 図 1・承認欄は id=approval なので数えない）
    let sections = d["sections"].as_vec().unwrap();
    let figures = d["figures"].as_vec().unwrap();
    assert_eq!(sections.len(), 6, "実の正本の節の数");
    assert_eq!(figures.len(), 1, "実の正本の図の数");
    assert_eq!(count("<section id=\"s"), 7, "章の帯の数");

    // item-row（5 つの表の節の行の合計 = 4 + 4 + 4 + 2 + 1）
    let rows: usize = sections
        .iter()
        .filter_map(|s| s["rows"].as_vec().map(Vec::len))
        .sum();
    assert_eq!(rows, 15, "実の正本の表の節の行の合計");
    assert_eq!(count("data-component=\"item-row\""), rows, "item-row の数");

    // 契約表の行の欄（write-set は無い・空の一覧の depends は hint を出さない・verify は在る）
    assert!(!html.contains("write-set"), "無い欄の hint が出ている");
    assert!(!html.contains("depends"), "空の一覧の欄の hint が出ている");
    assert!(html.contains(">検証</span>"), "検証の hint が無い: {html}");

    // 図の card と「まだ分からない」の 1 か所（図の cd の字面だけ＝行き先の無い参照は 1 つも無い）
    assert_eq!(count("<div class=\"card accent warn\">"), 1, "図の card");
    // 生成器が出す「まだ分からない」は図の cd の 1 か所だけ（行き先の無い参照は 1 つも無い。
    // 正本の逐語の中の「まだ分からない」は α なので数えない）
    assert_eq!(
        count("<p class=\"cd\">図の生成はまだ無い（要件書 FR15 の便で足す）＝まだ分からない。"),
        1,
        "図の cd が 1 つでない: {html}"
    );
    assert_eq!(
        count("（まだ分からない）"),
        0,
        "行き先の無い参照が在る: {html}"
    );
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
    fs::remove_file(td.join("contracts/schema.toml")).unwrap();
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
            "  note: 手書きの見本。面の骨格だけを測る。\n  approval: {who: 持ち主, date: 2026-09-18, ruling: f2-648.43 notes 2026-09-18, verbatim: 承認する, surface: R-8}",
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

// ── mode ──

#[test]
fn face_note_mode_is_exactly_one() {
    for args in [&["--check", "--write"][..], &[][..]] {
        let out = Command::new(env!("CARGO_BIN_EXE_folio"))
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
