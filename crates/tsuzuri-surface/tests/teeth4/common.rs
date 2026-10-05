//! tsuzuri-surface の歯の群 teeth4 の共通の手（群の 2 つ以上の module が字で同じ写しを持っていた helper を 1 つだけ置く・畳みの道具 fold2.py が寄せる・判断の記録 ADR-63 の決定 (8) の (c)）。
//! 写しを持っていた module は use crate::common::… で呼ぶ。字が 1 文字でも違う物はここへ寄せない。
#![cfg(test)]

use std::path::PathBuf;
use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::wire;

pub(crate) const FIXTURE: &str = "../../tests/fixtures/surface/around-doc.json";

/// 着地済みの pipe_ の歯と同じ今（UTC の日の正午）。
pub(crate) const NOW: u64 = 1_790_510_400;

pub(crate) fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(crate) fn fixture() -> AccountDoc {
    wire::decode(&read("../../tests/fixtures/account/acct-doc.json"))
        .expect("fixture が AccountDoc として読める")
}

pub(crate) fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}
