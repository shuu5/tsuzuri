//! 相談の窓の守りの hook の判じ（判断の記録 ADR-29 決定 (7)）。
//! 入力は Claude Code の PreToolUse の hook の JSON。道具の名が起動の閉じた列（`TOOLS` の 8 つ）に無ければ断り、
//! Bash の命令が tz consult の席の口（`SEAT_VERBS`）を含むか、囲いの外で撃ち直す印（dangerouslyDisableSandbox）を
//! 持てば断る。所見の口 answer と束の口 bundle は通す。JSON の object でない入力と道具の名の無い入力も断る（断る側に倒す）。
//! 命令の語は問いの起票の門と同じ分け方（`gate::segments`）で切り、引用の中の字も空白で切って照らす。

use serde_json::Value;

use super::launch::TOOLS;
use crate::gate::segments;

/// 窓から撃てない tz consult の口（席と起動の口と守りの口）。
pub const SEAT_VERBS: [&str; 8] = [
    "open", "launch", "watch", "show", "dispose", "list", "close", "guard",
];

/// 命令の語の列（一続きごとの語と、語の中の空白で切った字）に、字 consult の次が席の口の並びが在れば、その口。
fn seat_verb(command: &str) -> Option<&'static str> {
    segments(command).iter().find_map(|seg| {
        let words: Vec<&str> = seg.iter().flat_map(|w| w.split_whitespace()).collect();
        words.windows(2).find_map(|pair| match pair {
            ["consult", verb] => SEAT_VERBS.iter().copied().find(|v| v == verb),
            _ => None,
        })
    })
}

/// hook の入力を判じる（通すなら Ok・断るなら理由の字）。
pub fn judge(payload: &str) -> Result<(), String> {
    let Ok(Value::Object(input)) = serde_json::from_str::<Value>(payload) else {
        return Err("hook の入力が JSON の object でない".to_string());
    };
    let tool = input
        .get("tool_name")
        .and_then(Value::as_str)
        .ok_or("道具の名が無い")?;
    if !TOOLS.split(',').any(|t| t == tool) {
        return Err(format!("道具 {tool} は窓の道具の閉じた列に無い"));
    }
    if tool != "Bash" {
        return Ok(());
    }
    let args = input.get("tool_input").and_then(Value::as_object);
    if args
        .and_then(|a| a.get("dangerouslyDisableSandbox"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        return Err("囲いの外で撃ち直す印は使えない".to_string());
    }
    let command = args
        .and_then(|a| a.get("command"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    match seat_verb(command) {
        Some(verb) => Err(format!(
            "窓から tz consult {verb} は撃てない（所見は answer・束は bundle）"
        )),
        None => Ok(()),
    }
}
