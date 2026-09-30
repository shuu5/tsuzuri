//! グラフの眺めの組の木と開き（行 c-graph-fold・裁定 t3-hub.52.53）。
//! 節点を帯 → 組 → 節点の木に入れ、どの節点も見えている箱のどれかに畳む（上限で切って落とさない）。
//! 開く列は、初めから開いている帯の後に求めの順に見て、見える箱が上限を越えれば古く開いた箱から畳み直す。

use std::collections::{BTreeMap, BTreeSet};

use tsuzuri_contract::graph::{EdgeType, GraphNode, NodeKind, natural_cmp};

use super::Graph;
use super::view::VIEW_CAP;

/// 箱の子を区切る塊の幅（塊の数が越えれば塊をさらにこの幅ずつまとめる）。
pub const CHUNK: usize = 12;

/// 口の query の open の字を開く列にする（字 , で分けて空の項を捨てる・前後の空白は切らない）。
pub fn open_list(s: &str) -> Vec<String> {
    s.split(',')
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect()
}

/// 節点の種類の帯（面の帯と同じ 7 つ・順も同じ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Band {
    Constitution,
    Rules,
    Adr,
    Srs,
    DesignNote,
    Beads,
    Pipeline,
}

impl Band {
    fn name(self) -> &'static str {
        match self {
            Band::Constitution => "constitution",
            Band::Rules => "rules",
            Band::Adr => "ADR",
            Band::Srs => "SRS",
            Band::DesignNote => "design-note",
            Band::Beads => "beads",
            Band::Pipeline => "pipeline",
        }
    }

    /// 帯の順の先頭の種類（帯の箱の種類）。
    fn head(self) -> NodeKind {
        match self {
            Band::Constitution => NodeKind::Article,
            Band::Rules => NodeKind::Rule,
            Band::Adr => NodeKind::Adr,
            Band::Srs => NodeKind::Goal,
            Band::DesignNote => NodeKind::NoteRow,
            Band::Beads => NodeKind::Epic,
            Band::Pipeline => NodeKind::Run,
        }
    }

    fn of(kind: NodeKind) -> Band {
        use NodeKind::*;
        match kind {
            Article | Norm => Band::Constitution,
            Rule => Band::Rules,
            Adr => Band::Adr,
            Goal | Req | Nfr | Ac | Constraint | Actor | Output => Band::Srs,
            NoteRow => Band::DesignNote,
            Epic | Task | Memo | Question | Ruling | Receipt | Policy => Band::Beads,
            Run => Band::Pipeline,
        }
    }
}

/// 組の id に使う種類の英字（要件書と台帳の種類だけ）。
fn word(kind: NodeKind) -> &'static str {
    use NodeKind::*;
    match kind {
        Goal => "goal",
        Req => "req",
        Nfr => "nfr",
        Ac => "ac",
        Constraint => "constraint",
        Actor => "actor",
        Output => "output",
        Task => "task",
        Memo => "memo",
        Question => "question",
        Ruling => "ruling",
        Receipt => "receipt",
        Policy => "policy",
        _ => "",
    }
}

/// 木の親（節点・組の id と種類・帯）。組と帯の帯は種類から決まる。
enum Up<'g> {
    Node(&'g str),
    Group(String, NodeKind),
    Band,
}

/// 台帳のほかの組と ~run（親の無い・親をたどって帯に着かない節点の置き場）。
fn loose<'g>(kind: NodeKind) -> Up<'g> {
    match kind {
        NodeKind::Run => Up::Group("~run".into(), kind),
        _ => Up::Group(format!("~led:{}", word(kind)), kind),
    }
}

/// 条の系（id の頭の ASCII の英字の並び・無ければ _）。
fn series(id: &str) -> &str {
    let end = id
        .bytes()
        .position(|c| !c.is_ascii_alphabetic())
        .unwrap_or(id.len());
    if end == 0 { "_" } else { &id[..end] }
}

/// 規範文を畳む条（id が 条の id と字 . と数字の並び）。
fn article_of<'g>(id: &'g str, index: &BTreeMap<&'g str, &'g GraphNode>) -> Option<&'g str> {
    id.rsplit_once('.')
        .filter(|(_, tail)| !tail.is_empty() && tail.bytes().all(|c| c.is_ascii_digit()))
        .map(|(head, _)| head)
        .filter(|head| index.get(head).is_some_and(|a| a.kind == NodeKind::Article))
}

/// 組の木の箱（帯の箱・組の箱・塊・節点の箱）。
#[derive(Debug, Clone)]
pub(super) struct TreeBox<'g> {
    pub id: String,
    pub kind: NodeKind,
    pub title: String,
    /// 節点の箱なら節点。
    pub node: Option<&'g GraphNode>,
    pub band: bool,
    pub parent: Option<usize>,
    /// 種類の順・id の自然な順（CHUNK を越えれば塊）。
    pub children: Vec<usize>,
}

/// 組の木。
#[derive(Debug, Clone)]
pub(super) struct Tree<'g> {
    pub boxes: Vec<TreeBox<'g>>,
    /// 帯の箱（帯の順）。
    bands: Vec<usize>,
    /// 節点の id ごとの箱（自分の箱・問いと裁定は畳み先の箱）。
    pub home: BTreeMap<&'g str, usize>,
    by_id: BTreeMap<String, usize>,
}

/// 開きの状態。
#[derive(Debug, Clone, Default)]
pub(super) struct Opening {
    /// 初めから開いている帯（帯の順）。
    initial: Vec<usize>,
    /// 求めで開いた箱（開いた順）。
    pub asked: Vec<usize>,
    /// 開かなかった id（求めの順）。
    pub refused: Vec<String>,
}

impl Opening {
    pub fn is_open(&self, b: usize) -> bool {
        self.initial.contains(&b) || self.asked.contains(&b)
    }
}

impl<'g> Tree<'g> {
    /// 節点を組の木に入れる。
    pub fn new(g: &'g Graph) -> Tree<'g> {
        use NodeKind::*;
        let index = g.index();
        let mut first: BTreeMap<(&'g str, EdgeType), &'g str> = BTreeMap::new();
        let mut checks: BTreeMap<&'g str, Vec<&'g str>> = BTreeMap::new();
        let spec = |k: NodeKind| matches!(k, Req | Nfr);
        for e in &g.edges {
            let (Some(a), Some(b)) = (index.get(e.from.as_str()), index.get(e.to.as_str())) else {
                continue;
            };
            let (from, to) = (a.id.as_str(), b.id.as_str());
            first.entry((from, e.edge_type)).or_insert(to);
            match e.edge_type {
                EdgeType::VerifyAc if spec(a.kind) => checks.entry(to).or_default().push(from),
                EdgeType::Verifies if spec(b.kind) => checks.entry(from).or_default().push(to),
                _ => {}
            }
        }
        // 畳み先（question → parent-child の先・ruling → answers の先）をたどった先（輪なら None）。
        let fold_to = |id: &'g str| {
            let t = match index.get(id)?.kind {
                Question => EdgeType::ParentChild,
                Ruling => EdgeType::Answers,
                _ => return None,
            };
            first.get(&(id, t)).copied()
        };
        let host = |id: &'g str| {
            let mut seen = BTreeSet::from([id]);
            let mut cur = id;
            while let Some(next) = fold_to(cur) {
                if !seen.insert(next) {
                    return None;
                }
                cur = next;
            }
            Some(cur)
        };

        let mut ups: BTreeMap<&'g str, Up<'g>> = BTreeMap::new();
        let mut hidden: BTreeMap<&'g str, &'g str> = BTreeMap::new();
        for (&id, n) in &index {
            let up = match n.kind {
                Question | Ruling => match host(id) {
                    Some(h) if h != id => {
                        hidden.insert(id, h);
                        continue;
                    }
                    _ => loose(n.kind),
                },
                Article => Up::Group(format!("~art:{}", series(id)), Article),
                Norm => match article_of(id, &index) {
                    Some(a) => Up::Node(a),
                    None => Up::Group(format!("~art:{}", series(id)), Article),
                },
                Rule => Up::Group("~rule".into(), Rule),
                Adr => Up::Group("~adr".into(), Adr),
                Ac => checks
                    .get(id)
                    .and_then(|c| c.iter().copied().min_by(|a, b| natural_cmp(a, b)))
                    .map_or_else(|| Up::Group("~srs:ac".into(), Ac), Up::Node),
                Goal | Req | Nfr | Constraint | Actor | Output => {
                    Up::Group(format!("~srs:{}", word(n.kind)), n.kind)
                }
                NoteRow => {
                    let note = id.split_once('#').map_or(id, |(d, _)| d);
                    Up::Group(format!("~note:{note}"), NoteRow)
                }
                Epic => Up::Band,
                Task | Memo | Run => {
                    let t = if n.kind == Run {
                        EdgeType::RunOf
                    } else {
                        EdgeType::ParentChild
                    };
                    first
                        .get(&(id, t))
                        .copied()
                        .filter(|p| {
                            index
                                .get(p)
                                .is_some_and(|n| matches!(n.kind, Epic | Task | Memo))
                        })
                        .map_or_else(|| loose(n.kind), Up::Node)
                }
                Receipt | Policy => loose(n.kind),
            };
            ups.insert(id, up);
        }
        // 親をたどって帯に着かない（輪になる）節点は台帳のほかの組へ。
        let stray: Vec<&'g str> = ups
            .keys()
            .copied()
            .filter(|id| {
                let mut seen = BTreeSet::from([*id]);
                let mut cur = *id;
                while let Some(Up::Node(p)) = ups.get(cur) {
                    if !seen.insert(*p) {
                        return true;
                    }
                    cur = p;
                }
                false
            })
            .collect();
        for id in stray {
            if let Some(n) = index.get(id) {
                ups.insert(id, loose(n.kind));
            }
        }

        let mut t = Tree {
            boxes: Vec::new(),
            bands: Vec::new(),
            home: BTreeMap::new(),
            by_id: BTreeMap::new(),
        };
        for &id in ups.keys() {
            let Some(&n) = index.get(id) else {
                continue;
            };
            let b = t.push(n.id.clone(), n.kind, n.title.clone(), Some(n), false, None);
            t.home.insert(id, b);
        }
        let mut bands: BTreeMap<Band, usize> = BTreeMap::new();
        let mut groups: BTreeMap<String, usize> = BTreeMap::new();
        for (id, up) in &ups {
            let parent = match up {
                Up::Node(p) => t.home.get(p).copied(),
                Up::Band => index
                    .get(id)
                    .map(|n| t.band(&mut bands, Band::of(n.kind))),
                Up::Group(gid, kind) => Some(match groups.get(gid) {
                    Some(b) => *b,
                    None => {
                        let band = t.band(&mut bands, Band::of(*kind));
                        let b = t.push(gid.clone(), *kind, gid.clone(), None, false, Some(band));
                        groups.insert(gid.clone(), b);
                        b
                    }
                }),
            };
            let (Some(parent), Some(&me)) = (parent, t.home.get(id)) else {
                continue;
            };
            if let Some(bx) = t.boxes.get_mut(me) {
                bx.parent = Some(parent);
            }
            if let Some(bx) = t.boxes.get_mut(parent) {
                bx.children.push(me);
            }
        }
        for (id, h) in hidden {
            let Some(&b) = t.home.get(h) else {
                continue;
            };
            t.home.insert(id, b);
        }
        t.bands = bands.into_values().collect();

        // 子を種類の順・自然な順に並べ、帯の箱でない箱の子が CHUNK を越えれば塊に区切る。
        let made = t.boxes.len();
        for b in 0..made {
            let Some(mut kids) = t
                .boxes
                .get_mut(b)
                .map(|bx| std::mem::take(&mut bx.children))
            else {
                continue;
            };
            kids.sort_by(|x, y| t.order(*x, *y));
            let owner = t
                .boxes
                .get(b)
                .filter(|bx| !bx.band && kids.len() > CHUNK)
                .map(|bx| bx.id.clone());
            if let Some(owner) = owner {
                kids = t.chunk(&owner, b, &kids, 0);
            }
            if let Some(bx) = t.boxes.get_mut(b) {
                bx.children = kids;
            }
        }
        for (i, b) in t.boxes.iter().enumerate() {
            t.by_id.entry(b.id.clone()).or_insert(i);
        }
        t
    }

    fn push(
        &mut self,
        id: String,
        kind: NodeKind,
        title: String,
        node: Option<&'g GraphNode>,
        band: bool,
        parent: Option<usize>,
    ) -> usize {
        let b = self.boxes.len();
        self.boxes.push(TreeBox {
            id,
            kind,
            title,
            node,
            band,
            parent,
            children: Vec::new(),
        });
        if let Some(p) = parent.and_then(|p| self.boxes.get_mut(p)) {
            p.children.push(b);
        }
        b
    }

    /// 箱 `x` と `y` の並び（種類の順・自然な順・無い箱は番号の順）。
    pub(super) fn order(&self, x: usize, y: usize) -> std::cmp::Ordering {
        match (self.boxes.get(x), self.boxes.get(y)) {
            (Some(a), Some(b)) => a.kind.cmp(&b.kind).then_with(|| natural_cmp(&a.id, &b.id)),
            _ => x.cmp(&y),
        }
    }

    /// 帯の箱（無ければ作る）。
    fn band(&mut self, bands: &mut BTreeMap<Band, usize>, band: Band) -> usize {
        if let Some(b) = bands.get(&band) {
            return *b;
        }
        let name = band.name();
        let b = self.push(format!("~b:{name}"), band.head(), name.into(), None, true, None);
        bands.insert(band, b);
        b
    }

    /// 箱 `owner` の子の `items`（子の列の `offset` 番目から）を塊に区切る。
    /// 塊の数が CHUNK を越えれば幅を CHUNK の冪にし、塊の中をまた区切る。
    fn chunk(&mut self, owner: &str, parent: usize, items: &[usize], offset: usize) -> Vec<usize> {
        if items.len() <= CHUNK {
            for c in items {
                if let Some(bx) = self.boxes.get_mut(*c) {
                    bx.parent = Some(parent);
                }
            }
            return items.to_vec();
        }
        let mut width = CHUNK;
        while items.len().div_ceil(width) > CHUNK {
            width *= CHUNK;
        }
        let mut out = Vec::new();
        for (i, piece) in items.chunks(width).enumerate() {
            let at = offset + i * width;
            let (Some(head), Some(last)) = (
                piece.first().and_then(|h| self.boxes.get(*h)),
                piece.last().and_then(|l| self.boxes.get(*l)),
            ) else {
                continue;
            };
            let id = format!("{owner}~{}-{}", at + 1, at + piece.len());
            let title = format!("{} … {}", head.id, last.id);
            let kind = head.kind;
            let c = self.push(id, kind, title, None, false, None);
            let kids = self.chunk(owner, c, piece, at);
            if let Some(bx) = self.boxes.get_mut(c) {
                bx.parent = Some(parent);
                bx.children = kids;
            }
            out.push(c);
        }
        out
    }

    /// 親をたどる（自分を含まない）。
    fn ancestors(&self, b: usize) -> impl Iterator<Item = usize> + '_ {
        let parent_of = move |b: usize| self.boxes.get(b).and_then(|x| x.parent);
        std::iter::successors(parent_of(b), move |p| parent_of(*p))
    }

    /// 見える箱の数（開いた帯の箱は数えず、ほかは開いていても 1）。
    fn size(&self, b: usize, st: &Opening) -> usize {
        let Some(bx) = self.boxes.get(b) else {
            return 0;
        };
        let open = st.is_open(b);
        let own = usize::from(!(open && bx.band));
        let inner: usize = if open {
            bx.children.iter().map(|c| self.size(*c, st)).sum()
        } else {
            0
        };
        own + inner
    }

    fn count(&self, st: &Opening) -> usize {
        self.bands.iter().map(|b| self.size(*b, st)).sum()
    }

    /// 箱とその子孫の開きを外す。
    fn close(&self, st: &mut Opening, b: usize) {
        let keep = |x: &usize| *x != b && !self.ancestors(*x).any(|a| a == b);
        st.initial.retain(keep);
        st.asked.retain(keep);
    }

    /// 箱 `b` を開いた状態が上限に収まるよう、`b` とその先祖を除いて、求めで開いた箱の古い順、
    /// 次に初めから開いている帯の順に畳み直す（収まらなければ None）。
    fn fit(&self, mut st: Opening, b: usize) -> Option<Opening> {
        if self.count(&st) <= VIEW_CAP {
            return Some(st);
        }
        let keep: BTreeSet<usize> = std::iter::once(b).chain(self.ancestors(b)).collect();
        let order: Vec<usize> = st
            .asked
            .iter()
            .chain(&st.initial)
            .copied()
            .filter(|c| !keep.contains(c))
            .collect();
        for c in order {
            if st.is_open(c) {
                self.close(&mut st, c);
                if self.count(&st) <= VIEW_CAP {
                    return Some(st);
                }
            }
        }
        None
    }

    /// 初めに帯を帯の順に開き、次に求めの列を順に開く。
    pub fn open(&self, asks: &[String]) -> Opening {
        let mut st = Opening::default();
        for &b in &self.bands {
            let mut next = st.clone();
            next.initial.push(b);
            if let Some(fit) = self.fit(next, b) {
                st = fit;
            }
        }
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for s in asks {
            let Some(&b) = self.by_id.get(s.as_str()) else {
                continue;
            };
            let Some(bx) = self.boxes.get(b) else {
                continue;
            };
            if bx.children.is_empty() || !seen.insert(s) {
                continue;
            }
            if !self.ancestors(b).all(|a| st.is_open(a)) || (!bx.band && st.is_open(b)) {
                continue;
            }
            let mut next = st.clone();
            next.initial.retain(|x| *x != b);
            next.asked.retain(|x| *x != b);
            next.asked.push(b);
            match self.fit(next, b) {
                Some(fit) => st = fit,
                None => st.refused.push(s.clone()),
            }
        }
        st
    }

    /// 見える箱（開いた帯の箱を除く・木の順）。
    pub fn shown(&self, st: &Opening) -> Vec<usize> {
        let mut out = Vec::new();
        let mut stack: Vec<usize> = self.bands.iter().rev().copied().collect();
        while let Some(b) = stack.pop() {
            let Some(bx) = self.boxes.get(b) else {
                continue;
            };
            let open = st.is_open(b);
            if !(open && bx.band) {
                out.push(b);
            }
            if open {
                stack.extend(bx.children.iter().rev().copied());
            }
        }
        out
    }

    /// 箱 `b` の見える箱（自分か、根からたどって最初の開いていない先祖）。
    pub fn visible_of(&self, b: usize, st: &Opening) -> usize {
        let mut path: Vec<usize> = std::iter::once(b).chain(self.ancestors(b)).collect();
        path.reverse();
        path.into_iter()
            .find(|p| *p == b || !st.is_open(*p))
            .unwrap_or(b)
    }
}
