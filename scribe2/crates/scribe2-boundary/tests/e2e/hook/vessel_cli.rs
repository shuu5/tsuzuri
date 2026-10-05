// flip-check: moved s2-07l.679
//! vessel の口の族の歯（接頭辞 `vessel_init_` / `vessel_update_` / `vessel_check_` / `vessel_args_` / `vessel_marker_`・設計 docs/design/carry-prep.md §9 行 g）。

use super::*;

#[test]
fn vessel_init_renders_two_lines_and_writes_state_dir_config() {
    let repo = git_repo();
    let state = linked(&repo);

    let text = fs::read_to_string(repo.join(MARKER)).expect("marker を読める");
    assert!(text.ends_with('\n'), "末尾改行が在る: {text:?}");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "marker は 2 行（母集団 {}）", lines.len());
    assert_eq!(lines.first().copied(), Some(format!("name={NAME}").as_str()));
    assert_eq!(
        lines.get(1).copied(),
        Some(format!("version={GENERATION}").as_str())
    );

    let configured = git(&repo, &["config", "--get", &format!("{NAME}.stateDir")]);
    assert_eq!(
        configured,
        state.display().to_string(),
        "state dir は repo の local 設定に在る"
    );
    clean(&[&repo, &state]);
}

#[test]
fn vessel_check_rc2_for_other_name() {
    let repo = git_repo();
    let other = Marker {
        name: "other-vessel".to_owned(),
        version: GENERATION,
    };
    fs::write(repo.join(MARKER), other.render()).expect("marker を書ける");
    let out = run_vessel(&["check", &repo.display().to_string()]);
    assert_eq!(
        out.status.code(),
        Some(i32::from(RC_BROKEN)),
        "別の器が名乗る repo は rc 2"
    );

    fs::remove_file(repo.join(MARKER)).expect("marker を消せる");
    let absent = run_vessel(&["check", &repo.display().to_string()]);
    assert_eq!(
        absent.status.code(),
        Some(i32::from(RC_REFUSED)),
        "名乗りが無い repo は rc 1"
    );
    clean(&[&repo]);
}

#[test]
fn vessel_init_refuses_to_overwrite_other_name() {
    let repo = git_repo();
    let other = Marker {
        name: "other-vessel".to_owned(),
        version: GENERATION,
    };
    let before = other.render();
    fs::write(repo.join(MARKER), &before).expect("marker を書ける");
    let state = tmp();

    let out = run_vessel(&[
        "init",
        "--state-dir",
        &state.display().to_string(),
        &repo.display().to_string(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(i32::from(RC_BROKEN)),
        "他の器の repo は奪わない（rc 2）"
    );
    let after = fs::read_to_string(repo.join(MARKER)).expect("marker を読める");
    assert_eq!(after, before, "1 byte も書き換えない");
    clean(&[&repo, &state]);
}

#[test]
fn vessel_check_needs_marker_and_state_dir() {
    let repo = git_repo();
    let mine = Marker {
        name: NAME.to_owned(),
        version: GENERATION,
    };
    fs::write(repo.join(MARKER), mine.render()).expect("marker を書ける");
    let out = run_vessel(&["check", &repo.display().to_string()]);
    assert_eq!(
        out.status.code(),
        Some(i32::from(RC_REFUSED)),
        "marker だけでは仕えない（state dir が紐づいて初めて ByMe）"
    );
    let state = linked(&repo);
    let out = run_vessel(&["check", &repo.display().to_string()]);
    assert_eq!(
        out.status.code(),
        Some(i32::from(RC_OK)),
        "紐づけば ByMe（rc 0）"
    );
    clean(&[&repo, &state]);
}

#[test]
fn vessel_marker_is_two_lf_lines() {
    let repo = git_repo();
    let state = linked(&repo);
    let bytes = fs::read(repo.join(MARKER)).expect("marker を読める");
    // 跨版 面 1 は別実装が byte で読む。LF・順序・末尾改行・余白なしを byte で pin する。
    assert_eq!(
        bytes,
        format!("name={NAME}\nversion={GENERATION}\n").into_bytes(),
        "marker の byte 列: {:?}",
        String::from_utf8_lossy(&bytes)
    );
    clean(&[&repo, &state]);
}

#[test]
fn vessel_init_leaves_no_marker_without_git_repo() {
    let bare = tmp();
    let state = tmp();
    let out = run_vessel(&[
        "init",
        "--state-dir",
        &state.display().to_string(),
        &bare.display().to_string(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(i32::from(RC_BROKEN)),
        "git repo でなければ rc 2"
    );
    assert!(
        !bare.join(MARKER).exists(),
        "設定に失敗した周に marker だけを残さない"
    );
    clean(&[&bare, &state]);
}

/// (8) `vessel` の口: 既知の 4 verb（`update` は consumer-sync.md §5 が足した）に**未知の flag** を足すと rc 2・理由の 1 行が flag を名指し usage を添え・marker も git の
/// local 設定も 1 byte も書かない。`--help` は usage を stdout へ出して rc 0（設計 pipeline.md §14 約束 3 / 4 / 8）。base の
/// `init` は未知の flag を読み飛ばして marker を書く＝RED。
#[test]
fn vessel_args_unknown_flag_is_refused_with_rc_2_on_every_verb() {
    let repo = git_repo();
    let state = tmp();
    let (root, dir) = (repo.display().to_string(), state.display().to_string());
    let config = repo.join(".git").join("config");
    let before = fs::read(&config).unwrap_or_default();
    let usage = vessel::hook::vessel::usage();
    for verb in [&["init", "--state-dir", dir.as_str()][..], &["show"], &["check"], &["update", "--state-dir", dir.as_str()]] {
        for (extra, rc, out_want, err_want) in [
            (&["--bogus", "x"][..], RC_BROKEN, String::new(), format!("vessel: 未知の引数 --bogus\n{usage}\n")),
            (&["--help"][..], RC_OK, format!("{usage}\n"), String::new()),
        ] {
            let mut args = verb.to_vec();
            args.extend_from_slice(extra);
            args.push(root.as_str());
            let out = run_vessel(&args);
            assert_eq!(out.status.code(), Some(i32::from(rc)), "{args:?}: {}", stderr_text(&out));
            assert_eq!(String::from_utf8_lossy(&out.stdout), out_want, "{args:?}: stdout");
            assert_eq!(stderr_text(&out), err_want, "{args:?}: stderr");
            assert!(!repo.join(MARKER).exists(), "{args:?}: marker を書かない");
            assert_eq!(fs::read(&config).unwrap_or_default(), before, "{args:?}: git の local 設定は不変");
        }
    }
    fs::remove_dir_all(&repo).ok();
    fs::remove_dir_all(&state).ok();
}

/// (1) 成功の周: argv が status → fetch → merge --ff-only → cargo install の順に写り、`InstallRecorded` が 1 件（sha12・
/// host・path）積まれ、stdout が 1 行（sha12 と path）である。base は `update` の verb を知らず rc 1 で 0 件（RED）。
#[test]
fn vessel_update_runs_ff_then_install_in_order_and_records_one_install() {
    crate::install_spawner();
    let place = update_place(true, "", 0, 0);
    let out = run_update(&place);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "成功の rc: {}", stderr_text(&out));
    assert_eq!(place.argv(), update_argv(), "順序固定の argv");
    let sha = UPDATE_HEAD.get(..12).unwrap_or_default();
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        format!("vessel: installed sha={sha} path={UPDATE_BIN}\n"),
        "stdout は 1 行"
    );
    let installs = place.installs();
    assert_eq!(installs.len(), 1, "InstallRecorded は 1 件: {installs:?}");
    let row = installs.first().expect("1 件在る");
    assert_eq!(row.host, vessel::fleet::cli::host(), "host 列は実 host");
    assert_eq!(
        row.install(),
        Some(vessel::fleet::Install { sha: sha.to_owned(), path: UPDATE_BIN.to_owned() }),
        "本体は sha12 と path"
    );
    assert!(row.run.is_empty() && row.bead.is_empty(), "便に紐づかない");
}

/// (2) 断り 4 形（否定の枝・1 形 1 本の表）: 宣言なし / dirty / not-fast-forward / install の失敗の各周で、**その段より後の
/// argv が 1 本も写らず** event は 0 件で、断りの語が stderr に出て stdout は空である。
#[test]
fn vessel_update_refusals_stop_before_the_next_stage_and_record_nothing() {
    let argv = update_argv();
    let upto = |n: usize| argv.get(..n).unwrap_or_default().to_vec();
    let cases = [
        ("宣言なし", update_place(false, "", 0, 0), "vessel: vessel-repo-undeclared", upto(0)),
        ("dirty", update_place(true, " M src/lib.rs\\n", 0, 0), "vessel: dirty", upto(1)),
        ("ff できない", update_place(true, "", 128, 0), "vessel: not-fast-forward", upto(4)),
        ("install の失敗", update_place(true, "", 0, 101), "vessel: install-failed rc=101", upto(6)),
    ];
    for (label, place, word, want) in cases {
        let out = run_update(&place);
        assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "{label}: rc");
        assert_eq!(place.argv(), want, "{label}: その段より後の argv は撃たない");
        assert!(place.installs().is_empty(), "{label}: event は 0 件");
        assert!(!vessel::fleet::store::events_path(&place.state).exists(), "{label}: log を作らない");
        assert_eq!(stderr_text(&out).lines().next(), Some(word), "{label}: 断りの語");
        assert!(out.stdout.is_empty(), "{label}: stdout は空");
    }
}

/// 入れ替えの門 (1): host のどこかで器の process（自分の外）が生きている周は、数えの段より後の argv（fetch・merge・器の
/// 世代・cargo）を 1 本も撃たず、event 0 件・stdout 空・stderr の 1 行目が `vessel: busy`・rc 1。生きた行の無い見本（偽 pgrep
/// の rc 1）は同じ置き場で数えの後の段へ進む。
#[test]
fn vswap_refuses_while_a_vessel_process_lives() {
    let argv = update_argv();
    let place = update_place(true, "", 0, 0);
    let live = format!("4242 {NAME} pipe run --design contracts/x.toml#a --bead b-1\n");
    fs::write(place.bin.join("live"), live).expect("生きた行を書ける");
    let out = run_update(&place);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "busy の rc: {}", stderr_text(&out));
    assert_eq!(stderr_text(&out).lines().next(), Some("vessel: busy"), "断りの語");
    assert_eq!(place.argv(), argv.get(..2).unwrap_or_default().to_vec(), "数えの段より後は撃たない");
    assert!(place.installs().is_empty() && out.stdout.is_empty(), "event 0 件・stdout 空");
    let idle = update_place(true, "", 0, 0);
    let out = run_update(&idle);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "生きた行の無い周は進む: {}", stderr_text(&out));
}

/// 留めた便の process の行（行 v-pin-swap）: 便 r1 の留めの形の器の語の行（起こし直しの driver と sh の包みの runner）・`--run r1` の
/// 行・`--bead b-2` の `pipe run` で便 r2 の札の所有者の pid の行（`runs` の置き場に r1 と r2 の留めと r2 の `RunCreated` と札を置く）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn pinned_lines(runs: &Path) -> String {
    let pin = |run: &str| runs.join("pipe").join(run).join("bin").join(NAME);
    for run in ["r1", "r2"] {
        fs::create_dir_all(runs.join("pipe").join(run).join("bin")).expect("bin の dir を作れる");
        fs::write(pin(run), "").expect("留めを置ける");
    }
    fs::write(runs.join("pipe").join("r2").join("driver"), "4304 1\n").expect("札を書ける");
    fs::create_dir_all(runs.join("fleet")).expect("log の dir を作れる");
    let created = "{\"schema\":1,\"ts\":\"2026-10-05T00:00:00Z\",\"kind\":\"RunCreated\",\"run\":\"r2\",\"bead\":\"b-2\",\"host\":\"h\",\"actor\":\"machine\",\"stage\":\"Intake\"}\n";
    fs::write(runs.join("fleet").join("events.jsonl"), created).expect("RunCreated を書ける");
    let (p1, s) = (pin("r1").display().to_string(), runs.display().to_string());
    format!(
        "4301 {p1} pipe resume --run r1 --state-dir {s} --drive\n4302 /usr/bin/sh -c {p1} runner --worktree /w\n\
         4303 {NAME} pipe land --run r1 --state-dir {s} --terminal-only\n4304 /opt/{NAME} pipe run --design d --bead b-2 --state-dir {s}\n"
    )
}

/// 入れ替えの門 (3)（行 v-pin-swap）: 偽 pgrep の行が留めた便の行だけの周は busy で断らず、撃ちの列と `InstallRecorded` 1 件と
/// stdout の 1 行は成功の見本と同じ。
#[test]
fn vpinswap_pinned_processes_do_not_hold_the_swap() {
    let place = update_place(true, "", 0, 0);
    let runs = tmp();
    fs::write(place.bin.join("live"), pinned_lines(&runs)).expect("生きた行を書ける");
    let out = run_update(&place);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "留めた行だけの周は進む: {}", stderr_text(&out));
    assert_eq!(place.argv(), update_argv(), "順序固定の argv（数えの後の段へ進む）");
    assert_eq!(place.installs().len(), 1, "InstallRecorded は 1 件");
    let sha = UPDATE_HEAD.get(..12).unwrap_or_default();
    assert_eq!(String::from_utf8_lossy(&out.stdout), format!("vessel: installed sha={sha} path={UPDATE_BIN}\n"), "stdout は 1 行");
}

/// 入れ替えの門 (4)（行 v-pin-swap）: 留めた行に、留めの無い便の行（`--run r9`）か、どの便にも解けない行（置き場を持たない sh の包み
/// の runner・札の所有者でない pid の `pipe run`）を 1 本足した周は、今までどおり数えの段より後を撃たず、event 0 件・stdout 空・
/// stderr の 1 行目が `vessel: busy`・rc 1。
#[test]
fn vpinswap_unpinned_or_unresolved_lines_keep_busy() {
    let argv = update_argv();
    let runs = tmp();
    let s = runs.display().to_string();
    for extra in [
        format!("4307 {NAME} pipe land --run r9 --state-dir {s}"),
        format!("4308 /usr/bin/sh -c {NAME} runner --worktree /w"),
        format!("4399 /opt/{NAME} pipe run --design d --bead b-2 --state-dir {s}"),
    ] {
        let place = update_place(true, "", 0, 0);
        fs::write(place.bin.join("live"), format!("{}{extra}\n", pinned_lines(&runs))).expect("生きた行を書ける");
        let out = run_update(&place);
        assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "{extra}: busy の rc: {}", stderr_text(&out));
        assert_eq!(stderr_text(&out).lines().next(), Some("vessel: busy"), "{extra}: 断りの語");
        assert_eq!(place.argv(), argv.get(..2).unwrap_or_default().to_vec(), "{extra}: 数えの段より後は撃たない");
        assert!(place.installs().is_empty() && out.stdout.is_empty(), "{extra}: event 0 件・stdout 空");
    }
}

/// 入れ替えの門 (2): PATH の器の世代（`--version` の括弧の sha12）と HEAD の差が repo の dir の下で空（`git diff --quiet <sha12>
/// HEAD -- .` の rc 0）の周は cargo を撃たず、event 0 件・stderr の 1 行目が `vessel: unchanged`・rc 1。差の在る見本（rc 1）と
/// 世代の括弧が `+dirty` の見本は組む（cargo を撃ち event 1 件）。
#[test]
fn vswap_skips_the_build_when_the_tree_is_unchanged() {
    let place = update_place(true, "", 0, 0);
    fs::write(place.bin.join("version"), format!("{NAME} 0.1.0 (89abcdef0123)\n")).expect("世代を書ける");
    fs::write(place.bin.join("diff.rc"), "0").expect("diff の rc を書ける");
    let out = run_update(&place);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "unchanged の rc: {}", stderr_text(&out));
    assert_eq!(stderr_text(&out).lines().next(), Some("vessel: unchanged"), "断りの語");
    let argv = place.argv();
    assert_eq!(argv.last().map(String::as_str), Some("git -C [repo] diff --quiet 89abcdef0123 HEAD -- ."), "最後は diff");
    assert!(!argv.iter().any(|line| line.starts_with("cargo ")), "cargo を撃たない: {argv:?}");
    assert!(place.installs().is_empty(), "event 0 件");
    for (label, version, rc) in [("差が在る", "89abcdef0123", "1"), ("+dirty", "89abcdef0123+dirty", "0")] {
        let built = update_place(true, "", 0, 0);
        fs::write(built.bin.join("version"), format!("{NAME} 0.1.0 ({version})\n")).expect("世代を書ける");
        fs::write(built.bin.join("diff.rc"), rc).expect("diff の rc を書ける");
        let out = run_update(&built);
        assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{label}: 組む: {}", stderr_text(&out));
        assert_eq!(built.installs().len(), 1, "{label}: event 1 件");
    }
}

/// 口の閉包: `update` は `--state-dir` を要り、positional を取らない（git も cargo も撃たない）。
#[test]
fn vessel_update_needs_state_dir_and_takes_no_root() {
    let place = update_place(true, "", 0, 0);
    let path = format!("{}:{}", place.bin.display(), std::env::var("PATH").unwrap_or_default());
    let state = place.state.display().to_string();
    for (args, want) in [
        (vec!["vessel", "update"], "vessel: --state-dir に値が無い"),
        (vec!["vessel", "update", "--state-dir", state.as_str(), "ROOT"], "vessel: 未知の引数 ROOT"),
    ] {
        let out = Command::new(bin()).args(&args).env("PATH", &path).output().expect("binary を起動できる");
        assert_eq!(out.status.code(), Some(i32::from(RC_BROKEN)), "{args:?}: 使い方の誤りは rc 2");
        assert_eq!(stderr_text(&out).lines().next(), Some(want), "{args:?}");
    }
    assert!(place.argv().is_empty(), "git も cargo も撃たない");
}
