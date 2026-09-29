//! 木の面（行 g-map-tree・持ち主の裁定 t3-hub.52.32 の案 A）: 設計と台帳の全体を葉までたどれる入れ子の一覧。
//! 設計の木は 帯 → 条 → 規範文・要件 → 受入基準・設計ノート → 行、台帳の木は epic → task → run・問い → 裁定。
//! 親子は辺の型の表（`NESTS`）の 1 か所だけで決め、親の端は契約の basis_end で引く（対の辺はどちらの向きでも同じ親）。
//! 閉じた項目だけを既定で畳む（開き閉じは kit の fold の記録で頁の一生の間だけ保つ）。
//! 木は電文の節点と辺から組む純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use std::collections::{BTreeMap, BTreeSet, HashSet};

use tsuzuri_contract::graph::{
    EdgeEnd, EdgeType, GraphDoc, GraphNode, GraphSource, NodeKind, basis_end, title36,
};

use super::band::{Band, band_of, unread_reason};
use super::{View, natural, open_question, retired_notes, shape_class, state};
use crate::kit::Item;

/// 木の面（閉じた 2・tab の順）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tree {
    Design,
    Ledger,
}

impl Tree {
    /// 閉じた一覧（順も固定）。
    pub const ALL: [Tree; 2] = [Tree::Design, Tree::Ledger];

    /// 地図の面（面の名と語の鍵は View が持つ）。
    pub fn view(self) -> View {
        match self {
            Tree::Design => View::Design,
            Tree::Ledger => View::Ledger,
        }
    }

    /// 木に入る帯（Band の ALL の順）。
    pub fn bands(self) -> &'static [Band] {
        match self {
            Tree::Design => &[
                Band::Constitution,
                Band::Rules,
                Band::Adr,
                Band::Srs,
                Band::DesignNote,
            ],
            Tree::Ledger => &[Band::Beads, Band::Pipeline],
        }
    }
}

/// 台帳の bead の 4 種。
pub const ISSUE_KINDS: [NodeKind; 4] = [
    NodeKind::Epic,
    NodeKind::Task,
    NodeKind::Memo,
    NodeKind::Question,
];

/// 親子の辺の型の表の 1 行（型・親の種類・子の種類）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nest {
    pub edge_type: EdgeType,
    pub parents: &'static [NodeKind],
    pub children: &'static [NodeKind],
}

const REQS: &[NodeKind] = &[NodeKind::Req, NodeKind::Nfr];

/// 親子の辺の型の表（木の親子はこの表の 1 か所だけで決める・ほかの型は木の親子にしない）。
pub const NESTS: [Nest; 6] = [
    Nest {
        edge_type: EdgeType::InArticle,
        parents: &[NodeKind::Article],
        children: &[NodeKind::Norm],
    },
    Nest {
        edge_type: EdgeType::VerifyAc,
        parents: REQS,
        children: &[NodeKind::Ac],
    },
    Nest {
        edge_type: EdgeType::Verifies,
        parents: REQS,
        children: &[NodeKind::Ac],
    },
    Nest {
        edge_type: EdgeType::ParentChild,
        parents: &ISSUE_KINDS,
        children: &ISSUE_KINDS,
    },
    Nest {
        edge_type: EdgeType::Answers,
        parents: &[NodeKind::Question],
        children: &[NodeKind::Ruling],
    },
    Nest {
        edge_type: EdgeType::RunOf,
        parents: &ISSUE_KINDS,
        children: &[NodeKind::Run],
    },
];

/// 辺の親の端（表の行の型で、basis_end の端が親の種類・他方が子の種類のときだけ・ほかは None）。
pub fn parent_end(edge_type: EdgeType, from: NodeKind, to: NodeKind) -> Option<EdgeEnd> {
    let end = basis_end(edge_type, from, to);
    let (parent, child) = match end {
        EdgeEnd::From => (from, to),
        EdgeEnd::To => (to, from),
    };
    NESTS
        .iter()
        .any(|n| {
            n.edge_type == edge_type && n.parents.contains(&parent) && n.children.contains(&child)
        })
        .then_some(end)
}

/// 閉じた項目の状態（閉じた bead・却下と置き換えの設計の項目）。
pub fn is_closed(state: Option<&str>) -> bool {
    matches!(state, Some("closed" | "rejected" | "superseded"))
}

/// 項の頭（帯・設計ノート・節点）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Head {
    /// 設計の木の 1 段目の帯（count は帯の節点の数）。
    Band { band: Band, count: usize },
    /// design-note の帯の下の設計ノート（note は行の id の最初の「#」より前の字・count は束ねた項の数）。
    Note { note: String, count: usize },
    /// design-note の帯の末尾の、廃止した設計ノートを束ねた段（count は束ねたノートの数・行 g-map-retired）。
    Retired(usize),
    Node(Item),
}

/// 木の 1 項（open は開き閉じの記録に値が無いときの初めの値）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub head: Head,
    pub open: bool,
    pub kids: Vec<Branch>,
}

/// 木の面の中身（読めなかった出所の理由と 1 段目の項）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Forest {
    pub unread: Vec<&'static str>,
    pub items: Vec<Branch>,
}

/// 開き閉じの記録の鍵（同じ節点が 2 か所に出れば同じ鍵で開き閉じを分ける）。
pub fn fold_key(head: &Head) -> String {
    match head {
        Head::Band { band, .. } => format!("tree:band:{}", band.name()),
        Head::Note { note, .. } => format!("tree:note:{note}"),
        Head::Retired(_) => "tree:retired".to_string(),
        Head::Node(item) => format!("tree:node:{}", item.id),
    }
}

/// 木の節点と親子（節点は (種類, id の自然な順) に並べ、子は同じ順の番号の集合）。
struct Graph<'a> {
    nodes: Vec<&'a GraphNode>,
    kids: Vec<BTreeSet<usize>>,
    has_parent: Vec<bool>,
}

impl<'a> Graph<'a> {
    fn new(doc: &'a GraphDoc, tree: Tree) -> Self {
        let bands = tree.bands();
        let mut seen = HashSet::new();
        let mut nodes: Vec<&GraphNode> = doc
            .nodes
            .iter()
            .filter(|n| {
                let band = band_of(n.kind);
                bands.contains(&band) && !doc.unread.contains(&band.source())
            })
            .filter(|n| seen.insert(n.id.as_str()))
            .collect();
        nodes.sort_by(|a, b| a.kind.cmp(&b.kind).then_with(|| natural(&a.id, &b.id)));
        let at: BTreeMap<&str, usize> = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id.as_str(), i))
            .collect();
        let mut kids = vec![BTreeSet::new(); nodes.len()];
        let mut has_parent = vec![false; nodes.len()];
        for e in &doc.edges {
            let (Some(&f), Some(&t)) = (at.get(e.from.as_str()), at.get(e.to.as_str())) else {
                continue;
            };
            let (parent, child) = match parent_end(e.edge_type, nodes[f].kind, nodes[t].kind) {
                Some(EdgeEnd::From) => (f, t),
                Some(EdgeEnd::To) => (t, f),
                None => continue,
            };
            kids[parent].insert(child);
            has_parent[child] = true;
        }
        Self {
            nodes,
            kids,
            has_parent,
        }
    }

    /// 根の番号（親の無い節点・どの根からもたどれない節点は残りの最初を根に足す・(種類, id) の順）。
    fn roots(&self, doc: &GraphDoc) -> Vec<(usize, Branch)> {
        let mut placed = vec![false; self.nodes.len()];
        let mut path = vec![false; self.nodes.len()];
        let mut roots: Vec<(usize, Branch)> = (0..self.nodes.len())
            .filter(|&i| !self.has_parent[i])
            .map(|i| (i, self.branch(doc, i, &mut path, &mut placed)))
            .collect();
        while let Some(i) = placed.iter().position(|p| !p) {
            roots.push((i, self.branch(doc, i, &mut path, &mut placed)));
        }
        roots.sort_by_key(|(i, _)| *i);
        roots
    }

    /// 節点 i の項（たどっている道の上の節点は子にしない）。
    fn branch(&self, doc: &GraphDoc, i: usize, path: &mut [bool], placed: &mut [bool]) -> Branch {
        placed[i] = true;
        path[i] = true;
        let mut kids = Vec::new();
        for &k in &self.kids[i] {
            if !path[k] {
                kids.push(self.branch(doc, k, path, placed));
            }
        }
        path[i] = false;
        let node = self.nodes[i];
        Branch {
            head: Head::Node(Item {
                shape: shape_class(doc, node),
                alert: open_question(doc, node),
                id: node.id.clone(),
                title: title36(&node.title),
                aside: String::new(),
                stage: None,
            }),
            open: !kids.is_empty() && !is_closed(state(doc, node)),
            kids,
        }
    }
}

/// 木の面の中身を組む（電文の節点と辺だけを読む）。
pub fn forest(doc: &GraphDoc, tree: Tree) -> Forest {
    let bands = tree.bands();
    let unread = GraphSource::ALL
        .into_iter()
        .filter(|s| doc.unread.contains(s) && bands.iter().any(|b| b.source() == *s))
        .map(unread_reason)
        .collect::<Vec<_>>();
    let g = Graph::new(doc, tree);
    let roots = g.roots(doc);
    let items = match tree {
        Tree::Ledger => roots.into_iter().map(|(_, b)| b).collect(),
        Tree::Design if !unread.is_empty() => Vec::new(),
        Tree::Design => bands
            .iter()
            .map(|&band| {
                let count = g.nodes.iter().filter(|n| band_of(n.kind) == band).count();
                let mine: Vec<Branch> = roots
                    .iter()
                    .filter(|(i, _)| band_of(g.nodes[*i].kind) == band)
                    .map(|(_, b)| b.clone())
                    .collect();
                let kids = if band == Band::DesignNote {
                    notes(mine, &retired_notes(doc))
                } else {
                    mine
                };
                Branch {
                    head: Head::Band { band, count },
                    open: !kids.is_empty(),
                    kids,
                }
            })
            .collect(),
    };
    Forest { unread, items }
}

/// design-note の帯の項: 「#」を持つ項を最初の「#」より前の字ごとに束ね（note の自然な順）、「#」を持たない項を後に置く。
/// `retired` が名指したノートは束ねた項を畳み、末尾の 1 つの段にまとめる（名指したノートが無ければ段も無い）。
fn notes(items: Vec<Branch>, retired: &BTreeSet<&str>) -> Vec<Branch> {
    let mut groups: Vec<(String, Vec<Branch>)> = Vec::new();
    let mut loose = Vec::new();
    for b in items {
        let note = match &b.head {
            Head::Node(item) => item.id.split_once('#').map(|(n, _)| n.to_string()),
            _ => None,
        };
        match note {
            Some(note) => match groups.iter_mut().find(|(n, _)| *n == note) {
                Some((_, kids)) => kids.push(b),
                None => groups.push((note, vec![b])),
            },
            None => loose.push(b),
        }
    }
    groups.sort_by(|a, b| natural(&a.0, &b.0));
    let (gone, kept): (Vec<_>, Vec<_>) = groups
        .into_iter()
        .partition(|(note, _)| retired.contains(note.as_str()));
    let note_branch = |open: bool| {
        move |(note, kids): (String, Vec<Branch>)| Branch {
            head: Head::Note {
                note,
                count: kids.len(),
            },
            open,
            kids,
        }
    };
    let shelf = (!gone.is_empty()).then(|| Branch {
        head: Head::Retired(gone.len()),
        open: false,
        kids: gone.into_iter().map(note_branch(false)).collect(),
    });
    kept.into_iter()
        .map(note_branch(true))
        .chain(loose)
        .chain(shelf)
        .collect()
}

#[cfg(target_arch = "wasm32")]
pub use dom::view;

/// 木の面の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::ev;
    use leptos::prelude::*;
    use tsuzuri_contract::graph::GraphDoc;
    use web_sys::wasm_bindgen::JsCast;

    use super::{Branch, Head, Tree, fold_key, forest};
    use crate::frame::{Mode, node_href};
    use crate::mapview::{RETIRED, band_chip, retired_unread};
    use crate::project::nodearound::mode_of;
    use crate::project::{ALERT_STYLE, fold, unmeasured};
    use crate::vocab::label;
    use crate::widgets::hover::{delegate, leaves};
    use crate::widgets::nodecard::card_of;

    /// event の的から最も近い項の題の a の id（項の題の中でなければ None）。
    fn item_key(target: Option<web_sys::EventTarget>) -> Option<String> {
        let el: web_sys::Element = target?.dyn_into().ok()?;
        el.closest("a[data-key]")
            .ok()
            .flatten()?
            .get_attribute("data-key")
    }

    pub fn view(doc: &GraphDoc, tree: Tree, search: RwSignal<String>) -> AnyView {
        let mode = mode_of(search);
        let f = forest(doc, tree);
        let wire = StoredValue::new(doc.clone());
        let hc = delegate();
        // 項の card は指が入った項だけをそのとき組む（同じ項の中の移りでは出し直さない）。
        let over = move |ev: ev::MouseEvent| {
            let card = item_key(ev.target()).and_then(|k| wire.with_value(|d| card_of(d, &k)));
            if let Some(card) = card {
                hc.show(&ev, card);
            }
        };
        let out = move |ev: ev::MouseEvent| {
            if leaves(
                item_key(ev.target()).as_deref(),
                item_key(ev.related_target()).as_deref(),
            ) {
                hc.leave(&ev);
            }
        };
        let unread = f
            .unread
            .iter()
            .map(|&reason| unmeasured(reason))
            .collect_view();
        let retired = (tree == Tree::Design)
            .then(|| retired_unread(doc))
            .flatten()
            .map(unmeasured);
        let empty = (f.unread.is_empty() && f.items.is_empty()).then(|| {
            view! { <div class="empty"><span>{label(tree.view().key())}</span><b class="num">"0"</b></div> }
        });
        let items = f
            .items
            .into_iter()
            .map(|b| branch_view(b, mode))
            .collect_view();
        view! {
            {unread}
            {retired}
            {empty}
            <ul class="items" on:mouseover=over on:mouseout=out>{items}</ul>
        }
        .into_any()
    }

    /// 項の頭（帯の chip と数・設計ノートの名と数・節点の印と題の link と子の数）。
    fn head_view<M>(head: &Head, kids: usize, mode: M) -> AnyView
    where
        M: Fn() -> Mode + Copy + Send + Sync + 'static,
    {
        match head {
            Head::Band { band, count } => {
                view! { {band_chip(*band)}<span class="chip num">{*count}</span> }.into_any()
            }
            Head::Note { note, count } => view! {
                <span class="nid">{note.clone()}</span><span class="chip num">{*count}</span>
            }
            .into_any(),
            Head::Retired(count) => view! {
                <span class="mono">{RETIRED}</span><span class="chip num">{*count}</span>
            }
            .into_any(),
            Head::Node(item) => {
                let style = if item.alert { ALERT_STYLE } else { "" };
                let id = item.id.clone();
                let href = move || node_href(&id, mode());
                let n = (kids > 0).then(|| view! { <span class="chip num">{kids}</span> });
                view! {
                    <span class=item.shape.clone() style=style aria-hidden="true"></span>
                    <a class="ttl" href=href data-key=item.id.clone()><span class="nid">{item.id.clone()}</span>" "<span data-t="">{item.title.clone()}</span></a>
                    {n}
                }
                .into_any()
            }
        }
    }

    /// 1 項（子の在る項は畳める段・開いている間だけ子の項を組む）。
    fn branch_view<M>(b: Branch, mode: M) -> AnyView
    where
        M: Fn() -> Mode + Copy + Send + Sync + 'static,
    {
        let head = head_view(&b.head, b.kids.len(), mode);
        if b.kids.is_empty() {
            return view! { <li>{head}</li> }.into_any();
        }
        let initial = b.open;
        let (open, record) = fold(fold_key(&b.head), move || initial);
        let shown = RwSignal::new(open());
        let toggle = move |ev: web_sys::Event| {
            shown.set(event_target::<web_sys::Element>(&ev).has_attribute("open"));
            record(ev);
        };
        let kids = StoredValue::new(b.kids);
        let nest = move || {
            shown.get().then(|| {
                let items = kids.with_value(|ks| {
                    ks.iter()
                        .cloned()
                        .map(|k| branch_view(k, mode))
                        .collect_view()
                });
                view! { <ul class="items">{items}</ul> }
            })
        };
        view! {
            <li class="nest">
                <details class="fold" prop:open=open on:toggle=toggle>
                    <summary>{head}</summary>
                    {nest}
                </details>
            </li>
        }
        .into_any()
    }
}
