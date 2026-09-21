//! `folio check` の相談窓口の正本（`intake.yaml`・ADR-6 決定 (1)）の形の検査（便 18・docs/design/delivery-18.md §1 (b)）。
//! 数えるのは 未知の節・欄の非空・行 id の重複（便 0 と同じ関数・同じ種別・同じ文言の形）と、
//! 行き先の解決・質問の数の上限・相談窓口の文の英字の語（種別 intake）。英字の語の切り出し・既知の集合・免除は便 4 の `vocab.rs` を共有し、
//! rules 行 R-9 の母集団（憲法・rules・要件書）には入れない（入口の正本を種別 index で数える `entrance.rs` と同じ持ち方）。
//! 節が欄の表でない・行の一覧が表の一覧でない・values や with や yes や no が一覧でない は「まだ分からない」。
//! meta の generated・approval は本便では数えない（発効の形は ADR-6 の発効の便で決める）。正規表現は使わない。

use std::collections::HashSet;

use crate::check::{duplicate_ids, non_empty, row_id, rows, unknown_sections};
use crate::schema::Floor;
use crate::verdict::Report;
use crate::vocab;
use crate::yaml::Node;

const FILE: &str = "intake.yaml";

/// 相談窓口の正本の節の閉じた一覧（床の定数・入口と同じ持ち方）。末尾の schema は生成区間（便 77・ADR-11 決定 (4)⑤）。
pub const INTAKE_TOP_LEVEL: [&str; 6] = ["meta", "answers", "targets", "questions", "sheet", "schema"];

/// 相談窓口の正本の schema 節（生成区間）の床の木（便 77 §1 (a)）。欄の順と字面は凍結 anchor
/// tests/fixtures/schema/intake-region.txt のとおり。床（`check_intake`）は生成区間の中身をこの木と突き合わせない。
pub(crate) const INTAKE_FLOOR: Floor = Floor::Map(&[
    ("top_level", Floor::Strs(&INTAKE_TOP_LEVEL)),
    (
        "top_level_note",
        Floor::Val(
            "最上位の節の閉じた一覧（ほかの節は床が落とす・N-3）。meta と answers と targets と questions と sheet は人が書き、schema は生成区間",
        ),
    ),
]);

/// 棚に無い行き先（憲法を AI の手元へ写す＝注入）。
pub const INJECT_TARGET: &str = "inject";

/// `answers.default` が取れる固定の値（答えの無い項目を各質問の推奨回答で進める）。
pub const DEFAULT_RECOMMEND: &str = "recommend";

/// 質問の数の上限（要件書 FR1 の規範文の数）。
pub const MAX_QUESTIONS: usize = 5;

/// 相談窓口の正本 `intake` の形を数える。`index` は行き先の集合に、`vocabulary` は既知の語の集合にだけ使う。
pub fn check_intake(intake: &Node, index: &Node, vocabulary: &Node, report: &mut Report) {
    unknown_sections(FILE, intake, &INTAKE_TOP_LEVEL, report);
    let mut body: vocab::Body = Vec::new();

    if let Some(meta) = section(intake, "meta", report) {
        non_empty(
            FILE,
            "meta",
            meta,
            &["id", "title", "version", "status"],
            report,
        );
        push_text(&mut body, "meta", meta, &["title"]);
    }

    // 回答の値域（各質問の recommend の行き先・default はこれに固定の値 recommend を足したもの）
    let mut values: HashSet<String> = HashSet::new();
    if let Some(answers) = section(intake, "answers", report) {
        non_empty(FILE, "answers", answers, &["values", "default"], report);
        for (i, value) in seq("answers", answers, "values", report).iter().enumerate() {
            if let Some(s) = value.as_str() {
                body.push((format!("answers の values[{i}]"), s.to_string()));
                values.insert(s.to_string());
            }
        }
        let mut allowed = values.clone();
        allowed.insert(DEFAULT_RECOMMEND.to_string());
        resolve("answers", answers, "default", &allowed, report);
    }

    // 写像の行き先（targets の id は棚の文書か注入・with は棚の付録）
    let target_ids = target_ids(index);
    let annex_ids = shelf_ids(index, "annexes");
    let targets = rows(FILE, intake, "targets", report);
    for row in &targets {
        let at = format!("targets の行 {}", row_id(row));
        non_empty(FILE, &at, row, &["id", "type"], report);
        push_text(&mut body, &at, row, &["type", "note"]);
        resolve(&at, row, "id", &target_ids, report);
        resolve_each(&at, row, "with", &annex_ids, report);
    }
    // 質問の yes と no の行き先 = 相談窓口の正本が並べた targets の行 id
    let target_rows: HashSet<String> = id_set(&targets);
    duplicate_ids(FILE, targets, report);

    let questions = rows(FILE, intake, "questions", report);
    if questions.len() > MAX_QUESTIONS {
        report.violation("intake", format!("{FILE}: questions が FR1 の上限を超える"));
    }
    for row in &questions {
        let at = format!("questions の行 {}", row_id(row));
        non_empty(FILE, &at, row, &["id", "ask", "recommend", "why"], report);
        push_text(&mut body, &at, row, &["ask", "why"]);
        resolve(&at, row, "recommend", &values, report);
        resolve_each(&at, row, "yes", &target_rows, report);
        resolve_each(&at, row, "no", &target_rows, report);
    }
    duplicate_ids(FILE, questions, report);

    if let Some(sheet) = section(intake, "sheet", report) {
        non_empty(FILE, "sheet", sheet, &["file", "title", "explain"], report);
        push_text(&mut body, "sheet", sheet, &["title", "explain"]);
        let sections = rows(FILE, sheet, "sections", report);
        for row in &sections {
            let at = format!("sheet.sections の行 {}", row_id(row));
            non_empty(FILE, &at, row, &["id", "text"], report);
            push_text(&mut body, &at, row, &["text"]);
        }
        duplicate_ids(FILE, sections, report);
    }

    // 既知の集合が読めない件は便 4 の検査が「まだ分からない」に数えてある
    let known = vocab::known_words(vocabulary, &mut Report::default());
    for (lw, at) in vocab::unknown_words(&body, &known) {
        report.violation("intake", format!("{FILE} {at}: 語彙に無い英字の語「{lw}」"));
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

/// 一覧の欄。無い・null は 0 個、一覧でなければ「まだ分からない」。
fn seq<'a>(at: &str, row: &'a Node, field: &str, report: &mut Report) -> &'a [Node] {
    match row.get(field) {
        None | Some(Node::Null) => &[],
        Some(Node::Seq(items)) => items,
        Some(_) => {
            report.unknown(format!("{FILE}: {at} の {field} が一覧でない"));
            &[]
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
        "intake",
        format!("{FILE}: {at} の {field}: 行き先「{shown}」が一覧に無い"),
    );
}

/// 一覧の欄の各要素が `targets` に解けるか。
fn resolve_each(at: &str, row: &Node, field: &str, targets: &HashSet<String>, report: &mut Report) {
    for (i, item) in seq(at, row, field, report).iter().enumerate() {
        let shown = match item {
            Node::Null => continue,
            Node::Scalar(s) if s.trim().is_empty() || targets.contains(s) => continue,
            Node::Scalar(s) => s.clone(),
            Node::Seq(_) => "（一覧）".to_string(),
            Node::Map(_) => "（表）".to_string(),
        };
        report.violation(
            "intake",
            format!("{FILE}: {at} の {field}[{i}]: 行き先「{shown}」が一覧に無い"),
        );
    }
}

/// targets の id の行き先 = 入口の棚の documents に注入を足したもの。
fn target_ids(index: &Node) -> HashSet<String> {
    let mut ids = shelf_ids(index, "documents");
    ids.insert(INJECT_TARGET.to_string());
    ids
}

/// 入口の棚の節の行 id（読めない形は空の集合・便 12 の入口の検査が数えてある）。
fn shelf_ids(index: &Node, section: &str) -> HashSet<String> {
    let rows: Vec<&Node> = index
        .get("shelf")
        .and_then(|shelf| shelf.get(section))
        .and_then(Node::as_seq)
        .unwrap_or_default()
        .iter()
        .filter(|n| n.as_map().is_some())
        .collect();
    id_set(&rows)
}

/// 行の一覧の id の集合（id の無い行は入れない）。
fn id_set(rows: &[&Node]) -> HashSet<String> {
    rows.iter()
        .map(|row| row_id(row))
        .filter(|id| id != "?")
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml;

    #[test]
    fn intake_question_limit_is_the_number_of_fr1_statements() {
        assert_eq!(MAX_QUESTIONS, 5);
    }

    #[test]
    fn intake_target_ids_are_the_shelf_documents_plus_inject() {
        let index = yaml::parse(
            "shelf:\n  documents:\n    - {id: constitution}\n    - {id: srs}\n    - 読み物\n  annexes:\n    - {id: vocabulary}\n",
        )
        .unwrap()
        .root;
        let sorted = |set: HashSet<String>| {
            let mut got: Vec<String> = set.into_iter().collect();
            got.sort_unstable();
            got
        };
        assert_eq!(
            sorted(target_ids(&index)),
            ["constitution", "inject", "srs"]
        );
        assert_eq!(sorted(shelf_ids(&index, "annexes")), ["vocabulary"]);
        // 棚が読めない形なら空の集合（入口の検査が数える）
        assert_eq!(sorted(target_ids(&Node::Null)), ["inject"]);
        assert!(shelf_ids(&Node::Null, "documents").is_empty());
    }
}
