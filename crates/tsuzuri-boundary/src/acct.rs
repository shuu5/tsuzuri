//! account board の読み（便 e-acct）。群の宣言（引数の state dir の host.toml）の anchor ごとに、git の
//! `-C <anchor> config --get scribe2.statedir` で state dir を、`-C <anchor> config --get tsuzuri.boardport` で
//! project board の port を引き（電文の行には port だけを置く・便 h-board-url）、state dir ごとに器の
//! `seat tick status --state-dir <dir>` と `doctor --state-dir <dir>` を 1 回だけ撃ち、口座は引数の state dir で
//! `fleet usage --show --state-dir <dir>` を 1 回、窓ごとの逼迫の閾値は
//! `rules get <id>`（`CAP_ROWS` の 3 行・`--state-dir` を付けない）を行ごとに 1 回撃つ（便 c-acct-thr）。
//! 退避までの残り秒は tick status の席の行の器の欄の写し（中核の席の card）で、rules 行も合図の file も読まない。
//! 台帳は anchor ごとに着地済みの台帳の読み（`Source`）で読む。子 process はどれも `capture` で撃ち、5 秒で返らなければ読めない。
//! anchor ごとの `Source` は持ち続けるので、台帳の読みが落ちても最後に読めた字を `READ_HOLD` まで返す（行 e-hold）。
//! 読む file は state dir ごとの event log と、doctor の orchestrator の席の dir の state.jsonl・tick-last と
//! （同じ dir の heartbeat-off・heartbeat-on は読まず、有無と更新時刻を印にだけ使う）、
//! 重ならない state dir ごとの doctor の登録の行の全部の席の dir の state.jsonl（休止中の席の材料・印にしない・行 c-dormant）と、
//! 群の記録（`<引数の state dir の親>/scribe2-host/groups` の下と、その下の history の下）と、口座の線の材料の
//! 引数の state dir の event log（印にしない・窓の棒と同じ周で新しくなる・行 c-acct-spark）。file は書かない。
//! state dir が引けない anchor の project は器の出力と file と台帳を読まない。
//! 器の出力は持ち回しの表（`Held`・行 e-held-acct・判断の記録 ADR-23 の決定 (3)）で出力ごとに持つ。usage は席の card の読みと
//! 同じ `read_held` で読み（鍵も印も同じ）、tick status と doctor は席の card の読みの鍵の末に字 `ACCT_KEY` を足した鍵と、
//! state dir の全部の席の dir の入力の印に event log を足した印（`vessel_marks`）で持ち、rules get は印の無い鍵で `SLOW_HOLD` 持つ。
//! file の読みは要求ごと。`marks` は最後の集めの印の一覧を返し、一度も集めていない時だけ集める（行 e-acct-hbmark）。
//! 集めのあいだは錠（`gate`）で次の要求を待たせる。
//! git の読み（state dir と board の port）は `GIT_HOLD` のあいだ持ち回す（宣言の anchor の列が変われば撃ち直す）。
//! 台帳は bd を撃つ前に台帳の印（`Source::mark`）を取り、印が同じで前の読みが読めていれば bd を撃たない（行 a-lean）。
//! 読めなかった読みは印が同じでも `FAILED_HOLD` の間は撃ち直さない。
//! 自分の repo の anchor の台帳は `with_own` の Source（server の見張りの読み）を分け合い、bd を撃たない。
//! 口の登録と変化の知らせへの印の足しは、つなぐ行 h-wire が行う。

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::account::AccountDoc;
use tsuzuri_core::account::host::{CAP_ROWS, HostTexts, ORCHESTRATOR, RECORD_KIND, declaration};
use tsuzuri_core::account::project::{self, ProjectTexts};
use tsuzuri_core::account::project_name;

use crate::server::held::{FAILED_HOLD, Held};
use crate::server::ledger::{Got, Mark, Source, capture};
use crate::server::runs::EVENTS_LOG;
use crate::server::seat::{
    DOCTOR_ARGS, GROUPS_DIR, HOST_TOML, SCRIBE2_TIMEOUT, SLOW_HOLD, STATE_LOG, Seat, TICK_ARGS,
    TICK_LAST, USAGE_ARGS, ceiling, input_marks, read_held,
};

/// 既定の git の program の名。
pub const GIT: &str = "git";

/// git に渡す引数の列の後ろ（前に `-C <anchor>` が付く）。
pub const GIT_ARGS: [&str; 3] = ["config", "--get", "scribe2.statedir"];

/// project board の port を引く git の引数の列の後ろ（前に `-C <anchor>` が付く・anchor の .git/config の鍵）。
pub const BOARD_ARGS: [&str; 3] = ["config", "--get", "tsuzuri.boardport"];

/// git の読み（anchor ごとの state dir と board の port）を集め直しのあいだ持ち回す時間（行 a-lean）。
pub const GIT_HOLD: Duration = Duration::from_secs(60);

/// 窓ごとの逼迫の閾値の出力の引数の列の頭（後ろに `CAP_ROWS` の行の id が付く）。
pub const CAP_ARGS: [&str; 2] = ["rules", "get"];

/// 合図の休みの印の file（席の dir の下・変化の印にだけ使う）。
pub const HEARTBEAT_OFF: &str = "heartbeat-off";

/// 合図の明示の on の印の file（席の dir の下・区画の既定が off の席を on にした記録・変化の印にだけ使う・行 e-hb-on-mark）。
pub const HEARTBEAT_ON: &str = "heartbeat-on";

/// 群の記録の履歴の dir（群の記録の dir の下）。
pub const HISTORY_DIR: &str = "history";

/// doctor の席の行の頭。
const SEAT_PREFIX: &str = "seat:";

/// 集めた字（電文の材料と、変化の印の file）。
#[derive(Debug, Clone, Default)]
struct Texts {
    host: HostTexts,
    projects: BTreeMap<String, ProjectTexts>,
    /// 宣言の anchor → git の返した project board の port の字（字の無い anchor は入れない）。
    boards: BTreeMap<String, String>,
    marks: Vec<PathBuf>,
    /// 台帳の読みが落ちた project の最後に読めた時刻のうち最も古い値（どれも読めれば None・行 e-hold）。
    stale: Option<Instant>,
}

/// tick status と doctor の持ち回しの鍵の末に足す字（席の card の読みと鍵と印を分ける）。
pub const ACCT_KEY: &str = "acct";

/// git の読みの字（宣言の anchor の順に、state dir と board の port の字）。
type GitTexts = (Vec<Option<PathBuf>>, Vec<Option<String>>);

/// git の読みを取った時刻と、読んだ宣言の anchor の列と、git の読みの字（一度も読んでいなければ None）。
type Gits = Option<(Instant, Vec<String>, GitTexts)>;

/// account board の読みの出所（器・git・bd の program と、引数の state dir と cwd）と、持ち回しの字。
#[derive(Debug)]
pub struct Acct {
    scribe2: OsString,
    git: OsString,
    bd: OsString,
    state_dir: PathBuf,
    cwd: PathBuf,
    /// 器の出力の持ち回しの表（server が席の card の読みと分け合う・行 e-held-acct）。
    held: Held,
    /// 集めるあいだ次の要求を待たせる錠（同じ出力を二重に撃たない）。
    gate: Mutex<()>,
    /// 最後の集めの印の一覧（一度も集めていなければ None）。
    last_marks: Mutex<Option<Vec<PathBuf>>>,
    /// anchor の字 → 台帳の読みの出所（持ち続けて最後に読めた字を次の gather に残す・行 e-hold）。
    ledgers: Mutex<BTreeMap<String, Source>>,
    /// anchor の字 → 読む前に取った台帳の印と、その読みの結果と、撃つ前の時刻（行 a-lean）。
    seen: Mutex<BTreeMap<String, (Mark, Got, Instant)>>,
    /// 持ち回しの git の読み。
    gits: Mutex<Gits>,
    /// git の読みの持ち回しの時間。
    git_hold: Duration,
    /// 自分の repo の台帳の読みの出所（server の見張りの Source・無ければ None）。
    own: Option<Source>,
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
            held: Held::new(),
            gate: Mutex::new(()),
            last_marks: Mutex::new(None),
            ledgers: Mutex::new(BTreeMap::new()),
            seen: Mutex::new(BTreeMap::new()),
            gits: Mutex::new(None),
            git_hold: GIT_HOLD,
            own: None,
        }
    }

    /// 自分の repo（`Source::repo` と同じ dir の anchor）の台帳を `own` で読む Acct（bd を撃たずに分け合う）。
    pub fn with_own(self, own: Source) -> Acct {
        Acct {
            own: Some(own),
            ..self
        }
    }

    /// 器の出力の持ち回しの表を、席の card の読みと分け合う表に替えた Acct。
    pub fn with_held(self, held: Held) -> Acct {
        Acct { held, ..self }
    }

    /// git の読みの持ち回しの時間を替えた Acct（歯が短い時間で試す）。
    pub fn with_git_hold(self, git_hold: Duration) -> Acct {
        Acct { git_hold, ..self }
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
        self.git_get(anchor, &GIT_ARGS).map(PathBuf::from)
    }

    /// anchor の project board の port の字（git の返した字の前後の空白を除いた字・port かどうかは見ない・
    /// git が落ちる・5 秒で返らない・空なら None）。
    pub fn board(&self, anchor: &Path) -> Option<String> {
        self.git_get(anchor, &BOARD_ARGS)
    }

    /// git に `-C <anchor>` と鍵の引数を渡して撃ち、返した字の前後の空白を除いた字（空なら None）。
    fn git_get(&self, anchor: &Path, tail: &[&str]) -> Option<String> {
        let mut args: Vec<&OsStr> = vec![OsStr::new("-C"), anchor.as_os_str()];
        args.extend(tail.iter().map(OsStr::new));
        let out = capture(&self.git, args, &self.cwd, SCRIBE2_TIMEOUT)?;
        let text = String::from_utf8(out).ok()?;
        let text = text.trim();
        (!text.is_empty()).then(|| text.to_string())
    }

    /// 電文（持ち回しの字を中核の組み立ての入口に渡し、project の行ごとに宣言の anchor の board の port を置く・
    /// 時計は読まない）。
    pub fn doc(&self, now: EpochSecs) -> AccountDoc {
        self.doc_read(now).0
    }

    /// `doc` と同じ電文と、台帳の読みが落ちた project の最後に読めた時刻のうち最も古い値
    /// （どれも読めれば None・行 e-hold）。
    pub fn doc_read(&self, now: EpochSecs) -> (AccountDoc, Option<Instant>) {
        let texts = self.texts();
        let mut doc = project::doc(&texts.host, &texts.projects, now);
        let declared = anchors(texts.host.host_toml.as_deref());
        for (row, anchor) in doc.projects.iter_mut().zip(&declared) {
            if row.name == project_name(anchor) {
                row.board = texts.boards.get(anchor).and_then(|t| board_port(t));
            }
        }
        (doc, texts.stale)
    }

    /// anchor の台帳の読みの出所（`own` の repo と同じ dir ならその clone・表に在ればその clone・
    /// 無ければ作って表に置く）。
    fn ledger(&self, anchor: &str) -> Source {
        if let Some(own) = self
            .own
            .as_ref()
            .filter(|own| same_dir(&own.repo.to_string_lossy(), anchor))
        {
            return own.clone();
        }
        let mut ledgers = self.ledgers.lock().unwrap_or_else(|e| e.into_inner());
        ledgers
            .entry(anchor.to_string())
            .or_insert_with(|| Source::new(anchor, self.bd.clone()))
            .clone()
    }

    /// anchor の台帳の読み（見張りの Source は `got` のまま・でなければ bd を撃つ前に印を取り、前の読みの印と
    /// 同じで前の読みが読めていれば前の読みを返し、読めなかった読み（stale が在る）は `FAILED_HOLD` の内だけ返し、
    /// ほかは撃って印と読みを置く・行 a-lean）。
    fn ledger_got(&self, anchor: &str) -> Got {
        let source = self.ledger(anchor);
        if source.is_watched() {
            return source.got();
        }
        let mark = source.mark();
        let held = self
            .seen
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(anchor)
            .filter(|(was, got, at)| {
                *was == mark && (got.stale.is_none() || at.elapsed() < FAILED_HOLD)
            })
            .map(|(_, got, _)| got.clone());
        if let Some(got) = held {
            return got;
        }
        let at = Instant::now();
        let got = source.got();
        self.seen
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(anchor.to_string(), (mark, got.clone(), at));
        got
    }

    /// 持ち回しの git の読み（取ってから `git_hold` より短く、読んだ anchor の列が同じときだけ）。
    fn held_gits(&self, anchors: &[String]) -> Option<GitTexts> {
        let gits = self.gits.lock().unwrap_or_else(|e| e.into_inner());
        let (at, was, texts) = gits.as_ref()?;
        (at.elapsed() < self.git_hold && was == anchors).then(|| texts.clone())
    }

    /// 変化の印の file（state dir ごとの event log・orchestrator の席の state.jsonl と tick-last と heartbeat-off と
    /// heartbeat-on・群の今の記録）。最後の集めの一覧を返し、一度も集めていない時だけ集める。
    pub fn marks(&self) -> Vec<PathBuf> {
        let last = self
            .last_marks
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        last.unwrap_or_else(|| self.texts().marks)
    }

    /// 集めた字（器の出力は表を通し、file は要求ごとに読む・集めるあいだは次の要求を待たせる・行 e-acct-hbmark）。
    /// 印は撃つ前に取るので、撃つ途中の変化は次の読みで撃ち直す。
    fn texts(&self) -> Texts {
        let _gate = self.gate.lock().unwrap_or_else(|e| e.into_inner());
        let texts = self.gather();
        *self.last_marks.lock().unwrap_or_else(|e| e.into_inner()) = Some(texts.marks.clone());
        texts
    }

    /// 器の出所（program・引数の dir を state dir にした席の読み・cwd）。target は空（席の dir は使わない）。
    fn vessel(&self, dir: &Path) -> Seat {
        Seat {
            program: self.scribe2.clone(),
            state_dir: dir.to_path_buf(),
            target: String::new(),
            cwd: self.cwd.clone(),
        }
    }

    /// 器の tick status か doctor の出力を表を通して読む（鍵は席の card の読みの鍵の末に `ACCT_KEY` を足した列・
    /// 印は `vessel_marks`）。
    fn held_out(&self, head: &[&str], dir: &Path) -> Option<String> {
        let seat = self.vessel(dir);
        let mut key = vec![seat.program.clone()];
        key.extend(seat.argv(head));
        key.push(seat.cwd.clone().into_os_string());
        key.push(ACCT_KEY.into());
        self.held
            .get(&key, &vessel_marks(&seat, head), ceiling(head), || {
                self.shoot_in(head, dir)
            })
    }

    /// 窓ごとの逼迫の閾値の出力を表を通して読む（印の無い鍵で `SLOW_HOLD` の間持つ）。
    fn held_rule(&self, rule: &str) -> Option<String> {
        let mut key = vec![self.scribe2.clone()];
        key.extend(CAP_ARGS.into_iter().chain([rule]).map(OsString::from));
        key.push(self.cwd.clone().into_os_string());
        self.held.get(&key, &[], SLOW_HOLD, || {
            self.shoot(CAP_ARGS.into_iter().chain([rule]))
        })
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
    /// 1 段目は anchor ごとの git（state dir と board の port・持ち回しの読みが在れば撃たない）と、
    /// 口座と閾値の行と引数の state dir の doctor。
    /// 2 段目は state dir ごとの tick status と doctor（引数の state dir の doctor は撃ち直さない）と、
    /// state dir の引けた anchor の台帳（`ledger_got`）。
    fn gather(&self) -> Texts {
        let host_toml = read(&self.state_dir.join(HOST_TOML));
        let anchors = anchors(host_toml.as_deref());
        let held = self.held_gits(&anchors);
        let (dirs, boards, usage, caps, doctor) = thread::scope(|s| {
            let git = held.is_none().then(|| {
                let dirs: Vec<_> = anchors
                    .iter()
                    .map(|a| s.spawn(|| self.state_dir(Path::new(a))))
                    .collect();
                let boards: Vec<_> = anchors
                    .iter()
                    .map(|a| s.spawn(|| self.board(Path::new(a))))
                    .collect();
                (dirs, boards)
            });
            let usage =
                s.spawn(|| read_held(&self.held, &self.vessel(&self.state_dir), &USAGE_ARGS));
            let caps: Vec<_> = CAP_ROWS
                .iter()
                .map(|&(_, rule)| (rule, s.spawn(move || self.held_rule(rule))))
                .collect();
            let doctor = self.held_out(&DOCTOR_ARGS, &self.state_dir);
            let (dirs, boards) = match git {
                Some((dirs, boards)) => {
                    let texts: GitTexts = (
                        dirs.into_iter().map(|h| h.join().ok().flatten()).collect(),
                        boards
                            .into_iter()
                            .map(|h| h.join().ok().flatten())
                            .collect(),
                    );
                    *self.gits.lock().unwrap_or_else(|e| e.into_inner()) =
                        Some((Instant::now(), anchors.clone(), texts.clone()));
                    texts
                }
                None => held.unwrap_or_default(),
            };
            (
                dirs,
                boards,
                usage.join().ok().flatten(),
                caps.into_iter()
                    .filter_map(|(rule, h)| Some((rule.to_string(), h.join().ok().flatten()?)))
                    .collect::<BTreeMap<_, _>>(),
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
                    let tick = s.spawn(|| self.held_out(&TICK_ARGS, dir));
                    let doctor = (**dir != self.state_dir)
                        .then(|| s.spawn(|| self.held_out(&DOCTOR_ARGS, dir)));
                    (tick, doctor)
                })
                .collect();
            let ledgers: Vec<_> = anchors
                .iter()
                .zip(&dirs)
                .map(|(a, dir)| dir.as_ref().map(|_| s.spawn(move || self.ledger_got(a))))
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
                    .map(|l| l.and_then(|l| l.join().ok()))
                    .collect::<Vec<_>>(),
            )
        });
        let output = |dir: &PathBuf| {
            let i = unique.iter().position(|d| *d == dir)?;
            Some(&outputs[i])
        };
        let mut texts = Texts::default();
        for (anchor, board) in anchors.iter().zip(boards) {
            if let Some(board) = board {
                texts.boards.insert(anchor.clone(), board);
            }
        }
        for (dir, (_, seat_doctor)) in unique.iter().zip(&outputs) {
            texts.marks.push(events_log(dir));
            // 休止中の席の材料（登録の行の全部の席の状態の記録・同じ席は最初の state dir の字・印にしない・行 c-dormant）。
            for target in seat_doctor.as_deref().map(seat_targets).unwrap_or_default() {
                if texts.host.seat_logs.contains_key(target) {
                    continue;
                }
                let log = Seat {
                    program: self.scribe2.clone(),
                    state_dir: (*dir).clone(),
                    target: target.to_string(),
                    cwd: self.cwd.clone(),
                }
                .seat_dir()
                .and_then(|d| read(&d.join(STATE_LOG)));
                if let Some(log) = log {
                    texts.host.seat_logs.insert(target.to_string(), log);
                }
            }
        }
        for ((anchor, dir), got) in anchors.iter().zip(&dirs).zip(ledgers) {
            let ledger = got.and_then(|got| {
                if let Some(at) = got.stale {
                    texts.stale = Some(texts.stale.map_or(at, |s| s.min(at)));
                }
                got.text
            });
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
                texts.marks.extend([
                    d.join(STATE_LOG),
                    d.join(TICK_LAST),
                    d.join(HEARTBEAT_OFF),
                    d.join(HEARTBEAT_ON),
                ]);
            }
            let in_seat = |name: &str| seat_dir.as_ref().and_then(|d| read(&d.join(name)));
            texts.projects.insert(
                anchor.clone(),
                ProjectTexts {
                    state_dir_known: true,
                    tick_status: tick.clone(),
                    state_log: in_seat(STATE_LOG),
                    tick_last: in_seat(TICK_LAST),
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
        texts.host.caps = caps;
        texts.host.events = read(&events_log(&self.state_dir));
        texts
    }
}

/// state dir の tick status か doctor の入力の印（state dir の seat の下の全部の席の dir ごとの `input_marks` の和
/// （群の記録の .account と .refused を含む）に、その state dir の event log を足す・path の順・重ねない）。
fn vessel_marks(seat: &Seat, head: &[&str]) -> Vec<PathBuf> {
    let as_target = |target: &str| Seat {
        target: target.to_string(),
        ..seat.clone()
    };
    let mut out = input_marks(&as_target(""), head);
    let dirs = std::fs::read_dir(seat.state_dir.join("seat"))
        .map(|entries| entries.flatten().collect::<Vec<_>>())
        .unwrap_or_default();
    for entry in dirs {
        if let (Ok(t), Some(name)) = (entry.file_type(), entry.file_name().to_str())
            && t.is_dir()
        {
            out.extend(input_marks(&as_target(name), head));
        }
    }
    out.push(events_log(&seat.state_dir));
    out.sort();
    out.dedup();
    out
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

/// project board の port の字の値（前後の空白を除いた字が 1 字以上の ASCII の数字だけで 1 以上 65535 以下ならその値・
/// 符号・コロン・斜線・host の字・中の空白・0・65536 以上は None）。
pub fn board_port(text: &str) -> Option<u16> {
    let digits = text.trim();
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse::<u16>().ok().filter(|p| *p != 0)
}

/// 末尾の「/」を除いて同じ path か。
fn same_path(a: &str, b: &str) -> bool {
    a.trim_end_matches('/') == b.trim_end_matches('/')
}

/// 同じ dir か（`same_path` か、どちらも実体の path が引けて同じ path）。
fn same_dir(a: &str, b: &str) -> bool {
    same_path(a, b)
        || matches!(
            (Path::new(a).canonicalize(), Path::new(b).canonicalize()),
            (Ok(a), Ok(b)) if a == b
        )
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

/// doctor の登録の行（頭が `seat:` の行・役を問わない）の席の名（行の順・空の名は飛ばす）。
fn seat_targets(doctor: &str) -> Vec<&str> {
    doctor
        .lines()
        .filter_map(|l| l.trim().strip_prefix(SEAT_PREFIX))
        .filter_map(|l| field(l, "target"))
        .filter(|t| !t.is_empty())
        .collect()
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
