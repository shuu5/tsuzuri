//! tz consult launch <窓 id> --follow [--session <会話の id>] [--wait <秒>]（判断の記録 ADR-55 決定 (3)(5)）。
//! 生きている話す窓を、席の今の口座（環境の CLAUDE_CONFIG_DIR）へ付いて来させる口。印の口座が今の口座と同じなら何もしない。
//! 会話の印の最後の行が手すき（中核の `follow::idle`）で、tmux の pane の入力欄が空（`follow::input_clear`）になるのを待ち
//! （会話の印を持たない窓は --session で席が会話の id を名指し、入力欄だけを待つ）、上限 --wait（既定 `WAIT` 秒）を越えたら
//! 起こし直さず、印 `.consult/held-<k>` を置いて固定の 1 行（`Event::Held`）を出して断る（見張りはこの印の窓のずれを出さない）。
//! 手すきなら、起こす口座に作業場の信頼の印を置き、tmux の窓を名を確かめて閉じ（close の `kill_named`）、前の process が終わるのを
//! 待ってから、撃ち直し（--again）の形で同じ作業場に起こし直して会話を続ける。台帳には開きの行（撃ち直し）だけを書き、閉じの行は書かない。

use std::path::Path;
use std::time::{Duration, Instant};

use tsuzuri_contract::consult::{Form, ProcMark, WindowId};
use tsuzuri_core::consult::follow::{idle, input_clear};
use tsuzuri_core::consult::lines::{Event, Line, notice};
use tsuzuri_core::consult::resume::pick;
use tsuzuri_core::consult::stamp::lines;

use super::close::kill_named;
use super::launch::{ACCOUNT_ENV, Shot, TMUX, last_conversation, launch, read_stamps};
use super::plain::append_plain;
use super::trust::place_trust;
use super::{
    Ctx, FAIL, GIT_TIMEOUT, Refused, UNKNOWN, alive, ledger, lines_of, procs, read_window, tzw,
    workspace,
};
use crate::out::emit;
use crate::server::ledger::capture;

/// 手すきを待つ既定の上限（秒）。
pub const WAIT: u64 = 300;

/// 手すきと入力欄を見る間。
const STEP: Duration = Duration::from_millis(500);

/// 閉じた窓の前の process の終わりを待つ上限。
const GONE: Duration = Duration::from_secs(5);

/// 付いてこなかった印の作業場からの相対 path（k は最後の process の印の番号・`plain` で見て書く）。
pub fn held_rel(k: u32) -> String {
    format!(".consult/held-{k}")
}

/// 作業場の会話の印の最後の行が手すきか（印の file が無いか読めなければ偽・読みは `launch::read_stamps`）。
pub fn stamped_idle(ws: &Path) -> bool {
    matches!(read_stamps(ws), Ok(Some(t)) if idle(&lines(&t)))
}

/// tmux の窓の pane の入力欄が空か（照らせなければ偽）。
fn pane_clear(ws: &Path, tmux_window: &str) -> bool {
    let args = ["capture-pane", "-p", "-J", "-t", tmux_window];
    capture(std::ffi::OsStr::new(TMUX), args, ws, GIT_TIMEOUT)
        .and_then(|o| String::from_utf8(o).ok())
        .and_then(|t| input_clear(&t))
        == Some(true)
}

/// 手すき（会話の印を持つ窓だけ）と入力欄が空になるのを `wait` まで待つ（なれば真）。
fn settle(ws: &Path, tmux_window: &str, stamped: bool, wait: Duration) -> bool {
    let end = Instant::now() + wait;
    loop {
        if (!stamped || stamped_idle(ws)) && pane_clear(ws, tmux_window) {
            return true;
        }
        if Instant::now() >= end {
            return false;
        }
        std::thread::sleep(STEP);
    }
}

/// 付いて来させる窓の最後の process の印（話す窓・閉じの行が無い・process が生きている）。
fn live_talk(c: &Ctx, id: WindowId, ws: &Path) -> Result<ProcMark, Refused> {
    let w = read_window(ws).ok_or((FAIL, format!("窓 {id} の作業場が無い: {}", ws.display())))?;
    if w.form != Form::Talk {
        let why = format!("問う窓 {id} は付いてこない（--again で撃ち直す）");
        return Err((FAIL, why));
    }
    let closed = |l: &Line| matches!(l, Line::Close { window, .. } if *window == id);
    if lines_of(&ledger(c)?.1).iter().any(closed) {
        return Err((FAIL, format!("窓 {id} は閉じた")));
    }
    let mark = procs(ws).pop().filter(|p| alive(p.pid));
    mark.ok_or((
        FAIL,
        format!("話す窓 {id} は動いていない（--again で開き直す）"),
    ))
}

/// tmux の窓を名を確かめて閉じ、前の process が `GONE` の内に終わるのを待つ。
fn close_old(ws: &Path, id: WindowId, mark: &ProcMark, tmux_window: &str) -> Result<(), Refused> {
    if !kill_named(ws, tmux_window, &id.name()) && alive(mark.pid) {
        let why = format!(
            "tmux の窓 {tmux_window} は {} でない（閉じない・起こし直さない）",
            id.name()
        );
        return Err((FAIL, why));
    }
    let end = Instant::now() + GONE;
    while alive(mark.pid) {
        if Instant::now() >= end {
            let why = format!(
                "前の process（pid {}）が終わらない（起こし直さない）",
                mark.pid
            );
            return Err((FAIL, why));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Ok(())
}

/// 付いて来させる（版の照らしの後）。
pub fn follow(c: &Ctx, id: WindowId, session: Option<&str>, wait: Duration) -> Result<(), Refused> {
    let ws = workspace(&c.drafts, id);
    let mark = live_talk(c, id, &ws)?;
    let account = std::env::var(ACCOUNT_ENV).map_err(|_| {
        let why = format!("席の環境に {ACCOUNT_ENV} が無い（付いてくる先の口座が分からない）");
        (FAIL, why)
    })?;
    if mark.account.as_deref() == Some(account.as_str()) {
        emit(&format!(
            "窓 {id} は席の今の口座で動いている（開き直さない）"
        ));
        return Ok(());
    }
    let tmux_window = mark
        .tmux_window
        .clone()
        .ok_or((FAIL, format!("窓 {id} の印に tmux の窓が無い")))?;
    let last = last_conversation(&ws)?;
    if last.is_none() && session.is_none() {
        let why = "会話の印が無い窓は --session で会話の id を名指す（持ち主に確かめてから）";
        return Err((FAIL, why.to_string()));
    }
    pick(Form::Talk, true, last.as_deref(), session).map_err(|why| (FAIL, why.to_string()))?;
    if !settle(&ws, &tmux_window, last.is_some(), wait) {
        let _ = append_plain(&ws, &held_rel(mark.k), b"");
        emit(&notice(&Event::Held(id), &tzw()));
        let why = format!(
            "話す窓 {id} は {} 秒の内に手すきにならなかった（起こし直さない）",
            wait.as_secs()
        );
        return Err((FAIL, why));
    }
    let real = ws
        .canonicalize()
        .map_err(|e| (UNKNOWN, format!("作業場が読めない: {e}")))?;
    place_trust(Path::new(&account), &real.display().to_string())?;
    close_old(&ws, id, &mark, &tmux_window)?;
    let shot = Shot {
        again: true,
        dry: false,
        session,
    };
    launch(c, id, &shot)
}
