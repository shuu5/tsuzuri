//! 導出グラフ（設計ノート surface-base 便 c・判断の記録 ADR-7 決定 (2)）。
//! 設計の索引・台帳の一覧・器の event log の 3 つの字から、1 つのグラフを毎回組み直す（`build`）。
//! 不変条件を 3 値で数え（`check`）、1 つの節点の近傍を返す（`around`）。
//! どの関数も file も子 process も触らない。字を読んで口に出す側は境界の crate が持つ。

pub mod around;
pub mod build;
pub mod check;

use std::collections::BTreeMap;

use serde::Serialize;
use tsuzuri_contract::graph::{EdgeType, GraphEdge, GraphNode, NodeKind};

pub use around::{Around, Cut, CutReason, Reached, around};
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
    /// 設計の索引（設計文書の 11 種）。
    Design,
    /// 台帳（bead の 4 種と notes から導く 3 種）。
    Ledger,
    /// 器の event log（走行）。
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
            Source::Runs => &[NodeKind::Run],
        }
    }

    /// 節点の種類の出所（設計ノートの行はこの便では組まないので None）。
    pub fn of(kind: NodeKind) -> Option<Source> {
        Source::ALL.into_iter().find(|s| s.kinds().contains(&kind))
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

/// 走行の属性（段と口座・口座は節点にしない）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RunAttr {
    /// 最後の event の段（器の語のまま）。
    pub stage: Option<String>,
    pub account: Option<String>,
}

/// 組まずに数えた辺の行（型が閉じた一覧に無い）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Skipped {
    /// 設計の索引の辺のうち 17 型の外。
    pub design_edges: usize,
    /// 台帳の dependencies のうち 4 型の外。
    pub ledger_edges: usize,
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
}

impl Graph {
    /// 出所を読めたか。
    pub fn is_read(&self, source: Source) -> bool {
        !self.unread.contains(&source)
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
}
