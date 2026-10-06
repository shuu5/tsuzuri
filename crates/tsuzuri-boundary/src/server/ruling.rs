//! 裁定の受付（口 POST /api/ruling）。server 自身は file を書かない。
//! 台帳に書くのは bdw（契約の型の `LedgerWrite` の argv・cwd は repo）、席へ送るのは器の CLI。
//! 受付の順:
//! 1. 逐語が空白だけなら断る（EmptyVerbatim）。
//! 2. 台帳を bd の読み取りの口で読み直し（持ち回しの値を使わない）、open の問いでなければ断る
//!    （UnknownQuestion）。走っている読みには合流しない。1 回の読みの上限は `READ_TIMEOUT`
//!    （表示の読みの `BD_TIMEOUT` でなく書きの前の読みの上限）。読みが落ちれば `RETRY_STEP` を空けて
//!    `READ_TRIES` 回まで撃ち直し（`reread`）、どれも読めなければ回ごとの落ちた訳を並べた
//!    `unread_line` の 1 行を標準エラーに書いて 503。
//! 3. 今の版の要約値が要求の値と違えば断る（StaleVersion）。
//! 4. 置き場（state dir）が無ければ断る（NoStateDir・何も撃たない）。
//! 5. 器の答えの口（`answer_argv`）を逐語を標準入力にして 1 回だけ撃つ（器が発話の分から id を作り、notes の行と
//!    問いの閉じを書く・書きは撃ち直さない）。rc が 0 でなければ AnswerFailed、rc 0 でも標準出力が答えた問いの id の形
//!    （`answered_id`）でなければ IdShape。裁定 id は器が返した字。
//! 6. 席の target と state dir の両方が在るときだけ、別の thread で `redeliver` を `PACE` で呼び、待たずに応答する
//!    （周ごとに台帳を読み直し、印の無い裁定が在れば器の配達の口を撃ち、rc 0 なら印を置く。
//!    受けなければ `DELIVER_STEP` を空けて `DELIVER_SPAN` まで撃ち直す・結果で応答は変えない）。
//!
//! 答えごとの撃ち直しの thread は process の中にだけ在り、server の起こし直しで消える。起動の入口（tz surface serve）は
//! 口を開いた後に `sweep_at_start` で起動の掃き（`sweep`）を別の thread で 1 度だけ始め、`PACE` の上限の内に記帳した
//! 印の無い裁定を台帳から拾って同じ `redeliver` で撃ち直す（読むだけの server と配達の先の無い server では始めない）。
//!
//! 口は 200 でない応答を返す前に `refusal_line` の 1 行を標準エラーに書く（逐語は書かない）。
//! 読むだけの server（引数 --read-only）は、答えと方針の口を受付の前に 403 の `READ_ONLY` で断り、
//! 台帳の読みも書きも配達も撃たない。
//!
//! 取り消し（口 POST /api/revoke・`revoke`）も同じ順で受け、問いを閉じる代わりに開き直す。
//! 取り消せるのは閉じた問いの効いている最後の裁定だけで、notes の末尾に取り消しの行を足してから開き直し、何も消さない。
//! 開き直しだけが落ちた後に同じ要求を撃ち直すと、行を足さず開き直しだけを撃ち直す。

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadId, LedgerWrite};
use tsuzuri_contract::surface::{
    QUESTION_FIELD, REVOKES, Refusal, RevokeRequest, RevokeResponse, RulingId, RulingRequest,
    RulingResponse, VERBATIM, pending_reopen, revocable,
};
use tsuzuri_core::delivery::{Pending, Route, mark_line, marked, undelivered};
use tsuzuri_core::graph::build::bind_line;
use tsuzuri_core::question::open_questions;

use super::Config;
use super::events;
use super::ledger::{Source, capture, parse_bd};
use super::proc::{Failed, run, run_input};
use crate::out::emit_err;

/// 口の path。
pub const PATH: &str = "/api/ruling";

/// 器の CLI の既定の program の名（引数 --scribe2 で替える）。
pub const SCRIBE2: &str = "scribe2";

/// notes の裁定の定型行の頭（導出グラフが裁定の節点を導く頭と同じ字）。
pub const LINE_PREFIX: &str = "裁定 id = ";

/// 定型行の id の終わりの字。
const ID_END: char = '・';

/// close と開き直しの理由の頭（器の close の理由の読み手が裁定と読む頭・2 語目に裁定 id を置く）。
pub const REASON_HEAD: &str = "裁定";

/// 方針の閉じの理由の 2 語目の頭（器の読み手が裁定 id の形として受ける `policy:<字>`）。
pub const POLICY_MARK: &str = "policy:";

/// bdw の 1 回の書きが返すまでの上限。越えれば止めて落ちた扱い。
/// bdw が機械で共通の錠を待つ上限の既定 60 秒に書きそのものの 60 秒を足した長さ
/// （bdw は錠を待ちきれなければ書かずに落ちるので、書きの途中で止めない・行 e-ruling-retry）。
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(120);

/// 書きの前の台帳の読みを撃つ回の上限（1 回の読みの上限は `READ_TIMEOUT`・行 e-ruling-retry）。
pub const READ_TRIES: u32 = 3;

/// 読みの撃ち直しの前に空ける時間。
pub const RETRY_STEP: Duration = Duration::from_secs(1);

/// 書きの前の台帳の 1 回の読みが返すまでの上限（書きそのものの前に 1 度だけ撃つ読みなので、表示の読みの
/// `BD_TIMEOUT` に揃えない・行 e-answer-reread）。
pub const READ_TIMEOUT: Duration = Duration::from_secs(20);

/// 書きの前の読みがどれも落ちたときの log の字（`unread_line`）。
pub const UNREAD: &str = "書きの前の台帳の読みが落ちた";

/// 口が断った応答の log の行の頭（`refusal_line`）。
pub const REFUSED_LOG: &str = "tz surface serve: 断った";

/// 読むだけの server が答えと方針の口を断る 403 の本文（行 e-ask-own-only）。
pub const READ_ONLY: &str = "read-only";

/// 器の配達の口が返すまでの上限。越えれば止めて落ちた扱い。
pub const DELIVER_TIMEOUT: Duration = Duration::from_secs(10);

/// 器の配達の口が受けなかったときの log の字（失敗でなくふつうの断り）。
pub const NOT_TAKEN: &str = "配達の口が今は受けない（印を置かず、席の停止の hook が拾う）";

/// 配達の撃ち直しの前に空ける時間（行 e-deliver-retry）。
pub const DELIVER_STEP: Duration = Duration::from_secs(15);

/// 最初の周から撃ち直しを続ける上限（30 分）。
pub const DELIVER_SPAN: Duration = Duration::from_secs(1800);

/// 配達の撃ち直しの間と上限の組。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pace {
    pub step: Duration,
    pub span: Duration,
}

/// server の配達の撃ち直しの組。
pub const PACE: Pace = Pace {
    step: DELIVER_STEP,
    span: DELIVER_SPAN,
};

/// 配達の撃ち直しを上限で止めたときの log の字。
pub const GAVE_UP: &str = "配達の撃ち直しを上限で止めた（印を置かず、席の停止の hook が拾う）";

/// 起動の掃きが印の無い裁定を見つけたときの log の字（行 c-deliver-retry）。
pub const SWEEP_FOUND: &str = "起動の周に印の無い裁定を撃ち直す";

/// 起動の掃きの台帳の読みが落ちたときの log の字（何も撃たない）。
pub const SWEEP_UNREAD: &str =
    "起動の周の台帳の読みが落ちた（撃ち直さず、印の無い裁定は席の停止の hook が拾う）";

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

/// 器の答えの口の撃ち先（器の program と置き場・置き場が無ければ None）。
pub type Vessel<'a> = (&'a OsStr, Option<&'a Path>);

/// 受付の結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// 書いた（200）。
    Recorded(RulingResponse),
    /// 断った（書きの前・状態の code は `Refusal::http_status`）。
    Refused(Refusal),
    /// 台帳が読めない（503・書きの前）。
    LedgerUnknown,
    /// 置き場が無く器の答えの口を撃てない（503・何も撃っていない）。
    NoStateDir,
    /// 器の答えの口が返した字が答えた問いの id の形でない（500）。
    IdShape,
    /// 器の答えの口が落ちた（502）。
    AnswerFailed,
}

/// 1 問の裁定を受ける（`now` は受付の時刻）。
pub fn accept(
    req: &RulingRequest,
    ledger: &Source,
    writer: &Writer,
    vessel: Vessel<'_>,
    now: EpochSecs,
) -> Outcome {
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
    let (program, Some(state_dir)) = vessel else {
        return Outcome::NoStateDir;
    };
    let argv = answer_argv(state_dir, &writer.repo, &req.question, None);
    let out = match run_input(
        program,
        argv,
        &writer.repo,
        req.verbatim.as_bytes(),
        WRITE_TIMEOUT,
    ) {
        Ok(out) => out,
        Err(failed) => {
            emit_err(&answer_failed_line(&req.question, &failed));
            return Outcome::AnswerFailed;
        }
    };
    let Some(id) = answered_id(&req.question, &out) else {
        return Outcome::IdShape;
    };
    if let Some(d) = writer.delivery.clone() {
        let (writer, ledger) = (writer.clone(), ledger.clone());
        let parcel = Parcel {
            id: id.clone(),
            pending: vec![Pending {
                question: req.question.clone(),
                ruling: id.clone(),
            }],
        };
        std::thread::spawn(move || redeliver(&d, &writer, &ledger, &parcel, PACE));
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
        let done = RevokeResponse {
            ruling: id,
            recorded_at: now,
            reopened_only: true,
        };
        return reopen(req, ledger, writer, done);
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
    let done = RevokeResponse {
        ruling: id,
        recorded_at: now,
        reopened_only: false,
    };
    reopen(req, ledger, writer, done)
}

/// 問いを開き直し、配達の先が在れば別の thread で取り消しの行を撃ち直しつきで配達する（`id` は取り消しの行の id・待たない）。
fn reopen(
    req: &RevokeRequest,
    ledger: &Source,
    writer: &Writer,
    RevokeResponse {
        ruling: id,
        recorded_at: now,
        reopened_only,
    }: RevokeResponse,
) -> Revoked {
    let w = LedgerWrite::ReopenItem {
        id: req.question.clone(),
        reason: reason(&id, Some(&format!("{REVOKES}{}", req.ruling))),
    };
    if !write(writer, &w) {
        return Revoked::ReopenFailed(id);
    }
    if let Some(d) = writer.delivery.clone() {
        let (writer, ledger) = (writer.clone(), ledger.clone());
        let parcel = Parcel {
            id: id.clone(),
            pending: vec![Pending {
                question: req.question.clone(),
                ruling: id.clone(),
            }],
        };
        std::thread::spawn(move || redeliver(&d, &writer, &ledger, &parcel, PACE));
    }
    Revoked::Recorded(RevokeResponse {
        ruling: id,
        recorded_at: now,
        reopened_only,
    })
}

/// close と開き直しの理由（`裁定 <id>`・続きが在れば半角の空白 1 つを挟んで `裁定 <id> <続き>`）。
/// 器の読み手は空白で割った 2 語目だけを裁定 id と読み、続きは読まない（台帳を人が読む時の手がかり）。
pub fn reason(id: &RulingId, tail: Option<&str>) -> String {
    match tail {
        Some(tail) => format!("{REASON_HEAD} {id} {tail}"),
        None => format!("{REASON_HEAD} {id}"),
    }
}

/// 方針の問いの閉じの理由（`裁定 policy:<方針の id>`・方針は承認に数えない印を 2 語目の頭に置く）。
pub fn policy_reason(id: &RulingId) -> String {
    format!("{REASON_HEAD} {POLICY_MARK}{id}")
}

/// bdw を 1 回撃つ（rc 0 で上限の内に返せば true）。
fn write(writer: &Writer, w: &LedgerWrite) -> bool {
    capture(&writer.bdw, w.argv(), &writer.repo, WRITE_TIMEOUT).is_some()
}

/// 書きの前の台帳の読み（合流しない読みを上限 `READ_TIMEOUT` で撃ち、落ちれば `RETRY_STEP` を空けて
/// `READ_TRIES` 回まで撃ち直し、最初に読めた字を返す・どれも落ちれば `unread_line` の 1 行を標準エラーに書いて None）。
pub fn reread(ledger: &Source) -> Option<String> {
    let mut words = Vec::new();
    for n in 1..=READ_TRIES {
        match ledger.text_within(READ_TIMEOUT) {
            Ok(text) => return Some(text),
            Err(word) => words.push(word),
        }
        if n < READ_TRIES {
            std::thread::sleep(RETRY_STEP);
        }
    }
    emit_err(&unread_line(&words));
    None
}

/// 書きの前の読みがどれも落ちたときの log の 1 行（`tz surface serve: <UNREAD>: <回ごとの訳を ・ で並べた字>`）。
pub fn unread_line(words: &[String]) -> String {
    format!("tz surface serve: {UNREAD}: {}", words.join("・"))
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

/// 配達の 1 周の結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Round {
    /// どれにも印が在った（撃たない）。
    Marked,
    /// 器が受けた（rc 0・印の書きは撃った）。
    Taken,
    /// 器が受けなかった（`Failed::word` の字・印を置かない）。
    NotTaken(String),
}

/// 受付の裁定を席へ配達する 1 周（`id` は器に渡す id・1 問は裁定の id・束は束の id）。
/// 1. 台帳を読み直し、読めて `pending` のどれにも印が在れば撃たない（停止の hook が既に返した）。
///    読めなければ撃つ側に倒す。
/// 2. 器の配達の口を 1 度だけ撃ち、受けなければ印を置かず NotTaken を返す（log は書かない）。
/// 3. rc 0 なら `pending` の順に、問いの notes に経路が配達の口の印を足す（落ちたものごとに 1 行）。
pub fn deliver(
    d: &Delivery,
    writer: &Writer,
    ledger: &Source,
    id: &RulingId,
    pending: &[Pending],
) -> Round {
    if let Some(text) = ledger.text_alone()
        && pending
            .iter()
            .all(|p| marked(&text, &p.question, &p.ruling))
    {
        return Round::Marked;
    }
    if let Err(failed) = run(&d.program, deliver_argv(d, id), &writer.repo, DELIVER_TIMEOUT) {
        return Round::NotTaken(failed.word());
    }
    let minute = minute(events::now());
    for p in pending {
        let mark = LedgerWrite::AppendNotes {
            id: p.question.clone(),
            line: mark_line(&p.ruling, Route::Deliver, &minute),
        };
        if !write(writer, &mark) {
            emit_err(&format!(
                "tz surface serve: 印を置けない: 裁定 {}（問い {}）は停止の hook が返す",
                p.ruling, p.question
            ));
        }
    }
    Round::Taken
}

/// 撃ち直す配達の荷（裁定か取り消しか束の id と、配達する問いと裁定の組の列）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parcel {
    pub id: RulingId,
    pub pending: Vec<Pending>,
}

/// `deliver` を Marked か Taken まで撃ち直す（周ごとに台帳を読み直す）。
/// NotTaken なら、最初の周と字が前の周と違う周だけ標準エラーに 1 行を書き、最初の周から `pace.span` を越えない間は
/// `pace.step` を空けて次の周を撃つ。次の周が上限を越えるなら `GAVE_UP` の 1 行を書いて終える。
pub fn redeliver(d: &Delivery, writer: &Writer, ledger: &Source, parcel: &Parcel, pace: Pace) {
    let Parcel { id, pending } = parcel;
    let start = Instant::now();
    let mut last: Option<String> = None;
    let mut rounds: u32 = 0;
    loop {
        rounds += 1;
        let Round::NotTaken(word) = deliver(d, writer, ledger, id, pending) else {
            return;
        };
        if last.as_deref() != Some(word.as_str()) {
            emit_err(&format!(
                "tz surface serve: {NOT_TAKEN}: {id}・席 {}・器 {word}",
                d.target
            ));
            last = Some(word);
        }
        if start.elapsed() + pace.step > pace.span {
            emit_err(&format!(
                "tz surface serve: {GAVE_UP}: {id}・席 {}・{rounds} 回",
                d.target
            ));
            return;
        }
        std::thread::sleep(pace.step);
    }
}

/// 起動の掃き（行 c-deliver-retry）。台帳を合流しない読みで上限 `READ_TIMEOUT` の 1 度だけ読み、印の無い裁定
/// （中核の `undelivered`・台帳の順）のうち、id の分の字（`id_minute`）が `now` から `pace.span` を引いた時刻の分の字より
/// 前でないものを 1 つずつ荷（id はその裁定の id・問いと裁定の組は 1 つ）にして、前の荷の `redeliver` が終えてから
/// 次の荷の `redeliver` を `pace` で撃つ（それより前の裁定と分の字の読めない裁定は、答えごとの撃ち直しも上限で止めていたので
/// 撃たず、席の停止の hook が拾う）。撃つ裁定が在れば撃つ前に `SWEEP_FOUND` の 1 行（裁定の id を , で並べた字と席）を書く。
/// 読みが落ちるか字が読めなければ `SWEEP_UNREAD` の 1 行（落ちた訳）を書いて何も撃たない。撃った荷の数を返す。
pub fn sweep(d: &Delivery, writer: &Writer, ledger: &Source, pace: Pace, now: EpochSecs) -> usize {
    let found = match ledger.text_within(READ_TIMEOUT).map(|t| undelivered(&t)) {
        Ok(Reading::Known(found)) => found,
        Ok(Reading::Unknown) => {
            emit_err(&format!(
                "tz surface serve: {SWEEP_UNREAD}: 台帳の字が読めない"
            ));
            return 0;
        }
        Err(word) => {
            emit_err(&format!("tz surface serve: {SWEEP_UNREAD}: {word}"));
            return 0;
        }
    };
    let since = minute(now.saturating_sub(pace.span.as_secs()));
    let found: Vec<Pending> = found
        .into_iter()
        .filter(|p| id_minute(&p.ruling).is_some_and(|m| m >= since.as_str()))
        .collect();
    if found.is_empty() {
        return 0;
    }
    let ids: Vec<&str> = found.iter().map(|p| p.ruling.as_str()).collect();
    emit_err(&format!(
        "tz surface serve: {SWEEP_FOUND}: {}・席 {}",
        ids.join(","),
        d.target
    ));
    for p in &found {
        let parcel = Parcel {
            id: p.ruling.clone(),
            pending: vec![p.clone()],
        };
        redeliver(d, writer, ledger, &parcel, pace);
    }
    found.len()
}

/// 裁定の id の分の字（`<頭>:<UTC の年月日 T 時分 Z>-<数>` の最後のコロンと最後の - の間の 14 字・形が違えば None）。
/// 字の順は時刻の順と同じ。
pub fn id_minute(id: &RulingId) -> Option<&str> {
    let (_, tail) = id.as_str().rsplit_once(':')?;
    tail_minute(tail)
}

/// 裁定の id の最後のコロンより後の字（`<UTC の年月日 T 時分 Z>-<数>`）の分の字（形が違えば None）。
fn tail_minute(tail: &str) -> Option<&str> {
    let (at, n) = tail.rsplit_once('-')?;
    let shaped = at.len() == 14
        && at.bytes().enumerate().all(|(i, c)| match i {
            8 => c == b'T',
            13 => c == b'Z',
            _ => c.is_ascii_digit(),
        })
        && !n.is_empty()
        && n.bytes().all(|c| c.is_ascii_digit());
    shaped.then_some(at)
}

/// 起動の掃きを別の thread で始める（待たない）。読むだけの server（`Config::read_only`）と、席の target か state dir の
/// 無い server では始めず false。配達の先と書きの持ち物は server の裁定の書きと同じ引数（--seat・--state-dir・--scribe2・
/// --repo・--bdw）から作り、台帳は --bd の合流しない読みで読む（行 c-deliver-retry）。
pub fn sweep_at_start(config: &Config) -> bool {
    if config.read_only {
        return false;
    }
    let (Some(target), Some(state_dir)) = (&config.seat, &config.state_dir) else {
        return false;
    };
    let d = Delivery {
        program: config.scribe2.clone(),
        state_dir: state_dir.clone(),
        target: target.clone(),
    };
    let writer = Writer {
        repo: config.repo.clone(),
        bdw: config.bdw.clone(),
        delivery: Some(d.clone()),
    };
    let ledger = Source::new(&config.repo, &config.bd);
    std::thread::spawn(move || {
        sweep(&d, &writer, &ledger, PACE, events::now());
    });
    true
}

/// 器の答えの口の引数の列（program の名は含めない・束の id が在れば末に `--batch <束>`）。
pub fn answer_argv(
    state_dir: &Path,
    repo: &Path,
    question: &BeadId,
    batch: Option<&RulingId>,
) -> Vec<OsString> {
    let mut argv: Vec<OsString> = vec![
        "seat".into(),
        "ruling".into(),
        "answer".into(),
        "--repo".into(),
        repo.into(),
        "--state-dir".into(),
        state_dir.into(),
        "--question".into(),
        question.as_str().into(),
    ];
    if let Some(batch) = batch {
        argv.push("--batch".into());
        argv.push(batch.as_str().into());
    }
    argv
}

/// 器の答えの口が落ちたときの log の字（`answer_failed_line` の頭）。
pub const ANSWER_FAILED: &str = "器の答えの口が落ちた";

/// 器の答えの口が落ちたときの log の 1 行（`tz surface serve: <ANSWER_FAILED>: 問い <問いの id>・器 <落ちた訳>`・逐語は書かない）。
pub fn answer_failed_line(question: &BeadId, failed: &Failed) -> String {
    format!(
        "tz surface serve: {ANSWER_FAILED}: 問い {question}・器 {}",
        failed.word()
    )
}

/// 器の答えの口の標準出力の裁定 id（前後の空白を除いた字が `<答えた問いの id>:<分の字>-<1 字以上の数字>` の形の時だけ Some）。
/// 形の照らしは `RulingId::new` に頼らず、ここで撃つ。
pub fn answered_id(question: &BeadId, stdout: &[u8]) -> Option<RulingId> {
    let text = std::str::from_utf8(stdout).ok()?.trim();
    let tail = text.strip_prefix(question.as_str())?.strip_prefix(':')?;
    tail_minute(tail)?;
    RulingId::new(text).ok()
}

/// 次の id（数は 1 から始め、notes に同じ id の定型行か器の結びの行が在れば 1 つずつ増やす）。
pub fn next_id(
    question: &BeadId,
    notes: &str,
    minute: &str,
) -> Result<RulingId, tsuzuri_contract::IdError> {
    let taken: Vec<&str> = notes
        .lines()
        .map(|l| l.trim_end_matches('\r'))
        .filter_map(|l| {
            l.strip_prefix(LINE_PREFIX)
                .map(|rest| rest.split(ID_END).next().unwrap_or(rest).trim())
                .or_else(|| bind_line(l, question.as_str()))
        })
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
    use super::{LINE_PREFIX, escape, minute, next_id, revoke_line};
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
        let notes = "前置き\n裁定 id = fx.1:20260927T1034Z-1・問い = fx.1・逐語 = x\n裁定 id = fx.1:20260927T1034Z-2・問い = fx.1・逐語 = y\r\n裁定 id = fx.1:20260927T1033Z-3・古い分";
        assert_eq!(
            next_id(&q, notes, m).expect("id").as_str(),
            "fx.1:20260927T1034Z-3"
        );
        let long = BeadId::new("a".repeat(60)).expect("id");
        assert!(next_id(&long, "", m).is_err(), "64 byte を越える id");
    }

    #[test]
    fn server_ask_line_prefix_is_graph_ruling() {
        assert_eq!(TYPED_LINES[0], (LINE_PREFIX, NodeKind::Ruling));
        let q = BeadId::new("fx.1").expect("id");
        let id = RulingId::new("fx.1:20260927T1034Z-2").expect("id");
        let revokes = RulingId::new("fx.1:20260927T1034Z-1").expect("id");
        let got = revoke_line(&id, &q, &revokes, "はい\nそれで");
        assert!(
            got.starts_with(
                "裁定 id = fx.1:20260927T1034Z-2・問い = fx.1・取り消す = fx.1:20260927T1034Z-1・逐語 = "
            ),
            "{got}"
        );
        assert!(got.ends_with("はい\\nそれで"), "{got}");
    }
}
