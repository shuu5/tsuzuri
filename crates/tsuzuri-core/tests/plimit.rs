//! pipeline の session の限度の印の歯（行 c-limit-pipe・接頭辞 plimit_）。
//! 器が run を上限で止めた印（RunStage の段 RateLimited）だけを写して行の state を Limit にし、
//! 残量の字（使用量・窓・閾値）は読まないことを見る。入力の字は歯の中で組む（fixture の file は置かない）。

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::json;
use tsuzuri_contract::account::SessionLine;
use tsuzuri_contract::board::{Reading, Stage};
use tsuzuri_contract::seat::SeatState;
use tsuzuri_contract::surface::SeatRole;
use tsuzuri_core::account::host::{self, HostTexts};
use tsuzuri_core::account::project::{ProjectTexts, RATE_LIMITED, doc, session_lines};

/// 2026-09-27T00:00:00Z。
const DAY0: u64 = 1_790_467_200;

/// 今 11:25。
const AT_1125: u64 = DAY0 + 11 * 3600 + 25 * 60;

/// 今 12:00。
const AT_1200: u64 = DAY0 + 12 * 3600;

/// 2026-09-27 の UTC の時と分の秒。
fn hm(h: u64, m: u64) -> u64 {
    DAY0 + h * 3600 + m * 60
}

const HOST_TOML: &str = r#"[[account]]
label = "acct-1"

[[account]]
label = "acct-2"

[[account-group]]
name = "main"
anchors = ["/work/proj-a"]
accounts = ["acct-1", "acct-2"]
"#;

const DOCTOR: &str = "group=main accounts=acct-1,acct-2 anchors=1 seat-accounts=acct-1 current=acct-1 next=acct-2 refused=-\n\
account=acct-1 dir=/work/accounts/acct-1 retired=no\n\
account=acct-2 dir=/work/accounts/acct-2 retired=no\n";

const SEAT_DOCTOR: &str =
    "seat: role=orchestrator anchor=/work/proj-a target=proj-a:0.1 account=acct-1 model=opus\n";

/// 残量の字の組 B。
const USAGE_B: &str = "usage: account=acct-1 five_hour=40% resets=2026-09-27T15:00:00Z seven_day=20% resets=2026-10-01T00:00:00Z model=opus:10% resets=none\n\
usage: account=acct-2 five_hour=5% resets=none seven_day=30% resets=none model=sonnet:0% resets=none\n";

/// 残量の字の組 F（両方の口座の 3 つの窓が 100%・戻る時刻は今より後）。
const USAGE_F: &str = "usage: account=acct-1 five_hour=100% resets=2026-09-27T15:00:00Z seven_day=100% resets=2026-10-01T00:00:00Z model=opus:100% resets=2026-10-01T00:00:00Z\n\
usage: account=acct-2 five_hour=100% resets=2026-09-27T15:00:00Z seven_day=100% resets=2026-10-01T00:00:00Z model=sonnet:100% resets=2026-10-01T00:00:00Z\n";

/// event log の行（種類・bead・run の時刻の札・時・分・stage・detail）。
const EVENTS: &[(&str, &str, &str, u64, u64, &str, &str)] = &[
    // pl-5: 上限で止まり、起こし直されない。
    ("RunCreated", "pl-5", "090000", 9, 0, "Intake", ""),
    ("RunStage", "pl-5", "090000", 9, 1, "Spawned", "account:acct-1"),
    ("SeatSpawned", "pl-5", "090000", 9, 1, "", ""),
    ("SeatStopped", "pl-5", "090000", 10, 30, "", ""),
    (
        "RunStage",
        "pl-5",
        "090000",
        10,
        30,
        "RateLimited",
        "rc:75,status:allowed_warning,account:acct-1",
    ),
    // pl-1: 11:40 に上限で止まる。
    ("RunCreated", "pl-1", "110000", 11, 0, "Intake", ""),
    ("RunStage", "pl-1", "110000", 11, 1, "Spawned", "account:acct-1"),
    ("SeatSpawned", "pl-1", "110000", 11, 1, "", ""),
    ("SeatStopped", "pl-1", "110000", 11, 40, "", ""),
    (
        "RunStage",
        "pl-1",
        "110000",
        11,
        40,
        "RateLimited",
        "rc:75,status:allowed_warning,account:acct-1",
    ),
    // pl-2: 11:20 に上限で止まり、11:30 に別の口座で起こし直される。
    ("RunCreated", "pl-2", "111000", 11, 10, "Intake", ""),
    ("RunStage", "pl-2", "111000", 11, 11, "Spawned", "account:acct-2"),
    ("SeatSpawned", "pl-2", "111000", 11, 11, "", ""),
    ("SeatStopped", "pl-2", "111000", 11, 20, "", ""),
    (
        "RunStage",
        "pl-2",
        "111000",
        11,
        20,
        "RateLimited",
        "rc:75,status:unknown,account:acct-2",
    ),
    (
        "RunStage",
        "pl-2",
        "111000",
        11,
        30,
        "Spawned",
        "account:acct-1,resume:rate-limit",
    ),
    ("SeatSpawned", "pl-2", "111000", 11, 30, "", ""),
    // pl-3: 通った審査で席が無い。
    ("RunCreated", "pl-3", "112000", 11, 20, "Intake", ""),
    ("RunStage", "pl-3", "112000", 11, 21, "Reviewed", "verdict:PASS"),
    // pl-4: 上限で止まった後に pipe stop で終わる。
    ("RunCreated", "pl-4", "112500", 11, 25, "Intake", ""),
    ("RunStage", "pl-4", "112500", 11, 26, "Spawned", "account:acct-2"),
    ("SeatSpawned", "pl-4", "112500", 11, 26, "", ""),
    ("SeatStopped", "pl-4", "112500", 11, 30, "", ""),
    (
        "RunStage",
        "pl-4",
        "112500",
        11,
        30,
        "RateLimited",
        "rc:75,status:allowed_warning,account:acct-2",
    ),
    ("RunStopped", "pl-4", "112500", 11, 35, "Stopped", ""),
    // pl-6: SeatStopped を書き落とした log（席が立ったままでも印が先に立つ）。
    ("RunCreated", "pl-6", "114500", 11, 45, "Intake", ""),
    ("RunStage", "pl-6", "114500", 11, 46, "Spawned", "account:acct-2"),
    ("SeatSpawned", "pl-6", "114500", 11, 46, "", ""),
    (
        "RunStage",
        "pl-6",
        "114500",
        11,
        50,
        "RateLimited",
        "rc:75,status:unknown,account:acct-2",
    ),
];

/// 歯の組の event log の字（1 行 1 件）。
fn events() -> String {
    EVENTS
        .iter()
        .map(|&(kind, bead, stamp, h, m, stage, detail)| {
            json!({
                "kind": kind,
                "run": format!("{bead}-20260927T{stamp}Z"),
                "bead": bead,
                "ts": format!("2026-09-27T{h:02}:{m:02}:00Z"),
                "stage": stage,
                "detail": detail,
            })
            .to_string()
                + "\n"
        })
        .collect()
}

fn host_with(usage: Option<&str>) -> HostTexts {
    HostTexts {
        host_toml: Some(HOST_TOML.into()),
        usage: usage.map(str::to_string),
        doctor: Some(DOCTOR.into()),
        seat_doctors: BTreeMap::from([("/work/proj-a".to_string(), SEAT_DOCTOR.to_string())]),
        ..HostTexts::default()
    }
}

fn proj_a() -> ProjectTexts {
    ProjectTexts {
        state_dir_known: true,
        state_log: Some("{\"state\":\"busy\",\"ts\":1790505060}\n".into()),
        events: Some(events()),
        ..ProjectTexts::default()
    }
}

fn projects_of(p: ProjectTexts) -> BTreeMap<String, ProjectTexts> {
    BTreeMap::from([("/work/proj-a".to_string(), p)])
}

fn projects() -> BTreeMap<String, ProjectTexts> {
    projects_of(proj_a())
}

/// 同じ入力で events を無しにした session_lines のただ 1 つの行。
fn orchestrator_only(host: &HostTexts, p: ProjectTexts, now: u64) -> SessionLine {
    let lines = session_lines(host, &projects_of(ProjectTexts { events: None, ..p }), now);
    assert_eq!(lines.len(), 1, "{lines:?}");
    lines.into_iter().next().expect("行")
}

fn pl(
    name: &str,
    account: Option<&str>,
    state: SeatState,
    stage: Option<Stage>,
    since: u64,
) -> SessionLine {
    SessionLine {
        project: "proj-a".into(),
        role: SeatRole::Pipeline,
        name: name.into(),
        account: account.map(str::to_string),
        state,
        stage,
        since: Some(since),
        spans: Reading::Unknown,
    }
}

#[test]
fn plimit_const_names_vessel_word() {
    assert_eq!(RATE_LIMITED, "RateLimited");
}

#[test]
fn plimit_marks_on_both_times() {
    let host = host_with(Some(USAGE_B));
    let cases = [
        (
            AT_1125,
            vec![
                pl(
                    "pl-5-20260927T090000Z",
                    Some("acct-1"),
                    SeatState::Limit,
                    None,
                    hm(10, 30),
                ),
                pl(
                    "pl-1-20260927T110000Z",
                    Some("acct-1"),
                    SeatState::Run,
                    Some(Stage::Running),
                    hm(11, 1),
                ),
                pl(
                    "pl-2-20260927T111000Z",
                    Some("acct-2"),
                    SeatState::Limit,
                    None,
                    hm(11, 20),
                ),
                pl(
                    "pl-3-20260927T112000Z",
                    None,
                    SeatState::Wait,
                    Some(Stage::Running),
                    hm(11, 21),
                ),
                pl(
                    "pl-4-20260927T112500Z",
                    None,
                    SeatState::Wait,
                    Some(Stage::Running),
                    hm(11, 25),
                ),
            ],
        ),
        (
            AT_1200,
            vec![
                pl(
                    "pl-1-20260927T110000Z",
                    Some("acct-1"),
                    SeatState::Limit,
                    None,
                    hm(11, 40),
                ),
                pl(
                    "pl-2-20260927T111000Z",
                    Some("acct-1"),
                    SeatState::Run,
                    Some(Stage::Running),
                    hm(11, 30),
                ),
                pl(
                    "pl-3-20260927T112000Z",
                    None,
                    SeatState::Wait,
                    Some(Stage::Running),
                    hm(11, 21),
                ),
                pl(
                    "pl-6-20260927T114500Z",
                    Some("acct-2"),
                    SeatState::Limit,
                    None,
                    hm(11, 50),
                ),
            ],
        ),
    ];
    assert_eq!(AT_1125, 1_790_508_300);
    assert_eq!(AT_1200, 1_790_510_400);
    for (now, pipes) in cases {
        let orch = orchestrator_only(&host, proj_a(), now);
        assert_eq!(
            (
                orch.project.as_str(),
                orch.role,
                orch.name.as_str(),
                orch.account.as_deref(),
                orch.state,
            ),
            (
                "proj-a",
                SeatRole::Orchestrator,
                "proj-a:0.1",
                Some("acct-1"),
                SeatState::Run
            )
        );
        let mut want = vec![orch];
        want.extend(pipes);
        assert_eq!(session_lines(&host, &projects(), now), want, "今 {now}");
    }
}

#[test]
fn plimit_seat_and_usage_apart() {
    let b = session_lines(&host_with(Some(USAGE_B)), &projects(), AT_1200);
    // 組 F は両方の口座の five_hour と seven_day が 100（host の accounts の値）。
    let rows = match host::accounts(&host_with(Some(USAGE_F))) {
        Reading::Known(rows) => rows,
        Reading::Unknown => panic!("accounts が Unknown"),
    };
    assert_eq!(rows.len(), 2);
    for row in &rows {
        let usage = match &row.usage {
            Reading::Known(u) => u,
            Reading::Unknown => panic!("{} の usage が Unknown", row.label),
        };
        for window in ["five_hour", "seven_day"] {
            let q = usage
                .iter()
                .find(|q| q.window == window)
                .unwrap_or_else(|| panic!("{} の {window}", row.label));
            assert_eq!(q.used_pct, 100, "{} の {window}", row.label);
        }
    }
    // 残量の字が無くても組 F でも、行の列は組 B と全部の欄で同じ。
    assert_eq!(session_lines(&host_with(None), &projects(), AT_1200), b);
    assert_eq!(
        session_lines(&host_with(Some(USAGE_F)), &projects(), AT_1200),
        b
    );
    // 合図の最後の判定が account-pressed なら orchestrator の行だけが Limit。
    let pressed = projects_of(ProjectTexts {
        tick_last: Some("ts=1790509800 decision=move reason=account-pressed\n".into()),
        ..proj_a()
    });
    let got = session_lines(&host_with(Some(USAGE_B)), &pressed, AT_1200);
    assert_eq!(got[0].role, SeatRole::Orchestrator);
    assert_eq!(got[0].state, SeatState::Limit);
    assert_eq!(b[0].state, SeatState::Run);
    assert_eq!(got[1..], b[1..]);
}

#[test]
fn plimit_doc_carries_mark() {
    let host = host_with(Some(USAGE_B));
    let d = doc(&host, &projects(), None, AT_1200);
    assert_eq!(d.sessions, session_lines(&host, &projects(), AT_1200));
    let limited: Vec<&SessionLine> = d
        .sessions
        .iter()
        .filter(|s| s.role == SeatRole::Pipeline && s.state == SeatState::Limit)
        .collect();
    let names: Vec<&str> = limited.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["pl-1-20260927T110000Z", "pl-6-20260927T114500Z"]);
    for s in limited {
        let v = serde_json::to_value(s).expect("電文");
        assert_eq!(v["state"], json!("limit"), "{}", s.name);
    }
}

/// 着地済みの歯と、ほかの行の歯の filter の語（78 語）。
const FILTER_WORDS: [&str; 78] = [
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
    "runsdoc_",
    "nbatch_",
    "kcli_",
    "hcard_",
    "ntime_",
    "ptitle_",
    "ncard_",
    "hsym_",
    "smore_",
    "lcard_",
    "aaround_",
    "hruling_",
    "sxaxis_",
    "bport_",
];

#[test]
fn plimit_own_names_clean() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/plimit.rs");
    let src = std::fs::read_to_string(&path).expect("自分の file を読む");
    let mut names = Vec::new();
    let mut after_attr = false;
    for line in src.lines().map(str::trim) {
        if line == "#[test]" {
            after_attr = true;
            continue;
        }
        if after_attr && !line.starts_with("#[") {
            let name = line
                .strip_prefix("fn ")
                .and_then(|l| l.split('(').next())
                .unwrap_or_else(|| panic!("#[test] の後が fn でない: {line}"));
            names.push(name.to_string());
            after_attr = false;
        }
    }
    assert_eq!(names.len(), 5, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("plimit_")
            .unwrap_or_else(|| panic!("{name} が plimit_ で始まらない"));
        for word in FILTER_WORDS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
