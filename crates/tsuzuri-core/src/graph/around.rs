//! 近傍: 1 つの節点から根拠の側（辺の根拠の側の端へ）と影響の側（根拠の側でない端へ）を段数の回だけたどる
//! （見本 ui.js の around）。次数が 30 を超える節点は hub で、hub から先は広げず、hub の次数を返す（規則の行 R-20 の値）。
//! 出す行は列ごとに 10 まで・合計 40 までに切る。

use std::collections::BTreeMap;
use std::ops::RangeInclusive;

use tsuzuri_contract::graph::{
    AroundDoc, AroundRow, EdgeEnd, EdgeType, Fold, GraphEdge, GraphNode, HubCut, basis_end,
    natural_cmp,
};

use super::Graph;
use super::view::count;

/// 既定の段数（各側・規則の行 R-20「近傍は各 2 段」）。
pub const AROUND_STEPS: u8 = 2;

/// 段数の幅（見本の値）。
pub const AROUND_STEPS_RANGE: RangeInclusive<u8> = 1..=3;

/// hub の閾値（次数がこれを超える節点は広げない・規則の行 R-20「hub の閾値（次数）30 超で畳む」）。
pub const HUB_DEGREE: usize = 30;

/// 1 つの列で出す行の上限（見本の値）。
pub const AROUND_PER_COL: usize = 10;

/// 出す行の合計の上限（中心を含む・見本の値）。
pub const AROUND_CAP: usize = 40;

/// 列を見る順。
const COL_ORDER: [i8; 6] = [-1, 1, -2, 2, -3, 3];

/// 着いた節点（列・何段目か・1 つ前の節点・辺の型）。
struct Reach<'g> {
    col: i8,
    hop: u8,
    via: &'g str,
    edge_type: EdgeType,
}

/// たどった結果（着いた節点・着いた順・広げなかった hub）。
#[derive(Default)]
struct Walked<'g> {
    reached: BTreeMap<&'g str, Reach<'g>>,
    order: Vec<&'g str>,
    hubs: Vec<&'g str>,
}

impl Walked<'_> {
    /// 節点の列（中心と着いていない節点は 0）。
    fn col(&self, id: &str) -> i8 {
        self.reached.get(id).map_or(0, |r| r.col)
    }
}

/// たどる材料（中心・段数・id の表・次数・節点につながる辺と、その辺の根拠の側の端）。
struct Walk<'g> {
    center: &'g str,
    steps: u8,
    index: BTreeMap<&'g str, &'g GraphNode>,
    degree: BTreeMap<&'g str, usize>,
    incident: BTreeMap<&'g str, Vec<(&'g GraphEdge, &'g str)>>,
}

impl<'g> Walk<'g> {
    fn new(g: &'g Graph, center: &'g str, steps: u8) -> Self {
        let index = g.index();
        // 辺の型の字（辺は型の字・from・to の順に字で比べて並べる）。
        let word: BTreeMap<EdgeType, String> = EdgeType::ALL
            .iter()
            .map(|t| (*t, serde_json::to_string(t).unwrap_or_default()))
            .collect();
        let mut incident: BTreeMap<&str, Vec<(&GraphEdge, &str)>> = BTreeMap::new();
        for e in &g.edges {
            let (Some(a), Some(b)) = (index.get(e.from.as_str()), index.get(e.to.as_str())) else {
                continue;
            };
            let up = match basis_end(e.edge_type, a.kind, b.kind) {
                EdgeEnd::From => e.from.as_str(),
                EdgeEnd::To => e.to.as_str(),
            };
            incident.entry(&e.from).or_default().push((e, up));
            if e.to != e.from {
                incident.entry(&e.to).or_default().push((e, up));
            }
        }
        for list in incident.values_mut() {
            list.sort_by(|(x, _), (y, _)| {
                (word.get(&x.edge_type), &x.from, &x.to)
                    .cmp(&(word.get(&y.edge_type), &y.from, &y.to))
            });
        }
        Walk {
            center,
            steps,
            index,
            degree: g.degrees(),
            incident,
        }
    }

    fn degree(&self, id: &str) -> usize {
        self.degree.get(id).copied().unwrap_or(0)
    }

    /// 根拠の側を先に、次に影響の側をたどる（畳んだ側はたどらない・着いた印は 2 つの側で共有する）。
    fn walk(&self, fold: Fold) -> Walked<'g> {
        let mut w = Walked::default();
        for (sign, folded) in [(-1i8, fold.folds_basis()), (1, fold.folds_impact())] {
            if folded {
                continue;
            }
            let mut frontier: Vec<&str> = vec![self.center];
            for hop in 1..=self.steps {
                let mut next = Vec::new();
                for u in frontier {
                    if u != self.center && self.degree(u) > HUB_DEGREE {
                        if !w.hubs.contains(&u) {
                            w.hubs.push(u);
                        }
                        continue;
                    }
                    for (e, up) in self.incident.get(u).into_iter().flatten() {
                        let v = if e.from == u {
                            e.to.as_str()
                        } else {
                            e.from.as_str()
                        };
                        if v == u || v == self.center || w.reached.contains_key(v) {
                            continue;
                        }
                        if (sign < 0) != (*up == v) {
                            continue;
                        }
                        w.reached.insert(
                            v,
                            Reach {
                                col: sign * i8::try_from(hop).unwrap_or(i8::MAX),
                                hop,
                                via: u,
                                edge_type: e.edge_type,
                            },
                        );
                        w.order.push(v);
                        next.push(v);
                    }
                }
                frontier = next;
            }
        }
        w
    }

    /// 出す行（中心と選んだ節点）と上限で切った数。
    /// 列を -1・1・-2・2・-3・3 の順に見て、列の中は種類の順・id の自然な順に並べ、
    /// 列の 10 個目まで・合計 40 に満たず・1 段目か 1 つ前の節点が出す行に入っていれば出す。
    fn select(&self, w: &Walked<'g>) -> (Vec<&'g str>, usize) {
        let mut shown: Vec<&str> = vec![self.center];
        let mut cut = 0;
        for col in COL_ORDER {
            let mut ids: Vec<&str> = w
                .order
                .iter()
                .copied()
                .filter(|id| w.col(id) == col)
                .collect();
            ids.sort_by(|a, b| self.order(a, b));
            for (i, id) in ids.into_iter().enumerate() {
                let Some(r) = w.reached.get(id) else {
                    continue;
                };
                if i < AROUND_PER_COL
                    && shown.len() < AROUND_CAP
                    && (r.hop == 1 || shown.contains(&r.via))
                {
                    shown.push(id);
                } else {
                    cut += 1;
                }
            }
        }
        (shown, cut)
    }

    /// 畳まずに歩いて出す行のうち、根拠の側（列が負）と影響の側（列が正）の数。
    fn sides(&self) -> (usize, usize) {
        let full = self.walk(Fold::None);
        let (rows, _) = self.select(&full);
        (
            rows.iter().filter(|id| full.col(id) < 0).count(),
            rows.iter().filter(|id| full.col(id) > 0).count(),
        )
    }

    /// 節点の種類の順・同じなら id の自然な順。
    fn order(&self, a: &str, b: &str) -> std::cmp::Ordering {
        let kind = |id: &str| self.index.get(id).map(|n| &n.kind);
        kind(a).cmp(&kind(b)).then_with(|| natural_cmp(a, b))
    }
}

/// 節点の近傍（中心の節点が無いときと、段数が幅の外のときは None）。
pub fn around(g: &Graph, center: &str, steps: u8, fold: Fold) -> Option<AroundDoc> {
    if !AROUND_STEPS_RANGE.contains(&steps) {
        return None;
    }
    let center = g.node(center)?.id.as_str();
    let walk = Walk::new(g, center, steps);

    let walked = walk.walk(fold);
    let (mut shown, cut_cap) = walk.select(&walked);
    shown.sort_by(|a, b| {
        walked
            .col(a)
            .cmp(&walked.col(b))
            .then_with(|| walk.order(a, b))
    });
    let (basis, impact) = walk.sides();
    let cut_hub: usize = walked
        .hubs
        .iter()
        .map(|h| walk.degree(h).saturating_sub(1))
        .sum();

    let rows: Vec<AroundRow> = shown
        .iter()
        .filter_map(|id| {
            let n = walk.index.get(id)?;
            let r = walked.reached.get(id);
            Some(AroundRow {
                node: (*n).clone(),
                status: g.status(n),
                col: walked.col(id),
                via: r.map(|r| r.via.to_string()),
                edge_type: r.map(|r| r.edge_type),
                degree: count(walk.degree(id)),
            })
        })
        .collect();
    Some(AroundDoc {
        center: center.to_string(),
        steps,
        fold,
        shown: count(rows.len()),
        rows,
        basis: count(basis),
        impact: count(impact),
        total: count(walked.order.len() + 1 + cut_hub),
        cut_hub: count(cut_hub),
        cut_cap: count(cut_cap),
        hubs: walked
            .hubs
            .iter()
            .map(|h| HubCut {
                id: h.to_string(),
                degree: count(walk.degree(h)),
            })
            .collect(),
        unread: g.unread_wire(),
    })
}
