//! bead の走行の時間軸の歯（接頭辞 runsdoc_・設計ノート surface-wave3b 行 e-runs の完了の条件）。
//! fixture: tests/fixtures/graph/real/events.jsonl（器の実物の写し・読むだけ）。短い字の組は歯の中に書く。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::runs::{PATH, RunCost, RunLine, RunStep, RunsDoc};
use tsuzuri_contract::wire;
use tsuzuri_core::pipeline::runs_of;

/// 着地済みの行とこの波の行の verify の filter の語（この行の接頭辞は並べない）。
const FILTERS: [&str; 65] = [
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "acctframe_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "askcard_",
    "batchpanel_",
    "board_min_",
    "contract_form_",
    "frame_",
    "gapspage_",
    "gquestion_",
    "graph_",
    "gview_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hook_",
    "hsblock_",
    "hsderive_",
    "hspage_",
    "ledgerblock_",
    "mapgraph_",
    "mapview_",
    "nextstep_",
    "nodepage_",
    "parts_",
    "pipe_",
    "project_",
    "question_",
    "seatblock_",
    "seatcard_",
    "server_",
    "skeleton_",
    "stage_",
    "stats_",
    "steady_",
    "topbar_",
    "tz_",
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
    "ilink_",
    "afocus_",
    "nxact_",
    "urpanel_",
    "saxis_",
    "ticker_",
    "hfig_",
    "pmore_",
    "lspark_",
    "apop_",
    "brand_",
    "nbatch_",
];

fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn real_log() -> String {
    read(&manifest("../../tests/fixtures/graph/real/events.jsonl"))
}

fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("bead の id")
}

/// Known の走行の列（Unknown なら落とす）。
fn known(doc: &RunsDoc) -> &[RunLine] {
    match &doc.runs {
        Reading::Known(lines) => lines,
        Reading::Unknown => panic!("runs が Unknown"),
    }
}

fn stages(line: &RunLine) -> Vec<&str> {
    line.steps.iter().map(|s| s.stage.as_str()).collect()
}

/// 段ごとの（verdict・verdict_kind）。
fn verdicts(line: &RunLine) -> Vec<(Option<&str>, Option<&str>)> {
    line.steps
        .iter()
        .map(|s| (s.verdict.as_deref(), s.verdict_kind.as_deref()))
        .collect()
}

#[test]
fn runsdoc_hub2_three_runs() {
    let doc = runs_of(&real_log(), &bead("t3-hub.2"));
    assert_eq!(doc.bead, bead("t3-hub.2"));
    let lines = known(&doc);
    let ids: Vec<&str> = lines.iter().map(|l| l.run.as_str()).collect();
    assert_eq!(
        ids,
        [
            "t3-hub.2-20260927T071348Z",
            "t3-hub.2-20260927T071750Z",
            "t3-hub.2-20260927T072139Z",
        ]
    );

    let first = &lines[0];
    assert_eq!(first.started_at, Some(1790493228));
    assert_eq!(first.account, None);
    assert_eq!(
        first.steps,
        [
            RunStep {
                at: Some(1790493228),
                stage: "Intake".to_string(),
                detail: Some("classes:".to_string()),
                verdict: None,
                verdict_kind: None,
            },
            RunStep {
                at: Some(1790493247),
                stage: "Reviewed".to_string(),
                detail: Some("verdict:FAIL kind:other".to_string()),
                verdict: Some("FAIL".to_string()),
                verdict_kind: Some("other".to_string()),
            },
        ]
    );
    assert_eq!(
        first.cost,
        RunCost {
            events: 1,
            turns: 1,
            wall_ms: 17235,
            tokens_in: 2,
            tokens_out: 1541,
            cache_read: 10896,
            cache_create: 8407,
        }
    );

    hub2_second(lines);

    hub2_third(lines);
}

/// 走行 t3-hub.2 の 2 つ目（段 5 つ・口座 acct-1）。
fn hub2_second(lines: &[RunLine]) {
    let second = &lines[1];
    assert_eq!(second.started_at, Some(1790493470));
    assert_eq!(second.account.as_deref(), Some("acct-1"));
    assert_eq!(
        stages(second),
        ["Intake", "Reviewed", "Spawned", "Implemented", "Gated"]
    );
    assert_eq!(
        verdicts(second),
        [
            (None, None),
            (Some("PASS"), None),
            (None, None),
            (None, None),
            (Some("FAIL"), None),
        ]
    );
    assert_eq!(second.steps[3].detail, None);
    assert_eq!(
        second.cost,
        RunCost {
            events: 2,
            turns: 33,
            wall_ms: 155603,
            tokens_in: 32,
            tokens_out: 15638,
            cache_read: 503756,
            cache_create: 43725,
        }
    );
}

/// 走行 t3-hub.2 の 3 つ目（段 7 つ・着地まで）。
fn hub2_third(lines: &[RunLine]) {
    let third = &lines[2];
    assert_eq!(third.started_at, Some(1790493699));
    assert_eq!(third.account.as_deref(), Some("acct-1"));
    assert_eq!(
        stages(third),
        [
            "Intake",
            "Reviewed",
            "Spawned",
            "Implemented",
            "Gated",
            "Gated",
            "Landed"
        ]
    );
    assert_eq!(
        verdicts(third),
        [
            (None, None),
            (Some("PASS"), None),
            (None, None),
            (None, None),
            (Some("PASS"), None),
            (None, None),
            (None, None),
        ]
    );
    assert_eq!(third.steps[5].detail.as_deref(), Some("turn:taken"));
    assert_eq!(
        third.cost,
        RunCost {
            events: 3,
            turns: 40,
            wall_ms: 212986,
            tokens_in: 38,
            tokens_out: 23140,
            cache_read: 706443,
            cache_create: 69522,
        }
    );
}

#[test]
fn runsdoc_hub5_second_run_inconclusive() {
    let doc = runs_of(&real_log(), &bead("t3-hub.5"));
    let lines = known(&doc);
    assert_eq!(lines.len(), 3);
    let second = &lines[1];
    assert_eq!(second.run, "t3-hub.5-20260927T080945Z");
    assert_eq!(stages(second), ["Intake", "Reviewed"]);
    assert_eq!(
        verdicts(second)[1],
        (Some("INCONCLUSIVE"), Some("section-material-missing"))
    );
}

#[test]
fn runsdoc_absent_bead_is_known_empty() {
    let doc = runs_of(&real_log(), &bead("t3-hub.9"));
    assert_eq!(doc.bead, bead("t3-hub.9"));
    assert_eq!(doc.runs, Reading::Known(Vec::new()));
}

#[test]
fn runsdoc_unread_log_is_unknown() {
    let broken = concat!(
        r#"{"kind":"RunCreated","run":"fx-1-20260927T000000Z","bead":"fx-1","stage":"Intake"}"#,
        "\n",
        "not json\n",
    );
    for text in ["", broken] {
        let doc = runs_of(text, &bead("fx-1"));
        assert_eq!(doc.runs, Reading::Unknown, "{text:?}");
        assert_eq!(doc.bead, bead("fx-1"));
    }
}

#[test]
fn runsdoc_early_events_dropped() {
    let text = concat!(
        r#"{"ts":"2026-09-27T00:00:01Z","kind":"RunStage","run":"fx-1-20260927T000000Z","stage":"Reviewed","detail":"verdict:PASS,account:acct-9"}"#,
        "\n",
        r#"{"ts":"2026-09-27T00:00:02Z","kind":"RunCost","run":"fx-1-20260927T000000Z","usage":"in:5","turns":1,"wall_ms":10}"#,
        "\n",
        r#"{"ts":"2026-09-27T00:00:03Z","kind":"RunCreated","run":"fx-1-20260927T000000Z","stage":"Intake"}"#,
        "\n",
        r#"{"ts":"2026-09-27T00:00:04Z","kind":"RunStage","run":"fx-1-20260927T000000Z","stage":"Gated","detail":"verdict:FAIL"}"#,
        "\n",
    );
    let doc = runs_of(text, &bead("fx-1"));
    let lines = known(&doc);
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    assert_eq!(line.run, "fx-1-20260927T000000Z");
    assert_eq!(line.account, None);
    assert_eq!(stages(line), ["Intake", "Gated"]);
    assert_eq!(verdicts(line), [(None, None), (Some("FAIL"), None)]);
    assert_eq!(line.cost, RunCost::default());
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

#[test]
fn runsdoc_wire_keys() {
    assert_eq!(PATH, "/api/runs");
    let doc = runs_of(&real_log(), &bead("t3-hub.2"));
    let text = wire::encode(&doc).expect("電文");
    let v: Value = serde_json::from_str(&text).expect("JSON");
    assert_eq!(keys(&v), set(&["bead", "runs"]));
    assert_eq!(v["bead"], "t3-hub.2");
    let lines = v["runs"]["known"].as_array().expect("runs の known の列");
    assert_eq!(lines.len(), 3);
    for line in lines {
        assert_eq!(
            keys(line),
            set(&[
                "run",
                "started_at",
                "account",
                "steps",
                "cost",
                "gate",
                "review",
            ])
        );
        assert_eq!(
            keys(&line["cost"]),
            set(&[
                "events",
                "turns",
                "wall_ms",
                "tokens_in",
                "tokens_out",
                "cache_read",
                "cache_create",
            ])
        );
        for step in line["steps"].as_array().expect("steps の列") {
            assert_eq!(
                keys(step),
                set(&["at", "stage", "detail", "verdict", "verdict_kind"])
            );
        }
    }
    assert_eq!(lines[0]["account"], Value::Null);
    assert_eq!(wire::decode::<RunsDoc>(&text).expect("読む"), doc);

    let unknown = runs_of("", &bead("t3-hub.2"));
    let text = wire::encode(&unknown).expect("電文");
    let v: Value = serde_json::from_str(&text).expect("JSON");
    assert_eq!(keys(&v), set(&["bead", "runs"]));
    assert_eq!(v["runs"], "unknown");
    assert_eq!(wire::decode::<RunsDoc>(&text).expect("読む"), unknown);
}

/// lint を黙らせる属性の行が無く、行頭の fn の宣言の引数がどれも 5 以下（規則の行 R-4）。
#[test]
fn runsdoc_no_lint_allow() {
    let text = read(&manifest("tests/teeth2/runsdoc.rs"));
    for line in text.lines() {
        let head = line.trim_start();
        for attr in ["#[allow(", "#![allow(", "#[expect(", "#![expect("] {
            assert!(!head.starts_with(attr), "lint を黙らせる属性の行: {line}");
        }
    }
    let mut decls = 0;
    for (at, _) in text.match_indices("\nfn ") {
        let rest = &text[at + 1..];
        let open = rest.find('(').expect("fn の宣言の開き括弧");
        let mut depth = 0;
        let mut args = 0;
        let mut body = rest[open..].chars().peekable();
        while let Some(c) = body.next() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                ':' if depth == 1 && body.peek() == Some(&' ') => args += 1,
                _ => {}
            }
        }
        let decl = rest.lines().next().unwrap_or_default();
        assert!(args <= 5, "引数が {args} の fn: {decl}");
        decls += 1;
    }
    assert!(decls >= 10, "fn の宣言が {decls}");
}

#[test]
fn runsdoc_own_names_clean() {
    let text = read(&manifest("tests/teeth2/runsdoc.rs"));
    let mut names = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "#[test]" {
            continue;
        }
        let decl = lines.next().expect("test の属性の次の行");
        let name = decl
            .trim()
            .strip_prefix("fn ")
            .and_then(|r| r.split('(').next())
            .unwrap_or_else(|| panic!("fn の宣言でない: {decl}"));
        names.push(name.to_string());
    }
    assert!(names.len() >= 7, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("runsdoc_")
            .unwrap_or_else(|| panic!("{name} が runsdoc_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
