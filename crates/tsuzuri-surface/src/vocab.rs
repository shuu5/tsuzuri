//! 語彙（見本の vocab.json を面の crate に写した file・便 g-frame）。見出しの語と「?」の注釈の字はここから引く。
//! 2 欄（english = 英語のまま + 注釈・rephrase = 日本語の見出し）の和集合を鍵で引く（見本の ui.js の vt と同じ）。
//! 面の crate は外の依存を足さないので、JSON は小さな読みで読む（object・array・字・数・真偽・null）。

use std::collections::BTreeMap;
use std::sync::OnceLock;

/// 面の crate に写した語彙の file の中身。
pub const SOURCE: &str = include_str!("../vocab.json");

/// 1 つの語（見出しの語・注釈の本文・内部の名）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    pub label: String,
    /// 注釈の本文（english の note・rephrase の plain）。
    pub note: String,
    /// 経験者に出す内部の名（rephrase の原語は末尾に足す）。
    pub internal: String,
}

/// 語彙の表（鍵 → 語）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vocab {
    terms: BTreeMap<String, Term>,
}

impl Vocab {
    /// 語彙の file を読む（english を先に引き、同じ鍵が rephrase にも在れば english が勝つ）。
    pub fn parse(text: &str) -> Result<Self, String> {
        let root = json::parse(text)?;
        let mut terms = BTreeMap::new();
        for (col, note_key) in [("rephrase", "plain"), ("english", "note")] {
            let Some(json::Value::Object(entries)) = root.get(col) else {
                return Err(format!("欄 {col} が無い"));
            };
            for (key, entry) in entries {
                let text_of = |k: &str| entry.get(k).and_then(json::Value::as_str).unwrap_or("");
                let mut internal = text_of("internal").to_string();
                if let Some(orig) = entry.get("orig").and_then(json::Value::as_str) {
                    internal.push_str("\n原語: ");
                    internal.push_str(orig);
                }
                terms.insert(
                    key.clone(),
                    Term {
                        label: text_of("label").to_string(),
                        note: text_of(note_key).to_string(),
                        internal,
                    },
                );
            }
        }
        Ok(Self { terms })
    }

    pub fn term(&self, key: &str) -> Option<&Term> {
        self.terms.get(key)
    }

    /// 見出しの語（表に無い鍵は無いと分かる字にする・見本と同じ）。
    pub fn label(&self, key: &str) -> String {
        match self.terms.get(key) {
            Some(t) => t.label.clone(),
            None => format!("〔語彙表に無い: {key}〕"),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.terms.keys().map(String::as_str)
    }
}

/// 面の crate に写した語彙（最初に引いたときに 1 度だけ読む・読めなければ空の語彙で、見出しは語彙表に無いの字）。
pub fn vocab() -> &'static Vocab {
    static VOCAB: OnceLock<Vocab> = OnceLock::new();
    VOCAB.get_or_init(|| {
        Vocab::parse(SOURCE).unwrap_or_else(|_| Vocab {
            terms: BTreeMap::new(),
        })
    })
}

/// 鍵の見出しの語。
pub fn label(key: &str) -> String {
    vocab().label(key)
}

/// 小さな JSON の読み（語彙の file のための・数は f64）。
mod json {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Value {
        Null,
        Bool(bool),
        Number(f64),
        String(String),
        Array(Vec<Value>),
        Object(Vec<(String, Value)>),
    }

    impl Value {
        pub fn get(&self, key: &str) -> Option<&Value> {
            match self {
                Value::Object(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
                _ => None,
            }
        }

        pub fn as_str(&self) -> Option<&str> {
            match self {
                Value::String(s) => Some(s),
                _ => None,
            }
        }
    }

    pub fn parse(text: &str) -> Result<Value, String> {
        let mut p = Parser {
            chars: text.chars().collect(),
            at: 0,
        };
        let value = p.value()?;
        p.space();
        if p.at != p.chars.len() {
            return Err(format!("{} 字目の後に余りがある", p.at));
        }
        Ok(value)
    }

    struct Parser {
        chars: Vec<char>,
        at: usize,
    }

    impl Parser {
        fn peek(&self) -> Option<char> {
            self.chars.get(self.at).copied()
        }

        fn next(&mut self) -> Result<char, String> {
            let c = self.peek().ok_or("途中で終わった")?;
            self.at += 1;
            Ok(c)
        }

        fn space(&mut self) {
            while self.peek().is_some_and(char::is_whitespace) {
                self.at += 1;
            }
        }

        fn expect(&mut self, want: char) -> Result<(), String> {
            let got = self.next()?;
            if got == want {
                Ok(())
            } else {
                Err(format!("{} 字目は {want} のはずが {got}", self.at - 1))
            }
        }

        fn word(&mut self, word: &str, value: Value) -> Result<Value, String> {
            for want in word.chars() {
                self.expect(want)?;
            }
            Ok(value)
        }

        fn value(&mut self) -> Result<Value, String> {
            self.space();
            match self.peek().ok_or("値が無い")? {
                '{' => self.object(),
                '[' => self.array(),
                '"' => self.string().map(Value::String),
                't' => self.word("true", Value::Bool(true)),
                'f' => self.word("false", Value::Bool(false)),
                'n' => self.word("null", Value::Null),
                _ => self.number(),
            }
        }

        fn object(&mut self) -> Result<Value, String> {
            self.expect('{')?;
            let mut entries = Vec::new();
            self.space();
            if self.peek() == Some('}') {
                self.at += 1;
                return Ok(Value::Object(entries));
            }
            loop {
                self.space();
                let key = self.string()?;
                self.space();
                self.expect(':')?;
                entries.push((key, self.value()?));
                self.space();
                match self.next()? {
                    ',' => continue,
                    '}' => return Ok(Value::Object(entries)),
                    c => {
                        return Err(format!(
                            "{} 字目の {c} は object の区切りでない",
                            self.at - 1
                        ));
                    }
                }
            }
        }

        fn array(&mut self) -> Result<Value, String> {
            self.expect('[')?;
            let mut items = Vec::new();
            self.space();
            if self.peek() == Some(']') {
                self.at += 1;
                return Ok(Value::Array(items));
            }
            loop {
                items.push(self.value()?);
                self.space();
                match self.next()? {
                    ',' => continue,
                    ']' => return Ok(Value::Array(items)),
                    c => {
                        return Err(format!(
                            "{} 字目の {c} は array の区切りでない",
                            self.at - 1
                        ));
                    }
                }
            }
        }

        fn string(&mut self) -> Result<String, String> {
            self.expect('"')?;
            let mut out = String::new();
            loop {
                match self.next()? {
                    '"' => return Ok(out),
                    '\\' => match self.next()? {
                        'n' => out.push('\n'),
                        't' => out.push('\t'),
                        'r' => out.push('\r'),
                        'b' => out.push('\u{8}'),
                        'f' => out.push('\u{c}'),
                        'u' => out.push(self.unicode()?),
                        c @ ('"' | '\\' | '/') => out.push(c),
                        c => return Err(format!("知らない escape \\{c}")),
                    },
                    c => out.push(c),
                }
            }
        }

        fn hex4(&mut self) -> Result<u32, String> {
            let mut n = 0;
            for _ in 0..4 {
                let d = self.next()?.to_digit(16).ok_or("\\u の後が 16 進でない")?;
                n = n * 16 + d;
            }
            Ok(n)
        }

        /// `\uXXXX`（代理対は 2 つ続けて 1 字にする）。
        fn unicode(&mut self) -> Result<char, String> {
            let hi = self.hex4()?;
            let code = if (0xD800..0xDC00).contains(&hi) {
                self.expect('\\')?;
                self.expect('u')?;
                let lo = self.hex4()?;
                0x10000 + ((hi - 0xD800) << 10) + (lo.wrapping_sub(0xDC00) & 0x3FF)
            } else {
                hi
            };
            char::from_u32(code).ok_or_else(|| format!("\\u{code:x} は字でない"))
        }

        fn number(&mut self) -> Result<Value, String> {
            let start = self.at;
            while self
                .peek()
                .is_some_and(|c| c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E'))
            {
                self.at += 1;
            }
            let text: String = self
                .chars
                .get(start..self.at)
                .unwrap_or_default()
                .iter()
                .collect();
            text.parse::<f64>()
                .map(Value::Number)
                .map_err(|_| format!("{start} 字目の {text:?} は数でない"))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{Value, parse};

        #[test]
        fn frame_json_reads_nested_values() {
            let v = parse(
                r#" {"a": [1, -2.5e1, true, false, null], "b": {"c": "x\n\"y\"é😀"}, "d": {}} "#,
            )
            .expect("読める");
            assert_eq!(
                v.get("a"),
                Some(&Value::Array(vec![
                    Value::Number(1.0),
                    Value::Number(-25.0),
                    Value::Bool(true),
                    Value::Bool(false),
                    Value::Null,
                ]))
            );
            assert_eq!(
                v.get("b").and_then(|b| b.get("c")).and_then(Value::as_str),
                Some("x\n\"y\"é😀")
            );
            assert_eq!(v.get("d"), Some(&Value::Object(vec![])));
            assert!(parse("{\"a\": 1} x").is_err());
            assert!(parse("{\"a\" 1}").is_err());
            assert!(parse("[1, 2").is_err());
        }
    }
}
