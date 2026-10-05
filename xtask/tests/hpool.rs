//! 歯の群の共通の module の歯（接頭辞 hpool_・判断の記録 ADR-63 の決定 (8) の (c)）: 群の dir の共通の module（common.rs か common/mod.rs）の
//! 頭の段の helper の item（#[test] の付かない fn・const・static・type・struct・enum・union・trait・impl）は、同じ群のほかの .rs の頭の段の
//! helper の item と、字 pub(crate) と空白の並びを除いて同じ字にならない（畳みの道具が字で同じ写しを寄せた後に、写しが群に残らない）。
//! item は直上の // と /// の行と属性を含めて読み、注と字と文字の literal の中の括弧は数えない（畳みの道具の読みと同じ）。
//! 外の依存を使わず、repo の根（xtask の manifest の dir の 1 つ上）からの相対の path で字を読む。
#![cfg(test)]

use std::path::{Path, PathBuf};

#[expect(
    dead_code,
    reason = "member と群の探しだけを使い、上限の読みと命令の測りは使わない"
)]
#[path = "../src/kcap.rs"]
mod kcap;

/// 比べる item の種（頭の語）。use・mod・macro・内の属性は比べない。
const HELPER: [&str; 9] = [
    "fn", "const", "static", "type", "struct", "enum", "union", "trait", "impl",
];
/// 括弧の閉じで終わる item の種（ほかの種は ; で終わる）。
const BRACED: [&str; 9] = [
    "fn", "struct", "enum", "union", "trait", "impl", "mod", "extern", "call",
];
/// 頭の語の前に付く語（可視の pub の括弧は別に飛ばす）。
const QUALIFIERS: [&str; 3] = ["async", "unsafe", "default"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tk {
    Word,
    Punct(u8),
    Lit,
    Life,
}

#[derive(Clone, Copy, Debug)]
struct Tok {
    kind: Tk,
    at: usize,
    end: usize,
}

/// 頭の段の item（lo と hi は直上の // と /// の行を含む行の番号・0 始まり）。
#[derive(Debug, PartialEq, Eq)]
struct Item {
    kind: String,
    test: bool,
    lo: usize,
    hi: usize,
    text: String,
}

fn repo_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map_or_else(|| manifest.join(".."), Path::to_path_buf)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

fn is_word(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c >= 0x80
}

fn find_from(b: &[u8], from: usize, pat: &[u8]) -> Option<usize> {
    b.get(from..)?
        .windows(pat.len())
        .position(|w| w == pat)
        .map(|p| p + from)
}

fn word_end(b: &[u8], mut j: usize) -> usize {
    while b.get(j).is_some_and(|&c| is_word(c)) {
        j += 1;
    }
    j
}

/// 入れ子の注 /* … */ の終わりの次の位置。
fn block_end(b: &[u8], mut j: usize) -> usize {
    let mut depth = 0;
    while j < b.len() {
        if b[j..].starts_with(b"/*") {
            depth += 1;
            j += 2;
        } else if b[j..].starts_with(b"*/") {
            depth -= 1;
            j += 2;
            if depth == 0 {
                return j;
            }
        } else {
            j += 1;
        }
    }
    j
}

/// raw の字（b か c の頭も可・r と # の並びと二重引用符）の終わりの次の位置。raw の字でなければ None。
fn raw_end(b: &[u8], i: usize) -> Option<usize> {
    let mut j = i;
    if matches!(b.get(j), Some(b'b' | b'c')) {
        j += 1;
    }
    if b.get(j) != Some(&b'r') {
        return None;
    }
    let hashes = b.get(j + 1..)?.iter().take_while(|&&c| c == b'#').count();
    j += 1 + hashes;
    if b.get(j) != Some(&b'"') {
        return None;
    }
    let mut close = vec![b'"'];
    close.extend(std::iter::repeat_n(b'#', hashes));
    find_from(b, j + 1, &close).map(|k| k + close.len())
}

/// 二重引用符の後の位置 j から、逆斜線の escape を飛ばして閉じの二重引用符の次の位置。
fn str_end(b: &[u8], mut j: usize) -> usize {
    while j < b.len() && b[j] != b'"' {
        j += if b[j] == b'\\' { 2 } else { 1 };
    }
    j + 1
}

/// 一重引用符の位置 k からの文字の literal か lifetime。
fn quote(text: &str, k: usize) -> (Tk, usize) {
    let b = text.as_bytes();
    if b.get(k + 1) == Some(&b'\\') {
        return (
            Tk::Lit,
            find_from(b, k + 3, b"'").map_or(b.len(), |e| e + 1),
        );
    }
    let width = text
        .get(k + 1..)
        .and_then(|s| s.chars().next())
        .map_or(1, char::len_utf8);
    if b.get(k + 1 + width) == Some(&b'\'') {
        return (Tk::Lit, k + 2 + width);
    }
    (Tk::Life, word_end(b, k + 1))
}

/// 位置 i からの token の種（空白と注は None）と終わりの次の位置。
fn next_tok(text: &str, i: usize) -> (Option<Tk>, usize) {
    let b = text.as_bytes();
    let c = b[i];
    let glued = i > 0 && is_word(b[i - 1]);
    let next = b.get(i + 1).copied();
    if c.is_ascii_whitespace() {
        return (None, i + 1);
    }
    if b[i..].starts_with(b"//") {
        return (None, find_from(b, i, b"\n").unwrap_or(b.len()));
    }
    if b[i..].starts_with(b"/*") {
        return (None, block_end(b, i));
    }
    if let Some(end) = raw_end(b, i).filter(|_| !glued) {
        return (Some(Tk::Lit), end);
    }
    if c == b'"' || (!glued && matches!(c, b'b' | b'c') && next == Some(b'"')) {
        return (
            Some(Tk::Lit),
            str_end(b, if c == b'"' { i + 1 } else { i + 2 }),
        );
    }
    if c == b'\'' || (!glued && c == b'b' && next == Some(b'\'')) {
        let (kind, end) = quote(text, if c == b'b' { i + 1 } else { i });
        return (Some(kind), end);
    }
    if c.is_ascii_digit() {
        let mut j = word_end(b, i);
        if b.get(j) == Some(&b'.') && b.get(j + 1).is_some_and(u8::is_ascii_digit) {
            j = word_end(b, j + 1);
        }
        return (Some(Tk::Lit), j);
    }
    if is_word(c) {
        return (Some(Tk::Word), word_end(b, i));
    }
    (Some(Tk::Punct(c)), i + 1)
}

/// Rust の字の token の列（注を捨て、字と文字の literal を 1 つの token にする）。
fn lex(text: &str) -> Vec<Tok> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let (kind, end) = next_tok(text, i);
        if let Some(kind) = kind {
            out.push(Tok { kind, at: i, end });
        }
        i = end.max(i + 1);
    }
    out
}

fn word<'a>(text: &'a str, toks: &[Tok], j: usize) -> Option<&'a str> {
    toks.get(j)
        .filter(|t| t.kind == Tk::Word)
        .map(|t| &text[t.at..t.end])
}

fn punct(toks: &[Tok], j: usize, c: u8) -> bool {
    toks.get(j).is_some_and(|t| t.kind == Tk::Punct(c))
}

/// toks[k] の開き括弧の対の閉じ括弧の index。
fn close(toks: &[Tok], k: usize) -> usize {
    let mut depth = 0usize;
    for (j, t) in toks.iter().enumerate().skip(k) {
        match t.kind {
            Tk::Punct(b'(' | b'[' | b'{') => depth += 1,
            Tk::Punct(b')' | b']' | b'}') => {
                depth -= 1;
                if depth == 0 {
                    return j;
                }
            }
            _ => {}
        }
    }
    panic!("閉じない括弧")
}

/// 属性の並び（toks[k] から）の後の index と、#[test] の有無と、内の属性（#!）か。
fn attrs(text: &str, toks: &[Tok], mut k: usize) -> (usize, bool, bool) {
    let mut test = false;
    while punct(toks, k, b'#') {
        let inner = punct(toks, k + 1, b'!');
        let open = if inner { k + 2 } else { k + 1 };
        let end = close(toks, open);
        test |= end == open + 2 && word(text, toks, open + 1) == Some("test");
        k = end + 1;
        if inner {
            return (k, test, true);
        }
    }
    (k, test, false)
}

/// item の頭（属性の後の toks[k] から）の種の語。macro の呼びは call、読めない頭は other。
fn head_kind(text: &str, toks: &[Tok], mut k: usize) -> String {
    if word(text, toks, k) == Some("pub") {
        k += 1;
        if punct(toks, k, b'(') {
            k = close(toks, k) + 1;
        }
    }
    loop {
        let w = word(text, toks, k);
        let next = word(text, toks, k + 1);
        if w.is_some_and(|w| QUALIFIERS.contains(&w))
            || (w == Some("const") && next.is_some_and(|n| ["fn", "unsafe", "async"].contains(&n)))
        {
            k += 1;
        } else {
            break;
        }
    }
    match word(text, toks, k) {
        Some(_) if punct(toks, k + 1, b'!') && !punct(toks, k + 2, b'=') => "call".to_string(),
        Some(w) => w.to_string(),
        None => "other".to_string(),
    }
}

/// 種 kind の item の頭（toks[k]）からの終わりの token の index（; か、括弧で終わる種の最初の波括弧の閉じ）。
fn item_end(toks: &[Tok], kind: &str, mut j: usize) -> usize {
    while j < toks.len() {
        match toks[j].kind {
            Tk::Punct(b';') => return j,
            Tk::Punct(b'{') if BRACED.contains(&kind) => return close(toks, j),
            Tk::Punct(b'(' | b'[' | b'{') => j = close(toks, j) + 1,
            _ => j += 1,
        }
    }
    panic!("終わらない item（{kind}）")
}

/// 行の頭の位置の列。
fn line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

/// 直上の // と /// の行（//! を除き、前の item の行に入らない）を含めた item の最初の行。
fn lead(lines: &[&str], first: usize, last: Option<usize>) -> usize {
    let mut lo = first;
    while lo > last.map_or(0, |l| l + 1) {
        let above = lines[lo - 1].trim();
        if !above.starts_with("//") || above.starts_with("//!") {
            break;
        }
        lo -= 1;
    }
    lo
}

/// file の頭の段の item の列。
fn items(text: &str) -> Vec<Item> {
    let toks = lex(text);
    let starts = line_starts(text);
    let line_of = |at: usize| starts.partition_point(|&s| s <= at) - 1;
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out = Vec::new();
    let (mut k, mut last) = (0, None);
    while k < toks.len() {
        let (body, test, inner) = attrs(text, &toks, k);
        let (kind, end) = if inner {
            ("inner".to_string(), body - 1)
        } else {
            let kind = head_kind(text, &toks, body);
            let end = item_end(&toks, &kind, body);
            (kind, end)
        };
        let (first, hi) = (line_of(toks[k].at), line_of(toks[end].at));
        let lo = lead(&lines, first, last);
        out.push(Item {
            kind,
            test,
            lo,
            hi,
            text: lines[lo..=hi].join("\n"),
        });
        last = Some(hi);
        k = end + 1;
    }
    out
}

/// 比べる helper の item（種が HELPER で #[test] の付かない物）。
fn helpers(text: &str) -> Vec<Item> {
    items(text)
        .into_iter()
        .filter(|it| !it.test && HELPER.contains(&it.kind.as_str()))
        .collect()
}

/// 字 pub(crate) と空白の並びを除いた字（畳みの道具は寄せる item に pub(crate) を足す）。
fn plain(text: &str) -> String {
    text.replace("pub(crate) ", "")
}

/// 共通の module の helper の item と同じ字（plain で比べる）の helper の item を持つ file の名と行（1 始まり）の列。
fn copies(common: &str, others: &[(String, String)]) -> Vec<String> {
    let pooled: Vec<String> = helpers(common).iter().map(|it| plain(&it.text)).collect();
    let mut out = Vec::new();
    for (name, text) in others {
        for it in helpers(text) {
            if pooled.contains(&plain(&it.text)) {
                out.push(format!("{name}:{}", it.lo + 1));
            }
        }
    }
    out
}

/// 共通の module を持つ群の (群の dir, 共通の module の file) の列（member の順・群の path の順）。
fn pools() -> Vec<(PathBuf, PathBuf)> {
    let root = repo_root();
    let mut out = Vec::new();
    for member in kcap::members(&root).expect("members の表") {
        for group in kcap::groups(&root, &member) {
            let file = [group.join("common/mod.rs"), group.join("common.rs")]
                .into_iter()
                .find(|f| f.is_file());
            if let Some(file) = file {
                out.push((group, file));
            }
        }
    }
    out
}

/// 群の dir の直下の .rs のうち main.rs と共通の module を除く物の (名, 字)。
fn others(group: &Path, pool: &Path) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = std::fs::read_dir(group)
        .expect("群の dir を読む")
        .map(|e| e.expect("群の項目").path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .filter(|p| p.file_name().and_then(|n| n.to_str()) != Some("main.rs") && p != pool)
        .map(|p| (p.display().to_string(), read(&p)))
        .collect();
    out.sort();
    out
}

#[test]
fn hpool_common_items_have_no_copy_in_their_group() {
    let (mut groups, mut pooled) = (0, 0);
    for (group, pool) in pools() {
        let common = read(&pool);
        let found = copies(&common, &others(&group, &pool));
        assert!(found.is_empty(), "{} の写し: {found:?}", pool.display());
        groups += 1;
        pooled += helpers(&common).len();
    }
    assert!(
        groups >= GROUPS_MIN && pooled >= POOLED_MIN,
        "共通の module を持つ群 {groups}・helper の item {pooled}"
    );
}

/// 今の木で共通の module を持つ群と、その helper の item の数の下限（寄せの行が足すと増える）。
const GROUPS_MIN: usize = 5;
const POOLED_MIN: usize = 49;

/// 読みの見本: 頭の注・内の属性・use・直上の /// と // の行と属性を持つ歯の fn（字と raw の字と文字の literal と入れ子の注の中に
/// 括弧と fn の字を持つ）・括弧を持つ const・struct・impl・macro。
const SAMPLE: &str = r##"//! 頭
#![expect(dead_code, reason = "見本")]

use std::fs;

/// 一
// 二
#[test]
fn a() {
    let s = "} fn x() {";
    let r = r#"}"{"#;
    let c = '{';
    let l: &'static str = "";
    /* } /* { */ */
}
const B: [u8; 2] = [1, 2];
struct C {
    d: u8,
}
impl C {
    fn e(&self) -> u8 {
        self.d
    }
}
macro_rules! m {
    () => {};
}
"##;

#[test]
fn hpool_items_read_lead_comments_attributes_and_literals() {
    let got: Vec<(String, bool, usize, usize)> = items(SAMPLE)
        .into_iter()
        .map(|it| (it.kind, it.test, it.lo, it.hi))
        .collect();
    let want = [
        ("inner", false, 1, 1),
        ("use", false, 3, 3),
        ("fn", true, 5, 14),
        ("const", false, 15, 15),
        ("struct", false, 16, 18),
        ("impl", false, 19, 23),
        ("call", false, 24, 26),
    ]
    .map(|(k, t, lo, hi)| (k.to_string(), t, lo, hi));
    assert_eq!(got, want);
    assert_eq!(
        helpers(SAMPLE).len(),
        3,
        "比べる item は const と struct と impl だけ（歯の fn と use と macro と内の属性は比べない）"
    );
}

/// 写しの見本の共通の module（直上の /// の行と pub(crate) を足した fn）。
const COMMON: &str = "//! 共通\n#![cfg(test)]\n\n/// 一つ\npub(crate) fn one() -> u8 {\n    1\n}\n";

#[test]
fn hpool_one_clause_mutants_are_named() {
    let others = [
        ("copy.rs", "/// 一つ\nfn one() -> u8 {\n    1\n}\n"),
        (
            "vis.rs",
            "use std::fs;\n\n/// 一つ\npub(crate) fn one() -> u8 {\n    1\n}\n",
        ),
        ("body.rs", "/// 一つ\nfn one() -> u8 {\n    2\n}\n"),
        ("doc.rs", "/// 二つ\nfn one() -> u8 {\n    1\n}\n"),
        (
            "lit.rs",
            "const S: &str = r#\"/// 一つ\nfn one() -> u8 {\n    1\n}\"#;\n",
        ),
        (
            "nested.rs",
            "mod inner {\n/// 一つ\nfn one() -> u8 {\n    1\n}\n}\n",
        ),
        ("test.rs", "/// 一つ\n#[test]\nfn one() -> u8 {\n    1\n}\n"),
    ]
    .map(|(n, t)| (n.to_string(), t.to_string()));
    assert_eq!(copies(COMMON, &others), ["copy.rs:1", "vis.rs:3"]);
}
