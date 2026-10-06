//! tz consult close <窓 id> --by seat|chat|button（判断の記録 ADR-29 決定 (3)(5)・持ち主が閉じると言った時）。
//! 話す窓は、最後の process の印の tmux の window id の窓の名が consult-cw<n> であることを確かめてから kill-window を
//! 撃つ（名が違うか窓が無ければ撃たない＝持ち主が先に閉じた窓と、id を使い回したほかの窓を殺さない）。
//! 私用の temp を消す（持ち主の uid の dir だけ・symlink は辿らない）。台帳の根に相談の閉じの行（起こし手・所見の数）を
//! 書き、所見の全部が処分されていれば作業場を退かせる（`dispose::retire`・消さない）。標準出力は「窓 cw<n> を閉じた」と、
//! 退かせた時は「退いた: <path>」。作業場の無い窓と閉じた窓は断る（rc 1）。

use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use tsuzuri_contract::consult::{Starter, WindowId};
use tsuzuri_core::consult::launch::private_tmp;
use tsuzuri_core::consult::lines::Line;

use super::dispose::retire;
use super::launch::TMUX;
use super::{
    COMMON, Ctx, FAIL, GIT_TIMEOUT, Refused, append, ctx, findings, flags, ledger, lines_of,
    minute_now, odd_files, odd_line, procs, read_window, refuse, workspace,
};
use crate::out::{emit, emit_err};
use crate::server::ledger::capture;

/// tz consult close の残りの引数を受けて終了 code を返す。
pub fn run(rest: &[&str]) -> u8 {
    let mut values = COMMON.to_vec();
    values.push("--by");
    let made = flags(rest, &values, &[], &[]).and_then(|f| {
        let [id] = f.pos.as_slice() else {
            return Err((FAIL, "窓の id を 1 つ渡す".to_string()));
        };
        let id = WindowId::parse(id).map_err(|_| (FAIL, format!("{id} は窓の id でない")))?;
        let by = match f.get("--by") {
            Some("seat") => Starter::Seat,
            Some("chat") => Starter::Chat,
            Some("button") => Starter::Button,
            _ => return Err((FAIL, "--by は seat・chat・button のどれか".to_string())),
        };
        let c = ctx(&f)?;
        close(&c, id, by)
    });
    match made {
        Ok(lines) => {
            lines.iter().for_each(|l| emit(l));
            0
        }
        Err(e) => refuse("close", e),
    }
}

/// 話す窓の tmux の窓を、名を確かめてから閉じる（撃ったら真）。
pub fn kill_named(ws: &Path, tmux_window: &str, name: &str) -> bool {
    let tmux = OsStr::new(TMUX);
    let shown = capture(
        tmux,
        ["display-message", "-p", "-t", tmux_window, "#{window_name}"],
        ws,
        GIT_TIMEOUT,
    )
    .and_then(|o| String::from_utf8(o).ok());
    if shown.as_deref().map(str::trim) != Some(name) {
        return false;
    }
    capture(tmux, ["kill-window", "-t", tmux_window], ws, GIT_TIMEOUT).is_some()
}

/// 私用の temp を消す（作業場の持ち主の uid の dir だけ・symlink は辿らない・消したら真）。
pub fn drop_tmp(ws: &Path) -> bool {
    let Ok(real) = ws.canonicalize() else {
        return false;
    };
    let tmp = private_tmp(&real.display().to_string());
    let owner = fs::metadata(&real).map(|m| m.uid()).ok();
    let mine = fs::symlink_metadata(&tmp).is_ok_and(|m| m.is_dir() && Some(m.uid()) == owner);
    mine && fs::remove_dir_all(&tmp).is_ok()
}

/// 閉じる（出す行の列を返す）。
fn close(c: &Ctx, id: WindowId, by: Starter) -> Result<Vec<String>, Refused> {
    let ws = workspace(&c.drafts, id);
    read_window(&ws).ok_or((
        FAIL,
        format!("窓 {id} の作業場が無い（退いた窓か知らない窓）"),
    ))?;
    for (rel, why) in odd_files(&ws) {
        emit_err(&format!("tz consult close: {}", odd_line(id, &rel, &why)));
    }
    let (_, items) = ledger(c)?;
    let mut lines = lines_of(&items);
    if lines
        .iter()
        .any(|l| matches!(l, Line::Close { window, .. } if *window == id))
    {
        return Err((FAIL, format!("窓 {id} はもう閉じた")));
    }
    if let Some(w) = procs(&ws).last().and_then(|p| p.tmux_window.clone())
        && !kill_named(&ws, &w, &id.name())
    {
        emit_err(&format!(
            "tz consult close: tmux の窓 {w} は {} でない（撃たない）",
            id.name()
        ));
    }
    drop_tmp(&ws);
    let count = u32::try_from(findings(&ws, id).len()).unwrap_or(u32::MAX);
    let line = Line::Close {
        window: id,
        by,
        findings: count,
        at: minute_now(),
    };
    append(c, &items, None, &line)?;
    lines.push(line);
    let mut out = vec![format!("窓 {id} を閉じた")];
    if let Some(gone) = retire(c, id, &lines)? {
        out.push(format!("退いた: {}", gone.display()));
    }
    Ok(out)
}
