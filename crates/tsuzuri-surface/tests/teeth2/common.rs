//! tsuzuri-surface の歯の群 teeth2 の共通の手（群の 2 つ以上の module が字で同じ写しを持っていた helper を 1 つだけ置く・畳みの道具 fold2.py が寄せる・判断の記録 ADR-63 の決定 (8) の (c)）。
//! 写しを持っていた module は use crate::common::… で呼ぶ。字が 1 文字でも違う物はここへ寄せない。
#![cfg(test)]

use std::path::PathBuf;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{PipelineCard, Stage};
use tsuzuri_contract::ledger::BeadId;

pub(crate) const NOW: EpochSecs = 1_790_000_000;

/// `start` から末尾までの字。
pub(crate) fn after<'a>(text: &'a str, start: &str) -> &'a str {
    let at = text
        .find(start)
        .unwrap_or_else(|| panic!("字 {start} が無い"));
    &text[at..]
}

pub(crate) fn card(id: &str, stage: Stage, since: u64) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(id).expect("id"),
        runs: 1,
        stage,
        reason: None,
        account: None,
        since: Some(since),
        ci: None,
    }
}

pub(crate) fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(crate) fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}
