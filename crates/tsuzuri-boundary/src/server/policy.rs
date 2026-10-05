//! 方針の受付（口 POST /api/policy）。全体への指示を、方針 1 つにつき
//! 根の直下の閉じた問いの bead に残す（要件 FR8）。server 自身は file を書かない。器の配達の口は撃たず、
//! 開いている問いは閉じない。今までの方針の memo は探さず書かない。
//! 作る問いの形（本文の頭の 4 行と metadata の effect）は器の要件 FR81 の (a) に従い server が埋める。
//! 受付の順:
//! 1. 逐語が空白だけなら断る（EmptyVerbatim）。
//! 2. 台帳を合流しない読みで読み直す（裁定の受付の `reread` で撃ち直し、どれも読めなければ 503）。
//! 3. 範囲は字 `all` か open の問いの id（どちらでもなければ 400 scope）。
//! 4. 根の epic を探す（無ければ 503 no-root）。
//! 5. 根の下に方針の問いを作る（落ちるか作った id が読めなければ 502 ledger-create）。
//! 6. 方針の id を発行する（`<作った問いの id>:<分>-1`・収まらなければ 500）。
//! 7. 作った問いの notes の末尾に 1 行を足す（落ちたら 502）。
//! 8. 作った問いを閉じる（落ちたら 502）。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::natural_cmp;
use tsuzuri_contract::ledger::{
    BeadId, ChildType, Effect, LedgerItem, LedgerWrite, POLICY_SCOPE_LABEL, QUESTION_LABEL,
};
use tsuzuri_contract::surface::{PolicyRequest, PolicyResponse, Refusal, RulingId};
use tsuzuri_core::question::{
    ENG_PREFIX, PLAIN_PREFIX, REASON_PREFIX, RECOMMEND_PREFIX, open_questions,
};

use super::ledger::{Source, capture, parse_bd};
use super::ruling::{WRITE_TIMEOUT, Writer, escape, minute, policy_reason, reread};

/// 口の path。
pub const PATH: &str = "/api/policy";

/// 範囲が全体のときの字。
pub const SCOPE_ALL: &str = "all";

/// 方針の問いの題の頭。
pub const TITLE: &str = "方針";

/// notes の方針の定型行の頭（導出グラフが方針の節点を導く頭と同じ字）。
pub const LINE_PREFIX: &str = "方針 id = ";

/// 定型行の id の終わりの字。
const ID_END: char = '・';

/// 本文の技術の行の範囲が全体のときの字。
const SCOPE_ALL_TEXT: &str = "全体";

/// 本文の理由の行の字。
const REASON: &str = "持ち主の方針（方針の欄）";

/// 本文の推奨の行の字（方針は持ち主の指示で、席が推す答えを持たない）。
const RECOMMEND: &str = "なし";

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
    /// 根の epic が台帳に無い（503・書きの前）。
    NoRoot,
    /// 問いを作る書きが落ちたか、作った id が読めない（502・作れたかが分からない）。
    CreateFailed,
    /// 発行した id が記帳 id の形に収まらない（500・作った問いは open のまま）。
    IdShape(BeadId),
    /// notes への追記が落ちた（502・作った問いは open のまま）。
    AppendFailed(BeadId),
    /// 問いを閉じる書きが落ちた（502・notes には方針の行が在る）。
    CloseFailed(RulingId),
}

/// 方針を受ける（`now` は受付の時刻）。
pub fn accept(req: &PolicyRequest, ledger: &Source, writer: &Writer, now: EpochSecs) -> Outcome {
    if req.verbatim.trim().is_empty() {
        return Outcome::Refused(Refusal::EmptyVerbatim);
    }
    // 走っている読みを分け合わず、新しい子 process で読み直す（便 e-coalesce）。
    let Some(text) = reread(ledger) else {
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
    let Some(root) = root_epic(&items) else {
        return Outcome::NoRoot;
    };
    let create = create_write(root, &req.scope, &req.verbatim);
    let Some(question) = capture(&writer.bdw, create.argv(), &writer.repo, WRITE_TIMEOUT)
        .and_then(|out| created_id(&out))
    else {
        return Outcome::CreateFailed;
    };
    let Ok(id) = RulingId::for_question(&question, &minute(now), 1) else {
        return Outcome::IdShape(question);
    };
    let append = LedgerWrite::AppendNotes {
        id: question.clone(),
        line: line(&id, &req.scope, &req.verbatim),
    };
    if !write(writer, &append) {
        return Outcome::AppendFailed(question);
    }
    let close = LedgerWrite::CloseItem {
        id: question,
        reason: policy_reason(&id),
    };
    if !write(writer, &close) {
        return Outcome::CloseFailed(id);
    }
    Outcome::Recorded(PolicyResponse {
        policy: id,
        recorded_at: now,
    })
}

/// bdw を 1 回撃つ（rc 0 で上限の内に返せば true）。
fn write(writer: &Writer, w: &LedgerWrite) -> bool {
    capture(&writer.bdw, w.argv(), &writer.repo, WRITE_TIMEOUT).is_some()
}

/// 方針の問いを作る書き（根の下の task・label は問いと範囲の札・効き先は操作・本文は器の問いの形の 4 行）。
pub fn create_write(root: &BeadId, scope: &str, verbatim: &str) -> LedgerWrite {
    let plain = verbatim
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or_default();
    let scope_text = if scope == SCOPE_ALL {
        SCOPE_ALL_TEXT
    } else {
        scope
    };
    LedgerWrite::CreateChild {
        parent: root.clone(),
        title: title(scope),
        child_type: ChildType::Task,
        description: format!(
            "{PLAIN_PREFIX}{plain}\n{ENG_PREFIX}範囲 = {scope_text}\n{REASON_PREFIX}{REASON}\n{RECOMMEND_PREFIX}{RECOMMEND}"
        ),
        labels: vec![QUESTION_LABEL.to_string(), scope_label(scope)],
        effect: Some(Effect::Operation),
    }
}

/// 根の epic（種類 epic・親が無い・closed でない bead）。いくつも在れば id の自然な順で最初。
pub fn root_epic(items: &[LedgerItem]) -> Option<&BeadId> {
    items
        .iter()
        .filter(|i| i.row.kind == "epic" && i.row.parent.is_none() && i.row.status != "closed")
        .map(|i| &i.row.id)
        .min_by(|a, b| natural_cmp(a.as_str(), b.as_str()))
}

/// 子を作る書きの標準出力から作った id を読む（前後の空白を除いた字が bead の id の形なら）。
pub fn created_id(stdout: &[u8]) -> Option<BeadId> {
    let text = std::str::from_utf8(stdout).ok()?;
    BeadId::new(text.trim()).ok()
}

/// 範囲の札（`policy-scope:<範囲>`）。
pub fn scope_label(scope: &str) -> String {
    format!("{POLICY_SCOPE_LABEL}{scope}")
}

/// 方針の問いの題（`方針（範囲 = <範囲>）`）。
pub fn title(scope: &str) -> String {
    format!("{TITLE}（範囲 = {scope}）")
}

/// notes に足す 1 行（`方針 id = <id>・範囲 = <範囲>・逐語 = <字>`）。
pub fn line(id: &RulingId, scope: &str, verbatim: &str) -> String {
    format!(
        "{LINE_PREFIX}{id}{ID_END}範囲 = {scope}{ID_END}逐語 = {}",
        escape(verbatim)
    )
}
