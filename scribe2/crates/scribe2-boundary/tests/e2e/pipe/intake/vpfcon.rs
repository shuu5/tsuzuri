//! 器の口 pipe preflight の契約の file の口 `--contract` の歯（接頭辞 `vpfcon_`・親 `tests/e2e/pipe/intake.rs` の helper を
//! `use super::*` で使う）。
//!
//! 契約表の導出の形の 1 行の `.toml` の file（欄 goal つき）を repo の外に置いて `--contract` で渡し、同じ行を `--design` で渡した撃ちと
//! 同じ判定になること・断りの字・run dir を作らないことを、rc と stdout と stderr から測る。

use super::*;

/// 行の verify（導出の歯 `derive_` の 1 本の nextest 行）。
const VERIFY: &str = "[\"cargo nextest run -p toy --no-tests=fail derive_\"]";

/// 契約の file の本文の字（欄 goal の値）。
const GOAL: &str = "\"本文。\"";

/// 行 `id`（欄 verify を [`VERIFY`] に替えた導出の行・`goal` が真なら欄 goal を足す）。
fn con_row(id: &str, goal: bool) -> String {
    let mut over = vec![("verify", VERIFY)];
    if goal {
        over.push(("goal", GOAL));
    }
    derive_row(id, &over)
}

/// 置き場の dir の隣（repo の外）の一時の dir に、字 `schema = 1` と `rows` を繋いだ契約の file `name` を書く（dir と file の path）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn con_file(state: &Path, name: &str, rows: &[String]) -> (PathBuf, PathBuf) {
    let mut dir_name = state.file_name().unwrap_or_default().to_os_string();
    dir_name.push("-con");
    let dir = state.with_file_name(dir_name);
    fs::create_dir_all(&dir).expect("dir を作れる");
    let body = rows.iter().fold("schema = 1\n".to_owned(), |text, row| format!("{text}\n{row}"));
    let file = dir.join(name);
    fs::write(&file, body).expect("契約の file を書ける");
    (dir, file)
}

/// `pipe preflight --contract <given> --bead s2-a --repo R --rules … --state-dir S` に `extra` を足して 1 回撃つ（`cwd` が在れば子の作業 dir にする）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn con_preflight(repo: &Path, state: &Path, given: &str, extra: &[&str], cwd: Option<&Path>) -> Output {
    let (rules, repo_arg, state_arg) = (ceiling_rules(state), repo.display().to_string(), state.display().to_string());
    let mut args = vec![
        "preflight", "--contract", given, "--bead", "s2-a", "--repo", &repo_arg, "--rules", &rules, "--state-dir", &state_arg,
    ];
    args.extend(extra);
    let mut cmd = pipe_cmd(&args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    cmd.output().expect("binary を起動できる")
}

/// 断りの撃ちの外形: rc 1・stdout 0 byte・stderr は `want` の 1 行・置き場に run dir を作らない。
fn assert_con_refused(out: &Output, state: &Path, want: &str) {
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "{want}: {}", stdout_of(out));
    assert!(out.stdout.is_empty(), "{want}: stdout は 0 byte: {}", stdout_of(out));
    assert_eq!(stderr_of(out).lines().collect::<Vec<_>>(), [want], "stderr の 1 行");
    assert_eq!(run_dirs(state), Vec::<String>::new(), "{want}: run dir を作らない");
}

/// 行 a の契約の file を絶対 path で渡すと、`--design` で同じ行を渡した撃ちと design= の行を除いて同じ行の列で、design= の行は file の絶対 path と
/// 行 id と § を持ち、末尾は ok・run dir を作らない。
#[test]
fn vpfcon_contract_file_matches_the_design_pointer() {
    let (repo, state) = derive_repo(&table_doc(&table_region(&[con_row("a", false)])));
    let (dir, file) = con_file(&state, "a.toml", &[con_row("a", true)]);
    let out = con_preflight(&repo, &state, &file.display().to_string(), &[], None);
    let text = stdout_of(&out);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{text} {}", stderr_of(&out));
    assert_eq!(fact_lines(&out, "design="), [format!("design={}#a section=1", file.display())], "{text}");
    assert_eq!(tail_line(&out), "preflight: ok", "{text}");
    let by_design = preflight_raw(&repo, &state, "docs/design/toy.md#a", "s2-a", true);
    let rest = |found: &Output| -> Vec<String> { stdout_of(found).lines().filter(|line| !line.starts_with("design=")).map(str::to_owned).collect() };
    assert_eq!(rest(&out), rest(&by_design), "design= の行を除いて --design と同じ判定: {text}");
    assert_eq!(run_dirs(&state), Vec::<String>::new(), "run dir を作らない");
    clean(&[&repo, &state, &dir]);
}

/// 相対 path は子の作業 dir に繋いで絶対にする（file の名だけを渡し、作業 dir を file の dir にする）。
#[test]
fn vpfcon_relative_path_is_made_absolute() {
    let (repo, state) = derive_repo(&table_doc(&table_region(&[con_row("a", false)])));
    let (dir, _) = con_file(&state, "a.toml", &[con_row("a", true)]);
    let real = dir.canonicalize().unwrap_or_else(|_| dir.clone());
    let out = con_preflight(&repo, &state, "a.toml", &[], Some(&real));
    let text = stdout_of(&out);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{text} {}", stderr_of(&out));
    assert_eq!(fact_lines(&out, "design="), [format!("design={}#a section=1", real.join("a.toml").display())], "{text}");
    assert_eq!(run_dirs(&state), Vec::<String>::new(), "run dir を作らない");
    clean(&[&repo, &state, &dir]);
}

/// 行を 0 本と 2 本持つ toml の file は行の数の断り、md の file は拡張子の断り（どれも rc 1・stdout 0 byte・stderr の 1 行・run dir 無し）。
#[test]
fn vpfcon_file_must_hold_one_toml_row() {
    let (repo, state) = derive_repo(&table_doc(&table_region(&[con_row("a", false)])));
    for (rows, count) in [(Vec::new(), 0), (vec![con_row("a", true), con_row("b", true)], 2)] {
        let (dir, file) = con_file(&state, "n.toml", &rows);
        let out = con_preflight(&repo, &state, &file.display().to_string(), &[], None);
        assert_con_refused(&out, &state, &format!("pipe: 契約の file は [[contract]] の行を 1 つだけ持つ（{count} 行）"));
        clean(&[&dir]);
    }
    let (dir, file) = con_file(&state, "contract.md", &[con_row("a", true)]);
    let given = file.display().to_string();
    let out = con_preflight(&repo, &state, &given, &[], None);
    assert_con_refused(&out, &state, &format!("pipe: --contract {given} は .toml の file でない"));
    clean(&[&repo, &state, &dir]);
}

/// --design と --contract の両方、--contract に --placed を足す撃ちは、どちらも使い方の断り（rc 1・stdout 0 byte・stderr の 1 行・run dir 無し）。
#[test]
fn vpfcon_design_and_contract_and_placed_are_exclusive() {
    let (repo, state) = derive_repo(&table_doc(&table_region(&[con_row("a", false)])));
    let (dir, file) = con_file(&state, "a.toml", &[con_row("a", true)]);
    let given = file.display().to_string();
    let both = con_preflight(&repo, &state, &given, &["--design", "docs/design/toy.md#a"], None);
    assert_con_refused(&both, &state, "pipe: --design と --contract は 1 つだけ渡す");
    let placed = con_preflight(&repo, &state, &given, &["--placed"], None);
    assert_con_refused(&placed, &state, "pipe: --placed は --design と使う");
    clean(&[&repo, &state, &dir]);
}

/// 欄 goal の無い 1 行の toml の file は、表の検査が節の本文を見つけられず断る（rc 1・末尾 refused n=1・stderr は空でない・run dir 無し）。
#[test]
fn vpfcon_goal_less_row_is_refused_like_the_table() {
    let (repo, state) = derive_repo(&table_doc(&table_region(&[con_row("a", false)])));
    let (dir, file) = con_file(&state, "a.toml", &[con_row("a", false)]);
    let out = con_preflight(&repo, &state, &file.display().to_string(), &[], None);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "{}", stdout_of(&out));
    assert_eq!(tail_line(&out), "preflight: refused n=1", "{}", stdout_of(&out));
    assert!(!stderr_of(&out).is_empty(), "理由は stderr に出す");
    assert_eq!(run_dirs(&state), Vec::<String>::new(), "run dir を作らない");
    clean(&[&repo, &state, &dir]);
}

/// crate b の manifest が `[[test]]` の name tz4 と path（別 crate の dir の `main.rs`）を持つ toy で、検証行 `--test tz4 fsch_` と done-teeth `1:=fsch_one` の
/// 契約の file の preflight は、歯の区間を manifest の path の `main.rs` の dir の下と読み、`teeth=fsch_:1@…/resolve.rs` を出して ok（rc 0）で終わる。
#[test]
fn vpfcon_manifest_test_path_gives_the_region() {
    let manifest = "[package]\nname = \"b\"\n\n[[test]]\nname = \"tz4\"\npath = \"../../folio2/crates/f/tests/tz4/main.rs\"\n";
    let files = [
        ("crates/b/Cargo.toml", manifest),
        ("folio2/crates/f/tests/tz4/main.rs", "mod resolve;\n"),
        ("folio2/crates/f/tests/tz4/resolve.rs", "#[test]\nfn fsch_one() {}\n"),
    ];
    let (repo, state) = derive_repo_with(&table_doc(&table_region(&[con_row("a", false)])), &files);
    let verify = "[\"cargo nextest run -p b --test tz4 --no-tests=fail fsch_\"]";
    let row = derive_row("a", &[("verify", verify), ("goal", GOAL), ("done", "\"(1) 歯が区間に入る\""), ("done-teeth", "[\"1:=fsch_one\"]")]);
    let (dir, file) = con_file(&state, "a.toml", &[row]);
    let out = con_preflight(&repo, &state, &file.display().to_string(), &[], None);
    let text = stdout_of(&out);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{text} {}", stderr_of(&out));
    assert_eq!(fact_lines(&out, "teeth="), ["teeth=fsch_:1@folio2/crates/f/tests/tz4/resolve.rs"], "{text}");
    assert_eq!(tail_line(&out), "preflight: ok", "{text}");
    clean(&[&repo, &state, &dir]);
}
