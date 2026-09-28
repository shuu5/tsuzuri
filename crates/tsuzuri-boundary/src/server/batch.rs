//! 束の受付（口 POST /api/batch・便 e-batch）。いくつもの問いに 1 度で答える。
//! server 自身は file を書かない。書きと配達の部品は裁定の受付（`ruling`）のものを使う。
//! 受付の順:
//! 1. 行が無ければ断る（EmptyBatch）。
//! 2. 同じ問いの id が 2 度在れば断る（400 duplicate）。
//! 3. 行ごとの逐語を決める（行の逐語が空白だけでなければその字・なければ束の逐語）。空白だけの行が在れば断る（EmptyVerbatim）。
//! 4. 台帳を合流しない読みで読み直す（読めなければ 503）。
//! 5. 行を要求の順に確かめ、最初に当たった行の理由で、何も書かずに断る
//!    （open の問いでなければ UnknownQuestion・A-1 の印は A1InBatch・版が違えば StaleVersion）。
//! 6. 束の id を発行する（`batch:<分>-<数>`・台帳の字に「束 = <id>・」が在れば数を増やす）。
//! 7. 行ごとの裁定の id を裁定の受付と同じ決め方で発行する。
//! 8. 要求の順に、行ごとに notes の末尾へ 1 行を足し、問いを閉じる。落ちたらそこで止めて 502 で、
//!    本文は要求の全部の行の結果を要求の順に持つ（2 回とも書き終えた行は Written・追記が落ちた行は Unwritten・
//!    閉じる書きが落ちた行は Unclosed・落ちた行より後の行は撃たずに Unwritten・何も消さず配達も撃たない）。
//! 9. 席の target と state dir の両方が在るときだけ、裁定の受付の `deliver` に束の id と行の順の裁定を
//!    1 度だけ渡す（台帳を読み直し、印の無い裁定が在れば器の配達の口を束の id で 1 度撃ち、
//!    rc 0 なら行の順に印を置く・結果で応答は変えない）。

use std::collections::HashSet;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadId, LedgerWrite};
use tsuzuri_contract::surface::{
    BatchItemResult, BatchRequest, BatchResponse, ItemOutcome, Refusal, RulingId,
};
use tsuzuri_core::delivery::Pending;
use tsuzuri_core::question::open_questions;

use super::ledger::{Source, capture};
use super::ruling::{LINE_PREFIX, WRITE_TIMEOUT, Writer, deliver, escape, minute, next_id};

/// 口の path。
pub const PATH: &str = "/api/batch";

/// 定型行の束の id の頭。
pub const BATCH_PREFIX: &str = "束 = ";

/// 定型行の id の終わりの字。
const ID_END: char = '・';

/// 受付の結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// 全部の行を書いた（200）。
    Recorded(BatchResponse),
    /// 断った（書きの前・状態の code は `Refusal::http_status`）。
    Refused(Refusal),
    /// 同じ問いの id が 2 度在る（400・書きの前）。
    Duplicate,
    /// 台帳が読めない（503・書きの前）。
    LedgerUnknown,
    /// 発行した id が記帳 id の形に収まらない（500・書きの前）。
    IdShape,
    /// どこかの書きが落ちた（502・行は要求の全部の行を要求の順に・2 回とも書き終えた行は Written・
    /// 追記が落ちた行は Unwritten・閉じる書きが落ちた行は Unclosed・落ちた行より後の撃っていない行は Unwritten）。
    WriteFailed(BatchResponse),
}

/// 書く前に決めた 1 行（問いの id・逐語・notes）。
struct Row<'a> {
    question: &'a BeadId,
    verbatim: &'a str,
    notes: String,
}

/// 束を受ける（`now` は受付の時刻）。
pub fn accept(req: &BatchRequest, ledger: &Source, writer: &Writer, now: EpochSecs) -> Outcome {
    if req.items.is_empty() {
        return Outcome::Refused(Refusal::EmptyBatch);
    }
    let mut seen = HashSet::new();
    if !req.items.iter().all(|i| seen.insert(&i.question)) {
        return Outcome::Duplicate;
    }
    let mut verbatims = Vec::with_capacity(req.items.len());
    for item in &req.items {
        let own = item.verbatim.as_deref().filter(|v| !v.trim().is_empty());
        let verbatim = own.unwrap_or(&req.verbatim);
        if verbatim.trim().is_empty() {
            return Outcome::Refused(Refusal::EmptyVerbatim);
        }
        verbatims.push(verbatim);
    }
    // 走っている読みを分け合わず、新しい子 process で読み直す（便 e-coalesce）。
    let Some(text) = ledger.text_alone() else {
        return Outcome::LedgerUnknown;
    };
    let Reading::Known(mut questions) = open_questions(&text) else {
        return Outcome::LedgerUnknown;
    };
    let mut rows = Vec::with_capacity(req.items.len());
    for (item, verbatim) in req.items.iter().zip(verbatims) {
        let Some(at) = questions.iter().position(|q| q.card.id == item.question) else {
            return Outcome::Refused(Refusal::UnknownQuestion);
        };
        let question = questions.swap_remove(at);
        if question.card.a1 {
            return Outcome::Refused(Refusal::A1InBatch);
        }
        if question.card.digest != item.seen_digest {
            return Outcome::Refused(Refusal::StaleVersion);
        }
        rows.push(Row {
            question: &item.question,
            verbatim,
            notes: question.notes,
        });
    }
    let minute = minute(now);
    let Ok(batch) = next_batch_id(&text, &minute) else {
        return Outcome::IdShape;
    };
    let Ok(ids) = rows
        .iter()
        .map(|r| next_id(r.question, &r.notes, &minute))
        .collect::<Result<Vec<_>, _>>()
    else {
        return Outcome::IdShape;
    };
    let mut items = Vec::with_capacity(rows.len());
    let mut failed = false;
    for (row, id) in rows.iter().zip(&ids) {
        let outcome = if failed {
            ItemOutcome::Unwritten
        } else {
            let append = LedgerWrite::AppendNotes {
                id: row.question.clone(),
                line: line(id, row.question, &batch, row.verbatim),
            };
            let close = LedgerWrite::CloseItem {
                id: row.question.clone(),
                reason: format!("裁定 {id}{ID_END}束 {batch}"),
            };
            if !write(writer, &append) {
                failed = true;
                ItemOutcome::Unwritten
            } else if !write(writer, &close) {
                failed = true;
                ItemOutcome::Unclosed { ruling: id.clone() }
            } else {
                ItemOutcome::Written { ruling: id.clone() }
            }
        };
        items.push(BatchItemResult {
            question: row.question.clone(),
            outcome,
        });
    }
    if failed {
        return Outcome::WriteFailed(BatchResponse { batch, items });
    }
    if let Some(d) = &writer.delivery {
        let pending: Vec<Pending> = rows
            .iter()
            .zip(ids)
            .map(|(row, ruling)| Pending {
                question: row.question.clone(),
                ruling,
            })
            .collect();
        deliver(d, writer, ledger, &batch, &pending);
    }
    Outcome::Recorded(BatchResponse { batch, items })
}

/// bdw を 1 回撃つ（rc 0 で上限の内に返せば true）。
fn write(writer: &Writer, w: &LedgerWrite) -> bool {
    capture(&writer.bdw, w.argv(), &writer.repo, WRITE_TIMEOUT).is_some()
}

/// 次の束の id（数は 1 から始め、台帳の字に「束 = <id>・」が在れば 1 つずつ増やす）。
pub fn next_batch_id(ledger: &str, minute: &str) -> Result<RulingId, tsuzuri_contract::IdError> {
    let mut n = 1;
    loop {
        let id = RulingId::for_batch(minute, n)?;
        if !ledger.contains(&format!("{BATCH_PREFIX}{id}{ID_END}")) {
            return Ok(id);
        }
        n += 1;
    }
}

/// notes に足す 1 行（`裁定 id = <id>・問い = <問いの id>・束 = <束の id>・逐語 = <字>`）。
pub fn line(id: &RulingId, question: &BeadId, batch: &RulingId, verbatim: &str) -> String {
    format!(
        "{LINE_PREFIX}{id}{ID_END}問い = {question}{ID_END}{BATCH_PREFIX}{batch}{ID_END}逐語 = {}",
        escape(verbatim)
    )
}
