//! tz hook agent-guard（行 ag-guard・判断の記録 ADR-59 決定 (3)(4)・要件 FR21）: 係の門。
//! 撃つのは Claude Code（PreToolUse の hook・matcher なし）。標準入力の hook の入力が係の呼び（係の id の在る呼び）の時だけ判じる。順:
//! 1. 標準入力を全部読む。係の呼びでなければ（席の呼び）何も読まず何も出さずに 0。
//! 2. 起草の置き場を解く（結びの口と同じ --drafts か repo の git config の鍵）。解けなければ標準エラーに書いて通す。
//! 3. 測りと同じ `resolve` で結びの名と札を読む。結びが無ければ通す（記帳は同じ呼びの後の測りが書く）。
//! 4. 測りの札 `<名>/meter.json` の使った量（札が無いか読めなければ 0）が札の予算より小さければ通す。
//! 5. 書き終えの段では、中核の `open` が通す呼び（SendMessage と `<名>/w` の下への書き）のほかは deny の答えを標準出力に 1 行で書いて 0。
//!
//! 係の dir に群の席の札 `group.json` の在る群の係は、4 の前に、読みの道具（Read）の割りの読む path の外の読みと係を起こす道具（Agent）の
//! 呼びを断り、読み直し（測りの札の `cache_read`）が上限 `MEMBER_READ` 以上なら 5 と同じ呼びだけを通す（行 ag-gwatch・判断の記録 ADR-61 決定 (4)(7)）。
//!
//! rc は 0 か 1（使い方の誤り）だけ。門は file を書かない。

use std::fs;
use std::io::Read;
use std::path::Path;

use tsuzuri_core::agent::meter::group::{fence, read_path, read_refusal};
use tsuzuri_core::agent::meter::guard::{OUT, open, refusal, spent, write_path};
use tsuzuri_core::agent::meter::{METER, Meter, sub_call};
use tsuzuri_core::agent::spec::deny;
use tsuzuri_core::agent::spec::group::{MEMBER_READ, SEAT, Seat};

use super::agent_meter::resolve;
use super::agent_spawn::{drafts, parse};
use crate::out::{emit, emit_err};

pub const USAGE: &str = "usage: tz hook agent-guard --repo <dir> [--drafts <dir>]";

/// 使い方の誤り。
const FAIL: u8 = 1;

/// 群の係の呼びの断りの理由（割りの外の読みと入れ子・読み直しが上限以上の書き終えの段・群の係でなければ None）。
fn member(
    seat: Option<&Seat>,
    tool: &str,
    payload: &str,
    meter: &Meter,
    out: &Path,
) -> Option<String> {
    let seat = seat?;
    fence(tool, read_path(payload).as_deref(), seat).or_else(|| {
        (spent(meter.cache_read, MEMBER_READ) && !open(tool, write_path(payload).as_deref(), out))
            .then(|| read_refusal(tool, meter.cache_read, out))
    })
}

/// tz hook agent-guard の残りの引数を受けて終了 code を返す（0 か 1 だけ）。
pub fn run(rest: &[&str]) -> u8 {
    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        payload.clear();
    }
    let Some(call) = sub_call(&payload) else {
        return 0;
    };
    let args = match parse(rest) {
        Ok(args) => args,
        Err(e) => {
            emit_err(&format!("tz hook agent-guard: {e}\n{USAGE}"));
            return FAIL;
        }
    };
    let Some(dir) = drafts(&args) else {
        emit_err("tz hook agent-guard: 起草の置き場を解けない（通す）");
        return 0;
    };
    let Ok((name, spec)) = resolve(&dir, &call) else {
        return 0;
    };
    let meter = fs::read_to_string(dir.join(&name).join(METER))
        .ok()
        .and_then(|t| Meter::parse(&t))
        .unwrap_or_default();
    let seat = fs::read_to_string(dir.join(&name).join(SEAT))
        .ok()
        .and_then(|t| Seat::parse(&t));
    let out = dir.join(&name).join(OUT);
    let used = meter.used;
    if let Some(why) = member(seat.as_ref(), &call.tool, &payload, &meter, &out) {
        emit(&deny(&why));
    } else if spent(used, spec.budget) && !open(&call.tool, write_path(&payload).as_deref(), &out) {
        emit(&deny(&refusal(&call.tool, used, spec.budget, &out)));
    }
    0
}
