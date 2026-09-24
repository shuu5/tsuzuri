//! `folio check` — design-intent の正本 7 file（憲法・rules・語彙・要件書・入口・相談窓口・天井）の形の床（FR5 / FR9）。
//! 数えるのは 重複キー・未知の節・欄の非空（便 0）と、参照 id の解決・rules 行の逆参照・憲法の件数（便 1・refs）と、語彙の検査 R-9（便 4・vocab）と、
//! 判断の記録（adr/）の欄の決まり（便 5・adr）と、判断の記録と正本 4 file・凍結 anchor の列の突き合わせ（便 6・link）と、
//! 凍結 anchor の列のうち版管理を見ない部分（便 7・anchor）と、入口の正本の形（便 12・entrance）と、
//! 相談窓口の正本の形（便 18・intake）と、設計ノートの正本の形（便 23・note）と、天井の正本の形（便 37・ceiling）と、
//! 憲法の条の値域を持つ欄の値（便 55・在る欄だけ）と、置き場の憲法の値域が組み立てた値域の部分集合か（便 122・FR25・
//! 条の値は置き場の値域で引く・部分集合でない鍵と引けない鍵は「まだ分からない」）と、憲法の meta・前文・条・規範文・mechanism と
//! 規則の表の行の未知の欄・mechanism の形の崩れ（便 128・一覧は組み立てた憲法の正本と規則の表の床の定数から）。
//! 参照 id・語彙 R-9・判断の記録との突き合わせ・凍結 anchor・読み物の生成は今も憲法・rules・語彙・要件書の 4 本だけを受ける。
//! 読めない・型が違う・節の決まりが読めない は「まだ分からない」（合格にしない）。

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::adr;
use crate::anchor;
use crate::catalog::FigureType;
use crate::ceiling;
use crate::constitution_enums as ce;
use crate::entrance;
use crate::floor::Floor;
use crate::ids;
use crate::intake;
use crate::link;
use crate::mentions;
use crate::note;
use crate::phase::{Flag, State};
use crate::refs;
use crate::rules;
use crate::verdict::Report;
use crate::vocab;
use crate::yaml::{self, Node};

/// 正本 7 file（読む順）。
pub const FILES: [&str; 7] = [
    "constitution",
    "rules",
    "vocabulary",
    "srs",
    "index",
    "intake",
    "ceiling",
];

/// 要件書の節の閉じた一覧（正本は床の定数・file の schema 節はその写し）。figures は任意の図の節（便 34・FR15）。
/// 末尾の schema は生成区間（便 77・ADR-11 決定 (4)⑤）。scope_m3 は M3 の範囲の節（便 117・判断の記録 ADR-16 決定 (1)(7)①・
/// 節の中身は要件書の版上げが書く）。
pub const SRS_TOP_LEVEL: [&str; 18] = [
    "meta",
    "goals",
    "scope",
    "scope_m1",
    "scope_m3",
    "actors",
    "outputs",
    "rail",
    "verdicts",
    "requirements",
    "nonfunctional",
    "acceptance",
    "not_frozen",
    "constraints",
    "sources",
    "glossary_pointer",
    "figures",
    "schema",
];

/// 要件書の図の節の行の欄（判断の記録の figures.entry と同じ形・便 34）。
const SRS_FIGURE_REQUIRED: [&str; 4] = ["id", "type", "caption", "spec"];
const SRS_FIGURE_OPTIONAL: [&str; 2] = ["refs", "note"];

/// 要件書の schema 節（生成区間）の床の木（便 77 §1 (a)）。欄の順と字面は凍結 anchor
/// tests/fixtures/schema/srs-region.txt のとおり。床（`check_srs`）は生成区間の中身をこの木と突き合わせない。
pub(crate) const SRS_FLOOR: Floor = Floor::Map(&[
    ("top_level", Floor::Strs(&SRS_TOP_LEVEL)),
    (
        "top_level_note",
        Floor::Val(
            "最上位の節の閉じた一覧（ほかの節は床が落とす）。schema のほかの節は人が書き、schema は生成区間",
        ),
    ),
    (
        "figures",
        Floor::Map(&[(
            "entry",
            Floor::Map(&[
                ("required", Floor::Strs(&SRS_FIGURE_REQUIRED)),
                ("optional", Floor::Strs(&SRS_FIGURE_OPTIONAL)),
            ]),
        )]),
    ),
    (
        "figures_note",
        Floor::Val(
            "要件書の図の節の行の欄（判断の記録と設計ノートの欄の決まりの figures.entry と同じ形）。型（type）の値域は部品目録が持つ",
        ),
    ),
    // 要件の行の欄の閉じた一覧の写し（便 86 §1 (b)）。葉は check_srs_item が読む定数そのもの
    (
        "requirement_row",
        Floor::Map(&[
            ("required_text", Floor::Strs(&SRS_ITEM_TEXT)),
            ("required_list", Floor::Strs(&SRS_ITEM_LIST)),
            ("optional", Floor::Strs(&SRS_ITEM_OPTIONAL)),
            (
                SRS_ITEM_VERIFY,
                Floor::Map(&[
                    ("required_text", Floor::Strs(&SRS_ITEM_VERIFY_TEXT)),
                    ("required_list", Floor::Strs(&SRS_ITEM_VERIFY_LIST)),
                ]),
            ),
        ]),
    ),
    (
        "requirement_row_note",
        Floor::Val(
            "要件の行（requirements と nonfunctional）の欄の閉じた一覧。required_text は非空の字・required_list は在ること（空の一覧でよい）・optional は任意・verify は表",
        ),
    ),
]);

/// 要件の行（requirements / nonfunctional）の欄（便 75・面の生成器 face_srs.rs の item_row が読む欄から導く）。
/// 字の欄は非空・一覧の欄は在ること（空の一覧は通る）・verify は表で中の字の欄は非空・ac は一覧。
const SRS_ITEM_TEXT: [&str; 7] = [
    "id", "title", "pattern", "strength", "when", "shall", "plain",
];
const SRS_ITEM_LIST: [&str; 3] = ["goals", "basis", "figures"];
const SRS_ITEM_VERIFY_TEXT: [&str; 2] = ["method", "how"];
/// adrs は判断の記録の id だけを受ける一覧（便 90・ADR-13 決定 (3-b)（ア））。
const SRS_ITEM_OPTIONAL: [&str; 4] = ["milestone", "rules", "adrs", "note"];
/// adrs の欄の字（中身を見る唯一の任意の欄）。
const SRS_ITEM_ADRS: &str = "adrs";
/// verify の欄の字と、その中の一覧の欄（便 86 §1 (a)・字面の写しを閉じて生成区間が集合の全部を覆う）。
const SRS_ITEM_VERIFY: &str = "verify";
const SRS_ITEM_VERIFY_LIST: [&str; 1] = ["ac"];

/// 語彙の節の閉じた一覧（同上）。末尾の schema は生成区間（便 77）。
pub const VOCABULARY_TOP_LEVEL: [&str; 4] = ["terms", "field_terms", "identifiers", "schema"];

/// 語彙の schema 節（生成区間）の床の木（便 77 §1 (a)）。欄の順と字面は凍結 anchor tests/fixtures/schema/vocabulary-region.txt のとおり。
pub(crate) const VOCABULARY_FLOOR: Floor = Floor::Map(&[
    ("top_level", Floor::Strs(&VOCABULARY_TOP_LEVEL)),
    (
        "top_level_note",
        Floor::Val(
            "最上位の節の閉じた一覧（ほかの節は床が落とす）。terms と field_terms と identifiers は人が書き、schema は生成区間",
        ),
    ),
]);

struct Sources {
    constitution: Node,
    rules: Node,
    vocabulary: Node,
    srs: Node,
    index: Node,
    intake: Node,
    ceiling: Node,
}

/// 凍結の後始末の材料（口の検査が残したもの・入口が口を出た直後に後始末へ渡す）。
/// 判断の記録が読めなかったときは 3 つとも無い。
pub struct Materials {
    /// 凍結 anchor の列の検査が残した列の結果。
    pub state: Option<State>,
    /// 読めた判断の記録。
    pub adr: Option<adr::Adr>,
    /// id の消失と改番の検査が読んだ現行の id。
    pub ids: Option<ids::Current>,
}

/// `dir` の正本 7 file を検査する。`flag` は便 9 の旗（検査の式は変えず、列の結果を材料に載せて返す）。
pub fn check_dir(dir: &Path, flag: Flag) -> (Report, Materials) {
    let mut report = Report::default();
    let mut state = None;
    let mut adr_records = None;
    let mut ids_cur = None;
    match load_all(dir, &mut report) {
        Some(src) => {
            let history = anchor::history_ids(dir);
            let range = place_range(&src.constitution, &mut report);
            check_constitution(&src.constitution, &range, &mut report);
            check_rules(&src.rules, &mut report);
            check_vocabulary(&src.vocabulary, &mut report);
            check_srs(&src.srs, &mut report);
            entrance::check_entrance(&src.index, &src.vocabulary, &mut report);
            intake::check_intake(&src.intake, &src.index, &src.vocabulary, &mut report);
            ceiling::check_ceiling(&src.ceiling, &src.vocabulary, &mut report);
            refs::check_refs(
                &src.constitution,
                &src.rules,
                &src.vocabulary,
                &src.srs,
                &history,
                &mut report,
            );
            vocab::check_vocab(
                &src.constitution,
                &src.rules,
                &src.vocabulary,
                &src.srs,
                &mut report,
            );
            if let Some(records) = adr::check_adr(dir, &mut report) {
                link::check_link(
                    dir,
                    &src.constitution,
                    &src.rules,
                    &src.vocabulary,
                    &src.srs,
                    &records,
                    &mut report,
                );
                state = anchor::check_anchor(dir, &records, &history, flag, &mut report);
                // 要件・判断・受入基準の id の消失と改番（便 88）
                ids_cur = Some(ids::check_ids(
                    dir,
                    &src.srs,
                    &records.records,
                    flag,
                    &mut report,
                ));
                adr_records = Some(records);
            }
            note::check_note(
                dir,
                &src.constitution,
                &src.rules,
                &src.srs,
                adr_records.as_ref(),
                &mut report,
            );
            // 散文の言及の歯 R-17（便 93）。判断の記録を読めたときだけ数える
            if let Some(records) = adr_records.as_ref() {
                mentions::check_mentions(
                    dir,
                    &[
                        ("constitution.yaml", &src.constitution),
                        ("rules.yaml", &src.rules),
                        ("vocabulary.yaml", &src.vocabulary),
                        ("srs.yaml", &src.srs),
                        ("index.yaml", &src.index),
                        ("intake.yaml", &src.intake),
                        ("ceiling.yaml", &src.ceiling),
                    ],
                    records,
                    &mut report,
                );
            }
        }
        None => debug_assert!(!report.unknowns.is_empty()),
    }
    let materials = Materials {
        state,
        adr: adr_records,
        ids: ids_cur,
    };
    (report, materials)
}

fn load_all(dir: &Path, report: &mut Report) -> Option<Sources> {
    if dir.is_symlink() {
        report.unknown(format!(
            "design-intent 自体が symlink（{}）＝認めない",
            dir.display()
        ));
        return None;
    }
    if !dir.is_dir() {
        report.unknown(format!("design-intent が dir でない: {}", dir.display()));
        return None;
    }
    let mut roots: Vec<Option<Node>> = FILES.iter().map(|name| load(dir, name, report)).collect();
    // 一覧の末尾から取り出す＝天井を先に取り、6 本の割り当てはずらさない
    let ceiling = roots.pop()??;
    let intake = roots.pop()??;
    let index = roots.pop()??;
    let srs = roots.pop()??;
    let vocabulary = roots.pop()??;
    let rules = roots.pop()??;
    let constitution = roots.pop()??;
    Some(Sources {
        constitution,
        rules,
        vocabulary,
        srs,
        index,
        intake,
        ceiling,
    })
}

fn load(dir: &Path, name: &str, report: &mut Report) -> Option<Node> {
    let file = format!("{name}.yaml");
    let path = dir.join(&file);
    if path.is_symlink() {
        report.unknown(format!("{file}: symlink は認めない"));
        return None;
    }
    if !path.exists() {
        report.unknown(format!("{file}: 正本が無い"));
        return None;
    }
    if !path.is_file() {
        report.unknown(format!("{file}: file でない"));
        return None;
    }
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            report.unknown(format!("{file}: 読めない: {e}"));
            return None;
        }
    };
    let doc = match yaml::parse(&text) {
        Ok(d) => d,
        Err(e) => {
            report.unknown(format!("{file}: parse できない: {e}"));
            return None;
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
        return None;
    }
    Some(doc.root)
}

/// 最上位の節が閉じた一覧に在るか。
pub(crate) fn unknown_sections(file: &str, root: &Node, allowed: &[&str], report: &mut Report) {
    for (key, _) in root.as_map().unwrap_or_default() {
        if !allowed.contains(&key.as_str()) {
            report.violation("未知の節", format!("{file}: 未知の節「{key}」"));
        }
    }
}

/// 節の決まり（`schema.top_level`）を正本から読む。読めなければ「まだ分からない」。憲法だけが使う（ADR-11 決定 (3)＝file が正本・
/// 規則の表は便 51 から床の定数）。
fn schema_top_level<'a>(file: &str, root: &'a Node, report: &mut Report) -> Option<Vec<&'a str>> {
    let list = root
        .get("schema")
        .and_then(|s| s.get("top_level"))
        .and_then(Node::as_seq);
    let names: Option<Vec<&str>> = list.and_then(|l| l.iter().map(Node::as_str).collect());
    if names.is_none() {
        report.unknown(format!(
            "{file}: schema.top_level（節の閉じた一覧）が読めない"
        ));
    }
    names
}

/// 行の一覧の節を読む。無い・null は 0 行、一覧でない・行が表でないは「まだ分からない」。
pub(crate) fn rows<'a>(
    file: &str,
    root: &'a Node,
    section: &str,
    report: &mut Report,
) -> Vec<&'a Node> {
    match root.get(section) {
        None | Some(Node::Null) => Vec::new(),
        Some(Node::Seq(items)) if items.iter().all(|n| n.as_map().is_some()) => {
            items.iter().collect()
        }
        Some(_) => {
            report.unknown(format!("{file}: {section} が表の一覧でない"));
            Vec::new()
        }
    }
}

/// 行の欄が空でないか。
pub(crate) fn non_empty(
    file: &str,
    where_: &str,
    row: &Node,
    fields: &[&str],
    report: &mut Report,
) {
    for field in fields {
        if row.get(field).is_none_or(Node::is_blank) {
            report.violation("欄の非空", format!("{file}: {where_} の {field} が空"));
        }
    }
}

pub(crate) fn row_id(row: &Node) -> String {
    row.get("id")
        .and_then(Node::as_str)
        .unwrap_or("?")
        .to_string()
}

/// 行 id の重複（行の表のキー）を数える。
pub(crate) fn duplicate_ids<'a>(
    file: &str,
    rows: impl IntoIterator<Item = &'a Node>,
    report: &mut Report,
) {
    let mut seen = HashSet::new();
    for row in rows {
        let id = row_id(row);
        if id != "?" && !seen.insert(id.clone()) {
            report.violation("重複キー", format!("{file}: 行 id「{id}」が重複"));
        }
    }
}

/// 条 id の形（P・A・N のどれか + 「-」+ 数字列 の全体一致）。
fn article_id_form(id: &str) -> bool {
    id.strip_prefix(['P', 'A', 'N'])
        .and_then(|r| r.strip_prefix('-'))
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// 規範文 id の重複を床の字面（種別 schema・重複した id の一覧を末尾に）で数える。
fn duplicate_statement_ids<'a>(rows: impl IntoIterator<Item = &'a Node>, report: &mut Report) {
    let mut seen = HashSet::new();
    let mut dups: Vec<String> = Vec::new();
    for row in rows {
        let id = row_id(row);
        if id != "?" && !seen.insert(id.clone()) && !dups.contains(&id) {
            dups.push(id);
        }
    }
    if !dups.is_empty() {
        report.violation(
            "schema",
            format!(
                "規範文 id が重複（同じ id の規範文が 2 本以上＝欄単位の消し込みが id で潰れる・P-7.1）: [{}]",
                dups.join(", ")
            ),
        );
    }
}

/// 置き場の憲法の値域（鍵の名 → 値の列・file の順・同じ値は 1 つに）。条の値はこの表で引く（便 122）。
type PlaceRange = HashMap<String, Vec<String>>;

/// 置き場の憲法の値域の節（schema.enums）の各鍵が、組み立てた版（`constitution_enums::ENUMS`）の同じ鍵の値の部分集合かを
/// 数える（便 122・FR25・ADR-16 決定 (2)(ウ)・順と重複は問わない）。部分集合でない鍵（組み立てた版に無い値・無い鍵）と、
/// 引けない鍵（組み立てた版の鍵が節に無い・値が文字列の一覧でない・節が表でない）は鍵ごとに「まだ分からない」（測れない）1 件。
/// 違反は出さない。返す表は文字列の一覧の鍵だけを持つ（部分集合でない鍵も入れる＝置き場の値域にも無い値は違反のまま）。
/// 値域を置き場ごとに広げる口は持たない（N-3.1）。
fn place_range(root: &Node, report: &mut Report) -> PlaceRange {
    const FILE: &str = "constitution.yaml";
    let mut range = PlaceRange::new();
    let Some(section) = root
        .get("schema")
        .and_then(|s| s.get("enums"))
        .and_then(Node::as_map)
    else {
        report.pending(format!(
            "{FILE}: schema.enums（置き場の憲法の値域の節）が表でない＝条の値を置き場の値域で引けない（FR25）"
        ));
        return range;
    };
    for (key, body) in section {
        let values: Option<Vec<&str>> = body
            .as_seq()
            .and_then(|l| l.iter().map(Node::as_str).collect());
        let Some(values) = values else {
            report.pending(format!(
                "{FILE}: schema.enums.{key} が文字列の一覧でない＝条の値を置き場の値域で引けない（FR25）"
            ));
            continue;
        };
        let mut list: Vec<String> = Vec::new();
        for v in values {
            if !list.iter().any(|x| x == v) {
                list.push(v.to_string());
            }
        }
        match ce::ENUMS.iter().find(|(k, _)| *k == key.as_str()) {
            None => report.pending(format!(
                "{FILE}: schema.enums.{key}: 組み立て時の値域に無い値がある（組み立てた版に無い鍵・値域を置き場ごとに広げる口は無い・FR25）"
            )),
            Some((_, built)) => {
                let outside: Vec<String> = list
                    .iter()
                    .filter(|v| !built.contains(&v.as_str()))
                    .map(|v| format!("「{v}」"))
                    .collect();
                if !outside.is_empty() {
                    report.pending(format!(
                        "{FILE}: schema.enums.{key}: 組み立て時の値域に無い値がある（{}・値域を置き場ごとに広げる口は無い・FR25）",
                        outside.join("・")
                    ));
                }
            }
        }
        range.insert(key.clone(), list);
    }
    for (key, _) in ce::ENUMS {
        if !section.iter().any(|(k, _)| k.as_str() == key) {
            report.pending(format!(
                "{FILE}: schema.enums に鍵 {key} が無い＝条の {key} の値を置き場の値域で引けない（FR25）"
            ));
        }
    }
    range
}

/// 表の欄のうち閉じた一覧に無いものを、1 つにつき種別 未知の欄 の違反 1 件にする（便 128・N-3.1・例外の口なし）。
/// 表でない holder は黙る（形の崩れは `map_holder` が数える）。
fn closed_fields(file: &str, at: &str, row: &Node, allowed: &[&str], report: &mut Report) {
    for (key, _) in row.as_map().unwrap_or_default() {
        if !allowed.contains(&key.as_str()) {
            report.violation("未知の欄", format!("{file}: {at} の未知の欄「{key}」"));
        }
    }
}

/// 鍵は在るが値が表でない holder（字・一覧・数・null）を種別 schema の違反 1 件にする（便 128・無効化の旗の形を塞ぐ）。
/// 表なら返し、鍵が無い・表でないは None（表でない holder の中の欄は数えない）。
fn map_holder<'a>(file: &str, at: &str, holder: Option<&'a Node>, report: &mut Report) -> Option<&'a Node> {
    let node = holder?;
    if node.as_map().is_some() {
        return Some(node);
    }
    report.violation("schema", format!("{file}: {at} が表でない"));
    None
}

/// mechanism 1 つの形（便 128 (b)）: 表でない・導出した必須の欄（kind・live）の欠け（null も欠け）・閉じた一覧に無い欄。
fn check_mechanism(file: &str, at: &str, holder: Option<&Node>, report: &mut Report) {
    let Some(m) = map_holder(file, at, holder, report) else {
        return;
    };
    for key in ce::MECHANISM_REQUIRED {
        if matches!(m.get(key), None | Some(Node::Null)) {
            report.violation("schema", format!("{file}: {at} の必須の欄 {key} が無い"));
        }
    }
    closed_fields(file, at, m, &ce::MECHANISM_FIELDS, report);
}

fn check_constitution(root: &Node, range: &PlaceRange, report: &mut Report) {
    const FILE: &str = "constitution.yaml";
    if let Some(top) = schema_top_level(FILE, root, report) {
        unknown_sections(FILE, root, &top, report);
    }
    // 便 128: 欄の閉じた一覧は道具を組み立てた憲法の正本の schema から（置き場の schema 節は読まない）
    if let Some(meta) = map_holder(FILE, "meta", root.get("meta"), report) {
        closed_fields(FILE, "meta", meta, &ce::META_FIELDS, report);
    }
    const PRECEDENCE: &str = "前文（precedence）";
    if let Some(prec) = map_holder(FILE, PRECEDENCE, root.get("precedence"), report) {
        closed_fields(FILE, PRECEDENCE, prec, &ce::PRECEDENCE_FIELDS, report);
        check_mechanism(
            FILE,
            &format!("{PRECEDENCE}の mechanism"),
            prec.get("mechanism"),
            report,
        );
    }
    non_empty(
        FILE,
        "前文（precedence）",
        root.get("precedence").unwrap_or(&Node::Null),
        &["text", "plain"],
        report,
    );
    let articles = rows(FILE, root, "articles", report);
    for article in &articles {
        let id = row_id(article);
        let shown = article.get("id").and_then(Node::as_str).unwrap_or("None");
        if !article_id_form(shown) {
            report.violation(
                "schema",
                format!("{shown}: 条 id の形（P-n / A-n / N-n）でない"),
            );
        }
        non_empty(
            FILE,
            &format!("条 {id}"),
            article,
            &["id", "title", "statements"],
            report,
        );
        // 条の plain だけは床の字面（種別 R-10）で出す
        if article.get("plain").is_none_or(Node::is_blank) {
            report.violation("R-10", format!("{id}: plain が無い"));
        }
        closed_fields(FILE, &format!("条 {id}"), article, &ce::ARTICLE_FIELDS, report);
        check_mechanism(
            FILE,
            &format!("条 {id} の mechanism"),
            article.get("mechanism"),
            report,
        );
        let statements = rows(FILE, article, "statements", report);
        for st in &statements {
            let at = format!("条 {id} の規範文 {}", row_id(st));
            non_empty(FILE, &at, st, &["id", "text"], report);
            closed_fields(FILE, &at, st, &ce::STATEMENT_FIELDS, report);
        }
        check_article_enums(&id, article, &statements, range, report);
        check_statement_polarity(&id, &statements, report);
        duplicate_statement_ids(statements, report);
    }
    duplicate_ids(FILE, articles, report);
}

/// 条 1 つの値域を持つ欄 10 か所（便 55・ADR-11 決定 (3)(ア)）= 条の tier / binds・規範文の pattern / strength・
/// rationale の各行の kind・mechanism の kind / live / stage / polarity・retreat の kind。在る欄だけ、値を置き場の値域
/// （`place_range` が返した表・便 122）の同じ鍵の値の列で引き、無ければ（文字列でない値も同じ）種別 schema の違反 1 件。
/// 表に鍵が無い（引けない鍵・`place_range` が「まだ分からない」を出した）欄と、欄が無い・null のときと、
/// rationale / mechanism / retreat が一覧や表でないときは黙る（必須の欄の有無は別の話）。
fn check_article_enums(
    id: &str,
    article: &Node,
    statements: &[&Node],
    range: &PlaceRange,
    report: &mut Report,
) {
    const FILE: &str = "constitution.yaml";
    let mut field = |holder: &Node, name: &str, path: &str, key: &str| {
        let Some(value) = holder.get(name) else {
            return;
        };
        if matches!(value, Node::Null) {
            return;
        }
        let Some(known) = range.get(key) else {
            return;
        };
        if !value.as_str().is_some_and(|v| known.iter().any(|k| k == v)) {
            report.violation(
                "schema",
                format!(
                    "{FILE}: 条 {id} の {path} の値「{}」が憲法の値域 schema.enums.{key} に無い",
                    value.as_str().unwrap_or("?")
                ),
            );
        }
    };
    field(article, "tier", "tier", "tier");
    field(article, "binds", "binds", "binds");
    for st in statements {
        let sid = row_id(st);
        field(
            st,
            "pattern",
            &format!("statements の {sid} の pattern"),
            "pattern",
        );
        field(
            st,
            "strength",
            &format!("statements の {sid} の strength"),
            "strength",
        );
    }
    if let Some(Node::Seq(items)) = article.get("rationale") {
        for (i, item) in items.iter().enumerate() {
            field(
                item,
                "kind",
                &format!("rationale の {} 行目の kind", i + 1),
                "rationale_kind",
            );
        }
    }
    if let Some(m) = article.get("mechanism") {
        field(m, "kind", "mechanism.kind", "mechanism_kind");
        field(m, "live", "mechanism.live", "mechanism_live");
        field(m, "stage", "mechanism.stage", "stage");
        field(m, "polarity", "mechanism.polarity", "polarity");
    }
    if let Some(r) = article.get("retreat") {
        field(r, "kind", "retreat.kind", "retreat_kind");
    }
}

/// 規則の表 R-11（便 59・day-1 の Python の床から戻した式）: 規範文の strength と文末の一致 = must-not ⇔ 文末が「ない。」／
/// must・should ⇔ それ以外。式は床の定数（憲法の schema.one_polarity は宣言で、床はその値を読まない・規則の表 R-11 の what が正本・
/// ADR-11 決定 (3)(エ)）。合わない 1 本につき種別 R-11 の違反 1 件。strength が組み立てた値域の外（`check_article_enums` が種別 schema で
/// 数えるか、置き場の値域に在れば `place_range` が「まだ分からない」を出す）・text が字でない（非空の検査が数える）ときは黙る＝二重に出さない。
fn check_statement_polarity(id: &str, statements: &[&Node], report: &mut Report) {
    for st in statements {
        let Some(strength) = st
            .get("strength")
            .and_then(Node::as_str)
            .and_then(ce::Strength::from_name)
        else {
            continue;
        };
        let Some(text) = st.get("text").and_then(Node::as_str) else {
            continue;
        };
        let negative = text.trim_end().ends_with("ない。");
        if (strength == ce::Strength::MustNot) != negative {
            report.violation(
                "R-11",
                format!(
                    "{id}: {}: strength {} と文末が合わない（must-not ⇔ 〜ない。）",
                    row_id(st),
                    strength.name()
                ),
            );
        }
    }
}

/// 規則の表。最上位の節の閉じた一覧は file の schema.top_level ではなく床の定数（`rules::RULES_TOP_LEVEL`）から読む
/// （便 51・file の側で節を足して通す口を塞ぐ・N-3.1）。file の schema.top_level の有無は見ない（写しの一致は `folio schema` の側）。
/// 行の欄の閉じた一覧も file の生成区間ではなく床の定数（閾値行・開発規律行の required と optional）から読む（便 128）。
fn check_rules(root: &Node, report: &mut Report) {
    const FILE: &str = "rules.yaml";
    unknown_sections(FILE, root, &rules::RULES_TOP_LEVEL, report);
    let thresholds = [&rules::THRESHOLD_REQUIRED[..], &rules::THRESHOLD_OPTIONAL[..]].concat();
    let discipline = [&rules::DISCIPLINE_REQUIRED[..], &rules::DISCIPLINE_OPTIONAL[..]].concat();
    let mut all = Vec::new();
    for (section, allowed) in [("thresholds", &thresholds), ("discipline", &discipline)] {
        for row in rows(FILE, root, section, report) {
            let at = format!("行 {}", row_id(row));
            non_empty(FILE, &at, row, &["id", "article", "what"], report);
            closed_fields(FILE, &at, row, allowed, report);
            check_rule_refs(row, report);
            all.push(row);
        }
    }
    duplicate_ids(FILE, all, report);
}

/// 規則の表の行の refs（便 91 §1 (c)）。各項は id の形で、その行自身の id でも article の値でもない（article の条以外）。
/// 実在は refs.rs（行 R-4 の 1 つ目の数え）と link.rs（A-2）の網が数えるので重ねない。
fn check_rule_refs(row: &Node, report: &mut Report) {
    const FILE: &str = "rules.yaml";
    let key = rules::ROW_REFS;
    let at = format!("行 {}", row_id(row));
    match row.get(key) {
        None | Some(Node::Null) => {}
        Some(Node::Seq(items)) => {
            let own = [
                row.get("id").and_then(Node::as_str),
                row.get("article").and_then(Node::as_str),
            ];
            for r in items {
                let value = r.as_str();
                if !value.is_some_and(adr::is_basis_id) {
                    report.violation(
                        "schema",
                        format!(
                            "{FILE}: {at} の {key}「{}」が id の形でない（条・要件・rules 行・判断の記録）",
                            value.unwrap_or("?")
                        ),
                    );
                } else if own.contains(&value) {
                    report.violation(
                        "schema",
                        format!(
                            "{FILE}: {at} の {key}「{}」が自分の id か article の条である（article の条以外を書く）",
                            value.unwrap_or("?")
                        ),
                    );
                }
            }
        }
        Some(_) => report.violation("schema", format!("{FILE}: {at} の {key} が一覧でない")),
    }
}

fn check_vocabulary(root: &Node, report: &mut Report) {
    const FILE: &str = "vocabulary.yaml";
    unknown_sections(FILE, root, &VOCABULARY_TOP_LEVEL, report);
    for (section, fields) in [
        ("terms", &["id", "term", "short", "def"][..]),
        ("field_terms", &["id", "term", "def"][..]),
    ] {
        let items = rows(FILE, root, section, report);
        for item in &items {
            non_empty(
                FILE,
                &format!("{section} の語 {}", row_id(item)),
                item,
                fields,
                report,
            );
        }
        duplicate_ids(FILE, items, report);
    }
    for group in rows(FILE, root, "identifiers", report) {
        let name = group.get("group").and_then(Node::as_str).unwrap_or("?");
        non_empty(
            FILE,
            &format!("identifiers の組 {name}"),
            group,
            &["group", "words"],
            report,
        );
    }
}

fn check_srs(root: &Node, report: &mut Report) {
    const FILE: &str = "srs.yaml";
    unknown_sections(FILE, root, &SRS_TOP_LEVEL, report);
    let mut all = Vec::new();
    for section in [
        "goals",
        "verdicts",
        "requirements",
        "nonfunctional",
        "acceptance",
        "constraints",
    ] {
        for row in rows(FILE, root, section, report) {
            let fields: &[&str] = match section {
                "requirements" | "nonfunctional" => {
                    check_srs_item(section, row, report);
                    all.push(row);
                    continue;
                }
                // 図 3 の答えの行（tone の値域は面の生成器が数える）
                "verdicts" => &["id", "name", "tone", "cond"],
                _ => &["id", "title"],
            };
            non_empty(
                FILE,
                &format!("{section} の {}", row_id(row)),
                row,
                fields,
                report,
            );
            all.push(row);
        }
    }
    // 任意の図の節（便 34）。図の id は他の節の id と重複しない
    for fig in rows(FILE, root, "figures", report) {
        check_srs_figure(fig, report);
        all.push(fig);
    }
    duplicate_ids(FILE, all, report);
}

/// 要件の行 1 つ（便 75）。欄の集合は閉じた一覧・字の欄は非空・一覧の欄は在ること・verify は表で中を数える。
/// verify が無い・表でないときは違反 1 件で中は見ない（1 つの欠けを 2 件に膨らませない）。
fn check_srs_item(section: &str, row: &Node, report: &mut Report) {
    const FILE: &str = "srs.yaml";
    let at = format!("{section} の {}", row_id(row));
    for (key, _) in row.as_map().unwrap_or_default() {
        let k = key.as_str();
        if !SRS_ITEM_TEXT.contains(&k)
            && !SRS_ITEM_LIST.contains(&k)
            && !SRS_ITEM_OPTIONAL.contains(&k)
            && k != SRS_ITEM_VERIFY
        {
            report.violation("未知の欄", format!("{FILE}: {at} の未知の欄「{key}」"));
        }
    }
    non_empty(FILE, &at, row, &SRS_ITEM_TEXT, report);
    for key in SRS_ITEM_LIST {
        seq_field(FILE, &at, row, key, report);
    }
    match row.get(SRS_ITEM_VERIFY) {
        Some(verify) if verify.as_map().is_some() => {
            let inner = format!("{at} の {SRS_ITEM_VERIFY}");
            non_empty(FILE, &inner, verify, &SRS_ITEM_VERIFY_TEXT, report);
            for key in SRS_ITEM_VERIFY_LIST {
                seq_field(FILE, &inner, verify, key, report);
            }
        }
        _ => report.violation("schema", format!("{FILE}: {at} の verify が無い（表）")),
    }
    // adrs は判断の記録の id の形だけを数える（便 90 §1 (c)）。実在は link.rs の網（A-2）が数えるので重ねない
    match row.get(SRS_ITEM_ADRS) {
        None | Some(Node::Null) => {}
        Some(Node::Seq(items)) => {
            for r in items {
                if !r
                    .as_str()
                    .is_some_and(|v| v.starts_with("ADR-") && adr::is_basis_id(v))
                {
                    report.violation(
                        "schema",
                        format!(
                            "{FILE}: {at} の {SRS_ITEM_ADRS}「{}」が判断の記録の id の形でない",
                            r.as_str().unwrap_or("?")
                        ),
                    );
                }
            }
        }
        Some(_) => report.violation(
            "schema",
            format!("{FILE}: {at} の {SRS_ITEM_ADRS} が一覧でない"),
        ),
    }
}

/// 欄が一覧であること（空の一覧は通る）。無い・null・一覧でないは同じ字面の違反 1 件。
fn seq_field(file: &str, at: &str, row: &Node, key: &str, report: &mut Report) {
    if !matches!(row.get(key), Some(Node::Seq(_))) {
        report.violation(
            "schema",
            format!("{file}: {at} の {key} が無い（一覧・空でよい）"),
        );
    }
}

/// 要件書の図の行 1 つ（便 34）。欄の集合・id と caption の非空（欠落も同じ形）・型は部品目録の図の型・spec は表・
/// refs の各項は id の形（判断の記録の basis と同じ判定）。違反の種類は schema（他の要件書の欄と同じ床の字面）。
fn check_srs_figure(fig: &Node, report: &mut Report) {
    const FILE: &str = "srs.yaml";
    let at = format!("figures の {}", row_id(fig));
    for (key, _) in fig.as_map().unwrap_or_default() {
        if !SRS_FIGURE_REQUIRED.contains(&key.as_str())
            && !SRS_FIGURE_OPTIONAL.contains(&key.as_str())
        {
            report.violation("未知の欄", format!("{FILE}: {at} の未知の欄「{key}」"));
        }
    }
    non_empty(FILE, &at, fig, &SRS_FIGURE_REQUIRED, report);
    if let Some(t) = fig.get("type")
        && !t.is_blank()
        && !t
            .as_str()
            .is_some_and(|v| FigureType::from_name(v).is_some())
    {
        report.violation(
            "schema",
            format!(
                "{FILE}: {at} の図の型「{}」が部品目録の一覧に無い",
                t.as_str().unwrap_or("?")
            ),
        );
    }
    if let Some(spec) = fig.get("spec")
        && !spec.is_blank()
        && spec.as_map().is_none()
    {
        report.violation("schema", format!("{FILE}: {at} の spec が表でない"));
    }
    match fig.get("refs") {
        None | Some(Node::Null) => {}
        Some(Node::Seq(items)) => {
            for r in items {
                if !r.as_str().is_some_and(adr::is_basis_id) {
                    report.violation(
                        "schema",
                        format!(
                            "{FILE}: {at} の refs「{}」が id の形（条・要件・rules 行・判断の記録）でない",
                            r.as_str().unwrap_or("?")
                        ),
                    );
                }
            }
        }
        Some(_) => report.violation("schema", format!("{FILE}: {at} の refs が一覧でない")),
    }
}

#[cfg(test)]
mod check_tests {
    use super::*;

    const SRS: &str = "goals:\n  - {id: GOAL1, title: 相談}\nverdicts:\n  - {id: pass, name: 合格, tone: ok, cond: 違反が無い}\n  - {id: fail, name: 不合格, tone: bad, cond: 違反が在る}\n  - {id: pending, name: まだ分からない, tone: neutral, cond: 動かせない}\nacceptance:\n  - {id: AC1, title: 受入}\n";

    fn srs_report(text: &str) -> Report {
        let doc = yaml::parse(text).unwrap();
        let mut report = Report::default();
        check_srs(&doc.root, &mut report);
        report
    }

    #[test]
    fn check_srs_accepts_three_verdicts() {
        let report = srs_report(SRS);
        assert!(report.violations.is_empty(), "{:?}", report.violations);
        assert!(report.unknowns.is_empty(), "{:?}", report.unknowns);
    }

    #[test]
    fn check_srs_counts_an_empty_verdict_cond() {
        let report = srs_report(&SRS.replacen("cond: 動かせない", "cond: \"\"", 1));
        assert_eq!(report.violations.len(), 1, "{:?}", report.violations);
        assert_eq!(report.violations[0].0, "欄の非空");
    }

    #[test]
    fn check_srs_counts_a_verdict_id_shared_with_acceptance() {
        let report = srs_report(&SRS.replacen("{id: fail,", "{id: AC1,", 1));
        assert_eq!(report.violations.len(), 1, "{:?}", report.violations);
        assert_eq!(report.violations[0].0, "重複キー");
    }
}
