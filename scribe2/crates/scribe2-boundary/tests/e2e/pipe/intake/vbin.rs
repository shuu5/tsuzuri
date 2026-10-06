//! 器の口 pipe intake と pipe preflight の bead の周の歯（接頭辞 `vbin_`・親 `tests/e2e/pipe/intake.rs` の helper を `use super::*;` で使う）。
//!
//! `--design` も `--contract` も渡さず `--bead` だけを渡す周は、台帳の bead（欄 acceptance に契約表の導出の形の `[[contract]]` の 1 行・本文に
//! 設計の節）から契約を組む。toy の導出の行 b の字を bead に置き、偽の `bd`（置き場の `ledger.json` を `cat` する script）で台帳を読ませて、
//! 同じ行を表に置いた撃ちと同じ判定になること・断りの字・上限・写しの置き場を、rc と stdout と stderr と置き場の file から測る。
//! 退けた行の id を凍結した file docs/design/contract-ids.txt を持つ repo は、表に無い凍結の id の bead も断る（表を先に見る）。

use super::*;
use std::os::unix::fs::PermissionsExt;
use vessel::rules::{RuleKind, ValueShape, ALL};

/// 行の verify（導出の歯 `derive_` の 1 本の nextest 行）。
const VERIFY: &str = "[\"cargo nextest run -p toy --no-tests=fail derive_\"]";

/// bead の本文の見本（9 byte）。
const BODY: &str = "本文。";

/// 裁定 id（3 行とも同じ）。
const RULING: &str = "user 2026-10-06T11:44Z 裁定 t3-hub.92.7.2";

/// bead の acceptance（導出の行 `id` から欄 section の行を除いた字）。
fn acceptance_of(id: &str) -> String {
    derive_row(id, &[("verify", VERIFY)]).lines().filter(|line| !line.starts_with("section")).map(|line| format!("{line}\n")).collect()
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

/// 台帳の bead 1 本（欄 id・status・labels・acceptance_criteria・description・dependencies）。
fn listed_bead(id: &str, status: &str, acceptance: &str, description: &str) -> String {
    format!(
        "{{\"id\":\"{id}\",\"status\":\"{status}\",\"labels\":[],\"acceptance_criteria\":\"{}\",\"description\":\"{}\",\"dependencies\":[]}}",
        escaped(acceptance),
        escaped(description)
    )
}

/// 置き場の `ledger.json` を cat する偽の `bd`（引数は読み飛ばす）を置き、その path を返す。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn fake_bd(state: &Path, beads: &[String]) -> String {
    let ledger = state.join("ledger.json");
    fs::write(&ledger, format!("[{}]\n", beads.join(","))).expect("偽の台帳を書ける");
    script_bd(state, &format!("cat '{}'\n", ledger.display()))
}

/// 本文 `body` の `/bin/sh` script を偽の `bd` として置く。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn script_bd(state: &Path, body: &str) -> String {
    let path = state.join("bd");
    fs::write(&path, format!("#!/bin/sh\n{body}")).expect("script を書ける");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("script に実行権を付ける");
    path.display().to_string()
}

/// 3 本の rules 行の既定の値（埋め込みの値・開いた契約の本数と本文と acceptance の順）。
fn default_caps() -> [u64; 3] {
    ["contract.open_max", "contract.body_max_bytes", "contract.acceptance_max_bytes"].map(embedded_int)
}

/// 受付の上限の写しに、台帳の待ち上限の行と 3 本の rules 行（値は `caps`）を足した manifest の path。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn bead_rules(state: &Path, caps: [u64; 3]) -> String {
    let base = fs::read_to_string(ceiling_rules(state)).expect("受付の写しを読める");
    let row = |id: &str, kind: &str, value: u64| {
        format!("[[rule]]\nid = \"{id}\"\nkind = \"{kind}\"\nvalue = {value}\nenabled = true\nruling = \"t\"\nruled_at = \"d\"\n")
    };
    let rows = [
        row("seat.ledger_timeout_s", "LedgerTimeoutS", 60),
        row("contract.open_max", "ContractOpenMax", caps[0]),
        row("contract.body_max_bytes", "ContractBodyMaxBytes", caps[1]),
        row("contract.acceptance_max_bytes", "ContractAcceptanceMaxBytes", caps[2]),
    ];
    let path = state.join(format!("rules-bead-{}-{}-{}.toml", caps[0], caps[1], caps[2]));
    fs::write(&path, format!("{base}\n{}", rows.join("\n"))).expect("写しを書ける");
    path.display().to_string()
}

/// `pipe <command> --bead s2-b --repo R --rules … --state-dir S --bd …` を 1 回撃つ（`--design` は渡さない・intake は審査の偽 lens を足す）。
fn bead_run(command: &str, repo: &Path, state: &Path, (bd, rules): (&str, &str)) -> Output {
    let (repo, state_dir, lens) = (repo.display().to_string(), state.display().to_string(), review_lens_pass(state));
    let mut args = vec![command, "--bead", "s2-b", "--repo", &repo, "--rules", rules, "--state-dir", &state_dir, "--bd", bd];
    if command == "intake" {
        args.extend(["--lens", &lens]);
    }
    run_pipe(&args)
}

/// 表に行 `rows` を持つ toy の repo と置き場。
fn toy(rows: &[String]) -> (PathBuf, PathBuf) {
    derive_repo(&table_doc(&table_region(rows)))
}

/// 写しの path（置き場の子 `bead-contracts` の子 `bead` の下の `<digest>.toml`）。
fn copy_of(state: &Path, bead: &str, acceptance: &str, description: &str) -> PathBuf {
    let key = vessel::pipe::bead::digest(acceptance, description);
    state.join("bead-contracts").join(bead).join(format!("{key}.toml"))
}

/// 断りの撃ちの外形: rc 1・stderr の 1 行目が `pipe: <want>`・置き場に run dir を作らない。
fn assert_refused_first_line(out: &Output, state: &Path, want: &str) {
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "{want}: {}", stderr_of(out));
    assert_eq!(stderr_of(out).lines().next(), Some(format!("pipe: {want}").as_str()), "stderr の 1 行目");
    assert_eq!(run_dirs(state), Vec::<String>::new(), "{want}: run dir を作らない");
}

/// [`assert_refused_first_line`] に加え、置き場に子 `bead-contracts` も作らない（写しを書く前に断る周）。
fn assert_refused_untouched(out: &Output, state: &Path, want: &str) {
    assert_refused_first_line(out, state, want);
    assert!(!state.join("bead-contracts").exists(), "{want}: 写しの dir を作らない");
}

/// design= の行を除いた stdout の行の列。
fn rest_of(out: &Output) -> Vec<String> {
    stdout_of(out).lines().filter(|line| !line.starts_with("design=")).map(str::to_owned).collect()
}

/// 行 b の bead に撃つ preflight は、同じ行 b を表に置いて `--design` で撃った周と design= の行を除いて同じ行の列で、design= の行は写しの絶対 path と
/// 行 id と section=s2-b・写しは置き場の子の `<digest>.toml` で節の形の字・末尾は ok・run dir を作らない。
#[test]
fn vbin_preflight_reads_the_bead_like_the_table_row() {
    let (repo, state) = toy(&[derive_row("a", &[("verify", VERIFY)])]);
    let acceptance = acceptance_of("b");
    let bd = fake_bd(&state, &[listed_bead("s2-b", "open", &acceptance, BODY)]);
    let out = bead_run("preflight", &repo, &state, (&bd, &bead_rules(&state, default_caps())));
    let text = stdout_of(&out);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{text} {}", stderr_of(&out));
    let copy = copy_of(&state, "s2-b", &acceptance, BODY);
    assert_eq!(fact_lines(&out, "design="), [format!("design={}#b section=s2-b", copy.display())], "{text}");
    let name = copy.file_name().map(|found| found.to_string_lossy().into_owned()).unwrap_or_default();
    assert!(name.len() == 16 + ".toml".len() && name.ends_with(".toml"), "digest の 16 桁の名: {name}");
    assert_eq!(dir_names(&state.join("bead-contracts").join("s2-b")), [name], "写しは 1 本");
    let want = format!("schema = 1\n\n{acceptance}section = \"s2-b\"\ngoal = \"本文。\"\n");
    assert_eq!(fs::read_to_string(&copy).unwrap_or_default(), want, "写しの字は節の形");
    assert_eq!(tail_line(&out), "preflight: ok", "{text}");
    assert_eq!(run_dirs(&state), Vec::<String>::new(), "run dir を作らない");
    let (table_repo, table_state) = toy(&[derive_row("b", &[("verify", VERIFY)])]);
    let by_design = preflight_raw(&table_repo, &table_state, "docs/design/toy.md#b", "s2-b", true);
    assert_eq!(by_design.status.code(), Some(i32::from(RC_OK)), "{} {}", stdout_of(&by_design), stderr_of(&by_design));
    assert_eq!(rest_of(&out), rest_of(&by_design), "design= の行を除いて同じ判定: {text}");
    clean(&[&repo, &state, &table_repo, &table_state]);
}

/// bead の周の intake は rc 0 で、便の契約 file の design は写しの絶対 path と井桁と行 id。その後に本文を替えた preflight は別の名の写しを作り、前の写しの字と
/// 便の契約 file の字は替わらない。
#[test]
fn vbin_intake_points_the_run_at_the_copy() {
    let (repo, state) = toy(&[derive_row("a", &[("verify", VERIFY)])]);
    let acceptance = acceptance_of("b");
    let bd = fake_bd(&state, &[listed_bead("s2-b", "open", &acceptance, BODY)]);
    let rules = bead_rules(&state, default_caps());
    let out = bead_run("intake", &repo, &state, (&bd, &rules));
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{} {}", stdout_of(&out), stderr_of(&out));
    let contract_file = state.join("pipe").join(run_id_of(&out)).join("contract.toml");
    let copy = copy_of(&state, "s2-b", &acceptance, BODY);
    let loaded = vessel::pipe::contract::Contract::load(&contract_file).map(|found| found.design).unwrap_or_default();
    assert_eq!(loaded, format!("{}#b", copy.display()), "便の契約 file の design は写しの path と行 id");
    let (copy_before, contract_before) = (fs::read_to_string(&copy).ok(), fs::read_to_string(&contract_file).ok());
    assert!(copy_before.is_some() && contract_before.is_some(), "写しと便の契約 file を読める");
    let changed = fake_bd(&state, &[listed_bead("s2-b", "open", &acceptance, "別の本文。")]);
    let after = bead_run("preflight", &repo, &state, (&changed, &rules));
    let second = copy_of(&state, "s2-b", &acceptance, "別の本文。");
    assert_ne!(second, copy, "本文が替わると別の名の写し: {}", stdout_of(&after));
    assert!(second.exists(), "別の名の写しができる: {}", stdout_of(&after));
    assert_eq!(dir_names(&state.join("bead-contracts").join("s2-b")).len(), 2, "写しは 2 本");
    assert_eq!(fs::read_to_string(&copy).ok(), copy_before, "前の写しの字は替わらない");
    assert_eq!(fs::read_to_string(&contract_file).ok(), contract_before, "便の契約 file の字は替わらない");
    clean(&[&repo, &state]);
}

/// 両方の形の bead は contract-bead-both-forms、台帳に無い bead・rc 1 で終わる偽の bd・acceptance が空の bead・本文に二重引用符を持つ bead は
/// contract-bead-unreadable と理由の句で、どれも rc 1 で run dir も子 `bead-contracts` も作らない。
#[test]
fn vbin_both_forms_and_unreadable_beads_are_refused() {
    let acceptance = acceptance_of("b");
    let both = format!("{acceptance}design = docs/design/toy.md#a\n");
    let cases: [(Option<String>, &str); 5] = [
        (Some(listed_bead("s2-b", "open", &both, BODY)), "contract-bead-both-forms bead=s2-b"),
        (Some(listed_bead("s2-z", "open", &acceptance, BODY)), "contract-bead-unreadable bead=s2-b 台帳に無い"),
        (None, "contract-bead-unreadable bead=s2-b 台帳を読めない"),
        (Some(listed_bead("s2-b", "open", "", BODY)), "contract-bead-unreadable bead=s2-b acceptance に [[contract]] の行も design の行も無い"),
        (
            Some(listed_bead("s2-b", "open", &acceptance, "引用符\"を持つ本文")),
            "contract-bead-unreadable bead=s2-b 本文に二重引用符か逆斜線が在る",
        ),
    ];
    for (listed, want) in cases {
        let (repo, state) = toy(&[derive_row("a", &[("verify", VERIFY)])]);
        let bd = match &listed {
            Some(found) => fake_bd(&state, std::slice::from_ref(found)),
            None => script_bd(&state, "exit 1\n"),
        };
        let out = bead_run("intake", &repo, &state, (&bd, &bead_rules(&state, default_caps())));
        assert_refused_untouched(&out, &state, want);
        clean(&[&repo, &state]);
    }
}

/// rules 行の値ちょうどは通り（preflight rc 0）、1 小さい値は断る（intake rc 1 の 1 行）: 本文と acceptance の byte・開いた契約の bead の本数
/// （閉じた契約の bead と design = の行だけの bead は数えない）。
#[test]
fn vbin_caps_refuse_one_over_the_value() {
    let acceptance = acceptance_of("b");
    let [open, body, accept] = default_caps();
    let (body_len, accept_len) = (BODY.len() as u64, acceptance.len() as u64);
    let cases = [
        ([open, body_len, accept], "contract-bytes-cap field=description bytes=9 cap=8".to_owned(), [open, body_len - 1, accept]),
        (
            [open, body, accept_len],
            format!("contract-bytes-cap field=acceptance bytes={accept_len} cap={}", accept_len - 1),
            [open, body, accept_len - 1],
        ),
    ];
    let (repo, state) = toy(&[derive_row("a", &[("verify", VERIFY)])]);
    let bd = fake_bd(&state, &[listed_bead("s2-b", "open", &acceptance, BODY)]);
    for (passes, want, refuses) in cases {
        let ok = bead_run("preflight", &repo, &state, (&bd, &bead_rules(&state, passes)));
        assert_eq!(ok.status.code(), Some(i32::from(RC_OK)), "{passes:?}: {} {}", stdout_of(&ok), stderr_of(&ok));
        let out = bead_run("intake", &repo, &state, (&bd, &bead_rules(&state, refuses)));
        assert_refused_first_line(&out, &state, &want);
    }
    let beads = [
        listed_bead("s2-b", "open", &acceptance, BODY),
        listed_bead("s2-c", "in_progress", &acceptance_of("c"), BODY),
        listed_bead("s2-d", "open", &acceptance_of("d"), BODY),
        listed_bead("s2-e", "closed", &acceptance_of("e"), BODY),
        listed_bead("s2-f", "open", "design = docs/design/toy.md#a", BODY),
    ];
    let bd = fake_bd(&state, &beads);
    let ok = bead_run("preflight", &repo, &state, (&bd, &bead_rules(&state, [3, body, accept])));
    assert_eq!(ok.status.code(), Some(i32::from(RC_OK)), "値 3 は通る: {} {}", stdout_of(&ok), stderr_of(&ok));
    let out = bead_run("intake", &repo, &state, (&bd, &bead_rules(&state, [2, body, accept])));
    assert_refused_first_line(&out, &state, "contract-open-cap open=3 cap=2");
    clean(&[&repo, &state]);
}

/// bead の行の id が表の行と同じなら by=<doc>#<id>（置き場の 2 つの doc）、閉じた bead の行と同じなら by=<bead の id>、どれも rc 1 で run dir を作らない。
/// ほかに同じ id の無い行は通る。
#[test]
fn vbin_contract_id_taken_names_the_holder() {
    let old = table_doc(&table_region(&[derive_row("c", &[("verify", VERIFY)])]));
    let (repo, state) = derive_repo_with(&table_doc(&table_region(&[derive_row("a", &[("verify", VERIFY)])])), &[("docs/design/old.md", &old)]);
    let closed = listed_bead("s2-c", "closed", &acceptance_of("d"), BODY);
    for (id, by) in [("a", "docs/design/toy.md#a"), ("c", "docs/design/old.md#c"), ("d", "s2-c")] {
        let bd = fake_bd(&state, &[listed_bead("s2-b", "open", &acceptance_of(id), BODY), closed.clone()]);
        let out = bead_run("intake", &repo, &state, (&bd, &bead_rules(&state, default_caps())));
        assert_refused_untouched(&out, &state, &format!("contract-id-taken id={id} by={by}"));
    }
    let bd = fake_bd(&state, &[listed_bead("s2-b", "open", &acceptance_of("e"), BODY), closed]);
    let ok = bead_run("preflight", &repo, &state, (&bd, &bead_rules(&state, default_caps())));
    assert_eq!(ok.status.code(), Some(i32::from(RC_OK)), "ほかに同じ id の無い行は通る: {} {}", stdout_of(&ok), stderr_of(&ok));
    clean(&[&repo, &state]);
}

/// 凍結の file docs/design/contract-ids.txt（行 a と y）を持つ repo で、表に無く凍結の file に在る id y の bead は by=docs/design/contract-ids.txt で断り、
/// 表と凍結の file の両方に在る id a は表を先に見て by=<doc>#<id> で断り、どれも rc 1 で run dir を作らない。どこにも無い id e の preflight は通る。
#[test]
fn vbin_frozen_ids_refuse_the_retired_rows() {
    let doc = table_doc(&table_region(&[derive_row("a", &[("verify", VERIFY)])]));
    let (repo, state) = derive_repo_with(&doc, &[("docs/design/contract-ids.txt", "a\ny\n")]);
    for (id, by) in [("y", "docs/design/contract-ids.txt"), ("a", "docs/design/toy.md#a")] {
        let bd = fake_bd(&state, &[listed_bead("s2-b", "open", &acceptance_of(id), BODY)]);
        let out = bead_run("intake", &repo, &state, (&bd, &bead_rules(&state, default_caps())));
        assert_refused_untouched(&out, &state, &format!("contract-id-taken id={id} by={by}"));
    }
    let bd = fake_bd(&state, &[listed_bead("s2-b", "open", &acceptance_of("e"), BODY)]);
    let ok = bead_run("preflight", &repo, &state, (&bd, &bead_rules(&state, default_caps())));
    assert_eq!(ok.status.code(), Some(i32::from(RC_OK)), "どこにも無い id は通る: {} {}", stdout_of(&ok), stderr_of(&ok));
    clean(&[&repo, &state]);
}

/// 埋め込みの manifest は 3 行を値 40・65536・24576 で持ち、形 Int・発効・裁定と裁定日は裁定 t3-hub.92.7.2 の値で、kind は ALL の RunnerGateFixRounds の直後に
/// ContractOpenMax・ContractBodyMaxBytes・ContractAcceptanceMaxBytes の順で並び、許可の読み手は持たない。
#[test]
fn vbin_rules_rows_hold_the_caps_after_gate_fix_rounds() {
    let manifest = Manifest::embedded().expect("埋め込み manifest を読める");
    let rows = [
        ("contract.open_max", RuleKind::ContractOpenMax, 40),
        ("contract.body_max_bytes", RuleKind::ContractBodyMaxBytes, 65536),
        ("contract.acceptance_max_bytes", RuleKind::ContractAcceptanceMaxBytes, 24576),
    ];
    for (id, kind, value) in rows {
        let row = manifest.get(id).expect("行が在る");
        assert_eq!((row.value.clone(), row.kind, row.enabled), (RuleValue::Int(value), kind, true), "{id}");
        assert_eq!(kind.shape(), ValueShape::Int, "{id}: 形は Int");
        assert_eq!((row.ruling.as_str(), row.ruled_at.as_str()), (RULING, "2026-10-06"), "{id}: 裁定と裁定日");
        assert_eq!(manifest.rows().iter().filter(|found| found.kind == kind).count(), 1, "{id}: kind の行は 1 本");
        assert!(!kind.has_permit_reader(), "{id}: 許可の読み手は持たない");
    }
    let at = ALL.iter().position(|kind| *kind == RuleKind::RunnerGateFixRounds).expect("RunnerGateFixRounds は ALL に在る");
    let after: Vec<RuleKind> = ALL.iter().skip(at + 1).take(3).copied().collect();
    assert_eq!(after, rows.map(|(_, kind, _)| kind), "kind は RunnerGateFixRounds の直後に順に並ぶ");
}
