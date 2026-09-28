//! 配達済みの印と未配達の裁定（設計ノート surface-wave6 行 f-mark・要件 FR9・器の要件 FR79）。
//! 入力は台帳の一覧の字（bd の読み取りの口が返す JSON の配列）と停止の hook の入力の字で、
//! 関数は file も子 process も時計も触らない（印の分の字は呼ぶ側が渡す）。
//!
//! 印の形: 裁定の行の在る問いの bead の notes の末尾に足す 1 行
//! 「配達 = <裁定の id>・経路 = <経路の語>・時刻 = <UTC の分>」。
//! 裁定が配達済みとは、同じ bead の notes に、行頭が `MARK_PREFIX` で頭の後から最初の「・」までの字
//! （前後の空白を除く）がその裁定の id と同じ行が 1 つ以上在ること（経路と時刻は判じに使わない）。

use serde_json::{Map, Value};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadId, QUESTION_LABEL};
use tsuzuri_contract::surface::RulingId;

use crate::graph::build::{BdBead, read_ledger};

/// 裁定の行の頭（導出グラフの `TYPED_LINES` の先頭の頭と同じ字）。
pub const RULING_PREFIX: &str = "裁定 id = ";

/// 配達済みの印の行の頭。
pub const MARK_PREFIX: &str = "配達 = ";

/// 行の id の終わりの字。
const ID_END: char = '・';

/// 印を置いた経路。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// server が器の配達の口の rc 0 の後に置く。
    Deliver,
    /// 停止の hook が答えを出した後に置く。
    Stop,
}

impl Route {
    /// 印の行の経路の語。
    pub fn word(self) -> &'static str {
        match self {
            Route::Deliver => "配達の口",
            Route::Stop => "停止",
        }
    }
}

/// 席に届いていない裁定の 1 つ（裁定の行の在る問いと、裁定の id）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    pub question: BeadId,
    pub ruling: RulingId,
}

/// 配達済みの印の 1 行（末尾に改行を足さない）。
pub fn mark_line(ruling: &RulingId, route: Route, minute: &str) -> String {
    format!(
        "{MARK_PREFIX}{ruling}{ID_END}経路 = {}{ID_END}時刻 = {minute}",
        route.word()
    )
}

/// notes の行のうち行頭が `prefix` の行の id（頭の後から最初の「・」までの字・前後の空白を除く）。
fn line_ids<'a>(notes: &'a str, prefix: &'a str) -> impl Iterator<Item = &'a str> {
    notes.lines().filter_map(move |line| {
        let rest = line.trim_end_matches('\r').strip_prefix(prefix)?;
        Some(rest.split(ID_END).next().unwrap_or(rest).trim())
    })
}

/// notes にその裁定の印が在るか。
fn has_mark(notes: &str, ruling: &RulingId) -> bool {
    line_ids(notes, MARK_PREFIX).any(|id| id == ruling.as_str())
}

fn is_question(bead: &BdBead) -> bool {
    let labels = bead.labels.as_deref().unwrap_or_default();
    labels.iter().any(|l| l == QUESTION_LABEL) && bead.status.as_deref() != Some("tombstone")
}

/// 席に届いていない裁定を台帳の配列の順（同じ bead の中は notes の行の順）に返す。
/// 問い（label `intake:question`・状態が tombstone でない・id が bead の id の形）の notes の裁定の行のうち、
/// 同じ bead の notes に印の無いものを出す（状態が open の問いも出す）。字が読めなければ Unknown。
pub fn undelivered(ledger: &str) -> Reading<Vec<Pending>> {
    let Some(beads) = read_ledger(ledger) else {
        return Reading::Unknown;
    };
    let mut out = Vec::new();
    for bead in beads.iter().filter(|b| is_question(b)) {
        let Ok(question) = BeadId::new(bead.id.clone()) else {
            continue;
        };
        let notes = bead.notes.as_deref().unwrap_or_default();
        let mut seen: Vec<RulingId> = Vec::new();
        for id in line_ids(notes, RULING_PREFIX) {
            let Ok(ruling) = RulingId::new(id) else {
                continue;
            };
            if has_mark(notes, &ruling) || seen.contains(&ruling) {
                continue;
            }
            seen.push(ruling.clone());
            out.push(Pending {
                question: question.clone(),
                ruling,
            });
        }
    }
    Reading::Known(out)
}

/// 台帳の問いの bead の notes にその裁定の印が在るか（読めない字・無い bead は偽）。
pub fn marked(ledger: &str, question: &BeadId, ruling: &RulingId) -> bool {
    read_ledger(ledger).is_some_and(|beads| {
        beads.iter().any(|b| {
            b.id == question.as_str() && has_mark(b.notes.as_deref().unwrap_or_default(), ruling)
        })
    })
}

/// 停止の hook の入力の鍵 `stop_hook_active` が JSON の true か（JSON の object でなければ None）。
pub fn stop_active(payload: &str) -> Option<bool> {
    match serde_json::from_str::<Value>(payload).ok()? {
        Value::Object(o) => Some(o.get("stop_hook_active") == Some(&Value::Bool(true))),
        _ => None,
    }
}

/// 停止の hook の答え（鍵 decision の字 block と鍵 reason の 2 つの JSON の object・未配達が無ければ None）。
/// reason は見出しの行と裁定ごとの行を改行でつなぐ（指し示しだけを返し、逐語は返さない）。
pub fn block(pending: &[Pending]) -> Option<String> {
    if pending.is_empty() {
        return None;
    }
    let mut lines = vec![format!(
        "席に届いていない持ち主の裁定が {} 件ある（逐語は台帳の問いの notes の裁定の行）。",
        pending.len()
    )];
    lines.extend(
        pending
            .iter()
            .map(|p| format!("裁定 {} が届いた（問い {}）", p.ruling, p.question)),
    );
    let mut answer = Map::new();
    answer.insert("decision".to_string(), Value::String("block".to_string()));
    answer.insert("reason".to_string(), Value::String(lines.join("\n")));
    Some(Value::Object(answer).to_string())
}
