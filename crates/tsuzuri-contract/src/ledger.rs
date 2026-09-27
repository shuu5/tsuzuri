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

/// 台帳の一覧の 1 行（口 ledger-list の出力: id・種類・題・状態・更新時刻）。
/// 種類と状態は bd の語をそのまま写す（bd の版で増える語を面が断らない）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerRow {
    pub id: BeadId,
    pub kind: String,
    pub title: String,
    pub status: String,
    pub updated_at: EpochSecs,
}

/// bead の中身（口 ledger-item の出力: 本文と notes。近傍の材料はグラフの口が返す）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerItem {
    pub row: LedgerRow,
    pub description: String,
    pub notes: String,
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

/// bd の issues.jsonl の 1 行のうち server が読む欄（電文ではない・知らない欄は読み捨てる）。
/// 空になりうる欄（種類・本文・notes）は bd が省くので既定を空にする。更新時刻は RFC 3339 の字のまま。
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
