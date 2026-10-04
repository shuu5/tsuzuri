//! pipeline の板（設計ノート surface-base 便 d・判断の記録 ADR-7 決定 (5)）。
//! 入力は台帳の一覧の字と器の event log の字と今の時刻で、file も子 process も時計も触らない。
//! 札は bead ごとに 1 枚で、その bead の走行のうち RunCreated がいちばん新しい 1 つの、最後の event で段を決める
//! （event log は追記の順なので、後の行ほど新しい）。段を決める event は `STAGE_EVENTS` の 4 種で、
//! ほかの event（RunCost・SeatSpawned など）は段を変えない。器の event から段への対応は `stage_of` の閉じた表。
//! 走行を 1 つも持たない open の契約は、acceptance に器の読める設計 pointer の行を持つ task の bead だけを Blocked か Queued の札にする
//! （読みは器の列の受付と同じ `dispatchable`・pointer の無い task は器が流さないので札にしない・行 c-pipe-queue）。
//! 器の RunStage の段 Failed と Stopped は段 Failed と Stopped の札にし、段の理由は detail の字にする（行 c-pipe-unmapped）。
//! detail が `RETIRED` の RunStage（器が worktree を畳んだ記帳）は段を決めない。
//! 台帳で閉じた bead の走行は、段が Landed でなければ（表に無い段も）段 Landed・段の理由 `CLOSED_TAG` と閉じた理由の頭の字の札にする
//! （閉じた（着地せず）・行 c-pipe-closed）。台帳が読めないときと台帳に無い bead は段を決めた最後の event の段のまま。
//! 問いの後に器が RunStopped で止めた走行は段 Questioned のまま、段の理由を `QUESTION_STOPPED` と about の字にする（行 c-pipe-questioned）。
//! bead の走行ごとの段の列・審査の結び・口座・費用は `runs_of` が同じ event log から読む（行 e-runs）。
//! 走行ごとの gate と審査の内訳は `with_verdicts` が便の dir の 2 つの file の字から置く（行 c-run-verdict）。
//! 板は札のほかに形の崩れた open の bead の一覧を持ち、器の doctor の台帳の形の行を `form_ids` で写して題を台帳から引く
//! （`board_with_doctor`・tsuzuri は形を判じない・判断の記録 ADR-16 の決定 (6)・行 c-pipe-misfit）。
//! doctor の字を受けない `board` の一覧は Unknown。
//! 着地の後の CI の読み（行 c-pipe-ci）: `stage_of` の段が Landed の札だけ、走行の event の行を判定の関数 `ci_reading` に渡す
//! （器の終端の RunDone の detail の語を `ci_after` で順に重ねる・判定は `ci_reading` の 1 つに閉じる）。
//! 段と理由の決め（閉じた bead の札を含む）の後に `with_ci` で重ねる: 読みが無いか、台帳で閉じた bead の読みが Waiting なら
//! 段と理由のままで ci は None。閉じていない bead（台帳が読めないときと台帳に無い bead も）の読みが `CI_STALLS` に在れば
//! その段にし、段の理由は器の終端の detail の字のまま。ほかは段と理由のままで ci はその読み。Blocked と Queued の札の ci は None。
//! 札の since は段を決めた最後の event の ts の時刻で、今を引かない（板の電文は今の時刻に依らない・経過は面が今から引く）。
//! 器の RunStage の段 Blocked（承認待ち・器の局面 run-blocked）は段 Blocked の札にし、段の理由は detail の字にする。
//! 走行の無い札の段は、器の局面の出力が読めてその契約の部品の局面が `QUEUED_PHASE` の時はその部品で決める
//! （`board_with_cases`・理由が `PARTNER_REASONS` なら Blocked・`HOLD` なら Held・ほかは Queued・since と理由は部品の字・行 c-case-columns）。
//! 出力が読めないか部品が無いか局面が違う札は、開いた blocker が在れば Blocked・無ければ Queued で、理由と since は None
//! （`queued_cards`）。
//! 局面の出力が読めれば、走行の在る札の段も契約の部品とその最新の便の部品の局面で決める（`phase_of`・行 c-ledger-lc）:
//! 契約の待ち（`QUEUED_PHASE` で理由が `SETTLED` でない・`REFUSED_PHASE`）は部品の理由と since、便の局面は `RUN_PHASES` の段で、
//! 理由と since は event log の読みが段を持てばその字・持たなければ便の部品の字。表に無い局面の語は「まだ分からない」
//! （札を作らず unmapped に数える・走行の無い札は台帳の blocks の割り）。契約の部品が無いか `CLOSED_PHASE` か便の部品が
//! 無ければ event log の段のまま。台帳で閉じた bead の札は今までどおり。CI の読みは段が Landed で event log も Landed の時だけ。
//! 段 Held（留め置き・列は Blocked）は契約の部品の局面が `QUEUED_PHASE` で理由が `HOLD`（席の止め）か `REFUSED_PHASE`（受付の断り）の札
//! （判断の記録 ADR-42 決定 (1)(2)・行 c-held-stage）。局面の出力が読めない間は Held を判じず台帳の blocks の割りのまま。

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{
    Ci, Misfit, MisfitBead, PipelineBoard, PipelineCard, Reading, Stage,
};
use tsuzuri_contract::case::{CaseDoc, CasePart};
use tsuzuri_contract::graph::NodeKind;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::runs::{
    GateFinding, GateVerdict, ReviewVerdict, RunCost, RunLine, RunStep, RunsDoc,
};

use crate::graph::build::{POINTER_PREFIX, read_events, run_bead};
use crate::ledger::{Bead, epoch_secs, read};

/// 段を決める event の種類。
pub const STAGE_EVENTS: [&str; 4] = ["RunCreated", "RunStage", "RunDone", "QuestionRaised"];

/// 落ちた審査の detail の頭（Reviewed と Gated で、この字で始まれば Failed）。
pub const FAILED_VERDICTS: [&str; 2] = ["verdict:FAIL", "verdict:INCONCLUSIVE"];

/// 通った審査の detail の頭（Reviewed でこの字で始まれば Running）。
pub const PASSED_VERDICT: &str = "verdict:PASS";

/// 口座の札の頭（detail の中）。
pub const ACCOUNT_TAG: &str = "account:";

/// 台帳で閉じた bead の札の段の理由の頭の字。
pub const CLOSED_TAG: &str = "closed:";

/// 段の理由に載せる閉じた理由の字数（char で数える）。
pub const CLOSED_CHARS: usize = 60;

/// 問いの後に器が止めた走行の札の段の理由の頭の字（見本の reasonShort の写し）。
pub const QUESTION_STOPPED: &str = "質問の後に止めた";

/// 器が worktree を畳んだことを残す RunStage の detail の字。器は段を動かさずに書くので、板の段を決めない。
pub const RETIRED: &str = "retired";

/// 器の doctor の台帳の形の行の頭（器の ledger/form.rs の PREFIX と同じ字）。
pub const FORM_PREFIX: &str = "ledger-form:";

/// 台帳の形の行から写す欄の語と崩れ（器の Report の欄 both と neither・この順）。
pub const FORM_FIELDS: [(&str, Misfit); 2] = [("both", Misfit::Both), ("neither", Misfit::Neither)];

/// 器の終端の RunDone の detail の頭（器の pipe/land/finish.rs の fn note の字）。
pub const TERMINAL_TAG: &str = "terminal:";

/// 終端の頭の後の push の語の頭（落ちの語でなければ CI を待つ）。
pub const PUSH_WORD: &str = "push:";

/// 終端の頭の後の CI の照合の語（字ちょうど）と読み。
pub const CI_WORDS: [(&str, Ci); 3] = [
    ("ci:success", Ci::Success),
    ("ci:failure", Ci::Failure),
    ("ci:unmeasurable", Ci::Unmeasurable),
];

/// 終端の頭の後の落ちの語の頭と読み（push の語より先に照らす）。
pub const FAULT_WORDS: [(&str, Ci); 3] = [
    ("push:failed:", Ci::PushFailed),
    ("close:failed:", Ci::CloseFailed),
    ("unreadable", Ci::Unreadable),
];

/// 閉じていない bead の札を止まった段へ移す読みと段（落ちたは Failed・分からないは Stopped）。
pub const CI_STALLS: [(Ci, Stage); 5] = [
    (Ci::Failure, Stage::Failed),
    (Ci::PushFailed, Stage::Failed),
    (Ci::CloseFailed, Stage::Failed),
    (Ci::Unmeasurable, Stage::Stopped),
    (Ci::Unreadable, Stage::Stopped),
];

/// 板と、表に無い段の走行の数（札を作らずに数える）。
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    pub board: PipelineBoard,
    pub unmapped: u32,
}

/// 局面の出力の契約の部品の種類の字。
pub const CONTRACT_PART: &str = "contract";

/// 局面の出力の契約の列の待ちの局面の語（器の case-lifecycle §2）。
pub const QUEUED_PHASE: &str = "contract-queued";

/// 列の待ちの理由のうち相手を待つ語（板の Blocked の列・ほかの語は Queued の列・判断の記録 ADR-27 の決定 (6)）。
pub const PARTNER_REASONS: [&str; 3] = ["dependency", "overlap", "reserved"];

/// 列の待ちの理由のうち席の止めの語（段 Held・判断の記録 ADR-42 決定 (1)・行 c-held-stage）。
pub const HOLD: &str = "hold";

/// 局面の出力の便の部品の種類の字。
pub const RUN_PART: &str = "run";

/// 契約の局面のうち段を最新の便の部品が持つ語（器の case-lifecycle §3・行 c-ledger-lc）。
pub const RUNNING_PHASE: &str = "contract-running";

/// 列の待ちの理由のうち手番を最新の便の部品が持つ語（器の case-lifecycle §3）。
pub const SETTLED: &str = "settled";

/// 契約の受付の断りの局面の語（段は Held・理由は断りの名・判断の記録 ADR-42 決定 (1)）。
pub const REFUSED_PHASE: &str = "contract-refused";

/// 閉じた契約の局面の語（札の段は event log と台帳の閉じのまま）。
pub const CLOSED_PHASE: &str = "contract-closed";

/// 便の局面の語から板の段（閉じた 14 語・器の case-lifecycle §2.1 の便の段の写しの語・行 c-ledger-lc）。
pub const RUN_PHASES: [(&str, Stage); 14] = [
    ("run-intake", Stage::Running),
    ("run-reviewed", Stage::Running),
    ("run-review-failed", Stage::Failed),
    ("run-blocked", Stage::Blocked),
    ("run-implementing", Stage::Running),
    ("run-asking", Stage::Questioned),
    ("run-rate-limited", Stage::Running),
    ("run-gating", Stage::Running),
    ("run-landing", Stage::Gated),
    ("run-gate-failed", Stage::Failed),
    ("run-ci-waiting", Stage::Landed),
    ("run-landed-open", Stage::Landed),
    ("run-stopped", Stage::Stopped),
    ("run-failed", Stage::Failed),
];

/// 局面の出力の部品で決めた札の段（行 c-ledger-lc）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phased<'p> {
    /// 契約の待ち（段と契約の部品）。
    Waiting(Stage, &'p CasePart),
    /// 便の局面（段と便の部品）。
    Run(Stage, &'p CasePart),
    /// 表に無い局面の語（まだ分からない）。
    Unknown,
}

/// 器の event から板の段と段の理由（閉じた表・None は表に無い段）。
/// kind は event の種類、stage は RunStage の段の名、detail は event の detail の字。
pub fn stage_of(kind: &str, stage: Option<&str>, detail: &str) -> Option<(Stage, Option<String>)> {
    let failed = FAILED_VERDICTS.iter().any(|v| detail.starts_with(v));
    let to = match (kind, stage) {
        ("RunDone", _) => Stage::Landed,
        ("QuestionRaised", _) | ("RunStage", Some("Questioned")) => Stage::Questioned,
        ("RunCreated", _) => Stage::Running,
        ("RunStage", Some("Reviewed" | "Gated")) if failed => {
            return Some((Stage::Failed, Some(detail.to_string())));
        }
        ("RunStage", Some("Gated")) => Stage::Gated,
        ("RunStage", Some("Reviewed")) if detail.starts_with(PASSED_VERDICT) => Stage::Running,
        ("RunStage", Some("Spawned" | "Implemented")) => Stage::Running,
        ("RunStage", Some("Blocked")) => {
            return Some((
                Stage::Blocked,
                (!detail.is_empty()).then(|| detail.to_string()),
            ));
        }
        ("RunStage", Some(s @ ("Failed" | "Stopped"))) => {
            let to = if s == "Failed" {
                Stage::Failed
            } else {
                Stage::Stopped
            };
            return Some((to, (!detail.is_empty()).then(|| detail.to_string())));
        }
        _ => return None,
    };
    Some((to, None))
}

/// detail の中の口座の札のうち最後のもの。
fn detail_account(event: &Value) -> Option<String> {
    event
        .get("detail")
        .and_then(Value::as_str)?
        .split([',', ' '])
        .filter_map(|tok| tok.strip_prefix(ACCOUNT_TAG))
        .rfind(|a| !a.is_empty())
        .map(str::to_string)
}

fn text<'a>(event: &'a Value, key: &str) -> Option<&'a str> {
    event.get(key).and_then(Value::as_str)
}

/// 1 つの detail の後の CI の読み（`ci_reading` だけが呼ぶ）。
/// detail が `TERMINAL_TAG` で始まらなければ前の読み。頭の後の語が `FAULT_WORDS` の語で始まればその読み、
/// ほかの `PUSH_WORD` で始まれば Waiting、`CI_WORDS` の語ちょうどならその読み、ほかは前の読み。
pub fn ci_after(prev: Option<Ci>, detail: &str) -> Option<Ci> {
    let Some(word) = detail.strip_prefix(TERMINAL_TAG) else {
        return prev;
    };
    if let Some((_, ci)) = FAULT_WORDS.iter().find(|(w, _)| word.starts_with(w)) {
        return Some(*ci);
    }
    if word.starts_with(PUSH_WORD) {
        return Some(Ci::Waiting);
    }
    CI_WORDS
        .iter()
        .find(|(w, _)| word == *w)
        .map_or(prev, |(_, ci)| Some(*ci))
}

/// 走行の event の行（ログの順）から着地の後の CI の読みと器の語（判定はこの関数 1 つに閉じる）。
/// 種類 RunDone の行の detail を順に重ね、器の語は最後の `TERMINAL_TAG` で始まる RunDone の detail の字。
/// 終端の行が無ければ None。
pub fn ci_reading(rows: &[&Value]) -> Option<(Ci, String)> {
    let mut ci = None;
    let mut word = None;
    for row in rows.iter().filter(|r| text(r, "kind") == Some("RunDone")) {
        let detail = text(row, "detail").unwrap_or_default();
        ci = ci_after(ci, detail);
        if detail.starts_with(TERMINAL_TAG) {
            word = Some(detail);
        }
    }
    Some((ci?, word?.to_string()))
}

/// 段と理由に CI の読みを重ねる（module の頭の決まり・`closed` は台帳で閉じた bead か）。
fn with_ci(
    stage: Stage,
    reason: Option<String>,
    reading: Option<(Ci, String)>,
    closed: bool,
) -> (Stage, Option<String>, Option<Ci>) {
    match reading {
        None => (stage, reason, None),
        Some((Ci::Waiting, _)) if closed => (stage, reason, None),
        Some((ci, word)) if !closed => match CI_STALLS.iter().find(|(c, _)| *c == ci) {
            Some((_, to)) => (*to, Some(word), Some(ci)),
            None => (stage, reason, Some(ci)),
        },
        Some((ci, _)) => (stage, reason, Some(ci)),
    }
}

/// 1 つの走行の読み（段を決めた最後の event と、口座の札の最後のものと、その event の後に RunStopped が来たか・走行の event の行の列）。
#[derive(Default)]
struct RunState<'a> {
    last: Option<&'a Value>,
    account: Option<String>,
    stopped: bool,
    rows: Vec<&'a Value>,
}

/// 問いの後に止めた走行の段の理由（段 Questioned の event の detail の about: の後の字を全角の括弧で添える）。
fn question_stopped(last: &Value) -> String {
    match text(last, "detail")
        .and_then(|d| d.strip_prefix("about:"))
        .map(str::trim)
        .filter(|a| !a.is_empty())
    {
        Some(about) => format!("{QUESTION_STOPPED}（{about}）"),
        None => QUESTION_STOPPED.to_string(),
    }
}

/// bead ごとの走行（RunCreated の数と、いちばん新しい走行）。
struct BeadRuns {
    runs: u32,
    latest: String,
}

/// 閉じた bead の札の段の理由（`CLOSED_TAG` の後に、閉じた理由の空白の続きを 1 つに畳み前後を除いた字の頭の `CLOSED_CHARS` 字）。
fn closed_reason(bead: &Bead) -> String {
    let words = bead
        .close_reason
        .as_deref()
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let head: String = words.chars().take(CLOSED_CHARS).collect();
    format!("{CLOSED_TAG}{head}")
}

/// 器の列の受付が便にできる設計 pointer を acceptance に持つか（器の受付と同じ読み）。
/// 前後の空白を除いた行のうち `POINTER_PREFIX` で始まる最初の行の残りが、井桁でちょうど 2 つに分かれ、
/// path の拡張子が toml か md で、行 id が空でなく空白を含まないときだけ真（最初の行が形に合わなければ後の行は見ない）。
fn dispatchable(acceptance: &str) -> bool {
    let Some(pointer) = acceptance
        .lines()
        .find_map(|l| l.trim().strip_prefix(POINTER_PREFIX))
    else {
        return false;
    };
    let parts: Vec<&str> = pointer.trim().split('#').collect();
    let [path, row] = parts.as_slice() else {
        return false;
    };
    let ext = std::path::Path::new(path).extension();
    matches!(ext.and_then(|e| e.to_str()), Some("toml" | "md"))
        && !row.is_empty()
        && !row.chars().any(char::is_whitespace)
}

/// 器の doctor の出力から台帳の形の行の崩れた bead の id と崩れ（`FORM_FIELDS` の欄の順・欄の中は行の順）。
/// 頭が `FORM_PREFIX` の行（前の空白は除かない）がちょうど 1 本で、その頭の後を空白で区切った欄のうち
/// `FORM_FIELDS` の語ごとに語と等号で始まる欄がちょうど 1 つ在り、等号の後の字が件数だけか件数とコロンと
/// コンマで区切った id の列で、件数が id の数と同じで、どの id も空でなく、同じ id が 2 度出ないときだけ Some。
/// ほかの欄は読まない（測れていない行は欄 both と neither を持たないので None）。
pub fn form_ids(doctor: &str) -> Option<Vec<(String, Misfit)>> {
    let mut lines = doctor.lines().filter_map(|l| l.strip_prefix(FORM_PREFIX));
    let (Some(line), None) = (lines.next(), lines.next()) else {
        return None;
    };
    let fields: Vec<&str> = line.split_whitespace().collect();
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for (word, misfit) in FORM_FIELDS {
        let key = format!("{word}=");
        let mut values = fields.iter().filter_map(|f| f.strip_prefix(key.as_str()));
        let (Some(value), None) = (values.next(), values.next()) else {
            return None;
        };
        let (count, ids) = match value.split_once(':') {
            Some((count, ids)) => (count, ids.split(',').collect()),
            None => (value, Vec::new()),
        };
        if count.parse::<usize>().ok()? != ids.len() {
            return None;
        }
        for id in ids {
            if id.is_empty() || !seen.insert(id) {
                return None;
            }
            out.push((id.to_string(), misfit));
        }
    }
    Some(out)
}

/// 形の崩れた open の bead の一覧（台帳の順・台帳に無い id と閉じた bead は出さない）。
/// 台帳か doctor の字が無いか、doctor の字の台帳の形の行が読めなければ Unknown。
fn misfits_of(
    beads: Option<&[Bead]>,
    doctor: Option<&str>,
    now: EpochSecs,
) -> Reading<Vec<MisfitBead>> {
    let (Some(beads), Some(ids)) = (beads, doctor.and_then(form_ids)) else {
        return Reading::Unknown;
    };
    let ids: BTreeMap<&str, Misfit> = ids.iter().map(|(id, m)| (id.as_str(), *m)).collect();
    Reading::Known(
        beads
            .iter()
            .filter(|b| b.is_open(now))
            .filter_map(|b| {
                let misfit = *ids.get(b.id.as_str())?;
                Some(MisfitBead {
                    bead: BeadId::new(b.id.as_str()).ok()?,
                    title: b.title.clone(),
                    misfit,
                })
            })
            .collect(),
    )
}

/// 台帳の一覧と event log の字と今の時刻から板を組む。
/// event log が読めなければ札は「まだ分からない」。台帳が読めなければ走行の無い契約の札を作らない。
/// 形の崩れの一覧は「まだ分からない」（doctor の字を受けるのは `board_with_doctor`）。
pub fn board(ledger: &str, events: &str, now: EpochSecs) -> Board {
    of_inputs(read(ledger).as_deref(), events, now)
}

/// `board` と同じ板に、器の doctor の字の台帳の形の行から写した形の崩れの一覧を置く（行 c-pipe-misfit）。
/// doctor の字が無いか台帳の形の行が読めないか台帳が読めなければ一覧は「まだ分からない」。
pub fn board_with_doctor(
    ledger: &str,
    events: &str,
    doctor: Option<&str>,
    now: EpochSecs,
) -> Board {
    let beads = read(ledger);
    let mut b = of_inputs(beads.as_deref(), events, now);
    b.board.misfits = misfits_of(beads.as_deref(), doctor, now);
    b
}

/// 読めた bead（None は台帳が読めない）と event log の字から板を組む。
pub(crate) fn of_inputs(beads: Option<&[Bead]>, events: &str, now: EpochSecs) -> Board {
    of_parsed(beads, read_events(events).as_deref(), now)
}

/// `board` と同じ板の札の段と理由と since を、局面の出力 `cases` が読めればその部品で決める（`phase_of`・
/// 部品が無いか局面が違う走行の無い札は台帳の blocks の割りのまま・行 c-case-columns・行 c-ledger-lc）。
pub fn board_with_cases(ledger: &str, events: &str, cases: &CaseDoc, now: EpochSecs) -> Board {
    let parts = match &cases.parts {
        Reading::Known(parts) => Some(parts.as_slice()),
        Reading::Unknown => None,
    };
    let beads = read(ledger);
    of_cases(beads.as_deref(), read_events(events).as_deref(), parts, now)
}

/// 読んだ bead（None は台帳が読めない）と読んだ event log の値（None は event log が読めない）から板を組む。
pub(crate) fn of_parsed(beads: Option<&[Bead]>, events: Option<&[Value]>, now: EpochSecs) -> Board {
    of_cases(beads, events, None, now)
}

/// `of_parsed` の板を、局面の出力の部品（None は読めない）も受けて組む。
fn of_cases(
    beads: Option<&[Bead]>,
    events: Option<&[Value]>,
    parts: Option<&[CasePart]>,
    now: EpochSecs,
) -> Board {
    let Some(events) = events else {
        return Board {
            board: PipelineBoard {
                cards: Reading::Unknown,
                misfits: Reading::Unknown,
            },
            unmapped: 0,
        };
    };
    let Tracked { runs, per_bead, order } = track(events);

    let mut cards = Vec::new();
    let mut unmapped = 0;
    for bead in &order {
        let Some((count, state, contract, last)) = latest(&runs, &per_bead, bead) else {
            continue;
        };
        let mapped = mapped_stage(last, state.stopped);
        let at = text(last, "ts").and_then(epoch_secs);
        let decided = decide(parts.and_then(|p| phase_of(p, bead)), mapped.clone(), at);
        let closed = closed_bead(beads, bead, now);
        let Some((stage, reason, since)) = (match (decided, closed) {
            (Some((Stage::Landed, reason, since)), _) => Some((Stage::Landed, reason, since)),
            (_, Some(b)) => Some((Stage::Landed, Some(closed_reason(b)), at)),
            (decided, None) => decided,
        }) else {
            unmapped += 1;
            continue;
        };
        let reading = match (stage, mapped) {
            (Stage::Landed, Some((Stage::Landed, _))) => ci_reading(&state.rows),
            _ => None,
        };
        let (stage, reason, ci) = with_ci(stage, reason, reading, closed.is_some());
        cards.push(PipelineCard {
            contract,
            runs: count,
            stage,
            reason,
            account: state.account.clone(),
            since,
            ci,
        });
    }

    queued_cards(beads, &per_bead, parts, now, &mut cards);

    Board {
        board: PipelineBoard {
            cards: Reading::Known(cards),
            misfits: Reading::Unknown,
        },
        unmapped,
    }
}

/// event log の走行の読み（走行ごとの読みと、bead ごとの走行と、走行の在る bead の出た順）。
struct Tracked<'a> {
    runs: BTreeMap<&'a str, RunState<'a>>,
    per_bead: BTreeMap<String, BeadRuns>,
    order: Vec<String>,
}

/// event log の値を順に読んで走行を組む（走行は欄 run を持つ RunCreated の最初の 1 件で作る）。
fn track(events: &[Value]) -> Tracked<'_> {
    let mut runs: BTreeMap<&str, RunState> = BTreeMap::new();
    let mut per_bead: BTreeMap<String, BeadRuns> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for event in events {
        let kind = text(event, "kind").unwrap_or_default();
        let Some(run) = text(event, "run") else {
            continue;
        };
        if kind == "RunCreated" && !runs.contains_key(run) {
            let Some(bead) = text(event, "bead").or_else(|| run_bead(run)) else {
                continue;
            };
            runs.insert(run, RunState::default());
            let entry = per_bead.entry(bead.to_string()).or_insert_with(|| {
                order.push(bead.to_string());
                BeadRuns {
                    runs: 0,
                    latest: String::new(),
                }
            });
            entry.runs += 1;
            entry.latest = run.to_string();
        }
        // RunCreated の無い走行の event は読み捨てる。
        let Some(state) = runs.get_mut(run) else {
            continue;
        };
        state.rows.push(event);
        if let Some(account) = detail_account(event) {
            state.account = Some(account);
        }
        let retired = kind == "RunStage" && text(event, "detail") == Some(RETIRED);
        if STAGE_EVENTS.contains(&kind) && !retired {
            state.last = Some(event);
            state.stopped = false;
        } else if kind == "RunStopped" {
            state.stopped = true;
        }
    }
    Tracked {
        runs,
        per_bead,
        order,
    }
}

/// bead の走行の回数と最新の走行の読みと契約の id と段を決めた最後の event（どれかが無ければ None）。
fn latest<'b, 'a>(
    runs: &'b BTreeMap<&'a str, RunState<'a>>,
    per_bead: &BTreeMap<String, BeadRuns>,
    bead: &str,
) -> Option<(u32, &'b RunState<'a>, BeadId, &'a Value)> {
    let entry = per_bead.get(bead)?;
    let state = runs.get(entry.latest.as_str())?;
    let contract = BeadId::new(bead).ok()?;
    Some((entry.runs, state, contract, state.last?))
}

/// 走行の在る札の段と理由と since（行 c-ledger-lc）。`phased` が None なら event log の段 `mapped` と時刻 `at`、
/// 契約の待ちは部品の理由と since、便の局面は event log が段を持てばその理由と `at`・持たなければ便の部品の字、表に無い語は None。
fn decide(
    phased: Option<Phased>,
    mapped: Option<(Stage, Option<String>)>,
    at: Option<EpochSecs>,
) -> Option<(Stage, Option<String>, Option<EpochSecs>)> {
    match (phased, mapped) {
        (None, mapped) => mapped.map(|(stage, reason)| (stage, reason, at)),
        (Some(Phased::Unknown), _) => None,
        (Some(Phased::Waiting(stage, part) | Phased::Run(stage, part)), None)
        | (Some(Phased::Waiting(stage, part)), Some(_)) => {
            Some((stage, part.reason.clone(), part.since))
        }
        (Some(Phased::Run(stage, _)), Some((_, reason))) => Some((stage, reason, at)),
    }
}

/// 走行の段を決めた最後の event の段と理由（問いの後に止めた走行は止めた理由）。
fn mapped_stage(last: &Value, stopped: bool) -> Option<(Stage, Option<String>)> {
    match stage_of(
        text(last, "kind").unwrap_or_default(),
        text(last, "stage"),
        text(last, "detail").unwrap_or_default(),
    ) {
        Some((Stage::Questioned, _)) if stopped => {
            Some((Stage::Questioned, Some(question_stopped(last))))
        }
        mapped => mapped,
    }
}

/// 台帳の bead `id` が今閉じていればその bead（台帳が読めなければ None）。
fn closed_bead<'b>(beads: Option<&'b [Bead]>, id: &str, now: EpochSecs) -> Option<&'b Bead> {
    beads
        .unwrap_or_default()
        .iter()
        .find(|b| b.id == id)
        .filter(|b| !b.is_open(now))
}

/// 走行の無い開いた task のうち配れる契約の札を足す。段と理由と since は、局面の出力の部品で決まれば部品から
/// （`phase_of`・行 c-case-columns・行 c-ledger-lc）、決まらないか表に無い語なら開いた blocker が在れば Blocked・
/// 無ければ Queued で理由と since は None。
fn queued_cards(
    beads: Option<&[Bead]>,
    per_bead: &BTreeMap<String, BeadRuns>,
    parts: Option<&[CasePart]>,
    now: EpochSecs,
    cards: &mut Vec<PipelineCard>,
) {
    for b in beads.unwrap_or_default() {
        if b.kind != NodeKind::Task
            || !b.is_open(now)
            || per_bead.contains_key(&b.id)
            || !dispatchable(&b.acceptance)
        {
            continue;
        }
        let Ok(contract) = BeadId::new(b.id.as_str()) else {
            continue;
        };
        let (stage, reason, since) = match parts.and_then(|p| phase_of(p, &b.id)) {
            Some(Phased::Waiting(stage, part) | Phased::Run(stage, part)) => {
                (stage, part.reason.clone(), part.since)
            }
            _ if b.has_open_blocker(beads.unwrap_or_default(), now) => (Stage::Blocked, None, None),
            _ => (Stage::Queued, None, None),
        };
        cards.push(PipelineCard {
            contract,
            runs: 0,
            stage,
            reason,
            account: None,
            since,
            ci: None,
        });
    }
}

/// 契約 `id` の部品とその最新の便の部品で決めた段（行 c-ledger-lc）。契約の部品が無いか局面が `CLOSED_PHASE` か、
/// 段を便の部品が持つ局面（`RUNNING_PHASE` か理由が `SETTLED` の `QUEUED_PHASE`）で結び runs の便の部品が無ければ None。
pub fn phase_of<'p>(parts: &'p [CasePart], id: &str) -> Option<Phased<'p>> {
    let find = |kind: &str, id: &str| parts.iter().find(|p| p.part == kind && p.id == id);
    let contract = find(CONTRACT_PART, id)?;
    let settled = contract.reason.as_deref() == Some(SETTLED);
    match contract.phase.as_str() {
        CLOSED_PHASE => None,
        QUEUED_PHASE if !settled => Some(Phased::Waiting(
            queued_of(contract.reason.as_deref()),
            contract,
        )),
        REFUSED_PHASE => Some(Phased::Waiting(Stage::Held, contract)),
        RUNNING_PHASE | QUEUED_PHASE => {
            let run = contract.links.runs.iter().find_map(|r| find(RUN_PART, r))?;
            Some(
                RUN_PHASES
                    .iter()
                    .find(|(word, _)| *word == run.phase)
                    .map_or(Phased::Unknown, |&(_, stage)| Phased::Run(stage, run)),
            )
        }
        _ => Some(Phased::Unknown),
    }
}

/// 列の待ちの理由の語の段（`PARTNER_REASONS` の語なら Blocked・`HOLD` なら Held・ほかの語と理由の無い部品は Queued）。
pub fn queued_of(reason: Option<&str>) -> Stage {
    match reason {
        Some(word) if PARTNER_REASONS.contains(&word) => Stage::Blocked,
        Some(HOLD) => Stage::Held,
        _ => Stage::Queued,
    }
}

/// detail を `,` と空白で切った札のうち `tag` で始まる最初の札の残り（空なら None）。
fn detail_tag(detail: &str, tag: &str) -> Option<String> {
    detail
        .split([',', ' '])
        .find_map(|tok| tok.strip_prefix(tag))
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

/// event の欄の整数（無いか u64 に読めなければ 0）。
fn count(event: &Value, key: &str) -> u64 {
    event.get(key).and_then(Value::as_u64).unwrap_or(0)
}

/// bead の走行の時間軸（行 e-runs・要件 FR10）。走行は `of_inputs` と同じ決まりで作る:
/// 欄 run を持つ event のうち kind が RunCreated の最初の 1 件で作り、作る前の event と作らない走行の event は読み捨てる。
/// 作った走行の event ごとに、口座の札（後ほど勝つ）・段（欄 stage を持つ event 1 つに 1 つ）・費用（RunCost の和）を読む。
/// event log が読めなければ runs は「まだ分からない」。
pub fn runs_of(events: &str, bead: &BeadId) -> RunsDoc {
    let Some(events) = read_events(events) else {
        return RunsDoc {
            bead: bead.clone(),
            runs: Reading::Unknown,
        };
    };
    // run の id から列の位置（None はほかの bead の走行）。
    let mut seen: BTreeMap<&str, Option<usize>> = BTreeMap::new();
    let mut lines: Vec<RunLine> = Vec::new();
    for event in &events {
        let kind = text(event, "kind").unwrap_or_default();
        let Some(run) = text(event, "run") else {
            continue;
        };
        if kind == "RunCreated" && !seen.contains_key(run) {
            let Some(of) = text(event, "bead").or_else(|| run_bead(run)) else {
                continue;
            };
            let at = (of == bead.as_str()).then(|| {
                lines.push(RunLine {
                    run: run.to_string(),
                    started_at: text(event, "ts").and_then(epoch_secs),
                    account: None,
                    steps: Vec::new(),
                    cost: RunCost::default(),
                    gate: Reading::Unknown,
                    review: Reading::Unknown,
                });
                lines.len() - 1
            });
            seen.insert(run, at);
        }
        let Some(&Some(at)) = seen.get(run) else {
            continue;
        };
        let Some(line) = lines.get_mut(at) else {
            continue;
        };
        if let Some(account) = detail_account(event) {
            line.account = Some(account);
        }
        if let Some(stage) = text(event, "stage") {
            let detail = text(event, "detail");
            line.steps.push(RunStep {
                at: text(event, "ts").and_then(epoch_secs),
                stage: stage.to_string(),
                detail: detail.map(str::to_string),
                verdict: detail.and_then(|d| detail_tag(d, "verdict:")),
                verdict_kind: detail.and_then(|d| detail_tag(d, "kind:")),
            });
        }
        if kind == "RunCost" {
            add_cost(&mut line.cost, event);
        }
    }
    RunsDoc {
        bead: bead.clone(),
        runs: Reading::Known(lines),
    }
}

/// RunCost の event 1 件の費用を足す（usage の字は鍵と字 : と数を字 , で並べた札の列・知らない鍵と数でない札は読み捨てる）。
fn add_cost(cost: &mut RunCost, event: &Value) {
    cost.events += 1;
    cost.turns += count(event, "turns");
    cost.wall_ms += count(event, "wall_ms");
    for tok in text(event, "usage").unwrap_or_default().split(',') {
        let Some((key, value)) = tok.split_once(':') else {
            continue;
        };
        let Ok(n) = value.parse::<u64>() else {
            continue;
        };
        match key {
            "in" => cost.tokens_in += n,
            "out" => cost.tokens_out += n,
            "cache_read" => cost.cache_read += n,
            "cache_create" => cost.cache_create += n,
            _ => {}
        }
    }
}

/// 便の dir の判定の file の schema（器の verdict.json と review.json の欄 schema の数）。
pub const VERDICT_SCHEMA: u64 = 1;

/// 走行の列の各走行に、便の dir の gate の判定と審査の判定を置く（行 c-run-verdict・要件 FR10）。
/// `gate` と `review` は走行の id から器の verdict.json と review.json の字を返す（無いか読めなければ None）。
/// 字の無い走行の判定は Unknown。runs が Unknown の doc はそのまま返す。
pub fn with_verdicts(
    mut doc: RunsDoc,
    gate: impl Fn(&str) -> Option<String>,
    review: impl Fn(&str) -> Option<String>,
) -> RunsDoc {
    if let Reading::Known(lines) = &mut doc.runs {
        for line in lines {
            line.gate = gate(&line.run).map_or(Reading::Unknown, |body| gate_of(&body));
            line.review = review(&line.run).map_or(Reading::Unknown, |body| review_of(&body));
        }
    }
    doc
}

/// 器の verdict.json の字の gate の判定（`judged` が読めたときだけ Known・時刻は欄 ts で、読めなければ None）。
/// 欄 findings が在れば、字 , で区切った札がどれも空でない観点の語と字 : と数のときだけ Known（欄が無ければ findings は None）。
pub fn gate_of(body: &str) -> Reading<GateVerdict> {
    let Some((v, verdict, evidence)) = judged(body) else {
        return Reading::Unknown;
    };
    let findings = match v.get("findings") {
        None => None,
        Some(f) => match f.as_str().and_then(findings_of) {
            Some(list) => Some(list),
            None => return Reading::Unknown,
        },
    };
    Reading::Known(GateVerdict {
        verdict,
        evidence,
        findings,
        at: text(&v, "ts").and_then(epoch_secs),
    })
}

/// 器の review.json の字の審査の判定（`judged` が読めたときだけ Known）。欄 kind と at は字のときだけ写し、時刻は欄 ts。
pub fn review_of(body: &str) -> Reading<ReviewVerdict> {
    let Some((v, verdict, evidence)) = judged(body) else {
        return Reading::Unknown;
    };
    Reading::Known(ReviewVerdict {
        verdict,
        evidence,
        kind: text(&v, "kind").map(str::to_string),
        place: text(&v, "at").map(str::to_string),
        at: text(&v, "ts").and_then(epoch_secs),
    })
}

/// 判定の file の字の共通の読み: JSON の object で、欄 schema が数 `VERDICT_SCHEMA` で、欄 verdict と evidence が字のときだけ
/// （値と verdict と evidence）。
fn judged(body: &str) -> Option<(Value, String, String)> {
    let v: Value = serde_json::from_str(body).ok()?;
    if v.get("schema").and_then(Value::as_u64) != Some(VERDICT_SCHEMA) {
        return None;
    }
    let verdict = text(&v, "verdict")?.to_string();
    let evidence = text(&v, "evidence")?.to_string();
    Some((v, verdict, evidence))
}

/// findings の字（観点の語と字 : と件数の札を字 , で並べた列・器の字の順のまま）。崩れた札が 1 つでも在れば None。
fn findings_of(s: &str) -> Option<Vec<GateFinding>> {
    s.split(',')
        .map(|tok| {
            let (category, count) = tok.split_once(':')?;
            if category.is_empty() {
                return None;
            }
            Some(GateFinding {
                category: category.to_string(),
                count: count.parse().ok()?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{detail_account, stage_of};
    use serde_json::json;
    use tsuzuri_contract::board::Stage;

    #[test]
    fn stats_stage_of_closed_table() {
        let s = |kind, stage, detail| stage_of(kind, stage, detail).map(|(s, _)| s);
        assert_eq!(s("RunDone", Some("Landed"), ""), Some(Stage::Landed));
        assert_eq!(
            s("RunStage", Some("Reviewed"), "verdict:PASS"),
            Some(Stage::Running)
        );
        assert_eq!(s("RunStage", Some("Reviewed"), ""), None);
        assert_eq!(
            s("RunStage", Some("Gated"), "turn:taken"),
            Some(Stage::Gated)
        );
        assert_eq!(s("RunStage", Some("Merging"), ""), None);
        assert_eq!(s("RunCost", None, ""), None);
        assert_eq!(
            stage_of("RunStage", Some("Gated"), "verdict:FAIL"),
            Some((Stage::Failed, Some("verdict:FAIL".to_string())))
        );
    }

    #[test]
    fn stats_detail_account_takes_the_last() {
        assert_eq!(
            detail_account(&json!({"detail": "account:a,base:x account:b"})),
            Some("b".to_string())
        );
        assert_eq!(detail_account(&json!({"account": "a"})), None);
        assert_eq!(detail_account(&json!({"detail": "account:"})), None);
    }
}
