//! 席の hook の相談の拾いの字（設計ノート surface-wave27b 行 cs-hooks・判断の記録 ADR-29 決定 (8)）。
//! 見張り（tz consult watch）が居ない間、席の hook（停止・入力の時・道具の周）は、board の未受けの口の答えの
//! 受けの無い所見（経路 hook）と頼みの固定の 1 行（`lines::notice`）を、裁定の答えの字と数を替えずにその後ろへ足す。
//! 受けの無い物が無いか読めない時、停止と入力の時の hook は見張りを置かせる 1 行（`missing`）だけを足す。

use serde_json::{Map, Value};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::consult::{ConsultUnreceived, Via};

use super::lines::{Event, NOTICE_HEAD, notice};

/// 見張りが居ない時の 1 行（`tzw` は tz の解き方の命令の字）。
pub fn missing(tzw: &str) -> String {
    format!("{NOTICE_HEAD}見張りが居ない（「{tzw} consult watch」を背景で置く）")
}

/// 受けの無い所見（経路 hook）と頼みの固定の 1 行（所見の列・頼みの列の順のまま）。
pub fn items(waiting: &ConsultUnreceived, tzw: &str) -> Vec<String> {
    let found = waiting.findings.iter().map(|id| Event::Finding {
        id: *id,
        via: Via::Hook,
    });
    let asked = waiting.requests.iter().cloned().map(Event::Request);
    found.chain(asked).map(|e| notice(&e, tzw)).collect()
}

/// 停止と入力の時の hook が足す行（受けの無い物の行・無いか読めなければ `missing` の 1 行だけ）。
pub fn nudge(waiting: &Reading<ConsultUnreceived>, tzw: &str) -> Vec<String> {
    let found = match waiting {
        Reading::Known(w) => items(w, tzw),
        Reading::Unknown => Vec::new(),
    };
    if found.is_empty() {
        vec![missing(tzw)]
    } else {
        found
    }
}

/// 答えの JSON の object（無いか object でなければ空）。
fn object(answer: Option<&str>) -> Map<String, Value> {
    match answer.map(serde_json::from_str::<Value>) {
        Some(Ok(Value::Object(o))) => o,
        _ => Map::new(),
    }
}

/// 前の字（字でなければ無し）の後ろに `extra` の行を改行でつないだ字。
fn joined(first: Option<&Value>, extra: &[String]) -> String {
    let mut lines: Vec<&str> = first.and_then(Value::as_str).into_iter().collect();
    lines.extend(extra.iter().map(String::as_str));
    lines.join("\n")
}

/// 停止の hook の答え（裁定の答え `answer` の reason の後ろに改行と `extra` の行を足す・裁定の答えが無ければ
/// decision の字 block と `extra` の行だけの reason・`extra` が空なら `answer` のまま）。
pub fn block_with(answer: Option<String>, extra: &[String]) -> Option<String> {
    if extra.is_empty() {
        return answer;
    }
    let mut out = object(answer.as_deref());
    let reason = joined(out.get("reason"), extra);
    out.insert("decision".to_string(), Value::String("block".to_string()));
    out.insert("reason".to_string(), Value::String(reason));
    Some(Value::Object(out).to_string())
}

/// 入力の時と道具の周の hook の答え（裁定の答え `answer` の additionalContext の後ろに改行と `extra` の行を足す・
/// 裁定の答えが無ければ `extra` の行だけ・hookEventName は `event`・`extra` が空なら `answer` のまま）。
pub fn context_with(answer: Option<String>, extra: &[String], event: &str) -> Option<String> {
    if extra.is_empty() {
        return answer;
    }
    let mut inner = match object(answer.as_deref()).remove("hookSpecificOutput") {
        Some(Value::Object(o)) => o,
        _ => Map::new(),
    };
    let context = joined(inner.get("additionalContext"), extra);
    inner.insert(
        "hookEventName".to_string(),
        Value::String(event.to_string()),
    );
    inner.insert("additionalContext".to_string(), Value::String(context));
    let mut out = Map::new();
    out.insert("hookSpecificOutput".to_string(), Value::Object(inner));
    Some(Value::Object(out).to_string())
}
