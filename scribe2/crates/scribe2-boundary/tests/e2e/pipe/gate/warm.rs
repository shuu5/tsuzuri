//! 主実測の温かい木の歯（接頭辞 `vwarm_`・設計 pipeline.md §71）: 宣言 `build-lanes` を名乗った repo は、着地の後の主実測を
//! path 固定の木 `.worktrees/scribe2/verify/warm` で撃ち、撃った後も畳まずに木の中の target を次の主実測へ残す。
//!
//! 共有の helper は親 module（`tests/e2e/pipe/gate.rs`）と `tests/e2e/pipe.rs` に在り、`use super::*` で使う。toy の便は
//! verdict の `tree` を便の base の木へ差し替えて主実測を撃つ周にし、共通 verify の stub が撃たれた木の path・target の印・
//! 置いた stray の file の有無・HEAD の木を git の共通 dir の 1 file へ 1 行ずつ足す（gate の便の木の行も同じ file に入る）。

use super::*;

/// 撃たれた木を記す共通 verify の stub（撃った後に `target/mark` を置く＝次の撃ちが同じ木なら `kept` を記す）。
const WARM_SCRIPT: &str = "top=\"$(pwd -P)\"\nmark=cold\ntest -f target/mark && mark=kept\nstray=none\n\
test -e stray.txt && stray=stray\n\
printf '%s %s %s %s\\n' \"$top\" \"$mark\" \"$stray\" \"$(git rev-parse 'HEAD^{tree}')\" >> \"$(git rev-parse --git-common-dir)/warm-shots\"\n\
mkdir -p target && touch target/mark\n";

/// 主実測の木を集める dir（repo 相対）。
const VERIFY_DIR: &str = ".worktrees/scribe2/verify";

/// stub を共通 verify に置き、`/target/` を無視する toy の repo（`declared` の周だけ宣言に `build-lanes = true` を足す）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn warm_repo(declared: bool) -> (PathBuf, PathBuf) {
    let (repo, state) = repo_with_state();
    fs::write(repo.join("verify-warm.sh"), WARM_SCRIPT).expect("stub を書ける");
    fs::write(repo.join(".gitignore"), "/target/\n").expect("無視の規則を書ける");
    write_vessel(&repo, VESSEL_ALLOWED, r#"["sh verify-warm.sh"]"#);
    if declared {
        let path = repo.join(".vessel.toml");
        let body = fs::read_to_string(&path).expect("宣言を読める");
        fs::write(&path, format!("{body}build-lanes = true\n")).expect("宣言を書ける");
    }
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "warm"]);
    (repo, state)
}

/// 便を 1 本 PASS まで通し、verdict の `tree` を便の base の木へ差し替えて（主実測を撃つ周）land する。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn land_shot(repo: &Path, state: &Path, bead: &str) -> (String, Output) {
    let design = write_contract(repo, &[], &[]);
    let base_tree = git(repo, &["rev-parse", "HEAD^{tree}"]);
    let id = intake_bead(repo, state, &design, bead);
    let out = run_pipe(&[
        "spawn", "--run", &id, "--repo", &repo.display().to_string(),
        "--state-dir", &state.display().to_string(), "--rules", &ceiling_rules(state),
        "--runner", "echo x >> src/lib.rs && git add -A && git commit -q -m runner",
    ]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "spawn は rc 0: {}", stderr_of(&out));
    let lens = fake_lens(&state.join("lens-ran"), &lens_verdict("PASS"));
    let gated = gate_once(repo, state, &id, Some(&lens));
    assert_eq!(gated.status.code(), Some(i32::from(RC_OK)), "PASS の gate は rc 0: {}", stderr_of(&gated));
    let verdict = state.join("pipe").join(&id).join("verdict.json");
    let text = fs::read_to_string(&verdict).expect("verdict.json を読める");
    let tree = value_of(&verdict_pairs(state, &id), "tree");
    fs::write(&verdict, verdict_tree_to_base(&text, &tree, &base_tree)).expect("verdict.json を差し替えられる");
    let landed = land_once(repo, state, &id);
    (id, landed)
}

/// 主実測の木で撃たれた stub の行（`<木の path> <target の印> <stray の印> <HEAD の木>`・gate の便の木の行は除く）。
fn main_shots(repo: &Path) -> Vec<Vec<String>> {
    let log = fs::read_to_string(repo.join(".git").join("warm-shots")).unwrap_or_default();
    log.lines()
        .filter(|line| line.contains("/.worktrees/scribe2/verify/"))
        .map(|line| line.split(' ').map(str::to_owned).collect())
        .collect()
}

/// 主実測の木の dir の下の `leaf`（symlink を解いた path・stub の `pwd -P` と比べる）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn verify_leaf(repo: &Path, leaf: &str) -> String {
    let root = fs::canonicalize(repo).expect("repo の path を解ける");
    root.join(VERIFY_DIR).join(leaf).display().to_string()
}

/// 名乗った repo の 2 本の着地は、主実測をどちらも同じ path の木 `verify/warm` で撃ち、2 本目は 1 本目が木の中に置いた
/// target の印を見つけ、木の中身は 2 本目の着地した木に替わり、着地の後も木と登録と印が残り、便ごとの木は切らない。
#[test]
fn vwarm_declared_repo_shoots_main_in_one_fixed_tree_and_keeps_its_target() {
    let (repo, state) = warm_repo(true);
    let (first, out) = land_shot(&repo, &state, "s2-warm1");
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "1 本目の land は rc 0: {}", stderr_of(&out));
    let first_tree = git(&repo, &["rev-parse", "refs/heads/main^{tree}"]);
    let (second, out) = land_shot(&repo, &state, "s2-warm2");
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "2 本目の land は rc 0: {}", stderr_of(&out));
    let second_tree = git(&repo, &["rev-parse", "refs/heads/main^{tree}"]);
    assert_ne!(first_tree, second_tree, "fixture: 2 本の着地の木は違う");
    let warm = verify_leaf(&repo, "warm");
    let shots = main_shots(&repo);
    assert_eq!(
        shots,
        [
            vec![warm.clone(), "cold".to_owned(), "none".to_owned(), first_tree],
            vec![warm, "kept".to_owned(), "none".to_owned(), second_tree],
        ],
        "主実測は 2 本とも verify/warm で撃ち、2 本目は target の印を見つけ、木の中身は着地した木"
    );
    let dir = repo.join(VERIFY_DIR);
    assert!(dir.join("warm").join("target").join("mark").is_file(), "着地の後も木の中の target は残る");
    assert!(!dir.join(&first).exists() && !dir.join(&second).exists(), "便ごとの木は切らない");
    assert!(git(&repo, &["worktree", "list"]).contains("/verify/warm"), "温かい木の登録は残る");
    clean(&[&repo, &state]);
}

/// 名乗らない repo（宣言の 1 行だけが違う対）は、主実測を便ごとの木 `verify/<run>` で撃って畳み、温かい木を作らない。
#[test]
fn vwarm_undeclared_repo_cuts_and_folds_a_tree_per_landing() {
    let (repo, state) = warm_repo(false);
    let (id, out) = land_shot(&repo, &state, "s2-warm1");
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "land は rc 0: {}", stderr_of(&out));
    let shots = main_shots(&repo);
    let paths: Vec<&str> = shots.iter().filter_map(|shot| shot.first().map(String::as_str)).collect();
    assert_eq!(paths, [verify_leaf(&repo, &id)], "主実測は便ごとの木で 1 度撃つ: {shots:?}");
    let dir = repo.join(VERIFY_DIR);
    assert!(!dir.join(&id).exists(), "便ごとの木は撃った後に畳む");
    assert!(!dir.join("warm").exists(), "温かい木を作らない");
    clean(&[&repo, &state]);
}

/// 名乗った repo でも、温かい木の lock を生きた所有者（この歯の process）が持つ周は、待ちの上限の後に便ごとの木で撃って
/// 畳み、温かい木を作らず、lock に触れない（land は緑）。
#[test]
fn vwarm_held_lock_falls_back_to_a_per_landing_tree() {
    let (repo, state) = warm_repo(true);
    let dir = repo.join(VERIFY_DIR);
    fs::create_dir_all(&dir).expect("置き場を作れる");
    let owner = format!("{}\n", std::process::id());
    fs::write(dir.join("warm.lock"), &owner).expect("lock を塞げる");
    let (id, out) = land_shot(&repo, &state, "s2-warm1");
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "land は rc 0: {}", stderr_of(&out));
    let shots = main_shots(&repo);
    let paths: Vec<&str> = shots.iter().filter_map(|shot| shot.first().map(String::as_str)).collect();
    assert_eq!(paths, [verify_leaf(&repo, &id)], "lock を取れない周は便ごとの木で撃つ: {shots:?}");
    assert!(!dir.join(&id).exists(), "便ごとの木は撃った後に畳む");
    assert!(!dir.join("warm").exists(), "温かい木を作らない");
    assert_eq!(fs::read_to_string(dir.join("warm.lock")).ok(), Some(owner), "生きた所有者の lock に触れない");
    clean(&[&repo, &state]);
}

/// 温かい木に置いた追跡の外の file と、替えた追跡の file は、次の主実測の前に消えて着地した中身に戻り、target の印は残る。
#[test]
fn vwarm_renew_drops_stray_files_before_the_check_and_keeps_the_target() {
    let (repo, state) = warm_repo(true);
    let (_, out) = land_shot(&repo, &state, "s2-warm1");
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "1 本目の land は rc 0: {}", stderr_of(&out));
    let warm = repo.join(VERIFY_DIR).join("warm");
    fs::write(warm.join("stray.txt"), "x\n").expect("追跡の外の file を置ける");
    fs::write(warm.join("src").join("lib.rs"), "// edited\n").expect("追跡の file を替えられる");
    let (_, out) = land_shot(&repo, &state, "s2-warm2");
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "2 本目の land は rc 0: {}", stderr_of(&out));
    let second = main_shots(&repo).get(1).cloned().unwrap_or_default();
    let marks: Vec<&str> = second.iter().skip(1).take(2).map(String::as_str).collect();
    assert_eq!(marks, ["kept", "none"], "撃つ前に stray は消え、target の印は残る: {second:?}");
    assert_eq!(second.last().cloned(), Some(git(&repo, &["rev-parse", "refs/heads/main^{tree}"])), "木は着地した木");
    assert!(!warm.join("stray.txt").exists(), "stray は消えたまま");
    assert_eq!(
        fs::read_to_string(warm.join("src").join("lib.rs")).ok(),
        Some(format!("{}\n", git(&repo, &["show", "refs/heads/main:src/lib.rs"]))),
        "替えた追跡の file は着地した中身に戻る"
    );
    clean(&[&repo, &state]);
}

/// 温かい木の path に作業木でない dir（`.git` の file が無い）が在る周は、その dir の中身を替えず、repo の作業木の HEAD も
/// main から外さず、便ごとの木で撃って畳む（land は緑）。
#[test]
fn vwarm_plain_dir_at_the_warm_path_is_left_and_the_anchor_stays_on_main() {
    let (repo, state) = warm_repo(true);
    let warm = repo.join(VERIFY_DIR).join("warm");
    fs::create_dir_all(&warm).expect("置き場を塞げる");
    fs::write(warm.join("keep.txt"), "x\n").expect("塞げる");
    let (id, out) = land_shot(&repo, &state, "s2-warm1");
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "land は rc 0: {}", stderr_of(&out));
    let shots = main_shots(&repo);
    let paths: Vec<&str> = shots.iter().filter_map(|shot| shot.first().map(String::as_str)).collect();
    assert_eq!(paths, [verify_leaf(&repo, &id)], "作業木でない dir の周は便ごとの木で撃つ: {shots:?}");
    assert!(warm.join("keep.txt").is_file(), "作業木でない dir の中身に触れない");
    assert_eq!(git(&repo, &["symbolic-ref", "HEAD"]), "refs/heads/main", "repo の作業木は main のまま");
    clean(&[&repo, &state]);
}
