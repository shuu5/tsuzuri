//! 入口の面（`folio face --face index`）の節「支度表」の歯（便 20・22・66）。binary 経由。
//! - 支度表の在る面が `folio parts --check` に合格する・節の census（支度表の無い側と在る側）
//! - 導出できない 4 つ・字面の 2 case・承認の行と折りたたみの逐語（便 66）
//! - 支度表の付録は index.yaml の annexes に解く・実の正本の写しで `folio intake --write` → 入口の面の 1 周（便 22）
//!
//! 便 105（docs/design/delivery-105.md §1 (b)）で `face_index.rs` から字を変えずに移した。入口の面の骨の歯は
//! `face_index.rs`、棚の行の歯は `face_index_shelf.rs` に在る。歯の file どうしは互いに use できないので、
//! helper は `face_index.rs` の写しを持つ（写しは字を変えない・この file の歯が呼ぶものだけ）。
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
        "ceiling.yaml",
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

/// fixture の正本 4 file と index.yaml・intake.yaml・adr/ADR-2.yaml・design-note/full.yaml を一時 dir の下の
/// src/ へ写す（支度表 intake-sheet.yaml と期待の面は写さない）。写す判断の記録は欄の揃った便 25 の 1 本
/// （便 26 §1 (d)）・写す設計ノートは便 28 の 1 本（便 29 §1 (c)）。入口の面は設計ノートの meta しか読まないので、
/// 器の導出 file（contracts/schema.toml）は写さない。
fn index_fixture_copy(case: &str) -> (PathBuf, PathBuf) {
    let (td, work) = fixture_copy(case);
    for name in ["index.yaml", "intake.yaml"] {
        fs::copy(fixture().join(name), work.join(name)).unwrap();
    }
    fs::create_dir_all(work.join("adr")).unwrap();
    fs::copy(
        fixture().join("adr/ADR-2.yaml"),
        work.join("adr/ADR-2.yaml"),
    )
    .unwrap();
    fs::create_dir_all(work.join("design-note")).unwrap();
    fs::copy(
        fixture().join("design-note/full.yaml"),
        work.join("design-note/full.yaml"),
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
    // 実の正本は支度表を持つ（持ち主の裁定 2026-09-18「aで」）ので、写しから支度表を外して「まだ無い」の形を測る
    let td = temp_dir("index-sheet-census");
    let work = td.join("design-intent");
    copy_tree(&design_intent(), &work);
    let n = load_yaml("intake.yaml");
    let i = load_yaml("index.yaml");
    let _ = fs::remove_file(work.join(text(&n["sheet"], "file")));
    let out = td.join("index.html");
    let run = folio_face("index", &work, &out, "--write");
    assert_eq!(
        code(&run, "folio face --face index --write"),
        0,
        "{}",
        stderr(&run)
    );
    let html = fs::read_to_string(&out).unwrap();
    let _ = fs::remove_dir_all(&td);
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

/// 支度表の写しの承認欄に 1 行押す（便 66 の逐語は行に出ない字）。
fn stamp_approval(work: &Path, verbatim: &str) {
    edit(&work.join("intake-sheet.yaml"), |t| {
        t.replacen(
            "approval: []\n",
            &format!("approval:\n  - {{when: 2026-09-03, verbatim: {verbatim}}}\n"),
            1,
        )
    });
}

#[test]
fn face_index_sheet_approval_is_the_first_row_when_it_is_stamped() {
    let (td, work) = index_sheet_copy("index-sheet-approval");
    stamp_approval(&work, "承認する");
    let (_, html) = index_from("承認あり", &work, &td);
    let _ = fs::remove_dir_all(&td);
    assert!(
        sheet_section(&html).contains(
            "<span class=\"k\">承認</span><span class=\"v\">承認済み（2026-09-03）</span>"
        ),
        "{}",
        sheet_section(&html)
    );
}

/// 便 66（天井の 11 周目 F-7）: 承認の行は「承認済み（日付）」だけで、持ち主の逐語は折りたたみの中にある。
#[test]
fn sheet_approval_shows_approved_with_date_and_hides_the_verbatim_in_details() {
    let (td, work) = index_sheet_copy("index-sheet-approval-details");
    stamp_approval(&work, "aで");
    let (_, html) = index_from("承認あり", &work, &td);
    let _ = fs::remove_dir_all(&td);
    let section = sheet_section(&html).to_string();

    // 行は「承認済み（日付）」
    let row = section
        .lines()
        .find(|l| l.contains("<span class=\"k\">承認</span>"))
        .unwrap_or_else(|| panic!("承認の行が無い: {section}"));
    assert!(
        row.contains("<span class=\"v\">承認済み（2026-09-03）</span>"),
        "{row}"
    );
    // 逐語は details の中だけ（行の本文には出ない）
    assert!(!row.contains("aで"), "行に逐語が出ている: {row}");
    assert_eq!(section.matches("aで").count(), 1, "逐語の数: {section}");
    let details = between(&section, "<details class=\"note\">", "</details>");
    assert!(
        details.contains("<summary>言葉どおりの記録</summary>"),
        "{details}"
    );
    assert!(details.contains("aで"), "折りたたみに逐語が無い: {details}");
    // 行の数は変わらない（持つ文書・推奨で進めた項目・承認・正本）
    assert_eq!(section.matches("<p class=\"st").count(), 4, "行の数");
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
    // 実の正本の支度表（持ち主の裁定 2026-09-18「aで」）は写しから外し、1 周は支度表なしから始める
    let n = load_yaml("intake.yaml");
    let _ = fs::remove_file(work.join(text(&n["sheet"], "file")));
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
