//! 器の局面の出力の電文（口 GET /api/cases・設計ノート surface-wave26a 行 c-case-read・判断の記録 ADR-27 の決定 (10)）。
//! 器の fleet/lifecycle.json の部品の欄のうち面が読む欄だけを写す（欄の正本は器の case-lifecycle §5.2）。
//! 部品の種類と局面と手番と理由の語は閉じた列で持たず字のまま運び、面が知らない語を「まだ分からない」に倒す。

use serde::{Deserialize, Serialize};

use crate::EpochSecs;
use crate::board::Reading;

/// 局面の出力の読みの口の path。
pub const PATH: &str = "/api/cases";

/// 局面の出力（出力か古さの印の file が無いか読めなければ parts は Unknown）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseDoc {
    /// 出力の生成の時刻（parts が Unknown か時刻が読めなければ None）。
    pub generated_at: Option<EpochSecs>,
    /// 古さの印の種類の字（古さの印の file の順・印が無ければ空・空でなければ出力は古い）。
    pub stale: Vec<String>,
    pub parts: Reading<Vec<CasePart>>,
    /// 出力の file は在るのに読めない版の周か（真なら parts は Unknown で、面は「まだ分からない」でなく「読めない」と出す）。
    /// 出力が JSON でない・版が 1 でない・部品の列が無い時と、出力が在って古さの印の file が読めない時に真。
    /// 出力が無い周は偽（「まだ分からない」）。false は電文に字を置かず、鍵の無い前の電文は false に読む（行 c-case-unreadable）。
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unreadable: bool,
}

/// 部品 1 つ（id は種類ごとの形の字のまま・bead id とは限らない）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CasePart {
    pub part: String,
    pub id: String,
    pub phase: String,
    pub turn: String,
    pub since: Option<EpochSecs>,
    pub reason: Option<String>,
    pub closed: bool,
    pub links: CaseLinks,
}

/// 部品の結びのうち待つ相手の 2 つ（欠けた key は空の列）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseLinks {
    /// 契約の待ちの理由が dependency・overlap・reserved のときの相手の bead id。
    pub on: Vec<String>,
    /// 便の run id。
    pub runs: Vec<String>,
}
