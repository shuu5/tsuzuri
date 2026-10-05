//! 席の card の読み（口 GET /api/seat）。器の 3 つの出力を子 process で撃ち、器の state dir の
//! file を読むだけで、書かない。撃つ形は `<program> seat tick status --state-dir <dir>`・
//! `<program> doctor --state-dir <dir>`・`<program> fleet usage --show --state-dir <dir>`
//! （cwd は repo の置き場・標準入力は空・標準エラーは捨てる）。usage は --show を必ず付ける（付けないと器が測り直して記録を書く）。
//! 起動できない・rc が 0 でない・UTF-8 でない・5 秒を超えて返さない、のどれでもその出力は読めない（None）。
//! 読む file は `<state dir>/seat/<席の dir>/state.jsonl`・同じ dir の `tick-last`・`<state dir>/host.toml`・
//! 群の記録（`<state dir の親>/scribe2-host/groups/<群の名>.account` と `history/<群の名>.account.*`）。
//! 3 つの出力は持ち回しの表（`Held`・判断の記録 ADR-23 の決定 (2)(3)）を通り、器の頭ごとに
//! 入力の印（`input_marks`・席の target に依らない）が撃つ前と同じで上限（`ceiling`）の内なら撃たない。tick status は `HOLD`（5 秒）、
//! doctor は `DOCTOR_HOLD`（30 秒）・usage は `SLOW_HOLD`（30 秒）。file の読みは持ち回さず要求のたびに読む。
//! heartbeat-off と heartbeat-on は読まず印にだけ使う（合図の値は tick status の席の行の欄 heartbeat= の字で読む）。
//! 席の target か state dir が無ければ器を撃たず、読む欄が全部「まだ分からない」の card を返す。

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::seat::SeatCard;
use tsuzuri_core::seat::{self as core, SeatTexts};

use super::held::Held;
use super::ledger::capture;
use super::runs::EVENTS_LOG;
use crate::acct::{HEARTBEAT_OFF, HEARTBEAT_ON};

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

/// 合図の健康の出力を持ち回す長さ。
pub const HOLD: Duration = Duration::from_secs(5);

/// 残量の出力と境界の acct の規則の行の読み（rules get）を持ち回す長さ（入力の印が動けば、この中でも撃ち直す）。
pub const SLOW_HOLD: Duration = Duration::from_secs(30);

/// doctor の出力を持ち回す長さ（入力の印が動けば、この中でも撃ち直す）。
/// 判断の記録 ADR-26 の決定 (2) で 60 秒にし、撤退の条件 (1) で 30 秒に戻した（残量の `SLOW_HOLD` と別の定数のまま）。
pub const DOCTOR_HOLD: Duration = Duration::from_secs(30);

/// 席の dir の file（状態の記録と合図の最後の判定と器の account の記録）。
pub const STATE_LOG: &str = "state.jsonl";
pub const TICK_LAST: &str = "tick-last";
pub const ACCOUNT: &str = "account";

/// 席の dir の親の dir の名（state dir の下）。
pub const SEAT_DIR: &str = "seat";

/// 席の dir の合図の梯子の記録と席の移動の合図の file（tick status の印にだけ使う）。
pub const POINTER_LADDER: &str = "pointer-ladder";
pub const MOVE_SIGNAL: &str = "move-signal";

/// 席の dir ごとに印にする file（tick status は合図の時計と梯子と移動の合図まで・doctor は停止と明示の on と account だけ）。
pub const TICK_SEAT_MARKS: [&str; 6] = [
    TICK_LAST,
    POINTER_LADDER,
    HEARTBEAT_OFF,
    HEARTBEAT_ON,
    MOVE_SIGNAL,
    ACCOUNT,
];
pub const DOCTOR_SEAT_MARKS: [&str; 3] = [HEARTBEAT_OFF, HEARTBEAT_ON, ACCOUNT];

/// 群の記録の dir の下で器が読む file の名の終わり（lock と .judged は器が書くが読まない）。
pub const GROUP_READ_SUFFIXES: [&str; 2] = [".account", ".refused"];

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
        plain(&name).then(|| self.state_dir.join(SEAT_DIR).join(name))
    }

    /// 変化の印の file（状態の記録と合図の最後の判定と停止の記録と明示の on の記録）。
    pub fn marks(&self) -> Vec<PathBuf> {
        self.seat_dir()
            .map(|d| {
                vec![
                    d.join(STATE_LOG),
                    d.join(TICK_LAST),
                    d.join(HEARTBEAT_OFF),
                    d.join(HEARTBEAT_ON),
                ]
            })
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
            .filter_map(|p| read_file(&p))
            .collect()
    }

    /// 3 つの出力を並べて撃ち（待ちは 1 本分の上限まで）、file を読む。
    pub fn gather(&self) -> SeatTexts {
        self.gather_with(|head| self.shoot(head))
    }

    /// `gather` と同じ組みで、3 つの出力を持ち回しの表を通して読む（印が動くか上限を過ぎた出力だけ撃ち直す）。
    pub fn gather_held(&self, held: &Held) -> SeatTexts {
        self.gather_with(|head| read_held(held, self, head))
    }

    /// 3 つの出力を `read`（器の頭 → 出力の字）で並べて読み、file を読む。
    fn gather_with(&self, read: impl Fn(&[&str]) -> Option<String> + Sync) -> SeatTexts {
        let (tick_status, doctor, usage) = thread::scope(|s| {
            let tick = s.spawn(|| read(&TICK_ARGS));
            let doctor = s.spawn(|| read(&DOCTOR_ARGS));
            let usage = read(&USAGE_ARGS);
            (
                tick.join().ok().flatten(),
                doctor.join().ok().flatten(),
                usage,
            )
        });
        let dir = self.seat_dir();
        let host_toml = read_file(&self.state_dir.join(HOST_TOML));
        let group = anchor(&self.target, doctor.as_deref())
            .zip(host_toml.as_deref())
            .and_then(|(a, h)| core::group_name(h, &a));
        SeatTexts {
            tick_status,
            doctor,
            usage,
            state_log: dir.as_ref().and_then(|d| read_file(&d.join(STATE_LOG))),
            tick_last: dir.as_ref().and_then(|d| read_file(&d.join(TICK_LAST))),
            host_toml,
            records: group.map(|g| self.records(&g)).unwrap_or_default(),
        }
    }
}

fn read_file(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// 器の頭ごとの持ち回しの上限（tick status は `HOLD`・doctor は `DOCTOR_HOLD`・残量は `SLOW_HOLD`・知らない頭は `HOLD`）。
pub fn ceiling(head: &[&str]) -> Duration {
    if head == DOCTOR_ARGS {
        DOCTOR_HOLD
    } else if head == USAGE_ARGS {
        SLOW_HOLD
    } else {
        HOLD
    }
}

/// 器の頭ごとの入力の印の file（器の CLI が読む file だけ・知らない頭は空・席の target には依らない）。
/// 残量は host.toml と fleet/events.jsonl。tick status と doctor はそれに、state dir の下の seat の dir そのものと、
/// その下の全部の席の dir（名の順・dir でない file は除く）ごとの記録（tick status は `TICK_SEAT_MARKS`・doctor は
/// `DOCTOR_SEAT_MARKS`）と、群の記録の dir の .account と .refused の file（名の順）を足す（判断の記録 ADR-23 の決定 (2)）。
/// tick-last は合図の時計が 15 秒ごとに書き替えるので doctor には入れない。
pub fn input_marks(seat: &Seat, head: &[&str]) -> Vec<PathBuf> {
    let names: &[&str] = if head == USAGE_ARGS {
        &[]
    } else if head == TICK_ARGS {
        &TICK_SEAT_MARKS
    } else if head == DOCTOR_ARGS {
        &DOCTOR_SEAT_MARKS
    } else {
        return Vec::new();
    };
    let events = EVENTS_LOG
        .iter()
        .fold(seat.state_dir.clone(), |p, s| p.join(s));
    let mut out = vec![seat.state_dir.join(HOST_TOML), events];
    if head == USAGE_ARGS {
        return out;
    }
    let seats = seat.state_dir.join(SEAT_DIR);
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&seats)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
                .map(|e| e.path())
                .collect()
        })
        .unwrap_or_default();
    dirs.sort();
    out.push(seats);
    for dir in dirs {
        out.extend(names.iter().map(|n| dir.join(n)));
    }
    if let Some(groups) = seat.groups_dir() {
        let mut files: Vec<PathBuf> = std::fs::read_dir(groups)
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| {
                        e.file_name()
                            .to_str()
                            .is_some_and(|n| GROUP_READ_SUFFIXES.iter().any(|s| n.ends_with(s)))
                    })
                    .map(|e| e.path())
                    .collect()
            })
            .unwrap_or_default();
        files.sort();
        out.extend(files);
    }
    out
}

/// 器の頭の出力を持ち回しの表を通して読む（鍵は program と引数の列と cwd・同じ表を分け合う席は state dir が鍵に入る）。
pub fn read_held(held: &Held, seat: &Seat, head: &[&str]) -> Option<String> {
    let mut key = vec![seat.program.clone()];
    key.extend(seat.argv(head));
    key.push(seat.cwd.clone().into_os_string());
    held.get(&key, &input_marks(seat, head), ceiling(head), || {
        seat.shoot(head)
    })
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

/// 席の読み（出所が無ければ器を撃たない）と、持ち回しの表。
#[derive(Debug)]
pub struct Seats {
    /// 席の target（引数 --seat・省けば空の字）。
    target: String,
    seat: Option<Seat>,
    /// 出力の持ち回しの表（server が 1 つ作って渡す）。
    held: Held,
    /// 集めるあいだ次の要求を待たせる錠（同じ出力を要求ごとに二重に撃たない）。
    gate: Mutex<()>,
}

impl Seats {
    /// 席の target と state dir の両方が在るときだけ出所を持つ。
    pub fn new(
        program: &OsString,
        state_dir: Option<&Path>,
        target: Option<&str>,
        cwd: &Path,
        held: Held,
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
            held,
            gate: Mutex::new(()),
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

    /// 集めた字（3 つの出力は表を通す・集めるあいだは次の要求を待たせる）。
    /// 印は撃つ前に取るので、撃つ途中の変化は次の読みで撃ち直す。
    fn texts(&self, seat: &Seat) -> SeatTexts {
        let _gate = self.gate.lock().unwrap_or_else(|e| e.into_inner());
        seat.gather_held(&self.held)
    }
}
