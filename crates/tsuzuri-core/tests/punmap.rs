//! 行 c-pipe-unmapped の歯（中核）: 器の RunStage の段 Failed と Stopped を板の段 Failed と Stopped にし、
//! 段の理由は detail の字にする。detail が `RETIRED` の RunStage（器が worktree を畳んだ記帳）は段を決めない。
//! 節の組は歯の中で組む（fixture の file は使わない）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::{Value, json};
use tsuzuri_contract::board::{NextMove, PipelineCard, Reading, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::seat::SeatState;
use tsuzuri_contract::stats::CheckResult;
use tsuzuri_contract::surface::SeatRole;
use tsuzuri_core::account::host::HostTexts;
use tsuzuri_core::account::project::{ProjectTexts, session_lines};
use tsuzuri_core::next_step::next_step;
use tsuzuri_core::pipeline::{Board, RETIRED, board, stage_of};

/// 節の今（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

fn bead(n: u32, status: &str) -> Value {
    let mut b = json!({"id": format!("ua.{n}"), "title": format!("t{n}"), "status": status, "issue_type": "task"});
    if status == "closed" {
        b["closed_at"] = json!("2026-09-27T11:45:00Z");
        b["close_reason"] = json!("withdrawn");
    }
    b
}

/// 節の台帳の一覧（ua.1 から ua.11・ua.10 だけ閉じている）。
fn ledger() -> String {
    Value::Array(
        (1..=11)
            .map(|n| bead(n, if n == 10 { "closed" } else { "open" }))
            .collect(),
    )
    .to_string()
}

/// event の 1 行（run は bead とハイフンと RunCreated の時刻の札・時刻は 2026-09-27 の時と分）。
fn ev(t: &str, kind: &str, run: &str, stage: Option<&str>, detail: Option<&str>) -> String {
    let (bead, _) = run.rsplit_once('-').expect("run の id");
    let mut e = json!({"schema": 1, "ts": format!("2026-09-27T{t}:00Z"), "kind": kind, "run": run, "bead": bead});
    if let Some(s) = stage {
        e["stage"] = json!(s);
    }
    if let Some(d) = detail {
        e["detail"] = json!(d);
    }
    e.to_string()
}

fn created(t: &str, run: &str) -> String {
    ev(t, "RunCreated", run, Some("Intake"), None)
}

fn stage(t: &str, run: &str, s: &str, detail: Option<&str>) -> String {
    ev(t, "RunStage", run, Some(s), detail)
}

fn stopped(t: &str, run: &str) -> String {
    ev(t, "RunStopped", run, Some("Stopped"), None)
}

/// 節の event log（11 の走行・31 行）。
fn events() -> Vec<String> {
    let r = |n: u32, t: &str| format!("ua.{n}-20260927T{}00Z", t.replace(':', ""));
    let (u1, u2, u3, u4) = (r(1, "10:00"), r(2, "10:10"), r(3, "10:20"), r(4, "10:30"));
    let (u5, u6, u7, u8) = (r(5, "10:50"), r(6, "11:00"), r(7, "11:10"), r(8, "11:20"));
    let (u9, u10, u11) = (r(9, "11:30"), r(10, "11:35"), r(11, "11:40"));
    vec![
        created("10:00", &u1),
        stage("10:01", &u1, "Spawned", Some("base:0a,account:acct-1")),
        stage("10:20", &u1, "Failed", Some("oom-kill")),
        stopped("10:21", &u1),
        created("10:10", &u2),
        stage("10:30", &u2, "Gated", Some("verdict:PASS")),
        stage("10:31", &u2, "Failed", Some("main-red")),
        created("10:20", &u3),
        stage("10:40", &u3, "Failed", Some("runner-rc:1,commits:0")),
        created("10:30", &u4),
        stage("10:42", &u4, "Gated", Some("verdict:INCONCLUSIVE")),
        stopped("10:43", &u4),
        stage("10:43", &u4, "Stopped", Some(RETIRED)),
        created("10:50", &u5),
        stage("10:51", &u5, "Questioned", Some("about:write-set")),
        stopped("10:52", &u5),
        stage("10:52", &u5, "Stopped", Some(RETIRED)),
        created("11:00", &u6),
        stage("11:05", &u6, "Gated", Some("verdict:FAIL")),
        stage("11:06", &u6, "Gated", Some(RETIRED)),
        created("11:10", &u7),
        stage("11:11", &u7, "Stopped", Some("reason:見本")),
        created("11:20", &u8),
        stage("11:21", &u8, "Failed", None),
        created("11:30", &u9),
        stage("11:31", &u9, "RateLimited", None),
        created("11:35", &u10),
        stage("11:36", &u10, "Failed", Some("oom-kill")),
        created("11:40", &u11),
        ev("11:50", "RunDone", &u11, Some("Landed"), Some("sha:0a")),
        stage("11:51", &u11, "Landed", Some(RETIRED)),
    ]
}

fn card(n: u32, stage: Stage, reason: Option<&str>, account: Option<&str>, elapsed: u64) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(format!("ua.{n}").as_str()).expect("bead の id"),
        runs: 1,
        stage,
        reason: reason.map(str::to_string),
        account: account.map(str::to_string),
        since: Some(NOW - elapsed),
        ci: None,
    }
}

fn cards(b: &Board) -> &[PipelineCard] {
    match &b.board.cards {
        Reading::Known(c) => c,
        Reading::Unknown => panic!("札が Unknown"),
    }
}

/// (1) stage_of の表は RunStage の段 Failed と Stopped を段 Failed と Stopped・理由は detail の字（空なら無し）にし、
/// RunStopped と小文字の段と段 RateLimited・Landed・Intake と段の名の無い RunStage は表に無い。
#[test]
fn punmap_table_arms() {
    assert_eq!(RETIRED, "retired");
    for detail in ["oom-kill", "retired", "runner-rc:1,commits:0", "verdict:FAIL"] {
        assert_eq!(
            stage_of("RunStage", Some("Failed"), detail),
            Some((Stage::Failed, Some(detail.to_string()))),
            "{detail}"
        );
        assert_eq!(
            stage_of("RunStage", Some("Stopped"), detail),
            Some((Stage::Stopped, Some(detail.to_string()))),
            "{detail}"
        );
    }
    assert_eq!(
        stage_of("RunStage", Some("Failed"), ""),
        Some((Stage::Failed, None))
    );
    assert_eq!(
        stage_of("RunStage", Some("Stopped"), ""),
        Some((Stage::Stopped, None))
    );
    for (kind, s, detail) in [
        ("RunStopped", Some("Stopped"), ""),
        ("RunStage", Some("failed"), "oom-kill"),
        ("RunStage", Some("RateLimited"), ""),
        ("RunStage", Some("Landed"), "retired"),
        ("RunStage", Some("Intake"), ""),
        ("RunStage", None, "oom-kill"),
    ] {
        assert_eq!(stage_of(kind, s, detail), None, "{kind} {s:?} {detail}");
    }
}

/// (2) 節の組の板は節の表の 10 枚と unmapped 1 で、台帳の字が空の板は ua.10 だけが段 Failed。
#[test]
fn punmap_board_cards() {
    let events = events();
    assert_eq!(events.len(), 31);
    let events = events.join("\n");
    let want = |u10: PipelineCard| {
        vec![
            card(1, Stage::Failed, Some("oom-kill"), Some("acct-1"), 6000),
            card(2, Stage::Failed, Some("main-red"), None, 5340),
            card(3, Stage::Failed, Some("runner-rc:1,commits:0"), None, 4800),
            card(4, Stage::Failed, Some("verdict:INCONCLUSIVE"), None, 4680),
            card(5, Stage::Questioned, Some("質問の後に止めた（write-set）"), None, 4140),
            card(6, Stage::Failed, Some("verdict:FAIL"), None, 3300),
            card(7, Stage::Stopped, Some("reason:見本"), None, 2940),
            card(8, Stage::Failed, None, None, 2340),
            u10,
            card(11, Stage::Landed, None, None, 600),
        ]
    };
    let b = board(&ledger(), &events, NOW);
    assert_eq!(
        cards(&b),
        want(card(10, Stage::Landed, Some("closed:withdrawn"), None, 1440)).as_slice()
    );
    assert_eq!(b.unmapped, 1);

    let b = board("", &events, NOW);
    assert_eq!(
        cards(&b),
        want(card(10, Stage::Failed, Some("oom-kill"), None, 1440)).as_slice()
    );
    assert_eq!(b.unmapped, 1);
}

/// (3) 止まっている走行は open の bead の段 Failed と Stopped の 7 枚で、対象は経過のいちばん長い ua.1。
#[test]
fn punmap_next_step_counts() {
    let step = next_step(&ledger(), &events().join("\n"), NOW);
    let stalled = step
        .checks
        .iter()
        .find(|c| c.kind == NextMove::StalledRun)
        .expect("止まっている走行の結果");
    assert_eq!((stalled.result, stalled.count), (CheckResult::Hit, 7));
    assert_eq!(stalled.target.as_ref().map(BeadId::as_str), Some("ua.1"));
    assert_eq!(step.lead, NextMove::StalledRun);
}

/// (4) 器が落とした席の立った生きている run の session の行は段 Failed・状態 Wait。
#[test]
fn punmap_session_line_failed() {
    let s1 = "se.1-20260927T115000Z";
    let events = [
        created("11:50", s1),
        ev("11:50", "SeatSpawned", s1, None, None),
        stage("11:51", s1, "Spawned", Some("base:0a,account:acct-1")),
        stage("11:55", s1, "Failed", Some("oom-kill")),
    ]
    .join("\n");
    let host = HostTexts {
        host_toml: Some("[[account-group]]\nname = 'g'\nanchors = ['/w/p']\n".to_string()),
        ..HostTexts::default()
    };
    let projects = BTreeMap::from([(
        "/w/p".to_string(),
        ProjectTexts {
            state_dir_known: true,
            events: Some(events.into()),
            ..ProjectTexts::default()
        },
    )]);
    let lines: Vec<_> = session_lines(&host, &projects, NOW)
        .into_iter()
        .filter(|l| l.role == SeatRole::Pipeline)
        .map(|l| {
            (
                l.project,
                l.name,
                l.account,
                l.state,
                l.stage,
                l.since,
                l.spans,
            )
        })
        .collect();
    assert_eq!(
        lines,
        [(
            "p".to_string(),
            s1.to_string(),
            Some("acct-1".to_string()),
            SeatState::Wait,
            Some(Stage::Failed),
            Some(1_790_510_100),
            Reading::Unknown,
        )]
    );
}

/// 着地済みの verify の filter の語を畳んだ 123 語と、後の行 g-gz の接頭辞。
const FILTERS: [&str; 124] = [
    "aaround_",
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "afocus_",
    "aord_",
    "apop_",
    "askcard_",
    "athr_",
    "batchpanel_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadopt_",
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "denv_",
    "ecache_",
    "flight_",
    "fmark_",
    "frame_",
    "fserve_",
    "fstop_",
    "gapspage_",
    "gfresh_",
    "ghb_",
    "glabel_",
    "gnav_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
    "hfig_",
    "hnunk_",
    "hook_",
    "hruling_",
    "hsblock_",
    "hsderive_",
    "hspage_",
    "hsym_",
    "iclose_",
    "ilink_",
    "kcli_",
    "klink_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lspark_",
    "mapview_",
    "mkeys_",
    "mlink_",
    "mstore_",
    "mtree_",
    "nact_",
    "nbatch_",
    "ncard_",
    "nextstep_",
    "nodepage_",
    "nstall_",
    "nsum_",
    "nsumw_",
    "ntime_",
    "nxact_",
    "parts_",
    "pclosed_",
    "pfold_",
    "pipe_",
    "plimit_",
    "pmore_",
    "pquest_",
    "project_",
    "ptitle_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "question_",
    "rhold_",
    "runsdoc_",
    "saxis_",
    "sclosed_",
    "seatblock_",
    "seatcard_",
    "server_",
    "sesplit_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "steady_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "topbar_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
    "pgz_",
];

/// (7) この file の歯の名はどれも punmap_ で始まり、残りの字は filter の語を含まない。
#[test]
fn punmap_own_names_clean() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/punmap.rs");
    let text = std::fs::read_to_string(&path).expect("tests/punmap.rs を読む");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == "#[test]")
        .map(|(i, _)| {
            lines[i + 1..]
                .iter()
                .find_map(|l| l.strip_prefix("fn "))
                .and_then(|rest| rest.split('(').next())
                .expect("test の属性の後の fn")
        })
        .collect();
    assert!(names.len() >= 5, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("punmap_")
            .unwrap_or_else(|| panic!("{name} が punmap_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
