//! 入口と列の差の当たりの判じの配線の歯（接頭辞 `vcwire_`・判断の記録 ADR-60 の決定 (3)(4)・親 `tests/e2e/pipe/dispatch.rs` の
//! helper を `use super::*` で使う）。
//!
//! 面の中の file 1 本を書く live な便（行 a）の下で、同じ file を書く行の受付（`pipe intake`）と列の 1 周（`pipe dispatch ls`）を、
//! 規則の行 `pipe.overlap_commute` の真偽で外形から測る。差の file は行ごとに 1 本（30 行の file の 1 行を替える）。

use super::super::intake::intake_with_rules;
use super::super::{events, run_dirs, write_rules};
use super::*;
use std::path::PathBuf;
use vessel::fleet::EventKind;

/// 行が書く面の中の file（固定の根 `crates/` の下・30 行・行 n は `l<n>`）。
const FILE: &str = "crates/toy/src/lib.rs";

/// 行の id と、差が替える FILE の行（行 d は差の file を持たない）。b は a と離れ、c は a と、f は b と文脈が重なる。
const ROWS: [(&str, usize); 5] = [("a", 5), ("b", 25), ("c", 6), ("f", 22), ("d", 0)];

/// 規則の行 id。
const ROW: &str = "pipe.overlap_commute";

/// FILE の行 `at` を `x<at>` に替える差（前後の文脈 3 行）。
fn patch(at: usize) -> String {
    let body: Vec<String> =
        (at - 3..=at + 3).map(|n| if n == at { format!("-l{n}\n+x{n}") } else { format!(" l{n}") }).collect();
    format!("diff --git a/{FILE} b/{FILE}\n--- a/{FILE}\n+++ b/{FILE}\n@@ -{0},7 +{0},7 @@\n{1}\n", at - 3, body.join("\n"))
}

/// FILE と行ごとの差の file（`docs/p/<行>.patch`）と行 5 本を 1 回で commit した repo と置き場。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn place() -> (PathBuf, PathBuf) {
    let (repo, state) = repo_with_state();
    fs::create_dir_all(repo.join("crates/toy/src")).expect("面の dir を作れる");
    fs::write(repo.join(FILE), (1..=30).map(|n| format!("l{n}\n")).collect::<String>()).expect("面の file を書ける");
    fs::create_dir_all(repo.join("docs/p")).expect("差の dir を作れる");
    let mut rows = Vec::new();
    for (id, at) in ROWS {
        let mut add = vec![format!("write-set = [\"{FILE}\"]")];
        if at > 0 {
            fs::write(repo.join(format!("docs/p/{id}.patch")), patch(at)).expect("差の file を書ける");
            add.push(format!("patch = \"docs/p/{id}.patch\""));
        }
        rows.push(row_fields(id, &["write-set"], &add.iter().map(String::as_str).collect::<Vec<&str>>()));
    }
    commit_rows(&repo, &rows);
    (repo, state)
}

/// manifest の写し `base` から規則の行を除き、`value` の行（`None` は行なし）を足した写し。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn rules(state: &Path, base: &str, name: &str, value: Option<bool>) -> String {
    let text = fs::read_to_string(base).expect("manifest の写しを読める");
    let kept: Vec<&str> = text.split("[[rule]]").filter(|block| !block.contains(&format!("\"{ROW}\""))).collect();
    let row = value.map_or_else(String::new, |value| {
        format!("[[rule]]\nid = \"{ROW}\"\nkind = \"PipeOverlapCommute\"\nvalue = {value}\nenabled = true\nruling = \"t\"\nruled_at = \"d\"\n")
    });
    let path = state.join(name);
    fs::write(&path, format!("{}\n{row}", kept.join("[[rule]]"))).expect("manifest の写しを書ける");
    path.display().to_string()
}

/// 受付の manifest（gate と lock の行の写しに規則の行 `value`）。
fn intake_rules(state: &Path, value: Option<bool>) -> String {
    let base = write_rules(state, "rules-vcwire-base.toml", 1, 1_000_000);
    rules(state, &base.display().to_string(), &format!("rules-vcwire-{value:?}.toml"), value)
}

/// 行 `row` を bead `s2-<row>` で受付に撃つ（rc を測らない）。
fn intake_row(repo: &Path, state: &Path, row: &str, rules: &str) -> Output {
    intake_with_rules(repo, state, &format!("{DESIGN_FILE}#{row}"), &format!("s2-{row}"), rules)
}

/// 行 a の live な便を起こして run id を返す。
fn live_a(repo: &Path, state: &Path, rules: &str) -> String {
    let out = intake_row(repo, state, "a", rules);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "live 0 本の周は通る（{}）", told(&out));
    run_id_of(&out)
}

/// 交差の断りの 1 行（結末の無い周は今の字）。
fn refusal(live: &str, verdict: Option<&str>) -> String {
    let tail = verdict.map_or_else(String::new, |found| format!("・{found}"));
    format!("pipe: write-set が live な run {live} と交差する（{FILE}{tail}）")
}

/// 列の 1 周を規則の行 `value` の manifest で撃つ（台帳は行 c・b・f・d の 4 件・priority はこの順に 1・2・3・4）。
fn ls_with(repo: &Path, state: &Path, value: Option<bool>) -> Output {
    let rules = rules(state, &dispatch_rules(state), &format!("rules-vcwire-ls-{value:?}.toml"), value);
    let issues: Vec<String> = [("s2-c", 1, "c"), ("s2-b", 2, "b"), ("s2-f", 3, "f"), ("s2-d", 4, "d")]
        .iter()
        .map(|&(bead, priority, row)| issue(bead, priority, row))
        .collect();
    let bd = fake_bd(state, &issues);
    let (state_dir, repo_dir) = (state.display().to_string(), repo.display().to_string());
    run_pipe(&["dispatch", "ls", "--state-dir", &state_dir, "--repo", &repo_dir, "--rules", &rules, "--bd", &bd])
}

/// 受付の記帳の `OverlapCommuted` の行（bead と detail）。
fn commuted(state: &Path) -> Vec<(String, Option<String>)> {
    events(state).into_iter().filter(|event| event.kind == EventKind::OverlapCommuted).map(|event| (event.bead, event.detail)).collect()
}

/// 規則の行が真の周、live な便と同じ file の離れた行を替える差の行は受付が通り、出来事の記録に `OverlapCommuted` を 1 行書く
/// （bead は候補・detail は受付の run id と相手の run id と交わった項と main の先端）。
#[test]
fn vcwire_intake_passes_a_commuting_pair_and_records_it() {
    let (repo, state) = place();
    let on = intake_rules(&state, Some(true));
    let live = live_a(&repo, &state, &on);
    let out = intake_row(&repo, &state, "b", &on);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "通す組は受け付ける（{}）", told(&out));
    let id = run_id_of(&out);
    let head = git(&repo, &["rev-parse", "HEAD"]);
    let want = vec![("s2-b".to_owned(), Some(format!("run={id} with={live} files={FILE} main={head}")))];
    assert_eq!(commuted(&state), want, "通した組を 1 行記帳する");
    clean(&[&repo, &state]);
}

/// 規則の行が真の周、当たらない差の行と差の file の無い行は受付が断り、断りの 1 行に結末を添える（not-commuting・no-patch）。
/// run dir も `OverlapCommuted` の行も増えない。
#[test]
fn vcwire_intake_refusal_names_the_verdict() {
    let (repo, state) = place();
    let on = intake_rules(&state, Some(true));
    let live = live_a(&repo, &state, &on);
    let dirs = run_dirs(&state);
    for (row, verdict) in [("c", "not-commuting"), ("d", "no-patch")] {
        let out = intake_row(&repo, &state, row, &on);
        assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "行 {row} は断る（{}）", told(&out));
        let first = stderr_of(&out).lines().next().map(str::to_owned);
        assert_eq!(first, Some(refusal(&live, Some(verdict))), "行 {row} の断りの 1 行");
    }
    assert_eq!(run_dirs(&state), dirs, "run dir を作らない");
    assert!(commuted(&state).is_empty(), "断った組は記帳しない");
    clean(&[&repo, &state]);
}

/// 規則の行が無い周と偽の周は、通す組の行も今の断り（結末を添えない）で、列は今の待ち（`overlap:<run>/1`）のまま。
#[test]
fn vcwire_rule_off_keeps_the_refusal_and_the_wait() {
    let (repo, state) = place();
    let live = live_a(&repo, &state, &intake_rules(&state, None));
    for value in [None, Some(false)] {
        let out = intake_row(&repo, &state, "b", &intake_rules(&state, value));
        assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "{value:?} の周は断る（{}）", told(&out));
        let first = stderr_of(&out).lines().next().map(str::to_owned);
        assert_eq!(first, Some(refusal(&live, None)), "{value:?} の周の断りの 1 行");
        let listed = ls_with(&repo, &state, value);
        assert_eq!(reason_of(&listed, "s2-b"), format!("overlap:{live}/1"), "{value:?} の周の待ち（{}）", told(&listed));
    }
    assert!(commuted(&state).is_empty(), "記帳しない");
    clean(&[&repo, &state]);
}

/// 規則の行が真の周の列の 1 周: 当たらない行は live な便を名乗って結末で待ち、通す行 b は起こす側に入り、同じ周に起こした b と
/// 当たらない行 f と差の file の無い行 d は b を名乗って結末で待つ（起こすのは 1 本）。
#[test]
fn vcwire_dispatch_starts_the_commuting_bead_and_names_the_verdict() {
    let (repo, state) = place();
    let live = live_a(&repo, &state, &intake_rules(&state, Some(true)));
    let out = ls_with(&repo, &state, Some(true));
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "ls は rc 0（{}）", told(&out));
    let reasons: Vec<String> = ["s2-c", "s2-b", "s2-f", "s2-d"].iter().map(|bead| reason_of(&out, bead)).collect();
    let after_b = ["overlap:s2-b/1/not-commuting".to_owned(), "overlap:s2-b/1/no-patch".to_owned()];
    let want = [[format!("overlap:{live}/1/not-commuting"), "-".to_owned()], after_b].concat();
    assert_eq!(reasons, want, "{}", told(&out));
    assert_eq!(count_of(&out), format!("{COUNT} total=4 ready=1"), "起こすのは b の 1 本");
    clean(&[&repo, &state]);
}

/// 列が通した行も、受付の前に main が動いて当たらなくなれば、受付は自分の周の main で判じ直して断る（入口が勝つ）。その後の列の
/// 周も同じ main で待つ。
#[test]
fn vcwire_intake_judges_again_after_main_moves() {
    let (repo, state) = place();
    let on = intake_rules(&state, Some(true));
    let live = live_a(&repo, &state, &on);
    assert_eq!(reason_of(&ls_with(&repo, &state, Some(true)), "s2-b"), "-", "動く前の列は通す");
    let moved: String = (1..=30).map(|n| if n == 25 { "m25\n".to_owned() } else { format!("l{n}\n") }).collect();
    fs::write(repo.join(FILE), moved).expect("main の file を書ける");
    git(&repo, &["commit", "-q", "-am", "main moves"]);
    let out = intake_row(&repo, &state, "b", &on);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "受付は断る（{}）", told(&out));
    assert_eq!(stderr_of(&out).lines().next().map(str::to_owned), Some(refusal(&live, Some("not-commuting"))), "受付の断り");
    let after = ls_with(&repo, &state, Some(true));
    assert_eq!(reason_of(&after, "s2-b"), format!("overlap:{live}/1/not-commuting"), "{}", told(&after));
    clean(&[&repo, &state]);
}
