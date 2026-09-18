//! 設計ノートの散文の門（要件書 FR12・rules 行 R-16・便 24・docs/design/delivery-24.md §1）。
//! 数えるのは「規範の印を持つ文」だけで、記述の文は自由である。印を持つ文は (1) 同じ文の中に参照 id を
//! 1 つ以上持ち、(2) 参照 id の外に「数字列 + 単位」を持たない。どちらかを欠けば種別 prose-gate の違反。
//! 印・「禁止」の直後の文字・単位の一覧は検査のたびに rules 行 R-16 の value（型付きデータ）から読み、
//! 生成器にも散文にも写さない（P-5.1・R-16 の note）。式は器 scribe2 の暫定の床と同じ（ADR-3 決定 (7)）。
//! 参照 id は形だけを取り、実在の解決は呼び手（便 23 の `note.rs` の参照の解決の母集団）が行う。
//! 正規表現は使わない（字の走査だけ・便 23 と同じ作り）。

use crate::yaml::Node;

/// 一覧を持つ rules 行の id と、その行が在る節。
const ROW_ID: &str = "R-16";
const RULE_SECTIONS: [&str; 2] = ["thresholds", "discipline"];

/// 違反の理由の名（文言に出す）。
const NO_POINTER: &str = "no-pointer";
const NUMBER_WITH_UNIT: &str = "number-with-unit";

/// 文言に載せる文の先頭の字数。
const HEAD: usize = 60;

/// 母集団から落とす行の印（code fence の開き / 閉じ・表の行・注釈の行）。
const FENCE: &str = "```";
const TABLE: char = '|';
const COMMENT: &str = "<!--";

/// 参照 id の接頭辞（条 id は枝番 `.<数>` を取れる・rules 行 id と要件 id は取れない）。
const ARTICLE: [&str; 3] = ["P-", "A-", "N-"];
const RULE: [&str; 2] = ["R-", "D-"];
const REQUIREMENT: [&str; 5] = ["GOAL", "NFR", "FR", "AC", "CON"];

/// rules 行 R-16 の value から読んだ一覧（検査 1 回ぶん）。
pub struct Gate {
    /// 規範の印（部分一致・英語は否定形も含む）
    marks: Vec<String>,
    /// 「禁止」（印になる語）
    word: String,
    /// 「禁止」の直後の文字（None = 文の末尾）
    clause_ends: Vec<Option<String>>,
    /// 数字列の直後に来れば違反になる単位
    units: Vec<String>,
}

/// 印を持つ文 1 つ。
pub struct Marked {
    /// body の中の 1 始まりの行番号
    pub line: usize,
    /// 文の先頭 60 字
    pub head: String,
    /// 文の中の参照 id（左から重ならず取る・形だけ）
    pub pointers: Vec<String>,
    /// 違反の理由（None = 門を通る）
    pub reason: Option<&'static str>,
}

// ── rules 行 R-16 の読み手 ──

/// rules の thresholds / discipline から R-16 の行を引き、value（型付きデータ）を読む。
/// 行が無い・欄が無い・型が違う は Err（呼び手は「まだ分からない」にする＝合格にしない）。
pub fn gate(rules: &Node) -> Result<Gate, String> {
    let row = RULE_SECTIONS
        .iter()
        .filter_map(|s| rules.get(s))
        .filter_map(Node::as_seq)
        .flatten()
        .find(|r| r.get("id").and_then(Node::as_str) == Some(ROW_ID))
        .ok_or_else(|| format!("{ROW_ID} の行が無い"))?;
    let value = row
        .get("value")
        .ok_or_else(|| "value の欄が無い".to_string())?;
    let marks = strings(value, "marks")?;
    let units = strings(value, "units")?;
    let prohibition = value
        .get("prohibition")
        .ok_or_else(|| "value.prohibition が無い".to_string())?;
    let word = prohibition
        .get("word")
        .and_then(Node::as_str)
        .filter(|w| !w.is_empty())
        .ok_or_else(|| "value.prohibition.word が文字列でない".to_string())?;
    let Some(ends) = prohibition.get("clause_ends").and_then(Node::as_seq) else {
        return Err("value.prohibition.clause_ends が一覧でない".to_string());
    };
    let mut clause_ends = Vec::with_capacity(ends.len());
    for end in ends {
        match end {
            Node::Null => clause_ends.push(None),
            Node::Scalar(s) if !s.is_empty() => clause_ends.push(Some(s.clone())),
            _ => {
                return Err(
                    "value.prohibition.clause_ends の要素が文字列でも null でもない".to_string(),
                );
            }
        }
    }
    if clause_ends.is_empty() {
        return Err("value.prohibition.clause_ends が空".to_string());
    }
    Ok(Gate {
        marks,
        word: word.to_string(),
        clause_ends,
        units,
    })
}

/// value の欄を空でない文字列の一覧として読む。
fn strings(value: &Node, key: &str) -> Result<Vec<String>, String> {
    let Some(items) = value.get(key).and_then(Node::as_seq) else {
        return Err(format!("value.{key} が一覧でない"));
    };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        match item.as_str().filter(|s| !s.is_empty()) {
            Some(s) => out.push(s.to_string()),
            None => return Err(format!("value.{key} に文字列でない要素が在る")),
        }
    }
    if out.is_empty() {
        return Err(format!("value.{key} が空"));
    }
    Ok(out)
}

// ── (a) 母集団と文の区切り ──

/// 散文の body を走り、印を持つ文だけを（違反の理由をつけて）返す。
/// code fence の内側・行頭が縦線の表の行・行頭が `<!--` の注釈の行は数えない。
pub fn scan(body: &str, gate: &Gate) -> Vec<Marked> {
    let mut out = Vec::new();
    let mut fenced = false;
    for (i, raw) in body.lines().enumerate() {
        let line = raw.trim_start();
        if line.starts_with(FENCE) {
            fenced = !fenced;
            continue;
        }
        if fenced || line.starts_with(TABLE) || line.starts_with(COMMENT) {
            continue;
        }
        for sentence in sentences(line) {
            if !marked(sentence, gate) {
                continue;
            }
            let spans = pointers(sentence);
            let reason = if spans.is_empty() {
                Some(NO_POINTER)
            } else if number_with_unit(sentence, &spans, gate) {
                Some(NUMBER_WITH_UNIT)
            } else {
                None
            };
            out.push(Marked {
                line: i + 1,
                head: sentence.chars().take(HEAD).collect(),
                pointers: spans.iter().map(|(a, b)| sentence[*a..*b].into()).collect(),
                reason,
            });
        }
    }
    out
}

/// 1 行を文に切る（区切りは「。」か行末・「。」は前の文に付ける）。前後の空白を落とし空の文は捨てる。
fn sentences(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, c) in line.char_indices() {
        if c == '。' {
            let end = i + c.len_utf8();
            out.push(&line[start..end]);
            start = end;
        }
    }
    out.push(&line[start..]);
    out.into_iter()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}

// ── (b) 印の判定 ──

/// 印（部分一致）を含むか、「禁止」の直後の文字が clause_ends のどれか（None = 文の末尾）か。
fn marked(sentence: &str, gate: &Gate) -> bool {
    if gate.marks.iter().any(|m| sentence.contains(m.as_str())) {
        return true;
    }
    let mut from = 0;
    while let Some(i) = sentence[from..].find(gate.word.as_str()) {
        let after = from + i + gate.word.len();
        let rest = &sentence[after..];
        let hit = gate.clause_ends.iter().any(|end| match end {
            None => rest.is_empty(),
            Some(t) => rest.starts_with(t.as_str()),
        });
        if hit {
            return true;
        }
        from = after;
    }
    false
}

// ── (c) 参照 id（形だけ） ──

/// 文の中の参照 id の byte 範囲を左から重ならず取る。
fn pointers(sentence: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < sentence.len() {
        if !sentence.is_char_boundary(i) {
            i += 1;
            continue;
        }
        // 英字で始まる形は、直前が ASCII の英数字でない位置でだけ読む（識別子の途中を拾わない）
        let inner = sentence[..i]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_ascii_alphanumeric());
        if !inner && let Some(end) = pointer_at(sentence, i) {
            out.push((i, end));
            i = end;
            continue;
        }
        i += 1;
    }
    out
}

/// 位置 i から始まる参照 id の終端（byte 位置）。
fn pointer_at(sentence: &str, i: usize) -> Option<usize> {
    let rest = &sentence[i..];
    for prefix in ARTICLE {
        if let Some(tail) = rest.strip_prefix(prefix) {
            let n = digits(tail);
            if n == 0 {
                continue;
            }
            let mut end = i + prefix.len() + n;
            // 枝番 `.<数>`（任意）
            if let Some(branch) = sentence[end..].strip_prefix('.') {
                let m = digits(branch);
                if m > 0 {
                    end += 1 + m;
                }
            }
            return Some(end);
        }
    }
    for prefix in RULE.iter().chain(REQUIREMENT.iter()) {
        if let Some(tail) = rest.strip_prefix(prefix) {
            let n = digits(tail);
            if n > 0 {
                return Some(i + prefix.len() + n);
            }
        }
    }
    contract_at(sentence, i)
}

/// 契約 id `<doc id>#<row id>`（どちらも欄の決まりの `^[a-z][a-z0-9-]*$`）。
fn contract_at(sentence: &str, i: usize) -> Option<usize> {
    let rest = &sentence[i..];
    let doc = lower_id(rest)?;
    let row = lower_id(rest[doc..].strip_prefix('#')?)?;
    Some(i + doc + 1 + row)
}

/// 先頭の数字列の長さ（無ければ 0）。
fn digits(s: &str) -> usize {
    s.bytes().take_while(u8::is_ascii_digit).count()
}

/// 先頭の `[a-z][a-z0-9-]*` の長さ（英小文字で始まらなければ None）。
fn lower_id(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    if !b.first().is_some_and(u8::is_ascii_lowercase) {
        return None;
    }
    let tail = b[1..]
        .iter()
        .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || **c == b'-')
        .count();
    Some(1 + tail)
}

// ── (d) 数と単位 ──

/// 参照 id の byte 範囲の外に「数字列 + 空白 1 つまで + 単位」が在るか。
/// 英字の単位（byte・KB・MB・s・ms）は直後が ASCII の英字でないときだけ単位と読む。
fn number_with_unit(sentence: &str, spans: &[(usize, usize)], gate: &Gate) -> bool {
    let inside = |p: usize| spans.iter().any(|(a, b)| p >= *a && p < *b);
    let b = sentence.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if !b[i].is_ascii_digit() || inside(i) || (i > 0 && b[i - 1].is_ascii_digit()) {
            i += 1;
            continue;
        }
        let mut end = i;
        while end < b.len() && b[end].is_ascii_digit() {
            end += 1;
        }
        let mut at = end;
        if let Some(c) = sentence[at..].chars().next()
            && c.is_whitespace()
        {
            at += c.len_utf8();
        }
        for unit in &gate.units {
            if !sentence[at..].starts_with(unit.as_str()) {
                continue;
            }
            let ascii = unit.bytes().all(|c| c.is_ascii_alphabetic());
            let next_alpha = sentence[at + unit.len()..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic());
            if !ascii || !next_alpha {
                return true;
            }
        }
        i = end;
    }
    false
}
