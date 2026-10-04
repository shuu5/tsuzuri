//! 閉じた bead の便の段（判断の記録 ADR-45 の門 H6）: 列の起こす側の 1 周（[`super::fire`]）が、同じ周の台帳の読みと event の
//! 列を借りて、台帳で閉じた bead の便を 2 つに分ける。
//!
//! - 終端の便（[`live`] が `Some(false)`・`Intake` を除く＝`pipe retire` の入口と同じ段の列）で作業木が在る便は、`pipe retire` の
//!   畳みと同じ 1 本（[`retire`] の `fold_only`）で畳む。clean の確かめ・並びの木の返し・記帳（段はそのまま・detail `retired`）は
//!   retire が持つ（第 2 の畳みを作らない・C2）。clean でない木は retire が断って残す（未 commit の仕事を運ばない）。
//! - live に数わるのに driver の居ない便（`Gated` の verdict PASS か `Reviewed` の審査 PASS で、札が無いか所有者が死んでいる）は
//!   亡骸と名指す。止めず畳まず、同じ周の起こし直しと終端の周の軸の idle の数えから外す（閉じた契約を着地させない・driver の居ない便の
//!   下で binary を替えても版は混ざらない）。止めるかは人か席が決める。
//!
//! 観測の口（[`super::turn`]・`dispatch ls`）は撃たない。台帳を読めない周は [`super::fire`] がこの段の前に返る（1 本も畳まない）。

use crate::cli_outcome::{RC_OK, RC_REFUSED};
use crate::fleet::store::LockPolicy;
use crate::fleet::{replay, Event, Stage};
use crate::pipe::cli::live;
use crate::pipe::land::{retire, Retire};
use crate::pipe::{driver_ticket, worktree_path, Ticket};
use crate::seat::ledger::Issue;
use std::collections::BTreeSet;

use super::{Input, CLOSED};

/// stderr の 1 行の書き出し。
const LINE: &str = "closed-runs";

/// 亡骸の無い周の便 id の列の字面。
const NONE: &str = "-";

/// 閉じた bead の便 1 本の扱い（**閉じた 3 値**）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// 終端の便（畳む候補・木の在否と clean は呼び手と retire が見る）。
    Settled,
    /// 亡骸（live に数わるのに driver が居ない）。
    Corpse,
    /// 触らない（`Intake`・live で driver が居る・live を測れない・札を読めない）。
    Kept,
}

/// 便 1 本の扱い（**pure**・札は亡骸の候補の周だけ読む）。live を測れない周は触らない（測れないを終端にも亡骸にも読み替えない・C10）。
fn kind_of(stage: Stage, live: Option<bool>, ticket: impl Fn() -> Ticket) -> Kind {
    match (stage, live) {
        (Stage::Intake, _) | (_, None) => Kind::Kept,
        (_, Some(false)) => Kind::Settled,
        (Stage::Gated | Stage::Reviewed, Some(true)) if matches!(ticket(), Ticket::Absent | Ticket::Dead) => Kind::Corpse,
        (_, Some(true)) => Kind::Kept,
    }
}

/// 1 周の結果（畳んだ数・畳めなかった数・亡骸の便 id）。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct Swept {
    /// 畳んだ便の数。
    folded: usize,
    /// 畳もうとして move か記帳が落ちた便の数（clean でない木の断りは数えない）。
    failed: usize,
    /// 亡骸の便 id（辞書順）。
    pub(super) corpses: BTreeSet<String>,
}

impl Swept {
    /// stderr の 1 行（`closed-runs folded=<n> failed=<n> live=<亡骸の数> ids=<便 id の列か ->`・畳んだ便か畳めなかった便か亡骸が在る周だけ）。
    pub(super) fn line(&self) -> Option<String> {
        if self.folded == 0 && self.failed == 0 && self.corpses.is_empty() {
            return None;
        }
        let ids = if self.corpses.is_empty() { NONE.to_owned() } else { self.corpses.iter().cloned().collect::<Vec<String>>().join(",") };
        Some(format!("{LINE} folded={} failed={} live={} ids={ids}", self.folded, self.failed, self.corpses.len()))
    }
}

/// 閉じた bead の便を畳み、亡骸を名指す（周の台帳の全件と event の列を借りる・event log を読めない周と lock の待ち方の行を読めない周は
/// 何もしない）。
pub(super) fn round(input: &Input<'_>, issues: &[Issue], events: Option<&[Event]>) -> Swept {
    let mut swept = Swept::default();
    let (Some(events), Ok(policy)) = (events, LockPolicy::from_rules(input.manifest)) else {
        return swept;
    };
    let closed: BTreeSet<&str> = issues.iter().filter(|issue| issue.status == CLOSED).map(|issue| issue.id.as_str()).collect();
    let state = replay(events);
    for run in state.runs.values().filter(|run| closed.contains(run.bead.as_str())) {
        let alive = live(input.state_dir, &run.id, run.stage);
        match kind_of(run.stage, alive, || driver_ticket(input.state_dir, &run.id)) {
            Kind::Settled if worktree_path(input.repo, &run.id).is_dir() => {
                let entry = Retire {
                    run: &run.id,
                    bead: &run.bead,
                    repo: input.repo,
                    state_dir: input.state_dir,
                    stage: run.stage,
                    policy,
                    bd: input.bd,
                    fold_only: true,
                    manifest: input.manifest,
                };
                match retire(&entry).rc {
                    RC_OK => swept.folded = swept.folded.saturating_add(1),
                    RC_REFUSED => {}
                    _ => swept.failed = swept.failed.saturating_add(1),
                }
            }
            Kind::Corpse => {
                swept.corpses.insert(run.id.clone());
            }
            Kind::Settled | Kind::Kept => {}
        }
    }
    swept
}

#[cfg(test)]
mod tests {
    use super::{kind_of, Kind, Swept};
    use crate::fleet::Stage;
    use crate::pipe::Ticket;

    /// 段・live・札の組ごとの扱い（`Intake` と測れない周は触らず、終端は札を読まずに畳む候補、`Gated` と `Reviewed` の live だけが札で亡骸に分かれる）。
    #[test]
    fn vcrun_kind_folds_settled_names_driverless_passes_and_keeps_the_rest() {
        let reads = std::cell::Cell::new(0_u32);
        let unread = || {
            reads.set(reads.get() + 1);
            Ticket::Absent
        };
        for stage in [Stage::Landed, Stage::Failed, Stage::Stopped, Stage::Gated, Stage::Reviewed] {
            assert_eq!(kind_of(stage, Some(false), unread), Kind::Settled, "{stage:?} の終端");
        }
        assert_eq!(kind_of(Stage::Intake, Some(false), unread), Kind::Kept, "Intake は retire の入口に無い");
        assert_eq!(kind_of(Stage::Intake, Some(true), unread), Kind::Kept, "生きた Intake も触らない");
        assert_eq!(kind_of(Stage::Gated, None, unread), Kind::Kept, "live を測れない周は触らない");
        for stage in [Stage::Gated, Stage::Reviewed] {
            assert_eq!(kind_of(stage, Some(true), || Ticket::Absent), Kind::Corpse, "{stage:?} の札なし");
            assert_eq!(kind_of(stage, Some(true), || Ticket::Dead), Kind::Corpse, "{stage:?} の死んだ札");
            assert_eq!(kind_of(stage, Some(true), || Ticket::Live), Kind::Kept, "{stage:?} の生きた札");
            assert_eq!(kind_of(stage, Some(true), || Ticket::Unreadable), Kind::Kept, "{stage:?} の読めない札");
        }
        for stage in [Stage::Blocked, Stage::Spawned, Stage::Questioned, Stage::RateLimited, Stage::Implemented] {
            assert_eq!(kind_of(stage, Some(true), unread), Kind::Kept, "{stage:?} は亡骸の段でない");
        }
        assert_eq!(reads.get(), 0, "終端の便と亡骸の段でない便は札を読まない");
    }

    /// 行は畳んだ数・畳めなかった数・亡骸の数と id の列（無い周は字 -）を持ち、3 つとも 0 の周は出ない。
    #[test]
    fn vcrun_line_names_counts_and_ids_only_when_something_happened() {
        assert_eq!(Swept::default().line(), None, "何も無い周は行を出さない");
        let folded = Swept { folded: 2, failed: 0, corpses: Default::default() };
        assert_eq!(folded.line().as_deref(), Some("closed-runs folded=2 failed=0 live=0 ids=-"));
        let failed = Swept { folded: 0, failed: 1, corpses: Default::default() };
        assert_eq!(failed.line().as_deref(), Some("closed-runs folded=0 failed=1 live=0 ids=-"));
        let corpses = Swept { folded: 0, failed: 0, corpses: ["b-2".to_owned(), "a-1".to_owned()].into_iter().collect() };
        assert_eq!(corpses.line().as_deref(), Some("closed-runs folded=0 failed=0 live=2 ids=a-1,b-2"));
    }
}
