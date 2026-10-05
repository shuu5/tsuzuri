//! 相談の窓の型（判断の記録 ADR-29・要件 FR19）。
//! 窓と所見と頼みの id の形、起こし手・経路・採否・形・窓の状態の閉じた語、所見の file の形（JSON・未知の鍵を断る）、
//! board の口の電文を決める。所見の欄の検めと台帳の行の字は中核の crate の `consult` が持つ。
//! 保存する添え物の写し先は木の根からの相対の path だけを決め、どの木の根に写すかは呼ぶ側が渡す。

use serde::{Deserialize, Serialize};

use crate::EpochSecs;
use crate::board::Reading;

/// 相談の窓の一覧の口の path。
pub const PATH: &str = "/api/consult";

/// 席に受けられていない所見と頼みの口の path。
pub const UNRECEIVED_PATH: &str = "/api/consult/unreceived";

/// board の button の頼みの口の path。
pub const REQUEST_PATH: &str = "/api/consult/request";

/// 保存する添え物を写す dir（写し先の木の根からの相対）。
pub const KEPT_DIR: &str = "docs/consult/kept";

/// 窓が書く所見の草稿の欄の名（判断の記録 ADR-10 決定 (3) の 9 つと、話題・添え物・囲いの断り）。
pub const DRAFT_FIELDS: [&str; 12] = [
    "topic",
    "model",
    "bundle_digest",
    "conclusion",
    "options",
    "recommendation",
    "basis",
    "claims",
    "ask_owner",
    "touches",
    "attachments",
    "sandbox_denials",
];

/// id の字が形に合わない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShapeError;

impl std::fmt::Display for ShapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("consult-id-shape")
    }
}

impl std::error::Error for ShapeError {}

/// 1 以上の 10 進の数（頭の 0 を断る）。
fn count(s: &str) -> Result<u32, ShapeError> {
    let digits = !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) && !s.starts_with('0');
    match s.parse::<u32>() {
        Ok(n) if digits => Ok(n),
        _ => Err(ShapeError),
    }
}

/// UTC の分の字か（`20261003T1412Z` の形・14 字）。
pub fn minute_ok(s: &str) -> bool {
    const SHAPE: &[u8] = b"ddddddddTddddZ";
    s.len() == SHAPE.len()
        && SHAPE.iter().zip(s.bytes()).all(|(want, got)| match want {
            b'd' => got.is_ascii_digit(),
            _ => *want == got,
        })
}

/// 窓の id（`cw<n>`・n は 1 から）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct WindowId(u32);

impl WindowId {
    /// n が 0 なら None。
    pub fn new(n: u32) -> Option<Self> {
        (n > 0).then_some(Self(n))
    }

    pub fn parse(s: &str) -> Result<Self, ShapeError> {
        s.strip_prefix("cw")
            .ok_or(ShapeError)
            .and_then(count)
            .map(Self)
    }

    pub fn n(self) -> u32 {
        self.0
    }

    /// tmux の窓と作業場の dir と `--name` の名（`consult-cw<n>`）。
    pub fn name(self) -> String {
        format!("consult-{self}")
    }

    /// 退いた作業場の dir の名（`retired-consult-cw<n>`）。
    pub fn retired_name(self) -> String {
        format!("retired-consult-{self}")
    }
}

impl std::fmt::Display for WindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cw{}", self.0)
    }
}

impl TryFrom<String> for WindowId {
    type Error = ShapeError;
    fn try_from(s: String) -> Result<Self, ShapeError> {
        Self::parse(&s)
    }
}

impl From<WindowId> for String {
    fn from(id: WindowId) -> String {
        id.to_string()
    }
}

/// 所見の id（`cw<n>-<k>`・k は窓の中で 1 から）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FindingId {
    window: WindowId,
    k: u32,
}

impl FindingId {
    /// k が 0 なら None。
    pub fn new(window: WindowId, k: u32) -> Option<Self> {
        (k > 0).then_some(Self { window, k })
    }

    pub fn parse(s: &str) -> Result<Self, ShapeError> {
        let (w, k) = s.split_once('-').ok_or(ShapeError)?;
        Ok(Self {
            window: WindowId::parse(w)?,
            k: count(k)?,
        })
    }

    pub fn window(self) -> WindowId {
        self.window
    }

    pub fn k(self) -> u32 {
        self.k
    }
}

impl std::fmt::Display for FindingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.window, self.k)
    }
}

impl TryFrom<String> for FindingId {
    type Error = ShapeError;
    fn try_from(s: String) -> Result<Self, ShapeError> {
        Self::parse(&s)
    }
}

impl From<FindingId> for String {
    fn from(id: FindingId) -> String {
        id.to_string()
    }
}

/// 頼みの id（`rq-<UTC の分>-<n>`・server が出す）。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RequestId {
    minute: String,
    n: u32,
}

impl RequestId {
    /// 分の字が形に合わないか n が 0 なら None。
    pub fn new(minute: &str, n: u32) -> Option<Self> {
        (minute_ok(minute) && n > 0).then(|| Self {
            minute: minute.to_string(),
            n,
        })
    }

    pub fn parse(s: &str) -> Result<Self, ShapeError> {
        let rest = s.strip_prefix("rq-").ok_or(ShapeError)?;
        let (minute, n) = rest.split_once('-').ok_or(ShapeError)?;
        Self::new(minute, count(n)?).ok_or(ShapeError)
    }

    pub fn minute(&self) -> &str {
        &self.minute
    }

    pub fn n(&self) -> u32 {
        self.n
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rq-{}-{}", self.minute, self.n)
    }
}

impl TryFrom<String> for RequestId {
    type Error = ShapeError;
    fn try_from(s: String) -> Result<Self, ShapeError> {
        Self::parse(&s)
    }
}

impl From<RequestId> for String {
    fn from(id: RequestId) -> String {
        id.to_string()
    }
}

/// 台帳の行と一覧に出す閉じた語（語と値は 1 対 1）。
pub trait Word: Sized + Copy + PartialEq + 'static {
    /// 全部の値（この順が語の一覧の順）。
    const ALL: &'static [Self];

    fn word(self) -> &'static str;

    fn from_word(word: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|v| v.word() == word)
    }
}

/// 窓の形（話す窓・問う窓）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Form {
    Talk,
    Ask,
}

impl Word for Form {
    const ALL: &'static [Self] = &[Form::Talk, Form::Ask];
    fn word(self) -> &'static str {
        match self {
            Form::Talk => "話す",
            Form::Ask => "問う",
        }
    }
}

/// 起こし手（席・持ち主のチャット・持ち主の button）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Starter {
    Seat,
    Chat,
    Button,
}

impl Word for Starter {
    const ALL: &'static [Self] = &[Starter::Seat, Starter::Chat, Starter::Button];
    fn word(self) -> &'static str {
        match self {
            Starter::Seat => "席",
            Starter::Chat => "持ち主のチャット",
            Starter::Button => "持ち主の button",
        }
    }
}

/// 席が所見と頼みを受けた経路（見張り・完了・hook・一覧）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Via {
    Watch,
    Done,
    Hook,
    List,
}

impl Word for Via {
    const ALL: &'static [Self] = &[Via::Watch, Via::Done, Via::Hook, Via::List];
    fn word(self) -> &'static str {
        match self {
            Via::Watch => "見張り",
            Via::Done => "完了",
            Via::Hook => "hook",
            Via::List => "一覧",
        }
    }
}

/// 所見の処分の採否（採る・一部採る・採らない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verdict {
    Adopt,
    Partial,
    Reject,
}

impl Word for Verdict {
    const ALL: &'static [Self] = &[Verdict::Adopt, Verdict::Partial, Verdict::Reject];
    fn word(self) -> &'static str {
        match self {
            Verdict::Adopt => "採る",
            Verdict::Partial => "一部採る",
            Verdict::Reject => "採らない",
        }
    }
}

/// 窓の状態（生きている・止まった・閉じた・退いた）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WindowState {
    Live,
    Stalled,
    Closed,
    Retired,
}

impl Word for WindowState {
    const ALL: &'static [Self] = &[
        WindowState::Live,
        WindowState::Stalled,
        WindowState::Closed,
        WindowState::Retired,
    ];
    fn word(self) -> &'static str {
        match self {
            WindowState::Live => "生きている",
            WindowState::Stalled => "止まった",
            WindowState::Closed => "閉じた",
            WindowState::Retired => "退いた",
        }
    }
}

/// 所見の候補の採否（採るのは 1 つ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Adoption {
    Adopted,
    Rejected,
}

/// 主張の確かさ（撃って確かめた・導いた・推し量った・分からない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Confidence {
    Verified,
    Deduced,
    Inferred,
    Uncertain,
}

/// 添え物の種類（レポート・プログラム・結果）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttachmentKind {
    Report,
    Program,
    Result,
}

/// 所見の候補の 1 つ。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindingOption {
    pub id: String,
    pub name: String,
    pub text: String,
    pub verdict: Adoption,
    pub reason: String,
}

/// 主張の 1 つと確かさ。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub text: String,
    pub confidence: Confidence,
    pub how: String,
}

/// 添え物の 1 つ（path は作業場からの相対）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attachment {
    pub path: String,
    pub kind: AttachmentKind,
    pub note: String,
}

/// 保存する添え物の写し先（写し先の木の根からの相対・`docs/consult/kept/<所見 id>/<添え物の path>`）。
pub fn kept_path(finding: FindingId, attachment: &str) -> String {
    format!("{KEPT_DIR}/{finding}/{attachment}")
}

/// 窓が書く所見の草稿（欄は `DRAFT_FIELDS`・未知の鍵を断る）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindingDraft {
    /// 題の bead の id か自由な題か「なし」。
    pub topic: String,
    /// 出した時の model と念入りさ。
    pub model: String,
    pub bundle_digest: String,
    pub conclusion: String,
    pub options: Vec<FindingOption>,
    pub recommendation: String,
    pub basis: Vec<String>,
    pub claims: Vec<Claim>,
    pub ask_owner: Vec<String>,
    pub touches: Vec<String>,
    pub attachments: Vec<Attachment>,
    /// 囲いに断られて困った事（空の列でよい）。
    pub sandbox_denials: Vec<String>,
}

/// 作業場の findings/ に置く所見（草稿の欄と、所見の口が埋める id・窓・日付・未知の鍵を断る）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub id: FindingId,
    pub window: WindowId,
    /// 所見の口が置いた UTC の分の字。
    pub date: String,
    pub topic: String,
    pub model: String,
    pub bundle_digest: String,
    pub conclusion: String,
    pub options: Vec<FindingOption>,
    pub recommendation: String,
    pub basis: Vec<String>,
    pub claims: Vec<Claim>,
    pub ask_owner: Vec<String>,
    pub touches: Vec<String>,
    pub attachments: Vec<Attachment>,
    pub sandbox_denials: Vec<String>,
}

impl Finding {
    /// 草稿に id と日付を足す（窓は id の窓）。
    pub fn new(id: FindingId, date: &str, draft: FindingDraft) -> Self {
        Self {
            id,
            window: id.window(),
            date: date.to_string(),
            topic: draft.topic,
            model: draft.model,
            bundle_digest: draft.bundle_digest,
            conclusion: draft.conclusion,
            options: draft.options,
            recommendation: draft.recommendation,
            basis: draft.basis,
            claims: draft.claims,
            ask_owner: draft.ask_owner,
            touches: draft.touches,
            attachments: draft.attachments,
            sandbox_denials: draft.sandbox_denials,
        }
    }
}

/// board の button の頼み（口 POST /api/consult/request の入力・題は省ける）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsultRequest {
    pub topic: Option<String>,
    pub form: Form,
    pub model: String,
}

/// 席に受けられていない所見と頼み（口 GET /api/consult/unreceived の出力）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsultUnreceived {
    pub findings: Vec<FindingId>,
    pub requests: Vec<RequestId>,
}

/// 席が自分で開いた問う窓の今日（UTC の日）の数と今の数（規則の行 R-38）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeatQuota {
    pub today: u32,
    pub live: u32,
}

/// 窓の一覧の 1 行。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowRow {
    pub id: WindowId,
    pub form: Form,
    pub topic: Option<String>,
    pub state: WindowState,
    pub opened: Option<EpochSecs>,
    pub findings: u32,
    pub undisposed: u32,
    /// 最後の process の印の口座の置き場の末の名（印が無いか口座の欄が無ければ None・判断の記録 ADR-55 決定 (4)）。
    #[serde(default)]
    pub account: Option<String>,
}

/// 処分の無い所見の 1 行。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingRow {
    pub id: FindingId,
    pub topic: Option<String>,
    pub arrived: Option<EpochSecs>,
    pub received: Option<EpochSecs>,
}

/// 受けの無い頼みの 1 行。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestRow {
    pub id: RequestId,
    pub topic: Option<String>,
    pub at: EpochSecs,
}

/// 相談の窓の一覧（口 GET /api/consult の出力・読めない材料の段は Unknown）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsultBoard {
    pub windows: Reading<Vec<WindowRow>>,
    pub findings: Reading<Vec<FindingRow>>,
    pub requests: Reading<Vec<RequestRow>>,
    pub quota: Reading<SeatQuota>,
}

/// 作業場の窓の控え（`.consult/window.json`・open が書き、起動と一覧と board の口が読む・未知の鍵を断る）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowFile {
    pub id: WindowId,
    pub form: Form,
    /// 題の bead の id か自由な題（無ければ None）。
    pub topic: Option<String>,
    pub model: String,
    pub effort: String,
    pub starter: Starter,
    /// 持ち主のチャットの発話の UTC の分の字（起こし手が持ち主のチャットの時だけ）。
    pub uttered: Option<String>,
    /// 頼みの id（起こし手が持ち主の button の時だけ）。
    pub request: Option<RequestId>,
    /// 作業場を用意した UTC の分の字。
    pub made: String,
}

/// 起こした process の印（`.consult/proc-<k>.json`・起動の口が起こすごとに 1 つ書く・未知の鍵を断る）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcMark {
    /// 窓の中の起こしの番号（1 から）。
    pub k: u32,
    pub form: Form,
    /// 問う窓は子の pid、話す窓は tmux の pane の pid。
    pub pid: u32,
    /// 起こした UTC の分の字。
    pub at: String,
    /// 撃ち直しか。
    pub again: bool,
    /// 話す窓の tmux の window id（`@<n>`・問う窓は None）。
    pub tmux_window: Option<String>,
    /// 起こした時の口座の置き場（起こし手の環境の `CLAUDE_CONFIG_DIR` の字・無ければ None・判断の記録 ADR-55 決定 (2)）。
    #[serde(default)]
    pub account: Option<String>,
}

/// 会話の印の事（会話の始まり・持ち主の入力・turn の終わり・Claude Code の hook の名と 1 対 1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StampEvent {
    Start,
    Prompt,
    Stop,
}

impl StampEvent {
    /// hook の名（`hook_event_name` の字）から読む（ほかの名は None）。
    pub fn of_hook(name: &str) -> Option<Self> {
        match name {
            "SessionStart" => Some(StampEvent::Start),
            "UserPromptSubmit" => Some(StampEvent::Prompt),
            "Stop" => Some(StampEvent::Stop),
            _ => None,
        }
    }
}

/// 会話の印の 1 行（`.consult/stamps.jsonl`・会話の印の口が hook ごとに 1 行足す・未知の鍵を断る・判断の記録 ADR-55 決定 (2)）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stamp {
    /// 足した時刻（UTC の epoch 秒）。
    pub at: EpochSecs,
    pub event: StampEvent,
    /// 会話の id（uuid の形）。
    pub sid: String,
    /// 会話の始まりの種類（SessionStart の source・ほかの事は None）。
    pub source: Option<String>,
}
