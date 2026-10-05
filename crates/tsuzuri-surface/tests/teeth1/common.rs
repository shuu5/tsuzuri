//! tsuzuri-surface の歯の群 teeth1 の共通の手（群の 2 つ以上の module が字で同じ写しを持っていた helper を 1 つだけ置く・畳みの道具 fold2.py が寄せる・判断の記録 ADR-63 の決定 (8) の (c)）。
//! 写しを持っていた module は use crate::common::… で呼ぶ。字が 1 文字でも違う物はここへ寄せない。
#![cfg(test)]

use std::path::{Path, PathBuf};
use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::wire;
use tsuzuri_surface::view::Fetched;

pub(crate) fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("bead の id")
}

pub(crate) fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(crate) fn fixture() -> AccountDoc {
    wire::decode(&fixture_text()).expect("fixture が AccountDoc として読める")
}

pub(crate) fn fixture_text() -> String {
    read("../../tests/fixtures/account/acct-doc.json")
}

pub(crate) fn ledger_questions() -> Fetched {
    Fetched::Body(read("../../tests/fixtures/surface/ledger-questions.json"))
}

pub(crate) fn question_list() -> Fetched {
    Fetched::Body(read("../../tests/fixtures/surface/question-list.json"))
}

pub(crate) fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// src の下の .rs の file の全部（path の順）。
pub(crate) fn sources() -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("src を読む").flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let mut paths = Vec::new();
    walk(&crate_dir().join("src"), &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|p| {
            let text = std::fs::read_to_string(&p).expect("src の file");
            (p, text)
        })
        .collect()
}
