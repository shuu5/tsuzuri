//! `folio check` の判断の記録（adr/）と正本 4 file・凍結 anchor の列の突き合わせ（便 6・docs/design/delivery-6.md §1）。
//! day-1 の床 `scripts/check_draft.py` の adr・refs・vocab の節のうち便 5 が残した突き合わせを同じ式で写す:
//! 撤退条件の種類の一致・対話面の行・改訂の範囲・判断の記録の id の参照・凍結 anchor の列に在った id・
//! 判断の記録の本文の英字語・改訂来歴（amended_by ⇔ amends）の双方向。
//! 凍結 anchor の列そのもの（digest の検算・索引・版管理との照合・現行の写しとの一致）は便 7。
//! 床の定数は `adr.rs` の `FLOOR` を読み口（`adr::floor_strs`）で読み、値は持ち直さない。正規表現は使わない。

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::adr::{self, Adr};
use crate::refs;
use crate::verdict::Report;
use crate::vocab;
use crate::yaml::{self, Node};

/// 要件書の id を持つ節（便 1 の解決先と同じ）。
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

/// 凍結 anchor の file 名の頭と尻（`constitution-*.yaml`）。
const ANCHOR_PREFIX: &str = "constitution-";
const ANCHOR_SUFFIX: &str = ".yaml";

/// (a)〜(g) を掛ける。
pub fn check_link(
    dir: &Path,
    constitution: &Node,
    rules: &Node,
    vocabulary: &Node,
    srs: &Node,
    adr: &Adr,
    report: &mut Report,
) {
    let records = &adr.records;
    let article_ids: Vec<String> = rows(constitution, "articles")
        .map(|a| py_str(a.get("id")))
        .collect();
    let scope = amendment_scope(constitution);

    retreat_kind(constitution, report);
    surface(rules, report);
    amendment_range(&article_ids, &scope, report);

    let history = history(dir);
    let known = known_ids(constitution, rules, srs, &history.ids);
    references(constitution, rules, vocabulary, srs, adr, &known, report);
    amends_targets(records, &article_ids, &scope, &history, report);
    body_words(vocabulary, adr, report);
    amended_by(constitution, records, report);
}

/// 節の行（表）。一覧でない節・表でない行は数えない（check.rs / refs.rs の側で「まだ分からない」に数える）。
fn rows<'a>(root: &'a Node, section: &str) -> impl Iterator<Item = &'a Node> {
    root.get(section)
        .and_then(Node::as_seq)
        .unwrap_or_default()
        .iter()
        .filter(|n| n.as_map().is_some())
}

/// 床の `str(x)` の写し（無い・null は None）。
fn py_str(node: Option<&Node>) -> String {
    match node {
        Some(Node::Scalar(s)) => s.clone(),
        Some(Node::Null) | None => "None".to_string(),
        Some(other) => format!("{other:?}"),
    }
}

/// 床の `str(x or '')` の写し（本文の欄）。
fn text_of(node: Option<&Node>) -> &str {
    node.and_then(Node::as_str).unwrap_or("")
}

/// 憲法の schema.amendment_scope（無ければ空の一覧）。
fn amendment_scope(constitution: &Node) -> Vec<String> {
    match constitution
        .get("schema")
        .and_then(|s| s.get("amendment_scope"))
    {
        None | Some(Node::Null) => Vec::new(),
        Some(Node::Seq(items)) => items.iter().map(|x| py_str(Some(x))).collect(),
        Some(Node::Scalar(s)) => s.chars().map(String::from).collect(),
        Some(Node::Map(entries)) => entries.iter().map(|(k, _)| k.clone()).collect(),
    }
}

/// (a) 撤退条件の種類。床の定数と憲法の値域の一覧の長さ・字面・順。
fn retreat_kind(constitution: &Node, report: &mut Report) {
    let floor = adr::floor_strs(&["enums", "retreat_kind"]);
    let Some(items) = constitution
        .get("schema")
        .and_then(|s| s.get("enums"))
        .and_then(|e| e.get("retreat_kind"))
        .and_then(Node::as_seq)
    else {
        report.unknown(
            "constitution.yaml: schema.enums.retreat_kind（撤退条件の種類の値域）が読めない",
        );
        return;
    };
    let same = items.len() == floor.len()
        && items
            .iter()
            .zip(floor.iter())
            .all(|(n, f)| n.as_str() == Some(f));
    if !same {
        let written: Vec<String> = items.iter().map(|n| py_str(Some(n))).collect();
        report.violation(
            "adr",
            format!(
                "床の retreat_kind [{}] が憲法の値域 [{}] と食い違う（憲法が正・床の定数を直す）",
                floor.join(", "),
                written.join(", ")
            ),
        );
    }
}

/// (b) 対話面の各 id が rules 行に在る。
fn surface(rules: &Node, report: &mut Report) {
    let rule_ids: HashSet<String> = RULE_SECTIONS
        .iter()
        .flat_map(|s| rows(rules, s))
        .map(|r| py_str(r.get("id")))
        .collect();
    for sid in adr::floor_strs(&["enums", "surface"]) {
        if !rule_ids.contains(*sid) {
            report.violation(
                "adr",
                format!("対話面 {sid} が rules 行に無い（対話面は rules 行の id で指す・R-8）"),
            );
        }
    }
}

/// (c) 改訂の範囲の下限と、条 id と節名の衝突。
fn amendment_range(article_ids: &[String], scope: &[String], report: &mut Report) {
    let minimum = adr::floor_strs(&["anchor", "scope_minimum"]);
    if !minimum.iter().all(|m| scope.iter().any(|s| s == m)) {
        report.violation(
            "anchor",
            format!(
                "憲法 schema.amendment_scope [{}] が A-2.2 の範囲の下限 [{}] を含まない",
                scope.join(", "),
                minimum.join(", ")
            ),
        );
    }
    for id in article_ids {
        if scope.contains(id) {
            report.violation(
                "schema",
                format!("条 id {id} が改訂の範囲の節名と同じ（欄の道が衝突する）"),
            );
        }
    }
}

/// (e) 凍結 anchor の列に在った条・規範文の id と範囲の節名。
struct History {
    ids: HashSet<String>,
    scopes: HashSet<String>,
}

/// `<dir>/anchors/` の直下の `constitution-*.yaml` を名前順に読む。読めない file は飛ばす（列の真偽は便 7）。
fn history(dir: &Path) -> History {
    let mut h = History {
        ids: HashSet::new(),
        scopes: HashSet::new(),
    };
    let anchors = dir.join("anchors");
    if !anchors.is_dir() {
        return h;
    }
    let mut names: Vec<String> = match fs::read_dir(&anchors) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| {
                n.len() >= ANCHOR_PREFIX.len() + ANCHOR_SUFFIX.len()
                    && n.starts_with(ANCHOR_PREFIX)
                    && n.ends_with(ANCHOR_SUFFIX)
            })
            .collect(),
        Err(_) => return h,
    };
    names.sort();
    for name in names {
        let Ok(text) = fs::read_to_string(anchors.join(&name)) else {
            continue;
        };
        // 床の読み手は重複キーを読めないとする
        let Ok(doc) = yaml::parse(&text) else {
            continue;
        };
        if !doc.duplicates.is_empty() {
            continue;
        }
        let d = doc.root;
        if let Some(content @ Node::Map(_)) = d.get("content") {
            for a in rows(content, "articles") {
                h.ids.insert(py_str(a.get("id")));
                for st in rows(a, "statements") {
                    h.ids.insert(py_str(st.get("id")));
                }
            }
        }
        if let Some(projection @ Node::Map(_)) = d.get("projection")
            && let Some(Node::Seq(items)) = projection.get("scope")
        {
            h.scopes.extend(items.iter().map(|x| py_str(Some(x))));
        }
    }
    h
}

/// 便 1 の解決先（要件書の id・条 id と規範文 id・rules 行 id）+ 凍結 anchor の列に在った id。
fn known_ids(
    constitution: &Node,
    rules: &Node,
    srs: &Node,
    history: &HashSet<String>,
) -> HashSet<String> {
    let mut known: HashSet<String> = history.clone();
    for section in SRS_ID_SECTIONS {
        known.extend(rows(srs, section).map(|r| py_str(r.get("id"))));
    }
    for a in rows(constitution, "articles") {
        known.insert(py_str(a.get("id")));
        known.extend(rows(a, "statements").map(|st| py_str(st.get("id"))));
    }
    for section in RULE_SECTIONS {
        known.extend(rows(rules, section).map(|r| py_str(r.get("id"))));
    }
    known
}

/// 全欄の文字列を欄の道つきで歩く（表のキーは数えない）。
fn walk(node: &Node, at: &str, f: &mut dyn FnMut(&str, &str)) {
    match node {
        Node::Null => {}
        Node::Scalar(text) => f(at, text),
        Node::Seq(items) => {
            for (i, item) in items.iter().enumerate() {
                walk(item, &format!("{at}[{i}]"), f);
            }
        }
        Node::Map(entries) => {
            for (k, v) in entries {
                walk(v, &format!("{at}.{k}"), f);
            }
        }
    }
}

/// 本文の中の判断の記録の id を拾う。床の式 `(?<![A-Za-z0-9-])(ADR-[1-9][0-9]*)(?![A-Za-z0-9])` を文字の走査で写す
/// （4 桁の ADR-0047 は 1〜9 で始まらないので形に当たらない）。
fn scan_adr_ids(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let head: Vec<char> = "ADR-".chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let free_before = i == 0 || !(chars[i - 1].is_ascii_alphanumeric() || chars[i - 1] == '-');
        if free_before
            && chars.get(i..i + head.len()) == Some(&head[..])
            && chars
                .get(i + head.len())
                .is_some_and(|c| matches!(c, '1'..='9'))
        {
            let start = i + head.len();
            let end = start
                + chars[start..]
                    .iter()
                    .take_while(|c| c.is_ascii_digit())
                    .count();
            if chars.get(end).is_none_or(|c| !c.is_ascii_alphanumeric()) {
                out.push(chars[i..end].iter().collect());
                i = end;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// (d) 判断の記録の id の参照（正本 4 file は A-2・判断の記録と欄の決まりの plain は adr）と、
/// 判断の記録の中の内部 3 空間の id の解決（adr）。
fn references(
    constitution: &Node,
    rules: &Node,
    vocabulary: &Node,
    srs: &Node,
    adr: &Adr,
    known: &HashSet<String>,
    report: &mut Report,
) {
    let adr_ids: HashSet<&str> = adr.records.iter().map(|(id, _)| id.as_str()).collect();
    let sources: [(&str, Vec<(String, &Node)>); 4] = [
        ("constitution.yaml", population_constitution(constitution)),
        ("rules.yaml", population(rules, |k| k != "schema")),
        ("vocabulary.yaml", population(vocabulary, |_| true)),
        ("srs.yaml", population(srs, |_| true)),
    ];
    for (file, pop) in &sources {
        for (at, node) in pop {
            walk(node, at, &mut |at, text| {
                for id in scan_adr_ids(text) {
                    if !adr_ids.contains(id.as_str()) {
                        report.violation(
                            "A-2",
                            format!("{file}: {at}: 判断の記録 {id} が実在しない"),
                        );
                    }
                }
            });
        }
    }

    let mut in_records = |at: &str, text: &str| {
        for id in refs::scan_ids(text) {
            if !known.contains(&id) {
                report.violation("adr", format!("{at}: id {id} が実在しない"));
            }
        }
        for id in scan_adr_ids(text) {
            if !adr_ids.contains(id.as_str()) {
                report.violation("adr", format!("{at}: 判断の記録 {id} が実在しない"));
            }
        }
    };
    for (aid, d) in &adr.records {
        walk(d, aid, &mut in_records);
    }
    if let Some(plain) = adr.schema.get("plain") {
        walk(plain, "adr/schema.yaml.plain", &mut in_records);
    }
}

/// 憲法: schema 節を除き、meta の changes_from で始まる欄を除く。
fn population_constitution(root: &Node) -> Vec<(String, &Node)> {
    let mut out = Vec::new();
    for (key, value) in root.as_map().unwrap_or_default() {
        match (key.as_str(), value.as_map()) {
            ("schema", _) => {}
            ("meta", Some(entries)) => out.extend(
                entries
                    .iter()
                    .filter(|(k, _)| !k.starts_with("changes_from"))
                    .map(|(k, v)| (format!("meta.{k}"), v)),
            ),
            _ => out.push((key.clone(), value)),
        }
    }
    out
}

fn population(root: &Node, keep: impl Fn(&str) -> bool) -> Vec<(String, &Node)> {
    root.as_map()
        .unwrap_or_default()
        .iter()
        .filter(|(k, _)| keep(k))
        .map(|(k, v)| (k.clone(), v))
        .collect()
}

/// (e) amends の対象 = 現行の条 id ∪ 改訂の範囲の節名 ∪ 列に在った条 id ∪ 列に在った節名。
fn amends_targets(
    records: &[(String, Node)],
    article_ids: &[String],
    scope: &[String],
    history: &History,
    report: &mut Report,
) {
    let mut targets: HashSet<&str> = article_ids.iter().map(String::as_str).collect();
    targets.extend(scope.iter().map(String::as_str));
    targets.extend(
        history
            .ids
            .iter()
            .filter(|i| !i.contains('.'))
            .map(String::as_str),
    );
    targets.extend(history.scopes.iter().map(String::as_str));
    let mut sections: Vec<&str> = scope
        .iter()
        .chain(history.scopes.iter())
        .map(String::as_str)
        .collect();
    sections.sort_unstable();
    sections.dedup();
    let articles = targets.len() - sections.len();
    for (aid, d) in records {
        for e in amends_list(d) {
            let target = py_str(e.get("target"));
            if !targets.contains(target.as_str()) {
                report.violation(
                    "A-2",
                    format!(
                        "{aid}: amends の対象 {target} が条 id でも改訂の範囲の節名でもない（現行と列に在ったもの: 条 {articles} 本・節 [{}]）",
                        sections.join(", ")
                    ),
                );
            }
        }
    }
}

/// amends の項のうち表のもの（一覧でなければ 0 項）。
fn amends_list(d: &Node) -> impl Iterator<Item = &Node> {
    d.get("amends")
        .and_then(Node::as_seq)
        .unwrap_or_default()
        .iter()
        .filter(|e| e.as_map().is_some())
}

/// (f) 判断の記録の本文の英字語（便 4 の切り出し・既知の集合・免除を共有する）。
fn body_words(vocabulary: &Node, adr: &Adr, report: &mut Report) {
    // 既知の集合が読めない件は便 4 の検査が「まだ分からない」に数えてある
    let known = vocab::known_words(vocabulary, &mut Report::default());
    let mut body: vocab::Body = Vec::new();
    for (aid, d) in &adr.records {
        for k in ["title", "context", "decision", "plain"] {
            body.push((format!("{aid} {k}"), text_of(d.get(k)).to_string()));
        }
        let condition = match d.get("retreat") {
            Some(rt @ Node::Map(_)) => text_of(rt.get("condition")),
            _ => "",
        };
        body.push((format!("{aid} retreat"), condition.to_string()));
        for o in d.get("options").and_then(Node::as_seq).unwrap_or_default() {
            let joined = if o.as_map().is_some() {
                ["name", "text", "reason"]
                    .iter()
                    .map(|k| text_of(o.get(k)))
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                String::new()
            };
            let oid = if o.as_map().is_some() {
                py_str(o.get("id"))
            } else {
                "?".to_string()
            };
            body.push((format!("{aid} option {oid}"), joined));
        }
        match d.get("consequences") {
            Some(Node::Seq(items)) => {
                for x in items {
                    body.push((format!("{aid} consequences"), text_of(Some(x)).to_string()));
                }
            }
            Some(Node::Map(entries)) => {
                for (k, _) in entries {
                    body.push((format!("{aid} consequences"), k.clone()));
                }
            }
            _ => {}
        }
    }
    body.push((
        "adr/schema.yaml plain".to_string(),
        text_of(adr.schema.get("plain")).to_string(),
    ));
    for (lw, at) in vocab::unknown_words(&body, &known) {
        report.violation(
            "adr",
            format!(
                "{at}: 語彙に無い英字の語「{lw}」（判断の記録の本文は「日本語（原語）」の形で書く）"
            ),
        );
    }
}

/// 発効した判断（status が effective_status で approval が表）。
fn is_effective(d: &Node) -> bool {
    adr::in_enum(d.get("status"), adr::floor_strs(&["effective_status"]))
        && matches!(d.get("approval"), Some(Node::Map(_)))
}

/// (g) 憲法の各条の amended_by と判断の記録の amends の双方向。
fn amended_by(constitution: &Node, records: &[(String, Node)], report: &mut Report) {
    let find = |id: &str| records.iter().find(|(k, _)| k == id).map(|(_, d)| d);
    let articles: Vec<&Node> = rows(constitution, "articles").collect();
    for a in &articles {
        let id = py_str(a.get("id"));
        let entries: &[Node] = match adr::present(a, "amended_by") {
            None => &[],
            Some(Node::Seq(items)) => items,
            Some(_) => {
                report.violation("A-2", format!("{id}: amended_by が一覧でない"));
                continue;
            }
        };
        for am in entries {
            if !adr::check_keys(
                "A-2",
                &format!("{id}.amended_by"),
                am,
                &adr::AMENDED_BY_ENTRY,
                report,
            ) {
                continue;
            }
            for k in ["approved_by", "ruling", "previous_text", "rationale"] {
                if !adr::non_empty(am.get(k)) {
                    report.violation("N-4", format!("{id}: amended_by.{k} が空"));
                }
            }
            adr::check_date(
                "N-4",
                &format!("{id}.amended_by.date"),
                am.get("date"),
                report,
            );
            if !adr::scalar(am.get("ruling")).is_some_and(adr::has_ledger_id) {
                report.violation(
                    "N-4",
                    format!(
                        "{id}: amended_by.ruling「{}」に台帳 id が無い",
                        adr::show(am.get("ruling"))
                    ),
                );
            }
            let adr_id = py_str(am.get("adr"));
            let Some(r) = find(&adr_id) else {
                report.violation(
                    "N-4",
                    format!("{id}: amended_by.adr {adr_id} の判断の記録が実在しない"),
                );
                continue;
            };
            if !is_effective(r) {
                report.violation(
                    "N-4",
                    format!(
                        "{id}: amended_by.adr {adr_id} が発効していない（status {}・承認欄 {}）",
                        adr::show(r.get("status")),
                        if matches!(r.get("approval"), Some(Node::Map(_))) {
                            "有"
                        } else {
                            "無"
                        }
                    ),
                );
                continue;
            }
            let who = r.get("approval").and_then(|ap| adr::present(ap, "who"));
            if adr::present(am, "approved_by") != who {
                report.violation(
                    "N-4",
                    format!(
                        "{id}: amended_by.approved_by「{}」が {adr_id} の承認者「{}」と違う",
                        adr::show(am.get("approved_by")),
                        adr::show(who)
                    ),
                );
            }
            let previous = py_str(am.get("previous_text"));
            let matched = amends_list(r)
                .filter(|e| adr::present(e, "target") == adr::present(a, "id"))
                .any(|e| py_str(e.get("previous_text")) == previous);
            if !matched {
                report.violation(
                    "A-2",
                    format!(
                        "{id}: amended_by.previous_text が {adr_id} の amends（対象 {id}）のどれとも一致しない"
                    ),
                );
            }
        }
    }

    for (aid, d) in records {
        if !is_effective(d) {
            continue;
        }
        let mut seen: Vec<&Node> = Vec::new();
        for t in amends_list(d).filter_map(|e| adr::present(e, "target")) {
            if seen.contains(&t) {
                continue;
            }
            seen.push(t);
            let Some(article) = articles
                .iter()
                .rev()
                .find(|a| adr::present(a, "id") == Some(t))
            else {
                continue;
            };
            let has = article
                .get("amended_by")
                .and_then(Node::as_seq)
                .unwrap_or_default()
                .iter()
                .any(|am| am.as_map().is_some() && py_str(am.get("adr")) == *aid);
            if !has {
                report.violation(
                    "A-2",
                    format!(
                        "{aid}: {} の amended_by に {aid} が無い（双方向）",
                        py_str(Some(t))
                    ),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adr_ids_follow_the_floor_pattern() {
        assert_eq!(
            scan_adr_ids("ADR-1・ADR-12 と（ADR-3）、ADR-0047・xADR-4・-ADR-5・ADR-6a・ADR-7-"),
            ["ADR-1", "ADR-12", "ADR-3", "ADR-7"]
        );
        assert!(scan_adr_ids("ADR-n ADR- ADR-0").is_empty());
    }

    #[test]
    fn floor_constants_are_read_through_the_floor() {
        assert_eq!(
            adr::floor_strs(&["enums", "retreat_kind"]),
            ["spike", "measure", "ruling"]
        );
        assert_eq!(adr::floor_strs(&["enums", "surface"]), ["R-8"]);
        assert_eq!(
            adr::floor_strs(&["anchor", "scope_minimum"]),
            ["schema", "precedence", "articles"]
        );
        assert!(adr::floor_strs(&["owner"]).is_empty());
    }
}
