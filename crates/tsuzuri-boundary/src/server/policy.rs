//! 方針の受付（口 POST /api/policy・便 e-batch）。全体への指示を方針の memo の notes に残す。
//! server 自身は file を書かない。問いは閉じず、器の配達の口は撃たない。
//! 受付の順:
//! 1. 逐語が空白だけなら断る（EmptyVerbatim）。
//! 2. 台帳を合流しない読みで読み直す（読めなければ 503）。
//! 3. 範囲は字 `all` か open の問いの id（どちらでもなければ 400 scope）。
//! 4. 方針の memo を探す（無ければ 503 no-policy-memo）。
//! 5. 方針の id を発行する（`policy:<分>-<数>`・memo の notes に「方針 id = <id>・」が在れば数を増やす）。
//! 6. memo の notes の末尾に 1 行を足す（落ちたら 502）。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::natural_cmp;
use tsuzuri_contract::ledger::{LedgerItem, LedgerWrite};
use tsuzuri_contract::surface::{PolicyRequest, PolicyResponse, Refusal, RulingId};
use tsuzuri_core::question::open_questions;

use super::ledger::{Source, capture, parse_bd};
use super::ruling::{WRITE_TIMEOUT, Writer, escape, minute};

/// 口の path。
pub const PATH: &str = "/api/policy";

/// 範囲が全体のときの字。
pub const SCOPE_ALL: &str = "all";

/// 方針の memo の題。
pub const MEMO_TITLE: &str = "方針";

/// notes の方針の定型行の頭（導出グラフが方針の節点を導く頭と同じ字）。
pub const LINE_PREFIX: &str = "方針 id = ";

/// 定型行の id の終わりの字。
const ID_END: char = '・';

/// 受付の結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// 書いた（200）。
    Recorded(PolicyResponse),
    /// 断った（書きの前・状態の code は `Refusal::http_status`）。
    Refused(Refusal),
    /// 台帳が読めない（503・書きの前）。
    LedgerUnknown,
    /// 範囲が `all` でも open の問いの id でもない（400・書きの前）。
    BadScope,
    /// 方針の memo が台帳に無い（503・書きの前）。
    NoMemo,
    /// 発行した id が記帳 id の形に収まらない（500・書きの前）。
    IdShape,
    /// notes への追記が落ちた（502）。
    AppendFailed,
}

/// 方針を受ける（`now` は受付の時刻）。
pub fn accept(req: &PolicyRequest, ledger: &Source, writer: &Writer, now: EpochSecs) -> Outcome {
    if req.verbatim.trim().is_empty() {
        return Outcome::Refused(Refusal::EmptyVerbatim);
    }
    // 走っている読みを分け合わず、新しい子 process で読み直す（便 e-coalesce）。
    let Some(text) = ledger.text_alone() else {
        return Outcome::LedgerUnknown;
    };
    let (Reading::Known(items), Reading::Known(questions)) =
        (parse_bd(&text), open_questions(&text))
    else {
        return Outcome::LedgerUnknown;
    };
    if req.scope != SCOPE_ALL && !questions.iter().any(|q| q.card.id.as_str() == req.scope) {
        return Outcome::BadScope;
    }
    let Some(memo) = policy_memo(&items) else {
        return Outcome::NoMemo;
    };
    let Ok(id) = next_policy_id(&memo.notes, &minute(now)) else {
        return Outcome::IdShape;
    };
    let append = LedgerWrite::AppendNotes {
        id: memo.row.id.clone(),
        line: line(&id, &req.scope, &req.verbatim),
    };
    if capture(&writer.bdw, append.argv(), &writer.repo, WRITE_TIMEOUT).is_none() {
        return Outcome::AppendFailed;
    }
    Outcome::Recorded(PolicyResponse {
        policy: id,
        recorded_at: now,
    })
}

/// 方針の memo（題が「方針」ちょうど・label `intake:memo`・closed でも tombstone でもない・親が根の epic）。
/// いくつも在れば id の自然な順で最初。
pub fn policy_memo(items: &[LedgerItem]) -> Option<&LedgerItem> {
    let root_epic = |id: &tsuzuri_contract::ledger::BeadId| {
        items
            .iter()
            .any(|i| &i.row.id == id && i.row.kind == "epic" && i.row.parent.is_none())
    };
    items
        .iter()
        .filter(|i| {
            i.row.title == MEMO_TITLE
                && i.row.is_memo()
                && i.row.status != "closed"
                && i.row.status != "tombstone"
                && i.row.parent.as_ref().is_some_and(root_epic)
        })
        .min_by(|a, b| natural_cmp(a.row.id.as_str(), b.row.id.as_str()))
}

/// 次の方針の id（数は 1 から始め、notes に「方針 id = <id>・」が在れば 1 つずつ増やす）。
pub fn next_policy_id(notes: &str, minute: &str) -> Result<RulingId, tsuzuri_contract::IdError> {
    let mut n = 1;
    loop {
        let id = RulingId::for_policy(minute, n)?;
        if !notes.contains(&format!("{LINE_PREFIX}{id}{ID_END}")) {
            return Ok(id);
        }
        n += 1;
    }
}

/// notes に足す 1 行（`方針 id = <id>・範囲 = <範囲>・逐語 = <字>`）。
pub fn line(id: &RulingId, scope: &str, verbatim: &str) -> String {
    format!(
        "{LINE_PREFIX}{id}{ID_END}範囲 = {scope}{ID_END}逐語 = {}",
        escape(verbatim)
    )
}
