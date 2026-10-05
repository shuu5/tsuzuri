//! 係の起こしの門の判じの歯（接頭辞 agsp_・設計ノート surface-wave29b 行 ag-spec・判断の記録 ADR-59 決定 (2)）。
//! 中核の `agent::spec` の頼みの頭の読みと、断りの理由の直した頭の見本と、生きた札との対象の重なりの判じを直に撃つ。
#![cfg(test)]

use tsuzuri_core::agent::spec::{
    Call, Head, HeadError, Refusal, Spec, bound, head, judge, reason, spawn_call,
};

/// 正しい頭の 4 行。
const LINES: [&str; 4] = [
    "予算: token 250000",
    "組み: なし",
    "対象: t3-hub.87",
    "出す物: notes.md,apply.py",
];

/// 頭の 4 行（`skip` の番の行を除く）と空の行と頼みの本文の prompt。
fn prompt(skip: Option<usize>) -> String {
    let head: Vec<&str> = (0..LINES.len())
        .filter(|i| Some(*i) != skip)
        .map(|i| LINES[i])
        .collect();
    format!("{}\n\n行 ag-spec を起草する。", head.join("\n"))
}

/// 起草係の型と名 w208b の呼び。
fn call(prompt: String) -> Call {
    Call {
        kind: Some("tsuzuri:drafter".into()),
        name: Some("w208b".into()),
        prompt,
    }
}

/// 対象 `target` の札（終えの印は `ended`）。
fn spec(name: &str, target: &str, ended: Option<u64>) -> Spec {
    Spec {
        name: name.into(),
        kind: "tsuzuri:drafter".into(),
        budget: 100_000,
        build: "軽".into(),
        target: target.into(),
        outputs: vec!["notes.md".into()],
        spawned: 1,
        agent_id: None,
        ended,
    }
}

/// 正しい頭は 4 つの値を返し、1 行ずつ欠いた 4 つの見本（ほかの行は正しい）は欠けた鍵の名だけを返す。
#[test]
fn agsp_head_reads_four_values_and_names_the_missing_key() {
    let want = Head {
        budget: 250_000,
        build: "なし".into(),
        target: "t3-hub.87".into(),
        outputs: vec!["notes.md".into(), "apply.py".into()],
    };
    assert_eq!(head(&prompt(None)), Ok(want));
    for (i, key) in ["予算", "組み", "対象", "出す物"].into_iter().enumerate() {
        assert_eq!(
            head(&prompt(Some(i))),
            Err(HeadError::Missing(vec![key])),
            "{key}"
        );
    }
}

/// 予算の値が token <数> の形でない見本と、組みの値が なし・軽・重 の外の見本を断る（ほかの行は正しい）。
#[test]
fn agsp_head_refuses_a_budget_that_is_not_a_number() {
    for value in ["token 25万", "250000", "token 0"] {
        let p = prompt(None).replacen(LINES[0], &format!("予算: {value}"), 1);
        assert_eq!(head(&p), Err(HeadError::Budget(value.into())), "{value}");
    }
    let p = prompt(None).replacen(LINES[1], "組み: 中", 1);
    assert_eq!(head(&p), Err(HeadError::Build("中".into())));
}

/// 予算の行の無い呼びの断りの理由は欠けた鍵の名を持ち、末に直した頭の見本（在る値はそのまま・欠けた値は書き方）の 4 行を持つ。
#[test]
fn agsp_reason_ends_with_the_fixed_head() {
    let p = prompt(Some(0));
    let r = judge(&call(p.clone()), &[], 9).unwrap_err();
    assert_eq!(r, Refusal::Head(HeadError::Missing(vec!["予算"])));
    let text = reason(&r, &p);
    assert!(text.contains("（頼みの頭に 予算 の行が無い）"), "{text}");
    let fixed = "\n予算: token <数>\n組み: なし\n対象: t3-hub.87\n出す物: notes.md,apply.py";
    assert!(text.ends_with(fixed), "{text}");
}

/// 生きた札（終えの印の無い札）と対象が同じ呼びを断り、同じ札に終えの印が在れば通して札を組む（対象の違う生きた札は通す）。
#[test]
fn agsp_judge_refuses_a_live_spec_with_the_same_target() {
    let c = call(prompt(None));
    let live = [
        spec("w300a", "t3-hub.1", None),
        spec("w208a", "t3-hub.87", None),
    ];
    let clash = Refusal::Clash {
        name: "w208a".into(),
        target: "t3-hub.87".into(),
    };
    assert_eq!(judge(&c, &live, 9), Err(clash));
    let ended = [
        spec("w300a", "t3-hub.1", None),
        spec("w208a", "t3-hub.87", Some(5)),
    ];
    let got = judge(&c, &ended, 9).unwrap();
    assert_eq!(
        (got.name.as_str(), got.target.as_str(), got.spawned),
        ("w208b", "t3-hub.87", 9)
    );
    assert_eq!((got.agent_id, got.ended), (None, None));
}

/// 門の入力（`head` は頭の欄の足し・`tail` は末の欄の足し・係の試しの形で会話の id と path は伏せた）。
fn input(head: &str, tool: &str, tail: &str) -> String {
    format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"/T/s.jsonl",{head}"hook_event_name":"PreToolUse","tool_name":"{tool}","tool_input":{{"description":"起草","prompt":"予算: token 1","subagent_type":"tsuzuri:drafter","name":"w208a"}}{tail}}}"#
    )
}

/// 係の id の欄（試しの係の id の形）。
const SUB: &str = r#""agent_id":"aa21461fe66d86b69","agent_type":"tsuzuri:drafter","#;

/// 起こしの門は席の Agent の呼びだけを読む（係の id の在る呼びと Agent でない道具は None）。
#[test]
fn agsp_spawn_call_reads_only_seat_agent_calls() {
    let want = Call {
        kind: Some("tsuzuri:drafter".into()),
        name: Some("w208a".into()),
        prompt: "予算: token 1".into(),
    };
    assert_eq!(spawn_call(&input("", "Agent", "")), Some(want));
    assert_eq!(spawn_call(&input(SUB, "Agent", "")), None);
    assert_eq!(spawn_call(&input("", "Bash", "")), None);
}

/// 型が tsuzuri の 3 本でない呼びと、名の無い呼びと、名の字が英数と - と _ の外の呼びを断る（頭は正しい）。
#[test]
fn agsp_judge_refuses_a_type_outside_the_three_and_a_missing_name() {
    let mut c = call(prompt(None));
    c.kind = Some("tryp:probe".into());
    assert_eq!(
        judge(&c, &[], 9),
        Err(Refusal::Type(Some("tryp:probe".into())))
    );
    let mut c = call(prompt(None));
    c.name = None;
    assert_eq!(judge(&c, &[], 9), Err(Refusal::Name(None)));
    c.name = Some("../w208b".into());
    assert_eq!(
        judge(&c, &[], 9),
        Err(Refusal::Name(Some("../w208b".into())))
    );
    assert!(judge(&call(prompt(None)), &[], 9).is_ok());
}

/// 結びの口は席の Agent の結果の agentId と起こしの名を読み、agentId の無い結果と係の id の在る呼びは None。
#[test]
fn agsp_bound_reads_the_name_and_the_agent_id() {
    let done = r#","tool_response":{"status":"async_launched","agentId":"a9f0c1"}"#;
    let want = Some(("w208a".to_string(), "a9f0c1".to_string()));
    assert_eq!(bound(&input("", "Agent", done)), want);
    let none = r#","tool_response":{"status":"async_launched"}"#;
    assert_eq!(bound(&input("", "Agent", none)), None);
    assert_eq!(bound(&input(SUB, "Agent", done)), None);
}
