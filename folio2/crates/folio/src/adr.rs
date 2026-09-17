//! `folio check` の判断の記録（`adr/`）の欄の決まりの検査（便 5・docs/design/delivery-5.md §1）。
//! day-1 の床 `scripts/check_draft.py` の adr の節のうち、判断の記録の file と欄の決まりの file だけで閉じる検査を同じ式で写す。
//! 憲法・rules・anchor と突き合わせる検査（amends の対象の実在・amended_by との双方向・対話面の行の実在・
//! 判断の記録の id の参照・本文の英字語）は便 6 の `link.rs` で、読んだ欄の決まりと判断の記録（`Adr`）と床の定数（`floor_strs`）を渡す。
//! 凍結 anchor の列そのものは便 7。
//! 欄の決まりの閾値・値域・置き場は床の定数（`FLOOR`）で持ち、adr/schema.yaml の schema 節はその写し（N-3.1）。
//! パターンの文字列は定数として字面で持つだけで、形の判定は字の走査で行う（正規表現は使わない）。

use std::fs;
use std::path::Path;

use crate::verdict::Report;
use crate::yaml::{self, Node};

/// 床の定数の木（adr/schema.yaml の schema 節と同じ形）。
#[derive(Debug)]
enum Floor {
    /// 値（yaml の値の字面・引用符を除く）
    Val(&'static str),
    /// 数（字面で比べる＝2.0 と 2 は違う）
    Num(usize),
    /// 値の一覧
    Strs(&'static [&'static str]),
    /// 表（欄の順は schema 節の順）
    Map(&'static [(&'static str, Floor)]),
}

/// 欄の集合（required / optional）。
pub(crate) struct Keys {
    required: &'static [&'static str],
    optional: &'static [&'static str],
}

const ID_PATTERN: &str = "^ADR-[1-9][0-9]*$";
const DATE_FORMAT: &str = r"^\d{4}-\d{2}-\d{2}$";
const RULING_PATTERN: &str = r"[a-z]\d-[0-9a-z]+(\.\d+)?";
const OWNER: &str = "持ち主";
const RECORD: Keys = Keys {
    required: &[
        "id", "title", "status", "date", "context", "decision", "options", "basis", "retreat",
        "plain",
    ],
    optional: &[
        "amends",
        "grill",
        "approval",
        "consequences",
        "supersedes",
        "superseded_by",
        "note",
    ],
};
const NON_EMPTY: &[&str] = &["title", "context", "decision", "plain"];
const STATUS: &[&str] = &["proposed", "accepted", "retired"];
const VERDICT: &[&str] = &["adopted", "rejected"];
const RETREAT_KIND: &[&str] = &["spike", "measure", "ruling"];
const APPROVER: &[&str] = &["持ち主", "planner 席"];
const SURFACE: &[&str] = &["R-8"];
const EFFECTIVE_STATUS: &[&str] = &["accepted", "retired"];
const OPTION: Keys = Keys {
    required: &["id", "name", "text", "verdict", "reason"],
    optional: &[],
};
const OPTIONS_MIN: usize = 2;
const OPTIONS_ADOPTED: usize = 1;
const RETREAT: Keys = Keys {
    required: &["kind", "condition"],
    optional: &[],
};
const AMENDS_ENTRY: Keys = Keys {
    required: &["target", "field", "version", "previous_text", "new_text"],
    optional: &[],
};
const GRILL: Keys = Keys {
    required: &["when", "who", "where", "summary"],
    optional: &[],
};
const APPROVAL: Keys = Keys {
    required: &["who", "date", "ruling", "verbatim", "surface"],
    optional: &[],
};
pub(crate) const AMENDED_BY_ENTRY: Keys = Keys {
    required: &[
        "adr",
        "date",
        "approved_by",
        "ruling",
        "previous_text",
        "rationale",
    ],
    optional: &[],
};

macro_rules! keys_floor {
    ($keys:expr) => {
        Floor::Map(&[
            ("required", Floor::Strs($keys.required)),
            ("optional", Floor::Strs($keys.optional)),
        ])
    };
}

/// 床の定数（値は day-1 の床の FLOOR と同じ）。
const FLOOR: Floor = Floor::Map(&[
    ("id_pattern", Floor::Val(ID_PATTERN)),
    ("date_format", Floor::Val(DATE_FORMAT)),
    ("ruling_pattern", Floor::Val(RULING_PATTERN)),
    ("owner", Floor::Val(OWNER)),
    ("required", Floor::Strs(RECORD.required)),
    ("optional", Floor::Strs(RECORD.optional)),
    ("non_empty", Floor::Strs(NON_EMPTY)),
    (
        "enums",
        Floor::Map(&[
            ("status", Floor::Strs(STATUS)),
            ("verdict", Floor::Strs(VERDICT)),
            ("retreat_kind", Floor::Strs(RETREAT_KIND)),
            ("approver", Floor::Strs(APPROVER)),
            ("surface", Floor::Strs(SURFACE)),
        ]),
    ),
    ("effective_status", Floor::Strs(EFFECTIVE_STATUS)),
    ("option", keys_floor!(OPTION)),
    (
        "options_rule",
        Floor::Map(&[
            ("min", Floor::Num(OPTIONS_MIN)),
            ("adopted", Floor::Num(OPTIONS_ADOPTED)),
        ]),
    ),
    ("retreat", keys_floor!(RETREAT)),
    (
        "amends_entry",
        Floor::Map(&[
            ("required", Floor::Strs(AMENDS_ENTRY.required)),
            ("optional", Floor::Strs(AMENDS_ENTRY.optional)),
            ("new_article_marker", Floor::Val("（新設）")),
            ("deleted_marker", Floor::Val("（削除）")),
            ("empty_marker", Floor::Val("（空）")),
        ]),
    ),
    ("grill", keys_floor!(GRILL)),
    ("approval", keys_floor!(APPROVAL)),
    ("amended_by_entry", keys_floor!(AMENDED_BY_ENTRY)),
    (
        "anchor",
        Floor::Map(&[
            ("dir", Floor::Val("anchors")),
            ("file_name", Floor::Val("constitution-<version>.yaml")),
            ("index_file", Floor::Val("index.yaml")),
            ("first_version", Floor::Val("v1.0")),
            (
                "root_digest",
                Floor::Val("acb52acd04b5d3a1feaf9ad5f0138f7614ce31964144b46ead914bde86e866ed"),
            ),
            ("version_pattern", Floor::Val(r"^v[0-9]+\.[0-9]+$")),
            ("digest_algo", Floor::Val("sha256-json-1")),
            (
                "file_keys",
                Floor::Strs(&[
                    "kind",
                    "digest_algo",
                    "version",
                    "previous",
                    "projection",
                    "meta_approval",
                    "approvals",
                    "content",
                    "digest",
                ]),
            ),
            (
                "projection_article_fields",
                Floor::Strs(&["id", "title", "tier", "binds", "statements"]),
            ),
            (
                "statement_fields",
                Floor::Strs(&["id", "text", "pattern", "strength"]),
            ),
            (
                "scope_minimum",
                Floor::Strs(&["schema", "precedence", "articles"]),
            ),
        ]),
    ),
]);

/// 床の定数の値の一覧を欄の道で読む（便 6 の突き合わせの読み口・値は変えない）。道が一覧に着かなければ空。
pub(crate) fn floor_strs(path: &[&str]) -> &'static [&'static str] {
    match floor_at(path) {
        Some(Floor::Strs(items)) => items,
        _ => &[],
    }
}

/// 床の定数の値を欄の道で読む（便 7 の凍結 anchor の読み口・値は変えない）。道が値に着かなければ None。
pub(crate) fn floor_val(path: &[&str]) -> Option<&'static str> {
    match floor_at(path) {
        Some(Floor::Val(v)) => Some(v),
        _ => None,
    }
}

/// 床の定数の数を欄の道で読む（同上）。道が数に着かなければ None。
/// 便 7 の検査は数型を読まない（読み口として置く）。
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn floor_num(path: &[&str]) -> Option<usize> {
    match floor_at(path) {
        Some(Floor::Num(n)) => Some(*n),
        _ => None,
    }
}

fn floor_at(path: &[&str]) -> Option<&'static Floor> {
    let mut cur: &'static Floor = &FLOOR;
    for key in path {
        let Floor::Map(fields) = cur else {
            return None;
        };
        cur = &fields.iter().find(|(k, _)| k == key)?.1;
    }
    Some(cur)
}

const SCHEMA_FILE: &str = "adr/schema.yaml";

/// 読めた欄の決まり（adr/schema.yaml の木）と判断の記録（id と木の組・名前順）。
pub(crate) struct Adr {
    pub schema: Node,
    pub records: Vec<(String, Node)>,
}

/// `dir/adr/` の欄の決まりと判断の記録を検査する。欄の決まりが読めなければ None（「まだ分からない」は立ててある）。
pub fn check_adr(dir: &Path, report: &mut Report) -> Option<Adr> {
    let adr_dir = dir.join("adr");
    if adr_dir.is_symlink() || !adr_dir.is_dir() {
        report.unknown(format!(
            "adr/ が dir でない（symlink・file・不在）: {}",
            adr_dir.display()
        ));
        return None;
    }
    let schema = load_schema(&adr_dir, report)?;
    let mut drift = Vec::new();
    floor_diff(
        &strip_notes(schema.get("schema").unwrap_or(&Node::Null)),
        &FLOOR,
        "",
        &mut drift,
    );
    for path in drift {
        report.violation(
            "adr",
            format!(
                "{SCHEMA_FILE} schema.{path} が床の定数と違う（欄の決まりの閾値・値域・置き場は床の定数の写し＝data 側で動かせない・N-3.1）"
            ),
        );
    }
    let records = load_records(dir, &adr_dir, report);
    check_between(&records, report);
    check_decided_by(&schema, &records, report);
    Some(Adr { schema, records })
}

fn read(path: &Path) -> Result<yaml::Doc, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    yaml::parse(&text)
}

/// 重複キーは床の読み手と同じく「読めない」（まだ分からない）。在れば true。
fn duplicates(file: &str, doc: &yaml::Doc, report: &mut Report) -> bool {
    for dup in &doc.duplicates {
        report.unknown(format!(
            "{file} {} 行: 読めない（重複キー「{}」＝同じ表に 2 度書いている）",
            dup.line, dup.key
        ));
    }
    !doc.duplicates.is_empty()
}

/// (a) 欄の決まりの file。読めない・形が違う は「まだ分からない」。
fn load_schema(adr_dir: &Path, report: &mut Report) -> Option<Node> {
    let path = adr_dir.join("schema.yaml");
    if path.is_symlink() {
        report.unknown(format!("{SCHEMA_FILE}: symlink は認めない"));
        return None;
    }
    if !path.exists() {
        report.unknown(format!("{SCHEMA_FILE}: 欄の決まりが無い"));
        return None;
    }
    if !path.is_file() {
        report.unknown(format!("{SCHEMA_FILE}: file でない"));
        return None;
    }
    let doc = match read(&path) {
        Ok(d) => d,
        Err(e) => {
            report.unknown(format!("{SCHEMA_FILE}: 読めない: {e}"));
            return None;
        }
    };
    if duplicates(SCHEMA_FILE, &doc, report) {
        return None;
    }
    let sections_ok = doc.root.as_map().is_some_and(|m| {
        m.iter()
            .all(|(k, _)| matches!(k.as_str(), "meta" | "schema" | "plain"))
    });
    if !sections_ok || !matches!(doc.root.get("schema"), Some(Node::Map(_))) {
        report.unknown(format!(
            "{SCHEMA_FILE}: 形が違う（節は meta / schema / plain・schema 節は欄の表）"
        ));
        return None;
    }
    Some(doc.root)
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

/// 写しと床の定数の違いを欄の道で並べる。
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

/// (c) 判断の記録の file を名前順に読み、(d) の欄を見る。読めた記録を id と組で返す。
fn load_records(dir: &Path, adr_dir: &Path, report: &mut Report) -> Vec<(String, Node)> {
    let mut names: Vec<String> = match fs::read_dir(adr_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".yaml") && n != "schema.yaml")
            .collect(),
        Err(e) => {
            report.unknown(format!("adr/: 読めない: {e}"));
            return Vec::new();
        }
    };
    names.sort();
    let root = fs::canonicalize(dir).ok();
    let mut records: Vec<(String, Node)> = Vec::new();
    for name in names {
        let path = adr_dir.join(&name);
        let mut real = true;
        if path.is_symlink() {
            report.violation("adr", format!("{name}: symlink は認めない"));
            real = false;
        }
        let inside = match (&root, fs::canonicalize(&path)) {
            (Some(r), Ok(p)) => p.starts_with(r),
            _ => false,
        };
        if !inside {
            report.violation("adr", format!("{name}: design-intent の外を指している"));
            real = false;
        }
        if !real {
            continue;
        }
        let doc = match read(&path) {
            Ok(d) => d,
            Err(e) => {
                report.violation(
                    "adr",
                    format!("{name}: 判断の記録が欄の表でない（読めない: {e}）"),
                );
                continue;
            }
        };
        if duplicates(&format!("adr/{name}"), &doc, report) {
            continue;
        }
        let d = doc.root;
        if d.as_map().is_none() {
            report.violation("adr", format!("{name}: 判断の記録が欄の表でない"));
            continue;
        }
        let id = scalar(d.get("id")).unwrap_or("?").to_string();
        check_keys("adr", &id, &d, &RECORD, report);
        if !is_adr_id(&id) {
            report.violation(
                "adr",
                format!("{name}: id「{id}」が形 {ID_PATTERN} でない（ゼロ詰めしない）"),
            );
        }
        if name.strip_suffix(".yaml") != Some(id.as_str()) {
            report.violation(
                "adr",
                format!("{name}: file 名が id {id} と違う（1 判断 = 1 file・file 名 = id）"),
            );
        }
        if find(&records, &id).is_some() {
            report.violation("adr", format!("{id}: id が重複（P-7）"));
            continue;
        }
        check_fields(&id, &d, report);
        records.push((id, d));
    }
    records
}

/// (d) 1 本の判断の記録の欄。
fn check_fields(id: &str, d: &Node, report: &mut Report) {
    if !in_enum(d.get("status"), STATUS) {
        report.violation(
            "adr",
            format!("{id}: status が値域外: {}", show(d.get("status"))),
        );
    }
    check_date("adr", &format!("{id}.date"), d.get("date"), report);
    for k in NON_EMPTY {
        if !non_empty(d.get(k)) {
            report.violation("adr", format!("{id}: {k} が空"));
        }
    }

    let options: &[Node] = match d.get("options") {
        Some(Node::Seq(items)) => items,
        _ => {
            report.violation("adr", format!("{id}: options が一覧でない"));
            &[]
        }
    };
    if options.len() < OPTIONS_MIN {
        report.violation(
            "adr",
            format!(
                "{id}: 案が {} 件（退けた案を含めて {OPTIONS_MIN} 件以上）",
                options.len()
            ),
        );
    }
    for o in options {
        let at = format!("{id}.options[{}]", show(o.get("id")));
        if !check_keys("adr", &at, o, &OPTION, report) {
            continue;
        }
        if !in_enum(o.get("verdict"), VERDICT) {
            report.violation(
                "adr",
                format!("{at}: verdict が値域外: {}", show(o.get("verdict"))),
            );
        }
        for k in ["name", "text", "reason"] {
            if !non_empty(o.get(k)) {
                report.violation("adr", format!("{at}.{k} が空"));
            }
        }
    }
    let adopted = options
        .iter()
        .filter(|o| scalar(o.get("verdict")) == Some("adopted"))
        .count();
    if adopted != OPTIONS_ADOPTED {
        report.violation(
            "adr",
            format!("{id}: 採用の案が {adopted} 件（{OPTIONS_ADOPTED} 件）"),
        );
    }

    let retreat = d.get("retreat").unwrap_or(&Node::Null);
    if check_keys("P-8", &format!("{id}.retreat"), retreat, &RETREAT, report) {
        if !in_enum(retreat.get("kind"), RETREAT_KIND) {
            report.violation(
                "P-8",
                format!("{id}: retreat.kind が値域外: {}", show(retreat.get("kind"))),
            );
        }
        if !non_empty(retreat.get("condition")) {
            report.violation("P-8", format!("{id}: 撤退条件が空（P-8.1）"));
        }
    }

    match d.get("basis") {
        Some(Node::Seq(items)) if !items.is_empty() => {
            for b in items {
                if !b.as_str().is_some_and(is_basis_id) {
                    report.violation(
                        "adr",
                        format!(
                            "{id}: basis「{}」が id の形（条・要件・rules 行・判断の記録）でない（P-5.2）",
                            show(Some(b))
                        ),
                    );
                }
            }
        }
        _ => report.violation("adr", format!("{id}: basis（根拠の id）が空")),
    }

    let amends: &[Node] = match present(d, "amends") {
        None => &[],
        Some(Node::Seq(items)) => items,
        Some(_) => {
            report.violation("A-2", format!("{id}: amends が一覧でない"));
            &[]
        }
    };
    for e in amends {
        let at = format!("{id}.amends[{}]", show(e.get("target")));
        if !check_keys("A-2", &at, e, &AMENDS_ENTRY, report) {
            continue;
        }
        for k in ["field", "version", "previous_text", "new_text"] {
            if !non_empty(e.get(k)) {
                report.violation("A-2", format!("{at}.{k} が空（空の値は印（空）で書く）"));
            }
        }
    }

    let approval = present(d, "approval");
    if in_enum(d.get("status"), EFFECTIVE_STATUS) && approval.is_none_or(Node::is_blank) {
        report.violation(
            "N-4",
            format!(
                "{id}: {} なのに approval（逐語・日付・裁定 id・対話面）が無い",
                show(d.get("status"))
            ),
        );
    }
    if let Some(ap) = approval {
        check_approval(&format!("{id}.approval"), ap, report);
    }

    if let Some(grill) = present(d, "grill") {
        let at = format!("{id}.grill");
        if check_keys("A-2", &at, grill, &GRILL, report) {
            check_date("A-2", &format!("{at}.when"), grill.get("when"), report);
            for k in ["who", "where", "summary"] {
                if !non_empty(grill.get(k)) {
                    report.violation("A-2", format!("{at}.{k} が空"));
                }
            }
        }
    }
}

fn check_approval(at: &str, ap: &Node, report: &mut Report) {
    const KIND: &str = "N-4";
    if !check_keys(KIND, at, ap, &APPROVAL, report) {
        return;
    }
    for k in ["ruling", "verbatim"] {
        if !non_empty(ap.get(k)) {
            report.violation(KIND, format!("{at}.{k} が空"));
        }
    }
    check_date(KIND, &format!("{at}.date"), ap.get("date"), report);
    if !in_enum(ap.get("who"), APPROVER) {
        report.violation(KIND, format!("{at}.who が値域外: {}", show(ap.get("who"))));
    }
    if !scalar(ap.get("ruling")).is_some_and(has_ledger_id) {
        report.violation(
            KIND,
            format!(
                "{at}.ruling「{}」に台帳 id（{RULING_PATTERN}）が無い",
                show(ap.get("ruling"))
            ),
        );
    }
    if !in_enum(ap.get("surface"), SURFACE) {
        report.violation(
            KIND,
            format!(
                "{at}.surface が値域外（対話面は rules 行の id で指す・R-8）: {}",
                show(ap.get("surface"))
            ),
        );
    }
}

/// (e) 判断の記録どうし。
fn check_between(records: &[(String, Node)], report: &mut Report) {
    for (id, d) in records {
        let effective = in_enum(d.get("status"), EFFECTIVE_STATUS)
            && matches!(d.get("approval"), Some(Node::Map(_)));
        let amends = matches!(d.get("amends"), Some(Node::Seq(items)) if items.iter().any(|e| e.as_map().is_some()));
        if effective && amends {
            let who = d.get("approval").and_then(|a| a.get("who"));
            if scalar(who) != Some(OWNER) {
                report.violation(
                    "N-4",
                    format!(
                        "{id}: 条文を改訂する発効した判断の承認者が {OWNER} でない（{}）",
                        show(who)
                    ),
                );
            }
            if !matches!(d.get("grill"), Some(Node::Map(_))) {
                report.violation(
                    "A-2",
                    format!("{id}: 条文を改訂する発効した判断に grill の記録が無い（A-2.3）"),
                );
            }
        }
        for k in ["supersedes", "superseded_by"] {
            if let Some(x) = present(d, k)
                && x.as_str().and_then(|x| find(records, x)).is_none()
            {
                report.violation(
                    "adr",
                    format!("{id}: {k} {} の判断の記録が実在しない", show(Some(x))),
                );
            }
        }
        let status = scalar(d.get("status"));
        let next = present(d, "superseded_by");
        if status == Some("retired") && next.is_none() {
            report.violation(
                "adr",
                format!("{id}: retired なのに superseded_by（後継）が無い（P-7.2）"),
            );
        }
        if let Some(n) = next {
            if status != Some("retired") {
                report.violation(
                    "adr",
                    format!("{id}: superseded_by を持つのに status が retired でない（P-7.2）"),
                );
            }
            if let Some(nx) = n.as_str().and_then(|n| find(records, n))
                && scalar(nx.get("supersedes")) != Some(id.as_str())
            {
                report.violation(
                    "adr",
                    format!(
                        "{id}: 後継 {} の supersedes に {id} が無い（双方向）",
                        show(Some(n))
                    ),
                );
            }
        }
        if let Some(p) = present(d, "supersedes")
            && let Some(pv) = p.as_str().and_then(|p| find(records, p))
            && scalar(pv.get("superseded_by")) != Some(id.as_str())
        {
            report.violation(
                "adr",
                format!(
                    "{id}: 置き換えた {} の superseded_by が {id} でない（双方向）",
                    show(Some(p))
                ),
            );
        }
    }

    // retired の後継の列は accepted に着く（輪・未発効の後継は落とす）。
    for (id, d) in records {
        if scalar(d.get("status")) != Some("retired") {
            continue;
        }
        let mut seen: Vec<&str> = vec![id];
        let mut cur = d;
        while let Some(nid) = scalar(cur.get("superseded_by")) {
            let Some(nx) = find(records, nid) else {
                break; // 実在しない後継は上で数えてある
            };
            if seen.contains(&nid) {
                report.violation(
                    "adr",
                    format!(
                        "{id}: retired の後継の列が輪になっている（{}→{nid}）＝発効している後継が無い（P-7.2）",
                        seen.join("→")
                    ),
                );
                break;
            }
            seen.push(nid);
            match scalar(nx.get("status")) {
                Some("accepted") => break,
                Some("retired") => cur = nx,
                other => {
                    report.violation(
                        "adr",
                        format!(
                            "{id}: retired の後継の列の先 {nid} が発効していない（status {}）",
                            other.unwrap_or("（無い）")
                        ),
                    );
                    break;
                }
            }
        }
    }
}

/// (a) meta.decided_by の非空と実在。
fn check_decided_by(schema: &Node, records: &[(String, Node)], report: &mut Report) {
    let items: Vec<&Node> = match schema.get("meta").and_then(|m| m.get("decided_by")) {
        Some(Node::Seq(items)) => items.iter().collect(),
        Some(n) if !n.is_blank() => vec![n],
        _ => Vec::new(),
    };
    if items.is_empty() {
        report.violation(
            "adr",
            format!("{SCHEMA_FILE} meta.decided_by が空（欄の決まりの出所の判断が無い）"),
        );
    }
    for x in items {
        if x.as_str().and_then(|x| find(records, x)).is_none() {
            report.violation(
                "adr",
                format!(
                    "{SCHEMA_FILE} meta.decided_by の {} が実在しない（欄の決まりの出所の判断が消えている）",
                    show(Some(x))
                ),
            );
        }
    }
}

fn find<'a>(records: &'a [(String, Node)], id: &str) -> Option<&'a Node> {
    records.iter().find(|(k, _)| k == id).map(|(_, d)| d)
}

/// 欄の集合が keys と一致するか（required の欠落・未知の欄を 1 件ずつ）。表でなければ false。
pub(crate) fn check_keys(
    kind: &str,
    at: &str,
    node: &Node,
    keys: &Keys,
    report: &mut Report,
) -> bool {
    let Some(entries) = node.as_map() else {
        report.violation(kind, format!("{at}: 型が違う（欄の表でない）"));
        return false;
    };
    let missing: Vec<&str> = keys
        .required
        .iter()
        .copied()
        .filter(|k| node.get(k).is_none())
        .collect();
    if !missing.is_empty() {
        report.violation(kind, format!("{at}: 必須欄が無い: {}", missing.join("・")));
    }
    let unknown: Vec<&str> = entries
        .iter()
        .map(|(k, _)| k.as_str())
        .filter(|k| !keys.required.contains(k) && !keys.optional.contains(k))
        .collect();
    if !unknown.is_empty() {
        report.violation(
            kind,
            format!("{at}: 未知の欄（N-3）: {}", unknown.join("・")),
        );
    }
    true
}

/// 欄が在って値を持つ（null でない）。
pub(crate) fn present<'a>(node: &'a Node, key: &str) -> Option<&'a Node> {
    node.get(key).filter(|v| !matches!(v, Node::Null))
}

pub(crate) fn scalar(node: Option<&Node>) -> Option<&str> {
    node.and_then(Node::as_str)
}

pub(crate) fn show(node: Option<&Node>) -> String {
    match node {
        Some(Node::Scalar(s)) => s.clone(),
        Some(Node::Seq(_)) => "（一覧）".to_string(),
        Some(Node::Map(_)) => "（表）".to_string(),
        Some(Node::Null) | None => "（無い）".to_string(),
    }
}

pub(crate) fn in_enum(node: Option<&Node>, values: &[&str]) -> bool {
    scalar(node).is_some_and(|s| values.contains(&s))
}

/// 前後の空白を落として空でない（null と空白だけの文字列が空）。
pub(crate) fn non_empty(node: Option<&Node>) -> bool {
    match node {
        None | Some(Node::Null) => false,
        Some(Node::Scalar(s)) => !s.trim().is_empty(),
        Some(_) => true,
    }
}

pub(crate) fn check_date(kind: &str, at: &str, node: Option<&Node>, report: &mut Report) {
    if !scalar(node).is_some_and(is_date) {
        report.violation(kind, format!("{at}「{}」が年-月-日でない", show(node)));
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

/// ADR- の後に 1〜9 で始まる数字列だけ（id_pattern）。
fn is_adr_id(s: &str) -> bool {
    s.strip_prefix("ADR-")
        .is_some_and(|n| digits(n) && !n.starts_with('0'))
}

/// basis の各項の id の形（条・要件・rules 行・判断の記録の全体一致）。
fn is_basis_id(s: &str) -> bool {
    let article = ["P-", "A-", "N-"].iter().any(|p| {
        s.strip_prefix(p)
            .is_some_and(|rest| match rest.split_once('.') {
                Some((n, sub)) => digits(n) && digits(sub),
                None => digits(rest),
            })
    });
    let req = ["FR", "NFR", "AC", "CON", "GOAL"]
        .iter()
        .any(|p| s.strip_prefix(p).is_some_and(digits));
    let row = ["R-", "D-"]
        .iter()
        .any(|p| s.strip_prefix(p).is_some_and(digits));
    article || req || row || is_adr_id(s)
}

/// 小文字の英字 1 字 + 数字 1 字 + 「-」+ 小文字の英字か数字 1 字 の並びを含む（ruling_pattern の search）。
pub(crate) fn has_ledger_id(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    chars.windows(4).any(|w| {
        w[0].is_ascii_lowercase()
            && w[1].is_ascii_digit()
            && w[2] == '-'
            && (w[3].is_ascii_lowercase() || w[3].is_ascii_digit())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_shapes_are_scanned_without_regex() {
        for ok in [
            "P-1", "A-2.3", "FR12", "NFR3", "GOAL2", "R-14", "D-1", "ADR-4",
        ] {
            assert!(is_basis_id(ok), "{ok}");
        }
        for ng in ["P-", "P-1.", "ADR-0047", "FR", "R-1.2", "X-1", "p-1"] {
            assert!(!is_basis_id(ng), "{ng}");
        }
        assert!(has_ledger_id("f2-648.2 notes"));
        assert!(!has_ledger_id("F2-648"));
        assert!(is_date("2026-09-17"));
        assert!(!is_date("2026-9-17"));
    }

    #[test]
    fn floor_values_and_numbers_are_read_through_the_floor() {
        assert_eq!(
            floor_val(&["anchor", "root_digest"]),
            Some("acb52acd04b5d3a1feaf9ad5f0138f7614ce31964144b46ead914bde86e866ed")
        );
        assert_eq!(floor_val(&["anchor", "first_version"]), Some("v1.0"));
        assert_eq!(floor_val(&["amends_entry", "empty_marker"]), Some("（空）"));
        assert_eq!(floor_val(&["anchor", "file_keys"]), None);
        assert_eq!(floor_num(&["options_rule", "min"]), Some(2));
        assert_eq!(floor_num(&["options_rule", "adopted"]), Some(1));
        assert_eq!(floor_num(&["owner"]), None);
    }

    #[test]
    fn floor_diff_compares_literally() {
        let doc = yaml::parse("min: '2'\nadopted: 2.0\nextra: x\n").unwrap();
        let mut out = Vec::new();
        floor_diff(
            &doc.root,
            &Floor::Map(&[
                ("min", Floor::Num(2)),
                ("adopted", Floor::Num(1)),
                ("x", Floor::Strs(&[])),
            ]),
            "options_rule",
            &mut out,
        );
        assert_eq!(
            out,
            [
                "options_rule.adopted",
                "options_rule.extra（未知の欄＝機械が読まない欄は *_note で終える）",
                "options_rule.x（欠落）"
            ]
        );
    }
}
