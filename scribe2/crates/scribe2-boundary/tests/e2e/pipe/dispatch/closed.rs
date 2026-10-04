//! 閉じた bead の便の段の歯（接頭辞 `vcrun_`・判断の記録 ADR-45 の門 H6・親 `tests/e2e/pipe/dispatch.rs` の helper を `use super::*` で使う）。
//!
//! 列の起こす側の 1 周（`pipe dispatch` の手動の 1 周）が、偽の台帳で閉じた bead の終端の便の作業木を畳み、閉じた bead の driver の
//! 居ない PASS の便を名指して起こし直しと終端の周の軸の idle の数えから外すことを外形から測る。

use super::super::{event_count, show_line, stages};
use super::*;
use vessel::fleet::Stage;

/// 閉じた bead の台帳の 1 件（acceptance の無い件＝列の候補にならない）。
fn closed_issue(bead: &str) -> String {
    listed(bead, "closed", 2, "", &[])
}

/// 閉じていない bead の台帳の 1 件（`in_progress`＝列の入力でもない）。
fn busy_issue(bead: &str) -> String {
    listed(bead, "in_progress", 2, "", &[])
}

/// bead を選んで `Implemented` まで進める（偽 runner は src/lib.rs に 1 行足して commit する）。
fn implemented_as(repo: &Path, state: &Path, bead: &str) -> String {
    let design = write_contract(repo, &[], &[]);
    let id = intake_bead(repo, state, &design, bead);
    let out = super::super::spawn_without_gate(repo, state, &id, IMPLEMENT);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{bead} の spawn は rc 0（{}）", told(&out));
    id
}

/// bead を選んで `Stopped` で終端させる（commit を 1 つ持つ clean な木が残る）。
fn stopped_as(repo: &Path, state: &Path, bead: &str) -> String {
    let id = implemented_as(repo, state, bead);
    stop_run_ok(state, &id);
    assert!(worktree_of(repo, &id).is_dir(), "前提: 止めた便の木は残る");
    id
}

/// bead を選んで `Gated` に着ける（verdict は偽 lens の 3 値・札は無い）。
fn gated_as(repo: &Path, state: &Path, bead: &str, verdict: &str) -> String {
    let id = implemented_as(repo, state, bead);
    let lens = fake_lens(&state.join(format!("vcrun-lens-{bead}")), &lens_verdict(verdict));
    let out = gate_once(repo, state, &id, Some(&lens));
    assert!(gated_pair_ok(verdict, &out), "{bead} の gate {verdict}（{}）", told(&out));
    assert!(!state.join("pipe").join(&id).join("driver").exists(), "前提: 札は無い");
    id
}

/// 手動の 1 周（起こす側・道具つき）を台帳 `bd` で撃つ。
fn round(repo: &Path, state: &Path, bd: &str) -> Output {
    run_pipe(&[
        "dispatch",
        "--state-dir", &state.display().to_string(),
        "--repo", &repo.display().to_string(),
        "--rules", &dispatch_rules(state),
        "--bd", bd,
        "--runner", "true",
    ])
}

/// stderr の `closed-runs` の行の列。
fn closed_lines(out: &Output) -> Vec<String> {
    stderr_of(out).lines().filter(|line| line.starts_with("closed-runs")).map(str::to_owned).collect()
}

/// 便の retired の置き場。
fn retired_of(repo: &Path, id: &str) -> std::path::PathBuf {
    repo.join(".worktrees").join("scribe2").join("retired").join(id)
}

/// 閉じた bead の終端の便（`Stopped`・`Gated` の verdict FAIL）の作業木を 1 周で retired の下へ移し、便ごとに段のままの
/// `RunStage detail=retired` を 1 件積み、stderr に `closed-runs folded=2 failed=0 live=0 ids=-` の 1 行を出す。branch と main は
/// 動かさず、2 周目は何も移さず行も出さない。
#[test]
fn vcrun_binary_folds_settled_runs_of_closed_beads() {
    let (repo, state) = repo_with_state();
    let stopped = stopped_as(&repo, &state, "cr-a");
    let failed = gated_as(&repo, &state, "cr-b", "FAIL");
    let main_before = git(&repo, &["rev-parse", "refs/heads/main"]);
    let bd = fake_bd(&state, &[closed_issue("cr-a"), closed_issue("cr-b")]);
    let out = round(&repo, &state, &bd);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "1 周は rc 0（{}）", told(&out));
    assert_eq!(closed_lines(&out), ["closed-runs folded=2 failed=0 live=0 ids=-"], "畳んだ 2 本の行（{}）", told(&out));
    for (id, stage) in [(&stopped, Stage::Stopped), (&failed, Stage::Gated)] {
        assert!(!worktree_of(&repo, id).exists(), "{id} の元の場所が空く");
        assert!(retired_of(&repo, id).join("src").join("lib.rs").exists(), "{id} は中身ごと retired の下へ");
        assert_eq!(stages(&state, id).last().cloned(), Some((Some(stage), Some("retired".to_owned()))), "{id} の段はそのまま");
        let branch = git(&repo, &["branch", "--list", &format!("scribe2/{id}")]);
        assert!(!branch.trim().is_empty(), "{id} の branch は消さない");
    }
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main"]), main_before, "main は動かない");
    let after = event_count(&state);
    let again = round(&repo, &state, &bd);
    assert!(closed_lines(&again).is_empty(), "2 周目は行を出さない（{}）", told(&again));
    assert_eq!(event_count(&state), after, "2 周目は記帳しない");
    clean(&[&repo, &state]);
}

/// 正しい見本（閉じた bead の clean な木の `Stopped`）から 1 句ずつ外した便は移さず記帳しない: bead が閉じていない・木が clean で
/// ない・木が無い・終端でない（`Implemented`）。見る側の 1 周（`dispatch ls`）は正しい見本も移さず、起こす側の 1 周は正しい見本だけを移す。
#[test]
fn vcrun_binary_keeps_open_live_dirty_and_absent_trees() {
    let (repo, state) = repo_with_state();
    let kept = stopped_as(&repo, &state, "cr-p");
    let open = stopped_as(&repo, &state, "cr-o");
    let dirty = stopped_as(&repo, &state, "cr-d");
    fs::write(worktree_of(&repo, &dirty).join("scratch.txt"), "x\n").unwrap_or_else(|err| panic!("木を汚せる: {err}"));
    let absent = stopped_as(&repo, &state, "cr-x");
    let away = state.join("moved-away");
    git(&repo, &["worktree", "move", &worktree_of(&repo, &absent).display().to_string(), &away.display().to_string()]);
    let live = implemented_as(&repo, &state, "cr-l");
    let ledger = [closed_issue("cr-p"), busy_issue("cr-o"), closed_issue("cr-d"), closed_issue("cr-x"), closed_issue("cr-l")];
    let bd = fake_bd(&state, &ledger);
    let before = event_count(&state);
    let seen = ls(&repo, &state, &bd);
    assert_eq!(seen.status.code(), Some(i32::from(RC_OK)), "見る側は rc 0（{}）", told(&seen));
    assert!(worktree_of(&repo, &kept).is_dir(), "見る側は正しい見本も移さない");
    assert_eq!(event_count(&state), before, "見る側は記帳しない");
    let out = round(&repo, &state, &bd);
    assert_eq!(closed_lines(&out), ["closed-runs folded=1 failed=0 live=0 ids=-"], "正しい見本だけを畳む（{}）", told(&out));
    assert!(retired_of(&repo, &kept).is_dir(), "正しい見本は畳まれる");
    assert_eq!(event_count(&state), before + 1, "記帳は正しい見本の 1 件だけ");
    for id in [&open, &dirty, &live] {
        assert!(worktree_of(&repo, id).is_dir(), "{id} の木は残る");
        assert!(!retired_of(&repo, id).exists(), "{id} は retired の下に無い");
    }
    assert!(away.is_dir() && !retired_of(&repo, &absent).exists(), "木の無い便は何もしない");
    assert!(worktree_of(&repo, &dirty).join("scratch.txt").exists(), "汚れた木の中身は動かない");
    clean(&[&repo, &state]);
}

/// 閉じた bead の verdict PASS の `Gated` の便で札が無いか所有者が死んでいる便は、移さず止めず起こし直さず、stderr の行に
/// `live=1 ids=<便 id>` が載る。札の所有者が生きている便と閉じていない bead の便は名指さない（閉じていない bead の札の無い便は
/// 今までどおり起こし直す）。
#[test]
fn vcrun_binary_names_live_runs_of_closed_beads() {
    let (repo, state) = repo_with_state();
    let id = gated_as(&repo, &state, "cr-g", "PASS");
    let closed = fake_bd(&state, &[closed_issue("cr-g")]);
    let named = format!("closed-runs folded=0 failed=0 live=1 ids={id}");
    let before = event_count(&state);
    let absent = round(&repo, &state, &closed);
    assert_eq!(closed_lines(&absent), std::slice::from_ref(&named), "札の無い亡骸を名指す（{}）", told(&absent));
    assert_eq!(stdout_of(&absent).trim_end(), resumed_line(0), "亡骸は起こし直さない（{}）", told(&absent));
    put_dead_ticket(&state, &id);
    let dead = round(&repo, &state, &closed);
    assert_eq!(closed_lines(&dead), [named], "所有者の死んだ札の亡骸も名指す（{}）", told(&dead));
    assert_eq!(stdout_of(&dead).trim_end(), resumed_line(0), "死んだ札でも起こし直さない（{}）", told(&dead));
    assert_eq!(event_count(&state), before, "名指すだけで記帳しない");
    assert!(worktree_of(&repo, &id).is_dir(), "亡骸の木は移さない");
    put_ticket_body(&state, &id, &format!("{}\n", std::process::id()));
    let alive = round(&repo, &state, &closed);
    assert!(closed_lines(&alive).is_empty(), "所有者の生きた札の便は名指さない（{}）", told(&alive));
    fs::remove_file(state.join("pipe").join(&id).join("driver")).unwrap_or_else(|err| panic!("札を外せる: {err}"));
    let open = round(&repo, &state, &fake_bd(&state, &[busy_issue("cr-g")]));
    assert!(closed_lines(&open).is_empty(), "閉じていない bead の便は名指さない（{}）", told(&open));
    assert_eq!(stdout_of(&open).trim_end(), resumed_line(1), "閉じていない bead の札の無い便は起こし直す（{}）", told(&open));
    assert_eq!(stage_reached(&state, &id, "Landed"), 1, "起こし直した便は着地する（段の並び: {}）", stages_of(&state, &id));
    clean(&[&repo, &state]);
}

/// 終端の周の軸の idle の数えは閉じた bead の亡骸（審査 PASS の `Reviewed`・札なし）を live に数えず、亡骸だけが残る置き場の
/// 1 周は上流の遅れを読む前に fetch を撃つ。同じ便の bead が閉じていない周は live に数えて fetch を撃たない。
#[test]
fn vcrun_binary_idle_ignores_live_runs_of_closed_beads() {
    let (repo, state) = repo_with_state();
    let design = write_contract(&repo, &[], &[]);
    let id = intake_bead(&repo, &state, &design, "cr-r");
    assert!(show_line(&repo, &state, &id).contains("stage=Reviewed"), "前提: 審査 PASS の Reviewed");
    assert!(!state.join("pipe").join(&id).join("driver").exists(), "前提: 札は無い");
    let place = vessel_place(&state, true, 0, "", 0);
    let turn = |bd: &str| {
        let (state_s, repo_s, rules) = (state.display().to_string(), repo.display().to_string(), dispatch_rules(&state));
        let args = ["dispatch", "--state-dir", &state_s, "--repo", &repo_s, "--rules", &rules, "--bd", bd, "--runner", "true"];
        super::super::run_pipe_with_path(&place.path, &args)
    };
    let open = turn(&fake_bd(&state, &[busy_issue("cr-r")]));
    assert_eq!(stdout_of(&open), format!("vessel=current\n{IDLE_LINE}\n"), "閉じていない bead の便が残る（{}）", told(&open));
    assert_eq!(place.argv(), ["git -C [vessel] rev-list --count HEAD..origin/main"], "live が残る周は fetch を撃たない");
    fs::remove_file(&place.log).unwrap_or_else(|err| panic!("argv の写しを空にできる: {err}"));
    let closed = turn(&fake_bd(&state, &[closed_issue("cr-r")]));
    assert_eq!(stdout_of(&closed), format!("vessel=current\n{IDLE_LINE}\n"), "亡骸だけが残る（{}）", told(&closed));
    assert_eq!(closed_lines(&closed), [format!("closed-runs folded=0 failed=0 live=1 ids={id}")], "亡骸を名指す（{}）", told(&closed));
    assert_eq!(place.argv(), fetch_then_count(), "亡骸だけの周は fetch を撃ってから数える");
    clean(&[&repo, &state]);
}

/// 台帳を読めない周（偽の台帳が rc 3）は閉じた bead の終端の便も 1 本も移さず、行も出さず、記帳もしない。
#[test]
fn vcrun_binary_ledger_unreadable_moves_nothing() {
    let (repo, state) = repo_with_state();
    let id = stopped_as(&repo, &state, "cr-u");
    let broken = script(&state.join("bd-broken"), "exit 3\n");
    let before = event_count(&state);
    let out = round(&repo, &state, &broken);
    assert_eq!(stdout_of(&out).trim_end(), "dispatch=unmeasured reason=ledger", "台帳を読めない周（{}）", told(&out));
    assert!(closed_lines(&out).is_empty(), "行を出さない（{}）", told(&out));
    assert!(worktree_of(&repo, &id).is_dir(), "木は残る");
    assert_eq!(event_count(&state), before, "記帳しない");
    let readable = round(&repo, &state, &fake_bd(&state, &[closed_issue("cr-u")]));
    assert_eq!(closed_lines(&readable), ["closed-runs folded=1 failed=0 live=0 ids=-"], "読める周は畳む（{}）", told(&readable));
    clean(&[&repo, &state]);
}
