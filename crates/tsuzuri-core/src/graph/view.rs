//! グラフの眺め: 節点を組の木（`fold`）の見えている箱へ畳み、辺を根拠の側へ向けて箱へまとめ、
//! 根拠の側からの段を決める（見本 map.html の graphModel を裁定 t3-hub.52.53 が改めた形・行 c-graph-fold）。
//! 画面は値を写して座標を決めるだけにする。

use std::collections::{BTreeMap, BTreeSet};

use tsuzuri_contract::graph::{
    BoxFold, EdgeEnd, EdgeType, GraphNode, GraphView, ViewEdge, ViewNode, basis_end, natural_cmp,
};

use super::Graph;
use super::fold::Tree;

/// 見える箱の数の上限（見本の値・規則の行にはまだ無い）。
pub const VIEW_CAP: usize = 40;

/// まとめた辺の鍵（D・U・型）。
type EdgeKey<'g> = (&'g str, &'g str, EdgeType);

/// グラフの眺め（開く列が空・初めから開いている帯だけ）。
pub fn view(g: &Graph) -> GraphView {
    view_open(g, &[])
}

/// 開く列を受けたグラフの眺め（列の決まりは `fold` の `Tree::open`）。
pub fn view_open(g: &Graph, open: &[String]) -> GraphView {
    let tree = Tree::new(g);
    let st = tree.open(open);
    let index = g.index();
    let degree = g.degrees();
    let id_of = |b: usize| tree.boxes[b].id.as_str();

    // 節点ごとの見える箱と、箱ごとの子の数（自分の箱を持たない節点の数）。
    let vis: BTreeMap<&str, usize> = tree
        .home
        .iter()
        .map(|(id, b)| (*id, tree.visible_of(*b, &st)))
        .collect();
    let mut kids: BTreeMap<usize, usize> = BTreeMap::new();
    let mut boxed = 0;
    for (id, b) in &vis {
        if tree.boxes[*b].node.is_some_and(|n| n.id == *id) {
            boxed += 1;
        } else {
            *kids.entry(*b).or_default() += 1;
        }
    }

    // 辺のまとめ（D → U・同じ箱の辺は捨てる）。
    let mut merged: BTreeMap<EdgeKey, usize> = BTreeMap::new();
    let mut next: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for e in &g.edges {
        let (Some(a), Some(b)) = (index.get(e.from.as_str()), index.get(e.to.as_str())) else {
            continue;
        };
        let (up, down) = match basis_end(e.edge_type, a.kind, b.kind) {
            EdgeEnd::From => (a.id.as_str(), b.id.as_str()),
            EdgeEnd::To => (b.id.as_str(), a.id.as_str()),
        };
        let (u, d) = (vis[up], vis[down]);
        if u == d {
            continue;
        }
        let (u, d) = (id_of(u), id_of(d));
        *merged.entry((d, u, e.edge_type)).or_default() += 1;
        next.entry(u).or_default().insert(d);
        next.entry(d).or_default().insert(u);
    }
    let mut edges: Vec<(EdgeKey, usize)> = merged.into_iter().collect();
    edges.sort_by(|((d1, u1, t1), _), ((d2, u2, t2), _)| {
        natural_cmp(d1, d2)
            .then_with(|| natural_cmp(u1, u2))
            .then_with(|| t1.cmp(t2))
    });

    // 段（D から U へたどる最も長い道の辺の数・たどっている途中へ戻れば 0 と数える）。
    let mut shown = tree.shown(&st);
    let mut outs: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for ((d, u, _), _) in &edges {
        outs.entry(d).or_default().push(u);
    }
    let mut order: Vec<&str> = shown.iter().map(|b| id_of(*b)).collect();
    order.sort_by(|a, b| natural_cmp(a, b));
    let mut rank: BTreeMap<&str, usize> = BTreeMap::new();
    let mut visiting: BTreeSet<&str> = BTreeSet::new();
    for id in &order {
        rank_of(id, &outs, &mut rank, &mut visiting);
    }

    shown.sort_by(|a, b| {
        let (x, y) = (&tree.boxes[*a], &tree.boxes[*b]);
        x.kind.cmp(&y.kind).then_with(|| natural_cmp(&x.id, &y.id))
    });
    let nodes: Vec<ViewNode> = shown
        .iter()
        .map(|b| {
            let bx = &tree.boxes[*b];
            let id = bx.id.as_str();
            let fold = if bx.children.is_empty() {
                BoxFold::Leaf
            } else if st.is_open(*b) {
                BoxFold::Open
            } else {
                BoxFold::Folded
            };
            let (node, status, deg) = match bx.node {
                Some(n) => (n.clone(), g.status(n), degree.get(id).copied().unwrap_or(0)),
                None => (
                    GraphNode {
                        id: bx.id.clone(),
                        kind: bx.kind,
                        file: None,
                        digest: None,
                        title: bx.title.clone(),
                        line: None,
                        plain: None,
                        eng: None,
                        updated: None,
                    },
                    None,
                    next.get(id).map_or(0, BTreeSet::len),
                ),
            };
            ViewNode {
                node,
                status,
                rank: count(rank.get(id).copied().unwrap_or(0)),
                kids: count(kids.get(b).copied().unwrap_or(0)),
                degree: count(deg),
                group: bx.node.is_none(),
                fold,
            }
        })
        .collect();
    let folded: usize = kids.values().sum();
    let total = g.nodes.len();
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
        cut: count(total.saturating_sub(boxed + folded)),
        total: count(total),
        unread: g.unread_wire(),
        open: st.asked.iter().map(|b| id_of(*b).to_string()).collect(),
        refused: st.refused,
    }
}

/// 箱の段（一度決めた段は決め直さない）。
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
