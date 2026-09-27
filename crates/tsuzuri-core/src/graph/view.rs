//! グラフの眺め: 規範文を条へ・question を親へ・ruling を答えた question の畳み先へ畳み、
//! 辺を根拠の側へ向けてまとめ、上限で切り、根拠の側からの段を決める（見本 map.html の graphModel）。
//! 画面は値を写して座標を決めるだけにする。

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

use tsuzuri_contract::graph::{
    EdgeEnd, EdgeType, GraphView, NodeKind, ViewEdge, ViewNode, basis_end, natural_cmp,
};

use super::Graph;

/// 出す節点の上限（見本の値・規則の行にはまだ無い）。
pub const VIEW_CAP: usize = 40;

/// まとめた辺の鍵（D・U・型）。
type EdgeKey<'g> = (&'g str, &'g str, EdgeType);

/// グラフの眺め。
pub fn view(g: &Graph) -> GraphView {
    let index = g.index();
    let degree = g.degrees();

    // 畳み先（規範文 → 条・question → parent-child の先・ruling → answers の先の畳み先）。
    let mut rep: BTreeMap<&str, &str> = index.keys().map(|id| (*id, *id)).collect();
    let target = |from: &str, t: EdgeType| {
        g.edges
            .iter()
            .find(|e| e.from == from && e.edge_type == t && index.contains_key(e.to.as_str()))
            .map(|e| e.to.as_str())
    };
    for (id, n) in &index {
        if n.kind != NodeKind::Norm {
            continue;
        }
        let article = id
            .rsplit_once('.')
            .filter(|(_, tail)| !tail.is_empty() && tail.bytes().all(|c| c.is_ascii_digit()))
            .map(|(head, _)| head)
            .filter(|head| index.get(head).is_some_and(|a| a.kind == NodeKind::Article));
        if let Some(article) = article {
            rep.insert(id, article);
        }
    }
    for (id, n) in &index {
        if n.kind == NodeKind::Question
            && let Some(parent) = target(id, EdgeType::ParentChild)
        {
            rep.insert(id, parent);
        }
    }
    for (id, n) in &index {
        if n.kind == NodeKind::Ruling
            && let Some(question) = target(id, EdgeType::Answers)
        {
            let r = rep[question];
            rep.insert(id, r);
        }
    }
    let mut kids: BTreeMap<&str, usize> = BTreeMap::new();
    for (id, r) in &rep {
        if id != r {
            *kids.entry(r).or_default() += 1;
        }
    }

    // 辺のまとめ（D → U・同じ畳み先の辺は捨てる）。
    let mut merged: BTreeMap<EdgeKey, usize> = BTreeMap::new();
    let mut view_degree: BTreeMap<&str, usize> = BTreeMap::new();
    for e in &g.edges {
        let (Some(a), Some(b)) = (index.get(e.from.as_str()), index.get(e.to.as_str())) else {
            continue;
        };
        let (up, down) = match basis_end(e.edge_type, a.kind, b.kind) {
            EdgeEnd::From => (a.id.as_str(), b.id.as_str()),
            EdgeEnd::To => (b.id.as_str(), a.id.as_str()),
        };
        let (u, d) = (rep[up], rep[down]);
        if u == d {
            continue;
        }
        *merged.entry((d, u, e.edge_type)).or_default() += 1;
        *view_degree.entry(u).or_default() += 1;
        *view_degree.entry(d).or_default() += 1;
    }

    // 出す節点（候補が上限を超えたら、子を持つ節点・眺めの次数の大きい順・id の自然な順で先頭から）。
    let mut candidates: Vec<&str> = view_degree.keys().copied().collect();
    let mut cut = 0;
    if candidates.len() > VIEW_CAP {
        candidates.sort_by(|a, b| {
            let key = |id: &str| (Reverse(kids.contains_key(id)), Reverse(view_degree[id]));
            key(a).cmp(&key(b)).then_with(|| natural_cmp(a, b))
        });
        cut = candidates.len() - VIEW_CAP;
        candidates.truncate(VIEW_CAP);
    }
    let keep: BTreeSet<&str> = candidates.iter().copied().collect();
    let mut edges: Vec<(EdgeKey, usize)> = merged
        .into_iter()
        .filter(|((d, u, _), _)| keep.contains(d) && keep.contains(u))
        .collect();
    edges.sort_by(|((d1, u1, t1), _), ((d2, u2, t2), _)| {
        natural_cmp(d1, d2)
            .then_with(|| natural_cmp(u1, u2))
            .then_with(|| t1.cmp(t2))
    });

    // 段（D から U へたどる最も長い道の辺の数・たどっている途中へ戻れば 0 と数える）。
    let mut outs: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for ((d, u, _), _) in &edges {
        outs.entry(d).or_default().push(u);
    }
    let mut order = candidates.clone();
    order.sort_by(|a, b| natural_cmp(a, b));
    let mut rank: BTreeMap<&str, usize> = BTreeMap::new();
    let mut visiting: BTreeSet<&str> = BTreeSet::new();
    for id in &order {
        rank_of(id, &outs, &mut rank, &mut visiting);
    }

    let folded = rep
        .iter()
        .filter(|(id, r)| id != r && keep.contains(*r))
        .count();
    let isolated = rep
        .iter()
        .filter(|(id, r)| id == r && !view_degree.contains_key(*id))
        .count();

    let mut shown: Vec<&str> = candidates;
    shown.sort_by(|a, b| {
        index[a]
            .kind
            .cmp(&index[b].kind)
            .then_with(|| natural_cmp(a, b))
    });
    let nodes: Vec<ViewNode> = shown
        .iter()
        .map(|id| {
            let n = index[id];
            ViewNode {
                node: n.clone(),
                status: g.status(n),
                rank: count(rank[id]),
                kids: count(kids.get(id).copied().unwrap_or(0)),
                degree: count(degree.get(id).copied().unwrap_or(0)),
            }
        })
        .collect();
    GraphView {
        shown: count(nodes.len()),
        nodes,
        edges: edges
            .into_iter()
            .map(|((d, u, t), n)| ViewEdge {
                from: d.to_string(),
                to: u.to_string(),
                edge_type: t,
                count: count(n),
            })
            .collect(),
        folded: count(folded),
        cut: count(cut + isolated),
        total: count(g.nodes.len()),
        unread: g.unread_wire(),
    }
}

/// 節点の段（一度決めた段は決め直さない）。
fn rank_of<'g>(
    id: &'g str,
    outs: &BTreeMap<&'g str, Vec<&'g str>>,
    rank: &mut BTreeMap<&'g str, usize>,
    visiting: &mut BTreeSet<&'g str>,
) -> usize {
    if let Some(r) = rank.get(id) {
        return *r;
    }
    if !visiting.insert(id) {
        return 0;
    }
    let mut r = 0;
    for to in outs.get(id).into_iter().flatten() {
        r = r.max(rank_of(to, outs, rank, visiting) + 1);
    }
    visiting.remove(id);
    rank.insert(id, r);
    r
}

/// 数を電文の幅へ（溢れたら上限に留める）。
pub(super) fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}
