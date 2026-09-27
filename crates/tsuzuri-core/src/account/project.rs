//! project の側の部分（project の行・session の行・電文の組み立て・設計ノート surface-base 便 e-acct-proj）。
//! 入力は host の側の字（`HostTexts`）と、anchor ごとの project の字（`ProjectTexts`）と、猶予の秒の字と今の時刻。
//! project の宣言の順は群の宣言の順で、群の中は anchors の配列の順（同じ anchor は最初の群だけ）。
//! 席の card・台帳の指標・次の一手は着地済みの関数の値をそのまま写す。run の 4 列の分け方と
//! 生きている run の境（`ALIVE_S`）は見本の acct.js の runColsAt と runAlive の決め方（設計席の承認で置く値）。
//! 読めない字の決まり（要件 NFR2）: state dir が引けない project は席と run と台帳と次の一手が「まだ分からない」、
//! event log の字が無いか読めなければ run の 4 列が、台帳の字が無ければ台帳と次の一手が「まだ分からない」。

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::account::{
    AccountDoc, AccountRow, GroupCard, MoveRow, ProjectRow, RunCounts, SessionLine,
};
use tsuzuri_contract::board::{Reading, Stage};
use tsuzuri_contract::seat::{SeatCard, SeatState};
use tsuzuri_contract::surface::SeatRole;

use super::host::{self, HostTexts, ORCHESTRATOR, RECORD_KIND, declaration};
use super::{field, project_name, same_path, value};
use crate::graph::build::{read_events, run_bead};
use crate::ledger::stats::stats;
use crate::ledger::{DAY, epoch_secs};
use crate::next_step::next_step_seat;
use crate::pipeline::{ACCOUNT_TAG, FAILED_VERDICTS, stage_of};
use crate::seat::{SeatTexts, card};

/// 生きている run の境（最後の event からの秒・見本の acct.js の runAlive）。
pub const ALIVE_S: u64 = 60 * 60;

/// run の段を決める event の種類（見本の acct.js の runColsAt）。
pub const RUN_STAGE_EVENTS: [&str; 5] = [
    "RunCreated",
    "RunStage",
    "RunDone",
    "RunStopped",
    "QuestionRaised",
];

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
    /// 席の移動の合図（`move-signal`）。
    pub move_signal: Option<String>,
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

/// 群の記録の `鍵=値` の行の値（空なら None）。
fn record_value<'a>(record: &'a str, key: &str) -> Option<&'a str> {
    record
        .lines()
        .find_map(|l| l.trim().strip_prefix(key)?.strip_prefix('='))
        .map(str::trim)
        .filter(|v| !v.is_empty())
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

/// 1 つの project の読みの材料（席の行と席の card）。
struct Seat<'a> {
    /// 席の行（無ければ None）。
    line: Option<&'a str>,
    card: Reading<SeatCard>,
}

/// その project の席（state dir が引けないか席の行が無ければ card は「まだ分からない」）。
fn seat<'a>(host: &'a HostTexts, d: &Declared, texts: &ProjectTexts, now: EpochSecs) -> Seat<'a> {
    let doctor = by_anchor(&host.seat_doctors, &d.anchor);
    let line = doctor.and_then(|doc| orchestrator_line(doc, &d.anchor));
    let target = line.and_then(|l| value(l, "target"));
    let card = match target {
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
    };
    Seat { line, card }
}

/// 退避までの残り秒。群の今の記録が在り、移動の合図の ts がその記録の ts と同じ字で、席の口座が記録の口座と違い、
/// 記録の ts に猶予を足した時刻が今より後のときだけ、その差の秒（ほかは None）。
fn move_left(
    host: &HostTexts,
    group: &str,
    seat_account: Option<&str>,
    move_signal: Option<&str>,
    grace: Option<u64>,
    now: EpochSecs,
) -> Option<u64> {
    let grace = grace?;
    let record = host.records.get(&current_record(group))?;
    let ts = record_value(record, "ts")?;
    let signal = move_signal?.lines().find_map(|l| value(l.trim(), "ts"))?;
    if signal != ts || seat_account? == record_value(record, "account")? {
        return None;
    }
    let until = epoch_secs(ts)?.checked_add(grace)?;
    (until > now).then(|| until - now)
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

/// run の 4 列の数（bead ごとに RunCreated がいちばん新しい run の、今までの最後の段で分ける）。
/// 段が Landed なら今日（UTC）の着地だけ land、verdict が落ちか段が Questioned・Failed・Stopped なら stop、
/// 段が Spawned・Implemented・Gated なら run、ほかは wait。event log の字が無いか読めなければ「まだ分からない」。
pub fn run_counts(events: Option<&str>, now: EpochSecs) -> Reading<RunCounts> {
    let Some(events) = events.and_then(read_events) else {
        return Reading::Unknown;
    };
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
    for run in latest.values() {
        let Some(last) = run.last_stage() else {
            continue;
        };
        let stage = match text(last, "kind") {
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

/// 猶予の秒の字（1 行の秒の数・読めなければ None）。
fn grace_secs(grace: Option<&str>) -> Option<u64> {
    grace?.trim().parse().ok()
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
    grace: Option<&str>,
    now: EpochSecs,
) -> Vec<ProjectRow> {
    let grace = grace_secs(grace);
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
                    let card = match &seat.card {
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
                move_left_s: move_left(
                    host,
                    &d.group,
                    seat.line.and_then(|l| value(l, "account")),
                    texts.move_signal.as_deref(),
                    grace,
                    now,
                ),
                group: Some(d.group),
                state_dir_known: true,
                seat: seat.card,
                runs: run_counts(texts.events.as_deref(), now),
                ledger,
                next,
                board: None,
            }
        })
        .collect()
}

/// session の行の列（project の宣言の順に、その project の orchestrator の行と、生きていて終わっていない
/// pipeline の run の行を RunCreated の順に）。席の card が「まだ分からない」の project は席なしの行にする。
/// pipeline の行の状態は、段が Questioned・Failed・Stopped なら wait、席が立っていれば run、ほかは wait。
pub fn session_lines(
    host: &HostTexts,
    projects: &BTreeMap<String, ProjectTexts>,
    now: EpochSecs,
) -> Vec<SessionLine> {
    let unknown = ProjectTexts::default();
    let mut out = Vec::new();
    for (d, texts) in with_texts(host, projects) {
        let texts = texts.unwrap_or(&unknown);
        let name = project_name(&d.anchor);
        out.push(match seat(host, &d, texts, now).card {
            Reading::Known(c) => SessionLine {
                project: name.clone(),
                role: SeatRole::Orchestrator,
                name: c.target,
                account: c.account,
                state: c.state,
                stage: None,
                since: c.since,
                spans: c.spans,
            },
            Reading::Unknown => SessionLine {
                project: name.clone(),
                role: SeatRole::Orchestrator,
                name: String::new(),
                account: None,
                state: SeatState::Unknown,
                stage: None,
                since: None,
                spans: Reading::Unknown,
            },
        });
        if !texts.state_dir_known {
            continue;
        }
        let Some(events) = texts.events.as_deref().and_then(read_events) else {
            continue;
        };
        for run in runs(&events, now) {
            if run.ended() || !run.alive(now) {
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
            let stalled = stage
                .is_some_and(|s| matches!(s, Stage::Questioned | Stage::Failed | Stage::Stopped));
            out.push(SessionLine {
                project: name.clone(),
                role: SeatRole::Pipeline,
                name: run.id.to_string(),
                account: run.account(),
                state: if !stalled && run.seated() {
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

/// 電文を組む（休止中の席は空の列）。
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
        groups,
        moves,
        projects,
        sessions,
        dormant: Vec::new(),
    }
}

/// 入口: host の側の字と anchor → project の字の表と猶予の秒の字と今の時刻から電文を組む。
pub fn doc(
    host: &HostTexts,
    projects: &BTreeMap<String, ProjectTexts>,
    grace: Option<&str>,
    now: EpochSecs,
) -> AccountDoc {
    assemble(
        now,
        host::accounts(host),
        host::groups(host),
        host::moves(host),
        project_rows(host, projects, grace, now),
        session_lines(host, projects, now),
    )
}
