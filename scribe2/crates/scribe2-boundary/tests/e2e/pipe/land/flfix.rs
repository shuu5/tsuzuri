//! 着地の追随の再 gate の FAIL の直しの周の歯（設計 pipeline.md §73・行 v-follow-fix・接頭辞 `flfix_`・親 `tests/e2e/pipe/land.rs` の helper を `use super::*` で使う）。
//!
//! 再 gate は着地の追随（main が便の base から検出線の面で進んだ便）が撃つ。便は Gated の PASS まで通し、main を面の中の commit で進めてから
//! `pipe resume`（Gated の枝）か `pipe run` で撃つ。偽 lens は呼びの印の file に 1 行ずつ足して呼びの数を数え、呼びの数に応じて判定を返す。
//! 偽 runner は turn ごとに stdin を写し、毎回違う 1 行を足して commit する（同じ字を書くと 2 回目の commit が空で Failed に倒れる）。

use super::*;

/// 行 `runner.gate_fix_rounds` を値で足した tmp manifest。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn fix_rules(state: &Path, name: &str, rounds: u64) -> PathBuf {
    let path = write_rules_full(state, name, (1, 1_000_000), FOLLOW_RETRIES, default_slots());
    let body = fs::read_to_string(&path).expect("tmp manifest を読める");
    let row = format!(
        "\n[[rule]]\nid = \"runner.gate_fix_rounds\"\nkind = \"RunnerGateFixRounds\"\nvalue = {rounds}\nenabled = true\nruling = \"t\"\nruled_at = \"d\"\n"
    );
    fs::write(&path, format!("{body}{row}")).expect("tmp manifest を書ける");
    path
}

/// 偽 lens: 呼びごとに `calls` へ 1 行足し、呼びの番号（1 始まり）の組 `(判定の語, 返す前に撃つ shell)` を返す。最後の組は以後の呼びに繰り返す。
fn scripted_lens(calls: &Path, steps: &[(&str, &str)]) -> String {
    let arms: String = steps
        .iter()
        .enumerate()
        .map(|(at, (verdict, before))| {
            let pattern = if at + 1 == steps.len() { "*".to_owned() } else { (at + 1).to_string() };
            format!("{pattern}) {before}; echo '{}';; ", lens_verdict(verdict))
        })
        .collect();
    format!(
        "cat >/dev/null; echo x >> '{c}'; N=$(wc -l < '{c}' | tr -d ' '); case \"$N\" in {arms}esac; :",
        c = calls.display()
    )
}

/// lens の呼びの数（印の file の行数・無ければ 0）。
fn calls_of(calls: &Path) -> usize {
    fs::read_to_string(calls).map(|text| text.lines().count()).unwrap_or(0)
}

/// 偽 runner: turn ごとに毎回違う語を `src/lib.rs` の末尾へ足して commit する（呼びの数と stdin は [`stub_calls`] / [`stub_stdin`] が読む）。
fn fix_runner(state: &Path, words: &[&str]) -> String {
    let turns: Vec<String> = words
        .iter()
        .map(|word| format!("printf '{word}\\n' >> src/lib.rs\ngit add -A\ngit commit -q -m '{word}'\nexit 0"))
        .collect();
    lifecycle::turn_runner(state, &turns)
}

/// main を検出線の面（`crates/`）に触れる commit で進める（再 gate を省く形に倒さない）。
fn advance_main_in_scope(repo: &Path) -> String {
    commit_other_in_scope(repo)
}

/// 偽 lens の中で main を面の中の commit で進める shell（`repo` の `crates/` へ 1 file 足して commit する）。
fn advance_main_shell(repo: &Path) -> String {
    let dir = repo.display();
    format!(
        "mkdir -p '{dir}/crates' && echo other > '{dir}/crates/other.txt' && git -C '{dir}' add -A && git -C '{dir}' commit -q -m other"
    )
}

/// Gated の PASS の便を作り、main を面の中の commit で進める（repo・置き場・便 id）。
fn gated_then_moved() -> (PathBuf, PathBuf, String) {
    let (repo, state) = repo_with_state();
    let design = write_contract(&repo, &[], &[]);
    let id = gated_pass(&repo, &state, &design, &state.join("lens-ran"));
    advance_main_in_scope(&repo);
    (repo, state, id)
}

/// 便を `pipe resume` で撃つ（`--runner` と `--rules` と `--lens`）。
fn resume_with(dirs: (&Path, &Path), id: &str, runner: &str, rules: &Path, lens: &str) -> Output {
    let (repo_arg, state_arg, rules_arg) = (dirs.0.display().to_string(), dirs.1.display().to_string(), rules.display().to_string());
    run_pipe(&[
        "resume", "--run", id, "--repo", &repo_arg, "--state-dir", &state_arg, "--runner", runner, "--rules", &rules_arg, "--lens", lens,
    ])
}

/// 便の段 `Implemented` の記帳のうち、detail が `gate-fix:` で始まる件数。
fn fix_marks(state: &Path, id: &str) -> usize {
    stages(state, id)
        .iter()
        .filter(|(stage, detail)| *stage == Some(Stage::Implemented) && detail.as_deref().is_some_and(|found| found.starts_with("gate-fix:")))
        .count()
}

/// 便の `RunStage` の列の中で、段と detail の述語が当たる最初の位置。
fn position_of(state: &Path, id: &str, from: usize, want: impl Fn(Option<Stage>, &str) -> bool) -> Option<usize> {
    stages(state, id)
        .iter()
        .enumerate()
        .skip(from)
        .find(|(_, (stage, detail))| want(*stage, detail.as_deref().unwrap_or_default()))
        .map(|(at, _)| at)
}

/// (1) 値 2 の行・Gated の PASS の便・main が面の中で進んだ後の `pipe resume` は、再 gate の 1 回目の FAIL で直しの周に入り、runner を 1 回起こし、
/// 直した後の gate の PASS で Landed まで進む（rc 0・`gate-fix=1/2`・lens は 2 回）。
#[test]
fn flfix_regate_fail_restarts_the_runner_and_lands() {
    let (repo, state, id) = gated_then_moved();
    let rules = fix_rules(&state, "rules-follow-fix-2.toml", 2);
    let runner = fix_runner(&state, &["fix-one"]);
    let calls = state.join("lens-calls");
    let lens = scripted_lens(&calls, &[("FAIL", ":"), ("PASS", ":")]);
    let out = resume_with((&repo, &state), &id, &runner, &rules, &lens);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{} / {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).lines().any(|line| line.contains(&format!("run={id} gate-fix=1/2"))), "gate-fix=1/2 の行: {}", stdout_of(&out));
    let failed = position_of(&state, &id, 0, |stage, detail| stage == Some(Stage::Gated) && detail.starts_with("verdict:FAIL"));
    let fixed = failed.and_then(|at| position_of(&state, &id, at, |stage, detail| stage == Some(Stage::Implemented) && detail == "gate-fix:1"));
    assert!(fixed.is_some(), "Gated の verdict:FAIL の後に Implemented の gate-fix:1: {:?}", stages(&state, &id));
    assert_eq!(fix_marks(&state, &id), 1, "直しの印は 1 件: {:?}", stages(&state, &id));
    assert!(show_line(&repo, &state, &id).contains("stage=Landed"), "最後の段は Landed: {}", show_line(&repo, &state, &id));
    assert_eq!(stub_calls(&state), 1, "偽 runner は 1 回");
    assert_eq!(calls_of(&calls), 2, "偽 lens は 2 回");
    clean(&[&repo, &state]);
}

/// (2) 値 1 の行で、gate の段の直しの周を 1 度使って PASS になった便の main を進めて再 gate が FAIL なら、周の上限は共有される: rc 1・stdout の末の行は
/// `gate-fix=exhausted:1`・`gate-fix:` の記帳は 1 件のまま・偽 runner は 1 回のまま。
#[test]
fn flfix_rounds_are_shared_with_the_gate_stage() {
    let (repo, state) = repo_with_state();
    let design = write_contract(&repo, &[], &[]);
    let id = implemented(&repo, &state, &design);
    let rules = fix_rules(&state, "rules-follow-fix-1.toml", 1);
    let runner = fix_runner(&state, &["fix-one", "fix-two"]);
    let calls = state.join("lens-calls");
    let lens = scripted_lens(&calls, &[("FAIL", ":"), ("PASS", ":"), ("FAIL", ":")]);
    let first = resume_with((&repo, &state), &id, &runner, &rules, &lens);
    assert_eq!(first.status.code(), Some(i32::from(RC_OK)), "gate の段の直しの周で PASS: {} / {}", stdout_of(&first), stderr_of(&first));
    assert_eq!((fix_marks(&state, &id), stub_calls(&state), calls_of(&calls)), (1, 1, 2), "直しの周は 1 度: {:?}", stages(&state, &id));
    advance_main_in_scope(&repo);
    let out = resume_with((&repo, &state), &id, &runner, &rules, &lens);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "{} / {}", stdout_of(&out), stderr_of(&out));
    assert_eq!(stdout_of(&out).lines().last(), Some(format!("run={id} gate-fix=exhausted:1").as_str()), "{}", stdout_of(&out));
    assert_eq!(fix_marks(&state, &id), 1, "gate-fix: の記帳は 1 件のまま: {:?}", stages(&state, &id));
    assert_eq!(stub_calls(&state), 1, "偽 runner は 1 回のまま");
    assert_eq!(calls_of(&calls), 3, "再 gate の lens は 3 回目");
    clean(&[&repo, &state]);
}

/// (3) 値 0 の行で再 gate が FAIL の便は、rc 1・stdout の末の行は `gate-fix=exhausted:0`・偽 runner は 0 回。
#[test]
fn flfix_zero_rounds_name_exhausted_without_a_runner() {
    let (repo, state, id) = gated_then_moved();
    let rules = fix_rules(&state, "rules-follow-fix-0.toml", 0);
    let runner = fix_runner(&state, &["fix-one"]);
    let lens = scripted_lens(&state.join("lens-calls"), &[("FAIL", ":")]);
    let out = resume_with((&repo, &state), &id, &runner, &rules, &lens);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "{} / {}", stdout_of(&out), stderr_of(&out));
    assert_eq!(stdout_of(&out).lines().last(), Some(format!("run={id} gate-fix=exhausted:0").as_str()), "{}", stdout_of(&out));
    assert_eq!(stub_calls(&state), 0, "偽 runner は起きない");
    assert_eq!(fix_marks(&state, &id), 0, "gate-fix: の記帳は無い");
    clean(&[&repo, &state]);
}

/// (4) 再 gate が FAIL の便を `--runner` つきの `pipe land` で撃つ時と、verdict が FAIL の Gated の便を `--runner` つきの `pipe resume` で撃つ時は、
/// どちらも rc 1・stdout に `gate-fix=` が無く・偽 runner は 0 回・`gate-fix:` の記帳は足されない。
#[test]
fn flfix_single_land_and_a_failed_verdict_stay_outside() {
    let (repo, state, id) = gated_then_moved();
    let rules = fix_rules(&state, "rules-follow-fix-2.toml", 2);
    let runner = fix_runner(&state, &["fix-one"]);
    let lens = scripted_lens(&state.join("lens-calls"), &[("FAIL", ":")]);
    let (repo_arg, state_arg, rules_arg) = (repo.display().to_string(), state.display().to_string(), rules.display().to_string());
    let landed = run_pipe(&["land", "--run", &id, "--repo", &repo_arg, "--state-dir", &state_arg, "--runner", &runner, "--rules", &rules_arg, "--lens", &lens]);
    assert_eq!(landed.status.code(), Some(i32::from(RC_REFUSED)), "{} / {}", stdout_of(&landed), stderr_of(&landed));
    assert!(stdout_of(&landed).contains("verdict=FAIL"), "再 gate の FAIL: {}", stdout_of(&landed));
    assert!(!stdout_of(&landed).contains("gate-fix="), "pipe land は輪を持たない: {}", stdout_of(&landed));
    assert_eq!((stub_calls(&state), fix_marks(&state, &id)), (0, 0), "runner は起きず記帳は足されない: {:?}", stages(&state, &id));
    let resumed = resume_with((&repo, &state), &id, &runner, &rules, &lens);
    assert_eq!(resumed.status.code(), Some(i32::from(RC_REFUSED)), "{} / {}", stdout_of(&resumed), stderr_of(&resumed));
    assert!(!stdout_of(&resumed).contains("gate-fix="), "FAIL の Gated の便は直しの周に入らない: {}", stdout_of(&resumed));
    assert_eq!((stub_calls(&state), fix_marks(&state, &id)), (0, 0), "runner は起きず記帳は足されない: {:?}", stages(&state, &id));
    clean(&[&repo, &state]);
}

/// (7) `pipe run` の経路: 偽 lens が呼びの順に審査の PASS・gate の PASS（その呼びの中で main を面の中の commit で進める）・再 gate の FAIL・直しの後の
/// gate の PASS を返すと、`--runner` つきの `pipe run` は rc 0・`gate-fix=1/2` の行・最後の段は Landed・偽 runner は 2 回（実装と直し）。
#[test]
fn flfix_run_regate_fail_restarts_the_runner_and_lands() {
    let (repo, state) = repo_with_state();
    let design = write_contract(&repo, &[], &[]);
    let rules = fix_rules(&state, "rules-follow-fix-2.toml", 2);
    let runner = fix_runner(&state, &["implement-one", "fix-one"]);
    let calls = state.join("lens-calls");
    let advance = advance_main_shell(&repo);
    let lens = scripted_lens(&calls, &[("PASS", ":"), ("PASS", &advance), ("FAIL", ":"), ("PASS", ":")]);
    let out = run_pipe(&[
        "run", "--design", &design, "--bead", "s2-fl", "--repo", &repo.display().to_string(), "--state-dir", &state.display().to_string(),
        "--rules", &rules.display().to_string(), "--runner", &runner, "--lens", &lens,
    ]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{} / {}", stdout_of(&out), stderr_of(&out));
    let id = run_id_of(&out);
    assert!(stdout_of(&out).lines().any(|line| line.contains(&format!("run={id} gate-fix=1/2"))), "gate-fix=1/2 の行: {}", stdout_of(&out));
    assert!(show_line(&repo, &state, &id).contains("stage=Landed"), "最後の段は Landed: {}", show_line(&repo, &state, &id));
    assert_eq!(stub_calls(&state), 2, "偽 runner は実装と直しの 2 回");
    assert_eq!(calls_of(&calls), 4, "偽 lens は審査・gate・再 gate・直しの後の gate の 4 回");
    clean(&[&repo, &state]);
}
