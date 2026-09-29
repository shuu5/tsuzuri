//! 入口の棚のカード全体の当たり判定 sc-hit の歯（便 143・docs/design/delivery-143.md §1 (c)）。binary を起動しない。
//! ブラウザの代わりに CSS の決まりの小さな写しで、面の要素に効く宣言を決める。写しの決まり:
//! - !important が勝つ・次に詳細度・同じなら後に書いた方（:not / :is / :has は引数の最大・:where は 0）
//! - 字に print を含む @media の塊は丸ごと読まない・他の @media と @supports は読む・他の @ の塊は読まない
//! - 注と擬似要素の selector は読まない・要素の style 属性（inline）は読まない
//! - 状態の擬似 class（hover など）は当たると読む（通さない側）
//! - 右端が当たるのに兄弟の結合子や構造の擬似 class で決められない selector は、黙って外さず Err を返す
//!
//! 写しは position・inset・z-index・display の勝ち負けだけを見る（inset は字だけ・top などの個別の辺は見ない）。
//! 凍結の面（tests/fixtures/face/）と正本の様式（design-intent/preview/folio.css）は読むだけで書き換えない。

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

// ---- 面の木（開始 tag と終了 tag だけを読む小さな写し）----

struct El {
    tag: String,
    attrs: Vec<(String, String)>,
    parent: Option<usize>,
}

impl El {
    fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    fn has_class(&self, class: &str) -> bool {
        self.attr("class")
            .is_some_and(|v| v.split_whitespace().any(|c| c == class))
    }
}

const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];
const RAW: &[&str] = &["script", "style"];

fn parse_html(s: &str) -> Vec<El> {
    let b = s.as_bytes();
    let mut els: Vec<El> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut i = 0;
    while let Some(off) = s[i..].find('<') {
        let at = i + off;
        let rest = &s[at..];
        if rest.starts_with("<!--") {
            i = at + rest.find("-->").map_or(rest.len(), |e| e + 3);
            continue;
        }
        if rest.starts_with("<!") || rest.starts_with("<?") {
            i = at + rest.find('>').map_or(rest.len(), |e| e + 1);
            continue;
        }
        if let Some(r) = rest.strip_prefix("</") {
            let end = r.find('>').unwrap_or(r.len());
            let name = r[..end].trim().to_ascii_lowercase();
            if let Some(pos) = stack.iter().rposition(|&k| els[k].tag == name) {
                stack.truncate(pos);
            }
            i = (at + 2 + end + 1).min(s.len());
            continue;
        }
        if !b.get(at + 1).is_some_and(u8::is_ascii_alphabetic) {
            i = at + 1;
            continue;
        }
        // 開始 tag: 名と属性（引用符の中の > は tag の終わりでない）
        let mut j = at + 1;
        while j < b.len() && !b[j].is_ascii_whitespace() && b[j] != b'>' && b[j] != b'/' {
            j += 1;
        }
        let tag = s[at + 1..j].to_ascii_lowercase();
        let mut attrs = Vec::new();
        let mut self_close = false;
        loop {
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if j >= b.len() {
                break;
            }
            if b[j] == b'>' {
                j += 1;
                break;
            }
            if b[j] == b'/' {
                self_close = true;
                j += 1;
                continue;
            }
            let n0 = j;
            while j < b.len() && !b[j].is_ascii_whitespace() && !matches!(b[j], b'=' | b'>' | b'/')
            {
                j += 1;
            }
            let name = s[n0..j].to_ascii_lowercase();
            let mut value = String::new();
            if j < b.len() && b[j] == b'=' {
                j += 1;
                if j < b.len() && (b[j] == b'"' || b[j] == b'\'') {
                    let q = b[j];
                    let v0 = j + 1;
                    j = v0;
                    while j < b.len() && b[j] != q {
                        j += 1;
                    }
                    value = s[v0..j].to_string();
                    j += 1;
                } else {
                    let v0 = j;
                    while j < b.len() && !b[j].is_ascii_whitespace() && b[j] != b'>' {
                        j += 1;
                    }
                    value = s[v0..j].to_string();
                }
            }
            attrs.push((name, value));
        }
        let idx = els.len();
        els.push(El {
            tag: tag.clone(),
            attrs,
            parent: stack.last().copied(),
        });
        i = j.min(s.len());
        if RAW.contains(&tag.as_str()) {
            let close = format!("</{tag}");
            i += s[i..].find(&close).unwrap_or(s.len() - i);
        } else if !self_close && !VOID.contains(&tag.as_str()) {
            stack.push(idx);
        }
    }
    els
}

fn is_descendant(els: &[El], d: usize, a: usize) -> bool {
    let mut p = els[d].parent;
    while let Some(k) = p {
        if k == a {
            return true;
        }
        p = els[k].parent;
    }
    false
}

// ---- 様式の定義（規則と宣言）----

struct Decl {
    name: String,
    value: String,
    important: bool,
}

struct Rule {
    sels: Vec<(String, Complex)>,
    decls: Vec<Decl>,
    order: usize,
}

/// 注を外す（引用符の中の /* は注でない）。
fn strip_comments(css: &str) -> Vec<char> {
    let c: Vec<char> = css.chars().collect();
    let mut out = Vec::with_capacity(c.len());
    let mut quote: Option<char> = None;
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        if let Some(q) = quote {
            out.push(ch);
            if ch == '\\' && i + 1 < c.len() {
                out.push(c[i + 1]);
                i += 1;
            } else if ch == q {
                quote = None;
            }
        } else if ch == '/' && c.get(i + 1) == Some(&'*') {
            i += 2;
            while i < c.len() && !(c[i] == '*' && c.get(i + 1) == Some(&'/')) {
                i += 1;
            }
            i += 1;
        } else {
            if ch == '"' || ch == '\'' {
                quote = Some(ch);
            }
            out.push(ch);
        }
        i += 1;
    }
    out
}

/// i から先で、引用符の外・括弧（() と []）の外に在る最初の stops の字の位置。
fn top_level(s: &[char], from: usize, stops: &[char]) -> Option<usize> {
    let mut quote: Option<char> = None;
    let mut depth = 0i32;
    let mut i = from;
    while i < s.len() {
        let ch = s[i];
        if let Some(q) = quote {
            if ch == '\\' {
                i += 1;
            } else if ch == q {
                quote = None;
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if ch == '(' || ch == '[' {
            depth += 1;
        } else if ch == ')' || ch == ']' {
            depth -= 1;
        } else if depth == 0 && stops.contains(&ch) {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn close_brace(s: &[char], open: usize) -> usize {
    let mut quote: Option<char> = None;
    let mut depth = 0i32;
    let mut i = open;
    while i < s.len() {
        let ch = s[i];
        if let Some(q) = quote {
            if ch == '\\' {
                i += 1;
            } else if ch == q {
                quote = None;
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return i;
            }
        }
        i += 1;
    }
    s.len()
}

fn split_top(s: &[char], sep: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(j) = top_level(s, i, &[sep]) {
        out.push(s[i..j].iter().collect::<String>().trim().to_string());
        i = j + 1;
    }
    out.push(s[i..].iter().collect::<String>().trim().to_string());
    out.retain(|p| !p.is_empty());
    out
}

fn parse_css(css: &str) -> Vec<Rule> {
    let s = strip_comments(css);
    let mut rules = Vec::new();
    parse_block(&s, &mut rules);
    rules
}

fn parse_block(s: &[char], rules: &mut Vec<Rule>) {
    let mut i = 0;
    while let Some(j) = top_level(s, i, &['{', ';']) {
        let prelude = s[i..j].iter().collect::<String>().trim().to_string();
        if s[j] == ';' {
            i = j + 1;
            continue;
        }
        let k = close_brace(s, j);
        let body = &s[j + 1..k.min(s.len())];
        if let Some(at) = prelude.strip_prefix('@') {
            let name = at
                .chars()
                .take_while(|c| c.is_ascii_alphabetic() || *c == '-')
                .collect::<String>()
                .to_ascii_lowercase();
            let read = match name.as_str() {
                "media" => !prelude.to_ascii_lowercase().contains("print"),
                "supports" => true,
                _ => false,
            };
            if read {
                parse_block(body, rules);
            }
        } else {
            let pc: Vec<char> = prelude.chars().collect();
            let sels = split_top(&pc, ',')
                .into_iter()
                .map(|t| {
                    let c = parse_selector(&t);
                    (t, c)
                })
                .collect();
            let order = rules.len();
            rules.push(Rule {
                sels,
                decls: parse_decls(body),
                order,
            });
        }
        i = k + 1;
        if i >= s.len() {
            break;
        }
    }
}

fn parse_decls(body: &[char]) -> Vec<Decl> {
    split_top(body, ';')
        .into_iter()
        .filter_map(|d| {
            let (name, value) = d.split_once(':')?;
            let mut value = value.trim();
            let mut important = false;
            if let Some(bang) = value.rfind('!')
                && value[bang + 1..].trim().eq_ignore_ascii_case("important")
            {
                important = true;
                value = value[..bang].trim();
            }
            Some(Decl {
                name: name.trim().to_ascii_lowercase(),
                value: value.to_string(),
                important,
            })
        })
        .collect()
}

// ---- selector ----

enum Simple {
    Type(String),
    Any,
    Class(String),
    Id(String),
    Attr(String, String, String),
    Pseudo(String),
    Func(String, Vec<Rel>),
    Elem,
    Opaque,
}

/// 関数の擬似 class の引数 1 つ。lead は :has の相対の結合子（' ' '>' '+' '~'）。
struct Rel {
    lead: char,
    sel: Complex,
}

struct Complex {
    parts: Vec<Vec<Simple>>,
    combs: Vec<char>,
    pseudo_element: bool,
}

const LEGACY_ELEM: &[&str] = &["before", "after", "first-line", "first-letter"];
const STATE: &[&str] = &[
    "hover",
    "focus",
    "focus-visible",
    "focus-within",
    "active",
    "visited",
    "link",
    "any-link",
    "target",
    "checked",
    "open",
    "popover-open",
];

fn ident(c: &[char], mut i: usize) -> (String, usize) {
    let mut out = String::new();
    while i < c.len() {
        let ch = c[i];
        if ch == '\\' && i + 1 < c.len() {
            out.push(c[i + 1]);
            i += 2;
        } else if ch.is_alphanumeric() || ch == '-' || ch == '_' || !ch.is_ascii() {
            out.push(ch);
            i += 1;
        } else {
            break;
        }
    }
    (out, i)
}

/// open の位置の開き括弧に対応する閉じ括弧の位置（引用符の中は数えない）。
fn close_paren(c: &[char], open: usize, o: char, e: char) -> usize {
    let mut quote: Option<char> = None;
    let mut depth = 0i32;
    let mut i = open;
    while i < c.len() {
        let ch = c[i];
        if let Some(q) = quote {
            if ch == q {
                quote = None;
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if ch == o {
            depth += 1;
        } else if ch == e {
            depth -= 1;
            if depth == 0 {
                return i;
            }
        }
        i += 1;
    }
    c.len()
}

fn parse_attr(inner: &str) -> Simple {
    let Some(e) = inner.find('=') else {
        return Simple::Attr(inner.trim().to_ascii_lowercase(), String::new(), String::new());
    };
    let (name, op) = match inner[..e].chars().last() {
        Some(p) if "~|^$*".contains(p) => (&inner[..e - 1], &inner[e - 1..=e]),
        _ => (&inner[..e], "="),
    };
    let v = inner[e + 1..].trim();
    let value = match v.chars().next() {
        Some(q @ ('"' | '\'')) => v[1..].split(q).next().unwrap_or_default(),
        _ => v.split_whitespace().next().unwrap_or_default(),
    };
    Simple::Attr(
        name.trim().to_ascii_lowercase(),
        op.to_string(),
        value.to_string(),
    )
}

fn parse_args(arg: &str) -> Vec<Rel> {
    let ac: Vec<char> = arg.chars().collect();
    split_top(&ac, ',')
        .into_iter()
        .map(|a| match a.chars().next() {
            Some(l @ ('>' | '+' | '~')) => Rel {
                lead: l,
                sel: parse_selector(&a[1..]),
            },
            _ => Rel {
                lead: ' ',
                sel: parse_selector(&a),
            },
        })
        .collect()
}

fn parse_selector(s: &str) -> Complex {
    let c: Vec<char> = s.trim().chars().collect();
    let mut parts: Vec<Vec<Simple>> = vec![Vec::new()];
    let mut combs = Vec::new();
    let mut pending: Option<char> = None;
    let mut pseudo_element = false;
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        if ch.is_whitespace() {
            if pending.is_none() {
                pending = Some(' ');
            }
            i += 1;
            continue;
        }
        if matches!(ch, '>' | '+' | '~') {
            pending = Some(ch);
            i += 1;
            continue;
        }
        if let Some(k) = pending.take()
            && parts.last().is_some_and(|p| !p.is_empty())
        {
            combs.push(k);
            parts.push(Vec::new());
        }
        let cur = parts.last_mut().expect("parts は空でない");
        match ch {
            '.' | '#' => {
                let (name, j) = ident(&c, i + 1);
                cur.push(if ch == '.' {
                    Simple::Class(name)
                } else {
                    Simple::Id(name)
                });
                i = j;
            }
            '*' => {
                cur.push(Simple::Any);
                i += 1;
            }
            '[' => {
                let e = close_paren(&c, i, '[', ']');
                cur.push(parse_attr(&c[i + 1..e.min(c.len())].iter().collect::<String>()));
                i = e + 1;
            }
            ':' => {
                let elem = c.get(i + 1) == Some(&':');
                let (name, mut j) = ident(&c, if elem { i + 2 } else { i + 1 });
                let name = name.to_ascii_lowercase();
                let mut arg = None;
                if c.get(j) == Some(&'(') {
                    let e = close_paren(&c, j, '(', ')');
                    arg = Some(c[j + 1..e.min(c.len())].iter().collect::<String>());
                    j = e + 1;
                }
                if elem || LEGACY_ELEM.contains(&name.as_str()) {
                    pseudo_element = true;
                    cur.push(Simple::Elem);
                } else if let (Some(a), true) = (
                    &arg,
                    matches!(name.as_str(), "not" | "is" | "where" | "matches" | "has"),
                ) {
                    cur.push(Simple::Func(name, parse_args(a)));
                } else {
                    cur.push(Simple::Pseudo(name));
                }
                i = j;
            }
            _ => {
                let (name, j) = ident(&c, i);
                if name.is_empty() {
                    cur.push(Simple::Opaque);
                    i += 1;
                } else {
                    cur.push(Simple::Type(name.to_ascii_lowercase()));
                    i = j;
                }
            }
        }
    }
    Complex {
        parts,
        combs,
        pseudo_element,
    }
}

type Spec = (u32, u32, u32);

fn add(a: Spec, b: Spec) -> Spec {
    (a.0 + b.0, a.1 + b.1, a.2 + b.2)
}

fn spec(c: &Complex) -> Spec {
    c.parts.iter().flatten().fold((0, 0, 0), |acc, s| {
        add(
            acc,
            match s {
                Simple::Id(_) => (1, 0, 0),
                Simple::Class(_) | Simple::Attr(..) | Simple::Pseudo(_) => (0, 1, 0),
                Simple::Type(_) | Simple::Elem => (0, 0, 1),
                Simple::Any | Simple::Opaque => (0, 0, 0),
                Simple::Func(n, _) if n == "where" => (0, 0, 0),
                Simple::Func(_, args) => args.iter().map(|r| spec(&r.sel)).max().unwrap_or_default(),
            },
        )
    })
}

// ---- 当たり（3 値）----

#[derive(Clone, Copy, PartialEq, Debug)]
enum M {
    Yes,
    No,
    Unknown,
}

fn and(a: M, b: M) -> M {
    match (a, b) {
        (M::No, _) | (_, M::No) => M::No,
        (M::Unknown, _) | (_, M::Unknown) => M::Unknown,
        _ => M::Yes,
    }
}

fn or(a: M, b: M) -> M {
    match (a, b) {
        (M::Yes, _) | (_, M::Yes) => M::Yes,
        (M::Unknown, _) | (_, M::Unknown) => M::Unknown,
        _ => M::No,
    }
}

fn from(b: bool) -> M {
    if b {
        M::Yes
    } else {
        M::No
    }
}

fn match_attr(el: &El, name: &str, op: &str, val: &str) -> bool {
    let Some(v) = el.attr(name) else {
        return false;
    };
    match op {
        "" => true,
        "=" => v == val,
        "~=" => v.split_whitespace().any(|w| w == val),
        "|=" => v == val || v.starts_with(&format!("{val}-")),
        "^=" => !val.is_empty() && v.starts_with(val),
        "$=" => !val.is_empty() && v.ends_with(val),
        "*=" => !val.is_empty() && v.contains(val),
        _ => false,
    }
}

fn match_rel(els: &[El], r: &Rel, el: usize) -> M {
    match r.lead {
        ' ' => (0..els.len())
            .filter(|&d| is_descendant(els, d, el))
            .fold(M::No, |acc, d| or(acc, match_complex(els, &r.sel, d))),
        '>' => (0..els.len())
            .filter(|&d| els[d].parent == Some(el))
            .fold(M::No, |acc, d| or(acc, match_complex(els, &r.sel, d))),
        _ => M::Unknown,
    }
}

fn match_simple(els: &[El], s: &Simple, el: usize) -> M {
    let e = &els[el];
    match s {
        Simple::Type(t) => from(e.tag == *t),
        Simple::Any => M::Yes,
        Simple::Class(c) => from(e.has_class(c)),
        Simple::Id(id) => from(e.attr("id") == Some(id.as_str())),
        Simple::Attr(n, op, v) => from(match_attr(e, n, op, v)),
        Simple::Pseudo(p) if p == "root" => from(e.parent.is_none() && e.tag == "html"),
        Simple::Pseudo(p) if STATE.contains(&p.as_str()) => M::Yes,
        Simple::Pseudo(_) | Simple::Elem | Simple::Opaque => M::Unknown,
        Simple::Func(n, args) if n == "has" => args
            .iter()
            .fold(M::No, |acc, r| or(acc, match_rel(els, r, el))),
        Simple::Func(n, args) => {
            let any = args
                .iter()
                .fold(M::No, |acc, r| or(acc, match_complex(els, &r.sel, el)));
            if n == "not" {
                match any {
                    M::Yes => M::No,
                    M::No => M::Yes,
                    M::Unknown => M::Unknown,
                }
            } else {
                any
            }
        }
    }
}

fn match_compound(els: &[El], part: &[Simple], el: usize) -> M {
    if part.is_empty() {
        return M::Unknown;
    }
    part.iter()
        .fold(M::Yes, |acc, s| and(acc, match_simple(els, s, el)))
}

fn match_from(els: &[El], c: &Complex, k: usize, el: usize) -> M {
    let m = match_compound(els, &c.parts[k], el);
    if m == M::No || k == 0 {
        return m;
    }
    let rest = match c.combs[k - 1] {
        ' ' => {
            let mut acc = M::No;
            let mut p = els[el].parent;
            while let Some(a) = p {
                acc = or(acc, match_from(els, c, k - 1, a));
                if acc == M::Yes {
                    break;
                }
                p = els[a].parent;
            }
            acc
        }
        '>' => els[el]
            .parent
            .map_or(M::No, |a| match_from(els, c, k - 1, a)),
        _ => M::Unknown,
    };
    and(m, rest)
}

fn match_complex(els: &[El], c: &Complex, el: usize) -> M {
    match_from(els, c, c.parts.len() - 1, el)
}

/// el に効く prop の値（宣言が無ければ None）。決められない selector がその prop を書けば Err。
fn style(rules: &[Rule], els: &[El], el: usize, prop: &str) -> Result<Option<String>, String> {
    type Key = (bool, Spec, usize, usize);
    let mut best: Option<(Key, String)> = None;
    for r in rules {
        if !r.decls.iter().any(|d| d.name == prop) {
            continue;
        }
        let mut hit: Option<Spec> = None;
        for (text, c) in &r.sels {
            if c.pseudo_element {
                continue;
            }
            match match_complex(els, c, el) {
                M::Yes => hit = hit.max(Some(spec(c))),
                M::No => {}
                M::Unknown => {
                    return Err(format!(
                        "決められない selector「{text}」が {prop} を書く（黙って外さない）"
                    ))
                }
            }
        }
        let Some(sp) = hit else {
            continue;
        };
        for (k, d) in r.decls.iter().enumerate() {
            if d.name != prop {
                continue;
            }
            let key = (d.important, sp, r.order, k);
            let wins = match &best {
                Some((b, _)) => key >= *b,
                None => true,
            };
            if wins {
                best = Some((key, d.value.to_ascii_lowercase()));
            }
        }
    }
    Ok(best.map(|(_, v)| v))
}

fn must(rules: &[Rule], els: &[El], el: usize, prop: &str) -> Option<String> {
    style(rules, els, el, prop).unwrap_or_else(|e| panic!("{e}"))
}

fn z_index(rules: &[Rule], els: &[El], el: usize) -> Option<i64> {
    must(rules, els, el, "z-index").and_then(|v| v.parse().ok())
}

fn positioned(rules: &[Rule], els: &[El], el: usize) -> bool {
    must(rules, els, el, "position").is_some_and(|v| v != "static")
}

/// 位置を持つ一番近い祖先（包含する箱）。
fn containing_block(rules: &[Rule], els: &[El], el: usize) -> Option<usize> {
    let mut p = els[el].parent;
    while let Some(a) = p {
        if positioned(rules, els, a) {
            return Some(a);
        }
        p = els[a].parent;
    }
    None
}

#[test]
fn f143_the_card_hit_area_covers_the_card_under_its_links() {
    let html = fs::read_to_string(repo_root().join("tests/fixtures/face/expected-index.html"))
        .expect("凍結の面");
    let css = fs::read_to_string(repo_root().join("design-intent/preview/folio.css"))
        .expect("正本の様式");
    let els = parse_html(&html);
    let rules = parse_css(&css);
    let hits: Vec<usize> = (0..els.len())
        .filter(|&i| els[i].tag == "a" && els[i].has_class("sc-hit"))
        .collect();
    assert_eq!(hits.len(), 3, "凍結の面の sc-hit は 3 つ（憲法・要件書・設計ノート）");
    for &h in &hits {
        let href = els[h].attr("href").unwrap_or_default();
        assert_eq!(
            must(&rules, &els, h, "position").as_deref(),
            Some("absolute"),
            "sc-hit（{href}）の position"
        );
        assert_eq!(
            must(&rules, &els, h, "inset").as_deref(),
            Some("0"),
            "sc-hit（{href}）の inset"
        );
        assert_ne!(
            must(&rules, &els, h, "display").as_deref(),
            Some("none"),
            "sc-hit（{href}）が消えている"
        );
        let z_hit = z_index(&rules, &els, h)
            .unwrap_or_else(|| panic!("sc-hit（{href}）の z-index が整数でない"));
        let card = containing_block(&rules, &els, h)
            .unwrap_or_else(|| panic!("sc-hit（{href}）に位置を持つ祖先が無い"));
        assert_eq!(
            els[card].attr("data-component"),
            Some("shelf-card"),
            "sc-hit（{href}）の広がる先が {} でカードでない",
            els[card].tag
        );
        let others: Vec<usize> = (0..els.len())
            .filter(|&i| i != h && els[i].tag == "a" && is_descendant(&els, i, card))
            .collect();
        assert!(
            !others.is_empty(),
            "sc-hit（{href}）のカードに他の a が無い"
        );
        for o in others {
            let cls = els[o].attr("class").unwrap_or_default();
            assert!(
                positioned(&rules, &els, o),
                "カード（{href}）の中の a.{cls} が位置を持たない"
            );
            let z = z_index(&rules, &els, o)
                .unwrap_or_else(|| panic!("カード（{href}）の中の a.{cls} の z-index が整数でない"));
            assert!(
                z > z_hit,
                "カード（{href}）の中の a.{cls} の z-index {z} が sc-hit の {z_hit} より上でない"
            );
        }
    }
}

const MINI_FACE: &str = r#"<figure data-component="doc-shelf"><article data-component="shelf-card" class="shelf-c" id="doc-x"><p class="sc-use">説明</p><p class="sc-row"><span class="up">更新</span><a class="sc-open" href="x.html">開く →</a><a class="sc-hit" href="x.html" aria-hidden="true" tabindex="-1"></a></p></article></figure>"#;

/// 手書きの小さな面の sc-hit に効く (position, z-index)。
fn probe(css: &str) -> Result<(Option<String>, Option<String>), String> {
    let els = parse_html(MINI_FACE);
    let rules = parse_css(css);
    let h = (0..els.len())
        .find(|&i| els[i].has_class("sc-hit"))
        .expect("手書きの面の sc-hit");
    Ok((
        style(&rules, &els, h, "position")?,
        style(&rules, &els, h, "z-index")?,
    ))
}

#[test]
fn f143_the_cascade_reader_follows_the_css_order() {
    let base = r#".sc-hit{position:absolute;inset:0;z-index:0}
[data-component="shelf-card"] .sc-open,[data-component="shelf-card"] .sc-row a{position:relative;z-index:1}"#;
    let fixed = r#".sc-hit{position:absolute;inset:0;z-index:0}
[data-component="shelf-card"] .sc-open,[data-component="shelf-card"] .sc-row a:not(.sc-hit){position:relative;z-index:1}"#;
    let cases: [(&str, &str, Option<&str>, Option<&str>); 11] = [
        ("base の形", base, Some("relative"), Some("1")),
        (":not で外す", fixed, Some("absolute"), Some("0")),
        (
            "同じ詳細度は後が勝つ",
            ".sc-hit{position:absolute}.sc-hit{position:fixed}",
            Some("fixed"),
            None,
        ),
        (
            "!important が勝つ",
            r#".sc-hit{position:absolute !important}[data-component="shelf-card"] .sc-row a{position:relative}"#,
            Some("absolute"),
            None,
        ),
        (
            "印刷の @media は読まない",
            ".sc-hit{position:absolute}@media print{.sc-hit{position:static}}",
            Some("absolute"),
            None,
        ),
        (
            "画面の @media は読む",
            ".sc-hit{position:absolute}@media (max-width:680px){.sc-hit{position:sticky}}",
            Some("sticky"),
            None,
        ),
        (
            "注は読まない",
            ".sc-hit{position:absolute}/* .sc-row .sc-hit{position:fixed} */",
            Some("absolute"),
            None,
        ),
        (
            "擬似要素は読まない",
            ".sc-hit{position:absolute}.sc-row .sc-hit::before{position:fixed}.sc-row .sc-hit:after{position:fixed}",
            Some("absolute"),
            None,
        ),
        (
            "hover は当たると読む",
            ".sc-hit{position:absolute}.sc-row a:hover{position:relative}",
            Some("relative"),
            None,
        ),
        (
            ":not の中の id も詳細度に数える",
            r#"[data-component="shelf-card"] .sc-row a{position:relative}a:not(#nope){position:absolute}"#,
            Some("absolute"),
            None,
        ),
        ("宣言が無い", ".sc-open{position:relative;z-index:1}", None, None),
    ];
    for (name, css, pos, z) in cases {
        let got = probe(css).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(
            (got.0.as_deref(), got.1.as_deref()),
            (pos, z),
            "{name} の (position, z-index)"
        );
    }
    // 兄弟の結合子: 右端が当たるなら決められずに落ちる（黙って外さない）・右端が外れるなら読まずに済む
    let err = probe(".sc-hit{position:absolute}.sc-open + .sc-hit{position:relative}")
        .expect_err("兄弟の結合子で決められない selector は Err");
    assert!(err.contains(".sc-open + .sc-hit"), "{err}");
    assert!(probe(".sc-open ~ .sc-hit{position:relative}").is_err());
    assert_eq!(
        probe(".sc-hit{position:absolute}.sc-open + .sc-row{position:relative}"),
        Ok((Some("absolute".to_string()), None)),
        "右端が外れる兄弟の結合子は当たりにならない"
    );
}
