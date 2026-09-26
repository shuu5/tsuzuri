//! 語彙の検査（rules 行 R-9・便 4・docs/design/delivery-4.md §1）。
//! day-1 の床（scripts/check_draft.py の vocab）と同じ式。正規表現は使わず文字の走査で判定する。
//! 判断の記録（adr/）の本文と凍結 anchor は R-9 の母集団に入れない。判断の記録の本文は便 6 の `link.rs` が
//! 同じ切り出し・既知の集合・免除（`known_words`・`unknown_words`）で数える（種別は adr）。

use std::collections::{BTreeSet, HashSet};

use crate::verdict::Report;
use crate::yaml::Node;

/// id の形の頭（直後に「-」が 0 個か 1 個・その次が数字）。
const ID_PREFIXES: [&str; 10] = ["P", "A", "N", "FR", "NFR", "AC", "CON", "GOAL", "R", "D"];

/// 本文の対（場所, 文字列）。
pub(crate) type Body = Vec<(String, String)>;

/// (a)〜(e) を掛ける。
pub fn check_vocab(
    constitution: &Node,
    rules: &Node,
    vocabulary: &Node,
    srs: &Node,
    report: &mut Report,
) {
    let known = known_words(vocabulary, report);
    let body = population(constitution, rules, vocabulary, srs, report);
    // 名札は置き場の規則の表に行 R-9 が在れば行 id、無ければ検査の名（便 156）
    let label = crate::rules::label(rules, "R-9");
    for (lw, at) in unknown_words(&body, &known) {
        report.violation(label, format!("{at}: 語彙に無い英字の語「{lw}」"));
    }
}

/// (d) 免除されない語を (小文字の語, 場所) で重ねずに並べる。
pub(crate) fn unknown_words(body: &Body, known: &HashSet<String>) -> BTreeSet<(String, String)> {
    let mut unknown: BTreeSet<(String, String)> = BTreeSet::new();
    for (at, text) in body {
        let glossed: HashSet<String> = glosses(text)
            .iter()
            .flat_map(|g| words(g))
            .map(|w| w.to_lowercase())
            .collect();
        for w in words(text) {
            let lw = w.to_lowercase();
            let exempt = is_id_shape(&w)
                || known.contains(&lw)
                || glossed.contains(&lw)
                || lw.chars().count() == 1
                || text.contains(&format!("--{w}"));
            if !exempt {
                unknown.insert((lw, at.clone()));
            }
        }
    }
    unknown
}

fn id_of(row: &Node) -> &str {
    row.get("id").and_then(Node::as_str).unwrap_or("?")
}

/// 節の行（表）。一覧でない節は check.rs の側で「まだ分からない」に数える。
fn rows<'a>(root: &'a Node, section: &str) -> impl Iterator<Item = &'a Node> {
    root.get(section)
        .and_then(Node::as_seq)
        .unwrap_or_default()
        .iter()
        .filter(|n| n.as_map().is_some())
}

/// 欄の文字列。無い・文字列でない は空（数えない）。
fn text<'a>(row: &'a Node, field: &str) -> &'a str {
    row.get(field).and_then(Node::as_str).unwrap_or("")
}

/// (c) 既知の集合。
pub(crate) fn known_words(vocabulary: &Node, report: &mut Report) -> HashSet<String> {
    const FILE: &str = "vocabulary.yaml";
    let mut known = HashSet::new();
    for section in ["terms", "field_terms"] {
        for t in rows(vocabulary, section) {
            for field in ["term", "en"] {
                if matches!(t.get(field), Some(Node::Seq(_) | Node::Map(_))) {
                    report.unknown(format!(
                        "{FILE}: {section} の語 {} の {field} が文字列でない（既知の語を集められない）",
                        id_of(t)
                    ));
                }
            }
            let joined = format!("{} {}", text(t, "term"), text(t, "en"));
            known.extend(words(&joined).iter().map(|w| w.to_lowercase()));
        }
    }
    for group in rows(vocabulary, "identifiers") {
        match group.get("words") {
            None | Some(Node::Null) => {}
            Some(Node::Seq(items)) => {
                known.extend(items.iter().filter_map(Node::as_str).map(str::to_lowercase))
            }
            Some(_) => report.unknown(format!(
                "{FILE}: identifiers の組 {} の words が一覧でない",
                group.get("group").and_then(Node::as_str).unwrap_or("?")
            )),
        }
    }
    known
}

/// (a) 母集団。
fn population(
    constitution: &Node,
    rules: &Node,
    vocabulary: &Node,
    srs: &Node,
    report: &mut Report,
) -> Body {
    let mut body: Body = Vec::new();
    let mut push = |at: String, s: &str| body.push((at, s.to_string()));
    for article in rows(constitution, "articles") {
        let aid = id_of(article);
        push(format!("{aid} title"), text(article, "title"));
        push(format!("{aid} plain"), text(article, "plain"));
        for st in rows(article, "statements") {
            push(id_of(st).to_string(), text(st, "text"));
        }
    }
    match constitution.get("precedence") {
        None | Some(Node::Null) => {}
        Some(pr @ Node::Map(_)) => {
            push("前文".to_string(), text(pr, "text"));
            push("前文 plain".to_string(), text(pr, "plain"));
        }
        Some(_) => report.unknown(
            "constitution.yaml: precedence が表でない（語彙の検査の母集団を集められない）",
        ),
    }
    for section in ["thresholds", "discipline"] {
        for row in rows(rules, section) {
            push(format!("{} what", id_of(row)), text(row, "what"));
        }
    }
    for (section, fields) in [
        ("requirements", &["title", "when", "shall", "plain"][..]),
        ("nonfunctional", &["title", "when", "shall", "plain"][..]),
        ("acceptance", &["title", "plain"][..]),
        ("constraints", &["title", "text"][..]),
    ] {
        for row in rows(srs, section) {
            for field in fields {
                push(format!("要件書 {} {field}", id_of(row)), text(row, field));
            }
        }
    }
    for row in rows(srs, "goals") {
        push(
            format!("要件書 {}", id_of(row)),
            &format!("{} {}", text(row, "title"), text(row, "text")),
        );
    }
    for t in rows(vocabulary, "terms") {
        push(
            format!("語彙 {} def", id_of(t)),
            &format!("{} {}", text(t, "short"), text(t, "def")),
        );
    }
    body
}

/// (b) 英字の語。ASCII の英字で始まり、英字・数字・「-」・「.」が続く限り伸ばし、末尾の「-」「.」を落とす。
pub fn words(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_alphabetic() {
            i += 1;
            continue;
        }
        let mut end = i + 1;
        while end < chars.len()
            && (chars[end].is_ascii_alphanumeric() || matches!(chars[end], '-' | '.'))
        {
            end += 1;
        }
        let mut last = end;
        while matches!(chars[last - 1], '-' | '.') {
            last -= 1;
        }
        out.push(chars[i..last].iter().collect());
        i = end;
    }
    out
}

/// (d)(i) id の形（元の大小文字で見る）。
fn is_id_shape(w: &str) -> bool {
    for prefix in ID_PREFIXES {
        if let Some(rest) = w.strip_prefix(prefix) {
            let rest = rest.strip_prefix('-').unwrap_or(rest);
            if rest.starts_with(|c: char| c.is_ascii_digit()) {
                return true;
            }
        }
    }
    if let Some(n) = w.strip_prefix("ADR-")
        && n.starts_with(|c: char| matches!(c, '1'..='9'))
        && n.chars().all(|c| c.is_ascii_digit())
    {
        return true;
    }
    if matches!(w, "ADR-n" | "R-n" | "D-n") {
        return true;
    }
    is_ledger_id(w)
}

/// 台帳 id の形（例 f2-648.14）。
fn is_ledger_id(w: &str) -> bool {
    let b = w.as_bytes();
    if b.len() < 4 || !b[0].is_ascii_lowercase() || !b[1].is_ascii_digit() || b[2] != b'-' {
        return false;
    }
    let lower_or_digit = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    };
    match w[3..].split_once('.') {
        Some((head, tail)) => {
            lower_or_digit(head) && !tail.is_empty() && tail.bytes().all(|c| c.is_ascii_digit())
        }
        None => lower_or_digit(&w[3..]),
    }
}

fn is_japanese(c: char) -> bool {
    matches!(c, '\u{3040}'..='\u{30FF}' | '\u{4E00}'..='\u{9FFF}')
}

/// (d)(iii) 「日本語（原語）」の形の括弧の中身。日本語の文字から「（」「）」を跨がずに続く「（」から次の「）」まで。
fn glosses(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let bracket = |from: usize| (from..chars.len()).find(|&j| matches!(chars[j], '（' | '）'));
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if is_japanese(chars[i])
            && let Some(open) = bracket(i + 1).filter(|&j| chars[j] == '（')
            && let Some(close) = bracket(open + 1).filter(|&k| chars[k] == '）')
        {
            out.push(chars[open + 1..close].iter().collect());
            i = close + 1;
            continue;
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_follow_the_floor_pattern() {
        assert_eq!(
            words("folio-2. の x・3abc v1.0 design-intent- と CLAUDE.md。ｆｕｌｌ"),
            ["folio-2", "x", "abc", "v1.0", "design-intent", "CLAUDE.md"]
        );
    }

    #[test]
    fn id_shapes() {
        for w in [
            "FR1",
            "P-17.4",
            "NFR3b",
            "R-9",
            "ADR-3",
            "ADR-n",
            "D-n",
            "f2-648.14",
            "s2-07l",
        ] {
            assert!(is_id_shape(w), "{w}");
        }
        for w in ["ADR-0047", "fr1", "Pa", "f2-648.1a", "ADR-3a", "GOALS"] {
            assert!(!is_id_shape(w), "{w}");
        }
    }

    #[test]
    fn glosses_need_japanese_before_the_bracket() {
        assert_eq!(
            glosses("(raw)（raw）型付きの表（gloss）・語 x（y） 表（a（b）"),
            ["gloss", "y"]
        );
    }
}
