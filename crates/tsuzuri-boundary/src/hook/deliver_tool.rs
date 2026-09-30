//! tz hook deliver-tool（行 f-deliver-tool・要件 FR9・器の要件 FR79）: 席の PostToolBatch の hook。
//! 考え中の席へ、tool の呼びの組が終わった後（次の model の呼びの前）に、印の無い裁定の逐語を席の文脈に足し、
//! 答えを書けた後にだけ、答えが名指した裁定に経路が tool の呼びの印を置く。
//! 撃つのは Claude Code（plugin の hooks.json の PostToolBatch・matcher なし）。順:
//! 1. 使い方の誤りか repo が dir でなければ rc 1（標準入力は読まない）。
//! 2. repo の下の .git が file（git の worktree）なら何もせずに 0。
//! 3. 標準入力が席の本体の入力（最上位に agent_id も agent_type も無い JSON の object）でなければ、子 process を撃たずに 0。
//! 4. 安い判じ（自分の board の口 GET `UNRECEIVED_PATH`）が Known の空の列なら 0。読めない・Unknown なら
//!    標準エラーに 1 行を書いて 0（台帳は読まない・印の無い裁定は停止の hook が拾う）。
//! 5. 空でなければ台帳を bd で 1 回読み、印の無い裁定の逐語が無ければ 0、読めなければ標準エラーに 1 行を書いて 0。
//! 6. 答えを標準出力に書いて flush し、その後に答えが名指した裁定にだけ印を bdw で置く（上限 `MARK_BUDGET`）。
//!
//! どの形でも 2 は返さない。hook は file を書かない（台帳の書きは bdw だけ）。
//! 待ちの上限は `WAIT`（git と tailnet の道具）・`REACH`（住所ごとの接続と読み）・`BD_TIMEOUT`・`MARK_BUDGET`。

use std::ffi::OsStr;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::time::{Duration, Instant};

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::LedgerWrite;
use tsuzuri_contract::surface::RulingId;
use tsuzuri_contract::wire;
use tsuzuri_core::delivery::{
    Route, Said, TOOL_EVENT, context_for, main_thread, mark_line, named, unmarked,
};

use crate::acct::{self, GIT};
use crate::out::emit_err;
use crate::server::events::now;
use crate::server::ledger::{Source, capture};
use crate::server::proc;
use crate::server::ruling::{WRITE_TIMEOUT, minute};
use crate::stage::url::{STATUS_ARGS, TAILNET};

use super::question_signal::{REACH, WAIT, places};
pub use super::stop::{Args, parse};

pub const USAGE: &str = "usage: tz hook deliver-tool --repo <dir> [--bd <program>] [--bdw <program>]";

/// 自分の board の口（席に届いていない裁定の id の列）。
pub const UNRECEIVED_PATH: &str = "/api/unreceived";

/// 1 回の印の書きの全部の上限（台帳の読みの `BD_TIMEOUT` などと足して hooks.json の timeout より短く）。
pub const MARK_BUDGET: Duration = Duration::from_secs(15);

/// 使い方の誤り。
const FAIL: u8 = 1;

/// 口を 1 本撃ち、応答を終わりまで読んで（状態の数・本文）を返す（接続と読み書きの待ちは `wait` まで）。
pub fn fetch(addr: SocketAddr, wait: Duration) -> Result<(u16, String), String> {
    let mut stream =
        TcpStream::connect_timeout(&addr, wait).map_err(|e| format!("{addr} に繋がらない: {e}"))?;
    stream
        .set_read_timeout(Some(wait))
        .and_then(|()| stream.set_write_timeout(Some(wait)))
        .map_err(|e| format!("{addr} の待ちの上限を置けない: {e}"))?;
    let request =
        format!("GET {UNRECEIVED_PATH} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("{addr} へ書けない: {e}"))?;
    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("{addr} の応答を読めない: {e}"))?;
    let text = String::from_utf8_lossy(&raw);
    let line = text.lines().next().unwrap_or_default();
    let status = line
        .split(' ')
        .nth(1)
        .and_then(|code| code.parse().ok())
        .ok_or_else(|| format!("{addr} の応答の状態の行が読めない: {line}"))?;
    let (_, body) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| format!("{addr} の応答に本文が無い"))?;
    Ok((status, body.to_string()))
}

/// 自分の board の口から、印の無い裁定の id の列を読む（最初に応答を読めた住所の答え・住所の列は問いの合図の送り手と同じ）。
pub fn unreceived(
    repo: &Path,
    git: &OsStr,
    tailnet: &OsStr,
) -> Result<Reading<Vec<RulingId>>, String> {
    let mut args = vec![OsStr::new("-C"), repo.as_os_str()];
    args.extend(acct::BOARD_ARGS.iter().map(OsStr::new));
    let port = proc::capture(git, &args, repo, WAIT)
        .and_then(|out| acct::board_port(&String::from_utf8_lossy(&out)))
        .ok_or_else(|| {
            format!(
                "{} の git config の tsuzuri.boardport が読めない（無いか 1 から 65535 の数でない）",
                repo.display()
            )
        })?;
    // status が読めなくても 127.0.0.1 は撃つ。
    let status = proc::capture(tailnet, STATUS_ARGS, repo, WAIT)
        .map(|out| String::from_utf8_lossy(&out).into_owned());
    let mut last = String::from("撃つ先が無い");
    for addr in places(status.as_deref(), port) {
        match fetch(addr, REACH) {
            Ok((200, body)) => {
                return wire::decode(&body)
                    .map_err(|e| format!("{addr} の本文が Reading の電文でない: {e}"));
            }
            Ok((code, _)) => return Err(format!("{addr} の board が状態 {code} で答えた")),
            Err(e) => last = e,
        }
    }
    Err(last)
}

/// 名指した裁定に tool の呼びの印を順に置き、書けた数を返す（`budget` は呼ばれた時から数える全部の上限）。
/// 落ちた裁定と上限で撃たなかった裁定は、その裁定の id を含む 1 行ずつを標準エラーに書いて次へ進む。
pub fn mark(args: &Args, named: &[Said], now: EpochSecs, budget: Duration) -> usize {
    let start = Instant::now();
    let minute = minute(now);
    let mut done = 0;
    for s in named {
        let left = budget.saturating_sub(start.elapsed());
        if left.is_zero() {
            emit_err(&format!(
                "tz hook deliver-tool: 時間の上限で印を置かない: 裁定 {}",
                s.ruling
            ));
            continue;
        }
        let write = LedgerWrite::AppendNotes {
            id: s.question.clone(),
            line: mark_line(&s.ruling, Route::Tool, &minute),
        };
        if capture(&args.bdw, write.argv(), &args.repo, left.min(WRITE_TIMEOUT)).is_some() {
            done += 1;
        } else {
            emit_err(&format!(
                "tz hook deliver-tool: 印の書きが落ちた: 裁定 {}",
                s.ruling
            ));
        }
    }
    done
}

/// tz hook deliver-tool の残りの引数を受けて終了 code を返す（0 か 1 だけ）。
pub fn run(rest: &[&str]) -> u8 {
    let args = match parse(rest) {
        Ok(args) => args,
        Err(e) => return usage(&e),
    };
    if !args.repo.is_dir() {
        return usage(&format!(
            "repo の置き場 {} が dir でない",
            args.repo.display()
        ));
    }
    // git の worktree（器の便の runner・人が worktree で起こした session）では黙る。
    if args.repo.join(".git").is_file() {
        return 0;
    }
    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        payload.clear();
    }
    if !main_thread(&payload) {
        return 0;
    }
    match unreceived(&args.repo, OsStr::new(GIT), OsStr::new(TAILNET)) {
        Ok(Reading::Known(ids)) if ids.is_empty() => return 0,
        Ok(Reading::Known(_)) => {}
        Ok(Reading::Unknown) => {
            emit_err("tz hook deliver-tool: board が台帳を読めていない（印の無い裁定を判じない）");
            return 0;
        }
        Err(e) => {
            emit_err(&format!("tz hook deliver-tool: {e}"));
            return 0;
        }
    }
    let said = match Source::new(&args.repo, &args.bd)
        .text_alone()
        .map(|text| unmarked(&text))
    {
        Some(Reading::Known(said)) => said,
        _ => {
            emit_err("tz hook deliver-tool: 台帳が読めない（逐語を写せない）");
            return 0;
        }
    };
    let Some(answer) = context_for(&said, TOOL_EVENT) else {
        return 0;
    };
    let mut out = std::io::stdout().lock();
    if let Err(e) = writeln!(out, "{answer}").and_then(|()| out.flush()) {
        emit_err(&format!(
            "tz hook deliver-tool: 答えを書けない（印を置かない）: {e}"
        ));
        return 0;
    }
    drop(out);
    mark(&args, &said[..named(&said)], now(), MARK_BUDGET);
    0
}

fn usage(what: &str) -> u8 {
    emit_err(&format!("tz hook deliver-tool: {what}\n{USAGE}"));
    FAIL
}
