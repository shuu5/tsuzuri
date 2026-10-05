//! 行 e-sess-closed の歯（中核）: account board の session の表は、台帳で今閉じている bead の run の行を出さない。
//! 節の組は歯の中で組む（fixture の file は使わない）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::common::{NOW, host};
use serde_json::{Value, json};
use tsuzuri_contract::account::SessionLine;
use tsuzuri_contract::board::{Reading, Stage};
use tsuzuri_contract::seat::SeatState;
use tsuzuri_contract::surface::SeatRole;
use tsuzuri_core::account::project::{ProjectTexts, session_lines};

/// event の 1 行（run は bead とハイフンと RunCreated の時刻の札）。
fn ev(ts: &str, kind: &str, run: &str, stage: Option<&str>, detail: Option<&str>) -> String {
    let (bead, _) = run.rsplit_once('-').expect("run の id");
    let mut e = json!({"schema": 1, "ts": ts, "kind": kind, "run": run, "bead": bead});
    if let Some(s) = stage {
        e["stage"] = json!(s);
    }
    if let Some(d) = detail {
        e["detail"] = json!(d);
    }
    e.to_string()
}

const SS1: &str = "ss.1-20260927T110000Z";
const SS2: &str = "ss.2-20260927T111000Z";
const SS3: &str = "ss.3-20260927T112000Z";
const SS4: &str = "ss.4-20260927T114000Z";
const SS5: &str = "ss.5-20260927T114500Z";

/// 節の event log（11 行・どの run も終わっておらず生きている）。
fn events() -> String {
    let d = |t: &str| format!("2026-09-27T{t}:00Z");
    let created = |t: &str, run: &str| ev(&d(t), "RunCreated", run, Some("Intake"), None);
    let stage = |t: &str, run: &str, s: &str, detail: &str| {
        ev(&d(t), "RunStage", run, Some(s), Some(detail))
    };
    let spawned = |t: &str, run: &str| ev(&d(t), "SeatSpawned", run, None, None);
    [
        created("11:00", SS1),
        stage("11:01", SS1, "Spawned", "base:0a,account:acct-1"),
        spawned("11:01", SS1),
        created("11:10", SS2),
        stage("11:11", SS2, "Reviewed", "verdict:FAIL"),
        created("11:20", SS3),
        stage("11:30", SS3, "Gated", "verdict:FAIL"),
        created("11:40", SS4),
        created("11:45", SS5),
        stage("11:46", SS5, "Spawned", "base:0b,account:acct-2"),
        spawned("11:46", SS5),
    ]
    .join("\n")
}

fn bead(id: &str, status: &str, closed_at: Option<&str>) -> Value {
    let mut b = json!({"id": id, "title": id, "status": status, "issue_type": "task"});
    if let Some(at) = closed_at {
        b["closed_at"] = json!(at);
    }
    b
}

/// 節の台帳の一覧（ss.4 は台帳に無い）。
fn ledger() -> String {
    Value::Array(vec![
        bead("ss.1", "closed", None),
        bead("ss.2", "open", None),
        bead("ss.3", "closed", Some("2026-09-27T11:35:00Z")),
        bead("ss.5", "in_progress", None),
    ])
    .to_string()
}

fn texts(ledger: Option<String>) -> BTreeMap<String, ProjectTexts> {
    BTreeMap::from([(
        "/w/p".to_string(),
        ProjectTexts {
            state_dir_known: true,
            events: Some(events().into()),
            ledger: ledger.map(Into::into),
            ..ProjectTexts::default()
        },
    )])
}

fn orchestrator() -> SessionLine {
    SessionLine {
        project: "p".to_string(),
        role: SeatRole::Orchestrator,
        name: String::new(),
        account: None,
        state: SeatState::Unknown,
        stage: None,
        since: None,
        spans: Reading::Unknown,
    }
}

fn pipeline(
    run: &str,
    account: Option<&str>,
    state: SeatState,
    stage: Stage,
    since: u64,
) -> SessionLine {
    SessionLine {
        project: "p".to_string(),
        role: SeatRole::Pipeline,
        name: run.to_string(),
        account: account.map(str::to_string),
        state,
        stage: Some(stage),
        since: Some(since),
        spans: Reading::Unknown,
    }
}

/// (1)(2) 台帳で閉じた bead の run の行は出さず、台帳の字が無いか読めなければ今のまま全部出す。
#[test]
fn sclosed_drops_closed_runs() {
    let ss1 = pipeline(SS1, Some("acct-1"), SeatState::Run, Stage::Running, 1_790_506_860);
    let ss2 = pipeline(SS2, None, SeatState::Wait, Stage::Failed, 1_790_507_460);
    let ss3 = pipeline(SS3, None, SeatState::Wait, Stage::Failed, 1_790_508_600);
    let ss4 = pipeline(SS4, None, SeatState::Wait, Stage::Running, 1_790_509_200);
    let ss5 = pipeline(SS5, Some("acct-2"), SeatState::Run, Stage::Running, 1_790_509_560);

    let with = session_lines(&host(), &texts(Some(ledger())), NOW);
    assert_eq!(
        with,
        vec![orchestrator(), ss2.clone(), ss4.clone(), ss5.clone()]
    );

    let plain = vec![orchestrator(), ss1, ss2, ss3, ss4, ss5];
    for led in [None, Some(String::new()), Some("not json".to_string())] {
        assert_eq!(
            session_lines(&host(), &texts(led.clone()), NOW),
            plain,
            "{led:?}"
        );
    }
}

/// 着地済みの行の verify の filter の語（畳んだ 118 語）と、後の行と同じノートの行の接頭辞。
const FILTERS: &[&str] = &[
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
    "pquest_",
];

/// (4) この file の歯の名はどれも sclosed_ で始まり、残りの字は filter の語を含まない。
#[test]
fn sclosed_own_names_clean() {
    assert_eq!(FILTERS.len(), 121);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/teeth2/sclosed.rs");
    let text = std::fs::read_to_string(&path).expect("tests/teeth2/sclosed.rs を読む");
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
    assert!(names.len() >= 2, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("sclosed_")
            .unwrap_or_else(|| panic!("{name} が sclosed_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
