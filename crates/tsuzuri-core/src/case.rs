//! 器の局面の出力の読み（行 c-case-read・判断の記録 ADR-27 の決定 (10)）。
//! 字は器の fleet/lifecycle.json（出力）と fleet/lifecycle.stale（古さの印）の 2 つ（欄の正本は器の case-lifecycle §5.2）。
//! 読めない時と古い時は器の読み手（器の fleet/lifecycle_read.rs の read を比べる入力の組を空にして呼んだ形）に合わせる:
//! 古さの印の file が在って読めない（JSON でない・版が 1 でない・印の列の形が違う・同じ種類の印が 2 つ在る）か、出力が無いか
//! 読めない（JSON でない・版が 1 でない・部品の列が無い）なら部品は「まだ分からない」。古さの印が 1 つでも在れば古い。
//! 出力の字が在るのに部品が「まだ分からない」になった周は電文の unreadable を真にする（面は「読めない」と出す）。
//! 出力の字が空（file が無いか読めない）の周は、古さの印が読めなくても偽のまま（行 c-case-unreadable）。
//! 欄が欠けるか型が違う部品は落とす（任意の欄 why は欠けても落とさない・行 c-held-stage）。部品の種類と局面と手番と理由の語と古さの印の種類は閉じた列で照らさず字のまま運ぶ
//! （器の読み手は知らない語の部品を落とすが、面は知らない語を「まだ分からない」に倒すので落とさない）。

use serde_json::Value;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::case::{CaseDoc, CaseLinks, CasePart};

use crate::ledger::epoch_secs;

/// 出力と古さの印の file の版。
const VERSION: u64 = 1;

/// 出力の字と古さの印の字から電文を組む。`json` は出力の file の字（無いか読めなければ空の字）、
/// `stale` は古さの印の file の字（file が無ければ None・在って読めなければ空の字）。
/// 部品が「まだ分からない」の電文は、出力の字が空でなければ unreadable が真。
pub fn cases_of(json: &str, stale: Option<&str>) -> CaseDoc {
    let unknown = CaseDoc {
        generated_at: None,
        stale: Vec::new(),
        parts: Reading::Unknown,
        unreadable: !json.is_empty(),
    };
    let marks = match stale.map(stale_kinds) {
        None => Vec::new(),
        Some(Some(kinds)) => kinds,
        Some(None) => return unknown,
    };
    let Some(out) = versioned(json) else {
        return unknown;
    };
    let Some(parts) = out.get("parts").and_then(Value::as_array) else {
        return unknown;
    };
    CaseDoc {
        generated_at: out
            .get("generated_at")
            .and_then(Value::as_str)
            .and_then(epoch_secs),
        stale: marks,
        parts: Reading::Known(parts.iter().filter_map(part_of).collect()),
        unreadable: false,
    }
}

/// 版が 1 の JSON（JSON でないか版が違えば None）。
fn versioned(text: &str) -> Option<Value> {
    let value: Value = serde_json::from_str(text).ok()?;
    (value.get("version").and_then(Value::as_u64) == Some(VERSION)).then_some(value)
}

/// 古さの印の種類の字（file の順・形が違うか同じ種類が 2 つ在れば None）。
fn stale_kinds(text: &str) -> Option<Vec<String>> {
    let value = versioned(text)?;
    let kinds: Vec<String> = value
        .get("marks")?
        .as_array()?
        .iter()
        .map(|mark| mark.get("kind")?.as_str().map(str::to_string))
        .collect::<Option<_>>()?;
    let distinct = kinds
        .iter()
        .enumerate()
        .all(|(at, kind)| !kinds.iter().skip(at + 1).any(|other| other == kind));
    distinct.then_some(kinds)
}

/// 部品 1 つ（欄が欠けるか型が違えば None・since は読めない時刻なら None）。
fn part_of(node: &Value) -> Option<CasePart> {
    let word = |key: &str| node.get(key)?.as_str().map(str::to_string);
    Some(CasePart {
        part: word("part")?,
        id: word("id")?,
        phase: word("phase")?,
        turn: word("turn")?,
        since: optional(node, "since")?.and_then(|at| epoch_secs(&at)),
        reason: optional(node, "reason")?,
        why: loose(node, "why")?,
        closed: node.get("closed")?.as_bool()?,
        links: links_of(node.get("links")?)?,
    })
}

/// 字か null の欄（欠けるか型が違えば None）。
fn optional(node: &Value, key: &str) -> Option<Option<String>> {
    match node.get(key)? {
        Value::Null => Some(None),
        Value::String(text) => Some(Some(text.clone())),
        _ => None,
    }
}

/// 任意の字の欄（鍵が無いか null は None・字は Some・型が違えば None で部品を落とす・行 c-held-stage）。
fn loose(node: &Value, key: &str) -> Option<Option<String>> {
    match node.get(key) {
        None => Some(None),
        Some(_) => optional(node, key),
    }
}

/// 結びの on と runs（欠けた key は空の列・object でないか在る key が字の列でなければ None）。
fn links_of(node: &Value) -> Option<CaseLinks> {
    let ids = |key: &str| match node.get(key) {
        None => Some(Vec::new()),
        Some(found) => found
            .as_array()?
            .iter()
            .map(|id| id.as_str().map(str::to_string))
            .collect(),
    };
    node.is_object().then_some(())?;
    Some(CaseLinks {
        on: ids("on")?,
        runs: ids("runs")?,
    })
}
