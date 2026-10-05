//! 行 g-vocab-sweep の歯（接頭辞 bvlex_）: 語の辞書（vocab.json）の鍵の全部と stylesheet（style.css）の選びの class の全部が、
//! 面の src の字から引かれる（src の字の literal か、この file の一覧の接頭辞で組む所）ことを見る。src は注と char の literal を飛ばして読む。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::common::crate_dir;
use tsuzuri_surface::vocab::vocab;

/// 鍵を組む接頭辞（接頭辞と字をつないで語の辞書を引く所: 地図の辺の型・台帳の局面と手番と未反映の種類・吹き出しの待ちの理由と受付の断りの名）。
const KEY_PREFIXES: [&str; 6] = ["e:", "lc:", "qr:", "rf:", "turn:", "unref:"];

/// class を組む接頭辞（接頭辞と値の字をつなぐ所・値の字は src の字の literal に在る）。
const CLASS_PREFIXES: [&str; 7] = ["c-", "fb-", "h-", "r-", "sd-", "sev-", "st-"];

/// src から引かれないが残す鍵（着地済みの歯 helpfig が字で名指す 2 つ・accthome が字で名指す 1 つ・測れていないを描く語 1 つ）。
const KEEP_KEYS: [&str; 4] = ["memo_promo", "move_state", "nx_na", "pressure"];

/// src から引かれないが残す class（着地済みの歯 frame が字で名指す l4・まだ分からないの meter・行 g-accept-fix の差の中の行の 2 つ・
/// 着地済みの歯が stylesheet に求める src の class c-sp・cmd・off の唯一の規則の祖先の 3 つ）。
const KEEP_CLASSES: [&str; 7] = [
    "hbdlg",
    "l4",
    "lrow",
    "m-grp",
    "m-need",
    "nxlist",
    "unmeasured",
];

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

/// dir の下の .rs の file の全部（名の順）。
fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{} を読む: {e}", dir.display()))
        .map(|e| e.expect("dir の項").path())
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

fn ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// 読みの 1 歩（飛ばして次の位置・字の literal の中身の頭と末と次の位置・class:名 の名と次の位置）。
enum Step {
    Skip(usize),
    Lit(usize, usize, usize),
    Dir(String, usize),
}

/// 字の literal の中身（"…"・b"…"・r#"…"#）と、literal の外の `class:名` の名（注と char の literal は飛ばす）。
fn lex(text: &str) -> (Vec<String>, Vec<String>) {
    let s: Vec<char> = text.chars().collect();
    let (mut lits, mut dirs) = (Vec::new(), Vec::new());
    let mut i = 0;
    while i < s.len() {
        i = match step(&s, i) {
            Step::Skip(j) => j,
            Step::Lit(body, end, j) => {
                lits.push(s[body..end].iter().collect());
                j
            }
            Step::Dir(name, j) => {
                dirs.push(name);
                j
            }
        };
    }
    (lits, dirs)
}

fn step(s: &[char], i: usize) -> Step {
    let after_ident = i > 0 && ident(s[i - 1]);
    match (s.get(i), s.get(i + 1)) {
        (Some('/'), Some('/')) => Step::Skip(line_end(s, i)),
        (Some('/'), Some('*')) => Step::Skip(block_end(s, i)),
        (Some('\''), _) => Step::Skip(char_end(s, i)),
        (Some('"'), _) => quoted(s, i + 1),
        (Some('b'), Some('"')) if !after_ident => quoted(s, i + 2),
        _ if !after_ident => raw(s, i)
            .or_else(|| directive(s, i))
            .unwrap_or(Step::Skip(i + 1)),
        _ => Step::Skip(i + 1),
    }
}

/// 行の注の末（改行の位置）。
fn line_end(s: &[char], i: usize) -> usize {
    s[i..]
        .iter()
        .position(|c| *c == '\n')
        .map_or(s.len(), |k| i + k)
}

/// 塊の注の末の次（入れ子を数える）。
fn block_end(s: &[char], mut i: usize) -> usize {
    let mut depth = 0;
    while i < s.len() {
        match (s[i], s.get(i + 1)) {
            ('/', Some('*')) => depth += 1,
            ('*', Some('/')) => depth -= 1,
            _ => {
                i += 1;
                continue;
            }
        }
        i += 2;
        if depth == 0 {
            return i;
        }
    }
    s.len()
}

/// char の literal（'x'・'\n'・'\u{..}'・'\''）の次の位置。lifetime（'a）は 1 字だけ進む。
fn char_end(s: &[char], i: usize) -> usize {
    if s.get(i + 1) == Some(&'\\') {
        let rest = s.get(i + 3..).unwrap_or_default();
        rest.iter()
            .position(|c| *c == '\'')
            .map_or(s.len(), |k| i + 3 + k + 1)
    } else if s.get(i + 2) == Some(&'\'') {
        i + 3
    } else {
        i + 1
    }
}

/// 引用符の中の字（逆斜線の次の 1 字は飛ばす）。
fn quoted(s: &[char], body: usize) -> Step {
    let mut j = body;
    while j < s.len() && s[j] != '"' {
        j += if s[j] == '\\' { 2 } else { 1 };
    }
    Step::Lit(body, j.min(s.len()), j + 1)
}

/// raw の字の literal（`r"`・`r#"`・`br"` ほか）。
fn raw(s: &[char], i: usize) -> Option<Step> {
    let r = if s.get(i) == Some(&'b') { i + 1 } else { i };
    if s.get(r) != Some(&'r') {
        return None;
    }
    let hashes = s[r + 1..].iter().take_while(|c| **c == '#').count();
    let body = r + 1 + hashes + 1;
    if s.get(body - 1) != Some(&'"') {
        return None;
    }
    let closes = |j: &usize| s[*j] == '"' && (1..=hashes).all(|h| s.get(j + h) == Some(&'#'));
    let end = (body..s.len()).find(closes).unwrap_or(s.len());
    Some(Step::Lit(body, end, end + 1 + hashes))
}

/// literal の外の `class:名`（view! の class の指示）。
fn directive(s: &[char], i: usize) -> Option<Step> {
    if !s[i..].starts_with(&['c', 'l', 'a', 's', 's', ':']) {
        return None;
    }
    let name: String = s[i + 6..]
        .iter()
        .take_while(|c| c.is_ascii_alphanumeric() || **c == '-' || **c == '_')
        .collect();
    let next = i + 6 + name.chars().count();
    (!name.is_empty()).then_some(Step::Dir(name, next))
}

/// 面の src の読み（字の literal・literal の中の語・class:名 の名）。
struct Src {
    lits: BTreeSet<String>,
    words: BTreeSet<String>,
    dirs: BTreeSet<String>,
}

fn src() -> Src {
    let mut files = Vec::new();
    rs_files(&crate_dir().join("src"), &mut files);
    assert!(files.len() > 50, "src の .rs が少ない: {}", files.len());
    let (mut lits, mut words, mut dirs) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    for f in &files {
        let (ls, ds) = lex(&read(f));
        for l in ls {
            words.extend(
                l.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
                    .filter(|w| !w.is_empty())
                    .map(str::to_string),
            );
            lits.insert(l);
        }
        dirs.extend(ds);
    }
    Src { lits, words, dirs }
}

/// stylesheet の選びの class の名（注を除いた字の `.名`・名は字か `-` か `_` で始まる）。
fn css_classes() -> BTreeSet<String> {
    let css = read(&crate_dir().join("style.css"));
    let mut text = String::new();
    let mut rest = css.as_str();
    while let Some(i) = rest.find("/*") {
        text.push_str(&rest[..i]);
        rest = rest[i..].find("*/").map_or("", |j| &rest[i + j + 2..]);
    }
    text.push_str(rest);
    let chars: Vec<char> = text.chars().collect();
    let mut out = BTreeSet::new();
    for i in 0..chars.len() {
        let starts = chars
            .get(i + 1)
            .is_some_and(|c| c.is_ascii_alphabetic() || *c == '_' || *c == '-');
        if chars[i] == '.' && starts {
            out.insert(
                chars[i + 1..]
                    .iter()
                    .take_while(|c| c.is_ascii_alphanumeric() || **c == '-' || **c == '_')
                    .collect(),
            );
        }
    }
    out
}

fn key_named(src: &Src, key: &str) -> bool {
    src.lits.contains(key) || KEY_PREFIXES.iter().any(|p| key.starts_with(p))
}

fn class_named(src: &Src, class: &str) -> bool {
    src.words.contains(class)
        || src.dirs.contains(class)
        || CLASS_PREFIXES.iter().any(|p| {
            class
                .strip_prefix(p)
                .is_some_and(|rest| src.lits.contains(rest))
        })
}

/// (2) 語の辞書の鍵の全部が、src の字の literal か鍵の接頭辞で引かれる（残す一覧の 4 つを除く）。
#[test]
fn bvlex_keys_named_in_src() {
    let src = src();
    let keys: Vec<&str> = vocab().keys().collect();
    assert!(keys.len() > 300, "語の辞書の鍵が少ない: {}", keys.len());
    let loose: Vec<&str> = keys
        .iter()
        .copied()
        .filter(|k| !key_named(&src, k) && !KEEP_KEYS.contains(k))
        .collect();
    assert!(loose.is_empty(), "src から引かれない鍵: {loose:?}");
}

/// (3) stylesheet の選びの class の全部が、src の字の literal の語か class:名 か、class の接頭辞と src の literal の字で引かれる（残す一覧の 7 つを除く）。
#[test]
fn bvlex_classes_named_in_src() {
    let src = src();
    let classes = css_classes();
    assert!(
        classes.len() > 400,
        "stylesheet の class が少ない: {}",
        classes.len()
    );
    let loose: Vec<&String> = classes
        .iter()
        .filter(|c| !class_named(&src, c) && !KEEP_CLASSES.contains(&c.as_str()))
        .collect();
    assert!(loose.is_empty(), "src から引かれない class: {loose:?}");
}

/// 字の literal が接頭辞で語を組む所か（接頭辞そのものの字か、語の頭に接頭辞を置き直後に { を開く format の字）。
fn builds(lit: &str, prefix: &str) -> bool {
    let word = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':';
    lit == prefix
        || lit
            .match_indices(&format!("{prefix}{{"))
            .any(|(k, _)| !lit[..k].chars().next_back().is_some_and(word))
}

/// (4) 接頭辞の一覧の全部が src で組む所を持ち（接頭辞そのものの字の literal か、語の頭に接頭辞を置き直後に { を開く format の字）、
/// どの接頭辞も鍵か class の 1 つ以上を引く。
#[test]
fn bvlex_prefix_sites_in_src() {
    let src = src();
    for p in KEY_PREFIXES.iter().chain(&CLASS_PREFIXES) {
        let site = src.lits.iter().any(|l| builds(l, p));
        assert!(site, "接頭辞 {p} を組む所が src に無い");
    }
    let keys: Vec<&str> = vocab().keys().collect();
    for p in KEY_PREFIXES {
        assert!(
            keys.iter().any(|k| k.starts_with(p)),
            "接頭辞 {p} の鍵が無い"
        );
    }
    let classes = css_classes();
    for p in CLASS_PREFIXES {
        assert!(
            classes.iter().any(|c| c.starts_with(p)),
            "接頭辞 {p} の class が無い"
        );
    }
}

/// (5) 残す一覧の名は語の辞書か stylesheet に在り、src からは引かれない（引かれたら一覧から外す）。
#[test]
fn bvlex_keep_lists_unnamed() {
    let src = src();
    let keys: Vec<&str> = vocab().keys().collect();
    for k in KEEP_KEYS {
        assert!(keys.contains(&k), "残す鍵 {k} が語の辞書に無い");
        assert!(!key_named(&src, k), "残す鍵 {k} は src から引かれる");
    }
    let classes = css_classes();
    for c in KEEP_CLASSES {
        assert!(classes.contains(c), "残す class {c} が stylesheet に無い");
        assert!(!class_named(&src, c), "残す class {c} は src から引かれる");
    }
}

/// (6) src の読みは注と char の literal を飛ばし、raw の字の literal と class:名 を拾う。組む所の見分けは語の途中の接頭辞を数えない。
#[test]
fn bvlex_lexer_reads_literals() {
    let text = concat!(
        "let q = '\"'; let e = '\\''; // \"in-line-note\"\n",
        "/* \"block-note\" /* \"nested\" */ */ fn f<'a>(x: &'a str) {}\n",
        "let r = r#\"raw \"q\" word\"#; let b = b\"bytes\";\n",
        "view! { <b class:ring=on class=\"one two\">\"text\"</b> }\n",
    );
    let (lits, dirs) = lex(text);
    assert_eq!(lits, ["raw \"q\" word", "bytes", "one two", "text"]);
    assert_eq!(dirs, ["ring"]);
    assert!(builds("col c-{}{empty}", "c-") && builds("r-", "r-"));
    assert!(!builds("abc-{}", "c-") && !builds("file:{}", "e:"));
}
