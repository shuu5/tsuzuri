//! 係の測り（行 ag-meter・判断の記録 ADR-59 決定 (3)(4)・要件 FR21）。
//! 係の呼びの門の入力（係の id `agent_id` の在る入力）を読み、係の記録の続き（前の offset から後の完全な行）の使用量を
//! 応答の id で重ねずに足す。新しく読み書きした量は input・cache_creation・output の和で、文脈の読み直し（cache_read）は別の欄。
//! 同じ応答の id の行は記録の中で続いて並び、後の行ほど output が伸びるので、続く同じ id の行は前の行の分を引いて足し直す。
//! 50・75・90% を越えた最初の 1 回だけ残りを注ぐ（越えた印は測りの札 `meter.json` に残す）。
//! 予算を越えた係の呼びの係の門の判じ（`guard`）は行 ag-guard が置く。

pub mod guard;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tsuzuri_contract::EpochSecs;

/// 係の dir の測りの札。
pub const METER: &str = "meter.json";

/// 残りを注ぐ印（予算の百分率）。
pub const MARKS: [u64; 3] = [50, 75, 90];

/// 測りの札（読んだ所・足した量・最後の応答の id とその分・注いだ印）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meter {
    pub offset: u64,
    pub used: u64,
    pub cache_read: u64,
    pub last_id: Option<String>,
    pub last_used: u64,
    pub last_cache_read: u64,
    pub marks: Vec<u64>,
}

/// 1 行の使用量（応答の id・新しい量・cache の読み）。assistant の行で usage の在る時だけ。
fn usage(line: &[u8]) -> Option<(Option<String>, u64, u64)> {
    let row: Value = serde_json::from_slice(line).ok()?;
    if row.get("type").and_then(Value::as_str) != Some("assistant") {
        return None;
    }
    let message = row.get("message")?;
    let u = message.get("usage")?;
    let n = |k: &str| u.get(k).and_then(Value::as_u64).unwrap_or(0);
    let id = message
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string);
    let new = n("input_tokens") + n("cache_creation_input_tokens") + n("output_tokens");
    Some((id, new, n("cache_read_input_tokens")))
}

impl Meter {
    /// 札の字を読む（JSON でなければ None）。
    pub fn parse(text: &str) -> Option<Self> {
        serde_json::from_str(text).ok()
    }

    /// 札の字（整えた JSON と末の改行）。
    pub fn render(&self) -> String {
        let mut text = serde_json::to_string_pretty(self).unwrap_or_default();
        text.push('\n');
        text
    }

    /// 記録の offset から後の字 `tail` を足し、最後の改行までを読んだ分だけ offset を進める（末の書きかけの行は次に読む）。
    pub fn feed(&mut self, tail: &[u8]) {
        let Some(end) = tail.iter().rposition(|b| *b == b'\n') else {
            return;
        };
        for line in tail.get(..end).unwrap_or_default().split(|b| *b == b'\n') {
            let Some((id, new, cache)) = usage(line) else {
                continue;
            };
            if id.is_some() && id == self.last_id {
                self.used = self.used.saturating_sub(self.last_used);
                self.cache_read = self.cache_read.saturating_sub(self.last_cache_read);
            }
            self.used += new;
            self.cache_read += cache;
            (self.last_id, self.last_used, self.last_cache_read) = (id, new, cache);
        }
        self.offset += end as u64 + 1;
    }

    /// 予算 `budget` の印のうち新しく越えたものを札に足し、その最も大きい印を返す（越えた印が無ければ None）。
    pub fn cross(&mut self, budget: u64) -> Option<u64> {
        let fresh: Vec<u64> = MARKS
            .into_iter()
            .filter(|m| self.used * 100 >= m * budget && !self.marks.contains(m))
            .collect();
        self.marks.extend(&fresh);
        fresh.last().copied()
    }
}

/// 係の呼びの門の入力の欄（係の id・門の事・道具の名・親の記録の path・無い欄は空の字）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubCall {
    pub agent_id: String,
    pub event: String,
    pub tool: String,
    pub parent: String,
}

/// 係の呼びの入力（JSON の object で、係の id が英数で始まり英数と `-` と `_` だけの 128 字以内の時）の欄。席の呼びは None。
pub fn sub_call(payload: &str) -> Option<SubCall> {
    let input: Value = serde_json::from_str(payload).ok()?;
    let text = |k: &str| {
        input
            .get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    let agent_id = text("agent_id");
    let safe = agent_id.len() <= 128
        && agent_id
            .bytes()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric())
        && agent_id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_');
    safe.then(|| SubCall {
        event: text("hook_event_name"),
        tool: text("tool_name"),
        parent: text("transcript_path"),
        agent_id,
    })
}

/// 結びの無い呼びの記帳の 1 行（時刻・係の id・門の事・道具の名の JSON・改行なし）。
pub fn unbound_line(call: &SubCall, at: EpochSecs) -> String {
    json!({"at": at, "agent_id": call.agent_id, "event": call.event, "tool": call.tool}).to_string()
}

/// 係の記録の path（親の記録 `transcript_path` の .jsonl を除いた dir の下の subagents/agent-<係の id>.jsonl）。
pub fn transcript(parent: &str, agent_id: &str) -> Option<String> {
    let stem = parent.strip_suffix(".jsonl")?;
    Some(format!("{stem}/subagents/agent-{agent_id}.jsonl"))
}

/// 注ぎの字（越えた印・使った量・予算・残り・別の欄の cache の読み・次の一手）。
pub fn notice(mark: u64, meter: &Meter, budget: u64) -> String {
    format!(
        "係の測り: 予算 {budget} の {mark}% を越えた（使った量 {} = input と cache_creation と output の和・残り {}・cache の読み {} は別の欄で数えない）。\
         次の一手 = 新しい調べを広げず、分かった所を w/ に書く。100% を越えると、係の門は自分の w/ への書きと SendMessage だけを通す",
        meter.used,
        budget.saturating_sub(meter.used),
        meter.cache_read
    )
}

/// PostToolUse の答えの JSON の字（係の文脈に `text` を足す）。
pub fn inject(text: &str) -> String {
    json!({
        "hookSpecificOutput": {
            "hookEventName": "PostToolUse",
            "additionalContext": text,
        }
    })
    .to_string()
}
