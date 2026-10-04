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
/// Held（留め置き）は依存でない理由で人の手が入るまで器が起こさない契約（席の止めと受付の断り）で、列は Blocked と同じ
/// （判断の記録 ADR-42 決定 (1)(3)・行 c-held-stage）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stage {
    Queued,
    Blocked,
    Held,
    Running,
    Gated,
    Questioned,
    Failed,
    Stopped,
    Landed,
}

impl Stage {
    pub const ALL: [Stage; 9] = [
        Stage::Queued,
        Stage::Blocked,
        Stage::Held,
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
            Stage::Blocked | Stage::Held => PipelineColumn::Blocked,
            Stage::Queued => PipelineColumn::Queued,
            Stage::Running | Stage::Gated => PipelineColumn::RunningGated,
            Stage::Questioned | Stage::Failed | Stage::Stopped => {
                PipelineColumn::QuestionedFailedStopped
            }
            Stage::Landed => PipelineColumn::Landed,
        }
    }
}

/// pipeline の板の列（閉じた 5・板の順・判断の記録 ADR-27 決定 (6)）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipelineColumn {
    #[serde(rename = "Blocked")]
    Blocked,
    #[serde(rename = "Queued")]
    Queued,
    #[serde(rename = "Running/Gated")]
    RunningGated,
    #[serde(rename = "Questioned/Failed/Stopped")]
    QuestionedFailedStopped,
    #[serde(rename = "Landed")]
    Landed,
}

/// 着地の後の CI の読み（閉じた 7・器の RunDone の終端の detail の語から読む・行 c-pipe-ci）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Ci {
    /// push の後で CI の結果が無い。
    Waiting,
    Success,
    /// 器は bead を閉じない。
    Failure,
    /// 器は bead を閉じない。
    Unmeasurable,
    /// push が落ちた。
    PushFailed,
    /// CI は通ったが close が落ちた。
    CloseFailed,
    /// 終端の宣言が読めない。
    Unreadable,
}

impl Ci {
    pub const ALL: [Ci; 7] = [
        Ci::Waiting,
        Ci::Success,
        Ci::Failure,
        Ci::Unmeasurable,
        Ci::PushFailed,
        Ci::CloseFailed,
        Ci::Unreadable,
    ];
}

/// pipeline の札（契約 bead・走行の回数・段・段の理由・口座・段を決めた時刻・着地の後の CI）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineCard {
    pub contract: BeadId,
    pub runs: u32,
    pub stage: Stage,
    pub reason: Option<String>,
    pub account: Option<String>,
    /// 段を決めた最後の event の時刻（走行の無い札は None・経過は面が今から引く）。
    pub since: Option<EpochSecs>,
    /// 着地の後の CI の読み（器が着地を push した走行だけ Some）。
    pub ci: Option<Ci>,
}

/// 形の崩れ（語は器の ledger/form.rs の欄の名・判断の記録 ADR-16 の決定 (1)）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Misfit {
    /// 控えの印も設計の参照も無い。
    Neither,
    /// 控えの印と設計の参照の両方が在る。
    Both,
}

impl Misfit {
    pub const ALL: [Misfit; 2] = [Misfit::Neither, Misfit::Both];
}

/// 形の崩れた open の bead（題は台帳の字のまま）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MisfitBead {
    pub bead: BeadId,
    pub title: String,
    pub misfit: Misfit,
}

/// pipeline の板（state dir が読めなければ札は「まだ分からない」）。
/// 形の崩れは器の doctor の判定を写し、その字か台帳が読めなければ「まだ分からない」。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelineBoard {
    pub cards: Reading<Vec<PipelineCard>>,
    /// 形の崩れた open の bead（器の doctor の台帳の形の行を写し、題は台帳から引く・台帳の順・行 c-pipe-misfit）。
    pub misfits: Reading<Vec<MisfitBead>>,
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
    /// 器の区画の行の写しか（器の doctor の群の行の欄 kind が park・false は電文に字を置かない・行 c-park-acct）。
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub park: bool,
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
