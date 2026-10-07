//! 便の状態の helper（設計 §5「subcommand の置き場」・`s2-07l.349` で `cli.rs` から純移動・本文は不変）。
//!
//! 段の生死（[`live`]）・replay の段（[`stage_of`]）・段の前提の解き（[`resolve`] とその弁別 [`discriminate`] /
//! [`gated_is`]）・`--run` の読み（[`by_run`]）。材料の型（`Resolved` / `Extra`）は親の `cli` が持つ（構築点を
//! 動かさない）。外から呼ぶ path は `cli` の再輸出で不変（`super::resolve` 等）。

use super::intake::run_repo;
use super::{need, refused, state_dir_of, Extra, Resolved};
use crate::cli_outcome::{Outcome, RC_BROKEN};
use crate::fleet::store::StoreError;
use crate::fleet::{Stage, State};
use crate::pipe::contract::Contract;
use crate::pipe::gate::Verdict;
use crate::pipe::land::verdict_of;
use crate::pipe::review::ReviewCheck;
use crate::pipe::table::{parse_pointer, Pointer};
use crate::pipe::{contract_path, current, driver_ticket, repo_of_run, worktree_path, Ticket};
use std::path::{Path, PathBuf};

/// live な便の行の名札（設計 vessel-hook.md §15 形 1・閉じた 3 形）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Tag {
    /// 写しの design が `<doc>#<行 id>`。
    Row(Pointer),
    /// `#` を持たない古い写し（その doc の行が 1 つでも変われば当たり）。
    Doc(String),
    /// 写しを読めない（どの doc でも行が 1 つでも変われば当たり・測れないを「関係ない」に倒さない）。
    Unread,
}

/// live な便 1 本（run id・段・行の名札・対象 repo と便の worktree の path・設計 vessel-hook.md §15 形 2）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LiveRun {
    /// run id。
    pub(crate) id: String,
    /// 最後の段。
    pub(crate) stage: Stage,
    /// 行の名札。
    pub(crate) tag: Tag,
    /// 便の対象 repo（書き留めが無ければ `None`）。
    pub(crate) repo: Option<PathBuf>,
    /// 便の worktree（repo が無ければ `None`）。
    pub(crate) worktree: Option<PathBuf>,
}

/// live な便の列（**生死の判定は [`live`] 1 本**・C2）。replay して全 run に [`live`] を撃ち、`Some(true)` と `None`
/// （測れない便も live に数える＝受付の交差と同じ向き）の便の写しの design を名札にする。`Landed` は [`live`] が
/// live に数えないので含まない。event log を読めない周は `Err`（呼び手が fail-closed に倒す）。
pub(crate) fn live_runs(state_dir: &Path) -> Result<Vec<LiveRun>, Vec<StoreError>> {
    let state = current(state_dir)?;
    let alive = state.runs.iter().filter(|(id, run)| live(state_dir, id, run.stage) != Some(false));
    Ok(alive
        .map(|(id, run)| {
            let tag = Contract::load(&contract_path(state_dir, id)).map_or(Tag::Unread, |found| tag_of(&found.design));
            let repo = repo_of_run(state_dir, id);
            let worktree = repo.as_deref().map(|found| worktree_path(found, id));
            LiveRun { id: id.clone(), stage: run.stage, tag, repo, worktree }
        })
        .collect())
}

/// 写しの design を名札にする（`#` を持たない空でない path は doc だけ・pointer として読めない形は読めない）。
fn tag_of(design: &str) -> Tag {
    if design.trim().is_empty() {
        return Tag::Unread;
    }
    if !design.contains('#') {
        return Tag::Doc(design.to_owned());
    }
    parse_pointer(design).map_or(Tag::Unread, Tag::Row)
}

/// 便が live か。
/// 終端 = `Landed` / `Failed` / `Stopped`、または `Gated` で verdict が FAIL、または `Reviewed` で verdict が PASS でない。
/// `Gated` / `Reviewed` の 判定を読めない周は `None`＝**測れなかった**で、呼び手が断る側へ倒す。
/// 出所: pipeline.md §4 contract-source.md §4 ADR-0020 §2.1 dispatcher.md §18
pub(in crate::pipe) fn live(state_dir: &Path, id: &str, stage: Stage) -> Option<bool> {
    match stage {
        Stage::Landed | Stage::Failed | Stage::Stopped => Some(false),
        Stage::Gated => verdict_of(state_dir, id).map(|found| found != Verdict::Fail),
        Stage::Reviewed => ReviewCheck::judge(state_dir, id).live(),
        Stage::Intake => match driver_ticket(state_dir, id) {
            Ticket::Live => Some(true),
            Ticket::Dead | Ticket::Absent => Some(false),
            Ticket::Unreadable => None,
        },
        Stage::Blocked
        | Stage::Spawned
        | Stage::Questioned
        | Stage::RateLimited
        | Stage::Implemented => Some(true),
    }
}

/// 前提の段を replay から読む。無ければ `Err`。
pub(in crate::pipe) fn stage_of(state: &State, id: &str) -> Result<Stage, String> {
    state
        .runs
        .get(id)
        .map(|run| run.stage)
        .ok_or(format!("run {id} が無い"))
}

/// 段の前提を確かめ、材料を永続面から解く。**3 つの段（spawn / gate / land）が共有する**。
///
/// **順序を変えない**: 置き場 → replay → 段 → 契約 → repo。段の検査を契約より後ろへ
/// 動かすと、段違いの周に契約の error（rc 2）が先に出て「前提違反は何もせず rc 1」が
/// 崩れる（event も 1 件も書かない、という不変条件はこの順序に乗っている）。
///
/// [`Extra`] は段だけでは決まらない周の弁別を頼む印である。この弁別も段の検査の一部ゆえ
/// **契約より前**に置く——外へ出すと「段違いなのに rc 2」が特定の段だけで起こり、上の
/// 不変条件が rc の語彙ごと崩れる（lens 実測 F1）。
pub(in crate::pipe) fn resolve(
    args: &[String],
    id: &str,
    allowed: &[Stage],
    extra: &Extra,
) -> Result<Resolved, Outcome> {
    let state_dir = state_dir_of(args).map_err(refused)?;
    let state = current(&state_dir).map_err(|errors| {
        Outcome::failed(RC_BROKEN, errors.iter().map(StoreError::to_string).collect())
    })?;
    let stage = stage_of(&state, id).map_err(refused)?;
    if !allowed.contains(&stage) {
        return Err(refused(format!("run {id} の段は {} である", stage.as_str())));
    }
    discriminate(extra, &state_dir, id, stage)?;
    let Some(run) = state.runs.get(id) else {
        return Err(refused(format!("run {id} が無い")));
    };
    let contract = Contract::load(&contract_path(&state_dir, id)).map_err(|errors| {
        Outcome::failed(RC_BROKEN, errors.iter().map(ToString::to_string).collect())
    })?;
    let repo = run_repo(args, &state_dir, id).map_err(refused)?;
    Ok(Resolved {
        state_dir,
        repo,
        contract,
        bead: run.bead.clone(),
        stage,
        approved: run.approved,
    })
}

/// 段の中の弁別。
/// PASS / FAIL は判定に届いた終端で、 判定が読めない周も測り直さない。
/// PASS はまだ land が残っており、INCONCLUSIVE は測り直せる側ゆえ断る。
/// FAIL / INCONCLUSIVE は終端で、判定を読めない周も起こさない。
/// 出所: pipeline.md §24 §12 contract-source.md §4 s2-07l.353
fn discriminate(extra: &Extra, state_dir: &Path, id: &str, stage: Stage) -> Result<(), Outcome> {
    match (extra, stage) {
        (&Extra::Regate, Stage::Gated) => gated_is(state_dir, id, Verdict::Inconclusive),
        (&Extra::Retire, Stage::Gated) => gated_is(state_dir, id, Verdict::Fail),
        (&Extra::Spawn, Stage::Reviewed) => match ReviewCheck::judge(state_dir, id) {
            ReviewCheck::Passed => Ok(()),
            found => Err(reviewed_refusal(id, found)),
        },
        (&Extra::Retire, Stage::Reviewed) => match ReviewCheck::judge(state_dir, id) {
            ReviewCheck::Stopped(_) => Ok(()),
            found => Err(reviewed_refusal(id, found)),
        },
        (&Extra::Nothing | &Extra::Regate | &Extra::Retire | &Extra::Spawn, _) => Ok(()),
    }
}

/// `Reviewed` の便を通さない断り（**字面は起こす側と畳む側で 1 本**）。括弧の中の語は
/// [`ReviewCheck::as_str`] の 4 語で閉じる＝段違いの一般則の字面（`run <id> の段は Reviewed である`・
/// 括弧を持たない）と読み分けられる。
fn reviewed_refusal(id: &str, found: ReviewCheck) -> Outcome {
    refused(format!("run {id} の段は Reviewed である（verdict={}）", found.as_str()))
}

/// `Gated` の便の判定が求める 3 値か。**判定を読めない周は断る**（fail-closed・C11.2）。
fn gated_is(state_dir: &Path, id: &str, want: Verdict) -> Result<(), Outcome> {
    let verdict = verdict_of(state_dir, id);
    match verdict == Some(want) {
        true => Ok(()),
        false => Err(refused(format!(
            "run {id} の段は Gated である（verdict={}）",
            verdict.map_or("読めない", Verdict::as_str)
        ))),
    }
}

/// `--run` を読んでから段の関数へ渡す。
pub(super) fn by_run(args: &[String], step: impl FnOnce(&str) -> Outcome) -> Outcome {
    match need(args, "--run") {
        Err(reason) => refused(reason),
        Ok(id) => step(id),
    }
}
