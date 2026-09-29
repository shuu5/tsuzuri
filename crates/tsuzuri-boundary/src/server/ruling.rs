//! 裁定の受付（口 POST /api/ruling・便 e-ask）。server 自身は file を書かない。
//! 台帳に書くのは bdw（契約の型の `LedgerWrite` の argv・cwd は repo）、席へ送るのは器の CLI。
//! 受付の順:
//! 1. 逐語が空白だけなら断る（EmptyVerbatim）。
//! 2. 台帳を bd の読み取りの口で読み直し（持ち回しの値を使わない）、open の問いでなければ断る
//!    （UnknownQuestion）。走っている読みには合流しない（便 e-coalesce）。読みが落ちれば `RETRY_STEP` を空けて
//!    `READ_TRIES` 回まで撃ち直し（`reread`・行 e-ruling-retry）、どれも読めなければ 503。
//! 3. 今の版の要約値が要求の値と違えば断る（StaleVersion）。
//! 4. id を発行する（`<問いの id>:<UTC の年月日 T 時分 Z>-<数>`・notes に同じ id の定型行が在れば数を増やす）。
//! 5. notes の末尾に 1 行を足し、問いを閉じる（1 回目が落ちたら 2 回目を撃たない・書きは撃ち直さない）。
//! 6. 席の target と state dir の両方が在るときだけ、別の thread で `deliver` を呼び、待たずに応答する
//!    （台帳を読み直し、印の無い裁定が在れば器の配達の口を 1 度撃ち、rc 0 なら印を置く・結果で応答は変えない）。
//!
//! 口は 200 でない応答を返す前に `refusal_line` の 1 行を標準エラーに書く（逐語は書かない）。
//!
//! 取り消し（口 POST /api/revoke・`revoke`・行 e-revoke）も同じ順で受け、問いを閉じる代わりに開き直す。
//! 取り消せるのは閉じた問いの効いている最後の裁定だけで、notes の末尾に取り消しの行を足してから開き直し、何も消さない。
//! 開き直しだけが落ちた後に同じ要求を撃ち直すと、行を足さず開き直しだけを撃ち直す。

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadId, LedgerWrite};
use tsuzuri_contract::surface::{
    QUESTION_FIELD, REVOKES, Refusal, RevokeRequest, RevokeResponse, RulingId, RulingRequest,
    RulingResponse, VERBATIM, pending_reopen, revocable,
};
use tsuzuri_core::delivery::{Pending, Route, mark_line, marked};
use tsuzuri_core::question::open_questions;

use super::events;
use super::ledger::{Source, capture, parse_bd};

/// 口の path。
pub const PATH: &str = "/api/ruling";

/// 器の CLI の既定の program の名（引数 --scribe2 で替える）。
pub const SCRIBE2: &str = "scribe2";

/// notes の裁定の定型行の頭（導出グラフが裁定の節点を導く頭と同じ字）。
pub const LINE_PREFIX: &str = "裁定 id = ";

/// 定型行の id の終わりの字。
const ID_END: char = '・';

/// bdw の 1 回の書きが返すまでの上限。越えれば止めて落ちた扱い。
/// bdw が機械で共通の錠を待つ上限の既定 60 秒に書きそのものの 60 秒を足した長さ
/// （bdw は錠を待ちきれなければ書かずに落ちるので、書きの途中で止めない・行 e-ruling-retry）。
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(120);

/// 書きの前の台帳の読みを撃つ回の上限（1 回の読みの上限は `BD_TIMEOUT`・行 e-ruling-retry）。
pub const READ_TRIES: u32 = 3;

/// 読みの撃ち直しの前に空ける時間。
pub const RETRY_STEP: Duration = Duration::from_secs(1);

/// 口が断った応答の log の行の頭（`refusal_line`）。
pub const REFUSED_LOG: &str = "tz surface serve: 断った";

/// 器の配達の口が返すまでの上限。越えれば止めて落ちた扱い。
pub const DELIVER_TIMEOUT: Duration = Duration::from_secs(10);

/// 器の配達の口が受けなかったときの log の字（失敗でなくふつうの断り）。
pub const NOT_TAKEN: &str = "配達の口が今は受けない（印を置かず、席の停止の hook が拾う）";

/// 配達の先（器の CLI・state dir・席の target）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delivery {
    pub program: OsString,
    pub state_dir: PathBuf,
    pub target: String,
}

/// 書きの出所（repo の置き場・bdw の program・配達の先）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Writer {
    pub repo: PathBuf,
    pub bdw: OsString,
    /// 席の target と state dir の両方が在るときだけ Some。
    pub delivery: Option<Delivery>,
}

/// 受付の結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// 書いた（200）。
    Recorded(RulingResponse),
    /// 断った（書きの前・状態の code は `Refusal::http_status`）。
    Refused(Refusal),
    /// 台帳が読めない（503・書きの前）。
    LedgerUnknown,
    /// 発行した id が記帳 id の形に収まらない（500・書きの前）。
    IdShape,
    /// notes への追記が落ちた（502・閉じる書きは撃っていない）。
    AppendFailed,
    /// 問いを閉じる書きが落ちた（502・notes には発行した id の行が在る）。
    CloseFailed(RulingId),
}

/// 1 問の裁定を受ける（`now` は受付の時刻）。
pub fn accept(req: &RulingRequest, ledger: &Source, writer: &Writer, now: EpochSecs) -> Outcome {
    if req.verbatim.trim().is_empty() {
        return Outcome::Refused(Refusal::EmptyVerbatim);
    }
    // 走っている読みを分け合わず、新しい子 process で読み直す（便 e-coalesce）。
    let Some(Reading::Known(questions)) = reread(ledger).map(|t| open_questions(&t)) else {
        return Outcome::LedgerUnknown;
    };
    let Some(question) = questions.into_iter().find(|q| q.card.id == req.question) else {
        return Outcome::Refused(Refusal::UnknownQuestion);
    };
    if question.card.digest != req.seen_digest {
        return Outcome::Refused(Refusal::StaleVersion);
    }
    let Ok(id) = next_id(&req.question, &question.notes, &minute(now)) else {
        return Outcome::IdShape;
    };
    let append = LedgerWrite::AppendNotes {
        id: req.question.clone(),
        line: line(&id, &req.question, &req.verbatim),
    };
    if !write(writer, &append) {
        return Outcome::AppendFailed;
    }
    let close = LedgerWrite::CloseItem {
        id: req.question.clone(),
        reason: format!("裁定 {id}"),
    };
    if !write(writer, &close) {
        return Outcome::CloseFailed(id);
    }
    if let Some(d) = writer.delivery.clone() {
        let (writer, ledger, ruling) = (writer.clone(), ledger.clone(), id.clone());
        let pending = vec![Pending {
            question: req.question.clone(),
            ruling: id.clone(),
        }];
        std::thread::spawn(move || deliver(&d, &writer, &ledger, &ruling, &pending));
    }
    Outcome::Recorded(RulingResponse {
        ruling: id,
        recorded_at: now,
    })
}

/// 取り消しの受付の結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Revoked {
    /// 書いた（200・開き直しだけの撃ち直しも）。
    Recorded(RevokeResponse),
    /// 断った（書きの前・取り消せない裁定は StaleVersion の 409）。
    Refused(Refusal),
    /// 台帳が読めない（503・書きの前）。
    LedgerUnknown,
    /// 発行した id が記帳 id の形に収まらない（500・書きの前）。
    IdShape,
    /// notes への追記が落ちた（502・開き直しは撃っていない）。
    AppendFailed,
    /// 開き直しが落ちた（502・notes には取り消しの行が在り、問いは閉じたまま）。
    ReopenFailed(RulingId),
}

/// 1 問の裁定を取り消す（`now` は受付の時刻）。
pub fn revoke(req: &RevokeRequest, ledger: &Source, writer: &Writer, now: EpochSecs) -> Revoked {
    if req.verbatim.trim().is_empty() {
        return Revoked::Refused(Refusal::EmptyVerbatim);
    }
    // 走っている読みを分け合わず、新しい子 process で読み直す（閉じた問いも読む）。
    let Some(Reading::Known(items)) = reread(ledger).map(|t| parse_bd(&t)) else {
        return Revoked::LedgerUnknown;
    };
    let Some(item) = items
        .into_iter()
        .find(|i| i.row.id == req.question && i.row.is_question())
    else {
        return Revoked::Refused(Refusal::UnknownQuestion);
    };
    if !revocable(&item, &req.ruling) {
        return Revoked::Refused(Refusal::StaleVersion);
    }
    // 開き直しだけが落ちた後の撃ち直し（行を足さず、要求の逐語は書かない）。
    if let Some(pending) = pending_reopen(&item.notes, &req.question, &req.ruling) {
        let Ok(id) = RulingId::new(pending) else {
            return Revoked::IdShape;
        };
        return reopen(req, ledger, writer, id, now, true);
    }
    let Ok(id) = next_id(&req.question, &item.notes, &minute(now)) else {
        return Revoked::IdShape;
    };
    let append = LedgerWrite::AppendNotes {
        id: req.question.clone(),
        line: revoke_line(&id, &req.question, &req.ruling, &req.verbatim),
    };
    if !write(writer, &append) {
        return Revoked::AppendFailed;
    }
    reopen(req, ledger, writer, id, now, false)
}

/// 問いを開き直し、配達の先が在れば別の thread で取り消しの行を配達する（`id` は取り消しの行の id・待たない）。
fn reopen(
    req: &RevokeRequest,
    ledger: &Source,
    writer: &Writer,
    id: RulingId,
    now: EpochSecs,
    reopened_only: bool,
) -> Revoked {
    let w = LedgerWrite::ReopenItem {
        id: req.question.clone(),
        reason: format!("裁定 {id}{ID_END}{REVOKES}{}", req.ruling),
    };
    if !write(writer, &w) {
        return Revoked::ReopenFailed(id);
    }
    if let Some(d) = writer.delivery.clone() {
        let (writer, ledger, ruling) = (writer.clone(), ledger.clone(), id.clone());
        let pending = vec![Pending {
            question: req.question.clone(),
            ruling: id.clone(),
        }];
        std::thread::spawn(move || deliver(&d, &writer, &ledger, &ruling, &pending));
    }
    Revoked::Recorded(RevokeResponse {
        ruling: id,
        recorded_at: now,
        reopened_only,
    })
}

/// bdw を 1 回撃つ（rc 0 で上限の内に返せば true）。
fn write(writer: &Writer, w: &LedgerWrite) -> bool {
    capture(&writer.bdw, w.argv(), &writer.repo, WRITE_TIMEOUT).is_some()
}

/// 書きの前の台帳の読み（合流しない読みを撃ち、落ちれば `RETRY_STEP` を空けて `READ_TRIES` 回まで撃ち直し、
/// 最初に読めた字を返す・どれも落ちれば None）。
pub fn reread(ledger: &Source) -> Option<String> {
    for n in 1..=READ_TRIES {
        if let Some(text) = ledger.text_alone() {
            return Some(text);
        }
        if n < READ_TRIES {
            std::thread::sleep(RETRY_STEP);
        }
    }
    None
}

/// 口が断った応答の log の 1 行（`<REFUSED_LOG>: <path> <状態の code> <本文の字>・問い <問いの id を , で並べた字>`）。
/// 本文の字は前後の空白を除く。逐語は応答の本文に無いので、この行にも無い。
pub fn refusal_line(path: &str, status: u16, body: &[u8], questions: &[&BeadId]) -> String {
    let ids: Vec<&str> = questions.iter().map(|q| q.as_str()).collect();
    format!(
        "{REFUSED_LOG}: {path} {status} {}・問い {}",
        String::from_utf8_lossy(body).trim(),
        ids.join(",")
    )
}

/// 器の配達の口の引数の列（program の名は含めない）。
pub fn deliver_argv(d: &Delivery, id: &RulingId) -> Vec<OsString> {
    vec![
        "seat".into(),
        "deliver".into(),
        "--state-dir".into(),
        d.state_dir.clone().into(),
        "--target".into(),
        d.target.clone().into(),
        "--ruling".into(),
        id.as_str().into(),
    ]
}

/// 受付の裁定を席へ配達する（`id` は器に渡す id・1 問は裁定の id・束は束の id）。
/// 1. 台帳を読み直し、読めて `pending` のどれにも印が在れば撃たない（停止の hook が既に返した）。
///    読めなければ撃つ側に倒す。
/// 2. 器の配達の口を 1 度だけ撃ち、受けなければ印を置かず標準エラーに 1 行を書く。
/// 3. rc 0 なら `pending` の順に、問いの notes に経路が配達の口の印を足す（落ちたものごとに 1 行）。
pub fn deliver(d: &Delivery, writer: &Writer, ledger: &Source, id: &RulingId, pending: &[Pending]) {
    if let Some(text) = ledger.text_alone()
        && pending
            .iter()
            .all(|p| marked(&text, &p.question, &p.ruling))
    {
        return;
    }
    if capture(&d.program, deliver_argv(d, id), &writer.repo, DELIVER_TIMEOUT).is_none() {
        eprintln!("tz surface serve: {NOT_TAKEN}: {id}・席 {}", d.target);
        return;
    }
    let minute = minute(events::now());
    for p in pending {
        let mark = LedgerWrite::AppendNotes {
            id: p.question.clone(),
            line: mark_line(&p.ruling, Route::Deliver, &minute),
        };
        if !write(writer, &mark) {
            eprintln!(
                "tz surface serve: 印を置けない: 裁定 {}（問い {}）は停止の hook が返す",
                p.ruling, p.question
            );
        }
    }
}

/// 次の id（数は 1 から始め、notes に同じ id の定型行が在れば 1 つずつ増やす）。
pub fn next_id(
    question: &BeadId,
    notes: &str,
    minute: &str,
) -> Result<RulingId, tsuzuri_contract::IdError> {
    let taken: Vec<&str> = notes
        .lines()
        .filter_map(|l| l.trim_end_matches('\r').strip_prefix(LINE_PREFIX))
        .map(|rest| rest.split(ID_END).next().unwrap_or(rest).trim())
        .collect();
    let mut n = 1;
    loop {
        let id = RulingId::for_question(question, minute, n)?;
        if !taken.contains(&id.as_str()) {
            return Ok(id);
        }
        n += 1;
    }
}

/// notes に足す 1 行（`裁定 id = <id>・問い = <問いの id>・逐語 = <字>`）。
pub fn line(id: &RulingId, question: &BeadId, verbatim: &str) -> String {
    format!(
        "{LINE_PREFIX}{id}{ID_END}問い = {question}{ID_END}逐語 = {}",
        escape(verbatim)
    )
}

/// notes に足す取り消しの行（`裁定 id = <id>・問い = <問いの id>・取り消す = <前の id>・逐語 = <字>`）。
pub fn revoke_line(id: &RulingId, question: &BeadId, revokes: &RulingId, verbatim: &str) -> String {
    format!(
        "{LINE_PREFIX}{id}{ID_END}{QUESTION_FIELD}{question}{ID_END}{REVOKES}{revokes}{ID_END}{VERBATIM}{}",
        escape(verbatim)
    )
}

/// 逐語を 1 行にする（逆斜線は逆斜線 2 つ・改行は逆斜線と n・復帰は逆斜線と r）。
pub fn escape(verbatim: &str) -> String {
    let mut out = String::with_capacity(verbatim.len());
    for c in verbatim.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out
}

/// epoch 秒の UTC の「年月日 T 時分 Z」（例 `20260927T1034Z`）。
pub fn minute(at: EpochSecs) -> String {
    let days = (at / 86_400) as i64;
    let secs = at % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}{m:02}{d:02}T{:02}{:02}Z",
        secs / 3600,
        secs % 3600 / 60
    )
}

/// 1970-01-01 からの日数を（年・月・日）にする（先発グレゴリオ暦）。
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::{LINE_PREFIX, escape, line, minute, next_id};
    use tsuzuri_contract::graph::NodeKind;
    use tsuzuri_contract::ledger::BeadId;
    use tsuzuri_contract::surface::RulingId;
    use tsuzuri_core::graph::build::TYPED_LINES;

    #[test]
    fn server_ask_minute_is_utc() {
        assert_eq!(minute(0), "19700101T0000Z");
        assert_eq!(minute(1_790_494_740), "20260927T0739Z");
        assert_eq!(minute(1_709_164_799), "20240228T2359Z");
        assert_eq!(minute(1_709_164_800), "20240229T0000Z");
        assert_eq!(minute(951_868_800), "20000301T0000Z");
    }

    #[test]
    fn server_ask_escape_one_line() {
        assert_eq!(escape("a\\b\nc\r\nd"), "a\\\\b\\nc\\r\\nd");
        assert_eq!(escape("そのまま"), "そのまま");
        assert!(!escape("1\n2\n").contains('\n'));
    }

    #[test]
    fn server_ask_next_id_skips_taken() {
        let q = BeadId::new("fx.1").expect("id");
        let m = "20260927T1034Z";
        assert_eq!(
            next_id(&q, "", m).expect("id").as_str(),
            "fx.1:20260927T1034Z-1"
        );
        let notes = format!(
            "前置き\n{}\n{}\r\n裁定 id = fx.1:20260927T1033Z-3・古い分",
            line(
                &RulingId::new("fx.1:20260927T1034Z-1").expect("id"),
                &q,
                "x"
            ),
            line(
                &RulingId::new("fx.1:20260927T1034Z-2").expect("id"),
                &q,
                "y"
            ),
        );
        assert_eq!(
            next_id(&q, &notes, m).expect("id").as_str(),
            "fx.1:20260927T1034Z-3"
        );
        let long = BeadId::new("a".repeat(60)).expect("id");
        assert!(next_id(&long, "", m).is_err(), "64 byte を越える id");
    }

    #[test]
    fn server_ask_line_prefix_is_graph_ruling() {
        assert_eq!(TYPED_LINES[0], (LINE_PREFIX, NodeKind::Ruling));
        let q = BeadId::new("fx.1").expect("id");
        let id = RulingId::new("fx.1:20260927T1034Z-1").expect("id");
        assert_eq!(
            line(&id, &q, "はい\nそれで"),
            "裁定 id = fx.1:20260927T1034Z-1・問い = fx.1・逐語 = はい\\nそれで"
        );
    }
}
