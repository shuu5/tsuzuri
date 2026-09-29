//! account board の読みの歯（接頭辞 server_acct_・設計ノート surface-base 便 e-acct の完了の条件）。
//! 偽の器・偽の git・偽の bd は、受けた argv（bd は cwd の最後の区切り）を記録の置き場（repo と state dir の外）に
//! 1 行ずつ足し、作業場の out・git・bd の下の決めた字を標準出力へ出す script（file が無ければ rc 1・
//! slow の下に同じ名の印が在れば 8 秒眠る）。器は argv の頭と最後の引数（state dir の最後の区切り・rules は
//! 3 つ目の argv の行の id）で、git は -C の次の path の最後の区切りで、bd は cwd の最後の区切りで字を選ぶ。
//! anchor は作業場の work の下の dir（proj-a ほか）で、字の中の path は実行の時に組む（行 D-4）。
//! 接頭辞 alean_ の歯は、集め直しの git の読みの持ち回しと、台帳の印を見て bd を撃たない読みを見る（行 a-lean）。

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::acct::{Acct, BOARD_ARGS, CAP_ARGS, GIT_ARGS, GIT_HOLD};
use tsuzuri_boundary::server::held::FAILED_HOLD;
use tsuzuri_boundary::server::ledger::Source;
use tsuzuri_boundary::server::seat::{HOLD, USAGE_ARGS};
use tsuzuri_contract::account::{
    AccountDoc, DormantSeat, ProjectRow, Spark, SparkLine, SparkPoint,
};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::SeatState;
use tsuzuri_contract::surface::SeatRole;
use tsuzuri_core::account::host::{CAP_ROWS, HostTexts};
use tsuzuri_core::account::project::{self, ProjectTexts};

/// 読みの今の時刻（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

/// 宣言の project（host.toml の中の書き方の末尾の「/」の有無・state dir の名・orchestrator の席の dir の名）。
/// proj-c は git が落ち、proj-d は git が空白だけを返す。
const PROJECTS: [(&str, &str, Option<&str>, Option<&str>); 5] = [
    ("proj-a", "", Some("state-a"), Some("proj-a_0.1")),
    ("proj-e", "/", Some("state-a"), Some("proj-e_0.1")),
    ("proj-b", "", Some("state-b"), Some("proj-b_0.1")),
    ("proj-c", "", None, None),
    ("proj-d", "", None, None),
];

/// proj-a の席の行は器の §20 の欄（reopens= と move= と grace_left=）を持つ。
const TICK_A: &str = "seat tick status: target=proj-a:0.1 last=1790510390 age=10 healthy=yes heartbeat=on step=5 next=1790510395 reopens=- move=acct-2 grace_left=1200\n\
seat tick status: target=proj-e:0.1 last=1790510000 age=400 healthy=no heartbeat=off step=5 next=1790510405\n";
const TICK_B: &str = "seat tick status: target=proj-b:0.1 last=1790510390 age=10 healthy=yes heartbeat=on step=5 next=1790510395\n";

const USAGE: &str = "usage: account=acct-1 five_hour=83% resets=2026-09-27T15:00:00Z seven_day=40% resets=none model=opus:12% resets=none\n\
usage: account=acct-2 five_hour=5% resets=none seven_day=30% resets=none model=sonnet:0% resets=none\n\
usage: account=acct-3 unmeasured reason=no-token\n";

/// 窓ごとの閾値の rules 行の出力（CAP_ROWS の順）。
const CAPS: [&str; 3] = ["85\n", "95\n", "95\n"];

/// 猶予の rules 行（読みは撃たない）と、偽の器がそれに返すはずの字。
const GRACE_RULE: (&str, &str) = ("seat.move_grace_s", "1800\n");

const GROUP_LINES: &str = "group=g-a accounts=acct-1,acct-2 anchors=2 seat-accounts=acct-2 current=acct-1 next=acct-2 refused=-\n\
group=g-b accounts=acct-2,acct-3 anchors=3 seat-accounts=acct-3 current=acct-3 next=none refused=-\n";

const STATE_A: &str = "{\"schema\":1,\"state\":\"idle\",\"event\":\"stop\",\"ts\":1790440000,\"sid\":\"s-1\"}\n\
{\"schema\":1,\"state\":\"busy\",\"event\":\"prompt\",\"ts\":1790500000,\"sid\":\"s-1\"}\n";
const STATE_E: &str =
    "{\"schema\":1,\"state\":\"idle\",\"event\":\"stop\",\"ts\":1790508000,\"sid\":\"s-2\"}\n";
const STATE_B: &str =
    "{\"schema\":1,\"state\":\"busy\",\"event\":\"prompt\",\"ts\":1790505000,\"sid\":\"s-3\"}\n";
/// 休止中の席の記録（登録の行の proj-a:1.1 と proj-z:0.1・最後の行は今より 110400 秒前・行 c-dormant）。
const STATE_PIPE: &str =
    "{\"schema\":1,\"state\":\"busy\",\"event\":\"prompt\",\"ts\":1790400000,\"sid\":\"s-9\"}\n";

const EVENTS_A: &str = "{\"schema\":1,\"ts\":\"2026-09-27T11:00:00Z\",\"kind\":\"RunCreated\",\"run\":\"r.1-20260927T110000Z\",\"bead\":\"r.1\",\"host\":\"host-1\",\"actor\":\"machine\",\"stage\":\"Intake\",\"detail\":\"classes:\"}\n\
{\"schema\":1,\"ts\":\"2026-09-27T11:10:00Z\",\"kind\":\"SeatSpawned\",\"run\":\"r.1-20260927T110000Z\",\"bead\":\"r.1\",\"host\":\"host-1\",\"actor\":\"machine\",\"detail\":\"account:acct-2\"}\n\
{\"schema\":1,\"ts\":\"2026-09-27T11:20:00Z\",\"kind\":\"RunStage\",\"run\":\"r.1-20260927T110000Z\",\"bead\":\"r.1\",\"host\":\"host-1\",\"actor\":\"machine\",\"stage\":\"Implemented\"}\n";
const EVENTS_B: &str = "{\"schema\":1,\"ts\":\"2026-09-27T09:00:00Z\",\"kind\":\"RunCreated\",\"run\":\"b.1-20260927T090000Z\",\"bead\":\"b.1\",\"host\":\"host-2\",\"actor\":\"machine\",\"stage\":\"Intake\",\"detail\":\"classes:\"}\n";

/// 引数の state dir の測りの行（acct-1 の five_hour と seven_day_model・acct-2 の seven_day・字の混じる JSON でない行・
/// 行 c-acct-spark）。
const MEASURED_H: &str = "{\"schema\":1,\"ts\":\"2026-09-27T11:50:00Z\",\"kind\":\"AllowanceMeasured\",\"host\":\"host-1\",\"actor\":\"machine\",\"account\":\"acct-1\",\"window\":\"five_hour\",\"endpoint\":\"usage\",\"used_pct\":83}\n\
{\"schema\":1,\"ts\":\"2026-09-27T11:50:00Z\",\"kind\":\"AllowanceMeasured\",\"host\":\"host-1\",\"actor\":\"machine\",\"account\":\"acct-1\",\"window\":\"seven_day_model\",\"endpoint\":\"usage\",\"used_pct\":12,\"model\":\"opus\"}\n\
{\"schema\":1,\"ts\":\"2026-09-27T09:00:00Z\",\"kind\":\"AllowanceMeasured\",\"host\":\"host-1\",\"actor\":\"machine\",\"account\":\"acct-2\",\"window\":\"seven_day\",\"endpoint\":\"usage\",\"used_pct\":300}\n\
AllowanceMeasured {not json\n";
/// project の state dir の測りの行（線には読まない）。
const MEASURED_B: &str = "{\"schema\":1,\"ts\":\"2026-09-27T11:00:00Z\",\"kind\":\"AllowanceMeasured\",\"host\":\"host-2\",\"actor\":\"machine\",\"account\":\"acct-3\",\"window\":\"five_hour\",\"endpoint\":\"usage\",\"used_pct\":7}\n";

const LEDGER_A: &str = "[{\"id\":\"r.1\",\"title\":\"t\",\"status\":\"open\",\"issue_type\":\"task\",\"updated_at\":\"2026-09-27T07:39:00Z\"}]\n";
const LEDGER_E: &str = "[]\n";

/// 群の記録の dir の file（名・字）。
const RECORDS: [(&str, &str); 2] = [
    (
        "g-a.account",
        "account=acct-1\nts=2026-09-27T11:50:00Z\nreason=account-pressed\nprevious=acct-2\n",
    ),
    (
        "g-b.request.20260927T110000Z.1",
        "account=acct-2\nts=2026-09-27T11:00:00Z\nreason=manual\nprevious=acct-3\n",
    ),
];

/// 群の記録の history の file（名・字）。
const HISTORY: [(&str, &str); 2] = [
    (
        "g-a.account.20260927T100000Z.1",
        "account=acct-2\nts=2026-09-27T10:00:00Z\nreason=account-pressed\nprevious=acct-1\n",
    ),
    (
        "g-b.account.20260926T000000Z.1",
        "account=acct-3\nts=2026-09-26T00:00:00Z\nreason=initial\n",
    ),
];

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 歯ごとの作業場。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    work: PathBuf,
    states: PathBuf,
    groups: PathBuf,
    /// 引数の state dir の名（state-a か state-h）。
    host: &'static str,
    /// 置いた file（path → 字）と偽の器の出力（out の名 → 字）。
    written: BTreeMap<PathBuf, String>,
    outs: BTreeMap<String, String>,
    host_toml: String,
}

impl Place {
    /// `separate` なら引数の state dir はどの anchor の state dir とも違う state-h、でなければ state-a。
    fn new(name: &str, separate: bool) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("server_acct")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let states = root.join("states");
        let mut place = Place {
            repo: root.join("repo"),
            work: root.join("work"),
            groups: states.join("scribe2-host/groups"),
            states,
            host: if separate { "state-h" } else { "state-a" },
            written: BTreeMap::new(),
            outs: BTreeMap::new(),
            host_toml: String::new(),
            root,
        };
        for dir in ["bin", "out", "git", "bd", "slow", "log"] {
            fs::create_dir_all(place.root.join(dir)).expect("置き場");
        }
        fs::create_dir_all(place.repo.join(".beads")).expect("repo");
        fs::create_dir_all(place.groups.join("history")).expect("群の記録の dir");
        fs::create_dir_all(place.groups.join("g-z.account")).expect("dir は読まない");
        fs::create_dir_all(place.states.join(place.host)).expect("引数の state dir");
        for (p, _, state, _) in PROJECTS {
            fs::create_dir_all(place.work.join(p)).expect("anchor");
            if let Some(state) = state {
                fs::create_dir_all(place.states.join(state).join("fleet")).expect("state dir");
            }
        }
        // 字の中の ROOT を作業場の path に替える。
        let root = place.root.display().to_string();
        script(
            &place.program("scribe2"),
            &"printf '%s\\n' \"$*\" >> 'ROOT/log/scribe2'\n\
              for a in \"$@\"; do last=\"$a\"; done\n\
              case \"$1\" in seat) f=\"tick-${last##*/}\" ;; doctor) f=\"doctor-${last##*/}\" ;; \
              fleet) f=\"usage-${last##*/}\" ;; rules) f=\"rules-$3\" ;; *) exit 2 ;; esac\n\
              if [ -e 'ROOT/slow/'\"$f\" ]; then exec sleep 8; fi\n\
              exec cat 'ROOT/out/'\"$f\""
                .replace("ROOT", &root),
        );
        script(
            &place.program("git"),
            &"printf '%s\\n' \"$*\" >> 'ROOT/log/git'\n\
              f=\"${2%/}\"; f=\"${f##*/}\"\n\
              if [ -e 'ROOT/slow/git-'\"$f\" ]; then exec sleep 8; fi\n\
              exec cat 'ROOT/git/'\"$f\""
                .replace("ROOT", &root),
        );
        script(
            &place.program("bd"),
            &"d=\"$(pwd -P)\"; d=\"${d##*/}\"\n\
              printf '%s\\n' \"$d\" >> 'ROOT/log/bd'\n\
              exec cat 'ROOT/bd/'\"$d\""
                .replace("ROOT", &root),
        );
        place.lay();
        place
    }

    /// 偽の program の path。
    fn program(&self, name: &str) -> PathBuf {
        self.root.join("bin").join(name)
    }

    fn anchor(&self, project: &str) -> String {
        self.work.join(project).display().to_string()
    }

    fn state(&self, name: &str) -> PathBuf {
        self.states.join(name)
    }

    fn host_state(&self) -> PathBuf {
        self.state(self.host)
    }

    fn put(&mut self, path: PathBuf, text: &str) {
        fs::create_dir_all(path.parent().expect("親")).expect("親の dir");
        fs::write(&path, text).expect("file を置く");
        self.written.insert(path, text.to_string());
    }

    fn out(&mut self, name: &str, text: &str) {
        self.put(self.root.join("out").join(name), text);
        self.outs.insert(name.to_string(), text.to_string());
    }

    /// 字を組んで置く（anchor の path は実行の時の作業場から）。
    fn lay(&mut self) {
        let [a, e, b, c, d] =
            ["proj-a", "proj-e", "proj-b", "proj-c", "proj-d"].map(|p| self.anchor(p));
        self.host_toml = format!(
            "[[account]]\nlabel = \"acct-1\"\n\n[[account]]\nlabel = \"acct-2\"\n\n[[account]]\nlabel = \"acct-3\"\n\n\
             [[account-group]]\nname = \"g-a\"\nanchors = [\"{a}\", \"{e}/\"]\naccounts = [\"acct-1\", \"acct-2\"]\n\n\
             [[account-group]]\nname = \"g-b\"\nanchors = [\n  \"{b}\",\n  \"{c}\",\n  \"{d}\",\n  \"{a}\",  # 同じ anchor\n]\n\
             accounts = [\"acct-2\", \"acct-3\"]\n"
        );
        self.put(self.host_state().join("host.toml"), &self.host_toml.clone());
        let root = self.root.display().to_string();
        let accounts: String = ["acct-1", "acct-2", "acct-3"]
            .iter()
            .map(|n| format!("account={n} dir={root}/accounts/{n} retired=no\n"))
            .collect();
        let host_lines = format!("doctor: state dir ok\n{GROUP_LINES}{accounts}");
        self.out(
            "doctor-state-a",
            &format!(
                "{host_lines}seat: role=pipeline anchor={a} target=proj-a:1.1 account=acct-2 model=opus\n\
                 seat: role=orchestrator anchor={e} target=proj-e:0.1 account=acct-1 model=opus\n\
                 seat: role=orchestrator anchor={a}/ target=proj-a:0.1 account=acct-2 model=opus\n"
            ),
        );
        self.out(
            "doctor-state-b",
            &format!(
                "{host_lines}seat: role=orchestrator anchor={a} target=proj-z:0.1 account=acct-1 model=opus\n\
                 seat: role=orchestrator anchor={b} target=proj-b:0.1 account=acct-3 model=sonnet\n"
            ),
        );
        if self.host == "state-h" {
            self.out("doctor-state-h", &host_lines);
        }
        self.out("tick-state-a", TICK_A);
        self.out("tick-state-b", TICK_B);
        self.out(&format!("usage-{}", self.host), USAGE);
        for ((_, rule), text) in CAP_ROWS.iter().zip(CAPS) {
            self.out(&format!("rules-{rule}"), text);
        }
        self.out(&format!("rules-{}", GRACE_RULE.0), GRACE_RULE.1);
        // git は state dir の path の前後に空白を付けて返す（除いて使う）。
        let (sa, sb) = (self.state("state-a"), self.state("state-b"));
        self.put(
            self.root.join("git/proj-a"),
            &format!("  {}  \n", sa.display()),
        );
        self.put(self.root.join("git/proj-e"), &format!("{}\n", sa.display()));
        self.put(self.root.join("git/proj-b"), &format!("{}\n", sb.display()));
        self.put(self.root.join("git/proj-d"), " \n");
        self.put(self.root.join("bd/proj-a"), LEDGER_A);
        self.put(self.root.join("bd/proj-e"), LEDGER_E);
        let seat_a = sa.join("seat/proj-a_0.1");
        let seat_e = sa.join("seat/proj-e_0.1");
        let seat_b = sb.join("seat/proj-b_0.1");
        self.put(seat_a.join("state.jsonl"), STATE_A);
        self.put(
            seat_a.join("tick-last"),
            "ts=1790510390 decision=noop reason=seat-busy\n",
        );
        self.put(
            seat_a.join("move-signal"),
            "ts=2026-09-27T11:50:00Z account=acct-1\n",
        );
        self.put(seat_e.join("state.jsonl"), STATE_E);
        self.put(
            seat_e.join("tick-last"),
            "ts=1790510000 decision=nudge reason=state-stale\n",
        );
        self.put(seat_e.join("heartbeat-off"), "");
        self.put(seat_b.join("state.jsonl"), STATE_B);
        self.put(sa.join("seat/proj-a_1.1/state.jsonl"), STATE_PIPE);
        self.put(sb.join("seat/proj-z_0.1/state.jsonl"), STATE_PIPE);
        self.put(sa.join("fleet/events.jsonl"), EVENTS_A);
        self.put(sb.join("fleet/events.jsonl"), EVENTS_B);
        for (name, text) in RECORDS {
            self.put(self.groups.join(name), text);
        }
        for (name, text) in HISTORY {
            self.put(self.groups.join("history").join(name), text);
        }
    }

    fn acct(&self) -> Acct {
        Acct::new(
            self.program("scribe2"),
            self.program("git"),
            self.program("bd"),
            self.host_state(),
            &self.repo,
        )
    }

    /// 偽の器の出力（または git の anchor の `git-<project>`）を落とす。
    fn fail(&self, name: &str) {
        let path = match name.strip_prefix("git-") {
            Some(p) => self.root.join("git").join(p),
            None => self.root.join("out").join(name),
        };
        fs::remove_file(path).expect("出力の file を消す");
    }

    /// 偽の器の出力（または git の anchor の `git-<project>`）を 5 秒で返さなくする。
    fn slow(&self, name: &str) {
        fs::write(self.root.join("slow").join(name), "").expect("眠りの印");
    }

    /// 偽の program が受けた argv（1 回 1 行・名の順）。
    fn calls(&self, program: &str) -> Vec<String> {
        let mut calls: Vec<String> = fs::read_to_string(self.root.join("log").join(program))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default();
        calls.sort();
        calls
    }

    fn file(&self, path: &Path) -> Option<String> {
        self.written.get(path).cloned()
    }

    /// 読みが集めたはずの字（`drop` の出力と git を読めない扱いにする）。
    fn expected(&self, drop: &[&str]) -> (HostTexts, BTreeMap<String, ProjectTexts>) {
        let out = |name: &str| {
            if drop.contains(&name) {
                None
            } else {
                self.outs.get(name).cloned()
            }
        };
        let mut host = HostTexts {
            host_toml: Some(self.host_toml.clone()),
            usage: out(&format!("usage-{}", self.host)),
            doctor: out(&format!("doctor-{}", self.host)),
            records: RECORDS
                .iter()
                .map(|(n, t)| (n.to_string(), t.to_string()))
                .collect(),
            history: HISTORY
                .iter()
                .map(|(n, t)| (n.to_string(), t.to_string()))
                .collect(),
            // 口座の線の材料は引数の state dir の event log（行 c-acct-spark）。
            events: self.file(&self.host_state().join("fleet/events.jsonl")),
            ..HostTexts::default()
        };
        // 偽の器は rules の頭に行の id の出力の字を出す。
        for (_, rule) in CAP_ROWS {
            if let Some(text) = out(&format!("rules-{rule}")) {
                host.caps.insert(rule.to_string(), text);
            }
        }
        // 重ならない state dir ごとに、doctor の登録の行の全部の席の状態の記録（同じ席は最初の state dir の字）。
        let mut dirs: Vec<&str> = Vec::new();
        for (p, _, state, _) in PROJECTS {
            if let Some(state) = state.filter(|_| !drop.contains(&format!("git-{p}").as_str()))
                && !dirs.contains(&state)
            {
                dirs.push(state);
            }
        }
        for state in dirs {
            let Some(doctor) = out(&format!("doctor-{state}")) else {
                continue;
            };
            for line in doctor.lines().filter_map(|l| l.strip_prefix("seat: ")) {
                let Some(target) = line
                    .split_whitespace()
                    .find_map(|t| t.strip_prefix("target="))
                else {
                    continue;
                };
                let log = self.file(
                    &self
                        .state(state)
                        .join("seat")
                        .join(target.replace(':', "_"))
                        .join("state.jsonl"),
                );
                if let Some(log) = log
                    && !host.seat_logs.contains_key(target)
                {
                    host.seat_logs.insert(target.to_string(), log);
                }
            }
        }
        let mut projects = BTreeMap::new();
        for (p, tail, state, seat) in PROJECTS {
            let anchor = format!("{}{tail}", self.anchor(p));
            let state = state.filter(|_| !drop.contains(&format!("git-{p}").as_str()));
            let Some(state) = state else {
                projects.insert(anchor, ProjectTexts::default());
                continue;
            };
            let dir = self.state(state);
            let doctor = out(&format!("doctor-{state}"));
            let seat_dir = seat
                .filter(|_| doctor.is_some())
                .map(|s| dir.join("seat").join(s));
            let in_seat = |name: &str| seat_dir.as_ref().and_then(|d| self.file(&d.join(name)));
            projects.insert(
                anchor.clone(),
                ProjectTexts {
                    state_dir_known: true,
                    tick_status: out(&format!("tick-{state}")),
                    state_log: in_seat("state.jsonl"),
                    tick_last: in_seat("tick-last"),
                    events: self.file(&dir.join("fleet/events.jsonl")),
                    ledger: self.file(&self.root.join("bd").join(p)),
                },
            );
            if let Some(doctor) = doctor {
                host.seat_doctors.insert(anchor, doctor);
            }
        }
        (host, projects)
    }

    /// 中核の組み立ての入口を、読みが集めたはずの字で直に呼んだ電文。
    fn core_doc(&self, drop: &[&str]) -> AccountDoc {
        let (host, projects) = self.expected(drop);
        project::doc(&host, &projects, NOW)
    }
}

fn row<'a>(doc: &'a AccountDoc, name: &str) -> &'a ProjectRow {
    doc.projects
        .iter()
        .find(|r| r.name == name)
        .unwrap_or_else(|| panic!("project {name} の行が無い"))
}

/// dir の中の file の path と byte の一覧。
fn tree(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).expect("dir を読む") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                out.push((path.clone(), Vec::new()));
                stack.push(path);
            } else {
                out.push((path.clone(), fs::read(&path).expect("file を読む")));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn server_acct_doc_matches_core() {
    for separate in [false, true] {
        let place = Place::new(&format!("match-{separate}"), separate);
        let got = place.acct().doc(NOW);
        assert_eq!(
            got,
            place.core_doc(&[]),
            "引数の state dir が別: {separate}"
        );
        assert!((1..=NOW).contains(&got.at), "時点は今より前の材料の時刻");
        let names: Vec<&str> = got.projects.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["proj-a", "proj-e", "proj-b", "proj-c", "proj-d"]);
        // git の引けた project は席と run が読める。
        for p in ["proj-a", "proj-e", "proj-b"] {
            let r = row(&got, p);
            assert!(r.state_dir_known, "{p}");
            assert!(matches!(r.seat, Reading::Known(_)), "{p}: {:?}", r.seat);
            assert!(matches!(r.runs, Reading::Known(_)), "{p}: {:?}", r.runs);
        }
        let Reading::Known(seat) = &row(&got, "proj-a").seat else {
            unreachable!()
        };
        assert_eq!(seat.state, SeatState::Run, "orchestrator の席の記録を読む");
        // 終わる時刻は器の合図の健康の行の grace_left=1200 に今を足した時刻（欄の無い席の行は無し）。
        assert_eq!(row(&got, "proj-a").move_until, Some(NOW + 1200));
        assert_eq!(row(&got, "proj-e").move_until, None);
        // git が落ちるか空を返す project は state dir なしの行。
        for p in ["proj-c", "proj-d"] {
            let r = row(&got, p);
            assert!(!r.state_dir_known, "{p}");
            assert_eq!(
                (&r.seat, &r.runs, &r.ledger, &r.next),
                (
                    &Reading::Unknown,
                    &Reading::Unknown,
                    &Reading::Unknown,
                    &Reading::Unknown
                ),
                "{p}"
            );
        }
        // 台帳が読めない project は台帳と次の一手が Unknown。
        let b = row(&got, "proj-b");
        assert_eq!((&b.ledger, &b.next), (&Reading::Unknown, &Reading::Unknown));
        for p in ["proj-a", "proj-e"] {
            let r = row(&got, p);
            assert!(matches!(r.ledger, Reading::Known(_)), "{p}");
            assert!(matches!(r.next, Reading::Known(_)), "{p}");
        }
        assert!(matches!(got.accounts, Reading::Known(_)));
        assert!(matches!(got.groups, Reading::Known(_)));
        let Reading::Known(moves) = &got.moves else {
            panic!("移動が Unknown");
        };
        assert_eq!(
            moves.len(),
            3,
            "今の記録と history の account の記録: {moves:?}"
        );
    }
}

#[test]
fn server_acct_argv_exact() {
    for separate in [false, true] {
        let place = Place::new(&format!("argv-{separate}"), separate);
        place.acct().doc(NOW);
        // git は宣言の anchor ごとに state dir と board の port を 1 回ずつ（同じ anchor は 1 回・書かれた字のまま）。
        let mut want: Vec<String> = PROJECTS
            .iter()
            .flat_map(|(p, tail, _, _)| {
                ["scribe2.statedir", "tsuzuri.boardport"]
                    .map(|key| format!("-C {}{tail} config --get {key}", place.anchor(p)))
            })
            .collect();
        want.sort();
        assert_eq!(place.calls("git"), want, "{separate}");
        assert_eq!(GIT_ARGS, ["config", "--get", "scribe2.statedir"]);
        assert_eq!(BOARD_ARGS, ["config", "--get", "tsuzuri.boardport"]);
        // 器は state dir ごとに tick と doctor を 1 回、口座と閾値の行ごとは 1 回（猶予の rules 行は撃たない）。
        let (sa, sb, sh) = (
            place.state("state-a").display().to_string(),
            place.state("state-b").display().to_string(),
            place.host_state().display().to_string(),
        );
        let mut want = vec![
            format!("seat tick status --state-dir {sa}"),
            format!("seat tick status --state-dir {sb}"),
            format!("doctor --state-dir {sa}"),
            format!("doctor --state-dir {sb}"),
            format!("fleet usage --show --state-dir {sh}"),
        ];
        want.extend(CAP_ROWS.map(|(_, rule)| format!("rules get {rule}")));
        if separate {
            want.push(format!("doctor --state-dir {sh}"));
        }
        want.sort();
        let calls = place.calls("scribe2");
        assert_eq!(calls, want, "{separate}");
        assert!(
            calls
                .iter()
                .filter(|c| c.starts_with("fleet usage"))
                .all(|c| c.starts_with("fleet usage --show "))
        );
        assert_eq!(USAGE_ARGS, ["fleet", "usage", "--show"]);
        assert!(calls.iter().all(|c| !c.contains(GRACE_RULE.0)));
        assert_eq!(CAP_ARGS, ["rules", "get"]);
        // bd は state dir の引けた anchor ごとに 1 回（cwd が anchor）。
        assert_eq!(
            place.calls("bd"),
            ["proj-a", "proj-b", "proj-e"],
            "{separate}"
        );
    }
}

#[test]
fn server_acct_state_dir_from_git() {
    let place = Place::new("state-dir", false);
    let acct = place.acct();
    let dir = |p: &str| acct.state_dir(&place.work.join(p));
    assert_eq!(
        dir("proj-a"),
        Some(place.state("state-a")),
        "前後の空白を除く"
    );
    assert_eq!(dir("proj-b"), Some(place.state("state-b")));
    assert_eq!(dir("proj-c"), None, "git が落ちる");
    assert_eq!(dir("proj-d"), None, "git が空を返す");
    let none = Acct::new(
        place.program("scribe2"),
        "/nonexistent/tz-no-such-git",
        place.program("bd"),
        place.host_state(),
        &place.repo,
    );
    assert_eq!(
        none.state_dir(&place.work.join("proj-a")),
        None,
        "git が撃てない"
    );
}

#[test]
fn server_acct_grace_unreadable_no_left() {
    // 猶予の rules 行の出力を落としても、撃たないので終わる時刻は器の grace_left= に今を足したままの写し。
    let place = Place::new("grace-rule-fail", false);
    place.fail(&format!("rules-{}", GRACE_RULE.0));
    let got = place.acct().doc(NOW);
    assert_eq!(got, place.core_doc(&[]));
    assert_eq!(row(&got, "proj-a").move_until, Some(NOW + 1200));
    // 器の字が unreadable なら全部の project で無し（席の card はほかの欄を読む）。
    let mut place = Place::new("grace-unreadable", false);
    place.out(
        "tick-state-a",
        &TICK_A.replace("grace_left=1200", "grace_left=unreadable"),
    );
    let got = place.acct().doc(NOW);
    assert_eq!(got, place.core_doc(&[]));
    assert!(got.projects.iter().all(|r| r.move_until.is_none()));
    let Reading::Known(seat) = &row(&got, "proj-a").seat else {
        panic!("proj-a の席の card が読めない");
    };
    assert_eq!(
        (&seat.move_to, &seat.grace_until),
        (&Reading::Known(Some("acct-2".to_string())), &Reading::Unknown)
    );
}

#[test]
fn server_acct_failed_output_only_its_parts() {
    for (separate, out) in [
        (false, "tick-state-a"),
        (false, "doctor-state-a"),
        (false, "doctor-state-b"),
        (false, "usage-state-a"),
        (true, "doctor-state-h"),
        (true, "usage-state-h"),
        (false, "git-proj-b"),
    ] {
        let place = Place::new(&format!("fail-{out}"), separate);
        place.fail(out);
        let got = place.acct().doc(NOW);
        assert_eq!(got, place.core_doc(&[out]), "{out} が落ちる");
        assert_ne!(got, place.core_doc(&[]), "{out} が落ちても同じ電文");
        // 台帳と run は器の出力に依らない。
        assert!(
            matches!(row(&got, "proj-a").runs, Reading::Known(_)),
            "{out}"
        );
        assert!(
            matches!(row(&got, "proj-a").ledger, Reading::Known(_)),
            "{out}"
        );
    }
}

#[test]
fn server_acct_slow_output_is_unknown() {
    let place = Place::new("slow", false);
    place.slow("usage-state-a");
    place.slow("git-proj-b");
    let from = Instant::now();
    let got = place.acct().doc(NOW);
    let took = from.elapsed();
    assert!(took >= Duration::from_secs(4), "5 秒待たずに返る: {took:?}");
    assert!(
        took < Duration::from_secs(8),
        "眠りの終わりまで待つ: {took:?}"
    );
    assert_eq!(got, place.core_doc(&["usage-state-a", "git-proj-b"]));
    assert!(!row(&got, "proj-b").state_dir_known);
    assert!(
        matches!(got.groups, Reading::Known(_)),
        "doctor の部分は残る"
    );
}

#[test]
fn server_acct_holds_five_seconds() {
    let place = Place::new("hold", false);
    let acct = place.acct();
    let first = acct.doc(NOW);
    let second = acct.doc(NOW);
    acct.marks();
    assert_eq!(first, second);
    let once = (
        place.calls("scribe2"),
        place.calls("git"),
        place.calls("bd"),
    );
    assert_eq!(once.0.len(), 8, "5 秒の中の読み: {:?}", once.0);
    assert_eq!(once.1.len(), 10, "{:?}", once.1);
    assert_eq!(once.2.len(), 3, "{:?}", once.2);
    for calls in [&once.0, &once.1, &once.2] {
        let distinct: BTreeSet<&String> = calls.iter().collect();
        assert_eq!(distinct.len(), calls.len(), "argv ごとに 1 回: {calls:?}");
    }
    thread::sleep(HOLD + Duration::from_millis(300));
    acct.doc(NOW);
    // 5 秒の後は tick status だけを撃ち直す（doctor と usage と rules get は SLOW_HOLD の間持つ・行 e-held-acct）。
    let mut want = once.0.clone();
    want.extend(once.0.iter().filter(|c| c.starts_with("seat tick status")).cloned());
    want.sort();
    assert_eq!(place.calls("scribe2"), want, "5 秒の後の読み");
    // git は GIT_HOLD のあいだ持ち回し、台帳は印が同じで読めていた proj-a と proj-e の bd を撃たない（行 a-lean）。
    assert_eq!(place.calls("git"), once.1);
    assert_eq!(place.calls("bd"), ["proj-a", "proj-b", "proj-b", "proj-e"]);
}

#[test]
fn server_acct_repo_and_state_bytes_unchanged() {
    let place = Place::new("bytes", true);
    let snapshot = || (tree(&place.repo), tree(&place.work), tree(&place.states));
    let before = snapshot();
    let acct = place.acct();
    acct.doc(NOW);
    acct.marks();
    assert!(!place.calls("scribe2").is_empty());
    assert!(
        before == snapshot(),
        "読みの後に repo か anchor か state dir か群の記録の byte が変わる"
    );
}

#[test]
fn server_acct_marks_list() {
    let place = Place::new("marks", true);
    let got: BTreeSet<PathBuf> = place.acct().marks().into_iter().collect();
    let (sa, sb) = (place.state("state-a"), place.state("state-b"));
    let mut want = BTreeSet::from([
        sa.join("fleet/events.jsonl"),
        sb.join("fleet/events.jsonl"),
        place.groups.join("g-a.account"),
        place.groups.join("g-b.account"),
    ]);
    for seat in [
        sa.join("seat/proj-a_0.1"),
        sa.join("seat/proj-e_0.1"),
        sb.join("seat/proj-b_0.1"),
    ] {
        for name in ["state.jsonl", "tick-last", "heartbeat-off", "heartbeat-on"] {
            want.insert(seat.join(name));
        }
    }
    assert_eq!(got, want);
    assert!(
        got.iter().all(|p| !p.ends_with("move-signal")),
        "move-signal は印にしない"
    );
}

#[test]
fn cdorm_acct_reads_all_seat_logs() {
    let place = Place::new("cdorm", true);
    let acct = place.acct();
    let got = acct.doc(NOW);
    let resting = |target: &str, account: &str| DormantSeat {
        project: "proj-a".to_string(),
        target: target.to_string(),
        account: Some(account.to_string()),
        last: 1_790_400_000,
        tick_healthy: Reading::Unknown,
        heartbeat: Reading::Unknown,
    };
    assert_eq!(
        got.dormant,
        [resting("proj-a:1.1", "acct-2"), resting("proj-z:0.1", "acct-1")]
    );
    let orchestrators: Vec<&str> = got
        .sessions
        .iter()
        .filter(|s| s.role == SeatRole::Orchestrator)
        .map(|s| s.name.as_str())
        .collect();
    // state dir の引けない proj-c と proj-d は名の空の行のまま。
    assert_eq!(
        orchestrators,
        ["proj-a:0.1", "proj-e:0.1", "proj-b:0.1", "", ""]
    );
    assert_eq!(got, place.core_doc(&[]));
    let (sa, sb) = (place.state("state-a"), place.state("state-b"));
    let resting_dirs = [sa.join("seat/proj-a_1.1"), sb.join("seat/proj-z_0.1")];
    assert!(
        acct.marks()
            .iter()
            .all(|p| resting_dirs.iter().all(|d| !p.starts_with(d))),
        "休止中の席の記録は印にしない"
    );
}

/// 字の中の口座の名（`鍵=値` の値・`account:` の札・TOML の label と accounts）と host の名（JSON の host の欄）。
fn names(text: &str) -> (Vec<String>, Vec<String>) {
    let mut accounts = Vec::new();
    for key in ["account=", "accounts=", "current=", "previous=", "account:"] {
        for (at, _) in text.match_indices(key) {
            let rest = &text[at + key.len()..];
            let end = rest
                .find(|c: char| c.is_whitespace() || c == '"' || c == '}')
                .unwrap_or(rest.len());
            accounts.extend(
                rest[..end]
                    .split(',')
                    .filter(|v| !v.is_empty() && !matches!(*v, "none" | "-"))
                    .map(str::to_string),
            );
        }
    }
    for line in text.lines().map(str::trim) {
        if line.starts_with("label") || line.starts_with("accounts =") {
            accounts.extend(line.split('"').skip(1).step_by(2).map(str::to_string));
        }
        // 群の行の次の口座（tick status の next= は時刻なので群の行だけ）。
        if line.starts_with("group=") {
            accounts.extend(
                line.split_whitespace()
                    .filter_map(|t| t.strip_prefix("next="))
                    .filter(|v| *v != "none")
                    .map(str::to_string),
            );
        }
    }
    let hosts = text
        .match_indices("\"host\":\"")
        .map(|(at, key)| {
            let rest = &text[at + key.len()..];
            rest[..rest.find('"').unwrap_or(rest.len())].to_string()
        })
        .collect();
    (accounts, hosts)
}

#[test]
fn server_acct_fixture_names() {
    let place = Place::new("names", true);
    let mut accounts = Vec::new();
    let mut hosts = Vec::new();
    for dir in [
        place.root.join("out"),
        place.root.join("git"),
        place.root.join("bd"),
        place.states.clone(),
    ] {
        for (path, bytes) in tree(&dir) {
            if path.is_dir() {
                continue;
            }
            let (a, h) = names(&String::from_utf8(bytes).expect("字"));
            accounts.extend(a);
            hosts.extend(h);
        }
    }
    assert!(accounts.len() > 10, "口座の名を拾えていない: {accounts:?}");
    assert!(!hosts.is_empty(), "host の名を拾えていない");
    for a in &accounts {
        assert!(a.starts_with("acct-"), "口座の名: {a}");
    }
    for h in &hosts {
        assert!(h.starts_with("host-"), "host の名: {h}");
    }
}

#[test]
fn server_acct_no_new_dependencies() {
    let deps = |krate: &str| -> Vec<String> {
        let text = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join(krate)
                .join("Cargo.toml"),
        )
        .expect("Cargo.toml");
        let mut names = Vec::new();
        let mut inside = false;
        for line in text.lines().map(str::trim) {
            if line.starts_with('[') {
                inside = line.ends_with("dependencies]");
                continue;
            }
            if inside
                && let Some((name, _)) = line.split_once('=')
                && !line.starts_with('#')
            {
                let name = name.trim();
                names.push(name.strip_suffix(".workspace").unwrap_or(name).to_string());
            }
        }
        names
    };
    assert_eq!(
        deps("tsuzuri-boundary"),
        ["tsuzuri-contract", "tsuzuri-core"]
    );
    assert_eq!(
        deps("tsuzuri-core"),
        ["serde", "serde_json", "tsuzuri-contract"]
    );
}

/// 電文の口座ごとの線（口座の列が読めなければ panic）。
fn cspk_sparks(doc: &AccountDoc) -> Vec<(String, Reading<Spark>)> {
    let Reading::Known(rows) = &doc.accounts else {
        panic!("口座の列が Unknown");
    };
    rows.iter()
        .map(|r| (r.label.clone(), r.spark.clone()))
        .collect()
}

/// 窓の線（点は (at, 使った割合) の列）。
fn cspk_line(window: &str, points: &[(u64, u8)]) -> SparkLine {
    SparkLine {
        window: window.to_string(),
        points: points
            .iter()
            .map(|&(at, used_pct)| SparkPoint { at, used_pct })
            .collect(),
    }
}

#[test]
fn cspk_acct_reads_host_log() {
    let mut place = Place::new("cspk", true);
    let names = ["acct-1", "acct-2", "acct-3"].map(str::to_string);
    let got = place.acct().doc(NOW);
    assert_eq!(
        cspk_sparks(&got),
        names
            .iter()
            .map(|n| (n.clone(), Reading::Unknown))
            .collect::<Vec<_>>(),
        "引数の state dir に log が無ければ線は Unknown"
    );
    assert_eq!(got, place.core_doc(&[]));
    let host_log = place.host_state().join("fleet/events.jsonl");
    place.put(host_log.clone(), MEASURED_H);
    let b_log = place.state("state-b").join("fleet/events.jsonl");
    place.put(b_log.clone(), &format!("{EVENTS_B}{MEASURED_B}"));
    let acct = place.acct();
    let got = acct.doc(NOW);
    let spark = |measured_at: Option<u64>, lines: [SparkLine; 3]| {
        Reading::Known(Spark {
            measured_at,
            lines: lines.to_vec(),
        })
    };
    let bare = [
        cspk_line("five_hour", &[]),
        cspk_line("seven_day", &[]),
        cspk_line("seven_day_model", &[]),
    ];
    assert_eq!(
        cspk_sparks(&got),
        [
            (
                names[0].clone(),
                spark(
                    Some(1_790_509_800),
                    [
                        cspk_line("five_hour", &[(1_790_509_800, 83)]),
                        cspk_line("seven_day", &[]),
                        cspk_line("seven_day_model", &[(1_790_509_800, 12)]),
                    ]
                )
            ),
            (
                names[1].clone(),
                spark(
                    Some(1_790_499_600),
                    [
                        cspk_line("five_hour", &[]),
                        cspk_line("seven_day", &[(1_790_499_600, 255)]),
                        cspk_line("seven_day_model", &[]),
                    ]
                )
            ),
            (names[2].clone(), spark(None, bare)),
        ]
    );
    assert_eq!(got, place.core_doc(&[]));
    let marks = acct.marks();
    assert!(!marks.contains(&host_log), "引数の state dir の log は印にしない");
    assert!(marks.contains(&b_log));
}

/// 停止の記録を置くか消し、偽の器の tick-state-a の字の proj-a:0.1 の行の heartbeat の語を合わせる（器と同じ決まり）。
fn acchold_switch(place: &Place, off: bool) {
    let path = place.state("state-a").join("seat/proj-a_0.1/heartbeat-off");
    if off {
        fs::write(&path, "").expect("停止の記録を置く");
    } else {
        fs::remove_file(&path).expect("停止の記録を消す");
    }
    let out = place.root.join("out/tick-state-a");
    let (from, to) = if off {
        ("heartbeat=on", "heartbeat=off")
    } else {
        ("heartbeat=off", "heartbeat=on")
    };
    let text: String = fs::read_to_string(&out)
        .expect("tick の出力の字")
        .lines()
        .map(|l| {
            let l = if l.contains("target=proj-a:0.1 ") {
                l.replace(from, to)
            } else {
                l.to_string()
            };
            format!("{l}\n")
        })
        .collect();
    fs::write(&out, text).expect("tick の出力の字を替える");
}

/// proj-a の project の行の席の card の heartbeat。
fn acchold_heartbeat(doc: &AccountDoc) -> Reading<bool> {
    match &row(doc, "proj-a").seat {
        Reading::Known(card) => card.heartbeat.clone(),
        other => panic!("proj-a の席の card が読めない: {other:?}"),
    }
}

#[test]
fn acchold_card_follows_off_file() {
    let place = Place::new("acchold-off", false);
    let acct = place.acct();
    let count = || place.calls("scribe2").len();
    assert_eq!(acchold_heartbeat(&acct.doc(NOW)), Reading::Known(true));
    assert_eq!(count(), 8, "{:?}", place.calls("scribe2"));
    acchold_switch(&place, true);
    assert_eq!(
        acchold_heartbeat(&acct.doc(NOW)),
        Reading::Known(false),
        "停止の記録の後の読み"
    );
    assert_eq!(count(), 10, "{:?}", place.calls("scribe2"));
    assert_eq!(acchold_heartbeat(&acct.doc(NOW)), Reading::Known(false));
    acct.marks();
    assert_eq!(count(), 10, "印が動かなければ持ち回す");
    acchold_switch(&place, false);
    assert_eq!(
        acchold_heartbeat(&acct.doc(NOW)),
        Reading::Known(true),
        "停止の記録を消した後の読み"
    );
    assert_eq!(count(), 12, "{:?}", place.calls("scribe2"));
    // state-a の tick status と doctor だけが 3 回、ほかは 1 回。
    let calls = place.calls("scribe2");
    let distinct: BTreeSet<&String> = calls.iter().collect();
    for argv in distinct {
        let want = if argv.ends_with("state-a") && (argv.starts_with("seat tick") || argv.starts_with("doctor")) { 3 } else { 1 };
        assert_eq!(calls.iter().filter(|c| *c == argv).count(), want, "{argv}");
    }
    assert_eq!(place.calls("git").len(), 10, "{:?}", place.calls("git"));
    assert_eq!(place.calls("bd"), ["proj-a", "proj-b", "proj-e"]);
}

#[test]
fn acchold_state_log_regathers() {
    let place = Place::new("acchold-state", false);
    let acct = place.acct();
    let session = |doc: &AccountDoc| {
        let line = doc
            .sessions
            .iter()
            .find(|s| s.name == "proj-a:0.1")
            .expect("session の行 proj-a:0.1");
        (line.state, line.since)
    };
    let first = acct.doc(NOW);
    assert_eq!(session(&first), (SeatState::Run, Some(1_790_500_000)));
    assert_eq!(place.calls("scribe2").len(), 8);
    let log = place.state("state-a").join("seat/proj-a_0.1/state.jsonl");
    fs::write(
        &log,
        format!(
            "{STATE_A}{{\"schema\":1,\"state\":\"idle\",\"event\":\"stop\",\"ts\":1790510300,\"sid\":\"s-1\"}}\n"
        ),
    )
    .expect("state.jsonl に idle の行を足す");
    let second = acct.doc(NOW);
    assert_eq!(session(&second), (SeatState::Wait, Some(1_790_510_300)));
    assert_eq!(place.calls("scribe2").len(), 8, "状態の記録は印にしない");
    assert_eq!(acct.doc(NOW), second, "印が動かなければ同じ電文");
    assert_eq!(place.calls("scribe2").len(), 8, "印が動かなければ持ち回す");
}

/// state dir の event log の更新時刻だけを 1 秒進める（字と長さは替えない・引数の state dir なら集め直しを起こす）。
fn alean_bump(state: &Path) {
    let log = File::options()
        .write(true)
        .open(state.join("fleet/events.jsonl"))
        .expect("event log を開く");
    let was = log
        .metadata()
        .and_then(|m| m.modified())
        .expect("event log の更新時刻");
    log.set_modified(was + Duration::from_secs(1))
        .expect("event log の更新時刻を替える");
}

/// project の anchor の .beads の issues.jsonl を置く（台帳の印を動かす）。
fn alean_mark(place: &Place, project: &str) {
    let beads = place.work.join(project).join(".beads");
    fs::create_dir_all(&beads).expect("anchor の .beads");
    fs::write(beads.join("issues.jsonl"), "").expect("anchor の issues.jsonl");
}

#[test]
fn alean_ledger_follows_mark() {
    let place = Place::new("alean-mark", false);
    let acct = place.acct();
    let first = acct.doc(NOW);
    assert_eq!(first, place.core_doc(&[]));
    assert_eq!(place.calls("bd"), ["proj-a", "proj-b", "proj-e"]);
    alean_bump(&place.host_state());
    assert_eq!(acct.doc(NOW), first, "印の同じ台帳は前の読み");
    assert_eq!(place.calls("scribe2").len(), 11, "usage と state-a の tick status と doctor");
    let unread = ["proj-a", "proj-b", "proj-e"];
    assert_eq!(place.calls("bd"), unread, "読めなかった proj-b は FAILED_HOLD の内は撃たない");
    alean_mark(&place, "proj-a");
    alean_bump(&place.host_state());
    assert_eq!(acct.doc(NOW), first, "台帳の印が動いた後の読み");
    assert_eq!(place.calls("scribe2").len(), 14);
    assert_eq!(
        place.calls("bd"),
        ["proj-a", "proj-a", "proj-b", "proj-e"],
        "印の動いた proj-a を読み直す"
    );
    thread::sleep(FAILED_HOLD + Duration::from_millis(300));
    assert_eq!(acct.doc(NOW), first);
    assert_eq!(
        place.calls("bd"),
        ["proj-a", "proj-a", "proj-b", "proj-b", "proj-e"],
        "FAILED_HOLD を過ぎれば proj-b を撃ち直す"
    );
}

#[test]
fn alean_git_held() {
    assert_eq!(GIT_HOLD, Duration::from_secs(60));
    let place = Place::new("alean-git", false);
    let acct = place.acct().with_git_hold(Duration::from_millis(400));
    let first = acct.doc(NOW);
    let once = place.calls("git");
    assert_eq!(once.len(), 10, "{once:?}");
    alean_bump(&place.host_state());
    assert_eq!(acct.doc(NOW), first);
    assert_eq!(place.calls("scribe2").len(), 11, "印が動けば usage と tick status と doctor");
    assert_eq!(place.calls("git"), once, "持ち回しの内は git を撃たない");
    thread::sleep(Duration::from_millis(500));
    alean_bump(&place.host_state());
    assert_eq!(acct.doc(NOW), first);
    let mut twice: Vec<String> = once.iter().flat_map(|c| [c.clone(), c.clone()]).collect();
    twice.sort();
    assert_eq!(place.calls("git"), twice, "持ち回しの後は撃ち直す");
}

#[test]
fn alean_own_ledger_shared() {
    let place = Place::new("alean-own", false);
    let own = Source::new(
        format!("{}/.", place.anchor("proj-a")),
        place.program("bd"),
    )
    .watched();
    own.read();
    assert_eq!(place.calls("bd"), ["proj-a"], "見張りの読み");
    let acct = place.acct().with_own(own);
    let first = acct.doc(NOW);
    assert_eq!(
        place.calls("bd"),
        ["proj-a", "proj-b", "proj-e"],
        "proj-a は見張りの読みを分け合う"
    );
    alean_mark(&place, "proj-a");
    alean_bump(&place.host_state());
    assert_eq!(acct.doc(NOW), first);
    assert_eq!(place.calls("bd"), ["proj-a", "proj-b", "proj-e"]);
    assert_eq!(first, place.acct().doc(NOW), "with_own の無い読みと同じ電文");
}

#[test]
fn alean_wiring_text() {
    let read = |path: &str| {
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect(path)
    };
    let (module, acct) = (read("src/server/mod.rs"), read("src/acct.rs"));
    assert_eq!(
        module.matches(".with_own(sources.ledger.clone())").count(),
        1,
        "server が見張りの Source を渡す"
    );
    assert_eq!(
        acct.matches(".map(|_| s.spawn(move || self.ledger_got(a)))")
            .count(),
        1,
        "台帳は印を見る読みで読む"
    );
    assert!(!acct.contains("s.spawn(move || source.got())"));
}

#[test]
fn acchold_events_doc_names_off() {
    let text = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/server/events.rs"),
    )
    .expect("events.rs");
    let doc: String = text
        .lines()
        .filter(|l| l.starts_with("//!"))
        .collect::<Vec<_>>()
        .join("\n");
    for word in ["heartbeat-off", "Acct::marks", "e-seat-hbmark"] {
        assert!(doc.contains(word), "module の doc に {word} が無い");
    }
}

/// 偽の器の tick-state-a の字の proj-a:0.1 の行の欄 heartbeat= を `words`（heartbeat= と heartbeat_by= の字）に替える。
fn hbon_acct_row(place: &Place, words: &str) {
    let out = place.root.join("out/tick-state-a");
    let text: String = fs::read_to_string(&out)
        .expect("tick の出力の字")
        .lines()
        .map(|l| {
            let l = if l.contains("target=proj-a:0.1 ") {
                l.split(' ')
                    .filter(|t| !t.starts_with("heartbeat_by="))
                    .map(|t| {
                        if t.starts_with("heartbeat=") {
                            words.to_string()
                        } else {
                            t.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                l.to_string()
            };
            format!("{l}\n")
        })
        .collect();
    fs::write(&out, text).expect("tick の出力の字を替える");
}

#[test]
fn hbon_acct_reads_tick_word() {
    let place = Place::new("hbon-acct", false);
    hbon_acct_row(&place, "heartbeat=off heartbeat_by=group");
    let acct = place.acct();
    let count = || place.calls("scribe2").len();
    let on = place.state("state-a").join("seat/proj-a_0.1/heartbeat-on");
    assert_eq!(acchold_heartbeat(&acct.doc(NOW)), Reading::Known(false));
    let first = count();
    assert!(first > 0, "器を撃たない");
    fs::write(&on, "").expect("明示の on の記録を置く");
    hbon_acct_row(&place, "heartbeat=on heartbeat_by=explicit");
    assert_eq!(
        acchold_heartbeat(&acct.doc(NOW)),
        Reading::Known(true),
        "明示の on の記録の後の読み"
    );
    assert_eq!(count(), first + 2, "{:?}", place.calls("scribe2"));
    assert_eq!(acchold_heartbeat(&acct.doc(NOW)), Reading::Known(true));
    acct.marks();
    assert_eq!(count(), first + 2, "印が動かなければ持ち回す");
    fs::remove_file(&on).expect("明示の on の記録を消す");
    assert_eq!(
        acchold_heartbeat(&acct.doc(NOW)),
        Reading::Known(true),
        "file の有無でなく行の字で読む"
    );
    assert_eq!(count(), first + 4, "{:?}", place.calls("scribe2"));
}

#[test]
fn acchold_marks_keep_last_gather() {
    let place = Place::new("acchold-marks", false);
    let acct = place.acct();
    acct.doc(NOW);
    let first = acct.marks();
    assert!(!first.is_empty());
    let shots = || (place.calls("scribe2"), place.calls("bd"));
    let after_doc = shots();
    thread::sleep(HOLD + Duration::from_millis(300));
    for _ in 0..2 {
        assert_eq!(acct.marks(), first, "最後の集めの印の一覧");
    }
    assert_eq!(shots(), after_doc, "marks は器と bd を撃たない");
}
