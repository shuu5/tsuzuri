//! 管理 tick の周の空の scope の片付けの歯（接頭辞 `vscpe_`・判断の記録 ADR-45 の門 H6・契約表の行 v-scope-empty）。
//!
//! 置き場は tick の歯の fixture（[`tick_place`]・登録 row・偽 tmux）を使い、`systemctl` は偽の bin の script が引数を 1 行ずつ
//! [`TICK_SCOPE_CALLS`] に残す（host の systemd を 1 度も撃たない）。一覧と show の答えは歯ごとに script へ書く（[`scope_shim`]）。
//! 呼出の字面は契約から組む（実装の導出を使わない）。

use super::*;

/// 一覧の呼出の字面（器の名の scope の active な行だけ・legend と pager なし）。
fn list_call() -> String {
    format!("--user list-units --no-legend --no-pager --plain --type=scope --state=active {NAME}-*.scope")
}

/// unit の task の数を引く呼出の字面。
fn show_call(unit: &str) -> String {
    format!("--user show {unit}.scope -p TasksCurrent --value")
}

/// unit を止める呼出の字面（待たない）。
fn stop_call(unit: &str) -> String {
    format!("--user stop --no-block {unit}.scope")
}

/// 待ち終えた `true` の pid（死んだ作り手）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn dead_pid() -> u32 {
    let mut gone = Command::new("true").spawn().expect("true を起こせる");
    let pid = gone.id();
    gone.wait().expect("true を待てる");
    pid
}

/// 一覧の 5 本と unit ごとの task の数: 正しい見本（作り手が死に段 common で task 0）と、そこから 1 句ずつ外した 4 本（task 2・
/// 作り手が生きている・段の語 seat・task が字 `[not set]`）。
fn five(dead: u32) -> [(String, &'static str); 5] {
    let own = std::process::id();
    [
        (format!("{NAME}-s2-kill-1-common-2-{dead}-2"), "0"),
        (format!("{NAME}-s2-kill-2-common-2-{dead}-3"), "2"),
        (format!("{NAME}-s2-kill-3-common-2-{own}-0"), "0"),
        (format!("{NAME}-{TICK_SEAT}-seat-0-{dead}-0"), "0"),
        (format!("{NAME}-s2-kill-4-common-2-{dead}-4"), "[not set]"),
    ]
}

/// 偽 systemctl を書き直す: 呼出を [`TICK_SCOPE_CALLS`] に 1 行残し、`list-units` は `units` を一覧の行で出して rc `list_rc`、
/// `show` は unit ごとの task の数を出して rc 0（`units` に無い unit は rc 1）、ほかは rc 0。
fn scope_shim(place: &TickPlace, units: &[(String, &str)], list_rc: u8) {
    let listed: String = units.iter().map(|(unit, _)| format!("printf '%s loaded active running x\\n' '{unit}.scope'\n")).collect();
    let shown: String = units.iter().map(|(unit, tasks)| format!("'{unit}.scope') printf '%s\\n' '{tasks}'; exit 0;;\n")).collect();
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{calls}'\ncase \"$2\" in\nlist-units)\n{listed}exit {list_rc};;\n\
         show)\ncase \"$3\" in\n{shown}esac\nexit 1;;\nesac\nexit 0\n",
        calls = place.at(TICK_SCOPE_CALLS).display()
    );
    crate::install_shim(&place.dir.join("bin").join("systemctl"), &script);
}

/// 偽 systemctl の呼出（引数の行・呼んだ順）。
fn scope_calls(at: &Path) -> Vec<String> {
    fs::read_to_string(at).unwrap_or_default().lines().map(str::to_owned).collect()
}

/// tick の外形（rc・stdout・stderr）。
fn told(out: &Output) -> (i32, String, String) {
    (rc_of(out), stdout_of(out), stderr_of(out))
}

/// 置き場の event log の byte（無ければ空）。
fn log_bytes(place: &TickPlace) -> Vec<u8> {
    fs::read(vessel::fleet::store::events_path(&place.state)).unwrap_or_default()
}

/// 登録 row の在る席の tick の周は、一覧の 5 本のうち作り手が死に段の語が seat でない 3 本にだけ show を撃ち、中の task がちょうど
/// 0 の 1 本だけに stop を `--no-block` で 1 回撃ち、kill と reset-failed を撃たない。tick の rc・stdout・stderr は一覧 0 行の対照
/// と同じで、event log の byte は tick の前と同じ。
#[test]
fn vscpe_tick_stops_only_the_empty_scope_of_a_dead_creator() {
    let control = tick_place(true);
    let expected = told(&tick_run_bare(&control, &[]));
    let place = tick_place(true);
    let units = five(dead_pid());
    scope_shim(&place, &units, 0);
    let before = log_bytes(&place);
    let out = tick_run_bare(&place, &[]);
    let [(empty, _), (busy, _), _, _, (unset, _)] = &units;
    let want = [list_call(), show_call(empty), show_call(busy), show_call(unset), stop_call(empty)];
    assert_eq!(scope_calls(&place.at(TICK_SCOPE_CALLS)), want, "一覧 1 回・show 3 回・stop 1 回だけ: stderr={}", stderr_of(&out));
    assert_eq!(told(&out), expected, "rc と判定行と stderr は対照と同じ");
    assert_eq!(log_bytes(&place), before, "event log は替わらない");
}

/// 正しい見本から 1 句ずつ外した周: 一覧が rc 0 でない周（偽が同じ 5 本の行を出して rc 1）は show も stop も撃たず（呼出は一覧の
/// 1 行だけ）tick の rc・stdout・stderr は対照と同じで、登録 row の無い target の周（event log は在る）は systemctl を撃たない。
#[test]
fn vscpe_tick_shoots_only_the_list_when_it_fails_and_nothing_without_a_row() {
    let control = tick_place(true);
    let expected = told(&tick_run_bare(&control, &[]));
    let units = five(dead_pid());
    let failed = tick_place(true);
    scope_shim(&failed, &units, 1);
    let out = tick_run_bare(&failed, &[]);
    assert_eq!(scope_calls(&failed.at(TICK_SCOPE_CALLS)), [list_call()], "一覧の 1 回だけ: stderr={}", stderr_of(&out));
    assert_eq!(told(&out), expected, "rc と判定行と stderr は対照と同じ");
    let other = tick_place(true);
    scope_shim(&other, &units, 0);
    let state = other.state.display().to_string();
    let out = Command::new(bin())
        .args(["seat", "tick", "--state-dir", &state, "--target", "tk:other"])
        .env("PATH", &other.path)
        .output()
        .expect("binary を起動できる");
    assert_eq!(rc_of(&out), i32::from(RC_OK), "stderr={}", stderr_of(&out));
    assert!(stdout_of(&out).contains(" reason=no-row "), "登録 row の無い target: {}", stdout_of(&out));
    assert!(scope_calls(&other.at(TICK_SCOPE_CALLS)).is_empty(), "systemctl 0 回");
}

/// 着地済みの tick の歯の置き場 2 つ（[`tick_shims`] と [`move_shims`] の組む PATH・host の PATH を後ろに持つ）で撃つ登録 row の
/// 在る tick の周は、systemctl の呼出が偽の bin の systemctl に届き、記録は一覧の 1 行だけ（host の systemd に届かない）。
#[test]
fn vscpe_tick_places_of_the_landed_teeth_answer_the_list_with_a_fake() {
    let place = tick_place(true);
    let out = tick_run_bare(&place, &[]);
    assert_eq!(rc_of(&out), i32::from(RC_OK), "stderr={}", stderr_of(&out));
    assert_eq!(scope_calls(&place.at(TICK_SCOPE_CALLS)), [list_call()], "tick_shims の偽");
    let root = tmp();
    let moved = move_place(&root, "state", MOVE_ANCHOR);
    let out = move_run(&moved);
    assert_eq!(rc_of(&out), i32::from(RC_OK), "stderr={}", stderr_of(&out));
    assert_eq!(scope_calls(&moved.at(TICK_SCOPE_CALLS)), [list_call()], "move_shims の偽");
}
