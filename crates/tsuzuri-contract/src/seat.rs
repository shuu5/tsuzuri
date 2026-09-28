//! 席の card の電文（画面の block「orchestrator と口座」が読む・設計ノート surface-base 便 b-cards）。
//! card を組むのは server で、ここは電文の形だけを決める。読めない出所はその欄だけ `Reading::Unknown` にする。

use serde::{Deserialize, Serialize};

use crate::EpochSecs;
use crate::board::{GroupRow, Reading};

/// 席の状態（閉じた 5: 動いている・待っている・限度・応答なし・測れていない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SeatState {
    Run,
    Wait,
    Limit,
    Silent,
    Unknown,
}

impl SeatState {
    pub const ALL: [SeatState; 5] = [
        SeatState::Run,
        SeatState::Wait,
        SeatState::Limit,
        SeatState::Silent,
        SeatState::Unknown,
    ];
}

/// 器の管理 tick の健康（器の doctor の席の行の tick の語・閉じた 4: 健全・古い・打刻が無い・読めない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TickHealth {
    Healthy,
    Stale,
    Absent,
    Unreadable,
}

impl TickHealth {
    pub const ALL: [TickHealth; 4] = [
        TickHealth::Healthy,
        TickHealth::Stale,
        TickHealth::Absent,
        TickHealth::Unreadable,
    ];

    /// 器の語（電文の字と同じ）。
    pub fn as_str(self) -> &'static str {
        match self {
            TickHealth::Healthy => "healthy",
            TickHealth::Stale => "stale",
            TickHealth::Absent => "absent",
            TickHealth::Unreadable => "unreadable",
        }
    }
}

fn unknown() -> Reading<TickHealth> {
    Reading::Unknown
}

/// 口座の窓の使った割合（窓の名・使った百分率・戻る時刻・限度の判定で数える窓か）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotaUsed {
    pub window: String,
    pub used_pct: u8,
    pub resets_at: Option<EpochSecs>,
    pub counted: bool,
}

/// 状態の区間（from から to まで state だった）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeatSpan {
    pub from: EpochSecs,
    pub to: EpochSecs,
    pub state: SeatState,
}

/// 口座の移動（移る前の口座が分からなければ from は None）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountMove {
    pub at: EpochSecs,
    pub from: Option<String>,
    pub to: String,
}

/// 席の card（口 seat の出力）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeatCard {
    /// 組んだ時刻。
    pub at: EpochSecs,
    /// 席の名。
    pub target: String,
    pub state: SeatState,
    /// 今の状態になった時刻。
    pub since: Option<EpochSecs>,
    /// 器の合図の健康。
    pub tick_healthy: Reading<bool>,
    /// 停止の切り替えが on か。
    pub heartbeat: Reading<bool>,
    /// 器の doctor の席の行の tick の語（4 つの語のほかと、行か語が無いときは Unknown）。
    #[serde(default = "unknown")]
    pub tick: Reading<TickHealth>,
    /// 合図の最後の判定の時刻。
    #[serde(default)]
    pub tick_at: Option<EpochSecs>,
    /// 登録の口座。
    pub account: Option<String>,
    pub model: Option<String>,
    /// 群の名・今の口座・候補・次の移り先。
    pub group: Reading<GroupRow>,
    /// 席の口座の窓ごとの使った割合。
    pub usage: Reading<Vec<QuotaUsed>>,
    /// 直近 24 時間の状態の区間。
    pub spans: Reading<Vec<SeatSpan>>,
    /// 口座の移動の履歴。
    pub moves: Reading<Vec<AccountMove>>,
}
