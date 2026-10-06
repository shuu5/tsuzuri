//! gate が終わりの門の緑を持ち越す歯（接頭辞 `vgcarry_`・設計 pipeline.md §66 形 11・tsuzuri の判断の記録 ADR-65・親
//! `tests/e2e/pipe/spawn.rs` の helper を `use super::*` で使う）。
//!
//! 門の最後の要約が緑で、撃った木と base が gate の木と base に等しい便だけ、gate は共通 verify を撃たずに skip record を
//! 1 本置く。要約の鍵を 1 つだけ崩した便と、門の後に木が動いた便は、gate が共通 verify を撃つ。

use super::*;
use vessel::fleet::json_lite::Value;

/// verify の record のうち kind が `common` の行（gate の共通 verify の段）。
fn common_rows(state: &Path, id: &str) -> Vec<Vec<(String, Value)>> {
    verify_rows(state, id)
        .into_iter()
        .filter(|row| field(row, "kind").and_then(Value::as_str) == Some("common"))
        .collect()
}

/// record の 1 欄。
fn field<'a>(row: &'a [(String, Value)], key: &str) -> Option<&'a Value> {
    row.iter().find(|(name, _)| name == key).map(|(_, value)| value)
}

/// 便の worktree の `HEAD^{tree}`。
fn tree_of(repo: &Path, id: &str) -> String {
    git(&worktree_of(repo, id), &["rev-parse", "HEAD^{tree}"])
}

/// 緑の中身を 1 回 commit する runner で spawn した便（repo・置き場・便 id）。
fn green_run() -> (PathBuf, PathBuf, String) {
    let (repo, state, id) = gate_run_intake();
    let runner = turn_runner(&state, &[commit_turn("green")]);
    let out = end_gate_spawn(&repo, &state, &id, &runner, None);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{} / {}", stdout_of(&out), stderr_of(&out));
    assert_eq!(end_gate_words(&state, &id), ["green"], "門は緑");
    (repo, state, id)
}

/// 偽 lens（PASS）で gate を 1 回撃ち、rc 0 を確かめる。
fn gate_pass(repo: &Path, state: &Path, id: &str) {
    let lens = fake_lens(&state.join("lens-ran"), &lens_verdict("PASS"));
    let gated = gate_once(repo, state, id, Some(&lens));
    assert_eq!(gated.status.code(), Some(i32::from(RC_OK)), "{} / {}", stdout_of(&gated), stderr_of(&gated));
}

/// 門の最後の要約の行の字 `from` を `to` に 1 か所だけ替える（ほかの行と欄は替えない）。
fn rewrite_summary(state: &Path, id: &str, from: &str, to: &str) {
    let path = state.join("pipe").join(id).join("end-gate.jsonl");
    let mut lines = end_gate_lines(state, id);
    let at = lines.iter().rposition(|line| line.starts_with("{\"end_gate\":")).unwrap_or_default();
    let line = lines.get(at).cloned().unwrap_or_default();
    assert_eq!(line.matches(from).count(), 1, "替える字は要約の行に 1 つ: {line}");
    if let Some(slot) = lines.get_mut(at) {
        *slot = line.replacen(from, to, 1);
    }
    fs::write(&path, format!("{}\n", lines.join("\n"))).unwrap_or_default();
}

/// 便を崩す手（repo・置き場・便 id を受ける）。
type Break = Box<dyn Fn(&Path, &Path, &str)>;

/// 共通 verify を撃った record の数（`rc` を持つ kind common の行）と、持ち越しの record の数（`skipped` を持つ行）。
fn fired_and_carried(state: &Path, id: &str) -> (usize, usize) {
    let rows = common_rows(state, id);
    let fired = rows.iter().filter(|row| field(row, "rc").is_some()).count();
    let carried = rows.iter().filter(|row| field(row, "skipped").is_some()).count();
    (fired, carried)
}

/// 門が緑の便は、要約が撃った木と base を持ち、gate は共通 verify を撃たず、kind common の record は持ち越しの 1 本だけで、
/// その欄は skipped common・門と gate の木・reason end-gate-green・from end-gate.jsonl#1。契約の verify は gate が撃つ。
#[test]
fn vgcarry_green_end_gate_on_the_same_tree_is_carried_by_the_gate() {
    let (repo, state, id) = green_run();
    let tree = tree_of(&repo, &id);
    let summary = end_gate_lines(&state, &id).into_iter().rfind(|line| line.starts_with("{\"end_gate\":")).unwrap_or_default();
    assert!(summary.contains(&format!("\"tree\":\"{tree}\"")), "要約は撃った木を持つ: {summary}");
    assert!(summary.contains("\"base\":\""), "要約は撃った base を持つ: {summary}");
    gate_pass(&repo, &state, &id);
    let rows = common_rows(&state, &id);
    assert_eq!(rows.len(), 1, "kind common の record は 1 本: {rows:?}");
    let row = rows.first().cloned().unwrap_or_default();
    assert_eq!(field(&row, "rc"), None, "共通 verify を撃っていない: {row:?}");
    for (key, want) in [("skipped", "common"), ("tree", tree.as_str()), ("reason", "end-gate-green"), ("from", "end-gate.jsonl#1")] {
        assert_eq!(field(&row, key).and_then(Value::as_str), Some(want), "欄 {key}: {row:?}");
    }
    let contract = verify_rows(&state, &id)
        .into_iter()
        .filter(|row| field(row, "kind").and_then(Value::as_str) == Some("contract"))
        .count();
    assert_eq!(contract, 1, "契約の verify は gate が撃つ");
    clean(&[&repo, &state]);
}

/// 要約の鍵を 1 つだけ崩した便（木を別の sha・base を別の sha・語を exhausted・記録を消す）と、門の後に worktree へ緑の
/// commit を足した便は、gate が共通 verify を撃ち（kind common の record は rc を持つ 1 本）、持ち越しの record を置かない。
#[test]
fn vgcarry_each_broken_key_or_a_moved_tree_makes_the_gate_fire() {
    let breaks: [(&str, Break); 5] = [
        ("木", Box::new(|repo, state, id| rewrite_summary(state, id, &tree_of(repo, id), &"0".repeat(40)))),
        ("base", Box::new(|_, state, id| {
            let summary = end_gate_lines(state, id).into_iter().rfind(|line| line.starts_with("{\"end_gate\":")).unwrap_or_default();
            let base = summary.split_once("\"base\":\"").and_then(|(_, rest)| rest.split('"').next()).unwrap_or_default().to_owned();
            rewrite_summary(state, id, &base, &"0".repeat(40));
        })),
        ("語", Box::new(|_, state, id| rewrite_summary(state, id, "\"result\":\"green\"", "\"result\":\"exhausted\""))),
        ("記録", Box::new(|_, state, id| {
            let _ = fs::remove_file(state.join("pipe").join(id).join("end-gate.jsonl"));
        })),
        ("木の動き", Box::new(|repo, _, id| {
            let tree = worktree_of(repo, id);
            fs::write(tree.join("src").join("lib.rs"), "green\ngreen-two\n").unwrap_or_default();
            git(&tree, &["commit", "-q", "-am", "green-two"]);
        })),
    ];
    for (name, broken) in &breaks {
        let (repo, state, id) = green_run();
        broken(&repo, &state, &id);
        gate_pass(&repo, &state, &id);
        assert_eq!(fired_and_carried(&state, &id), (1, 0), "{name} を崩した便は gate が共通 verify を撃つ: {:?}", common_rows(&state, &id));
        clean(&[&repo, &state]);
    }
}

/// 門の時に追跡外の file が在った便（runner が緑を commit し、追跡外の file を残した）の要約は、緑でも木と base を持たない。
#[test]
fn vgcarry_an_unclean_worktree_at_the_end_gate_writes_no_tree() {
    let (repo, state, id) = gate_run_intake();
    let runner = turn_runner(&state, &[format!("printf 'stray\\n' > stray.txt\n{}", commit_turn("green").replace("git add -A", "git add src/lib.rs"))]);
    let out = end_gate_spawn(&repo, &state, &id, &runner, None);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{} / {}", stdout_of(&out), stderr_of(&out));
    assert_eq!(end_gate_words(&state, &id), ["green"], "門は緑");
    let summary = end_gate_lines(&state, &id).into_iter().rfind(|line| line.starts_with("{\"end_gate\":")).unwrap_or_default();
    assert!(!summary.contains("\"tree\"") && !summary.contains("\"base\""), "clean でない木は名指さない: {summary}");
    clean(&[&repo, &state]);
}
