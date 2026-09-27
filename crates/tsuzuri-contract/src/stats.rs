//! 台帳の指標と次の一手と未反映の一覧の電文（設計ノート surface-base 便 d・便 e-view・判断の記録 ADR-7 決定 (6)・(8)）。
//! 数え方は中核の crate が持ち、ここは電文の形だけを決める。作業中の数と最古の task の欄は持たない。
//! 台帳が読めなければ指標は `Reading::Unknown` で運び、判定は台帳なしにする。

use serde::{Deserialize, Serialize};

use crate::EpochSecs;
use crate::board::{LedgerJudge, NextMove, Reading};
use crate::ledger::BeadId;

/// 種類ごとの open の数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenCounts {
    pub task: u32,
    pub memo: u32,
    pub question: u32,
    pub epic: u32,
}

/// lead（直近 30 日に閉じた task の created_at から closed_at までの日数）の中央値と p90。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LeadDays {
    pub p50: f64,
    pub p90: f64,
}

/// 1 日の数（その日に作られた task・閉じた task・その日の終わりの open の task）。burndown と sparkline の材料。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayCount {
    /// その日の終わりの時刻。
    pub end: EpochSecs,
    pub created: u32,
    pub closed: u32,
    pub open: u32,
}

/// epic の進み（直下の task の閉じた数と全体の数）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpicProgress {
    pub epic: BeadId,
    pub closed: u32,
    pub total: u32,
}

/// memo の数（open・昇格待ち・直近 7 日に閉じた数・open の memo の年齢の中央値（日））。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MemoStats {
    pub open: u32,
    pub awaiting_promotion: u32,
    pub closed_7d: u32,
    pub age_p50_days: Option<f64>,
}

/// 未反映の種類（閉じた 3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnreflectedKind {
    Memo,
    /// 処分の宣言の無い裁定。
    Ruling,
    /// 反映されていない要望。
    Request,
}

impl UnreflectedKind {
    pub const ALL: [UnreflectedKind; 3] = [
        UnreflectedKind::Memo,
        UnreflectedKind::Ruling,
        UnreflectedKind::Request,
    ];
}

/// 未反映の 1 件（bead の id の字・題・年齢の秒・作った時刻が読めなければ年齢は None）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnreflectedRow {
    pub id: String,
    pub title: String,
    pub age_s: Option<u64>,
}

/// 未反映の一覧（種類ごとに、読めた一覧か「まだ分からない」・便 e-view）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnreflectedList {
    pub memos: Reading<Vec<UnreflectedRow>>,
    pub rulings: Reading<Vec<UnreflectedRow>>,
    pub requests: Reading<Vec<UnreflectedRow>>,
}

/// 台帳の指標（時点・判定・主指標・日ごとの 14 本・epic の進み・memo・未反映）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerStats {
    pub at: EpochSecs,
    pub judge: LedgerJudge,
    pub open: OpenCounts,
    pub blocked: u32,
    pub ready: u32,
    pub stale: u32,
    pub net_drop_24h: i64,
    pub net_drop_7d: i64,
    pub closed_7d: u32,
    pub closed_per_day: f64,
    /// 直近 30 日に閉じた task が無ければ None。
    pub lead: Option<LeadDays>,
    /// 古い日から順に 14 本。
    pub days: Vec<DayCount>,
    pub epics: Vec<EpicProgress>,
    pub memo: MemoStats,
    /// 未反映の数（読めた種類の数の和・今は memo の数）。
    pub unreflected: u32,
    /// 未反映のうち「まだ分からない」種類。
    pub unreflected_unknown: Vec<UnreflectedKind>,
}

/// 次の一手の 1 種を判じた結果（閉じた 3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckResult {
    /// 当たった。
    Hit,
    /// 当たらない。
    Miss,
    /// 判じなかった（材料が入力に無い・読めない）。
    NotJudged,
}

impl CheckResult {
    pub const ALL: [CheckResult; 3] = [CheckResult::Hit, CheckResult::Miss, CheckResult::NotJudged];
}

/// 次の一手の 1 種の結果（種類・結果・件数・対象の id）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextCheck {
    pub kind: NextMove,
    pub result: CheckResult,
    pub count: u32,
    pub target: Option<BeadId>,
}

/// 次の一手（7 種の結果を優先の順に並べ、最初に当たった 1 つを大きく出す）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextStep {
    pub checks: Vec<NextCheck>,
    pub lead: NextMove,
}

impl NextMove {
    /// 日本語の名（規則の行 R-24 の字）。
    pub fn label(self) -> &'static str {
        match self {
            NextMove::LimitOrMove => "限度と移動",
            NextMove::Unresponsive => "応答なし",
            NextMove::StalledRun => "止まっている走行",
            NextMove::BatchApproval => "束の承認",
            NextMove::Question => "質問",
            NextMove::AwaitingEffect => "発効待ち",
            NextMove::Nothing => "なし",
        }
    }
}
