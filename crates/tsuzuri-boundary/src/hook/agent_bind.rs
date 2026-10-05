//! tz hook agent-bind（行 ag-spec・判断の記録 ADR-59 決定 (4)・要件 FR21）: 結びの口。
//! 撃つのは Claude Code（PostToolUse の hook・matcher Agent）。席の Agent の呼びの結果に係の id（`tool_response.agentId`）が在り、
//! 起こしの名の係の札が在る時だけ、札の `agent_id` に係の id を書き、起草の置き場の `.agents/<係の id>` に名を書く。
//! それ以外（係の呼び・ほかの道具・係の id の無い結果・札の無い名・置き場を解けない）は何も書かない。出力は無く、rc は 0 か 1（使い方の誤り）だけ。

use std::fs;
use std::io::Read;

use tsuzuri_core::agent::spec::{AGENTS, SPEC, Spec, bound};

use super::agent_spawn::{drafts, parse};
use crate::out::emit_err;

pub const USAGE: &str = "usage: tz hook agent-bind --repo <dir> [--drafts <dir>]";

/// 使い方の誤り。
const FAIL: u8 = 1;

/// tz hook agent-bind の残りの引数を受けて終了 code を返す（0 か 1 だけ）。
pub fn run(rest: &[&str]) -> u8 {
    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        payload.clear();
    }
    let Some((name, id)) = bound(&payload) else {
        return 0;
    };
    let args = match parse(rest) {
        Ok(args) => args,
        Err(e) => {
            emit_err(&format!("tz hook agent-bind: {e}\n{USAGE}"));
            return FAIL;
        }
    };
    let Some(dir) = drafts(&args) else {
        emit_err("tz hook agent-bind: 起草の置き場を解けない（書かない）");
        return 0;
    };
    let file = dir.join(&name).join(SPEC);
    let Some(mut spec) = fs::read_to_string(&file).ok().and_then(|t| Spec::parse(&t)) else {
        emit_err(&format!(
            "tz hook agent-bind: 係 {name} の札が無い（書かない）"
        ));
        return 0;
    };
    spec.agent_id = Some(id.clone());
    let wrote = fs::write(&file, spec.render())
        .and_then(|()| fs::create_dir_all(dir.join(AGENTS)))
        .and_then(|()| fs::write(dir.join(AGENTS).join(&id), format!("{name}\n")));
    if let Err(e) = wrote {
        emit_err(&format!("tz hook agent-bind: 結びを書けない: {e}"));
    }
    0
}
