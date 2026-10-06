// flip-check: moved s2-07l.684
//! retire と train の族の歯（接頭辞 `pipe_retire_` / `pipe_train_` / `vcipf_`・設計 docs/design/carry-prep.md §10 行 l・親 `tests/e2e/pipe/land.rs` の helper を `use super::*` で使う）。

use super::*;
use super::cilast::{CLOSED, PUSHED};

#[test]
fn pipe_retire_moves_pr_landed_worktree_and_keeps_branch() {
    let (repo, state) = repo_with_state();
    let path = write_contract(&repo, &[], &[]);
    let marker = state.join("lens-ran");
    let id = landed_pr(&repo, &state, &path, &marker);
    let live = worktree_of(&repo, &id);
    // `--pr-cmd` 形は worktree を畳まない（merge は人が押す）＝retire の入口の前提である。
    assert!(live.exists(), "PR 形の land の後も便の worktree は在る");

    let out = retire_fold_only(&repo, &state, &id);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "retire は rc 0: {}", stderr_of(&out));
    let retired = repo.join(".worktrees").join("scribe2").join("retired").join(&id);
    assert!(
        stdout_of(&out).contains(&format!("retired={}", retired.display())),
        "畳んだ先を名乗る: {}",
        stdout_of(&out)
    );
    // **削除しない**（N1.2）: 中身が move で運ばれている。
    assert!(retired.join("src").join("lib.rs").exists(), "中身ごと運ぶ（消さない）");
    assert!(!live.exists(), "元の場所には残らない");
    let branches = git(&repo, &["branch", "--list", &format!("scribe2/{id}")]);
    assert!(!branches.trim().is_empty(), "branch は消さない: {branches}");
    let log = fs::read_to_string(state.join("fleet").join("events.jsonl")).expect("event log");
    let last = log.lines().rfind(|line| !line.is_empty()).unwrap_or_default();
    assert!(
        last.contains("\"stage\":\"Landed\"") && last.contains("\"detail\":\"retired\""),
        "最終行は Landed detail=retired（段は Landed のまま）: {last}"
    );
    let after = event_count(&state);

    // 2 度目は前提（worktree が在る）を満たさない＝**rc 1 で何も書かない**。畳んだ先へ
    // 2 周目の move を当てると、retired/<id>/<id> のような入れ子が静かに生まれる。
    let again = retire_fold_only(&repo, &state, &id);
    assert_eq!(again.status.code(), Some(i32::from(RC_REFUSED)), "2 度目は rc 1");
    assert_eq!(event_count(&state), after, "前提違反は event を 1 件も書かない");
    assert!(retired.exists(), "畳んだ先は在るまま");
    clean(&[&repo, &state]);
}

#[test]
fn pipe_retire_refuses_unless_landed_and_clean() {
    // **2 つの前提は「断ってから解いて通す」で測る**。rc 1 だけを見ると、subcommand を
    // 持っていない器でも同じ rc 1 が返るので歯が空虚になる（stderr の文言は pin しない）。
    //
    // (a) 段が Gated のまま＝**終端していない便の worktree は畳まない**。
    let (repo, state) = repo_with_state();
    let path = write_contract(&repo, &[], &[]);
    let marker = state.join("lens-ran");
    let id = gated_pass(&repo, &state, &path, &marker);
    let before = event_count(&state);
    let early = retire_fold_only(&repo, &state, &id);
    assert_eq!(early.status.code(), Some(i32::from(RC_REFUSED)), "Landed 以外は rc 1");
    assert!(worktree_of(&repo, &id).exists(), "断った周は worktree を動かさない");
    assert_eq!(event_count(&state), before, "event を 1 件も書かない");
    // 段だけを解くと同じ便が通る＝上の rc 1 は**段**を理由にしている。
    land_pr(&repo, &state, &id);
    let landed = retire_fold_only(&repo, &state, &id);
    assert_eq!(landed.status.code(), Some(i32::from(RC_OK)), "Landed なら通る: {}", stderr_of(&landed));
    clean(&[&repo, &state]);

    // (b) Landed でも worktree が dirty なら畳まない（fail-closed・untracked も数える）。
    // move は中身ごと運ぶので、未 commit の仕事を持った worktree を黙って動かすと
    // 「どこへ行ったか」が便の外から読めなくなる。
    let (dirty_repo, dirty_state) = repo_with_state();
    let dirty_path = write_contract(&dirty_repo, &[], &[]);
    let dirty_marker = dirty_state.join("lens-ran");
    let dirty_id = landed_pr(&dirty_repo, &dirty_state, &dirty_path, &dirty_marker);
    let live = worktree_of(&dirty_repo, &dirty_id);
    let stray = live.join("dirty.txt");
    fs::write(&stray, "x\n").expect("worktree を汚せる");
    let dirty_before = event_count(&dirty_state);
    let out = retire_fold_only(&dirty_repo, &dirty_state, &dirty_id);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "dirty な worktree は rc 1");
    assert!(live.exists(), "断った周は worktree を動かさない");
    assert!(stray.exists(), "汚れもそのまま残す（掃除しない）");
    assert_eq!(event_count(&dirty_state), dirty_before, "event を 1 件も書かない");
    let dirty_retired = dirty_repo.join(".worktrees").join("scribe2").join("retired").join(&dirty_id);
    assert!(!dirty_retired.exists(), "retired/<id> を作らない");
    // 汚れだけを拭うと同じ便が通る＝上の rc 1 は**clean**を理由にしている。
    fs::remove_file(&stray).expect("汚れを拭える");
    let cleaned = retire_fold_only(&dirty_repo, &dirty_state, &dirty_id);
    assert_eq!(cleaned.status.code(), Some(i32::from(RC_OK)), "clean なら通る: {}", stderr_of(&cleaned));
    assert!(dirty_retired.exists(), "畳んだ先が出来る");
    clean(&[&dirty_repo, &dirty_state]);
}

/// 同一変更の 2 便: 2 本目が `Failed detail=rebase-empty` で終端した後、その worktree を
/// `pipe retire` が畳む（`s2-07l.128`）。成果は既に main に在り**入れ物だけが残る**形は
/// `--pr-cmd` 形の `Landed` と同じで、畳み方も同じ 1 本（move・branch は残す・main 不変）。
/// 残す event の段は **`Failed` のまま**＝retire は終端を動かさない。
#[test]
fn pipe_retire_rebase_empty_folds_failed_run_and_keeps_stage() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let (id_b, landed) = gated_run_whose_change_is_already_on_main(&repo, &state, &marker);
    let lens = fake_lens(&marker, &lens_verdict("PASS"));
    let empty = run_pipe(&[
        "land", "--run", &id_b, "--repo", &repo.display().to_string(),
        "--state-dir", &state.display().to_string(), "--lens", &lens,
    ]);
    assert_eq!(
        empty.status.code(),
        Some(i32::from(RC_REFUSED)),
        "2 本目は rebase-empty で終端する: {}",
        stderr_of(&empty)
    );
    assert!(show_line(&repo, &state, &id_b).contains("stage=Failed"), "終端の段は Failed");
    let live = worktree_of(&repo, &id_b);
    assert!(live.exists(), "終端した便の worktree は残る（retire の入口の前提）");

    let out = retire_once(&repo, &state, &id_b);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "rebase-empty の便も畳める: {}", stderr_of(&out));
    let retired = repo.join(".worktrees").join("scribe2").join("retired").join(&id_b);
    assert!(
        stdout_of(&out).contains(&format!("retired={}", retired.display())),
        "畳んだ先を名乗る: {}",
        stdout_of(&out)
    );
    assert!(retired.join("src").join("lib.rs").exists(), "中身ごと運ぶ（消さない）");
    assert!(!live.exists(), "元の場所が空く");
    let branches = git(&repo, &["branch", "--list", &format!("scribe2/{id_b}")]);
    assert!(!branches.trim().is_empty(), "branch は消さない: {branches}");
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), landed, "main は 1 byte も動かない");
    let log = fs::read_to_string(state.join("fleet").join("events.jsonl")).expect("event log");
    let last = log.lines().rfind(|line| !line.is_empty()).unwrap_or_default();
    assert!(
        last.contains("\"stage\":\"Failed\"") && last.contains("\"detail\":\"retired\""),
        "最終行は Failed detail=retired（段を Landed へ動かさない）: {last}"
    );
    assert!(show_line(&repo, &state, &id_b).contains("stage=Failed"), "畳んだ後も段は Failed");
    clean(&[&repo, &state]);
}

/// `Failed` の便は **detail を問わず**畳める（設計 §24 の約束 1 / 2）。base が断っていた 4 つの
/// detail を 1 つずつ名指す 4 本の 1 本目——**main の実測が赤かった便**（`main-red`）。
///
/// 段の弁別から detail を外しても **clean の検査は残る**ので、汚れた木で断ってから拭って通す
/// 対で測る（上の rc 1 は clean を理由にしており、detail ではない）。
#[test]
fn pipe_retire_failed_any_detail_main_red_folds_and_keeps_stage() {
    let (repo, state) = repo_with_state();
    // 1 回目（worktree）は緑・2 回目（main の実測）は赤になる verify 行＝`main-red` で終端する。
    let path = write_contract(&repo, &["verify"], &[r#"verify = ["sh verify-once.sh"]"#]);
    let marker = state.join("lens-ran");
    let id = gated_pass(&repo, &state, &path, &marker);
    make_tree_differ(&repo, &state, &id, "refs/heads/main");
    let red = land_once(&repo, &state, &id);
    assert_eq!(red.status.code(), Some(i32::from(RC_REFUSED)), "main が赤い land は rc 1: {}", stderr_of(&red));
    assert_eq!(
        stages(&state, &id).last().cloned(),
        Some((Some(Stage::Failed), Some("main-red".to_owned()))),
        "終端の理由は main-red: {:?}",
        stages(&state, &id)
    );

    // 対の負例: 汚れた木は畳まない（rc 1・worktree 不動・event 0 増）。
    let live = worktree_of(&repo, &id);
    let stray = live.join("dirty.txt");
    fs::write(&stray, "x\n").expect("worktree を汚せる");
    let before = event_count(&state);
    let dirty = retire_once(&repo, &state, &id);
    assert_eq!(dirty.status.code(), Some(i32::from(RC_REFUSED)), "dirty な worktree は rc 1: {}", stdout_of(&dirty));
    assert!(live.exists(), "断った周は worktree を動かさない");
    assert!(
        !repo.join(".worktrees").join("scribe2").join("retired").join(&id).exists(),
        "retired/<run> を作らない"
    );
    assert_eq!(event_count(&state), before, "前提違反は event を 1 件も書かない");
    fs::remove_file(&stray).expect("汚れを拭える");

    folds_and_keeps_stage(&repo, &state, &id, Stage::Failed);
    assert!(show_line(&repo, &state, &id).contains("stage=Failed"), "畳んだ後も段は Failed");
    clean(&[&repo, &state]);
}

/// 2 本目: main を**実測できなかった**便（`main-unmeasured`・rc 2 で終端する側）。
#[test]
fn pipe_retire_failed_any_detail_main_unmeasured_folds_and_keeps_stage() {
    let (repo, state) = repo_with_state();
    let path = write_contract(&repo, &[], &[]);
    let marker = state.join("lens-ran");
    let id = gated_pass(&repo, &state, &path, &marker);
    make_tree_differ(&repo, &state, &id, "refs/heads/main");
    // main 実測用の tmp の置き場を塞ぐ＝**verify を 1 行も撃てない**
    // （`pipe_land_reports_unmeasured_main_apart_from_red` と同じ fixture）。
    let blocked = repo.join(".worktrees").join("scribe2").join("verify").join(&id);
    fs::create_dir_all(&blocked).expect("tmp の置き場を塞げる");
    fs::write(blocked.join("occupied"), "x\n").expect("塞げる");
    let out = land_once(&repo, &state, &id);
    assert_eq!(out.status.code(), Some(i32::from(RC_BROKEN)), "測れない周は rc 2: {}", stderr_of(&out));
    assert_eq!(
        stages(&state, &id).last().cloned(),
        Some((Some(Stage::Failed), Some("main-unmeasured".to_owned()))),
        "終端の理由は main-unmeasured: {:?}",
        stages(&state, &id)
    );
    folds_and_keeps_stage(&repo, &state, &id, Stage::Failed);
    assert!(show_line(&repo, &state, &id).contains("stage=Failed"), "畳んだ後も段は Failed");
    clean(&[&repo, &state]);
}

/// 3 本目: 追随の rebase の**途中で** turn が終わった便（`rebase-dirty`）。木が rebase の途中＝
/// dirty なので retire は clean の検査で断り、木を戻すと同じ便が通る——この対で「断りは clean・
/// 段の弁別は `Failed` を detail ごと通す」の両方が測れる。
#[test]
fn pipe_retire_failed_any_detail_rebase_dirty_folds_and_keeps_stage() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let runner = stub_runner(&state, LEAVE_MID_REBASE);
    let (id, _base, moved) = conflicting_run(&repo, &state, &marker, &runner);
    let out = land_extra(&repo, &state, &id, &["--runner", &runner]);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "rebase の途中は rc 1: {}", stderr_of(&out));
    assert_eq!(
        stages(&state, &id).last().cloned(),
        Some((Some(Stage::Failed), Some("rebase-dirty".to_owned()))),
        "終端の理由は rebase-dirty: {:?}",
        stages(&state, &id)
    );
    assert!(mid_rebase(&repo, &id), "木は rebase の途中のまま（器は触らない）");

    let live = worktree_of(&repo, &id);
    let before = event_count(&state);
    let dirty = retire_once(&repo, &state, &id);
    assert_eq!(dirty.status.code(), Some(i32::from(RC_REFUSED)), "rebase の途中の木は rc 1: {}", stdout_of(&dirty));
    assert!(live.exists(), "断った周は worktree を動かさない");
    assert_eq!(event_count(&state), before, "前提違反は event を 1 件も書かない");
    // 木を戻すと同じ便が通る＝上の rc 1 は clean を理由にしている（終端の理由ではない）。
    git(&live, &["rebase", "--abort"]);
    folds_and_keeps_stage(&repo, &state, &id, Stage::Failed);
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), moved, "main は 1 byte も動かない");
    clean(&[&repo, &state]);
}

/// 4 本目: gate の前提違反で終端した便（`precheck:…`）。commit を 1 本も持たない木は gate が
/// `Failed detail=precheck:…` で残す（木は clean のまま）＝ここでの rc 0 は **detail の弁別が
/// 無い**ことだけを測る。
#[test]
fn pipe_retire_failed_any_detail_precheck_folds_and_keeps_stage() {
    let (repo, state) = repo_with_state();
    let path = write_contract(&repo, &[], &[]);
    let id = implemented(&repo, &state, &path);
    // 便の commit を捨てる＝precheck が「commit が 1 本も無い」で落ちる（木は clean のまま）。
    let live = worktree_of(&repo, &id);
    git(&live, &["reset", "--hard", "refs/heads/main"]);
    assert!(git(&live, &["status", "--porcelain"]).is_empty(), "木は clean のまま");
    let marker = state.join("lens-ran");
    let lens = fake_lens(&marker, &lens_verdict("PASS"));
    let out = gate_once(&repo, &state, &id, Some(&lens));
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "前提違反は rc 1: {}", stderr_of(&out));
    assert!(!marker.exists(), "**lens を起動しない**（precheck で止まる周）");
    let terminal = stages(&state, &id).last().cloned();
    assert!(
        matches!(&terminal, Some((Some(Stage::Failed), Some(detail))) if detail.starts_with("precheck:")),
        "終端の理由は precheck:…: {terminal:?}"
    );
    folds_and_keeps_stage(&repo, &state, &id, Stage::Failed);
    assert!(show_line(&repo, &state, &id).contains("stage=Failed"), "畳んだ後も段は Failed");
    clean(&[&repo, &state]);
}

/// 退行の pin（設計 §24 の約束 4）: **非終端**（`Spawned` / `Implemented`）と `Gated(PASS)` は
/// 畳まない。`Failed` から detail の弁別を外しても `allowed` の列は不変で、段の一般則は効き
/// 続ける——`Spawned` は終端させると同じ便が通る＝その rc 1 は**段**を理由にしている。
#[test]
fn pipe_retire_failed_any_detail_still_refuses_live_runs_and_gated_pass() {
    // (a) `Spawned`（runner が生きている便）。
    let (repo, state) = repo_with_state();
    let id = spawned_run(&repo, &state);
    let live = worktree_of(&repo, &id);
    let before = event_count(&state);
    let spawned = retire_once(&repo, &state, &id);
    assert_eq!(spawned.status.code(), Some(i32::from(RC_REFUSED)), "Spawned は rc 1: {}", stdout_of(&spawned));
    assert!(live.exists(), "断った周は worktree を動かさない");
    assert_eq!(event_count(&state), before, "event を 1 件も書かない");
    // 段を終端（`Stopped`）へ解くと同じ便が通る＝`allowed` の列は不変である。
    stop_run_ok(&state, &id);
    let stopped = retire_once(&repo, &state, &id);
    assert_eq!(stopped.status.code(), Some(i32::from(RC_OK)), "Stopped なら通る: {}", stderr_of(&stopped));
    clean(&[&repo, &state]);

    // (b) `Implemented`（gate を待つ便）→ (c) `Gated(PASS)`（land が残っている便）。
    let (other, other_state) = repo_with_state();
    let path = write_contract(&other, &[], &[]);
    let waiting_id = implemented(&other, &other_state, &path);
    let waiting_tree = worktree_of(&other, &waiting_id);
    let waiting_before = event_count(&other_state);
    let waiting = retire_once(&other, &other_state, &waiting_id);
    assert_eq!(waiting.status.code(), Some(i32::from(RC_REFUSED)), "Implemented は rc 1: {}", stdout_of(&waiting));
    assert!(waiting_tree.exists(), "断った周は worktree を動かさない");
    assert_eq!(event_count(&other_state), waiting_before, "event を 1 件も書かない");

    let marker = other_state.join("lens-ran");
    let lens = fake_lens(&marker, &lens_verdict("PASS"));
    let gated = gate_once(&other, &other_state, &waiting_id, Some(&lens));
    assert_eq!(gated.status.code(), Some(i32::from(RC_OK)), "PASS の gate は rc 0: {}", stderr_of(&gated));
    let pass_before = event_count(&other_state);
    let pass = retire_once(&other, &other_state, &waiting_id);
    assert_eq!(pass.status.code(), Some(i32::from(RC_REFUSED)), "Gated(PASS) は rc 1: {}", stdout_of(&pass));
    assert!(waiting_tree.exists(), "断った周は worktree を動かさない");
    assert!(
        !other.join(".worktrees").join("scribe2").join("retired").join(&waiting_id).exists(),
        "retired/<run> を作らない"
    );
    assert_eq!(event_count(&other_state), pass_before, "event を 1 件も書かない");
    clean(&[&other, &other_state]);
}

/// `Stopped` の便を `pipe retire` が畳む（`s2-07l.284`・設計 pipeline-conflict.md §5）。stop は
/// 畳まない（C2）ので、止めた便の commit 0・clean の worktree はこの口でしか動かせない。畳み方は
/// 他の終端と同じ 1 本（move・branch は残す・main 不変）で、残す event の段は **`Stopped` のまま**。
#[test]
fn pipe_retire_stopped_folds_a_clean_stopped_run_and_keeps_stage() {
    let (repo, state) = repo_with_state();
    let main_before = git(&repo, &["rev-parse", "refs/heads/main"]);
    let id = stopped_run(&repo, &state);
    let live = worktree_of(&repo, &id);

    let out = retire_once(&repo, &state, &id);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "Stopped の便も畳める: {}", stderr_of(&out));
    let retired = repo.join(".worktrees").join("scribe2").join("retired").join(&id);
    assert!(
        stdout_of(&out).contains(&format!("retired={}", retired.display())),
        "畳んだ先を名乗る: {}",
        stdout_of(&out)
    );
    assert!(retired.join("src").join("lib.rs").exists(), "中身ごと運ぶ（消さない）");
    assert!(!live.exists(), "元の場所が空く");
    let branches = git(&repo, &["branch", "--list", &format!("scribe2/{id}")]);
    assert!(!branches.trim().is_empty(), "branch は消さない: {branches}");
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), main_before, "main は 1 byte も動かない");
    let log = fs::read_to_string(state.join("fleet").join("events.jsonl")).expect("event log");
    let last = log.lines().rfind(|line| !line.is_empty()).unwrap_or_default();
    assert!(
        last.contains("\"stage\":\"Stopped\"") && last.contains("\"detail\":\"retired\""),
        "最終行は Stopped detail=retired（段を Landed へ動かさない）: {last}"
    );
    assert!(show_line(&repo, &state, &id).contains("stage=Stopped"), "畳んだ後も段は Stopped");
    let after = event_count(&state);

    // 2 度目は前提（worktree が在る）を満たさない＝rc 1 で何も書かない。
    let again = retire_once(&repo, &state, &id);
    assert_eq!(again.status.code(), Some(i32::from(RC_REFUSED)), "2 度目は rc 1");
    assert_eq!(event_count(&state), after, "前提違反は event を 1 件も書かない");
    assert!(retired.exists(), "畳んだ先は在るまま");
    clean(&[&repo, &state]);
}

/// 負例: `Stopped` でも worktree が dirty なら畳まない（rc 1・worktree 不動・event 0 増）。
/// 負例を**clean 検査で断る位置**に置く＝段の検査は通っている（上の歯との対で、`Stopped` に
/// `Extra::Retire` の clean 検査が同じく効くことを担保する・untracked も数える）。
#[test]
fn pipe_retire_stopped_refuses_a_dirty_worktree() {
    let (repo, state) = repo_with_state();
    let id = stopped_run(&repo, &state);
    let live = worktree_of(&repo, &id);
    let stray = live.join("dirty.txt");
    fs::write(&stray, "x\n").expect("worktree を汚せる");
    let before = event_count(&state);

    let out = retire_once(&repo, &state, &id);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "dirty な worktree は rc 1: {}", stdout_of(&out));
    assert!(live.exists(), "断った周は worktree を動かさない");
    assert!(stray.exists(), "汚れもそのまま残す（掃除しない）");
    let retired = repo.join(".worktrees").join("scribe2").join("retired").join(&id);
    assert!(!retired.exists(), "retired/<run> を作らない");
    assert_eq!(event_count(&state), before, "event を 1 件も書かない");
    assert!(show_line(&repo, &state, &id).contains("stage=Stopped"), "段は Stopped のまま");
    // 汚れだけを拭うと同じ便が通る＝上の rc 1 は**clean**を理由にしている（段ではない）。
    fs::remove_file(&stray).expect("汚れを拭える");
    let cleaned = retire_once(&repo, &state, &id);
    assert_eq!(cleaned.status.code(), Some(i32::from(RC_OK)), "clean なら通る: {}", stderr_of(&cleaned));
    assert!(retired.exists(), "畳んだ先が出来る");
    clean(&[&repo, &state]);
}

/// 審査が **FAIL** で終端した `Reviewed` の便を畳む（判定に届いた終端・段の列に `Reviewed` が在る）。
/// 畳み方は他の終端と同じ 1 本（move・branch は残す・main 不変）で、残す event の段は `Reviewed` のまま。
#[test]
fn pipe_retire_reviewed_fail_folds_and_keeps_stage() {
    let (repo, state) = repo_with_state();
    let id = reviewed_with_worktree(&repo, &state, &review_body("FAIL"));
    let main_before = git(&repo, &["rev-parse", "refs/heads/main"]);
    folds_and_keeps_stage(&repo, &state, &id, Stage::Reviewed);
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), main_before, "main は 1 byte も動かない");
    assert!(show_line(&repo, &state, &id).contains("stage=Reviewed"), "畳んだ後も段は Reviewed");
    let after = event_count(&state);

    // 2 度目は前提（worktree が在る）を満たさない＝rc 1 で何も書かない（入れ子の retired/<run>/<run> を作らない）。
    let again = retire_once(&repo, &state, &id);
    assert_eq!(again.status.code(), Some(i32::from(RC_REFUSED)), "2 度目は rc 1");
    assert_eq!(event_count(&state), after, "前提違反は event を 1 件も書かない");
    clean(&[&repo, &state]);
}

/// 審査が **INCONCLUSIVE** で終端した便も同じく畳める（FAIL との弁別は入口に無い＝どちらも「この材料では
/// 通らなかった」終端）。残す event の段は `Reviewed` のまま・`detail=retired`。
#[test]
fn pipe_retire_reviewed_inconclusive_folds_and_keeps_stage() {
    let (repo, state) = repo_with_state();
    let id = reviewed_with_worktree(&repo, &state, &review_body("INCONCLUSIVE"));
    let main_before = git(&repo, &["rev-parse", "refs/heads/main"]);
    folds_and_keeps_stage(&repo, &state, &id, Stage::Reviewed);
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), main_before, "main は 1 byte も動かない");
    assert!(show_line(&repo, &state, &id).contains("stage=Reviewed"), "畳んだ後も段は Reviewed");
    clean(&[&repo, &state]);
}

/// 負例 (1): 審査が **PASS** の便は畳まない——これから起こす側（live）で、入れ物は次の段が使う。
///
/// 断りは**段だけでなく判定も名乗る**。字面は 1 行丸ごとで測る＝段違いの一般則の字面（`run <id> の段は
/// Reviewed である`・verdict の括弧を持たない）では通らず、括弧の語は `ReviewCheck::as_str` が `Passed` に
/// 返す `PASS` である。一般則そのものの形は末尾で `Implemented` の便から実測して対に置く。
#[test]
fn pipe_retire_reviewed_pass_refused_and_names_the_verdict() {
    let (repo, state) = repo_with_state();
    let id = reviewed_with_worktree(&repo, &state, &review_body("PASS"));
    let live = worktree_of(&repo, &id);
    let before = event_count(&state);

    let out = retire_once(&repo, &state, &id);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "Reviewed(PASS) は rc 1: {}", stdout_of(&out));
    assert_eq!(
        stderr_of(&out).trim(),
        format!("pipe: run {id} の段は Reviewed である（verdict=PASS）"),
        "断りは判定の語まで名乗る"
    );
    assert!(live.exists(), "断った周は worktree を動かさない");
    assert!(
        !repo.join(".worktrees").join("scribe2").join("retired").join(&id).exists(),
        "retired/<run> を作らない"
    );
    assert_eq!(event_count(&state), before, "前提違反は event を 1 件も書かない");
    // 判定だけを終端の側へ解くと同じ便が通る＝上の rc 1 は **verdict** を理由にしている（段ではない）。
    write_review_verdict(&state, &id, &review_body("FAIL"));
    let folded = retire_once(&repo, &state, &id);
    assert_eq!(folded.status.code(), Some(i32::from(RC_OK)), "FAIL なら通る: {}", stderr_of(&folded));
    clean(&[&repo, &state]);

    // 対の実測: 段の列に無い段（`Implemented`）の断りは **一般則のまま**で、verdict の括弧を持たない。
    let (other, other_state) = repo_with_state();
    let other_path = write_contract(&other, &[], &[]);
    let waiting = implemented(&other, &other_state, &other_path);
    let general = retire_once(&other, &other_state, &waiting);
    assert_eq!(general.status.code(), Some(i32::from(RC_REFUSED)), "Implemented は rc 1: {}", stdout_of(&general));
    assert_eq!(
        stderr_of(&general).trim(),
        format!("pipe: run {waiting} の段は Implemented である"),
        "一般則は段だけを名乗る"
    );
    clean(&[&other, &other_state]);
}

/// 断りの照合に使う stderr（終端の周が写す `notify=` の行を除いた本文・設計 dispatcher.md §29 形 4）。
fn refusal_of(out: &Output) -> String {
    stderr_of(out).lines().filter(|line| !line.starts_with("notify=")).collect::<Vec<_>>().join("\n").trim().to_owned()
}

// flip-check: retroactive s2-07l.732
/// 負例 (2): 判定を**読めない**便も畳まない（fail-closed・読めない判定を終端に読み替えない）。母集団は
/// 「JSON でない本文」と「3 値の外」の 2 つで、どちらも同じ 1 行（括弧の語は `Unreadable` の `読めない`）。
#[test]
fn pipe_retire_reviewed_unreadable_refused_and_names_the_verdict() {
    let (repo, state) = repo_with_state();
    let id = reviewed_with_worktree(&repo, &state, "not json\n");
    let live = worktree_of(&repo, &id);
    let before = event_count(&state);
    let reason = format!("pipe: run {id} の段は Reviewed である（verdict=読めない）");

    let out = retire_once(&repo, &state, &id);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "読めない判定は rc 1: {}", stdout_of(&out));
    assert_eq!(refusal_of(&out), reason, "断りは「読めない」を名乗る");
    assert!(live.exists(), "断った周は worktree を動かさない");
    assert!(
        !repo.join(".worktrees").join("scribe2").join("retired").join(&id).exists(),
        "retired/<run> を作らない"
    );
    assert_eq!(event_count(&state), before, "前提違反は event を 1 件も書かない");
    // 3 値の外も同じ断り（PASS でない字面を「終端」に読み替えない）。
    write_review_verdict(&state, &id, &review_body("MAYBE"));
    let outside = retire_once(&repo, &state, &id);
    assert_eq!(outside.status.code(), Some(i32::from(RC_REFUSED)), "3 値の外も rc 1: {}", stdout_of(&outside));
    assert_eq!(refusal_of(&outside), reason, "3 値の外も「読めない」");
    assert_eq!(event_count(&state), before, "event を 1 件も書かない");
    // 判定を読める終端へ直すと同じ便が通る＝上の rc 1 は**判定の読めなさ**を理由にしている（段ではない）。
    write_review_verdict(&state, &id, &review_body("INCONCLUSIVE"));
    let folded = retire_once(&repo, &state, &id);
    assert_eq!(folded.status.code(), Some(i32::from(RC_OK)), "読める終端なら通る: {}", stderr_of(&folded));
    clean(&[&repo, &state]);
}

/// (a) 上限 3 で先頭を land すると 3 本が**列の順に**着地する: 親の連鎖（base → a → b → c）・main の先端は c・`Landed`
/// 3 件・面 5 は 3 行で後続 2 本が `order=train`・後続の追随は 0 回。先頭の `verify.jsonl` に共通 verify が `train=3` で
/// 1 組、後続の `verify.jsonl` に足された record は契約 verify だけ（検出線は写しに無い）。着地済みの便の land は
/// rc 0 で main も event も動かさない。base（`train=` の無い経路）は a だけが載り b / c は Gated のまま＝RED。
#[test]
fn pipe_train_three_runs_land_in_queue_order_with_one_candidate_check() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let [id_a, id_b, id_c] = train_runs(&repo, &state, &marker, [ALL_GREEN, ALL_GREEN, ALL_GREEN]);
    let base = git(&repo, &["rev-parse", "refs/heads/main"]);
    let before: Vec<usize> = [&id_a, &id_b, &id_c].iter().map(|id| verify_rows(&state, id).len()).collect();
    let rules = write_rules_train(&state, "rules-train.toml", Some(3));
    let out = land_extra(&repo, &state, &id_a, &["--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "列の land は rc 0: {} / {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains(&format!("run={id_a} train=3")), "stdout に train=3: {}", stdout_of(&out));
    let main = assert_train_chain(&repo, &state, &base, [&id_a, &id_b, &id_c]);
    assert_train_records(&state, [&id_a, &id_b, &id_c], &before);
    let events = event_count(&state);
    let again = land_extra(&repo, &state, &id_b, &["--rules", &rules]);
    assert_eq!(again.status.code(), Some(i32::from(RC_OK)), "着地済みの便の land は rc 0: {}", stderr_of(&again));
    assert!(stdout_of(&again).contains("already-landed"), "already-landed を名乗る: {}", stdout_of(&again));
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), main, "main は不変");
    assert_eq!(event_count(&state), events, "event は増えない");
    clean(&[&repo, &state]);
}

/// (a) 番待ちで待っている 2 本目の land（子 process・`pipe.land_wait_s` の窓）と並行に先頭が列で着地すると、2 本目は
/// 起きた後に rc 0 の `already-landed` で終端し event を 1 件も足さない（追随へ進まない）。
#[test]
fn pipe_train_waiting_second_run_wakes_already_landed() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let [id_a, id_b, _id_c] = train_runs(&repo, &state, &marker, [ALL_GREEN, ALL_GREEN, ALL_GREEN]);
    let rules = write_rules_train(&state, "rules-train.toml", Some(3));
    let lens = fake_lens(&marker, &lens_verdict("PASS"));
    let mut waiting = land_in_background(&repo, &state, &id_b, &rules, &lens);
    std::thread::sleep(Duration::from_secs(2));
    assert!(waiting.try_wait().expect("子の状態を読める").is_none(), "2 本目は列の前が空くまで待っている");
    let first = land_extra(&repo, &state, &id_a, &["--rules", &rules]);
    assert_eq!(first.status.code(), Some(i32::from(RC_OK)), "先頭の land: {} / {}", stdout_of(&first), stderr_of(&first));
    assert!(stdout_of(&first).contains("train=3"), "列で着地した: {}", stdout_of(&first));
    let events = event_count(&state);
    let main = git(&repo, &["rev-parse", "refs/heads/main"]);
    let out = waiting.wait_with_output().expect("待っていた land が終わる");
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "起きた 2 本目は rc 0: {} / {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("already-landed"), "already-landed で終端: {}", stdout_of(&out));
    assert_eq!(event_count(&state), events, "event は増えない");
    assert_eq!(follow_count(&state, &id_b), 0, "追随しない: {:?}", stages(&state, &id_b));
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), main, "main は動かない");
    clean(&[&repo, &state]);
}

/// (b) 3 本目の契約 verify が候補の木で赤なら列を解き、先頭だけが既存の経路で着地する。後続 2 本は Gated PASS の
/// まま列に残り、stdout に `dissolved`。
#[test]
fn pipe_train_red_contract_dissolves_and_only_the_front_lands() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let [id_a, id_b, id_c] = train_runs(&repo, &state, &marker, [ALL_GREEN, ALL_GREEN, TRAIN_RED]);
    let base = git(&repo, &["rev-parse", "refs/heads/main"]);
    let rules = write_rules_train(&state, "rules-train.toml", Some(3));
    let out = land_extra(&repo, &state, &id_a, &["--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "先頭は着地する: {} / {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains(&format!("run={id_a} train=3 dissolved")), "列を解いた: {}", stdout_of(&out));
    let main = git(&repo, &["rev-parse", "refs/heads/main"]);
    assert_eq!(landed_sha_of(&state, &id_a), main, "先頭だけが載る");
    assert_eq!(git(&repo, &["rev-list", "--count", &format!("{base}..{main}")]), "1", "main は 1 本だけ進む");
    for id in [&id_b, &id_c] {
        assert!(show_line(&repo, &state, id).contains("stage=Gated"), "後続は Gated のまま: {id}");
        assert_eq!(value_of(&verdict_pairs(&state, id), "verdict"), "PASS", "後続の判定は PASS のまま: {id}");
        assert!(worktree_of(&repo, id).is_dir(), "後続の worktree は列に残る: {id}");
        assert_eq!(landed_count(&state, id), 0, "後続は着地しない: {id}");
    }
    assert_eq!(verdict_lines(&state), 1, "面 5 は先頭の 1 行");
    clean(&[&repo, &state]);
}

/// (c) 2 本目が先頭と衝突する周（gate の後に 2 本目の branch が先頭と同じ file を足した形）は 2 本目を候補から外して
/// 1 本目と 3 本目が着地する。2 本目の worktree は clean で HEAD も動かず、event は 1 件も増えない。
#[test]
fn pipe_train_conflicting_second_is_left_out_and_the_rest_lands() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let [id_a, id_b, id_c] = train_runs(&repo, &state, &marker, [ALL_GREEN, ALL_GREEN, ALL_GREEN]);
    let second = worktree_of(&repo, &id_b);
    fs::create_dir_all(second.join("crates").join("toy")).expect("衝突の dir を作れる");
    fs::write(second.join("crates").join("toy").join("a.rs"), "conflict\n").expect("衝突の file を書ける");
    git(&second, &["add", "-A"]);
    git(&second, &["commit", "-q", "-m", "conflict"]);
    let head_before = git(&second, &["rev-parse", "HEAD"]);
    let trail_before = trail(&state, &id_b).len();
    let rules = write_rules_train(&state, "rules-train.toml", Some(3));
    let out = land_extra(&repo, &state, &id_a, &["--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "列の land は rc 0: {} / {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains(&format!("run={id_a} train=2")), "積んだのは 2 本: {}", stdout_of(&out));
    let (sha_a, sha_c) = (landed_sha_of(&state, &id_a), landed_sha_of(&state, &id_c));
    assert_eq!(git(&repo, &["rev-parse", &format!("{sha_c}^")]), sha_a, "c は a の上に詰めて載る");
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), sha_c, "main の先端は c");
    assert!(show_line(&repo, &state, &id_b).contains("stage=Gated"), "2 本目は Gated のまま");
    assert_eq!(trail(&state, &id_b).len(), trail_before, "2 本目の event は増えない");
    assert_eq!(git(&second, &["rev-parse", "HEAD"]), head_before, "2 本目の HEAD は動かない");
    assert!(git(&second, &["status", "--porcelain"]).is_empty(), "2 本目の worktree は clean");
    assert_eq!(exported_order(&state, &id_c), "train", "3 本目は train");
    clean(&[&repo, &state]);
}

/// (d) 上限 1 と行の不在は先頭だけが着地し（`train=` を出さない）、後続は自分の land で従来どおり追随 1 回。
#[test]
fn pipe_train_limit_one_and_absent_row_land_only_the_front() {
    for (name, limit) in [("limit-1", Some(1)), ("absent", None)] {
        let (repo, state) = repo_with_state();
        let marker = state.join("lens-ran");
        let [id_a, id_b, _id_c] = train_runs(&repo, &state, &marker, [ALL_GREEN, ALL_GREEN, ALL_GREEN]);
        let rules = write_rules_train(&state, "rules-train.toml", limit);
        let first = land_extra(&repo, &state, &id_a, &["--rules", &rules]);
        assert_eq!(first.status.code(), Some(i32::from(RC_OK)), "{name}: 先頭の land: {}", stderr_of(&first));
        assert!(!stdout_of(&first).contains("train="), "{name}: 列を積まない: {}", stdout_of(&first));
        assert!(show_line(&repo, &state, &id_b).contains("stage=Gated"), "{name}: 2 本目は Gated のまま");
        let lens = fake_lens(&marker, &lens_verdict("PASS"));
        let second = land_extra(&repo, &state, &id_b, &["--rules", &rules, "--lens", &lens]);
        assert_eq!(second.status.code(), Some(i32::from(RC_OK)), "{name}: 2 本目の land: {}", stderr_of(&second));
        assert_eq!(follow_count(&state, &id_b), 1, "{name}: 追随は 1 回: {:?}", stages(&state, &id_b));
        assert!(show_line(&repo, &state, &id_b).contains("stage=Landed"), "{name}: 2 本目は Landed");
        clean(&[&repo, &state]);
    }
}

/// 列の 3 本を常に `conclusion` の偽 CI・偽 remote・偽 bd を宣言した repo で先頭の land から着地させ、(道具, stdout,
/// 列の便 id, 先端の sha) を返す（先端の sha は `sha:` を持つ `Landed` の行から読む＝終端の行を飛ばす）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn land_train_with_terminal(repo: &Path, state: &Path, conclusion: &str) -> (FakeTerminal, String, [String; 3], String) {
    let tools = fake_terminal(repo, state, conclusion);
    let marker = state.join("lens-ran");
    let ids = train_runs(repo, state, &marker, [ALL_GREEN, ALL_GREEN, ALL_GREEN]);
    let rules = write_rules_train(state, "rules-train.toml", Some(3));
    let bd = state.join("fake-bd.sh").display().to_string();
    let out = land_extra(repo, state, &ids[0], &["--bd", &bd, "--rules", &rules]);
    let stdout = stdout_of(&out);
    assert!(stdout.contains(&format!("run={} train=3", ids[0])), "列で着地した: {stdout} / {}", stderr_of(&out));
    let tip_sha = sha_of_landing(state, &ids[2]).expect("先端の便の sha を読める");
    (tools, stdout, ids, tip_sha)
}

/// 便の着地そのものを記した `Landed` の行の `sha:`（終端の行 `terminal:` は `sha:` を持たないので飛ばす）。
fn sha_of_landing(state: &Path, id: &str) -> Option<String> {
    landed_details(state, id)
        .iter()
        .find_map(|detail| detail.split_whitespace().find_map(|word| word.strip_prefix("sha:")).map(str::to_owned))
}

/// 便の `Landed` の後ろに並ぶ終端の行（`terminal:`）。
fn terminal_tail(state: &Path, id: &str) -> Vec<String> {
    landed_details(state, id).into_iter().filter(|detail| detail.starts_with("terminal:")).collect()
}

/// 列の終端（設計 contract-source.md §53・行 be・行 v-ci-child-cut）: 常に success の偽 CI を持つ 3 本の列の着地で、3 本とも
/// push → close の 2 段だけを通して子を起こさず、偽 CI の呼びは 0 回で、偽 remote の main は先端の sha。
#[test]
fn pipe_train_terminal_every_run_closes_without_a_ci_child() {
    let (repo, state) = repo_with_state();
    let (tools, stdout, [id_a, id_b, id_c], tip_sha) = land_train_with_terminal(&repo, &state, "success");
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), tip_sha, "main の先端は c");
    for id in [&id_a, &id_b, &id_c] {
        assert_eq!(terminal_tail(&state, id), [PUSHED, CLOSED], "列の便は push → close だけ: {id}");
        let line = stdout.lines().find(|line| line.starts_with(&format!("run={id} landed="))).map(str::to_owned);
        assert!(
            line.as_deref().is_some_and(|found| found.ends_with("terminal=closed")),
            "stdout の terminal= は closed: {id} {stdout}"
        );
    }
    assert_eq!(tools.ci_call_count(), 0, "偽 CI は撃たれない");
    assert_eq!(git(&tools.remote, &["rev-parse", "refs/heads/main"]), tip_sha, "偽 remote の main は先端の sha");
    clean(&[&repo, &state]);
}

/// 列の close の理由（設計 contract-source.md §53・行 be・台帳の問い t3-hub.90.2 の裁定）: 1 周目は常に success の偽 CI の列の
/// 着地で、偽 bd の最後の close（最後に終端する列の先頭の便 a）の reason が `landed <a の sha> host=green`（先端を理由に置かない）。
/// 2 周目は偽 CI が failure を返す列の着地で、3 本とも host の緑で閉じ、終端の行は push と close の 2 件ずつ。
#[test]
fn pipe_train_tip_close_reason_is_host_green_and_failure_still_closes() {
    let (repo, state) = repo_with_state();
    let (tools, _, [id_a, _, _], tip_sha) = land_train_with_terminal(&repo, &state, "success");
    let sha_a = sha_of_landing(&state, &id_a).expect("先頭の便の sha を読める");
    assert_ne!(sha_a, tip_sha, "前提: 先頭の便は先端でない");
    let argv = fs::read_to_string(&tools.bd_log).expect("偽 bd が撃たれた");
    let words: Vec<&str> = argv.lines().collect();
    assert_eq!(
        words.get(..3),
        Some(["close", "s2-2e5", "--reason"].as_slice()),
        "最後の close は先頭の便の bead（train_runs の a）: {words:?}"
    );
    assert_eq!(words.get(3).copied(), Some(format!("landed {sha_a} host=green").as_str()), "reason は host の緑: {words:?}");
    clean(&[&repo, &state]);

    let (repo, state) = repo_with_state();
    let (tools, stdout, ids, _) = land_train_with_terminal(&repo, &state, "failure");
    for id in &ids {
        assert_eq!(terminal_tail(&state, id), [PUSHED, CLOSED], "failure でも閉じる: {id}");
        let line = stdout.lines().find(|line| line.starts_with(&format!("run={id} landed="))).map(str::to_owned);
        assert!(line.as_deref().is_some_and(|found| found.ends_with("terminal=closed")), "stdout: {id} {stdout}");
    }
    assert!(tools.bd_log.exists(), "台帳は閉じた");
    clean(&[&repo, &state]);
}

/// (g) 先端の木の主実測が赤の周は列の便すべてが `Failed detail=main-red` で `Landed` 0 件・main は 3 本ぶん進んだまま
/// （巻き戻さない・既存の極性・どの便が赤かは帰属しない）。
#[test]
fn pipe_train_red_main_fails_every_run_and_keeps_main_advanced() {
    let (repo, state) = repo_with_state();
    let marker = state.join("lens-ran");
    let [id_a, id_b, id_c] = train_runs(&repo, &state, &marker, [MAIN_RED, ALL_GREEN, ALL_GREEN]);
    let base = git(&repo, &["rev-parse", "refs/heads/main"]);
    let rules = write_rules_train(&state, "rules-train.toml", Some(3));
    let out = land_extra(&repo, &state, &id_a, &["--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "main が赤い周は rc 1: {} / {}", stdout_of(&out), stderr_of(&out));
    let main = git(&repo, &["rev-parse", "refs/heads/main"]);
    assert_eq!(git(&repo, &["rev-list", "--count", &format!("{base}..{main}")]), "3", "main は 3 本ぶん進んだまま");
    for id in [&id_a, &id_b, &id_c] {
        assert_eq!(
            stages(&state, id).last().cloned(),
            Some((Some(Stage::Failed), Some("main-red".to_owned()))),
            "列の便はすべて main-red: {id}"
        );
        assert_eq!(landed_count(&state, id), 0, "Landed は 0 件: {id}");
    }
    assert_eq!(verdict_lines(&state), 0, "面 5 へ書かない");
    clean(&[&repo, &state]);
}

// ───── PR で着地した便は閉じた契約と `--fold-only` だけを畳む（設計 contract-source.md §61・接頭辞 `pr_retire_` と `vcipf_`） ─────
//
// fixture: PR の便（`landed_pr`）・PATH の先頭の偽 gh（撃たれないことを測る）・`--bd` の偽 client（台帳の JSON を答え、close は
// 撃たれないことを測る）。答えは file で替える。

/// retire に `--fold-only` を足して撃つ（親 `land.rs` の `retire_once` は他の畳みの歯が使うので変えない）。
fn retire_fold_only(repo: &Path, state: &Path, id: &str) -> Output {
    run_pipe(&[
        "retire", "--run", id, "--repo", &repo.display().to_string(),
        "--state-dir", &state.display().to_string(), "--fold-only",
    ])
}

/// PR の便の retire の道具一式（答えは `dir` の file）。
struct PrTools {
    repo: PathBuf,
    state: PathBuf,
    id: String,
    bead: String,
    /// 答えの file と記録の file を置く dir。
    dir: PathBuf,
    /// 偽 gh を先頭に積んだ PATH。
    path: String,
    /// 偽の台帳 client。
    bd: String,
}

impl PrTools {
    /// 答えの file を書く。
    #[expect(
        clippy::expect_used,
        reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
    )]
    fn put(&self, name: &str, body: &str) {
        fs::write(self.dir.join(name), body).expect("答えの file を書ける");
    }

    /// 記録の file の行（撃たれなければ空）。
    fn lines(&self, name: &str) -> Vec<String> {
        fs::read_to_string(self.dir.join(name)).unwrap_or_default().lines().map(str::to_owned).collect()
    }

    /// 偽の台帳の JSON の全文。
    fn ledger(&self) -> String {
        fs::read_to_string(self.dir.join("ledger")).unwrap_or_default()
    }

    /// 畳んだ先。
    fn retired(&self) -> PathBuf {
        self.repo.join(".worktrees").join("scribe2").join("retired").join(&self.id)
    }

    /// 台帳の契約を closed にして理由を `reason` にする。
    fn close_in_ledger(&self, reason: &str) {
        self.put("ledger", &format!("[{{\"id\":\"{}\",\"status\":\"closed\",\"close_reason\":\"{reason}\"}}]\n", self.bead));
    }

    /// retire を撃つ（偽 gh を PATH の先頭に・`--bd` は偽 client）。
    fn retire(&self, extra: &[&str]) -> Output {
        let (repo, state) = (self.repo.display().to_string(), self.state.display().to_string());
        let mut args = vec!["retire", "--run", &self.id, "--repo", &repo, "--state-dir", &state, "--bd", &self.bd];
        args.extend(extra);
        run_pipe_with_path(&self.path, &args)
    }

    /// 撃つと出る stdout の 1 行（通らない周の形）。
    fn refusal_line(&self, word: &str) -> String {
        format!("run={} retire={word}", self.id)
    }
}

/// PR の便と偽の道具を用意する。答えの初期値は台帳の契約が open・台帳の list rc 0。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn pr_tools() -> PrTools {
    let (repo, state) = repo_with_state();
    let design = write_contract(&repo, &[], &[]);
    let id = landed_pr(&repo, &state, &design, &state.join("lens-ran"));
    let bead = events(&state).into_iter().find(|event| event.run == id).map(|event| event.bead).unwrap_or_default();
    let dir = state.join("pr-tools");
    let bin = dir.join("bin");
    fs::create_dir_all(&bin).expect("道具の dir を作れる");
    exec_script(&bin.join("gh"), &format!("echo \"$*\" >> '{0}/gh-calls'\n", dir.display()));
    let bd = exec_script(
        &dir.join("fake-bd.sh"),
        &format!("d='{0}'\nif [ \"$1\" = close ]; then\n  echo \"$*\" >> \"$d/close-log\"\n  exit 0\nfi\ncat \"$d/ledger\"\nexit $(cat \"$d/list-rc\")\n", dir.display()),
    );
    let path = format!("{}:{}", bin.display(), crate::toolbox_path(&state));
    let tools = PrTools { repo, state, id, bead, dir, path, bd };
    tools.put("ledger", &format!("[{{\"id\":\"{}\",\"status\":\"open\",\"close_reason\":\"\"}}]\n", tools.bead));
    tools.put("list-rc", "0\n");
    tools
}

/// 断った周の後の面を測る（rc 1・stdout の閉じた語の 1 行・偽 gh と close の記録が空・worktree は元の場所・event の数と台帳の JSON は不変）。
fn assert_refused_untouched(tools: &PrTools, name: &str, word: &str, (events_before, ledger_before): (usize, String)) {
    let out = tools.retire(&[]);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "{name}: rc 1: {} / {}", stdout_of(&out), stderr_of(&out));
    assert_eq!(stdout_of(&out).trim_end(), tools.refusal_line(word), "{name}: stdout は閉じた語の 1 行");
    assert!(tools.lines("gh-calls").is_empty(), "{name}: 偽 gh は撃たれない");
    assert!(tools.lines("close-log").is_empty(), "{name}: close は撃たれない");
    assert!(worktree_of(&tools.repo, &tools.id).exists(), "{name}: worktree は元の場所");
    assert!(!tools.retired().exists(), "{name}: 畳まない");
    assert_eq!(event_count(&tools.state), events_before, "{name}: event を書かない");
    assert_eq!(tools.ledger(), ledger_before, "{name}: 台帳の JSON は不変");
}

/// PR の便の retire が畳んだ周の後の面を測る（rc 0・stdout は畳んだ先の 1 行で `close=ok` を持たない・偽 gh と close の記録が空・
/// 中身ごと畳む・元の場所は空く・branch は残る・最後の event は段 Landed の RunStage で detail retired）。
fn assert_folded_without_close(tools: &PrTools, out: &Output) {
    let id = &tools.id;
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "畳んだ周は rc 0: {} / {}", stdout_of(out), stderr_of(out));
    let retired = tools.retired();
    assert_eq!(stdout_of(out).trim(), format!("run={id} retired={}", retired.display()), "stdout は畳んだ先の 1 行");
    assert!(tools.lines("gh-calls").is_empty(), "偽 gh は撃たれない");
    assert!(tools.lines("close-log").is_empty(), "close は撃たれない");
    assert!(retired.join("src").join("lib.rs").exists(), "中身ごと畳む");
    assert!(!worktree_of(&tools.repo, id).exists(), "元の場所は空く");
    assert!(!git(&tools.repo, &["branch", "--list", &format!("scribe2/{id}")]).is_empty(), "branch は残る");
    assert_eq!(
        trail(&tools.state, id).last().cloned(),
        Some((EventKind::RunStage, Some(Stage::Landed), Some("retired".to_owned()))),
        "最後の event は Landed の RunStage で detail は retired"
    );
}

/// 閉じた契約の理由（頭の語が `landed` の見本・host の緑の尾）。
fn landed_reason() -> String {
    format!("landed {} host=green", "0".repeat(40))
}

/// (b) 通らない 3 周: worktree が clean でない（`worktree-unready`）・台帳を読めない（`unmeasured`）・契約が開いている（`not-closed`）。
/// どれも閉じた語の 1 行・rc 1・worktree は元の場所・event と偽の台帳は不変。汚れを拭って台帳を閉じた見本にした撃ち直しは畳む。
#[test]
fn pr_retire_refuses_with_one_closed_word_and_writes_nothing() {
    type Break = fn(&PrTools);
    let cases: [(&str, Break, &str); 3] = [
        ("dirty", write_dirty, "worktree-unready"),
        ("ledger-rc", |t| t.put("list-rc", "1\n"), "unmeasured"),
        ("open", |_| {}, "not-closed"),
    ];
    for (name, breakage, word) in cases {
        let tools = pr_tools();
        breakage(&tools);
        assert_refused_untouched(&tools, name, word, (event_count(&tools.state), tools.ledger()));
        if name == "dirty" {
            // 汚れを拭って台帳を閉じた見本にすると、同じ撃ちが畳む（close も照合も撃たない）。
            fs::remove_file(worktree_of(&tools.repo, &tools.id).join("dirty.txt")).ok();
            tools.close_in_ledger(&landed_reason());
            let again = tools.retire(&[]);
            assert_folded_without_close(&tools, &again);
        }
        clean(&[&tools.repo, &tools.state]);
    }
}

/// 台帳の契約が開いた周と、閉じていて理由の頭の語が `landed` でない周（`superseded`）の PR の便の retire は、語 `not-closed` の 1 行と rc 1 で
/// 断る（行 v-ci-proof-cut・閉じていない便を照合して close しない）: 偽 gh と close は撃たれず、worktree は元の場所・event の数と台帳の JSON は不変。
#[test]
fn vcipf_pr_retire_refuses_an_open_contract_with_one_word() {
    for (name, closed_reason) in [("open", None), ("superseded", Some("superseded 別の便が着地した"))] {
        let tools = pr_tools();
        if let Some(reason) = closed_reason {
            tools.close_in_ledger(reason);
        }
        assert_refused_untouched(&tools, name, "not-closed", (event_count(&tools.state), tools.ledger()));
        clean(&[&tools.repo, &tools.state]);
    }
}

/// 台帳で契約が閉じていて理由が `landed` と 40 桁の sha と `host=green` の PR の便の retire は、照合も close もせず畳む（行 v-ci-proof-cut）:
/// rc 0 と `run=<id> retired=<畳んだ先>`（`close=ok` を持たない）・偽 gh と close は撃たれず、最後の event は段 Landed の RunStage の `retired`。
#[test]
fn vcipf_pr_retire_folds_a_closed_contract_without_checks() {
    let tools = pr_tools();
    tools.close_in_ledger(&landed_reason());
    let out = tools.retire(&[]);
    assert_folded_without_close(&tools, &out);
    clean(&[&tools.repo, &tools.state]);
}

/// worktree を汚す。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn write_dirty(tools: &PrTools) {
    fs::write(worktree_of(&tools.repo, &tools.id).join("dirty.txt"), "x\n").expect("worktree を汚せる");
}

/// (c) `--fold-only` は台帳も forge も読まず畳み、契約は開いたまま残す。
#[test]
fn pr_retire_fold_only_and_closed_contracts_fold_without_checks() {
    fold_only_folds_and_leaves_the_contract_open();
}

/// (c) の本体: 契約が開いた fixture に `--fold-only` を付けた撃ち。
fn fold_only_folds_and_leaves_the_contract_open() {
    let open = pr_tools();
    let out = open.retire(&["--fold-only"]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "--fold-only は契約が開いていても畳む: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out).trim(), format!("run={} retired={}", open.id, open.retired().display()), "close=ok を名乗らない");
    assert!(open.retired().exists() && !worktree_of(&open.repo, &open.id).exists(), "畳んだ");
    assert!(open.lines("close-log").is_empty(), "close は撃たない");
    assert!(open.lines("gh-calls").is_empty(), "偽 gh は撃たれない");
    assert_eq!(open.ledger().matches("\"open\"").count(), 1, "契約は開いたまま");
    clean(&[&open.repo, &open.state]);
}
