//! 台帳（beads）の型: bead の id・一覧の行と中身・一覧と変化の知らせ・bd の行の読み・
//! 台帳への書き（閉じた enum から bdw の argv を組む）。
//! server の台帳の書きは起票の門の外なので、門が断る形（notes の置き換え・親の無い子の作成）を型で表せなくする。

use serde::{Deserialize, Serialize};

use crate::board::Reading;
use crate::graph::NodeKind;
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

/// 方針の問いの範囲の札の頭（後に範囲 `all` か問いの id を付ける・行 e-policy-q）。
pub const POLICY_SCOPE_LABEL: &str = "policy-scope:";

/// bead の種類（epic・memo・問い・契約の順に決める・どの bead も 1 つに当たる）。
/// 中核の地図の節点と面の台帳の一覧の項が、同じこの 1 つの読みを引く（行 g-ledger-kind）。
pub fn bead_kind(issue_type: &str, labels: &[String]) -> NodeKind {
    let has = |label: &str| labels.iter().any(|l| l == label);
    if issue_type == "epic" {
        NodeKind::Epic
    } else if has(MEMO_LABEL) {
        NodeKind::Memo
    } else if has(QUESTION_LABEL) {
        NodeKind::Question
    } else {
        NodeKind::Task
    }
}

impl LedgerRow {
    /// 地図の節点と同じ種類（欄 kind と labels を `bead_kind` に渡す）。
    pub fn node_kind(&self) -> NodeKind {
        bead_kind(&self.kind, &self.labels)
    }

    /// 問いの bead か（label `intake:question` を持つ）。
    pub fn is_question(&self) -> bool {
        self.labels.iter().any(|l| l == QUESTION_LABEL)
    }

    /// memo の bead か（label `intake:memo` を持つ）。
    pub fn is_memo(&self) -> bool {
        self.labels.iter().any(|l| l == MEMO_LABEL)
    }

    /// 方針の問いか（label のどれかが範囲の札の頭 `policy-scope:` で始まる）。
    pub fn is_policy(&self) -> bool {
        self.labels.iter().any(|l| l.starts_with(POLICY_SCOPE_LABEL))
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

/// bead の事実の一覧の口の path（routes/beads.rs と同じ字・面の吹き出しが読む・行 g-pop）。
pub const FACTS_PATH: &str = "/api/beads";

/// bead の事実の一覧（口 GET /api/beads の出力・行 c-bead-facts）。台帳が読めなければ行は `Unknown`（0 件と区別する）。
/// 面は台帳の一覧の行と id で結ぶ（一覧の行の型 `LedgerRow` は替えない）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeadFacts {
    pub rows: Reading<Vec<BeadFact>>,
}

/// bead の 1 本の事実（id・起票の時刻・短い題・blocks の相手・判断の記録 ADR-27 決定 (9)）。
/// 本文の概要は持たない（吹き出しは開いた時に 1 本の引きの口から読む・判断の記録 ADR-30 決定 (3)・行 c-fact-trim）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeadFact {
    pub id: BeadId,
    /// 起票の時刻（bd の欄 created_at・読めなければ None）。
    pub created_at: Option<EpochSecs>,
    /// 短い題（metadata の鍵 short の字・無ければ題から機械で作った字・規則の行 R-19 の字数を越えれば頭の字と … で切る）。
    pub short: String,
    /// short が metadata の鍵 short の字か（偽なら題から機械で作った字）。
    pub short_set: bool,
    /// blocks の相手（依存の type が blocks の先の bead の全部・台帳の順）。
    pub blocks: Vec<BeadId>,
}

/// 台帳の変化の知らせ（SSE の口 events の `ledger-changed` の data・便 e-min）。
/// 中身は運ばない。面は受けたら一覧の口を読み直す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerChanged {
    pub at: EpochSecs,
}

/// SSE の event の名（台帳の変化）。
pub const LEDGER_CHANGED_EVENT: &str = "ledger-changed";

/// 台帳の読みが落ちて最後に読めた字を返した応答の頭の名（値は最後に読めた時からの秒の 10 進の字・行 e-hold）。
/// 値は規則の行 R-21 に足した値。server が `READ_HOLD_S` とこの名を使い、面の後の行 g-fresh が
/// `READ_WARN_S` と `READ_HOLD_S` とこの名を使う。
pub const READ_AGE_HEADER: &str = "X-Tz-Read-Age";

/// 読みが落ちても最後に読めた中身を出し続ける上限の秒（規則の行 R-21）。越えれば中身を測れていないにする。
pub const READ_HOLD_S: u64 = 60;

/// 最後に読めた時からこの秒を越えれば、面は一番上の帯に読み込み不良の注意の印を出す（規則の行 R-21）。
pub const READ_WARN_S: u64 = 15;

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

/// 問いの答えの効き先（器の問いの形の metadata の鍵 `effect` の閉じた 2 値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Effect {
    /// 答えを文書へ写す問い。
    Document,
    /// 答えが操作で済む問い。
    Operation,
}

impl Effect {
    pub fn as_str(self) -> &'static str {
        match self {
            Effect::Document => "document",
            Effect::Operation => "operation",
        }
    }
}

/// 台帳への書き（閉じた enum・server の書きはこの 4 つだけで、どれも bdw を経て、どれも消さない）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum LedgerWrite {
    /// notes の末尾に 1 行を足す（置き換えない）。
    AppendNotes { id: BeadId, line: String },
    /// bead を閉じる。
    CloseItem { id: BeadId, reason: String },
    /// 親の下に子を作る（親は必須）。
    /// label は字 `,` を含まない語で、親の label は継がない。metadata は効き先が在れば鍵 `effect` の 1 対だけ。
    /// 標準出力は作った子の id だけ。
    CreateChild {
        parent: BeadId,
        title: String,
        child_type: ChildType,
        description: String,
        labels: Vec<String>,
        effect: Option<Effect>,
    },
    /// 閉じた bead を open に戻す（取り消しの行 e-revoke）。
    ReopenItem { id: BeadId, reason: String },
}

/// GET の 1 本の引きの口の path の頭（後に bead の id を付ける・routes/item.rs と同じ字）。
pub const ITEM_PATH: &str = "/api/ledger/";

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
                labels,
                effect,
            } => {
                let mut argv = vec![
                    "create".into(),
                    format!("{PARENT_FLAG}={parent}"),
                    format!("--type={}", child_type.as_str()),
                ];
                if !labels.is_empty() {
                    argv.push(format!("--labels={}", labels.join(",")));
                }
                if let Some(effect) = effect {
                    argv.push(format!(r#"--metadata={{"effect":"{}"}}"#, effect.as_str()));
                }
                argv.extend([
                    "--no-inherit-labels=true".into(),
                    "--silent=true".into(),
                    format!("--description={description}"),
                    "--".into(),
                    title.clone(),
                ]);
                argv
            }
            LedgerWrite::ReopenItem { id, reason } => {
                vec!["reopen".into(), id.to_string(), format!("--reason={reason}")]
            }
        }
    }

    /// 子を作る書きか。
    pub fn creates_child(&self) -> bool {
        matches!(self, LedgerWrite::CreateChild { .. })
    }
}
