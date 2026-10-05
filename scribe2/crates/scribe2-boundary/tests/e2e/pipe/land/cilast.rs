//! 着地の終端の CI の照合の歯（接頭辞 `vcil_`・器の memo t3-hub.74.49.6・判断の記録 ADR-45 の門 H6・親 `tests/e2e/pipe/land.rs` の
//! helper を `use super::*` で使う）。
//!
//! 終端は CI の待ちが最後に読んだ答えで分けて読み直さないこと（道 1）と、push が押した commit（remote の追跡の ref）が自分の
//! sha の子孫で自分の sha でない周はその commit の CI で照合すること（道 2）を、偽 remote・偽 CI・偽 bd と、push の前後で
//! main を動かす偽 git の外形から測る。

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

/// (道 1) 待ちが満ちた周に読んだ success で閉じ、読み直さない: 1 回目は success・2 回目から rc 1 を返す偽 CI の着地は
/// `terminal=closed`（rc 0）で、終端の行は push・ci:success・close:ok、偽 CI の呼び出しは 1 回、台帳の close の理由は
/// `landed <sha> ci=success`。base は待ちの後の読み直しが rc 1 で `ci:unmeasurable` に止まる＝RED。
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
    assert_eq!(terminal_rows(&state, &id), ["terminal:push:fake", "terminal:ci:success", "terminal:close:ok"]);
    assert_eq!(tools.ci_call_count(), 1, "満ちた後に CI を読み直さない");
    let sha = landed_sha(&state, &id);
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), sha, "main は着地の sha");
    let argv = fs::read_to_string(&tools.bd_log).expect("台帳が閉じられた");
    assert_eq!(argv.lines().nth(3), Some(format!("landed {sha} ci=success").as_str()), "理由: {argv}");
    clean(&[&repo, &state]);
}

/// (道 2) push の前に別の commit が main を進め、push がその commit を押した周は、その commit の CI で照合して閉じる:
/// `terminal=closed`（rc 0）・終端の行は push・ci:success・close:ok・偽 remote の main と CI の argv は進めた commit・close の
/// 理由は `landed <自分の sha> ci=success tip=<進めた commit>`。base は自分の sha（run の無い commit）を待って
/// `ci:unmeasurable` に止まる＝RED。
#[test]
fn vcil_terminal_checks_the_commit_the_push_carried() {
    let (repo, state) = repo_with_state();
    let (out, id, tools) = land_with_push_race(&repo, &state, "\"$sha\"", true);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "閉じた終端は rc 0: {} {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("terminal=closed"), "終端の token: {}", stdout_of(&out));
    assert_eq!(terminal_rows(&state, &id), ["terminal:push:fake", "terminal:ci:success", "terminal:close:ok"]);
    let (next, sha) = (raced(&state), landed_sha(&state, &id));
    assert_eq!(git(&repo, &["rev-parse", &format!("{next}^")]), sha, "前提: main は着地の sha の子へ進んだ");
    assert_eq!(git(&tools.remote, &["rev-parse", "refs/heads/main"]), next, "push は進めた commit を押した");
    let ci_argv = fs::read_to_string(&tools.ci_log).expect("偽 CI が撃たれた");
    assert_eq!(ci_argv.lines().collect::<Vec<&str>>(), [next.as_str()], "照合は push が押した commit");
    let argv = fs::read_to_string(&tools.bd_log).expect("台帳が閉じられた");
    assert_eq!(argv.lines().nth(3), Some(format!("landed {sha} ci=success tip={next}").as_str()), "理由: {argv}");
    clean(&[&repo, &state]);
}

/// (道 2 の裏) push の後に main が動いた周は、push が押した自分の sha で照合する（追跡の ref を読み、local の main を読まない）:
/// `terminal=closed`（rc 0）・偽 remote の main は自分の sha・local の main は進めた commit・CI の argv は自分の sha・close の
/// 理由は `tip=` を持たない `landed <sha> ci=success`。
#[test]
fn vcil_terminal_ignores_a_main_moved_after_the_push() {
    let (repo, state) = repo_with_state();
    let (out, id, tools) = land_with_push_race(&repo, &state, "\"$sha\"", false);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "閉じた終端は rc 0: {} {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("terminal=closed"), "終端の token: {}", stdout_of(&out));
    assert_eq!(terminal_rows(&state, &id), ["terminal:push:fake", "terminal:ci:success", "terminal:close:ok"]);
    let (next, sha) = (raced(&state), landed_sha(&state, &id));
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), next, "前提: local の main は push の後に進んだ");
    assert_eq!(git(&repo, &["rev-parse", &format!("{next}^")]), sha, "前提: 進めた commit は着地の sha の子");
    assert_eq!(git(&tools.remote, &["rev-parse", "refs/heads/main"]), sha, "push が押したのは着地の sha");
    let ci_argv = fs::read_to_string(&tools.ci_log).expect("偽 CI が撃たれた");
    assert_eq!(ci_argv.lines().collect::<Vec<&str>>(), [sha.as_str()], "照合は push が押した自分の sha");
    let argv = fs::read_to_string(&tools.bd_log).expect("台帳が閉じられた");
    assert_eq!(argv.lines().nth(3), Some(format!("landed {sha} ci=success").as_str()), "理由に tip= を置かない: {argv}");
    clean(&[&repo, &state]);
}

/// (道 2 の否定) push が押した commit が自分の sha の子孫でない周（push の前に main を自分の sha の兄弟へ動かした）は、その
/// commit で照合せず、自分の sha で照合して閉じない: rc 1・`terminal=ci:unmeasurable`・終端の行は push と ci:unmeasurable・
/// CI の argv は自分の sha だけ・台帳を撃たない。
#[test]
fn vcil_terminal_keeps_its_own_sha_when_the_pushed_commit_is_not_a_descendant() {
    let (repo, state) = repo_with_state();
    let (out, id, tools) = land_with_push_race(&repo, &state, "\"$sha^\"", true);
    assert_eq!(out.status.code(), Some(1), "閉じない終端は rc 1: {} {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("terminal=ci:unmeasurable"), "終端の token: {}", stdout_of(&out));
    assert_eq!(terminal_rows(&state, &id), ["terminal:push:fake", "terminal:ci:unmeasurable"]);
    let (next, sha) = (raced(&state), landed_sha(&state, &id));
    assert_eq!(git(&tools.remote, &["rev-parse", "refs/heads/main"]), next, "push は兄弟の commit を押した");
    let ancestor = Command::new("git").arg("-C").arg(&repo).args(["merge-base", "--is-ancestor", &sha, &next]).status();
    assert!(ancestor.is_ok_and(|status| status.code() == Some(1)), "前提: 兄弟の commit は自分の sha の子孫でない");
    let ci_argv = fs::read_to_string(&tools.ci_log).expect("偽 CI が撃たれた");
    assert!(ci_argv.lines().all(|line| line == sha), "照合は自分の sha だけ（兄弟を照合しない）: {ci_argv}");
    assert!(!tools.bd_log.exists(), "閉じない周は台帳を撃たない");
    clean(&[&repo, &state]);
}
