//! 表示面の小さな JSON の読み書き（行 i-3）。
//! 境界の crate は serde に直に依存しないので std だけで書く。値の型は組まず、字の切り出しだけを持つ
//! （CDP の応答と event・行 i-2 の /json/version と /json/list の応答・行 i-5 の写真と DOM と console の応答が使う）。

/// 字を引用符で囲んだ JSON の字にする。
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// JSON の object の字の一番上の段の鍵が key の組の値の字（前後の空白を除く・字の値は引用符ごと・object と配列は括弧ごと）。
/// object でない字・鍵の無い字・形の読めない字は None。
pub fn member<'a>(object: &'a str, key: &str) -> Option<&'a str> {
    let b = object.as_bytes();
    let mut i = skip_ws(b, 0);
    if b.get(i) != Some(&b'{') {
        return None;
    }
    i = skip_ws(b, i + 1);
    let mut found = None;
    if b.get(i) == Some(&b'}') {
        i += 1;
    } else {
        loop {
            if b.get(i) != Some(&b'"') {
                return None;
            }
            let name_end = skip_string(b, i)?;
            let name = unquote(&object[i..name_end])?;
            i = skip_ws(b, name_end);
            if b.get(i) != Some(&b':') {
                return None;
            }
            let start = skip_ws(b, i + 1);
            let end = skip_value(b, start)?;
            if found.is_none() && name == key {
                found = Some(&object[start..end]);
            }
            i = skip_ws(b, end);
            match b.get(i) {
                Some(b',') => i = skip_ws(b, i + 1),
                Some(b'}') => {
                    i += 1;
                    break;
                }
                _ => return None,
            }
        }
    }
    if skip_ws(b, i) != b.len() {
        return None;
    }
    found
}

/// JSON の配列の字の一番上の段の要素の字の列（並びの順・前後の空白を除く・字の値は引用符ごと・object と配列は括弧ごと）。
/// 配列でない字・閉じの括弧の無い字・閉じの括弧の後に字の残る字・コンマの前後の要素の欠けた字は None。
pub fn items(array: &str) -> Option<Vec<&str>> {
    let b = array.as_bytes();
    let mut i = skip_ws(b, 0);
    if b.get(i) != Some(&b'[') {
        return None;
    }
    i = skip_ws(b, i + 1);
    let mut out = Vec::new();
    if b.get(i) == Some(&b']') {
        i += 1;
    } else {
        loop {
            let end = skip_value(b, i)?;
            out.push(&array[i..end]);
            i = skip_ws(b, end);
            match b.get(i) {
                Some(b',') => i = skip_ws(b, i + 1),
                Some(b']') => {
                    i += 1;
                    break;
                }
                _ => return None,
            }
        }
    }
    (skip_ws(b, i) == b.len()).then_some(out)
}

/// JSON の字の値を字に戻す。字の値でない字・知らない逃がし・対の欠けた代理の値は None。
pub fn unquote(text: &str) -> Option<String> {
    let inner = text.strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return None,
            '\\' => match chars.next()? {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'b' => out.push('\u{8}'),
                'f' => out.push('\u{c}'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                'u' => {
                    let high = hex4(&mut chars)?;
                    let code = match high {
                        0xD800..=0xDBFF => {
                            if chars.next()? != '\\' || chars.next()? != 'u' {
                                return None;
                            }
                            let low = hex4(&mut chars)?;
                            if !(0xDC00..=0xDFFF).contains(&low) {
                                return None;
                            }
                            0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00)
                        }
                        0xDC00..=0xDFFF => return None,
                        _ => high,
                    };
                    out.push(char::from_u32(code)?);
                }
                _ => return None,
            },
            c if u32::from(c) < 0x20 => return None,
            c => out.push(c),
        }
    }
    Some(out)
}

fn hex4(chars: &mut std::str::Chars) -> Option<u32> {
    let mut value = 0;
    for _ in 0..4 {
        value = value * 16 + chars.next()?.to_digit(16)?;
    }
    Some(value)
}

fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while matches!(b.get(i), Some(b' ' | b'\t' | b'\r' | b'\n')) {
        i += 1;
    }
    i
}

/// i の引用符で始まる字の値の終わりの次の位置。
fn skip_string(b: &[u8], i: usize) -> Option<usize> {
    let mut j = i + 1;
    loop {
        match *b.get(j)? {
            b'"' => return Some(j + 1),
            b'\\' => j += 2,
            c if c < 0x20 => return None,
            _ => j += 1,
        }
    }
}

/// i で始まる値の終わりの次の位置（入れ子は括弧の対と字の値だけを見て跨ぐ）。
fn skip_value(b: &[u8], i: usize) -> Option<usize> {
    match *b.get(i)? {
        b'"' => skip_string(b, i),
        open @ (b'{' | b'[') => {
            let mut stack = vec![open];
            let mut j = i + 1;
            while let Some(&top) = stack.last() {
                match *b.get(j)? {
                    b'"' => {
                        j = skip_string(b, j)?;
                        continue;
                    }
                    c @ (b'{' | b'[') => stack.push(c),
                    b'}' if top == b'{' => {
                        stack.pop();
                    }
                    b']' if top == b'[' => {
                        stack.pop();
                    }
                    b'}' | b']' => return None,
                    _ => {}
                }
                j += 1;
            }
            Some(j)
        }
        _ => {
            let rest = b.get(i..)?;
            let len = rest
                .iter()
                .position(|c| matches!(c, b',' | b'}' | b']' | b' ' | b'\t' | b'\r' | b'\n'))
                .unwrap_or(rest.len());
            let end = i + len;
            let word = rest.get(..len)?;
            let number = word.iter().any(u8::is_ascii_digit)
                && word
                    .iter()
                    .all(|c| c.is_ascii_digit() || matches!(c, b'-' | b'+' | b'.' | b'e' | b'E'));
            (number || matches!(word, b"true" | b"false" | b"null")).then_some(end)
        }
    }
}
