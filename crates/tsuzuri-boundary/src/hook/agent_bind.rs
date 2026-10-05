//! tz hook agent-bind（判断の記録 ADR-59 決定 (4)・要件 FR21）: 結びの口。
//! 撃つのは Claude Code（PostToolUse の hook・matcher Agent）。席の Agent の呼びの結果に係の id（`tool_response.agentId`）が在り、
//! 起こしの名の係の札が在る時だけ、札の `agent_id` に係の id を書き、起草の置き場の `.agents/<係の id>` に名を書く。
//! それ以外（係の呼び・ほかの道具・係の id の無い結果・札の無い名・置き場を解けない）は何も書かない。出力は無く、rc は 0 か 1（使い方の誤り）だけ。
//! 係の id の無い結果（team の形の起こし）の係は、係の門と測りと終える前の門が `late` で係の記録の隣の meta.json の名の札に結ぶ
//! （中核の `tie`・判断の記録 ADR-59 の帰結の (1)）。

use std::fs;
use std::io::{self, Read};
use std::path::Path;

use tsuzuri_core::agent::meter::SubCall;
use tsuzuri_core::agent::spec::tie::{Miss, claim, meta_name, meta_path};
use tsuzuri_core::agent::spec::{AGENTS, SPEC, Spec, bound};

use super::agent_spawn::{drafts, parse};
use crate::out::emit_err;

pub const USAGE: &str = "usage: tz hook agent-bind --repo <dir> [--drafts <dir>]";

/// 使い方の誤り。
const FAIL: u8 = 1;

/// 札 `spec` の `agent_id` に係の id を書いて `<名>/spec.json` に書き、`.agents/<係の id>` に名を書く。
pub fn tie(dir: &Path, name: &str, spec: &mut Spec, id: &str) -> io::Result<()> {
    spec.agent_id = Some(id.to_string());
    fs::write(dir.join(name).join(SPEC), spec.render())
        .and_then(|()| fs::create_dir_all(dir.join(AGENTS)))
        .and_then(|()| fs::write(dir.join(AGENTS).join(id), format!("{name}\n")))
}

/// 係の呼びの係の id を、係の記録の隣の meta.json の名の札に結ぶ（札の係の id が無いか同じで、終えの印が無い時だけ）。
pub fn late(dir: &Path, call: &SubCall) -> Result<(String, Spec), Miss> {
    let text = meta_path(&call.parent, &call.agent_id)
        .and_then(|p| fs::read_to_string(p).ok())
        .ok_or(Miss::Meta)?;
    let name = meta_name(&text).ok_or(Miss::Name)?;
    let mut spec = fs::read_to_string(dir.join(&name).join(SPEC))
        .ok()
        .and_then(|t| Spec::parse(&t))
        .filter(|s| s.name == name)
        .ok_or_else(|| Miss::Spec(name.clone()))?;
    claim(&spec, &call.agent_id)?;
    tie(dir, &name, &mut spec, &call.agent_id).map_err(|e| Miss::Write(e.to_string()))?;
    Ok((name, spec))
}

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
    if let Err(e) = tie(&dir, &name, &mut spec, &id) {
        emit_err(&format!("tz hook agent-bind: 結びを書けない: {e}"));
    }
    0
}
