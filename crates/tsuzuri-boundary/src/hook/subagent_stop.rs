//! tz hook agent-stop（行 ag-stop・判断の記録 ADR-59 決定 (4)・要件 FR21）: 係の終える前の門。
//! 撃つのは Claude Code（SubagentStop の hook）。標準入力の hook の入力が係の終わり（係の id の在る入力）の時だけ判じる。順:
//! 1. 標準入力を全部読む。係の id が無ければ何も読まず何も出さずに 0。
//! 2. 起草の置き場を解く（結びの口と同じ --drafts か repo の git config の鍵）。解けなければ標準エラーに書いて通す。
//! 3. 測りと同じ `resolve` で結びの名と札を読む。結びが無ければ測りと同じ `unbound` で `.agents/unbound.jsonl` に 1 行足して通す。
//! 4. 係の記録（入力の agent_transcript_path）と出力の dir `<名>/w` と最後の文（last_assistant_message）から、中核の `lacks` で欠けを数える。
//! 5. 欠けが在り 1 度目の終わり（stop_hook_active が真でない）なら、理由を標準エラーに書いて 2
//!    （Claude Code は SubagentStop の rc 2 を係の続けと読み、標準エラーを係に渡す）。
//! 6. 通す時は、欠けが在れば `<名>/w/STOP-GATE.txt` に欠けを 1 行ずつ書き、札の `ended` に今の時刻を書いて 0（書けなければ標準エラーに書いて通す）。
//!
//! 係の dir に群の席の札 `group.json` の在る群の係は、4 で出す物の主張の表の欠けも中核の `claim_lacks` で数える
//! （割りの主張ごとの表の行と確かさの印と証拠・行 ag-gstop・判断の記録 ADR-61 決定 (4)(8)）。
//!
//! rc は 0 か 1（使い方の誤り）か 2（1 度目の終わりの止め）。

use std::fs;
use std::io::Read;
use std::path::Path;

use tsuzuri_core::agent::meter::sub_call;
use tsuzuri_core::agent::spec::group::{SEAT, Seat};
use tsuzuri_core::agent::spec::{SPEC, Spec};
use tsuzuri_core::agent::stop::{GATE, OUT, claim_lacks, end, hold, lacks};

use super::agent_meter::{resolve, unbound};
use super::agent_spawn::{drafts, parse};
use crate::out::emit_err;
use crate::server::events::now;

pub const USAGE: &str = "usage: tz hook agent-stop --repo <dir> [--drafts <dir>]";

/// 使い方の誤り。
const FAIL: u8 = 1;

/// 1 度目の終わりの止め（Claude Code は係を続けさせる）。
const HOLD: u8 = 2;

/// 群の係（係の dir `agent` に群の席の札の在る係）の出す物の主張の表の欠け（群の係でなければ空）。
fn claim_holes(agent: &Path, spec: &Spec, out: &Path) -> Vec<String> {
    let Some(seat) = fs::read_to_string(agent.join(SEAT))
        .ok()
        .and_then(|t| Seat::parse(&t))
    else {
        return Vec::new();
    };
    let texts: Vec<String> = spec
        .outputs
        .iter()
        .filter_map(|o| fs::read_to_string(out.join(o)).ok())
        .collect();
    claim_lacks(&texts, &seat.claims)
}

/// tz hook agent-stop の残りの引数を受けて終了 code を返す（0 か 1 か 2）。
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
            emit_err(&format!("tz hook agent-stop: {e}\n{USAGE}"));
            return FAIL;
        }
    };
    let Some(dir) = drafts(&args) else {
        emit_err("tz hook agent-stop: 起草の置き場を解けない（通す）");
        return 0;
    };
    let Some((name, mut spec)) = resolve(&dir, &call.agent_id) else {
        unbound(&dir, &call);
        return 0;
    };
    let stop = end(&payload);
    let out = dir.join(&name).join(OUT);
    let shown = out.display().to_string();
    let record = fs::read(&stop.record).unwrap_or_default();
    let mut holes = lacks(
        &record,
        &spec.outputs,
        |o| out.join(o).is_file(),
        &stop.last,
        &shown,
    );
    holes.extend(claim_holes(&dir.join(&name), &spec, &out));
    if !holes.is_empty() && !stop.again {
        emit_err(&hold(&holes, &shown));
        return HOLD;
    }
    if !holes.is_empty() {
        let text: String = holes.iter().map(|h| format!("{h}\n")).collect();
        if let Err(e) = fs::create_dir_all(&out).and_then(|()| fs::write(out.join(GATE), text)) {
            emit_err(&format!(
                "tz hook agent-stop: 欠けを記帳できない（通す）: {e}"
            ));
        }
    }
    spec.ended = Some(now());
    if let Err(e) = fs::write(dir.join(&name).join(SPEC), spec.render()) {
        emit_err(&format!(
            "tz hook agent-stop: 札に終えの印を書けない（通す）: {e}"
        ));
    }
    0
}
