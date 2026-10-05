//! 行 c-pipe-questioned の歯（中核）: 器の RunStage の段 Questioned を板の段 Questioned に置き、
//! 問いの後に器が RunStopped で止めた走行は段を変えず、段の理由を `QUESTION_STOPPED` と about の字にする。
//! 節の組は歯の中で組む（fixture の file は pquest_fixture_card だけが読む）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::common::{NOW, cards};
use serde_json::{Value, json};
use tsuzuri_contract::board::{NextMove, PipelineCard, Reading, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::seat::SeatState;
use tsuzuri_contract::stats::CheckResult;
use tsuzuri_contract::surface::SeatRole;
use tsuzuri_core::account::host::HostTexts;
use tsuzuri_core::account::project::{ProjectTexts, session_lines};
use tsuzuri_core::next_step::next_step;
use tsuzuri_core::pipeline::{QUESTION_STOPPED, board, stage_of};

fn bead(n: u32, status: &str) -> Value {
    let mut b = json!({"id": format!("qa.{n}"), "title": format!("t{n}"), "status": status, "issue_type": "task"});
    if status == "closed" {
        b["closed_at"] = json!("2026-09-27T11:05:00Z");
        b["close_reason"] = json!("withdrawn");
    }
    b
}

/// 節の台帳の一覧（qa.1 から qa.8・qa.6 だけ閉じている）。
fn ledger() -> String {
    Value::Array(
        (1..=8)
            .map(|n| bead(n, if n == 6 { "closed" } else { "open" }))
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

fn raised(t: &str, run: &str) -> String {
    ev(t, "QuestionRaised", run, None, Some("問いの字"))
}

fn stage(t: &str, run: &str, s: &str, detail: Option<&str>) -> String {
    ev(t, "RunStage", run, Some(s), detail)
}

fn stopped(t: &str, run: &str) -> String {
    ev(t, "RunStopped", run, Some("Stopped"), None)
}

/// 節の event log（8 つの走行・27 行）。
fn events() -> Vec<String> {
    let r = |n: u32, t: &str| format!("qa.{n}-20260927T{}00Z", t.replace(':', ""));
    let (q1, q2, q3, q4) = (r(1, "10:00"), r(2, "10:10"), r(3, "10:30"), r(4, "10:40"));
    let (q5, q6, q7, q8) = (r(5, "10:50"), r(6, "11:00"), r(7, "11:10"), r(8, "11:20"));
    vec![
        created("10:00", &q1),
        raised("10:05", &q1),
        stage("10:05", &q1, "Questioned", Some("about:write-set")),
        created("10:10", &q2),
        raised("10:15", &q2),
        stage("10:15", &q2, "Questioned", Some("about:design")),
        stopped("10:20", &q2),
        created("10:30", &q3),
        stage("10:31", &q3, "Questioned", Some("contract:refreshed")),
        stopped("10:32", &q3),
        created("10:40", &q4),
        raised("10:41", &q4),
        stopped("10:42", &q4),
        created("10:50", &q5),
        stage("10:51", &q5, "Questioned", Some("about:verify")),
        stopped("10:52", &q5),
        stage("10:53", &q5, "Spawned", Some("base:0a,account:acct-1")),
        created("11:00", &q6),
        stage("11:01", &q6, "Questioned", Some("about:done")),
        stopped("11:02", &q6),
        created("11:10", &q7),
        stage("11:11", &q7, "Spawned", None),
        stopped("11:12", &q7),
        created("11:20", &q8),
        stage("11:21", &q8, "Questioned", Some("about:write-set")),
        stopped("11:22", &q8),
        stage("11:22", &q8, "Stopped", Some("retired")),
    ]
}

fn card(n: u32, stage: Stage, reason: Option<&str>, account: Option<&str>, elapsed: u64) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(format!("qa.{n}").as_str()).expect("bead の id"),
        runs: 1,
        stage,
        reason: reason.map(str::to_string),
        account: account.map(str::to_string),
        since: Some(NOW - elapsed),
        ci: None,
    }
}

/// (1) stage_of の表は RunStage の段 Questioned を detail を見ずに段 Questioned・理由無しにし、
/// RunStopped と小文字の段と段の名の無い event は表に無い。
#[test]
fn pquest_table_arm() {
    assert_eq!(QUESTION_STOPPED, "質問の後に止めた");
    for detail in ["about:write-set", "", "contract:refreshed"] {
        assert_eq!(
            stage_of("RunStage", Some("Questioned"), detail),
            Some((Stage::Questioned, None)),
            "{detail}"
        );
    }
    assert_eq!(
        stage_of("QuestionRaised", None, "問いの字"),
        Some((Stage::Questioned, None))
    );
    for (kind, s, detail) in [
        ("RunStopped", Some("Stopped"), ""),
        ("RunStage", Some("questioned"), "about:write-set"),
        ("RunStage", None, "about:write-set"),
    ] {
        assert_eq!(stage_of(kind, s, detail), None, "{kind} {s:?} {detail}");
    }
}

/// (2) 節の組の板は節の表の 8 枚と unmapped 0 で、台帳の字が空の板は qa.6 だけが段 Questioned。
#[test]
fn pquest_board_cards() {
    let events = events();
    assert_eq!(events.len(), 27);
    let events = events.join("\n");
    let head = |qa6: PipelineCard| {
        vec![
            card(1, Stage::Questioned, None, None, 6900),
            card(2, Stage::Questioned, Some("質問の後に止めた（design）"), None, 6300),
            card(3, Stage::Questioned, Some(QUESTION_STOPPED), None, 5340),
            card(4, Stage::Questioned, Some(QUESTION_STOPPED), None, 4740),
            card(5, Stage::Running, None, Some("acct-1"), 4020),
            qa6,
            card(7, Stage::Running, None, None, 2940),
            card(8, Stage::Questioned, Some("質問の後に止めた（write-set）"), None, 2340),
        ]
    };
    let b = board(&ledger(), &events, NOW);
    assert_eq!(
        cards(&b),
        head(card(6, Stage::Landed, Some("closed:withdrawn"), None, 3540)).as_slice()
    );
    assert_eq!(b.unmapped, 0);

    let b = board("", &events, NOW);
    assert_eq!(
        cards(&b),
        head(card(
            6,
            Stage::Questioned,
            Some(&format!("{QUESTION_STOPPED}（done）")),
            None,
            3540
        ))
        .as_slice()
    );
    assert_eq!(b.unmapped, 0);
}

/// (3) 問いの走行の札は止まっている走行に数えない。
#[test]
fn pquest_next_step_not_stalled() {
    let step = next_step(&ledger(), &events().join("\n"), NOW);
    let stalled = step
        .checks
        .iter()
        .find(|c| c.kind == NextMove::StalledRun)
        .expect("止まっている走行の結果");
    assert_eq!((stalled.result, stalled.count), (CheckResult::Miss, 0));
    assert_ne!(step.lead, NextMove::StalledRun);
}

/// (4) 問いを上げて止められる前の席の立った run の session の行は段 Questioned・状態 Wait。
#[test]
fn pquest_session_line_questioned() {
    let (s2, s1) = ("sq.2-20260927T114000Z", "sq.1-20260927T115000Z");
    let events = [
        created("11:40", s2),
        stage("11:41", s2, "Questioned", Some("about:design")),
        stopped("11:42", s2),
        created("11:50", s1),
        ev("11:50", "SeatSpawned", s1, None, None),
        raised("11:55", s1),
        stage("11:55", s1, "Questioned", Some("about:write-set")),
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
            None,
            SeatState::Wait,
            Some(Stage::Questioned),
            Some(1_790_510_100),
            Reading::Unknown,
        )]
    );
}

/// (5) fixture の RunStage の段 Questioned の走行は fx-pl.16 だけで、期待の札は段 Questioned・理由は write-set を添えた字。
#[test]
fn pquest_fixture_card() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/pipeline/pipeline.json");
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("fixture を読む"))
        .expect("fixture の JSON");
    let beads: Vec<&str> = v["events"]
        .as_array()
        .expect("events")
        .iter()
        .filter(|e| e["kind"] == "RunStage" && e["stage"] == "Questioned")
        .filter_map(|e| e["bead"].as_str())
        .collect();
    assert_eq!(beads, ["fx-pl.16"]);
    let want: Vec<PipelineCard> = serde_json::from_value(v["cards"].clone()).expect("期待の札");
    let c = want
        .iter()
        .find(|c| c.contract.as_str() == "fx-pl.16")
        .expect("fx-pl.16 の札");
    assert_eq!(c.stage, Stage::Questioned);
    assert_eq!(c.reason, Some(format!("{QUESTION_STOPPED}（write-set）")));
    assert_eq!(v["unmapped"], 1);

    let lines = |key: &str| {
        v[key]
            .as_array()
            .expect(key)
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
    };
    let got = board(
        &v["ledger"].to_string(),
        &lines("events").join("\n"),
        v["now"].as_u64().expect("now"),
    );
    assert_eq!(got.board.cards, Reading::Known(want));
    assert_eq!(got.unmapped, 1);
}

/// 着地済みの verify の filter の語を畳んだ 118 語と、後の行 g-gz・e-cache・e-sess-closed の接頭辞。
const FILTERS: [&str; 121] = [
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
    "ecache_",
    "sclosed_",
];

/// (7) この file の歯の名はどれも pquest_ で始まり、残りの字は filter の語を含まない。
#[test]
fn pquest_own_names_clean() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/teeth2/pquest.rs");
    let text = std::fs::read_to_string(&path).expect("tests/teeth2/pquest.rs を読む");
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
    assert!(names.len() >= 6, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("pquest_")
            .unwrap_or_else(|| panic!("{name} が pquest_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
