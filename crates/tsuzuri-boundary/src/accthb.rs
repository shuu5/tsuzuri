//! 停止の切り替えの受付（要件 FR12・便 e-acct-hb）。面は state dir の file を書かず、器の CLI
//! `seat heartbeat <on|off> --state-dir <dir> --target <target>` を 1 回撃つだけにする
//! （off は器が席の置き場に停止の記録を置き、on は器がそれを消す）。
//! 要求の本文は project の名と向きだけを持つ。state dir と target は本文の字でなく、群の宣言（引数の state dir の
//! host.toml）の anchor と git config（`Acct::state_dir`）と doctor の席の行（role=orchestrator）から引く。
//! Origin の検査と本文の大きさの上限と口の登録は、つなぐ行 h-wire が server の `guarded` と一緒に通す。
//! 子 process は `capture` で撃ち、落ちる・rc が 0 でない・5 秒で返らないは、どれも器が撃てない扱い（502）。

use std::ffi::OsStr;
use std::path::Path;

use tsuzuri_contract::account::{Heartbeat, HeartbeatRequest, HeartbeatResponse};
use tsuzuri_contract::wire;
use tsuzuri_core::account::host::declaration;
use tsuzuri_core::account::project_name;

use crate::acct::{Acct, orchestrator_target};
use crate::server::ledger::capture;
use crate::server::seat::{DOCTOR_ARGS, HOST_TOML, SCRIBE2_TIMEOUT};

/// 停止の切り替えの引数の列の頭（この後に `<on|off> --state-dir <dir> --target <target>` が続く）。
pub const HEARTBEAT_ARGS: [&str; 2] = ["seat", "heartbeat"];

/// 本文が HeartbeatRequest として読めない（400）。
pub const BAD_BODY: &str = "bad-body";

/// 本文の project が群の宣言の anchor に無い（404）。
pub const NO_PROJECT: &str = "no-project";

/// anchor の state dir が引けないか、doctor に orchestrator の席の行が無い（404）。
pub const NO_SEAT: &str = "no-seat";

/// 器が落ちるか rc が 0 でないか 5 秒で返らない（502）。
pub const VESSEL_FAILED: &str = "vessel-failed";

/// 向きの字（器の argv と電文の字が同じ）。
pub fn word(to: Heartbeat) -> &'static str {
    match to {
        Heartbeat::On => "on",
        Heartbeat::Off => "off",
    }
}

/// 停止の切り替えの受付（状態の数と本文の字）。200 の本文は HeartbeatResponse の JSON、断りの本文は字だけ。
/// no-project と bad-body は器を撃たず、no-seat は seat heartbeat を撃たない。file は書かない。
pub fn accept(acct: &Acct, body: &str) -> (u16, String) {
    let Ok(request) = wire::decode::<HeartbeatRequest>(body) else {
        return refuse(400, BAD_BODY);
    };
    let Some(anchor) = anchor(acct.host_state_dir(), &request.project) else {
        return refuse(404, NO_PROJECT);
    };
    let Some(dir) = acct.state_dir(Path::new(&anchor)) else {
        return refuse(404, NO_SEAT);
    };
    let mut doctor: Vec<&OsStr> = DOCTOR_ARGS.iter().map(OsStr::new).collect();
    doctor.extend([OsStr::new("--state-dir"), dir.as_os_str()]);
    let Some(out) = shoot(acct, doctor) else {
        return refuse(502, VESSEL_FAILED);
    };
    let Some(target) = orchestrator_target(&out, &anchor) else {
        return refuse(404, NO_SEAT);
    };
    let mut args: Vec<&OsStr> = HEARTBEAT_ARGS.iter().map(OsStr::new).collect();
    args.extend([
        OsStr::new(word(request.to)),
        OsStr::new("--state-dir"),
        dir.as_os_str(),
        OsStr::new("--target"),
        OsStr::new(target),
    ]);
    if shoot(acct, args).is_none() {
        return refuse(502, VESSEL_FAILED);
    }
    let response = HeartbeatResponse {
        target: target.to_string(),
        to: request.to,
    };
    match wire::encode(&response) {
        Ok(json) => (200, json),
        Err(_) => refuse(500, "encode"),
    }
}

fn refuse(status: u16, text: &str) -> (u16, String) {
    (status, text.to_string())
}

/// 器を撃った出力（落ちる・rc が 0 でない・5 秒で返らない・UTF-8 でないなら None）。
fn shoot(acct: &Acct, args: Vec<&OsStr>) -> Option<String> {
    let out = capture(acct.scribe2(), args, acct.cwd(), SCRIBE2_TIMEOUT)?;
    String::from_utf8(out).ok()
}

/// 群の宣言の anchor のうち、最後の区切りが project の名と同じ最初の anchor（書かれた字のまま・空の名は無し）。
fn anchor(host_state_dir: &Path, project: &str) -> Option<String> {
    if project.is_empty() {
        return None;
    }
    let toml = std::fs::read_to_string(host_state_dir.join(HOST_TOML)).ok()?;
    declaration(&toml)
        .groups
        .into_iter()
        .flat_map(|g| g.anchors.unwrap_or_default())
        .find(|a| project_name(a) == project)
}
