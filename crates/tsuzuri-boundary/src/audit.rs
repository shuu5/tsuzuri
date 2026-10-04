//! 面の受入 12 条の数え（行 j-count・要件 NFR1・規則の行 R-23・受入 AC9）。
//! 1 つの画面を 1 つの幅と mode で測った事実の JSON の字を読み、12 条ごとの違反の数と report の行を返す。
//! host の純粋な関数だけを持ち、file も browser も触らない（事実を測る式と runner は後の行 j-runner）。
//! 判定の字数と閾値と origin の比べと語彙表の引きはこの側で持つ（歯が browser 無しで撃てる）。
//! 行 j-runner で runner（全画面 × 幅 × 2 mode を頁の口 `Page` で開いて測る `case` と `sweep`）を足す。
//! 幅は規則の行 R-23 の 4 幅で、画面は project board の 1 枚の画面と帯の印が開く窓と account board の tab
//! （行 g-accept）。
//! 頁の口は CDP の Session が実装し、歯は偽の頁で撃つ。
//! runner は頁を開いた後、面が描き、どの block も読みの印を出さなくなるまで上限つきで待ち、上限で残る画面を
//! 違反 0 と数えずまだ分からないとし、今の選びの切り替えは押さない（行 g-accept-runner・憲法 P-7）。

use std::fmt::Write;

use crate::stage::cdp::{Command, Session};
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

/// 節点の札 1 つの指の受け手（札の名と、自分から body の手前の祖先まで段ごとに持つ受け手の種類の字・空白で区切る）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reach {
    pub name: String,
    pub up: Vec<String>,
}

impl Reach {
    /// hover の card を出せるか: どれかの段が pointerenter を持つか、1 つの段が mouseover と mouseout を対で持つ
    /// （委ねの口・指が入ると card を出し、出ると猶予に入る・規則の行 R-20）。片方だけの段は数えない（行 g-accept-fix）。
    pub fn has_card(&self) -> bool {
        self.up.iter().any(|level| {
            let kinds: Vec<&str> = level.split_whitespace().collect();
            kinds.contains(&"pointerenter")
                || (kinds.contains(&"mouseover") && kinds.contains(&"mouseout"))
        })
    }
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
    /// click の受け手の無い吹き出しの口（札と一覧の行）。
    pub nocard: Vec<String>,
    /// 節点の札（節点の頁への link と近傍の図の節点）の指の受け手。
    pub reach: Vec<Reach>,
    /// 散文と文の予算から外した置き場（台帳の字の印は `ledger <名>`・表の行を開いた中の語と値の対は `pair <名>`）と、
    /// 箱を持つが描かれないのでどの条にも数えない要素（`hidden <名>`）（行 g-accept-runner）。
    pub skipped: Vec<String>,
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
        reach: objects(text, "reach", |o| {
            Some(Reach {
                name: unquote(member(o, "name")?)?,
                up: items(member(o, "up")?)?
                    .into_iter()
                    .map(unquote)
                    .collect::<Option<_>>()?,
            })
        })?,
        skipped: objects(text, "skipped", |o| {
            unquote(o).filter(|s| SKIP_KINDS.iter().any(|k| s.starts_with(&format!("{k} "))))
        })?,
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

/// 外した置き場の種類（台帳の字の印・表の行を開いた中の語と値の対・描かれない要素・report の字は SKIP_WORDS の同じ順）。
pub const SKIP_KINDS: [&str; 3] = ["ledger", "pair", "hidden"];

/// 外した置き場の種類の report の字。
pub const SKIP_WORDS: [&str; 3] = ["台帳の字", "語と値の対", "見えない要素"];

/// 外した置き場の数（SKIP_KINDS の順）。
pub fn skipped_counts(facts: &Facts) -> [usize; 3] {
    SKIP_KINDS.map(|k| {
        let head = format!("{k} ");
        facts
            .skipped
            .iter()
            .filter(|s| s.starts_with(&head))
            .count()
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
        facts.nocard.len() + facts.reach.iter().filter(|r| !r.has_card()).count(),
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

/// 数えた違反の中身（条の鍵と空白と要素の名か字の 1 行を違反ごとに・行の数は count の和と同じ・RULES の順・行 g-accept-runner）。
/// 撃つ時刻のデータで出る違反を report から追えるようにする（散文は字数だけ、逐語は印の位置だけを書き、字を写さない）。
pub fn details(facts: &Facts, vocab: &str) -> Vec<String> {
    RULES
        .iter()
        .zip(found(facts, vocab))
        .flat_map(|((key, _), items)| {
            items
                .into_iter()
                .map(move |i| format!("{key} {}", i.replace('\n', " ")))
        })
        .collect()
}

/// 条ごとに数えた要素の名か字（RULES の順・count が数える物と同じ）。
fn found(facts: &Facts, vocab: &str) -> [Vec<String>; 12] {
    let prose: usize = facts.first.iter().map(|p| prose_chars(p)).sum();
    let hover = facts.nocard.iter().cloned().chain(
        facts
            .reach
            .iter()
            .filter(|r| !r.has_card())
            .map(|r| r.name.clone()),
    );
    let heading = facts
        .headings
        .iter()
        .filter(|h| label(vocab, &h.key).as_deref() != Some(h.text.as_str()))
        .map(|h| format!("{} {}", h.key, h.text));
    let budget = facts
        .titles
        .iter()
        .filter(|t| t.chars().count() > TITLE_MAX)
        .chain(facts.first.iter().filter(|p| heavy_prose(p)))
        .cloned();
    let nodeid = facts
        .nodes
        .iter()
        .filter(|n| !n.text.contains(n.id.as_str()))
        .map(|n| n.id.clone());
    [
        facts.overflow.clone(),
        facts.overlap.clone(),
        (facts.hscroll > 0)
            .then(|| format!("{} px", facts.hscroll))
            .into_iter()
            .collect(),
        hover.collect(),
        heading.collect(),
        (prose > PROSE_MAX)
            .then(|| format!("{prose} 字"))
            .into_iter()
            .collect(),
        facts.errors.clone(),
        budget.collect(),
        nodeid.collect(),
        facts
            .switches
            .iter()
            .filter(|s| s.before == s.after)
            .map(|s| s.label.clone())
            .collect(),
        verbatim_at(&facts.text)
            .map(|at| format!("位置 {at}"))
            .collect(),
        facts
            .libraries
            .iter()
            .filter(|l| foreign(&facts.url, l))
            .cloned()
            .collect(),
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
    verbatim_at(text).count()
}

/// 持ち主の逐語の印（字 user と空白と日時の頭）の byte の位置。
fn verbatim_at(text: &str) -> impl Iterator<Item = usize> + '_ {
    const SHAPE: &[u8] = b"dddd-dd-ddTdd:";
    text.match_indices("user ")
        .filter(|(at, word)| {
            let rest = text.as_bytes().get(at + word.len()..).unwrap_or_default();
            rest.len() >= SHAPE.len()
                && SHAPE.iter().zip(rest).all(|(want, got)| match want {
                    b'd' => got.is_ascii_digit(),
                    _ => want == got,
                })
        })
        .map(|(at, _)| at)
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

/// 測る幅（px・規則の行 R-23 の 4 幅・行 g-accept）。
pub const WIDTHS: [u32; 4] = [1280, 960, 700, 390];

/// 測る mode（URL の mode= の値）。
pub const MODES: [&str; 2] = ["beginner", "expert"];

/// 測る画面の query の頭（board の URL の後に続け、その後に mode= と mode の値を足す）。
/// project board の 1 枚の画面と、その上に帯の印が開く 8 つの窓（query の win・行 g-accept・相談の窓は行 cs-bar）と account board の
/// 3 つの tab（節点の頁は sweep が組ごとに足す・地図の頁と 6 つの面は行 m-map-page で、質問の頁と抜けの検査の頁は
/// 行 g-one-screen-a で消した）。
pub const SCREENS: [&str; 12] = [
    "?",
    "?win=ask&",
    "?win=stalled&",
    "?win=notices&",
    "?win=seat&",
    "?win=gaps&",
    "?win=legend&",
    "?win=dest&",
    "?win=consult&",
    "?board=account&tab=home&",
    "?board=account&tab=session&",
    "?board=account&tab=projects&",
];

/// 節点の頁に開く最初の節点を採る画面（home・台帳の一覧の番号の札・地図の圧縮の面は行 m-map-page で消した）。
const FIRST_NODE: &str = "?";

/// 狭い幅（この幅までは高さ 844 の mobile で撃つ・広い幅は高さ 800）。
const NARROW: u32 = 390;

/// 頁へ移った後に待つ間（面の wasm の起動と読みの応答を待つ）。
const SETTLE_MS: u64 = 1000;

/// 面の block が口をまだ読んでいない間に出す理由の字（面の kit の NOT_READ と同じ字・行 g-accept-runner）。
pub const NOT_READ: &str = "この block の口をまだ読んでいない";

/// 読みの印を見直す間（ms・行 g-accept-runner）。
pub const POLL_MS: u64 = 300;

/// 読みの印を見る回数の上限（SETTLE_MS の後に 1 回と、POLL_MS ごとに残りの回・間の待ちだけで約 30 秒・行 g-accept-runner）。
pub const READ_POLLS: usize = 100;

/// 読みの印が上限まで残った画面の字（report の行の末と case の Err・行 g-accept-runner）。
pub const UNSETTLED: &str = "まだ分からない（面の block の読みの印が待ちの上限の後も残る・server の口の応答を確かめて撃ち直す）";

/// 頁の見える字が、まだ測れない頁の字か（面がまだ描いていない空の字か、どれかの block が読みの印を出す）。
pub fn unread(text: &str) -> bool {
    text.trim().is_empty() || text.contains(NOT_READ)
}

/// runner の頁の口（CDP の Session が実装する・歯は偽の頁で撃つ）。
pub trait Page {
    /// 命令を撃ち、Call の歩の応答の字を返す。
    fn run(&mut self, command: &Command) -> Result<Vec<String>, String>;
    /// 測りの式を撃ち、事実の JSON の字を返す。
    fn measure(&mut self) -> Result<String, String>;
    /// 頁の今の URL。
    fn url(&mut self) -> Result<String, String>;
    /// 受けた event の字（受けた順）。
    fn events(&self) -> &[String];
}

impl Page for Session {
    fn run(&mut self, command: &Command) -> Result<Vec<String>, String> {
        Session::run(self, command)
    }

    fn measure(&mut self) -> Result<String, String> {
        Session::measure(self)
    }

    fn url(&mut self) -> Result<String, String> {
        Session::url(self)
    }

    fn events(&self) -> &[String] {
        Session::events(self)
    }
}

/// event の字のうち page error の字（Runtime.exceptionThrown の exceptionDetails の text と、
/// Log.entryAdded の entry の level が error の text）を受けた順に返す。
pub fn errors(events: &[String]) -> Vec<String> {
    events
        .iter()
        .filter_map(|event| {
            let params = member(event, "params")?;
            match unquote(member(event, "method")?)?.as_str() {
                "Runtime.exceptionThrown" => {
                    unquote(member(member(params, "exceptionDetails")?, "text")?)
                }
                "Log.entryAdded" => {
                    let entry = member(params, "entry")?;
                    if unquote(member(entry, "level")?)? != "error" {
                        return None;
                    }
                    unquote(member(entry, "text")?)
                }
                _ => None,
            }
        })
        .collect()
}

/// 1 つの画面を 1 つの幅で測る。viewport と console を撃って頁へ移り、測りの式の事実に URL と、
/// 移ってから測るまでの page error と、測りの返す switch_at の在りかごとに押した切り替え（押すたびに頁へ戻る）を埋める。
/// 読みの印が待ちの上限の後も残る画面は Err（字は UNSETTLED を含む・行 g-accept-runner）。
pub fn case(page: &mut impl Page, url: &str, width: u32) -> Result<Facts, String> {
    settled(page, url, width)?.ok_or_else(|| format!("{url}: {UNSETTLED}"))
}

/// case の中身（読みの印が待ちの上限の後も残る画面は None）。今の選び（switch_at の now が真）は押さない。
fn settled(page: &mut impl Page, url: &str, width: u32) -> Result<Option<Facts>, String> {
    let narrow = width <= NARROW;
    page.run(&Command::Viewport {
        width,
        height: if narrow { 844 } else { 800 },
        scale: 1,
        mobile: narrow,
    })?;
    page.run(&Command::Console)?;
    let from = page.events().len();
    let Some((mut facts, text)) = open(page, url)? else {
        return Ok(None);
    };
    facts.url = url.to_string();
    facts.errors = errors(page.events().get(from..).unwrap_or_default());
    let at = objects(&text, "switch_at", |o| {
        let spot = |key| member(o, key)?.parse::<u32>().ok();
        let now = match member(o, "now")? {
            "true" => true,
            "false" => false,
            _ => return None,
        };
        Some((unquote(member(o, "label")?)?, now, spot("x")?, spot("y")?))
    })
    .map_err(|e| format!("{url}: {e}"))?;
    facts.switches.clear();
    for (label, _, x, y) in at.into_iter().filter(|(_, now, _, _)| !now) {
        let before = page.url()?;
        page.run(&Command::Click { x, y })?;
        let after = page.url()?;
        if open(page, url)?.is_none() {
            return Ok(None);
        }
        facts.switches.push(Switch {
            label,
            before,
            after,
        });
    }
    Ok(Some(facts))
}

/// 頁へ移り、面が描いてどの block も読みの印を出さなくなるまで待ち、その時の事実と測りの字を返す。
/// SETTLE_MS の後に測り、事実の字 text が unread なら POLL_MS ごとに測り直し、READ_POLLS 回の測りの後も
/// unread なら None。事実の字が読めなければ待たずに Err。
fn open(page: &mut impl Page, url: &str) -> Result<Option<(Facts, String)>, String> {
    page.run(&Command::Navigate {
        url: url.to_string(),
    })?;
    page.run(&Command::Wait { ms: SETTLE_MS })?;
    for poll in 0..READ_POLLS {
        if poll > 0 {
            page.run(&Command::Wait { ms: POLL_MS })?;
        }
        let text = page.measure()?;
        let read = facts(&text).map_err(|e| format!("{url}: {e}"))?;
        if !unread(&read.text) {
            return Ok(Some((read, text)));
        }
    }
    Ok(None)
}

/// sweep の数え（report の字・違反の和・まだ分からない画面の数）。
struct Tally<'a> {
    vocab: &'a str,
    report: String,
    total: usize,
    unknown: usize,
    skipped: [usize; 3],
}

impl Tally<'_> {
    /// report に画面の 1 行を足す（測れた画面は line の 12 の数と、外した置き場が在ればその数・測れない画面は幅と
    /// mode と URL と UNSETTLED）。
    fn add(&mut self, width: u32, mode: &str, url: &str, facts: Option<&Facts>) {
        match facts {
            Some(facts) => {
                let counts = count(facts, self.vocab);
                self.total += counts.iter().sum::<usize>();
                self.report.push_str(&line(width, mode, url, &counts));
                let skipped = skipped_counts(facts);
                if skipped.iter().sum::<usize>() > 0 {
                    self.report.push_str(&skip_words(&skipped));
                }
                for (all, n) in self.skipped.iter_mut().zip(skipped) {
                    *all += n;
                }
                // 違反の中身は画面の行の下に 1 つずつ（2 つの空白で字下げ・撃つ時刻で出る違反を追う）。
                for item in details(facts, self.vocab) {
                    let _ = write!(self.report, "\n  {item}");
                }
            }
            None => {
                self.unknown += 1;
                let _ = write!(self.report, "{width} {mode} {url} {UNSETTLED}");
            }
        }
        self.report.push('\n');
    }
}

/// 全画面を 4 幅 × 2 mode で測り、report の字と違反の和とまだ分からない画面の数を返す。組ごとに SCREENS の
/// 11 の画面と、その組の home の最初の節点の頁を測る。読みの印が上限まで残る画面は違反に数えず、まだ分からないの
/// 行にして続ける（home がまだ分からない組は、開く節点も分からないので節点の頁も id の無い URL でまだ分からない）。
/// report は head の行・画面ごとの行・違反 計 の行・まだ分からない 計 の行。
pub fn sweep(
    page: &mut impl Page,
    board: &str,
    vocab: &str,
) -> Result<(String, usize, usize), String> {
    let mut tally = Tally {
        vocab,
        report: head(),
        total: 0,
        unknown: 0,
        skipped: [0; 3],
    };
    tally.report.push('\n');
    for width in WIDTHS {
        for mode in MODES {
            // home の最初の節点（外の None は home がまだ分からない・中の None は節点の無い home）。
            let mut first = None;
            for screen in SCREENS {
                let url = format!("{board}{screen}mode={mode}");
                let facts = settled(page, &url, width)?;
                tally.add(width, mode, &url, facts.as_ref());
                if screen == FIRST_NODE {
                    first = facts.map(|f| f.nodes.into_iter().next().map(|n| n.id));
                }
            }
            let Some(first) = first else {
                tally.add(width, mode, &format!("{board}?page=node&mode={mode}"), None);
                continue;
            };
            let id = first.ok_or_else(|| {
                format!("{width} {mode}: home の画面に節点が無い（節点の頁を開けない）")
            })?;
            let url = format!("{board}?page=node&id={}&mode={mode}", encode(&id));
            let facts = settled(page, &url, width)?;
            tally.add(width, mode, &url, facts.as_ref());
        }
    }
    let _ = writeln!(tally.report, "違反 計 {}", tally.total);
    let _ = writeln!(tally.report, "まだ分からない 計 {}", tally.unknown);
    let _ = writeln!(
        tally.report,
        "外した置き場 計{}",
        skip_words(&tally.skipped)
    );
    Ok((tally.report, tally.total, tally.unknown))
}

/// 外した置き場の数の字（種類ごとに空白と SKIP_WORDS の字と空白と数・頭に 外し は付けない）。
fn skip_words(skipped: &[usize; 3]) -> String {
    SKIP_WORDS
        .iter()
        .zip(skipped)
        .map(|(word, n)| format!(" {word} {n}"))
        .collect()
}

/// query の値の字（英数と - . _ ~ のほかの byte を %XX にする・面の節点の頁への link と同じ）。
fn encode(text: &str) -> String {
    let mut out = String::new();
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(b));
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}
