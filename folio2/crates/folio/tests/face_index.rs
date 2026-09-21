//! 入口の面（`folio face --face index`）の歯（便 16・20〜22 の歯を便 26・docs/design/delivery-26.md §1 (c) で
//! `face.rs` から移した面 + 便 26 §1 (e) の 6 本）。binary 経由。
//! - 凍結 fixture（tests/fixtures/face/）との byte 一致（P-10.1・回帰の anchor で唯一の判定にしない）
//! - 実の正本で生成した面が `folio parts --check` に合格する（AC2 の機構・生成側から独立した物差し 1）
//! - 逐語と件数の census（yaml-rust2 で正本を直に読む・生成側から独立した物差し 2）
//! - check の 3 値・導出できない入力・節「支度表」・棚の「判断の記録」の行（記録が在る側と無い側）
//!
//! 版管理の `design-intent/preview/` と `design-intent/` の正本は書き換えない（`--out` と写しは必ず一時 dir の中）。

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

/// 実の正本から憲法の面を一時 file へ書く。戻り値 = (一時 dir, 出力先, 面の本文)。
fn real_face(case: &str) -> (PathBuf, PathBuf, String) {
    let td = temp_dir(case);
    let out = td.join("constitution.html");
    let run = folio_face("constitution", &design_intent(), &out, "--write");
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert!(stdout(&run).contains("書いた"), "{}", stdout(&run));
    let html = fs::read_to_string(&out).unwrap();
    (td, out, html)
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

// ── 入口の面（便 16・docs/design/delivery-16.md §1 (d)）──

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

/// 写しから入口の面を書き、凍結した期待の面と byte で比べる。
fn index_frozen(case: &str, work: &Path, td: &Path, expected: &str) {
    let (out, _) = index_from(case, work, td);
    let written = fs::read(&out).unwrap();
    let frozen = fs::read(fixture().join(expected)).unwrap();
    assert_same_bytes(&written, &frozen, expected);
}

#[test]
fn face_index_write_matches_the_frozen_fixture() {
    let (td, work) = index_fixture_copy("index-anchor");
    index_frozen("支度表なし", &work, &td, "expected-index.html");
    let _ = fs::remove_dir_all(&td);
}

#[test]
fn face_index_write_with_a_sheet_matches_the_frozen_fixture() {
    let (td, work) = index_sheet_copy("index-anchor-sheet");
    index_frozen("支度表あり", &work, &td, "expected-index-sheet.html");
    let _ = fs::remove_dir_all(&td);
}

#[test]
fn face_index_face_is_writable() {
    let (td, work) = index_fixture_copy("index");
    let out = td.join("index.html");
    let run = folio_face("index", &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --face index"), 0, "{}", stderr(&run));
    assert!(stdout(&run).contains("書いた"), "{}", stdout(&run));
    assert!(
        !stderr(&run).contains("生成器はまだ無い"),
        "{}",
        stderr(&run)
    );
    assert!(exists, "入口の面を出力先に書いていない");
}

/// 実の正本から入口の面を一時 file へ書く。戻り値 = (一時 dir, 出力先, 面の本文)。
fn real_index(case: &str) -> (PathBuf, PathBuf, String) {
    let td = temp_dir(case);
    let out = td.join("index.html");
    let run = folio_face("index", &design_intent(), &out, "--write");
    assert_eq!(
        code(&run, "folio face --face index --write"),
        0,
        "{}",
        stderr(&run)
    );
    assert!(stdout(&run).contains("書いた"), "{}", stdout(&run));
    let html = fs::read_to_string(&out).unwrap();
    (td, out, html)
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

#[test]
fn face_index_on_the_real_sources_passes_parts_check() {
    let (td, out, _) = real_index("index-parts");
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
fn face_index_all_three_faces_on_the_real_sources_pass_parts_check_together() {
    let (td_i, index, _) = real_index("three-index");
    let (td_c, constitution, _) = real_face("three-constitution");
    let (td_s, srs, _) = real_srs("three-srs");
    let check = parts_check(&[
        format!("index={}", index.display()),
        format!("constitution={}", constitution.display()),
        format!("srs={}", srs.display()),
    ]);
    for td in [td_i, td_c, td_s] {
        let _ = fs::remove_dir_all(&td);
    }
    assert_eq!(
        code(&check, "folio parts --check（3 面）"),
        0,
        "{}{}",
        stdout(&check),
        stderr(&check)
    );
    assert!(stdout(&check).contains("違反 0"), "{}", stdout(&check));
}

// ── 棚の「判断の記録」の行（便 26・docs/design/delivery-26.md §1 (a)(e)）──

/// 棚の判断の記録の置き場（`shelf-adr` の div の中・上向きの関係の矢印と card）。
fn adr_card(html: &str) -> &str {
    between(html, "<div class=\"shelf-adr\">", "\n</div>")
}

/// 棚の見出しの「いま読めるのは <数> 面」の数。
fn readable_faces(html: &str) -> &str {
    between(html, "いま読めるのは <b>", " 面</b>")
}

/// status-line の「まだ無い」の行の v。
fn still_absent(html: &str) -> &str {
    between(
        html,
        "<span class=\"k\">まだ無い</span><span class=\"v\">",
        "</span>",
    )
}

/// 実の正本の判断の記録を id の数の昇順に（戻り値 = (id, status)）。
fn real_records() -> Vec<(String, String)> {
    let mut rows: Vec<(u64, String, String)> = fs::read_dir(design_intent().join("adr"))
        .unwrap()
        .filter_map(|e| {
            let name = e.unwrap().file_name().to_string_lossy().into_owned();
            if !name.starts_with("ADR-") || !name.ends_with(".yaml") {
                return None;
            }
            let y = load_yaml_at(&design_intent().join("adr"), &name);
            let id = text(&y, "id").to_string();
            let n: u64 = id.split('-').nth(1).unwrap().parse().unwrap();
            Some((n, id, text(&y, "status").to_string()))
        })
        .collect();
    rows.sort_by_key(|(n, _, _)| *n);
    rows.into_iter().map(|(_, id, st)| (id, st)).collect()
}

#[test]
fn face_index_shelf_adr_card_is_readable_with_one_record() {
    let (td, work) = index_fixture_copy("index-adr-one");
    let (_, html) = index_from("記録 1 本", &work, &td);
    let _ = fs::remove_dir_all(&td);
    let card = adr_card(&html);

    assert!(
        !card.contains("is-absent"),
        "記録が在るのに is-absent: {card}"
    );
    assert!(
        card.contains(
            "<span class=\"state ok\">● 1 本</span><span>ADR-2〜ADR-2（提案中 1）</span>"
        ),
        "{card}"
    );
    for want in [
        "<a class=\"xref\" href=\"adr-2.html\">ADR-2</a>",
        "<a class=\"sc-open\" href=\"adr-2.html\">開く →</a>",
        "<a class=\"sc-hit\" href=\"adr-2.html\"",
    ] {
        assert!(card.contains(want), "{want} が無い: {card}");
    }
    assert_eq!(
        html.matches("ページはまだ無い").count(),
        0,
        "記録が在れば「ページはまだ無い」は出ない"
    );
    assert!(!card.contains("sc-none"), "{card}");
    // 写しは設計ノートも 1 本持つ（便 29）ので、読める面は 入口 + 憲法 + 要件書 + 設計ノート + 記録
    assert_eq!(readable_faces(&html), "5");
    assert!(
        !still_absent(&html).contains("判断の記録"),
        "「まだ無い」に判断の記録が残っている: {}",
        still_absent(&html)
    );
}

#[test]
fn face_index_shelf_adr_card_is_absent_without_records() {
    let (td, work) = index_fixture_copy("index-adr-none");
    fs::remove_file(work.join("adr/ADR-2.yaml")).unwrap();
    let (_, html) = index_from("記録 0 本", &work, &td);
    let _ = fs::remove_dir_all(&td);
    let card = adr_card(&html);
    let i = load_yaml_at(&fixture(), "index.yaml");
    let absent = seq(&i["shelf"]["documents"], "documents")
        .iter()
        .find(|d| text(d, "id") == "adr")
        .map(|d| esc(text(d, "absent")))
        .expect("index.yaml の棚に adr の行が無い");

    assert!(card.contains("class=\"is-absent\""), "{card}");
    assert!(
        card.contains(&format!(
            "<span class=\"state\">○ 0 本</span><span>{absent}</span>"
        )),
        "{card}"
    );
    assert!(card.contains("sc-none"), "{card}");
    // 設計ノートは写しに 1 本残っているので、読める面は 入口 + 憲法 + 要件書 + 設計ノート
    assert_eq!(readable_faces(&html), "4");
}

#[test]
fn face_index_unknown_when_a_record_status_is_outside_the_table() {
    let (td, work) = index_fixture_copy("index-adr-status");
    edit(&work.join("adr/ADR-2.yaml"), |t| {
        t.replacen("status: proposed", "status: draft", 1)
    });
    let out = td.join("never.html");
    let run = folio_face("index", &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --face index"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("状態"), "{}", stderr(&run));
    assert!(!exists, "導出できないのに出力先に書いた");
}

#[test]
fn face_index_unknown_when_two_records_share_a_number() {
    let (td, work) = index_fixture_copy("index-adr-duplicate");
    let body = fs::read_to_string(work.join("adr/ADR-2.yaml")).unwrap();
    let same = body.replacen("id: ADR-2\n", "id: ADR-02\n", 1);
    assert_ne!(body, same, "変異が当たっていない");
    fs::write(work.join("adr/ADR-02.yaml"), same).unwrap();
    let out = td.join("never.html");
    let run = folio_face("index", &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --face index"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("2 本以上"), "{}", stderr(&run));
    assert!(!exists, "導出できないのに出力先に書いた");
}

/// `open` から `close` までを（`close` も含めて）1 度だけ切り落とした残り。
fn cut(html: &str, open: &str, close: &str) -> String {
    let start = html
        .find(open)
        .unwrap_or_else(|| panic!("「{open}」が無い"));
    let end = start
        + html[start..]
            .find(close)
            .unwrap_or_else(|| panic!("「{open}」の後に「{close}」が無い"))
        + close.len();
    format!("{}{}", &html[..start], &html[end..])
}

// ── 棚の「設計ノート」の行（便 29・docs/design/delivery-29.md §1 (a)(e)）──

/// 棚の設計ノートの card（class の字から `</article>` まで＝class の残りも見える）。
fn note_card(html: &str) -> String {
    format!(
        "class=\"shelf-d{}",
        between(html, "class=\"shelf-d", "</article>")
    )
}

/// 棚の見出しの型の列挙（「いま読めるのは <数> 面 ＝ このページ（入口）と、」の後）。
fn readable_types(html: &str) -> &str {
    between(
        html,
        " ＝ このページ（入口）と、",
        " <span class=\"fig-tools\">",
    )
}

/// shelf-head の sub（「これから増える文書 …」）。
fn shelf_head(html: &str) -> &str {
    between(html, "<span class=\"sub\">これから増える文書 ", "</span>")
}

/// status-line の「揃っている」の行の v。
fn all_there(html: &str) -> &str {
    between(
        html,
        "<span class=\"k\">揃っている</span><span class=\"v\">",
        "</span>",
    )
}

/// 実の正本の設計ノート（`design-note/` の `.yaml` から欄の決まりを除く）を id の字の昇順に。
/// 戻り値 = (id, status)。
fn real_notes() -> Vec<(String, String)> {
    let dir = design_intent().join("design-note");
    let mut rows: Vec<(String, String)> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| {
            let name = e.unwrap().file_name().to_string_lossy().into_owned();
            if !name.ends_with(".yaml") || name == "schema.yaml" {
                return None;
            }
            let y = load_yaml_at(&dir, &name);
            Some((
                text(&y["meta"], "id").to_string(),
                text(&y["meta"], "status").to_string(),
            ))
        })
        .collect();
    rows.sort();
    rows
}

#[test]
fn face_index_shelf_note_card_is_readable_with_one_note() {
    let (td, work) = index_fixture_copy("index-note-one");
    let (_, html) = index_from("設計ノート 1 本", &work, &td);
    let _ = fs::remove_dir_all(&td);
    let card = note_card(&html);

    assert!(
        !card.contains("is-absent"),
        "設計ノートが在るのに is-absent: {card}"
    );
    assert!(
        card.contains("<span class=\"state ok\">● 1 本</span><span>下書き 1</span>"),
        "{card}"
    );
    for want in [
        "<a class=\"xref\" href=\"note-full.html\">full</a>",
        "<a class=\"sc-open\" href=\"note-full.html\">開く →</a>",
        "<a class=\"sc-hit\" href=\"note-full.html\"",
    ] {
        assert!(card.contains(want), "{want} が無い: {card}");
    }
    assert!(!card.contains("sc-none"), "{card}");
    assert_eq!(readable_faces(&html), "5");
    assert_eq!(
        readable_types(&html),
        "憲法・要件書・設計ノート・判断の記録"
    );
    assert!(
        shelf_head(&html).starts_with("なし ／"),
        "{}",
        shelf_head(&html)
    );
    assert_eq!(still_absent(&html), "なし");
    assert!(
        all_there(&html).starts_with("憲法・要件書・設計ノート 1 本・判断の記録 1 本が読める"),
        "{}",
        all_there(&html)
    );
    assert!(
        html.contains("<a class=\"state ok\" href=\"#doc-design-note\">"),
        "小窓の設計ノートが読める側でない"
    );
    assert!(html.contains("design-note/"), "foot の sources に無い");
}

#[test]
fn face_index_shelf_note_card_is_absent_without_notes() {
    let (td, work) = index_fixture_copy("index-note-none");
    fs::remove_file(work.join("design-note/full.yaml")).unwrap();
    let (_, html) = index_from("設計ノート 0 本", &work, &td);
    let _ = fs::remove_dir_all(&td);
    let card = note_card(&html);
    let i = load_yaml_at(&fixture(), "index.yaml");
    let absent = seq(&i["shelf"]["documents"], "documents")
        .iter()
        .find(|d| text(d, "id") == "design-note")
        .map(|d| esc(text(d, "absent")))
        .expect("index.yaml の棚に design-note の行が無い");

    assert!(card.contains("class=\"shelf-d is-absent\""), "{card}");
    assert!(
        card.contains(&format!(
            "<span class=\"state\">○ まだ無い</span><span>{absent}</span>"
        )),
        "{card}"
    );
    assert!(card.contains("sc-none"), "{card}");
    assert_eq!(readable_faces(&html), "4");
    assert!(
        shelf_head(&html).starts_with("1（設計ノート） ／"),
        "{}",
        shelf_head(&html)
    );
    assert!(
        still_absent(&html).contains("設計ノート"),
        "{}",
        still_absent(&html)
    );
}

#[test]
fn face_index_unknown_when_a_note_status_is_outside_the_table() {
    let (td, work) = index_fixture_copy("index-note-status");
    edit(&work.join("design-note/full.yaml"), |t| {
        t.replacen("status: draft", "status: final", 1)
    });
    let out = td.join("never.html");
    let run = folio_face("index", &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --face index"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("状態"), "{}", stderr(&run));
    assert!(!exists, "導出できないのに出力先に書いた");
}

#[test]
fn face_index_unknown_when_a_note_id_differs_from_the_file_name() {
    let (td, work) = index_fixture_copy("index-note-id");
    edit(&work.join("design-note/full.yaml"), |t| {
        t.replacen("  id: full\n", "  id: fill\n", 1)
    });
    let out = td.join("never.html");
    let run = folio_face("index", &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --face index"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("file 名"), "{}", stderr(&run));
    assert!(!exists, "導出できないのに出力先に書いた");
}

#[test]
fn face_index_unknown_when_the_note_dir_is_missing() {
    let (td, work) = index_fixture_copy("index-note-dir");
    fs::remove_dir_all(work.join("design-note")).unwrap();
    let out = td.join("never.html");
    let run = folio_face("index", &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --face index"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("design-note/"), "{}", stderr(&run));
    assert!(!exists, "導出できないのに出力先に書いた");
}

#[test]
fn face_index_sections_outside_the_shelf_do_not_move_with_the_notes() {
    let (td, work) = index_fixture_copy("index-note-outside");
    let (_, with) = index_from("設計ノート 1 本", &work, &td);
    fs::remove_file(work.join("design-note/full.yaml")).unwrap();
    let (_, without) = index_from("設計ノート 0 本", &work, &td);
    let _ = fs::remove_dir_all(&td);
    // 棚の図と状態の行だけが動く（foot の sources はどちらも design-note/ を持つ）
    assert_ne!(with, without, "設計ノートの有無で面が変わっていない");
    assert_same_bytes(
        outside_the_shelf(&with).as_bytes(),
        outside_the_shelf(&without).as_bytes(),
        "棚の節の外",
    );
}

/// 面から棚の図と状態の行の節を除いた残り（cover・読む順番・相談窓口・支度表・foot）。
fn outside_the_shelf(html: &str) -> String {
    let rest = cut(html, "<figure ", "</figure>\n");
    cut(
        &rest,
        "<section data-component=\"status-line\" aria-label=\"いまの状態\">",
        "</section>\n",
    )
}

#[test]
fn face_index_sections_outside_the_shelf_do_not_move_with_the_records() {
    let (td, work) = index_fixture_copy("index-adr-outside");
    let (_, with) = index_from("記録 1 本", &work, &td);
    fs::remove_file(work.join("adr/ADR-2.yaml")).unwrap();
    let (_, without) = index_from("記録 0 本", &work, &td);
    let _ = fs::remove_dir_all(&td);
    // 棚の図と状態の行だけが動く（前の版の凍結 fixture を歯に埋め込まず、同じ写しの 2 面で比べる）
    assert_ne!(with, without, "記録の有無で面が変わっていない");
    assert_same_bytes(
        outside_the_shelf(&with).as_bytes(),
        outside_the_shelf(&without).as_bytes(),
        "棚の節の外",
    );
}

#[test]
fn face_index_census_on_the_real_sources_counts_and_verbatims() {
    let (td, _, html) = real_index("index-census");
    let _ = fs::remove_dir_all(&td);
    let i = load_yaml("index.yaml");
    let c = load_yaml("constitution.yaml");
    let s = load_yaml("srs.yaml");
    let v = load_yaml("vocabulary.yaml");
    let r = load_yaml("rules.yaml");

    // 逐語
    let mut wants = vec![text(&i["audience"], "text").to_string()];
    let shelf = &i["shelf"];
    for d in seq(&shelf["documents"], "documents") {
        wants.push(text(d, "use").to_string());
    }
    for rel in seq(&shelf["relations"], "relations") {
        wants.push(text(rel, "label").to_string());
    }
    let rows = seq(&i["lanes"]["rows"], "rows");
    let mut stops = 0;
    for row in rows {
        wants.push(text(row, "who").to_string());
        wants.push(text(row, "why").to_string());
        for st in seq(&row["stops"], "stops") {
            wants.push(text(st, "label").to_string());
            let at = text(st, "at");
            // 判断の記録は面が 1 本 1 枚なので節の anchor を持たない（便 66）
            let href = match text(st, "doc") {
                "constitution" => format!("<li><a href=\"constitution.html#{at}\">"),
                "srs" => format!("<li><a href=\"srs.html#{at}\">"),
                "adr" => format!("<li><a href=\"{}.html\">", at.to_lowercase()),
                other => panic!("面の無い行き先「{other}」"),
            };
            assert!(html.contains(&href), "行き先の href が無い: {href}");
            stops += 1;
        }
    }
    wants.push(text(&i["intake"], "text").to_string());
    for step in seq(&i["intake"]["steps"], "steps") {
        wants.push(step.as_str().unwrap().to_string());
    }
    for want in wants {
        let want = esc(&want);
        assert!(html.contains(&want), "面に無い: {want}");
    }
    let lanes = between(&html, "<div class=\"lane-grid\"", "\n</div>\n</div>");
    assert_eq!(lanes.matches("<li><a href=\"").count(), stops, "行き先の数");

    // 件数
    let parts = components(&html);
    let parts_of = |name: &str| parts.iter().filter(|p| **p == name).count();
    assert_eq!(parts_of("shelf-card"), 4, "shelf-card の数");
    assert_eq!(parts_of("shelf-link"), 4, "shelf-link の数");
    assert_eq!(parts_of("reader-lane"), rows.len(), "reader-lane の数");
    let articles = seq(&c["articles"], "articles").len();
    let fr = seq(&s["requirements"], "requirements").len();
    let terms = seq(&v["terms"], "terms").len();
    let rules =
        seq(&r["thresholds"], "thresholds").len() + seq(&r["discipline"], "discipline").len();
    let records = real_records();
    let adr = records.len();
    for want in [
        format!("{articles} 条"),
        format!("機能 {fr}"),
        format!("{terms} 語"),
        format!("{rules} 行"),
        format!("{adr} 本"),
    ] {
        assert!(html.contains(&want), "数の字面が無い: {want}");
    }

    // 判断の記録の行（便 26）: 記録の数だけ面へのリンクが id の順に並び、要約は状態ごとの数
    let card = adr_card(&html);
    let links = records
        .iter()
        .map(|(id, _)| {
            format!(
                "<a class=\"xref\" href=\"{}.html\">{id}</a>",
                id.to_lowercase()
            )
        })
        .collect::<Vec<_>>()
        .join("・");
    assert!(
        card.contains(&links),
        "記録へのリンクが id の順に無い: {card}"
    );
    let kinds = [
        ("accepted", "発効"),
        ("proposed", "提案中"),
        ("retired", "廃止"),
    ]
    .iter()
    .filter_map(|(key, label)| {
        let n = records.iter().filter(|(_, st)| st == key).count();
        (n > 0).then(|| format!("{label} {n}"))
    })
    .collect::<Vec<_>>()
    .join("・");
    let summary = format!("{}〜{}（{kinds}）", records[0].0, records[adr - 1].0);
    assert!(card.contains(&summary), "要約が無い: {summary}\n{card}");

    // 設計ノートの行（便 29）: 設計ノートの数だけ面へのリンクが id の字の順に並び、要約は状態ごとの数
    let notes = real_notes();
    let note_card = note_card(&html);
    let note_links = notes
        .iter()
        .map(|(id, _)| format!("<a class=\"xref\" href=\"note-{id}.html\">{id}</a>"))
        .collect::<Vec<_>>()
        .join("・");
    assert!(
        note_card.contains(&note_links),
        "設計ノートへのリンクが id の順に無い: {note_card}"
    );
    let note_kinds = [
        ("draft", "下書き"),
        ("effective", "発効"),
        ("retired", "廃止"),
        ("example", "見本"),
    ]
    .iter()
    .filter_map(|(key, label)| {
        let n = notes.iter().filter(|(_, st)| st == key).count();
        (n > 0).then(|| format!("{label} {n}"))
    })
    .collect::<Vec<_>>()
    .join("・");
    assert!(
        note_card.contains(&format!(
            "<span class=\"state ok\">● {} 本</span><span>{note_kinds}</span>",
            notes.len()
        )),
        "設計ノートの要約が無い: {note_kinds}\n{note_card}"
    );
    assert_eq!(
        readable_faces(&html),
        (3 + adr + notes.len()).to_string(),
        "読める面の数"
    );

    // 部品の名札は 13 種の中だけ
    const ALLOWED: [&str; 13] = [
        "freshness-stamp",
        "ceiling-stamp",
        "font-size-control",
        "hub-cover",
        "figure-panel",
        "doc-shelf",
        "shelf-card",
        "shelf-link",
        "status-line",
        "intake-line",
        "chapter-deck-band",
        "reader-lane",
        "intake-callout",
    ];
    assert!(!parts.is_empty());
    for p in &parts {
        assert!(ALLOWED.contains(p), "13 種に無い部品「{p}」");
    }
    assert!(!html.contains("layer-line"));
}

#[test]
fn face_index_check_has_three_values() {
    let (td, work) = index_fixture_copy("index-check");
    let out = td.join("index.html");

    let write = folio_face("index", &work, &out, "--write");
    let ok = folio_face("index", &work, &out, "--check");
    let mut bytes = fs::read(&out).unwrap();
    bytes[0] ^= 0x20;
    fs::write(&out, &bytes).unwrap();
    let drift = folio_face("index", &work, &out, "--check");
    fs::remove_file(&out).unwrap();
    let missing = folio_face("index", &work, &out, "--check");
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

fn index_edit(from: &'static str, to: &'static str) -> impl FnOnce(&Path) {
    move |w| edit(&w.join("index.yaml"), |t| t.replacen(from, to, 1))
}

#[test]
fn face_index_unknown_when_a_document_is_missing() {
    index_unknown(
        "documents",
        index_edit(
            "    - {id: adr, type: 判断の記録, use: 戻せない判断の記録。, absent: まだ無い判断}\n",
            "",
        ),
    );
}

#[test]
fn face_index_unknown_when_a_relation_id_is_outside_the_table() {
    index_unknown(
        "relations",
        index_edit("{id: before-build,", "{id: before,"),
    );
}

#[test]
fn face_index_unknown_when_a_third_annex_is_added() {
    index_unknown(
        "annexes",
        index_edit(
            "    - {id: rules, type: 数値の表, inside: constitution}\n",
            "    - {id: rules, type: 数値の表, inside: constitution}\n    - {id: glossary, type: 用語, inside: constitution}\n",
        ),
    );
}

#[test]
fn face_index_unknown_when_a_stop_anchor_is_outside_the_face() {
    index_unknown(
        "stop-at",
        index_edit("at: s0, label: 目指すこと}", "at: s9, label: 目指すこと}"),
    );
}

#[test]
fn face_index_unknown_when_a_stop_names_a_document_without_a_face() {
    index_unknown(
        "stop-doc",
        index_edit(
            "{doc: srs, at: fig-rail,",
            "{doc: design-note, at: fig-rail,",
        ),
    );
}

#[test]
fn face_index_unknown_when_a_legend_id_is_outside_the_table() {
    index_unknown(
        "legend",
        index_edit("{id: binds, text: 矢印}", "{id: arrow, text: 矢印}"),
    );
}

/// 便 76 §1 (f)(h)8: 凡例も表の id と過不足なく（行を 1 つ消すと面が出ない）。
#[test]
fn f76_face_index_unknown_when_a_legend_row_is_missing() {
    index_unknown(
        "legend-missing",
        index_edit("    - {id: absent, text: まだ無い}\n", ""),
    );
}

/// 便 76 §1 (f)(h)9: 凡例の並びは表が決める（正本の行の並びを入れ替えても面は同じ byte）。
#[test]
fn f76_face_index_legend_order_comes_from_the_table() {
    let (td, work) = index_fixture_copy("legend-order-before");
    let (_, before) = index_from("入れ替える前", &work, &td);
    let _ = fs::remove_dir_all(&td);

    let (td, work) = index_fixture_copy("legend-order-after");
    edit(&work.join("index.yaml"), |t| {
        let rows = "    - {id: readable, text: 読める}\n    - {id: absent, text: まだ無い}\n    - {id: binds, text: 矢印}\n    - {id: inside, text: 点線}\n";
        let swapped = "    - {id: inside, text: 点線}\n    - {id: binds, text: 矢印}\n    - {id: absent, text: まだ無い}\n    - {id: readable, text: 読める}\n";
        assert_eq!(t.matches(rows).count(), 1, "凡例の 4 行が表の並びで無い");
        t.replacen(rows, swapped, 1)
    });
    let (_, after) = index_from("入れ替えた後", &work, &td);
    let _ = fs::remove_dir_all(&td);
    assert_same_bytes(
        after.as_bytes(),
        before.as_bytes(),
        "凡例の並びを入れ替えた面",
    );
}

#[test]
fn face_index_unknown_when_constitution_counts_differ() {
    index_unknown("counts", |w| {
        edit(&w.join("constitution.yaml"), |t| {
            t.replacen(
                "{always: 1, ask-first: 1, never: 1}",
                "{always: 2, ask-first: 1, never: 1}",
                1,
            )
        })
    });
}

#[test]
fn face_index_unknown_when_the_adr_dir_is_missing() {
    index_unknown("adr-dir", |w| fs::remove_dir_all(w.join("adr")).unwrap());
}

// ── 読む順番の行き先の判断の記録（便 66・docs/design/delivery-66.md §1 (a)(b)）──

/// fixture の入口の正本の読む順番に行き先を 1 つ足す（判断の記録の行）。
fn add_stop(at: &'static str, label: &'static str) -> impl FnOnce(&Path) {
    move |w: &Path| {
        edit(&w.join("index.yaml"), |t| {
            t.replacen(
                "        - {doc: srs, at: fig-rail, label: 段の図}\n",
                &format!(
                    "        - {{doc: srs, at: fig-rail, label: 段の図}}\n        - {{doc: adr, at: {at}, label: {label}}}\n"
                ),
                1,
            )
        })
    }
}

/// fixture の入口の棚が判断の記録に与える型（「判断の記録」）。
fn adr_type() -> String {
    let i = load_yaml_at(&fixture(), "index.yaml");
    seq(&i["shelf"]["documents"], "documents")
        .iter()
        .find(|d| text(d, "id") == "adr")
        .map(|d| esc(text(d, "type")))
        .expect("index.yaml の棚に adr の行が無い")
}

#[test]
fn lane_adr_stop_links_to_the_record_face() {
    let (td, work) = index_fixture_copy("index-lane-adr");
    // fixture の写しが持つ判断の記録は ADR-2 の 1 本
    add_stop("ADR-2", "なぜそう決めたか")(&work);
    let (_, html) = index_from("判断の記録への行き先", &work, &td);
    let _ = fs::remove_dir_all(&td);
    let want = format!(
        "<li><a href=\"adr-2.html\">{} なぜそう決めたか</a></li>",
        adr_type()
    );
    assert!(
        html.contains(&want),
        "行き先が無い: {want}\n{}",
        between(&html, "<div class=\"lane-grid\"", "\n</div>\n</div>")
    );
}

#[test]
fn lane_adr_stop_with_unknown_record_is_refused() {
    index_unknown("lane-adr-unknown", add_stop("ADR-99", "無い記録"));
}

#[test]
fn lane_adr_stop_anchor_rejects_other_shapes() {
    let (td, work) = index_fixture_copy("index-lane-adr-shape");
    add_stop("s1", "節の id")(&work);
    let out = td.join("never.html");
    let run = folio_face("index", &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --face index"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("まだ分からない"), "{}", stderr(&run));
    assert!(stderr(&run).contains("行き先"), "{}", stderr(&run));
    assert!(!exists, "導出できないのに出力先に書いた");
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
