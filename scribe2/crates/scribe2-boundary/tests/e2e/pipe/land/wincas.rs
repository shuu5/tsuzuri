//! 着地列の窓が CAS を過ぎた便を列に数えない歯（接頭辞 `vwcas_`・設計 pipeline.md §19 約束 8・tsuzuri の判断の記録 ADR-65・
//! 親 `tests/e2e/pipe/land.rs` の helper を `use super::*` で使う）。
//!
//! 列の 2 便のうち、自分の trailer（`run: <便 id>` と字が等しい行）を持つ commit が local main の祖先に在る便だけを `queue=` から
//! 外すことを、`pipe land-window` の rc と 1 行から測る。

use super::*;

/// 本文の末に `trailer` の 1 行を持つ空の commit を `branch` に積む（squash の印だけを置く形）。
fn commit_with_trailer(repo: &Path, branch: &str, trailer: &str) {
    git(repo, &["checkout", "-q", branch]);
    git(repo, &["commit", "-q", "--allow-empty", "-m", &format!("squash\n\n{trailer}")]);
    git(repo, &["checkout", "-q", "main"]);
}

/// 列の 2 便のうち、trailer が local main に在る便だけが `queue=` から外れ（rc 1 の busy は残る 1 便を名指す）、両方が在れば
/// rc 0 の `clear remote=none`。main は窓の口で動かない。
#[test]
fn vwcas_runs_whose_squash_is_on_local_main_leave_the_queue() {
    let (repo, state) = repo_with_state();
    let (id_a, id_b) = two_gated_runs(&repo, &state, &state.join("lens-ran"));
    commit_with_trailer(&repo, "main", &format!("run: {id_a}"));
    let out = land_window_once(&repo, &state);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "1 便が残る窓は rc 1: {}", stderr_of(&out));
    assert_eq!(
        stdout_of(&out).trim(),
        format!("land-window=busy queue={id_b} following=- unpushed=- remote=none"),
        "CAS を過ぎた {id_a} は列に数えない"
    );
    commit_with_trailer(&repo, "main", &format!("run: {id_b}"));
    let main = git(&repo, &["rev-parse", "refs/heads/main"]);
    let out = land_window_once(&repo, &state);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "両方が CAS を過ぎた窓は rc 0: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out).trim(), "land-window=clear remote=none", "列は空");
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), main, "main は動かない");
    clean(&[&repo, &state]);
}

/// trailer の字が便 id の後ろに 1 字多い commit（別の便の id が接頭辞で重なる形）と、local main の祖先でない branch だけに在る
/// 正しい trailer の commit は、どちらも便を列から外さない（rc 1 の busy が 2 便を名指す）。
#[test]
fn vwcas_a_longer_trailer_or_a_side_branch_keeps_the_run_queued() {
    let (repo, state) = repo_with_state();
    let (id_a, id_b) = two_gated_runs(&repo, &state, &state.join("lens-ran"));
    let both = format!("land-window=busy queue={id_a},{id_b} following=- unpushed=- remote=none");
    commit_with_trailer(&repo, "main", &format!("run: {id_a}x"));
    let out = land_window_once(&repo, &state);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "{}", stderr_of(&out));
    assert_eq!(stdout_of(&out).trim(), both, "1 字多い trailer は {id_a} の squash でない");
    git(&repo, &["branch", "side"]);
    commit_with_trailer(&repo, "side", &format!("run: {id_a}"));
    let out = land_window_once(&repo, &state);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "{}", stderr_of(&out));
    assert_eq!(stdout_of(&out).trim(), both, "main の祖先でない commit は数えない");
    clean(&[&repo, &state]);
}
