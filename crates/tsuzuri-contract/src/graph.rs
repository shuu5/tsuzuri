//! 導出グラフの型: 節点の種類（閉じた 20）・辺の型（閉じた 30）・節点と辺（判断の記録 ADR-7 決定 (2)・設計ノート surface §17・§18）。
//! 設計文書の 11 種と 17 型は folio の語（graph.yaml の node_kinds と edge_types の写し）をそのまま電文の語にする。
//! memo の昇格先の辺（promoted_to）は候補で、型の名と正本は便 c で決めるのでここには置かない。

use serde::{Deserialize, Serialize};

/// 節点の種類（閉じた 20・順は設計文書の 11 種・設計ノートの行・台帳の 7 種・走行）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    #[serde(rename = "条")]
    Article,
    #[serde(rename = "規範文")]
    Norm,
    #[serde(rename = "規則行")]
    Rule,
    #[serde(rename = "目的")]
    Goal,
    #[serde(rename = "要件")]
    Req,
    #[serde(rename = "非機能要件")]
    Nfr,
    #[serde(rename = "受入基準")]
    Ac,
    #[serde(rename = "制約")]
    Constraint,
    #[serde(rename = "登場人物")]
    Actor,
    #[serde(rename = "出力")]
    Output,
    #[serde(rename = "判断の記録")]
    Adr,
    /// 設計ノートの行（id は `<文書 id>#<行 id>`）。
    #[serde(rename = "note-row")]
    NoteRow,
    #[serde(rename = "epic")]
    Epic,
    /// 契約（acceptance に設計の pointer 行を持つ bead）。
    #[serde(rename = "task")]
    Task,
    #[serde(rename = "memo")]
    Memo,
    #[serde(rename = "question")]
    Question,
    /// あなたの決定（問いの notes の定型行から導く）。
    #[serde(rename = "ruling")]
    Ruling,
    /// orchestrator の受け（問いの notes の「受け」の定型行から導く）。
    #[serde(rename = "receipt")]
    Receipt,
    /// 全体への指示（根の直下の memo「方針」の notes の定型行から導く）。
    #[serde(rename = "policy")]
    Policy,
    /// 走行（器の event RunCreated から導く）。
    #[serde(rename = "run")]
    Run,
}

impl NodeKind {
    /// 閉じた一覧（順も固定）。
    pub const ALL: [NodeKind; 20] = [
        NodeKind::Article,
        NodeKind::Norm,
        NodeKind::Rule,
        NodeKind::Goal,
        NodeKind::Req,
        NodeKind::Nfr,
        NodeKind::Ac,
        NodeKind::Constraint,
        NodeKind::Actor,
        NodeKind::Output,
        NodeKind::Adr,
        NodeKind::NoteRow,
        NodeKind::Epic,
        NodeKind::Task,
        NodeKind::Memo,
        NodeKind::Question,
        NodeKind::Ruling,
        NodeKind::Receipt,
        NodeKind::Policy,
        NodeKind::Run,
    ];
}

/// 辺の型（閉じた 30・順は設計文書の 17 型・台帳の 4 型・結びの 6 型・走行の 3 型）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    #[serde(rename = "in-article")]
    InArticle,
    #[serde(rename = "relations.articles")]
    RelationsArticles,
    #[serde(rename = "relations.reqs")]
    RelationsReqs,
    #[serde(rename = "relations.rules")]
    RelationsRules,
    #[serde(rename = "relations.sections")]
    RelationsSections,
    #[serde(rename = "amended_by")]
    AmendedBy,
    #[serde(rename = "article")]
    ArticleRef,
    #[serde(rename = "refs")]
    Refs,
    #[serde(rename = "basis")]
    Basis,
    #[serde(rename = "goals")]
    Goals,
    #[serde(rename = "rules")]
    Rules,
    #[serde(rename = "adrs")]
    Adrs,
    #[serde(rename = "verify.ac")]
    VerifyAc,
    #[serde(rename = "verifies")]
    Verifies,
    #[serde(rename = "figures")]
    Figures,
    #[serde(rename = "produced")]
    Produced,
    #[serde(rename = "amends")]
    Amends,
    #[serde(rename = "parent-child")]
    ParentChild,
    #[serde(rename = "blocks")]
    Blocks,
    #[serde(rename = "relates-to")]
    RelatesTo,
    #[serde(rename = "discovered-from")]
    DiscoveredFrom,
    /// 契約 → 設計ノートの行。
    #[serde(rename = "design")]
    Design,
    /// 判断の記録・規則行・発効した文書 → 裁定。
    #[serde(rename = "ruled_by")]
    RuledBy,
    /// 裁定 → 問い。
    #[serde(rename = "answers")]
    Answers,
    /// 問い → 任意の節点。
    #[serde(rename = "touches")]
    Touches,
    /// 問い → 方針。
    #[serde(rename = "premises")]
    Premises,
    /// memo → 裁定か走行。
    #[serde(rename = "source")]
    Source,
    /// 走行 → 契約 bead。
    #[serde(rename = "run_of")]
    RunOf,
    /// 走行 → 問い。
    #[serde(rename = "raised")]
    Raised,
    /// 走行 → 口座（口座は節点にせず走行の属性の札で出す）。
    #[serde(rename = "ran_by")]
    RanBy,
}

impl EdgeType {
    /// 閉じた一覧（順も固定）。
    pub const ALL: [EdgeType; 30] = [
        EdgeType::InArticle,
        EdgeType::RelationsArticles,
        EdgeType::RelationsReqs,
        EdgeType::RelationsRules,
        EdgeType::RelationsSections,
        EdgeType::AmendedBy,
        EdgeType::ArticleRef,
        EdgeType::Refs,
        EdgeType::Basis,
        EdgeType::Goals,
        EdgeType::Rules,
        EdgeType::Adrs,
        EdgeType::VerifyAc,
        EdgeType::Verifies,
        EdgeType::Figures,
        EdgeType::Produced,
        EdgeType::Amends,
        EdgeType::ParentChild,
        EdgeType::Blocks,
        EdgeType::RelatesTo,
        EdgeType::DiscoveredFrom,
        EdgeType::Design,
        EdgeType::RuledBy,
        EdgeType::Answers,
        EdgeType::Touches,
        EdgeType::Premises,
        EdgeType::Source,
        EdgeType::RunOf,
        EdgeType::Raised,
        EdgeType::RanBy,
    ];
}

/// 題の字数の上限（Unicode の字・folio の索引の題と同じ）。
pub const TITLE_MAX: usize = 36;

/// 題の形（空白を 1 つに畳んで Unicode の字で 36 に切った 1 行）。
pub fn title36(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(TITLE_MAX)
        .collect()
}

/// グラフの節点（id・種類・所属 file・要約値・題 36 字）。
/// 所属 file と要約値は file に書かれた行だけが持つ（台帳と走行の節点は持たない）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub kind: NodeKind,
    pub file: Option<String>,
    pub digest: Option<String>,
    pub title: String,
}

/// グラフの辺（from・to・型）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub edge_type: EdgeType,
}
