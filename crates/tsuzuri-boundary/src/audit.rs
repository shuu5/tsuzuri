//! 面の受入 12 条の数え（行 j-count・要件 NFR1・規則の行 R-23・受入 AC9）。
//! 1 つの画面を 1 つの幅と mode で測った事実の JSON の字を読み、12 条ごとの違反の数と report の行を返す。
//! host の純粋な関数だけを持ち、file も browser も触らない（事実を測る式と runner は後の行 j-runner）。
//! 判定の字数と閾値と origin の比べと語彙表の引きはこの側で持つ（歯が browser 無しで撃てる）。

use crate::stage::json::{items, member, unquote};

/// 受入の 12 条（鍵と名）。規則の行 R-23 の順。
pub const RULES: [(&str, &str); 12] = [
    ("overflow", "はみ出し"),
    ("overlap", "重なり"),
    ("hscroll", "横 scroll"),
    ("hover", "hover の欠け"),
    ("heading", "語彙表に無い見出し"),
    ("prose", "最初の画面の散文 300 字 超"),
    ("error", "page error"),
    ("budget", "文の予算の違反"),
    ("nodeid", "id の無い節点"),
    ("url", "URL に残らない切り替え"),
    ("verbatim", "持ち主の逐語の混入"),
    ("library", "外部の配信元からの library"),
];

/// 最初の画面の散文の字数の上限（文字の字だけを数える・この数までは違反でない）。
pub const PROSE_MAX: usize = 300;

/// 画面の題の字数の上限（この数までは違反でない）。
pub const TITLE_MAX: usize = 36;

/// 見出し 1 つ（data-v の語の鍵と、出ている字）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub key: String,
    pub text: String,
}

/// 節点の札 1 つ（節点の id と、札の字）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub text: String,
}

/// 押した切り替え 1 つ（札の字と、押す前と押した後の URL）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Switch {
    pub label: String,
    pub before: String,
    pub after: String,
}

/// 1 つの画面を 1 つの幅と mode で測った事実。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    /// 頁の URL。
    pub url: String,
    /// 横の溢れ（文書の scroll の幅から窓の幅を引いた px）。
    pub hscroll: i64,
    /// 中身が箱を越える要素。
    pub overflow: Vec<String>,
    /// 重なる要素の組。
    pub overlap: Vec<String>,
    /// hover の card の受け手の無い要素。
    pub nocard: Vec<String>,
    /// 見出し。
    pub headings: Vec<Heading>,
    /// 最初の画面の散文の片。
    pub first: Vec<String>,
    /// page error。
    pub errors: Vec<String>,
    /// 画面の題。
    pub titles: Vec<String>,
    /// 節点の札。
    pub nodes: Vec<Node>,
    /// 押した切り替え。
    pub switches: Vec<Switch>,
    /// 頁の見える字。
    pub text: String,
    /// 読み先（script と stylesheet と preload）。
    pub libraries: Vec<String>,
}

/// 事実の JSON の object の字を読む。鍵の欠けと形の違いは鍵の名を含む Err・余る鍵は読み捨てる。
pub fn facts(text: &str) -> Result<Facts, String> {
    if !text.trim_start().starts_with('{') {
        return Err("事実の字が object でない".to_string());
    }
    let hscroll = field(text, "hscroll")?;
    Ok(Facts {
        url: string(text, "url")?,
        hscroll: hscroll
            .parse()
            .map_err(|_| "事実の鍵 hscroll が整数でない".to_string())?,
        overflow: strings(text, "overflow")?,
        overlap: strings(text, "overlap")?,
        nocard: strings(text, "nocard")?,
        headings: objects(text, "headings", |o| {
            Some(Heading {
                key: unquote(member(o, "key")?)?,
                text: unquote(member(o, "text")?)?,
            })
        })?,
        first: strings(text, "first")?,
        errors: strings(text, "errors")?,
        titles: strings(text, "titles")?,
        nodes: objects(text, "nodes", |o| {
            Some(Node {
                id: unquote(member(o, "id")?)?,
                text: unquote(member(o, "text")?)?,
            })
        })?,
        switches: objects(text, "switches", |o| {
            Some(Switch {
                label: unquote(member(o, "label")?)?,
                before: unquote(member(o, "before")?)?,
                after: unquote(member(o, "after")?)?,
            })
        })?,
        text: string(text, "text")?,
        libraries: strings(text, "libraries")?,
    })
}

fn field<'a>(text: &'a str, key: &str) -> Result<&'a str, String> {
    member(text, key).ok_or_else(|| format!("事実の鍵 {key} が無い"))
}

fn string(text: &str, key: &str) -> Result<String, String> {
    unquote(field(text, key)?).ok_or_else(|| format!("事実の鍵 {key} が字でない"))
}

fn strings(text: &str, key: &str) -> Result<Vec<String>, String> {
    objects(text, key, unquote)
}

fn objects<T>(
    text: &str,
    key: &str,
    read: impl Fn(&str) -> Option<T>,
) -> Result<Vec<T>, String> {
    items(field(text, key)?)
        .and_then(|list| list.into_iter().map(read).collect())
        .ok_or_else(|| format!("事実の鍵 {key} の形が違う"))
}

/// 語彙表の JSON の字から鍵の label を引く（english の欄を先に引き、無ければ rephrase の欄）。
pub fn label(vocab: &str, key: &str) -> Option<String> {
    ["english", "rephrase"].iter().find_map(|column| {
        let entry = member(member(vocab, column)?, key)?;
        unquote(member(entry, "label")?)
    })
}

/// 散文の片の字数（字の種類が文字の字だけ・数字と記号と約物と空白を数えない）。
pub fn prose_chars(text: &str) -> usize {
    text.chars().filter(|c| c.is_alphabetic()).count()
}

/// 12 条ごとの違反の数（RULES の順）。
pub fn count(facts: &Facts, vocab: &str) -> [usize; 12] {
    let prose: usize = facts.first.iter().map(|p| prose_chars(p)).sum();
    let long_titles = facts
        .titles
        .iter()
        .filter(|t| t.chars().count() > TITLE_MAX)
        .count();
    let heavy = facts.first.iter().filter(|p| heavy_prose(p)).count();
    [
        facts.overflow.len(),
        facts.overlap.len(),
        usize::from(facts.hscroll > 0),
        facts.nocard.len(),
        facts
            .headings
            .iter()
            .filter(|h| label(vocab, &h.key).as_deref() != Some(h.text.as_str()))
            .count(),
        usize::from(prose > PROSE_MAX),
        facts.errors.len(),
        long_titles + heavy,
        facts
            .nodes
            .iter()
            .filter(|n| !n.text.contains(n.id.as_str()))
            .count(),
        facts.switches.iter().filter(|s| s.before == s.after).count(),
        verbatim_marks(&facts.text),
        facts
            .libraries
            .iter()
            .filter(|l| foreign(&facts.url, l))
            .count(),
    ]
}

/// 散文の片が文の予算を越えるか（中黒を 2 つ以上持つか、括弧を入れ子にする）。
fn heavy_prose(text: &str) -> bool {
    if text.matches('・').count() >= 2 {
        return true;
    }
    let mut depth = 0usize;
    for c in text.chars() {
        match c {
            '（' | '(' => {
                depth += 1;
                if depth >= 2 {
                    return true;
                }
            }
            '）' | ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    false
}

/// 持ち主の逐語の記録の印（`user YYYY-MM-DDTHH:`）の数。
fn verbatim_marks(text: &str) -> usize {
    const SHAPE: &[u8] = b"dddd-dd-ddTdd:";
    text.match_indices("user ")
        .filter(|(at, word)| {
            let rest = &text.as_bytes()[at + word.len()..];
            rest.len() >= SHAPE.len()
                && SHAPE.iter().zip(rest).all(|(want, got)| match want {
                    b'd' => got.is_ascii_digit(),
                    _ => want == got,
                })
        })
        .count()
}

/// 読み先が頁と別の origin（scheme と host と port）から来るか。
/// 相対の path と、authority を持たない scheme の字（data: と blob: ほか）は外と見ない。
/// scheme の無い `//` の形は頁の scheme で読む。
fn foreign(page: &str, source: &str) -> bool {
    let Some(page_origin) = origin(page) else {
        return false;
    };
    let absolute;
    let source = if source.starts_with("//") {
        let scheme = page.split_once(':').map_or("", |(s, _)| s);
        absolute = format!("{scheme}:{source}");
        absolute.as_str()
    } else {
        source
    };
    origin(source).is_some_and(|o| o != page_origin)
}

/// URL の origin（小文字の scheme と host と、既定を埋めた port）。authority を持たない字は None。
fn origin(url: &str) -> Option<(String, String, String)> {
    let (scheme, rest) = url.split_once(':')?;
    if scheme.is_empty()
        || !scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    {
        return None;
    }
    let rest = rest.strip_prefix("//")?;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host_port = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let (host, port) = match host_port.rfind(':') {
        Some(at) if !host_port[at..].contains(']') => (&host_port[..at], &host_port[at + 1..]),
        _ => (host_port, ""),
    };
    let scheme = scheme.to_ascii_lowercase();
    let port = match (port, scheme.as_str()) {
        ("", "http" | "ws") => "80".to_string(),
        ("", "https" | "wss") => "443".to_string(),
        (p, _) => p.to_string(),
    };
    Some((scheme, host.to_ascii_lowercase(), port))
}

/// report の見出しの行（width mode url と 12 の鍵と 計）。
pub fn head() -> String {
    let mut words = vec!["width", "mode", "url"];
    words.extend(RULES.iter().map(|(key, _)| *key));
    words.push("計");
    words.join(" ")
}

/// report の 1 行（幅と mode と URL と 12 の数と 計 と数の和）。
pub fn line(width: u32, mode: &str, url: &str, counts: &[usize; 12]) -> String {
    let mut words = vec![width.to_string(), mode.to_string(), url.to_string()];
    words.extend(counts.iter().map(usize::to_string));
    words.push("計".to_string());
    words.push(counts.iter().sum::<usize>().to_string());
    words.join(" ")
}
