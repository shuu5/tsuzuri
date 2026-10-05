//! tsuzuri-surface の歯の群 teeth5 の共通の手（群の 2 つ以上の module が字で同じ写しを持っていた helper を 1 つだけ置く・畳みの道具 fold2.py が寄せる・判断の記録 ADR-63 の決定 (8) の (c)）。
//! 写しを持っていた module は use crate::common::… で呼ぶ。字が 1 文字でも違う物はここへ寄せない。
#![cfg(test)]

use std::path::PathBuf;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::ledger::BeadId;

pub(crate) const AT: EpochSecs = NOW - 7_200;

/// 正時でない時点（2026-09-27 12:40Z・日本時間の 21:40）。
pub(crate) const AT_OFF: u64 = 1_790_512_800;

pub(crate) const E: &str = "E-字 surface を 2 面にする。";

pub(crate) const ID: &str = "t-1.2";

pub(crate) const NOW: EpochSecs = 1_790_000_000;

pub(crate) const P: &str = "P-字は画面を 2 つにする話です。";

pub(crate) const WHY: &str = "書く file の重なりを席が確かめる";

pub(crate) const X: &str = "X-字 本文の頭の 1 行";

pub(crate) fn bead(s: &str) -> BeadId {
    BeadId::new(s).expect("bead の id")
}

pub(crate) fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(crate) fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// file の字の start の字から end の字の前まで（どちらも在ることを断言する）。
pub(crate) fn span(src: &str, start: &str, end: &str) -> String {
    let at = src.find(start).unwrap_or_else(|| panic!("{start} が無い"));
    let rest = &src[at..];
    let to = rest.find(end).unwrap_or_else(|| panic!("{end} が無い"));
    rest[..to].to_string()
}

/// 先の歯の filter の語の前半（55 語）。
pub(crate) fn words_front() -> &'static [&'static str] {
    &[
        "accept_",
        "account_",
        "acctcore_",
        "acctdoc_",
        "acctframe_",
        "accthb_",
        "accthome_",
        "acctled_",
        "acctlook_",
        "acctpcore_",
        "acctproj_",
        "acctsess_",
        "acctwin_",
        "acctwire_",
        "askcard_",
        "batchpanel_",
        "board_min_",
        "contract_form_",
        "frame_",
        "gapspage_",
        "gquestion_",
        "graph_",
        "gview_",
        "hbconf_",
        "hbpost_",
        "hbproc_",
        "hbroute_",
        "hook_",
        "hsblock_",
        "hsderive_",
        "hspage_",
        "ledgerblock_",
        "mapgraph_",
        "mapview_",
        "nextstep_",
        "nodepage_",
        "parts_",
        "pipe_",
        "project_",
        "question_",
        "seatblock_",
        "seatcard_",
        "server_",
        "skeleton_",
        "stage_",
        "stats_",
        "steady_",
        "topbar_",
        "tz_",
        "mlink_",
        "gnav_",
        "mkeys_",
        "klink_",
        "ilink_",
        "afocus_",
    ]
}
