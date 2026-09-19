//! `folio parts`（便 13・docs/design/delivery-13.md §1）。3 面の生成器が使ってよい部品の一覧を型で閉じ（組み立て時に
//! 部品目録 `preview/parts.json` から build.rs が導出する・P-6.4）、できた面を部品目録と突き合わせる（要件書 AC2 の機構）。
//! --print は導出した一覧を JSON の 1 行で出す。--check は 部品目録と組み立て時の写しの一致 → 面の class（様式の定義の
//! class の集合に在る）・部品の名札（部品の一覧に在り、その面に置ける）・行内の様式（許す性質だけ）を数える。
//! 面と様式の定義は外部 crate も正規表現も使わない手書きの走査で読む。

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::verdict::Report;
use crate::yaml::{self, Node, json_str};

/// 組み立て時に部品目録から導出した閉じた一覧（build.rs が OUT_DIR に書く）。
#[allow(dead_code)]
pub mod catalog {
    include!(concat!(env!("OUT_DIR"), "/parts_catalog.rs"));
}

use catalog::{Component, FigureType, ShelfType, StyleProp};

/// 面の名（--page の左辺・部品目録の faces の値）。
pub const FACES: [&str; 5] = ["index", "constitution", "srs", "adr", "note"];

/// 検査する 3 つの属性。
const ATTRS: [&str; 3] = ["class", "style", "data-component"];

// ── --print ──

/// 導出した一覧の JSON 1 行（末尾に改行 1 つ）。キーは components・figure_types・shelf_types・style_props の順。
pub fn print_catalog() -> String {
    let mut out = String::from("{\"components\":{");
    for (i, c) in Component::ALL.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        json_str(c.name(), &mut out);
        out.push(':');
        json_list(c.faces().iter().copied(), &mut out);
    }
    out.push_str("},\"figure_types\":");
    json_list(FigureType::ALL.iter().map(|t| t.name()), &mut out);
    out.push_str(",\"shelf_types\":");
    json_list(ShelfType::ALL.iter().map(|t| t.name()), &mut out);
    out.push_str(",\"style_props\":");
    json_list(StyleProp::ALL.iter().map(|p| p.name()), &mut out);
    out.push_str("}\n");
    out
}

fn json_list<'a>(items: impl Iterator<Item = &'a str>, out: &mut String) {
    out.push('[');
    for (i, s) in items.enumerate() {
        if i > 0 {
            out.push(',');
        }
        json_str(s, out);
    }
    out.push(']');
}

// ── --check ──

/// 1 回の検査。`pages` は「<面の名>=<path>」（空なら dir の下の preview/ の 3 面）・`css` が None なら dir の下の preview/folio.css。
pub fn check(dir: &Path, css: Option<&Path>, pages: &[String]) -> Report {
    let mut report = Report::default();

    // 1. 部品目録と組み立て時の写しの一致（違えば以下は数えない）
    if let Err(msg) = catalog_matches(&dir.join("preview/parts.json")) {
        report.unknown(msg);
        return report;
    }

    // 面の名と path
    let mut faces: Vec<(&str, PathBuf)> = Vec::new();
    if pages.is_empty() {
        for face in FACES {
            faces.push((face, dir.join("preview").join(format!("{face}.html"))));
        }
    }
    for page in pages {
        match page.split_once('=') {
            Some((face, path)) => match FACES.iter().find(|f| **f == face) {
                Some(face) => faces.push((face, PathBuf::from(path))),
                None => report.unknown(format!(
                    "--page「{page}」: 面の名「{face}」は index・constitution・srs・adr・note のどれでもない"
                )),
            },
            None => report.unknown(format!("--page「{page}」: <面の名>=<path> の形でない")),
        }
    }

    // 様式の定義
    let css_path = css.map_or_else(|| dir.join("preview/folio.css"), Path::to_path_buf);
    let classes = match fs::read_to_string(&css_path) {
        Ok(text) => match css_classes(&text) {
            Ok(set) => Some(set),
            Err(e) => {
                report.unknown(format!("様式の定義 {}: {e}", css_path.display()));
                None
            }
        },
        Err(e) => {
            report.unknown(format!(
                "様式の定義 {}: 読めない（{e}）",
                css_path.display()
            ));
            None
        }
    };

    for (face, path) in &faces {
        check_face(face, path, classes.as_ref(), &mut report);
    }
    report
}

/// 実行時の部品目録を読み、組み立て時に導出した一覧と過不足なく同じ順で一致するか。違えば Err（まだ分からない）。
fn catalog_matches(path: &Path) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("parts.json: 読めない（{e}）"))?;
    let doc = yaml::parse(&text).map_err(|e| format!("parts.json: 読めない（{e}）"))?;
    if let Some(d) = doc.duplicates.first() {
        return Err(format!(
            "parts.json: 同じキー「{}」が 2 度ある（{} 行）",
            d.key, d.line
        ));
    }
    let root = &doc.root;
    let components = root.get("components").and_then(Node::as_map);
    let same_components = components.is_some_and(|m| {
        m.len() == Component::ALL.len()
            && m.iter().zip(Component::ALL).all(|((name, body), c)| {
                name == c.name() && same_list(body.get("faces"), c.faces().iter().copied())
            })
    });
    let same = same_components
        && same_list(
            root.get("figure_type_enum"),
            FigureType::ALL.iter().map(|t| t.name()),
        )
        && same_list(
            root.get("shelf_type_enum"),
            ShelfType::ALL.iter().map(|t| t.name()),
        )
        && same_list(
            root.get("style_props_allowed"),
            StyleProp::ALL.iter().map(|p| p.name()),
        );
    if same {
        Ok(())
    } else {
        Err("parts.json: 組み立て時の部品目録と違う（組み立て直す）".to_string())
    }
}

/// 文字列の一覧が `want` と同じ順で同じか（一覧でない・文字列でない値が在る、は違う）。
fn same_list<'a>(node: Option<&Node>, want: impl ExactSizeIterator<Item = &'a str>) -> bool {
    let Some(seq) = node.and_then(Node::as_seq) else {
        return false;
    };
    seq.len() == want.len() && seq.iter().zip(want).all(|(n, w)| n.as_str() == Some(w))
}

/// 面 1 つ: 読めなければ「まだ分からない」1 件で終わり、読めれば class・部品の名札・行内の様式を数える。
fn check_face(face: &str, path: &Path, classes: Option<&HashSet<String>>, report: &mut Report) {
    let file = path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            report.unknown(format!("面 {}: 読めない（{e}）", path.display()));
            return;
        }
    };
    let attrs = match scan_tags(&text).and_then(|tags| face_attrs(&tags)) {
        Ok(a) => a,
        Err(e) => {
            report.unknown(format!("{file}: {e}"));
            return;
        }
    };
    let mut seen = HashSet::new();
    for (name, value) in &attrs {
        let value = value.as_str();
        match *name {
            "class" => {
                let Some(classes) = classes else { continue };
                for word in value.split_ascii_whitespace() {
                    if !classes.contains(word) && seen.insert(word.to_string()) {
                        report.violation("R-3", format!("{file}: 部品目録に無い class「{word}」"));
                    }
                }
            }
            "data-component" => match Component::from_name(value) {
                None => report.violation("parts", format!("{file}: 部品目録に無い部品「{value}」")),
                Some(c) if !c.faces().contains(&face) => {
                    report.violation(
                        "parts",
                        format!("{file}: 部品「{value}」はこの面に置けない"),
                    );
                }
                Some(_) => {}
            },
            _ => {
                for prop in style_violations(value) {
                    report.violation("parts", format!("{file}: 許されない行内の様式「{prop}」"));
                }
            }
        }
    }
}

/// 面の全部の開始タグから、検査する 3 つの属性の（名, 値）を出た順に取り出す。
/// 値が引用符で囲まれていない・値に「&」が在る（文字参照は解かない）は Err（その面はまだ分からない）。
fn face_attrs(tags: &[Tag]) -> Result<Vec<(&'static str, String)>, String> {
    let mut out = Vec::new();
    for tag in tags {
        for (name, value) in &tag.attrs {
            let Some(attr) = ATTRS.iter().find(|a| **a == name.as_str()) else {
                continue;
            };
            let AttrValue::Quoted(value) = value else {
                return Err(format!(
                    "<{}> の属性 {attr} の値が引用符で囲まれていない",
                    tag.name
                ));
            };
            if value.contains('&') {
                return Err(format!(
                    "<{}> の属性 {attr} の値に「&」が在る（文字参照は解かない）",
                    tag.name
                ));
            }
            out.push((*attr, value.clone()));
        }
    }
    Ok(out)
}

/// 属性 style の値を「;」で割り、違反する片の性質の名を出た順に返す。各片の最初の「:」より前（前後の空白を除く）を
/// 性質の名とし、許す一覧に無ければ違反。「:」の無い空でない片は、その字面が許す名と同じでも違反（性質の名として
/// 引かない・報告の名はその片の字面）。空の片と空白だけの片は数えない。
fn style_violations(value: &str) -> Vec<&str> {
    value
        .split(';')
        .map(str::trim)
        .filter(|piece| !piece.is_empty())
        .filter_map(|piece| match piece.split_once(':') {
            Some((name, _)) => {
                let name = name.trim();
                StyleProp::from_name(name).is_none().then_some(name)
            }
            None => Some(piece),
        })
        .collect()
}

// ── 面の読み方（手書きの走査）──

/// 開始タグの属性の値。
#[derive(Debug, PartialEq)]
enum AttrValue {
    /// 二重引用符か一重引用符で囲んだ値（引用符は含まない）
    Quoted(String),
    /// 引用符なしの値（中身は使わない＝検査する属性なら読めない扱い）
    Unquoted,
    /// 値の無い属性
    Bare,
}

/// 開始タグ 1 つ（名は小文字にする）。
#[derive(Debug)]
struct Tag {
    name: String,
    attrs: Vec<(String, AttrValue)>,
}

fn find(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    hay.get(from..)?
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

fn find_ignore_case(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    hay.get(from..)?
        .windows(needle.len())
        .position(|w| w.eq_ignore_ascii_case(needle))
        .map(|p| p + from)
}

/// 面の全部の開始タグを出た順に読む。注釈（<!-- から -->）と script 要素・style 要素の中身は読み飛ばす。
/// 閉じない注釈・閉じないタグ・閉じない引用符・閉じない script / style 要素は Err。
fn scan_tags(text: &str) -> Result<Vec<Tag>, String> {
    let b = text.as_bytes();
    let mut tags = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'<' {
            i += 1;
            continue;
        }
        if b[i..].starts_with(b"<!--") {
            i = find(b, b"-->", i + 4).ok_or("閉じない注釈")? + 3;
            continue;
        }
        if !b.get(i + 1).is_some_and(u8::is_ascii_alphabetic) {
            i += 1;
            continue;
        }
        let start = i + 1;
        i = start;
        while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'-') {
            i += 1;
        }
        let name = text[start..i].to_ascii_lowercase();
        let mut attrs = Vec::new();
        loop {
            while i < b.len() && b[i].is_ascii_whitespace() {
                i += 1;
            }
            match b.get(i) {
                None => return Err(format!("閉じないタグ <{name}>")),
                Some(b'>') => {
                    i += 1;
                    break;
                }
                Some(b'/') => {
                    i += 1;
                    continue;
                }
                Some(_) => {}
            }
            let ns = i;
            while i < b.len() && !b[i].is_ascii_whitespace() && !matches!(b[i], b'=' | b'>' | b'/')
            {
                i += 1;
            }
            if i == ns {
                return Err(format!("読めないタグ <{name}>（属性の名が無い）"));
            }
            let aname = text[ns..i].to_ascii_lowercase();
            let mut j = i;
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if b.get(j) != Some(&b'=') {
                attrs.push((aname, AttrValue::Bare));
                continue;
            }
            i = j + 1;
            while i < b.len() && b[i].is_ascii_whitespace() {
                i += 1;
            }
            match b.get(i) {
                Some(&q) if q == b'"' || q == b'\'' => {
                    let vs = i + 1;
                    let ve = find(b, &[q], vs)
                        .ok_or_else(|| format!("閉じない引用符（<{name}> の属性 {aname}）"))?;
                    attrs.push((aname, AttrValue::Quoted(text[vs..ve].to_string())));
                    i = ve + 1;
                }
                _ => {
                    while i < b.len() && !b[i].is_ascii_whitespace() && b[i] != b'>' {
                        i += 1;
                    }
                    attrs.push((aname, AttrValue::Unquoted));
                }
            }
        }
        if name == "script" || name == "style" {
            let close = format!("</{name}");
            i = find_ignore_case(b, close.as_bytes(), i)
                .ok_or_else(|| format!("閉じない {name} 要素"))?
                + close.len();
        }
        tags.push(Tag { name, attrs });
    }
    Ok(tags)
}

// ── 様式の定義の読み方（手書きの走査）──

/// 様式の定義の class の名の集合。注釈（/* から */）と二重引用符・一重引用符の文字列の中を読み飛ばし、残りの中で
/// 「.」の直後が英字か「_」か（「-」とその次が英字か「_」か「-」）で始まり、英数字・「_」・「-」が続く限りを 1 つの名とする。
/// 「.」の直後が数字（小数の値）は class ではない。閉じない注釈・閉じない文字列は Err。
fn css_classes(text: &str) -> Result<HashSet<String>, String> {
    let b = text.as_bytes();
    let mut set = HashSet::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'/' if b.get(i + 1) == Some(&b'*') => {
                i = find(b, b"*/", i + 2).ok_or("閉じない注釈")? + 2;
            }
            q @ (b'"' | b'\'') => {
                i += 1;
                loop {
                    match b.get(i) {
                        None => return Err("閉じない文字列".to_string()),
                        Some(b'\\') => i += 2,
                        Some(&c) if c == q => {
                            i += 1;
                            break;
                        }
                        Some(_) => i += 1,
                    }
                }
            }
            b'.' => {
                let s = i + 1;
                let ident = |c: &u8| c.is_ascii_alphabetic() || *c == b'_';
                let head = match b.get(s) {
                    Some(c) if ident(c) => true,
                    Some(b'-') => b.get(s + 1).is_some_and(|c| ident(c) || *c == b'-'),
                    _ => false,
                };
                if head {
                    let mut j = s;
                    while j < b.len()
                        && (b[j].is_ascii_alphanumeric() || matches!(b[j], b'_' | b'-'))
                    {
                        j += 1;
                    }
                    set.insert(text[s..j].to_string());
                    i = j;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    Ok(set)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(set: &HashSet<String>) -> Vec<&str> {
        let mut v: Vec<&str> = set.iter().map(String::as_str).collect();
        v.sort_unstable();
        v
    }

    #[test]
    fn parts_css_reader_skips_comments_strings_and_decimals() {
        let css = "/* .ghost{} */ .a{content:'.in-str';margin:.5em}\n.b-c::before{content:\".q\\\".r\"} .-d,.--e,._f{x:1.25}\n.-1{} .9x{} .a.b{}";
        let set = css_classes(css).unwrap();
        assert_eq!(sorted(&set), ["--e", "-d", "_f", "a", "b", "b-c"]);
        assert!(css_classes("a{} /* 閉じない").is_err());
        assert!(css_classes("a{content:'閉じない}").is_err());
        assert!(css_classes("a{content:\"閉じない\\\"}").is_err());
    }

    #[test]
    fn parts_page_reader_skips_comments_and_script_and_style() {
        let html = "<!DOCTYPE html>\n<!-- <p class=\"ghost-c\"> -->\n<html><head><script type='text/javascript'>var s='<p class=\"ghost-s\">';</script>\n<STYLE>.x{}</STYLE></head>\n<body><div class='a b' data-component=\"hub-cover\"><input disabled class=\"c\"/><p style=\"--rail-n:3\">x &lt; y</p></div><br/></body></html>";
        let tags = scan_tags(html).unwrap();
        let names: Vec<&str> = tags.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "html", "head", "script", "style", "body", "div", "input", "p", "br"
            ]
        );
        let attrs = face_attrs(&tags).unwrap();
        let attrs: Vec<(&str, &str)> = attrs.iter().map(|(k, v)| (*k, v.as_str())).collect();
        assert_eq!(
            attrs,
            [
                ("class", "a b"),
                ("data-component", "hub-cover"),
                ("class", "c"),
                ("style", "--rail-n:3"),
            ]
        );
        assert_eq!(
            tags[6].attrs,
            [
                ("disabled".to_string(), AttrValue::Bare),
                ("class".to_string(), AttrValue::Quoted("c".to_string())),
            ]
        );
    }

    #[test]
    fn parts_page_reader_rejects_unreadable_shapes() {
        for (html, why) in [
            ("<p class=a>x</p>", "引用符で囲まれていない"),
            ("<p class=\"a\">x</p><!-- 閉じない", "閉じない注釈"),
            ("<p class=\"a\"", "閉じないタグ"),
            ("<p class=\"a>x</p>", "閉じない引用符"),
            ("<p class='a\">x</p>", "閉じない引用符"),
            ("<script>var s='<p class=\"x\">';", "閉じない script"),
            ("<p class=\"a&amp;b\">x</p>", "「&」"),
            ("<p data-component=x>x</p>", "引用符で囲まれていない"),
        ] {
            let e = scan_tags(html).and_then(|t| face_attrs(&t)).unwrap_err();
            assert!(e.contains(why), "{html}: {e}");
        }
    }

    #[test]
    fn parts_style_violations_allow_only_listed_props_with_a_colon() {
        // 許す性質と値の片は違反 0（前後の空白は除く）・空の片と空白だけの片は数えない
        assert!(style_violations(" --rail-n : 3 ; --shelf-n:2;;  ; ").is_empty());
        assert!(style_violations("").is_empty());
        assert!(style_violations("  ").is_empty());
        // 許さない性質の片は違反
        assert_eq!(
            style_violations("--rail-n:3; color: red;margin:0"),
            ["color", "margin"]
        );
        // 「:」の無い空でない片は、許す名と同じ字面でも違反（性質の名として引かない）
        assert_eq!(style_violations("--shelf-n"), ["--shelf-n"]);
        assert_eq!(
            style_violations("--shelf-n:3; --shelf-n ; nocolon"),
            ["--shelf-n", "nocolon"]
        );
    }

    #[test]
    fn parts_catalog_names_are_derived_from_the_catalog() {
        assert_eq!(Component::ALL.len(), 29);
        assert_eq!(FigureType::ALL.len(), 8);
        assert_eq!(ShelfType::ALL.len(), 1);
        assert_eq!(StyleProp::ALL.len(), 4);
        assert_eq!(
            Component::from_name("pipeline-rail"),
            Some(Component::PipelineRail)
        );
        assert_eq!(Component::HubCover.faces(), ["index"]);
        assert_eq!(
            FigureType::from_name("archify-workflow").map(|t| t.name()),
            Some("archify-workflow")
        );
        assert_eq!(ShelfType::from_name("doc-shelf"), Some(ShelfType::DocShelf));
        assert_eq!(StyleProp::from_name("--rail-n"), Some(StyleProp::RailN));
        assert!(StyleProp::from_name("color").is_none());
    }
}
