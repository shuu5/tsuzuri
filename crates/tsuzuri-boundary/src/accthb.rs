//! 停止の切り替えの受付（要件 FR12・便 e-acct-hb）。面は state dir の file を書かず、器の CLI
//! `seat heartbeat <on|off> --state-dir <dir> --target <target>` を 1 回撃つだけにする
//! （off は器が席の置き場に停止の記録を置き、on は器がそれを消す）。
//! 要求の本文は project の名と向きだけを持つ。state dir と target は本文の字でなく、群の宣言（引数の state dir の
//! host.toml）の anchor と git config（`Acct::state_dir`）と doctor の席の行（role=orchestrator）から引く。
//! Origin の検査と本文の大きさの上限と口の登録は、つなぐ行 h-wire が server の `guarded` と一緒に通す。
//! 子 process は `capture` で撃ち、落ちる・rc が 0 でない・5 秒で返らないは、どれも器が撃てない扱い（502）。
//! 2 つ目の受付 `accept_own` は project board の口（行 e-seat-hb・裁定 t3-hub.52.29 の案 A）で、本文は向きだけを持ち、
//! anchor を名でなく server の --repo と同じ dir で引く（最後の名が同じ anchor が 2 つ在っても、この board の席だけを止める）。
//! anchor を引いた後の順と断りと 200 の電文は `accept` と同じ。

use std::ffi::OsStr;
use std::path::Path;

use tsuzuri_contract::account::{Heartbeat, HeartbeatRequest, HeartbeatResponse};
use tsuzuri_contract::seathb::SeatHeartbeatRequest;
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
    shoot_anchor(acct, &anchor, request.to)
}

/// project board の停止の切り替えの受付（本文は向きだけ・anchor は `repo` と同じ dir を指す群の宣言の最初の anchor）。
/// 状態の数と本文の字と、anchor を引いた後の順は `accept` と同じ。bad-body と no-project は git も器も撃たない。
pub fn accept_own(acct: &Acct, repo: &Path, body: &str) -> (u16, String) {
    let Ok(request) = wire::decode::<SeatHeartbeatRequest>(body) else {
        return refuse(400, BAD_BODY);
    };
    let Some(anchor) = own_anchor(acct.host_state_dir(), repo) else {
        return refuse(404, NO_PROJECT);
    };
    shoot_anchor(acct, &anchor, request.to)
}

/// 引いた anchor（宣言に書かれた字のまま）から state dir・doctor・席・seat heartbeat を 1 回撃つ。
fn shoot_anchor(acct: &Acct, anchor: &str, to: Heartbeat) -> (u16, String) {
    let Some(dir) = acct.state_dir(Path::new(anchor)) else {
        return refuse(404, NO_SEAT);
    };
    let mut doctor: Vec<&OsStr> = DOCTOR_ARGS.iter().map(OsStr::new).collect();
    doctor.extend([OsStr::new("--state-dir"), dir.as_os_str()]);
    let Some(out) = shoot(acct, doctor) else {
        return refuse(502, VESSEL_FAILED);
    };
    let Some(target) = orchestrator_target(&out, anchor) else {
        return refuse(404, NO_SEAT);
    };
    let mut args: Vec<&OsStr> = HEARTBEAT_ARGS.iter().map(OsStr::new).collect();
    args.extend([
        OsStr::new(word(to)),
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
        to,
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
/// 表示先の設定の受付（`stagecall`）も project の名の引きに使う。
pub(crate) fn anchor(host_state_dir: &Path, project: &str) -> Option<String> {
    if project.is_empty() {
        return None;
    }
    anchors(host_state_dir)?.find(|a| project_name(a) == project)
}

/// 群の宣言の anchor のうち、`repo` と同じ dir を指す最初の anchor（書かれた字のまま）。
/// 両方を canonicalize できればその path で、どちらかができなければ字の Path で比べる（末尾の斜線を問わない）。
fn own_anchor(host_state_dir: &Path, repo: &Path) -> Option<String> {
    let own = repo.canonicalize().ok();
    anchors(host_state_dir)?.find(|a| {
        let path = Path::new(a);
        match (path.canonicalize().ok(), &own) {
            (Some(p), Some(own)) => p == *own,
            _ => path == repo,
        }
    })
}

/// 群の宣言の anchor の列（群の順・配列の順・宣言が読めなければ None）。
fn anchors(host_state_dir: &Path) -> Option<impl Iterator<Item = String>> {
    let toml = std::fs::read_to_string(host_state_dir.join(HOST_TOML)).ok()?;
    Some(
        declaration(&toml)
            .groups
            .into_iter()
            .flat_map(|g| g.anchors.unwrap_or_default()),
    )
}
