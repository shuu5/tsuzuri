//! 席の「見て」の知らせの電文（行 i-11・要件 FR16・判断の記録 ADR-15 の決定 (6)）。
//! 材料は tz stage notify が project ごとに書く最新の 1 つの記録だけで、server はそれを読んで返す（書かない）。
//! project board は自分の名の 1 つを、account board は全部を同じ電文から読む。

use serde::{Deserialize, Serialize};

use crate::EpochSecs;

/// 知らせの口の path（面と server はこの定数を使う）。
pub const PATH: &str = "/api/notices";

/// project の最新の知らせ（記録の時刻・project の名・題・board の URL）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notice {
    pub at: EpochSecs,
    pub project: String,
    pub title: String,
    pub url: String,
}

/// 知らせの口の電文（組んだ時刻・この board の project の名・project ごとの最新の 1 つ〔at の新しい順・同じ at は名の順〕・
/// 記録の file は在るが読めない project の名〔名の順〕）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notices {
    pub at: EpochSecs,
    pub project: String,
    pub latest: Vec<Notice>,
    pub unread: Vec<String>,
}
