//! Cargo.toml の読み（器の読み手が読める TOML の部分集合・読めない形は `None`＝撃つ側・設計 contract-source.md §28）。
//!
//! 受付の base の木の実走（[`super::cli::base_run`]）が `[package]` と `[workspace]` と test の target の名を読み、歯の区間
//! （[`super::closure`] の `--test <名>` の読み）が `[[test]]` の表ごとの `name` と `path` の対を読む（[`test_paths`]）。
//! 字句 [`tokens`] と読み手 [`Parser`] は 1 本で、依存は足さない。

use std::iter::Peekable;
use std::str::Chars;

/// Cargo.toml から読む事実。
#[derive(Debug, Default)]
pub(crate) struct CargoToml {
    /// `[package]` の表を持つ。
    pub(crate) package_table: bool,
    /// `[package]` の `name`（文字列の周だけ）。
    pub(crate) package: Option<String>,
    /// `[workspace]` の表を持つ。
    pub(crate) workspace: bool,
    /// `[workspace]` の `members`。
    pub(crate) members: Vec<String>,
    /// test の target の名（`[[test]]` の表と、`test` の key の inline の表の配列の `name`）。
    pub(crate) tests: Vec<String>,
    /// `[[test]]` の表ごとの (`name`, `path`)（見出しを読むたびに空の対を 1 つ積み、鍵の文字列を最後の対に入れる・読めた
    /// 欄だけが非空）。
    test_paths: Vec<(String, String)>,
}

impl CargoToml {
    /// 1 つの key と値を取り込む（`members` の要素が文字列でない周は読めない）。
    fn take(&mut self, table: &str, array: bool, key: &str, value: Toml) -> Option<()> {
        match (table, array, key, value) {
            ("package", false, "name", Toml::Text(name)) => self.package = Some(name),
            ("workspace", false, "members", Toml::List(items)) => {
                for item in items {
                    let Toml::Text(member) = item else {
                        return None;
                    };
                    self.members.push(member);
                }
            }
            ("test", true, "name", Toml::Text(name)) => {
                self.tests.push(name.clone());
                if let Some(last) = self.test_paths.last_mut() {
                    last.0 = name;
                }
            }
            ("test", true, "path", Toml::Text(path)) => {
                if let Some(last) = self.test_paths.last_mut() {
                    last.1 = path;
                }
            }
            (_, _, "test", Toml::List(items)) => self.tests.extend(items.into_iter().filter_map(inline_name)),
            _ => {}
        }
        Some(())
    }
}

/// Cargo.toml の本文の `[[test]]` の表ごとの (`name`, `path`)（宣言順・両方を持つ表だけ・読めない本文は空の列）。
/// inline の表の配列の形（`test` の key の値の配列）の `path` は読まない。
pub(crate) fn test_paths(text: &str) -> Vec<(String, String)> {
    let found = cargo_toml(text).map(|found| found.test_paths).unwrap_or_default();
    found.into_iter().filter(|(name, path)| !name.is_empty() && !path.is_empty()).collect()
}

/// inline の表の `name`（文字列の周だけ）。
fn inline_name(item: Toml) -> Option<String> {
    let Toml::Table(pairs) = item else {
        return None;
    };
    pairs.into_iter().find_map(|(key, value)| match value {
        Toml::Text(name) if key == "name" => Some(name),
        _ => None,
    })
}

/// Cargo.toml の本文を読む（表の見出し・key と値の行・空行とコメントだけを受ける）。
pub(crate) fn cargo_toml(text: &str) -> Option<CargoToml> {
    let mut parser = Parser { tokens: tokens(text)?, at: 0 };
    let mut found = CargoToml::default();
    let (mut table, mut array) = (String::new(), false);
    loop {
        parser.skip_newlines();
        match parser.next() {
            None => return Some(found),
            Some(Token::Mark('[')) => {
                array = parser.eat('[');
                table = parser.key()?;
                if !parser.eat(']') || (array && !parser.eat(']')) {
                    return None;
                }
                found.package_table |= !array && table == "package";
                found.workspace |= !array && table == "workspace";
                if array && table == "test" {
                    found.test_paths.push((String::new(), String::new()));
                }
            }
            Some(Token::Bare(key) | Token::Text(key)) => {
                if !parser.eat('=') {
                    return None;
                }
                let value = parser.value()?;
                found.take(&table, array, &key, value)?;
            }
            Some(_) => return None,
        }
        if !matches!(parser.next(), None | Some(Token::Newline)) {
            return None;
        }
    }
}

/// TOML の字句。
#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    /// 引用符の文字列。
    Text(String),
    /// 裸の語（key・数・真偽・日付）。
    Bare(String),
    /// 記号（`[` `]` `{` `}` `=` `,`）。
    Mark(char),
    /// 改行。
    Newline,
}

/// TOML の値（読み手が要る形だけを残す）。
enum Toml {
    /// 文字列。
    Text(String),
    /// 裸の値（数・真偽・日付）。
    Bare,
    /// 配列。
    List(Vec<Toml>),
    /// inline の表。
    Table(Vec<(String, Toml)>),
}

/// 裸の語を成す字か。
fn bare_char(found: char) -> bool {
    found.is_ascii_alphanumeric() || "_-.+:".contains(found)
}

/// 本文を字句に割る（複数行の文字列と未知の字は読めない）。
fn tokens(text: &str) -> Option<Vec<Token>> {
    let mut found = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(head) = chars.next() {
        match head {
            ' ' | '\t' | '\r' => {}
            '\n' => found.push(Token::Newline),
            '#' => while chars.next_if(|next| *next != '\n').is_some() {},
            '[' | ']' | '{' | '}' | '=' | ',' => found.push(Token::Mark(head)),
            '"' | '\'' => found.push(Token::Text(quoted(&mut chars, head)?)),
            _ if bare_char(head) => {
                let mut word = String::from(head);
                while let Some(next) = chars.next_if(|next| bare_char(*next)) {
                    word.push(next);
                }
                found.push(Token::Bare(word));
            }
            _ => return None,
        }
    }
    Some(found)
}

/// 引用符 `quote` の文字列の残り（開きの後から）。`"` は `\` の escape を 5 つだけ読む・複数行の形は読めない。
fn quoted(chars: &mut Peekable<Chars<'_>>, quote: char) -> Option<String> {
    if chars.next_if_eq(&quote).is_some() {
        return chars.next_if_eq(&quote).is_none().then(String::new);
    }
    let mut text = String::new();
    loop {
        match chars.next()? {
            found if found == quote => return Some(text),
            '\n' => return None,
            '\\' if quote == '"' => text.push(match chars.next()? {
                '"' => '"',
                '\\' => '\\',
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                _ => return None,
            }),
            other => text.push(other),
        }
    }
}

/// 字句の読み手。
struct Parser {
    /// 字句の列。
    tokens: Vec<Token>,
    /// 次に読む位置。
    at: usize,
}

impl Parser {
    /// 次の字句を取る。
    fn next(&mut self) -> Option<Token> {
        let found = self.tokens.get(self.at).cloned();
        self.at = self.at.saturating_add(1);
        found
    }

    /// 次が記号 `mark` なら取って真。
    fn eat(&mut self, mark: char) -> bool {
        let hit = self.tokens.get(self.at) == Some(&Token::Mark(mark));
        if hit {
            self.at = self.at.saturating_add(1);
        }
        hit
    }

    /// 改行を読み飛ばす。
    fn skip_newlines(&mut self) {
        while self.tokens.get(self.at) == Some(&Token::Newline) {
            self.at = self.at.saturating_add(1);
        }
    }

    /// key（裸の語か文字列）。
    fn key(&mut self) -> Option<String> {
        match self.next()? {
            Token::Bare(key) | Token::Text(key) => Some(key),
            Token::Mark(_) | Token::Newline => None,
        }
    }

    /// 値 1 つ（配列は改行を跨げる・inline の表は 1 行）。
    fn value(&mut self) -> Option<Toml> {
        match self.next()? {
            Token::Text(text) => Some(Toml::Text(text)),
            Token::Bare(_) => Some(Toml::Bare),
            Token::Mark('[') => self.list().map(Toml::List),
            Token::Mark('{') => self.table().map(Toml::Table),
            Token::Mark(_) | Token::Newline => None,
        }
    }

    /// 配列の残り（開きの後から閉じまで・末尾の `,` を許す）。
    fn list(&mut self) -> Option<Vec<Toml>> {
        let mut items = Vec::new();
        loop {
            self.skip_newlines();
            if self.eat(']') {
                return Some(items);
            }
            items.push(self.value()?);
            self.skip_newlines();
            if !self.eat(',') {
                self.skip_newlines();
                return self.eat(']').then_some(items);
            }
        }
    }

    /// inline の表の残り（開きの後から閉じまで）。
    fn table(&mut self) -> Option<Vec<(String, Toml)>> {
        let mut pairs = Vec::new();
        if self.eat('}') {
            return Some(pairs);
        }
        loop {
            let key = self.key()?;
            if !self.eat('=') {
                return None;
            }
            pairs.push((key, self.value()?));
            if self.eat('}') {
                return Some(pairs);
            }
            if !self.eat(',') {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_paths;

    /// 歯 vtpath_cargo_toml_pairs_test_name_and_path: tsuzuri-boundary と同じ形の本文（`[package]`・`[[bin]]`・inline の表の依存・
    /// 5 つの `[[test]]`）と path の無い `[[test]]` の表 1 つで、対は 5 つ・path の無い表の名は対に無い。
    #[test]
    fn vtpath_cargo_toml_pairs_test_name_and_path() {
        let mut text = String::from(
            "[package]\nname = \"tsuzuri-boundary\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
             [[bin]]\nname = \"tsuzuri-boundary\"\npath = \"src/main.rs\"\n\n\
             [dependencies]\nfolio = { path = \"../../folio2/crates/folio\" }\n\n",
        );
        for n in 1..=5 {
            text.push_str(&format!("[[test]]\nname = \"tz{n}\"\npath = \"../../folio2/crates/folio/tests/tz{n}/main.rs\"\n\n"));
        }
        text.push_str("[[test]]\nname = \"nopath\"\n");
        let pairs = test_paths(&text);
        let want = ("tz4".to_owned(), "../../folio2/crates/folio/tests/tz4/main.rs".to_owned());
        assert!(pairs.contains(&want), "tz4 の対が在る: {pairs:?}");
        assert_eq!(pairs.len(), 5, "対は path を持つ 5 つの表だけ: {pairs:?}");
        assert!(pairs.iter().all(|(name, _)| name != "nopath"), "path の無い表の名は対に無い: {pairs:?}");
        assert!(test_paths("[[test]]\nname = \"\"\"x\"\"\"\n").is_empty(), "読めない本文は空の列");
    }
}
