//! 問いの card の電文（画面の block「問いの card」が読む・設計ノート surface-base 便 b-cards）。
//! card を組むのは中核の crate で、ここは電文の形だけを決める。見た版の要約値は `LedgerItem::digest` が付ける。

use serde::{Deserialize, Serialize};

use crate::EpochSecs;
use crate::board::Reading;
use crate::ledger::BeadId;

/// 問いの card（1 本の問いの bead・本文の定型行の無い欄は None）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestionCard {
    pub id: BeadId,
    pub title: String,
    /// 作られた時刻。
    pub posted_at: EpochSecs,
    /// 非エンジニア向けの概要。
    pub plain: Option<String>,
    /// エンジニア向けの概要。
    pub eng: Option<String>,
    pub reason: Option<String>,
    pub recommend: Option<String>,
    /// A-1 の印を持つか。
    pub a1: bool,
    /// 名指した節点の id の列。
    pub touches: Vec<String>,
    /// この問いの答えを待って止まった task の id の列（台帳の順・鍵の無い電文は空の列に読む）。
    #[serde(default)]
    pub blocking: Vec<String>,
    /// 見た版の要約値（`LedgerItem::digest` の字）。
    pub digest: String,
}

/// 問いの一覧（口 questions の出力）。台帳が読めなければ card は `Unknown`（0 件と区別する）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestionList {
    pub cards: Reading<Vec<QuestionCard>>,
    /// server が答えを受けるか（読むだけの server は偽・鍵の無い電文は真に読む・行 e-ask-own-only）。
    #[serde(default = "answers")]
    pub answerable: bool,
}

/// 鍵 answerable の無い電文の値（答えを受ける・前の server と組の面が今のまま動く）。
fn answers() -> bool {
    true
}

/// ほかの project の問いの一覧（札は置き場の dir の名・台帳が読めなければ card は `Unknown`・行 e-multi-ask）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectQuestions {
    pub project: String,
    /// その project の問いに答えを受けるか（鍵の無い電文は偽に読む）。
    #[serde(default)]
    pub answerable: bool,
    pub cards: Reading<Vec<QuestionCard>>,
}

/// 自分の問いの一覧とほかの project の問いの一覧（口 questions の出力・行 e-multi-ask）。
/// others が空なら字は own の `QuestionList` の字と同じで、鍵 others の在る字を `QuestionList` で読むと
/// 鍵 others は捨てられ自分の問いの一覧になる。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllQuestions {
    #[serde(flatten)]
    pub own: QuestionList,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub others: Vec<ProjectQuestions>,
}
