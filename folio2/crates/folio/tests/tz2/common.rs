//! folio の歯の群 tz2 の共通の手（群の 2 つ以上の module が字で同じ写しを持っていた helper を 1 つだけ置く・畳みの道具 fold2.py が寄せる・判断の記録 ADR-63 の決定 (8) の (c)）。
//! 写しを持っていた module は use crate::common::… で呼ぶ。字が 1 文字でも違う物はここへ寄せない。
#![cfg(test)]

use folio::yaml_rust2::{Yaml, YamlLoader};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 棚の判断の記録の置き場（`shelf-adr` の div の中・上向きの関係の矢印と card）。
pub(crate) fn adr_card(html: &str) -> &str {
    between(html, "<div class=\"shelf-adr\">", "\n</div>")
}

/// 判断の記録の番号の出現（byte の位置・番号の数字列）。床の形（前が英数字でも「-」でもない ADR- に 1〜9 で始まる
/// 数字列が続き、後ろが英数字でない）を歯の側で手で写す。
pub(crate) fn adr_mentions(text: &str) -> Vec<(usize, String)> {
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

/// 2 つの byte 列が同じでなければ最初の差の前後を見せて落ちる。
pub(crate) fn assert_same_bytes(written: &[u8], frozen: &[u8], what: &str) {
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

/// `open` の後の最初の `close` までの字面。
pub(crate) fn between<'a>(html: &'a str, open: &str, close: &str) -> &'a str {
    let start = html
        .find(open)
        .unwrap_or_else(|| panic!("「{open}」が無い"))
        + open.len();
    let end = html[start..]
        .find(close)
        .unwrap_or_else(|| panic!("「{open}」の後に「{close}」が無い"));
    &html[start..start + end]
}

/// 面の本文（`<body>` から後）のうち `<svg>` の中を除いた字。
pub(crate) fn body_outside_svg(html: &str) -> String {
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

pub(crate) fn code(out: &Output, what: &str) -> i32 {
    out.status.code().unwrap_or_else(|| {
        panic!(
            "{what} が signal で終わった: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

/// 属性 data-component の値を出た順に。
pub(crate) fn components(html: &str) -> Vec<&str> {
    let needle = "data-component=\"";
    html.match_indices(needle)
        .map(|(i, _)| {
            let rest = &html[i + needle.len()..];
            &rest[..rest.find('"').unwrap()]
        })
        .collect()
}

pub(crate) fn copy_dir(from: &Path, to: &Path) {
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

/// `a` から `b` の直前までを切り取る（a が無ければそのまま・a の後の最初の b・b が無ければ末尾まで）。
pub(crate) fn cut(html: &str, a: &str, b: &str) -> String {
    let Some(start) = html.find(a) else {
        return html.to_string();
    };
    let end = html[start..].find(b).map_or(html.len(), |e| start + e);
    format!("{}{}", &html[..start], &html[end..])
}

/// `a` で始まる行を改行ごと切り取る（a が無ければそのまま）。
pub(crate) fn cut_line(html: &str, a: &str) -> String {
    let Some(start) = html.find(a) else {
        return html.to_string();
    };
    let end = html[start..]
        .find('\n')
        .map_or(html.len(), |e| start + e + 1);
    format!("{}{}", &html[..start], &html[end..])
}

pub(crate) fn design_intent() -> PathBuf {
    repo_root().join("design-intent")
}

pub(crate) fn edit(path: &Path, f: impl FnOnce(&str) -> String) {
    let before = fs::read_to_string(path).unwrap();
    let after = f(&before);
    assert_ne!(before, after, "変異が当たっていない: {}", path.display());
    fs::write(path, after).unwrap();
}

pub(crate) fn fixture() -> PathBuf {
    repo_root().join("tests/fixtures/face")
}

/// fixture の正本 4 file を一時 dir の下の src/ へ写す（expected.html は写さない）。戻り値 = (一時 dir, 写し)。
pub(crate) fn fixture_copy(case: &str) -> (PathBuf, PathBuf) {
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

pub(crate) fn folio_face(face: &str, dir: &Path, out: &Path, mode: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tz"))
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

/// fixture の正本 4 file と index.yaml・intake.yaml・adr/ADR-2.yaml・design-note/full.yaml を一時 dir の下の
/// src/ へ写す（支度表 intake-sheet.yaml と期待の面は写さない）。写す判断の記録は欄の揃った便 25 の 1 本
/// （便 26 §1 (d)）・写す設計ノートは便 28 の 1 本（便 29 §1 (c)）。入口の面は設計ノートの meta しか読まないので、
/// 器の導出 file（contracts/field-schema/schema.toml）は写さない。
pub(crate) fn index_fixture_copy(case: &str) -> (PathBuf, PathBuf) {
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
pub(crate) fn index_from(case: &str, work: &Path, td: &Path) -> (PathBuf, String) {
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

/// 入口の写しに支度表（fixture の intake-sheet.yaml）も置く（便 20）。
pub(crate) fn index_sheet_copy(case: &str) -> (PathBuf, PathBuf) {
    let (td, work) = index_fixture_copy(case);
    fs::copy(
        fixture().join("intake-sheet.yaml"),
        work.join("intake-sheet.yaml"),
    )
    .unwrap();
    (td, work)
}

/// 入口の面で、fixture の写しに変異を 1 つ当て、`--write` = 2 ∧「まだ分からない」∧ 出力先が出来ていない。
pub(crate) fn index_unknown(case: &str, mutate: impl FnOnce(&Path)) {
    index_unknown_from(
        index_fixture_copy(&format!("index-unknown-{case}")),
        case,
        mutate,
    );
}

pub(crate) fn index_unknown_from((td, work): (PathBuf, PathBuf), case: &str, mutate: impl FnOnce(&Path)) {
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

pub(crate) fn load_yaml(name: &str) -> Yaml {
    load_yaml_at(&design_intent(), name)
}

pub(crate) fn load_yaml_at(dir: &Path, name: &str) -> Yaml {
    let text = fs::read_to_string(dir.join(name)).unwrap();
    YamlLoader::load_from_str(&text).unwrap().remove(0)
}

/// 棚の設計ノートの card（class の字から `</article>` まで＝class の残りも見える）。
pub(crate) fn note_card(html: &str) -> String {
    format!(
        "class=\"shelf-d{}",
        between(html, "class=\"shelf-d", "</article>")
    )
}

pub(crate) fn parts_check(pages: &[String]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tz"));
    cmd.arg("parts")
        .arg("--check")
        .arg("--dir")
        .arg(design_intent());
    for page in pages {
        cmd.arg("--page").arg(page);
    }
    cmd.output().unwrap()
}

/// 棚の見出しの「いま読めるのは <数> 面」の数。
pub(crate) fn readable_faces(html: &str) -> &str {
    between(html, "いま読めるのは <b>", " 面</b>")
}

/// 実の正本から要件書の面を一時 file へ書く。戻り値 = (一時 dir, 出力先, 面の本文)。
pub(crate) fn real_srs(case: &str) -> (PathBuf, PathBuf, String) {
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

pub(crate) fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../folio2")
}

pub(crate) fn seq<'a>(y: &'a Yaml, what: &str) -> &'a Vec<Yaml> {
    y.as_vec().unwrap_or_else(|| panic!("{what} が一覧でない"))
}

pub(crate) fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

pub(crate) fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// 図の本体の数（章の帯の kicker の絵記号 `<svg class="ico"` は数えない）。
pub(crate) fn svg_bodies(html: &str) -> usize {
    html.matches("<svg").count() - html.matches("<svg class=\"ico\"").count()
}

pub(crate) fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-face-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    fs::create_dir_all(&td).unwrap();
    td
}

pub(crate) fn text<'a>(y: &'a Yaml, key: &str) -> &'a str {
    y[key]
        .as_str()
        .unwrap_or_else(|| panic!("欄 {key} が文字列でない: {y:?}"))
}

/// 判断の記録の面へのリンクの包みを外す（便 135・歯の側の手書きの式）。`<a class="xref" href="adr-` で始まるリンクごとに、
/// 中の字が判断の記録の番号（ADR- に数字列）で行き先がその番号の面（adr-<数>.html）であることを確かめてから、字だけを残す。
pub(crate) fn unlink_adr(html: &str) -> String {
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

pub(crate) fn vendor() -> PathBuf {
    repo_root().join("vendor/archify")
}
