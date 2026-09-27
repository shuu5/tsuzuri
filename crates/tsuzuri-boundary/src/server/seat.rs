//! 席の card の読み（口 GET /api/seat・便 e-seat）。器の 3 つの出力を子 process で撃ち、器の state dir の
//! file を読むだけで、書かない。撃つ形は `<program> seat tick status --state-dir <dir>`・
//! `<program> doctor --state-dir <dir>`・`<program> fleet usage --show --state-dir <dir>`
//! （cwd は repo の置き場・標準入力は空・標準エラーは捨てる）。usage は --show を必ず付ける（付けないと器が測り直して記録を書く）。
//! 起動できない・rc が 0 でない・UTF-8 でない・5 秒を超えて返さない、のどれでもその出力は読めない（None）。
//! 読む file は `<state dir>/seat/<席の dir>/state.jsonl`・同じ dir の `tick-last`・`<state dir>/host.toml`・
//! 群の記録（`<state dir の親>/scribe2-host/groups/<群の名>.account` と `history/<群の名>.account.*`）。
//! 3 つの出力と file の読みは 5 秒のあいだ持ち回す（要求のたびに器を撃たない）。
//! 席の target か state dir が無ければ器を撃たず、読む欄が全部「まだ分からない」の card を返す。

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::seat::SeatCard;
use tsuzuri_core::seat::{self as core, SeatTexts};

use super::ledger::capture;

/// 口の path。
pub const PATH: &str = "/api/seat";

/// 合図の健康の出力の引数の頭（この後に `--state-dir <dir>` が続く）。
pub const TICK_ARGS: [&str; 3] = ["seat", "tick", "status"];

/// doctor の出力の引数の頭。
pub const DOCTOR_ARGS: [&str; 1] = ["doctor"];

/// 残量の出力の引数の頭（--show を必ず付ける）。
pub const USAGE_ARGS: [&str; 3] = ["fleet", "usage", "--show"];

/// 器の出力が返すまでの上限（要件 NFR2 の上限）。越えれば止めて読めない。
pub const SCRIBE2_TIMEOUT: Duration = Duration::from_secs(5);

/// 出力と file の読みを持ち回す長さ。
pub const HOLD: Duration = Duration::from_secs(5);

/// 席の dir の file（状態の記録と合図の最後の判定）。
pub const STATE_LOG: &str = "state.jsonl";
pub const TICK_LAST: &str = "tick-last";

/// 群の宣言の file（state dir の下）。
pub const HOST_TOML: &str = "host.toml";

/// 群の記録の dir（state dir の親の下の path の区切りの列）。
pub const GROUPS_DIR: [&str; 2] = ["scribe2-host", "groups"];

/// 席の読みの出所（器の CLI・state dir・席の target・cwd にする repo の置き場）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seat {
    pub program: OsString,
    pub state_dir: PathBuf,
    pub target: String,
    pub cwd: PathBuf,
}

/// path の 1 区切りとして使える名か（空でなく、「/」を含まず、「.」で始まらない）。
fn plain(name: &str) -> bool {
    !name.is_empty() && !name.contains('/') && !name.starts_with('.')
}

impl Seat {
    /// 器に渡す引数の列（`head` の後に `--state-dir <dir>`）。
    pub fn argv(&self, head: &[&str]) -> Vec<OsString> {
        let mut args: Vec<OsString> = head.iter().map(OsString::from).collect();
        args.push("--state-dir".into());
        args.push(self.state_dir.clone().into_os_string());
        args
    }

    /// 席の dir（席の名の「:」を「_」に替えた字・path の区切りにならない名なら None）。
    pub fn seat_dir(&self) -> Option<PathBuf> {
        let name = self.target.replace(':', "_");
        plain(&name).then(|| self.state_dir.join("seat").join(name))
    }

    /// 変化の印の file（状態の記録と合図の最後の判定）。
    pub fn marks(&self) -> Vec<PathBuf> {
        self.seat_dir()
            .map(|d| vec![d.join(STATE_LOG), d.join(TICK_LAST)])
            .unwrap_or_default()
    }

    /// 群の記録の dir（state dir に親が無ければ None）。
    pub fn groups_dir(&self) -> Option<PathBuf> {
        let parent = self.state_dir.parent()?;
        Some(
            GROUPS_DIR
                .iter()
                .fold(parent.to_path_buf(), |p, s| p.join(s)),
        )
    }

    fn shoot(&self, head: &[&str]) -> Option<String> {
        let out = capture(&self.program, self.argv(head), &self.cwd, SCRIBE2_TIMEOUT)?;
        String::from_utf8(out).ok()
    }

    /// 群の記録の字（今の記録と、過去の記録を file の名の順に・読めない file は飛ばす）。
    fn records(&self, group: &str) -> Vec<String> {
        let Some(dir) = self.groups_dir().filter(|_| plain(group)) else {
            return Vec::new();
        };
        let current = format!("{group}.account");
        let prefix = format!("{current}.");
        let mut history: Vec<PathBuf> = std::fs::read_dir(dir.join("history"))
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| {
                        e.file_name()
                            .to_str()
                            .is_some_and(|n| n.starts_with(&prefix))
                    })
                    .map(|e| e.path())
                    .collect()
            })
            .unwrap_or_default();
        history.sort();
        std::iter::once(dir.join(current))
            .chain(history)
            .filter_map(|p| read(&p))
            .collect()
    }

    /// 3 つの出力を並べて撃ち（待ちは 1 本分の上限まで）、file を読む。
    pub fn gather(&self) -> SeatTexts {
        let (tick_status, doctor, usage) = thread::scope(|s| {
            let tick = s.spawn(|| self.shoot(&TICK_ARGS));
            let doctor = s.spawn(|| self.shoot(&DOCTOR_ARGS));
            let usage = self.shoot(&USAGE_ARGS);
            (
                tick.join().ok().flatten(),
                doctor.join().ok().flatten(),
                usage,
            )
        });
        let dir = self.seat_dir();
        let host_toml = read(&self.state_dir.join(HOST_TOML));
        let group = anchor(&self.target, doctor.as_deref())
            .zip(host_toml.as_deref())
            .and_then(|(a, h)| core::group_name(h, &a));
        SeatTexts {
            tick_status,
            doctor,
            usage,
            state_log: dir.as_ref().and_then(|d| read(&d.join(STATE_LOG))),
            tick_last: dir.as_ref().and_then(|d| read(&d.join(TICK_LAST))),
            host_toml,
            records: group.map(|g| self.records(&g)).unwrap_or_default(),
        }
    }
}

fn read(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// 席の anchor（doctor の席の行から）。
fn anchor(target: &str, doctor: Option<&str>) -> Option<String> {
    doctor.and_then(|d| core::anchor(d, target))
}

/// 集めた字から席の card を組む（anchor は doctor の席の行から取る）。
pub fn card(target: &str, texts: &SeatTexts, now: EpochSecs) -> SeatCard {
    let anchor = anchor(target, texts.doctor.as_deref());
    core::card(target, anchor.as_deref(), texts, now)
}

/// 席の読み（出所が無ければ器を撃たない）と、持ち回しの字。
#[derive(Debug)]
pub struct Seats {
    /// 席の target（引数 --seat・省けば空の字）。
    target: String,
    seat: Option<Seat>,
    held: Mutex<Option<(Instant, SeatTexts)>>,
}

impl Seats {
    /// 席の target と state dir の両方が在るときだけ出所を持つ。
    pub fn new(
        program: &OsString,
        state_dir: Option<&Path>,
        target: Option<&str>,
        cwd: &Path,
    ) -> Seats {
        let seat = target.zip(state_dir).map(|(target, state_dir)| Seat {
            program: program.clone(),
            state_dir: state_dir.to_path_buf(),
            target: target.to_string(),
            cwd: cwd.to_path_buf(),
        });
        Seats {
            target: target.unwrap_or_default().to_string(),
            seat,
            held: Mutex::new(None),
        }
    }

    /// 変化の印の file（出所が無ければ空）。
    pub fn marks(&self) -> Vec<PathBuf> {
        self.seat.as_ref().map(Seat::marks).unwrap_or_default()
    }

    /// 席の card（出所が無ければ、読む欄が全部「まだ分からない」の card）。
    pub fn card(&self, now: EpochSecs) -> SeatCard {
        self.known(now)
            .unwrap_or_else(|| card(&self.target, &SeatTexts::default(), now))
    }

    /// 出所が在るときだけ席の card（次の一手の口が使う）。
    pub fn known(&self, now: EpochSecs) -> Option<SeatCard> {
        let seat = self.seat.as_ref()?;
        Some(card(&seat.target, &self.texts(seat), now))
    }

    /// 持ち回しの字（`HOLD` を過ぎていれば集め直す・集めるあいだは次の要求を待たせる）。
    fn texts(&self, seat: &Seat) -> SeatTexts {
        let mut held = self.held.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((at, texts)) = held.as_ref()
            && at.elapsed() < HOLD
        {
            return texts.clone();
        }
        let texts = seat.gather();
        *held = Some((Instant::now(), texts.clone()));
        texts
    }
}
