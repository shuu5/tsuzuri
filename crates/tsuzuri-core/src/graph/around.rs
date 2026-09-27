//! 近傍: 1 つの節点から根拠の側（辺を逆にたどる）と影響の側（辺を順にたどる）を各 2 段たどる。
//! 次数が 30 を超える節点は hub で、hub から先は広げず、切った理由と hub の次数を返す（規則の行 R-20 の値）。

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use tsuzuri_contract::graph::{EdgeType, NodeKind};

use super::Graph;

/// たどる段の数（各側・規則の行 R-20「近傍は各 2 段」）。
pub const AROUND_STEPS: u8 = 2;

/// hub の閾値（次数がこれを超える節点は広げない・規則の行 R-20「hub の閾値（次数）30 超で畳む」）。
pub const HUB_DEGREE: usize = 30;

/// 近傍でたどり着いた節点。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reached {
    pub id: String,
    /// 節点の種類（辺の先に節点が無ければ None）。
    pub kind: Option<NodeKind>,
    /// 何段目か（1 から）。
    pub step: u8,
    /// 1 つ前の節点。
    pub via: String,
    pub edge_type: EdgeType,
}

/// 広げずに切った理由（閉じた一覧）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CutReason {
    /// 次数が閾値を超える。
    Hub,
}

/// 広げずに切った節点。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Cut {
    pub id: String,
    pub reason: CutReason,
    pub degree: usize,
}

/// 1 つの節点の近傍。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Around {
    pub center: String,
    /// 根拠の側（辺を逆にたどる）。
    pub basis: Vec<Reached>,
    /// 影響の側（辺を順にたどる）。
    pub impact: Vec<Reached>,
    /// 広げなかった hub（id の重複なし・見つけた順）。
    pub cut: Vec<Cut>,
}

/// 隣の一覧（節点 → (隣・辺の型)）。
type Adjacent<'g> = BTreeMap<&'g str, Vec<(&'g str, EdgeType)>>;

/// 節点の近傍（節点が無ければ None）。
pub fn around(g: &Graph, id: &str) -> Option<Around> {
    g.node(id)?;
    let mut degree: BTreeMap<&str, usize> = BTreeMap::new();
    let mut forward: Adjacent = BTreeMap::new();
    let mut backward: Adjacent = BTreeMap::new();
    for e in &g.edges {
        *degree.entry(&e.from).or_default() += 1;
        *degree.entry(&e.to).or_default() += 1;
        forward
            .entry(&e.from)
            .or_default()
            .push((&e.to, e.edge_type));
        backward
            .entry(&e.to)
            .or_default()
            .push((&e.from, e.edge_type));
    }
    let mut cut: Vec<Cut> = Vec::new();
    let basis = walk(g, id, &backward, &degree, &mut cut);
    let impact = walk(g, id, &forward, &degree, &mut cut);
    Some(Around {
        center: id.to_string(),
        basis,
        impact,
        cut,
    })
}

/// 1 つの側を `AROUND_STEPS` 段たどる。hub は広げず `cut` に足す。
fn walk(
    g: &Graph,
    center: &str,
    adjacent: &Adjacent,
    degree: &BTreeMap<&str, usize>,
    cut: &mut Vec<Cut>,
) -> Vec<Reached> {
    let mut seen: BTreeSet<&str> = BTreeSet::from([center]);
    let mut frontier: Vec<&str> = vec![center];
    let mut reached = Vec::new();
    for step in 1..=AROUND_STEPS {
        let mut next = Vec::new();
        for node in frontier {
            let d = degree.get(node).copied().unwrap_or(0);
            if d > HUB_DEGREE {
                if !cut.iter().any(|c| c.id == node) {
                    cut.push(Cut {
                        id: node.to_string(),
                        reason: CutReason::Hub,
                        degree: d,
                    });
                }
                continue;
            }
            for (to, edge_type) in adjacent.get(node).into_iter().flatten() {
                if seen.insert(to) {
                    reached.push(Reached {
                        id: to.to_string(),
                        kind: g.node(to).map(|n| n.kind),
                        step,
                        via: node.to_string(),
                        edge_type: *edge_type,
                    });
                    next.push(*to);
                }
            }
        }
        frontier = next;
    }
    reached
}
