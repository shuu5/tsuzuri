//! `folio check` の天井の正本（`ceiling.yaml`・ADR-8 決定 (1)(3)(4)）の形の検査（便 37・docs/design/delivery-37.md §1 (b)）。
//! 数えるのは 未知の節・欄の非空・行 id の重複（便 0 と同じ関数・同じ種別・同じ文言の形）と、
//! 固定の一覧（観点の列・文書の集合）・生成区間 schema と床の木の突き合わせ・行き先の解決・天井の正本の文の英字の語（種別 ceiling）。
//! 英字の語の切り出し・既知の集合・免除は便 4 の `vocab.rs` を共有し、rules 行 R-9 の母集団（憲法・rules・要件書）には入れない
//! （相談窓口の正本を種別 intake で数える `intake.rs` と同じ持ち方）。fields の値と schema の欄の名は機械の層なので語彙に通さない。
//! 節が欄の表でない・行の一覧が表の一覧でない・values や fields が一覧でない は「まだ分からない」。
//! documents の note・meta の generated・approval は数えない。材料の束を組む命令は `bundle.rs`、所見 file の検査は `findings.rs`。
//!
//! 天井の床の定数は全部この 1 枚に置く（便 47・docs/design/delivery-47.md §1・ADR-11 決定 (4)①・P-5.1 / P-5.6）。床の木 `FLOOR` は
//! 天井の正本の最上位の節 `schema`（生成区間）の正本で、葉は下の定数と同じ配列を指す（同じ一覧を 2 回書かない）。`_note` で終わる
//! 欄は人が読む説明の注で、凍結 anchor tests/fixtures/schema/ceiling-region.txt の順と字面のまま持つ（`schema.rs` の `derive` が
//! 書き、`floor_diff` は読まない）。束を組む命令（`bundle.rs`）と所見の検査（`findings.rs`）の読み手も一覧をここから取る。
//! 節 `schema` は必須で `FLOOR` と突き合わせる（便 48・docs/design/delivery-48.md §1 (b)）。第 2 版までの人が書いた 4 節
//! （verdicts・finding・record・bundle）の受け口は無く、その名の節は未知の節で落ちる。
//! 束の読み手も読む所見と束の閉じた一覧 8 本は便 110 で `ceiling_src.rs` へ降ろした（ADR-15・層 1 読む・床の木の葉は同じ配列）。

use std::collections::HashSet;

use crate::ceiling_src::{
    BUNDLE_DIGEST, BUNDLE_SKELETON, FINDING_OPTIONAL, FINDING_REQUIRED, PLACE_REQUIRED,
    RECORD_REQUIRED, REFUTE_VALUES, VERDICT_VALUES,
};
use crate::check::{duplicate_ids, non_empty, row_id, rows, unknown_sections};
use crate::floor::{Floor, floor_diff, strip_notes};
use crate::floor_adr::{ANCHOR_ARTICLE_FIELDS, ANCHOR_STATEMENT_FIELDS, EFFECTIVE_STATUS};
use crate::verdict::Report;
use crate::vocab;
use crate::yaml::Node;

const FILE: &str = "ceiling.yaml";

/// 天井の正本の最上位の節の閉じた一覧（`FLOOR` の top_level・人が書く 4 節 + 生成区間 schema・ほかの名は未知の節）。
pub const CEILING_TOP_LEVEL: [&str; 5] = ["meta", "weights", "documents", "viewpoints", "schema"];

/// 観点の id の列（ADR-8 決定 (1)・順も固定・増減はどちらも違反）。
pub const VIEWPOINT_IDS: [&str; 4] = ["fidelity", "readability", "coherence", "reality"];

/// 読む文書の id の集合 = 正本 7 file + 判断の記録 + 設計ノート + 索引の欄の決まり（増減はどちらも違反・便 102 で graph）。
pub const DOCUMENT_IDS: [&str; 10] = [
    "constitution",
    "rules",
    "vocabulary",
    "srs",
    "index",
    "intake",
    "ceiling",
    "adr",
    "design-note",
    "graph",
];

/// 材料の束の中身（順も同じ・`bundle.rs` が組む）。
pub const BUNDLE_CONTENTS: [&str; 5] = ["sources", "faces", "question", "finding", "reads"];

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

// ── 周の引き金の閉じた一覧（規範の欄・便 126・docs/design/delivery-126.md §1 (b)・ADR-18 決定 (1)）──
// 文書の id ごとに、節の名と各行から取る欄（点は入れ子の欄）。whole は節を丸ごと。引き金の要約値（`gate.rs` の
// `trigger_digest`）はこの定数だけを読み、床の木の trigger の葉も同じ配列を指す。

/// 行の一覧の節（節の名・各行から取る欄）。
pub(crate) type TriggerRows = [(&'static str, &'static [&'static str])];

/// 憲法: 凍結 anchor の写しと同じ条の欄と規範文の欄（判断の記録の床の定数の配列そのもの）。
pub(crate) const TRIGGER_CONSTITUTION_ARTICLES: &[&str] = ANCHOR_ARTICLE_FIELDS;
pub(crate) const TRIGGER_CONSTITUTION_STATEMENTS: &[&str] = ANCHOR_STATEMENT_FIELDS;
/// 判断の記録: status の値がこのどれかの記録（発効の値域）ごとに fields。
pub(crate) const TRIGGER_ADR_STATUS: &[&str] = EFFECTIVE_STATUS;
pub(crate) const TRIGGER_ADR_FIELDS: [&str; 8] = [
    "id",
    "status",
    "decision",
    "retreat",
    "amends",
    "revises",
    "supersedes",
    "superseded_by",
];
/// 要件書: 要件と非機能要件の行の欄・受入基準の行の欄・制約の行の欄・丸ごと取る節。
const TRIGGER_SRS_REQUIREMENT: [&str; 7] = [
    "id",
    "shall",
    "strength",
    "pattern",
    "when",
    "verify",
    "milestone",
];
const TRIGGER_SRS_ACCEPTANCE: [&str; 4] = ["id", "title", "verifies", "red_test.sentence"];
const TRIGGER_SRS_CONSTRAINT: [&str; 2] = ["id", "text"];
pub(crate) const TRIGGER_SRS_ROWS: [(&str, &[&str]); 4] = [
    ("requirements", &TRIGGER_SRS_REQUIREMENT),
    ("nonfunctional", &TRIGGER_SRS_REQUIREMENT),
    ("acceptance", &TRIGGER_SRS_ACCEPTANCE),
    ("constraints", &TRIGGER_SRS_CONSTRAINT),
];
pub(crate) const TRIGGER_SRS_WHOLE: [&str; 4] = ["goals", "scope", "scope_m1", "scope_m3"];
/// 規則の表: sections の各行の fields（注・裁定・来歴〔note・ruling・ruled_at・refs・basis〕を除く中身の欄）。
pub(crate) const TRIGGER_RULES_SECTIONS: [&str; 2] = ["thresholds", "discipline"];
pub(crate) const TRIGGER_RULES_FIELDS: [&str; 10] = [
    "id",
    "article",
    "what",
    "value",
    "kind",
    "status",
    "stage",
    "population",
    "same_failure",
    "projection",
];
/// 天井の正本: 重さの節を丸ごと・文書の行の id と file・観点の行の読み手と問いと読む欄（観点の名は数えない）。
pub(crate) const TRIGGER_CEILING_WHOLE: [&str; 1] = ["weights"];
const TRIGGER_CEILING_DOCUMENT: [&str; 2] = ["id", "file"];
const TRIGGER_CEILING_VIEWPOINT: [&str; 4] = ["id", "reader", "question", "reads"];
pub(crate) const TRIGGER_CEILING_ROWS: [(&str, &[&str]); 2] = [
    ("documents", &TRIGGER_CEILING_DOCUMENT),
    ("viewpoints", &TRIGGER_CEILING_VIEWPOINT),
];

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
            ("skeleton", Floor::Strs(&BUNDLE_SKELETON)),
        ]),
    ),
    (
        "bundle_note",
        Floor::Val(
            "材料の束の中身（観点ごとに 1 つの置き場）と要約値の規則。要約値は束の file を path の byte 順に並べ、中身を連結した sha256（rules 行 R-15 の写しの要約値と同じ規則）。skeleton は sources の写しで常に残す最上位の節の閉じた一覧（観点が読むと宣言した欄に関わらず残す＝どの版の何を読んでいるかを決める欄）",
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
    (
        "trigger",
        Floor::Map(&[
            (
                "constitution",
                Floor::Map(&[
                    ("articles", Floor::Strs(TRIGGER_CONSTITUTION_ARTICLES)),
                    ("statements", Floor::Strs(TRIGGER_CONSTITUTION_STATEMENTS)),
                ]),
            ),
            (
                "adr",
                Floor::Map(&[
                    ("status", Floor::Strs(TRIGGER_ADR_STATUS)),
                    ("fields", Floor::Strs(&TRIGGER_ADR_FIELDS)),
                ]),
            ),
            (
                "srs",
                Floor::Map(&[
                    (TRIGGER_SRS_ROWS[0].0, Floor::Strs(TRIGGER_SRS_ROWS[0].1)),
                    (TRIGGER_SRS_ROWS[1].0, Floor::Strs(TRIGGER_SRS_ROWS[1].1)),
                    (TRIGGER_SRS_ROWS[2].0, Floor::Strs(TRIGGER_SRS_ROWS[2].1)),
                    (TRIGGER_SRS_ROWS[3].0, Floor::Strs(TRIGGER_SRS_ROWS[3].1)),
                    ("whole", Floor::Strs(&TRIGGER_SRS_WHOLE)),
                ]),
            ),
            (
                "rules",
                Floor::Map(&[
                    ("sections", Floor::Strs(&TRIGGER_RULES_SECTIONS)),
                    ("fields", Floor::Strs(&TRIGGER_RULES_FIELDS)),
                ]),
            ),
            (
                "ceiling",
                Floor::Map(&[
                    ("whole", Floor::Strs(&TRIGGER_CEILING_WHOLE)),
                    (TRIGGER_CEILING_ROWS[0].0, Floor::Strs(TRIGGER_CEILING_ROWS[0].1)),
                    (TRIGGER_CEILING_ROWS[1].0, Floor::Strs(TRIGGER_CEILING_ROWS[1].1)),
                ]),
            ),
        ]),
    ),
    (
        "trigger_note",
        Floor::Val(
            "周の引き金の閉じた一覧（規範の欄）。文書の id ごとに、節の名と各行から取る欄（点は入れ子の欄）。whole は節を丸ごと、adr は status の値の判断の記録ごとに fields、rules は sections の各行の fields、constitution は凍結 anchor の写しと同じ条の欄と規範文の欄。この写しを決まった順に並べた要約値が引き金の要約値で、印と門が同じ関数で測る。何にするかの裁定の正本は判断の記録 ADR-18 決定 (1)",
        ),
    ),
]);

/// 天井の正本 `ceiling` の形を数える。`vocabulary` は既知の語の集合にだけ使う。
pub fn check_ceiling(ceiling: &Node, vocabulary: &Node, report: &mut Report) {
    unknown_sections(FILE, ceiling, &CEILING_TOP_LEVEL, report);
    // 決まりの部分は生成区間 schema が持つ（必須）。床の木と突き合わせる
    match ceiling.get("schema") {
        Some(schema) => check_schema(schema, report),
        None => report.violation(
            "ceiling",
            format!(
                "{FILE}: schema の節が無い（決まりの部分の生成区間・folio schema --write が書く）"
            ),
        ),
    }
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
/// 違反 1 件（文言は判断の記録・設計ノートの側と同じ型）。一覧の要素のずれ（`path[i]`）はその一覧 1 本の道に寄せる＝観点の id の
/// 列の検査（`same_list`）と同じく、順の入れ替えを 1 件で数える。
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
        assert_eq!(crate::floor::derive(&FLOOR), anchor);
    }

    /// 床の突き合わせは FLOOR の注（`_note`）を読まない＝注を持つ FLOOR と注の無い写しの差は 0。
    #[test]
    fn ceiling_floor_notes_are_outside_the_diff() {
        let Floor::Map(fields) = &FLOOR else {
            panic!("FLOOR は表");
        };
        let notes = fields.iter().filter(|(k, _)| k.ends_with("_note")).count();
        assert_eq!(notes, 9);
        let mut out = Vec::new();
        floor_diff(&strip_notes(&Node::Map(Vec::new())), &FLOOR, "", &mut out);
        assert_eq!(out.len(), fields.len() - notes, "{out:?}");
        assert!(out.iter().all(|p| p.ends_with("（欠落）")), "{out:?}");
    }

    /// 引き金の一覧の名はどれも実在の文書と節を指し、範囲の節・凍結 anchor の写しの配列・発効の値域を取りこぼさない
    /// （便 126 §1 (e) の 9）。各行から取る欄の名の実在は凍結 anchor との byte 一致が縛る。
    #[test]
    fn f126_trigger_lists_name_real_documents_and_sections() {
        let Floor::Map(fields) = &FLOOR else {
            panic!("FLOOR は表");
        };
        let Some((_, Floor::Map(docs))) = fields.iter().find(|(k, _)| *k == "trigger") else {
            panic!("FLOOR に trigger の表が無い");
        };
        for (doc, _) in docs.iter() {
            assert!(DOCUMENT_IDS.contains(doc), "{doc}: 読む文書の id に無い");
        }
        let srs: Vec<&str> = TRIGGER_SRS_ROWS
            .iter()
            .map(|(s, _)| *s)
            .chain(TRIGGER_SRS_WHOLE)
            .collect();
        for section in &srs {
            assert!(crate::check::SRS_TOP_LEVEL.contains(section), "{section}: 要件書の節に無い");
        }
        for section in crate::check::SRS_TOP_LEVEL {
            if section == "goals" || section == "scope" || section.starts_with("scope_") {
                assert!(TRIGGER_SRS_WHOLE.contains(&section), "{section}: 目的と範囲の節を取りこぼす");
            }
        }
        for section in TRIGGER_RULES_SECTIONS {
            assert!(crate::rules::RULES_TOP_LEVEL.contains(&section), "{section}: 規則の表の節に無い");
        }
        for section in TRIGGER_CEILING_ROWS.iter().map(|(s, _)| *s).chain(TRIGGER_CEILING_WHOLE) {
            assert!(CEILING_TOP_LEVEL.contains(&section), "{section}: 天井の正本の節に無い");
        }
        assert_eq!(TRIGGER_ADR_STATUS, crate::adr::floor_strs(&["effective_status"]));
        assert_eq!(
            TRIGGER_CONSTITUTION_ARTICLES,
            crate::adr::floor_strs(&["anchor", "projection_article_fields"])
        );
        assert_eq!(
            TRIGGER_CONSTITUTION_STATEMENTS,
            crate::adr::floor_strs(&["anchor", "statement_fields"])
        );
    }

    #[test]
    fn ceiling_document_ids_are_the_seven_files_plus_adr_design_note_and_graph() {
        let mut expected: Vec<&str> = FILES.to_vec();
        expected.extend(["adr", "design-note", "graph"]);
        assert_eq!(DOCUMENT_IDS.to_vec(), expected);
    }

    /// 骨格の 6 語は、実の正本（天井の正本の documents の行が指す file・dir 形は直下の .yaml）の最上位の節
    /// （列 0 の `<名>:` の行）として 1 語につき 1 本以上の file に実在する（便 102 §1 (g)4・数は固定しない）。
    #[test]
    fn f102_every_skeleton_word_is_a_top_level_section_of_a_real_source() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../design-intent");
        let text = std::fs::read_to_string(dir.join(FILE)).unwrap();
        let ceiling = crate::yaml::parse(&text).unwrap().root;
        let mut paths = Vec::new();
        for row in rows(FILE, &ceiling, "documents", &mut Report::default()) {
            let file = row.get("file").and_then(Node::as_str).unwrap();
            let path = dir.join(file);
            if file.ends_with('/') {
                for entry in std::fs::read_dir(&path).unwrap() {
                    let p = entry.unwrap().path();
                    if p.extension().is_some_and(|e| e == "yaml") {
                        paths.push(p);
                    }
                }
            } else {
                paths.push(path);
            }
        }
        assert!(paths.len() >= DOCUMENT_IDS.len(), "{paths:?}");
        for word in BUNDLE_SKELETON {
            let head = format!("{word}:");
            let hits = paths
                .iter()
                .filter(|p| {
                    std::fs::read_to_string(p).unwrap().lines().any(|l| {
                        l.strip_prefix(&head)
                            .is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', '\t']))
                    })
                })
                .count();
            assert!(hits >= 1, "骨格の語「{word}」を最上位の節に持つ正本が無い");
        }
    }

    #[test]
    fn ceiling_fixed_lists_count_order_and_set_differently() {
        let actual = |s: &[&str]| s.iter().map(|w| w.to_string()).collect::<Vec<_>>();
        let mut report = Report::default();
        same_list("a", &actual(&["y", "x"]), &["x", "y"], &mut report);
        same_set("b", &actual(&["y", "x"]), &["x", "y"], &mut report);
        same_set("c", &actual(&["x", "z"]), &["x", "y"], &mut report);
        assert_eq!(report.violations.len(), 2, "{:?}", report.violations);
        assert_eq!(
            report.violations[0].1,
            "ceiling.yaml: a: 一覧「y, x」が床の定数と違う"
        );
        assert_eq!(
            report.violations[1].1,
            "ceiling.yaml: c: 一覧「x, z」が床の定数と違う（無い: y）（余分: z）"
        );
    }

    /// 最上位の節は `FLOOR` の top_level だけ = 第 2 版までの 4 節の名は未知の節で落ちる（便 48 §1 (b)）。
    #[test]
    fn ceiling_top_level_has_no_room_for_the_former_sections() {
        for former in ["verdicts", "finding", "record", "bundle"] {
            assert!(!CEILING_TOP_LEVEL.contains(&former), "{former}");
        }
    }
}
