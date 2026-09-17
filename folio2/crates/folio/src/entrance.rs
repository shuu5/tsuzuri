//! `folio check` の入口の面の正本（`index.yaml`・ADR-5 決定 (2)）の形の検査（便 12・docs/design/delivery-12.md §1 (b)）。
//! 数えるのは 未知の節・欄の非空・行 id の重複（便 0 と同じ関数・同じ種別・同じ文言の形）と、
//! 行き先の解決・minutes・入口の文の英字の語（種別 index）。英字の語の切り出し・既知の集合・免除は便 4 の `vocab.rs` を共有し、
//! rules 行 R-9 の母集団（憲法・rules・要件書）には入れない（判断の記録の本文を種別 adr で数える `link.rs` と同じ持ち方）。
//! 節が欄の表でない・行の一覧が表の一覧でない・stops や steps が一覧でない は「まだ分からない」。
//! meta の approval・generated と発効の状態の値域は本便では数えない。正規表現は使わない。

use std::collections::HashSet;

use crate::check::{duplicate_ids, non_empty, row_id, rows, unknown_sections};
use crate::verdict::Report;
use crate::vocab;
use crate::yaml::Node;

const FILE: &str = "index.yaml";

/// 入口の正本の節の閉じた一覧（床の定数・要件書と同じ持ち方）。
pub const INDEX_TOP_LEVEL: [&str; 5] = ["meta", "audience", "shelf", "lanes", "intake"];

/// 入口の正本 `index` の形を数える。`vocabulary` は既知の語の集合にだけ使う。
pub fn check_entrance(index: &Node, vocabulary: &Node, report: &mut Report) {
    unknown_sections(FILE, index, &INDEX_TOP_LEVEL, report);
    let mut body: vocab::Body = Vec::new();

    if let Some(meta) = section(index, "meta", report) {
        non_empty(
            FILE,
            "meta",
            meta,
            &["id", "title", "version", "status"],
            report,
        );
        push_text(&mut body, "meta", meta, &["title"]);
    }

    if let Some(audience) = section(index, "audience", report) {
        non_empty(
            FILE,
            "audience",
            audience,
            &["label", "short", "text"],
            report,
        );
        push_text(&mut body, "audience", audience, &["label", "short", "text"]);
    }

    // 行き先の集合のうち documents だけのもの（annexes の inside と stops の doc の行き先）
    let mut document_ids = HashSet::new();
    if let Some(shelf) = section(index, "shelf", report) {
        non_empty(FILE, "shelf", shelf, &["title", "explain"], report);
        push_text(&mut body, "shelf", shelf, &["title", "explain"]);

        let legend = rows(FILE, shelf, "legend", report);
        for row in &legend {
            let at = format!("shelf.legend の行 {}", row_id(row));
            non_empty(FILE, &at, row, &["id", "text"], report);
            push_text(&mut body, &at, row, &["text"]);
        }
        duplicate_ids(FILE, legend, report);

        let documents = rows(FILE, shelf, "documents", report);
        for row in &documents {
            let at = format!("shelf.documents の行 {}", row_id(row));
            non_empty(FILE, &at, row, &["id", "type", "use"], report);
            // absent は null でよい（数えない）・文字列なら語彙の母集団に入れる
            push_text(&mut body, &at, row, &["type", "use", "absent"]);
        }
        let annexes = rows(FILE, shelf, "annexes", report);
        for row in &annexes {
            let at = format!("shelf.annexes の行 {}", row_id(row));
            non_empty(FILE, &at, row, &["id", "type", "inside"], report);
            push_text(&mut body, &at, row, &["type"]);
        }
        // 棚 = documents と annexes を合わせた 1 つの空間（relations の from と to の行き先）
        let shelf_ids = shelf_id_set(&documents, &annexes);
        document_ids = shelf_id_set(&documents, &[]);
        for row in &annexes {
            let at = format!("shelf.annexes の行 {}", row_id(row));
            resolve(&at, row, "inside", &document_ids, report);
        }
        duplicate_ids(FILE, documents.iter().chain(&annexes).copied(), report);

        let relations = rows(FILE, shelf, "relations", report);
        for row in &relations {
            let at = format!("shelf.relations の行 {}", row_id(row));
            non_empty(
                FILE,
                &at,
                row,
                &["id", "from", "to", "label", "hint"],
                report,
            );
            resolve(&at, row, "from", &shelf_ids, report);
            resolve(&at, row, "to", &shelf_ids, report);
            push_text(&mut body, &at, row, &["label", "hint"]);
        }
        duplicate_ids(FILE, relations, report);
    }

    if let Some(lanes) = section(index, "lanes", report) {
        non_empty(FILE, "lanes", lanes, &["title", "lead"], report);
        push_text(&mut body, "lanes", lanes, &["title", "lead"]);
        let lane_rows = rows(FILE, lanes, "rows", report);
        for row in &lane_rows {
            let id = row_id(row);
            let at = format!("lanes の行 {id}");
            non_empty(
                FILE,
                &at,
                row,
                &["id", "mark", "who", "why", "minutes", "stops"],
                report,
            );
            push_text(&mut body, &at, row, &["mark", "who", "why"]);
            if row.get("minutes").is_some_and(|m| !m.is_blank()) && !is_minutes(row.get("minutes"))
            {
                report.violation(
                    "index",
                    format!("{FILE}: lanes の行 {id}: minutes が 1 以上の整数でない"),
                );
            }
            // stops の doc の行き先は shelf.documents の id だけ（annexes は入れない）
            for (i, stop) in rows(FILE, row, "stops", report).iter().enumerate() {
                let at = format!("{at} の stops[{i}]");
                non_empty(FILE, &at, stop, &["doc", "label"], report);
                resolve(&at, stop, "doc", &document_ids, report);
                push_text(&mut body, &at, stop, &["label"]);
            }
        }
        duplicate_ids(FILE, lane_rows, report);
    }

    if let Some(intake) = section(index, "intake", report) {
        non_empty(
            FILE,
            "intake",
            intake,
            &["title", "lead", "heading", "text", "steps"],
            report,
        );
        // note は任意
        push_text(
            &mut body,
            "intake",
            intake,
            &["title", "lead", "heading", "text", "note"],
        );
        match intake.get("steps") {
            None | Some(Node::Null) => {}
            Some(Node::Seq(items)) => {
                for (i, step) in items.iter().enumerate() {
                    if let Some(s) = step.as_str() {
                        body.push((format!("intake の steps[{i}]"), s.to_string()));
                    }
                }
            }
            Some(_) => report.unknown(format!("{FILE}: steps が一覧でない")),
        }
    }

    // 既知の集合が読めない件は便 4 の検査が「まだ分からない」に数えてある
    let known = vocab::known_words(vocabulary, &mut Report::default());
    for (lw, at) in vocab::unknown_words(&body, &known) {
        report.violation("index", format!("{FILE} {at}: 語彙に無い英字の語「{lw}」"));
    }
}

/// 欄の表の節。無い・null は空の表として扱い（欄の非空が数える）、表でなければ「まだ分からない」。
fn section<'a>(root: &'a Node, name: &str, report: &mut Report) -> Option<&'a Node> {
    match root.get(name) {
        None | Some(Node::Null) => Some(&Node::Null),
        Some(node @ Node::Map(_)) => Some(node),
        Some(_) => {
            report.unknown(format!("{FILE}: {name} が欄の表でない"));
            None
        }
    }
}

/// 文を持つ欄を語彙の母集団に足す（文字列でない欄は数えない）。
fn push_text(body: &mut vocab::Body, at: &str, row: &Node, fields: &[&str]) {
    for field in fields {
        if let Some(s) = row.get(field).and_then(Node::as_str) {
            body.push((format!("{at} の {field}"), s.to_string()));
        }
    }
}

/// 行き先の欄が `targets` に解けるか。空の欄は欄の非空が数えるのでここでは数えない。
fn resolve(at: &str, row: &Node, field: &str, targets: &HashSet<String>, report: &mut Report) {
    let shown = match row.get(field) {
        None | Some(Node::Null) => return,
        Some(Node::Scalar(s)) => {
            if s.trim().is_empty() || targets.contains(s) {
                return;
            }
            s.clone()
        }
        Some(Node::Seq(items)) if items.is_empty() => return,
        Some(Node::Map(entries)) if entries.is_empty() => return,
        Some(Node::Seq(_)) => "（一覧）".to_string(),
        Some(Node::Map(_)) => "（表）".to_string(),
    };
    report.violation(
        "index",
        format!("{FILE}: {at} の {field}: 行き先「{shown}」が棚に無い"),
    );
}

/// 棚の id の集合 = shelf.documents と shelf.annexes を合わせた 1 つの空間。
fn shelf_id_set(documents: &[&Node], annexes: &[&Node]) -> HashSet<String> {
    documents
        .iter()
        .chain(annexes)
        .map(|r| row_id(r))
        .filter(|id| id != "?")
        .collect()
}

/// minutes が 1 以上の整数か（数字だけの字面で値が 1 以上）。
fn is_minutes(node: Option<&Node>) -> bool {
    node.and_then(Node::as_str)
        .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|s| s.parse::<u64>().ok())
        .is_some_and(|n| n >= 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scalar(s: &str) -> Node {
        Node::Scalar(s.to_string())
    }

    #[test]
    fn entrance_minutes_must_be_a_positive_integer() {
        for ok in ["1", "10", "120"] {
            assert!(is_minutes(Some(&scalar(ok))), "{ok}");
        }
        for ng in ["0", "-1", "1.5", "十", "ten", "", " 1"] {
            assert!(!is_minutes(Some(&scalar(ng))), "{ng}");
        }
        assert!(!is_minutes(None));
        assert!(!is_minutes(Some(&Node::Null)));
        assert!(!is_minutes(Some(&Node::Seq(vec![scalar("1")]))));
    }

    #[test]
    fn entrance_shelf_ids_join_documents_and_annexes() {
        let row = |id: &str| Node::Map(vec![("id".to_string(), scalar(id))]);
        let documents = [row("constitution"), row("srs")];
        let annexes = [row("vocabulary"), Node::Map(vec![])];
        let set = shelf_id_set(
            &documents.iter().collect::<Vec<_>>(),
            &annexes.iter().collect::<Vec<_>>(),
        );
        let mut got: Vec<&str> = set.iter().map(String::as_str).collect();
        got.sort_unstable();
        assert_eq!(got, ["constitution", "srs", "vocabulary"]);
    }
}
