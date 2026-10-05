//! 会話の印の読み（設計ノート surface-wave29b 行 cs-acct-mark・判断の記録 ADR-55 決定 (2)）。
//! 窓の会話の印の hook（会話の始まり・持ち主の入力・turn の終わり）の入力の JSON から、印の 1 行（事と会話の id と始まりの種類）を組み、
//! 作業場の `.consult/stamps.jsonl` の字から読める行の列と最後の会話の id を引く。会話の id は uuid の形の字だけを受ける。

use serde_json::Value;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::consult::{Stamp, StampEvent};

/// uuid の形（8-4-4-4-12 の 16 進の小字か大字）。
pub fn is_uuid(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    parts.len() == 5
        && parts
            .iter()
            .zip([8, 4, 4, 4, 12])
            .all(|(p, n)| p.len() == n && p.bytes().all(|c| c.is_ascii_hexdigit()))
}

/// hook の入力の JSON から印の 1 行を組む（知らない事・会話の id が uuid の形でない・JSON でない は None）。
pub fn read(payload: &str, at: EpochSecs) -> Option<Stamp> {
    let v: Value = serde_json::from_str(payload).ok()?;
    let event = StampEvent::of_hook(v.get("hook_event_name")?.as_str()?)?;
    let sid = v.get("session_id")?.as_str()?;
    if !is_uuid(sid) {
        return None;
    }
    let source = match event {
        StampEvent::Start => v.get("source").and_then(Value::as_str).map(str::to_string),
        StampEvent::Prompt | StampEvent::Stop => None,
    };
    Some(Stamp {
        at,
        event,
        sid: sid.to_string(),
        source,
    })
}

/// 印の file の字の読める行（読めない行は飛ばす・順はそのまま）。
pub fn lines(text: &str) -> Vec<Stamp> {
    text.lines()
        .filter_map(|l| serde_json::from_str::<Stamp>(l).ok())
        .collect()
}

/// 最後の会話の id（読める行の最後の行の id・無ければ None）。
pub fn last_sid(lines: &[Stamp]) -> Option<&str> {
    lines.last().map(|s| s.sid.as_str())
}
