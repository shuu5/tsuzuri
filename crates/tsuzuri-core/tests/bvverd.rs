//! run の gate と審査の内訳の歯（接頭辞 bvverd_・設計ノート surface-wave26a 行 c-run-verdict の完了の条件）。
//! 判定の file の字は器の便の dir の verdict.json と review.json の実物の形を写した小さな字（歯の中に書く）。
#![cfg(test)]

use std::collections::BTreeSet;

use serde_json::Value;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::runs::{GateFinding, GateVerdict, ReviewVerdict, RunLine, RunsDoc};
use tsuzuri_contract::wire;
use tsuzuri_core::pipeline::{gate_of, review_of, runs_of, with_verdicts};

const RUN_A: &str = "fx-v.1-20261001T000000Z";
const RUN_B: &str = "fx-v.1-20261001T010000Z";

/// bead fx-v.1 の 2 つの走行（1 つ目は審査で止め、2 つ目は gate を経て着地）とほかの種類の event。
const LOG: &str = concat!(
    r#"{"ts":"2026-10-01T00:00:00Z","kind":"RunCreated","run":"fx-v.1-20261001T000000Z","bead":"fx-v.1","stage":"Intake"}"#,
    "\n",
    r#"{"ts":"2026-10-01T00:01:00Z","kind":"RunStage","run":"fx-v.1-20261001T000000Z","bead":"fx-v.1","stage":"Reviewed","detail":"verdict:FAIL kind:literal-mismatch"}"#,
    "\n",
    r#"{"ts":"2026-10-01T00:01:30Z","kind":"QuestionRaised","run":"fx-v.1-20261001T000000Z","bead":"fx-v.1"}"#,
    "\n",
    r#"{"ts":"2026-10-01T00:02:00Z","kind":"RunStopped","run":"fx-v.1-20261001T000000Z","bead":"fx-v.1","stage":"Stopped"}"#,
    "\n",
    r#"{"ts":"2026-10-01T01:00:00Z","kind":"RunCreated","run":"fx-v.1-20261001T010000Z","bead":"fx-v.1","stage":"Intake"}"#,
    "\n",
    r#"{"ts":"2026-10-01T01:05:00Z","kind":"SeatSpawned","run":"fx-v.1-20261001T010000Z","bead":"fx-v.1","pid":7}"#,
    "\n",
    r#"{"ts":"2026-10-01T01:20:00Z","kind":"RunStage","run":"fx-v.1-20261001T010000Z","bead":"fx-v.1","stage":"Gated","detail":"verdict:FAIL"}"#,
    "\n",
    r#"{"ts":"2026-10-01T01:21:00Z","kind":"RunCost","run":"fx-v.1-20261001T010000Z","bead":"fx-v.1","usage":"in:5","turns":1,"wall_ms":10}"#,
    "\n",
    r#"{"ts":"2026-10-01T01:30:00Z","kind":"RunDone","run":"fx-v.1-20261001T010000Z","bead":"fx-v.1","stage":"Landed","detail":"terminal:close:ok"}"#,
    "\n",
);

/// gate の判定の file（審査役に届いた周・findings と母集団を持つ）。
const GATE_FAIL: &str = r#"{"schema":1,"run":"fx-v.1-20261001T010000Z","verdict":"FAIL","evidence":"done (2) の歯が字を測らない","verify_red":0,"diff_bytes":4096,"tree":"0000000000000000000000000000000000000000","findings":"contract-fit:1,teeth-nonvacuous:0,constitution:0,delete:0,stdlib:0,native:0,yagni:0,shrink:2","population":"files:3,lines:120","ts":"2026-10-01T01:19:30Z"}"#;

/// gate の判定の file（審査役に届かなかった周・findings の欄が無い）。
const GATE_CAP: &str = r#"{"schema":1,"run":"fx-v.1-20261001T010000Z","verdict":"INCONCLUSIVE","evidence":"diff 160000 byte が cap 150000 を超えた","verify_red":0,"diff_bytes":160000,"tree":"0000000000000000000000000000000000000000","ts":"2026-10-01T01:19:00Z"}"#;

/// 審査の判定の file（FAIL の周・kind と at を持つ）。
const REVIEW_FAIL: &str = r#"{"schema":1,"run":"fx-v.1-20261001T000000Z","verdict":"FAIL","evidence":"節の字と done の字が違う","kind":"literal-mismatch","at":"§3,行 c","ts":"2026-10-01T00:00:50Z"}"#;

/// 審査の判定の file（PASS の周・kind と at が無い）。
const REVIEW_PASS: &str = r#"{"schema":1,"run":"fx-v.1-20261001T010000Z","verdict":"PASS","evidence":"goal と done が節の字のまま","tree":"0000000000000000000000000000000000000000","ts":"2026-10-01T01:01:00Z"}"#;

fn bead() -> BeadId {
    BeadId::new("fx-v.1").expect("bead の id")
}

/// Known の走行の列（Unknown なら落とす）。
fn known(doc: &RunsDoc) -> &[RunLine] {
    match &doc.runs {
        Reading::Known(lines) => lines,
        Reading::Unknown => panic!("runs が Unknown"),
    }
}

/// 走行ごとの段の名の列。
fn stages(doc: &RunsDoc) -> Vec<Vec<&str>> {
    known(doc)
        .iter()
        .map(|l| l.steps.iter().map(|s| s.stage.as_str()).collect())
        .collect()
}

/// object の鍵の集合。
fn keys(v: &Value) -> BTreeSet<&str> {
    v.as_object()
        .unwrap_or_else(|| panic!("object でない: {v}"))
        .keys()
        .map(String::as_str)
        .collect()
}

fn set<'a>(words: &[&'a str]) -> BTreeSet<&'a str> {
    words.iter().copied().collect()
}

/// 走行の id から判定の file の字を返す手（`files` の組に無い id は None）。
fn reader<'a>(files: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |run| {
        files
            .iter()
            .find(|(id, _)| *id == run)
            .map(|(_, body)| (*body).to_string())
    }
}

/// 器の 4 つの走行の event（RunCreated・RunStage・RunDone・RunStopped）は段を 1 つずつ写し、ほかの種類は写さない。
#[test]
fn bvverd_stopped_and_done_are_steps() {
    let doc = runs_of(LOG, &bead());
    assert_eq!(
        stages(&doc),
        [
            vec!["Intake", "Reviewed", "Stopped"],
            vec!["Intake", "Gated", "Landed"]
        ]
    );
    let lines = known(&doc);
    assert_eq!(lines[0].steps[2].at, Some(1790812920));
    assert_eq!(lines[0].steps[2].detail, None);
    assert_eq!(
        lines[1].steps[2].detail.as_deref(),
        Some("terminal:close:ok")
    );
    for line in lines {
        assert_eq!(line.gate, Reading::Unknown, "{}", line.run);
        assert_eq!(line.review, Reading::Unknown, "{}", line.run);
    }
}

/// verdict.json の字の verdict・evidence・findings（器の字の順・0 も持つ）・ts の時刻を写す。
#[test]
fn bvverd_verdict_json_fields() {
    let count = |category: &str, count: u64| GateFinding {
        category: category.to_string(),
        count,
    };
    assert_eq!(
        gate_of(GATE_FAIL),
        Reading::Known(GateVerdict {
            verdict: "FAIL".to_string(),
            evidence: "done (2) の歯が字を測らない".to_string(),
            findings: Some(vec![
                count("contract-fit", 1),
                count("teeth-nonvacuous", 0),
                count("constitution", 0),
                count("delete", 0),
                count("stdlib", 0),
                count("native", 0),
                count("yagni", 0),
                count("shrink", 2),
            ]),
            at: Some(1790817570),
        })
    );
}

/// findings の欄が無い verdict.json は Known で findings が None。
#[test]
fn bvverd_no_findings_is_none() {
    let Reading::Known(gate) = gate_of(GATE_CAP) else {
        panic!("Known でない");
    };
    assert_eq!(gate.verdict, "INCONCLUSIVE");
    assert_eq!(gate.evidence, "diff 160000 byte が cap 150000 を超えた");
    assert_eq!(gate.findings, None);
    assert_eq!(gate.at, Some(1790817540));
}

/// review.json の字の verdict・evidence・kind・at（場所）・ts の時刻を写す。PASS の周は kind と場所が None。
#[test]
fn bvverd_review_json_fields() {
    assert_eq!(
        review_of(REVIEW_FAIL),
        Reading::Known(ReviewVerdict {
            verdict: "FAIL".to_string(),
            evidence: "節の字と done の字が違う".to_string(),
            kind: Some("literal-mismatch".to_string()),
            place: Some("§3,行 c".to_string()),
            at: Some(1790812850),
        })
    );
    assert_eq!(
        review_of(REVIEW_PASS),
        Reading::Known(ReviewVerdict {
            verdict: "PASS".to_string(),
            evidence: "goal と done が節の字のまま".to_string(),
            kind: None,
            place: None,
            at: Some(1790816460),
        })
    );
}

/// JSON でない字・schema が 1 でない字（無い・2・字の 1）・verdict か evidence が字でない字は、gate も審査も Unknown。
/// findings の札が崩れた verdict.json も Unknown。
#[test]
fn bvverd_unread_is_unknown() {
    let bad = [
        "",
        "{broken",
        "[]",
        r#"{"run":"x","verdict":"PASS","evidence":"ok"}"#,
        r#"{"schema":2,"verdict":"PASS","evidence":"ok"}"#,
        r#"{"schema":"1","verdict":"PASS","evidence":"ok"}"#,
        r#"{"schema":1,"evidence":"ok"}"#,
        r#"{"schema":1,"verdict":3,"evidence":"ok"}"#,
        r#"{"schema":1,"verdict":"PASS"}"#,
        r#"{"schema":1,"verdict":"PASS","evidence":null}"#,
    ];
    for body in bad {
        assert_eq!(gate_of(body), Reading::Unknown, "{body}");
        assert_eq!(review_of(body), Reading::Unknown, "{body}");
    }
    for findings in [
        r#""contract-fit""#,
        r#""contract-fit:x""#,
        r#""contract-fit:1,""#,
        r#"":1""#,
        r#""contract-fit:-1""#,
        "7",
    ] {
        let body =
            format!(r#"{{"schema":1,"verdict":"FAIL","evidence":"e","findings":{findings}}}"#);
        assert_eq!(gate_of(&body), Reading::Unknown, "{body}");
    }
}

/// with_verdicts は走行ごとに、その走行の id の file の字だけを読んで置き、file の無い走行は Unknown。
/// runs が Unknown の doc は替えない。
#[test]
fn bvverd_with_files_per_run() {
    let gates = [(RUN_B, GATE_FAIL)];
    let reviews = [(RUN_A, REVIEW_FAIL), (RUN_B, REVIEW_PASS)];
    let doc = with_verdicts(runs_of(LOG, &bead()), reader(&gates), reader(&reviews));
    let lines = known(&doc);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].run, RUN_A);
    assert_eq!(lines[0].gate, Reading::Unknown);
    assert_eq!(lines[0].review, review_of(REVIEW_FAIL));
    assert_eq!(lines[1].run, RUN_B);
    assert_eq!(lines[1].gate, gate_of(GATE_FAIL));
    assert_eq!(lines[1].review, review_of(REVIEW_PASS));
    assert!(matches!(lines[1].gate, Reading::Known(_)));
    assert_eq!(
        stages(&doc),
        stages(&runs_of(LOG, &bead())),
        "段の列は替えない"
    );

    let unread = with_verdicts(runs_of("", &bead()), reader(&gates), reader(&reviews));
    assert_eq!(unread.runs, Reading::Unknown);
}

/// 電文の走行は鍵 gate と review を持ち、判定の object の鍵は型の欄のとおり。鍵 gate と review の無い走行の電文は Unknown に読む。
#[test]
fn bvverd_wire_keys_and_old_shape() {
    let gates = [(RUN_B, GATE_FAIL)];
    let reviews = [(RUN_A, REVIEW_FAIL)];
    let doc = with_verdicts(runs_of(LOG, &bead()), reader(&gates), reader(&reviews));
    let text = wire::encode(&doc).expect("電文");
    let v: Value = serde_json::from_str(&text).expect("JSON");
    let lines = v["runs"]["known"].as_array().expect("runs の known の列");
    assert_eq!(lines[0]["gate"], "unknown");
    assert_eq!(
        keys(&lines[0]["review"]["known"]),
        set(&["verdict", "evidence", "kind", "place", "at"])
    );
    assert_eq!(
        keys(&lines[1]["gate"]["known"]),
        set(&["verdict", "evidence", "findings", "at"])
    );
    assert_eq!(
        keys(&lines[1]["gate"]["known"]["findings"][0]),
        set(&["category", "count"])
    );
    assert_eq!(lines[1]["review"], "unknown");
    assert_eq!(wire::decode::<RunsDoc>(&text).expect("読む"), doc);

    let old: RunLine = wire::decode(
        r#"{"run":"fx-v.1-20261001T000000Z","started_at":null,"account":null,"steps":[],
        "cost":{"events":0,"turns":0,"wall_ms":0,"tokens_in":0,"tokens_out":0,"cache_read":0,"cache_create":0}}"#,
    )
    .expect("鍵 gate と review の無い走行の電文");
    assert_eq!(old.gate, Reading::Unknown);
    assert_eq!(old.review, Reading::Unknown);
}
