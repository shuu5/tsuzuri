//! project board の停止の切り替えの電文。
//! 本文は向きだけを持ち、撃つ席は server が自分の --repo から引く（本文の字で席を選ばない）。
//! 応答は account の `HeartbeatResponse` をそのまま使う。

use serde::{Deserialize, Serialize};

use crate::account::Heartbeat;

/// project board の停止の切り替えの口の path（面と server はこの定数を使う）。
pub const PATH: &str = "/api/seat/heartbeat";

/// project board の停止の切り替えの要求（向きだけ・ほかの鍵を持つ本文は読まない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeatHeartbeatRequest {
    pub to: Heartbeat,
}
