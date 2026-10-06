//! 着地の終端が CI を見張らない歯（接頭辞 `vcioff_` と `vcicut_`・判断の記録 ADR-72 の決定 (4)・台帳の問い t3-hub.90.2 の裁定・
//! 親 `tests/e2e/pipe/land.rs` の helper を `use super::*` で使う）。
//!
//! 終端は push の後にこの host の緑で台帳を閉じ、close の後に何も起こさない（GitHub の検査を読む子 process も、`ci:` の行も無い）。
//! `vcioff_` は宣言の鍵 `ci-watch = false` の repo、`vcicut_` は鍵の無い repo（偽 CI が failure を答える）・終端だけの撃ち直し・
//! 退けた旗 `--ci-only` を、偽 remote・偽 CI・偽 bd の外形から測る。

use super::*;

/// 赤を返す偽 CI の答え（完了して failure）。
const FAILED: &str = r#"[{"status":"completed","conclusion":"failure"}]"#;

/// 便の `Landed` の後ろに並ぶ終端の行（`terminal:`）。
fn terminal_rows(state: &Path, id: &str) -> Vec<String> {
    landed_details(state, id).into_iter().filter(|detail| detail.starts_with("terminal:")).collect()
}

/// 便の `Landed` の detail の `sha:` の値（着地の sha・終端の行より前の 1 件目）。
fn landed_sha(state: &Path, id: &str) -> String {
    landed_details(state, id)
        .first()
        .and_then(|detail| detail.split_whitespace().find_map(|word| word.strip_prefix("sha:")).map(str::to_owned))
        .unwrap_or_default()
}

/// 終端の親が記す行（push・close）。
pub(super) const PUSHED: &str = "terminal:push:fake";
/// 台帳の close が通った行。
pub(super) const CLOSED: &str = "terminal:close:ok";

/// 宣言が `ci-watch = false` の repo の land は CI を見張らない（行 v-ci-watch-off）: 赤を返す偽 CI の便で、rc 0・
/// `terminal=closed`・台帳の argv は close と便の bead と `--reason` と `landed <sha> host=green`、便の終端の行は push と close の
/// 2 件だけで、偽 CI は 1 度も撃たれない。
#[test]
fn vcioff_land_closes_without_watching_ci() {
    let (repo, state) = repo_with_state();
    let tools = fake_terminal_json(&repo, &state, FAILED);
    ci_watch_off(&repo);
    let design = write_contract(&repo, &[], &[]);
    let id = gated_pass(&repo, &state, &design, &state.join("lens-ran"));
    let (bd, rules) = (state.join("fake-bd.sh").display().to_string(), ceiling_rules(&state));
    let out = land_extra(&repo, &state, &id, &["--bd", &bd, "--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "見張らない周の land は rc 0: {} {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("terminal=closed"), "終端の token: {}", stdout_of(&out));
    let sha = landed_sha(&state, &id);
    let argv = fs::read_to_string(&tools.bd_log).expect("台帳が閉じられた");
    let reason = format!("landed {sha} host=green");
    assert_eq!(argv.lines().collect::<Vec<&str>>(), ["close", "s2-2e5", "--reason", reason.as_str()], "台帳の argv: {argv}");
    assert_eq!(terminal_rows(&state, &id), [PUSHED, CLOSED], "終端の行は push と close の 2 件だけ");
    assert_eq!(tools.ci_call_count(), 0, "偽 CI は撃たれない");
    clean(&[&repo, &state]);
}

/// 鍵 `ci-watch` の無い repo の land も子を起こさず host の緑で閉じる（行 v-ci-child-cut）: 赤を返す偽 CI の便の land は rc 0・
/// `terminal=closed`・偽 remote の main は着地した sha・偽の台帳の argv は close と便の bead と `--reason` と
/// `landed <sha> host=green`・便の終端の行は push と close の 2 件だけ（子が在れば `ci:spawned` が親の記しで同期に載る）・偽 CI の
/// 呼びは 0 回。
#[test]
fn vcicut_land_closes_without_a_ci_child() {
    let (repo, state) = repo_with_state();
    let tools = fake_terminal_json(&repo, &state, FAILED);
    let design = write_contract(&repo, &[], &[]);
    let id = gated_pass(&repo, &state, &design, &state.join("lens-ran"));
    let (bd, rules) = (state.join("fake-bd.sh").display().to_string(), ceiling_rules(&state));
    let out = land_extra(&repo, &state, &id, &["--bd", &bd, "--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "赤の CI でも閉じた land は rc 0: {} {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("terminal=closed"), "終端の token: {}", stdout_of(&out));
    let sha = landed_sha(&state, &id);
    assert_eq!(git(&tools.remote, &["rev-parse", "refs/heads/main"]), sha, "偽 remote の main は着地した sha");
    let argv = fs::read_to_string(&tools.bd_log).expect("台帳が閉じられた");
    let reason = format!("landed {sha} host=green");
    assert_eq!(argv.lines().collect::<Vec<&str>>(), ["close", "s2-2e5", "--reason", reason.as_str()], "台帳の argv: {argv}");
    assert_eq!(terminal_rows(&state, &id), [PUSHED, CLOSED], "終端の行は push と close の 2 件だけ");
    assert_eq!(tools.ci_call_count(), 0, "偽 CI は撃たれない");
    clean(&[&repo, &state]);
}

/// 旗 `--ci-only` は無くなった（行 v-ci-child-cut）: 着地した便の land に `--ci-only` と着地した 40 桁の sha を渡すと、rc 2 で
/// stderr に `未知の引数 --ci-only` を出し、event を 1 件も書かない。
#[test]
fn vcicut_ci_only_flag_is_unknown() {
    let (repo, state) = repo_with_state();
    let _tools = fake_terminal_json(&repo, &state, FAILED);
    let design = write_contract(&repo, &[], &[]);
    let id = gated_pass(&repo, &state, &design, &state.join("lens-ran"));
    let (bd, rules) = (state.join("fake-bd.sh").display().to_string(), ceiling_rules(&state));
    let landed = land_extra(&repo, &state, &id, &["--bd", &bd, "--rules", &rules]);
    assert_eq!(landed.status.code(), Some(i32::from(RC_OK)), "前提: 着地して閉じた: {} {}", stdout_of(&landed), stderr_of(&landed));
    let sha = landed_sha(&state, &id);
    assert_eq!(sha.len(), 40, "前提: 着地した 40 桁の sha: {sha}");
    let before = event_count(&state);
    let out = land_extra(&repo, &state, &id, &["--bd", &bd, "--rules", &rules, "--ci-only", &sha]);
    assert_eq!(out.status.code(), Some(2), "旗は断られる: {} {}", stdout_of(&out), stderr_of(&out));
    assert!(stderr_of(&out).contains("未知の引数 --ci-only"), "断りの字: {}", stderr_of(&out));
    assert_eq!(event_count(&state), before, "event の数は不変");
    clean(&[&repo, &state]);
}
