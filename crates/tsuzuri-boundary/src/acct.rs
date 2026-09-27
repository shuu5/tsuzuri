//! account board の読み（便 e-acct）。群の宣言（引数の state dir の host.toml）の anchor ごとに、git の
//! `-C <anchor> config --get scribe2.statedir` で state dir を引き、state dir ごとに器の
//! `seat tick status --state-dir <dir>` と `doctor --state-dir <dir>` を 1 回だけ撃ち、口座は引数の state dir で
//! `fleet usage --show --state-dir <dir>` を 1 回、猶予は `rules get seat.move_grace_s` を 1 回撃つ。
//! 台帳は anchor ごとに着地済みの台帳の読み（`Source`）で読む。子 process はどれも `capture` で撃ち、5 秒で返らなければ読めない。
//! 読む file は state dir ごとの event log と、doctor の orchestrator の席の dir の state.jsonl・tick-last・move-signal と、
//! 群の記録（`<引数の state dir の親>/scribe2-host/groups` の下と、その下の history の下）。file は書かない。
//! state dir が引けない anchor の project は器の出力と file と台帳を読まない。集めた字は 5 秒のあいだ持ち回す。
//! 口の登録と変化の知らせへの印の足しは、つなぐ行 h-wire が行う。

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::Instant;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::account::AccountDoc;
use tsuzuri_core::account::host::{HostTexts, ORCHESTRATOR, RECORD_KIND, declaration};
use tsuzuri_core::account::project::{self, ProjectTexts};

use crate::server::ledger::{Source, capture};
use crate::server::runs::EVENTS_LOG;
use crate::server::seat::{
    DOCTOR_ARGS, GROUPS_DIR, HOLD, HOST_TOML, SCRIBE2_TIMEOUT, STATE_LOG, Seat, TICK_ARGS,
    TICK_LAST, USAGE_ARGS,
};

/// 既定の git の program の名。
pub const GIT: &str = "git";

/// git に渡す引数の列の後ろ（前に `-C <anchor>` が付く）。
pub const GIT_ARGS: [&str; 3] = ["config", "--get", "scribe2.statedir"];

/// 猶予の秒の出力の引数の列。
pub const GRACE_ARGS: [&str; 3] = ["rules", "get", "seat.move_grace_s"];

/// 席の移動の合図の file（席の dir の下）。
pub const MOVE_SIGNAL: &str = "move-signal";

/// 合図の休みの印の file（席の dir の下・変化の印にだけ使う）。
pub const HEARTBEAT_OFF: &str = "heartbeat-off";

/// 群の記録の履歴の dir（群の記録の dir の下）。
pub const HISTORY_DIR: &str = "history";

/// doctor の席の行の頭。
const SEAT_PREFIX: &str = "seat:";

/// 集めた字（電文の材料と、変化の印の file）。
#[derive(Debug, Clone, Default)]
struct Texts {
    host: HostTexts,
    projects: BTreeMap<String, ProjectTexts>,
    grace: Option<String>,
    marks: Vec<PathBuf>,
}

/// account board の読みの出所（器・git・bd の program と、引数の state dir と cwd）と、持ち回しの字。
#[derive(Debug)]
pub struct Acct {
    scribe2: OsString,
    git: OsString,
    bd: OsString,
    state_dir: PathBuf,
    cwd: PathBuf,
    held: Mutex<Option<(Instant, Texts)>>,
}

impl Acct {
    pub fn new(
        scribe2: impl Into<OsString>,
        git: impl Into<OsString>,
        bd: impl Into<OsString>,
        state_dir: impl Into<PathBuf>,
        cwd: impl Into<PathBuf>,
    ) -> Acct {
        Acct {
            scribe2: scribe2.into(),
            git: git.into(),
            bd: bd.into(),
            state_dir: state_dir.into(),
            cwd: cwd.into(),
            held: Mutex::new(None),
        }
    }

    /// 器の program。
    pub fn scribe2(&self) -> &OsStr {
        &self.scribe2
    }

    /// 引数の state dir（群の宣言の host.toml の置き場）。
    pub fn host_state_dir(&self) -> &Path {
        &self.state_dir
    }

    /// 子 process の cwd。
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    /// anchor の state dir（git の返した 1 行の前後の空白を除いた字・git が落ちる・5 秒で返らない・空なら None）。
    pub fn state_dir(&self, anchor: &Path) -> Option<PathBuf> {
        let mut args: Vec<&OsStr> = vec![OsStr::new("-C"), anchor.as_os_str()];
        args.extend(GIT_ARGS.iter().map(OsStr::new));
        let out = capture(&self.git, args, &self.cwd, SCRIBE2_TIMEOUT)?;
        let text = String::from_utf8(out).ok()?;
        let dir = text.trim();
        (!dir.is_empty()).then(|| PathBuf::from(dir))
    }

    /// 電文（持ち回しの字を中核の組み立ての入口に渡す・時計は読まない）。
    pub fn doc(&self, now: EpochSecs) -> AccountDoc {
        let texts = self.texts();
        project::doc(&texts.host, &texts.projects, texts.grace.as_deref(), now)
    }

    /// 変化の印の file（state dir ごとの event log・orchestrator の席の state.jsonl と tick-last と heartbeat-off・
    /// 群の今の記録）。
    pub fn marks(&self) -> Vec<PathBuf> {
        self.texts().marks
    }

    /// 持ち回しの字（`HOLD` を過ぎていれば集め直す・集めるあいだは次の要求を待たせる）。
    fn texts(&self) -> Texts {
        let mut held = self.held.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((at, texts)) = held.as_ref()
            && at.elapsed() < HOLD
        {
            return texts.clone();
        }
        let texts = self.gather();
        *held = Some((Instant::now(), texts.clone()));
        texts
    }

    /// 器に `head` の後に `--state-dir <dir>` を付けて撃つ。
    fn shoot_in(&self, head: &[&str], dir: &Path) -> Option<String> {
        let mut args: Vec<&OsStr> = head.iter().map(OsStr::new).collect();
        args.push(OsStr::new("--state-dir"));
        args.push(dir.as_os_str());
        self.shoot(args)
    }

    fn shoot<I, S>(&self, args: I) -> Option<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let out = capture(&self.scribe2, args, &self.cwd, SCRIBE2_TIMEOUT)?;
        String::from_utf8(out).ok()
    }

    /// 群の記録の dir（引数の state dir に親が無ければ None）。
    fn groups_dir(&self) -> Option<PathBuf> {
        let parent = self.state_dir.parent()?;
        Some(
            GROUPS_DIR
                .iter()
                .fold(parent.to_path_buf(), |p, s| p.join(s)),
        )
    }

    /// 子 process を 2 段に並べて撃ち（待ちは 1 段ごとに 1 本分の上限まで）、file を読む。
    /// 1 段目は anchor ごとの git と、口座と猶予と引数の state dir の doctor。2 段目は state dir ごとの
    /// tick status と doctor（引数の state dir の doctor は撃ち直さない）と、state dir の引けた anchor の台帳。
    fn gather(&self) -> Texts {
        let host_toml = read(&self.state_dir.join(HOST_TOML));
        let anchors = anchors(host_toml.as_deref());
        let (dirs, usage, grace, doctor) = thread::scope(|s| {
            let dirs: Vec<_> = anchors
                .iter()
                .map(|a| s.spawn(|| self.state_dir(Path::new(a))))
                .collect();
            let usage = s.spawn(|| self.shoot_in(&USAGE_ARGS, &self.state_dir));
            let grace = s.spawn(|| self.shoot(GRACE_ARGS));
            let doctor = self.shoot_in(&DOCTOR_ARGS, &self.state_dir);
            (
                dirs.into_iter()
                    .map(|h| h.join().ok().flatten())
                    .collect::<Vec<_>>(),
                usage.join().ok().flatten(),
                grace.join().ok().flatten(),
                doctor,
            )
        });
        let mut unique: Vec<&PathBuf> = Vec::new();
        for dir in dirs.iter().flatten() {
            if !unique.contains(&dir) {
                unique.push(dir);
            }
        }
        let (outputs, ledgers) = thread::scope(|s| {
            let outputs: Vec<_> = unique
                .iter()
                .map(|dir| {
                    let tick = s.spawn(|| self.shoot_in(&TICK_ARGS, dir));
                    let doctor = (**dir != self.state_dir)
                        .then(|| s.spawn(|| self.shoot_in(&DOCTOR_ARGS, dir)));
                    (tick, doctor)
                })
                .collect();
            let ledgers: Vec<_> = anchors
                .iter()
                .zip(&dirs)
                .map(|(a, dir)| {
                    dir.as_ref()
                        .map(|_| s.spawn(move || Source::new(a, self.bd.clone()).text()))
                })
                .collect();
            (
                outputs
                    .into_iter()
                    .map(|(tick, d)| {
                        let tick = tick.join().ok().flatten();
                        let d = match d {
                            Some(d) => d.join().ok().flatten(),
                            None => doctor.clone(),
                        };
                        (tick, d)
                    })
                    .collect::<Vec<_>>(),
                ledgers
                    .into_iter()
                    .map(|l| l.and_then(|l| l.join().ok().flatten()))
                    .collect::<Vec<_>>(),
            )
        });
        let output = |dir: &PathBuf| {
            let i = unique.iter().position(|d| *d == dir)?;
            Some(&outputs[i])
        };
        let mut texts = Texts::default();
        for dir in &unique {
            texts.marks.push(events_log(dir));
        }
        for ((anchor, dir), ledger) in anchors.iter().zip(&dirs).zip(ledgers) {
            let Some((dir, (tick, seat_doctor))) = dir.as_ref().and_then(|d| Some((d, output(d)?)))
            else {
                texts
                    .projects
                    .insert(anchor.clone(), ProjectTexts::default());
                continue;
            };
            let seat_dir = seat_doctor
                .as_deref()
                .and_then(|d| orchestrator_target(d, anchor))
                .and_then(|target| {
                    Seat {
                        program: self.scribe2.clone(),
                        state_dir: dir.clone(),
                        target: target.to_string(),
                        cwd: self.cwd.clone(),
                    }
                    .seat_dir()
                });
            if let Some(d) = &seat_dir {
                texts
                    .marks
                    .extend([d.join(STATE_LOG), d.join(TICK_LAST), d.join(HEARTBEAT_OFF)]);
            }
            let in_seat = |name: &str| seat_dir.as_ref().and_then(|d| read(&d.join(name)));
            texts.projects.insert(
                anchor.clone(),
                ProjectTexts {
                    state_dir_known: true,
                    tick_status: tick.clone(),
                    state_log: in_seat(STATE_LOG),
                    tick_last: in_seat(TICK_LAST),
                    move_signal: in_seat(MOVE_SIGNAL),
                    events: read(&events_log(dir)),
                    ledger,
                },
            );
            if let Some(d) = seat_doctor {
                texts.host.seat_doctors.insert(anchor.clone(), d.clone());
            }
        }
        if let Some(groups) = self.groups_dir() {
            if let Some(toml) = host_toml.as_deref() {
                texts.marks.extend(
                    declaration(toml)
                        .groups
                        .iter()
                        .map(|g| groups.join(format!("{}.{RECORD_KIND}", g.name))),
                );
            }
            texts.host.records = files(&groups);
            texts.host.history = files(&groups.join(HISTORY_DIR));
        }
        texts.host.host_toml = host_toml;
        texts.host.usage = usage;
        texts.host.doctor = doctor;
        texts.grace = grace;
        texts
    }
}

fn read(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// state dir の event log。
fn events_log(dir: &Path) -> PathBuf {
    EVENTS_LOG.iter().fold(dir.to_path_buf(), |p, s| p.join(s))
}

/// dir の直の file の名 → 字（dir・名が UTF-8 でない file・読めない file は飛ばす）。
fn files(dir: &Path) -> BTreeMap<String, String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
                .filter_map(|e| Some((e.file_name().into_string().ok()?, read(&e.path())?)))
                .collect()
        })
        .unwrap_or_default()
}

/// 末尾の「/」を除いて同じ path か。
fn same_path(a: &str, b: &str) -> bool {
    a.trim_end_matches('/') == b.trim_end_matches('/')
}

/// 群の宣言の anchor（群の宣言の順・群の中は anchors の配列の順・同じ path は最初の 1 つ）。
fn anchors(host_toml: Option<&str>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for g in host_toml.map(declaration).unwrap_or_default().groups {
        for a in g.anchors.unwrap_or_default() {
            if !out.iter().any(|d| same_path(d, &a)) {
                out.push(a);
            }
        }
    }
    out
}

/// 行の最初の `鍵=値` の値。
fn field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.split_whitespace()
        .filter_map(|t| t.split_once('='))
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v)
}

/// doctor の席の行のうち、役が orchestrator で anchor が同じ path の最初の行の席の名（空なら None）。
pub fn orchestrator_target<'a>(doctor: &'a str, anchor: &str) -> Option<&'a str> {
    doctor
        .lines()
        .filter_map(|l| l.trim().strip_prefix(SEAT_PREFIX))
        .find(|l| {
            field(l, "role") == Some(ORCHESTRATOR)
                && field(l, "anchor").is_some_and(|a| same_path(a, anchor))
        })
        .and_then(|l| field(l, "target"))
        .filter(|t| !t.is_empty())
}
