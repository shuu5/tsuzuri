//! 正本の読み手。yaml-rust2 の低水準の event から木を組み、同じ表に同じキーを 2 度書いた箇所を拾う
//! （高水準の読み手は後のキーで黙って上書きする＝人と機械の読みが割れる）。

use std::collections::HashMap;

use yaml_rust2::parser::{Event, MarkedEventReceiver, Parser};
use yaml_rust2::scanner::{Marker, TScalarStyle};

/// 読んだ木。表は書かれた順に持つ（重複キーは最初の値を残し、`Doc::duplicates` に記録する）。
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// 値の無い欄（`~`・`null`・空）
    Null,
    Scalar(String),
    Seq(Vec<Node>),
    Map(Vec<(String, Node)>),
}

impl Node {
    pub fn as_map(&self) -> Option<&[(String, Node)]> {
        match self {
            Node::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_seq(&self) -> Option<&[Node]> {
        match self {
            Node::Seq(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Node::Scalar(s) => Some(s),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&Node> {
        self.as_map()?
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// 空の欄か（無い・null・空白だけの文字列・空の一覧・空の表）。
    pub fn is_blank(&self) -> bool {
        match self {
            Node::Null => true,
            Node::Scalar(s) => s.trim().is_empty(),
            Node::Seq(s) => s.is_empty(),
            Node::Map(m) => m.is_empty(),
        }
    }
}

/// 同じ表に 2 度書かれたキー（1 始まりの行番号は 2 度目の位置）。
#[derive(Debug, Clone, PartialEq)]
pub struct Duplicate {
    pub key: String,
    pub line: usize,
}

#[derive(Debug)]
pub struct Doc {
    pub root: Node,
    pub duplicates: Vec<Duplicate>,
}

enum Frame {
    Seq(Vec<Node>, usize),
    /// 表・読みかけのキー・表の anchor
    Map(Vec<(String, Node)>, Option<String>, usize),
}

#[derive(Default)]
struct Builder {
    stack: Vec<Frame>,
    root: Option<Node>,
    anchors: HashMap<usize, Node>,
    duplicates: Vec<Duplicate>,
    error: Option<String>,
    docs: usize,
}

impl Builder {
    fn push_value(&mut self, node: Node, anchor: usize, mark: Marker) {
        if anchor > 0 {
            self.anchors.insert(anchor, node.clone());
        }
        match self.stack.last_mut() {
            None => self.root = Some(node),
            Some(Frame::Seq(items, _)) => items.push(node),
            Some(Frame::Map(entries, pending, _)) => match pending.take() {
                None => match node {
                    Node::Scalar(k) => *pending = Some(k),
                    Node::Null => *pending = Some(String::new()),
                    _ => {
                        self.error.get_or_insert_with(|| {
                            format!("{} 行: 表のキーが文字列でない", mark.line())
                        });
                        *pending = Some(String::new());
                    }
                },
                Some(key) => {
                    if entries.iter().any(|(k, _)| *k == key) {
                        self.duplicates.push(Duplicate {
                            key,
                            line: mark.line(),
                        });
                    } else {
                        entries.push((key, node));
                    }
                }
            },
        }
    }
}

impl MarkedEventReceiver for Builder {
    fn on_event(&mut self, ev: Event, mark: Marker) {
        match ev {
            Event::DocumentStart => {
                self.docs += 1;
                if self.docs > 1 {
                    self.error
                        .get_or_insert_with(|| "文書が 2 つ以上ある".to_string());
                }
            }
            Event::Scalar(value, style, anchor, tag) => {
                let null = tag.is_none()
                    && matches!(style, TScalarStyle::Plain)
                    && matches!(value.as_str(), "" | "~" | "null" | "Null" | "NULL");
                let node = if null {
                    Node::Null
                } else {
                    Node::Scalar(value)
                };
                self.push_value(node, anchor, mark);
            }
            Event::SequenceStart(anchor, _) => self.stack.push(Frame::Seq(Vec::new(), anchor)),
            Event::MappingStart(anchor, _) => self.stack.push(Frame::Map(Vec::new(), None, anchor)),
            Event::SequenceEnd => {
                if let Some(Frame::Seq(items, anchor)) = self.stack.pop() {
                    self.push_value(Node::Seq(items), anchor, mark);
                }
            }
            Event::MappingEnd => {
                if let Some(Frame::Map(entries, _, anchor)) = self.stack.pop() {
                    self.push_value(Node::Map(entries), anchor, mark);
                }
            }
            Event::Alias(id) => match self.anchors.get(&id).cloned() {
                Some(node) => self.push_value(node, 0, mark),
                None => {
                    self.error.get_or_insert_with(|| {
                        format!("{} 行: 未定義の別名（alias）", mark.line())
                    });
                }
            },
            _ => {}
        }
    }
}

/// YAML の文字列を読む。構文が壊れている・文書が 1 つでない・キーが文字列でない、は Err（読めない）。
/// 重複キーは読めたうえでの違反として `Doc::duplicates` に返す。
pub fn parse(text: &str) -> Result<Doc, String> {
    let mut builder = Builder::default();
    Parser::new_from_str(text)
        .load(&mut builder, true)
        .map_err(|e| e.to_string())?;
    if let Some(e) = builder.error {
        return Err(e);
    }
    let root = builder.root.ok_or_else(|| "文書が空".to_string())?;
    Ok(Doc {
        root,
        duplicates: builder.duplicates,
    })
}

/// 型付きの木（便 7 (b)）。day-1 の床の読み手（PyYAML の既定の解決）のうち、正本と anchor に出る型だけを持つ。
/// 表は書かれた順に持つ（重複キーは最初の値を残す・重複の記録は `parse` の側）。
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    /// 整数の字面（先頭の「+」は落とす）
    Int(String),
    Float(f64),
    /// 年-月-日の字面
    Date(String),
    Str(String),
    Seq(Vec<Value>),
    Map(Vec<(Value, Value)>),
}

impl Value {
    pub fn as_map(&self) -> Option<&[(Value, Value)]> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_seq(&self) -> Option<&[Value]> {
        match self {
            Value::Seq(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    /// 文字列のキーで引く。
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.as_map()?
            .iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .map(|(_, v)| v)
    }

    /// 床の `str(x)`（Python の文字列化）の写し。
    pub fn py_str(&self) -> String {
        match self {
            Value::Null => "None".to_string(),
            Value::Bool(true) => "True".to_string(),
            Value::Bool(false) => "False".to_string(),
            Value::Int(s) | Value::Date(s) | Value::Str(s) => s.clone(),
            Value::Float(f) => float_text(*f).unwrap_or_else(|| f.to_string()),
            Value::Seq(_) => "（一覧）".to_string(),
            Value::Map(_) => "（表）".to_string(),
        }
    }
}

enum TypedFrame {
    Seq(Vec<Value>, usize),
    /// 表・読みかけのキー・表の anchor
    Map(Vec<(Value, Value)>, Option<Value>, usize),
}

#[derive(Default)]
struct TypedBuilder {
    stack: Vec<TypedFrame>,
    root: Option<Value>,
    anchors: HashMap<usize, Value>,
    error: Option<String>,
    docs: usize,
}

impl TypedBuilder {
    /// 今の位置の欄の道（エラーの文言用）。
    fn path(&self) -> String {
        let mut out = String::new();
        for frame in &self.stack {
            match frame {
                TypedFrame::Seq(items, _) => out.push_str(&format!("[{}]", items.len())),
                TypedFrame::Map(_, Some(k), _) => out.push_str(&format!(".{}", k.py_str())),
                TypedFrame::Map(_, None, _) => out.push_str(".（キー）"),
            }
        }
        out
    }

    fn push_value(&mut self, value: Value, anchor: usize) {
        if anchor > 0 {
            self.anchors.insert(anchor, value.clone());
        }
        match self.stack.last_mut() {
            None => self.root = Some(value),
            Some(TypedFrame::Seq(items, _)) => items.push(value),
            Some(TypedFrame::Map(entries, pending, _)) => match pending.take() {
                None => *pending = Some(value),
                Some(key) => {
                    if !entries.iter().any(|(k, _)| *k == key) {
                        entries.push((key, value));
                    }
                }
            },
        }
    }
}

impl MarkedEventReceiver for TypedBuilder {
    fn on_event(&mut self, ev: Event, mark: Marker) {
        match ev {
            Event::DocumentStart => {
                self.docs += 1;
                if self.docs > 1 {
                    self.error
                        .get_or_insert_with(|| "文書が 2 つ以上ある".to_string());
                }
            }
            Event::Scalar(value, style, anchor, tag) => {
                let typed = if tag.is_some() {
                    Err(format!("{} 行: 型の印（tag）を持つ scalar", mark.line()))
                } else if matches!(style, TScalarStyle::Plain) {
                    resolve_plain(&value).ok_or_else(|| format!("正規化できない scalar「{value}」"))
                } else {
                    Ok(Value::Str(value))
                };
                match typed {
                    Ok(v) => self.push_value(v, anchor),
                    Err(e) => {
                        let at = self.path();
                        self.error
                            .get_or_insert_with(|| format!("{e}（欄の道 {at}）"));
                        self.push_value(Value::Null, anchor);
                    }
                }
            }
            Event::SequenceStart(anchor, _) => self.stack.push(TypedFrame::Seq(Vec::new(), anchor)),
            Event::MappingStart(anchor, _) => {
                self.stack.push(TypedFrame::Map(Vec::new(), None, anchor))
            }
            Event::SequenceEnd => {
                if let Some(TypedFrame::Seq(items, anchor)) = self.stack.pop() {
                    self.push_value(Value::Seq(items), anchor);
                }
            }
            Event::MappingEnd => {
                if let Some(TypedFrame::Map(entries, _, anchor)) = self.stack.pop() {
                    self.push_value(Value::Map(entries), anchor);
                }
            }
            Event::Alias(id) => match self.anchors.get(&id).cloned() {
                Some(v) => self.push_value(v, 0),
                None => {
                    self.error.get_or_insert_with(|| {
                        format!("{} 行: 未定義の別名（alias）", mark.line())
                    });
                }
            },
            _ => {}
        }
    }
}

/// YAML の文字列を型付きの木として読む（便 7 (b)）。読めない・この便で解かない plain の scalar の形は Err（まだ分からない）。
/// 重複キーは数えない（`parse` の側で数える）。
pub fn parse_typed(text: &str) -> Result<Value, String> {
    let mut builder = TypedBuilder::default();
    Parser::new_from_str(text)
        .load(&mut builder, true)
        .map_err(|e| e.to_string())?;
    if let Some(e) = builder.error {
        return Err(e);
    }
    builder.root.ok_or_else(|| "文書が空".to_string())
}

fn all_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

fn strip_sign(s: &str) -> &str {
    s.strip_prefix(['-', '+']).unwrap_or(s)
}

/// plain の scalar を型に解く。この便で解かない型付きの形（八進・「_」入り・0x / 0o / 0b・六十進・指数・「.」で始まる／終わる浮動小数・
/// .inf / .nan・時刻付きの timestamp・merge / value の印）は None。
fn resolve_plain(s: &str) -> Option<Value> {
    match s {
        "" | "~" | "null" | "Null" | "NULL" => return Some(Value::Null),
        "true" | "True" | "TRUE" | "yes" | "Yes" | "YES" | "on" | "On" | "ON" => {
            return Some(Value::Bool(true));
        }
        "false" | "False" | "FALSE" | "no" | "No" | "NO" | "off" | "Off" | "OFF" => {
            return Some(Value::Bool(false));
        }
        "<<" | "=" => return None,
        _ => {}
    }
    let body = strip_sign(s);
    if all_digits(body) {
        if body == "0" {
            return Some(Value::Int("0".to_string()));
        }
        if body.starts_with('0') {
            return None;
        }
        return Some(Value::Int(s.strip_prefix('+').unwrap_or(s).to_string()));
    }
    if let Some((int, frac)) = body.split_once('.')
        && all_digits(int)
        && all_digits(frac)
    {
        return s.parse::<f64>().ok().map(Value::Float);
    }
    if is_date(s) {
        return valid_date(s).then(|| Value::Date(s.to_string()));
    }
    if pyyaml_typed(s) {
        return None;
    }
    Some(Value::Str(s.to_string()))
}

/// 数字 4 桁-数字 2 桁-数字 2 桁。
fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && all_digits(&s[..4])
        && all_digits(&s[5..7])
        && all_digits(&s[8..])
}

/// 暦に在る日付か（床の読み手は在らない日付を読めない）。
fn valid_date(s: &str) -> bool {
    let (Ok(y), Ok(m), Ok(d)) = (
        s[..4].parse::<u32>(),
        s[5..7].parse::<u32>(),
        s[8..].parse::<u32>(),
    ) else {
        return false;
    };
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    y >= 1 && (1..=days).contains(&d)
}

/// 床の読み手（PyYAML）が数・時刻として型に解く形か（この便で解く部分集合を除いた残りを拾うのに使う）。
fn pyyaml_typed(s: &str) -> bool {
    let body = strip_sign(s);
    let bytes_in = |t: &str, ok: fn(u8) -> bool| !t.is_empty() && t.bytes().all(ok);
    let dig_ = |b: u8| b.is_ascii_digit() || b == b'_';
    // 0b / 0x / 0o
    for (prefix, ok) in [
        (
            "0b",
            (|b: u8| matches!(b, b'0' | b'1' | b'_')) as fn(u8) -> bool,
        ),
        ("0x", |b: u8| b.is_ascii_hexdigit() || b == b'_'),
        ("0o", |b: u8| matches!(b, b'0'..=b'7' | b'_')),
    ] {
        if body.strip_prefix(prefix).is_some_and(|r| bytes_in(r, ok)) {
            return true;
        }
    }
    // 「_」入り・先頭 0 の整数
    if body.as_bytes().first().is_some_and(u8::is_ascii_digit) && bytes_in(body, dig_) {
        return true;
    }
    // .inf / .nan
    if matches!(body, ".inf" | ".Inf" | ".INF") || matches!(s, ".nan" | ".NaN" | ".NAN") {
        return true;
    }
    // 浮動小数（指数付き・「.」で始まる／終わる・「_」入り）
    let (mantissa, exp_ok) = match body.find(['e', 'E']) {
        Some(i) => {
            let e = &body[i + 1..];
            let ok = e.len() >= 2 && matches!(e.as_bytes()[0], b'-' | b'+') && all_digits(&e[1..]);
            (&body[..i], ok)
        }
        None => (body, true),
    };
    if exp_ok && let Some((int, frac)) = mantissa.split_once('.') {
        let frac_ok = frac.bytes().all(dig_);
        let int_ok = if int.is_empty() {
            s == body && frac.as_bytes().first().is_some_and(u8::is_ascii_digit)
        } else {
            int.as_bytes()[0].is_ascii_digit() && int.bytes().all(dig_)
        };
        if int_ok && frac_ok {
            return true;
        }
    }
    // 六十進（「:」で区切る整数 `[1-9][0-9_]*(:[0-5]?[0-9])+`・浮動小数 `[0-9][0-9_]*(:[0-5]?[0-9])+\.[0-9_]*`）
    if let Some((head, rest)) = body.split_once(':')
        && head.as_bytes().first().is_some_and(u8::is_ascii_digit)
        && head.bytes().all(dig_)
    {
        let (groups, frac) = match rest.split_once('.') {
            Some((g, f)) => (g, Some(f)),
            None => (rest, None),
        };
        let group_ok = |g: &str| match g.as_bytes() {
            [d] => d.is_ascii_digit(),
            [a, d] => matches!(a, b'0'..=b'5') && d.is_ascii_digit(),
            _ => false,
        };
        let groups_ok = groups.split(':').all(group_ok);
        let form_ok = match frac {
            None => !head.starts_with('0'),
            Some(f) => f.bytes().all(dig_),
        };
        if groups_ok && form_ok {
            return true;
        }
    }
    is_timestamp_with_time(s)
}

/// 時刻付きの timestamp（PyYAML の timestamp の式のうち日付だけの形を除いたもの）。
fn is_timestamp_with_time(s: &str) -> bool {
    let b = s.as_bytes();
    let mut i = 0;
    let digits_at = |i: &mut usize, min: usize, max: usize| {
        let n = b[*i..]
            .iter()
            .take(max)
            .take_while(|c| c.is_ascii_digit())
            .count();
        *i += n;
        n >= min
    };
    let byte_at = |i: &mut usize, want: &[u8]| {
        if b.get(*i).is_some_and(|c| want.contains(c)) {
            *i += 1;
            true
        } else {
            false
        }
    };
    let date = digits_at(&mut i, 4, 4)
        && byte_at(&mut i, b"-")
        && digits_at(&mut i, 1, 2)
        && byte_at(&mut i, b"-")
        && digits_at(&mut i, 1, 2);
    if !date || i == b.len() {
        return false;
    }
    if !byte_at(&mut i, b"Tt") {
        let n = b[i..]
            .iter()
            .take_while(|c| matches!(c, b' ' | b'\t'))
            .count();
        if n == 0 {
            return false;
        }
        i += n;
    }
    let time = digits_at(&mut i, 1, 2)
        && byte_at(&mut i, b":")
        && digits_at(&mut i, 2, 2)
        && byte_at(&mut i, b":")
        && digits_at(&mut i, 2, 2);
    if !time {
        return false;
    }
    if byte_at(&mut i, b".") {
        i += b[i..].iter().take_while(|c| c.is_ascii_digit()).count();
    }
    i += b[i..]
        .iter()
        .take_while(|c| matches!(c, b' ' | b'\t'))
        .count();
    if i == b.len() {
        return true;
    }
    if byte_at(&mut i, b"Z") {
        return i == b.len();
    }
    if !(byte_at(&mut i, b"-+") && digits_at(&mut i, 1, 2)) {
        return false;
    }
    if byte_at(&mut i, b":") && !digits_at(&mut i, 2, 2) {
        return false;
    }
    i == b.len()
}

/// 浮動小数の Python の repr と同じ字面（指数表記になる範囲は None）。
fn float_text(f: f64) -> Option<String> {
    if !f.is_finite() || (f != 0.0 && (f.abs() >= 1e16 || f.abs() < 1e-4)) {
        return None;
    }
    let mut s = f.to_string();
    if !s.contains(['.', 'e']) {
        s.push_str(".0");
    }
    Some(s)
}

/// json の文字列の字面（床の json.dumps の既定・ensure_ascii=False と同じ escape）。
pub(crate) fn json_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// 正規化（便 7 (c)）: json の字面（キー順固定・空白なし・非 ASCII はそのまま・日付は文字列）。
/// 正規化できない値（指数表記になる浮動小数・文字列でないキー）は Err（欄の道つき）。
pub fn canonical(value: &Value) -> Result<String, String> {
    let mut out = String::new();
    write_canonical(value, "", &mut out)?;
    Ok(out)
}

fn write_canonical(value: &Value, at: &str, out: &mut String) -> Result<(), String> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(s) => out.push_str(if s == "-0" { "0" } else { s }),
        Value::Float(f) => match float_text(*f) {
            Some(s) => out.push_str(&s),
            None => return Err(format!("正規化できない scalar「{f}」（欄の道 {at}）")),
        },
        Value::Date(s) | Value::Str(s) => json_str(s, out),
        Value::Seq(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(item, &format!("{at}[{i}]"), out)?;
            }
            out.push(']');
        }
        Value::Map(entries) => {
            let mut keyed: Vec<(&str, &Value)> = Vec::with_capacity(entries.len());
            for (k, v) in entries {
                match k {
                    Value::Str(k) => keyed.push((k, v)),
                    other => {
                        return Err(format!(
                            "正規化できない scalar（文字列でないキー「{}」・欄の道 {at}）",
                            other.py_str()
                        ));
                    }
                }
            }
            keyed.sort_by(|a, b| a.0.cmp(b.0));
            out.push('{');
            for (i, (k, v)) in keyed.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                json_str(k, out);
                out.push(':');
                write_canonical(v, &format!("{at}.{k}"), out)?;
            }
            out.push('}');
        }
    }
    Ok(())
}

/// yaml の書き手（便 9 (d)）。型付きの木を、PyYAML の既定の読み手と `parse_typed` の両方で同じ型付きの木に読める字面で書く。
/// 表は各欄を 1 行「キー: 値」（入れ子は 2 空白の字下げ）・一覧の各要素は「- 」・空の一覧は「[]」・空の表は「{}」・
/// 文字列（キーを含む）は必ず二重引用符で囲む。1 行目に `header`（「# 」で始まる注釈）を置く。
/// 書けない値（指数表記になる浮動小数）は Err（欄の道つき）。
pub fn write(value: &Value, header: &str) -> Result<String, String> {
    let mut out = String::new();
    out.push_str(header);
    out.push('\n');
    match value {
        Value::Seq(items) if !items.is_empty() => write_block(value, 0, "", &mut out)?,
        Value::Map(entries) if !entries.is_empty() => write_block(value, 0, "", &mut out)?,
        other => {
            write_scalar(other, "", &mut out)?;
            out.push('\n');
        }
    }
    Ok(out)
}

/// 空でない一覧・表を `indent` 空白の字下げで 1 行ずつ書く。
fn write_block(value: &Value, indent: usize, at: &str, out: &mut String) -> Result<(), String> {
    let pad = " ".repeat(indent);
    match value {
        Value::Seq(items) => {
            for (i, item) in items.iter().enumerate() {
                let here = format!("{at}[{i}]");
                if is_block(item) {
                    // 子の区間を indent + 2 で書き、1 行目の字下げを「- 」に替える
                    let mut child = String::new();
                    write_block(item, indent + 2, &here, &mut child)?;
                    out.push_str(&pad);
                    out.push_str("- ");
                    out.push_str(&child[indent + 2..]);
                } else {
                    out.push_str(&pad);
                    out.push_str("- ");
                    write_scalar(item, &here, out)?;
                    out.push('\n');
                }
            }
        }
        Value::Map(entries) => {
            for (k, v) in entries {
                let Value::Str(key) = k else {
                    return Err(format!(
                        "書けない値（文字列でないキー「{}」・欄の道 {at}）",
                        k.py_str()
                    ));
                };
                let here = format!("{at}.{key}");
                out.push_str(&pad);
                quoted(key, out);
                out.push(':');
                if is_block(v) {
                    out.push('\n');
                    write_block(v, indent + 2, &here, out)?;
                } else {
                    out.push(' ');
                    write_scalar(v, &here, out)?;
                    out.push('\n');
                }
            }
        }
        other => {
            out.push_str(&pad);
            write_scalar(other, at, out)?;
            out.push('\n');
        }
    }
    Ok(())
}

/// 空でない一覧・表（区間として書くもの）か。
fn is_block(value: &Value) -> bool {
    match value {
        Value::Seq(items) => !items.is_empty(),
        Value::Map(entries) => !entries.is_empty(),
        _ => false,
    }
}

/// 1 行に収まる値（scalar・空の一覧・空の表）。
fn write_scalar(value: &Value, at: &str, out: &mut String) -> Result<(), String> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(s) => out.push_str(s),
        Value::Float(f) => match float_text(*f) {
            Some(s) => out.push_str(&s),
            None => return Err(format!("書けない値「{f}」（欄の道 {at}）")),
        },
        Value::Date(s) => out.push_str(s),
        Value::Str(s) => quoted(s, out),
        Value::Seq(_) => out.push_str("[]"),
        Value::Map(_) => out.push_str("{}"),
    }
    Ok(())
}

/// yaml の二重引用符の字面。二重引用符と逆斜線は逆斜線を前置・改行は \n・他の制御文字（と yaml が改行や
/// 読めない字とみなす字）は \u + 4 桁・他はそのまま。
fn quoted(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20
                || ('\u{7f}'..='\u{9f}').contains(&c)
                || matches!(
                    c,
                    '\u{2028}' | '\u{2029}' | '\u{feff}' | '\u{fffe}' | '\u{ffff}'
                ) =>
            {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 書いて読み直すと同じ型付きの木（と同じ正規化）になる。
    fn round_trip(v: &Value) {
        let text = write(v, "# 注釈").unwrap();
        assert!(text.starts_with("# 注釈\n"), "{text}");
        let back = parse_typed(&text).unwrap_or_else(|e| panic!("{e}\n{text}"));
        assert_eq!(&back, v, "{text}");
        assert_eq!(canonical(&back).unwrap(), canonical(v).unwrap());
        assert!(parse(&text).unwrap().duplicates.is_empty());
    }

    #[test]
    fn yaml_writer_round_trips_the_main_anchor() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../design-intent/anchors/constitution-v1.0.yaml"
        );
        let text = std::fs::read_to_string(path).unwrap();
        let v = parse_typed(&text).unwrap();
        round_trip(&v);
        let digest = |v: &Value| {
            let body: Vec<(Value, Value)> = v
                .as_map()
                .unwrap()
                .iter()
                .filter(|(k, _)| k.as_str() != Some("digest"))
                .cloned()
                .collect();
            crate::sha256::hex(canonical(&Value::Map(body)).unwrap().as_bytes())
        };
        let back = parse_typed(&write(&v, "# x").unwrap()).unwrap();
        assert_eq!(digest(&back), digest(&v));
        assert_eq!(
            Some(digest(&back).as_str()),
            v.get("digest").and_then(Value::as_str)
        );
    }

    #[test]
    fn yaml_writer_round_trips_awkward_strings() {
        let s = |x: &str| Value::Str(x.to_string());
        let v = Value::Map(vec![
            (s("q\"k"), s("二重\"引用符")),
            (s("back\\slash"), s("C:\\dir\\")),
            (s("nl"), s("一行目\n二行目\r\t\u{1}\u{7f}\u{85}\u{2028}")),
            (
                s("truthy"),
                Value::Seq(vec![
                    s("true"),
                    s("no"),
                    s("null"),
                    s("~"),
                    s("1"),
                    s("1.5"),
                    s("2026-09-17"),
                    s(""),
                    s("- x"),
                    s("# 注釈"),
                    s("a: b"),
                    s("[1]"),
                    s("{a: 1}"),
                    s("&x"),
                    s("*x"),
                    s(" 前後の空白 "),
                ]),
            ),
            (
                s("typed"),
                Value::Seq(vec![
                    Value::Null,
                    Value::Bool(true),
                    Value::Bool(false),
                    Value::Int("-12".into()),
                    Value::Float(1.5),
                    Value::Date("2026-09-17".into()),
                    Value::Seq(vec![]),
                    Value::Map(vec![]),
                ]),
            ),
            (
                s("nested"),
                Value::Seq(vec![
                    Value::Map(vec![
                        (s("a"), Value::Seq(vec![s("x"), Value::Seq(vec![s("y")])])),
                        (s("b"), Value::Map(vec![(s("c"), Value::Null)])),
                    ]),
                    Value::Seq(vec![Value::Map(vec![(s("d"), s("e"))]), s("f")]),
                ]),
            ),
            (s("empty"), Value::Map(vec![])),
        ]);
        round_trip(&v);
        assert!(write(&Value::Float(1e20), "# x").is_err());
    }

    #[test]
    fn typed_plain_scalars_follow_the_floor_reader() {
        let v = parse_typed(
            "a: \nb: ~\nc: yes\nd: Off\ne: -12\nf: +3\ng: 1.50\nh: 2026-09-17\ni: '1'\nj: v1.0\nk: 12a\n",
        )
        .unwrap();
        let want = [
            ("a", Value::Null),
            ("b", Value::Null),
            ("c", Value::Bool(true)),
            ("d", Value::Bool(false)),
            ("e", Value::Int("-12".into())),
            ("f", Value::Int("3".into())),
            ("g", Value::Float(1.5)),
            ("h", Value::Date("2026-09-17".into())),
            ("i", Value::Str("1".into())),
            ("j", Value::Str("v1.0".into())),
            ("k", Value::Str("12a".into())),
        ];
        for (k, w) in want {
            assert_eq!(v.get(k), Some(&w), "{k}");
        }
        for ng in [
            "010",
            "1_000",
            "0x1f",
            "0o17",
            "0b10",
            "1:30",
            "1.0e+5",
            ".5",
            "1.",
            ".inf",
            "-.INF",
            ".nan",
            "2026-09-17 10:00:00",
            "2026-09-17T10:00:00Z",
            "2026-02-30",
        ] {
            let e = parse_typed(&format!("x:\n  y: {ng}\n")).unwrap_err();
            assert!(
                e.contains("正規化できない") && e.contains(".x.y"),
                "{ng}: {e}"
            );
        }
        assert!(parse_typed("x: '010'\n").is_ok());
    }

    #[test]
    fn canonical_is_the_floor_json() {
        let v = parse_typed(
            "b: [1, 2.0, -0.5, true, null]\na: \"q\\\"\\\\\\n\\t\\u0001あ\"\nd: 2026-09-17\nc: {z: 1, y: x}\n",
        )
        .unwrap();
        assert_eq!(
            canonical(&v).unwrap(),
            "{\"a\":\"q\\\"\\\\\\n\\t\\u0001あ\",\"b\":[1,2.0,-0.5,true,null],\"c\":{\"y\":\"x\",\"z\":1},\"d\":\"2026-09-17\"}"
        );
        assert!(canonical(&Value::Float(1e16)).is_err());
        assert!(canonical(&Value::Float(0.00001)).is_err());
        assert_eq!(canonical(&Value::Float(0.0)).unwrap(), "0.0");
        assert!(canonical(&parse_typed("1: x\n").unwrap()).is_err());
    }

    #[test]
    fn duplicate_keys_are_found_in_nested_and_flow_maps() {
        let doc = parse("a: 1\nb:\n  c: 2\n  c: 3\nd: [{e: 1, e: 2}]\n").unwrap();
        let keys: Vec<_> = doc.duplicates.iter().map(|d| d.key.as_str()).collect();
        assert_eq!(keys, ["c", "e"]);
        assert_eq!(
            doc.root.get("b").unwrap().get("c"),
            Some(&Node::Scalar("2".into()))
        );
    }

    #[test]
    fn blank_and_broken() {
        let doc = parse("a: \"\"\nb:\nc: ~\nd: x\n").unwrap();
        for k in ["a", "b", "c"] {
            assert!(doc.root.get(k).unwrap().is_blank(), "{k}");
        }
        assert!(!doc.root.get("d").unwrap().is_blank());
        assert!(parse("a: [1, 2\n").is_err());
        assert!(parse("").is_err());
    }
}
