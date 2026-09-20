//! `folio check` — design-intent の正本 7 file（憲法・rules・語彙・要件書・入口・相談窓口・天井）の形の床（FR5 / FR9）。
//! 数えるのは 重複キー・未知の節・欄の非空（便 0）と、参照 id の解決・rules 行の逆参照・憲法の件数（便 1・refs）と、語彙の検査 R-9（便 4・vocab）と、
//! 判断の記録（adr/）の欄の決まり（便 5・adr）と、判断の記録と正本 4 file・凍結 anchor の列の突き合わせ（便 6・link）と、
//! 凍結 anchor の列のうち版管理を見ない部分（便 7・anchor）と、入口の正本の形（便 12・entrance）と、
//! 相談窓口の正本の形（便 18・intake）と、設計ノートの正本の形（便 23・note）と、天井の正本の形（便 37・ceiling）と、
//! 憲法の条の値域を持つ欄の値（便 55・在る欄だけ・組み立て時に憲法から導出した型で引く）。
//! 参照 id・語彙 R-9・判断の記録との突き合わせ・凍結 anchor・読み物の生成は今も憲法・rules・語彙・要件書の 4 本だけを受ける。
//! 読めない・型が違う・節の決まりが読めない は「まだ分からない」（合格にしない）。

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::adr;
use crate::anchor;
use crate::ceiling;
use crate::constitution_enums as ce;
use crate::entrance;
use crate::freeze::{self, After, Flag};
use crate::intake;
use crate::link;
use crate::note;
use crate::parts::catalog::FigureType;
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

/// 要件書の節の閉じた一覧（要件書は schema 節を持たないので床の定数で持つ）。figures は任意の図の節（便 34・FR15）。
pub const SRS_TOP_LEVEL: [&str; 16] = [
    "meta",
    "goals",
    "scope",
    "scope_m1",
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
];

/// 要件書の図の節の行の欄（判断の記録の figures.entry と同じ形・便 34）。
const SRS_FIGURE_REQUIRED: [&str; 4] = ["id", "type", "caption", "spec"];
const SRS_FIGURE_OPTIONAL: [&str; 2] = ["refs", "note"];

/// 語彙の節の閉じた一覧（同上）。
pub const VOCABULARY_TOP_LEVEL: [&str; 3] = ["terms", "field_terms", "identifiers"];

struct Sources {
    constitution: Node,
    rules: Node,
    vocabulary: Node,
    srs: Node,
    index: Node,
    intake: Node,
    ceiling: Node,
}

/// `dir` の正本 7 file を検査する。`flag` は便 9 の旗（検査の式は変えず、列の結果を `freeze.rs` へ渡す）。
pub fn check_dir(dir: &Path, flag: Flag) -> (Report, After) {
    let mut report = Report::default();
    let mut state = None;
    let mut adr_records = None;
    match load_all(dir, &mut report) {
        Some(src) => {
            let history = anchor::history_ids(dir);
            check_constitution(&src.constitution, &mut report);
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
        }
        None => debug_assert!(!report.unknowns.is_empty()),
    }
    let after = freeze::after(dir, flag, state.as_ref(), adr_records.as_ref(), &mut report);
    (report, after)
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

fn check_constitution(root: &Node, report: &mut Report) {
    const FILE: &str = "constitution.yaml";
    if let Some(top) = schema_top_level(FILE, root, report) {
        unknown_sections(FILE, root, &top, report);
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
        let statements = rows(FILE, article, "statements", report);
        for st in &statements {
            non_empty(
                FILE,
                &format!("条 {id} の規範文 {}", row_id(st)),
                st,
                &["id", "text"],
                report,
            );
        }
        check_article_enums(&id, article, &statements, report);
        check_statement_polarity(&id, &statements, report);
        duplicate_statement_ids(statements, report);
    }
    duplicate_ids(FILE, articles, report);
}

/// 条 1 つの値域を持つ欄 10 か所（便 55・ADR-11 決定 (3)(ア)）= 条の tier / binds・規範文の pattern / strength・
/// rationale の各行の kind・mechanism の kind / live / stage / polarity・retreat の kind。在る欄だけ、値を憲法の正本から
/// 組み立て時に導出した型（`constitution_enums`）の from_name で引き、引けなければ（文字列でない値も同じ）種別 schema の違反 1 件。
/// 欄が無い・null のときと、rationale / mechanism / retreat が一覧や表でないときは黙る（必須の欄の有無は別の話）。
fn check_article_enums(id: &str, article: &Node, statements: &[&Node], report: &mut Report) {
    const FILE: &str = "constitution.yaml";
    let mut field = |holder: &Node, name: &str, path: &str, key: &str, known: fn(&str) -> bool| {
        let Some(value) = holder.get(name) else {
            return;
        };
        if matches!(value, Node::Null) {
            return;
        }
        if !value.as_str().is_some_and(known) {
            report.violation(
                "schema",
                format!(
                    "{FILE}: 条 {id} の {path} の値「{}」が憲法の値域 schema.enums.{key} に無い",
                    value.as_str().unwrap_or("?")
                ),
            );
        }
    };
    field(article, "tier", "tier", "tier", |v| {
        ce::Tier::from_name(v).is_some()
    });
    field(article, "binds", "binds", "binds", |v| {
        ce::Binds::from_name(v).is_some()
    });
    for st in statements {
        let sid = row_id(st);
        field(
            st,
            "pattern",
            &format!("statements の {sid} の pattern"),
            "pattern",
            |v| ce::Pattern::from_name(v).is_some(),
        );
        field(
            st,
            "strength",
            &format!("statements の {sid} の strength"),
            "strength",
            |v| ce::Strength::from_name(v).is_some(),
        );
    }
    if let Some(Node::Seq(items)) = article.get("rationale") {
        for (i, item) in items.iter().enumerate() {
            field(
                item,
                "kind",
                &format!("rationale の {} 行目の kind", i + 1),
                "rationale_kind",
                |v| ce::RationaleKind::from_name(v).is_some(),
            );
        }
    }
    if let Some(m) = article.get("mechanism") {
        field(m, "kind", "mechanism.kind", "mechanism_kind", |v| {
            ce::MechanismKind::from_name(v).is_some()
        });
        field(m, "live", "mechanism.live", "mechanism_live", |v| {
            ce::MechanismLive::from_name(v).is_some()
        });
        field(m, "stage", "mechanism.stage", "stage", |v| {
            ce::Stage::from_name(v).is_some()
        });
        field(m, "polarity", "mechanism.polarity", "polarity", |v| {
            ce::Polarity::from_name(v).is_some()
        });
    }
    if let Some(r) = article.get("retreat") {
        field(r, "kind", "retreat.kind", "retreat_kind", |v| {
            ce::RetreatKind::from_name(v).is_some()
        });
    }
}

/// 規則の表 R-11（便 59・day-1 の Python の床から戻した式）: 規範文の strength と文末の一致 = must-not ⇔ 文末が「ない。」／
/// must・should ⇔ それ以外。式は床の定数（憲法の schema.one_polarity は宣言で、床はその値を読まない・規則の表 R-11 の what が正本・
/// ADR-11 決定 (3)(エ)）。合わない 1 本につき種別 R-11 の違反 1 件。strength が値域の外（`check_article_enums` が種別 schema で
/// 数える）・text が字でない（非空の検査が数える）ときは黙る＝二重に出さない。
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
fn check_rules(root: &Node, report: &mut Report) {
    const FILE: &str = "rules.yaml";
    unknown_sections(FILE, root, &rules::RULES_TOP_LEVEL, report);
    let mut all = Vec::new();
    for section in ["thresholds", "discipline"] {
        for row in rows(FILE, root, section, report) {
            non_empty(
                FILE,
                &format!("行 {}", row_id(row)),
                row,
                &["id", "article", "what"],
                report,
            );
            all.push(row);
        }
    }
    duplicate_ids(FILE, all, report);
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
                // 図 3 の答えの行（tone の値域は面の生成器が数える）
                "verdicts" => &["id", "name", "tone", "cond"],
                "requirements" | "nonfunctional" => &["id", "title", "shall", "plain"],
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
