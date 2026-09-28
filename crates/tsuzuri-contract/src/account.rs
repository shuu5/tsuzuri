//! account board の電文（口座ごとの窓・群の枠・移動・project の行・session の行・停止の切り替え・設計ノート surface-base 便 b-acct）。
//! 電文を組むのは server で、ここは電文の形と口の path だけを決める。読めない出所はその部分だけ `Reading::Unknown` にする。
//! 着地済みの `board::AccountBoard` は使わない（窓・席の card・移動・部分ごとの測れていない・停止の切り替えを持たない）。

use serde::{Deserialize, Serialize};

use crate::EpochSecs;
use crate::board::{GroupRow, Reading, Stage};
use crate::seat::{QuotaUsed, SeatCard, SeatSpan, SeatState};
use crate::stats::{LedgerStats, NextStep};
use crate::surface::SeatRole;

/// 読みの口の path（面と server はこの定数を使う）。
pub const PATH: &str = "/api/account";

/// 停止の切り替えの口の path（面と server はこの定数を使う）。
pub const HEARTBEAT_PATH: &str = "/api/account/heartbeat";

/// account board の data（時点・口座・閾値・群・移動・知らせ・project・session・休止中の席）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountDoc {
    /// 組んだ時刻。
    pub at: EpochSecs,
    pub accounts: Reading<Vec<AccountRow>>,
    /// 窓ごとの逼迫の閾値（器の rules 行の写し）。
    pub caps: Vec<WindowCap>,
    pub groups: Reading<Vec<GroupCard>>,
    pub moves: Reading<Vec<MoveRow>>,
    /// 群の逼迫の知らせと移動の断り（器の event の写し・新しい順）。
    pub notices: Reading<Vec<GroupNotice>>,
    pub projects: Vec<ProjectRow>,
    pub sessions: Vec<SessionLine>,
    /// 休止中の席（器の登録の行の席のうち、状態の記録の最後の読めた行から規則の行 R-28 の境を越えた席・行 c-dormant）。
    pub dormant: Vec<DormantSeat>,
}

/// 休止中の席（経過の秒は電文の at から last を引く・欄に置かない）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DormantSeat {
    /// 登録の行の anchor の project の名（anchor が無ければ空の字）。
    pub project: String,
    pub target: String,
    /// 登録の口座（空なら None）。
    pub account: Option<String>,
    /// 状態の記録の最後の読めた行の ts。
    pub last: EpochSecs,
    /// 合図の健康の出力の席の行の healthy と heartbeat（行が無ければ Unknown）。
    pub tick_healthy: Reading<bool>,
    pub heartbeat: Reading<bool>,
}

/// 口座の 7 日の線の幅の秒（持ち主の裁定 t3-hub.53.10・行 c-acct-spark）。
pub const SPARK_SPAN_S: u64 = 7 * 24 * 60 * 60;

/// 口座の 7 日の線の刻みの秒（刻みごとに 1 点・1 本 `SPARK_SPAN_S / SPARK_STEP_S` 点以下）。
pub const SPARK_STEP_S: u64 = 60 * 60;

/// 口座の行（名・退役・占有の群・model の窓の model の名・窓ごとの使った割合・7 日の線）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountRow {
    pub label: String,
    pub retired: bool,
    /// 占有している群の名（どの群の今の口座でもなければ None）。
    pub occupant: Option<String>,
    pub model: Option<String>,
    pub usage: Reading<Vec<QuotaUsed>>,
    /// 器の event log の測りの行の線（log が読めなければ Unknown・行 c-acct-spark）。
    pub spark: Reading<Spark>,
}

/// 口座の 7 日の線（最後に測った時刻と窓ごとの線）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spark {
    /// 口座の読めた測りの行の ts の最大（窓と古さを問わない・行が無ければ None）。
    pub measured_at: Option<EpochSecs>,
    /// 窓ごとの線（five_hour・seven_day・seven_day_model の順）。
    pub lines: Vec<SparkLine>,
}

/// 1 つの窓の線（点は at の古い順）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparkLine {
    pub window: String,
    pub points: Vec<SparkPoint>,
}

/// 線の 1 点（測りの行の ts と使った割合）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparkPoint {
    pub at: EpochSecs,
    pub used_pct: u8,
}

/// 窓の逼迫の閾値の 1 行（窓の名・器の rules 行の id・行の値・読めなければ cap は Unknown）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowCap {
    pub window: String,
    pub rule: String,
    pub cap: Reading<u64>,
}

/// 群の逼迫の知らせと移動の断りの行（器の event の欄の写し・tsuzuri は判じない）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum GroupNotice {
    /// 逼迫の知らせ（account は逼迫の口座・window は窓の名・sent は知らせを送った席の数）。
    Pressure {
        at: EpochSecs,
        group: String,
        account: String,
        window: String,
        used: u64,
        cap: u64,
        sent: u64,
    },
    /// 移動の断り（account は断った周の群の今の口座・reason は器の理由の語）。
    Refused {
        at: EpochSecs,
        group: String,
        account: String,
        reason: String,
    },
}

/// 群の枠（群と口座の行・今の記録・project の列・doctor の断り）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupCard {
    pub row: GroupRow,
    /// 今の記録の ts（記録が無ければ None）。
    pub since: Option<EpochSecs>,
    /// 前の口座。
    pub previous: Option<String>,
    /// 今の記録が在るか。
    pub recorded: bool,
    pub members: Vec<GroupMember>,
    /// doctor の refused の字（無ければ None）。
    pub refused: Option<String>,
}

/// 群の project（席の口座が群の今の口座と一致するか・席の行が無ければ Unknown）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupMember {
    pub project: String,
    pub seat_account: Option<String>,
    pub matches: Reading<bool>,
}

/// 移動の行（移る前の口座が分からなければ from は None）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveRow {
    pub at: EpochSecs,
    pub group: String,
    pub from: Option<String>,
    pub to: String,
}

/// run の 4 列の数（待ち・走行・止まり・着地）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunCounts {
    pub wait: u32,
    pub run: u32,
    pub stop: u32,
    pub land: u32,
}

/// project の行（state dir が引けなければ席と run と台帳は Unknown）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectRow {
    pub name: String,
    pub group: Option<String>,
    pub state_dir_known: bool,
    pub seat: Reading<SeatCard>,
    /// 退避までの残り秒（移動中でなければ None）。
    pub move_left_s: Option<u64>,
    pub runs: Reading<RunCounts>,
    pub ledger: Reading<LedgerStats>,
    pub next: Reading<NextStep>,
    /// project board の port（account board と同じ host で配る・host と住所は持たない・無ければ開けない）。
    pub board: Option<u16>,
}

/// session の行（project → 役 → 名 → 口座 → 状態 → 段 → いつから → 稼働の記録）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionLine {
    pub project: String,
    pub role: SeatRole,
    pub name: String,
    pub account: Option<String>,
    pub state: SeatState,
    pub stage: Option<Stage>,
    pub since: Option<EpochSecs>,
    pub spans: Reading<Vec<SeatSpan>>,
}

/// 停止の切り替えの向き（閉じた 2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Heartbeat {
    On,
    Off,
}

impl Heartbeat {
    pub const ALL: [Heartbeat; 2] = [Heartbeat::On, Heartbeat::Off];
}

/// 停止の切り替えの要求（project の名と向きだけ・state dir と席は server が引く）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub project: String,
    pub to: Heartbeat,
}

/// 停止の切り替えの応答（撃った席と向き）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub target: String,
    pub to: Heartbeat,
}
