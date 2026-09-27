//! 台帳（beads）の型: bead の id・一覧の行と中身・台帳への書き（閉じた enum から bdw の argv を組む）。
//! server の台帳の書きは起票の門の外なので、門が断る形（notes の置き換え・親の無い子の作成）を型で表せなくする。

use serde::{Deserialize, Serialize};

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
