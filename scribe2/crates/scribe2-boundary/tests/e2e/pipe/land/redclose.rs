//! 便の終わりの段を 1 つに定める歯（接頭辞 `vredc_` と `vcipf_`・判断の記録 ADR-45 の門 H6・親 `tests/e2e/pipe/land.rs` の helper を `use super::*` で使う）。
//!
//! 着地の口が squash の前に段を読み直すこと（列の先頭が着地させた便の撃ち直しを止める）と、終端だけの撃ち直しが `Failed` の便を
//! 段の前提で断って何も撃たないことを外形から測る。

use super::*;

/// 便の event の 1 行を、段 `Landed` の `RunDone`（detail `sha:<sha> main:<sha>`＝列の先頭が着地させた形）に替えた行。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn landed_line(state: &Path, id: &str, sha: &str) -> String {
    let last = events(state).into_iter().rfind(|event| event.run == id).expect("便の event が在る");
    let landed = vessel::fleet::Event {
        kind: EventKind::RunDone,
        stage: Some(Stage::Landed),
        detail: Some(format!("sha:{sha} main:{sha}")),
        ..last
    };
    landed.to_line()
}

/// toy repo の `reference-transaction` hook: 最初の `committed` の段で `line` を event log の末に 1 行足す（**1 度だけ**・印を
/// `marker` に置く）。land の追随の rebase（便の branch の更新）が最初の ref の書きで、番待ちの直後の段の読みの後に列の先頭が自分を
/// 着地させた形を land の外の手で作る。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn install_landing_hook(repo: &Path, state: &Path, line: &str, marker: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let body = state.join("landed-line.jsonl");
    fs::write(&body, format!("{line}\n")).expect("足す行を書ける");
    let hook = repo.join(".git").join("hooks").join("reference-transaction");
    fs::create_dir_all(hook.parent().expect("hooks の親 dir")).expect("hooks dir を作れる");
    let script = format!(
        "#!/bin/sh\n[ \"$1\" = committed ] || exit 0\n[ -f '{marker}' ] && exit 0\ntouch '{marker}'\ncat '{body}' >> '{log}'\nexit 0\n",
        marker = marker.display(),
        body = body.display(),
        log = state.join("fleet").join("events.jsonl").display()
    );
    fs::write(&hook, script).expect("hook を書ける");
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).expect("hook に実行権を付ける");
}

/// 既着地の追随の fixture（main に自分の squash・主実測の共通 verify は赤）で、追随の rebase の後に段が `Landed` になった周の land は
/// squash も CAS も主実測も撃たず、stdout に `already-landed=1` を出して rc 0 で返り、`Failed` を記帳しない。段の読み直しの無い周
/// （hook を置かない同じ fixture）は今までどおり主実測を撃って `Failed main-red` で落ちる。
#[test]
fn vredc_land_rereads_the_stage_before_the_squash() {
    for hooked in [true, false] {
        let (repo, state) = repo_with_state();
        let marker = state.join("lens-ran");
        let (id, squash) = gated_run_squashed_on_main(&repo, &state, &marker, run_trailer);
        make_tree_differ(&repo, &state, &id, &format!("{squash}^"));
        let frozen = vessel::pipe::vessel_path(&state, &id);
        let text = fs::read_to_string(&frozen).unwrap_or_else(|err| panic!("宣言の写しを読める: {err}"));
        let common = format!("common-verify = {VESSEL_COMMON}\n");
        fs::write(&frozen, text.replace(&common, "common-verify = [\"sh verify-red.sh\"]\n")).unwrap_or_else(|err| panic!("写しを書ける: {err}"));
        let hook_ran = state.join("landing-hook-ran");
        if hooked {
            install_landing_hook(&repo, &state, &landed_line(&state, &id, &squash), &hook_ran);
        }
        let out = land_once(&repo, &state, &id);
        let measured = state.join("pipe").join(&id).join("verify-main.jsonl").exists();
        let failed = trail(&state, &id).iter().any(|(_, stage, _)| *stage == Some(Stage::Failed));
        assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), squash, "hooked={hooked}: main は動かない");
        if hooked {
            assert!(hook_ran.exists(), "前提: 追随の rebase で hook が段を Landed にした");
            assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "読み直した周は rc 0: {}", stderr_of(&out));
            assert!(stdout_of(&out).contains(&format!("run={id} already-landed=1")), "already-landed の行: {}", stdout_of(&out));
            assert!(!measured, "主実測を撃たない（verify-main.jsonl が無い）");
            assert!(!failed, "Failed を記帳しない: {:?}", trail(&state, &id));
            assert_eq!(landed_details(&state, &id), [format!("sha:{squash} main:{squash}")], "Landed の記帳は列の先頭の 1 件だけ");
        } else {
            assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "読み直しの無い周は main-red の rc 1: {}", stderr_of(&out));
            assert!(measured && failed, "読み直しの無い周は主実測を撃って Failed で落ちる");
        }
        clean(&[&repo, &state]);
    }
}

/// main-red の便の fixture: 偽 remote と success を答える偽 CI を宣言した toy で、主実測だけが赤い便を land して `Failed main-red`
/// で終わらせ（squash は local の main に残る）、後の便の commit を 1 つ積んで偽 remote の main へ押す。返すのは道具・便・`--bd` と
/// `--rules` の値・squash・押した先端。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn red_run(repo: &Path, state: &Path) -> (FakeTerminal, String, [String; 2], String, String) {
    let json = "[{\"status\":\"completed\",\"conclusion\":\"success\"}]";
    let tools = fake_terminal_json(repo, state, json);
    let design = write_contract(repo, &["verify"], &[r#"verify = ["sh verify-once.sh"]"#]);
    let id = gated_pass(repo, state, &design, &state.join("lens-ran"));
    make_tree_differ(repo, state, &id, "refs/heads/main");
    let red = land_once(repo, state, &id);
    assert_eq!(red.status.code(), Some(i32::from(RC_REFUSED)), "前提: main-red の rc 1: {}", stderr_of(&red));
    assert_eq!(stages(state, &id).last().cloned(), Some((Some(Stage::Failed), Some("main-red".to_owned()))), "前提: Failed main-red");
    let squash = git(repo, &["rev-parse", "refs/heads/main"]);
    fs::write(repo.join("later.md"), "後の便\n").expect("後の便の file を書ける");
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-q", "-m", "later-run"]);
    let tip = git(repo, &["rev-parse", "refs/heads/main"]);
    git(repo, &["push", "-q", "fake", "main:main"]);
    // 押していない local だけの commit（終端の撃ち直しが push を撃てば偽 remote の main が動く形）。
    fs::write(repo.join("local.md"), "local だけ\n").expect("local の file を書ける");
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-q", "-m", "local-only"]);
    let flags = [state.join("fake-bd.sh").display().to_string(), ceiling_rules(state)];
    (tools, id, flags, squash, tip)
}

/// 終端だけを撃ち直す（`pipe land --terminal-only`・`--bd` と `--rules` は fixture の値）。
fn terminal_only(repo: &Path, state: &Path, id: &str, flags: &[String; 2]) -> Output {
    land_extra(repo, state, id, &["--bd", &flags[0], "--rules", &flags[1], "--terminal-only"])
}

/// `Failed main-red` の便（squash が偽 remote の main の先端の祖先に在り、偽 CI は success を答える）の終端だけの撃ち直しは、retire で
/// 木を畳む前も後も段の前提で断る（行 v-ci-proof-cut・host の緑の無い便を黙って閉じない）: rc 1 と stderr の `run <id> の段は Failed である`、
/// stdout は空で、event の数は不変・偽 CI と偽 bd は撃たれず、偽 remote の main は押した先端のまま。
#[test]
fn vcipf_terminal_only_refuses_a_failed_run() {
    for folded in [false, true] {
        let (repo, state) = repo_with_state();
        let (tools, id, flags, _, tip) = red_run(&repo, &state);
        if folded {
            let out = retire_once(&repo, &state, &id);
            assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "前提: main-red の便の木を畳める: {}", stderr_of(&out));
        }
        let before = event_count(&state);
        let out = terminal_only(&repo, &state, &id, &flags);
        assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "folded={folded}: rc 1: {}", stderr_of(&out));
        assert!(stderr_of(&out).contains(&format!("run {id} の段は Failed である")), "folded={folded}: 段の断り: {}", stderr_of(&out));
        assert_eq!(stdout_of(&out), "", "folded={folded}: stdout は空");
        assert_eq!(event_count(&state), before, "folded={folded}: 何も積まない");
        assert_eq!(tools.ci_call_count(), 0, "folded={folded}: 偽 CI は撃たれない");
        assert!(!tools.ci_log.exists(), "folded={folded}: 偽 CI の argv の記録も無い");
        assert!(!tools.bd_log.exists(), "folded={folded}: 偽 bd は撃たれない");
        assert_eq!(git(&tools.remote, &["rev-parse", "refs/heads/main"]), tip, "folded={folded}: 偽 remote の main は動かない");
        clean(&[&repo, &state]);
    }
}
