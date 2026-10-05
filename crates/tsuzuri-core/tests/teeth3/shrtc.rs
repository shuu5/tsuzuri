//! 起票の短い題の門の判じの歯（接頭辞 shrtc_・設計ノート surface-wave27b 行 c-short-gate・規則の行 R-39・判断の記録 ADR-30 決定 (6)）。
//! hook の入力の字は歯の中で組む（command の字は JSON の字に写す）。
#![cfg(test)]

use serde_json::{Value, json};
use tsuzuri_core::gate::{ShortWhy, creates, short_gate, short_output, short_why};

/// Bash の PreToolUse の hook の入力の字。
fn payload(command: &str) -> String {
    json!({"hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": command}})
        .to_string()
}

/// 全角の字を n 字。
fn wide(n: usize) -> String {
    "短い題の字".chars().cycle().take(n).collect()
}

#[test]
fn shrtc_creates_all() {
    let command = "bd create --labels=intake:memo --metadata='{\"short\":\"m\"}' 題1; \
                   bdw create -l intake:question --metadata x 題2; \
                   bd update fx.1 --metadata='{}'; echo bd create 題3; \
                   FOO=1 /usr/bin/bdw create 題4 && bd list";
    assert_eq!(
        creates(&payload(command)),
        vec![
            Some("{\"short\":\"m\"}".to_string()),
            Some("x".to_string()),
            None
        ]
    );
    let other = json!({"tool_name": "Edit", "tool_input": {"command": "bd create x"}}).to_string();
    assert_eq!(creates(&other), Vec::<Option<String>>::new());
}

#[test]
fn shrtc_short_reasons() {
    let ok = format!(r#"{{"short":"{}","touches":["FR4"]}}"#, wide(20));
    assert_eq!(short_why(Some(&ok)), None, "20 字は通す");
    let cases = [
        (None, ShortWhy::NoMetadata),
        (
            Some(format!(r#"{{"short":"{}","touches":["FR4"]"#, wide(20))),
            ShortWhy::NotObject,
        ),
        (
            Some(format!(r#"["short","{}"]"#, wide(20))),
            ShortWhy::NotObject,
        ),
        (
            Some(r#"{"touches":["FR4"]}"#.to_string()),
            ShortWhy::Missing,
        ),
        (
            Some(r#"{"short":20,"touches":["FR4"]}"#.to_string()),
            ShortWhy::Missing,
        ),
        (
            Some(r#"{"short":"","touches":["FR4"]}"#.to_string()),
            ShortWhy::Empty,
        ),
        (
            Some(r#"{"short":"  ","touches":["FR4"]}"#.to_string()),
            ShortWhy::Empty,
        ),
        (
            Some(format!(r#"{{"short":"{}","touches":["FR4"]}}"#, wide(21))),
            ShortWhy::Long,
        ),
    ];
    for (metadata, why) in cases {
        assert_eq!(short_why(metadata.as_deref()), Some(why), "{metadata:?}");
    }
    let words: Vec<&str> = ShortWhy::ALL.iter().map(|w| w.word()).collect();
    assert_eq!(
        words,
        [
            "short-no-metadata",
            "short-not-object",
            "short-missing",
            "short-empty",
            "short-long"
        ]
    );
}

#[test]
fn shrtc_gate_first_refusal() {
    let good = "bd create --metadata='{\"short\":\"a\"}' 題1";
    assert_eq!(short_gate(&payload(good)), None);
    assert_eq!(
        short_gate(&payload("ls; bd list")),
        None,
        "create の無い呼び"
    );
    let two = format!("{good}; bdw create 題2; bd create --metadata='{{}}' 題3");
    assert_eq!(
        short_gate(&payload(&two)),
        Some(ShortWhy::NoMetadata),
        "command の順の最初"
    );
    let text = short_output(ShortWhy::Long);
    let v: Value = serde_json::from_str(&text).expect("JSON の字");
    let out = &v["hookSpecificOutput"];
    assert_eq!(out["hookEventName"], "PreToolUse");
    assert_eq!(out["permissionDecision"], "deny");
    let reason = out["permissionDecisionReason"].as_str().expect("理由");
    assert!(
        reason.starts_with("起票の短い題の門は止める（short-long） 次の一手 = "),
        "{reason}"
    );
}
