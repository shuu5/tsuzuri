//! `folio check` の設計ノートの正本（`design-note/*.yaml`・ADR-3 決定 (1)・要件書 FR9 / FR10）の形の検査
//! （便 23・docs/design/delivery-23.md §1）。
//! 数えるのは 欄の決まり（同じ dir の `schema.yaml`）の schema 節が定める形（文書と meta の欄・節の番号と型・
//! 型ごとの行の欄と値域・承認欄の要否）と、契約表の節の欄（器 scribe2 の導出 file `contracts/schema.toml` から読む・
//! 欄の一覧も値域も自分の型にも散文にも持たない・FR10）と、参照 id の解決（folio2 が所有する id 空間）である。
//! 散文の門（FR12・rules 行 R-16）は便 24 で入り、索引（FR14）は 2026-09-22 に着地した（folio graph --print・graph.rs）。導出物（FR11）は未実装（欄の決まりの注が明記する）。
//! 欄の決まりの閾値・値域・置き場は床の定数（`FLOOR`）で持ち、`design-note/schema.yaml` の schema 節はその写し
//! （判断の記録の欄の決まり `adr.rs` と同じ作り・N-3.1）。パターンの文字列は定数として字面で持つだけで、
//! 形の判定は字の走査で行う（正規表現は使わない）。器の導出 file（TOML）も行走査で読む（外部 crate を足さない）。
//! 便 46 から床の機械（床の木の型・突き合わせ）は `schema.rs` のものを使い、schema 節は生成区間で
//! `folio schema --write` が `FLOOR` から導出する＝説明の注（`_note` で終わる欄）も FLOOR が file の順と字面のまま持つ。
//! 床の定数（欄の集合の型とその method・欄の決まりの定数・`FLOOR`）は便 114 で `floor_note.rs`（層 1）へ降ろした。検査の本体はここに残す。

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::adr::Adr;
use crate::catalog::FigureType;
use crate::check::{duplicate_ids, non_empty, row_id, unknown_sections};
use crate::floor::{floor_diff, strip_notes};
use crate::floor_note::{
    APPROVAL_REQUIRED, CONTRACT_TABLE, DOC, DOC_META, EFFECTIVE_STATUS, EXTERNAL_HEAD,
    EXTERNAL_NEED, EXTERNAL_PATH, EXTERNAL_ROW_FIELDS, EXTERNAL_ROWS_KEY, EXTERNAL_SHAPE,
    FIELDS_ROW, FIGURE_ENTRY, FLOOR, FORBIDS_ROWS, ID_PATTERN, Keys, NEED_ENUM, NEEDS_BODY,
    NEEDS_ROWS, PARTS_ROW, PORTS_ROW, PROSE, ROW_ID_PATTERN, SECTION, SHAPE_ENUM, STATUS_ENUM,
    STATUS_EXAMPLE, STATUS_RETIRED, SURFACE_ENUM, TEETH_ROW, TYPE_ENUM, VERSION_PATTERN,
};
use crate::link;
use crate::prose;
use crate::refs;
use crate::verdict::Report;
use crate::yaml::{self, Node};

/// 設計ノートの置き場（正本の dir の直下）と欄の決まりの file。
const DIR: &str = "design-note";
const SCHEMA_FILE: &str = "design-note/schema.yaml";

/// 違反の種別（設計ノートの形・未知の欄・散文の門）。
const KIND: &str = "note";
const UNKNOWN_FIELD: &str = "未知の欄";
const PROSE_GATE: &str = "prose-gate";

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
    unknown_fields(file, "meta", meta, &DOC_META, report);

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
        && !crate::catalog::PROFILES.contains(&v)
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
    // 節の欄は SECTION の和集合で見る（型ごとの required / forbid は下の by_type が別に数える）
    unknown_fields(file, &at, section, &SECTION, report);

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
        unknown_fields(file, &at, entry, &FIGURE_ENTRY, report);
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

/// 欄の表の未知の欄（欄の決まりの required と optional の和集合に無い鍵）を 1 件ずつ数える
/// （要件書 FR9 の正本の形・要件書の図の行の検査と同じ字面）。表でなければ 0 件（形の側が数えてある）。
fn unknown_fields(file: &str, at: &str, node: &Node, keys: &Keys, report: &mut Report) {
    for (key, _) in node.as_map().unwrap_or_default() {
        if !keys.has(key) {
            report.violation(UNKNOWN_FIELD, format!("{file}: {at} の未知の欄「{key}」"));
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 床の木の導出は凍結 anchor（設計判断の席が独立の実装で組んだ・P-10.1）と byte 一致（便 57 §1 (c)3・
    /// `ceiling.rs` / `rules.rs` の同名の歯と同じ形で `schema.rs` の pub の `derive` を呼ぶ）。
    #[test]
    fn note_floor_derives_the_frozen_anchor_byte_for_byte() {
        let anchor = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/schema/note-region.txt"
        ))
        .unwrap();
        assert_eq!(crate::floor::derive(&FLOOR), anchor);
    }
}
