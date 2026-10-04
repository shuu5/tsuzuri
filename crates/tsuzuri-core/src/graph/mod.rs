//! 導出グラフ（設計ノート surface-base 便 c・判断の記録 ADR-7 決定 (2)）。
//! 設計の索引・台帳の一覧・器の event log の 3 つの字から、1 つのグラフを毎回組み直す（`build`）。
//! 不変条件を 3 値で数え（`check`）、1 つの節点の近傍を返す（`around`）。
//! どの関数も file も子 process も触らない。字を読んで口に出す側は境界の crate が持つ。
//! 裁定の書き出し（folio check --emit-rulings の行）は build の後に `build::add_rulings` が節点へ結ぶ（行 c-g3g7）。
//! ほかの project の台帳（外の台帳）は節点にも辺にもせず、`Graph::outside` に読みだけを置いて g-3 が族で確かめる（行 c-g3-extern）。
//! 着地の commit は event log の RunDone から組み（行 c-commit-node）、着地の commit の無い着地した契約は床の値で
//! `check::unlanded_contracts` が名指す。

pub mod around;
pub mod build;
pub mod check;

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use tsuzuri_contract::graph::{EdgeType, GraphEdge, GraphNode, GraphSource, NodeKind};

pub use around::around;
pub use build::build;
pub use check::{Invariant, Verdict, check};

/// 組む材料の 3 つの字。
#[derive(Debug, Clone, Copy)]
pub struct Inputs<'a> {
    /// 設計の索引（設計の道具の graph の口が出す節点と辺の表・タブ区切り）。
    pub design_index: &'a str,
    /// 台帳の一覧（bd の読み取りの口が返す JSON の配列）。
    pub ledger: &'a str,
    /// 器の event log（1 行 1 件の JSON）。
    pub events: &'a str,
}

/// 入力の出所（閉じた 3）。読めない出所はその種類の節点だけ「まだ分からない」にする。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    /// 設計の索引（設計文書の 11 種と設計ノートの行）。
    Design,
    /// 台帳（bead の 4 種と notes から導く 3 種）。
    Ledger,
    /// 器の event log（走行と着地の commit）。
    Runs,
}

impl Source {
    pub const ALL: [Source; 3] = [Source::Design, Source::Ledger, Source::Runs];

    /// この出所から組む節点の種類。
    pub fn kinds(self) -> &'static [NodeKind] {
        match self {
            Source::Design => &NodeKind::ALL[..build::DESIGN_KINDS],
            Source::Ledger => &[
                NodeKind::Epic,
                NodeKind::Task,
                NodeKind::Memo,
                NodeKind::Question,
                NodeKind::Ruling,
                NodeKind::Receipt,
                NodeKind::Policy,
            ],
            Source::Runs => &[NodeKind::Run, NodeKind::Commit],
        }
    }

    /// 節点の種類の出所（閉じた 21 の種類はどれも 1 つの出所に当たる）。
    pub fn of(kind: NodeKind) -> Option<Source> {
        Source::ALL.into_iter().find(|s| s.kinds().contains(&kind))
    }

    /// 契約の型の出所への写し（字は同じ）。
    pub fn wire(self) -> GraphSource {
        match self {
            Source::Design => GraphSource::Design,
            Source::Ledger => GraphSource::Ledger,
            Source::Runs => GraphSource::Runs,
        }
    }
}

/// bead の属性（辺にしない欄: 種類・状態・label・pointer の行・metadata の touches）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BeadAttr {
    pub kind: NodeKind,
    pub status: String,
    pub labels: Vec<String>,
    /// acceptance の中の「design = 」で始まる行（契約の pointer の行）。
    pub pointers: Vec<String>,
    /// metadata の touches の欄の id。
    pub touches: Vec<String>,
}

impl BeadAttr {
    /// open か（closed と tombstone でない）。
    pub fn is_open(&self) -> bool {
        self.status != "closed" && self.status != "tombstone"
    }
}

/// 走行の属性（段と口座と答えの無い問いの数・口座は節点にしない）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RunAttr {
    /// 最後の event の段（器の語のまま）。
    pub stage: Option<String>,
    pub account: Option<String>,
    /// 答えの無い問いの数。問いは走行の QuestionRaised の 1 つずつで、
    /// n 番目の問いは n 番目の QuestionRaised から次の QuestionRaised の直前までに
    /// 同じ走行の QuestionAnswered が在れば答えが在る。
    pub unanswered: usize,
}

/// 裁定の書き出しの行の形（閉じた 3・folio の字）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RulingForm {
    /// 器の問いの印の付いた裁定 id（台帳の notes の裁定の節点の id）。
    Question,
    /// notes の日時の付いた裁定 id（節点の id にならない）。
    NotesTime,
    /// 台帳の id と同じ字。
    Bead,
}

/// 裁定の書き出しの 1 行（folio check --emit-rulings・欄 line は読み捨てる）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RulingRow {
    pub ruling: String,
    pub form: RulingForm,
    pub bead: String,
    /// 承認の字を持つ節点の id（設計ノートの承認欄・判断の表の行・承認欄の stamp の行では無し）。
    pub node: Option<String>,
    pub file: String,
    /// 承認の字の欄の道（例 `meta.approval[0].ruling`）。
    pub field: String,
}

impl RulingRow {
    /// ruled_by の辺の先（問いの形は裁定 id・ほかは台帳の id）。
    pub fn target(&self) -> &str {
        match self.form {
            RulingForm::Question => &self.ruling,
            RulingForm::NotesTime | RulingForm::Bead => &self.bead,
        }
    }
}

/// ほかの project の台帳の読み（外の台帳・欄はどれも字の集まり・`build::outside` が組む・行 c-g3-extern）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Outside {
    /// bead の id の族（最初の「.」の前の字）。
    pub families: BTreeSet<String>,
    /// bead の id。
    pub beads: BTreeSet<String>,
    /// notes の裁定の定型行の id。
    pub rulings: BTreeSet<String>,
}

impl Outside {
    /// 裁定の書き出しの行の先が在るか（問いの形は notes の裁定の定型行の id・ほかは bead の id・`Graph::holds` と同じ分け）。
    pub fn holds(&self, row: &RulingRow) -> bool {
        match row.form {
            RulingForm::Question => self.rulings.contains(&row.ruling),
            RulingForm::NotesTime | RulingForm::Bead => self.beads.contains(&row.bead),
        }
    }
}

/// 方針の属性（範囲）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PolicyAttr {
    /// 方針の定型行の範囲の欄（字 `all` か問いの id）。
    pub scope: String,
}

/// 組まずに数えた行（種類か型が閉じた一覧に無い・設計の索引の辺は端が組まなかった節点の行の id の辺も）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Skipped {
    /// 設計の索引の辺のうち 19 型の外と、端が組まなかった節点の行の id の辺。
    pub design_edges: usize,
    /// 台帳の dependencies のうち 4 型の外。
    pub ledger_edges: usize,
    /// 設計の索引の節点の行のうち種類の語が設計の 12 種の外。
    pub design_nodes: usize,
}

/// 導出グラフ（repo に書かない・毎回組み直す）。
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Graph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    /// 読めなかった出所（その種類の節点は「まだ分からない」）。
    pub unread: Vec<Source>,
    pub skipped: Skipped,
    /// bead の id ごとの属性。
    pub beads: BTreeMap<String, BeadAttr>,
    /// 走行の id ごとの属性。
    pub runs: BTreeMap<String, RunAttr>,
    /// 方針の id ごとの属性。
    pub policies: BTreeMap<String, PolicyAttr>,
    /// 節点の id ごとの結んだ裁定の書き出しの行（`build::add_rulings` が置く・読めなければ無し）。
    pub rulings: Option<BTreeMap<String, Vec<RulingRow>>>,
    /// ほかの project の台帳の読み（置き場の順・読めない置き場は無し・既定は空の列・g-3 だけが見る・行 c-g3-extern）。
    pub outside: Vec<Option<Outside>>,
}

impl Graph {
    /// 出所を読めたか。
    pub fn is_read(&self, source: Source) -> bool {
        !self.unread.contains(&source)
    }

    /// 裁定の書き出しの行の先が在るか（問いの形は同じ id の裁定の節点・ほかは台帳の bead）。
    pub fn holds(&self, row: &RulingRow) -> bool {
        match row.form {
            RulingForm::Question => self
                .nodes
                .iter()
                .any(|n| n.kind == NodeKind::Ruling && n.id == row.ruling),
            RulingForm::NotesTime | RulingForm::Bead => self.beads.contains_key(&row.bead),
        }
    }

    /// 「まだ分からない」の節点の種類（読めない出所の種類の全部）。
    pub fn unknown_kinds(&self) -> Vec<NodeKind> {
        self.unread
            .iter()
            .flat_map(|s| s.kinds().iter().copied())
            .collect()
    }

    /// 種類ごとの節点の数。
    pub fn count_nodes(&self, kind: NodeKind) -> usize {
        self.nodes.iter().filter(|n| n.kind == kind).count()
    }

    /// 型ごとの辺の数。
    pub fn count_edges(&self, edge_type: EdgeType) -> usize {
        self.edges
            .iter()
            .filter(|e| e.edge_type == edge_type)
            .count()
    }

    /// id の節点（無ければ None）。
    pub fn node(&self, id: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// id から節点への表（同じ id が 2 つ在れば先の節点・`node` と同じ）。
    pub fn index(&self) -> BTreeMap<&str, &GraphNode> {
        let mut index = BTreeMap::new();
        for n in &self.nodes {
            index.entry(n.id.as_str()).or_insert(n);
        }
        index
    }

    /// 節点ごとの次数（辺でつながる隣の節点の数）。
    /// 同じ隣は辺が何本でも 1 と数え、自分へ戻る辺と、端の節点が無い辺は数えない。
    pub fn degrees(&self) -> BTreeMap<&str, usize> {
        let index = self.index();
        let mut next: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for e in &self.edges {
            let both = index.contains_key(e.from.as_str()) && index.contains_key(e.to.as_str());
            if e.from == e.to || !both {
                continue;
            }
            next.entry(&e.from).or_default().insert(&e.to);
            next.entry(&e.to).or_default().insert(&e.from);
        }
        next.into_iter().map(|(id, s)| (id, s.len())).collect()
    }

    /// 節点の状態（bead は属性の状態の字・走行は属性の段の字・ほかは無し）。
    pub fn status(&self, node: &GraphNode) -> Option<String> {
        if node.kind == NodeKind::Run {
            return self.runs.get(&node.id).and_then(|r| r.stage.clone());
        }
        self.beads.get(&node.id).map(|b| b.status.clone())
    }

    /// 読めなかった出所の契約の型の列。
    pub fn unread_wire(&self) -> Vec<GraphSource> {
        self.unread.iter().map(|s| s.wire()).collect()
    }
}
