//! 束の受付（口 POST /api/batch）。いくつもの問いに 1 度で答える。
//! server 自身は file を書かない。書きと配達の部品は裁定の受付（`ruling`）のものを使う。
//! 受付の順:
//! 1. 行が無ければ断る（EmptyBatch）。
//! 2. 同じ問いの id が 2 度在れば断る（400 duplicate）。
//! 3. 行ごとの逐語を決める（行の逐語が空白だけでなければその字・なければ束の逐語）。空白だけの行が在れば断る（EmptyVerbatim）。
//! 4. 台帳を合流しない読みで読み直す（裁定の受付の `reread` で撃ち直し、どれも読めなければ 503）。
//! 5. 行を要求の順に確かめ、最初に当たった行の理由で、何も書かずに断る
//!    （open の問いでなければ UnknownQuestion・A-1 の印は A1InBatch・版が違えば StaleVersion）。
//! 6. 置き場（state dir）が無ければ断る（NoStateDir・何も撃たない）。
//! 7. 束の id を発行する（`batch:<分>-<数>`・台帳の字に「束 = <id>・」か縦線の欄の「 | <id> | 」が在れば数を増やす）。
//! 8. 要求の順に、行ごとに器の答えの口を束の id つきで撃つ（逐語は標準入力・器が裁定 id を作り notes の行と問いの閉じを書く）。
//!    落ちたらそこで止めて 502 で、本文は要求の全部の行の結果を要求の順に持つ（rc 0 で裁定 id に読めた行は Written・
//!    落ちた行と落ちた行より後の撃たない行は Unwritten・配達も撃たない）。落ちた行の器の 1 行は標準エラーに写す。
//! 9. 席の target と state dir の両方が在るときだけ、別の thread で裁定の受付の `redeliver` に束の id と
//!    行の順の裁定を `PACE` で渡し、待たずに応答する（周ごとに台帳を読み直し、印の無い裁定が在れば器の配達の口を
//!    束の id で撃ち、rc 0 なら行の順に印を置く。受けなければ間を空けて上限まで撃ち直す・結果で応答は変えない）。

use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::surface::{
    BatchItemResult, BatchRequest, BatchResponse, ItemOutcome, Refusal, RulingId,
};
use tsuzuri_core::delivery::Pending;
use tsuzuri_core::graph::build::BIND_SEP;
use tsuzuri_core::question::{OpenQuestion, open_questions};

use super::ledger::Source;
use super::proc::run_input;
use super::ruling::{
    PACE, Parcel, Vessel, WRITE_TIMEOUT, Writer, answer_argv, answer_failed_line, answered_id,
    minute, redeliver, reread,
};
use crate::out::emit_err;

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
    /// 置き場が無く器の答えの口を撃てない（503・何も撃っていない）。
    NoStateDir,
    /// 発行した id が記帳 id の形に収まらない（500・書きの前）。
    IdShape,
    /// どこかの行の器の答えの口が落ちた（502・行は要求の全部の行を要求の順に・
    /// 書き終えた行は Written・落ちた行と落ちた行より後の撃っていない行は Unwritten）。
    WriteFailed(BatchResponse),
}

/// 書く前に決めた 1 行（問いの id・逐語）。
struct Row<'a> {
    question: &'a BeadId,
    verbatim: &'a str,
}

/// 束を受ける（`now` は受付の時刻）。
pub fn accept(
    req: &BatchRequest,
    ledger: &Source,
    writer: &Writer,
    vessel: Vessel<'_>,
    now: EpochSecs,
) -> Outcome {
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
    let Some(text) = reread(ledger) else {
        return Outcome::LedgerUnknown;
    };
    let Reading::Known(questions) = open_questions(&text) else {
        return Outcome::LedgerUnknown;
    };
    let rows = match rows_of(req, verbatims, questions) {
        Ok(rows) => rows,
        Err(refusal) => return Outcome::Refused(refusal),
    };
    let (program, Some(state_dir)) = vessel else {
        return Outcome::NoStateDir;
    };
    let Ok(batch) = next_batch_id(&text, &minute(now)) else {
        return Outcome::IdShape;
    };
    let (items, failed) = answer_rows(writer, (program, state_dir), &rows, &batch);
    if failed {
        return Outcome::WriteFailed(BatchResponse { batch, items });
    }
    if let Some(d) = writer.delivery.clone() {
        let pending: Vec<Pending> = items
            .iter()
            .filter_map(|item| match &item.outcome {
                ItemOutcome::Written { ruling } => Some(Pending {
                    question: item.question.clone(),
                    ruling: ruling.clone(),
                }),
                _ => None,
            })
            .collect();
        let (writer, ledger) = (writer.clone(), ledger.clone());
        let parcel = Parcel {
            id: batch.clone(),
            pending,
        };
        std::thread::spawn(move || redeliver(&d, &writer, &ledger, &parcel, PACE));
    }
    Outcome::Recorded(BatchResponse { batch, items })
}

/// 行を要求の順に確かめて書く前の行の列を組む（最初に当たった行の断りの理由を返す）。
fn rows_of<'a>(
    req: &'a BatchRequest,
    verbatims: Vec<&'a str>,
    mut questions: Vec<OpenQuestion>,
) -> Result<Vec<Row<'a>>, Refusal> {
    let mut rows = Vec::with_capacity(req.items.len());
    for (item, verbatim) in req.items.iter().zip(verbatims) {
        let Some(at) = questions.iter().position(|q| q.card.id == item.question) else {
            return Err(Refusal::UnknownQuestion);
        };
        let question = questions.swap_remove(at);
        if question.card.a1 {
            return Err(Refusal::A1InBatch);
        }
        if question.card.digest != item.seen_digest {
            return Err(Refusal::StaleVersion);
        }
        rows.push(Row {
            question: &item.question,
            verbatim,
        });
    }
    Ok(rows)
}

/// 要求の順に行ごとに器の答えの口を束の id つきで撃ち、行ごとの結果と落ちたかを返す
/// （落ちた行より後の行は撃たずに Unwritten・落ちた行の器の 1 行は標準エラーに写す）。
fn answer_rows(
    writer: &Writer,
    (program, state_dir): (&OsStr, &Path),
    rows: &[Row<'_>],
    batch: &RulingId,
) -> (Vec<BatchItemResult>, bool) {
    let mut items = Vec::with_capacity(rows.len());
    let mut failed = false;
    for row in rows {
        let outcome = if failed {
            ItemOutcome::Unwritten
        } else {
            let argv = answer_argv(state_dir, &writer.repo, row.question, Some(batch));
            match run_input(
                program,
                argv,
                &writer.repo,
                row.verbatim.as_bytes(),
                WRITE_TIMEOUT,
            ) {
                Ok(out) => match answered_id(row.question, &out) {
                    Some(ruling) => ItemOutcome::Written { ruling },
                    None => {
                        failed = true;
                        ItemOutcome::Unwritten
                    }
                },
                Err(f) => {
                    emit_err(&answer_failed_line(row.question, &f));
                    failed = true;
                    ItemOutcome::Unwritten
                }
            }
        };
        items.push(BatchItemResult {
            question: row.question.clone(),
            outcome,
        });
    }
    (items, failed)
}

/// 次の束の id（数は 1 から始め、台帳の字に「束 = <id>・」か縦線の欄の「 | <id> | 」が在れば 1 つずつ増やす）。
pub fn next_batch_id(ledger: &str, minute: &str) -> Result<RulingId, tsuzuri_contract::IdError> {
    let mut n = 1;
    loop {
        let id = RulingId::for_batch(minute, n)?;
        let taken = ledger.contains(&format!("{BATCH_PREFIX}{id}{ID_END}"))
            || ledger.contains(&format!("{BIND_SEP}{id}{BIND_SEP}"));
        if !taken {
            return Ok(id);
        }
        n += 1;
    }
}
