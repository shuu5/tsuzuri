//! 群の起こしの門の判じの歯（接頭辞 aggj_・設計ノート surface-wave29b 行 ag-gjudge・判断の記録 ADR-61 決定 (2)(4)(5)）。
//! 中核の `agent::group` の判じを直に撃つ。見本は群の試し g1 の計画と一覧の形（反証役 3 体・割りの主張 6・5・3・一覧 14 行）を写し、
//! 断りの見本は正しい見本から 1 句だけ外す。
#![cfg(test)]

use serde_json::{Value, json};
use tsuzuri_core::agent::spec::group::judge::{Ask, Ledger, judge, reason, unmarked};
use tsuzuri_core::agent::spec::group::{Plan, Seat, shared, workflow, workflow_reason};
use tsuzuri_core::agent::spec::{Head, Spec};

const NOW: u64 = 1_800_000_000;
const DAY: u64 = 86_400;
const RULING: &str = "t3-hub.90.1:20261005T0900Z-1";

/// 主張の id U<a>〜U<b>。
fn ids(a: usize, b: usize) -> Vec<String> {
    (a..=b).map(|n| format!("U{n}")).collect()
}

/// g1 の形の計画（qv181 が U1〜U6・qv182 が U7〜U11・qv183 が U12〜U14）。
fn plan() -> Value {
    let m = |name: &str, claims: Vec<String>, path: &str| json!({"name": name, "model": "sonnet", "claims": claims, "paths": [path]});
    json!({"type": "tsuzuri:verifier", "target": "t3-hub.89", "draft": "緩める問い", "attacker": "qv180",
        "members": [m("qv181", ids(1, 6), "/S/a.md"), m("qv182", ids(7, 11), "/S/b.rs"), m("qv183", ids(12, 14), "/S/c.md")]})
}

/// 一覧の file の n 行（U1〜U<n>）。
fn list(n: usize) -> String {
    (1..=n)
        .map(|i| format!("U{i}\t主張 {i}\t判じられない訳 {i}\t/S/a.md\n"))
        .collect()
}

/// 対象 `target` の生きた検証役の札と群 `group` の席の札。
fn peer(name: &str, target: &str, spawned: u64, group: &str) -> (Spec, Seat) {
    let spec = Spec {
        name: name.into(),
        kind: "tsuzuri:verifier".into(),
        budget: 150_000,
        build: "なし".into(),
        target: target.into(),
        outputs: vec!["notes.md".into()],
        spawned,
        agent_id: None,
        ended: None,
    };
    let seat = Seat {
        group: group.into(),
        i: 2,
        k: 3,
        claims: vec![],
        paths: vec![],
    };
    (spec, seat)
}

/// 判じの入力の持ち物（呼びの名は qv181）。
struct Case {
    kind: String,
    line: String,
    head: Head,
    model: Option<String>,
    plan: Option<String>,
    list: Option<String>,
    peers: Vec<(Spec, Seat)>,
    totals: Option<(u64, u64)>,
    ledger: Option<Ledger>,
}

impl Case {
    /// 通る見本（同じ群の生きた係 2 体・群の合計 0）。
    fn base() -> Case {
        Case {
            kind: "tsuzuri:verifier".into(),
            line: "g1 1/3".into(),
            head: Head {
                budget: 150_000,
                build: "なし".into(),
                target: "t3-hub.89".into(),
                outputs: vec!["notes.md".into()],
            },
            model: Some("sonnet".into()),
            plan: Some(plan().to_string()),
            list: Some(list(14)),
            peers: vec![
                peer("qv182", "t3-hub.89", NOW - 60, "g1"),
                peer("qv183", "t3-hub.89", NOW - 60, "g1"),
            ],
            totals: Some((0, 0)),
            ledger: None,
        }
    }

    fn run(&self) -> Result<Seat, &'static str> {
        let ask = Ask {
            kind: Some(&self.kind),
            name: "qv181",
            line: &self.line,
            head: &self.head,
            model: self.model.as_deref(),
            plan: self.plan.as_deref(),
            list: self.list.as_deref(),
            peers: &self.peers,
            totals: self.totals,
            ledger: self.ledger.as_ref(),
            now: NOW,
        };
        judge(&ask).map_err(|r| r.key)
    }
}

type Edit = Box<dyn Fn(&mut Case)>;

/// 計画の JSON を 1 か所替える。
fn plan_edit(f: impl Fn(&mut Value) + 'static) -> Edit {
    Box::new(move |c| {
        let mut p = plan();
        f(&mut p);
        c.plan = Some(p.to_string());
    })
}

/// 見本ごとに、通る見本へ 1 句だけ当てた断りの鍵を比べる。
fn refuses(cases: Vec<(&str, Edit)>) {
    for (key, edit) in cases {
        let mut c = Case::base();
        edit(&mut c);
        assert_eq!(c.run().err(), Some(key), "{key}");
    }
}

#[test]
fn aggj_judge_passes_the_g1_plan_and_the_edges_of_each_value() {
    let seat = Case::base().run().unwrap();
    let want = Seat {
        group: "g1".into(),
        i: 1,
        k: 3,
        claims: ids(1, 6),
        paths: vec!["/S/a.md".into()],
    };
    assert_eq!(seat, want);
    let edges: Vec<Edit> = vec![
        Box::new(|c| c.totals = Some((404_999, 6_749_999))),
        Box::new(|c| c.line = "g1 3/3".into()),
        Box::new(|c| {
            let mut old = peer("qv170", "t3-hub.70", NOW - DAY, "g0");
            old.0.ended = Some(NOW - 60);
            c.peers.push(old);
        }),
        Box::new(|c| {
            c.list = Some(list(18));
            let mut p = plan();
            for (n, a) in [(0, 1), (1, 7), (2, 13)] {
                p["members"][n]["claims"] = json!(ids(a, a + 5));
            }
            c.plan = Some(p.to_string());
        }),
        Box::new(|c| {
            c.list = Some(list(3));
            let mut p = plan();
            for n in 0..3 {
                p["members"][n]["claims"] = json!(ids(n + 1, n + 1));
            }
            c.plan = Some(p.to_string());
        }),
        Box::new(|c| {
            let mut old = peer("qv170", "t3-hub.89", NOW - 7 * DAY, "g0");
            old.0.ended = Some(NOW - DAY);
            c.peers.push(old);
        }),
    ];
    for (n, edit) in edges.into_iter().enumerate() {
        let mut c = Case::base();
        edit(&mut c);
        assert!(c.run().is_ok(), "縁 {n}");
    }
}

#[test]
fn aggj_judge_refuses_a_missing_or_malformed_line_or_file() {
    refuses(vec![
        ("line-form", Box::new(|c| c.line = "g1 1-3".into())),
        ("line-form", Box::new(|c| c.line = "g1 0/3".into())),
        ("plan-missing", Box::new(|c| c.plan = None)),
        (
            "plan-form",
            Box::new(|c| c.plan = Some("{\"type\": \"tsuzuri:verifier\"".into())),
        ),
        ("plan-form", plan_edit(|p| p["target"] = json!("t3-hub.88"))),
        ("list-missing", Box::new(|c| c.list = None)),
        (
            "list-form",
            Box::new(|c| c.list = Some(list(13) + "U13\t主張 14\t訳 14\t/S/a.md\n")),
        ),
        (
            "list-form",
            Box::new(|c| c.list = Some(list(13) + "U14\t主張 14\t/S/a.md\n")),
        ),
    ]);
    let plans = [("g1".to_string(), plan().to_string())];
    assert_eq!(
        unmarked("qv181", &plans).map(|r| r.key),
        Some("line-missing")
    );
    assert_eq!(unmarked("qv189", &plans), None);
    assert_eq!(
        unmarked("qv181", &[("g1".to_string(), "{".to_string())]),
        None
    );
}

#[test]
fn aggj_judge_refuses_the_type_the_trigger_and_the_counts() {
    refuses(vec![
        ("type", Box::new(|c| c.kind = "tsuzuri:drafter".into())),
        ("draft", plan_edit(|p| p["draft"] = json!("調べ"))),
        ("few", Box::new(|c| c.list = Some(list(2)))),
        ("many", Box::new(|c| c.list = Some(list(19)))),
        ("index", Box::new(|c| c.line = "g1 4/3".into())),
        ("wide", Box::new(|c| c.line = "g1 1/4".into())),
        (
            "claims",
            plan_edit(|p| {
                p["members"][0]["claims"] = json!(ids(1, 7));
                p["members"][1]["claims"] = json!(ids(8, 11));
            }),
        ),
    ]);
}

#[test]
fn aggj_judge_refuses_overlapping_missing_and_extra_claims() {
    refuses(vec![
        (
            "overlap",
            plan_edit(|p| p["members"][1]["claims"] = json!(ids(6, 11))),
        ),
        (
            "missing",
            plan_edit(|p| p["members"][2]["claims"] = json!(ids(12, 13))),
        ),
        (
            "extra",
            plan_edit(|p| p["members"][2]["claims"] = json!(["U12", "U13", "U14", "U99"])),
        ),
    ]);
}

#[test]
fn aggj_judge_refuses_the_live_count_the_budget_the_model_and_the_totals() {
    refuses(vec![
        (
            "live",
            Box::new(|c| {
                c.peers
                    .push(peer("qv190", "t3-hub.70", NOW - 8 * DAY, "g0"))
            }),
        ),
        ("budget", Box::new(|c| c.head.budget = 150_001)),
        ("model", Box::new(|c| c.model = Some("opus".into()))),
        ("model", Box::new(|c| c.model = None)),
        ("stop", Box::new(|c| c.totals = Some((405_000, 0)))),
        ("stop", Box::new(|c| c.totals = Some((0, 6_750_000)))),
        ("totals-unread", Box::new(|c| c.totals = None)),
    ]);
}

/// 同じ対象のほかの群 g0 の係が 6 日前に在り、計画が裁定 id `ruling` を持ち、台帳の問いの字が `notes` と `desc`。
fn near(ruling: Option<&str>, ledger: Option<Ledger>) -> Edit {
    let ruling = ruling.map(str::to_string);
    Box::new(move |c| {
        let mut old = peer("qv170", "t3-hub.89", NOW - 6 * DAY, "g0");
        old.0.ended = Some(NOW - DAY);
        c.peers.push(old);
        let mut p = plan();
        p["ruling"] = json!(ruling);
        c.plan = Some(p.to_string());
        c.ledger = ledger.clone();
    })
}

fn read(ruling: &str, desc: &str) -> Option<Ledger> {
    let notes = format!(
        "前の行\n裁定 id = {ruling}・問い = {}・逐語 = はい",
        ruling.split(':').next().unwrap()
    );
    Some(Ledger::Read {
        notes,
        description: desc.into(),
    })
}

#[test]
fn aggj_judge_checks_the_seven_day_window_against_the_ledger() {
    let mut c = Case::base();
    near(Some(RULING), read(RULING, "検証の群（g1）を起こす"))(&mut c);
    assert!(c.run().is_ok());
    let other = "t3-hub.90.1:20261005T0800Z-1";
    let mut cases: Vec<(&str, Edit)> = vec![
        ("window", near(None, None)),
        (
            "ruling-absent",
            near(Some(RULING), read(other, "検証の群（g1）を起こす")),
        ),
        (
            "group-unnamed",
            near(Some(RULING), read(RULING, "検証の群を起こす")),
        ),
        (
            "group-unnamed",
            near(Some(RULING), read(RULING, "検証の群（g12）を起こす")),
        ),
        ("ledger-unread", near(Some(RULING), Some(Ledger::Unread))),
        ("ledger-unread", near(Some(RULING), None)),
    ];
    for q in ["t3-hub.87.1", "t3-hub.87.2", "t3-hub.87.6"] {
        let r: &'static str = Box::leak(format!("{q}:20261005T0639Z-1").into_boxed_str());
        cases.push((
            "uncounted",
            near(Some(r), read(r, "検証の群（g1）を起こす")),
        ));
    }
    refuses(cases);
}

#[test]
fn aggj_shared_paths_pass_and_the_seat_workflow_call_is_refused() {
    let mut p = plan();
    p["members"][1]["paths"] = json!(["/S/b.rs", "/S/a.md"]);
    let mut c = Case::base();
    c.plan = Some(p.to_string());
    assert!(c.run().is_ok());
    assert_eq!(
        shared(&Plan::parse(&p.to_string()).unwrap(), "qv181"),
        vec!["/S/a.md".to_string()]
    );
    assert_eq!(
        shared(&Plan::parse(&plan().to_string()).unwrap(), "qv181"),
        Vec::<String>::new()
    );
    let call = |tool: &str, agent: Option<&str>| {
        json!({"session_id": "s", "hook_event_name": "PreToolUse", "tool_name": tool, "agent_id": agent,
            "tool_input": {"script": "x", "plan": "g1/plan.json", "ruling": RULING}})
        .to_string()
    };
    assert!(workflow(&call("Workflow", None)));
    assert!(!workflow(&call("Agent", None)));
    assert!(!workflow(&call("Workflow", Some("a77"))));
    let why = workflow_reason();
    assert!(
        why.starts_with("群の起こしの門は止める（席の流れの道具 Workflow の呼び"),
        "{why}"
    );
    assert!(
        why.contains("次の一手 = 流れの道具で係を起こすには持ち主に問う"),
        "{why}"
    );
    let mut c = Case::base();
    c.head.budget = 150_001;
    let r = judge(&Ask {
        name: "qv181",
        kind: Some("tsuzuri:verifier"),
        line: "g1 1/3",
        head: &c.head,
        model: Some("sonnet"),
        plan: c.plan.as_deref(),
        list: c.list.as_deref(),
        peers: &[],
        totals: Some((0, 0)),
        ledger: None,
        now: NOW,
    })
    .unwrap_err();
    assert_eq!(
        reason(&r),
        "群の起こしの門は止める（頭の予算 150001 が係ごとの上限 150000 を越える） 次の一手 = 計画の file・一覧の file・頼みの頭を直すか、群: の行の無い 1 体の起こしにする（群の値と形は判断の記録 ADR-61）"
    );
}
