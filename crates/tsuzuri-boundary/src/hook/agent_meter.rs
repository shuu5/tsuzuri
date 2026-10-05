//! tz hook agent-meter（判断の記録 ADR-59 決定 (3)(4)・要件 FR21）: 係の測り。
//! 撃つのは Claude Code（PostToolUse の hook・matcher なし）。標準入力の hook の入力が係の呼び（係の id の在る呼び）の時だけ測る。順:
//! 1. 標準入力を全部読む。係の呼びでなければ（席の呼び）何も読まず何も出さずに 0。
//! 2. 起草の置き場を解く（結びの口と同じ --drafts か repo の git config の鍵）。解けなければ標準エラーに書いて通す。
//! 3. 置き場の `.agents/<係の id>` の名と、その名の係の札を読む。結びが無ければ結びの口の `late` で係の記録の隣の meta.json の名の札に結び、
//!    結べなければ訳を標準エラーに 1 行書き、`.agents/unbound.jsonl` に訳と 1 行足して通す（fail-open）。
//! 4. 係の記録（親の記録の dir の下の subagents/agent-<係の id>.jsonl）を測りの札の offset から読み、中核の `feed` で足す。
//! 5. 測りの札 `<名>/meter.json` を書き、新しく越えた印が在れば残りの注ぎを PostToolUse の答えで 1 行出して 0。
//!
//! rc は 0 か 1（使い方の誤り）だけ。結びの名の解きと結びの無い呼びの記帳は、係の門と終える前の門も使う。
//! 係の dir に群の席の札 `group.json` の在る群の係は、5 で読み直しの欄も上限 `MEMBER_READ` の印で数えて両方の欄の注ぎを出し、
//! 読みの道具（Read）の path を群の id の dir に記帳する（判断の記録 ADR-61 決定 (5)(7)）。

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use tsuzuri_core::agent::meter::group::{
    READS, TWICE, first_reader, member_notice, read_line, read_path, twice_line,
};
use tsuzuri_core::agent::meter::{
    METER, Meter, SubCall, inject, notice, sub_call, transcript, unbound_line,
};
use tsuzuri_core::agent::spec::group::{MEMBER_READ, SEAT, Seat};
use tsuzuri_core::agent::spec::tie::Miss;
use tsuzuri_core::agent::spec::{AGENTS, SPEC, Spec};

use super::agent_bind::late;
use super::agent_spawn::{drafts, parse};
use crate::out::{emit, emit_err};
use crate::server::events::now;

pub const USAGE: &str = "usage: tz hook agent-meter --repo <dir> [--drafts <dir>]";

/// 結びの無い呼びの記帳の file（`.agents` の下）。
pub const UNBOUND: &str = "unbound.jsonl";

/// 使い方の誤り。
const FAIL: u8 = 1;

/// 係の呼びの係の id に結んだ名と札。`.agents/<係の id>` が無ければ `late` で meta.json の名の札に結ぶ（結べないか札が読めなければ訳）。
pub fn resolve(drafts: &Path, call: &SubCall) -> Result<(String, Spec), Miss> {
    let Ok(name) = fs::read_to_string(drafts.join(AGENTS).join(&call.agent_id)) else {
        return late(drafts, call);
    };
    let name = name.trim();
    fs::read_to_string(drafts.join(name).join(SPEC))
        .ok()
        .and_then(|t| Spec::parse(&t))
        .filter(|s| s.name == name)
        .map(|s| (name.to_string(), s))
        .ok_or_else(|| Miss::Spec(name.to_string()))
}

/// 結びの無い呼びの訳を標準エラーに 1 行書き、`.agents/unbound.jsonl` に訳と 1 行足す（書けなければ標準エラー）。
pub fn unbound(drafts: &Path, call: &SubCall, miss: &Miss) {
    let cause = miss.text();
    emit_err(&format!(
        "tz hook agent: 係の id {} を係の札に結べない（{cause}・門は通す）",
        call.agent_id
    ));
    let line = unbound_line(call, &cause, now());
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

/// 群の係の両方の欄の新しく越えた印の注ぎの字（越えた印が無ければ None）。
fn member_marks(meter: &mut Meter, budget: u64) -> Option<String> {
    let (new, read) = (meter.cross(budget), meter.cross_read(MEMBER_READ));
    (new.is_some() || read.is_some()).then(|| member_notice(new, read, meter, budget))
}

/// 群の係の読みの path を群の id の dir の reads.jsonl に足し、同じ群のほかの係が先に読んだ path なら twice.jsonl にも足す
/// （記帳だけで断らない・書けなければ標準エラー）。
fn record(drafts: &Path, name: &str, seat: &Seat, path: &str) {
    let dir = drafts.join(&seat.group);
    let reads = fs::read_to_string(dir.join(READS)).unwrap_or_default();
    let at = now();
    let mut lines = vec![(READS, read_line(name, path, at))];
    if let Some(first) = first_reader(&reads, name, path) {
        lines.push((TWICE, twice_line(name, path, &first, at)));
    }
    for (file, line) in lines {
        let wrote = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join(file))
            .and_then(|mut f| writeln!(f, "{line}"));
        if let Err(e) = wrote {
            emit_err(&format!(
                "tz hook agent-meter: 読みを記帳できない（通す）: {e}"
            ));
        }
    }
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
    let (name, spec) = match resolve(&dir, &call) {
        Ok(found) => found,
        Err(miss) => {
            unbound(&dir, &call, &miss);
            return 0;
        }
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
    let seat = fs::read_to_string(dir.join(&name).join(SEAT))
        .ok()
        .and_then(|t| Seat::parse(&t));
    let text = match &seat {
        Some(_) => member_marks(&mut meter, spec.budget),
        None => meter
            .cross(spec.budget)
            .map(|mark| notice(mark, &meter, spec.budget)),
    };
    if let Err(e) = fs::write(&file, meter.render()) {
        emit_err(&format!("tz hook agent-meter: 測りの札を書けない: {e}"));
    }
    if let (Some(seat), Some(read)) = (&seat, read_path(&payload)) {
        record(&dir, &name, seat, &read);
    }
    if let Some(text) = text {
        emit(&inject(&text));
    }
    0
}
