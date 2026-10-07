//! 席が起こす検証の群（判断の記録 ADR-61 決定 (1)(2)(4)(5)・要件 FR22）。
//! 群の係の起こしは、頼みの頭に `群: <群 id> <i>/<k>` の行（5 行目）を持つ席の Agent の呼びで、起草の置き場の群の id の dir の
//! 計画の file（`plan.json`）を持ち、攻めの係が自分の出力の dir（`<attacker>/w/`）に書く分からない（U）の主張の一覧の file
//! （`unknowns.tsv`・攻めの係の名は計画の欄 attacker）を持つ。ここは値と file の字の読みを、
//! 子の `judge` は起こしの門の判じを置く。値は判断の記録 ADR-61 の決定 (2) の字で、規則の表の行にしない（値を替えるのは判断の記録・歯が値を数える）。
//! file と台帳を読んで字を渡すのは境界の口で、ここは字を読むだけ。

pub mod judge;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::surface::RULING_LINE;

use crate::ledger::DAY;

/// 群の id の dir の計画の file。
pub const PLAN: &str = "plan.json";

/// 攻めの係の出力の dir（`<attacker>/w/`）の分からない（U）の主張の一覧の file（1 行に id・主張・判じられない訳・資料の path をタブで区切る）。
pub const LIST: &str = "unknowns.tsv";

/// 係の dir の群の席の札（通した起こしの群の id と番と割り）。
pub const SEAT: &str = "group.json";

/// 群の id の dir の割りの読む path の重なりの記帳（断らない・決定 (2)(5)）。
pub const SHARED: &str = "shared.jsonl";

/// 頼みの頭の群の行の鍵。
pub const KEY: &str = "群";

/// 群に入れる係の型（検証役だけ・決定 (1)）。
pub const TYPE: &str = "tsuzuri:verifier";

/// 計画の file の下書きの種類の閉じた列（決定 (2)）。
pub const DRAFTS: [&str; 4] = ["問いの本文", "判断の記録", "消す行", "緩める問い"];

/// 引き金の一覧の行の下限。
pub const LIST_MIN: usize = 3;

/// 群を起こさない一覧の行の上限（越えると割って攻め直すか 1 体の道）。
pub const LIST_MAX: usize = 18;

/// k の上限（反証役 3 体まで）。
pub const K_MAX: u32 = 3;

/// 同時に生きた群の係の上限。
pub const LIVE_MAX: usize = 3;

/// 1 体の割りの主張の上限。
pub const CLAIMS_MAX: usize = 6;

/// 係ごとの上限の新しい量（input と cache_creation と output の和）。
pub const MEMBER_NEW: u64 = 150_000;

/// 係ごとの上限の文脈の読み直し（cache_read）。
pub const MEMBER_READ: u64 = 2_500_000;

/// 群の上限の新しい量（止める線はその 9 割）。
pub const GROUP_NEW: u64 = 450_000;

/// 群の上限の読み直し（止める線はその 9 割）。
pub const GROUP_READ: u64 = 7_500_000;

/// 同じ対象の群を断る窓（7 日の秒）。
pub const WINDOW: u64 = 7 * DAY;

/// 7 日の例外に数えない裁定の問い（前の 2 つの問いとこの記録の裁定の問い）。
pub const UNCOUNTED: [&str; 3] = ["t3-hub.87.1", "t3-hub.87.2", "t3-hub.87.6"];

/// 席の流れの道具の名。
pub const WORKFLOW: &str = "Workflow";

/// 計画の file の係ごとの割り（名・モデル・主張の id の列・読める path の列）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Member {
    pub name: String,
    pub model: String,
    pub claims: Vec<String>,
    pub paths: Vec<String>,
}

/// 計画の file（群の型・対象・下書きの種類・攻めの係の名・係ごとの割り・在れば裁定 id）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(rename = "type")]
    pub kind: String,
    pub target: String,
    pub draft: String,
    pub attacker: String,
    pub members: Vec<Member>,
    #[serde(default)]
    pub ruling: Option<String>,
}

impl Plan {
    /// 計画の file の字を読む（JSON でないか、欄が欠けるか余るか、係の割りが無ければ None）。
    pub fn parse(text: &str) -> Option<Self> {
        serde_json::from_str::<Plan>(text)
            .ok()
            .filter(|p| !p.members.is_empty())
    }
}

/// 群の席の札（`<名>/group.json` の字・群の id と番と割り）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seat {
    pub group: String,
    pub i: u32,
    pub k: u32,
    pub claims: Vec<String>,
    pub paths: Vec<String>,
}

impl Seat {
    /// 札の字を読む（JSON でないか欄が欠ければ None）。
    pub fn parse(text: &str) -> Option<Self> {
        serde_json::from_str(text).ok()
    }

    /// 札の字（整えた JSON と末の改行）。
    pub fn render(&self) -> String {
        let mut text = serde_json::to_string_pretty(self).unwrap_or_default();
        text.push('\n');
        text
    }
}

/// 群の行の値（群の id と番 i と数 k）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub group: String,
    pub i: u32,
    pub k: u32,
}

/// 群の id の字（英数で始まり、英数と `-` と `_` だけで 64 字以内）。起草の置き場の下の dir の名に使う。
fn id_ok(s: &str) -> bool {
    s.len() <= 64
        && s.bytes().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}

impl Line {
    /// `<群 id> <i>/<k>` を読む（i と k は 1 以上の数・群の id の字が安全でなければ None）。
    pub fn parse(value: &str) -> Option<Self> {
        let (group, n) = value.trim().split_once(' ')?;
        let (i, k) = n.trim().split_once('/')?;
        let (i, k): (u32, u32) = (i.parse().ok()?, k.parse().ok()?);
        (id_ok(group) && i > 0 && k > 0).then(|| Line {
            group: group.to_string(),
            i,
            k,
        })
    }
}

/// 頼みの頭（prompt の頭の空でない行の塊）の群の行の値（行が無ければ None・空の値は空の字）。
pub fn group_line(prompt: &str) -> Option<&str> {
    prompt
        .trim_start()
        .lines()
        .take_while(|l| !l.trim().is_empty())
        .filter_map(|l| l.split_once(':'))
        .find(|(k, _)| k.trim() == KEY)
        .map(|(_, v)| v.trim())
}

/// 一覧の file の主張の id の列（空の行は数えない・タブで区切った 4 つの空でない欄でない行か、2 度目の id が在れば None）。
pub fn list_ids(text: &str) -> Option<Vec<String>> {
    let mut ids: Vec<String> = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let fields: Vec<&str> = line.split('\t').map(str::trim).collect();
        let [id, _, _, _] = fields.as_slice() else {
            return None;
        };
        if fields.iter().any(|f| f.is_empty()) || ids.iter().any(|x| x == id) {
            return None;
        }
        ids.push((*id).to_string());
    }
    Some(ids)
}

/// 群の合計が止める線（どちらかの欄で群の上限の 9 割）に届いたか。
pub fn reached(new: u64, read: u64) -> bool {
    new.saturating_mul(10) >= GROUP_NEW * 9 || read.saturating_mul(10) >= GROUP_READ * 9
}

/// 裁定 id の問いの欄（最初の `:` の前・無ければ全部）。
pub fn question_of(ruling: &str) -> &str {
    ruling.split_once(':').map_or(ruling, |(q, _)| q)
}

/// notes の裁定の定型行（頭 `裁定 id = `）の id に `ruling` が在るか。
pub fn ruled(notes: &str, ruling: &str) -> bool {
    notes
        .lines()
        .filter_map(|l| l.trim_end_matches('\r').strip_prefix(RULING_LINE))
        .any(|rest| rest.split('・').next().map(str::trim) == Some(ruling))
}

/// `text` が群の id を語として（前後が英数字でない所で）名指すか。
pub fn named(text: &str, group: &str) -> bool {
    !group.is_empty()
        && text.match_indices(group).any(|(at, _)| {
            let before = text.get(..at).and_then(|s| s.chars().next_back());
            let after = text.get(at + group.len()..).and_then(|s| s.chars().next());
            !before.is_some_and(|c| c.is_ascii_alphanumeric())
                && !after.is_some_and(|c| c.is_ascii_alphanumeric())
        })
}

/// 割りの読む path のうち、計画のほかの係の割りにも在る path（重なりは記帳し、断らない）。
pub fn shared(plan: &Plan, name: &str) -> Vec<String> {
    let Some(me) = plan.members.iter().find(|m| m.name == name) else {
        return Vec::new();
    };
    me.paths
        .iter()
        .filter(|p| {
            plan.members
                .iter()
                .any(|m| m.name != name && m.paths.contains(p))
        })
        .cloned()
        .collect()
}

/// 席の流れの道具の呼びか（JSON の object で、係の id が無く、道具の名が Workflow）。tool_input の中身は見ない。
pub fn workflow(payload: &str) -> bool {
    let Ok(Value::Object(input)) = serde_json::from_str::<Value>(payload) else {
        return false;
    };
    input.get("tool_name").and_then(Value::as_str) == Some(WORKFLOW)
        && input.get("agent_id").is_none_or(Value::is_null)
}

/// 席の流れの道具の呼びの断りの理由の字（持ち主に問う事を名指す）。
pub fn workflow_reason() -> String {
    format!(
        "群の起こしの門は止める（席の流れの道具 {WORKFLOW} の呼びは計画の file と裁定 id を持っても全部断る・判断の記録 ADR-61 の決定 (4)） \
         次の一手 = 流れの道具で係を起こすには持ち主に問う（別の問いの裁定と判断の記録で決める）。今は Agent で 1 体ずつか検証の群で起こす"
    )
}

/// 群の起こしの呼びの tool_input のモデル（無いか字でなければ None・行 ag-gspawn）。
pub fn model(payload: &str) -> Option<String> {
    let input: Value = serde_json::from_str(payload).ok()?;
    input
        .pointer("/tool_input/model")?
        .as_str()
        .map(str::to_string)
}

/// 台帳の読み取りの口（bd の show の JSON・bead の配列か object）の最初の bead の notes と本文（JSON の bead でなければ読めない・行 ag-gspawn）。
pub fn ledger_of(out: &[u8]) -> judge::Ledger {
    let Ok(v) = serde_json::from_slice::<Value>(out) else {
        return judge::Ledger::Unread;
    };
    let bead = v.as_array().map_or(Some(&v), |a| a.first());
    let Some(bead) = bead.filter(|b| b.is_object()) else {
        return judge::Ledger::Unread;
    };
    let text = |k: &str| {
        bead.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    judge::Ledger::Read {
        notes: text("notes"),
        description: text("description"),
    }
}

/// 割りの読む path の重なりの記帳の 1 行（時刻・係の名・重なる path の JSON・改行なし・行 ag-gspawn）。
pub fn shared_line(name: &str, paths: &[String], at: EpochSecs) -> String {
    serde_json::json!({"at": at, "name": name, "paths": paths}).to_string()
}
