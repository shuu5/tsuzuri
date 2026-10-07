//! tz hook agent-guard（判断の記録 ADR-59 決定 (3)(4)・要件 FR21）: 係の門。
//! 撃つのは Claude Code（PreToolUse の hook・matcher なし）。標準入力の hook の入力が係の呼び（係の id の在る呼び）の時だけ判じる。順:
//! 1. 標準入力を全部読む。係の呼びでなければ（席の呼び）何も読まず何も出さずに 0。
//! 2. 起草の置き場を解く（結びの口と同じ --drafts か repo の git config の鍵）。解けなければ標準エラーに書いて通す。
//! 3. 測りと同じ `resolve` で結びの名と札を読む。結びが無ければ通す（記帳は同じ呼びの後の測りが書く）。
//! 4. 測りの札 `<名>/meter.json` の使った量（札が無いか読めなければ 0）が札の予算より小さければ通す。
//! 5. 書き終えの段では、中核の `open` が通す呼び（SendMessage と `<名>/w` の下への書き）のほかは deny の答えを標準出力に 1 行で書いて 0。
//! 6. 全時間の検め（条 N-2）: 札の組みが なし の係の Bash の command の頭の語が cargo なら、係の Bash の command の頭の語が台帳か器の state を
//!    書く語なら（組みによらない）、書きの道具が `<名>/w` の下と写し `try-<名>` の下のどちらでもない所へ書くなら、deny の答えを 1 行で書いて 0。
//!
//! 係の dir に群の席の札 `group.json` の在る群の係は、4 の前に、読みの道具（Read）の割りの読む path の外の読みと係を起こす道具（Agent）の
//! 呼びを断り、読み直し（測りの札の `cache_read`）が上限 `MEMBER_READ` 以上なら 5 と同じ呼びだけを通す（判断の記録 ADR-61 決定 (4)(7)）。
//!
//! rc は 0 か 1（使い方の誤り）だけ。門は file を書かない。

use std::fs;
use std::io::Read;
use std::path::Path;

use tsuzuri_core::agent::guard::{
    CLONE, bash_command, build_refusal, cargo_word, fence_refusal, fenced, ledger_refusal,
    ledger_word, open, refusal, spent, write_path,
};
use tsuzuri_core::agent::meter::group::{fence as read_fence, read_path, read_refusal};
use tsuzuri_core::agent::meter::{METER, Meter, sub_call};
use tsuzuri_core::agent::spec::{BUILDS, OUT, Spec};
use tsuzuri_core::agent::spec::group::{MEMBER_READ, SEAT, Seat};
use tsuzuri_core::gate::deny_json;

use super::agent_args::begin;
use super::agent_meter::resolve;
use crate::out::emit;

pub const USAGE: &str = "usage: tz hook agent-guard --repo <dir> [--drafts <dir>]";

/// 群の係の呼びの断りの理由（割りの外の読みと入れ子・読み直しが上限以上の書き終えの段・群の係でなければ None）。
fn member(
    seat: Option<&Seat>,
    tool: &str,
    payload: &str,
    meter: &Meter,
    out: &Path,
) -> Option<String> {
    let seat = seat?;
    read_fence(tool, read_path(payload).as_deref(), seat).or_else(|| {
        (spent(meter.cache_read, MEMBER_READ) && !open(tool, write_path(payload).as_deref(), out))
            .then(|| read_refusal(tool, meter.cache_read, out))
    })
}

/// 全時間の検めの断りの理由（組み なし の係の Bash の cargo・係の Bash の台帳か器の state を書く語・出力の dir と写しの dir の外への書き・
/// どれでもなければ None）。`agent` は係の dir。
fn fence(spec: &Spec, tool: &str, payload: &str, agent: &Path, clone: &Path) -> Option<String> {
    let out = agent.join(OUT);
    let command = bash_command(payload);
    if spec.build == BUILDS[0]
        && let Some(word) = command.as_deref().and_then(cargo_word)
    {
        return Some(build_refusal(&word, &out));
    }
    if let Some(word) = command.as_deref().and_then(|c| ledger_word(c, agent)) {
        return Some(ledger_refusal(&word, agent, &out));
    }
    let path = write_path(payload);
    fenced(tool, path.as_deref(), &out, clone)
        .then(|| fence_refusal(tool, path.as_deref(), &out, clone))
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
    let (_, dir) = match begin(USAGE, "通す", rest) {
        Ok(found) => found,
        Err(rc) => return rc,
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
    let agent = dir.join(&name);
    let out = agent.join(OUT);
    let clone = dir.join(format!("{CLONE}{name}"));
    let used = meter.used;
    if let Some(why) = member(seat.as_ref(), &call.tool, &payload, &meter, &out) {
        emit(&deny_json(why));
    } else if spent(used, spec.budget) && !open(&call.tool, write_path(&payload).as_deref(), &out) {
        emit(&deny_json(refusal(&call.tool, used, spec.budget, &out)));
    } else if let Some(why) = fence(&spec, &call.tool, &payload, &agent, &clone) {
        emit(&deny_json(why));
    }
    0
}
