//! `folio check` の天井の正本（`ceiling.yaml`・ADR-8 決定 (1)(3)(4)）の形の検査（便 37・docs/design/delivery-37.md §1 (b)）。
//! 数えるのは 未知の節・欄の非空・行 id の重複（便 0 と同じ関数・同じ種別・同じ文言の形）と、
//! 固定の一覧（3 値・観点の列・文書の集合・所見と記録の欄の名・束の中身）・行き先の解決・天井の正本の文の英字の語（種別 ceiling）。
//! 英字の語の切り出し・既知の集合・免除は便 4 の `vocab.rs` を共有し、rules 行 R-9 の母集団（憲法・rules・要件書）には入れない
//! （相談窓口の正本を種別 intake で数える `intake.rs` と同じ持ち方）。fields の値と finding / record の欄の名は機械の層なので語彙に通さない。
//! 節が欄の表でない・行の一覧が表の一覧でない・values や refute や required や contents や fields が一覧でない は「まだ分からない」。
//! documents の note・finding の optional・meta の generated・approval は数えない。材料の束を組む命令・所見 file の検査・面の名札は後続の便。
//!
//! 天井の床の定数は全部この 1 枚に置く（便 47・docs/design/delivery-47.md §1・ADR-11 決定 (4)①・P-5.1 / P-5.6）。床の木 `FLOOR` は
//! 天井の正本の最上位の節 `schema`（生成区間）の正本で、葉は下の定数と同じ配列を指す（同じ一覧を 2 回書かない）。`_note` で終わる
//! 欄は人が読む説明の注で、凍結 anchor tests/fixtures/schema/ceiling-region.txt の順と字面のまま持つ（`schema.rs` の `derive` が
//! 書き、`floor_diff` は読まない）。束を組む命令（`bundle.rs`）と所見の検査（`findings.rs`）の読み手も一覧をここから取る。
//! 節 `schema` が在れば `FLOOR` と突き合わせ、旧 4 節（verdicts・finding・record・bundle）は無くてもよい。無ければ今までどおり旧 4 節が必須。

use std::collections::HashSet;

use crate::check::{duplicate_ids, non_empty, row_id, rows, unknown_sections};
use crate::schema::{Floor, floor_diff, strip_notes};
use crate::verdict::Report;
use crate::vocab;
use crate::yaml::Node;

const FILE: &str = "ceiling.yaml";

/// 天井の正本の最上位の節の閉じた一覧（`FLOOR` の top_level・人が書く 4 節 + 生成区間 schema）。
pub const CEILING_TOP_LEVEL: [&str; 5] = ["meta", "weights", "documents", "viewpoints", "schema"];

/// 移行のあいだだけ許す旧 4 節（人が書いた一覧・第 3 版で生成区間 schema へ移る・便 48 で消す）。
pub const LEGACY_SECTIONS: [&str; 4] = ["verdicts", "finding", "record", "bundle"];

/// 観点ごとの 3 値（要件書 FR5 の 3 値・順も固定）。
pub const VERDICT_VALUES: [&str; 3] = ["合格", "不合格", "まだ分からない"];

/// 観点の id の列（ADR-8 決定 (1)・順も固定・増減はどちらも違反）。
pub const VIEWPOINT_IDS: [&str; 4] = ["fidelity", "readability", "coherence", "reality"];

/// 読む文書の id の集合 = 正本 7 file + 判断の記録 + 設計ノート（増減はどちらも違反）。
pub const DOCUMENT_IDS: [&str; 9] = [
    "constitution",
    "rules",
    "vocabulary",
    "srs",
    "index",
    "intake",
    "ceiling",
    "adr",
    "design-note",
];

/// 所見 1 件が必ず持つ欄。
pub const FINDING_REQUIRED: [&str; 5] = ["id", "viewpoint", "place", "weight", "evidence"];

/// 所見 1 件が持ってよい欄。
pub const FINDING_OPTIONAL: [&str; 2] = ["refute", "note"];

/// 所見の場所が必ず持つ欄。
pub const PLACE_REQUIRED: [&str; 2] = ["doc", "at"];

/// 反証の結果の値域。
pub const REFUTE_VALUES: [&str; 3] = ["支持", "退けた", "まだ分からない"];

/// 起動の記録が必ず持つ欄。
pub const RECORD_REQUIRED: [&str; 5] = ["model", "effort", "at", "read", "bundle"];

/// 材料の束の中身（順も同じ・`bundle.rs` が組む）。
pub const BUNDLE_CONTENTS: [&str; 5] = ["sources", "faces", "question", "finding", "reads"];

/// 束の要約値の規則の名（digest.txt の頭）。
pub const BUNDLE_DIGEST: &str = "sha256-files-1";

/// 反証の束の中身（名の byte 順・digest.txt と result.yaml は数えない・`findings.rs` が組む）。
pub const REFUTE_CONTENTS: [&str; 5] = [
    "finding.yaml",
    "question.yaml",
    "reads.yaml",
    "schema.yaml",
    "sources.txt",
];

/// 反証役が書く結果 file の欄（他の欄は違反・全部空でない文）。
pub const RESULT_REQUIRED: [&str; 6] = ["id", "refute", "model", "effort", "at", "bundle"];

/// 反証の規則の文（逐語・反証の束の question.yaml へ写す）。
pub const REFUTE_RULE: &str = "所見を出した文脈から独立して中立に検証する。根拠が正本に逐語で在り、主張が正本の文から裏付けられれば 支持。根拠が無い、または主張が正本の文と両立しないと裏付けられれば 退けた。材料だけでは決められなければ まだ分からない（所見は残る）。";

/// 床の木（天井の正本の最上位の節 `schema` の正本・便 47 §1 (a)）。欄と順と字面は凍結 anchor
/// tests/fixtures/schema/ceiling-region.txt のとおり（`derive` の結果が byte 一致・単体の歯が数える）。葉は上の定数と同じ配列。
pub(crate) const FLOOR: Floor = Floor::Map(&[
    ("top_level", Floor::Strs(&CEILING_TOP_LEVEL)),
    (
        "top_level_note",
        Floor::Val(
            "最上位の節の閉じた一覧（ほかの節は床が落とす）。meta・weights・documents・viewpoints は人が書き、schema は生成区間",
        ),
    ),
    (
        "verdicts",
        Floor::Map(&[("values", Floor::Strs(&VERDICT_VALUES))]),
    ),
    (
        "verdicts_note",
        Floor::Val(
            "観点ごとの 3 値（順も固定）。1 つでも「まだ分からない」なら天井は合格にならない。何にするかの裁定の正本は要件書 FR5 と判断の記録 ADR-8 決定 (4)",
        ),
    ),
    ("viewpoints", Floor::Strs(&VIEWPOINT_IDS)),
    (
        "viewpoints_note",
        Floor::Val(
            "観点の閉じた id の列（順も固定）。viewpoints の行はこの id に 1 つずつ要る。何にするかの裁定の正本は判断の記録 ADR-8 決定 (1)。観点を増やさないことは憲法 N-5",
        ),
    ),
    ("documents", Floor::Strs(&DOCUMENT_IDS)),
    (
        "documents_note",
        Floor::Val(
            "読む文書の閉じた id の集合（順は問わない）。documents の行はこの id に 1 つずつ要る。所見の場所の doc と観点の reads の doc はこの行に解く",
        ),
    ),
    (
        "finding",
        Floor::Map(&[
            ("required", Floor::Strs(&FINDING_REQUIRED)),
            ("optional", Floor::Strs(&FINDING_OPTIONAL)),
            (
                "place",
                Floor::Map(&[("required", Floor::Strs(&PLACE_REQUIRED))]),
            ),
            (
                "refute",
                Floor::Map(&[("values", Floor::Strs(&REFUTE_VALUES))]),
            ),
        ]),
    ),
    (
        "finding_note",
        Floor::Val(
            "所見 1 件の欄の決まり。place は documents の id と、欄か節か行の id。evidence は正本の逐語の引用（床が実在を照合する）。weight の値域と反証に回す重さは人が書く weights の節が持つ",
        ),
    ),
    (
        "record",
        Floor::Map(&[("required", Floor::Strs(&RECORD_REQUIRED))]),
    ),
    (
        "record_note",
        Floor::Val(
            "起動の記録の欄（観点ごとに 1 つ・合格の観点にも必須）。1 つでも欠けるか bundle の要約値が束と合わなければ、床がその観点を「まだ分からない」に落とす（判断の記録 ADR-8 決定 (3)）",
        ),
    ),
    (
        "bundle",
        Floor::Map(&[
            ("contents", Floor::Strs(&BUNDLE_CONTENTS)),
            ("digest", Floor::Val(BUNDLE_DIGEST)),
        ]),
    ),
    (
        "bundle_note",
        Floor::Val(
            "材料の束の中身（観点ごとに 1 つの置き場）と要約値の規則。要約値は束の file を path の byte 順に並べ、中身を連結した sha256（rules 行 R-15 の写しの要約値と同じ規則）",
        ),
    ),
    (
        "refute",
        Floor::Map(&[
            ("contents", Floor::Strs(&REFUTE_CONTENTS)),
            ("result_required", Floor::Strs(&RESULT_REQUIRED)),
            ("rule", Floor::Val(REFUTE_RULE)),
        ]),
    ),
    (
        "refute_note",
        Floor::Val(
            "反証の束の中身（名の byte 順）・反証役が書く結果の file の欄（全部空でない文）・反証役へ渡す規則の文（逐語）",
        ),
    ),
]);

/// 天井の正本 `ceiling` の形を数える。`vocabulary` は既知の語の集合にだけ使う。
pub fn check_ceiling(ceiling: &Node, vocabulary: &Node, report: &mut Report) {
    let allowed: Vec<&str> = CEILING_TOP_LEVEL
        .iter()
        .chain(LEGACY_SECTIONS.iter())
        .copied()
        .collect();
    unknown_sections(FILE, ceiling, &allowed, report);
    // 生成区間 schema が在れば床の木と突き合わせ、旧 4 節は無くてもよい（在れば今までどおり数える）
    let generated = match ceiling.get("schema") {
        Some(schema) => {
            check_schema(schema, report);
            true
        }
        None => false,
    };
    let mut body: vocab::Body = Vec::new();

    if let Some(meta) = section(ceiling, "meta", report) {
        non_empty(
            FILE,
            "meta",
            meta,
            &["id", "title", "version", "status"],
            report,
        );
        push_text(&mut body, "meta", meta, &["title"]);
    }

    if let Some(verdicts) = legacy(ceiling, "verdicts", generated, report) {
        non_empty(FILE, "verdicts", verdicts, &["values"], report);
        if let Some(values) = seq("verdicts", verdicts, "values", report) {
            same_list(
                "verdicts の values",
                &strings(values),
                &VERDICT_VALUES,
                report,
            );
        }
    }

    // 反証に回す重さは重さの値域のどれか
    if let Some(weights) = section(ceiling, "weights", report) {
        non_empty(FILE, "weights", weights, &["values", "refute"], report);
        let values: HashSet<String> = seq("weights", weights, "values", report)
            .map(|items| strings(items).into_iter().collect())
            .unwrap_or_default();
        resolve_each("weights", weights, "refute", &values, report);
    }

    // 読む文書の一覧（所見の場所と観点の reads の行き先）
    let documents = table(ceiling, "documents", report);
    let mut doc_ids: HashSet<String> = HashSet::new();
    if let Some(documents) = &documents {
        for row in documents {
            let at = format!("documents の行 {}", row_id(row));
            non_empty(FILE, &at, row, &["id", "file"], report);
            push_text(&mut body, &at, row, &["note"]);
        }
        doc_ids = id_set(documents);
        same_set("documents の id", &ids(documents), &DOCUMENT_IDS, report);
        duplicate_ids(FILE, documents.iter().copied(), report);
    }

    if let Some(viewpoints) = table(ceiling, "viewpoints", report) {
        for row in &viewpoints {
            let at = format!("viewpoints の行 {}", row_id(row));
            non_empty(
                FILE,
                &at,
                row,
                &["id", "name", "reader", "question", "reads"],
                report,
            );
            push_text(&mut body, &at, row, &["name", "reader", "question"]);
            for (i, read) in reads(&at, row, report).iter().enumerate() {
                let at = format!("{at} の reads[{i}]");
                non_empty(FILE, &at, read, &["doc", "fields"], report);
                // 行き先の集合が読めない形なら documents の側が「まだ分からない」に数えてある
                if documents.is_some() {
                    resolve(&at, read, "doc", &doc_ids, report);
                }
                seq(&at, read, "fields", report);
            }
        }
        same_list(
            "viewpoints の id",
            &ids(&viewpoints),
            &VIEWPOINT_IDS,
            report,
        );
        duplicate_ids(FILE, viewpoints, report);
    }

    if let Some(finding) = legacy(ceiling, "finding", generated, report) {
        non_empty(
            FILE,
            "finding",
            finding,
            &["required", "place", "refute"],
            report,
        );
        if let Some(required) = seq("finding", finding, "required", report) {
            contains_all(
                "finding の required",
                &strings(required),
                &FINDING_REQUIRED,
                report,
            );
        }
        if let Some(place) = section(finding, "place", report) {
            non_empty(FILE, "finding.place", place, &["required"], report);
            if let Some(required) = seq("finding.place", place, "required", report) {
                contains_all(
                    "finding.place の required",
                    &strings(required),
                    &PLACE_REQUIRED,
                    report,
                );
            }
        }
        if let Some(refute) = section(finding, "refute", report) {
            non_empty(FILE, "finding.refute", refute, &["values"], report);
            if let Some(values) = seq("finding.refute", refute, "values", report) {
                same_set(
                    "finding.refute の values",
                    &strings(values),
                    &REFUTE_VALUES,
                    report,
                );
            }
        }
    }

    if let Some(record) = legacy(ceiling, "record", generated, report) {
        non_empty(FILE, "record", record, &["required"], report);
        if let Some(required) = seq("record", record, "required", report) {
            contains_all(
                "record の required",
                &strings(required),
                &RECORD_REQUIRED,
                report,
            );
        }
    }

    if let Some(bundle) = legacy(ceiling, "bundle", generated, report) {
        non_empty(FILE, "bundle", bundle, &["contents", "digest"], report);
        if let Some(contents) = seq("bundle", bundle, "contents", report) {
            contains_all(
                "bundle の contents",
                &strings(contents),
                &BUNDLE_CONTENTS,
                report,
            );
        }
    }

    // 既知の集合が読めない件は便 4 の検査が「まだ分からない」に数えてある
    let known = vocab::known_words(vocabulary, &mut Report::default());
    for (lw, at) in vocab::unknown_words(&body, &known) {
        report.violation(
            "ceiling",
            format!("{FILE} {at}: 語彙に無い英字の語「{lw}」"),
        );
    }
}

/// 生成区間 schema を床の木 `FLOOR` と突き合わせる（便 47 §1 (c)）。欄の表でなければ「まだ分からない」。食い違いの道 1 本につき
/// 違反 1 件（文言は判断の記録・設計ノートの側と同じ型）。一覧の要素のずれ（`path[i]`）はその一覧 1 本の道に寄せる＝旧 4 節の
/// 同じ一覧の検査（`same_list`）と同じく、順の入れ替えを 1 件で数える。
fn check_schema(schema: &Node, report: &mut Report) {
    if schema.as_map().is_none() {
        report.unknown(format!("{FILE}: schema が欄の表でない"));
        return;
    }
    let mut drift = Vec::new();
    floor_diff(&strip_notes(schema), &FLOOR, "", &mut drift);
    let mut fields: Vec<String> = Vec::new();
    for path in drift {
        let field = match path.rfind('[') {
            Some(i) if path.ends_with(']') => path[..i].to_string(),
            _ => path,
        };
        if !fields.contains(&field) {
            fields.push(field);
        }
    }
    for field in fields {
        report.violation("ceiling", format!("{FILE}: 床の定数と違う: schema.{field}"));
    }
}

/// 旧 4 節（`LEGACY_SECTIONS`）。生成区間 schema が在るとき（`generated`）は無くてもよく、在れば今までどおり数える。
/// 無いときは今までどおり必須（`section` が空の表として扱い、欄の非空が数える）。
fn legacy<'a>(
    root: &'a Node,
    name: &str,
    generated: bool,
    report: &mut Report,
) -> Option<&'a Node> {
    if generated && root.get(name).is_none() {
        return None;
    }
    section(root, name, report)
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

/// 行の一覧の節。無い・null は 0 行、読めない形は None（便 0 の `rows` が「まだ分からない」に数える）。
fn table<'a>(root: &'a Node, section: &str, report: &mut Report) -> Option<Vec<&'a Node>> {
    let before = report.unknowns.len();
    let list = rows(FILE, root, section, report);
    (report.unknowns.len() == before).then_some(list)
}

/// 観点の reads（表の一覧）。読めない形は「まだ分からない」で 0 行。
fn reads<'a>(at: &str, row: &'a Node, report: &mut Report) -> Vec<&'a Node> {
    match row.get("reads") {
        None | Some(Node::Null) => Vec::new(),
        Some(Node::Seq(items)) if items.iter().all(|n| n.as_map().is_some()) => {
            items.iter().collect()
        }
        Some(_) => {
            report.unknown(format!("{FILE}: {at} の reads が表の一覧でない"));
            Vec::new()
        }
    }
}

/// 一覧の欄。無い・null は 0 個、一覧でなければ「まだ分からない」で None。
fn seq<'a>(at: &str, row: &'a Node, field: &str, report: &mut Report) -> Option<&'a [Node]> {
    match row.get(field) {
        None | Some(Node::Null) => Some(&[]),
        Some(Node::Seq(items)) => Some(items),
        Some(_) => {
            report.unknown(format!("{FILE}: {at} の {field} が一覧でない"));
            None
        }
    }
}

/// 一覧の文字列の要素（文字列でない要素は数えない）。
fn strings(items: &[Node]) -> Vec<String> {
    items
        .iter()
        .filter_map(Node::as_str)
        .map(str::to_string)
        .collect()
}

/// 行の一覧の id の列（id の無い行は「?」のまま・重複の関数と同じ見え方）。
fn ids(rows: &[&Node]) -> Vec<String> {
    rows.iter().map(|row| row_id(row)).collect()
}

/// 行の一覧の id の集合（id の無い行は入れない）。
fn id_set(rows: &[&Node]) -> HashSet<String> {
    ids(rows).into_iter().filter(|id| id != "?").collect()
}

/// 文を持つ欄を語彙の母集団に足す（文字列でない欄は数えない）。
fn push_text(body: &mut vocab::Body, at: &str, row: &Node, fields: &[&str]) {
    for field in fields {
        if let Some(s) = row.get(field).and_then(Node::as_str) {
            body.push((format!("{at} の {field}"), s.to_string()));
        }
    }
}

/// 固定の一覧との違いを 1 件で数える。空の一覧は欄の非空が数える（行の一覧は 0 行も違い）。
fn list_differs(
    at: &str,
    actual: &[String],
    missing: Vec<&str>,
    extra: Vec<&str>,
    report: &mut Report,
) {
    let mut msg = format!(
        "{FILE}: {at}: 一覧「{}」が床の定数と違う",
        actual.join(", ")
    );
    if !missing.is_empty() {
        msg.push_str(&format!("（無い: {}）", missing.join(", ")));
    }
    if !extra.is_empty() {
        msg.push_str(&format!("（余分: {}）", extra.join(", ")));
    }
    report.violation("ceiling", msg);
}

/// 順まで同じか。
fn same_list(at: &str, actual: &[String], expected: &[&str], report: &mut Report) {
    if actual
        .iter()
        .map(String::as_str)
        .eq(expected.iter().copied())
    {
        return;
    }
    let missing = expected
        .iter()
        .copied()
        .filter(|e| !actual.iter().any(|a| a == e))
        .collect();
    let extra = actual
        .iter()
        .map(String::as_str)
        .filter(|a| !expected.contains(a))
        .collect();
    list_differs(at, actual, missing, extra, report);
}

/// 集合として同じか（順は見ない・重複は行 id の重複が数える）。
fn same_set(at: &str, actual: &[String], expected: &[&str], report: &mut Report) {
    let missing: Vec<&str> = expected
        .iter()
        .copied()
        .filter(|e| !actual.iter().any(|a| a == e))
        .collect();
    let extra: Vec<&str> = actual
        .iter()
        .map(String::as_str)
        .filter(|a| !expected.contains(a))
        .collect();
    if missing.is_empty() && extra.is_empty() {
        return;
    }
    list_differs(at, actual, missing, extra, report);
}

/// 床の定数を全部含むか（余分は数えない）。空の一覧は欄の非空が数える。
fn contains_all(at: &str, actual: &[String], expected: &[&str], report: &mut Report) {
    let missing: Vec<&str> = expected
        .iter()
        .copied()
        .filter(|e| !actual.iter().any(|a| a == e))
        .collect();
    if actual.is_empty() || missing.is_empty() {
        return;
    }
    list_differs(at, actual, missing, Vec::new(), report);
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
        "ceiling",
        format!("{FILE}: {at} の {field}: 行き先「{shown}」が一覧に無い"),
    );
}

/// 一覧の欄の各要素が `targets` に解けるか。
fn resolve_each(at: &str, row: &Node, field: &str, targets: &HashSet<String>, report: &mut Report) {
    for (i, item) in seq(at, row, field, report)
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        let shown = match item {
            Node::Null => continue,
            Node::Scalar(s) if s.trim().is_empty() || targets.contains(s) => continue,
            Node::Scalar(s) => s.clone(),
            Node::Seq(_) => "（一覧）".to_string(),
            Node::Map(_) => "（表）".to_string(),
        };
        report.violation(
            "ceiling",
            format!("{FILE}: {at} の {field}[{i}]: 行き先「{shown}」が一覧に無い"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::FILES;

    /// 床の木の導出は凍結 anchor（設計判断の席が独立の実装で組んだ・P-10.1）と byte 一致（便 47 §1 (e)1）。
    #[test]
    fn ceiling_floor_derives_the_frozen_anchor_byte_for_byte() {
        let anchor = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/schema/ceiling-region.txt"
        ))
        .unwrap();
        assert_eq!(crate::schema::derive(&FLOOR), anchor);
    }

    /// 床の突き合わせは FLOOR の注（`_note`）を読まない＝注を持つ FLOOR と注の無い写しの差は 0。
    #[test]
    fn ceiling_floor_notes_are_outside_the_diff() {
        let Floor::Map(fields) = &FLOOR else {
            panic!("FLOOR は表");
        };
        let notes = fields.iter().filter(|(k, _)| k.ends_with("_note")).count();
        assert_eq!(notes, 8);
        let mut out = Vec::new();
        floor_diff(&strip_notes(&Node::Map(Vec::new())), &FLOOR, "", &mut out);
        assert_eq!(out.len(), fields.len() - notes, "{out:?}");
        assert!(out.iter().all(|p| p.ends_with("（欠落）")), "{out:?}");
    }

    #[test]
    fn ceiling_document_ids_are_the_seven_files_plus_adr_and_design_note() {
        let mut expected: Vec<&str> = FILES.to_vec();
        expected.extend(["adr", "design-note"]);
        assert_eq!(DOCUMENT_IDS.to_vec(), expected);
    }

    #[test]
    fn ceiling_fixed_lists_count_order_set_and_containment_differently() {
        let actual = |s: &[&str]| s.iter().map(|w| w.to_string()).collect::<Vec<_>>();
        let mut report = Report::default();
        same_list("a", &actual(&["y", "x"]), &["x", "y"], &mut report);
        same_set("b", &actual(&["y", "x"]), &["x", "y"], &mut report);
        contains_all("c", &actual(&["x", "z"]), &["x", "y"], &mut report);
        contains_all("d", &actual(&["y", "x", "z"]), &["x", "y"], &mut report);
        assert_eq!(report.violations.len(), 2, "{:?}", report.violations);
        assert_eq!(
            report.violations[0].1,
            "ceiling.yaml: a: 一覧「y, x」が床の定数と違う"
        );
        assert_eq!(
            report.violations[1].1,
            "ceiling.yaml: c: 一覧「x, z」が床の定数と違う（無い: y）"
        );
    }
}
