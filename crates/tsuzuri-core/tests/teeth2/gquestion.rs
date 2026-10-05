//! 走行の答えの無い問いと g-12 の歯（便 c-g12）: questions-4.json・runs-5.json。
//! 器の問いは台帳の bead でなく event log の中に在る。n 番目の問いは走行の n 番目の QuestionRaised で、
//! 次の QuestionRaised の直前までに同じ走行の QuestionAnswered が在れば答えが在る。
#![cfg(test)]

use crate::common::{root, verdict_of};
use serde_json::Value;
use tsuzuri_contract::graph::EdgeType;
use tsuzuri_core::graph::{Graph, Inputs, RunAttr, Source, Verdict, build};

const Q_A: &str = "r.1-20260927T000000Z";
const Q_B: &str = "r.1-20260927T000100Z";
const Q_C: &str = "r.1-20260927T000200Z";
const Q_D: &str = "r.1-20260927T000300Z";

fn fixture(name: &str) -> Value {
    let rel = format!("tests/fixtures/graph/{name}");
    let text =
        std::fs::read_to_string(root().join(&rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{rel} は JSON: {e}"))
}

/// fixture の欄を入力の字にする（tests/teeth2/graph.rs と同じ形）。
fn input_text(v: &Value, key: &str) -> String {
    match &v[key] {
        Value::String(s) => s.clone(),
        Value::Array(items) if key != "ledger" => items
            .iter()
            .map(|item| match item {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Null => panic!("fixture に欄 {key} が無い"),
        other => other.to_string(),
    }
}

/// fixture の 3 つの欄から組む（tests/teeth2/graph.rs と同じ形）。
fn build_from(v: &Value) -> Graph {
    let design_index = input_text(v, "design_index");
    let ledger = input_text(v, "ledger");
    let events = input_text(v, "events");
    build(&Inputs {
        design_index: &design_index,
        ledger: &ledger,
        events: &events,
    })
}

fn attr(stage: &str, account: Option<&str>, unanswered: usize) -> RunAttr {
    RunAttr {
        stage: Some(stage.to_string()),
        account: account.map(str::to_string),
        unanswered,
    }
}

/// 走行ごとの段と答えの無い問いの数（fixture の形のまま・question の欄は無い）。
#[test]
fn gquestion_counts_unanswered_per_run() {
    let v = fixture("questions-4.json");
    let events = v["events"].as_array().expect("events");
    assert!(
        events
            .iter()
            .filter(|e| e["kind"] == "QuestionRaised" || e["kind"] == "QuestionAnswered")
            .all(|e| e.get("question").is_none() && e["detail"].is_string()),
        "問いの event は detail だけを持つ（実物の形）"
    );
    let g = build_from(&v);
    assert!(g.unread.is_empty());
    let runs: Vec<(&str, &RunAttr)> = g.runs.iter().map(|(id, a)| (id.as_str(), a)).collect();
    assert_eq!(
        runs,
        vec![
            (Q_A, &attr("Questioned", None, 1)),
            (Q_B, &attr("Spawned", Some("acct-1"), 0)),
            (Q_C, &attr("Questioned", None, 0)),
            (Q_D, &attr("Questioned", None, 1)),
        ]
    );
    assert_eq!(
        g.count_edges(EdgeType::Raised),
        0,
        "question の欄が無ければ raised の辺は組まない"
    );
}

/// g-12 は段が Questioned で答えの無い問いを持たない走行だけを名指す。
#[test]
fn gquestion_g12_names_only_q_c() {
    let g = build_from(&fixture("questions-4.json"));
    assert_eq!(
        verdict_of(&g, "g-12"),
        Verdict::Violation(vec![Q_C.to_string()])
    );
}

/// question の欄を足しても数え方は変わらず、欄の在る入力では raised の辺を今の通り組む。
#[test]
fn gquestion_question_field_does_not_change_the_count() {
    let mut v = fixture("questions-4.json");
    for (i, e) in v["events"]
        .as_array_mut()
        .expect("events")
        .iter_mut()
        .enumerate()
    {
        if e["kind"] == "QuestionRaised" {
            e["question"] = Value::String(format!("q.{i}"));
        }
    }
    let with = build_from(&v);
    let without = build_from(&fixture("questions-4.json"));
    assert_eq!(with.runs, without.runs);
    assert_eq!(with.count_edges(EdgeType::Raised), 5);
    assert_eq!(
        verdict_of(&with, "g-12"),
        Verdict::Violation(vec![Q_C.to_string()])
    );
}

/// 答えは直前の問いにだけ当たる（問いの前の答え・同じ問いへの 2 つ目の答えは数えない）。
#[test]
fn gquestion_answer_falls_on_the_last_question_only() {
    let run = "k.1-20260927T000000Z";
    let line =
        |kind: &str| format!("{{\"kind\":\"{kind}\",\"run\":\"{run}\",\"detail\":\"見本\"}}");
    let events = [
        line("RunCreated"),
        line("QuestionAnswered"),
        line("QuestionRaised"),
        line("QuestionRaised"),
        line("QuestionAnswered"),
        line("QuestionAnswered"),
        "{\"kind\":\"QuestionAnswered\",\"run\":\"other-20260927T000000Z\"}".to_string(),
    ]
    .join("\n");
    let g = build(&Inputs {
        design_index: "",
        ledger: "",
        events: &events,
    });
    assert_eq!(g.runs[run].unanswered, 1);
}

/// 着地済みの runs-5.json では、問いの無い Questioned の走行を今の通り名指す。
#[test]
fn gquestion_runs_5_still_names_the_questioned_run() {
    let g = build_from(&fixture("runs-5.json"));
    assert!(g.runs.values().all(|r| r.unanswered == 0));
    assert_eq!(
        verdict_of(&g, "g-12"),
        Verdict::Violation(vec!["r.1-20260927T000400Z".to_string()])
    );
}

/// 走行の event log が読めなければ g-12 はまだ分からない。
#[test]
fn gquestion_unread_runs_leave_g12_unknown() {
    let mut v = fixture("questions-4.json");
    v["events"] = Value::String(String::new());
    let g = build_from(&v);
    assert_eq!(g.unread, vec![Source::Runs]);
    assert_eq!(verdict_of(&g, "g-12"), Verdict::Unknown);
}
