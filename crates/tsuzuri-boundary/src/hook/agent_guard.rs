//! tz hook agent-guard（行 ag-guard・判断の記録 ADR-59 決定 (3)(4)・要件 FR21）: 係の門。
//! 撃つのは Claude Code（PreToolUse の hook・matcher なし）。標準入力の hook の入力が係の呼び（係の id の在る呼び）の時だけ判じる。順:
//! 1. 標準入力を全部読む。係の呼びでなければ（席の呼び）何も読まず何も出さずに 0。
//! 2. 起草の置き場を解く（結びの口と同じ --drafts か repo の git config の鍵）。解けなければ標準エラーに書いて通す。
//! 3. 測りと同じ `resolve` で結びの名と札を読む。結びが無ければ通す（記帳は同じ呼びの後の測りが書く）。
//! 4. 測りの札 `<名>/meter.json` の使った量（札が無いか読めなければ 0）が札の予算より小さければ通す。
//! 5. 書き終えの段では、中核の `open` が通す呼び（SendMessage と `<名>/w` の下への書き）のほかは deny の答えを標準出力に 1 行で書いて 0。
//!
//! rc は 0 か 1（使い方の誤り）だけ。門は file を書かない。

use std::fs;
use std::io::Read;

use tsuzuri_core::agent::meter::guard::{OUT, open, refusal, spent, write_path};
use tsuzuri_core::agent::meter::{METER, Meter, sub_call};
use tsuzuri_core::agent::spec::deny;

use super::agent_meter::resolve;
use super::agent_spawn::{drafts, parse};
use crate::out::{emit, emit_err};

pub const USAGE: &str = "usage: tz hook agent-guard --repo <dir> [--drafts <dir>]";

/// 使い方の誤り。
const FAIL: u8 = 1;

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
    let Some((name, spec)) = resolve(&dir, &call.agent_id) else {
        return 0;
    };
    let used = fs::read_to_string(dir.join(&name).join(METER))
        .ok()
        .and_then(|t| Meter::parse(&t))
        .map_or(0, |m| m.used);
    let out = dir.join(&name).join(OUT);
    if spent(used, spec.budget) && !open(&call.tool, write_path(&payload).as_deref(), &out) {
        emit(&deny(&refusal(&call.tool, used, spec.budget, &out)));
    }
    0
}
