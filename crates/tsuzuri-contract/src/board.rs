//! 板の型: pipeline の札と板・account board の data・次の一手（閉じた 7）・台帳の判定（閉じた 5）
//! （判断の記録 ADR-7 決定 (5)〜(8)・設計ノート surface §20）。

use serde::{Deserialize, Serialize};

use crate::EpochSecs;
use crate::ledger::BeadId;
use crate::surface::{SeatHealth, SeatRole};

/// 読めたか「まだ分からない」か（読めない出所はその種類だけ Unknown にする・0 件と区別する）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Reading<T> {
    Known(T),
    Unknown,
}

/// 走行の段（段の名は英語のまま面に出す）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stage {
    Queued,
    Blocked,
    Running,
    Gated,
    Questioned,
    Failed,
    Stopped,
    Landed,
}

impl Stage {
    pub const ALL: [Stage; 8] = [
        Stage::Queued,
        Stage::Blocked,
        Stage::Running,
        Stage::Gated,
        Stage::Questioned,
        Stage::Failed,
        Stage::Stopped,
        Stage::Landed,
    ];

    /// 段が載る pipeline の列。
    pub fn column(self) -> PipelineColumn {
        match self {
            Stage::Queued | Stage::Blocked => PipelineColumn::QueuedBlocked,
            Stage::Running | Stage::Gated => PipelineColumn::RunningGated,
            Stage::Questioned | Stage::Failed | Stage::Stopped => {
                PipelineColumn::QuestionedFailedStopped
            }
            Stage::Landed => PipelineColumn::Landed,
        }
    }
}

/// pipeline の板の列（閉じた 4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipelineColumn {
    #[serde(rename = "Queued/Blocked")]
    QueuedBlocked,
    #[serde(rename = "Running/Gated")]
    RunningGated,
    #[serde(rename = "Questioned/Failed/Stopped")]
    QuestionedFailedStopped,
    #[serde(rename = "Landed")]
    Landed,
}

/// pipeline の札（契約 bead・走行の回数・段・段の理由・口座・経過）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineCard {
    pub contract: BeadId,
    pub runs: u32,
    pub stage: Stage,
    pub reason: Option<String>,
    pub account: Option<String>,
    pub elapsed_s: Option<u64>,
}

/// pipeline の板（state dir が読めなければ札は「まだ分からない」）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelineBoard {
    pub cards: Reading<Vec<PipelineCard>>,
}

/// 次の一手（閉じた 7・宣言の順が優先の順）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NextMove {
    /// 限度と移動。
    LimitOrMove,
    /// 応答なし。
    Unresponsive,
    /// 止まっている走行。
    StalledRun,
    /// 束の承認。
    BatchApproval,
    /// 質問。
    Question,
    /// 発効待ち。
    AwaitingEffect,
    /// なし。
    Nothing,
}

impl NextMove {
    /// 閉じた一覧（優先の順）。
    pub const ALL: [NextMove; 7] = [
        NextMove::LimitOrMove,
        NextMove::Unresponsive,
        NextMove::StalledRun,
        NextMove::BatchApproval,
        NextMove::Question,
        NextMove::AwaitingEffect,
        NextMove::Nothing,
    ];
}

/// 台帳の判定（閉じた 5: 順調・停滞・滞り・積み増し・台帳なし）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LedgerJudge {
    OnTrack,
    Stalled,
    Clogged,
    PilingUp,
    NoLedger,
}

impl LedgerJudge {
    pub const ALL: [LedgerJudge; 5] = [
        LedgerJudge::OnTrack,
        LedgerJudge::Stalled,
        LedgerJudge::Clogged,
        LedgerJudge::PilingUp,
        LedgerJudge::NoLedger,
    ];
}

/// 口座の窓の残量（窓の名と残りの百分率）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotaLeft {
    pub window: String,
    pub left_pct: u8,
}

/// 群と口座の行（今の口座は 1 つ・候補は集合・次の移り先は器の doctor の値の写しで None は移り先なし）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupRow {
    pub group: String,
    pub account: String,
    pub candidates: Vec<String>,
    pub next_account: Option<String>,
    pub remaining: Vec<QuotaLeft>,
}

/// session の行（project → role → session → 口座 → 段 → 経過 → 稼働）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRow {
    pub project: String,
    pub role: SeatRole,
    pub session: String,
    pub account: String,
    pub stage: Option<Stage>,
    pub elapsed_s: Option<u64>,
    pub health: SeatHealth,
}

/// project の指標（判定 1 語と主指標と次の一手・作業中の数と最古は持たない）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectMetrics {
    pub project: String,
    pub judge: LedgerJudge,
    pub open_tasks: u32,
    pub net_drop_24h: i64,
    pub net_drop_7d: i64,
    pub closed_per_day: f64,
    pub unreflected: u32,
    pub memos: u32,
    pub next_move: NextMove,
}

/// account board の data（時点・群と口座・session の行・各 project の指標と次の一手）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountBoard {
    pub at: EpochSecs,
    pub groups: Vec<GroupRow>,
    pub sessions: Vec<SessionRow>,
    pub projects: Vec<ProjectMetrics>,
}
