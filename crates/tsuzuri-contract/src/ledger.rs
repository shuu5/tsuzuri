//! 台帳（beads）の型: bead の id・一覧の行と中身・一覧と変化の知らせ・bd の行の読み・
//! 台帳への書き（閉じた enum から bdw の argv を組む）。
//! server の台帳の書きは起票の門の外なので、門が断る形（notes の置き換え・親の無い子の作成）を型で表せなくする。

use serde::{Deserialize, Serialize};

use crate::board::Reading;
use crate::{EpochSecs, IdError, id_shape};

/// bead の id（例 `t3-hub.5`）。ASCII の英数字と `. - _` だけの 1 語・先頭は英数字・64 byte 以下。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct BeadId(String);

impl BeadId {
    pub fn new(s: impl Into<String>) -> Result<Self, IdError> {
        let s = s.into();
        id_shape(&s, b".-_")?;
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for BeadId {
    type Error = IdError;
    fn try_from(s: String) -> Result<Self, IdError> {
        Self::new(s)
    }
}

impl From<BeadId> for String {
    fn from(id: BeadId) -> String {
        id.0
    }
}

impl std::fmt::Display for BeadId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// 台帳の一覧の 1 行（口 ledger-list の出力: id・種類・題・状態・更新時刻・親・label）。
/// 種類と状態と label は bd の語をそのまま写す（bd の版で増える語を面が断らない）。
/// 問いと memo は label で見分ける（`QUESTION_LABEL`・`MEMO_LABEL`・設計ノート surface の節点の一覧）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerRow {
    pub id: BeadId,
    pub kind: String,
    pub title: String,
    pub status: String,
    pub updated_at: EpochSecs,
    /// 親の bead（根は None）。
    pub parent: Option<BeadId>,
    pub labels: Vec<String>,
}

/// 問い（席が置く問い）の label。
pub const QUESTION_LABEL: &str = "intake:question";

/// memo の label。
pub const MEMO_LABEL: &str = "intake:memo";

impl LedgerRow {
    /// 問いの bead か（label `intake:question` を持つ）。
    pub fn is_question(&self) -> bool {
        self.labels.iter().any(|l| l == QUESTION_LABEL)
    }

    /// memo の bead か（label `intake:memo` を持つ）。
    pub fn is_memo(&self) -> bool {
        self.labels.iter().any(|l| l == MEMO_LABEL)
    }
}

/// bead の中身（口 ledger-item の出力: 本文と notes。近傍の材料はグラフの口が返す）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerItem {
    pub row: LedgerRow,
    pub description: String,
    pub notes: String,
}

impl LedgerItem {
    /// 見た版の要約値（id・題・状態・本文・notes をこの順に改行 1 つでつないだ字の FNV-1a 64 bit・16 字の 16 進の小文字）。
    pub fn digest(&self) -> String {
        let text = [
            self.row.id.as_str(),
            &self.row.title,
            &self.row.status,
            &self.description,
            &self.notes,
        ]
        .join("\n");
        format!("{:016x}", fnv1a64(text.as_bytes()))
    }
}

/// FNV-1a の 64 bit（初期値 0xcbf29ce484222325・乗数 0x100000001b3）。
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &b| {
        (hash ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// 台帳の一覧（口 ledger-list の出力・便 e-min）。台帳が読めなければ行は `Unknown`（0 件と区別する）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerList {
    pub rows: Reading<Vec<LedgerRow>>,
}

/// 台帳の変化の知らせ（SSE の口 events の `ledger-changed` の data・便 e-min）。
/// 中身は運ばない。面は受けたら一覧の口を読み直す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerChanged {
    pub at: EpochSecs,
}

/// SSE の event の名（台帳の変化）。
pub const LEDGER_CHANGED_EVENT: &str = "ledger-changed";

/// bd の読み取りの口（`bd --readonly list --all --limit 0 --json`）が返す配列の 1 本のうち server が読む欄
/// （電文ではない・知らない欄は読み捨てる・便 e-src）。
/// 空になりうる欄（種類・本文・notes・親・label）は bd が省くので既定を空にする。更新時刻は RFC 3339 の字のまま。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct BdLine {
    pub id: BeadId,
    pub title: String,
    pub status: String,
    pub updated_at: String,
    #[serde(default)]
    pub issue_type: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub parent: Option<BeadId>,
    #[serde(default)]
    pub labels: Vec<String>,
}

impl BdLine {
    /// bd が消した bead（一覧に出さない）。
    pub fn is_tombstone(&self) -> bool {
        self.status == "tombstone"
    }

    /// 更新時刻を読んだ後の bead の中身。
    pub fn into_item(self, updated_at: EpochSecs) -> LedgerItem {
        LedgerItem {
            row: LedgerRow {
                id: self.id,
                kind: self.issue_type,
                title: self.title,
                status: self.status,
                updated_at,
                parent: self.parent,
                labels: self.labels,
            },
            description: self.description,
            notes: self.notes,
        }
    }
}

/// 子の bead の種類（bd create の --type に渡す語）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChildType {
    Task,
    Epic,
}

impl ChildType {
    pub fn as_str(self) -> &'static str {
        match self {
            ChildType::Task => "task",
            ChildType::Epic => "epic",
        }
    }
}

/// 台帳への書き（閉じた enum・server の書きはこの 3 つだけで、どれも bdw を経る）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum LedgerWrite {
    /// notes の末尾に 1 行を足す（置き換えない）。
    AppendNotes { id: BeadId, line: String },
    /// bead を閉じる。
    CloseItem { id: BeadId, reason: String },
    /// 親の下に子を作る（親は必須）。
    CreateChild {
        parent: BeadId,
        title: String,
        child_type: ChildType,
        description: String,
    },
}

/// 書きを撃つ program。
pub const BDW: &str = "bdw";

/// notes を置き換える旗（どの argv もこれを持たない）。
pub const NOTES_REPLACE_FLAG: &str = "--notes";

/// 親を名指す旗（子を作る argv は必ず持つ）。
pub const PARENT_FLAG: &str = "--parent";

impl LedgerWrite {
    /// bdw に渡す引数の列（program の名 `BDW` は含めない）。
    /// 人の字（行・理由・題・本文）は `--旗=値` の 1 語か `--` の後に置くので、旗に化けない。
    pub fn argv(&self) -> Vec<String> {
        match self {
            LedgerWrite::AppendNotes { id, line } => vec![
                "update".into(),
                id.to_string(),
                format!("--append-notes={line}"),
            ],
            LedgerWrite::CloseItem { id, reason } => {
                vec!["close".into(), id.to_string(), format!("--reason={reason}")]
            }
            LedgerWrite::CreateChild {
                parent,
                title,
                child_type,
                description,
            } => vec![
                "create".into(),
                format!("{PARENT_FLAG}={parent}"),
                format!("--type={}", child_type.as_str()),
                format!("--description={description}"),
                "--".into(),
                title.clone(),
            ],
        }
    }

    /// 子を作る書きか。
    pub fn creates_child(&self) -> bool {
        matches!(self, LedgerWrite::CreateChild { .. })
    }
}
