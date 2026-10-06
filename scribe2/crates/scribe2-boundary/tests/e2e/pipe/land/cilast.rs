//! 着地の終端の CI の照合の歯（接頭辞 `vcil_` と `vclhost_` と `vcioff_`・器の memo t3-hub.74.49.6・判断の記録 ADR-45 の門 H6・台帳の問い
//! t3-hub.90.2 の裁定・親 `tests/e2e/pipe/land.rs` の helper を `use super::*` で使う）。
//!
//! 終端は push の後にこの host の緑で台帳を閉じ、GitHub の検査は着地の後の CI の口（`pipe land --ci-only`）の子 process が後から
//! 読む。子は待ちが最後に読んだ答えで記して読み直さないこと（道 1）と、push が押した commit（remote の追跡の ref）が自分の sha の
//! 子孫で自分の sha でない周はその commit が子へ渡ること（道 2）を、偽 remote・偽 CI・偽 bd と、push の前後で main を動かす偽 git
//! の外形から測る。子 process は誰も待たないので、歯は片付けの前に子の答えの行を待つ（[`settled_rows`]）。

use super::*;

/// 完了して success の CI の答え（偽 CI が返す 1 行）。
const PASSED: &str = r#"[{"status":"completed","conclusion":"success"}]"#;

/// 偽 CI を差し替える: argv を log へ写し、呼ばれた回数の file に 1 行を足してから、`answer`（sh の断片）で答える。
fn ci_answer(tools: &FakeTerminal, state: &Path, answer: &str) {
    let (log, calls) = (tools.ci_log.display(), tools.ci_calls.display());
    exec_script(&state.join("fake-ci.sh"), &format!("printf '%s\\n' \"$@\" > '{log}'\necho call >> '{calls}'\n{answer}\n"));
}

/// 偽 remote の main を指す sha だけに success を返し、ほかの sha には run が無い（`[]`）と答える偽 CI（forge の CI は
/// push の先端にだけ run を持つ）。
fn ci_only_on_the_remote_main(tools: &FakeTerminal, state: &Path) {
    let remote = tools.remote.display();
    let answer = format!(
        "head=$(git --git-dir '{remote}' rev-parse refs/heads/main 2>/dev/null)\n\
         if [ \"$1\" = \"$head\" ]; then printf '%s\\n' '{PASSED}'; else printf '%s\\n' '[]'; fi"
    );
    ci_answer(tools, state, &answer);
}

/// 実の git の path（偽 git の中から自分を呼ばずに push を撃つため）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn real_git() -> String {
    let out = Command::new("sh").args(["-c", "command -v git"]).output().expect("git を引ける");
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

/// main を `parent` の上の新しい commit（木は今の main と同じ）へ進め、その sha を `<state>/raced.txt` に書く sh の断片
/// （偽 git の中で撃つ・`"$2"` は器が `-C` に渡した repo）。
fn advance_main(state: &Path, parent: &str) -> String {
    format!(
        "sha=$(git -C \"$2\" rev-parse refs/heads/main)\n\
         next=$(git -C \"$2\" commit-tree \"$sha^{{tree}}\" -p {parent} -m raced)\n\
         git -C \"$2\" update-ref refs/heads/main \"$next\" \"$sha\"\n\
         printf '%s\\n' \"$next\" > '{}'",
        state.join("raced.txt").display()
    )
}

/// Gated(PASS) の便を、終端の push の前（`before` が真）か後に main を動かす偽 git の下で着地させる。返すのは land の出力・
/// 便の id・道具一式。偽 CI は偽 remote の main だけに success を返す。
fn land_with_push_race(repo: &Path, state: &Path, parent: &str, before: bool) -> (Output, String, FakeTerminal) {
    let tools = fake_terminal_json(repo, state, "[]");
    ci_only_on_the_remote_main(&tools, state);
    let design = write_contract(repo, &[], &[]);
    let id = gated_pass(repo, state, &design, &state.join("lens-ran"));
    let race = advance_main(state, parent);
    let script = if before {
        format!("case \"$*\" in *' push fake main:main') {race} ;; esac")
    } else {
        format!("case \"$*\" in *' push fake main:main') '{}' \"$@\" || exit $?\n{race}\nexit 0 ;; esac", real_git())
    };
    let path = shim_path(state, "race-bin", &script);
    let (bd, rules) = (state.join("fake-bd.sh").display().to_string(), ceiling_rules(state));
    let (repo_arg, state_arg) = (repo.display().to_string(), state.display().to_string());
    let args = ["land", "--run", &id, "--repo", &repo_arg, "--state-dir", &state_arg, "--bd", &bd, "--rules", &rules];
    (run_pipe_with_path(&path, &args), id, tools)
}

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

/// 偽 git が書いた、動かした後の main の sha。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn raced(state: &Path) -> String {
    fs::read_to_string(state.join("raced.txt")).expect("偽 git が main を動かした").trim().to_owned()
}

/// 終端の親が記す行（push・close・着地の後の CI の口の子 process を起こした印）。
pub(super) const PUSHED: &str = "terminal:push:fake";
/// 台帳の close が通った行。
pub(super) const CLOSED: &str = "terminal:close:ok";
/// 子 process を起こした行。
pub(super) const SPAWNED: &str = "terminal:ci:spawned";

/// CI の答えの行（閉じた 3 値・子 process が記すか、照合する commit の無い周に親が記す）。
const CI_ANSWERS: [&str; 3] = ["terminal:ci:success", "terminal:ci:failure", "terminal:ci:unmeasurable"];

/// 子 process の終わりを待つ上限（秒）。
const CHILD_WAIT_S: u64 = 60;

/// 便の終端の行のうち CI の答えの行が `n` 件に届くまで待ち（上限 [`CHILD_WAIT_S`]）、終端の行を返す。子 process は誰も待たない
/// ので、歯は片付けの前にここで子の答えを待つ（片付けの後に子が置き場を作り直さない）。
pub(super) fn settled_rows(state: &Path, id: &str, n: usize) -> Vec<String> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(CHILD_WAIT_S);
    loop {
        let rows = terminal_rows(state, id);
        if rows.iter().filter(|row| CI_ANSWERS.contains(&row.as_str())).count() >= n || std::time::Instant::now() >= deadline {
            return rows;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// 終端の行から子の答え `answer` の最後の 1 件を外した列（親の記帳の順）。子の追記は親の `ci:spawned` の記帳と競うので、答えの
/// 位置は最後の close の後であることだけを断言する。
pub(super) fn parent_rows(rows: &[String], answer: &str) -> Vec<String> {
    let at = rows.iter().rposition(|row| row == answer);
    let closed = rows.iter().rposition(|row| row == CLOSED);
    assert!(at.is_some_and(|found| closed.is_some_and(|close| close < found)), "子の答え {answer} は最後の close の後: {rows:?}");
    rows.iter().enumerate().filter(|(index, _)| Some(*index) != at).map(|(_, row)| row.clone()).collect()
}

/// 着地の後の CI の口を直に撃つ（`pipe land --run <id> --ci-only <sha>`・`--bd` は偽 bd・`--rules` は呼び手の値）。
pub(super) fn ci_only(repo: &Path, state: &Path, id: &str, sha: &str, rules: &str) -> Output {
    let bd = state.join("fake-bd.sh").display().to_string();
    land_extra(repo, state, id, &["--bd", &bd, "--rules", rules, "--ci-only", sha])
}

/// 口の stdout の 1 行目の `ci=` の値（無ければ空）。
pub(super) fn answer_of(out: &Output) -> String {
    let stdout = stdout_of(out);
    stdout.lines().next().and_then(|line| line.split_whitespace().find_map(|word| word.strip_prefix("ci="))).unwrap_or_default().to_owned()
}

/// 着地の後の CI の口の歯の fixture: success の偽 CI の便を着地させ（終端は閉じて子を起こす）、子の答えを待ってから偽 CI を
/// `json` の答えに差し替え、呼ばれた回数の file と偽 bd の log を消す。返すのは道具・便 id・着地した sha。
pub(super) fn landed_for_ci(repo: &Path, state: &Path, json: &str) -> (FakeTerminal, String, String) {
    let tools = fake_terminal_json(repo, state, PASSED);
    let design = write_contract(repo, &[], &[]);
    let id = gated_pass(repo, state, &design, &state.join("lens-ran"));
    let (bd, rules) = (state.join("fake-bd.sh").display().to_string(), ceiling_rules(state));
    let out = land_extra(repo, state, &id, &["--bd", &bd, "--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "前提: 着地して閉じた: {} {}", stdout_of(&out), stderr_of(&out));
    settled_rows(state, &id, 1);
    ci_answer(&tools, state, &format!("printf '%s\\n' '{json}'"));
    fs::remove_file(&tools.ci_calls).ok();
    fs::remove_file(&tools.bd_log).ok();
    let sha = landed_sha(state, &id);
    (tools, id, sha)
}

/// (道 1) 待ちが満ちた周に読んだ success で答え、読み直さない: 1 回目は success・2 回目から rc 1 を返す偽 CI の着地は
/// `terminal=closed`（rc 0）で、親の終端の行は push・close:ok・ci:spawned、子の答えは ci:success、偽 CI の呼び出しは 1 回、
/// 台帳の close の理由は `landed <sha> host=green`。読み直す実装は 2 回目の rc 1 で `ci:unmeasurable` を答える。
#[test]
fn vcil_terminal_closes_on_the_read_the_wait_saw() {
    let (repo, state) = repo_with_state();
    let tools = fake_terminal_json(&repo, &state, "[]");
    let calls = tools.ci_calls.display().to_string();
    ci_answer(&tools, &state, &format!("[ \"$(wc -l < '{calls}')\" -gt 1 ] && exit 1\nprintf '%s\\n' '{PASSED}'"));
    let design = write_contract(&repo, &[], &[]);
    let id = gated_pass(&repo, &state, &design, &state.join("lens-ran"));
    let (bd, rules) = (state.join("fake-bd.sh").display().to_string(), ceiling_rules(&state));
    let out = land_extra(&repo, &state, &id, &["--bd", &bd, "--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "閉じた終端は rc 0: {} {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("terminal=closed"), "終端の token: {}", stdout_of(&out));
    let rows = settled_rows(&state, &id, 1);
    assert_eq!(parent_rows(&rows, "terminal:ci:success"), [PUSHED, CLOSED, SPAWNED], "子は満ちた答えを記す: {rows:?}");
    assert_eq!(tools.ci_call_count(), 1, "満ちた後に CI を読み直さない");
    let sha = landed_sha(&state, &id);
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), sha, "main は着地の sha");
    let argv = fs::read_to_string(&tools.bd_log).expect("台帳が閉じられた");
    assert_eq!(argv.lines().nth(3), Some(format!("landed {sha} host=green").as_str()), "理由: {argv}");
    clean(&[&repo, &state]);
}

/// (道 2) push の前に別の commit が main を進め、push がその commit を押した周は、子がその commit の CI を読む:
/// `terminal=closed`（rc 0）・偽 remote の main と子の CI の argv は進めた commit・子の答えは ci:success・close の理由は
/// `landed <自分の sha> host=green`（先端は子へ渡すだけで理由に置かない）。自分の sha（run の無い commit）を渡す実装は子が
/// `ci:unmeasurable` を答える。
#[test]
fn vcil_terminal_checks_the_commit_the_push_carried() {
    let (repo, state) = repo_with_state();
    let (out, id, tools) = land_with_push_race(&repo, &state, "\"$sha\"", true);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "閉じた終端は rc 0: {} {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("terminal=closed"), "終端の token: {}", stdout_of(&out));
    let rows = settled_rows(&state, &id, 1);
    assert_eq!(parent_rows(&rows, "terminal:ci:success"), [PUSHED, CLOSED, SPAWNED], "子は進めた commit の success: {rows:?}");
    let (next, sha) = (raced(&state), landed_sha(&state, &id));
    assert_eq!(git(&repo, &["rev-parse", &format!("{next}^")]), sha, "前提: main は着地の sha の子へ進んだ");
    assert_eq!(git(&tools.remote, &["rev-parse", "refs/heads/main"]), next, "push は進めた commit を押した");
    let ci_argv = fs::read_to_string(&tools.ci_log).expect("偽 CI が撃たれた");
    assert_eq!(ci_argv.lines().collect::<Vec<&str>>(), [next.as_str()], "照合は push が押した commit");
    let argv = fs::read_to_string(&tools.bd_log).expect("台帳が閉じられた");
    assert_eq!(argv.lines().nth(3), Some(format!("landed {sha} host=green").as_str()), "理由: {argv}");
    clean(&[&repo, &state]);
}

/// (道 2 の裏) push の後に main が動いた周は、子が push の押した自分の sha の CI を読む（追跡の ref を読み、local の main を
/// 読まない）: `terminal=closed`（rc 0）・偽 remote の main は自分の sha・local の main は進めた commit・子の CI の argv は自分の
/// sha・子の答えは ci:success・close の理由は `landed <sha> host=green`。
#[test]
fn vcil_terminal_ignores_a_main_moved_after_the_push() {
    let (repo, state) = repo_with_state();
    let (out, id, tools) = land_with_push_race(&repo, &state, "\"$sha\"", false);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "閉じた終端は rc 0: {} {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("terminal=closed"), "終端の token: {}", stdout_of(&out));
    let rows = settled_rows(&state, &id, 1);
    assert_eq!(parent_rows(&rows, "terminal:ci:success"), [PUSHED, CLOSED, SPAWNED], "子は自分の sha の success: {rows:?}");
    let (next, sha) = (raced(&state), landed_sha(&state, &id));
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), next, "前提: local の main は push の後に進んだ");
    assert_eq!(git(&repo, &["rev-parse", &format!("{next}^")]), sha, "前提: 進めた commit は着地の sha の子");
    assert_eq!(git(&tools.remote, &["rev-parse", "refs/heads/main"]), sha, "push が押したのは着地の sha");
    let ci_argv = fs::read_to_string(&tools.ci_log).expect("偽 CI が撃たれた");
    assert_eq!(ci_argv.lines().collect::<Vec<&str>>(), [sha.as_str()], "照合は push が押した自分の sha");
    let argv = fs::read_to_string(&tools.bd_log).expect("台帳が閉じられた");
    assert_eq!(argv.lines().nth(3), Some(format!("landed {sha} host=green").as_str()), "理由: {argv}");
    clean(&[&repo, &state]);
}

/// (道 2 の否定) push が押した commit が自分の sha の子孫でない周（push の前に main を自分の sha の兄弟へ動かした）は、その
/// commit を子へ渡さず、自分の sha を渡す: 終端は host の緑で閉じ（rc 0）、子の CI の argv は自分の sha だけで、run の無い自分の
/// sha の答えは上限で ci:unmeasurable。
#[test]
fn vcil_terminal_keeps_its_own_sha_when_the_pushed_commit_is_not_a_descendant() {
    let (repo, state) = repo_with_state();
    let (out, id, tools) = land_with_push_race(&repo, &state, "\"$sha^\"", true);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "閉じた終端は rc 0: {} {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("terminal=closed"), "終端の token: {}", stdout_of(&out));
    let rows = settled_rows(&state, &id, 1);
    assert_eq!(parent_rows(&rows, "terminal:ci:unmeasurable"), [PUSHED, CLOSED, SPAWNED], "子は自分の sha を測れない: {rows:?}");
    let (next, sha) = (raced(&state), landed_sha(&state, &id));
    assert_eq!(git(&tools.remote, &["rev-parse", "refs/heads/main"]), next, "push は兄弟の commit を押した");
    let ancestor = Command::new("git").arg("-C").arg(&repo).args(["merge-base", "--is-ancestor", &sha, &next]).status();
    assert!(ancestor.is_ok_and(|status| status.code() == Some(1)), "前提: 兄弟の commit は自分の sha の子孫でない");
    let ci_argv = fs::read_to_string(&tools.ci_log).expect("偽 CI が撃たれた");
    assert!(ci_argv.lines().all(|line| line == sha), "照合は自分の sha だけ（兄弟を照合しない）: {ci_argv}");
    clean(&[&repo, &state]);
}

/// 赤を返す偽 CI の答え（完了して failure）。
const FAILED: &str = r#"[{"status":"completed","conclusion":"failure"}]"#;

/// この host の緑で閉じる（台帳の問い t3-hub.90.2 の裁定）: 赤を返す偽 CI の便の land は rc 0 で `terminal=closed`、偽の台帳
/// client の argv は close と便の bead と `--reason` と `landed <着地した sha> host=green`、`Landed` の後ろの親の行は
/// push・close・子の起こしの順で、子の答え `ci:failure` は close の後に 1 件だけ並ぶ（GitHub の検査の答えを待たずに閉じる）。
#[test]
fn vclhost_land_closes_on_host_green_before_a_red_ci_answer() {
    let (repo, state) = repo_with_state();
    let tools = fake_terminal_json(&repo, &state, FAILED);
    let design = write_contract(&repo, &[], &[]);
    let id = gated_pass(&repo, &state, &design, &state.join("lens-ran"));
    let (bd, rules) = (state.join("fake-bd.sh").display().to_string(), ceiling_rules(&state));
    let out = land_extra(&repo, &state, &id, &["--bd", &bd, "--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "赤の CI でも閉じた land は rc 0: {} {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("terminal=closed"), "終端の token: {}", stdout_of(&out));
    let sha = landed_sha(&state, &id);
    let argv = fs::read_to_string(&tools.bd_log).expect("台帳が閉じられた");
    let reason = format!("landed {sha} host=green");
    assert_eq!(argv.lines().collect::<Vec<&str>>(), ["close", "s2-2e5", "--reason", reason.as_str()], "台帳の argv: {argv}");
    let rows = settled_rows(&state, &id, 1);
    assert_eq!(parent_rows(&rows, "terminal:ci:failure"), [PUSHED, CLOSED, SPAWNED], "親の 3 件の順: {rows:?}");
    assert_eq!(rows.iter().filter(|row| row.as_str() == "terminal:ci:failure").count(), 1, "子の答えは 1 件: {rows:?}");
    clean(&[&repo, &state]);
}

/// 宣言が `ci-watch = false` の repo の land は CI を見張らない（行 v-ci-watch-off）: [`vclhost_land_closes_on_host_green_before_a_red_ci_answer`]
/// と同じ fixture（赤を返す偽 CI）で、rc 0・`terminal=closed`・台帳の argv は close と便の bead と `--reason` と
/// `landed <sha> host=green`、便の終端の行は push と close の 2 件だけで、偽 CI は 1 度も撃たれない（子を起こさないので待たない）。
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

/// 便の終端の行 `row` の数。
fn count_of(state: &Path, id: &str, row: &str) -> usize {
    terminal_rows(state, id).iter().filter(|found| found.as_str() == row).count()
}

/// 着地の後の CI の口の赤（設計 contract-source.md §5 手順 3）: 赤を返す偽 CI の周の口は rc 0 で、stdout の 1 行目が
/// `run=<id> ci=failure`・続く行が `notify=` の行で、終端の行 `terminal:ci:failure` が 1 件増え、偽の台帳 client は撃たれない。
#[test]
fn vclhost_ci_only_red_notifies_and_leaves_the_bead() {
    let (repo, state) = repo_with_state();
    let (tools, id, sha) = landed_for_ci(&repo, &state, FAILED);
    let before = count_of(&state, &id, "terminal:ci:failure");
    let out = ci_only(&repo, &state, &id, &sha, &ceiling_rules(&state));
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "口は rc 0: {} {}", stdout_of(&out), stderr_of(&out));
    let stdout = stdout_of(&out);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.first().copied(), Some(format!("run={id} ci=failure").as_str()), "1 行目: {stdout}");
    assert!(lines.get(1).is_some_and(|line| line.starts_with("notify=")), "赤は席へ知らせる: {stdout}");
    assert_eq!(count_of(&state, &id, "terminal:ci:failure"), before + 1, "赤の答えは 1 件増える");
    assert!(!tools.bd_log.exists(), "札は替えない（台帳 client を撃たない）");
    clean(&[&repo, &state]);
}

/// 着地の後の CI の口の緑: success を返す偽 CI の周の口は rc 0 で stdout は `run=<id> ci=success` の 1 行だけ（`notify=` の行を
/// 持たない）、終端の行 `terminal:ci:success` が 1 件増え、偽の台帳 client は撃たれない。
#[test]
fn vclhost_ci_only_green_reads_success_without_a_notice() {
    let (repo, state) = repo_with_state();
    let (tools, id, sha) = landed_for_ci(&repo, &state, PASSED);
    let before = count_of(&state, &id, "terminal:ci:success");
    let out = ci_only(&repo, &state, &id, &sha, &ceiling_rules(&state));
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "口は rc 0: {} {}", stdout_of(&out), stderr_of(&out));
    assert_eq!(stdout_of(&out).lines().collect::<Vec<&str>>(), [format!("run={id} ci=success").as_str()], "緑は 1 行だけ");
    assert_eq!(count_of(&state, &id, "terminal:ci:success"), before + 1, "緑の答えは 1 件増える");
    assert!(!tools.bd_log.exists(), "札は替えない（台帳 client を撃たない）");
    clean(&[&repo, &state]);
}

/// 着地の後の CI の口の値の断り: 12 桁の 16 進（短縮 sha）と、40 桁で 1 字だけ 16 進でない値は rc 1 で、event を 1 件も書かず
/// 偽 CI も撃たない。対照に、同じ便に 40 桁の sha を渡した口は rc 0 で偽 CI を 1 回撃つ。
#[test]
fn vclhost_ci_only_refuses_a_value_that_is_not_40_hex_digits() {
    let (repo, state) = repo_with_state();
    let (tools, id, sha) = landed_for_ci(&repo, &state, PASSED);
    let rules = ceiling_rules(&state);
    let short = sha.chars().take(12).collect::<String>();
    let bent = format!("g{}", sha.chars().skip(1).collect::<String>());
    assert_eq!((short.len(), bent.len()), (12, 40), "前提: 短縮と 40 桁");
    for value in [short, bent] {
        let before = event_count(&state);
        let out = ci_only(&repo, &state, &id, &value, &rules);
        assert_eq!(out.status.code(), Some(1), "40 桁の 16 進でない値は断る: {value} {}", stderr_of(&out));
        assert_eq!(event_count(&state), before, "event を 1 件も書かない: {value}");
        assert_eq!(tools.ci_call_count(), 0, "偽 CI は撃たれない: {value}");
    }
    let out = ci_only(&repo, &state, &id, &sha, &rules);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "対照の 40 桁の sha は読む: {}", stderr_of(&out));
    assert_eq!(tools.ci_call_count(), 1, "対照の口は偽 CI を 1 回撃つ");
    clean(&[&repo, &state]);
}
