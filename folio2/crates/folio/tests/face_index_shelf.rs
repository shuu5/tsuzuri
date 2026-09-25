//! 入口の面（`folio face --face index`）の棚の行の歯（便 26・29）。binary 経由。
//! - 棚の「判断の記録」の行: 記録が在る側と無い側の card・状態が表に無い・番号が重なる（便 26）
//! - 棚の「設計ノート」の行: 設計ノートが在る側と無い側の card・状態が表に無い・id と file 名のずれ・置き場の不在
//!   （便 29）
//! - 棚の図と状態の行の外は、記録や設計ノートの有無で動かない
//!
//! 便 105（docs/design/delivery-105.md §1 (b)）で `face_index.rs` から字を変えずに移した。入口の面の骨の歯は
//! `face_index.rs`、節「支度表」の歯は `face_index_sheet.rs` に在る。歯の file どうしは互いに use できないので、
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

fn load_yaml_at(dir: &Path, name: &str) -> Yaml {
    let text = fs::read_to_string(dir.join(name)).unwrap();
    YamlLoader::load_from_str(&text).unwrap().remove(0)
}

fn seq<'a>(y: &'a Yaml, what: &str) -> &'a Vec<Yaml> {
    y.as_vec().unwrap_or_else(|| panic!("{what} が一覧でない"))
}

fn text<'a>(y: &'a Yaml, key: &str) -> &'a str {
    y[key]
        .as_str()
        .unwrap_or_else(|| panic!("欄 {key} が文字列でない: {y:?}"))
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
        "<li><a class=\"xref\" href=\"adr-2.html\">ADR-2</a><span>見本の判断の記録</span></li>",
    ] {
        assert!(card.contains(want), "{want} が無い: {card}");
    }
    // 当たり判定は置かない（効けば一覧を覆う・便 139）
    assert!(!card.contains("sc-hit"), "{card}");
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
