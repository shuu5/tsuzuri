//! tz hook agent-meter（行 ag-meter・判断の記録 ADR-59 決定 (3)(4)・要件 FR21）: 係の測り。
//! 撃つのは Claude Code（PostToolUse の hook・matcher なし）。標準入力の hook の入力が係の呼び（係の id の在る呼び）の時だけ測る。順:
//! 1. 標準入力を全部読む。係の呼びでなければ（席の呼び）何も読まず何も出さずに 0。
//! 2. 起草の置き場を解く（結びの口と同じ --drafts か repo の git config の鍵）。解けなければ標準エラーに書いて通す。
//! 3. 置き場の `.agents/<係の id>` の名と、その名の係の札を読む。結びが無ければ `.agents/unbound.jsonl` に 1 行足して通す（fail-open）。
//! 4. 係の記録（親の記録の dir の下の subagents/agent-<係の id>.jsonl）を測りの札の offset から読み、中核の `feed` で足す。
//! 5. 測りの札 `<名>/meter.json` を書き、新しく越えた印が在れば残りの注ぎを PostToolUse の答えで 1 行出して 0。
//!
//! rc は 0 か 1（使い方の誤り）だけ。結びの名の解きと結びの無い呼びの記帳は、係の門と終える前の門も使う。

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use tsuzuri_core::agent::meter::{
    METER, Meter, SubCall, inject, notice, sub_call, transcript, unbound_line,
};
use tsuzuri_core::agent::spec::{AGENTS, SPEC, Spec};

use super::agent_spawn::{drafts, parse};
use crate::out::{emit, emit_err};
use crate::server::events::now;

pub const USAGE: &str = "usage: tz hook agent-meter --repo <dir> [--drafts <dir>]";

/// 結びの無い呼びの記帳の file（`.agents` の下）。
pub const UNBOUND: &str = "unbound.jsonl";

/// 使い方の誤り。
const FAIL: u8 = 1;

/// 係の id に結んだ名と札（`.agents/<係の id>` の名の係の札が読めなければ None）。
pub fn resolve(drafts: &Path, agent_id: &str) -> Option<(String, Spec)> {
    let name = fs::read_to_string(drafts.join(AGENTS).join(agent_id)).ok()?;
    let name = name.trim();
    let spec = Spec::parse(&fs::read_to_string(drafts.join(name).join(SPEC)).ok()?)?;
    (spec.name == name).then(|| (name.to_string(), spec))
}

/// 結びの無い呼びを `.agents/unbound.jsonl` に 1 行足す（書けなければ標準エラー）。
pub fn unbound(drafts: &Path, call: &SubCall) {
    let line = unbound_line(call, now());
    let wrote = fs::create_dir_all(drafts.join(AGENTS)).and_then(|()| {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(drafts.join(AGENTS).join(UNBOUND))
            .and_then(|mut f| writeln!(f, "{line}"))
    });
    if let Err(e) = wrote {
        emit_err(&format!("tz hook agent: 結びの無い呼びを記帳できない: {e}"));
    }
}

/// 係の記録の offset から後の字（読めなければ空）。
fn tail(path: &str, offset: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    if let Ok(mut f) = File::open(path)
        && f.seek(SeekFrom::Start(offset)).is_ok()
        && f.read_to_end(&mut bytes).is_err()
    {
        bytes.clear();
    }
    bytes
}

/// tz hook agent-meter の残りの引数を受けて終了 code を返す（0 か 1 だけ）。
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
            emit_err(&format!("tz hook agent-meter: {e}\n{USAGE}"));
            return FAIL;
        }
    };
    let Some(dir) = drafts(&args) else {
        emit_err("tz hook agent-meter: 起草の置き場を解けない（通す）");
        return 0;
    };
    let Some((name, spec)) = resolve(&dir, &call.agent_id) else {
        unbound(&dir, &call);
        return 0;
    };
    let Some(path) = transcript(&call.parent, &call.agent_id) else {
        return 0;
    };
    let file = dir.join(&name).join(METER);
    let mut meter = fs::read_to_string(&file)
        .ok()
        .and_then(|t| Meter::parse(&t))
        .unwrap_or_default();
    meter.feed(&tail(&path, meter.offset));
    let mark = meter.cross(spec.budget);
    if let Err(e) = fs::write(&file, meter.render()) {
        emit_err(&format!("tz hook agent-meter: 測りの札を書けない: {e}"));
    }
    if let Some(mark) = mark {
        emit(&inject(&notice(mark, &meter, spec.budget)));
    }
    0
}
