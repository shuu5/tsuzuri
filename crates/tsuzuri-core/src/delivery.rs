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
    /// tool の呼びの組の後の hook が答えを出した後に置く（行 f-deliver-tool）。
    Tool,
}

impl Route {
    /// 印の行の経路の語。
    pub fn word(self) -> &'static str {
        match self {
            Route::Deliver => "配達の口",
            Route::Stop => "停止",
            Route::Tool => "tool の呼び",
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

/// 器の配達の口が待ちの席へ送る指し示しの行の頭（頭の前の器の名の字は見ない）。
pub const POINTER_HEAD: &str = "seat: 裁定 ";

/// 指し示しの行の尾（頭と尾の間が記帳 id）。
pub const POINTER_TAIL: &str = " が届いた（在りかは裁定面の記帳）";

/// 裁定の行の束の欄の頭（境界の server の束の受付が書く字と同じ）。
pub const BATCH_FIELD: &str = "束 = ";

/// 席の文脈に写す逐語の字数の合計の上限（Claude Code は hook の文脈が 10000 字を越えると file に落とす）。
pub const CONTEXT_CAP: usize = 8000;

/// 裁定の行の逐語の欄の前の区切り（server の書きが欄の間に置く字と欄の頭）。
const VERBATIM_FIELD: &str = "・逐語 = ";

/// 名指された裁定の 1 つ（問い・裁定の id・戻した逐語）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Said {
    pub question: BeadId,
    pub ruling: RulingId,
    pub verbatim: String,
}

/// UserPromptSubmit の hook の入力の鍵 `prompt` の字の行から、指し示しの記帳 id を行の順に重なりを除いて拾う
/// （記帳 id の形でない字・尾の無い行・JSON の object でない入力・鍵 prompt が字でない入力は拾わない）。
pub fn pointed(payload: &str) -> Vec<RulingId> {
    let Ok(Value::Object(input)) = serde_json::from_str::<Value>(payload) else {
        return Vec::new();
    };
    let Some(Value::String(prompt)) = input.get("prompt") else {
        return Vec::new();
    };
    let mut out: Vec<RulingId> = Vec::new();
    for line in prompt.lines() {
        let Some((_, rest)) = line.split_once(POINTER_HEAD) else {
            continue;
        };
        let Some((id, _)) = rest.split_once(POINTER_TAIL) else {
            continue;
        };
        if let Ok(id) = RulingId::new(id)
            && !out.contains(&id)
        {
            out.push(id);
        }
    }
    out
}

/// server の escape の逆（逆斜線 2 つは逆斜線・逆斜線と n は改行・逆斜線と r は復帰・ほかの逆斜線はそのまま）。
fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.clone().next() {
            Some('\\') => out.push('\\'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            _ => {
                out.push('\\');
                continue;
            }
        }
        chars.next();
    }
    out
}

/// 名指された裁定の逐語を台帳の順に返す。問い（label `intake:question`・状態が tombstone でない）の notes の
/// 裁定の行のうち、行の id か束の欄の id が `ids` に在り、逐語の欄の在る行を出す（印は見ない）。字が読めなければ Unknown。
pub fn said(ledger: &str, ids: &[RulingId]) -> Reading<Vec<Said>> {
    let Some(beads) = read_ledger(ledger) else {
        return Reading::Unknown;
    };
    let named = |id: &str| ids.iter().any(|n| n.as_str() == id);
    let mut out = Vec::new();
    for bead in beads.iter().filter(|b| is_question(b)) {
        let Ok(question) = BeadId::new(bead.id.clone()) else {
            continue;
        };
        for line in bead.notes.as_deref().unwrap_or_default().lines() {
            let Some(rest) = line.trim_end_matches('\r').strip_prefix(RULING_PREFIX) else {
                continue;
            };
            let Some((head, verbatim)) = rest.split_once(VERBATIM_FIELD) else {
                continue;
            };
            let mut fields = head.split(ID_END);
            let Ok(ruling) = RulingId::new(fields.next().unwrap_or_default().trim()) else {
                continue;
            };
            let batch = fields.find_map(|f| f.strip_prefix(BATCH_FIELD)).map(str::trim);
            if named(ruling.as_str()) || batch.is_some_and(named) {
                out.push(Said {
                    question: question.clone(),
                    ruling,
                    verbatim: unescape(verbatim),
                });
            }
        }
    }
    Reading::Known(out)
}

/// tool の呼びの組の後の hook の入力の鍵 `hook_event_name` の字（行 f-deliver-tool）。
pub const TOOL_EVENT: &str = "PostToolBatch";

/// 下請けの agent の中の呼びと本体の呼びを分ける hook の入力の最上位の鍵（在れば本体の呼びでない）。
pub const AGENT_KEYS: [&str; 2] = ["agent_id", "agent_type"];

/// hook の入力が席の本体の呼びか（JSON の object で、最上位に `AGENT_KEYS` のどの鍵も無い時だけ真・入れ子の鍵は見ない）。
pub fn main_thread(payload: &str) -> bool {
    match serde_json::from_str::<Value>(payload) {
        Ok(Value::Object(input)) => !AGENT_KEYS.iter().any(|key| input.contains_key(*key)),
        _ => false,
    }
}

/// 印の無い裁定（`undelivered` の組）の逐語を `undelivered` の順に 1 度ずつ返す（逐語の欄の無い行は出さない）。
/// 字が読めなければ Unknown。
pub fn unmarked(ledger: &str) -> Reading<Vec<Said>> {
    let Reading::Known(pending) = undelivered(ledger) else {
        return Reading::Unknown;
    };
    let ids: Vec<RulingId> = pending.iter().map(|p| p.ruling.clone()).collect();
    let Reading::Known(found) = said(ledger, &ids) else {
        return Reading::Unknown;
    };
    Reading::Known(
        pending
            .iter()
            .filter_map(|p| {
                found
                    .iter()
                    .find(|s| s.question == p.question && s.ruling == p.ruling)
                    .cloned()
            })
            .collect(),
    )
}

/// 前から逐語の字数の合計が `CONTEXT_CAP` 以下の間の数（文脈が逐語を写す裁定の数）。
fn copied(said: &[Said]) -> usize {
    let mut total = 0;
    said.iter()
        .take_while(|s| {
            total += s.verbatim.chars().count();
            total <= CONTEXT_CAP
        })
        .count()
}

/// 答えが名指す裁定の数（逐語を写す裁定と、上限で写さない最初の裁定）。
pub fn named(said: &[Said]) -> usize {
    (copied(said) + 1).min(said.len())
}

/// UserPromptSubmit の hook の答え（鍵 hookSpecificOutput の下の hookEventName と additionalContext・空なら None）。
pub fn context(said: &[Said]) -> Option<String> {
    context_for(said, "UserPromptSubmit")
}

/// `context` の hookEventName を `event` の字にした答え。
/// 文脈は見出しの行と、裁定ごとの行「裁定 <id>（問い <問いの id>）の逐語:」と逐語を改行でつなぐ（事実の形の字）。
/// 写した逐語の字数の合計が `CONTEXT_CAP` を越える裁定からは逐語を写さず、在りかの 1 行で終える。
pub fn context_for(said: &[Said], event: &str) -> Option<String> {
    if said.is_empty() {
        return None;
    }
    let mut lines = vec![format!(
        "席に届いた持ち主の裁定の逐語（{} 件・台帳の問いの notes の裁定の行の写し）:",
        said.len()
    )];
    let copied = copied(said);
    for s in said.iter().take(copied) {
        lines.push(format!("裁定 {}（問い {}）の逐語:", s.ruling, s.question));
        lines.push(s.verbatim.clone());
    }
    if let Some(first) = said.get(copied) {
        lines.push(format!(
            "裁定 {}（問い {}）を含む {} 件の逐語は上限 {CONTEXT_CAP} 字を越えるので写さない（在りかは台帳の問いの notes の裁定の行）",
            first.ruling,
            first.question,
            said.len() - copied
        ));
    }
    let mut inner = Map::new();
    inner.insert(
        "hookEventName".to_string(),
        Value::String(event.to_string()),
    );
    inner.insert(
        "additionalContext".to_string(),
        Value::String(lines.join("\n")),
    );
    let mut answer = Map::new();
    answer.insert("hookSpecificOutput".to_string(), Value::Object(inner));
    Some(Value::Object(answer).to_string())
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
