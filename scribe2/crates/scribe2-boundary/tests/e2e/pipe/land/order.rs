// flip-check: moved s2-07l.684
//! terminal と order の族の歯（接頭辞 `pipe_terminal_` / `pipe_order_` / `pipe_replay_` と、終端が子を起こさない歯 `vcicut_`・設計 docs/design/carry-prep.md §10 行 l・親 `tests/e2e/pipe/land.rs` の helper を `use super::*` で使う）。

use super::*;
use super::cilast::{CLOSED, PUSHED};

/// 着地の順番（設計 gate-cost.md §6）: 後から Gated になった便は前の便が列に居る間は待ち、上限
/// （fixture 1 秒）で**待たずに進む**（`order=degraded`・断らない・止めない）。main は動いていないので
/// 追随せずに Landed。
#[test]
fn pipe_order_later_run_degrades_at_the_limit_and_lands() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let (id_a, id_b) = two_gated_runs(&repo, &state, &marker);
    let rules = write_rules_land_wait(&state, "rules-order.toml", Some(LAND_WAIT_S));
    let out = land_extra(&repo, &state, &id_b, &["--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "上限で進んで land する: {}", stderr_of(&out));
    assert_eq!(order_token(&out), "degraded", "stdout の land 行: {}", stdout_of(&out));
    assert_eq!(exported_order(&state, &id_b), "degraded", "面 5 の record");
    assert!(show_line(&repo, &state, &id_b).contains("stage=Landed"), "段は Landed");
    assert!(show_line(&repo, &state, &id_a).contains("stage=Gated"), "前の便は列に残ったまま");
    clean(&[&repo, &state]);
}

/// 先に Gated になった便は待たない（`order=first`）。
#[test]
fn pipe_order_oldest_run_lands_first_without_waiting() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let (id_a, _id_b) = two_gated_runs(&repo, &state, &marker);
    let rules = write_rules_land_wait(&state, "rules-order.toml", Some(30));
    let out = land_extra(&repo, &state, &id_a, &["--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "land: {}", stderr_of(&out));
    assert_eq!(order_token(&out), "first", "stdout の land 行: {}", stdout_of(&out));
    assert_eq!(exported_order(&state, &id_a), "first", "面 5 の record");
    clean(&[&repo, &state]);
}

/// 待っている便は、前の便が land して列を空けた時点で**上限を待たずに**進み（`order=waited:<n>`・n < 上限）、
/// 追随 1 回・gate の撃ち直し 1 回で Landed（撃ち直しの間は main が動かない＝(vi) が起きない）。
#[test]
fn pipe_order_waiting_run_lands_after_the_front_with_one_follow() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let (id_a, id_b) = two_gated_runs(&repo, &state, &marker);
    let rules = write_rules_land_wait(&state, "rules-order.toml", Some(30));
    let lens = fake_lens(&marker, &lens_verdict("PASS"));
    let mut waiting = land_in_background(&repo, &state, &id_b, &rules, &lens);
    std::thread::sleep(Duration::from_secs(2));
    assert!(waiting.try_wait().expect("子の状態を読める").is_none(), "後の便は列の前が空くまで待っている");
    let first = land_extra(&repo, &state, &id_a, &["--rules", &rules]);
    assert_eq!(first.status.code(), Some(i32::from(RC_OK)), "前の便の land: {}", stderr_of(&first));
    assert_eq!(order_token(&first), "first", "前の便は待たない: {}", stdout_of(&first));
    let moved = git(&repo, &["rev-parse", "refs/heads/main"]);

    let out = waiting.wait_with_output().expect("待っていた land が終わる");
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "待っていた便も land する: {}", stderr_of(&out));
    let token = order_token(&out);
    let waited: u64 = token.strip_prefix("waited:").and_then(|secs| secs.parse().ok()).unwrap_or(u64::MAX);
    assert!(waited < 30, "上限を待たずに進んだ（order={token}）: {}", stdout_of(&out));
    assert_eq!(exported_order(&state, &id_b), token, "面 5 の record も同じ値");
    assert_eq!(follow_count(&state, &id_b), 1, "追随は 1 回: {:?}", stages(&state, &id_b));
    assert_eq!(gate_count(&state, &id_b), 2, "gate は初回 + 撃ち直し 1 回: {:?}", stages(&state, &id_b));
    let new = git(&repo, &["rev-parse", "refs/heads/main"]);
    assert_eq!(git(&repo, &["rev-parse", &format!("{new}^")]), moved, "前の便の上に載る");
    assert!(show_line(&repo, &state, &id_b).contains("stage=Landed"), "段は Landed");
    clean(&[&repo, &state]);
}

/// 前の便の worktree が在らない（retire 済み＝move 済み）なら列から外れ、後の便は待たない。
#[test]
fn pipe_order_front_run_without_worktree_leaves_the_queue() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let (id_a, id_b) = two_gated_runs(&repo, &state, &marker);
    let rules = write_rules_land_wait(&state, "rules-order.toml", Some(30));
    let front = worktree_of(&repo, &id_a);
    let retired = repo.join(".worktrees").join("scribe2").join("retired").join(&id_a);
    fs::create_dir_all(retired.parent().unwrap_or(&repo)).expect("retired の親を作れる");
    git(&repo, &["worktree", "move", &front.display().to_string(), &retired.display().to_string()]);
    let out = land_extra(&repo, &state, &id_b, &["--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "land: {}", stderr_of(&out));
    assert_eq!(order_token(&out), "first", "列の前が空: {}", stdout_of(&out));
    assert!(show_line(&repo, &state, &id_b).contains("stage=Landed"), "段は Landed");
    clean(&[&repo, &state]);
}

/// 3 便（Gated の ts 順 a < b < c）: a を land した後に b と c の land を背景で撃つと、b が追随 →
/// 撃ち直しの間も b は列の先頭に残り c は待つ（`order=waited:<n>`）。追随は b・c とも **1 回ずつ**
/// （2 回の便が 0＝c が (vi) の `stale base` を踏まない）で、main には a → b → c の順に載る
/// （lens の指摘 2026-09-13T04:05Z の形・撃ち直し中の便が列から外れると c が b と並行に撃ち直す）。
#[test]
fn pipe_order_three_runs_follow_once_each_and_land_in_gated_order() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let (id_a, id_b, id_c) = three_gated_runs(&repo, &state, &marker);
    let rules = write_rules_land_wait(&state, "rules-order.toml", Some(30));
    let lens = fake_lens(&marker, &lens_verdict("PASS"));
    let first = land_extra(&repo, &state, &id_a, &["--rules", &rules]);
    assert_eq!(first.status.code(), Some(i32::from(RC_OK)), "a の land: {}", stderr_of(&first));
    assert_eq!(order_token(&first), "first", "a は待たない: {}", stdout_of(&first));
    let second = land_in_background(&repo, &state, &id_b, &rules, &lens);
    let third = land_in_background(&repo, &state, &id_c, &rules, &lens);
    let out_b = second.wait_with_output().expect("b の land が終わる");
    let out_c = third.wait_with_output().expect("c の land が終わる");
    assert_eq!(out_b.status.code(), Some(i32::from(RC_OK)), "b の land: {}", stderr_of(&out_b));
    assert_eq!(out_c.status.code(), Some(i32::from(RC_OK)), "c の land: {}", stderr_of(&out_c));
    assert_eq!(order_token(&out_b), "first", "a の着地後の b は列の先頭: {}", stdout_of(&out_b));
    let token = order_token(&out_c);
    let waited: u64 = token.strip_prefix("waited:").and_then(|secs| secs.parse().ok()).unwrap_or(u64::MAX);
    assert!(waited < 30, "c は b が列を空けるまで待ち、上限は待たない（order={token}）: {}", stdout_of(&out_c));
    assert_eq!(exported_order(&state, &id_c), token, "面 5 の record も同じ値");
    assert_eq!(follow_count(&state, &id_b), 1, "b の追随は 1 回: {:?}", stages(&state, &id_b));
    assert_eq!(follow_count(&state, &id_c), 1, "c の追随は 1 回: {:?}", stages(&state, &id_c));
    let (sha_a, sha_b, sha_c) = (landed_token(&first), landed_token(&out_b), landed_token(&out_c));
    assert_eq!(git(&repo, &["rev-parse", &format!("{sha_b}^")]), sha_a, "b は a の上に載る");
    assert_eq!(git(&repo, &["rev-parse", &format!("{sha_c}^")]), sha_b, "c は b の上に載る");
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), sha_c, "main の先頭は c");
    clean(&[&repo, &state]);
}

/// 撃ち直しが FAIL になった便（`Gated` のまま verdict が FAIL）は列から外れ、後続は待たずに進む。
/// 「待たなかった」は**待ちの record** で pin する（設計 gate-cost.md §23 形 (1)）: stdout の `order=first` と
/// 面 5（`verdicts.jsonl`）の `order` = `first`（待った周は `waited:<s>`）。壁時計は測らない——負荷下では
/// land 自体（rebase + 再 gate + 主実測）が上限を超えて偽に落ちる。
#[test]
fn pipe_order_regate_fail_leaves_the_queue() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let (id_a, id_b, id_c) = three_gated_runs(&repo, &state, &marker);
    let rules = write_rules_land_wait(&state, "rules-order.toml", Some(30));
    let first = land_extra(&repo, &state, &id_a, &["--rules", &rules]);
    assert_eq!(first.status.code(), Some(i32::from(RC_OK)), "a の land: {}", stderr_of(&first));
    let fail = fake_lens(&marker, &lens_verdict("FAIL"));
    let failed = land_extra(&repo, &state, &id_b, &["--rules", &rules, "--lens", &fail]);
    assert_ne!(failed.status.code(), Some(i32::from(RC_OK)), "撃ち直しが FAIL なら land しない: {}", stdout_of(&failed));
    assert_eq!(follow_count(&state, &id_b), 1, "b は追随して撃ち直した: {:?}", stages(&state, &id_b));
    assert!(show_line(&repo, &state, &id_b).contains("stage=Gated"), "b は Gated(FAIL) のまま");
    let pass = fake_lens(&marker, &lens_verdict("PASS"));
    let out = land_extra(&repo, &state, &id_c, &["--rules", &rules, "--lens", &pass]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "c の land: {}", stderr_of(&out));
    assert_eq!(order_token(&out), "first", "FAIL の b は列に居ない: {}", stdout_of(&out));
    assert_eq!(exported_order(&state, &id_c), "first", "面 5 の record も `first`（待った周は waited:<s>）");
    assert!(show_line(&repo, &state, &id_c).contains("stage=Landed"), "c は Landed");
    clean(&[&repo, &state]);
}

/// `--pr-cmd` の形は列を見ない（main を動かさない）: 前の便が列に居ても待たず、`order=` を出さない。
/// 列を見ない形は面 5 へも `order` を書かないので、「待たなかった」の pin は stdout に `order=` が無いこと
/// だけである（設計 gate-cost.md §23 形 (1)・記録を書かない面に空文字の pin を置いても RED を作れない）。
/// 壁時計は測らない（負荷下で偽に落ちる）。
#[test]
fn pipe_order_pr_cmd_does_not_look_at_the_queue() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let (_id_a, id_b) = two_gated_runs(&repo, &state, &marker);
    let rules = write_rules_land_wait(&state, "rules-order.toml", Some(30));
    let main = git(&repo, &["rev-parse", "refs/heads/main"]);
    let out = land_extra(&repo, &state, &id_b, &["--rules", &rules, "--pr-cmd", "true"]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "PR の口: {}", stderr_of(&out));
    assert!(stdout_of(&out).contains("landed=pr"), "{}", stdout_of(&out));
    assert!(!stdout_of(&out).contains("order="), "列を見ない形は order= を出さない: {}", stdout_of(&out));
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), main, "main は動かない");
    clean(&[&repo, &state]);
}

/// 列を導けない周（worktree 在りの `Gated` の便の判定が読めない）は `order=unmeasured` で**進む**
/// （rc 2 にしない＝待ちは deny の関門でない・main 実測の「測れなかった」とは別の極性）。
#[test]
fn pipe_order_unreadable_front_verdict_is_unmeasured_and_lands() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let (id_a, id_b) = two_gated_runs(&repo, &state, &marker);
    let rules = write_rules_land_wait(&state, "rules-order.toml", Some(30));
    fs::write(state.join("pipe").join(&id_a).join("verdict.json"), "{broken\n").expect("判定を壊せる");
    let out = land_extra(&repo, &state, &id_b, &["--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "読めない周も進む: {}", stderr_of(&out));
    assert_eq!(order_token(&out), "unmeasured", "stdout の land 行: {}", stdout_of(&out));
    assert_eq!(exported_order(&state, &id_b), "unmeasured", "面 5 の record");
    assert!(show_line(&repo, &state, &id_b).contains("stage=Landed"), "段は Landed");
    clean(&[&repo, &state]);
}

/// 先頭の便の札が死んだ pid（設計 pipeline.md §36）: 後続の land は待たずに `first` で進み、stdout の `order=` の直後に
/// `skipped-dead=1`、面 5 の `skipped_dead` に先頭の便 id を載せる。外すだけで先頭の段と worktree は動かない。
#[test]
fn pipe_order_dead_front_driver_is_skipped_and_named() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let (id_a, id_b) = two_gated_runs(&repo, &state, &marker);
    put_driver_pid(&state, &id_a, dead_pid());
    let rules = write_rules_land_wait(&state, "rules-order.toml", Some(30));
    let out = land_extra(&repo, &state, &id_b, &["--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "land: {}", stderr_of(&out));
    assert_eq!(order_token(&out), "first", "死んだ先頭は数えない: {}", stdout_of(&out));
    assert!(stdout_of(&out).contains(" order=first skipped-dead=1 "), "order= の直後に外した本数: {}", stdout_of(&out));
    assert_eq!(exported_order(&state, &id_b), "first", "面 5 の order");
    assert_eq!(exported_skipped_dead(&state, &id_b), id_a, "面 5 の skipped_dead は外した便 id");
    assert!(show_line(&repo, &state, &id_b).contains("stage=Landed"), "後続は Landed");
    assert!(show_line(&repo, &state, &id_a).contains("stage=Gated"), "死んだ便の段は動かない");
    assert!(worktree_of(&repo, &id_a).is_dir(), "死んだ便の worktree は残る");
    clean(&[&repo, &state]);
}

/// 先頭の便の札が生きている（所有者 = この test の process）周は従来どおり待つ（上限 1 秒で `degraded`・外した便は
/// 名指さない）。
#[test]
fn pipe_order_dead_live_front_driver_still_waits() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let (id_a, id_b) = two_gated_runs(&repo, &state, &marker);
    put_driver_pid(&state, &id_a, std::process::id());
    let rules = write_rules_land_wait(&state, "rules-order.toml", Some(LAND_WAIT_S));
    let out = land_extra(&repo, &state, &id_b, &["--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "land: {}", stderr_of(&out));
    assert_eq!(order_token(&out), "degraded", "生きている先頭を待つ: {}", stdout_of(&out));
    assert!(!stdout_of(&out).contains("skipped-dead="), "外した便は無い: {}", stdout_of(&out));
    assert_eq!(exported_skipped_dead(&state, &id_b), "", "面 5 に skipped_dead を書かない");
    assert!(show_line(&repo, &state, &id_a).contains("stage=Gated"), "先頭は列に残ったまま");
    clean(&[&repo, &state]);
}

/// 先頭の便に札が無い（`pipe spawn` で起こした便）周も従来どおり待つ（札の無いを「死んだ」に読み替えない）。
#[test]
fn pipe_order_dead_absent_front_driver_still_waits() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let (id_a, id_b) = two_gated_runs(&repo, &state, &marker);
    assert!(!state.join("pipe").join(&id_a).join("driver").exists(), "前提: 先頭は札を持たない");
    let rules = write_rules_land_wait(&state, "rules-order.toml", Some(LAND_WAIT_S));
    let out = land_extra(&repo, &state, &id_b, &["--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "land: {}", stderr_of(&out));
    assert_eq!(order_token(&out), "degraded", "札の無い先頭を待つ: {}", stdout_of(&out));
    assert!(!stdout_of(&out).contains("skipped-dead="), "外した便は無い: {}", stdout_of(&out));
    assert_eq!(exported_skipped_dead(&state, &id_b), "", "面 5 に skipped_dead を書かない");
    clean(&[&repo, &state]);
}

/// `pipe.land_wait_s` の行が無い manifest は land を 1 byte も動かさない（rc 2・event 0 増・main 不変）。
#[test]
fn pipe_order_missing_land_wait_row_moves_nothing() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let path = write_contract(&repo, &[], &[]);
    let id = gated_pass(&repo, &state, &path, &marker);
    let rules = write_rules_land_wait(&state, "rules-order.toml", None);
    let main = git(&repo, &["rev-parse", "refs/heads/main"]);
    let before = event_count(&state);
    let out = land_extra(&repo, &state, &id, &["--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_BROKEN)), "行の欠落は rc 2: {}", stdout_of(&out));
    assert!(stderr_of(&out).contains("pipe.land_wait_s が無い"), "行を名指す: {}", stderr_of(&out));
    assert_eq!(event_count(&state), before, "event を 1 件も書かない");
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), main, "main は動かない");
    clean(&[&repo, &state]);
}

/// (設計 pipeline.md 行 ap) 運転手の cwd が**消えた dir**でも、台帳の close は repo を cwd にして撃たれ
/// `terminal:close:ok` まで進む。
///
/// 偽 bd は本物と同じく**台帳を cwd から探す**（`.vessel.toml` が cwd に無ければ rc 1 で断る）うえで cwd を
/// 記録に残す。運転手の cwd を継ぐ実装では、消えた dir から撃たれて `close:failed` で止まる。
#[test]
fn pipe_terminal_land_close_cwd_survives_a_vanished_driver_cwd() {
    let (repo, state) = repo_with_state();
    let _tools = fake_terminal(&repo, &state, "success");
    let cwd_log = state.join("bd-cwd.txt");
    let bd = exec_script(
        &state.join("fake-bd-cwd.sh"),
        &format!(
            "pwd -P > '{}' 2>&1\ntest -e .vessel.toml || {{ echo 'no ledger found from cwd' >&2; exit 1; }}\n",
            cwd_log.display()
        ),
    );
    let marker = state.join("lens-ran");
    let design = write_contract(&repo, &[], &[]);
    let id = gated_pass(&repo, &state, &design, &marker);
    let rules = ceiling_rules(&state);
    let gone = state.join("vanishing-cwd");
    fs::create_dir_all(&gone).expect("消える dir を作れる");
    let (repo_arg, state_arg) = (repo.display().to_string(), state.display().to_string());
    let args = [
        "land", "--run", id.as_str(), "--repo", repo_arg.as_str(), "--state-dir", state_arg.as_str(),
        "--bd", bd.as_str(), "--rules", rules.as_str(),
    ];
    let mut cmd = pipe_cmd(&args);
    // **後置の cwd が勝つ**（[`bin_cmd`] の tmp dir を上書き）。起こした直後に dir を消す＝子の cwd は消えた dir。
    let child = cmd.current_dir(&gone).spawn().expect("binary を起動できる");
    fs::remove_dir(&gone).expect("子の cwd を消せる");
    assert!(!gone.exists(), "fixture: 子の cwd は消えている");
    let out = child.wait_with_output().expect("子の終わりを待てる");
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "終端まで通った land は rc 0: {}", stderr_of(&out));
    assert_eq!(terminal_lines(&state, &id), [PUSHED, CLOSED], "消えた cwd からでも close まで進む");
    let written = fs::read_to_string(&cwd_log).expect("偽 bd が撃たれた");
    assert_eq!(written.trim_end(), repo.display().to_string(), "偽 bd の cwd は repo");
    clean(&[&repo, &state]);
}

/// 台帳 client が rc 1 で断った close の行。
const CLOSE_REFUSED: &str = "terminal:close:failed:rc=1";

/// 撃ち直しの歯の fixture（設計 contract-source.md §58・行 bm）: 台帳 client が断る着地（push の後の close で止まる）の後に
/// 台帳 client を直す（直した偽 bd は argv を写す）。返すのは道具・便・`--bd` と `--rules` の値・着地した sha。
fn replay_fixture(repo: &Path, state: &Path) -> (FakeTerminal, String, [String; 2], String) {
    let tools = fake_terminal(repo, state, "success");
    let design = write_contract(repo, &[], &[]);
    let id = gated_pass(repo, state, &design, &state.join("lens-ran"));
    let flags = [exec_script(&state.join("fake-bd.sh"), "exit 1\n"), ceiling_rules(state)];
    let first = land_extra(repo, state, &id, &["--bd", &flags[0], "--rules", &flags[1]]);
    assert_eq!(first.status.code(), Some(1), "1 周目は close しない: {}", stderr_of(&first));
    assert_eq!(terminal_lines(state, &id), [PUSHED, CLOSE_REFUSED], "前提: close で止まり子を起こさない");
    assert_eq!(tools.ci_call_count(), 0, "前提: 偽 CI はまだ撃たれていない");
    let landed = git(repo, &["rev-parse", "refs/heads/main"]);
    // 台帳 client を直す（宣言は同じ path を指したまま）。
    exec_script(&state.join("fake-bd.sh"), &format!("printf '%s\\n' \"$@\" > '{}'\n", tools.bd_log.display()));
    (tools, id, flags, landed)
}

/// 終端だけを撃ち直す（`pipe land --terminal-only`・`--bd` と `--rules` は fixture の値）。
fn replay(repo: &Path, state: &Path, id: &str, flags: &[String; 2]) -> Output {
    land_extra(repo, state, id, &["--bd", &flags[0], "--rules", &flags[1], "--terminal-only"])
}

/// anchor の main と偽 remote の main を `sha` へ付け替える（remote の ref は bare repo の中で直に動かす）。
fn point_main(repo: &Path, tools: &FakeTerminal, sha: &str) {
    git(&tools.remote, &["update-ref", "refs/heads/main", sha]);
    git(repo, &["update-ref", "refs/heads/main", sha]);
}

/// 終端だけの撃ち直しは子を起こさず close し、照らす commit を選ばない（行 v-ci-child-cut・設計 contract-source.md §58・§65）:
/// close で止まった便の撃ち直しを 2 周撃つ。(1) main が着地した sha から別の commit へ進んだ周、(2) main が本文に run の行を持つ
/// 写しの上の commit へ付け替わった周は、どちらも rc 0 と 1 行 `run=<id> terminal=closed`・close の理由 `landed <着地した sha>
/// host=green`（(2) は `landed <写しの sha> host=green`）・終端の行は 1 周目の push と close の拒みに push と close の 2 件を足した
/// 4 件だけ・偽 CI の呼びは 0 回。
#[test]
fn vcicut_terminal_only_closes_without_a_ci_child() {
    let (repo, state) = repo_with_state();
    let (tools, id, flags, landed) = replay_fixture(&repo, &state);
    fs::write(repo.join("unrelated.md"), "別の便\n").expect("別の便の file を書ける");
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "another-run"]);
    let moved = git(&repo, &["rev-parse", "refs/heads/main"]);
    assert_ne!(moved, landed, "前提: main は着地した sha から動いた");
    let again = replay(&repo, &state, &id, &flags);
    assert_eq!(again.status.code(), Some(i32::from(RC_OK)), "継いだ終端は rc 0: {}", stderr_of(&again));
    assert_eq!(stdout_of(&again).trim(), format!("run={id} terminal=closed"), "終端だけの 1 行");
    let argv = fs::read_to_string(&tools.bd_log).expect("2 周目で台帳が閉じられた");
    let reason = format!("landed {landed} host=green");
    assert_eq!(argv.lines().nth(3), Some(reason.as_str()), "理由は着地した sha と host の緑: {argv}");
    assert_eq!(terminal_lines(&state, &id), [PUSHED, CLOSE_REFUSED, PUSHED, CLOSED], "撃ち直しは push と close の 2 件だけを足す");
    assert_eq!(tools.ci_call_count(), 0, "偽 CI は撃たれない");
    clean(&[&repo, &state]);

    let (repo, state) = repo_with_state();
    let (tools, id, flags, landed) = replay_fixture(&repo, &state);
    let body = git(&repo, &["log", "-n", "1", "--format=%B", &landed]);
    assert!(body.contains(&format!("run: {id}")), "前提: 本文に run の行が在る: {body}");
    let tree = format!("{landed}^{{tree}}");
    let other = git(&repo, &["commit-tree", &tree, "-p", &format!("{landed}^"), "-m", "another-run"]);
    let (copy, top) = relocated(&repo, &landed, &other, &body);
    assert_ne!(copy, landed, "前提: 写しは着地した commit と別の sha");
    point_main(&repo, &tools, &top);
    let own = replay(&repo, &state, &id, &flags);
    assert_eq!(own.status.code(), Some(i32::from(RC_OK)), "写しを見つけた周は close する: {}", stderr_of(&own));
    assert_eq!(stdout_of(&own).trim(), format!("run={id} terminal=closed"), "終端の 1 行");
    let argv = fs::read_to_string(&tools.bd_log).expect("2 周目で台帳が閉じられた");
    let reason = format!("landed {copy} host=green");
    assert_eq!(argv.lines().nth(3), Some(reason.as_str()), "理由は写しの sha と host の緑: {argv}");
    assert_eq!(terminal_lines(&state, &id), [PUSHED, CLOSE_REFUSED, PUSHED, CLOSED], "撃ち直しは push と close の 2 件だけを足す");
    assert_eq!(tools.ci_call_count(), 0, "偽 CI は撃たれない");
    clean(&[&repo, &state]);
}

/// (§58 形 1) `refs/heads/main` を読めない撃ち直しは先端の側に倒さず、何も書かず何も撃たずに断る（rc 1・stderr 1 行）。
/// base は終端へ進んで `terminal:unreadable` を記す＝RED。
#[test]
fn pipe_replay_tip_unreadable_main_refuses_without_events() {
    let (repo, state) = repo_with_state();
    let (tools, id, flags, _) = replay_fixture(&repo, &state);
    git(&repo, &["update-ref", "-d", "refs/heads/main"]);
    let (events, calls) = (event_count(&state), tools.ci_call_count());
    let out = replay(&repo, &state, &id, &flags);
    assert_eq!(out.status.code(), Some(1), "断りは rc 1: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "", "stdout は空");
    assert!(stderr_of(&out).contains("refs/heads/main を読めない"), "断りの 1 行: {}", stderr_of(&out));
    assert_eq!(event_count(&state), events, "event を 1 件も書かない");
    assert_eq!(tools.ci_call_count(), calls, "偽 CI は撃たれない");
    assert!(!tools.bd_log.exists(), "偽 bd は撃たれない");
    clean(&[&repo, &state]);
}

/// 撃ち直しの写しの fixture（設計 §65）: 着地した commit の親の上に別の便の commit を積み、その上に着地した commit の木と
/// `body` で写しの commit を作り、写しの上に 1 commit を置く。偽 remote へは別の ref で押す（main はまだ動かさない・force の
/// push は使わない）。返すのは（写しの sha・写しの上の commit の sha）。
fn relocated(repo: &Path, landed: &str, other: &str, body: &str) -> (String, String) {
    let tree = format!("{landed}^{{tree}}");
    let copy = git(repo, &["commit-tree", &tree, "-p", other, "-m", body]);
    let top = git(repo, &["commit-tree", &tree, "-p", &copy, "-m", "top"]);
    git(repo, &["push", "-q", "fake", &format!("{top}:refs/heads/relocated-{top}")]);
    (copy, top)
}

/// (§65 形 2) 本文を字のまま写した写しそのものが main の先端の撃ち直しは、`landed <写しの sha> host=green` で close する
/// （終端は子を起こさず、偽 CI は撃たれない）。
#[test]
fn pipe_replay_relocate_copy_at_the_tip_closes_without_tip() {
    let (repo, state) = repo_with_state();
    let (tools, id, flags, landed) = replay_fixture(&repo, &state);
    let body = git(&repo, &["log", "-n", "1", "--format=%B", &landed]);
    let tree = format!("{landed}^{{tree}}");
    let other = git(&repo, &["commit-tree", &tree, "-p", &format!("{landed}^"), "-m", "another-run"]);
    let (copy, top) = relocated(&repo, &landed, &other, &body);
    git(&repo, &["push", "-q", "fake", &format!("{copy}:refs/heads/relocated-{copy}")]);
    assert_ne!(top, copy, "前提: 写しの上の commit は別");
    point_main(&repo, &tools, &copy);
    let out = replay(&repo, &state, &id, &flags);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "写しが先端の周は close する: {}", stderr_of(&out));
    assert_eq!(tools.ci_call_count(), 0, "偽 CI は撃たれない");
    let argv = fs::read_to_string(&tools.bd_log).expect("台帳が閉じられた");
    let reason = format!("landed {copy} host=green");
    assert_eq!(argv.lines().nth(3), Some(reason.as_str()), "reason は tip= を持たない: {argv}");
    assert_eq!(terminal_lines(&state, &id), [PUSHED, CLOSE_REFUSED, PUSHED, CLOSED], "Landed の後ろの終端の行は 4 件だけ");
    clean(&[&repo, &state]);
}

/// (§5 手順 4) record の `generation` は **binary の build 元 commit**（`--version` の括弧の中身と同じ 1 本）で、
/// 同じ行の `sha`（着地した commit）とは**別の値**である。
///
/// 同値の欄を 2 つ並べると、読み手はどちらを版の比較（§12）に使うのか判じられない（C10）。
#[test]
fn pipe_terminal_land_generation_is_the_binary_build_commit_not_the_landed_sha() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let design = write_contract(&repo, &[], &[]);
    let id = gated_pass(&repo, &state, &design, &marker);
    let out = land_extra(&repo, &state, &id, &[]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "land は rc 0: {}", stderr_of(&out));
    let pairs = exported_pairs(&state, &id);
    let landed = git(&repo, &["rev-parse", "refs/heads/main"]);
    assert_eq!(value_of(&pairs, "sha"), landed, "sha は着地した commit: {pairs:?}");
    // `--version` の括弧の中身を現物から採る（器の字面を借りずに外形から測る）。
    let version = String::from_utf8_lossy(&bin_cmd().arg("--version").output().expect("--version").stdout)
        .trim()
        .to_owned();
    let generation = version
        .rsplit_once('(')
        .and_then(|(_, tail)| tail.strip_suffix(')'))
        .unwrap_or_default()
        .to_owned();
    assert!(!generation.is_empty(), "--version の括弧の中身を読める: {version}");
    assert_eq!(value_of(&pairs, "generation"), generation, "generation は build 元 commit: {pairs:?}");
    assert_ne!(value_of(&pairs, "generation"), value_of(&pairs, "sha"), "同値の欄を 2 つ並べない: {pairs:?}");
    clean(&[&repo, &state]);
}

/// (§50 形 1) `pipe.ci_poll_s` の行が無い manifest は `pipe.ci_wait_s` と同じ極性で断る（rc 2・行を名指す・event 0 増・
/// main 不変）。
#[test]
fn pipe_terminal_ci_poll_missing_row_moves_nothing() {
    let (repo, state) = repo_with_state();
    let design = write_contract(&repo, &[], &[]);
    let id = gated_pass(&repo, &state, &design, &state.join("lens-ran"));
    let rules = write_rules_ci(&state, CI_WAIT_S, None);
    let main = git(&repo, &["rev-parse", "refs/heads/main"]);
    let before = event_count(&state);
    let out = land_extra(&repo, &state, &id, &["--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_BROKEN)), "行の欠落は rc 2: {}", stdout_of(&out));
    assert!(stderr_of(&out).contains("pipe.ci_poll_s が無い"), "行を名指す: {}", stderr_of(&out));
    assert_eq!(event_count(&state), before, "event を 1 件も書かない");
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), main, "main は動かない");
    clean(&[&repo, &state]);
}
