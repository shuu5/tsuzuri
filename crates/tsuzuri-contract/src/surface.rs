//! 面の電文の型: 面の event・面の状態・問いの合図・裁定と束と方針の要求と応答・記帳 id の形
//! （設計ノート surface §3・§20・判断の記録 ADR-7 決定 (4)）。
//! 決定の取り消しの要求と応答と、取り消せるかを notes から判じる関数（行 e-revoke・server の受付と面の button が同じ関数で判じる）。

use serde::{Deserialize, Serialize};

use crate::board::{NextMove, Stage};
use crate::ledger::{BeadId, LedgerItem};
use crate::{EpochSecs, IdError, id_shape};

/// 記帳 id（ASCII の英数字と `. - _ :` だけの 1 語・先頭は英数字・64 byte 以下）。
/// 1 問の裁定は `<問い bead id>:<UTC の年月日 T 時分 Z>-<n>`・束は `batch:<同じ時刻>-<n>`。
/// 方針は、方針 1 つごとに作る閉じた問いの id を使う問いの形（`<方針の問い bead id>:<同じ時刻>-1`・`for_question` で作る・行 e-policy-q）。
/// 今までの memo「方針」の notes の行の `policy:<同じ時刻>-<n>` は読むだけで、もう発行しない。
/// server だけが発行する。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RulingId(String);

impl RulingId {
    pub fn new(s: impl Into<String>) -> Result<Self, IdError> {
        let s = s.into();
        id_shape(&s, b".-_:")?;
        Ok(Self(s))
    }

    /// 1 問の裁定の id（`minute` は `20260926T1437Z` の形）。
    pub fn for_question(question: &BeadId, minute: &str, n: u32) -> Result<Self, IdError> {
        Self::new(format!("{question}:{minute}-{n}"))
    }

    /// 束の id。
    pub fn for_batch(minute: &str, n: u32) -> Result<Self, IdError> {
        Self::new(format!("batch:{minute}-{n}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RulingId {
    type Error = IdError;
    fn try_from(s: String) -> Result<Self, IdError> {
        Self::new(s)
    }
}

impl From<RulingId> for String {
    fn from(id: RulingId) -> String {
        id.0
    }
}

impl std::fmt::Display for RulingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// 席の役（orchestrator が上・pipeline が下）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SeatRole {
    Orchestrator,
    Pipeline,
}

/// 席の稼働（閉じた 5）。応答なしは tick が stale のときだけ・停止は heartbeat-off のとき。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SeatHealth {
    /// 待ち（入力を待つ）。
    Waiting,
    /// 考え中。
    Working,
    /// 応答なし。
    Unresponsive,
    /// 停止中。
    Stopped,
    /// まだ分からない（器の file が読めない）。
    Unknown,
}

/// 席の状態（席の target・役・稼働・口座・退避までの残り秒）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeatView {
    pub seat: String,
    pub role: SeatRole,
    pub health: SeatHealth,
    pub account: Option<String>,
    pub move_left_s: Option<u64>,
}

/// 面の状態（席の状態・次の一手・最終更新・席に届いていない裁定）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceState {
    pub seats: Vec<SeatView>,
    pub next_move: NextMove,
    pub updated_at: EpochSecs,
    /// 裁定から 2 分で席の受けが無い記帳 id（面に「席に届いていない」）。
    pub undelivered: Vec<RulingId>,
}

/// 面の event（SSE で流す閉じた 4）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum SurfaceEvent {
    SeatState {
        seat: SeatView,
        at: EpochSecs,
    },
    QuestionPosted {
        question: BeadId,
        at: EpochSecs,
    },
    RulingRead {
        ruling: RulingId,
        seat: String,
        at: EpochSecs,
    },
    RunStage {
        run: String,
        contract: BeadId,
        stage: Stage,
        reason: Option<String>,
        at: EpochSecs,
    },
}

/// SSE の event の名（器の event の記録か設計文書の変化・便 g-parts）。面は受けたら登録された口を全部読み直す。
pub const BOARD_CHANGED_EVENT: &str = "board-changed";

/// 問いの合図（席が問いを bdw で置いた後に server へ送る）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestionNudge {
    pub question: BeadId,
}

/// 裁定の要求（問いの id・見た版の要約値・逐語）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RulingRequest {
    pub question: BeadId,
    pub seen_digest: String,
    pub verbatim: String,
}

/// 裁定の応答（記帳 id・記帳時刻）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RulingResponse {
    pub ruling: RulingId,
    pub recorded_at: EpochSecs,
}

/// 取り消しの口の path（POST・契約の型の RevokeRequest）。
pub const REVOKE_PATH: &str = "/api/revoke";

/// notes の裁定の定型行の頭（server の受付・中核の配達と導出グラフが読む頭と同じ字）。
pub const RULING_LINE: &str = "裁定 id = ";

/// 裁定の行の問いの欄の頭。
pub const QUESTION_FIELD: &str = "問い = ";

/// 裁定の行の取り消す欄の頭（取り消しの行だけが持つ・値は取り消す前の裁定の id）。
pub const REVOKES: &str = "取り消す = ";

/// 裁定の行の逐語の欄の頭（この欄より後の字は欄として読まない）。
pub const VERBATIM: &str = "逐語 = ";

/// 閉じた bead の状態の字。
pub const CLOSED_STATUS: &str = "closed";

/// 裁定の行の欄の終わりの字。
const FIELD_END: char = '・';

/// 取り消しの要求（問いの id・取り消す裁定の id・理由の逐語）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevokeRequest {
    pub question: BeadId,
    pub ruling: RulingId,
    pub verbatim: String,
}

/// 取り消しの応答（取り消しの行の id・記帳時刻・開き直しだけを撃ち直したか）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevokeResponse {
    pub ruling: RulingId,
    pub recorded_at: EpochSecs,
    pub reopened_only: bool,
}

/// notes の裁定の行 1 つ（id と、逐語の欄より前の問いの欄と取り消す欄の字）。
struct Line<'a> {
    id: &'a str,
    question: Option<&'a str>,
    revokes: Option<&'a str>,
}

/// notes の裁定の行を notes の順に読む（行の末の復帰を除いてから頭を見る・id が空の行は数えない）。
fn ruling_lines(notes: &str) -> Vec<Line<'_>> {
    notes
        .lines()
        .filter_map(|l| l.trim_end_matches('\r').strip_prefix(RULING_LINE))
        .filter_map(|rest| {
            let mut fields = rest.split(FIELD_END);
            let id = fields.next().unwrap_or_default().trim();
            if id.is_empty() {
                return None;
            }
            let mut line = Line {
                id,
                question: None,
                revokes: None,
            };
            for field in fields.take_while(|f| !f.starts_with(VERBATIM)) {
                if let Some(q) = field.strip_prefix(QUESTION_FIELD) {
                    line.question = Some(q.trim());
                } else if let Some(r) = field.strip_prefix(REVOKES) {
                    line.revokes = Some(r.trim());
                }
            }
            Some(line)
        })
        .collect()
}

/// 効いている最後の裁定の id（取り消しの行でも取り消された行でもない裁定の行のうち notes の順で最後）。
pub fn latest_ruling(notes: &str) -> Option<&str> {
    let lines = ruling_lines(notes);
    let revoked: Vec<&str> = lines.iter().filter_map(|l| l.revokes).collect();
    lines
        .iter()
        .rev()
        .find(|l| l.revokes.is_none() && !revoked.contains(&l.id))
        .map(|l| l.id)
}

/// 開き直しだけが残った取り消しの行の id（notes の最後の裁定の行が、その問いの欄でその裁定を取り消す行のときだけ）。
pub fn pending_reopen<'a>(notes: &'a str, question: &BeadId, ruling: &RulingId) -> Option<&'a str> {
    let lines = ruling_lines(notes);
    let last = lines.last()?;
    (last.question == Some(question.as_str()) && last.revokes == Some(ruling.as_str()))
        .then_some(last.id)
}

/// 取り消せるか（閉じた問いで、その裁定が効いている最後か、その裁定の開き直しだけが残っている）。
pub fn revocable(item: &LedgerItem, ruling: &RulingId) -> bool {
    item.row.is_question()
        && item.row.status == CLOSED_STATUS
        && (latest_ruling(&item.notes) == Some(ruling.as_str())
            || pending_reopen(&item.notes, &item.row.id, ruling).is_some())
}

/// 束の 1 行（問いの id・見た版の要約値・個別の逐語。個別が無ければ束の逐語を使う）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchItem {
    pub question: BeadId,
    pub seen_digest: String,
    pub verbatim: Option<String>,
}

/// 束の要求（問いの列・束の逐語）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchRequest {
    pub items: Vec<BatchItem>,
    pub verbatim: String,
}

/// 断りの理由（閉じた一覧）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Refusal {
    /// 逐語が空。
    EmptyVerbatim,
    /// 台帳に無い id。
    UnknownQuestion,
    /// 見た版が古い。
    StaleVersion,
    /// A-1 の印を持つ問いに個別の逐語が無い。
    A1NeedsOwnVerbatim,
    /// A-1 の印を持つ問いを束に含む。
    A1InBatch,
    /// 空の束。
    EmptyBatch,
}

impl Refusal {
    /// HTTP の状態 code（古い版は 409・台帳に無い id は 404・ほかは 400）。
    pub fn http_status(self) -> u16 {
        match self {
            Refusal::StaleVersion => 409,
            Refusal::UnknownQuestion => 404,
            Refusal::EmptyVerbatim
            | Refusal::A1NeedsOwnVerbatim
            | Refusal::A1InBatch
            | Refusal::EmptyBatch => 400,
        }
    }
}

/// 行ごとの結果（書いた・飛ばした・断った・閉じていない・書いていない）。飛ばしたは notes に同じ id が既に在るとき。
/// 閉じていないと書いていないは、束の書きが途中で落ちた 502 の応答だけが持つ。
/// 閉じていないは notes への追記は済み閉じる書きが落ちた（問いは open のまま・notes にこの id の裁定の行が残る）。
/// 書いていないはこの束が何も書いていない（追記が落ちた行と、落ちた行より後の撃っていない行）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "kebab-case")]
pub enum ItemOutcome {
    Written { ruling: RulingId },
    Skipped { ruling: RulingId },
    Refused { reason: Refusal },
    Unclosed { ruling: RulingId },
    Unwritten,
}

/// 束の行の結果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchItemResult {
    pub question: BeadId,
    #[serde(flatten)]
    pub outcome: ItemOutcome,
}

/// 束の応答（束の id・行ごとの結果）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchResponse {
    pub batch: RulingId,
    pub items: Vec<BatchItemResult>,
}

/// 方針の要求（範囲・逐語）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyRequest {
    pub scope: String,
    pub verbatim: String,
}

/// 方針の応答（方針の id・記帳時刻）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyResponse {
    pub policy: RulingId,
    pub recorded_at: EpochSecs,
}

/// 断りの応答（理由）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefusalResponse {
    pub reason: Refusal,
}
