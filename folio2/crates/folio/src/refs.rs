//! 参照 id の解決（rules 行 R-4・NFR3 / AC6）と rules 行の逆参照・憲法の件数（便 1・docs/design/delivery-1.md §1）。
//! day-1 の床（scripts/check_draft.py の refs / counts）と同じ式。正規表現は使わず文字の走査で判定する。
//! 判断の記録の id（ADR-n）は解かない（便 6 の link.rs）。凍結 anchor の列に在った過去の id は解決先に足す（便 7 (j)）。

use std::collections::{HashMap, HashSet};

use crate::verdict::Report;
use crate::yaml::Node;

/// 要件書の id を持つ節（解決先）。
const SRS_ID_SECTIONS: [&str; 7] = [
    "goals",
    "requirements",
    "nonfunctional",
    "acceptance",
    "constraints",
    "actors",
    "outputs",
];

/// rules 行の節。
const RULE_SECTIONS: [&str; 2] = ["thresholds", "discipline"];

/// 憲法の relations の名前空間（閉じた一覧）。
const RELATION_NAMESPACES: [&str; 4] = ["reqs", "rules", "articles", "sections"];

/// 要件書の id の頭（数字が直に続く）。
const SRS_ID_PREFIXES: [&str; 5] = ["FR", "NFR", "AC", "CON", "GOAL"];

/// (a)(b)(c) を掛ける。
pub fn check_refs(
    constitution: &Node,
    rules: &Node,
    vocabulary: &Node,
    srs: &Node,
    history: &HashSet<String>,
    report: &mut Report,
) {
    let articles = seq_of_maps("constitution.yaml", constitution, "articles", report);
    let rule_rows: Vec<&Node> = RULE_SECTIONS
        .iter()
        .flat_map(|s| seq_of_maps("rules.yaml", rules, s, report))
        .collect();

    let mut known = known_ids(&articles, &rule_rows, srs, report);
    known.extend(history.iter().cloned());
    resolve(
        "constitution.yaml",
        &population_constitution(constitution),
        &known,
        report,
    );
    resolve(
        "rules.yaml",
        &population_without_schema(rules),
        &known,
        report,
    );
    resolve(
        "vocabulary.yaml",
        &population_all(vocabulary),
        &known,
        report,
    );
    resolve("srs.yaml", &population_all(srs), &known, report);

    reverse(&articles, &rule_rows, report);
    counts(constitution, &articles, report);
}

/// 節の行の一覧。無い・null は 0 行、一覧でない・行が表でないは「まだ分からない」。
fn seq_of_maps<'a>(
    file: &str,
    root: &'a Node,
    section: &str,
    report: &mut Report,
) -> Vec<&'a Node> {
    match root.get(section) {
        None | Some(Node::Null) => Vec::new(),
        Some(Node::Seq(items)) if items.iter().all(|n| n.as_map().is_some()) => {
            items.iter().collect()
        }
        Some(_) => {
            report.unknown(format!(
                "{file}: {section} が表の一覧でない（参照 id を集められない）"
            ));
            Vec::new()
        }
    }
}

fn id_of(row: &Node) -> Option<&str> {
    row.get("id").and_then(Node::as_str)
}

/// 解決先 = 要件書の id・条 id と規範文 id・rules 行 id（便 23 の設計ノートの id 空間も同じ式を crate の中から呼ぶ）。
pub(crate) fn known_ids(
    articles: &[&Node],
    rule_rows: &[&Node],
    srs: &Node,
    report: &mut Report,
) -> HashSet<String> {
    let mut known = HashSet::new();
    for section in SRS_ID_SECTIONS {
        for row in seq_of_maps("srs.yaml", srs, section, report) {
            match id_of(row) {
                Some(id) => {
                    known.insert(id.to_string());
                }
                None => report.unknown(format!("srs.yaml: {section} に id の無い行がある")),
            }
        }
    }
    for article in articles {
        known.extend(id_of(article).map(str::to_string));
        if let Some(Node::Seq(statements)) = article.get("statements") {
            known.extend(statements.iter().filter_map(id_of).map(str::to_string));
        }
    }
    known.extend(
        rule_rows
            .iter()
            .filter_map(|r| id_of(r))
            .map(str::to_string),
    );
    known
}

/// 母集団（欄の道・値）。
type Population<'a> = Vec<(String, &'a Node)>;

/// 憲法: schema 節を除き、meta の changes_from で始まる欄を除く。
fn population_constitution(root: &Node) -> Population<'_> {
    let mut out = Vec::new();
    for (key, value) in root.as_map().unwrap_or_default() {
        match key.as_str() {
            "schema" => {}
            "meta" => match value.as_map() {
                Some(entries) => out.extend(
                    entries
                        .iter()
                        .filter(|(k, _)| !k.starts_with("changes_from"))
                        .map(|(k, v)| (format!("meta.{k}"), v)),
                ),
                None => out.push((key.clone(), value)),
            },
            _ => out.push((key.clone(), value)),
        }
    }
    out
}

fn population_without_schema(root: &Node) -> Population<'_> {
    root.as_map()
        .unwrap_or_default()
        .iter()
        .filter(|(k, _)| k != "schema")
        .map(|(k, v)| (k.clone(), v))
        .collect()
}

fn population_all(root: &Node) -> Population<'_> {
    root.as_map()
        .unwrap_or_default()
        .iter()
        .map(|(k, v)| (k.clone(), v))
        .collect()
}

/// 全欄の文字列を歩き、解決できない参照 id を違反に数える（表のキーは数えない）。
fn resolve(file: &str, population: &Population<'_>, known: &HashSet<String>, report: &mut Report) {
    fn walk(file: &str, node: &Node, at: &str, known: &HashSet<String>, report: &mut Report) {
        match node {
            Node::Null => {}
            Node::Scalar(text) => {
                for id in scan_ids(text) {
                    if !known.contains(&id) {
                        report.violation("参照 id", format!("{file}: {at}: id {id} が実在しない"));
                    }
                }
            }
            Node::Seq(items) => {
                for (i, item) in items.iter().enumerate() {
                    walk(file, item, &format!("{at}[{i}]"), known, report);
                }
            }
            Node::Map(entries) => {
                for (k, v) in entries {
                    walk(file, v, &format!("{at}.{k}"), known, report);
                }
            }
        }
    }
    for (at, node) in population {
        walk(file, node, at, known, report);
    }
}

/// 本文の中の参照 id を拾う。床の式
/// `(?<![A-Za-z0-9-])((?:P|A|N)-\d+(?:\.\d+)?|(?:FR|NFR|AC|CON|GOAL)\d+|(?:R|D)-\d+)(?![A-Za-z0-9])`
/// を文字の走査で写す（findall と同じく、拾った id の後ろから続ける）。
pub fn scan_ids(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        match id_end(&chars, i) {
            Some(end) => {
                out.push(chars[i..end].iter().collect());
                i = end;
            }
            None => i += 1,
        }
    }
    out
}

fn digits_from(chars: &[char], start: usize) -> usize {
    chars
        .get(start..)
        .unwrap_or_default()
        .iter()
        .take_while(|c| c.is_ascii_digit())
        .count()
}

fn starts_with_at(chars: &[char], start: usize, prefix: &str) -> bool {
    let p: Vec<char> = prefix.chars().collect();
    chars.get(start..start + p.len()) == Some(&p[..])
}

/// `start` から参照 id が始まるなら、その終わりの位置。
pub(crate) fn id_end(chars: &[char], start: usize) -> Option<usize> {
    if start > 0 {
        let prev = chars[start - 1];
        if prev.is_ascii_alphanumeric() || prev == '-' {
            return None;
        }
    }
    let free = |end: usize| chars.get(end).is_none_or(|c| !c.is_ascii_alphanumeric());
    let head = chars[start];
    // 条 id（枝番「.数字」付きも）。枝番まで含めて後ろが空かなければ、枝番を外した形を試す（正規表現の後戻りと同じ）。
    if matches!(head, 'P' | 'A' | 'N') && chars.get(start + 1) == Some(&'-') {
        let n = digits_from(chars, start + 2);
        if n > 0 {
            let end = start + 2 + n;
            if chars.get(end) == Some(&'.') {
                let m = digits_from(chars, end + 1);
                if m > 0 && free(end + 1 + m) {
                    return Some(end + 1 + m);
                }
            }
            if free(end) {
                return Some(end);
            }
        }
    }
    // 要件書の id
    for prefix in SRS_ID_PREFIXES {
        if starts_with_at(chars, start, prefix) {
            let after = start + prefix.len();
            let n = digits_from(chars, after);
            if n > 0 && free(after + n) {
                return Some(after + n);
            }
        }
    }
    // rules 行 id
    if matches!(head, 'R' | 'D') && chars.get(start + 1) == Some(&'-') {
        let n = digits_from(chars, start + 2);
        if n > 0 && free(start + 2 + n) {
            return Some(start + 2 + n);
        }
    }
    None
}

/// 規範文の本文に現れる rules 行 id。床の式 `\b([RD]-\d+)\b`（語の文字 = Unicode の英数字と「_」）を写す。
fn rule_ids_in_text(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let is_word = |c: &char| c.is_alphanumeric() || *c == '_';
    let mut out = Vec::new();
    for start in 0..chars.len() {
        if !matches!(chars[start], 'R' | 'D') || chars.get(start + 1) != Some(&'-') {
            continue;
        }
        if start > 0 && is_word(&chars[start - 1]) {
            continue;
        }
        let n = digits_from(&chars, start + 2);
        let end = start + 2 + n;
        if n > 0 && !chars.get(end).is_some_and(is_word) {
            out.push(chars[start..end].iter().collect());
        }
    }
    out
}

/// (b) rules 行 ⇔ 条 の双方向と relations の名前空間。
fn reverse(articles: &[&Node], rule_rows: &[&Node], report: &mut Report) {
    const FILE: &str = "constitution.yaml";
    let mut referenced: HashSet<String> = HashSet::new();
    let mut article_rules: HashMap<String, HashSet<String>> = HashMap::new();
    for article in articles {
        let aid = id_of(article).unwrap_or("?").to_string();
        let mut rules_here = HashSet::new();
        match article.get("relations") {
            None => {}
            Some(rel) if rel.is_blank() => {}
            Some(Node::Map(entries)) => {
                for (namespace, _) in entries {
                    if !RELATION_NAMESPACES.contains(&namespace.as_str()) {
                        report.violation(
                            "逆参照",
                            format!("{FILE}: {aid}: relations の未知の名前空間 {namespace}"),
                        );
                    }
                }
                match article.get("relations").and_then(|r| r.get("rules")) {
                    None => {}
                    Some(Node::Seq(items)) => {
                        for item in items {
                            match item.as_str() {
                                Some(rid) => {
                                    rules_here.insert(rid.to_string());
                                }
                                None => report.unknown(format!(
                                    "{FILE}: {aid}: relations.rules に文字列でない値がある"
                                )),
                            }
                        }
                    }
                    Some(_) => {
                        report.unknown(format!("{FILE}: {aid}: relations.rules が一覧でない"))
                    }
                }
            }
            Some(_) => report.violation(
                "逆参照",
                format!("{FILE}: {aid}: relations が型付き map でない"),
            ),
        }
        if let Some(Node::Seq(statements)) = article.get("statements") {
            for st in statements {
                match st.get("text").and_then(Node::as_str) {
                    Some(text) => referenced.extend(rule_ids_in_text(text)),
                    None => report.unknown(format!(
                        "{FILE}: {aid}: 規範文 {} の本文が文字列でない",
                        id_of(st).unwrap_or("?")
                    )),
                }
            }
        }
        referenced.extend(rules_here.iter().cloned());
        article_rules.entry(aid).or_default().extend(rules_here);
    }
    for row in rule_rows {
        let Some(rid) = id_of(row) else { continue };
        if !referenced.contains(rid) {
            report.violation(
                "逆参照",
                format!("rules.yaml: 行 {rid} はどの条からも参照されていない（双方向）"),
            );
        }
        let article = row.get("article").and_then(Node::as_str).unwrap_or("?");
        if !article_rules.get(article).is_some_and(|s| s.contains(rid)) {
            report.violation(
                "逆参照",
                format!("rules.yaml: 行 {rid}: article={article} だが {article} の relations.rules に無い"),
            );
        }
    }
}

/// (c) meta.counts と条の tier の実数。
fn counts(constitution: &Node, articles: &[&Node], report: &mut Report) {
    const FILE: &str = "constitution.yaml";
    let Some(declared) = constitution
        .get("meta")
        .and_then(|m| m.get("counts"))
        .and_then(Node::as_map)
    else {
        report.unknown(format!("{FILE}: meta.counts（条の件数）が読めない"));
        return;
    };
    let mut actual: HashMap<&str, usize> = HashMap::new();
    for article in articles {
        match article.get("tier").and_then(Node::as_str) {
            Some(tier) => *actual.entry(tier).or_default() += 1,
            None => {
                report.unknown(format!(
                    "{FILE}: 条 {} の tier が読めない（件数を数えられない）",
                    id_of(article).unwrap_or("?")
                ));
                return;
            }
        }
    }
    for (tier, value) in declared {
        let real = actual.get(tier.as_str()).copied().unwrap_or(0);
        let written = value.as_str().unwrap_or("?");
        if written.parse::<usize>().ok() != Some(real) {
            report.violation(
                "件数",
                format!("{FILE}: meta.counts.{tier}={written} だが実数 {real}"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_follow_the_floor_pattern() {
        assert_eq!(
            scan_ids("P-1・P-17.4 と FR1/NFR3、R-4 と D-10（ADR-3・xP-1・P-1a・-R-2・NFR1b）"),
            ["P-1", "P-17.4", "FR1", "NFR3", "R-4", "D-10"]
        );
        // 枝番の後ろが空かなければ枝番を外す（後戻り）・後ろの「-」は止めない
        assert_eq!(scan_ids("P-1.2a P-3-4 GOAL12"), ["P-1", "P-3", "GOAL12"]);
        assert!(scan_ids("§6 図2-1 v1.0 2026-09-12").is_empty());
    }

    #[test]
    fn rule_ids_need_word_boundaries() {
        assert_eq!(rule_ids_in_text("行 R-4 と (D-1)。"), ["R-4", "D-1"]);
        assert!(rule_ids_in_text("R-4の床・はR-5").is_empty());
    }
}
