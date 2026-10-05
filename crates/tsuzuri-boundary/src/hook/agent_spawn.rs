//! tz hook agent-spawn（判断の記録 ADR-59 決定 (2)(4)・要件 FR21）: 係の起こしの門。
//! 撃つのは Claude Code（PreToolUse の hook・matcher Agent）。標準入力の hook の入力が席の Agent の呼び（係の id の無い呼び）の時だけ判じる。順:
//! 1. 標準入力を全部読む。席の Agent の呼びでなければ（係の呼び・ほかの道具）何も読まず何も出さずに 0。
//! 2. 起草の置き場を --drafts か、repo の git config の鍵 `tsuzuri.draftsdir` から解く。解けなければ標準エラーに書いて通す（fail-open）。
//! 3. 置き場の直下の dir の係の札を全部読み、中核の `judge` で判じる。断る時は deny の答えを標準出力に 1 行で書いて 0。
//! 4. 通す時は `<名>/brief.md` に prompt の字を、`<名>/spec.json` に係の札を書いて 0（書けなければ標準エラーに書いて通す）。
//!
//! rc は 0 か 1（使い方の誤り）だけ。置き場の解き方と引数の読みは係の口の共通の `agent_args` が持つ。
//! 席の流れの道具（Workflow）の呼びは 1 の前に全部断り、頼みの頭の群の行と計画の file の判じは 3 の前に子の `group` が持つ
//! （判断の記録 ADR-61・要件 FR22）。

pub mod group;

use std::fs;
use std::io::Read;
use std::path::Path;

use tsuzuri_core::agent::spec::group::{workflow, workflow_reason};
use tsuzuri_core::agent::spec::{BRIEF, SPEC, Spec, judge, reason, spawn_call};
use tsuzuri_core::gate::deny_json;

use super::agent_args::begin;
use crate::out::{emit, emit_err};
use crate::server::events::now;

pub const USAGE: &str = "usage: tz hook agent-spawn --repo <dir> [--drafts <dir>]";

/// 置き場の直下の dir の読める係の札の全部（読めない札は飛ばす）。
pub fn specs(drafts: &Path) -> Vec<Spec> {
    let Ok(dir) = fs::read_dir(drafts) else {
        return Vec::new();
    };
    dir.flatten()
        .filter_map(|e| fs::read_to_string(e.path().join(SPEC)).ok())
        .filter_map(|t| Spec::parse(&t))
        .collect()
}

/// 通した頼みと札を係の dir に書く。
fn place(drafts: &Path, prompt: &str, spec: &Spec) -> std::io::Result<()> {
    let dir = drafts.join(&spec.name);
    fs::create_dir_all(&dir)?;
    fs::write(dir.join(BRIEF), prompt)?;
    fs::write(dir.join(SPEC), spec.render())
}

/// tz hook agent-spawn の残りの引数を受けて終了 code を返す（0 か 1 だけ）。
pub fn run(rest: &[&str]) -> u8 {
    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        payload.clear();
    }
    if workflow(&payload) {
        emit(&deny_json(workflow_reason()));
        return 0;
    }
    let Some(call) = spawn_call(&payload) else {
        return 0;
    };
    let (args, dir) = match begin(USAGE, "通す", rest) {
        Ok(found) => found,
        Err(rc) => return rc,
    };
    let gate = match group::gate(&dir, &args.repo, &call, &payload) {
        Ok(gate) => gate,
        Err(why) => {
            emit(&deny_json(why));
            return 0;
        }
    };
    match judge(&call, &gate.live, now()) {
        Ok(spec) => {
            let seat = gate.seat.as_ref();
            let wrote = place(&dir, &call.prompt, &spec)
                .and_then(|()| seat.map_or(Ok(()), |(s, p)| group::place(&dir, &spec.name, s, p)));
            if let Err(e) = wrote {
                emit_err(&format!(
                    "tz hook agent-spawn: 頼みと札を書けない（通す）: {e}"
                ));
            }
        }
        Err(r) => emit(&deny_json(reason(&r, &call.prompt))),
    }
    0
}
