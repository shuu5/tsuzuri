//! live な便の契約の読みと、宣言の同時の数の上限の判じ（vessel 宣言の任意 key `run-cap` と `run-cap-paths`・tsuzuri の判断の記録
//! ADR-63 の決定 (13)）。
//!
//! live の読みは交差（親の [`super::crossings`]）と上限の判じが同じ 1 本（[`live_contracts`]）で持ち、上限の判じは受付の断り
//! （[`exclude_run_cap`]）と列の候補の待ち（`pipe::dispatch` の候補の段）が同じ 1 本（[`capped`]）を読む（C2）。

use super::refusal::{denied, refuse, DENIAL_STORE};
use super::Denial;
use crate::cli_outcome::{Outcome, RC_BROKEN};
use crate::fleet::store::StoreError;
use crate::pipe::cli::live;
use crate::pipe::contract::Contract;
use crate::pipe::declaration::RunCap;
use crate::pipe::refuse::Refuse;
use crate::pipe::{contract_path, current};
use std::path::Path;

/// live な便と、その写しの契約（run id の順）。live な便の写しを 1 つでも読めない周は `WriteSetUnreadable`（rc 2・fail-closed）。
pub(super) fn live_contracts(state_dir: &Path) -> Result<Vec<(String, Contract)>, Denial> {
    let state = current(state_dir).map_err(|errors| {
        denied(DENIAL_STORE, Outcome::failed(RC_BROKEN, errors.iter().map(StoreError::to_string).collect()))
    })?;
    let mut found = Vec::new();
    for (id, run) in &state.runs {
        let Some(alive) = live(state_dir, id, run.stage) else {
            return Err(refuse(&Refuse::WriteSetUnreadable { run: id.clone() }, &[]));
        };
        if !alive {
            continue;
        }
        let Ok(live_contract) = Contract::load(&contract_path(state_dir, id)) else {
            return Err(refuse(&Refuse::WriteSetUnreadable { run: id.clone() }, &[]));
        };
        found.push((id.clone(), live_contract));
    }
    Ok(found)
}

/// 宣言 `run-cap` の上限の判じ。数える契約の周だけ live な便を読み、同じ周に起こした bead（`started`・起こした順）と live な便を
/// 数えて上限に達した周に先頭の 1 本の名を返す（数えない契約と上限の内は `None`）。読めない極性は交差と同じ。
pub(in crate::pipe) fn capped(state_dir: &Path, cap: &RunCap, contract: &Contract, started: &[(String, &Contract)]) -> Result<Option<String>, Denial> {
    if !cap.counts(&contract.write_set) {
        return Ok(None);
    }
    let live = live_contracts(state_dir)?;
    let others: Vec<(&str, &[String])> = started
        .iter()
        .map(|(bead, theirs)| (bead.as_str(), theirs.write_set.as_slice()))
        .chain(live.iter().map(|(id, theirs)| (id.as_str(), theirs.write_set.as_slice())))
        .collect();
    Ok(cap.holder(&others).map(str::to_owned))
}

/// 宣言 `run-cap` の上限に達した周の契約を断る（便を作る前だけ撃つ＝走行中の便は止めない）。宣言を読めない周は撃たない
/// （受付の `freeze` が宣言の不備で断る）。
pub(super) fn exclude_run_cap(repo: &Path, state_dir: &Path, contract: &Contract) -> Result<(), Denial> {
    let Ok(Some(cap)) = RunCap::at_head(repo) else {
        return Ok(());
    };
    match capped(state_dir, &cap, contract, &[])? {
        Some(run) => Err(refuse(&Refuse::RunCap { run, cap: cap.cap }, &[])),
        None => Ok(()),
    }
}
