//! 次の一手（設計ノート surface-base 便 d・判断の記録 ADR-7 決定 (6)・規則の行 R-24）。
//! 閉じた 7 種を宣言の順（優先の順）に判じ、7 種それぞれの結果（当たった・当たらない・判じなかった）と
//! 件数と対象の id を返し、最初に当たった 1 つを大きく出す。入力は台帳の一覧の字と event log の字と今の時刻。
//! この便の入力で判じるのは 3 種（止まっている走行・質問・なし）。限度と移動・応答なし・発効待ちは
//! 材料（席と口座の状態・発効待ちの記録）が入力に無いので判じない。
//! 便 e-seat は席の card も受ける関数（`next_step_seat`）を足す。限度と移動と応答なしを席の card から判じ、
//! なしと大きく出す 1 つは同じ決め方で決め直す。席の card を受けない `next_step` の値は変えない。
//! 行 c-next-batch は束の承認を台帳の字から判じる。束の受付と束の block と同じ一覧（`open_questions`）の
//! A-1 の印の無い問いが `BATCH_MIN` 本以上なら当たる（要件 FR7: A-1 の問いは束に入れない）。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{NextMove, Reading, Stage};
use tsuzuri_contract::graph::NodeKind;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::seat::{SeatCard, SeatState};
use tsuzuri_contract::stats::{CheckResult, NextCheck, NextStep};

use crate::ledger::{Bead, read};
use crate::pipeline;
use crate::question::open_questions;

/// 止まっている走行の段（行 c-next-stall）。Questioned は質問の側で数えない
/// （見本の stoppedRuns と語の辞書の nx_c と同じ）。
pub const STALLED_STAGES: [Stage; 2] = [Stage::Failed, Stage::Stopped];

/// この版の入力に材料が無く判じない種類（行 c-next-stall）。なしの判じから外す
/// （この種類が判じなかったでも、なしを判じなかったにしない）。
/// 後の行 c-next-effect が発効待ちを判じるときに、ここから外す。
pub const NO_INPUT: [NextMove; 1] = [NextMove::AwaitingEffect];

/// 束の承認が当たる、A-1 の印の無い open の問いの最小の本数。
pub const BATCH_MIN: usize = 2;

/// 判じた当たり（件数と対象）。件数 0 は当たらない。
struct Found {
    count: usize,
    target: Option<BeadId>,
}

/// 止まっている走行: 段が Failed か Stopped の札で、その bead が open のもの
/// （対象はいちばん古い札 = 経過のいちばん長い札の bead）。event log か台帳が読めなければ判じない。
fn stalled_run(beads: Option<&[Bead]>, events: &str, now: EpochSecs) -> Option<Found> {
    let beads = beads?;
    let Reading::Known(cards) = pipeline::of_inputs(Some(beads), events, now).board.cards else {
        return None;
    };
    let stalled: Vec<_> = cards
        .into_iter()
        .filter(|c| STALLED_STAGES.contains(&c.stage))
        .filter(|c| {
            beads
                .iter()
                .any(|b| b.id == c.contract.as_str() && b.is_open(now))
        })
        .collect();
    Some(Found {
        count: stalled.len(),
        target: stalled
            .iter()
            .min_by_key(|c| std::cmp::Reverse(c.elapsed_s))
            .map(|c| c.contract.clone()),
    })
}

/// 質問: open の問い（対象は created_at がいちばん古い 1 本）。台帳が読めなければ判じない。
fn question(beads: Option<&[Bead]>, now: EpochSecs) -> Option<Found> {
    let open: Vec<&Bead> = beads?
        .iter()
        .filter(|b| b.kind == NodeKind::Question && b.is_open(now))
        .collect();
    Some(Found {
        count: open.len(),
        target: open
            .iter()
            .min_by_key(|b| (b.created.is_none(), b.created))
            .and_then(|b| BeadId::new(b.id.as_str()).ok()),
    })
}

/// 束の承認: A-1 の印の無い open の問いが `BATCH_MIN` 本以上なら、その本数（対象は無し）。
/// 問いの一覧が読めなければ判じない。
fn batch(ledger: &str) -> Option<Found> {
    let Reading::Known(qs) = open_questions(ledger) else {
        return None;
    };
    let n = qs.iter().filter(|q| !q.card.a1).count();
    Some(Found {
        count: if n >= BATCH_MIN { n } else { 0 },
        target: None,
    })
}

fn check(kind: NextMove, found: Option<Found>) -> NextCheck {
    match found {
        None => NextCheck {
            kind,
            result: CheckResult::NotJudged,
            count: 0,
            target: None,
        },
        Some(f) => NextCheck {
            kind,
            result: if f.count > 0 {
                CheckResult::Hit
            } else {
                CheckResult::Miss
            },
            count: u32::try_from(f.count).unwrap_or(u32::MAX),
            target: f.target,
        },
    }
}

/// 席の card から判じる 2 種（限度と移動・応答なし）。card が無いか状態が unknown なら判じない。
/// 限度と移動は、状態が limit か、登録の口座が群の今の口座と違うときに当たる（対象は無し・件数は 1）。
/// 応答なしは、状態が silent のときに当たる（対象は無し・件数は 1）。
fn seat_found(kind: NextMove, seat: Option<&SeatCard>) -> Option<Found> {
    let card = seat.filter(|c| c.state != SeatState::Unknown)?;
    let hit = match kind {
        NextMove::LimitOrMove => {
            let moved = match (&card.account, &card.group) {
                (Some(account), Reading::Known(group)) => *account != group.account,
                _ => false,
            };
            card.state == SeatState::Limit || moved
        }
        NextMove::Unresponsive => card.state == SeatState::Silent,
        _ => return None,
    };
    Some(Found {
        count: usize::from(hit),
        target: None,
    })
}

/// 台帳の一覧の字と event log の字と今の時刻から次の一手を判じる。
/// なしは、ほかの種類のどれかが当たれば当たらない。どれも当たらず、`NO_INPUT` に無い種類のどれかを
/// 判じなかったなら、なしも判じなかった（要件 NFR2: 読めないときは 0 件でなく測れていない）。ほかは当たる。
/// 大きく出す 1 つはいつも在る（当たる種類が無ければなし）が、なしを判じなかったときの lead のなしは
/// 測れていないの意味。
pub fn next_step(ledger: &str, events: &str, now: EpochSecs) -> NextStep {
    judge(ledger, events, now, None)
}

/// 席の card も受けて次の一手を判じる（card が無ければ限度と移動と応答なしは判じない）。
pub fn next_step_seat(
    ledger: &str,
    events: &str,
    now: EpochSecs,
    seat: Option<&SeatCard>,
) -> NextStep {
    judge(ledger, events, now, seat)
}

fn judge(ledger: &str, events: &str, now: EpochSecs, seat: Option<&SeatCard>) -> NextStep {
    let beads = read(ledger);
    let beads = beads.as_deref();
    let mut checks: Vec<NextCheck> = NextMove::ALL
        .into_iter()
        .filter(|&m| m != NextMove::Nothing)
        .map(|m| {
            let found = match m {
                NextMove::StalledRun => stalled_run(beads, events, now),
                NextMove::BatchApproval => batch(ledger),
                NextMove::Question => question(beads, now),
                NextMove::LimitOrMove | NextMove::Unresponsive => seat_found(m, seat),
                _ => None,
            };
            check(m, found)
        })
        .collect();
    let any_hit = checks.iter().any(|c| c.result == CheckResult::Hit);
    let unjudged = checks
        .iter()
        .any(|c| c.result == CheckResult::NotJudged && !NO_INPUT.contains(&c.kind));
    checks.push(NextCheck {
        kind: NextMove::Nothing,
        result: if any_hit {
            CheckResult::Miss
        } else if unjudged {
            CheckResult::NotJudged
        } else {
            CheckResult::Hit
        },
        count: 0,
        target: None,
    });
    let lead = checks
        .iter()
        .find(|c| c.result == CheckResult::Hit)
        .map_or(NextMove::Nothing, |c| c.kind);
    NextStep { checks, lead }
}
