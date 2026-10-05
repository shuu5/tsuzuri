//! tsuzuri-core の歯の群 teeth2 の共通の手（群の 2 つ以上の module が字で同じ写しを持っていた helper を 1 つだけ置く・畳みの道具 fold2.py が寄せる・判断の記録 ADR-63 の決定 (8) の (c)）。
//! 写しを持っていた module は use crate::common::… で呼ぶ。字が 1 文字でも違う物はここへ寄せない。
#![cfg(test)]

use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tsuzuri_contract::board::{PipelineCard, Reading};
use tsuzuri_contract::seat::SeatCard;
use tsuzuri_core::account::host::HostTexts;
use tsuzuri_core::graph::{Graph, Verdict, check};
use tsuzuri_core::pipeline::Board;
use tsuzuri_core::seat::{SeatTexts, anchor, card};

#[derive(Deserialize)]
pub(crate) struct Case {
    #[serde(flatten)]
    pub(crate) texts: SeatTexts,
    pub(crate) card: SeatCard,
}

pub(crate) const FIXTURE: &str = "tests/fixtures/seat/seat-inputs.json";

/// 節の今（2026-09-27T12:00:00Z）。
pub(crate) const NOW: u64 = 1_790_510_400;

/// server と同じ組み方（anchor は doctor の席の行から）。
pub(crate) fn build(target: &str, texts: &SeatTexts, now: u64) -> SeatCard {
    let anchor = texts.doctor.as_deref().and_then(|d| anchor(d, target));
    card(target, anchor.as_deref(), texts, now)
}

pub(crate) fn cards(b: &Board) -> &[PipelineCard] {
    match &b.board.cards {
        Reading::Known(c) => c,
        Reading::Unknown => panic!("札が Unknown"),
    }
}

pub(crate) fn fixture(rel: &str) -> Value {
    serde_json::from_str(&read(rel)).unwrap_or_else(|e| panic!("{rel} は JSON: {e}"))
}

pub(crate) fn host() -> HostTexts {
    HostTexts {
        host_toml: Some("[[account-group]]\nname = 'g'\nanchors = ['/w/p']\n".to_string()),
        ..HostTexts::default()
    }
}

pub(crate) fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

pub(crate) fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(crate) fn verdict_of(g: &Graph, id: &str) -> Verdict {
    check(g)
        .into_iter()
        .find(|i| i.id == id)
        .unwrap_or_else(|| panic!("不変条件 {id}"))
        .verdict
}
