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
//! 板は札のほかに形の崩れた open の bead の一覧を持ち、器の doctor の台帳の形の行を `form_ids` で写して題を台帳から引く
//! （`board_with_doctor`・tsuzuri は形を判じない・判断の記録 ADR-16 の決定 (6)・行 c-pipe-misfit）。
//! doctor の字を受けない `board` の一覧は Unknown。
//! 着地の後の CI の読み（行 c-pipe-ci）: `stage_of` の段が Landed の札だけ、走行の event の行を判定の関数 `ci_reading` に渡す
//! （器の終端の RunDone の detail の語を `ci_after` で順に重ねる・判定は `ci_reading` の 1 つに閉じる）。
//! 段と理由の決め（閉じた bead の札を含む）の後に `with_ci` で重ねる: 読みが無いか、台帳で閉じた bead の読みが Waiting なら
//! 段と理由のままで ci は None。閉じていない bead（台帳が読めないときと台帳に無い bead も）の読みが `CI_STALLS` に在れば
//! その段にし、段の理由は器の終端の detail の字のまま。ほかは段と理由のままで ci はその読み。Blocked と Queued の札の ci は None。
//! 札の since は段を決めた最後の event の ts の時刻で、今を引かない（板の電文は今の時刻に依らない・経過は面が今から引く）。

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{
    Ci, Misfit, MisfitBead, PipelineBoard, PipelineCard, Reading, Stage,
};
use tsuzuri_contract::graph::NodeKind;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::runs::{RunCost, RunLine, RunStep, RunsDoc};

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
    let Some(events) = read_events(events) else {
        return Board {
            board: PipelineBoard {
                cards: Reading::Unknown,
                misfits: Reading::Unknown,
            },
            unmapped: 0,
        };
    };
    let mut runs: BTreeMap<&str, RunState> = BTreeMap::new();
    let mut per_bead: BTreeMap<String, BeadRuns> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for event in &events {
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

    let mut cards = Vec::new();
    let mut unmapped = 0;
    for bead in &order {
        let entry = &per_bead[bead];
        let state = &runs[entry.latest.as_str()];
        let Ok(contract) = BeadId::new(bead.as_str()) else {
            continue;
        };
        let Some(last) = state.last else {
            continue;
        };
        let mapped = match stage_of(
            text(last, "kind").unwrap_or_default(),
            text(last, "stage"),
            text(last, "detail").unwrap_or_default(),
        ) {
            Some((Stage::Questioned, _)) if state.stopped => {
                Some((Stage::Questioned, Some(question_stopped(last))))
            }
            mapped => mapped,
        };
        let reading = match mapped {
            Some((Stage::Landed, _)) => ci_reading(&state.rows),
            _ => None,
        };
        let closed = beads
            .unwrap_or_default()
            .iter()
            .find(|b| &b.id == bead)
            .filter(|b| !b.is_open(now));
        let Some((stage, reason)) = (match (mapped, closed) {
            (Some((Stage::Landed, reason)), _) => Some((Stage::Landed, reason)),
            (_, Some(b)) => Some((Stage::Landed, Some(closed_reason(b)))),
            (mapped, None) => mapped,
        }) else {
            unmapped += 1;
            continue;
        };
        let (stage, reason, ci) = with_ci(stage, reason, reading, closed.is_some());
        cards.push(PipelineCard {
            contract,
            runs: entry.runs,
            stage,
            reason,
            account: state.account.clone(),
            since: text(last, "ts").and_then(epoch_secs),
            ci,
        });
    }

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
        let blocked = b.has_open_blocker(beads.unwrap_or_default(), now);
        cards.push(PipelineCard {
            contract,
            runs: 0,
            stage: if blocked {
                Stage::Blocked
            } else {
                Stage::Queued
            },
            reason: None,
            account: None,
            since: None,
            ci: None,
        });
    }

    Board {
        board: PipelineBoard {
            cards: Reading::Known(cards),
            misfits: Reading::Unknown,
        },
        unmapped,
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
                });
                lines.len() - 1
            });
            seen.insert(run, at);
        }
        let Some(&Some(at)) = seen.get(run) else {
            continue;
        };
        let line = &mut lines[at];
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
            let cost = &mut line.cost;
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
    }
    RunsDoc {
        bead: bead.clone(),
        runs: Reading::Known(lines),
    }
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
