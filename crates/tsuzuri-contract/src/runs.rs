//! bead の走行の時間軸の電文（節点の頁が読む・設計ノート surface-wave3b 行 e-runs・要件 FR10）。
//! 走行ごとに段の列・審査の結び・口座・費用を持つ。器の event log から中核の pipeline の `runs_of` が組む。

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunLine {
    pub run: String,
    pub started_at: Option<EpochSecs>,
    /// detail の口座の札のうち最後のもの。
    pub account: Option<String>,
    /// 段の event ごとに 1 つ（同じ段の続く event もまとめない）。
    pub steps: Vec<RunStep>,
    pub cost: RunCost,
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
