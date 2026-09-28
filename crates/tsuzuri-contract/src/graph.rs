//! 導出グラフの型: 節点の種類（閉じた 20）・辺の型（閉じた 30）・節点と辺（判断の記録 ADR-7 決定 (2)・設計ノート surface §17・§18）。
//! 設計文書の 11 種と 17 型は folio の語（graph.yaml の node_kinds と edge_types の写し）をそのまま電文の語にする。
//! memo の昇格先の辺（promoted_to）は候補で、型の名と正本は便 c で決めるのでここには置かない。
//! 導出グラフの電文（GraphDoc）は便 e-read で足す（中核の crate の Graph と check の値の写し）。
//! 辺の向き（basis_end）・id の自然な順（natural_cmp）と、グラフの眺めと近傍の電文（GraphView・AroundDoc）は便 c-view で足す。

use std::cmp::Ordering;
use std::collections::BTreeMap;

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
    /// 全体への指示（方針 1 つごとに根の直下に作る閉じた問い〔label policy-scope:〕の notes の定型行「方針 id = 」から導く・
    /// 今までの memo「方針」の notes の同じ頭の行も読む・行 e-policy-q）。
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

/// グラフの節点（id・種類・所属 file・要約値・題 36 字・行・2 つの概要）。
/// 所属 file と要約値は file に書かれた行だけが持つ（台帳と走行の節点は持たない）。
/// 行と概要は folio の要約の字から写す（要件 FR15・写すまでは無し）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub kind: NodeKind,
    pub file: Option<String>,
    pub digest: Option<String>,
    pub title: String,
    /// 所属 file の中で id が書かれた行の番号（1 から数える）。
    #[serde(default)]
    pub line: Option<u32>,
    /// 非エンジニア向けの概要。
    #[serde(default)]
    pub plain: Option<String>,
    /// エンジニア向けの概要。
    #[serde(default)]
    pub eng: Option<String>,
}

/// グラフの辺（from・to・型）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub edge_type: EdgeType,
}

/// 導出グラフの入力の出所（閉じた 3・便 e-read）。読めない出所はその種類の節点だけ「まだ分からない」。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GraphSource {
    /// 設計の索引（設計文書の 11 種）。
    Design,
    /// 台帳（bead の 4 種と notes から導く 3 種）。
    Ledger,
    /// 器の event log（走行）。
    Runs,
}

impl GraphSource {
    /// 閉じた一覧（順も固定）。
    pub const ALL: [GraphSource; 3] = [GraphSource::Design, GraphSource::Ledger, GraphSource::Runs];
}

/// bead の属性（辺にしない欄: 種類・状態・label・pointer の行・touches）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeadAttr {
    pub kind: NodeKind,
    pub status: String,
    pub labels: Vec<String>,
    /// acceptance の中の「design = 」で始まる行（契約の pointer の行）。
    pub pointers: Vec<String>,
    /// metadata の touches の欄の id。
    pub touches: Vec<String>,
}

/// 走行の属性（段と口座・口座は節点にしない）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunAttr {
    /// 最後の event の段（器の語のまま）。
    pub stage: Option<String>,
    pub account: Option<String>,
}

/// 不変条件の判定（3 値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verdict {
    Pass,
    Violation,
    Unknown,
}

impl Verdict {
    /// 閉じた一覧（順も固定）。
    pub const ALL: [Verdict; 3] = [Verdict::Pass, Verdict::Violation, Verdict::Unknown];
}

/// 1 本の不変条件の判定（id・3 値・違反の数・違反の id の一覧）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantCheck {
    pub id: String,
    pub verdict: Verdict,
    pub violations: u32,
    /// 違反が名指す id（名の順・重複なし・違反でなければ空）。
    pub ids: Vec<String>,
}

/// 組まずに数えた行の数（種類か型が閉じた一覧に無い・設計の索引の辺は端が組まなかった節点の行の id の辺も）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedEdges {
    /// 設計の索引の辺のうち 17 型の外と、端が組まなかった節点の行の id の辺。
    pub design: u32,
    /// 台帳の dependencies のうち 4 型の外。
    pub ledger: u32,
    /// 設計の索引の節点の行のうち種類の語が設計の 12 種の外（欄の無い電文は 0）。
    #[serde(default)]
    pub design_nodes: u32,
}

/// 導出グラフの電文（口 GET /api/graph・便 e-read）。repo に書かず、要求のたびに組み直す。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphDoc {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    /// 読めなかった出所（その種類の節点は「まだ分からない」）。
    pub unread: Vec<GraphSource>,
    /// bead の id ごとの属性。
    pub beads: BTreeMap<String, BeadAttr>,
    /// 走行の id ごとの属性。
    pub runs: BTreeMap<String, RunAttr>,
    /// 不変条件の 12 本の判定（id の順）。
    pub invariants: Vec<InvariantCheck>,
    pub skipped: SkippedEdges,
}

/// 辺の端（閉じた 2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeEnd {
    From,
    To,
}

impl EdgeEnd {
    /// 閉じた一覧（順も固定）。
    pub const ALL: [EdgeEnd; 2] = [EdgeEnd::From, EdgeEnd::To];
}

/// 条を含む側から条でない側へ向く型（条から出る辺は from が根拠の側）。
const ARTICLE_SIDE_TYPES: [EdgeType; 6] = [
    EdgeType::InArticle,
    EdgeType::ArticleRef,
    EdgeType::RelationsArticles,
    EdgeType::RelationsReqs,
    EdgeType::RelationsRules,
    EdgeType::RelationsSections,
];

/// 辺の 2 つの端のうち根拠の側（上から順に当てる）。
/// 条を含む型で from が条・to が条でなければ from、verify.ac と amended_by は from、それ以外は to。
/// 対の型（in-article の両向き・article と relations.rules・verify.ac と verifies・amended_by と amends）は、
/// どちらの向きの辺でも同じ節点が根拠の側になる。
pub fn basis_end(edge_type: EdgeType, from: NodeKind, to: NodeKind) -> EdgeEnd {
    if ARTICLE_SIDE_TYPES.contains(&edge_type)
        && from == NodeKind::Article
        && to != NodeKind::Article
    {
        return EdgeEnd::From;
    }
    match edge_type {
        EdgeType::VerifyAc | EdgeType::AmendedBy => EdgeEnd::From,
        _ => EdgeEnd::To,
    }
}

/// id の字を数字の並びとそれ以外の字の並びに分ける。
fn chunks(s: &str) -> impl Iterator<Item = &str> {
    let mut rest = s;
    std::iter::from_fn(move || {
        let first = rest.chars().next()?;
        let digit = first.is_ascii_digit();
        let end = rest
            .char_indices()
            .find(|(_, c)| c.is_ascii_digit() != digit)
            .map_or(rest.len(), |(i, _)| i);
        let (head, tail) = rest.split_at(end);
        rest = tail;
        Some(head)
    })
}

/// id の自然な順（数字の並びは数として、それ以外は字の順で比べる・R-2 は R-10 より前）。
/// 数として同じ並び（01 と 1）は最後に字の順で比べる（同じ字だけが同じ）。
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let mut xs = chunks(a);
    let mut ys = chunks(b);
    loop {
        let ord = match (xs.next(), ys.next()) {
            (None, None) => return a.cmp(b),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                let digits = |s: &str| s.bytes().all(|c| c.is_ascii_digit());
                if digits(x) && digits(y) {
                    let x = x.trim_start_matches('0');
                    let y = y.trim_start_matches('0');
                    x.len().cmp(&y.len()).then_with(|| x.cmp(y))
                } else {
                    x.cmp(y)
                }
            }
        };
        if ord != Ordering::Equal {
            return ord;
        }
    }
}

/// 眺めの節点（節点・状態・段・子の数・次数）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewNode {
    pub node: GraphNode,
    /// bead は属性の状態の字、走行は属性の段の字、ほかは無し。
    pub status: Option<String>,
    /// まとめた辺を影響の側から根拠の側へたどる最も長い道の辺の数。
    pub rank: u32,
    /// この節点へ畳んだ節点の数。
    pub kids: u32,
    /// 辺でつながる隣の節点の数。
    pub degree: u32,
    /// 組の箱なら真（行 c-graph-fold）。
    #[serde(default)]
    pub group: bool,
    /// 箱の開き閉じ（Leaf は開けない・Folded は押すと開く・Open は押すと畳む）。
    #[serde(default)]
    pub fold: BoxFold,
}

/// 眺めの箱の開き閉じ（閉じた 3・行 c-graph-fold）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BoxFold {
    /// 子の無い箱（開けない）。
    #[default]
    Leaf,
    /// 畳んだ箱（押すと開く）。
    Folded,
    /// 開いた箱（押すと畳む）。
    Open,
}

impl BoxFold {
    /// 閉じた一覧（順も固定）。
    pub const ALL: [BoxFold; 3] = [BoxFold::Leaf, BoxFold::Folded, BoxFold::Open];
}

/// 眺めのまとめた辺（from は影響の側・to は根拠の側の畳み先）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewEdge {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub edge_type: EdgeType,
    /// まとめた辺の本数。
    pub count: u32,
}

/// グラフの眺めの電文（節点を組の箱へ畳んだグラフの面・行 c-graph-fold）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphView {
    /// 出す箱（節点の種類の順・同じなら id の自然な順）。
    pub nodes: Vec<ViewNode>,
    /// まとめた辺（from の id の自然な順・to の id の自然な順・辺の型の順）。
    pub edges: Vec<ViewEdge>,
    pub shown: u32,
    /// 箱へ畳まれて自分の箱を持たない節点の数（箱の子の数の和）。
    pub folded: u32,
    /// どの箱にも入らない節点の数（同じ id の 2 つ目の節点だけ・通常 0）。
    pub cut: u32,
    /// グラフの節点の数。
    pub total: u32,
    pub unread: Vec<GraphSource>,
    /// 開いたままの箱の id（開いた順・初めから開いている帯は入らない）。
    #[serde(default)]
    pub open: Vec<String>,
    /// 開くと上限を越え、畳み直しても収まらないので開かなかった id（求めの順）。
    #[serde(default)]
    pub refused: Vec<String>,
}

/// 近傍の畳み（閉じた 4・up は根拠の側を畳む・down は影響の側を畳む）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Fold {
    #[default]
    None,
    Up,
    Down,
    Both,
}

impl Fold {
    /// 閉じた一覧（順も固定）。
    pub const ALL: [Fold; 4] = [Fold::None, Fold::Up, Fold::Down, Fold::Both];

    /// 根拠の側を畳むか。
    pub fn folds_basis(self) -> bool {
        matches!(self, Fold::Up | Fold::Both)
    }

    /// 影響の側を畳むか。
    pub fn folds_impact(self) -> bool {
        matches!(self, Fold::Down | Fold::Both)
    }
}

/// 近傍の行（中心は列 0 で via と type が無い）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AroundRow {
    pub node: GraphNode,
    /// 眺めと同じ決め方の状態。
    pub status: Option<String>,
    /// 列（根拠の側は段の数の負・影響の側は正・-3 から 3）。
    pub col: i8,
    /// 1 つ前の節点。
    pub via: Option<String>,
    /// 着いた辺の型。
    #[serde(rename = "type")]
    pub edge_type: Option<EdgeType>,
    pub degree: u32,
}

/// 広げなかった hub（id と次数）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HubCut {
    pub id: String,
    pub degree: u32,
}

/// 1 つの節点の近傍の電文。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AroundDoc {
    pub center: String,
    pub steps: u8,
    pub fold: Fold,
    /// 出す行（列の小さい順・種類の順・id の自然な順）。
    pub rows: Vec<AroundRow>,
    /// 畳みなしでたどったときの、出す行のうち列が負の数。
    pub basis: u32,
    /// 畳みなしでたどったときの、出す行のうち列が正の数。
    pub impact: u32,
    /// 出す行の数（中心を含む）。
    pub shown: u32,
    /// 着いた節点の数と、中心の 1 と、hub で隠れた数の和。
    pub total: u32,
    /// hub で隠れた数（hub ごとの次数から 1 を引いた数の和）。
    pub cut_hub: u32,
    /// 上限で切った数。
    pub cut_cap: u32,
    /// 広げなかった hub（見つけた順）。
    pub hubs: Vec<HubCut>,
    pub unread: Vec<GraphSource>,
}
