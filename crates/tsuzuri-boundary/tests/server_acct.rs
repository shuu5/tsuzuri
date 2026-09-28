//! account board の読みの歯（接頭辞 server_acct_・設計ノート surface-base 便 e-acct の完了の条件）。
//! 偽の器・偽の git・偽の bd は、受けた argv（bd は cwd の最後の区切り）を記録の置き場（repo と state dir の外）に
//! 1 行ずつ足し、作業場の out・git・bd の下の決めた字を標準出力へ出す script（file が無ければ rc 1・
//! slow の下に同じ名の印が在れば 8 秒眠る）。器は argv の頭と最後の引数（state dir の最後の区切り）で、
//! git は -C の次の path の最後の区切りで、bd は cwd の最後の区切りで字を選ぶ。
//! anchor は作業場の work の下の dir（proj-a ほか）で、字の中の path は実行の時に組む（行 D-4）。

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::acct::{Acct, BOARD_ARGS, GIT_ARGS, GRACE_ARGS};
use tsuzuri_boundary::server::seat::{HOLD, USAGE_ARGS};
use tsuzuri_contract::account::{AccountDoc, ProjectRow};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::SeatState;
use tsuzuri_core::account::host::HostTexts;
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

const TICK_A: &str = "seat tick status: target=proj-a:0.1 last=1790510390 age=10 healthy=yes heartbeat=on step=5 next=1790510395\n\
seat tick status: target=proj-e:0.1 last=1790510000 age=400 healthy=no heartbeat=off step=5 next=1790510405\n";
const TICK_B: &str = "seat tick status: target=proj-b:0.1 last=1790510390 age=10 healthy=yes heartbeat=on step=5 next=1790510395\n";

const USAGE: &str = "usage: account=acct-1 five_hour=83% resets=2026-09-27T15:00:00Z seven_day=40% resets=none model=opus:12% resets=none\n\
usage: account=acct-2 five_hour=5% resets=none seven_day=30% resets=none model=sonnet:0% resets=none\n\
usage: account=acct-3 unmeasured reason=no-token\n";

const GRACE: &str = "1800\n";

const GROUP_LINES: &str = "group=g-a accounts=acct-1,acct-2 anchors=2 seat-accounts=acct-2 current=acct-1 next=acct-2 refused=-\n\
group=g-b accounts=acct-2,acct-3 anchors=3 seat-accounts=acct-3 current=acct-3 next=none refused=-\n";

const STATE_A: &str = "{\"schema\":1,\"state\":\"idle\",\"event\":\"stop\",\"ts\":1790440000,\"sid\":\"s-1\"}\n\
{\"schema\":1,\"state\":\"busy\",\"event\":\"prompt\",\"ts\":1790500000,\"sid\":\"s-1\"}\n";
const STATE_E: &str =
    "{\"schema\":1,\"state\":\"idle\",\"event\":\"stop\",\"ts\":1790508000,\"sid\":\"s-2\"}\n";
const STATE_B: &str =
    "{\"schema\":1,\"state\":\"busy\",\"event\":\"prompt\",\"ts\":1790505000,\"sid\":\"s-3\"}\n";
/// pipeline の席の記録（読まない）。
const STATE_PIPE: &str =
    "{\"schema\":1,\"state\":\"busy\",\"event\":\"prompt\",\"ts\":1790400000,\"sid\":\"s-9\"}\n";

const EVENTS_A: &str = "{\"schema\":1,\"ts\":\"2026-09-27T11:00:00Z\",\"kind\":\"RunCreated\",\"run\":\"r.1-20260927T110000Z\",\"bead\":\"r.1\",\"host\":\"host-1\",\"actor\":\"machine\",\"stage\":\"Intake\",\"detail\":\"classes:\"}\n\
{\"schema\":1,\"ts\":\"2026-09-27T11:10:00Z\",\"kind\":\"SeatSpawned\",\"run\":\"r.1-20260927T110000Z\",\"bead\":\"r.1\",\"host\":\"host-1\",\"actor\":\"machine\",\"detail\":\"account:acct-2\"}\n\
{\"schema\":1,\"ts\":\"2026-09-27T11:20:00Z\",\"kind\":\"RunStage\",\"run\":\"r.1-20260927T110000Z\",\"bead\":\"r.1\",\"host\":\"host-1\",\"actor\":\"machine\",\"stage\":\"Implemented\"}\n";
const EVENTS_B: &str = "{\"schema\":1,\"ts\":\"2026-09-27T09:00:00Z\",\"kind\":\"RunCreated\",\"run\":\"b.1-20260927T090000Z\",\"bead\":\"b.1\",\"host\":\"host-2\",\"actor\":\"machine\",\"stage\":\"Intake\",\"detail\":\"classes:\"}\n";

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
              fleet) f=\"usage-${last##*/}\" ;; rules) f=grace ;; *) exit 2 ;; esac\n\
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
        self.out("grace", GRACE);
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
    fn expected(
        &self,
        drop: &[&str],
    ) -> (HostTexts, BTreeMap<String, ProjectTexts>, Option<String>) {
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
            ..HostTexts::default()
        };
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
                    move_signal: in_seat("move-signal"),
                    events: self.file(&dir.join("fleet/events.jsonl")),
                    ledger: self.file(&self.root.join("bd").join(p)),
                },
            );
            if let Some(doctor) = doctor {
                host.seat_doctors.insert(anchor, doctor);
            }
        }
        (host, projects, out("grace"))
    }

    /// 中核の組み立ての入口を、読みが集めたはずの字で直に呼んだ電文。
    fn core_doc(&self, drop: &[&str]) -> AccountDoc {
        let (host, projects, grace) = self.expected(drop);
        project::doc(&host, &projects, grace.as_deref(), NOW)
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
        assert_eq!(got.at, NOW);
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
        assert_eq!(row(&got, "proj-a").move_left_s, Some(1200));
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
        // 器は state dir ごとに tick と doctor を 1 回、口座と猶予は 1 回。
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
            "rules get seat.move_grace_s".to_string(),
        ];
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
        assert_eq!(GRACE_ARGS, ["rules", "get", "seat.move_grace_s"]);
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
    let place = Place::new("grace-fail", false);
    place.fail("grace");
    let got = place.acct().doc(NOW);
    assert_eq!(got, place.core_doc(&["grace"]));
    assert!(got.projects.iter().all(|r| r.move_left_s.is_none()));
    let place = Place::new("grace-bad", false);
    fs::write(place.root.join("out/grace"), "soon\n").expect("数でない猶予");
    let got = place.acct().doc(NOW);
    assert!(got.projects.iter().all(|r| r.move_left_s.is_none()));
    assert_eq!(
        row(&got, "proj-a").seat,
        row(&place.core_doc(&[]), "proj-a").seat
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
    assert_eq!(once.0.len(), 6, "5 秒の中の読み: {:?}", once.0);
    assert_eq!(once.1.len(), 10, "{:?}", once.1);
    assert_eq!(once.2.len(), 3, "{:?}", once.2);
    for calls in [&once.0, &once.1, &once.2] {
        let distinct: BTreeSet<&String> = calls.iter().collect();
        assert_eq!(distinct.len(), calls.len(), "argv ごとに 1 回: {calls:?}");
    }
    thread::sleep(HOLD + Duration::from_millis(300));
    acct.doc(NOW);
    let twice = |calls: &[String]| {
        let mut doubled: Vec<String> = calls.iter().flat_map(|c| [c.clone(), c.clone()]).collect();
        doubled.sort();
        doubled
    };
    assert_eq!(place.calls("scribe2"), twice(&once.0), "5 秒の後の読み");
    assert_eq!(place.calls("git"), twice(&once.1));
    assert_eq!(place.calls("bd"), twice(&once.2));
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
        for name in ["state.jsonl", "tick-last", "heartbeat-off"] {
            want.insert(seat.join(name));
        }
    }
    assert_eq!(got, want);
    assert!(
        got.iter().all(|p| !p.ends_with("move-signal")),
        "move-signal は印にしない"
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
