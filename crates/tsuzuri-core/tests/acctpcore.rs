//! account board の project の側の部分と組み立ての歯（便 e-acct-proj・接頭辞 acctpcore_）。
//! fixture: tests/fixtures/account/acct-inputs.json（host の側の字）・acct-doc.json（組み立ての部分）。どちらも読むだけ。
//! project の側の字（event log・状態の記録・台帳・合図の健康の行の器の欄）は歯の中で組む。今は 2026-09-27T12:00:00Z。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::Deserialize;
use serde_json::json;
use tsuzuri_contract::account::{
    AccountDoc, AccountRow, GroupCard, MoveRow, RunCounts, SessionLine,
};
use tsuzuri_contract::board::{Reading, Stage};
use tsuzuri_contract::seat::{SeatCard, SeatState};
use tsuzuri_contract::surface::SeatRole;
use tsuzuri_core::account::host::{CAP_ROWS, HostTexts};
use tsuzuri_core::account::project::{
    Parsed, ParsedMap, ProjectTexts, assemble, doc, doc_with, project_rows, run_counts,
    session_lines,
};
use tsuzuri_core::ledger::stats::stats;
use tsuzuri_core::next_step::next_step_seat;
use tsuzuri_core::seat::{SeatTexts, card};

const INPUTS: &str = "tests/fixtures/account/acct-inputs.json";
const DOC: &str = "tests/fixtures/account/acct-doc.json";

/// 今（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

#[derive(Deserialize)]
struct Inputs {
    texts: HostTexts,
    accounts: Reading<Vec<AccountRow>>,
    groups: Reading<Vec<GroupCard>>,
    moves: Reading<Vec<MoveRow>>,
}

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

fn inputs() -> Inputs {
    serde_json::from_str(&read(INPUTS)).expect("fixture の形")
}

fn known<T: std::fmt::Debug>(r: Reading<T>) -> T {
    match r {
        Reading::Known(v) => v,
        Reading::Unknown => panic!("Unknown"),
    }
}

/// 節の run と session の歯の組（bead 11 本）。欄は種類・bead・run の時刻の札・ts・stage・detail で、
/// ts は日の無い `HH:MM` なら 2026-09-27 の UTC。run の id は `<bead>-2026<札>`。
const EVENTS: &[(&str, &str, &str, &str, &str, &str)] = &[
    // fx-1: RunCreated だけで wait。
    ("RunCreated", "fx-1", "0927T090000Z", "09:00", "Intake", ""),
    // fx-2: Spawned で run・席が立っていて生きている。
    ("RunCreated", "fx-2", "0927T090100Z", "09:01", "Intake", ""),
    ("RunStage", "fx-2", "0927T090100Z", "09:02", "Spawned", ""),
    ("SeatSpawned", "fx-2", "0927T090100Z", "09:02", "", ""),
    // fx-3: Gated で run・最後の event が 60 分以内で生きている。
    ("RunCreated", "fx-3", "0927T112000Z", "11:20", "Intake", ""),
    ("SeatSpawned", "fx-3", "0927T112000Z", "11:21", "", ""),
    (
        "RunStage",
        "fx-3",
        "0927T112000Z",
        "11:30",
        "Gated",
        "verdict:PASS,account:acct-2",
    ),
    ("SeatStopped", "fx-3", "0927T112000Z", "11:31", "", ""),
    // fx-4: 落ちた審査で stop・最後の event が 60 分より前。
    ("RunCreated", "fx-4", "0927T090500Z", "09:05", "Intake", ""),
    ("SeatSpawned", "fx-4", "0927T090500Z", "09:05", "", ""),
    (
        "RunStage",
        "fx-4",
        "0927T090500Z",
        "09:06",
        "Reviewed",
        "verdict:FAIL",
    ),
    ("SeatStopped", "fx-4", "0927T090500Z", "09:07", "", ""),
    // fx-5: 問いで stop。
    ("RunCreated", "fx-5", "0927T090700Z", "09:07", "Intake", ""),
    ("QuestionRaised", "fx-5", "0927T090700Z", "09:08", "", ""),
    // fx-6: 今日の着地で land・終わった run。
    ("RunCreated", "fx-6", "0927T090900Z", "09:09", "Intake", ""),
    ("RunDone", "fx-6", "0927T090900Z", "09:10", "Landed", ""),
    // fx-7: 昨日の着地は数えない。
    (
        "RunCreated",
        "fx-7",
        "0926T090000Z",
        "2026-09-26T09:00:00Z",
        "Intake",
        "",
    ),
    (
        "RunDone",
        "fx-7",
        "0926T090000Z",
        "2026-09-26T09:30:00Z",
        "Landed",
        "",
    ),
    // fx-8: 古い run の着地と、新しい run の Spawned（新しい方だけを数えて run）。
    ("RunCreated", "fx-8", "0927T080000Z", "08:00", "Intake", ""),
    ("RunDone", "fx-8", "0927T080000Z", "08:30", "Landed", ""),
    ("RunCreated", "fx-8", "0927T091100Z", "09:11", "Intake", ""),
    ("RunStage", "fx-8", "0927T091100Z", "09:12", "Spawned", ""),
    // fx-9: RunCreated が今より後で数えない。
    ("RunCreated", "fx-9", "0927T130000Z", "13:00", "Intake", ""),
    // fx-10: RunStopped で stop・終わった run。
    ("RunCreated", "fx-10", "0927T091900Z", "09:19", "Intake", ""),
    (
        "RunStopped",
        "fx-10",
        "0927T091900Z",
        "09:20",
        "Stopped",
        "",
    ),
    // fx-11: 通った審査で wait。
    ("RunCreated", "fx-11", "0927T091400Z", "09:14", "Intake", ""),
    (
        "RunStage",
        "fx-11",
        "0927T091400Z",
        "09:15",
        "Reviewed",
        "verdict:PASS",
    ),
];

/// 歯の組の event log の字（1 行 1 件）。
fn events() -> String {
    EVENTS
        .iter()
        .map(|&(kind, bead, stamp, ts, stage, detail)| {
            let ts = if ts.contains('T') {
                ts.to_string()
            } else {
                format!("2026-09-27T{ts}:00Z")
            };
            json!({
                "kind": kind,
                "run": format!("{bead}-2026{stamp}"),
                "bead": bead,
                "ts": ts,
                "stage": stage,
                "detail": detail,
            })
            .to_string()
                + "\n"
        })
        .collect()
}

/// 台帳の一覧（bd の出力の形）。
fn ledger() -> String {
    json!([
        {"id": "fx-4", "title": "t4", "issue_type": "task", "status": "open", "created_at": "2026-09-20T00:00:00Z"},
        {"id": "fx-5", "title": "t5", "issue_type": "task", "status": "open", "created_at": "2026-09-21T00:00:00Z"},
        {"id": "fx-6", "title": "t6", "issue_type": "task", "status": "closed", "created_at": "2026-09-22T00:00:00Z", "closed_at": "2026-09-27T09:10:00Z"}
    ])
    .to_string()
}

fn proj_a() -> ProjectTexts {
    ProjectTexts {
        state_dir_known: true,
        tick_status: Some("seat tick status: target=proj-a:0.1 healthy=yes heartbeat=on\n".into()),
        state_log: Some(
            "{\"state\":\"idle\",\"ts\":1790500000}\n{\"state\":\"busy\",\"ts\":1790505060}\n"
                .into(),
        ),
        tick_last: None,
        events: Some(events().into()),
        ledger: Some(ledger().into()),
    }
}

fn proj_b() -> ProjectTexts {
    ProjectTexts {
        state_dir_known: true,
        tick_status: Some("seat tick status: target=proj-b:0.1 healthy=no heartbeat=off\n".into()),
        state_log: Some("{\"state\":\"idle\",\"ts\":1790508900}\n".into()),
        tick_last: None,
        events: None,
        ledger: None,
    }
}

/// anchor → project の字（proj-c は state dir が引けない）。
fn projects() -> BTreeMap<String, ProjectTexts> {
    BTreeMap::from([
        ("/work/proj-a".to_string(), proj_a()),
        ("/work/proj-b".to_string(), proj_b()),
        (
            "/work/proj-c".to_string(),
            ProjectTexts {
                state_dir_known: false,
                ..ProjectTexts::default()
            },
        ),
    ])
}

/// 着地済みの席の card の関数に、その project の state dir の字を渡した値。
fn seat_card(
    host: &HostTexts,
    anchor: &str,
    target: &str,
    p: &ProjectTexts,
    records: &[&str],
    now: u64,
) -> SeatCard {
    let texts = SeatTexts {
        tick_status: p.tick_status.clone(),
        doctor: host.seat_doctors.get(anchor).cloned(),
        usage: host.usage.clone(),
        state_log: p.state_log.clone(),
        tick_last: p.tick_last.clone(),
        host_toml: host.host_toml.clone(),
        records: records.iter().map(|r| r.to_string()).collect(),
    };
    card(target, Some(anchor), &texts, now)
}

fn card_a(host: &HostTexts, now: u64) -> SeatCard {
    let records = [
        host.records["main.account"].as_str(),
        host.history["main.account.20260926T080000Z.1"].as_str(),
        host.history["main.account.20260927T100000Z.1"].as_str(),
    ];
    seat_card(host, "/work/proj-a", "proj-a:0.1", &proj_a(), &records, now)
}

fn card_b(host: &HostTexts, now: u64) -> SeatCard {
    let records = [
        host.history["aux.account.20260925T000000Z.1"].as_str(),
        host.history["aux.account.20260927T100000Z.1"].as_str(),
    ];
    seat_card(host, "/work/proj-b", "proj-b:0.1", &proj_b(), &records, now)
}

fn counts(wait: u32, run: u32, stop: u32, land: u32) -> RunCounts {
    RunCounts {
        wait,
        run,
        stop,
        land,
    }
}

#[test]
fn acctpcore_run_counts_four_columns() {
    assert_eq!(
        run_counts(Some(&events()), NOW),
        Reading::Known(counts(2, 3, 3, 1))
    );
    // event log の字が無いか読めなければ Unknown。
    assert_eq!(run_counts(None, NOW), Reading::Unknown);
    assert_eq!(run_counts(Some(""), NOW), Reading::Unknown);
    assert_eq!(run_counts(Some("not json\n"), NOW), Reading::Unknown);
    // 今が 13:00 に進めば fx-9 が wait に入り、fx-3 の後の event が無いので列は変わらない。
    assert_eq!(
        run_counts(Some(&events()), NOW + 3600),
        Reading::Known(counts(3, 3, 3, 1))
    );
    // 翌日の今では fx-6 の着地は数えない。
    assert_eq!(
        run_counts(Some(&events()), NOW + 86_400),
        Reading::Known(counts(3, 3, 3, 0))
    );
    // 今が 09:30 なら fx-3・fx-9 は無く、fx-10 と fx-11 は RunCreated が在る。
    assert_eq!(
        run_counts(Some(&events()), NOW - 9_000),
        Reading::Known(counts(2, 2, 3, 1))
    );
}

#[test]
fn acctpcore_project_rows_seat_runs_and_order() {
    let host = inputs().texts;
    let rows = project_rows(&host, &projects(), NOW);
    let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, ["proj-a", "proj-c", "proj-b"]);
    let groups: Vec<Option<&str>> = rows.iter().map(|r| r.group.as_deref()).collect();
    assert_eq!(groups, [Some("main"), Some("main"), Some("aux")]);
    assert!(rows.iter().all(|r| r.board.is_none()));
    // 席の card は着地済みの関数の値と 1 字も違わない。
    let a = &rows[0];
    let want_a = card_a(&host, NOW);
    assert_eq!(want_a.state, SeatState::Run);
    assert_eq!(
        serde_json::to_string(&a.seat).expect("電文"),
        serde_json::to_string(&Reading::Known(want_a)).expect("電文")
    );
    assert_eq!(a.runs, Reading::Known(counts(2, 3, 3, 1)));
    assert!(a.state_dir_known);
    let b = &rows[2];
    assert_eq!(b.seat, Reading::Known(card_b(&host, NOW)));
    assert_eq!(b.runs, Reading::Unknown);
    // state dir の引けない project は席・run・台帳・次の一手が Unknown で、退避の終わる時刻は無し。
    let c = &rows[1];
    assert!(!c.state_dir_known);
    assert_eq!(
        (&c.seat, &c.runs, &c.ledger, &c.next, c.move_until),
        (
            &Reading::Unknown,
            &Reading::Unknown,
            &Reading::Unknown,
            &Reading::Unknown,
            None
        )
    );
    // 席の行が無ければ席は Unknown（ほかの部分は組む）。
    let mut no_seat = host.clone();
    no_seat.seat_doctors.remove("/work/proj-a");
    let a = &project_rows(&no_seat, &projects(), NOW)[0];
    assert_eq!(a.seat, Reading::Unknown);
    assert_eq!(a.runs, Reading::Known(counts(2, 3, 3, 1)));
    // 表に無い anchor は state dir が引けない扱い・群の宣言が無ければ project は無い。
    let mut only_a = projects();
    only_a.remove("/work/proj-b");
    assert!(!project_rows(&host, &only_a, NOW)[2].state_dir_known);
    let mut no_host = host.clone();
    no_host.host_toml = None;
    assert!(project_rows(&no_host, &projects(), NOW).is_empty());
}

#[test]
fn acctpcore_ledger_and_next_step_as_landed() {
    let host = inputs().texts;
    let rows = project_rows(&host, &projects(), NOW);
    let a = &rows[0];
    let want = stats(&ledger(), NOW);
    assert!(matches!(want, Reading::Known(_)));
    assert_eq!(a.ledger, want);
    let card = card_a(&host, NOW);
    assert_eq!(
        a.next,
        Reading::Known(next_step_seat(&ledger(), &events(), NOW, Some(&card)))
    );
    // 台帳の字が無い project は台帳と次の一手が Unknown。
    let b = &rows[2];
    assert_eq!((&b.ledger, &b.next), (&Reading::Unknown, &Reading::Unknown));
    // event log の字が無ければ空の字として渡す・席の card が無ければ無しとして渡す。
    let mut no_events = projects();
    no_events.get_mut("/work/proj-a").expect("proj-a").events = None;
    let mut no_seat = host.clone();
    no_seat.seat_doctors.remove("/work/proj-a");
    let a = &project_rows(&no_seat, &no_events, NOW)[0];
    assert_eq!(
        a.next,
        Reading::Known(next_step_seat(&ledger(), "", NOW, None))
    );
}

#[test]
fn acctpcore_move_left_seconds() {
    let host = inputs().texts;
    // proj-a の合図の健康の行の末に器の欄を足した project の字。
    let with = |tail: &str| {
        let mut ps = projects();
        let a = ps.get_mut("/work/proj-a").expect("proj-a");
        let tick = a.tick_status.take().expect("合図の健康の出力");
        a.tick_status = Some(format!("{} {tail}\n", tick.trim_end()));
        ps
    };
    let left = |host: &HostTexts, ps: &BTreeMap<String, ProjectTexts>, now| {
        project_rows(host, ps, now)[0].move_until
    };
    // 器の grace_left= の秒に組んだ今を足した終わる時刻を写す（0 は今そのもの）。
    let ps = with("reopens=- move=acct-2 grace_left=1101");
    for now in [NOW, NOW + 60, NOW + 5_000] {
        assert_eq!(left(&host, &ps, now), Some(now + 1101));
    }
    assert_eq!(
        left(&host, &with("move=acct-2 grace_left=0"), NOW),
        Some(NOW)
    );
    // 群の今の記録が無くても同じ値（群の記録から計算しない）。
    let mut no_record = host.clone();
    no_record.records.remove("main.account");
    assert_eq!(left(&no_record, &ps, NOW), Some(NOW + 1101));
    // move= が口座でないか grace_left= が秒でなければ無し・欄が無ければ無し。
    for tail in [
        "move=- grace_left=-",
        "move=- grace_left=120",
        "move=acct-2 grace_left=-",
        "move=unreadable grace_left=unreadable",
        "move=acct-2 grace_left=soon",
    ] {
        assert_eq!(left(&host, &with(tail), NOW), None, "{tail}");
    }
    assert_eq!(left(&host, &projects(), NOW), None);
    // 席の行が無ければ（card が Unknown）無し。
    let mut no_seat = host.clone();
    no_seat.seat_doctors.remove("/work/proj-a");
    assert_eq!(left(&no_seat, &ps, NOW), None);
}

#[test]
fn acctpcore_session_lines_match_expected() {
    let host = inputs().texts;
    let a = card_a(&host, NOW);
    let b = card_b(&host, NOW);
    let orch = |project: &str, c: &SeatCard| SessionLine {
        project: project.into(),
        role: SeatRole::Orchestrator,
        name: c.target.clone(),
        account: c.account.clone(),
        state: c.state,
        stage: None,
        since: c.since,
        spans: c.spans.clone(),
    };
    let pipe = |name: &str, account: Option<&str>, stage, since, state| SessionLine {
        project: "proj-a".into(),
        role: SeatRole::Pipeline,
        name: name.into(),
        account: account.map(str::to_string),
        state,
        stage: Some(stage),
        since: Some(since),
        spans: Reading::Unknown,
    };
    let seatless = SessionLine {
        project: "proj-c".into(),
        role: SeatRole::Orchestrator,
        name: String::new(),
        account: None,
        state: SeatState::Unknown,
        stage: None,
        since: None,
        spans: Reading::Unknown,
    };
    let want = vec![
        orch("proj-a", &a),
        pipe(
            "fx-2-20260927T090100Z",
            None,
            Stage::Running,
            1_790_499_720,
            SeatState::Run,
        ),
        pipe(
            "fx-3-20260927T112000Z",
            Some("acct-2"),
            Stage::Gated,
            1_790_508_600,
            SeatState::Wait,
        ),
        seatless,
        orch("proj-b", &b),
    ];
    assert_eq!(session_lines(&host, &projects(), NOW), want);
    // 13:00 の今では fx-3 は最後の event から 60 分より後で生きていない（fx-2 は席が立っているので残り、
    // fx-9 は RunCreated が今以下になって最後の event が 60 分以内）。
    let later: Vec<String> = session_lines(&host, &projects(), NOW + 3600)
        .into_iter()
        .filter(|s| s.role == SeatRole::Pipeline)
        .map(|s| s.name)
        .collect();
    assert_eq!(later, ["fx-2-20260927T090100Z", "fx-9-20260927T130000Z"]);
}

#[test]
fn acctpcore_assemble_equals_doc_fixture() {
    let want: AccountDoc = serde_json::from_str(&read(DOC)).expect("fixture の形");
    let got = assemble(
        want.accounts.clone(),
        want.groups.clone(),
        want.moves.clone(),
        want.projects.clone(),
        want.sessions.clone(),
    );
    // assemble は閾値と知らせを読まない（CAP_ROWS の順の 3 つで cap は Unknown・知らせは Unknown）。
    let rows: Vec<(&str, &str)> = got
        .caps
        .iter()
        .map(|c| (c.window.as_str(), c.rule.as_str()))
        .collect();
    assert_eq!(rows, CAP_ROWS);
    assert!(got.caps.iter().all(|c| c.cap == Reading::Unknown));
    assert_eq!(got.notices, Reading::Unknown);
    let filled = AccountDoc {
        caps: want.caps.clone(),
        notices: want.notices.clone(),
        ..got.clone()
    };
    assert_eq!(filled, want);
    assert!(got.dormant.is_empty());
}

#[test]
fn acctpcore_doc_host_parts_from_inputs() {
    let i = inputs();
    let ps = projects();
    let d = doc(&i.texts, &ps, NOW);
    // 時点は今ではなく材料の時刻（今が動いても変わらない）。
    assert!(d.at <= NOW);
    assert_eq!(d.at, doc(&i.texts, &ps, NOW + 3600).at);
    assert_eq!(d.accounts, i.accounts);
    assert_eq!(d.groups, i.groups);
    assert_eq!(d.moves, i.moves);
    assert_eq!(d.projects, project_rows(&i.texts, &ps, NOW));
    assert_eq!(d.sessions, session_lines(&i.texts, &ps, NOW));
    assert!(d.dormant.is_empty());
    // 群の宣言の字が無ければ 3 つの列が Unknown で、project と session は無い。
    let mut no_host = i.texts.clone();
    no_host.host_toml = None;
    let d = doc(&no_host, &ps, NOW);
    assert_eq!(
        (&d.accounts, &d.groups, &d.moves),
        (&Reading::Unknown, &Reading::Unknown, &Reading::Unknown)
    );
    assert!(d.projects.is_empty() && d.sessions.is_empty());
    assert!(!known(i.moves).is_empty());
}

/// project ごとの `Parsed::of` の値の表。
fn parsed_of(ps: &BTreeMap<String, ProjectTexts>) -> ParsedMap {
    ps.iter()
        .map(|(a, t)| (a.clone(), Arc::new(Parsed::of(t))))
        .collect()
}

#[test]
fn acctpcore_doc_with_parsed_equals_doc() {
    let i = inputs();
    let ps = projects();
    let parsed = parsed_of(&ps);
    for now in [NOW, NOW + 3600] {
        assert_eq!(
            doc_with(&i.texts, &ps, &parsed, now),
            doc(&i.texts, &ps, now)
        );
    }
    // 1 つめの project の event log を None にした字では、読み直した値で doc と同じ電文、前の字の値のままでは違う電文。
    let mut none = ps.clone();
    none.get_mut("/work/proj-a").expect("proj-a").events = None;
    let want = doc(&i.texts, &none, NOW);
    assert_eq!(doc_with(&i.texts, &none, &parsed_of(&none), NOW), want);
    assert_ne!(doc_with(&i.texts, &none, &parsed, NOW), want);
}
