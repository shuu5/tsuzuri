//! `folio check` — design-intent の正本 4 file（憲法・rules・語彙・要件書）の形の床（FR5 / FR9）。
//! 数えるのは 重複キー・未知の節・欄の非空（便 0）と、参照 id の解決・rules 行の逆参照・憲法の件数（便 1・refs）と、語彙の検査 R-9（便 4・vocab）と、
//! 判断の記録（adr/）の欄の決まり（便 5・adr）と、判断の記録と正本 4 file・凍結 anchor の列の突き合わせ（便 6・link）。
//! 凍結 anchor の列そのものは便 7。
//! 読めない・型が違う・節の決まりが読めない は「まだ分からない」（合格にしない）。

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::adr;
use crate::link;
use crate::refs;
use crate::verdict::Report;
use crate::vocab;
use crate::yaml::{self, Node};

/// 正本 4 file（読む順）。
pub const FILES: [&str; 4] = ["constitution", "rules", "vocabulary", "srs"];

/// 要件書の節の閉じた一覧（要件書は schema 節を持たないので床の定数で持つ）。
pub const SRS_TOP_LEVEL: [&str; 14] = [
    "meta",
    "goals",
    "scope",
    "scope_m1",
    "actors",
    "outputs",
    "rail",
    "requirements",
    "nonfunctional",
    "acceptance",
    "not_frozen",
    "constraints",
    "sources",
    "glossary_pointer",
];

/// 語彙の節の閉じた一覧（同上）。
pub const VOCABULARY_TOP_LEVEL: [&str; 3] = ["terms", "field_terms", "identifiers"];

struct Sources {
    constitution: Node,
    rules: Node,
    vocabulary: Node,
    srs: Node,
}

/// `dir` の正本 4 file を検査する。
pub fn check_dir(dir: &Path) -> Report {
    let mut report = Report::default();
    match load_all(dir, &mut report) {
        Some(src) => {
            check_constitution(&src.constitution, &mut report);
            check_rules(&src.rules, &mut report);
            check_vocabulary(&src.vocabulary, &mut report);
            check_srs(&src.srs, &mut report);
            refs::check_refs(
                &src.constitution,
                &src.rules,
                &src.vocabulary,
                &src.srs,
                &mut report,
            );
            vocab::check_vocab(
                &src.constitution,
                &src.rules,
                &src.vocabulary,
                &src.srs,
                &mut report,
            );
            if let Some(records) = adr::check_adr(dir, &mut report) {
                link::check_link(
                    dir,
                    &src.constitution,
                    &src.rules,
                    &src.vocabulary,
                    &src.srs,
                    &records,
                    &mut report,
                );
            }
        }
        None => debug_assert!(!report.unknowns.is_empty()),
    }
    report
}

fn load_all(dir: &Path, report: &mut Report) -> Option<Sources> {
    if dir.is_symlink() {
        report.unknown(format!(
            "design-intent 自体が symlink（{}）＝認めない",
            dir.display()
        ));
        return None;
    }
    if !dir.is_dir() {
        report.unknown(format!("design-intent が dir でない: {}", dir.display()));
        return None;
    }
    let mut roots: Vec<Option<Node>> = FILES.iter().map(|name| load(dir, name, report)).collect();
    let srs = roots.pop()??;
    let vocabulary = roots.pop()??;
    let rules = roots.pop()??;
    let constitution = roots.pop()??;
    Some(Sources {
        constitution,
        rules,
        vocabulary,
        srs,
    })
}

fn load(dir: &Path, name: &str, report: &mut Report) -> Option<Node> {
    let file = format!("{name}.yaml");
    let path = dir.join(&file);
    if path.is_symlink() {
        report.unknown(format!("{file}: symlink は認めない"));
        return None;
    }
    if !path.exists() {
        report.unknown(format!("{file}: 正本が無い"));
        return None;
    }
    if !path.is_file() {
        report.unknown(format!("{file}: file でない"));
        return None;
    }
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            report.unknown(format!("{file}: 読めない: {e}"));
            return None;
        }
    };
    let doc = match yaml::parse(&text) {
        Ok(d) => d,
        Err(e) => {
            report.unknown(format!("{file}: parse できない: {e}"));
            return None;
        }
    };
    for dup in &doc.duplicates {
        report.violation(
            "重複キー",
            format!(
                "{file} {} 行: 同じ表にキー「{}」を 2 度書いている",
                dup.line, dup.key
            ),
        );
    }
    if doc.root.as_map().is_none() {
        report.unknown(format!("{file}: 最上位が欄の表でない"));
        return None;
    }
    Some(doc.root)
}

/// 最上位の節が閉じた一覧に在るか。
fn unknown_sections(file: &str, root: &Node, allowed: &[&str], report: &mut Report) {
    for (key, _) in root.as_map().unwrap_or_default() {
        if !allowed.contains(&key.as_str()) {
            report.violation("未知の節", format!("{file}: 未知の節「{key}」"));
        }
    }
}

/// 節の決まり（`schema.top_level`）を正本から読む。読めなければ「まだ分からない」。
fn schema_top_level<'a>(file: &str, root: &'a Node, report: &mut Report) -> Option<Vec<&'a str>> {
    let list = root
        .get("schema")
        .and_then(|s| s.get("top_level"))
        .and_then(Node::as_seq);
    let names: Option<Vec<&str>> = list.and_then(|l| l.iter().map(Node::as_str).collect());
    if names.is_none() {
        report.unknown(format!(
            "{file}: schema.top_level（節の閉じた一覧）が読めない"
        ));
    }
    names
}

/// 行の一覧の節を読む。無い・null は 0 行、一覧でない・行が表でないは「まだ分からない」。
fn rows<'a>(file: &str, root: &'a Node, section: &str, report: &mut Report) -> Vec<&'a Node> {
    match root.get(section) {
        None | Some(Node::Null) => Vec::new(),
        Some(Node::Seq(items)) if items.iter().all(|n| n.as_map().is_some()) => {
            items.iter().collect()
        }
        Some(_) => {
            report.unknown(format!("{file}: {section} が表の一覧でない"));
            Vec::new()
        }
    }
}

/// 行の欄が空でないか。
fn non_empty(file: &str, where_: &str, row: &Node, fields: &[&str], report: &mut Report) {
    for field in fields {
        if row.get(field).is_none_or(Node::is_blank) {
            report.violation("欄の非空", format!("{file}: {where_} の {field} が空"));
        }
    }
}

fn row_id(row: &Node) -> String {
    row.get("id")
        .and_then(Node::as_str)
        .unwrap_or("?")
        .to_string()
}

/// 行 id の重複（行の表のキー）を数える。
fn duplicate_ids<'a>(file: &str, rows: impl IntoIterator<Item = &'a Node>, report: &mut Report) {
    let mut seen = HashSet::new();
    for row in rows {
        let id = row_id(row);
        if id != "?" && !seen.insert(id.clone()) {
            report.violation("重複キー", format!("{file}: 行 id「{id}」が重複"));
        }
    }
}

fn check_constitution(root: &Node, report: &mut Report) {
    const FILE: &str = "constitution.yaml";
    if let Some(top) = schema_top_level(FILE, root, report) {
        unknown_sections(FILE, root, &top, report);
    }
    non_empty(
        FILE,
        "前文（precedence）",
        root.get("precedence").unwrap_or(&Node::Null),
        &["text", "plain"],
        report,
    );
    let articles = rows(FILE, root, "articles", report);
    for article in &articles {
        let id = row_id(article);
        non_empty(
            FILE,
            &format!("条 {id}"),
            article,
            &["id", "title", "plain", "statements"],
            report,
        );
        let statements = rows(FILE, article, "statements", report);
        for st in &statements {
            non_empty(
                FILE,
                &format!("条 {id} の規範文 {}", row_id(st)),
                st,
                &["id", "text"],
                report,
            );
        }
        duplicate_ids(FILE, statements, report);
    }
    duplicate_ids(FILE, articles, report);
}

fn check_rules(root: &Node, report: &mut Report) {
    const FILE: &str = "rules.yaml";
    if let Some(top) = schema_top_level(FILE, root, report) {
        unknown_sections(FILE, root, &top, report);
    }
    let mut all = Vec::new();
    for section in ["thresholds", "discipline"] {
        for row in rows(FILE, root, section, report) {
            non_empty(
                FILE,
                &format!("行 {}", row_id(row)),
                row,
                &["id", "article", "what"],
                report,
            );
            all.push(row);
        }
    }
    duplicate_ids(FILE, all, report);
}

fn check_vocabulary(root: &Node, report: &mut Report) {
    const FILE: &str = "vocabulary.yaml";
    unknown_sections(FILE, root, &VOCABULARY_TOP_LEVEL, report);
    for (section, fields) in [
        ("terms", &["id", "term", "short", "def"][..]),
        ("field_terms", &["id", "term", "def"][..]),
    ] {
        let items = rows(FILE, root, section, report);
        for item in &items {
            non_empty(
                FILE,
                &format!("{section} の語 {}", row_id(item)),
                item,
                fields,
                report,
            );
        }
        duplicate_ids(FILE, items, report);
    }
    for group in rows(FILE, root, "identifiers", report) {
        let name = group.get("group").and_then(Node::as_str).unwrap_or("?");
        non_empty(
            FILE,
            &format!("identifiers の組 {name}"),
            group,
            &["group", "words"],
            report,
        );
    }
}

fn check_srs(root: &Node, report: &mut Report) {
    const FILE: &str = "srs.yaml";
    unknown_sections(FILE, root, &SRS_TOP_LEVEL, report);
    let mut all = Vec::new();
    for section in [
        "goals",
        "requirements",
        "nonfunctional",
        "acceptance",
        "constraints",
    ] {
        for row in rows(FILE, root, section, report) {
            let mut fields = vec!["id", "title"];
            if matches!(section, "requirements" | "nonfunctional") {
                fields.extend(["shall", "plain"]);
            }
            non_empty(
                FILE,
                &format!("{section} の {}", row_id(row)),
                row,
                &fields,
                report,
            );
            all.push(row);
        }
    }
    duplicate_ids(FILE, all, report);
}
