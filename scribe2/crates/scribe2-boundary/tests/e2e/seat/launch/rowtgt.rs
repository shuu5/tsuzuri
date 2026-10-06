//! 引数の無い `seat launch` の既定の target の歯（接頭辞 `rowtgt_`・設計 docs/design/host-init.md §5）。
//! 既定の target は同じ鍵（役割 × anchor）の登録 row の target を先に引き、row が無い時だけ repo の dir 名と役割名で作る。

use super::*;

/// 引数の無い形と `--role orchestrator` だけの形は、明示の `--target` で起こして書いた row の target（`other:win`）へ起こし直す。
#[test]
fn rowtgt_bare_launch_reuses_the_registered_row_target() {
    for (case, extra) in [("引数無し", &[][..]), ("--role だけ", &["--role", "orchestrator"][..])] {
        let (place, path) = launch_default_place();
        let repo = place.dir.join(DEFAULT_REPO);
        let first = launch_default_run(&repo, &path, &["--target", "other:win"]);
        launch_default_assert_launched(&place, &first, "other:win", "git-config", case);
        let again = launch_default_run(&repo, &path, extra);
        launch_default_assert_launched(&place, &again, "other:win", "git-config", case);
        assert_eq!(acct_rows(&place.state).len(), 2, "{case}: row は 2 件");
        fs::remove_dir_all(&place.dir).ok();
    }
}

/// 鍵の anchor が違う row は読まない: 役割 orchestrator でも anchor が見本の repo と違う row だけが在る置き場では、既定の target は
/// repo の dir 名と役割名のまま。
#[test]
fn rowtgt_row_of_another_anchor_is_not_read() {
    let (place, path) = launch_default_place();
    let row = vessel::fleet::Registration {
        role: vessel::seat::role::Role::Orchestrator,
        anchor: place.dir.join("elsewhere").display().to_string(),
        target: "other:win".to_owned(),
        sid: None,
        account: "l1".to_owned(),
        launch: String::new(),
        model: None,
    };
    vessel::seat::role::register(&place.state, row).unwrap_or_else(|err| panic!("row を積める: {err:?}"));
    let out = launch_default_run(&place.dir.join(DEFAULT_REPO), &path, &[]);
    launch_default_assert_launched(&place, &out, &format!("{DEFAULT_REPO}:orchestrator"), "git-config", "別の anchor の row");
    fs::remove_dir_all(&place.dir).ok();
}

/// event log を読めない置き場（file の場所に dir）は、鍵を 1 つも送らず `log-unreadable` で断る。
#[test]
fn rowtgt_unreadable_log_refuses_before_any_key() {
    let (place, path) = launch_default_place();
    let log = place.state.join("fleet").join("events.jsonl");
    fs::remove_file(&log).ok();
    fs::create_dir_all(&log).unwrap_or_else(|err| panic!("log の場所に dir を作れる: {err}"));
    let out = launch_default_run(&place.dir.join(DEFAULT_REPO), &path, &[]);
    assert_eq!(rc_of(&out), i32::from(RC_REFUSED), "stdout={}", stdout_of(&out));
    assert!(stdout_of(&out).is_empty(), "stdout は空");
    let line = format!("seat launch: refused reason=log-unreadable target={DEFAULT_REPO}_orchestrator{}\n", provenance(&place.state, "git-config"));
    assert_eq!(stderr_of(&out), line);
    assert!(!place.dir.join(LAUNCH_TMUX_ARGS).exists(), "tmux を 1 度も撃たない");
    fs::remove_dir_all(&place.dir).ok();
}
