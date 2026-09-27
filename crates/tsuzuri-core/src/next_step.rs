//! 次の一手（設計ノート surface-base 便 d・判断の記録 ADR-7 決定 (6)・規則の行 R-24）。
//! 閉じた 7 種を宣言の順（優先の順）に判じ、7 種それぞれの結果（当たった・当たらない・判じなかった）と
//! 件数と対象の id を返し、最初に当たった 1 つを大きく出す。入力は台帳の一覧の字と event log の字と今の時刻。
//! この便の入力で判じるのは 3 種（止まっている走行・質問・なし）。限度と移動・応答なし・束の承認・発効待ちは
//! 材料（席と口座の状態・束・発効待ちの記録）が入力に無いので判じない。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{NextMove, Reading, Stage};
use tsuzuri_contract::graph::NodeKind;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::stats::{CheckResult, NextCheck, NextStep};

use crate::ledger::{Bead, read};
use crate::pipeline;

/// 止まっている走行の段。
pub const STALLED_STAGES: [Stage; 3] = [Stage::Questioned, Stage::Failed, Stage::Stopped];

/// 判じた当たり（件数と対象）。件数 0 は当たらない。
struct Found {
    count: usize,
    target: Option<BeadId>,
}

/// 止まっている走行: 段が Questioned か Failed か Stopped の札で、その bead が open のもの
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

/// 台帳の一覧の字と event log の字と今の時刻から次の一手を判じる。
/// なしは、判じた種類のどれも当たらないときに当たる（だから大きく出す 1 つはいつも在る）。
pub fn next_step(ledger: &str, events: &str, now: EpochSecs) -> NextStep {
    let beads = read(ledger);
    let beads = beads.as_deref();
    let mut checks: Vec<NextCheck> = NextMove::ALL
        .into_iter()
        .filter(|&m| m != NextMove::Nothing)
        .map(|m| {
            let found = match m {
                NextMove::StalledRun => stalled_run(beads, events, now),
                NextMove::Question => question(beads, now),
                _ => None,
            };
            check(m, found)
        })
        .collect();
    let any_hit = checks.iter().any(|c| c.result == CheckResult::Hit);
    checks.push(NextCheck {
        kind: NextMove::Nothing,
        result: if any_hit {
            CheckResult::Miss
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
