//! 計画の設計ノート（便 183・docs/design/delivery-183.md §1・判断の記録 ADR-31 決定 (2)・要件書 FR27・責務の層 1 読む）。
//! 置き場が規則の表に計画の名札の行（欄 key が plan-note の閾値の行・値は計画のノートの文書 id）を置くと、その設計ノートに
//! 行の索引（節の型 row-index・生成区間）と計画だけの行（row-plan）を持たせ、食い違いを床で数える。行の索引は置き場の設計ノートの
//! 契約表の行（`folio derive` と同じ母集団と読み手）から file 名の順・表の中の順に導き、書くのは `derive.rs`（folio derive --write）、
//! 比べるのは床（`note.rs` の `check_note` から `check_plan`）と folio derive --check で、2 つは同じ関数 `drift` を使う（P-6.3・P-15.2）。
//! 行の索引と計画だけの行の節は、名札の行が名指すノートにだけ置ける（名札の行が無くても掛かる・決定 (2)(イ)）。
//! 判断の表（decision-table）の行の裁定の欄は決定の欄の床（`ruling.rs`）が数える。現在地は器の持ち分（決定 (2)(オ)）。

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::floor_note::{CONTRACT_TABLE, ROW_INDEX, ROW_PLAN, ROWS_BEGIN, ROWS_END};
use crate::note::NoteDoc;
use crate::rules;
use crate::shelf::is_doc_id;
use crate::verdict::Report;
use crate::yaml::{self, Node};

/// 違反の種別（設計ノートの形と同じ）。
const KIND: &str = "note";

/// 行の索引の行 1 つ（契約表の行の id・所属の文書 id）。
pub(crate) type IndexRow = (String, String);

/// 生成区間（begin の行の次の byte・end の行の頭の byte・begin の行の頭の空白）。
struct Region {
    start: usize,
    end: usize,
    indent: String,
}

/// 置き場の設計ノート（名の昇順に読んだもの）の契約表の行から行の索引を導く（file 名の順・表の中の順）。
/// 行 id か文書 id が id の形でなければ書けない（Err・呼び手は まだ分からない）。
pub(crate) fn index_rows(notes: &[NoteDoc]) -> Result<Vec<IndexRow>, String> {
    let mut out = Vec::new();
    for note in notes {
        for section in seq(note.root.get("sections")).filter(|s| is_type(s, CONTRACT_TABLE)) {
            for row in seq(section.get("rows")) {
                let id = row.get("id").and_then(Node::as_str).unwrap_or_default();
                if !is_doc_id(id) || !is_doc_id(&note.id) {
                    return Err(format!(
                        "design-note/{}: 契約表の行 id「{id}」が id の形でない＝行の索引に書けない",
                        note.file
                    ));
                }
                out.push((id.to_string(), note.id.clone()));
            }
        }
    }
    Ok(out)
}

/// 生成区間の中身（行ごとに begin の印と同じ頭の空白で `- {id: <id>, doc: <文書 id>}`）。
fn render(rows: &[IndexRow], indent: &str) -> String {
    rows.iter()
        .map(|(id, doc)| format!("{indent}- {{id: {id}, doc: {doc}}}\n"))
        .collect()
}

/// 印の行（頭の空白を除いた全部が字面のとおり）がそれぞれちょうど 1 本で begin が先のときだけ定まる。
fn region(text: &str) -> Result<Region, String> {
    let mut begins = Vec::new();
    let mut ends = Vec::new();
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let body = line.strip_suffix('\n').unwrap_or(line);
        let trimmed = body.trim_start_matches(' ');
        if trimmed == ROWS_BEGIN {
            begins.push((at + line.len(), body[..body.len() - trimmed.len()].to_string()));
        }
        if trimmed == ROWS_END {
            ends.push(at);
        }
        at += line.len();
    }
    match (&begins[..], &ends[..]) {
        ([(start, indent)], [end]) if start <= end => Ok(Region {
            start: *start,
            end: *end,
            indent: indent.clone(),
        }),
        _ => Err(format!(
            "行の索引の生成区間の印が 1 対でない（begin {}・end {}）",
            begins.len(),
            ends.len()
        )),
    }
}

/// 印の間の字だけが導出と違う理由（契約表との突き合わせ・床はつながりに数える・ほかの理由は計画のノートの形で止める・便 199）。
const CONTENT_DRIFT: &str = "行の索引の生成区間が契約表からの導出と違う（folio derive --write で書き直す）";

/// 計画のノート（木と file の字）の行の索引が導出 `rows` と食い違う理由（無ければ None）。床と folio derive --check が使う。
/// 行の索引の節がちょうど 1 つ・印が 1 対・印の間が導出と byte で同じ・節の行が導出と同じ（印が節の外なら割れる）。
pub(crate) fn drift(root: &Node, text: &str, rows: &[IndexRow]) -> Option<String> {
    const REDO: &str = "folio derive --write で書き直す";
    let sections: Vec<&Node> = seq(root.get("sections")).filter(|s| is_type(s, ROW_INDEX)).collect();
    let [section] = sections[..] else {
        return Some(format!("行の索引の節（{ROW_INDEX}）が {} 個ある（ちょうど 1 つ）", sections.len()));
    };
    let r = match region(text) {
        Ok(r) => r,
        Err(e) => return Some(e),
    };
    if text[r.start..r.end] != render(rows, &r.indent) {
        return Some(CONTENT_DRIFT.to_string());
    }
    let have: Vec<IndexRow> = seq(section.get("rows"))
        .map(|row| (text_of(row, "id"), text_of(row, "doc")))
        .collect();
    (have != rows).then(|| format!("行の索引の節の行が生成区間の導出と違う（印が節の rows の外に在る・{REDO}）"))
}

/// 計画のノートの file の字の生成区間を導出 `rows` に書き換えた字（印が 1 対でなければ Err）。folio derive --write が使う。
pub(crate) fn rewrite(text: &str, rows: &[IndexRow]) -> Result<String, String> {
    let r = region(text)?;
    Ok(format!("{}{}{}", &text[..r.start], render(rows, &r.indent), &text[r.end..]))
}

/// 床（`note.rs` の `check_note` が読めた設計ノート全部を渡す）。名札の行の読みが割れれば まだ分からない 1 つで止める。
pub(crate) fn check_plan(nd: &Path, notes: &[NoteDoc], rules: &Node, report: &mut Report) {
    // 名札の行が退けられていれば（状態が廃止）置き場の決まりも計画の床も数えない
    if rules::retired(rules, rules::PLAN_NOTE) {
        return;
    }
    let plan = match rules::plan_note(rules) {
        Ok(p) => p,
        Err(e) => {
            report.unknown(format!("rules.yaml: 計画のノートの名札の行が読めない: {e}"));
            return;
        }
    };
    // 置き場の決まり（名札の行の有無に関わらず）
    for note in notes {
        for section in seq(note.root.get("sections")) {
            let ty = section.get("type").and_then(Node::as_str).unwrap_or_default();
            if (ty == ROW_INDEX || ty == ROW_PLAN) && plan != Some(note.id.as_str()) {
                let n = section.get("n").and_then(Node::as_str).unwrap_or("?");
                let named = plan.map_or("名札の行が無い".to_string(), |p| format!("名指すのは {p}"));
                report.link(
                    KIND,
                    format!(
                        "design-note/{}: §{n}: 節の型 {ty} は計画の名札の行（欄 key が {} の閾値の行）が名指す計画のノートにだけ置ける（{named}）",
                        note.file,
                        rules::PLAN_NOTE
                    ),
                );
            }
        }
    }
    let Some(id) = plan else { return };
    let Some(note) = notes.iter().find(|n| n.id == id) else {
        report.unknown(format!("rules.yaml: 計画の名札の行が名指す計画のノート design-note/{id}.yaml が無い"));
        return;
    };
    let file = format!("design-note/{}", note.file);
    let status = note.root.get("meta").and_then(|m| m.get("status")).and_then(Node::as_str);
    if status != Some("effective") {
        report.unknown(format!(
            "{file}: 計画のノートの状態が effective でない（{}）＝計画の床を数えない",
            status.unwrap_or("無い")
        ));
        return;
    }
    let index = match index_rows(notes) {
        Ok(rows) => rows,
        Err(e) => {
            report.unknown(e);
            return;
        }
    };
    match fs::read_to_string(nd.join(&note.file)) {
        Ok(text) => {
            match drift(&note.root, &text, &index) {
                Some(why) if why == CONTENT_DRIFT => report.link(KIND, format!("{file}: {why}")),
                Some(why) => report.violation(KIND, format!("{file}: {why}")),
                None => {}
            }
        }
        Err(e) => report.unknown(format!("{file}: 読めない: {e}")),
    }
    plan_only_rows(note, &index, &file, report);
}

/// 計画だけの行の床（`index` は行の索引・`file` は計画のノートの file の字）。
fn plan_only_rows(note: &NoteDoc, index: &[IndexRow], file: &str, report: &mut Report) {
    // 計画だけの行（一意・索引に無い・依存は索引か計画だけの行に在る）
    let indexed: HashSet<&str> = index.iter().map(|(id, _)| id.as_str()).collect();
    let rows: Vec<&Node> = seq(note.root.get("sections"))
        .filter(|s| is_type(s, ROW_PLAN))
        .flat_map(|s| seq(s.get("rows")))
        .collect();
    let mut planned = HashSet::new();
    for row in &rows {
        let Some(rid) = row.get("id").and_then(Node::as_str) else { continue };
        if !planned.insert(rid) {
            report.violation(KIND, format!("{file}: 計画だけの行 id「{rid}」が 2 度在る"));
        }
        if indexed.contains(rid) {
            report.link(
                KIND,
                format!("{file}: 計画だけの行「{rid}」が行の索引に在る（契約の行が在る＝計画だけの行の節から外す）"),
            );
        }
    }
    for row in &rows {
        let rid = row.get("id").and_then(Node::as_str).unwrap_or("?");
        for dep in seq(row.get("depends")).filter_map(Node::as_str) {
            if !indexed.contains(dep) && !planned.contains(dep) {
                report.link(
                    KIND,
                    format!("{file}: 計画だけの行「{rid}」の depends「{dep}」が行の索引にも計画だけの行にも無い（宙に浮いた依存）"),
                );
            }
        }
    }
}

/// 一覧の項（一覧でなければ 0 個）。
fn seq(node: Option<&Node>) -> impl Iterator<Item = &Node> {
    node.and_then(Node::as_seq).unwrap_or_default().iter()
}

fn is_type(section: &Node, ty: &str) -> bool {
    section.get("type").and_then(Node::as_str) == Some(ty)
}

fn text_of(row: &Node, key: &str) -> String {
    row.get(key).and_then(Node::as_str).unwrap_or_default().to_string()
}

/// 規則の表を置き場から読む（folio derive が使う・無ければ None＝名札の行が無いと同じ・読めなければ Err）。
pub(crate) fn load_rules(dir: &Path) -> Result<Option<Node>, String> {
    let path = dir.join("rules.yaml");
    if path.symlink_metadata().is_err() {
        return Ok(None);
    }
    if path.is_symlink() || !path.is_file() {
        return Err("rules.yaml: file でない（symlink を含む）".to_string());
    }
    let text = fs::read_to_string(&path).map_err(|e| format!("rules.yaml: 読めない: {e}"))?;
    let doc = yaml::parse(&text).map_err(|e| format!("rules.yaml: parse できない: {e}"))?;
    Ok(Some(doc.root))
}

#[cfg(test)]
mod tests {
    use super::*;

    const B: &str = ROWS_BEGIN;
    const E: &str = ROWS_END;

    fn root(text: &str) -> Node {
        yaml::parse(text).unwrap().root
    }

    fn plan(rows: &str) -> String {
        format!("sections:\n  - n: 1\n    type: row-index\n    title: 行の索引\n    rows:\n      {B}\n{rows}      {E}\n")
    }

    /// 便 183 (c): 生成区間は begin の頭の空白で行を書き、書き直した字は導出と一致して drift が None。
    /// 印が 1 対でない・節が 2 つ・区間の字の違い・印が節の外、はそれぞれ理由を返す。
    #[test]
    fn f183_the_region_is_written_and_read_by_one_function() {
        let rows = vec![("a".to_string(), "wave-a".to_string()), ("b".to_string(), "wave-b".to_string())];
        let empty = plan("");
        let text = rewrite(&empty, &rows).unwrap();
        assert!(text.contains("      - {id: a, doc: wave-a}\n      - {id: b, doc: wave-b}\n"), "{text}");
        assert_eq!(drift(&root(&text), &text, &rows), None);
        assert_eq!(rewrite(&text, &rows).unwrap(), text);
        assert_eq!(drift(&root(&empty), &empty, &[]), None);
        let stale = drift(&root(&empty), &empty, &rows).unwrap();
        assert!(stale.contains("導出と違う（folio derive --write で書き直す）"), "{stale}");
        let unmarked = text.replace(B, "# x");
        assert!(drift(&root(&unmarked), &unmarked, &rows).unwrap().contains("印が 1 対でない（begin 0・end 1）"));
        assert!(rewrite(&unmarked, &rows).is_err());
        let two = format!("{text}  - n: 2\n    type: row-index\n    title: x\n    rows: []\n");
        assert!(drift(&root(&two), &two, &rows).unwrap().contains("が 2 個ある"));
        // 印を散文の body の中に置くと、区間の字は合っても節の行が割れる
        let outside = format!(
            "sections:\n  - n: 1\n    type: prose\n    title: x\n    body: |\n      {B}\n      - {{id: a, doc: wave-a}}\n      {E}\n  - n: 2\n    type: row-index\n    title: y\n    rows: []\n"
        );
        let one = &rows[..1];
        assert!(drift(&root(&outside), &outside, one).unwrap().contains("印が節の rows の外"));
    }
}
