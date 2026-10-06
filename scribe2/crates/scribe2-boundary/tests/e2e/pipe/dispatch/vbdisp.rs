//! 契約を台帳の bead に置く形の列の歯（接頭辞 `vbdisp_`・親 `tests/e2e/pipe/dispatch.rs` の helper を `use super::*;` で使う）。
//!
//! 欄 acceptance に契約表の導出の形の `[[contract]]` の 1 行・欄 description に本文を置いた bead を、列（`pipe dispatch` と
//! `dispatch ls`）が候補に数え、起こし、列外の鍵（acceptance と本文の digest）で待たせることを、偽の台帳（置き場の `ledger.json` を
//! `cat` する script）と受付 `pipe intake --bead` の外形から測る。

use super::*;
use super::super::{embedded_int, events};
use std::path::PathBuf;
use vessel::fleet::{EventKind, Stage};
use vessel::pipe::bead::digest;
use vessel::pipe::contract::Contract;

/// bead の本文の見本。
const BODY: &str = "本文。";

/// 列の受付の断りの理由の語頭（両方の形の bead は受付の断りの名で待つ）。
const BOTH_FORMS: &str = "admission:contract-bead-both-forms";

/// 契約の bead の写しを組む rules 行の id（埋め込みの値を使う・3 行とも受付が読む）。
const CONTRACT_ROWS: [(&str, &str); 3] = [
    ("contract.open_max", "ContractOpenMax"),
    ("contract.body_max_bytes", "ContractBodyMaxBytes"),
    ("contract.acceptance_max_bytes", "ContractAcceptanceMaxBytes"),
];

/// 契約の行の bead 形（字 `[[contract]]` の行の後に、行 `id` の欄を改行で繋ぐ・欄 section は bead の id が持つので落とす）。
fn acceptance_of(id: &str, write_set: &str) -> String {
    let fields = row_fields(id, &["write-set", "section"], &[&format!("write-set = [\"{write_set}\"]")]);
    format!("[[contract]]\n{}", fields.join("\n"))
}

/// 見本の契約の行 c の bead 形（表の行 a と b の id とも重ならない）。
fn row_c() -> String {
    acceptance_of("c", "src/lib.rs")
}

/// 見本の契約の行 d の bead 形（行 c と交わらず、新しい file の印つきの write-set）。
fn row_d() -> String {
    acceptance_of("d", "+src/d.rs")
}

/// JSON の字の escape（二重引用符・逆斜線・タブ・CR・LF）。
fn escaped(text: &str) -> String {
    text.chars().fold(String::new(), |mut out, c| {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
        out
    })
}

/// 台帳の bead 1 本（[`listed`] の形に欄 description を足す・acceptance と本文は JSON の escape にする・依存の要素は [`listed`] と同じ key）。
fn bead_json(id: &str, status: &str, acceptance: &str, description: &str, deps: &[(&str, &str)]) -> String {
    let deps: Vec<String> = deps
        .iter()
        .map(|(on, kind)| format!("{{\"issue_id\":\"{id}\",\"depends_on_id\":\"{on}\",\"type\":\"{kind}\"}}"))
        .collect();
    format!(
        "{{\"id\":\"{id}\",\"status\":\"{status}\",\"priority\":2,\"labels\":[],\"acceptance_criteria\":\"{}\",\"description\":\"{}\",\"dependencies\":[{}]}}",
        escaped(acceptance),
        escaped(description),
        deps.join(",")
    )
}

/// 列の写しに契約の bead の上限の 3 行（値は埋め込みの値）を足した manifest の path。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn bead_rules(state: &Path) -> String {
    let base = fs::read_to_string(dispatch_rules(state)).expect("列の写しを読める");
    let rows: Vec<String> = CONTRACT_ROWS
        .iter()
        .map(|(id, kind)| {
            format!(
                "[[rule]]\nid = \"{id}\"\nkind = \"{kind}\"\nvalue = {}\nenabled = true\nruling = \"t\"\nruled_at = \"d\"\n",
                embedded_int(id)
            )
        })
        .collect();
    let path = state.join("rules-dispatch-bead.toml");
    fs::write(&path, format!("{base}\n{}", rows.join("\n"))).expect("写しを書ける");
    path.display().to_string()
}

/// `pipe dispatch ls` を 1 回撃つ（契約の bead の上限の行つきの写し）。
fn bead_ls(repo: &Path, state: &Path, bd: &str) -> Output {
    run_pipe(&[
        "dispatch", "ls",
        "--state-dir", &state.display().to_string(),
        "--repo", &repo.display().to_string(),
        "--rules", &bead_rules(state),
        "--bd", bd,
    ])
}

/// 受付 `pipe intake --bead` を 1 回通して run id を返す（`--design` も `--contract` も渡さない・審査は偽 PASS の lens）。
fn bead_intake(repo: &Path, state: &Path, bd: &str, bead: &str) -> String {
    let out = run_pipe(&[
        "intake", "--bead", bead,
        "--repo", &repo.display().to_string(), "--state-dir", &state.display().to_string(),
        "--rules", &bead_rules(state), "--lens", &review_lens_pass(state), "--bd", bd,
    ]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "bead の受付は rc 0: {}", told(&out));
    run_id_of(&out)
}

/// 便の判定 file を FAIL に書き替える（審査の段が置いた材料の写しはそのまま・終端の `Reviewed` になる）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn fail_review(state: &Path, id: &str) {
    fs::write(state.join("pipe").join(id).join(REVIEW_FILE), "{\"verdict\":\"FAIL\"}\n").expect("審査の判定を書ける");
}

/// 表の行 a と b を持つ toy の repo と置き場。
fn toy() -> (PathBuf, PathBuf) {
    let (repo, state) = repo_with_state();
    two_rows(&repo);
    (repo, state)
}

/// 契約の行 c の bead は候補で、理由の欄は字 `-`。設計 pointer の行も契約の行も持たない bead は `no-design-pointer` で待つ。
#[test]
fn vbdisp_bead_contract_is_a_candidate() {
    let (repo, state) = toy();
    let bd = fake_bd(
        &state,
        &[bead_json("s2-vb.1", "open", &row_c(), BODY, &[]), bead_json("s2-vb.2", "open", "字だけの受け入れ", BODY, &[])],
    );
    let out = bead_ls(&repo, &state, &bd);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{}", told(&out));
    assert_eq!(reason_of(&out, "s2-vb.1"), "-", "契約の行の bead は候補（{}）", told(&out));
    assert_eq!(reason_of(&out, "s2-vb.2"), "no-design-pointer", "どちらの形も無い bead は待つ（{}）", told(&out));
    assert_eq!(count_of(&out), format!("{COUNT} total=2 ready=1"), "{}", told(&out));
    clean(&[&repo, &state]);
}

/// 設計 pointer の行と契約の行の両方を持つ bead は、受付の断りの名 `contract-bead-both-forms` で待つ。
#[test]
fn vbdisp_both_forms_wait_as_refused() {
    let (repo, state) = toy();
    let both = format!("{}\ndesign = {DESIGN_FILE}#a", row_c());
    let bd = fake_bd(&state, &[bead_json("s2-vb.1", "open", &both, BODY, &[])]);
    let out = bead_ls(&repo, &state, &bd);
    assert!(reason_of(&out, "s2-vb.1").starts_with(BOTH_FORMS), "両方の形は断りの名で待つ（{}）", told(&out));
    assert_eq!(count_of(&out), format!("{COUNT} total=1 ready=0"), "{}", told(&out));
    clean(&[&repo, &state]);
}

/// 契約の行 c の bead は `pipe dispatch` で便がちょうど 1 本でき、便の契約 file の design は置き場の写しの path と行 id（`--design` なしで起きる）。
#[test]
fn vbdisp_fired_bead_contract_runs_without_design() {
    let (repo, state) = toy();
    let bead = "s2-vb.1";
    let bd = fake_bd(&state, &[bead_json(bead, "open", &row_c(), BODY, &[])]);
    let out = run_pipe(&[
        "dispatch",
        "--state-dir", &state.display().to_string(),
        "--repo", &repo.display().to_string(),
        "--rules", &bead_rules(&state),
        "--bd", &bd,
        "--lens", &review_lens_pass(&state),
        "--runner", "true",
    ]);
    assert_eq!(stdout_of(&out).trim_end(), "dispatch=started:1,resumed:0,waiting:0", "1 本起こす（{}）", told(&out));
    assert_eq!(created(&state, &[bead], 1), 1, "便がちょうど 1 本（{}）", told(&out));
    let runs: Vec<String> = events(&state)
        .iter()
        .filter(|found| found.kind == EventKind::RunCreated && found.bead == bead)
        .map(|found| found.run.clone())
        .collect();
    assert_eq!(runs.len(), 1, "RunCreated は 1 件: {runs:?}");
    let run = runs.first().cloned().unwrap_or_default();
    let design = Contract::load(&state.join("pipe").join(&run).join("contract.toml")).map(|found| found.design).unwrap_or_default();
    let want = format!("{}/bead-contracts/{bead}/{}.toml#c", state.display(), digest(&row_c(), BODY));
    assert_eq!(design, want, "便の契約 file の design は写しの path と行 id");
    clean(&[&repo, &state]);
}

/// 受付で起こして判定を FAIL にした契約の bead は `settled`。鍵は acceptance と本文の digest で、どちらかの 1 字が替わると列に戻り、戻すと
/// 前の便がまた選ばれる。
#[test]
fn vbdisp_settled_key_is_the_bead_digest() {
    let (repo, state) = toy();
    let bead = "s2-vb.1";
    let ledger = |acceptance: &str, body: &str| fake_bd(&state, &[bead_json(bead, "open", acceptance, body, &[])]);
    let bd = ledger(&row_c(), BODY);
    let id = bead_intake(&repo, &state, &bd, bead);
    fail_review(&state, &id);
    let first = bead_ls(&repo, &state, &bd);
    let settled = reason_of(&first, bead);
    assert!(settled.starts_with("settled:"), "同じ digest の便は列外（{}）", told(&first));
    assert!(settled.ends_with("/Reviewed"), "段は Reviewed: {settled}");
    assert_eq!(count_of(&first), format!("{COUNT} total=1 ready=0"), "起こさない");
    let again = bead_ls(&repo, &state, &bd);
    assert_eq!(reason_of(&again, bead), settled, "何も替えない 2 度目も同じ字（{}）", told(&again));

    ledger(&row_c(), "本文！");
    let body_moved = bead_ls(&repo, &state, &bd);
    assert_eq!(reason_of(&body_moved, bead), "-", "本文の 1 字で列に戻る（{}）", told(&body_moved));
    assert_eq!(count_of(&body_moved), format!("{COUNT} total=1 ready=1"), "戻った契約は起こせる（{}）", told(&body_moved));

    let title_moved = row_c().replace("縦 1 本を通す", "縦 2 本を通す");
    assert_ne!(title_moved, row_c(), "前提: title の字が動く");
    ledger(&title_moved, BODY);
    let accept = bead_ls(&repo, &state, &bd);
    assert_eq!(reason_of(&accept, bead), "-", "acceptance の title の 1 字で列に戻る（{}）", told(&accept));
    assert_eq!(count_of(&accept), format!("{COUNT} total=1 ready=1"), "{}", told(&accept));

    ledger(&row_c(), BODY);
    let restored = bead_ls(&repo, &state, &bd);
    assert_eq!(reason_of(&restored, bead), settled, "両方を戻せば前の便がまた選ばれる（{}）", told(&restored));
    clean(&[&repo, &state]);
}

/// 契約の bead は兄弟を持たない: 契約の行 c の bead B が FAIL で終端しても、契約の行 d の bead C は B を待たない。
#[test]
fn vbdisp_bead_contracts_have_no_siblings() {
    let (repo, state) = toy();
    let (b, c) = ("s2-vb.1", "s2-vb.2");
    let bd = fake_bd(&state, &[bead_json(b, "open", &row_c(), BODY, &[])]);
    let id = bead_intake(&repo, &state, &bd, b);
    fail_review(&state, &id);
    let both = fake_bd(&state, &[bead_json(b, "open", &row_c(), BODY, &[]), bead_json(c, "open", &row_d(), BODY, &[])]);
    let out = bead_ls(&repo, &state, &both);
    assert_eq!(reason_of(&out, c), "-", "兄弟の待ちを持たない（{}）", told(&out));
    assert!(reason_of(&out, b).starts_with("settled:"), "B は列外（{}）", told(&out));
    clean(&[&repo, &state]);
}

/// 台帳の blocks が契約の bead の順を持つ: 契約の行 d の bead C は、契約の行 c の open な bead B に blocks で依る周は `dependency:<B>`
/// で待ち、B を closed にした周は待たない。
#[test]
fn vbdisp_blocks_order_bead_contracts() {
    let (repo, state) = toy();
    let (b, c) = ("s2-vb.1", "s2-vb.2");
    let waits = [("s2-vb.1", "blocks")];
    let bd = fake_bd(&state, &[bead_json(b, "open", &row_c(), BODY, &[]), bead_json(c, "open", &row_d(), BODY, &waits)]);
    let held = bead_ls(&repo, &state, &bd);
    assert_eq!(reason_of(&held, c), format!("dependency:{b}"), "open な B を待つ（{}）", told(&held));
    let bd = fake_bd(&state, &[bead_json(b, "closed", &row_c(), BODY, &[]), bead_json(c, "open", &row_d(), BODY, &waits)]);
    let freed = bead_ls(&repo, &state, &bd);
    assert_eq!(reason_of(&freed, c), "-", "B が closed なら待たない（{}）", told(&freed));
    assert_eq!(count_of(&freed), format!("{COUNT} total=1 ready=1"), "{}", told(&freed));
    clean(&[&repo, &state]);
}

/// 段 `Reviewed` の event の detail（便 `run` のもの・1 件）。
fn reviewed_detail(state: &Path, run: &str) -> String {
    let found: Vec<String> = events(state)
        .iter()
        .filter(|event| event.run == run && event.kind == EventKind::RunStage && event.stage == Some(Stage::Reviewed))
        .filter_map(|event| event.detail.clone())
        .collect();
    assert_eq!(found.len(), 1, "段 Reviewed の event は 1 件: {found:?}");
    found.into_iter().next().unwrap_or_default()
}

/// 契約の bead の審査の event の detail は `verdict:PASS` に `material:` と `copy:` の 2 語が続き、2 つの値は同じ 16 桁。`--design` の行を
/// 通した便の detail は `verdict:PASS` だけ。
#[test]
fn vbdisp_review_detail_carries_material_and_copy_digests() {
    let (repo, state) = toy();
    let bead = "s2-vb.1";
    let bd = fake_bd(&state, &[bead_json(bead, "open", &row_c(), BODY, &[])]);
    let id = bead_intake(&repo, &state, &bd, bead);
    let detail = reviewed_detail(&state, &id);
    let words: Vec<&str> = detail.split(' ').collect();
    assert_eq!(words.first(), Some(&"verdict:PASS"), "{detail}");
    assert_eq!(words.len(), 3, "verdict に material と copy の 2 語が続く: {detail}");
    let material = words.get(1).and_then(|word| word.strip_prefix("material:")).unwrap_or_default();
    let copy = words.get(2).and_then(|word| word.strip_prefix("copy:")).unwrap_or_default();
    assert!(material.len() == 16 && material.chars().all(|c| c.is_ascii_hexdigit()), "material は 16 桁: {detail}");
    assert_eq!(material, copy, "2 つの値は同じ 16 桁: {detail}");
    stop_run_ok(&state, &id);
    let design = intake_bead(&repo, &state, &format!("{DESIGN_FILE}#a"), "s2-toy.1");
    assert_eq!(reviewed_detail(&state, &design), "verdict:PASS", "表の側の便の detail は替わらない");
    clean(&[&repo, &state]);
}
