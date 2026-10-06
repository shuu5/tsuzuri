//! 着地の終端が CI を見張らない歯と着地の後の撃ちの歯（接頭辞 `vcicut_` と `valand_`・判断の記録 ADR-72 の決定 (4)・
//! 台帳の問い t3-hub.90.2 の裁定・親 `tests/e2e/pipe/land.rs` の helper を `use super::*` で使う）。
//!
//! 終端は push の後にこの host の緑で台帳を閉じ、close の後に何も起こさない（GitHub の検査を読む子 process も、`ci:` の行も無い）。
//! `vcicut_` は鍵 `ci-cmd` の無い repo（偽 CI が failure を答える）・終端だけの撃ち直し・
//! 退けた旗 `--ci-only` を、偽 remote・偽 CI・偽 bd の外形から測る。`valand_` は宣言の鍵 `after-land` の行を、着地の後に子が anchor で
//! 撃つ形（`after-land:` の行・子は待たないので [`wait_after_land`] の後に片付ける）を測る。

use super::super::dispatch::floor_rules;
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

/// 着地の後の撃ちの子の答えを待つ上限（秒・[`wait_after_land`]・`gate.rs` の子を待つ上限とは別の名）。
const AFTER_LAND_WAIT_S: u64 = 60;

/// 宣言の file の末に `after-land` の行を足して add と commit する（[`fake_terminal_decl`] と同じ書き）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn after_land_decl(repo: &Path, lines: &[&str]) {
    let body = fs::read_to_string(repo.join(".vessel.toml")).expect("宣言を読める");
    let list = lines.iter().map(|line| format!("\"{line}\"")).collect::<Vec<String>>().join(", ");
    fs::write(repo.join(".vessel.toml"), format!("{body}after-land = [{list}]\n")).expect("宣言を書ける");
    git(repo, &["add", "-f", ".vessel.toml"]);
    git(repo, &["commit", "-q", "-m", "after-land-decl"]);
}

/// 便の `Landed` の後ろに並ぶ着地の後の撃ちの行（`after-land:`）。
fn after_land_rows(state: &Path, id: &str) -> Vec<String> {
    landed_details(state, id).into_iter().filter(|detail| detail.starts_with("after-land:")).collect()
}

/// 子の答えの行（`after-land:<n>:rc=<数>`・`after-land:<n>:timeout`・`after-land:<n>:unfireable`）か。
fn answered(row: &str) -> bool {
    row.strip_prefix("after-land:").and_then(|rest| rest.split_once(':')).is_some_and(|(n, word)| {
        n.parse::<usize>().is_ok() && (word.starts_with("rc=") || word == "timeout" || word == "unfireable")
    })
}

/// 子の答えの行が `n` 件に届くまで 100 ミリ秒ごとに読み直し、届いた所までの `after-land:` の行を返す（上限は
/// [`AFTER_LAND_WAIT_S`]・届かない周は届いた所までを返す＝歯の断言が落ちる）。
fn wait_after_land(state: &Path, id: &str, n: usize) -> Vec<String> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(AFTER_LAND_WAIT_S);
    loop {
        let rows = after_land_rows(state, id);
        if rows.iter().filter(|row| answered(row)).count() >= n || std::time::Instant::now() >= deadline {
            return rows;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// 終端の親が記す行（push・close）。
pub(super) const PUSHED: &str = "terminal:push:fake";
/// 台帳の close が通った行。
pub(super) const CLOSED: &str = "terminal:close:ok";

/// 鍵 `ci-cmd` の無い repo の land も子を起こさず host の緑で閉じる（行 v-ci-child-cut）: 赤を返す偽 CI の便の land は rc 0・
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

/// 鍵 `after-land` の行は着地の後に子が anchor で順に撃つ（行 v-after-land）: 2 行の tag を宣言した repo の land は rc 0・
/// `terminal=closed`、子の答えを待った後の `after-land:` の行は spawned と 1 行目と 2 行目の rc 0 の 3 件をこの順で持ち、anchor の
/// 2 つの tag はどちらも着地した sha を指す。
#[test]
fn valand_land_fires_the_declared_lines_in_the_anchor() {
    let (repo, state) = repo_with_state();
    let _tools = fake_terminal_json(&repo, &state, FAILED);
    after_land_decl(&repo, &["git tag valand-mark", "git tag valand-second"]);
    let design = write_contract(&repo, &[], &[]);
    let id = gated_pass(&repo, &state, &design, &state.join("lens-ran"));
    let (bd, rules) = (state.join("fake-bd.sh").display().to_string(), floor_rules(&state, Some(60)));
    let out = land_extra(&repo, &state, &id, &["--bd", &bd, "--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "着地して閉じた land は rc 0: {} {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("terminal=closed"), "終端の token: {}", stdout_of(&out));
    let rows = wait_after_land(&state, &id, 2);
    assert_eq!(rows, ["after-land:spawned", "after-land:1:rc=0", "after-land:2:rc=0"], "着地の後の撃ちの行");
    let sha = landed_sha(&state, &id);
    for tag in ["valand-mark", "valand-second"] {
        assert_eq!(git(&repo, &["rev-parse", &format!("refs/tags/{tag}^{{commit}}")]), sha, "tag {tag} は着地した sha を指す");
    }
    clean(&[&repo, &state]);
}

/// rc 0 でない行は着地を取り消さず後の行を撃たない（行 v-after-land）: 1 行目が rc 1 で終わる宣言の repo の land は rc 0・
/// `terminal=closed`・偽 bd の argv は close と便の bead と `--reason` と `landed <sha> host=green`、子の答えの行は spawned と
/// `1:rc=1` の 2 件だけで、2 行目の tag は anchor に無い。
#[test]
fn valand_failed_line_keeps_the_landing_and_stops() {
    let (repo, state) = repo_with_state();
    let tools = fake_terminal_json(&repo, &state, FAILED);
    after_land_decl(&repo, &["git rev-parse --verify -q refs/heads/valand-none", "git tag valand-mark"]);
    let design = write_contract(&repo, &[], &[]);
    let id = gated_pass(&repo, &state, &design, &state.join("lens-ran"));
    let (bd, rules) = (state.join("fake-bd.sh").display().to_string(), floor_rules(&state, Some(60)));
    let out = land_extra(&repo, &state, &id, &["--bd", &bd, "--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "行が赤でも land は rc 0: {} {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("terminal=closed"), "終端の token: {}", stdout_of(&out));
    let rows = wait_after_land(&state, &id, 1);
    assert_eq!(rows, ["after-land:spawned", "after-land:1:rc=1"], "赤の行で止まる");
    let sha = landed_sha(&state, &id);
    let argv = fs::read_to_string(&tools.bd_log).expect("台帳が閉じられた");
    let reason = format!("landed {sha} host=green");
    assert_eq!(argv.lines().collect::<Vec<&str>>(), ["close", "s2-2e5", "--reason", reason.as_str()], "台帳の argv: {argv}");
    assert_eq!(git(&repo, &["tag", "--list", "valand-mark"]), "", "2 行目は撃たれない");
    clean(&[&repo, &state]);
}

/// 鍵の無い宣言の repo は子を起こさない（行 v-after-land）: land は rc 0・`terminal=closed`・`after-land:` の行を 1 件も持たない。
#[test]
fn valand_no_key_fires_nothing() {
    let (repo, state) = repo_with_state();
    let _tools = fake_terminal_json(&repo, &state, FAILED);
    let design = write_contract(&repo, &[], &[]);
    let id = gated_pass(&repo, &state, &design, &state.join("lens-ran"));
    let (bd, rules) = (state.join("fake-bd.sh").display().to_string(), ceiling_rules(&state));
    let out = land_extra(&repo, &state, &id, &["--bd", &bd, "--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "鍵の無い周の land は rc 0: {} {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("terminal=closed"), "終端の token: {}", stdout_of(&out));
    assert_eq!(after_land_rows(&state, &id), Vec::<String>::new(), "子は起こされない");
    clean(&[&repo, &state]);
}
