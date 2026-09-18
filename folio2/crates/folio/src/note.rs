//! `folio check` の設計ノートの正本（`design-note/*.yaml`・ADR-3 決定 (1)・要件書 FR9 / FR10）の形の検査
//! （便 23・docs/design/delivery-23.md §1）。
//! 数えるのは 欄の決まり（同じ dir の `schema.yaml`）の schema 節が定める形（文書と meta の欄・節の番号と型・
//! 型ごとの行の欄と値域・承認欄の要否）と、契約表の節の欄（器 scribe2 の導出 file `contracts/schema.toml` から読む・
//! 欄の一覧も値域も自分の型にも散文にも持たない・FR10）と、参照 id の解決（folio2 が所有する id 空間）である。
//! 散文の門（FR12・rules 行 R-16）と導出物（FR11）は本便に入れない。
//! 欄の決まりの閾値・値域・置き場は床の定数（`FLOOR`）で持ち、`design-note/schema.yaml` の schema 節はその写し
//! （判断の記録の欄の決まり `adr.rs` と同じ作り・N-3.1）。パターンの文字列は定数として字面で持つだけで、
//! 形の判定は字の走査で行う（正規表現は使わない）。器の導出 file（TOML）も行走査で読む（外部 crate を足さない）。

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::adr::Adr;
use crate::check::{duplicate_ids, non_empty, row_id, unknown_sections};
use crate::link;
use crate::parts::catalog::FigureType;
use crate::prose;
use crate::refs;
use crate::verdict::Report;
use crate::yaml::{self, Node};

/// 設計ノートの置き場（正本の dir の直下）と欄の決まりの file。
const DIR: &str = "design-note";
const SCHEMA_FILE: &str = "design-note/schema.yaml";

/// 違反の種別（設計ノートの形・散文の門）。
const KIND: &str = "note";
const PROSE_GATE: &str = "prose-gate";

// ── 床の定数（design-note/schema.yaml の schema 節の写し） ──

/// 床の定数の木（`adr.rs` の `Floor` と同じ形・一覧の中に表が並ぶ欄のため `Seq` を持つ）。
#[derive(Debug)]
enum Floor {
    /// 値（yaml の値の字面・引用符を除く）
    Val(&'static str),
    /// 数（字面で比べる＝2.0 と 2 は違う）
    Num(usize),
    /// 値の一覧
    Strs(&'static [&'static str]),
    /// 木の一覧（順も比べる）
    Seq(&'static [Floor]),
    /// 表（欄の順は schema 節の順）
    Map(&'static [(&'static str, Floor)]),
}

/// 欄の集合（required / optional）。
struct Keys {
    required: &'static [&'static str],
    optional: &'static [&'static str],
}

impl Keys {
    fn has(&self, key: &str) -> bool {
        self.required.contains(&key) || self.optional.contains(&key)
    }

    fn all(&self) -> Vec<&'static str> {
        self.required
            .iter()
            .chain(self.optional.iter())
            .copied()
            .collect()
    }
}

const PATH_BASE: &str = "repo-root";
const DATE_FORMAT: &str = r"^\d{4}-\d{2}-\d{2}$";
const DOC: Keys = Keys {
    required: &["meta", "sections"],
    optional: &["figures", "sources"],
};
const DOC_META: Keys = Keys {
    required: &["id", "title", "version", "status", "generated", "profile"],
    optional: &["approval", "supersedes", "superseded_by", "note"],
};
const ID_PATTERN: &str = "^[a-z][a-z0-9-]*$";
const VERSION_PATTERN: &str = r"^v[0-9]+\.[0-9]+$";
const STATUS_ENUM: &[&str] = &["draft", "effective", "retired", "example"];
const EFFECTIVE_STATUS: &[&str] = &["effective", "retired"];
const STATUS_EXAMPLE: &str = "example";
const STATUS_RETIRED: &str = "retired";
const PROFILE_ENUM: &[&str] = &["design-note"];
const APPROVAL_REQUIRED: &[&str] = &["who", "date", "ruling", "verbatim", "surface"];
const SURFACE_ENUM: &[&str] = &["R-8"];
const SECTION: Keys = Keys {
    required: &["n", "type", "title"],
    optional: &["body", "rows", "note"],
};
const TYPE_ENUM: &[&str] = &[
    "prose",
    "parts-table",
    "ports-table",
    "fields-table",
    "teeth-table",
    "contract-table",
];
const PROSE: &str = "prose";
const CONTRACT_TABLE: &str = "contract-table";
/// 節の型ごとの required / forbid（by_type）。
const NEEDS_BODY: &[&str] = &["body"];
const NEEDS_ROWS: &[&str] = &["rows"];
const FORBIDS_ROWS: &[&str] = &["rows"];
const PARTS_ROW: Keys = Keys {
    required: &["id", "name", "role"],
    optional: &["ref", "note"],
};
const PORTS_ROW: Keys = Keys {
    required: &["id", "name", "input", "output", "refuses"],
    optional: &["ref", "note"],
};
const REFUSES_NONE_MARKER: &str = "なし";
const FIELDS_ROW: Keys = Keys {
    required: &["id", "name", "need", "shape"],
    optional: &["enum", "note"],
};
const NEED_ENUM: &[&str] = &["required", "optional"];
const SHAPE_ENUM: &[&str] = &["text", "list", "number", "bool", "table"];
const TEETH_ROW: Keys = Keys {
    required: &["id", "name", "red_when", "fixture"],
    optional: &["ref", "note"],
};
/// 器（scribe2）の導出 file の置き場（repo の根からの相対）と読み手の期待する形。
const EXTERNAL_PATH: &str = "contracts/schema.toml";
const EXTERNAL_HEAD: &str = "schema = 1";
const EXTERNAL_ROWS_KEY: &str = "field";
const EXTERNAL_ROW_FIELDS: &[&str] = &["name", "need", "shape"];
/// 導出 file の need / shape の値域（読めない値は「まだ分からない」）。
const EXTERNAL_NEED: &[&str] = &["required", "optional"];
const EXTERNAL_SHAPE: &[&str] = &["text", "list"];
const ROW_ID_PATTERN: &str = "^[a-z][a-z0-9-]*$";
const FIGURE_ENTRY: Keys = Keys {
    required: &["id", "type", "caption", "spec"],
    optional: &["refs", "note"],
};

macro_rules! keys_floor {
    ($keys:expr) => {
        Floor::Map(&[
            ("required", Floor::Strs($keys.required)),
            ("optional", Floor::Strs($keys.optional)),
        ])
    };
}

/// 床の定数（値は欄の決まり design-note/schema.yaml の schema 節の字面と 1 字も違わない）。
const FLOOR: Floor = Floor::Map(&[
    ("path_base", Floor::Val(PATH_BASE)),
    ("date_format", Floor::Val(DATE_FORMAT)),
    (
        "doc",
        Floor::Map(&[
            ("required", Floor::Strs(DOC.required)),
            ("optional", Floor::Strs(DOC.optional)),
            (
                "sources",
                Floor::Map(&[("in_ref_population", Floor::Val("false"))]),
            ),
        ]),
    ),
    (
        "doc_meta",
        Floor::Map(&[
            ("required", Floor::Strs(DOC_META.required)),
            ("optional", Floor::Strs(DOC_META.optional)),
            ("id_pattern", Floor::Val(ID_PATTERN)),
            ("version_pattern", Floor::Val(VERSION_PATTERN)),
            ("status_enum", Floor::Strs(STATUS_ENUM)),
            ("effective_status", Floor::Strs(EFFECTIVE_STATUS)),
            ("approval_required_when", Floor::Val("effective_status")),
            ("profile_enum", Floor::Strs(PROFILE_ENUM)),
            (
                "approval",
                Floor::Map(&[
                    ("required", Floor::Strs(APPROVAL_REQUIRED)),
                    ("surface_enum", Floor::Strs(SURFACE_ENUM)),
                ]),
            ),
        ]),
    ),
    (
        "section",
        Floor::Map(&[
            ("required", Floor::Strs(SECTION.required)),
            ("optional", Floor::Strs(SECTION.optional)),
            (
                "n_rule",
                Floor::Map(&[
                    ("start", Floor::Num(1)),
                    ("order", Floor::Val("ascending")),
                    ("append_only", Floor::Val("true")),
                    ("gaps_allowed", Floor::Val("true")),
                ]),
            ),
            ("type_enum", Floor::Strs(TYPE_ENUM)),
            (
                "by_type",
                Floor::Map(&[
                    (
                        "prose",
                        Floor::Map(&[
                            ("required", Floor::Strs(NEEDS_BODY)),
                            ("forbid", Floor::Strs(FORBIDS_ROWS)),
                            ("prose_gate_rules_row", Floor::Val("R-16")),
                        ]),
                    ),
                    (
                        "parts-table",
                        Floor::Map(&[
                            ("required", Floor::Strs(NEEDS_ROWS)),
                            ("row", keys_floor!(PARTS_ROW)),
                        ]),
                    ),
                    (
                        "ports-table",
                        Floor::Map(&[
                            ("required", Floor::Strs(NEEDS_ROWS)),
                            ("row", keys_floor!(PORTS_ROW)),
                            ("refuses_none_marker", Floor::Val(REFUSES_NONE_MARKER)),
                        ]),
                    ),
                    (
                        "fields-table",
                        Floor::Map(&[
                            ("required", Floor::Strs(NEEDS_ROWS)),
                            ("row", keys_floor!(FIELDS_ROW)),
                            ("need_enum", Floor::Strs(NEED_ENUM)),
                            ("shape_enum", Floor::Strs(SHAPE_ENUM)),
                        ]),
                    ),
                    (
                        "teeth-table",
                        Floor::Map(&[
                            ("required", Floor::Strs(NEEDS_ROWS)),
                            ("row", keys_floor!(TEETH_ROW)),
                        ]),
                    ),
                    (
                        "contract-table",
                        Floor::Map(&[
                            ("required", Floor::Strs(NEEDS_ROWS)),
                            ("section_ref_type", Floor::Val(PROSE)),
                        ]),
                    ),
                ]),
            ),
        ]),
    ),
    (
        "contract_table",
        Floor::Map(&[
            (
                "external_schema",
                Floor::Map(&[
                    ("owner", Floor::Val("scribe2")),
                    ("path", Floor::Val(EXTERNAL_PATH)),
                    ("format", Floor::Val("toml")),
                    (
                        "reader_expects",
                        Floor::Map(&[
                            ("head", Floor::Val(EXTERNAL_HEAD)),
                            ("rows_key", Floor::Val(EXTERNAL_ROWS_KEY)),
                            ("row_fields", Floor::Strs(EXTERNAL_ROW_FIELDS)),
                        ]),
                    ),
                    ("value_domains", Floor::Val("from-file")),
                    ("unknown_value", Floor::Val("まだ分からない")),
                ]),
            ),
            (
                "reads",
                Floor::Strs(&["design-doc-contract-table", "external-schema-file"]),
            ),
            ("never_reads", Floor::Strs(&["per-run-contract-file"])),
            (
                "row_id",
                Floor::Map(&[
                    ("pattern", Floor::Val(ROW_ID_PATTERN)),
                    ("owner", Floor::Val("folio2")),
                    ("scope", Floor::Val("own-id-space")),
                ]),
            ),
            ("semantic_check_owner", Floor::Val("scribe2")),
            (
                "folio_check",
                Floor::Strs(&["yaml-form", "derived-diff-zero", "own-id-space"]),
            ),
        ]),
    ),
    (
        "derived",
        Floor::Map(&[
            ("format", Floor::Val("toml-subset")),
            ("extension", Floor::Val(".toml")),
            ("granularity", Floor::Val("one-file-per-doc")),
            ("head", Floor::Val(EXTERNAL_HEAD)),
            ("array", Floor::Val("contract")),
            (
                "row_fields",
                Floor::Val("external_schema の field の name をそのまま + goal"),
            ),
            (
                "goal",
                Floor::Val(
                    "行の section が指す節の body の逐語を単一行に写す（各行を trim し空行を落とし空白 1 つで繋ぐ・引用符と逆斜線は escape しない）",
                ),
            ),
            ("section_value_shape", Floor::Val("text")),
            ("empty_list", Floor::Val("omit-key")),
            (
                "value_grammar",
                Floor::Strs(&["text", "list-of-text", "number", "bool"]),
            ),
            (
                "placement",
                Floor::Val(
                    "消費側（器 scribe2）の repo に版管理で置く。path は消費側が宣言する（拡張子 .toml・全文を同じ parser に渡す）",
                ),
            ),
            (
                "check",
                Floor::Map(&[
                    ("command", Floor::Val("folio build --check")),
                    ("verdict_on_diff", Floor::Val("nonzero")),
                    ("stage", Floor::Val("post")),
                ]),
            ),
        ]),
    ),
    (
        "landing",
        Floor::Map(&[
            ("source_of_truth", Floor::Val("scribe2 の記録（record）")),
            (
                "read_port",
                Floor::Val(
                    "統合先の commit（squash）の本文の末尾の印（trailer・契約 id と要件 id・器が書く）",
                ),
            ),
            (
                "trailer_name_source",
                Floor::Map(&[
                    ("owner", Floor::Val("scribe2")),
                    ("derived_from", Floor::Val("器の名前の定数")),
                    ("unreadable", Floor::Val("まだ分からない")),
                ]),
            ),
            (
                "verdict_values",
                Floor::Map(&[
                    ("used", Floor::Strs(&["着地", "まだ分からない"])),
                    ("never", Floor::Strs(&["未着地"])),
                ]),
            ),
            (
                "verdict_cases",
                Floor::Seq(&[
                    Floor::Map(&[
                        ("when", Floor::Val("印が在る")),
                        (
                            "verdict",
                            Floor::Val("着地（印の commit の要約値（sha）を添える）"),
                        ),
                    ]),
                    Floor::Map(&[
                        ("when", Floor::Val("印が無い")),
                        ("verdict", Floor::Val("まだ分からない")),
                    ]),
                    Floor::Map(&[
                        (
                            "when",
                            Floor::Val(
                                "自分の repo への取り込みの要求で終わる形（印も記録も持たない）",
                            ),
                        ),
                        ("verdict", Floor::Val("まだ分からない（恒久）")),
                    ]),
                    Floor::Map(&[
                        ("when", Floor::Val("印の名か commit が読めない")),
                        ("verdict", Floor::Val("まだ分からない")),
                    ]),
                ]),
            ),
            ("forbidden_wording", Floor::Val("未着地")),
        ]),
    ),
    (
        "index",
        Floor::Map(&[
            ("entries", Floor::Strs(&["doc", "requirement", "contract"])),
            ("entry_fields", Floor::Strs(&["id", "title"])),
        ]),
    ),
    (
        "figures",
        Floor::Map(&[
            (
                "spec",
                Floor::Val(
                    "図の道具（archify）の型付き記述（JSON の 5 型の欄の決まりそのまま・ADR-4 決定 (1)）",
                ),
            ),
            ("entry", keys_floor!(FIGURE_ENTRY)),
            (
                "type_enum_ref",
                Floor::Val("design-intent/preview/parts.json figure_type_enum"),
            ),
            (
                "body_classes_ref",
                Floor::Val("design-intent/preview/parts.json figure_body_classes"),
            ),
            ("body_classes_rules_row", Floor::Val("R-3")),
            ("semantic_attrs", Floor::Val("keep")),
            ("viewer_chrome", Floor::Val("discard")),
            ("quality_rules_row", Floor::Val("R-14")),
            ("tool_version_rules_row", Floor::Val("R-15")),
            ("network_commands", Floor::Val("forbid")),
            ("skill_listing", Floor::Val("forbid")),
            ("retry_rules_row", Floor::Val("R-7")),
            ("retry_record", Floor::Val("ledger")),
        ]),
    ),
    (
        "guards",
        Floor::Map(&[
            ("in_loop", Floor::Strs(&[])),
            (
                "post",
                Floor::Strs(&[
                    "yaml-form",
                    "derived-diff-zero",
                    "own-id-space",
                    "prose-gate",
                ]),
            ),
            ("polarity_list_feed", Floor::Val("true")),
            ("p18_4_judged_by", Floor::Val("R-13")),
        ]),
    ),
]);

// ── 入口 ──

/// 読めた設計ノート 1 本（file 名・id（file 名の stem）・木）。
struct NoteDoc {
    file: String,
    id: String,
    root: Node,
}

/// 器（scribe2）の導出 file の 1 欄。
struct Field {
    name: String,
    need: String,
    shape: String,
}

/// (a) `<dir>/design-note/` の欄の決まりの写しと設計ノートを検査する。
/// dir が無い = 設計ノート 0 本（違反でも「まだ分からない」でもない）。
pub fn check_note(
    dir: &Path,
    constitution: &Node,
    rules: &Node,
    srs: &Node,
    adr: Option<&Adr>,
    report: &mut Report,
) {
    let nd = dir.join(DIR);
    if !nd.exists() {
        return;
    }
    if nd.is_symlink() || !nd.is_dir() {
        report.unknown(format!(
            "{DIR}/ が dir でない（symlink・file）: {}",
            nd.display()
        ));
        return;
    }
    check_schema_copy(&nd, report);
    let notes = load_notes(&nd, report);
    if notes.is_empty() {
        return;
    }
    // 器の導出 file は契約表の節を持つ設計ノートが 1 本以上あるときだけ読む
    let external = if notes.iter().any(|n| has_contract_table(&n.root)) {
        load_external(dir, report)
    } else {
        None
    };
    // 散文の門の一覧は検査のたびに rules 行 R-16 の value から読む（R-16 の note・P-5.1）
    let gate = prose::gate(rules)
        .inspect_err(|e| report.unknown(format!("rules.yaml: R-16 の value が読めない: {e}")))
        .ok();
    let known = base_known_ids(constitution, rules, srs, adr);
    let requirements = requirement_ids(srs);
    let note_ids: HashSet<&str> = notes.iter().map(|n| n.id.as_str()).collect();
    for note in &notes {
        check_one(
            note,
            &note_ids,
            &known,
            &requirements,
            external.as_deref(),
            gate.as_ref(),
            report,
        );
    }
}

// ── (b) 欄の決まりの写し ──

/// 欄の決まりの file を読み、schema 節（`_note` で終わらない欄）が床の定数と 1 字も違わないことを確かめる。
fn check_schema_copy(nd: &Path, report: &mut Report) {
    let path = nd.join("schema.yaml");
    if path.is_symlink() {
        report.unknown(format!("{SCHEMA_FILE}: symlink は認めない"));
        return;
    }
    if !path.exists() {
        report.unknown(format!("{SCHEMA_FILE}: 欄の決まりが無い"));
        return;
    }
    if !path.is_file() {
        report.unknown(format!("{SCHEMA_FILE}: file でない"));
        return;
    }
    let doc = match read(&path) {
        Ok(d) => d,
        Err(e) => {
            report.unknown(format!("{SCHEMA_FILE}: 読めない: {e}"));
            return;
        }
    };
    if !doc.duplicates.is_empty() {
        for dup in &doc.duplicates {
            report.unknown(format!(
                "{SCHEMA_FILE} {} 行: 読めない（重複キー「{}」）",
                dup.line, dup.key
            ));
        }
        return;
    }
    let Some(schema @ Node::Map(_)) = doc.root.get("schema") else {
        report.unknown(format!(
            "{SCHEMA_FILE}: 形が違う（schema 節が欄の表でない）"
        ));
        return;
    };
    let mut drift = Vec::new();
    floor_diff(&strip_notes(schema), &FLOOR, "", &mut drift);
    for path in drift {
        report.violation(
            KIND,
            format!("{SCHEMA_FILE}: 床の定数と違う: schema.{path}"),
        );
    }
}

/// 名前が `_note` で終わる欄を（入れ子の表の中も含めて）落とす。
fn strip_notes(node: &Node) -> Node {
    match node {
        Node::Map(entries) => Node::Map(
            entries
                .iter()
                .filter(|(k, _)| !k.ends_with("_note"))
                .map(|(k, v)| (k.clone(), strip_notes(v)))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// 写しと床の定数の違いを欄の道で並べる（`adr.rs` の便 5 と同じ式）。
fn floor_diff(data: &Node, floor: &Floor, path: &str, out: &mut Vec<String>) {
    match floor {
        Floor::Map(fields) => {
            let Some(entries) = data.as_map() else {
                out.push(format!(
                    "{}（欄の表でない）",
                    if path.is_empty() { "schema" } else { path }
                ));
                return;
            };
            let mut keys: Vec<&str> = entries
                .iter()
                .map(|(k, _)| k.as_str())
                .chain(fields.iter().map(|(k, _)| *k))
                .collect();
            keys.sort_unstable();
            keys.dedup();
            for key in keys {
                let p = if path.is_empty() {
                    key.to_string()
                } else {
                    format!("{path}.{key}")
                };
                match (fields.iter().find(|(k, _)| *k == key), data.get(key)) {
                    (None, _) => out.push(format!(
                        "{p}（未知の欄＝機械が読まない欄は *_note で終える）"
                    )),
                    (Some(_), None) => out.push(format!("{p}（欠落）")),
                    (Some((_, f)), Some(d)) => floor_diff(d, f, &p, out),
                }
            }
        }
        Floor::Seq(items) => match data.as_seq() {
            Some(seq) if seq.len() == items.len() => {
                for (i, (d, f)) in seq.iter().zip(items.iter()).enumerate() {
                    floor_diff(d, f, &format!("{path}[{i}]"), out);
                }
            }
            _ => out.push(path.to_string()),
        },
        Floor::Strs(items) => match data.as_seq() {
            Some(seq) if seq.len() == items.len() => {
                for (i, (d, v)) in seq.iter().zip(items.iter()).enumerate() {
                    if d.as_str() != Some(v) {
                        out.push(format!("{path}[{i}]"));
                    }
                }
            }
            _ => out.push(path.to_string()),
        },
        Floor::Val(v) => {
            if data.as_str() != Some(v) {
                out.push(path.to_string());
            }
        }
        Floor::Num(n) => {
            if data.as_str() != Some(n.to_string().as_str()) {
                out.push(path.to_string());
            }
        }
    }
}

// ── (a) 正本の読み ──

fn read(path: &Path) -> Result<yaml::Doc, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    yaml::parse(&text)
}

/// dir の直下の `.yaml`（`schema.yaml` を除く）を名の昇順に読む。
/// symlink・読めない・parse できない・最上位が欄の表でない は「まだ分からない」（他の 6 file と同じ読み手）。
fn load_notes(nd: &Path, report: &mut Report) -> Vec<NoteDoc> {
    let mut names: Vec<String> = match fs::read_dir(nd) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".yaml") && n != "schema.yaml")
            .collect(),
        Err(e) => {
            report.unknown(format!("{DIR}/: 読めない: {e}"));
            return Vec::new();
        }
    };
    names.sort();
    let mut notes = Vec::new();
    for name in names {
        let path = nd.join(&name);
        let file = format!("{DIR}/{name}");
        if path.is_symlink() {
            report.unknown(format!("{file}: symlink は認めない"));
            continue;
        }
        if !path.is_file() {
            report.unknown(format!("{file}: file でない"));
            continue;
        }
        let doc = match read(&path) {
            Ok(d) => d,
            Err(e) => {
                report.unknown(format!("{file}: parse できない: {e}"));
                continue;
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
            continue;
        }
        notes.push(NoteDoc {
            id: name.trim_end_matches(".yaml").to_string(),
            file: name,
            root: doc.root,
        });
    }
    notes
}

/// 契約表の節を 1 つでも持つか。
fn has_contract_table(root: &Node) -> bool {
    root.get("sections")
        .and_then(Node::as_seq)
        .unwrap_or_default()
        .iter()
        .any(|s| s.get("type").and_then(Node::as_str) == Some(CONTRACT_TABLE))
}

// ── (a) 器の導出 file（行走査で読む） ──

/// `<dir>` の親 dir の `contracts/schema.toml` を読む。読めない・期待する形でない は「まだ分からない」。
fn load_external(dir: &Path, report: &mut Report) -> Option<Vec<Field>> {
    let mut unreadable = |why: String| -> Option<Vec<Field>> {
        report.unknown(format!("{EXTERNAL_PATH}: 器の導出 file が読めない: {why}"));
        None
    };
    let Some(parent) = dir.parent() else {
        return unreadable("正本の置き場の親 dir が無い".to_string());
    };
    let path = parent.join(EXTERNAL_PATH);
    if path.is_symlink() {
        return unreadable("symlink は認めない".to_string());
    }
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => return unreadable(e.to_string()),
    };
    let head = format!("[[{EXTERNAL_ROWS_KEY}]]");
    let mut rows: Vec<Vec<(String, String)>> = Vec::new();
    let mut seen_head = false;
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if !seen_head {
            if t != EXTERNAL_HEAD {
                return unreadable(format!("先頭が「{EXTERNAL_HEAD}」でない: {t}"));
            }
            seen_head = true;
            continue;
        }
        if t == head {
            rows.push(Vec::new());
            continue;
        }
        if let Some(pair) = quoted_pair(t)
            && let Some(row) = rows.last_mut()
        {
            row.push(pair);
        }
    }
    if !seen_head {
        return unreadable(format!("「{EXTERNAL_HEAD}」の行が無い"));
    }
    if rows.is_empty() {
        return unreadable(format!("{head} の行が無い"));
    }
    let mut fields = Vec::with_capacity(rows.len());
    for row in &rows {
        let value = |key: &str| {
            row.iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .filter(|v| !v.trim().is_empty())
        };
        let (Some(name), Some(need), Some(shape)) = (value("name"), value("need"), value("shape"))
        else {
            return unreadable(format!(
                "{head} の行に {} が揃わない",
                EXTERNAL_ROW_FIELDS.join("・")
            ));
        };
        if !EXTERNAL_NEED.contains(&need.as_str()) {
            return unreadable(format!("欄「{name}」の need「{need}」を知らない"));
        }
        if !EXTERNAL_SHAPE.contains(&shape.as_str()) {
            return unreadable(format!("欄「{name}」の shape「{shape}」を知らない"));
        }
        fields.push(Field { name, need, shape });
    }
    Some(fields)
}

/// `名 = "値"` の行（引用符で囲んだ値だけ・正規表現は使わない）。
fn quoted_pair(line: &str) -> Option<(String, String)> {
    let (key, rest) = line.split_once('=')?;
    let key = key.trim();
    let rest = rest.trim();
    let body = rest.strip_prefix('"')?.strip_suffix('"')?;
    if key.is_empty() || body.contains('"') {
        return None;
    }
    Some((key.to_string(), body.to_string()))
}

// ── id 空間 ──

/// rules 行の節（便 1 と同じ）。
const RULE_SECTIONS: [&str; 2] = ["thresholds", "discipline"];

/// 要件書の id を持つ節（便 1 と同じ）。
const SRS_ID_SECTIONS: [&str; 7] = [
    "goals",
    "requirements",
    "nonfunctional",
    "acceptance",
    "constraints",
    "actors",
    "outputs",
];

/// 節の行（表）。一覧でない節・表でない行は数えない（便 0・便 1 の側が数えてある）。
fn maps<'a>(root: &'a Node, section: &str) -> Vec<&'a Node> {
    root.get(section)
        .and_then(Node::as_seq)
        .unwrap_or_default()
        .iter()
        .filter(|n| n.as_map().is_some())
        .collect()
}

/// 既知の id = 条 id と規範文 id・rules 行 id・要件書の 7 節の id（便 1 の `refs.rs`）+ 判断の記録の id（便 6 の `link.rs`）。
/// 読めない形の申告は便 1・便 5 の検査が数えてあるので、ここでは捨てる。
fn base_known_ids(
    constitution: &Node,
    rules: &Node,
    srs: &Node,
    adr: Option<&Adr>,
) -> HashSet<String> {
    let articles = maps(constitution, "articles");
    let rule_rows: Vec<&Node> = RULE_SECTIONS.iter().flat_map(|s| maps(rules, s)).collect();
    let mut known = refs::known_ids(&articles, &rule_rows, srs, &mut Report::default());
    if let Some(adr) = adr {
        known.extend(link::adr_ids(adr).iter().map(|id| (*id).to_string()));
    }
    known
}

/// 要件書の id（契約表の行の req の解決先）。
fn requirement_ids(srs: &Node) -> HashSet<String> {
    let mut ids = HashSet::new();
    for section in SRS_ID_SECTIONS {
        for row in maps(srs, section) {
            if let Some(id) = row.get("id").and_then(Node::as_str) {
                ids.insert(id.to_string());
            }
        }
    }
    ids
}

// ── (c) 設計ノート 1 本 ──

#[allow(clippy::too_many_arguments)]
fn check_one(
    note: &NoteDoc,
    note_ids: &HashSet<&str>,
    base: &HashSet<String>,
    requirements: &HashSet<String>,
    external: Option<&[Field]>,
    gate: Option<&prose::Gate>,
    report: &mut Report,
) {
    let file = format!("{DIR}/{}", note.file);
    let root = &note.root;

    for key in DOC.required {
        if root.get(key).is_none() {
            report.violation(KIND, format!("{file}: 節「{key}」が無い"));
        }
    }
    unknown_sections(&file, root, &DOC.all(), report);

    check_meta(&file, note, note_ids, report);

    // 参照 id の母集団に、この文書の契約表の行 id（裸の形と「<doc id>#<row id>」の形）を足す
    let sections = row_list(&file, "sections", root, "sections", report);
    let mut known = base.clone();
    for section in &sections {
        if section.get("type").and_then(Node::as_str) != Some(CONTRACT_TABLE) {
            continue;
        }
        for row in section
            .get("rows")
            .and_then(Node::as_seq)
            .unwrap_or_default()
        {
            let id = row_id(row);
            if id != "?" {
                known.insert(format!("{}#{id}", note.id));
                known.insert(id);
            }
        }
    }

    // prose の節の番号（契約表の行の section の解決先）
    let prose_ns: HashSet<&str> = sections
        .iter()
        .filter(|s| s.get("type").and_then(Node::as_str) == Some(PROSE))
        .filter_map(|s| s.get("n").and_then(Node::as_str))
        .collect();

    let mut previous: Option<u64> = None;
    for section in &sections {
        check_section(
            &file,
            section,
            &mut previous,
            &known,
            requirements,
            &prose_ns,
            external,
            gate,
            report,
        );
    }

    check_figures(&file, root, &known, report);
}

/// meta の欄。
fn check_meta(file: &str, note: &NoteDoc, note_ids: &HashSet<&str>, report: &mut Report) {
    let blank = Node::Null;
    let meta = note.root.get("meta").unwrap_or(&blank);
    non_empty(file, "meta", meta, DOC_META.required, report);

    if let Some(id) = field(meta, "id") {
        if id != note.id {
            report.violation(
                KIND,
                format!(
                    "{file}: meta.id「{id}」が file 名の stem「{}」と違う",
                    note.id
                ),
            );
        }
        if !is_lower_id(id) {
            report.violation(
                KIND,
                format!("{file}: meta.id「{id}」が形 {ID_PATTERN} でない"),
            );
        }
    }
    if let Some(v) = field(meta, "version")
        && !is_version(v)
    {
        report.violation(
            KIND,
            format!("{file}: meta.version「{v}」が形 {VERSION_PATTERN} でない"),
        );
    }
    if let Some(v) = field(meta, "generated")
        && !is_date(v)
    {
        report.violation(
            KIND,
            format!("{file}: meta.generated「{v}」が年-月-日でない"),
        );
    }
    if let Some(v) = field(meta, "profile")
        && !PROFILE_ENUM.contains(&v)
    {
        report.violation(KIND, format!("{file}: meta.profile「{v}」が一覧に無い"));
    }
    let status = field(meta, "status");
    if let Some(v) = status
        && !STATUS_ENUM.contains(&v)
    {
        report.violation(KIND, format!("{file}: meta.status「{v}」が一覧に無い"));
    }

    let approval = row_list(file, "meta の approval", meta, "approval", report);
    let effective = status.is_some_and(|v| EFFECTIVE_STATUS.contains(&v));
    if effective && approval.is_empty() {
        report.violation(
            KIND,
            format!(
                "{file}: meta: status {} なのに approval（承認欄）が無い",
                status.unwrap_or("")
            ),
        );
    }
    if status == Some(STATUS_EXAMPLE) && !approval.is_empty() {
        report.violation(
            KIND,
            format!("{file}: meta: status {STATUS_EXAMPLE} は approval（承認欄）を持たない"),
        );
    }
    for (i, row) in approval.iter().enumerate() {
        let at = format!("meta の approval[{i}]");
        non_empty(file, &at, row, APPROVAL_REQUIRED, report);
        if let Some(v) = field(row, "date")
            && !is_date(v)
        {
            report.violation(KIND, format!("{file}: {at}: date「{v}」が年-月-日でない"));
        }
        if let Some(v) = field(row, "surface")
            && !SURFACE_ENUM.contains(&v)
        {
            report.violation(
                KIND,
                format!(
                    "{file}: {at}: surface「{v}」が一覧に無い（対話面は rules 行の id で指す・R-8）"
                ),
            );
        }
    }

    for key in ["supersedes", "superseded_by"] {
        if let Some(v) = field(meta, key)
            && !(note_ids.contains(v) && v != note.id)
        {
            report.violation(
                KIND,
                format!("{file}: meta.{key}「{v}」の設計ノートが実在しない"),
            );
        }
    }
    if status == Some(STATUS_RETIRED) && field(meta, "superseded_by").is_none() {
        report.violation(
            KIND,
            format!("{file}: meta: {STATUS_RETIRED} なのに superseded_by（後継）が無い（P-7.2）"),
        );
    }
}

/// 節 1 つ（番号・型・型ごとの行）。
#[allow(clippy::too_many_arguments)]
fn check_section(
    file: &str,
    section: &Node,
    previous: &mut Option<u64>,
    known: &HashSet<String>,
    requirements: &HashSet<String>,
    prose_ns: &HashSet<&str>,
    external: Option<&[Field]>,
    gate: Option<&prose::Gate>,
    report: &mut Report,
) {
    let shown = section.get("n").and_then(Node::as_str).unwrap_or("?");
    let at = format!("§{shown}");
    non_empty(file, &at, section, SECTION.required, report);

    match shown.parse::<u64>() {
        Ok(n) if n >= 1 => {
            if previous.is_some_and(|p| n <= p) {
                report.violation(
                    KIND,
                    format!(
                        "{file}: {at}: n が前の節（{}）より大きくない（同じ・戻るは認めない）",
                        previous.unwrap_or(0)
                    ),
                );
            }
            *previous = Some(n);
        }
        _ => report.violation(
            KIND,
            format!("{file}: {at}: n「{shown}」が 1 以上の整数でない"),
        ),
    }

    let Some(ty) = field(section, "type") else {
        return; // 欄の非空が数えてある
    };
    if !TYPE_ENUM.contains(&ty) {
        report.violation(KIND, format!("{file}: {at}: 節の型「{ty}」が一覧に無い"));
        return;
    }
    for key in if ty == PROSE { NEEDS_BODY } else { NEEDS_ROWS } {
        if section.get(key).is_none() {
            report.violation(
                KIND,
                format!("{file}: {at}: 節の型 {ty} の欄「{key}」が無い"),
            );
        }
    }
    if ty == PROSE {
        for key in FORBIDS_ROWS {
            if section.get(key).is_some() {
                report.violation(
                    KIND,
                    format!("{file}: {at}: 節の型 {ty} に置けない欄「{key}」が在る"),
                );
            }
        }
        // 散文の門（FR12・R-16）。印を持つ文の参照 id は同じ母集団で解く
        if let (Some(gate), Some(body)) = (gate, field(section, "body")) {
            for m in prose::scan(body, gate) {
                if let Some(why) = m.reason {
                    report.violation(
                        PROSE_GATE,
                        format!("{file}: 節 {shown} 行 {}: {why} {}", m.line, m.head),
                    );
                }
                for id in m.pointers.iter().filter(|id| !known.contains(*id)) {
                    report.violation(
                        KIND,
                        format!("{file}: {at}: 散文の参照 id「{id}」が実在しない"),
                    );
                }
            }
        }
        return;
    }

    let rows = row_list(file, &format!("{at} の rows"), section, "rows", report);
    if ty == CONTRACT_TABLE {
        check_contract_rows(file, &at, &rows, requirements, prose_ns, external, report);
    } else {
        check_table_rows(file, &at, ty, &rows, known, report);
    }
    duplicate_ids(file, rows, report);
}

/// 部品・口・欄・歯の表の行（行の欄の集合と値域・参照 id）。
fn check_table_rows(
    file: &str,
    at: &str,
    ty: &str,
    rows: &[&Node],
    known: &HashSet<String>,
    report: &mut Report,
) {
    let keys = match ty {
        "parts-table" => &PARTS_ROW,
        "ports-table" => &PORTS_ROW,
        "fields-table" => &FIELDS_ROW,
        _ => &TEETH_ROW,
    };
    for row in rows {
        let rat = format!("{at} の行 {}", row_id(row));
        non_empty(file, &rat, row, keys.required, report);
        for (key, _) in row.as_map().unwrap_or_default() {
            if !keys.has(key) {
                report.violation(
                    KIND,
                    format!("{file}: {rat}: 行の欄「{key}」が欄の決まりに無い"),
                );
            }
        }
        if ty == "fields-table" {
            if let Some(v) = field(row, "need")
                && !NEED_ENUM.contains(&v)
            {
                report.violation(KIND, format!("{file}: {rat}: need「{v}」が一覧に無い"));
            }
            if let Some(v) = field(row, "shape")
                && !SHAPE_ENUM.contains(&v)
            {
                report.violation(KIND, format!("{file}: {rat}: shape「{v}」が一覧に無い"));
            }
        }
        resolve_ids(file, &rat, row, "ref", known, report);
    }
}

/// 契約表の行（欄は器の導出 file が決める・id と参照は folio2 が持つ）。
fn check_contract_rows(
    file: &str,
    at: &str,
    rows: &[&Node],
    requirements: &HashSet<String>,
    prose_ns: &HashSet<&str>,
    external: Option<&[Field]>,
    report: &mut Report,
) {
    let row_ids: HashSet<String> = rows
        .iter()
        .map(|row| row_id(row))
        .filter(|id| id != "?")
        .collect();
    for row in rows {
        let id = row_id(row);
        let rat = format!("{at} の行 {id}");
        if let Some(fields) = external {
            let required: Vec<&str> = fields
                .iter()
                .filter(|f| f.need == "required")
                .map(|f| f.name.as_str())
                .collect();
            non_empty(file, &rat, row, &required, report);
            for (key, _) in row.as_map().unwrap_or_default() {
                if !fields.iter().any(|f| &f.name == key) {
                    report.violation(
                        KIND,
                        format!("{file}: {rat}: 契約表の欄「{key}」が器の導出 file に無い"),
                    );
                }
            }
            for f in fields {
                let Some(value) = row.get(&f.name) else {
                    continue;
                };
                let ok = match f.shape.as_str() {
                    "text" => matches!(value, Node::Null | Node::Scalar(_)),
                    _ => match value {
                        Node::Null => true,
                        Node::Seq(items) => items.iter().all(|x| x.as_str().is_some()),
                        _ => false,
                    },
                };
                if !ok {
                    report.violation(
                        KIND,
                        format!(
                            "{file}: {rat}: 欄「{}」が {} の形でない",
                            f.name,
                            if f.shape == "text" {
                                "text（文字列）"
                            } else {
                                "list（文字列の一覧）"
                            }
                        ),
                    );
                }
            }
        }
        if id != "?" && !is_lower_id(&id) {
            report.violation(
                KIND,
                format!("{file}: {rat}: 行 id「{id}」が形 {ROW_ID_PATTERN} でない"),
            );
        }
        if let Some(v) = field(row, "section")
            && !prose_ns.contains(v)
        {
            report.violation(
                KIND,
                format!("{file}: {rat}: section「{v}」が同じ文書の {PROSE} の節の n でない"),
            );
        }
        resolve_ids(file, &rat, row, "req", requirements, report);
        resolve_ids(file, &rat, row, "depends", &row_ids, report);
    }
}

// ── (c) 図 ──

fn check_figures(file: &str, root: &Node, known: &HashSet<String>, report: &mut Report) {
    for entry in row_list(file, "figures", root, "figures", report) {
        let at = format!("figures の {}", row_id(entry));
        non_empty(file, &at, entry, FIGURE_ENTRY.required, report);
        if let Some(v) = field(entry, "type")
            && FigureType::from_name(v).is_none()
        {
            report.violation(
                KIND,
                format!("{file}: {at}: 図の型「{v}」が部品目録の一覧に無い"),
            );
        }
        if let Some(spec) = entry.get("spec")
            && !matches!(spec, Node::Map(_))
        {
            report.violation(KIND, format!("{file}: {at}: spec が表でない"));
        }
        resolve_ids(file, &at, entry, "refs", known, report);
    }
}

// ── 小さな読み手 ──

/// 行の一覧。無い・null は 0 行、表の一覧でなければ「まだ分からない」。
fn row_list<'a>(
    file: &str,
    place: &str,
    node: &'a Node,
    key: &str,
    report: &mut Report,
) -> Vec<&'a Node> {
    match node.get(key) {
        None | Some(Node::Null) => Vec::new(),
        Some(Node::Seq(items)) if items.iter().all(|n| n.as_map().is_some()) => {
            items.iter().collect()
        }
        Some(_) => {
            report.unknown(format!("{file}: {place} が表の一覧でない"));
            Vec::new()
        }
    }
}

/// 値を持つ文字列の欄（無い・null・空白だけは None＝欄の非空の側が数える）。
fn field<'a>(node: &'a Node, key: &str) -> Option<&'a str> {
    node.get(key)
        .and_then(Node::as_str)
        .filter(|s| !s.trim().is_empty())
}

/// 一覧の欄の各要素が既知の id に解けるか。
fn resolve_ids(
    file: &str,
    at: &str,
    node: &Node,
    key: &str,
    known: &HashSet<String>,
    report: &mut Report,
) {
    match node.get(key) {
        None | Some(Node::Null) => {}
        Some(Node::Seq(items)) => {
            for (i, item) in items.iter().enumerate() {
                match item.as_str() {
                    Some(v) if known.contains(v) => {}
                    Some(v) => report.violation(
                        KIND,
                        format!("{file}: {at} の {key}[{i}]: id「{v}」が実在しない"),
                    ),
                    None => {
                        report.unknown(format!("{file}: {at} の {key}[{i}] が文字列でない"));
                    }
                }
            }
        }
        Some(_) => report.unknown(format!("{file}: {at} の {key} が一覧でない")),
    }
}

fn digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// 年 4 桁-月 2 桁-日 2 桁（date_format）。
fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && digits(&s[..4])
        && digits(&s[5..7])
        && digits(&s[8..])
}

/// 英小文字で始まり、英小文字・数字・「-」だけが続く（id_pattern / row_id.pattern）。
fn is_lower_id(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// 「v」+ 数字列 + 「.」+ 数字列（version_pattern）。
fn is_version(s: &str) -> bool {
    s.strip_prefix('v')
        .and_then(|rest| rest.split_once('.'))
        .is_some_and(|(major, minor)| digits(major) && digits(minor))
}
