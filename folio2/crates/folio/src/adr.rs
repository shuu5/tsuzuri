//! `folio check` の判断の記録（`adr/`）の欄の決まりの検査（便 5・docs/design/delivery-5.md §1）。
//! day-1 の床 `scripts/check_draft.py` の adr の節のうち、判断の記録の file と欄の決まりの file だけで閉じる検査を同じ式で写す。
//! 憲法・rules・anchor と突き合わせる検査（amends の対象の実在・amended_by との双方向・対話面の行の実在・
//! 判断の記録の id の参照・本文の英字語）は便 6 の `link.rs` で、読んだ欄の決まりと判断の記録（`Adr`）と床の定数（`floor_strs`）を渡す。
//! 凍結 anchor の列そのものは便 7。
//! 欄の決まりの閾値・値域・置き場は床の定数（`FLOOR`）で持ち、adr/schema.yaml の schema 節はその写し（N-3.1）。
//! 便 45（ADR-9）から schema 節は生成区間で、`folio schema --write` が `FLOOR` から導出する＝説明の注（`_note` で終わる欄）も
//! `FLOOR` の側に file の順と字面のまま持つ（床の突き合わせは注を読まない）。床の機械（木の型・突き合わせ）は `schema.rs`。
//! パターンの文字列は定数として字面で持つだけで、形の判定は字の走査で行う（正規表現は使わない）。
//! 任意の図の節（figures・便 33）は設計ノートの図の節と同じ形（欄の集合・型は部品目録の一覧・spec は表・refs は
//! basis と同じ id の形・図の id は 1 本の記録の中で一意）を見る。行き先の解決は床では数えない（面が「まだ分からない」で表す）。
//! 床の定数（欄の集合の型・欄の決まりの定数・`FLOOR`）は便 113 で `floor_adr.rs`（層 1）へ降ろした。床の読み口はここに残す。

use std::fs;
use std::path::Path;

use crate::catalog::FigureType;
use crate::floor::{Floor, floor_diff_for, strip_notes};
use crate::floor_adr::{
    AMENDS_ENTRY, APPROVAL, APPROVER, EFFECTIVE_STATUS, FIGURE_ENTRY, FIGURE_TYPE_ENUM_REF, FLOOR,
    GRILL, ID_PATTERN, Keys, NON_EMPTY, OPTION, OPTIONS_ADOPTED, OPTIONS_MIN, OWNER, PRODUCED,
    RECORD, RETREAT, RETREAT_KIND, REVISE_KIND, REVISES, REVISES_ENTRY, ROOT_DIGESTS,
    RULING_PATTERN, STATUS, SURFACE, VERDICT,
};
use crate::verdict::Report;
use crate::yaml::{self, Node};

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

/// 列の根の表（`ROOT_DIGESTS`・便 121・ADR-16 決定 (2)(ア)）を憲法の名で引く。名が無いか表に無ければ None。
/// 床の列の照らし（`anchor.rs`）と凍結の命令（`freeze.rs`）が共有する。
pub(crate) fn root_digest(name: Option<&str>) -> Option<&'static str> {
    ROOT_DIGESTS
        .iter()
        .find(|(k, _)| Some(*k) == name)
        .map(|(_, v)| *v)
}

/// 置き場の憲法の名（`<dir>/constitution.yaml` の meta.id）。読めなければ理由の字（便 121）。
/// 欄の決まりの写しの突き合わせ（`check_adr`）と `folio schema` が共有する。
pub(crate) fn place_name(dir: &Path) -> Result<String, String> {
    const FILE: &str = "constitution.yaml";
    let path = dir.join(FILE);
    if path.is_symlink() || !path.is_file() {
        return Err(format!("{FILE}: 読めない（symlink・file でない・不在）＝meta.id を読めない"));
    }
    let doc = read(&path).map_err(|e| format!("{FILE}: 読めない: {e}＝meta.id を読めない"))?;
    doc.root
        .get("meta")
        .and_then(|m| m.get("id"))
        .and_then(Node::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("{FILE}: meta.id が無い（字でない）"))
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
    // 列の根の表は置き場の名の行だけと突き合わせる（便 121・名が読めなければ空の表）
    let name = place_name(dir).ok();
    let mut drift = Vec::new();
    floor_diff_for(
        &strip_notes(schema.get("schema").unwrap_or(&Node::Null)),
        &FLOOR,
        name.as_deref(),
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

    // 帰結の欄（便 92）: 条でない id の形・自分と basis に無い id。実在は link.rs の網が数える
    match present(d, PRODUCED) {
        None => {}
        Some(Node::Seq(items)) => {
            let basis = d.get("basis").and_then(Node::as_seq).unwrap_or(&[]);
            for p in items {
                let v = p.as_str();
                if !v.is_some_and(|s| is_basis_id(s) && !is_article_id(s)) {
                    report.violation(
                        "adr",
                        format!(
                            "{id}: {PRODUCED}「{}」が id の形でない（要件・rules 行・判断の記録だけ・P-5.2）",
                            show(Some(p))
                        ),
                    );
                } else if v == Some(id) || basis.iter().any(|b| b.as_str() == v) {
                    report.violation(
                        "adr",
                        format!(
                            "{id}: {PRODUCED}「{}」が自分の id か根拠（basis）に在る",
                            show(Some(p))
                        ),
                    );
                }
            }
        }
        Some(_) => report.violation("adr", format!("{id}: {PRODUCED} が一覧でない")),
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

    check_revises(id, d, report);

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

    check_figures(id, d, report);
}

/// (d) 任意の改訂の欄（便 101）。欄の集合・4 欄の非空・target は自分でない判断の記録の id・kind の値域・
/// target と decision の対は 1 本の記録の中で一意。実在は link.rs の網が数える。
fn check_revises(id: &str, d: &Node, report: &mut Report) {
    let items: &[Node] = match present(d, REVISES) {
        None => return,
        Some(Node::Seq(items)) => items,
        Some(_) => {
            report.violation("adr", format!("{id}: {REVISES} が一覧でない"));
            return;
        }
    };
    let mut seen: Vec<(&str, &str)> = Vec::new();
    for (i, e) in items.iter().enumerate() {
        let at = format!("{id}.{REVISES}[{i}]");
        if !check_keys("adr", &at, e, &REVISES_ENTRY, report) {
            continue;
        }
        for k in REVISES_ENTRY.required {
            if !non_empty(e.get(k)) {
                report.violation("adr", format!("{at}.{k} が空"));
            }
        }
        let target = scalar(e.get("target"));
        if target.is_some_and(|t| !t.trim().is_empty())
            && !target.is_some_and(|t| is_adr_id(t) && t != id)
        {
            report.violation(
                "adr",
                format!(
                    "{at}.target が判断の記録の id でない（自分の id も書かない）: {}",
                    show(e.get("target"))
                ),
            );
        }
        if non_empty(e.get("kind")) && !in_enum(e.get("kind"), REVISE_KIND) {
            report.violation(
                "adr",
                format!("{at}: kind「{}」が値域でない", show(e.get("kind"))),
            );
        }
        if let (Some(t), Some(dec)) = (target, scalar(e.get("decision"))) {
            if seen.contains(&(t, dec)) {
                report.violation(
                    "adr",
                    format!("{at}: {t} の decision「{dec}」が 2 行に在る（対は一意）"),
                );
            } else {
                seen.push((t, dec));
            }
        }
    }
}

/// (d) 任意の図の節（便 33）。欄の集合・id と caption の非空・型は部品目録の一覧・spec は表・refs は basis と同じ
/// id の形・図の id は 1 本の記録の中で一意。行き先の解決は数えない。
fn check_figures(id: &str, d: &Node, report: &mut Report) {
    let figures: &[Node] = match present(d, "figures") {
        None => &[],
        Some(Node::Seq(items)) => items,
        Some(_) => {
            report.violation("adr", format!("{id}: figures が一覧でない"));
            &[]
        }
    };
    let mut seen: Vec<&str> = Vec::new();
    for f in figures {
        let at = format!("{id}.figures[{}]", show(f.get("id")));
        if !check_keys("adr", &at, f, &FIGURE_ENTRY, report) {
            continue;
        }
        // 欠落は check_keys が数えてあるので、非空は在る欄だけ見る
        for k in ["id", "caption"] {
            if f.get(k).is_some() && !non_empty(f.get(k)) {
                report.violation("adr", format!("{at}.{k} が空"));
            }
        }
        if let Some(t) = f.get("type")
            && !t
                .as_str()
                .is_some_and(|v| FigureType::from_name(v).is_some())
        {
            report.violation(
                "adr",
                format!(
                    "{at}: 図の型「{}」が部品目録の一覧に無い（{FIGURE_TYPE_ENUM_REF}）",
                    show(Some(t))
                ),
            );
        }
        if let Some(spec) = f.get("spec")
            && !matches!(spec, Node::Map(_))
        {
            report.violation("adr", format!("{at}: spec が表でない"));
        }
        match present(f, "refs") {
            None => {}
            Some(Node::Seq(items)) => {
                for r in items {
                    if !r.as_str().is_some_and(is_basis_id) {
                        report.violation(
                            "adr",
                            format!(
                                "{at}: refs「{}」が id の形（条・要件・rules 行・判断の記録）でない",
                                show(Some(r))
                            ),
                        );
                    }
                }
            }
            Some(_) => report.violation("adr", format!("{at}: refs が一覧でない")),
        }
        if let Some(fid) = scalar(f.get("id")) {
            if seen.contains(&fid) {
                report.violation(
                    "adr",
                    format!("{id}: 図の id「{fid}」が重複（1 本の記録の中で一意）"),
                );
            } else {
                seen.push(fid);
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

/// basis の各項の id の形（条・要件・rules 行・判断の記録の全体一致）。要件書の図の refs（`check.rs`・便 34）も同じ判定を呼ぶ。
pub(crate) fn is_basis_id(s: &str) -> bool {
    let article = is_article_id(s);
    let req = ["FR", "NFR", "AC", "CON", "GOAL"]
        .iter()
        .any(|p| s.strip_prefix(p).is_some_and(digits));
    let row = ["R-", "D-"]
        .iter()
        .any(|p| s.strip_prefix(p).is_some_and(digits));
    article || req || row || is_adr_id(s)
}

/// 条と規範文の id（P-n / A-n / N-n と P-n.m の形）。帰結の欄 produced は受けない（便 92）。
fn is_article_id(s: &str) -> bool {
    ["P-", "A-", "N-"].iter().any(|p| {
        s.strip_prefix(p)
            .is_some_and(|rest| match rest.split_once('.') {
                Some((n, sub)) => digits(n) && digits(sub),
                None => digits(rest),
            })
    })
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
        // 列の根は憲法の名で引く表（便 121）＝値の読み口では読めない
        assert_eq!(
            root_digest(Some("folio2-constitution")),
            Some("acb52acd04b5d3a1feaf9ad5f0138f7614ce31964144b46ead914bde86e866ed")
        );
        assert_eq!(root_digest(Some("fixture-constitution")), None);
        assert_eq!(root_digest(None), None);
        assert_eq!(floor_val(&["anchor", "root_digests"]), None);
        assert_eq!(floor_val(&["anchor", "first_version"]), Some("v1.0"));
        assert_eq!(floor_val(&["amends_entry", "empty_marker"]), Some("（空）"));
        assert_eq!(floor_val(&["anchor", "file_keys"]), None);
        assert_eq!(floor_num(&["options_rule", "min"]), Some(2));
        assert_eq!(floor_num(&["options_rule", "adopted"]), Some(1));
        assert_eq!(floor_num(&["owner"]), None);
    }

    /// 床の突き合わせは FLOOR の説明の注（`_note`）を読まない＝注を持つ FLOOR と注の無い写しの差は 0。
    #[test]
    fn floor_notes_are_outside_the_diff() {
        let Floor::Map(fields) = &FLOOR else {
            unreachable!()
        };
        let notes = fields.iter().filter(|(k, _)| k.ends_with("_note")).count();
        assert_eq!(notes, 24);
        let mut out = Vec::new();
        crate::floor::floor_diff(&strip_notes(&Node::Map(Vec::new())), &FLOOR, "", &mut out);
        // 空の写し = 値の欄が全部（欠落）・注は 1 本も立たない
        assert_eq!(out.len(), fields.len() - notes, "{out:?}");
        assert!(out.iter().all(|p| p.ends_with("（欠落）")), "{out:?}");
        assert!(out.iter().all(|p| !p.contains("_note")), "{out:?}");
    }

    /// 床の木の導出は凍結 anchor（設計判断の席が独立の実装で組んだ・P-10.1）と byte 一致（便 58 §1 (e)3・
    /// `ceiling.rs` / `rules.rs` / `note.rs` の同名の歯と同じ形で `schema.rs` の pub の `derive` を呼ぶ）。
    /// 便 121 から名つきの導出（folio2 の憲法の名＝列の根の表の folio2 の行だけを写す）。
    #[test]
    fn adr_floor_derives_the_frozen_anchor_byte_for_byte() {
        let anchor = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/schema/adr-region.txt"
        ))
        .unwrap();
        assert_eq!(
            crate::floor::derive_for(&FLOOR, Some("folio2-constitution")),
            anchor
        );
    }

    /// 列の根の表はちょうど 2 行（便 134 §1 (c)1）: folio2 の行が先・tsuzuri の行が 2 行目で、鍵と digest の全桁は
    /// 歯の中の手で写した字（凍結 anchor・P-10.1）。改名の前の名 scribe3-constitution からは何も引かない。
    /// 行を足す次の便はこの字も同じ要求で直す。
    #[test]
    fn f134_the_root_table_holds_folio2_and_tsuzuri() {
        const FOLIO2: (&str, &str) = (
            "folio2-constitution",
            "acb52acd04b5d3a1feaf9ad5f0138f7614ce31964144b46ead914bde86e866ed",
        );
        const TSUZURI: (&str, &str) = (
            "tsuzuri-constitution",
            "35eb6b369f0504167571a27b50c950e1361609d9b19b71e0f1e9de832f8c5356",
        );
        assert_eq!(ROOT_DIGESTS, [FOLIO2, TSUZURI]);
        assert_eq!(root_digest(Some(TSUZURI.0)), Some(TSUZURI.1));
        assert_eq!(root_digest(Some(FOLIO2.0)), Some(FOLIO2.1));
        assert_eq!(root_digest(Some("scribe3-constitution")), None);
    }

    /// 承認者の値域は 持ち主・planner 席・orchestrator 席 の 3 つ（席の呼び名の裁定 2026-09-20・便 58 §1 (a)1・
    /// planner 席 は凍結の場合と過去の記録のために残す）。
    #[test]
    fn adr_floor_approver_enum_has_three_values() {
        assert_eq!(APPROVER, ["持ち主", "planner 席", "orchestrator 席"]);
    }
}
