//! 宣言の同時の数の上限の歯（接頭辞 `vrcap_`・tsuzuri の判断の記録 ADR-63 の決定 (13)・親 `tests/e2e/pipe/dispatch.rs` の helper を
//! `use super::*` で使う）。
//!
//! 宣言の末に `run-cap` と `run-cap-paths` を足した toy repo で、数える dir（`vessel/`）の下を書く行 a・b・d と外を書く行 c を、
//! live な便の下の受付（`pipe intake`）と列の 1 周（`pipe dispatch ls`）の外形で測る。

use super::super::intake::intake_with_rules;
use super::super::{run_dirs, write_rules};
use super::*;
use std::path::PathBuf;

/// 行の id と書く file（a・b・d は数える dir の下・c は外）。
const ROWS: [(&str, &str); 4] = [("a", "vessel/a.rs"), ("b", "vessel/b.rs"), ("c", "other/c.rs"), ("d", "vessel/d.rs")];

/// 上限 1 本の宣言の 2 行。
const ONE: &str = "run-cap = 1\nrun-cap-paths = [\"vessel/\"]\n";

/// 行 4 本を commit し、宣言の末に `extra` を足して commit した repo と置き場。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn place(extra: &str) -> (PathBuf, PathBuf) {
    let (repo, state) = repo_with_state();
    let rows: Vec<Vec<String>> =
        ROWS.iter().map(|&(id, file)| row_fields(id, &["write-set"], &[&format!("write-set = [\"{file}\"]")])).collect();
    commit_rows(&repo, &rows);
    let mut text = fs::read_to_string(repo.join(".vessel.toml")).expect("宣言を読める");
    text.push_str(extra);
    fs::write(repo.join(".vessel.toml"), text).expect("宣言を書ける");
    git(&repo, &["add", "-f", ".vessel.toml"]);
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "run-cap"]);
    (repo, state)
}

/// 行 `row` を bead `s2-<row>` で受付に撃つ（rc を測らない）。
fn intake_row(repo: &Path, state: &Path, row: &str) -> Output {
    let rules = write_rules(state, "rules-vrcap.toml", 1, 1_000_000).display().to_string();
    intake_with_rules(repo, state, &format!("{DESIGN_FILE}#{row}"), &format!("s2-{row}"), &rules)
}

/// 行 `row` の live な便を起こして run id を返す。
fn live(repo: &Path, state: &Path, row: &str) -> String {
    let out = intake_row(repo, state, row);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "行 {row} は通る（{}）", told(&out));
    run_id_of(&out)
}

/// 断りの 1 行目。
fn first_err(out: &Output) -> Option<String> {
    stderr_of(out).lines().next().map(str::to_owned)
}

/// 列の 1 周（台帳は bead と priority の組・行は bead の `s2-` の後ろ）の bead ごとの理由と件数の行。
fn listed(repo: &Path, state: &Path, beads: &[(&str, u64)]) -> (Vec<String>, String) {
    let issues: Vec<String> =
        beads.iter().map(|&(bead, priority)| issue(bead, priority, bead.trim_start_matches("s2-"))).collect();
    let out = ls(repo, state, &fake_bd(state, &issues));
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "ls は rc 0（{}）", told(&out));
    (beads.iter().map(|&(bead, _)| reason_of(&out, bead)).collect(), count_of(&out))
}

/// 上限 1 本の宣言の受付は、数える live な便の下で数える行 b を run id と本数の 1 行で断って run dir を作らず、外の行 c は通す。
#[test]
fn vrcap_intake_refuses_a_second_counted_run_and_passes_the_outside_row() {
    let (repo, state) = place(ONE);
    let held = live(&repo, &state, "a");
    let dirs = run_dirs(&state);
    let out = intake_row(&repo, &state, "b");
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "行 b は断る（{}）", told(&out));
    assert_eq!(first_err(&out), Some(format!("pipe: run-cap run={held} cap=1")), "{}", told(&out));
    assert_eq!(run_dirs(&state), dirs, "run dir を作らない");
    let outside = intake_row(&repo, &state, "c");
    assert_eq!(outside.status.code(), Some(i32::from(RC_OK)), "外の行 c は通る（{}）", told(&outside));
    clean(&[&repo, &state]);
}

/// 上限 1 本の宣言の列: live 0 本の周は同じ周に起こした b を名乗って d が待ち、数える live な便の下では b と d がその run id で
/// 待ち、外の行 c はどちらの周も起こす側に入る。
#[test]
fn vrcap_dispatch_waits_on_the_bead_started_in_the_round_and_on_the_live_run() {
    let (repo, state) = place(ONE);
    let beads = [("s2-b", 1), ("s2-d", 2), ("s2-c", 3)];
    let (reasons, count) = listed(&repo, &state, &beads);
    assert_eq!(reasons, ["-", "run-cap:s2-b", "-"], "同じ周に起こした b を数える");
    assert_eq!(count, format!("{COUNT} total=3 ready=2"));
    let held = live(&repo, &state, "a");
    let (reasons, count) = listed(&repo, &state, &beads);
    let wait = format!("run-cap:{held}");
    assert_eq!(reasons, [wait.as_str(), wait.as_str(), "-"], "live な a を数える");
    assert_eq!(count, format!("{COUNT} total=3 ready=1"));
    clean(&[&repo, &state]);
}

/// 本数は宣言の値: 上限 2 本の宣言は数える live な便 1 本の下で b を通して d を断り、2 key の無い宣言は b も d も通す。
#[test]
fn vrcap_cap_comes_from_the_declaration_and_no_keys_keep_todays_intake() {
    let (repo, state) = place("run-cap = 2\nrun-cap-paths = [\"vessel/\"]\n");
    let held = live(&repo, &state, "a");
    assert_eq!(intake_row(&repo, &state, "b").status.code(), Some(i32::from(RC_OK)), "2 本目は通る");
    let out = intake_row(&repo, &state, "d");
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "3 本目は断る（{}）", told(&out));
    assert_eq!(first_err(&out), Some(format!("pipe: run-cap run={held} cap=2")), "{}", told(&out));
    clean(&[&repo, &state]);
    let (repo, state) = place("");
    live(&repo, &state, "a");
    for row in ["b", "d"] {
        let out = intake_row(&repo, &state, row);
        assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "宣言の無い行 {row} は通る（{}）", told(&out));
    }
    clean(&[&repo, &state]);
}
