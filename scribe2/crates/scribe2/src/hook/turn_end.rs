//! turn の終わりの止め（設計 docs/design/dialogue-surface.md §12・ADR-0087・FR88 / NFR5）: `SessionStart` が session の置き場へ event log の
//! 長さを開始の位置として 1 度だけ書き（[`start`]）、`Stop` が印の無い turn は台帳を読まずに開始の位置から log の末尾までを読んで、その session の
//! 未仕分けの発話（[`crate::utterance::sorted_of`] が [`Standing::Unsorted`] と判じたもの）のうち未告のものが在る周を 1 度だけ rc 2 で止める
//! （[`stop`]）。止めた周は打刻せず（席は Busy のまま）、続く再入で Idle に戻す。読めない周は止めず [`Reason`] の 1 語で
//! `TurnEndUnjudged` を記帳して今のまま打刻する。`EventKind` と `Case` の match の arm は書かない（`==` と構築だけ）。
//!
//! PreToolUse の門が全部通した周は、台帳の書きか設計の種別の編集を session の置き場の印 `wrote` に足し（[`mark`]）、印の在る turn の
//! `Stop` は未仕分けの止めが無い周に台帳を 1 回だけ読んで、作業中の bead のどれにも turn の最初の書きの分以上の頭の段の定型の行が無ければ
//! 1 度だけ止める（[`stale_of`]・印は判定の前に空にする）。

use super::guard::GUARDED;
use super::ledger_guard::notes::head_time;
use super::ledger_guard::{segments, write_of, WRITES};
use super::role_guard::{locate, PathKind};
use super::utterance::is_runner;
use super::{command, command_of, field, flag_of, stamp, Hooked, FLAG_PLUGIN_ROOT, KEY_FILE, KEY_NOTEBOOK, KEY_SESSION_ID, KEY_TOOL};
use crate::cli_outcome::{Outcome, RC_BROKEN};
use crate::fleet::store::{self, LockPolicy};
use crate::fleet::{cli as fleet_cli, epoch_of, Case, Event, EventKind};
use crate::name::NAME;
use crate::pipe::declaration::path_kinds::PathKinds;
use crate::polarity::{OnFailure, Polarity, Timing};
use crate::rules::manifest::Manifest;
use crate::seat::ledger::{self, LedgerError};
use crate::seat::recent::{self, Bead, WIP_HEADS};
use crate::seat::state::{now_secs, Event as Stamped};
use crate::utterance::{sorted_of, Standing};
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// この境界の極性: turn の終わりの時点で止め、読めない周は止めずに記帳して通す。
pub const POLARITY: Polarity = Polarity {
    timing: Timing::InLoop,
    on_failure: OnFailure::FailOpen,
};

/// session の置き場の dir 名（`<state_dir>/session/<session_id>/`・跨版でない）。
const DIR: &str = "session";
/// 開始の位置の file 名（event log の長さの 10 進）。
const START_FILE: &str = "start";
/// 告げ済みの控えの file 名（1 行 1 記録）。
const TOLD_FILE: &str = "told";
/// 書きの印の file 名（1 行 1 書き・`<UTC の秒の時刻> <語>`・turn の最初の書きの時刻が古さの下の端）。
const WROTE_FILE: &str = "wrote";
/// session id の字の数の上限（file 名に使うので）。
const SESSION_MAX: usize = 128;
/// 作業中の bead の status の字面。
const IN_PROGRESS: &str = "in_progress";
/// 時刻を分に切る秒の単位。
const MINUTE: u64 = 60;

/// 判じられなかった理由（**閉じた 8 語**・`TurnEndUnjudged` の `reason`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// payload に session_id が無いか、path に使えない字を持つ。
    NoSession,
    /// 開始の位置が無い（この止めの着地の前に始まった session）。
    NoStart,
    /// event log を開始の位置から読めない。
    LogUnreadable,
    /// 告げ済みの控えを読めない。
    ToldUnreadable,
    /// 告げ済みの控えを書けない。
    ToldUnwritable,
    /// 書きの印 `wrote` が在るのに読めないか空にできない。
    MarksUnreadable,
    /// 台帳を読めない（rules を読めない・行が無い・子が落ちる・JSON を読めない）。
    LedgerUnreadable,
    /// 台帳を待ち上限までに読み切れなかった。
    LedgerTimeout,
}

/// [`Reason`] の全 variant（宣言順・`enum-slices` が集合完全性を測る）。
pub const REASONS: &[Reason] = &[
    Reason::NoSession,
    Reason::NoStart,
    Reason::LogUnreadable,
    Reason::ToldUnreadable,
    Reason::ToldUnwritable,
    Reason::MarksUnreadable,
    Reason::LedgerUnreadable,
    Reason::LedgerTimeout,
];

impl Reason {
    /// 記帳する語。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoSession => "no-session",
            Self::NoStart => "no-start",
            Self::LogUnreadable => "log-unreadable",
            Self::ToldUnreadable => "told-unreadable",
            Self::ToldUnwritable => "told-unwritable",
            Self::MarksUnreadable => "marks-unreadable",
            Self::LedgerUnreadable => "ledger-unreadable",
            Self::LedgerTimeout => "ledger-timeout",
        }
    }

    /// 字面から引く。未知なら `None`。
    pub fn parse(text: &str) -> Option<Self> {
        REASONS.iter().copied().find(|found| found.as_str() == text)
    }

    /// 控えで重ねを判じられる語か（session を持ち、控えを読め・書ける周の 4 語）。ほかの 4 語は毎回記帳する。
    pub const fn keeps_record(self) -> bool {
        matches!(self, Self::NoStart | Self::LogUnreadable | Self::LedgerUnreadable | Self::LedgerTimeout)
    }
}

/// 告げ済みの控えの 1 記録（1 行）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Record {
    /// 止めの 1 行で告げた未仕分けの発話の ts。
    Told(String),
    /// 止めの 1 行で告げた古さの下の端（turn の最初の書きの時刻の字）。
    Stale(String),
    /// 止めた（この行の後の再入で席を Idle に戻す）。
    Block,
    /// 止めの続きの終わりで席を Idle に戻した。
    Released,
    /// 控えを持てる語を記帳した。
    Unjudged(Reason),
}

impl Record {
    /// 控えの 1 行。
    pub fn to_line(&self) -> String {
        match self {
            Self::Told(ts) => format!("told {ts}"),
            Self::Stale(ts) => format!("stale {ts}"),
            Self::Block => "block".to_owned(),
            Self::Released => "released".to_owned(),
            Self::Unjudged(reason) => format!("unjudged {}", reason.as_str()),
        }
    }

    /// 控えの 1 行を読む。読めない行は `None`。
    pub fn parse(line: &str) -> Option<Self> {
        match line.split_once(' ') {
            Some(("told", ts)) if !ts.is_empty() => Some(Self::Told(ts.to_owned())),
            Some(("stale", ts)) if !ts.is_empty() => Some(Self::Stale(ts.to_owned())),
            Some(("unjudged", word)) => Reason::parse(word).map(Self::Unjudged),
            None if line == "block" => Some(Self::Block),
            None if line == "released" => Some(Self::Released),
            _ => None,
        }
    }
}

/// 告げ済みの控え（読めた記録の列・読めない行は捨てる）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Told(Vec<Record>);

impl Told {
    /// 控えの全文から読む。
    pub fn parse(text: &str) -> Self {
        Self(text.lines().filter_map(Record::parse).collect())
    }

    /// `ts` を告げ済みか。
    pub fn has_told(&self, ts: &str) -> bool {
        self.0.iter().any(|found| matches!(found, Record::Told(told) if told == ts))
    }

    /// 最後の記録が [`Record::Block`] か（再入で席を Idle に戻す周）。
    pub fn last_is_block(&self) -> bool {
        self.0.last() == Some(&Record::Block)
    }

    /// `reason` を控えに記帳済みか。
    pub fn has_unjudged(&self, reason: Reason) -> bool {
        self.0.contains(&Record::Unjudged(reason))
    }
}

/// turn の終わりの判定（**この境界の閉じた 4 値**）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnEndDecision {
    /// 告げる未仕分けも古さも無い（打刻して通す）。
    Pass,
    /// 未告の未仕分けが在る: `all` はその session の未仕分けの ts の全部、`fresh` はそのうち控えに無いもの。
    Block { all: Vec<String>, fresh: Vec<String> },
    /// 書いた turn の作業中の bead の定型の行が古い: `wrote` は turn の最初の書きの時刻の字、`wip` は作業中の bead の id（台帳の順）。
    Stale { wrote: String, wip: Vec<String> },
    /// 判じられなかった（止めずに記帳して通す）。
    Unjudged(Reason),
}

/// 読めた未仕分けの列と控えから判定する（pure）。未告が 1 つも無ければ [`TurnEndDecision::Pass`]。
pub fn decide(unsorted: Result<Vec<String>, Reason>, told: &Told) -> TurnEndDecision {
    match unsorted {
        Err(reason) => TurnEndDecision::Unjudged(reason),
        Ok(all) => {
            let fresh: Vec<String> = all.iter().filter(|ts| !told.has_told(ts)).cloned().collect();
            if fresh.is_empty() {
                TurnEndDecision::Pass
            } else {
                TurnEndDecision::Block { all, fresh }
            }
        }
    }
}

/// turn の書きの種類（印 `wrote` の行の語）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wrote {
    /// 台帳の書き（`bd` / `bdw` の subcommand が [`WRITES`] に在る片を持つ command）。
    Ledger,
    /// 設計の種別（`design-intent` か `design-doc`）の file の編集の道具。
    Design,
}

impl Wrote {
    /// 印の行の語。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ledger => "ledger",
            Self::Design => "design",
        }
    }

    /// 印の行の語から引く。未知なら `None`。
    fn parse(word: &str) -> Option<Self> {
        [Self::Ledger, Self::Design].into_iter().find(|found| found.as_str() == word)
    }
}

/// 通った操作が turn の書きか（pure）。`tool` が Bash なら `command` の片のどれかが台帳の書きの時 [`Wrote::Ledger`]、編集の道具（[`GUARDED`]）なら
/// 編集先の種別 `kind` が設計の 2 種のどれかの時 [`Wrote::Design`]、ほかは `None`。
pub fn wrote_of(tool: &str, command: Option<&str>, kind: Option<PathKind>) -> Option<Wrote> {
    if tool == command::BASH {
        let parts = segments(command.unwrap_or_default());
        let ledger = parts.iter().any(|words| write_of(words).is_some_and(|found| WRITES.contains(&found.subcommand.as_str())));
        return ledger.then_some(Wrote::Ledger);
    }
    (GUARDED.contains(&tool) && matches!(kind, Some(PathKind::DesignIntent | PathKind::DesignDoc))).then_some(Wrote::Design)
}

/// 作業中の bead の定型の行の古さ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Staleness {
    /// 作業中の bead のどれかに新しい段の定型の行が在る。
    Fresh,
    /// 無い: `wip` は作業中の bead の id（台帳の順・0 本なら空）。
    Stale { wip: Vec<String> },
}

/// 作業中（in_progress）の bead のどれか 1 本に、`since`（turn の最初の書きの UNIX 秒）を分に切った値以上の時刻を持つ定型の行が在れば
/// [`Staleness::Fresh`]（pure）。定型の行は [`WIP_HEADS`] のどれかで始まり頭の後が空白だけでない行、行の時刻はその行より前の最後の段の頭の行
/// （[`head_time`] が読める行）の下の端。
pub fn stale_of(since: u64, beads: &[Bead]) -> Staleness {
    let floor = since.saturating_sub(since % MINUTE);
    let working: Vec<&Bead> = beads.iter().filter(|bead| bead.status == IN_PROGRESS).collect();
    if working.iter().any(|bead| has_fresh_line(&bead.notes, floor)) {
        return Staleness::Fresh;
    }
    Staleness::Stale { wip: working.iter().map(|bead| bead.id.clone()).collect() }
}

/// notes に、`floor` 以上の時刻の段の下の定型の行が在るか。
fn has_fresh_line(notes: &str, floor: u64) -> bool {
    let mut at: Option<u64> = None;
    notes.lines().any(|line| {
        if let Some((_, secs)) = head_time(line) {
            at = Some(secs);
            return false;
        }
        let state = WIP_HEADS.iter().any(|head| line.trim_start().strip_prefix(head).is_some_and(|rest| !rest.trim().is_empty()));
        state && at.is_some_and(|secs| secs >= floor)
    })
}

/// `PreToolUse` の門が全部通した周の枝: 台帳の書きか設計の種別の編集なら、session の置き場の印 `wrote` に `<UTC の秒の時刻> <語>` の 1 行を足す。
/// session の id の無い周と runner（`--plugin-root` が置き場の `pipe` の下）は印を書かない。書けない周は黙る（門の allow は変えない）。
pub(super) fn mark(hooked: &Hooked, args: &[String], payload: &str) {
    let Some(session) = session_of(payload) else {
        return;
    };
    if is_runner(hooked.dir, flag_of(args, FLAG_PLUGIN_ROOT)) {
        return;
    }
    let tool = field(payload, KEY_TOOL).unwrap_or_default();
    let command = (tool == command::BASH).then(|| command_of(payload)).flatten();
    let Some(wrote) = wrote_of(&tool, command.as_deref(), design_kind(hooked, payload, &tool)) else {
        return;
    };
    let dir = place(hooked.dir, &session);
    let line = format!("{} {}\n", fleet_cli::format_utc(now_secs()), wrote.as_str());
    let _ = fs::create_dir_all(&dir)
        .and_then(|()| OpenOptions::new().create(true).append(true).open(dir.join(WROTE_FILE)))
        .and_then(|mut found| found.write_all(line.as_bytes()));
}

/// 編集の道具の編集先の種別（anchor の HEAD の宣言で分ける）。編集の道具でない・編集先が無い・repo の外の周は `None`
/// （repo の外は宣言を読まない＝git の子 process を撃たない）。
fn design_kind(hooked: &Hooked, payload: &str, tool: &str) -> Option<PathKind> {
    if !GUARDED.contains(&tool) {
        return None;
    }
    let target = field(payload, KEY_FILE).or_else(|| field(payload, KEY_NOTEBOOK))?;
    let kind_in = |kinds: &PathKinds| locate(Some(hooked.root), hooked.cwd, &target, kinds).kind;
    (kind_in(&PathKinds::Default) != PathKind::Outside).then(|| kind_in(&PathKinds::read_at_head(hooked.root)))
}

/// `SessionStart` の枝: session の置き場へ event log の今の長さを開始の位置として 1 度だけ書く。置き場に既に在れば上書きしない
/// （圧縮と resume の SessionStart）。session_id の無い周・log の長さを測れない周は書かない。書けなかった周だけ stderr の行を返す。
pub(super) fn start(state_dir: &Path, payload: &str) -> Vec<String> {
    let Some(session) = session_of(payload) else {
        return Vec::new();
    };
    let Some(len) = log_len(state_dir) else {
        return Vec::new();
    };
    let file = place(state_dir, &session).join(START_FILE);
    let made = file.parent().map_or(Ok(()), fs::create_dir_all);
    let written = made.and_then(|()| OpenOptions::new().write(true).create_new(true).open(&file)).and_then(|mut found| found.write_all(len.to_string().as_bytes()));
    match written {
        Err(err) if err.kind() == ErrorKind::AlreadyExists => Vec::new(),
        Err(err) => vec![format!("{NAME}: turn-end 開始の位置を書けない: {err}")],
        Ok(()) => Vec::new(),
    }
}

/// `Stop` の枝。再入（`stop_hook_active`）は止めず、控えの最後が `block` の周だけ Idle を打って `released` を足す（印は消さない）。
/// 再入でない周は判定し、未告が在れば控えを書いてから rc 2・stdout 0 byte・stderr 1 行で止める（打刻しない・印は消さない）。未仕分けの止めが
/// 無い周だけ書きの印を判じ（[`judge_stale`]）、古ければ控えを書いて同じ外形で止める。それ以外は打刻・rc 0。
pub(super) fn stop(args: &[String], payload: &str, hooked: &Hooked) -> Outcome {
    let state_dir = hooked.dir;
    let session = session_of(payload);
    if stamp::is_reentry(payload) {
        return release(args, payload, state_dir, session.as_deref());
    }
    let Some(session) = session else {
        return settle(args, payload, state_dir, note(state_dir, None, Reason::NoSession));
    };
    let first = judge(state_dir, &session);
    if let TurnEndDecision::Block { all, fresh } = first {
        let told = fresh.into_iter().map(Record::Told).collect();
        return match block(state_dir, &session, told) {
            Ok(()) => Outcome::failed_line(RC_BROKEN, block_line(&all)),
            Err(reason) => settle(args, payload, state_dir, note(state_dir, Some(&session), reason)),
        };
    }
    let mut notes = match first {
        TurnEndDecision::Unjudged(reason) => note(state_dir, Some(&session), reason),
        _ => Vec::new(),
    };
    match judge_stale(hooked, &session) {
        TurnEndDecision::Stale { wrote, wip } => match block(state_dir, &session, vec![Record::Stale(wrote.clone())]) {
            Ok(()) => Outcome::failed_line(RC_BROKEN, stale_line(&wrote, &wip)),
            Err(reason) => {
                notes.extend(note(state_dir, Some(&session), reason));
                settle(args, payload, state_dir, notes)
            }
        },
        TurnEndDecision::Unjudged(reason) => {
            notes.extend(note(state_dir, Some(&session), reason));
            settle(args, payload, state_dir, notes)
        }
        TurnEndDecision::Pass | TurnEndDecision::Block { .. } => settle(args, payload, state_dir, notes),
    }
}

/// 止めなかった周の外形: 今のまま打刻（stdout 0 byte・rc 0）し、記帳の失敗の行だけを stderr へ足す。
fn settle(args: &[String], payload: &str, state_dir: &Path, notes: Vec<String>) -> Outcome {
    let mut outcome = Outcome::ok(Vec::new());
    outcome.err = stamp::stamp(args, payload, Stamped::Stop, state_dir);
    outcome.err.extend(notes);
    outcome
}

/// 再入の周: 控えの最後が `block` のときだけ Idle を打って `released` を足す。ほかは黙る。
fn release(args: &[String], payload: &str, state_dir: &Path, session: Option<&str>) -> Outcome {
    let mut outcome = Outcome::ok(Vec::new());
    let Some(session) = session else {
        return outcome;
    };
    if read_told(state_dir, session).is_ok_and(|told| told.last_is_block()) {
        outcome.err = stamp::stamp_release(args, payload, state_dir);
        if let Err(err) = append_told(state_dir, session, &[Record::Released]) {
            outcome.err.push(format!("{NAME}: turn-end released を書けない: {err}"));
        }
    }
    outcome
}

/// 控えと開始の位置と log を読んで判定する（台帳は読まない）。
fn judge(state_dir: &Path, session: &str) -> TurnEndDecision {
    let Ok(told) = read_told(state_dir, session) else {
        return TurnEndDecision::Unjudged(Reason::ToldUnreadable);
    };
    decide(unsorted(state_dir, session), &told)
}

/// その session の未仕分けの発話の ts（log の順）。開始の位置が無ければ [`Reason::NoStart`]、log を読めなければ [`Reason::LogUnreadable`]。
fn unsorted(state_dir: &Path, session: &str) -> Result<Vec<String>, Reason> {
    let text = fs::read_to_string(place(state_dir, session).join(START_FILE)).map_err(|_| Reason::NoStart)?;
    let start = text.trim().parse::<u64>().map_err(|_| Reason::NoStart)?;
    let (mut said, mut marks) = (Vec::new(), Vec::new());
    for event in read_from(state_dir, start).ok_or(Reason::LogUnreadable)? {
        match &event.case {
            Some(Case::Utterance { session: Some(found), .. }) if found == session => said.push(event.ts.clone()),
            Some(Case::Sorted { .. } | Case::Ruling { .. }) => marks.push(event),
            _ => {}
        }
    }
    Ok(said.into_iter().filter(|ts| sorted_of(ts, &marks) == Standing::Unsorted).collect())
}

/// event log を開始の位置から末尾まで読む（読む byte は開始の位置より前の大きさに依らない）。読めない log（開けない・開始の位置より
/// 短い）は `None`。log が無く開始の位置が 0 なら空。読めない行は捨てる。
fn read_from(state_dir: &Path, start: u64) -> Option<Vec<Event>> {
    let mut file = match File::open(store::events_path(state_dir)) {
        Ok(found) => found,
        Err(err) if err.kind() == ErrorKind::NotFound => return (start == 0).then(Vec::new),
        Err(_) => return None,
    };
    if file.metadata().ok()?.len() < start {
        return None;
    }
    let mut bytes = Vec::new();
    file.seek(SeekFrom::Start(start)).and_then(|_| file.read_to_end(&mut bytes)).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    Some(text.lines().filter_map(|line| Event::from_line(line).ok()).collect())
}

/// event log の今の長さ（byte）。log が無ければ 0。file でない・測れない log は `None`。
fn log_len(state_dir: &Path) -> Option<u64> {
    match fs::metadata(store::events_path(state_dir)) {
        Ok(found) => found.is_file().then_some(found.len()),
        Err(err) if err.kind() == ErrorKind::NotFound => Some(0),
        Err(_) => None,
    }
}

/// 書きの印を判じる（印の無い turn は台帳を読まない）。印を読んで空にしてから（止める前に空にする＝続く再入でない stop は通す）、台帳を
/// 1 回だけ読んで [`stale_of`] で判じる。印が在るのに読めない・空にできない周は [`Reason::MarksUnreadable`]、台帳を読めない周は
/// [`Reason::LedgerUnreadable`] か [`Reason::LedgerTimeout`]。
fn judge_stale(hooked: &Hooked, session: &str) -> TurnEndDecision {
    let file = place(hooked.dir, session).join(WROTE_FILE);
    let first = match fs::read_to_string(&file) {
        Err(err) if err.kind() == ErrorKind::NotFound => return TurnEndDecision::Pass,
        Err(_) => None,
        Ok(text) => first_mark(&text),
    };
    let cleared = fs::remove_file(&file).is_ok();
    let Some((wrote, since)) = first.filter(|_| cleared) else {
        return TurnEndDecision::Unjudged(Reason::MarksUnreadable);
    };
    match ledger_beads(hooked) {
        Err(reason) => TurnEndDecision::Unjudged(reason),
        Ok(beads) => match stale_of(since, &beads) {
            Staleness::Fresh => TurnEndDecision::Pass,
            Staleness::Stale { wip } => TurnEndDecision::Stale { wrote, wip },
        },
    }
}

/// 印の最初の読める行の (時刻の字, UNIX 秒)。
fn first_mark(text: &str) -> Option<(String, u64)> {
    text.lines().find_map(|line| {
        let (time, word) = line.split_once(' ')?;
        Wrote::parse(word)?;
        Some((time.to_owned(), epoch_of(time)?))
    })
}

/// 台帳を 1 回読む（client は `--bd` か PATH の既定・待ち上限は rules 行 `seat.ledger_timeout_s`・子の cwd は hook の cwd）。
fn ledger_beads(hooked: &Hooked) -> Result<Vec<Bead>, Reason> {
    let manifest = hooked.rules.map_or_else(Manifest::embedded, |path| Manifest::load(Path::new(path)));
    let timeout = manifest.ok().and_then(|found| ledger::timeout_of(&found)).ok_or(Reason::LedgerUnreadable)?;
    let bd = hooked.bd.filter(|found| !found.trim().is_empty()).unwrap_or(ledger::DEFAULT_BD);
    let text = ledger::read_text(bd, hooked.cwd, timeout).map_err(|error| match error {
        LedgerError::Unreadable => Reason::LedgerUnreadable,
        LedgerError::Timeout => Reason::LedgerTimeout,
    })?;
    recent::beads_of(&text).ok_or(Reason::LedgerUnreadable)
}

/// 控えを書いて止める前の準備: 告げる記録（未告の ts か古さの下の端）と `block` を 1 回の書きで足す。書けなければ [`Reason::ToldUnwritable`]。
fn block(state_dir: &Path, session: &str, mut records: Vec<Record>) -> Result<(), Reason> {
    records.push(Record::Block);
    append_told(state_dir, session, &records).map_err(|_| Reason::ToldUnwritable)
}

/// 古さの止めの 1 行（作業中の bead の id は台帳の順・0 本は `-`）。
fn stale_line(wrote: &str, wip: &[String]) -> String {
    let ids = if wip.is_empty() { "-".to_owned() } else { wip.join(",") };
    format!(
        "{NAME}: turn-end stale 作業中の bead の定型の行が古い wrote={wrote} wip={ids} — 作業中の bead の notes に頭 [席 <UTC の時刻>] の段で \
         計画: 次の手: 優先: 未決: の行を足す（作業中の bead が無ければ手を付けた epic か memo を in_progress にする）"
    )
}

/// 止めの 1 行（未仕分けの ts を全部・仕分けの口・bind・show の名・逐語は運ばない）。
fn block_line(all: &[String]) -> String {
    format!(
        "{NAME}: turn-end block 未仕分けの発話 ts={} — 発話ごとに {NAME} utterance sort <ts> --as request --memo <id> か --as chat で仕分け、\
         問いへの答えは {NAME} seat ruling bind で結ぶ（中身は {NAME} utterance show <ts>）",
        all.join(",")
    )
}

/// 判じられなかった周の記帳。控えを持てる語（[`Reason::keeps_record`]）は session ごとに 1 度だけ記帳し（控えに足せなければ
/// [`Reason::ToldUnwritable`] として毎回）、ほかは毎回記帳する。記帳できなかったときだけ stderr の行を返す。
fn note(state_dir: &Path, session: Option<&str>, reason: Reason) -> Vec<String> {
    let mut reason = reason;
    if let (true, Some(found)) = (reason.keeps_record(), session) {
        match read_told(state_dir, found) {
            Err(_) => reason = Reason::ToldUnreadable,
            Ok(told) if told.has_unjudged(reason) => return Vec::new(),
            Ok(_) if append_told(state_dir, found, &[Record::Unjudged(reason)]).is_err() => reason = Reason::ToldUnwritable,
            Ok(_) => {}
        }
    }
    match LockPolicy::embedded().and_then(|policy| store::append(state_dir, &unjudged_event(session, reason), policy)) {
        Ok(_) => Vec::new(),
        Err(err) => vec![format!("{NAME}: turn-end unjudged reason={} を記帳できない: {err}", reason.as_str())],
    }
}

/// 書く行（`TurnEndUnjudged`・actor machine・session は任意・run 無し）。
fn unjudged_event(session: Option<&str>, reason: Reason) -> Event {
    Event {
        schema: crate::fleet::SCHEMA,
        ts: fleet_cli::now_utc(),
        kind: EventKind::TurnEndUnjudged,
        run: String::new(),
        bead: String::new(),
        host: fleet_cli::host(),
        actor: EventKind::TurnEndUnjudged.default_actor().to_owned(),
        stage: None,
        seat: None,
        pid: None,
        detail: None,
        allowance: None,
        registration: None,
        mark: None,
        account: None,
        cost: None,
        rule: None,
        case: Some(Case::TurnEnd { session: session.map(str::to_owned), reason: reason.as_str().to_owned() }),
    }
}

/// payload の session_id（path に使える字だけ・無い周と使えない周は `None`）。
fn session_of(payload: &str) -> Option<String> {
    let ok = |sid: &str| {
        !sid.is_empty() && sid.len() <= SESSION_MAX && sid.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
    };
    field(payload, KEY_SESSION_ID).filter(|sid| ok(sid))
}

/// session の置き場。
fn place(state_dir: &Path, session: &str) -> PathBuf {
    state_dir.join(DIR).join(session)
}

/// 控えを読む。file が無ければ空・それ以外の読めなさは `Err`。
fn read_told(state_dir: &Path, session: &str) -> Result<Told, std::io::Error> {
    match fs::read_to_string(place(state_dir, session).join(TOLD_FILE)) {
        Ok(text) => Ok(Told::parse(&text)),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(Told::default()),
        Err(err) => Err(err),
    }
}

/// 控えへ記録を 1 回の書きで足す（置き場の dir は無ければ作る）。
fn append_told(state_dir: &Path, session: &str, records: &[Record]) -> Result<(), std::io::Error> {
    let dir = place(state_dir, session);
    fs::create_dir_all(&dir)?;
    let text: String = records.iter().map(|record| format!("{}\n", record.to_line())).collect();
    OpenOptions::new().create(true).append(true).open(dir.join(TOLD_FILE))?.write_all(text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::{decide, Reason, Record, Told, TurnEndDecision, REASONS};

    const TS_A: &str = "2026-09-30T07:05:09.123Z";
    const TS_B: &str = "2026-09-30T07:05:09.456Z";

    fn tss(list: &[&str]) -> Vec<String> {
        list.iter().map(|ts| (*ts).to_owned()).collect()
    }

    /// 理由の語は閉じた 8 語で、字面が重ならず往復し、控えを持てるのは no-start・log-unreadable・ledger-unreadable・ledger-timeout の 4 語だけ。
    #[test]
    fn hook_unsorted_stop_reasons_are_eight_closed_words_and_four_keep_a_record() {
        let words: Vec<&str> = REASONS.iter().map(|reason| reason.as_str()).collect();
        assert_eq!(
            words,
            ["no-session", "no-start", "log-unreadable", "told-unreadable", "told-unwritable", "marks-unreadable", "ledger-unreadable", "ledger-timeout"]
        );
        for reason in REASONS {
            assert_eq!(Reason::parse(reason.as_str()), Some(*reason), "{reason:?}");
        }
        assert_eq!(Reason::parse("lock"), None);
        let keeping: Vec<&str> = REASONS.iter().filter(|reason| reason.keeps_record()).map(|reason| reason.as_str()).collect();
        assert_eq!(keeping, ["no-start", "log-unreadable", "ledger-unreadable", "ledger-timeout"]);
    }

    /// 控えの 4 形は 1 行 1 記録で往復し、読めない行・未知の語・空の ts は捨てる。
    #[test]
    fn hook_unsorted_stop_told_reads_four_forms_and_drops_the_rest() {
        let records = [Record::Told(TS_A.to_owned()), Record::Block, Record::Released, Record::Unjudged(Reason::NoStart)];
        for record in &records {
            assert_eq!(Record::parse(&record.to_line()).as_ref(), Some(record), "{record:?}");
        }
        let text = format!("told {TS_A}\nblock\nnonsense\ntold \nunjudged maybe\nunjudged no-start\n");
        let told = Told::parse(&text);
        assert!(told.has_told(TS_A) && !told.has_told(TS_B), "告げ済みの ts");
        assert!(told.has_unjudged(Reason::NoStart) && !told.has_unjudged(Reason::LogUnreadable), "記帳済みの語");
        assert!(!told.last_is_block(), "最後は unjudged");
        assert_eq!(told, Told::parse(&format!("told {TS_A}\nblock\nunjudged no-start\n")), "読めない行は捨てる");
    }

    /// 再入の表: 最後の記録が block の周だけ真（空・released・告げ済みだけは偽）。
    #[test]
    fn hook_unsorted_stop_reentry_table_releases_only_after_a_block() {
        let table = [
            ("空", "", false),
            ("block", "told a\nblock\n", true),
            ("block の後の released", "told a\nblock\nreleased\n", false),
            ("released の後の block", "block\nreleased\ntold b\nblock\n", true),
            ("告げ済みだけ", "told a\n", false),
        ];
        for (name, text, expected) in table {
            assert_eq!(Told::parse(text).last_is_block(), expected, "{name}");
        }
    }

    /// 判定の表: 未仕分け 0 と告げ済みだけは通し、未告が 1 つでも在れば全部の ts を並べて止め、読めない周は理由を運ぶ。
    #[test]
    fn hook_unsorted_stop_decides_pass_block_or_unjudged() {
        let told = Told::parse(&format!("told {TS_A}\nblock\n"));
        let table = [
            ("未仕分け 0", Ok(tss(&[])), TurnEndDecision::Pass),
            ("告げ済みだけ", Ok(tss(&[TS_A])), TurnEndDecision::Pass),
            ("未告が増えた", Ok(tss(&[TS_A, TS_B])), TurnEndDecision::Block { all: tss(&[TS_A, TS_B]), fresh: tss(&[TS_B]) }),
            ("読めない", Err(Reason::LogUnreadable), TurnEndDecision::Unjudged(Reason::LogUnreadable)),
        ];
        for (name, unsorted, expected) in table {
            assert_eq!(decide(unsorted.clone(), &told), expected, "{name}");
            assert_eq!(decide(unsorted, &told), expected, "{name}: 同じ入力は同じ結果");
        }
        let first = decide(Ok(tss(&[TS_A, TS_B])), &Told::default());
        assert_eq!(first, TurnEndDecision::Block { all: tss(&[TS_A, TS_B]), fresh: tss(&[TS_A, TS_B]) }, "控えが空なら全部が未告");
    }
}
