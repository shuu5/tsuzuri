//! 受付の live な便との交差と同時本数（`intake.rs` から純移動・判断の記録 ADR-77 の決定 (7) の行 77-1・本文は不変）。

use super::run_cap::live_contracts;
use super::{broken, denied, int_row, refuse, Denial, Material, DENIAL_RULES};
use crate::pipe::commute::{self, Crossed};
use crate::pipe::contract::Contract;
use crate::pipe::refuse::{overlaps, Refuse};
use crate::rules::manifest::Manifest;
use std::path::Path;

/// host で同時に走る便（live な便）の本数の最大値を持つ rules 行（設計 gate-cost.md §24・値は読むだけ・C1）。
const ROW_MAX_LIVE: &str = "pipe.max_live";

/// live な便（終端でない run）と write-set が交差する契約を断る（設計 pipeline-conflict.md §2）。
///
/// **読めない側が勝つ**: live な便の写しを 1 つでも読めなければ、交差の有無に関わらず
/// `WriteSetUnreadable`（rc 2）で止まる。読めない store を「交差なし」に読み替えると、
/// 排他が黙って無効化される（fail-closed・NFR4）。
///
/// 交差した周は**全組を stderr へ並べ**、理由の 1 行は先頭の 1 組を名乗る。dir 項目は base の tracked file に
/// 展開してから数える（設計 contract-source.md §3・[`overlaps`]）。通った周は突き合わせた live な run を [`Crossed`]
/// で返す（交差は 0 か [`Crossed::settle`] が通した組・§21 の `overlap=` の材料）。
pub(super) fn exclude_overlap(material: &Material<'_>, state_dir: &Path, contract: &Contract, tracked: &[String]) -> Result<Crossed, Denial> {
    let mut found = crossings(state_dir, contract, tracked)?;
    found.settle(&commute::Scene { repo: material.repo, manifest: material.manifest, state_dir, tracked }, contract);
    match &found.first {
        None => Ok(found),
        Some(reason) => Err(refuse(reason, &found.lines)),
    }
}

/// 置き場の live な便の本数が rules 行 `pipe.max_live` の値以上の周を断る（設計 gate-cost.md §24・受付だけ）。
///
/// 数えるのは [`crossings`] が突き合わせた live な run（**交差と同じ 1 本**の live の判定・C2）で、読めない便が在る周は
/// 交差と同じ `WriteSetUnreadable`（rc 2・fail-closed）。便を作る前だけ撃つ＝走行中の便は止めない。行の無い manifest は
/// 受付を動かさない（rc 2）。
pub(super) fn exclude_max_live(manifest: &Manifest, state_dir: &Path, contract: &Contract, tracked: &[String]) -> Result<(), Denial> {
    let cap = int_row(manifest, ROW_MAX_LIVE).map_err(|reason| denied(DENIAL_RULES, broken(reason)))?;
    let live = u64::try_from(crossings(state_dir, contract, tracked)?.runs.len()).unwrap_or(u64::MAX);
    if live >= cap {
        return Err(refuse(&Refuse::MaxLive { live, cap }, &[]));
    }
    Ok(())
}

/// live な便との交差を**測るだけ**の 1 本（断りは作らない・設計 dispatcher.md §3）。
///
/// [`exclude_overlap`]（受付＝交差 1 件で断る）と `pipe::dispatch`（列＝交差した相手を待ちの理由にする）が
/// **同じこの 1 本**を読む（判定が 2 か所にならない・憲法 C2）。読めない側が勝つ極性はここが持つ。
pub(in crate::pipe) fn crossings(state_dir: &Path, contract: &Contract, tracked: &[String]) -> Result<Crossed, Denial> {
    let mut first: Option<Refuse> = None;
    let mut lines: Vec<String> = Vec::new();
    let mut runs: Vec<(String, Vec<String>)> = Vec::new();
    for (id, live_contract) in live_contracts(state_dir)? {
        let mut crossed: Vec<String> = Vec::new();
        for (mine, theirs) in overlaps(&contract.write_set, &live_contract.write_set, tracked) {
            if first.is_none() {
                first = Some(Refuse::WriteSetOverlap { run: id.clone(), path: mine.clone(), verdict: None });
            }
            lines.push(format!("pipe: overlap run={id} contract={mine} live={theirs}"));
            crossed.push(mine);
        }
        runs.push((id, crossed));
    }
    Ok(Crossed { runs, first, lines, commuted: None })
}
