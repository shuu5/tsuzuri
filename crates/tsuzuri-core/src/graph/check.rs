//! 不変条件の 12 本を 3 値（合格・違反・まだ分からない）で数える。
//! この便で数えるのは 9 本。g-3・g-7・g-9 は材料が 3 つの入力に無いので、つねに「まだ分からない」。
//! 要る出所が読めなければ「まだ分からない」で、合格にしない。違反は名指す id を持つ。
//! 要約の無い節点は不変条件でなく床の値で、`unsummarized` が数えて名指す（要件 FR15）。

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::{EdgeType, NodeKind};
use tsuzuri_contract::ledger::MEMO_LABEL;

use super::{Graph, Source};

/// 不変条件の判定（3 値）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "verdict", content = "ids", rename_all = "kebab-case")]
pub enum Verdict {
    Pass,
    /// 違反（名指す id・名の順・重複なし）。
    Violation(Vec<String>),
    Unknown,
}

/// 1 本の不変条件の判定。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Invariant {
    pub id: &'static str,
    pub verdict: Verdict,
}

type Rule = fn(&Graph) -> Verdict;

/// 不変条件の 12 本（id の順）。
const RULES: [(&str, Rule); 12] = [
    ("g-1", g1_edge_ends),
    ("g-2", g2_one_pointer),
    ("g-3", unmeasured),
    ("g-4", g4_touches),
    ("g-5", g5_reaches_root),
    ("g-6", g6_no_loop),
    ("g-7", unmeasured),
    ("g-8", g8_memo_or_contract),
    ("g-9", unmeasured),
    ("g-10", g10_disjoint_ids),
    ("g-11", g11_run_of),
    ("g-12", g12_raised),
];

/// 不変条件の id の一覧（順も固定）。
pub const INVARIANTS: [&str; 12] = [
    "g-1", "g-2", "g-3", "g-4", "g-5", "g-6", "g-7", "g-8", "g-9", "g-10", "g-11", "g-12",
];

/// つねに「まだ分からない」を返す 3 本（材料が 3 つの入力に無い）。
pub const UNMEASURED: [&str; 3] = ["g-3", "g-7", "g-9"];

/// 12 本を数える（id の順）。
pub fn check(g: &Graph) -> Vec<Invariant> {
    RULES
        .iter()
        .map(|(id, rule)| Invariant {
            id,
            verdict: rule(g),
        })
        .collect()
}

/// 要る出所が全部読めていれば、名指す id の有無で合格か違反。読めない出所が在れば「まだ分からない」。
fn judge(g: &Graph, needs: &[Source], bad: BTreeSet<String>) -> Verdict {
    if needs.iter().any(|s| !g.is_read(*s)) {
        Verdict::Unknown
    } else if bad.is_empty() {
        Verdict::Pass
    } else {
        Verdict::Violation(bad.into_iter().collect())
    }
}

fn unmeasured(_: &Graph) -> Verdict {
    Verdict::Unknown
}

/// g-1 組んだ辺の両端の節点が実在する。片端の無い辺が在り、読めない出所が在れば「まだ分からない」。
fn g1_edge_ends(g: &Graph) -> Verdict {
    let ids: BTreeSet<&str> = g.nodes.iter().map(|n| n.id.as_str()).collect();
    let bad: BTreeSet<String> = g
        .edges
        .iter()
        .flat_map(|e| [&e.from, &e.to])
        .filter(|id| !ids.contains(id.as_str()))
        .cloned()
        .collect();
    if bad.is_empty() {
        Verdict::Pass
    } else {
        judge(g, &Source::ALL, bad)
    }
}

/// g-2 open の契約は pointer の行をちょうど 1 つ持つ（closed の契約は数えない）。
fn g2_one_pointer(g: &Graph) -> Verdict {
    let bad = g
        .beads
        .iter()
        .filter(|(_, b)| b.kind == NodeKind::Task && b.is_open() && b.pointers.len() != 1)
        .map(|(id, _)| id.clone())
        .collect();
    judge(g, &[Source::Ledger], bad)
}

/// g-4 open の問いは metadata の touches の欄に id を 1 つ以上持つ。
fn g4_touches(g: &Graph) -> Verdict {
    let bad = g
        .beads
        .iter()
        .filter(|(_, b)| b.kind == NodeKind::Question && b.is_open() && b.touches.is_empty())
        .map(|(id, _)| id.clone())
        .collect();
    judge(g, &[Source::Ledger], bad)
}

/// 辺の型ごとの (from, to) の一覧。
fn pairs(g: &Graph, edge_type: EdgeType) -> Vec<(&str, &str)> {
    g.edges
        .iter()
        .filter(|e| e.edge_type == edge_type)
        .map(|e| (e.from.as_str(), e.to.as_str()))
        .collect()
}

/// bead ごとの親（parent-child の先・辺の順）。
fn parents(g: &Graph) -> BTreeMap<&str, Vec<&str>> {
    let mut map: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (child, parent) in pairs(g, EdgeType::ParentChild) {
        map.entry(child).or_default().push(parent);
    }
    map
}

/// g-5 根（親を持たない epic）でない bead は、parent-child をたどって根に着く。
fn g5_reaches_root(g: &Graph) -> Verdict {
    let parents = parents(g);
    let is_root = |id: &str| {
        g.beads.get(id).is_some_and(|b| b.kind == NodeKind::Epic) && !parents.contains_key(id)
    };
    let reaches_root = |start: &str| {
        let mut seen = BTreeSet::from([start]);
        let mut cur = start;
        loop {
            if is_root(cur) {
                return true;
            }
            match parents.get(cur).and_then(|p| p.first()) {
                Some(p) if g.beads.contains_key(*p) && seen.insert(*p) => cur = p,
                _ => return false,
            }
        }
    };
    let bad = g
        .beads
        .keys()
        .filter(|id| !reaches_root(id))
        .cloned()
        .collect();
    judge(g, &[Source::Ledger], bad)
}

/// 輪の上か、輪と輪の間に在る節点（入りの無い節点と出の無い節点を繰り返し除いた残り）。
/// 残りが空なら輪は無い。
fn on_loop(pairs: &[(&str, &str)]) -> BTreeSet<String> {
    let mut live: Vec<(&str, &str)> = pairs.to_vec();
    loop {
        let froms: BTreeSet<&str> = live.iter().map(|(f, _)| *f).collect();
        let tos: BTreeSet<&str> = live.iter().map(|(_, t)| *t).collect();
        let before = live.len();
        live.retain(|(f, t)| tos.contains(f) && froms.contains(t));
        if live.len() == before {
            break;
        }
    }
    live.iter()
        .flat_map(|(f, t)| [*f, *t])
        .map(str::to_string)
        .collect()
}

/// g-6 blocks に輪が無く、parent-child は木（親は 1 つまで・輪が無い）。
fn g6_no_loop(g: &Graph) -> Verdict {
    let mut bad = on_loop(&pairs(g, EdgeType::Blocks));
    bad.extend(on_loop(&pairs(g, EdgeType::ParentChild)));
    bad.extend(
        parents(g)
            .into_iter()
            .filter(|(_, ps)| ps.iter().collect::<BTreeSet<_>>().len() > 1)
            .map(|(child, _)| child.to_string()),
    );
    judge(g, &[Source::Ledger], bad)
}

/// g-8 memo の label と pointer の行の両方を持つ bead が無い。
fn g8_memo_or_contract(g: &Graph) -> Verdict {
    let bad = g
        .beads
        .iter()
        .filter(|(_, b)| b.labels.iter().any(|l| l == MEMO_LABEL) && !b.pointers.is_empty())
        .map(|(id, _)| id.clone())
        .collect();
    judge(g, &[Source::Ledger], bad)
}

/// g-10 設計の索引と台帳と走行の id が互いに重ならない。重なりが在れば、読めない出所が在っても違反。
fn g10_disjoint_ids(g: &Graph) -> Verdict {
    let mut families: BTreeMap<&str, BTreeSet<Source>> = BTreeMap::new();
    for n in &g.nodes {
        if let Some(s) = Source::of(n.kind) {
            families.entry(n.id.as_str()).or_default().insert(s);
        }
    }
    let bad: BTreeSet<String> = families
        .into_iter()
        .filter(|(_, fs)| fs.len() > 1)
        .map(|(id, _)| id.to_string())
        .collect();
    if bad.is_empty() {
        judge(g, &Source::ALL, bad)
    } else {
        Verdict::Violation(bad.into_iter().collect())
    }
}

/// g-11 走行は run_of の先の bead を持つ。
fn g11_run_of(g: &Graph) -> Verdict {
    let run_of = pairs(g, EdgeType::RunOf);
    let bad = g
        .runs
        .keys()
        .filter(|run| {
            !run_of
                .iter()
                .any(|(from, to)| from == run && g.beads.contains_key(*to))
        })
        .cloned()
        .collect();
    judge(g, &[Source::Runs, Source::Ledger], bad)
}

/// 走行の段の Questioned（器の語）。
pub const QUESTIONED: &str = "Questioned";

/// g-12 段が Questioned の走行は答えの無い問いを持つ（器の問いは event log の中だけに在り、raised の辺は見ない）。
fn g12_raised(g: &Graph) -> Verdict {
    let bad = g
        .runs
        .iter()
        .filter(|(_, r)| r.stage.as_deref() == Some(QUESTIONED) && r.unanswered == 0)
        .map(|(run, _)| run.clone())
        .collect();
    judge(g, &[Source::Runs], bad)
}

/// 要約を数える種類（設計文書の 11 と、台帳の epic・task・memo・問い）。
/// 裁定・受け・方針は notes の 1 行から、走行は event log から導き、要約の欄を持たないので数えない。
pub const SUMMARY_KINDS: [NodeKind; 15] = [
    NodeKind::ALL[0],
    NodeKind::ALL[1],
    NodeKind::ALL[2],
    NodeKind::ALL[3],
    NodeKind::ALL[4],
    NodeKind::ALL[5],
    NodeKind::ALL[6],
    NodeKind::ALL[7],
    NodeKind::ALL[8],
    NodeKind::ALL[9],
    NodeKind::ALL[10],
    NodeKind::Epic,
    NodeKind::Task,
    NodeKind::Memo,
    NodeKind::Question,
];

/// 要約の無い節点（種類が `SUMMARY_KINDS` に在り、plain と eng のどちらも空でない字を持たない節点の id・字の順・重複なし）。
/// `summary` は設計の索引の要約の字を読めたか（`add_summary` の返り）。
/// 設計の索引か台帳が読めないか、要約の字が読めなければ「まだ分からない」（0 件とも名指しとも言わない）。
pub fn unsummarized(g: &Graph, summary: bool) -> Reading<Vec<String>> {
    if !summary || !g.is_read(Source::Design) || !g.is_read(Source::Ledger) {
        return Reading::Unknown;
    }
    let has = |s: &Option<String>| s.as_deref().is_some_and(|s| !s.is_empty());
    let ids: BTreeSet<String> = g
        .nodes
        .iter()
        .filter(|n| SUMMARY_KINDS.contains(&n.kind) && !has(&n.plain) && !has(&n.eng))
        .map(|n| n.id.clone())
        .collect();
    Reading::Known(ids.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::{INVARIANTS, RULES, on_loop};

    #[test]
    fn graph_rules_follow_the_id_list() {
        let ids: Vec<&str> = RULES.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, INVARIANTS.to_vec());
    }

    #[test]
    fn graph_on_loop_finds_only_loops() {
        assert!(on_loop(&[("a", "b"), ("b", "c")]).is_empty());
        let got: Vec<String> = on_loop(&[("x", "a"), ("a", "b"), ("b", "a"), ("b", "y")])
            .into_iter()
            .collect();
        assert_eq!(got, vec!["a", "b"]);
        assert_eq!(on_loop(&[("s", "s")]).len(), 1);
    }
}
