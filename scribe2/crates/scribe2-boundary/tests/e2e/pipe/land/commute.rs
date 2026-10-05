//! 差の当たりで通した組の後の着地の記帳の歯（接頭辞 `vcledger_`・判断の記録 ADR-60 の決定 (4)・親 `tests/e2e/pipe/land.rs` の
//! helper を `use super::*` で使う）。
//!
//! 受付が通した組の行（kind `OverlapCommuted`）を置き場に手で 1 件積み、組が名指す便の着地の経路（積み直しの衝突・stale・追随の
//! 再 gate の不合格・着地の列の候補の木の赤・着地）が kind `OverlapFollowed` に記す行を、出来事の記録から外形で測る。

use super::*;
use vessel::fleet::Case;

/// 差の file の置き場（toy repo の根から）。
const PATCH: &str = "docs/p/x.patch";

/// 便の契約の bead（[`intake`] が起こす便）。
const BEAD: &str = "s2-2e5";

/// 組の行を 1 件積む（kind `OverlapCommuted`・bead は候補の契約・detail は組）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn pair(state: &Path, detail: &str) {
    let event = Event {
        schema: vessel::fleet::SCHEMA,
        ts: "2026-10-05T00:00:00Z".to_owned(),
        kind: EventKind::OverlapCommuted,
        run: String::new(),
        bead: "s2-zz".to_owned(),
        host: "h".to_owned(),
        actor: vessel::fleet::ACTOR_MACHINE.to_owned(),
        stage: None,
        seat: None,
        pid: None,
        detail: Some(detail.to_owned()),
        allowance: None,
        registration: None,
        mark: None,
        account: None,
        cost: None,
        rule: None,
        case: Some(Case::Commuted),
    };
    let policy = vessel::fleet::store::LockPolicy::embedded().expect("埋め込みの lock 行を読める");
    vessel::fleet::store::append(state, &event, policy).expect("組の行を積める");
}

/// 記帳の行（kind `OverlapFollowed` の bead と detail・積んだ順）。
fn followed(state: &Path) -> Vec<(String, String)> {
    events(state)
        .into_iter()
        .filter(|event| event.kind == EventKind::OverlapFollowed)
        .map(|event| (event.bead, event.detail.unwrap_or_default()))
        .collect()
}

/// 便 `id`（bead [`BEAD`]）の記帳の行 1 つ（`word` は語と尾）。
fn row(id: &str, word: &str) -> (String, String) {
    (BEAD.to_owned(), format!("run={id} word={word}"))
}

/// `+x` と `+y` の 2 行を足す差の file と、それを欄 patch で名指す契約の行を commit して設計 pointer を返す（便の実装は `x` の
/// 1 行だけを足すので、着地の差の変えた行は 1・差の file は 2・片方にだけ在る行は `+y` の 1）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn patched_contract(repo: &Path) -> String {
    fs::create_dir_all(repo.join("docs/p")).expect("差の dir を作れる");
    let text = "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1,3 @@\n // seed\n+x\n+y\n";
    fs::write(repo.join(PATCH), text).expect("差の file を書ける");
    write_contract(repo, &[], &[&format!("patch = \"{PATCH}\"")])
}

/// 欄 patch を持つ便を PASS まで通し、組の行 `detail`（`{id}` を便 id に替える）を積んで着地させ、便 id と記帳の行を返す。
fn land_paired(detail: &str) -> (String, Vec<(String, String)>) {
    let (repo, state) = repo_with_state();
    let path = patched_contract(&repo);
    let id = gated_pass(&repo, &state, &path, &state.join("lens-ran"));
    pair(&state, &detail.replace("{id}", &id));
    let out = land_once(&repo, &state, &id);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "着地する: {}", stderr_of(&out));
    let found = followed(&state);
    clean(&[&repo, &state]);
    (id, found)
}

/// 組の行の `with=` が名指す便の着地は、語 landed に、着地の差の変えた行の数・差の file の変えた行の数・片方にだけ在る行の数を
/// 添えて 1 件記す（`diff=1 patch=2 off=1`）。
#[test]
fn vcledger_landing_of_a_named_run_records_the_line_counts() {
    let (id, found) = land_paired("run=s2-zz-1 with=s2-zz-2,{id} files=src/lib.rs main=0");
    assert_eq!(found, [row(&id, "landed diff=1 patch=2 off=1")]);
}

/// 組の行が便を名指さない（`with=` の 1 つだけを別の便 id にした）周の着地は何も記さない。
#[test]
fn vcledger_landing_of_a_run_the_pair_does_not_name_records_nothing() {
    let (_, found) = land_paired("run=s2-zz-1 with=s2-zz-2,s2-zz-3 files=src/lib.rs main=0");
    assert!(found.is_empty(), "記さない: {found:?}");
}

/// 撃ち直しの間に main がまた動いた周は語 stale を記し、追随し直した後の着地を語 landed で記す（この順・欄 patch の無い契約は
/// `patch=- off=-`）。
#[test]
fn vcledger_stale_and_the_landing_after_it_are_recorded_in_order() {
    let (repo, state) = repo_with_state();
    let path = write_contract(&repo, &[], &[]);
    let id = gated_pass(&repo, &state, &path, &state.join("lens-ran"));
    pair(&state, &format!("run={id} with=s2-zz-2 files=src/lib.rs main=0"));
    commit_other_in_scope(&repo);
    let out = land_extra(&repo, &state, &id, &["--lens", &racing_lens(&repo, &state, false)]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "追随し直して着地する: {}", stderr_of(&out));
    assert_eq!(followed(&state), [row(&id, "stale"), row(&id, "landed diff=1 patch=- off=-")]);
    clean(&[&repo, &state]);
}

/// 追随の積み直しが衝突した周は語 conflict を 1 件記す（着地しない）。
#[test]
fn vcledger_a_rebase_conflict_is_recorded() {
    let (repo, state) = repo_with_state();
    let runner = stub_runner(&state, KEEP_CONFLICT);
    let (id, _, _) = conflicting_run(&repo, &state, &state.join("lens-ran"), &runner);
    pair(&state, &format!("run={id} with=s2-zz-2 files=src/lib.rs main=0"));
    let out = land_extra(&repo, &state, &id, &["--runner", &runner]);
    assert_eq!(out.status.code(), Some(i32::from(RC_INCONCLUSIVE)), "起こし直して止まる: {}", stderr_of(&out));
    assert_eq!(followed(&state), [row(&id, "conflict")]);
    clean(&[&repo, &state]);
}

/// 追随の再 gate の判定 `verdict` の周に、組が名指す便が記す行（main を面の内へ 1 つ進めてから着地させる）。
fn regated(verdict: &str) -> (String, Vec<(String, String)>) {
    let (repo, state) = repo_with_state();
    let path = write_contract(&repo, &[], &[]);
    let marker = state.join("lens-ran");
    let id = gated_pass(&repo, &state, &path, &marker);
    pair(&state, &format!("run={id} with=s2-zz-2 files=src/lib.rs main=0"));
    commit_other_in_scope(&repo);
    let out = land_extra(&repo, &state, &id, &["--lens", &fake_lens(&marker, &lens_verdict(verdict))]);
    assert_ne!(out.status.code(), Some(i32::from(RC_OK)), "{verdict} の撃ち直しは着地しない: {}", stdout_of(&out));
    let found = followed(&state);
    clean(&[&repo, &state]);
    (id, found)
}

/// 追随の再 gate が FAIL の周は語 regate-fail を 1 件記し、INCONCLUSIVE で止まった周は何も記さない。
#[test]
fn vcledger_a_failed_regate_is_recorded_and_an_inconclusive_one_is_not() {
    let (id, found) = regated("FAIL");
    assert_eq!(found, [row(&id, "regate-fail")]);
    let (_, found) = regated("INCONCLUSIVE");
    assert!(found.is_empty(), "INCONCLUSIVE は記さない: {found:?}");
}

/// 着地の列の候補の木の検査が赤の周は、積んだ便のうち組が名指す便（先頭 a と 3 本目 c）に語 train-red を尾 `train=3` で積んだ
/// 順に記し、名指さない 2 本目 b には記さない。列を解いた後に既存の経路で着地した先頭は語 landed も記す。
#[test]
fn vcledger_a_red_candidate_tree_is_recorded_for_each_named_rider() {
    let (repo, state) = repo_with_state();
    let [id_a, _id_b, id_c] = train_runs(&repo, &state, &state.join("lens-ran"), [ALL_GREEN, ALL_GREEN, TRAIN_RED]);
    pair(&state, &format!("run={id_a} with={id_c} files=crates/toy/a.rs main=0"));
    let rules = write_rules_train(&state, "rules-train.toml", Some(3));
    let out = land_extra(&repo, &state, &id_a, &["--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "先頭は着地する: {}", stderr_of(&out));
    assert!(stdout_of(&out).contains(&format!("run={id_a} train=3 dissolved why=verify")), "列を解いた: {}", stdout_of(&out));
    let c = ("s2-4cz".to_owned(), format!("run={id_c} word=train-red train=3"));
    assert_eq!(followed(&state), [row(&id_a, "train-red train=3"), c, row(&id_a, "landed diff=1 patch=- off=-")]);
    clean(&[&repo, &state]);
}
