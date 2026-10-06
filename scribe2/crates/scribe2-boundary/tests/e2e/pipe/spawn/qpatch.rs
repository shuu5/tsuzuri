//! 質問の判じが、載せ替えで sha だけが替わった便自身の commit を数えない歯（接頭辞 `qpatch_`・設計
//! docs/design/pipeline-conflict.md §11・親 `tests/e2e/pipe/spawn.rs` の helper を `use super::*` で使う）。
//!
//! turn 1 は赤い中身を commit して終わりの門を赤にし、turn 2（起こし直し）は repo の main を別 file の 1 commit で進めて
//! `git rebase refs/heads/main` で前の周の commit を載せ替える。その後で質問の record を出す。

use super::*;

/// turn 2 の頭: main を別 file `other.txt` の 1 commit で進め、worktree で `git rebase refs/heads/main` を撃つ。
fn rebase_onto_moved_main(repo: &Path) -> String {
    format!(
        "printf 'o\\n' > '{repo}/other.txt'\ngit -C '{repo}' add other.txt\ngit -C '{repo}' commit -q -m other\n\
         git rebase refs/heads/main\n",
        repo = repo.display()
    )
}

/// 質問の record を出して rc 76 で終える本文。
const ASK_AFTER: &str = "printf '%s\\n' '{\"question\":\"載せ替えの後で聞く\",\"about\":\"design\"}'\nexit 76";

/// 便を intake し、turn 1 は赤い commit・turn 2 は載せ替えの後に `after` を撃つ偽 runner で spawn する（repo・置き場・便 id・出力）。
fn replayed_then(after: &str) -> (PathBuf, PathBuf, String, Output) {
    let (repo, state, id) = gate_run_intake();
    let second = format!("{}{after}", rebase_onto_moved_main(&repo));
    let runner = turn_runner(&state, &[commit_turn("red-one"), second]);
    let out = end_gate_spawn(&repo, &state, &id, &runner, None);
    (repo, state, id, out)
}

/// 段の最後が `Failed detail=runner-rc:76,commits:1` で、質問は記帳されていない。
fn assert_failed_with_one_commit(state: &Path, id: &str) {
    assert_eq!(
        stages(state, id).last().cloned(),
        Some((Some(Stage::Failed), Some("runner-rc:76,commits:1".to_owned()))),
        "turn で増えた新しい変更を数える: {:?}",
        stages(state, id)
    );
    assert!(
        !trail(state, id).iter().any(|(kind, _, _)| *kind == EventKind::QuestionRaised),
        "質問は記帳しない"
    );
}

/// 載せ替えただけで（code を書かずに）質問した周は `Questioned`: sha が替わった前の周の commit は数えない。
#[test]
fn qpatch_replayed_commit_then_asking_stops_at_questioned() {
    let (repo, state, id, out) = replayed_then(ASK_AFTER);
    assert_eq!(out.status.code(), Some(i32::from(RC_BLOCKED)), "質問は rc 3: {} / {}", stdout_of(&out), stderr_of(&out));
    assert_eq!(
        stages(&state, &id).last().map(|(stage, _)| *stage),
        Some(Some(Stage::Questioned)),
        "段は Questioned: {:?}",
        stages(&state, &id)
    );
    let raised = trail(&state, &id).iter().filter(|(kind, _, _)| *kind == EventKind::QuestionRaised).count();
    assert_eq!(raised, 1, "質問の逐語が 1 件残る");
    assert_eq!(stub_calls(&state), 2, "runner は 2 回起きる");
    assert_eq!(
        git(&worktree_of(&repo, &id), &["rev-parse", "HEAD~1"]),
        git(&repo, &["rev-parse", "refs/heads/main"]),
        "載せ替えた commit は進めた main の真上に 1 本"
    );
    clean(&[&repo, &state]);
}

/// 載せ替えの後で新しい commit を 1 本作ってから質問した周は `Failed`（載せ替えた commit だけを打ち消す）。
#[test]
fn qpatch_new_commit_after_replay_is_a_failure() {
    let after = format!("printf 'z\\n' >> src/lib.rs\ngit add -A\ngit commit -q -m extra\n{ASK_AFTER}");
    let (repo, state, id, _out) = replayed_then(&after);
    assert_failed_with_one_commit(&state, &id);
    clean(&[&repo, &state]);
}

/// 載せ替えた commit の中身を `--amend` で替えてから質問した周は `Failed`（sha と中身の両方が替わった commit は
/// patch-id が合わないので数える＝数の差でなく patch-id で照らす）。
#[test]
fn qpatch_amended_replay_is_a_failure() {
    let after = format!("printf 'q\\n' >> src/lib.rs\ngit commit -q -a --amend --no-edit\n{ASK_AFTER}");
    let (repo, state, id, _out) = replayed_then(&after);
    assert_failed_with_one_commit(&state, &id);
    clean(&[&repo, &state]);
}
