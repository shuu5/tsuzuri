//! 受付の同型の停止と焼き直しの門（設計 contract-source.md §23・`intake.rs` から純移動・判断の記録 ADR-77 の決定 (7) の行 77-1・本文は不変）。

use super::refusal::refuse_of;
use super::row::{pointed_row, Pointed};
use super::{broken, denied, int_row, refuse, Denial, Judged, Material, Materials, DENIAL_RULES, DENIAL_STORE};
use crate::cli_outcome::{Outcome, RC_BROKEN};
use crate::fleet::store::{self, StoreError};
use crate::fleet::{replay, Event, EventKind, Stage, State};
use crate::pipe::contract::Contract;
use crate::pipe::fall::{self, Attempt, Fall, Tally};
use crate::pipe::refuse::Refuse;
use crate::pipe::review::{self, Judgement, ROW_SAME_KIND_STOP};
use crate::pipe::table::ContractRow;
use crate::pipe::contract_path;
use std::path::Path;

/// 同じ bead の直前までの便 1 つ（新しい順の列の要素・§23）。
struct Past {
    /// 便 id。
    id: String,
    /// `review.json` の判定。
    judgement: Judgement,
}

/// 今回の材料（§23・受付が写す形の契約 file〔導出値を置いた後〕と base から読む節の本文〔審査と同じ 1 本
/// [`review::design_material`]〕と、行の `touches`）。
struct Today {
    /// 契約 file の字面（[`crate::pipe::contract::render`]＝`create` が写す形と同じ 1 本）。
    contract: String,
    /// 節の本文（`review/design.txt` と同じ形）。
    design: String,
    /// 今回の write-set（弁別済み＝導出値を置いた後）。
    write_set: Vec<String>,
    /// 設計 pointer の行。
    row: ContractRow,
    /// 行が約束の行を持つ（[`WriteSet::Promised`]・§33 行 ah: 焼き直しの門は契約 file の字面だけを見る）。
    promised: bool,
}

/// 受付の 2 門（§23・[`exclude_same_kind`] と [`exclude_unaddressed`]）を撃ち、各門の断りを列に積む。
///
/// rules 行 `review.same_kind_stop` は**置き場の有無に依らず**要る（行の無い manifest は受付を 1 byte も動かさない・
/// rc 2・`pipe.land_wait_s` と同じ極性）。置き場が無い周（preflight の `--state-dir` 無し・列の余地の判定）は便の履歴を
/// 読めないので 2 門とも撃たない（交差と同じ扱い）。pointer でない `design` は行を持たないので撃たない。
pub(super) fn exclude_repeats(material: &Material<'_>, measured: &Contract, judged: &mut Judged) {
    let Material { repo, manifest, state_dir, bead, materials, .. } = *material;
    let stop = match int_row(manifest, ROW_SAME_KIND_STOP) {
        Ok(found) => found,
        Err(reason) => return judged.denials.push(denied(DENIAL_RULES, broken(reason))),
    };
    let Some(state_dir) = state_dir else {
        return;
    };
    let today = match today_of(repo, measured, materials) {
        Ok(Some(found)) => found,
        Ok(None) => return,
        Err(denial) => return judged.denials.push(denial),
    };
    let events = match store::read_all(state_dir) {
        Ok(found) => found,
        Err(errors) => return judged.denials.push(denied(DENIAL_STORE, Outcome::failed(RC_BROKEN, errors.iter().map(StoreError::to_string).collect()))),
    };
    let state = replay(&events);
    let past = match history(state_dir, &state, bead) {
        Ok(found) => found,
        Err(denial) => return judged.denials.push(denial),
    };
    if let Err(denial) = exclude_same_kind(state_dir, (&events, &state), bead, stop, &today) {
        judged.denials.push(denial);
    }
    if let Err(denial) = exclude_unaddressed(state_dir, &past, &today, materials) {
        judged.denials.push(denial);
    }
}

/// 今回の材料を組む（行の無い pointer は `Ok(None)`・行の解けない周は [`pointed_row`] の断り）。
fn today_of(repo: &Path, measured: &Contract, materials: &Materials) -> Result<Option<Today>, Denial> {
    let Some(Pointed { row, promised }) = pointed_row(repo, measured, materials)? else {
        return Ok(None);
    };
    let contract = crate::pipe::contract::render(&row, &measured.design, &measured.write_set);
    let design = review::design_material(repo, &measured.design);
    Ok(Some(Today { contract, design, write_set: measured.write_set.clone(), row, promised: promised.is_some() }))
}

/// 置き場の replay から同じ bead の便を **id の新しい順**に読む（run id は `<bead>-<UTC の秒>`＝id の降順が時系列の
/// 逆順）。段が Reviewed 以降の便は `review.json` を要り、読めない便は `WriteSetUnreadable`（rc 2・`live` と同じ読み手・
/// 読めなさを「判定なし」に読み替えない）。段が `Intake` の便と、審査に届く前に終端した便（`review.json` の file が
/// 無い `Landed` / `Failed` / `Stopped`）は数えない。
fn history(state_dir: &Path, state: &State, bead: &str) -> Result<Vec<Past>, Denial> {
    let mut found = Vec::new();
    for (id, run) in state.runs.iter().rev().filter(|(_, run)| run.bead == bead) {
        if !reviewed_stage(state_dir, id, run.stage) {
            continue;
        }
        let Some(judgement) = review::judgement_of(state_dir, id) else {
            return Err(refuse(&Refuse::WriteSetUnreadable { run: id.clone() }, &[]));
        };
        found.push(Past { id: id.clone(), judgement });
    }
    Ok(found)
}

/// 便が審査の判定を持つ段か（**段の網羅 match**・段が増えたら compile で気付く）。`Intake` は審査の前。終端の 3 段は
/// 審査に届く前（lens の途中で process が消えた便を `stop` した周）にも着くので、`review.json` の file の有無で分ける。
fn reviewed_stage(state_dir: &Path, id: &str, stage: Stage) -> bool {
    match stage {
        Stage::Intake => false,
        Stage::Landed | Stage::Failed | Stage::Stopped => review::review_path(state_dir, id).exists(),
        Stage::Reviewed
        | Stage::Blocked
        | Stage::Spawned
        | Stage::Questioned
        | Stage::RateLimited
        | Stage::Implemented
        | Stage::Gated => true,
    }
}

/// 同型の停止（§23 (2)・判断の記録 ADR-77 の決定 (7) の行 77-4・条 P-10.2）: 同じ bead の落ちを型の語ごとに累計し
/// （[`fall::counted`]・便の落ちの型と受付の断り）、止めて書き直しの便（[`rewritten_runs`]）を型の語 `rewritten` の証に足して、
/// 証の数が行の値に達した最初の型（型の語の順）を [`Refuse::SameKindRepeated`] で断る。数えは累計で、PASS の便も契約か節の字の
/// 替えも別の型の落ちを挟むことも戻さない。
fn exclude_same_kind(
    state_dir: &Path,
    (events, state): (&[Event], &State),
    bead: &str,
    stop: u64,
    today: &Today,
) -> Result<(), Denial> {
    let mut tallies = fall::counted(events, &[bead], fall::UNCOUNTED_FOR_BEAD);
    let proofs = rewritten_runs(state_dir, (events, state), bead, today)?;
    if !proofs.is_empty() {
        let word = Fall::Rewritten.as_str();
        match tallies.iter_mut().find(|found| found.kind == word) {
            Some(found) => {
                found.proofs.extend(proofs);
                found.proofs.sort_by(|left, right| right.cmp(left));
            }
            None => {
                // 型の語の順: 落ちの型（`rewritten` は末尾）の後に受付の断りの型。
                let at = tallies.iter().position(|found| found.kind.starts_with("intake-")).unwrap_or(tallies.len());
                tallies.insert(at, Tally { kind: word.to_owned(), proofs });
            }
        }
    }
    let reached = tallies.into_iter().find(|found| found.proofs.len() as u64 >= stop);
    match reached {
        Some(Tally { kind, proofs }) => Err(refuse(&Refuse::SameKindRepeated { kind, runs: proofs, stop }, &[])),
        None => Ok(()),
    }
}

/// 止めて書き直しの便（[`fall::rewritten`]）: event の列の最初の `RunFell` の行より後に `RunCreated` の行を持つ同じ bead の便の
/// 材料（契約 file の写しと `design.txt` の写し）を古い順に読んで渡す。審査の前に止めた便が無い周は材料を読まない。契約 file の写しを
/// 読めない便は `WriteSetUnreadable`（材料は便を起こす時に置かれるので、無い便は壊れた store）。
fn rewritten_runs(
    state_dir: &Path,
    (events, state): (&[Event], &State),
    bead: &str,
    today: &Today,
) -> Result<Vec<String>, Denial> {
    let Some(first) = events.iter().position(|event| event.kind == EventKind::RunFell) else {
        return Ok(Vec::new());
    };
    let created = events.iter().skip(first).filter(|event| event.kind == EventKind::RunCreated && event.bead == bead);
    let runs: Vec<(&str, bool)> = created
        .filter_map(|event| state.runs.get(&event.run))
        .map(|run| (run.id.as_str(), run.stage == Stage::Stopped && !review::review_path(state_dir, &run.id).exists()))
        .collect();
    if !runs.iter().any(|(_, unreviewed)| *unreviewed) {
        return Ok(Vec::new());
    }
    let mut attempts = Vec::new();
    for (id, unreviewed) in runs {
        let Ok(contract) = std::fs::read_to_string(contract_path(state_dir, id)) else {
            return Err(refuse(&Refuse::WriteSetUnreadable { run: id.to_owned() }, &[]));
        };
        let design = std::fs::read_to_string(review::review_dir(state_dir, id).join(review::DESIGN_FILE)).ok();
        attempts.push(Attempt { run: id.to_owned(), unreviewed, contract, design });
    }
    Ok(fall::rewritten(&attempts, (&today.contract, &today.design)))
}

/// 焼き直しの門（§23 (3)）: 直前の便（新しい順の先頭）の verdict が PASS でない周、その指摘（`kind` と `at`）に対応する
/// 差分が今回の材料に在るかを kind ごとの物差し（[`review::unaddressed`]）で測り、対応の無い項目が 1 つでも在れば
/// [`Refuse::FindingUnaddressed`]（項目は辞書順）。測れない型と `at` の空な周は物差しが空を返す＝通す。
///
/// Promised の行（§33 行 ah (iv)）は `at` の物差しを撃たずに通す: 契約 file は約束の行からの生成値で、焼き直しは
/// 契約 file の字面が変わったかだけで測る（同じ型の落ちの数えは材料の替えで戻らない・[`exclude_same_kind`] が断る）。
fn exclude_unaddressed(state_dir: &Path, past: &[Past], today: &Today, materials: &Materials) -> Result<(), Denial> {
    if today.promised {
        return Ok(());
    }
    let Some(head) = past.first() else {
        return Ok(());
    };
    let Some(kind) = head.judgement.kind else {
        return Ok(());
    };
    let Ok(previous) = std::fs::read_to_string(review::review_dir(state_dir, &head.id).join(review::DESIGN_FILE)) else {
        return Err(refuse(&Refuse::WriteSetUnreadable { run: head.id.clone() }, &[]));
    };
    let rework = review::Rework {
        write_set: &today.write_set,
        contract: &today.contract,
        design: &today.design,
        previous_design: &previous,
        touches: &today.row.touches,
        tracked: &materials.tracked,
        sources: &materials.sources,
    };
    let at = review::unaddressed(kind, &head.judgement.at, &rework).map_err(|error| refuse(&refuse_of(error, &today.row), &[]))?;
    if at.is_empty() {
        return Ok(());
    }
    Err(refuse(&Refuse::FindingUnaddressed { kind, at }, &[]))
}
