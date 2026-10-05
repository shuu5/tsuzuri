//! 係の起こしの門と結びの口の判じ（行 ag-spec・判断の記録 ADR-59 決定 (1)(2)(4)・要件 FR21）。
//! 起こしの門は席の Agent の呼び（PreToolUse の入力・係の id `agent_id` の無い呼び）を読み、型が tsuzuri の 3 本か・名が在るか・
//! 頼みの頭の 4 行（予算・組み・対象・出す物）が在るか・生きた係（終えの印の無い札）と対象が重ならないかを判じて、係の札を組む。
//! 頭の予算は下限（`BUDGET_MIN`・判断の記録 ADR-63 決定 (3)）より小さければ、直した頭の見本を付けて断る。
//! 結びの口は Agent の呼びの結果（PostToolUse の入力の `tool_response.agentId`）から起こしの名と係の id を読む。
//! 置き場の名（起草の置き場の下の `<名>/brief.md`・`<名>/spec.json`・`.agents/<係の id>`）もここに置く。
//! 検証の群の起こしの門の判じ（`group`・判断の記録 ADR-61・要件 FR22）は行 ag-gjudge が置く。

pub mod group;
pub mod outs;
pub mod tie;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tsuzuri_contract::EpochSecs;

/// 席が起こせる係の型（決定 (1)・数は 3 を上限とする）。
pub const TYPES: [&str; 3] = ["tsuzuri:drafter", "tsuzuri:verifier", "tsuzuri:researcher"];

/// 頼みの頭の鍵（この順で見本を書く）。
pub const KEYS: [&str; 4] = ["予算", "組み", "対象", "出す物"];

/// 頭の値の書き方（直した頭の見本で、欠けたか形の違う値の代わりに書く）。
pub const HOLES: [&str; 4] = [
    "token <数>",
    "なし|軽|重",
    "<bead か記録の id>",
    "<file>,<file>",
];

/// 組みの値。
pub const BUILDS: [&str; 3] = ["なし", "軽", "重"];

/// 頭の予算の下限（新しい量・全部の起こし）。群の係ごとの上限（`group::MEMBER_NEW`）とは中身が違うので共有しない。
pub const BUDGET_MIN: u64 = 150_000;

/// 係の dir の頼みの file（prompt の字のまま）。
pub const BRIEF: &str = "brief.md";

/// 係の dir の札の file。
pub const SPEC: &str = "spec.json";

/// 起草の置き場の下の結びの dir（file の名は係の id・中身は起こしの名と改行）。
pub const AGENTS: &str = ".agents";

/// 頼みの頭の 4 つの値。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Head {
    pub budget: u64,
    pub build: String,
    pub target: String,
    pub outputs: Vec<String>,
}

/// 頭の読みの断り（欠けた鍵の名の列・予算の値の形・下限より小さい予算・組みの値の形）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadError {
    Missing(Vec<&'static str>),
    Budget(String),
    Low(u64),
    Build(String),
}

/// 起こしの門が見る席の Agent の呼び（tool_input の型・名・頼み）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub kind: Option<String>,
    pub name: Option<String>,
    pub prompt: String,
}

/// 起こしの門の断り。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    Type(Option<String>),
    Name(Option<String>),
    Head(HeadError),
    Clash { name: String, target: String },
    Outputs(Vec<String>),
}

/// 係の札（`<名>/spec.json` の字）。終えの印（`ended`）の無い札が生きた係。係の id（`agent_id`）は結びの口が書く。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spec {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub budget: u64,
    pub build: String,
    pub target: String,
    pub outputs: Vec<String>,
    pub spawned: EpochSecs,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub ended: Option<EpochSecs>,
}

impl Spec {
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

/// 頭の字の 4 つの値（prompt の頭の空でない行の塊の `鍵: 値`・鍵ごとに最初の行・空の値は無いと同じ）。
fn fields(prompt: &str) -> [Option<&str>; 4] {
    let lines: Vec<(&str, &str)> = prompt
        .trim_start()
        .lines()
        .take_while(|l| !l.trim().is_empty())
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim(), v.trim()))
        .collect();
    KEYS.map(|key| {
        lines
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
            .filter(|v| !v.is_empty())
    })
}

/// 予算の値（`token <数>`・数は 1 以上）。
fn budget(v: &str) -> Option<u64> {
    v.strip_prefix("token ")?
        .trim()
        .parse()
        .ok()
        .filter(|n| *n > 0)
}

/// 出す物の値（`,` で区切った空でない名の列）。
fn outputs(v: &str) -> Vec<String> {
    v.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// 頼みの頭を読む（欠けた鍵が在れば全部の名・予算と組みの値の形の誤り・下限より小さい予算）。
pub fn head(prompt: &str) -> Result<Head, HeadError> {
    let f = fields(prompt);
    let missing: Vec<&'static str> = KEYS
        .iter()
        .zip(f)
        .filter(|(k, v)| {
            v.is_none() || (**k == "出す物" && v.is_some_and(|v| outputs(v).is_empty()))
        })
        .map(|(k, _)| *k)
        .collect();
    let [Some(b), Some(build), Some(target), Some(out)] = f else {
        return Err(HeadError::Missing(missing));
    };
    if !missing.is_empty() {
        return Err(HeadError::Missing(missing));
    }
    let budget = budget(b).ok_or_else(|| HeadError::Budget(b.to_string()))?;
    if budget < BUDGET_MIN {
        return Err(HeadError::Low(budget));
    }
    if !BUILDS.contains(&build) {
        return Err(HeadError::Build(build.to_string()));
    }
    Ok(Head {
        budget,
        build: build.to_string(),
        target: target.to_string(),
        outputs: outputs(out),
    })
}

/// 直した頭の見本（形の合う値はそのまま・欠けたか形の違う値は書き方・下限より小さい予算は下限から上の書き方）。
pub fn fixed(prompt: &str) -> String {
    let f = fields(prompt);
    let ok = |i: usize, v: &str| match i {
        0 => budget(v).is_some_and(|n| n >= BUDGET_MIN),
        1 => BUILDS.contains(&v),
        3 => !outputs(v).is_empty(),
        _ => true,
    };
    let low = format!("token <{BUDGET_MIN} 以上の数>");
    KEYS.iter()
        .zip(HOLES)
        .zip(f)
        .enumerate()
        .map(|(i, ((k, hole), v))| {
            let hole = if i == 0 && v.and_then(budget).is_some() {
                low.as_str()
            } else {
                hole
            };
            format!("{k}: {}", v.filter(|v| ok(i, v)).unwrap_or(hole))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// 名か係の id の字（英数で始まり、英数と `-` と `_` だけで `max` 字以内）。起草の置き場の下の file の名に使うので、ほかの字を断る。
fn safe(s: &str, max: usize) -> bool {
    s.len() <= max
        && s.bytes().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}

/// 係の id の無い Agent の呼びの入力（JSON でない・Agent でない道具・係の id の在る呼びは None）。
fn seat_agent(payload: &str) -> Option<Map<String, Value>> {
    let Ok(Value::Object(input)) = serde_json::from_str::<Value>(payload) else {
        return None;
    };
    let agent = input.get("tool_name").and_then(Value::as_str) == Some("Agent");
    (agent && input.get("agent_id").is_none_or(Value::is_null)).then_some(input)
}

/// 起こしの門の入力が席の Agent の呼びなら、型と名と頼みを読む。
pub fn spawn_call(payload: &str) -> Option<Call> {
    let input = seat_agent(payload)?;
    let tool = input.get("tool_input").and_then(Value::as_object);
    let text = |k: &str| {
        tool.and_then(|t| t.get(k))
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    Some(Call {
        kind: text("subagent_type"),
        name: text("name"),
        prompt: text("prompt").unwrap_or_default(),
    })
}

/// 起こしの門の判じ（型・名・頭・生きた札との対象の重なりの順）。通す時は係の札（係の id と終えの印は無い）。
pub fn judge(call: &Call, live: &[Spec], now: EpochSecs) -> Result<Spec, Refusal> {
    let Some(kind) = call.kind.as_deref().filter(|k| TYPES.contains(k)) else {
        return Err(Refusal::Type(call.kind.clone()));
    };
    let Some(name) = call.name.as_deref().filter(|n| safe(n, 64)) else {
        return Err(Refusal::Name(call.name.clone()));
    };
    let head = head(&call.prompt).map_err(Refusal::Head)?;
    let astray = outs::astray(&head.outputs);
    if !astray.is_empty() {
        return Err(Refusal::Outputs(astray));
    }
    if let Some(s) = live
        .iter()
        .find(|s| s.ended.is_none() && s.target == head.target)
    {
        return Err(Refusal::Clash {
            name: s.name.clone(),
            target: s.target.clone(),
        });
    }
    Ok(Spec {
        name: name.to_string(),
        kind: kind.to_string(),
        budget: head.budget,
        build: head.build,
        target: head.target,
        outputs: head.outputs,
        spawned: now,
        agent_id: None,
        ended: None,
    })
}

/// 断りの理由の字（何が欠けたかと次の一手・頭の断りは直した頭の見本を付ける）。
pub fn reason(r: &Refusal, prompt: &str) -> String {
    let what = match r {
        Refusal::Type(k) => {
            let k = k.as_deref().unwrap_or("（無し）");
            return format!(
                "係の起こしの門は止める（型 {k} は席が起こせる係の型でない） 次の一手 = subagent_type を {} のどれかにする",
                TYPES.join("・")
            );
        }
        Refusal::Outputs(bad) => return outs::reason(bad, prompt),
        Refusal::Name(_) => {
            return "係の起こしの門は止める（名が無いか、英数と - と _ の 64 字以内でない） 次の一手 = Agent の name に係の名を書く（頼みの頭には書かない）".to_string();
        }
        Refusal::Clash { name, target } => {
            return format!(
                "係の起こしの門は止める（生きた係 {name} が同じ対象 {target} を持つ） 次の一手 = 係 {name} の終えを待つか、対象を替える"
            );
        }
        Refusal::Head(HeadError::Missing(keys)) => {
            format!("頼みの頭に {} の行が無い", keys.join("・"))
        }
        Refusal::Head(HeadError::Budget(v)) => format!("予算の値 {v} が token <数> の形でない"),
        Refusal::Head(HeadError::Low(n)) => format!("予算の値 {n} が下限 {BUDGET_MIN} より小さい"),
        Refusal::Head(HeadError::Build(v)) => {
            format!("組みの値 {v} が {} のどれでもない", BUILDS.join("・"))
        }
    };
    format!(
        "係の起こしの門は止める（{what}） 次の一手 = prompt の頭に次の 4 行を置き、空の行の後に頼みを書く\n{}",
        fixed(prompt)
    )
}

/// PreToolUse の deny の答えの JSON の字（理由は `reason`）。
pub fn deny(reason: &str) -> String {
    json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    })
    .to_string()
}

/// 結びの口の入力（席の Agent の呼びの結果）から起こしの名と係の id を読む（係の id が無いか、名か係の id の字が安全でなければ None）。
pub fn bound(payload: &str) -> Option<(String, String)> {
    let input = seat_agent(payload)?;
    let name = input
        .get("tool_input")?
        .get("name")?
        .as_str()
        .filter(|n| safe(n, 64))?;
    let id = input
        .get("tool_response")?
        .get("agentId")?
        .as_str()
        .filter(|i| safe(i, 128))?;
    Some((name.to_string(), id.to_string()))
}
