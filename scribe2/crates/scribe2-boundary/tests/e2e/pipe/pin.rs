//! 便ごとの器の binary の留め（器の memo t3-hub.74.50.2 の候補 1・行 v-pin・接頭辞 `vpin_`）。
//!
//! 偽の器（`--version` に答え、ほかの周は `$0` と 1 語目を写す script）を `InstallRecorded` の path と道具箱の PATH に置き、
//! 実装役と審査役の命令を器の名で始めて撃つ。道具箱の偽の器は、留めを撃たなかった命令が host の器へ届かないための受け口でもある。

use super::{
    bin_cmd, ceiling_rules, clean, commit_rows, lens_verdict, repo_with_state, row_fields, run_id_of, run_pipe, show_line,
    stderr_of, stdout_of, write_contract, DESIGN_FILE,
};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Output;
use vessel::cli_outcome::RC_OK;
use vessel::name::NAME;

/// 歯の binary の `--version` の 1 行目。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn version_of_bin() -> String {
    let out = bin_cmd().arg("--version").output().expect("binary を起動できる");
    String::from_utf8_lossy(&out.stdout).lines().next().unwrap_or_default().to_owned()
}

/// 偽の器を `file` に置く: `--version` は `version` を答え、ほかの周は `$0 $1 $2` を `rec` に足し `$0` の file の dev と inode を
/// `rec` の名に `.inode` を足した file に書き、runner は 1 commit・lens は PASS を返す。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn fake_vessel(file: &Path, version: &str, rec: &Path) -> PathBuf {
    fs::create_dir_all(file.parent().expect("親の dir")).expect("親の dir を作れる");
    let (rec, verdict) = (rec.display(), lens_verdict("PASS"));
    let body = format!(
        "#!/bin/sh\n[ \"$1\" = --version ] && {{ echo '{version}'; exit 0; }}\nprintf '%s %s %s\\n' \"$0\" \"$1\" \"$2\" >> '{rec}'\nstat -c '%d %i' \"$0\" > '{rec}.inode'\n\
         case \"$1\" in\n  runner) printf 'x\\n' >> src/lib.rs && git add -A && git commit -q -m runner ;;\n  \
         lens) cat >/dev/null; echo '{verdict}' ;;\nesac\nexit 0\n"
    );
    fs::write(file, body).expect("偽の器を書ける");
    fs::set_permissions(file, fs::Permissions::from_mode(0o755)).expect("実行権を付ける");
    file.to_path_buf()
}

/// 置き場の event log に `InstallRecorded` を 1 行足す（path は呼び手が選ぶ）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn record_install(state: &Path, file: &Path) {
    let log = vessel::fleet::store::events_path(state);
    fs::create_dir_all(log.parent().expect("log の dir")).expect("log の dir を作れる");
    let line = format!(
        "{{\"schema\":1,\"ts\":\"2026-10-05T00:00:00Z\",\"kind\":\"InstallRecorded\",\"host\":\"h\",\"actor\":\"machine\",\"detail\":\"sha=0123456789ab path={}\"}}\n",
        file.display()
    );
    let mut text = fs::read_to_string(&log).unwrap_or_default();
    text.push_str(&line);
    fs::write(&log, text).expect("log に足せる");
}

/// 道具箱の dir（`pipe` の撃ちが PATH の先頭に積む）。
fn toolbox(state: &Path) -> PathBuf {
    state.join(crate::TOOLBOX_BIN)
}

/// 偽の器が写した行（無ければ空）。
fn calls(rec: &Path) -> Vec<String> {
    fs::read_to_string(rec).unwrap_or_default().lines().map(str::to_owned).collect()
}

/// 器の名で始まる runner と lens で toy の契約を 1 本 `pipe run` で流す。
fn run_named(repo: &Path, state: &Path, bead: &str) -> Output {
    let path = write_contract(repo, &[], &[]);
    let (runner, lens) = (format!("{NAME} runner --worktree {{worktree}}"), format!("{NAME} lens --contract {{contract}}"));
    run_pipe(&[
        "run", "--design", &path, "--bead", bead, "--repo", &repo.display().to_string(), "--state-dir", &state.display().to_string(),
        "--rules", &ceiling_rules(state), "--runner", &runner, "--lens", &lens,
    ])
}

/// (2)(5) 同じ版の `InstallRecorded` の在る置き場の `pipe run` は便の bin の下に inode の同じ留めを置いて stderr に
/// `pipe: pin=linked` を出し、審査の lens・runner・gate の lens（器の名で始まる命令）はこの順に留めの path で撃たれ、便は着地する
/// （着地した便の留めは終端の掃除が消すので、inode は撃たれた留めが自分で写す）。
#[test]
fn vpin_run_pins_the_recorded_binary_for_its_runner_and_lenses() {
    let (repo, state) = repo_with_state();
    let rec = state.join("vessel-calls");
    let version = version_of_bin();
    let installed = fake_vessel(&state.join("installed").join(NAME), &version, &rec);
    fake_vessel(&toolbox(&state).join(NAME), &version, &rec);
    record_install(&state, &installed);
    let out = run_named(&repo, &state, "s2-pin");
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{} / {}", stdout_of(&out), stderr_of(&out));
    let id = run_id_of(&out);
    assert!(show_line(&repo, &state, &id).contains("stage=Landed"), "着地まで: {}", stdout_of(&out));
    let pin = state.join("pipe").join(&id).join("bin").join(NAME);
    let inode = fs::metadata(&installed).map(|found| format!("{} {}\n", found.dev(), found.ino())).ok();
    let seen = fs::read_to_string(format!("{}.inode", rec.display())).ok();
    assert_eq!(seen, inode, "撃たれた留めは InstallRecorded の file の hard link（中身を写さない）");
    assert!(stderr_of(&out).lines().any(|line| line == "pipe: pin=linked"), "stderr: {}", stderr_of(&out));
    let shown = pin.display();
    let (lens, runner) = (format!("{shown} lens --contract"), format!("{shown} runner --worktree"));
    assert_eq!(calls(&rec), [lens.clone(), runner, lens], "子は留めを撃つ");
    clean(&[&repo, &state]);
}

/// (3)(5) `InstallRecorded` の無い置き場と、1 行目の違う file を最後の `InstallRecorded` が名指す置き場の `pipe run` は bin の dir を
/// 作らず stderr に `pipe: pin=skipped:no-install`・`pipe: pin=skipped:other-version` を出し、器の名で始まる命令は PATH の器で
/// 撃たれ、便は着地する。
#[test]
fn vpin_run_without_a_same_version_record_keeps_the_path_vessel() {
    for word in ["no-install", "other-version"] {
        let (repo, state) = repo_with_state();
        let rec = state.join("vessel-calls");
        let path_vessel = fake_vessel(&toolbox(&state).join(NAME), &version_of_bin(), &rec);
        if word == "other-version" {
            let other = fake_vessel(&state.join("installed").join(NAME), &format!("{NAME} 0.0.0 (000000000000)"), &rec);
            record_install(&state, &other);
        }
        let out = run_named(&repo, &state, "s2-pin");
        assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{word}: {} / {}", stdout_of(&out), stderr_of(&out));
        let id = run_id_of(&out);
        assert!(show_line(&repo, &state, &id).contains("stage=Landed"), "{word}: 着地まで");
        assert!(!state.join("pipe").join(&id).join("bin").exists(), "{word}: bin の dir を作らない");
        let line = format!("pipe: pin=skipped:{word}");
        assert!(stderr_of(&out).lines().any(|found| found == line), "{word}: stderr: {}", stderr_of(&out));
        let shown = path_vessel.display();
        let (lens, runner) = (format!("{shown} lens --contract"), format!("{shown} runner --worktree"));
        assert_eq!(calls(&rec), [lens.clone(), runner, lens], "{word}: PATH の器");
        clean(&[&repo, &state]);
    }
}

/// (5) 呼ばれ方が留めの形（置き場の pipe の下の便の bin の下の器の名）の列の 1 周が起こす新しい便は、留めでなく器の名（PATH の器）
/// で起き、`pipe run` の argv を受ける。
#[test]
fn vpin_new_runs_from_a_pinned_dispatcher_use_the_path_vessel() {
    let (repo, state) = repo_with_state();
    commit_rows(&repo, &[row_fields("a", &["write-set"], &[r#"write-set = ["src/lib.rs"]"#])]);
    let ledger = state.join("ledger.json");
    let issue = format!(
        "[{{\"id\":\"s2-pin.1\",\"status\":\"open\",\"priority\":1,\"labels\":[],\"acceptance_criteria\":\"design = {DESIGN_FILE}#a\",\"dependencies\":[]}}]\n"
    );
    fs::write(&ledger, issue).expect("偽の台帳を書ける");
    let bd = state.join("bd");
    fs::write(&bd, format!("#!/bin/sh\ncat '{}'\n", ledger.display())).expect("偽の bd を書ける");
    fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("実行権を付ける");
    let rules = state.join("rules-pin.toml");
    let row = "[[rule]]\nid = \"seat.ledger_timeout_s\"\nkind = \"LedgerTimeoutS\"\nvalue = 60\nenabled = true\nruling = \"t\"\nruled_at = \"d\"\n";
    fs::write(&rules, format!("{}\n{row}", fs::read_to_string(ceiling_rules(&state)).expect("写しを読める"))).expect("写しを書ける");
    let rec = state.join("vessel-calls");
    let path_vessel = fake_vessel(&toolbox(&state).join(NAME), &version_of_bin(), &rec);
    let link = state.join("pipe").join("r0").join("bin").join(NAME);
    fs::create_dir_all(link.parent().expect("bin の dir")).expect("bin の dir を作れる");
    std::os::unix::fs::symlink(super::bin(), &link).expect("留めの形の呼ばれ方を置ける");
    let (state_arg, repo_arg) = (state.display().to_string(), repo.display().to_string());
    let out = std::process::Command::new(&link)
        .current_dir(std::env::temp_dir())
        .env("PATH", crate::toolbox_path(&state))
        .args(["pipe", "dispatch", "--state-dir", &state_arg, "--repo", &repo_arg, "--rules", &rules.display().to_string()])
        .args(["--bd", &bd.display().to_string(), "--lens", "true", "--runner", "true"])
        .output()
        .expect("留めの形で起こせる");
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{} / {}", stdout_of(&out), stderr_of(&out));
    assert!(stdout_of(&out).contains("dispatch=started:1"), "1 本起こす: {}", stdout_of(&out));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while calls(&rec).is_empty() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert_eq!(calls(&rec), [format!("{} pipe run", path_vessel.display())], "新しい便は PATH の器で起きる");
    clean(&[&repo, &state]);
}

/// (6) 器の掃除の周（`pipe stop` の終端）は live でない便の bin の dir を消し、live の便の留めを残し、stderr の sweep の行に
/// `pins=` と消した数が載る。
#[test]
fn vpin_sweep_drops_pins_of_settled_runs() {
    let (repo, state) = repo_with_state();
    for run in ["r-pin", "r-live"] {
        let out = bin_cmd()
            .args(["fleet", "record", "--kind", "RunStage", "--run", run, "--bead", "b", "--stage", "Implemented", "--detail", "x"])
            .arg("--state-dir")
            .arg(&state)
            .output()
            .expect("binary を起動できる");
        assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{run}: {}", stderr_of(&out));
        let pin = state.join("pipe").join(run).join("bin").join(NAME);
        fs::create_dir_all(pin.parent().expect("bin の dir")).expect("bin の dir を作れる");
        fs::write(&pin, "").expect("留めを置ける");
    }
    let out = run_pipe(&["stop", "--run", "r-pin", "--state-dir", &state.display().to_string()]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "終端の rc: {}", stderr_of(&out));
    assert!(!state.join("pipe").join("r-pin").join("bin").exists(), "止めた便の bin の dir は消える");
    assert!(state.join("pipe").join("r-live").join("bin").join(NAME).is_file(), "live の便の留めは残る");
    let sweep = stderr_of(&out).lines().find(|line| line.starts_with("sweep:")).unwrap_or_default().to_owned();
    assert_eq!(sweep, "sweep: removed=0 runs=0 failed=0 pins=1", "stderr: {}", stderr_of(&out));
    clean(&[&repo, &state]);
}
