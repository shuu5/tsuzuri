//! bead の走行の時間軸の電文（節点の頁が読む・設計ノート surface-wave3b 行 e-runs・要件 FR10）。
//! 走行ごとに段の列・審査の結び・口座・費用を持つ。器の event log から中核の pipeline の `runs_of` が組む。
//! 走行ごとの gate と審査の内訳は、便の dir の 2 つの file の字から中核の pipeline の `with_verdicts` が置く（行 c-run-verdict）。

use serde::{Deserialize, Serialize};

use crate::EpochSecs;
use crate::board::Reading;
use crate::ledger::BeadId;

/// 走行の読みの口の path（query の bead で bead を名指す）。
pub const PATH: &str = "/api/runs";

/// bead の走行の列（event log が読めなければ runs は Unknown）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunsDoc {
    pub bead: BeadId,
    pub runs: Reading<Vec<RunLine>>,
}

/// 1 つの走行（並びは RunCreated の log の順）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunLine {
    pub run: String,
    pub started_at: Option<EpochSecs>,
    /// detail の口座の札のうち最後のもの。
    pub account: Option<String>,
    /// 段の event ごとに 1 つ（同じ段の続く event もまとめない）。
    pub steps: Vec<RunStep>,
    pub cost: RunCost,
    /// 便の dir の gate の判定（器の verdict.json・無いか読めないか schema が 1 でなければ Unknown・鍵の無い電文も Unknown）。
    #[serde(default = "unknown")]
    pub gate: Reading<GateVerdict>,
    /// 便の dir の審査の判定（器の review.json・決まりは gate と同じ）。
    #[serde(default = "unknown")]
    pub review: Reading<ReviewVerdict>,
}

fn unknown<T>() -> Reading<T> {
    Reading::Unknown
}

/// 走行の gate の判定（器は便ごとに最後の判定だけを残す）。語は器の字のまま運ぶ（閉じた列で持たない）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateVerdict {
    /// 判定の語（器の 3 語は PASS・FAIL・INCONCLUSIVE）。
    pub verdict: String,
    /// 根拠の全文。
    pub evidence: String,
    /// 観点ごとの件数（器の字の順・0 も持つ）。None は欄が無い（判定が審査役に届かなかった）。
    pub findings: Option<Vec<GateFinding>>,
    /// 判定の時刻（欄 ts）。
    pub at: Option<EpochSecs>,
}

/// gate の 1 つの観点の件数（器の findings の字の 1 札）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateFinding {
    pub category: String,
    pub count: u64,
}

/// 走行の審査の判定（器は便ごとに最後の判定だけを残す）。語は器の字のまま運ぶ。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewVerdict {
    /// 判定の語（器の 3 語は PASS・FAIL・INCONCLUSIVE）。
    pub verdict: String,
    /// 根拠の全文。
    pub evidence: String,
    /// 理由の型（PASS の判定は持たない）。
    pub kind: Option<String>,
    /// 審査役が指した場所の字（器の欄 at）。
    pub place: Option<String>,
    /// 判定の時刻（欄 ts）。
    pub at: Option<EpochSecs>,
}

/// 走行の 1 つの段（欄 stage を持つ event 1 つ）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunStep {
    pub at: Option<EpochSecs>,
    pub stage: String,
    pub detail: Option<String>,
    /// detail の verdict: の札の残り（審査の結び）。
    pub verdict: Option<String>,
    /// detail の kind: の札の残り。
    pub verdict_kind: Option<String>,
}

/// 走行の費用（RunCost の event の和）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunCost {
    pub events: u32,
    pub turns: u64,
    pub wall_ms: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub cache_read: u64,
    pub cache_create: u64,
}
