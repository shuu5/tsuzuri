//! project の側の部分（project の行・session の行・電文の組み立て・設計ノート surface-base 便 e-acct-proj）。
//! 入力は host の側の字（`HostTexts`）と、anchor ごとの project の字（`ProjectTexts`）と今の時刻。
//! 退避までの残り秒は席の card の器の欄 grace_left の写しで、tsuzuri は計算しない（規則の行 R-22・行 c-grace-acct）。
//! project の宣言の順は群の宣言の順で、群の中は anchors の配列の順（同じ anchor は最初の群だけ）。
//! 席の card・台帳の指標・次の一手は着地済みの関数の値をそのまま写す。run の 4 列の分け方と
//! 生きている run の境（`ALIVE_S`）は見本の acct.js の runColsAt と runAlive の決め方（設計席の承認で置く値）。
//! 読めない字の決まり（要件 NFR2）: state dir が引けない project は席と run と台帳と次の一手が「まだ分からない」、
//! event log の字が無いか読めなければ run の 4 列が、台帳の字が無ければ台帳と次の一手が「まだ分からない」。
//! 休止中の席の境（`DORMANT_S`）は規則の行 R-28 の 12 時間（見本 mock v3 の承認・行 c-dormant）。

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use serde_json::Value;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::account::{
    AccountDoc, AccountRow, DormantSeat, GroupCard, MoveRow, ProjectRow, RunCounts, SessionLine,
};
use tsuzuri_contract::board::{Reading, Stage};
use tsuzuri_contract::seat::{SeatCard, SeatState};
use tsuzuri_contract::surface::SeatRole;

use super::host::{self, HostTexts, ORCHESTRATOR, RECORD_KIND, declaration};
use super::{field, project_name, same_path, value};
use crate::graph::build::{read_events, run_bead};
use crate::ledger::stats::stats;
use crate::ledger::{DAY, epoch_secs, read};
use crate::next_step::next_step_seat;
use crate::pipeline::{ACCOUNT_TAG, FAILED_VERDICTS, stage_of};
use crate::seat::{SeatTexts, card};

/// 生きている run の境（最後の event からの秒・見本の acct.js の runAlive）。
pub const ALIVE_S: u64 = 60 * 60;

/// 休止中の席の境（席の状態の記録の最後の読めた行からの秒・規則の行 R-28 の 12 時間）。
pub const DORMANT_S: u64 = 12 * 60 * 60;

/// run の段を決める event の種類（見本の acct.js の runColsAt）。
pub const RUN_STAGE_EVENTS: [&str; 5] = [
    "RunCreated",
    "RunStage",
    "RunDone",
    "RunStopped",
    "QuestionRaised",
];

/// 器が run を上限で止めた段の名（器の RunStage の段 RateLimited・器の印の写しで tsuzuri は判じない）。
pub const RATE_LIMITED: &str = "RateLimited";

/// 止まりの列に入る段の名。
const STOP_STAGES: [&str; 3] = ["Questioned", "Failed", "Stopped"];

/// 走行の列に入る段の名。
const RUN_STAGES: [&str; 3] = ["Spawned", "Implemented", "Gated"];

/// 着地の段の名。
const LANDED: &str = "Landed";

/// doctor の席の行の頭。
const SEAT_PREFIX: &str = "seat:";

/// project の側の材料の字（anchor ごとに 1 つ・無い字は None）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct ProjectTexts {
    /// state dir が引けたか。
    #[serde(default)]
    pub state_dir_known: bool,
    /// 合図の健康の出力（`seat tick status --state-dir <dir>`）。
    pub tick_status: Option<String>,
    /// 席の状態の記録（`state.jsonl`）。
    pub state_log: Option<String>,
    /// 席の合図の最後の判定（`tick-last`）。
    pub tick_last: Option<String>,
    /// 器の event log。
    pub events: Option<String>,
    /// 台帳の一覧。
    pub ledger: Option<String>,
}

/// 宣言の順の project（anchor の path と群の名）。
struct Declared {
    anchor: String,
    group: String,
}

/// 群の宣言の順の project（群の中は anchors の配列の順・同じ anchor は最初の群だけ・読めない anchors の群は数えない）。
fn declared(host: &HostTexts) -> Vec<Declared> {
    let mut out: Vec<Declared> = Vec::new();
    let Some(toml) = host.host_toml.as_deref() else {
        return out;
    };
    for g in declaration(toml).groups {
        for a in g.anchors.unwrap_or_default() {
            if !out.iter().any(|d| same_path(&d.anchor, &a)) {
                out.push(Declared {
                    anchor: a,
                    group: g.name.clone(),
                });
            }
        }
    }
    out
}

/// 表のうち、鍵が anchor と同じ path の最初の値。
fn by_anchor<'a, T>(map: &'a BTreeMap<String, T>, anchor: &str) -> Option<&'a T> {
    map.iter()
        .find(|(a, _)| same_path(a, anchor))
        .map(|(_, v)| v)
}

/// doctor の席の行のうち、役が orchestrator で anchor が同じ path の最初の行（頭を除いた字）。
fn orchestrator_line<'a>(doctor: &'a str, anchor: &str) -> Option<&'a str> {
    doctor
        .lines()
        .filter_map(|l| l.trim().strip_prefix(SEAT_PREFIX))
        .find(|l| {
            field(l, "role") == Some(ORCHESTRATOR)
                && field(l, "anchor").is_some_and(|a| same_path(a, anchor))
        })
}

/// 群の今の記録の file の名。
fn current_record(group: &str) -> String {
    format!("{group}.{RECORD_KIND}")
}

/// 群の記録の字の列（今の記録と、history の `<群>.account.` で始まる名の記録を名の順に）。
fn group_records(host: &HostTexts, group: &str) -> Vec<String> {
    let current = current_record(group);
    let prefix = format!("{current}.");
    host.records
        .get(&current)
        .into_iter()
        .chain(
            host.history
                .iter()
                .filter(|(name, _)| name.starts_with(&prefix))
                .map(|(_, text)| text),
        )
        .cloned()
        .collect()
}

/// その project の席の card（state dir が引けないか席の行が無ければ「まだ分からない」）。
fn seat(host: &HostTexts, d: &Declared, texts: &ProjectTexts, now: EpochSecs) -> Reading<SeatCard> {
    let doctor = by_anchor(&host.seat_doctors, &d.anchor);
    let line = doctor.and_then(|doc| orchestrator_line(doc, &d.anchor));
    let target = line.and_then(|l| value(l, "target"));
    match target {
        Some(target) if texts.state_dir_known => {
            let seat_texts = SeatTexts {
                tick_status: texts.tick_status.clone(),
                doctor: doctor.cloned(),
                usage: host.usage.clone(),
                state_log: texts.state_log.clone(),
                tick_last: texts.tick_last.clone(),
                host_toml: host.host_toml.clone(),
                records: group_records(host, &d.group),
            };
            Reading::Known(card(target, Some(&d.anchor), &seat_texts, now))
        }
        _ => Reading::Unknown,
    }
}

/// 退避までの残り秒。席の card の欄 move_to が口座で grace_left が秒のときだけその秒（器の字のまま・0 も写す・
/// ほかは None・起点は器の合図の at で tsuzuri は計算しない）。
fn move_left(card: &Reading<SeatCard>) -> Option<u64> {
    match card {
        Reading::Known(SeatCard {
            move_to: Reading::Known(Some(_)),
            grace_left: Reading::Known(Some(left)),
            ..
        }) => Some(*left),
        _ => None,
    }
}

fn text<'a>(event: &'a Value, key: &str) -> Option<&'a str> {
    event.get(key).and_then(Value::as_str)
}

/// event の ts（読めなければ None）。
fn event_ts(event: &Value) -> Option<EpochSecs> {
    text(event, "ts").and_then(epoch_secs)
}

/// 1 つの run の今までの event（ts が今以下のもの・log の順）。
struct Run<'a> {
    id: &'a str,
    events: Vec<&'a Value>,
}

impl<'a> Run<'a> {
    /// 段を決める最後の event。
    fn last_stage(&self) -> Option<&'a Value> {
        self.events
            .iter()
            .rev()
            .find(|e| RUN_STAGE_EVENTS.contains(&text(e, "kind").unwrap_or_default()))
            .copied()
    }

    fn count(&self, kind: &str) -> usize {
        self.events
            .iter()
            .filter(|e| text(e, "kind") == Some(kind))
            .count()
    }

    /// SeatSpawned の数が SeatStopped の数より多いか。
    fn seated(&self) -> bool {
        self.count("SeatSpawned") > self.count("SeatStopped")
    }

    /// RunDone か RunStopped を持つか。
    fn ended(&self) -> bool {
        self.events
            .iter()
            .any(|e| matches!(text(e, "kind"), Some("RunDone" | "RunStopped")))
    }

    /// 席が立っているか、最後の event が今から `ALIVE_S` 以内か。
    fn alive(&self, now: EpochSecs) -> bool {
        self.seated()
            || self
                .events
                .last()
                .and_then(|e| event_ts(e))
                .is_some_and(|t| now.saturating_sub(t) <= ALIVE_S)
    }

    /// detail の中の口座の札のうち最後のもの。
    fn account(&self) -> Option<String> {
        self.events
            .iter()
            .filter_map(|e| text(e, "detail"))
            .flat_map(|d| d.split([',', ' ']))
            .filter_map(|tok| tok.strip_prefix(ACCOUNT_TAG))
            .rfind(|a| !a.is_empty())
            .map(str::to_string)
    }
}

/// event log の run（RunCreated の ts が今以下のもの）を RunCreated の順に。各 run は ts が今以下の event を持つ。
/// RunCreated の無い run の event と、ts の読めない event は読み捨てる。
fn runs(events: &[Value], now: EpochSecs) -> Vec<Run<'_>> {
    let mut out: Vec<Run> = Vec::new();
    for e in events {
        let (Some(id), Some(ts)) = (text(e, "run"), event_ts(e)) else {
            continue;
        };
        if ts > now {
            continue;
        }
        match out.iter_mut().find(|r| r.id == id) {
            Some(run) => run.events.push(e),
            None if text(e, "kind") == Some("RunCreated") => out.push(Run {
                id,
                events: vec![e],
            }),
            None => {}
        }
    }
    out
}

/// run の bead（欄 bead か run の id の前半）。
fn bead_of<'a>(run: &Run<'a>) -> Option<&'a str> {
    text(run.events[0], "bead").or_else(|| run_bead(run.id))
}

/// 台帳で今閉じている bead の id（台帳の字が無いか読めなければ空・台帳に無い bead は入らない）。
fn closed_beads(ledger: Option<&str>, now: EpochSecs) -> BTreeSet<String> {
    ledger
        .and_then(read)
        .unwrap_or_default()
        .into_iter()
        .filter(|b| !b.is_open(now))
        .map(|b| b.id)
        .collect()
}

/// run の 4 列の数（bead ごとに RunCreated がいちばん新しい run の、今までの最後の段で分ける）。
/// 段が Landed なら今日（UTC）の着地だけ land、verdict が落ちか段が Questioned・Failed・Stopped なら stop、
/// 段が Spawned・Implemented・Gated なら run、ほかは wait。event log の字が無いか読めなければ「まだ分からない」。
pub fn run_counts(events: Option<&str>, now: EpochSecs) -> Reading<RunCounts> {
    run_counts_of(events, None, now)
}

/// run の 4 列の数（`run_counts` の決まりに、台帳で今閉じている bead の最新の run の段を Landed と読み替える決まりを足す・
/// 行 c-pipe-closed）。台帳の字が無いか読めなければ読み替えない。
pub fn run_counts_of(
    events: Option<&str>,
    ledger: Option<&str>,
    now: EpochSecs,
) -> Reading<RunCounts> {
    let Some(events) = events.and_then(read_events) else {
        return Reading::Unknown;
    };
    let closed = closed_beads(ledger, now);
    let all = runs(&events, now);
    let mut latest: BTreeMap<&str, &Run> = BTreeMap::new();
    for run in &all {
        if let Some(bead) = bead_of(run) {
            latest.insert(bead, run);
        }
    }
    let mut counts = RunCounts {
        wait: 0,
        run: 0,
        stop: 0,
        land: 0,
    };
    for (bead, run) in &latest {
        let Some(last) = run.last_stage() else {
            continue;
        };
        let stage = match text(last, "kind") {
            // 台帳で閉じた bead の run は着地の段と読み替える（verdict の落ちも見ない）。
            _ if closed.contains(*bead) => LANDED,
            Some("QuestionRaised") => "Questioned",
            _ => text(last, "stage").unwrap_or_default(),
        };
        let failed = FAILED_VERDICTS
            .iter()
            .any(|v| text(last, "detail").unwrap_or_default().starts_with(v));
        if stage == LANDED {
            if event_ts(last).is_some_and(|t| t / DAY == now / DAY) {
                counts.land += 1;
            }
        } else if failed || STOP_STAGES.contains(&stage) {
            counts.stop += 1;
        } else if RUN_STAGES.contains(&stage) {
            counts.run += 1;
        } else {
            counts.wait += 1;
        }
    }
    Reading::Known(counts)
}

/// 宣言の project と、その project の字（表に無い anchor は state dir が引けない扱い）。
fn with_texts<'a>(
    host: &HostTexts,
    projects: &'a BTreeMap<String, ProjectTexts>,
) -> Vec<(Declared, Option<&'a ProjectTexts>)> {
    declared(host)
        .into_iter()
        .map(|d| {
            let texts = by_anchor(projects, &d.anchor);
            (d, texts)
        })
        .collect()
}

/// project の行の列（project の宣言の順・project board の URL はこの便では無し）。
pub fn project_rows(
    host: &HostTexts,
    projects: &BTreeMap<String, ProjectTexts>,
    now: EpochSecs,
) -> Vec<ProjectRow> {
    let unknown = ProjectTexts::default();
    with_texts(host, projects)
        .into_iter()
        .map(|(d, texts)| {
            let texts = texts.unwrap_or(&unknown);
            let name = project_name(&d.anchor);
            if !texts.state_dir_known {
                return ProjectRow {
                    name,
                    group: Some(d.group),
                    state_dir_known: false,
                    seat: Reading::Unknown,
                    move_left_s: None,
                    runs: Reading::Unknown,
                    ledger: Reading::Unknown,
                    next: Reading::Unknown,
                    board: None,
                };
            }
            let seat = seat(host, &d, texts, now);
            let (ledger, next) = match texts.ledger.as_deref() {
                Some(ledger) => {
                    let card = match &seat {
                        Reading::Known(c) => Some(c),
                        Reading::Unknown => None,
                    };
                    let events = texts.events.as_deref().unwrap_or_default();
                    (
                        stats(ledger, now),
                        Reading::Known(next_step_seat(ledger, events, now, card)),
                    )
                }
                None => (Reading::Unknown, Reading::Unknown),
            };
            ProjectRow {
                name,
                move_left_s: move_left(&seat),
                group: Some(d.group),
                state_dir_known: true,
                seat,
                runs: run_counts_of(texts.events.as_deref(), texts.ledger.as_deref(), now),
                ledger,
                next,
                board: None,
            }
        })
        .collect()
}

/// 状態の記録の 1 行（読む欄だけ）。
#[derive(Deserialize)]
struct StateLine {
    state: String,
    ts: serde_json::Number,
}

/// 状態の記録の最後の読めた行の ts（末尾から見て、JSON でない行・state が busy でも idle でもない行・
/// ts が整数でも 0 以上の有限の小数でもない行は飛ばす・小数は切り捨て・読めた行が無ければ None）。
fn last_state_ts(text: &str) -> Option<EpochSecs> {
    text.lines().rev().find_map(|l| {
        let r = serde_json::from_str::<StateLine>(l.trim()).ok()?;
        if !matches!(r.state.as_str(), "busy" | "idle") {
            return None;
        }
        r.ts.as_u64().or_else(|| {
            r.ts.as_f64()
                .filter(|f| f.is_finite() && *f >= 0.0)
                .map(|f| f as u64)
        })
    })
}

/// 休止中の席の列。宣言の順に state dir の引けた project の doctor の登録の行（頭が `seat:` の行・役を問わない）を
/// 行の順に見て、同じ席の名は最初の 1 つだけ判じ、状態の記録の最後の読めた行の ts から今までの秒が `DORMANT_S` を
/// 越える席を入れる（ts が今より後なら経過 0・状態の記録が無いか読めた行が無い席は入れない）。
/// 合図の健康の印はその project の合図の健康の出力の席の行から読む。
pub fn dormant(
    host: &HostTexts,
    projects: &BTreeMap<String, ProjectTexts>,
    now: EpochSecs,
) -> Vec<DormantSeat> {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut out = Vec::new();
    for (d, texts) in with_texts(host, projects) {
        let Some(texts) = texts.filter(|t| t.state_dir_known) else {
            continue;
        };
        let Some(doctor) = by_anchor(&host.seat_doctors, &d.anchor) else {
            continue;
        };
        for line in doctor
            .lines()
            .filter_map(|l| l.trim().strip_prefix(SEAT_PREFIX))
        {
            let Some(target) = value(line, "target") else {
                continue;
            };
            if !seen.insert(target) {
                continue;
            }
            let Some(last) = host.seat_logs.get(target).and_then(|t| last_state_ts(t)) else {
                continue;
            };
            if now.saturating_sub(last) <= DORMANT_S {
                continue;
            }
            let anchor = value(line, "anchor");
            let seat_texts = SeatTexts {
                tick_status: texts.tick_status.clone(),
                ..SeatTexts::default()
            };
            let c = card(target, anchor, &seat_texts, now);
            out.push(DormantSeat {
                project: anchor.map(project_name).unwrap_or_default(),
                target: target.to_string(),
                account: value(line, "account").map(str::to_string),
                last,
                tick_healthy: c.tick_healthy,
                heartbeat: c.heartbeat,
            });
        }
    }
    out
}

/// session の行の列（project の宣言の順に、その project の orchestrator の行と、生きていて終わっていない
/// pipeline の run の行を RunCreated の順に）。席の card が「まだ分からない」の project は席なしの行にする。
/// 席の card が読めて、その席が休止中（`dormant`）の project は orchestrator の行を出さない（run の行は出す・行 c-dormant）。
/// pipeline の行の状態は、段を決める最後の event が器の上限の印（RunStage の段 `RATE_LIMITED`）なら limit、
/// 段が Questioned・Failed・Stopped なら wait、席が立っていれば run、ほかは wait。
/// 台帳で今閉じている bead の run の行は出さない（台帳の字が無いか読めなければ外さない・行 e-sess-closed）。
pub fn session_lines(
    host: &HostTexts,
    projects: &BTreeMap<String, ProjectTexts>,
    now: EpochSecs,
) -> Vec<SessionLine> {
    let unknown = ProjectTexts::default();
    let resting: BTreeSet<String> = dormant(host, projects, now)
        .into_iter()
        .map(|s| s.target)
        .collect();
    let mut out = Vec::new();
    for (d, texts) in with_texts(host, projects) {
        let texts = texts.unwrap_or(&unknown);
        let name = project_name(&d.anchor);
        match seat(host, &d, texts, now) {
            Reading::Known(c) if resting.contains(&c.target) => {}
            Reading::Known(c) => out.push(SessionLine {
                project: name.clone(),
                role: SeatRole::Orchestrator,
                name: c.target,
                account: c.account,
                state: c.state,
                stage: None,
                since: c.since,
                spans: c.spans,
            }),
            Reading::Unknown => out.push(SessionLine {
                project: name.clone(),
                role: SeatRole::Orchestrator,
                name: String::new(),
                account: None,
                state: SeatState::Unknown,
                stage: None,
                since: None,
                spans: Reading::Unknown,
            }),
        }
        if !texts.state_dir_known {
            continue;
        }
        let Some(events) = texts.events.as_deref().and_then(read_events) else {
            continue;
        };
        let closed = closed_beads(texts.ledger.as_deref(), now);
        for run in runs(&events, now) {
            if run.ended()
                || !run.alive(now)
                || bead_of(&run).is_some_and(|b| closed.contains(b))
            {
                continue;
            }
            let last = run.last_stage();
            let stage = last
                .and_then(|e| {
                    stage_of(
                        text(e, "kind").unwrap_or_default(),
                        text(e, "stage"),
                        text(e, "detail").unwrap_or_default(),
                    )
                })
                .map(|(s, _)| s);
            let limited = last.is_some_and(|e| {
                text(e, "kind") == Some("RunStage") && text(e, "stage") == Some(RATE_LIMITED)
            });
            let stalled = stage
                .is_some_and(|s| matches!(s, Stage::Questioned | Stage::Failed | Stage::Stopped));
            out.push(SessionLine {
                project: name.clone(),
                role: SeatRole::Pipeline,
                name: run.id.to_string(),
                account: run.account(),
                state: if limited {
                    SeatState::Limit
                } else if !stalled && run.seated() {
                    SeatState::Run
                } else {
                    SeatState::Wait
                },
                stage,
                since: last.and_then(event_ts),
                spans: Reading::Unknown,
            });
        }
    }
    out
}

/// 電文を組む（休止中の席は空の列・閾値は読んでいない 3 つの窓・知らせは「まだ分からない」）。
pub fn assemble(
    at: EpochSecs,
    accounts: Reading<Vec<AccountRow>>,
    groups: Reading<Vec<GroupCard>>,
    moves: Reading<Vec<MoveRow>>,
    projects: Vec<ProjectRow>,
    sessions: Vec<SessionLine>,
) -> AccountDoc {
    AccountDoc {
        at,
        accounts,
        caps: host::caps(&HostTexts::default()),
        groups,
        moves,
        notices: Reading::Unknown,
        projects,
        sessions,
        dormant: Vec::new(),
    }
}

/// 入口: host の側の字と anchor → project の字の表と今の時刻から電文を組む。
/// 知らせは宣言の project のうち state dir の引けた project の event log の字を宣言の順に読む。
/// 休止中の席は `dormant` の値（行 c-dormant）。口座の線は host の側の字の event log を読む（行 c-acct-spark）。
pub fn doc(
    host: &HostTexts,
    projects: &BTreeMap<String, ProjectTexts>,
    now: EpochSecs,
) -> AccountDoc {
    let mut doc = assemble(
        now,
        host::accounts_at(host, now),
        host::groups(host),
        host::moves(host),
        project_rows(host, projects, now),
        session_lines(host, projects, now),
    );
    let logs: Vec<&str> = with_texts(host, projects)
        .into_iter()
        .filter_map(|(_, texts)| {
            texts
                .filter(|t| t.state_dir_known)
                .and_then(|t| t.events.as_deref())
        })
        .collect();
    doc.caps = host::caps(host);
    doc.notices = host::notices(host, &logs);
    doc.dormant = dormant(host, projects, now);
    doc
}
