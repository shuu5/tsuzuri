//! 行 c-dormant の歯（中核）: account board の休止中の席は、器の登録の行の席のうち、状態の記録の最後の読めた行から
//! 規則の行 R-28 の境（`DORMANT_S`）を越えた席で、session の表から休止中の orchestrator の行を外す。
//! 節の組は歯の中で組む（fixture の file は使わない）。

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::json;
use tsuzuri_contract::account::DormantSeat;
use tsuzuri_contract::board::Reading;
use tsuzuri_core::account::host::HostTexts;
use tsuzuri_core::account::project::{
    DORMANT_S, ProjectTexts, doc, dormant, project_rows, session_lines,
};

/// 節の今（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

/// 節の run（生きていて終わっていない）。
const RUN: &str = "r.1-20260927T110000Z";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 状態の記録の 1 行。
fn state(state: &str, ts: serde_json::Value) -> String {
    json!({"schema": 1, "state": state, "event": "stop", "ts": ts, "sid": "s-1"}).to_string()
}

/// 節の host の字（`logs` なら席の状態の記録の表を持つ）。
fn host(logs: bool) -> HostTexts {
    let doctor_a = "doctor: state dir ok\n\
        seat: role=orchestrator anchor=/w/p-a target=p-a:0.1 account=acct-1 model=opus\n\
        seat: role=orchestrator anchor=/w/p-x target=p-x:0.1 account=acct-3 model=opus\n\
        seat: role=orchestrator anchor=/w/p-y target=p-y:0.1 account= model=opus\n\
        seat: role=orchestrator anchor=/w/p-z target=p-z:0.1 account=acct-3 model=opus\n\
        seat: role=worker anchor=/w/p-w/ target=p-w:0.1 account=acct-2 model=sonnet\n";
    let doctor_b = "doctor: state dir ok\n\
        seat: role=orchestrator anchor=/w/p-a target=p-a:0.1 account=acct-9 model=opus\n\
        seat: role=orchestrator anchor=/w/p-b target=p-b:0.1 account=acct-3 model=opus\n";
    let seat_logs = if logs {
        BTreeMap::from([
            (
                "p-a:0.1".to_string(),
                [
                    state("busy", json!(NOW - 50_000)),
                    state("idle", json!(NOW - 43_201)),
                ]
                .join("\n"),
            ),
            ("p-x:0.1".to_string(), state("idle", json!(NOW - 43_200))),
            (
                "p-z:0.1".to_string(),
                ["not json".to_string(), state("gone", json!(NOW - 90_000))].join("\n"),
            ),
            (
                "p-w:0.1".to_string(),
                [
                    state("busy", json!(1_790_400_000.75)),
                    state("idle", json!("late")),
                    state("busy", json!(-5)),
                ]
                .join("\n"),
            ),
            ("p-b:0.1".to_string(), state("busy", json!(NOW - 60))),
        ])
    } else {
        BTreeMap::new()
    };
    HostTexts {
        host_toml: Some(
            "[[account-group]]\nname = 'g'\nanchors = ['/w/p-a', '/w/p-b']\naccounts = ['acct-1']\n"
                .to_string(),
        ),
        seat_doctors: BTreeMap::from([
            ("/w/p-a".to_string(), doctor_a.to_string()),
            ("/w/p-b".to_string(), doctor_b.to_string()),
        ]),
        seat_logs,
        ..HostTexts::default()
    }
}

/// 節の event log（run は席が立っていて終わっていない）。
fn events() -> String {
    let ev = |ts: &str, kind: &str, stage: Option<&str>| {
        let mut e = json!({"schema": 1, "ts": ts, "kind": kind, "run": RUN, "bead": "r.1"});
        if let Some(s) = stage {
            e["stage"] = json!(s);
        }
        e.to_string()
    };
    [
        ev("2026-09-27T11:00:00Z", "RunCreated", Some("Intake")),
        ev("2026-09-27T11:10:00Z", "SeatSpawned", None),
        ev("2026-09-27T11:20:00Z", "RunStage", Some("Implemented")),
    ]
    .join("\n")
}

/// 節の project の字（`a_known` でなければ /w/p-a は state dir の引けない既定の字）。
fn projects(a_known: bool) -> BTreeMap<String, ProjectTexts> {
    let a = if a_known {
        ProjectTexts {
            state_dir_known: true,
            tick_status: Some(
                "seat tick status: target=p-a:0.1 last=1790467199 age=43201 healthy=no heartbeat=off step=5 next=1790510405\n"
                    .to_string(),
            ),
            events: Some(events().into()),
            ..ProjectTexts::default()
        }
    } else {
        ProjectTexts::default()
    };
    BTreeMap::from([
        ("/w/p-a".to_string(), a),
        (
            "/w/p-b".to_string(),
            ProjectTexts {
                state_dir_known: true,
                ..ProjectTexts::default()
            },
        ),
    ])
}

fn seat(
    project: &str,
    target: &str,
    account: &str,
    last: u64,
    tick: Reading<bool>,
) -> DormantSeat {
    DormantSeat {
        project: project.to_string(),
        target: target.to_string(),
        account: Some(account.to_string()),
        last,
        tick_healthy: tick.clone(),
        heartbeat: tick,
    }
}

#[test]
fn cdorm_bound_matches_rule_r28() {
    assert_eq!(DORMANT_S, 43_200);
    let rules = read("design-intent/rules.yaml");
    let row = rules
        .lines()
        .find(|l| l.contains("id: R-28,"))
        .expect("rules の file に行 R-28 が在る");
    let value = row
        .split_once("value: \"")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map(|(v, _)| v)
        .expect("行 R-28 の value");
    assert_eq!(value, "12 時間（43200 秒）", "{row}");
    let secs: u64 = value
        .split_once('（')
        .and_then(|(_, rest)| rest.split_once('）'))
        .and_then(|(inner, _)| inner.trim().strip_suffix('秒'))
        .and_then(|n| n.trim().parse().ok())
        .expect("括弧の中の秒の数");
    assert_eq!(secs, DORMANT_S, "{row}");
}

#[test]
fn cdorm_lists_seats_past_bound() {
    assert_eq!(
        dormant(&host(true), &projects(true), NOW),
        [
            seat("p-a", "p-a:0.1", "acct-1", 1_790_467_199, Reading::Known(false)),
            seat("p-w", "p-w:0.1", "acct-2", 1_790_400_000, Reading::Unknown),
        ]
    );
    assert_eq!(dormant(&host(false), &projects(true), NOW), []);
    assert_eq!(
        dormant(&host(true), &projects(false), NOW),
        [seat("p-a", "p-a:0.1", "acct-9", 1_790_467_199, Reading::Unknown)]
    );
}

#[test]
fn cdorm_session_table_drops_resting_orchestrator() {
    let names = |host: &HostTexts| -> Vec<String> {
        session_lines(host, &projects(true), NOW)
            .into_iter()
            .map(|s| s.name)
            .collect()
    };
    assert_eq!(names(&host(true)), [RUN, "p-b:0.1"]);
    assert_eq!(names(&host(false)), ["p-a:0.1", RUN, "p-b:0.1"]);
    let (h, p) = (host(true), projects(true));
    let got = doc(&h, &p, NOW);
    assert_eq!(got.dormant, dormant(&h, &p, NOW));
    assert_eq!(got.sessions, session_lines(&h, &p, NOW));
    assert_eq!(got.projects, project_rows(&host(false), &p, NOW));
}
