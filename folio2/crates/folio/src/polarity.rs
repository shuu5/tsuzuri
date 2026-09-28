//! 極性一覧と編集時の止めの下限（便 200・docs/design/delivery-200.md §1・判断の記録 ADR-33 決定 (5)(7)・要件書 FR29・AC32・
//! 条 P-18.3 / P-18.4）。極性一覧は、置き場の憲法の各条の機構のうち種別が拒む（reject）か生成時の検査（build-check）のもの
//! （段・極性）と、規則の表の閾値の行（段・極性は行の条の機構の極性）と、床の定数の仕掛けの一覧（`floor_note.rs` の床の木の
//! guards・段は in_loop か post・極性は fail-closed）から 1 仕掛け 1 行で組む。床（`check.rs` の `check_dir`）は、欄 key が
//! in-loop-min の閾値の行が 1 本在るとき、段が in-loop の本数がその値を割れば違反にする。その行が無ければ数えず知らせ、
//! 2 本以上在るか値の形が違えば まだ分からない とする。in_loop の名と行 R-13 の欄 key は器の行の着地の後（決定 (7)・本便は運ばない）。

use std::path::Path;

use crate::check;
use crate::constitution_enums::{MechanismKind, Stage};
use crate::floor::Floor;
use crate::floor_note;
use crate::rules;
use crate::verdict::Report;
use crate::yaml::Node;

/// 下限の行が無くて数えなかった知らせ（素の床の標準エラー・判定の外）。
pub const OFF: &str = "# 欄 key が in-loop-min の閾値の行が規則の表に無い＝編集時の止めの本数の下限は数えていない（床の判定の外・条 P-18.4）";

/// 値が分からない欄の字（欄が無いか字でない）。
const NONE: &str = "無い";

/// 極性一覧の 1 行（仕掛けの名・段・極性・出所）。
pub struct Guard {
    pub name: String,
    pub stage: String,
    pub polarity: String,
    pub from: &'static str,
}

impl Guard {
    pub fn line(&self) -> String {
        format!("{} · {} · {} · {}", self.name, self.stage, self.polarity, self.from)
    }
}

fn text(node: &Node, key: &str) -> String {
    node.get(key).and_then(Node::as_str).unwrap_or(NONE).to_string()
}

/// 床の定数の仕掛けの一覧（床の木の guards の in_loop と post）。
fn floor_guards() -> Vec<(&'static str, &'static [&'static str])> {
    let Floor::Map(top) = floor_note::FLOOR else { return Vec::new() };
    let Some((_, Floor::Map(guards))) = top.iter().find(|(k, _)| *k == "guards") else { return Vec::new() };
    [("in_loop", Stage::InLoop), ("post", Stage::Post)]
        .into_iter()
        .filter_map(|(key, stage)| match guards.iter().find(|(k, _)| *k == key) {
            Some((_, Floor::Strs(names))) => Some((stage.name(), *names)),
            _ => None,
        })
        .collect()
}

/// 極性一覧（憲法の条の機構 → 規則の表の閾値の行 → 床の定数の順・それぞれ書かれた順）。
pub fn list(constitution: &Node, rules: &Node) -> Vec<Guard> {
    let articles = constitution.get("articles").and_then(Node::as_seq).unwrap_or_default();
    let mechanism = |a: &Node| a.get("mechanism").cloned().unwrap_or(Node::Null);
    let mut out = Vec::new();
    for a in articles {
        let m = mechanism(a);
        let kind = m.get("kind").and_then(Node::as_str).and_then(MechanismKind::from_name);
        if matches!(kind, Some(MechanismKind::Reject | MechanismKind::BuildCheck)) {
            out.push(Guard { name: text(a, "id"), stage: text(&m, "stage"), polarity: text(&m, "polarity"), from: "憲法の条の機構" });
        }
    }
    for row in rules.get(rules::RULES_TOP_LEVEL[1]).and_then(Node::as_seq).unwrap_or_default() {
        let article = row.get("article").and_then(Node::as_str);
        let owner = articles.iter().find(|a| a.get("id").and_then(Node::as_str) == article);
        let polarity = owner.map_or(NONE.to_string(), |a| text(&mechanism(a), "polarity"));
        out.push(Guard { name: text(row, "id"), stage: text(row, "stage"), polarity, from: "規則の表の閾値の行" });
    }
    for (stage, names) in floor_guards() {
        for name in names {
            out.push(Guard { name: name.to_string(), stage: stage.to_string(), polarity: "fail-closed".to_string(), from: "床の定数の仕掛けの一覧" });
        }
    }
    out
}

/// 段が in-loop の本数。
fn in_loop(guards: &[Guard]) -> usize {
    guards.iter().filter(|g| g.stage == Stage::InLoop.name()).count()
}

/// 床の下限の数え。下限の行が無くて数えなかったら真（呼び手が `OFF` を出す）。
pub fn check_floor(constitution: &Node, rules: &Node, report: &mut Report) -> bool {
    match rules::in_loop_min(rules) {
        Err(e) => report.unknown(format!("rules.yaml: {e}")),
        Ok(None) => return true,
        Ok(Some((id, min))) => {
            let n = in_loop(&list(constitution, rules));
            if n < min {
                report.violation(
                    "P-18",
                    format!("極性一覧の編集時（in-loop）の仕掛けが {n} 本で、行 {id} の下限 {min} 本以上を割る（P-18.4）"),
                );
            }
        }
    }
    false
}

/// `folio check --polarity` の出力の行（1 仕掛け 1 行と集計の 1 行）。正本は床と同じ読み口で読み（`check.rs` の load_pair）、
/// 読めないか下限の行が読めなければ Err（まだ分からない の字の列）。
pub fn render(dir: &Path) -> Result<Vec<String>, Vec<String>> {
    let (constitution, rules) = check::load_pair(dir)?;
    let guards = list(&constitution, &rules);
    let n = in_loop(&guards);
    let post = guards.iter().filter(|g| g.stage == Stage::Post.name()).count();
    let floor = match rules::in_loop_min(&rules).map_err(|e| vec![e])? {
        None => "下限の行が無い＝数えない".to_string(),
        Some((id, min)) => format!("下限 {min} 本以上（行 {id}）に{}", if n < min { "足りない" } else { "足りる" }),
    };
    let mut out: Vec<String> = guards.iter().map(Guard::line).collect();
    out.push(format!("folio check --polarity: 仕掛け {}（in-loop {n}・post {post}）・{floor}", guards.len()));
    Ok(out)
}
