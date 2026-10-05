//! tsuzuri-boundary の歯の群 teeth2 の共通の手（群の 2 つ以上の module が字で同じ写しを持っていた helper を 1 つだけ置く・畳みの道具 fold2.py が寄せる・判断の記録 ADR-63 の決定 (8) の (c)）。
//! 写しを持っていた module は use crate::common::… で呼ぶ。字が 1 文字でも違う物はここへ寄せない。
#![cfg(test)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::{SystemTime, UNIX_EPOCH};
use tsuzuri_boundary::cli::graph::NEXT;
use tsuzuri_contract::ledger::BeadId;

/// event log（走行 1 本）。
pub(crate) const EVENTS: &str = "{\"schema\":1,\"ts\":\"2026-09-27T00:00:00Z\",\"kind\":\"RunCreated\",\"run\":\"k.1-20260927T000000Z\",\"bead\":\"k.1\",\"stage\":\"Intake\"}\n";

/// 着地済みの行とこの文書の行の verify の filter の語。
pub(crate) const FILTERS: [&str; 49] = [
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
    "hook_",
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
    "hbproc_",
    "hbconf_",
    "hbroute_",
    "hbpost_",
    "hsblock_",
    "hspage_",
    "hsderive_",
];

pub(crate) type Run = fn(&[&str]) -> u8;

/// 器の配達の口の target。
pub(crate) const TARGET: &str = "tsuzuri:0.1";

/// drop で path を消す守り（歯が通っても落ちても、worktree を模した .git を CARGO_TARGET_TMPDIR の下に残さない）。
pub(crate) struct Tidy(pub(crate) PathBuf);

pub(crate) fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("bead id")
}

/// 偽の bd の記録の file の行の数。
pub(crate) fn calls(root: &Path) -> usize {
    fs::read_to_string(root.join("calls"))
        .unwrap_or_default()
        .lines()
        .count()
}

/// bead 8 本の fixture（bd の読み取りの口の出力の形）。
pub(crate) fn fixture() -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/ledger/bd-list-8.json"),
    )
    .expect("fixture")
}

pub(crate) fn next(id: &str) -> &'static str {
    NEXT.iter()
        .find(|(i, _)| *i == id)
        .map(|(_, n)| *n)
        .expect("次の 1 手")
}

pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_secs()
}

pub(crate) fn owned(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

pub(crate) fn read_fixture(rel: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(rel),
    )
    .unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

pub(crate) fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

pub(crate) fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("標準エラーの字")
}

pub(crate) fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("標準出力の字")
}

pub(crate) fn summary(unknowns: usize) -> String {
    format!("tz graph --check: まだ分からない（違反 0・まだ分からない {unknowns}）\n")
}

pub(crate) fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8")
}

/// dir の中の file の path と byte の一覧（書かれていないことを比べる）。
pub(crate) fn tree(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).expect("dir を読む") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                out.push((path.clone(), Vec::new()));
                stack.push(path);
            } else {
                out.push((path.clone(), fs::read(&path).expect("file を読む")));
            }
        }
    }
    out.sort();
    out
}

impl Drop for Tidy {
    fn drop(&mut self) {
        match fs::symlink_metadata(&self.0) {
            Ok(meta) if meta.is_dir() => {
                let _ = fs::remove_dir_all(&self.0);
            }
            _ => {
                let _ = fs::remove_file(&self.0);
            }
        }
    }
}
