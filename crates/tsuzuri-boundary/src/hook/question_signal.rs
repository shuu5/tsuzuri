//! tz hook question-signal（要件 NFR2・規則の行 R-21）: 問いの合図の送り手。
//! 撃つのは Claude Code（PostToolUse の hook・matcher Bash・async で席は待たない）。標準入力の hook の入力が
//! 問いの起票（門と同じ読み）で、tool_response の stdout に起票の行が在るときだけ、自分の board の口
//! POST /api/surface/questions（`NUDGE_PATH`）へ本文 QuestionNudge を 1 本送る。順:
//! 1. 標準入力を全部読む（読めなければ空の字）。使い方の誤りは rc 1。
//! 2. 起票の行が無ければ子 process を撃たずに 0。
//! 3. 在れば repo の git config の tsuzuri.boardport を git で、この host の住所を tailnet の道具の status で読み、
//!    TailscaleIPs の住所・127.0.0.1 の順に送り、最初に応答を読めた住所で止める。落ちは標準エラーに 1 行を書いて 0。
//!
//! host の名・住所・port は code に書かず実行の時に読む（行 D-4）。hook は file を書かない。
//! async の hook には Claude Code の timeout が効かないので、待ちの上限は `WAIT` と `REACH` で hook 自身が持つ。
//! 合図が落ちても、問いは見張りの周期の読みで画面に出る。

use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::surface::QuestionNudge;
use tsuzuri_contract::wire;
use tsuzuri_core::gate;

use crate::acct::{self, GIT};
use crate::out::emit_err;
use crate::server::events::NUDGE_PATH;
use crate::server::proc;
use crate::stage::json;
use crate::stage::url::{self, STATUS_ARGS, TAILNET};

pub const USAGE: &str = "usage: tz hook question-signal --repo <dir>";

/// bd の create が標準出力に書く起票の行の印（この後の最初の語が id・bd の版で変われば、ここだけを直す）。
pub const CREATED: &str = "Created issue: ";

/// git と tailnet の道具の待ちの上限。
pub const WAIT: Duration = Duration::from_secs(2);

/// 1 つの住所への接続と読み書きの待ちの上限。
pub const REACH: Duration = Duration::from_millis(500);

/// 使い方の誤り。
const FAIL: u8 = 1;

/// `--repo 値` か `--repo=値` の 1 つだけを読む（空の値とほかの形は断る）。
pub fn parse(rest: &[&str]) -> Result<PathBuf, String> {
    let value = match rest {
        ["--repo", value] => *value,
        [one] => one
            .strip_prefix("--repo=")
            .ok_or_else(|| format!("知らない引数 {one}"))?,
        _ => return Err("--repo <dir> の 1 つだけを受ける".into()),
    };
    if value.is_empty() {
        return Err("--repo の値が空".into());
    }
    Ok(PathBuf::from(value))
}

/// 問いの起票の呼びの tool_response の stdout の起票の行の最初の id（起票でない呼び・起票の行の無い呼びは None）。
pub fn question(payload: &str) -> Option<BeadId> {
    if gate::drafts(payload).is_empty() {
        return None;
    }
    let response = json::member(payload, "tool_response")?;
    let stdout = json::unquote(json::member(response, "stdout")?)?;
    stdout.lines().find_map(|line| {
        let (_, rest) = line.split_once(CREATED)?;
        BeadId::new(rest.split_whitespace().next()?).ok()
    })
}

/// 送り先の列（status の Self の住所として読める字の順・最後に 127.0.0.1 を 1 度・status が読めなければ 127.0.0.1 だけ）。
pub fn places(status: Option<&str>, port: u16) -> Vec<SocketAddr> {
    let loopback = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let mut out: Vec<SocketAddr> = Vec::new();
    for host in status.and_then(url::self_hosts).unwrap_or_default() {
        let Ok(ip) = host.parse::<IpAddr>() else {
            continue;
        };
        let addr = SocketAddr::new(ip, port);
        if addr != loopback && !out.contains(&addr) {
            out.push(addr);
        }
    }
    out.push(loopback);
    out
}

/// 合図の要求を 1 本送り、応答を終わりまで読んで状態の数を返す（接続と読み書きの待ちは `wait` まで）。
pub fn send(addr: SocketAddr, body: &str, wait: Duration) -> Result<u16, String> {
    let mut stream =
        TcpStream::connect_timeout(&addr, wait).map_err(|e| format!("{addr} に繋がらない: {e}"))?;
    stream
        .set_read_timeout(Some(wait))
        .and_then(|()| stream.set_write_timeout(Some(wait)))
        .map_err(|e| format!("{addr} の待ちの上限を置けない: {e}"))?;
    // Origin は付けない（口の守りは Origin の無い要求を同じ出所と見る）。
    let request = format!(
        "POST {NUDGE_PATH} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("{addr} へ書けない: {e}"))?;
    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("{addr} の応答を読めない: {e}"))?;
    let text = String::from_utf8_lossy(&raw);
    let line = text.lines().next().unwrap_or_default();
    line.split(' ')
        .nth(1)
        .and_then(|code| code.parse().ok())
        .ok_or_else(|| format!("{addr} の応答の状態の行が読めない: {line}"))
}

/// 問いの起票なら自分の board へ合図を 1 本送る（起票でなければ子 process を撃たずに None・送れたら住所と 202）。
pub fn signal(
    repo: &Path,
    git: &OsStr,
    tailnet: &OsStr,
    payload: &str,
) -> Result<Option<(SocketAddr, u16)>, String> {
    let Some(id) = question(payload) else {
        return Ok(None);
    };
    let mut args = vec![OsString::from("-C"), repo.as_os_str().to_os_string()];
    args.extend(acct::BOARD_ARGS.iter().map(OsString::from));
    let port = proc::capture(git, &args, repo, WAIT)
        .and_then(|out| acct::board_port(&String::from_utf8_lossy(&out)))
        .ok_or_else(|| {
            format!(
                "{} の git config の tsuzuri.boardport が読めない（無いか 1 から 65535 の数でない）",
                repo.display()
            )
        })?;
    // status が読めなくても 127.0.0.1 へは送る。
    let status = proc::capture(tailnet, STATUS_ARGS, repo, WAIT)
        .map(|out| String::from_utf8_lossy(&out).into_owned());
    let body = wire::encode(&QuestionNudge { question: id })
        .map_err(|e| format!("合図の電文を組めない: {e}"))?;
    let mut last = String::from("送り先が無い");
    for addr in places(status.as_deref(), port) {
        match send(addr, &body, REACH) {
            Ok(202) => return Ok(Some((addr, 202))),
            Ok(code) => return Err(format!("{addr} の board が合図を状態 {code} で断った")),
            Err(e) => last = e,
        }
    }
    Err(last)
}

/// tz hook question-signal の残りの引数を受けて終了 code を返す（使い方の誤りだけ 1・ほかは 0）。
pub fn run(rest: &[&str]) -> u8 {
    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        payload.clear();
    }
    let repo = match parse(rest) {
        Ok(repo) => repo,
        Err(e) => {
            emit_err(&format!("tz hook question-signal: {e}\n{USAGE}"));
            return FAIL;
        }
    };
    if let Err(e) = signal(&repo, OsStr::new(GIT), OsStr::new(TAILNET), &payload) {
        emit_err(&format!("tz hook question-signal: {e}"));
    }
    0
}
