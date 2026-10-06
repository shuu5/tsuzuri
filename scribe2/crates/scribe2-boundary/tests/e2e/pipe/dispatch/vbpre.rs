//! 契約を台帳の bead に置く形の事前審査・直しの束・memo の同梱の引き金の歯（接頭辞 `vbpre_`・親 `tests/e2e/pipe/dispatch.rs` の helper を
//! `use super::*;` で使う）。
//!
//! 欄 acceptance に契約表の導出の形の `[[contract]]` の 1 行・欄 description に本文を置いた bead を、事前審査が母集団と祖先の walk に数え、
//! 束が写しの pointer と bead の周の測り直しで書き、memo の同梱の引き金が開いた契約の write-set に数えることを、偽の台帳と起こす側の 1 周と
//! `dispatch ls` の外形から測る。toy は行の素の項目を base の木に seed せず、祖先の宣言を貸さない実装では行の write-set が解けない形にする。

use super::*;
use vessel::fleet::json_lite;

/// bead の本文の見本。
const BODY: &str = "本文。";

/// 契約の bead の写しを組む rules 行（id と kind・値は埋め込みの値）。
const CONTRACT_ROWS: [(&str, &str); 3] = [
    ("contract.open_max", "ContractOpenMax"),
    ("contract.body_max_bytes", "ContractBodyMaxBytes"),
    ("contract.acceptance_max_bytes", "ContractAcceptanceMaxBytes"),
];

/// 表の行 `id` の欄（write-set だけ差し替える）。
fn table_row(id: &str, write_set: &str) -> Vec<String> {
    row_fields(id, &["write-set"], &[&format!("write-set = [\"{write_set}\"]")])
}

/// 見本の bead の契約の acceptance（字 `[[contract]]` の行の後に、行 `id` の欄を改行で繋ぐ・欄 section は bead の id が持つので落とす）。
fn contract_of(id: &str, write_set: &str) -> String {
    let fields = row_fields(id, &["write-set", "section"], &[&format!("write-set = [\"{write_set}\"]")]);
    format!("[[contract]]\n{}", fields.join("\n"))
}

/// 台帳の bead 1 件（[`listed`] と同じ欄の並びに欄 description を足す・acceptance と本文は JSON の字にする・依存は (依存先, 種別) の列）。
fn bead_of(id: &str, status: &str, acceptance: &str, description: &str, deps: &[(&str, &str)]) -> String {
    let deps: Vec<String> = deps
        .iter()
        .map(|(on, kind)| format!("{{\"issue_id\":\"{id}\",\"depends_on_id\":\"{on}\",\"type\":\"{kind}\"}}"))
        .collect();
    format!(
        "{{\"id\":\"{id}\",\"status\":\"{status}\",\"priority\":2,\"labels\":[],\"acceptance_criteria\":{},\"description\":{},\"dependencies\":[{}]}}",
        json_lite::quote(acceptance),
        json_lite::quote(description),
        deps.join(",")
    )
}

/// 台帳の memo 1 件（[`waiting`] の `memo_of` と同じ欄と本文の形・時刻の欄は持たない）。
fn memo_of(id: &str, triggers: &[&str]) -> String {
    let description = format!("### 出所\nx\n### 観測\nx\n### 候補\nx\n### 昇格条件\n{}\n", triggers.join("\n"));
    format!(
        "{{\"id\":\"{id}\",\"status\":\"open\",\"priority\":2,\"labels\":[\"intake:memo\"],\"description\":{},\"notes\":\"\"}}",
        json_lite::quote(&description)
    )
}

/// 表の行と file を**そのまま** 1 回 commit した toy の repo と置き場（行の素の項目を seed しない＝base に無い file を行が名指せる）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn bead_repo(rows: &[Vec<String>], files: &[(&str, &str)]) -> (std::path::PathBuf, std::path::PathBuf) {
    let (repo, state) = repo_with_state();
    write_design(&repo, &design_doc_rows(rows));
    for (path, body) in files {
        let target = repo.join(path);
        fs::create_dir_all(target.parent().expect("親 dir が在る")).expect("dir を作れる");
        fs::write(&target, body).expect("file を書ける");
    }
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "vbpre-rows"]);
    (repo, state)
}

/// 列の写しに契約の bead の上限の 3 行（値は埋め込みの値）を足した manifest の path（置き場の `rules-vbpre.toml`）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn rules_of(state: &Path) -> String {
    let base = fs::read_to_string(dispatch_rules(state)).expect("列の写しを読める");
    let rows: Vec<String> = CONTRACT_ROWS
        .iter()
        .map(|(id, kind)| {
            let value = super::super::embedded_int(id);
            format!("[[rule]]\nid = \"{id}\"\nkind = \"{kind}\"\nvalue = {value}\nenabled = true\nruling = \"t\"\nruled_at = \"d\"\n")
        })
        .collect();
    let path = state.join("rules-vbpre.toml");
    fs::write(&path, format!("{base}\n{}", rows.join("\n"))).expect("写しを書ける");
    path.display().to_string()
}

/// 介入 `hold` を打つ（起こさない・`hold` は 1 周を撃たない）。
fn hold_all(state: &Path, beads: &[&str]) {
    for &bead in beads {
        let out = run_pipe(&["dispatch", "hold", bead, "--reason", "起こさない", "--state-dir", &state.display().to_string()]);
        assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "hold は rc 0（{}）", told(&out));
    }
}

/// 起こす側の手動の 1 周（道具つき・runner は偽の 1 行）。
fn turn(repo: &Path, state: &Path, bd: &str) -> Output {
    let out = run_pipe(&[
        "dispatch",
        "--state-dir", &state.display().to_string(),
        "--repo", &repo.display().to_string(),
        "--rules", &rules_of(state),
        "--bd", bd,
        "--lens", &review_lens_pass(state),
        "--runner", "true",
    ]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "1 周は rc 0（{}）", told(&out));
    out
}

/// `dispatch ls` を 1 回撃つ。
fn list(repo: &Path, state: &Path, bd: &str) -> Output {
    let out = run_pipe(&[
        "dispatch", "ls",
        "--state-dir", &state.display().to_string(),
        "--repo", &repo.display().to_string(),
        "--rules", &rules_of(state),
        "--bd", bd,
    ]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "ls は rc 0（{}）", told(&out));
    out
}

/// 置き場の事前審査の結果の file の本文（無ければ空）。
fn kept(state: &Path, bead: &str) -> String {
    fs::read_to_string(state.join("pipe").join("precheck").join(bead)).unwrap_or_default()
}

/// 結果の file の `<key>` で始まる最初の行の残り（無ければ空）。
fn kept_field(state: &Path, bead: &str, key: &str) -> String {
    kept(state, bead).lines().find_map(|line| line.strip_prefix(key)).unwrap_or_default().to_owned()
}

/// dir の下の file の path の列（dir が無ければ空・名の順）。
fn files_under(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut found: Vec<std::path::PathBuf> =
        fs::read_dir(dir).map(|entries| entries.flatten().map(|entry| entry.path()).filter(|path| path.is_file()).collect()).unwrap_or_default();
    found.sort();
    found
}

/// 表の行 a（`+src/fresh.rs` を宣言する）を持つ toy の (repo, 置き場)。行 a を指す bead s2-pre.1 は hold する。
fn waiting_toy() -> (std::path::PathBuf, std::path::PathBuf) {
    let (repo, state) = bead_repo(&[table_row("a", "+src/fresh.rs")], &[]);
    hold_all(&state, &["s2-pre.1"]);
    (repo, state)
}

/// [`waiting_toy`] の台帳: s2-pre.1 に blocks で依る bead の契約 c（`src/fresh.rs`・本文は `body`）と d（`src/nowhere.rs`）。
fn waiting_ledger(state: &Path, body: &str) -> String {
    let waits = [("s2-pre.1", "blocks")];
    fake_bd(
        state,
        &[
            issue("s2-pre.1", 2, "a"),
            bead_of("s2-vb.c", "open", &contract_of("c", "src/fresh.rs"), body, &waits),
            bead_of("s2-vb.d", "open", &contract_of("d", "src/nowhere.rs"), BODY, &waits),
        ],
    )
}

/// 依存を待つ bead の契約は事前審査の母集団に入る: c（`src/fresh.rs`）は祖先 a の宣言の `+` で clean、d（`src/nowhere.rs`）はどこにも無い
/// file で確定の finding を持ち、`dispatch ls` は c の行を `base=current` で出す。
#[test]
fn vbpre_bead_contract_waiting_rows_are_prechecked() {
    let (repo, state) = waiting_toy();
    let bd = waiting_ledger(&state, BODY);
    turn(&repo, &state, &bd);
    assert_eq!(kept_field(&state, "s2-vb.c", "result="), "clean", "祖先の宣言の + を素で持つ契約は clean: {}", kept(&state, "s2-vb.c"));
    assert_eq!(kept_field(&state, "s2-vb.d", "result="), "firm:1,provisional:0", "どこにも無い file は確定: {}", kept(&state, "s2-vb.d"));
    let firm = kept(&state, "s2-vb.d");
    let line = firm.lines().find(|line| line.starts_with("finding=")).unwrap_or_default();
    assert!(line.starts_with("finding=firm name=write-set-item-unresolved at=files:src/nowhere.rs new=true"), "{firm}");
    let listed = list(&repo, &state, &bd);
    let line = "[DISPATCH-PRECHECK] bead=s2-vb.c result=clean base=current";
    assert!(stdout_of(&listed).lines().any(|found| found == line), "{}", told(&listed));
    clean(&[&repo, &state]);
}

/// bead の契約を祖先に持つ表の行は、その祖先の宣言を重ねて予想する: base の木に `src/fresh.rs` が無くても、祖先 e が `+src/fresh.rs` を宣言して
/// いるので、行 b（`src/fresh.rs`）の bead は clean。
#[test]
fn vbpre_bead_ancestor_lends_its_declared_write_set() {
    let (repo, state) = bead_repo(&[table_row("b", "src/fresh.rs")], &[]);
    let bd = fake_bd(
        &state,
        &[
            bead_of("s2-vb.e", "open", &contract_of("e", "+src/fresh.rs"), BODY, &[]),
            listed("s2-pre.2", "open", 2, &format!("design = {DESIGN_FILE}#b"), &[("s2-vb.e", "blocks")]),
        ],
    );
    hold_all(&state, &["s2-vb.e"]);
    turn(&repo, &state, &bd);
    assert_eq!(kept_field(&state, "s2-pre.2", "result="), "clean", "祖先の宣言を貸す: {}", kept(&state, "s2-pre.2"));
    clean(&[&repo, &state]);
}

/// 周の鍵は bead の中身で替わる: 同じ台帳の 2 度の周は同じ key で、本文の 1 字を替えた周は違う key になり、結果は clean のまま。
#[test]
fn vbpre_bead_text_is_in_the_key() {
    let (repo, state) = waiting_toy();
    let bd = waiting_ledger(&state, BODY);
    turn(&repo, &state, &bd);
    let first = kept_field(&state, "s2-vb.c", "key=");
    turn(&repo, &state, &bd);
    assert_eq!(kept_field(&state, "s2-vb.c", "key="), first, "同じ台帳の 2 度目は同じ key");
    assert!(first.contains("copy:"), "key に写しの digest を持つ: {first}");
    let moved = waiting_ledger(&state, "本文！");
    turn(&repo, &state, &moved);
    assert_ne!(kept_field(&state, "s2-vb.c", "key="), first, "本文の 1 字で key が替わる");
    assert_eq!(kept_field(&state, "s2-vb.c", "result="), "clean", "{}", kept(&state, "s2-vb.c"));
    clean(&[&repo, &state]);
}

/// 束は bead の契約の写しの pointer を名乗り、bead の周で測り直す: 写しは bead ごとに 1 file で、束の見出しの pointer は d の写しを指し、
/// 測り直しの argv は `pipe preflight --bead s2-vb.d --repo <repo>`、節の部は本文を持つ。
#[test]
fn vbpre_bundle_names_the_bead_copy_and_remeasures_by_bead() {
    let (repo, state) = waiting_toy();
    turn(&repo, &state, &waiting_ledger(&state, BODY));
    let copies = |bead: &str| files_under(&state.join("bead-contracts").join(bead));
    assert_eq!((copies("s2-vb.c").len(), copies("s2-vb.d").len()), (1, 1), "写しは bead ごとにちょうど 1 file");
    let bundles: Vec<std::path::PathBuf> = files_under(&state.join("pipe").join("precheck").join("bundle"))
        .into_iter()
        .filter(|path| path.file_name().is_some_and(|name| !name.to_string_lossy().contains('.')))
        .collect();
    assert_eq!(bundles.len(), 1, "束の file はちょうど 1 つ: {bundles:?}");
    let text = bundles.first().and_then(|path| fs::read_to_string(path).ok()).unwrap_or_default();
    let pointer = text.lines().find_map(|line| line.strip_prefix("row=s2-vb.d pointer=")).unwrap_or_default();
    let (path, id) = pointer.split_once('#').unwrap_or_default();
    assert_eq!(id, "d", "{text}");
    assert!(path.starts_with(&format!("{}/bead-contracts/s2-vb.d/", state.display())) && path.ends_with(".toml"), "{pointer}");
    assert_eq!(copies("s2-vb.d"), [std::path::PathBuf::from(path)], "pointer の path は d の写しの 1 file");
    assert!(fs::read_to_string(path).unwrap_or_default().contains(BODY), "写しは本文を持つ");
    let argv = text.lines().find_map(|line| line.strip_prefix("remeasure=s2-vb.d argv=")).unwrap_or_default();
    assert!(argv.ends_with(&format!("pipe preflight --bead s2-vb.d --repo {}", repo.display())), "{argv}");
    assert!(text.lines().skip_while(|line| *line != "== section s2-vb.d").skip(1).any(|line| line == BODY), "節の部は本文: {text}");
    clean(&[&repo, &state]);
}

/// memo の同梱の引き金は開いた bead の契約の write-set で満ち、閉じた bead の契約の write-set では満ちない。
#[test]
fn vbpre_memo_bundle_trigger_meets_an_open_bead_contract() {
    let (repo, state) = bead_repo(&[table_row("a", "src/lib.rs")], &[(".beads/config.yaml", "issue-prefix: s2\n"), ("src/b.rs", "// b\n")]);
    let bd = fake_bd(
        &state,
        &[
            bead_of("s2-vb.c", "open", &contract_of("c", "src/lib.rs"), BODY, &[]),
            bead_of("s2-vb.k", "closed", &contract_of("k", "src/b.rs"), BODY, &[]),
            memo_of("s2-m.1", &["引き金: 同梱 src/lib.rs"]),
            memo_of("s2-m.2", &["引き金: 同梱 src/b.rs"]),
        ],
    );
    let out = list(&repo, &state, &bd);
    let trigger = |id: &str| {
        let line = stdout_of(&out).lines().find(|line| line.starts_with("[DISPATCH-MEMO]") && line.contains(&format!(" memo={id} "))).map(str::to_owned);
        line.and_then(|found| found.split("trigger=").nth(1).and_then(|rest| rest.split(' ').next()).map(str::to_owned)).unwrap_or_default()
    };
    assert_eq!((trigger("s2-m.1"), trigger("s2-m.2")), ("met:同梱".to_owned(), "unmet".to_owned()), "{}", told(&out));
    clean(&[&repo, &state]);
}
