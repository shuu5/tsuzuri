//! 表示先の設定と窓を開く頼みの受付（要件 FR16・判断の記録 ADR-15 の決定 (2)(6)）。
//! 面と server は設定の file を自分で書かず、tz の口（`tz stage target show --json`・`set`・`clear`・`tz stage open`）を
//! 1 回撃つだけにする（読みの字は tz が組む・名に空白を含められるので server は行を割って読まない）。
//! 撃ちはどれも引数の後に --repo と --scribe2 を足し、cwd は --repo の dir で、子の環境から席の印を除く
//! （tz stage open は席の中の撃ちを断るので、印を渡すと持ち主の button の頼みが断られる・席が撃つ口は Origin の頭で断る）。
//! 受付の順は、本文の読み（400 bad-body）→ 名の形（400 bad-name）→ project の引き（404 no-project）→ 撃ち
//! （502 tz-failed）→ 200 と tz の標準出力の字（`accthb::accept` と同じ並び）。

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::Duration;

use tsuzuri_contract::stage::{OpenRequest, OwnTarget, StageTargets, Targets};
use tsuzuri_contract::wire;

use crate::accthb;
use crate::server::proc::capture_unset;
use crate::stage::cli;
use crate::stage::target;

/// 既定の tz の program の名。
pub const TZ: &str = "tz";

/// 表示先の設定の口の引数の頭（この後に命令の語と旗が続く）。
pub const TARGET_ARGS: [&str; 2] = ["stage", "target"];

/// 窓を開く口の引数の頭。
pub const OPEN_ARGS: [&str; 2] = ["stage", "open"];

/// 表示先の設定の口の待ち（tz の中の器の検めの 10 秒より長い）。
pub const TARGET_TIMEOUT: Duration = Duration::from_secs(20);

/// 窓を開く口の待ち（tz stage open の錠の待ち 30 秒と ssh の待ちより長い）。
pub const OPEN_TIMEOUT: Duration = Duration::from_secs(60);

/// 本文が契約の型として読めない（400）。
pub const BAD_BODY: &str = accthb::BAD_BODY;

/// 本文の project が --repo の project でも群の宣言の anchor の名でもない（404）。
pub const NO_PROJECT: &str = accthb::NO_PROJECT;

/// 名が argv に載せられない形（400）。
pub const BAD_NAME: &str = "bad-name";

/// tz が撃てないか rc が 0 でないか時間内に返らない（502）。
pub const TZ_FAILED: &str = "tz-failed";

/// tz の標準出力が電文として読めない（502）。
pub const BAD_REPLY: &str = "bad-reply";

/// 撃つ相手（tz と器の program と server の --repo の dir）。server の `Shared` の欄 stage。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caller {
    pub tz: OsString,
    pub scribe2: OsString,
    pub repo: PathBuf,
}

/// argv に載せてよい名か（設定に書ける名で、字 - で始まらない・旗に化けない）。
pub fn named(name: &str) -> bool {
    target::shaped(name) && !name.starts_with('-')
}

fn refuse(status: u16, text: &str) -> (u16, String) {
    (status, text.to_string())
}

/// tz を 1 回撃った標準出力（撃てない・rc が 0 でない・時間切れ・UTF-8 でないなら None）。
/// 引数は `head` と `rest` の後に --repo `dir` と --scribe2 を足し、cwd は `dir`。
fn shoot(
    caller: &Caller,
    head: [&str; 2],
    rest: &[&str],
    dir: &Path,
    timeout: Duration,
) -> Option<String> {
    let repo = std::path::absolute(dir).unwrap_or_else(|_| dir.to_path_buf());
    let mut args: Vec<&OsStr> = head.iter().chain(rest).map(OsStr::new).collect();
    args.extend([
        OsStr::new("--repo"),
        repo.as_os_str(),
        OsStr::new("--scribe2"),
        caller.scribe2.as_os_str(),
    ]);
    let out = capture_unset(&caller.tz, args, dir, timeout, &[cli::SEAT_ENV])?;
    String::from_utf8(out).ok()
}

/// 撃ちの結果を応答にする（撃てなければ 502 tz-failed・撃てれば 200 と標準出力の字）。
fn reply(done: Option<String>) -> (u16, String) {
    match done {
        Some(text) => (200, text),
        None => refuse(502, TZ_FAILED),
    }
}

/// 表示先の設定の読み（show --json の標準出力を trim して `StageTargets` に読めれば 200 とその字）。
/// 読めなければ 502 bad-reply、撃てなければ 502 tz-failed。
pub fn read(caller: &Caller) -> (u16, String) {
    let Some(text) = shoot(
        caller,
        TARGET_ARGS,
        &["show", "--json"],
        &caller.repo,
        TARGET_TIMEOUT,
    ) else {
        return refuse(502, TZ_FAILED);
    };
    let line = text.trim();
    if wire::decode::<StageTargets>(line).is_err() {
        return refuse(502, BAD_REPLY);
    }
    (200, line.to_string())
}

/// project board の書きの受付（自分の project だけ・to が名なら set --project・null なら clear --project）。
pub fn accept_own(caller: &Caller, body: &str) -> (u16, String) {
    let Ok(request) = wire::decode::<OwnTarget>(body) else {
        return refuse(400, BAD_BODY);
    };
    if request.to.as_deref().is_some_and(|to| !named(to)) {
        return refuse(400, BAD_NAME);
    }
    let Ok(own) = cli::project(&caller.repo) else {
        return refuse(404, NO_PROJECT);
    };
    let done = match &request.to {
        Some(to) => shoot(
            caller,
            TARGET_ARGS,
            &["set", "--project", &own, to],
            &caller.repo,
            TARGET_TIMEOUT,
        ),
        None => shoot(
            caller,
            TARGET_ARGS,
            &["clear", "--project", &own],
            &caller.repo,
            TARGET_TIMEOUT,
        ),
    };
    reply(done)
}

/// account board の書きの受付（一括は set --all・project ごとは名が --repo の project か群の宣言の anchor の名の時だけ
/// set か clear）。`host_state_dir` は群の宣言の置き場（無ければ --repo の project だけ）。
pub fn accept_all(caller: &Caller, host_state_dir: Option<&Path>, body: &str) -> (u16, String) {
    let Ok(request) = wire::decode::<Targets>(body) else {
        return refuse(400, BAD_BODY);
    };
    let shaped = match &request {
        Targets::All { to } => named(to),
        Targets::Project { project, to } => named(project) && to.as_deref().is_none_or(named),
    };
    if !shaped {
        return refuse(400, BAD_NAME);
    }
    if let Targets::Project { project, .. } = &request {
        let own = cli::project(&caller.repo).is_ok_and(|own| own == *project);
        let listed = host_state_dir.is_some_and(|dir| accthb::anchor(dir, project).is_some());
        if !own && !listed {
            return refuse(404, NO_PROJECT);
        }
    }
    let done = match &request {
        Targets::All { to } => shoot(
            caller,
            TARGET_ARGS,
            &["set", "--all", to],
            &caller.repo,
            TARGET_TIMEOUT,
        ),
        Targets::Project {
            project,
            to: Some(to),
        } => shoot(
            caller,
            TARGET_ARGS,
            &["set", "--project", project, to],
            &caller.repo,
            TARGET_TIMEOUT,
        ),
        Targets::Project { project, to: None } => shoot(
            caller,
            TARGET_ARGS,
            &["clear", "--project", project],
            &caller.repo,
            TARGET_TIMEOUT,
        ),
    };
    reply(done)
}

/// 窓を開く受付（project が null なら --repo の dir・名なら群の宣言の anchor の dir で tz stage open を撃つ）。
pub fn accept_open(caller: &Caller, host_state_dir: Option<&Path>, body: &str) -> (u16, String) {
    let Ok(request) = wire::decode::<OpenRequest>(body) else {
        return refuse(400, BAD_BODY);
    };
    let dir = match request.project.as_deref() {
        None => caller.repo.clone(),
        Some(project) => {
            if !named(project) {
                return refuse(400, BAD_NAME);
            }
            match host_state_dir.and_then(|dir| accthb::anchor(dir, project)) {
                Some(anchor) => PathBuf::from(anchor),
                None => return refuse(404, NO_PROJECT),
            }
        }
    };
    reply(shoot(caller, OPEN_ARGS, &[], &dir, OPEN_TIMEOUT))
}
