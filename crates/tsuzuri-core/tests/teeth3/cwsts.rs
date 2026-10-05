//! 窓の状態の 1 行の字の歯（接頭辞 cwsts_・設計ノート surface-wave29b 行 cs-status-text・判断の記録 ADR-57 決定 (1)(コ)・(4)）。
//! 命令の字・Claude Code の session の JSON からの文脈の割合の読み・1 行の字の組み・制御の字の落としと長さの切りを見る。
#![cfg(test)]

use tsuzuri_contract::consult::{Form, Starter, WindowFile, WindowId};
use tsuzuri_core::consult::status::{
    DIGEST_HEAD, FIELD_MAX, SEP, VERB, clean, command, context_percent, render,
};

fn window() -> WindowFile {
    WindowFile {
        id: WindowId::parse("cw3").expect("窓 id"),
        form: Form::Talk,
        topic: Some("t3-hub.78".into()),
        model: "fable".into(),
        effort: "xhigh".into(),
        starter: Starter::Chat,
        uttered: Some("20261004T2207Z".into()),
        request: None,
        made: "20261004T2207Z".into(),
    }
}

#[test]
fn cwsts_command_names_the_verb() {
    assert_eq!(VERB, "statusline");
    assert_eq!(command("/T", "/W"), "timeout 2 /T consult statusline /W");
}

#[test]
fn cwsts_context_percent_reads_used_percentage_only() {
    let read = |t: &str| context_percent(t);
    assert_eq!(
        read(r#"{"context_window":{"used_percentage":42.4}}"#),
        Some(42)
    );
    assert_eq!(
        read(r#"{"context_window":{"used_percentage":42.5}}"#),
        Some(43)
    );
    assert_eq!(
        read(r#"{"model":{"id":"x"},"context_window":{"used_percentage":99.6,"x":1}}"#),
        Some(100)
    );
    assert_eq!(read(r#"{"context_window":{"used_percentage":0}}"#), Some(0));
    for bad in [
        "",
        "not json",
        "[]",
        r#"{"context_window":{}}"#,
        r#"{"context_window":{"used_percentage":null}}"#,
        r#"{"context_window":{"used_percentage":"42"}}"#,
        r#"{"context_window":{"used_percentage":-1}}"#,
        r#"{"context_window":{"used_percentage":100.1}}"#,
        r#"{"used_percentage":42}"#,
    ] {
        assert_eq!(read(bad), None, "{bad}");
    }
}

#[test]
fn cwsts_render_joins_the_present_fields() {
    let w = window();
    let line = render(Some(&w), Some(2), Some("3766ac7797cdf78f\n"), Some(42));
    let want = [
        "窓 cw3",
        "題 t3-hub.78",
        "fable・xhigh",
        "所見 2",
        "束 3766ac77",
        "文脈 42%",
    ];
    assert_eq!(line, want.join(SEP));
    assert_eq!((DIGEST_HEAD, SEP), (8, " ｜ "));
    let mut no_topic = window();
    no_topic.topic = None;
    assert_eq!(
        render(Some(&no_topic), None, None, None),
        ["窓 cw3", "題 なし", "fable・xhigh"].join(SEP)
    );
    assert_eq!(
        render(None, Some(0), Some(" \n"), None),
        ["窓 ?", "所見 0"].join(SEP)
    );
    assert!(render(Some(&w), None, Some("abc"), None).ends_with("束 abc"));
}

/// 制御の字（ESC・BEL・改行・DEL・C1 の CSI・向きの上書き）は題・model・念入りさ・束のどの欄からも落ち、
/// 題と model と念入りさは頭の `FIELD_MAX` 字に、束は頭の `DIGEST_HEAD` 字に切る。
#[test]
fn cwsts_render_drops_control_chars() {
    let evil = "\u{1b}]52;c;QUFB\u{7}a\nb\u{7f}c\u{9b}d\u{202e}e\u{2066}f";
    assert_eq!(clean(evil, 99), "]52;c;QUFBabcdef");
    assert_eq!(clean("あいうえお", 3), "あいう");
    let mut w = window();
    w.topic = Some(evil.into());
    w.model = format!("\u{1b}[2J{}", "m".repeat(50));
    w.effort = "x\u{1b}high".into();
    let line = render(Some(&w), None, Some("\u{1b}[31m3766ac77"), None);
    assert!(!line.chars().any(char::is_control), "{line:?}");
    let want = [
        "窓 cw3".to_string(),
        "題 ]52;c;QUFBabcdef".to_string(),
        format!("[2J{}・xhigh", "m".repeat(FIELD_MAX - 3)),
        "束 [31m3766".to_string(),
    ];
    assert_eq!(line, want.join(SEP));
    let mut long = window();
    long.topic = Some("題".repeat(FIELD_MAX + 5));
    assert!(
        render(Some(&long), None, None, None)
            .contains(&format!("題 {}{SEP}", "題".repeat(FIELD_MAX)))
    );
}
