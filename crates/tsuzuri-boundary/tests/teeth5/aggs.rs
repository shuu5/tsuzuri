//! 起こしの門の群の口の歯（接頭辞 aggs_・設計ノート surface-wave29b 行 ag-gspawn・判断の記録 ADR-61 決定 (2)(4)(5)(7)）。
//! tz の binary の tz hook agent-spawn に、席の Agent と Workflow の呼びの形（会話の id と path は伏せた）を標準入力で渡し、
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）の drafts/ を --drafts で渡す。台帳は PATH の先の偽の bd で渡す（本物の台帳は読まない）。
//! 計画と一覧の見本は群の試し g1 の形（反証役 3 体・割りの主張 6・5・3・一覧 14 行）を写した。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const RULING: &str = "t3-hub.90.1:20261005T0900Z-1";

/// 歯ごとの置き場（前の撃ちの残りを消して作る）と、その下の drafts/ と、群 g1 の dir に g1 の形の計画（`ruling` の裁定 id）、
/// 計画の attacker qv180 の出力の dir に一覧。
fn place(name: &str, paths2: &str, ruling: &str) -> (PathBuf, PathBuf) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("aggs")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    let drafts = root.join("drafts");
    fs::create_dir_all(drafts.join("g1")).unwrap();
    fs::create_dir_all(drafts.join("qv180/w")).unwrap();
    let ids = |a: u32, b: u32| {
        (a..=b)
            .map(|n| format!("\"U{n}\""))
            .collect::<Vec<_>>()
            .join(",")
    };
    let member = |n: &str, c: String, p: &str| {
        format!(r#"{{"name":"{n}","model":"sonnet","claims":[{c}],"paths":[{p}]}}"#)
    };
    let members = [
        member("qv181", ids(1, 6), r#""/S/a.md""#),
        member("qv182", ids(7, 11), paths2),
        member("qv183", ids(12, 14), r#""/S/c.md""#),
    ];
    let plan = format!(
        r#"{{"type":"tsuzuri:verifier","target":"t3-hub.89","draft":"緩める問い","attacker":"qv180","members":[{}]{ruling}}}"#,
        members.join(",")
    );
    fs::write(drafts.join("g1/plan.json"), plan).unwrap();
    let list: String = (1..=14)
        .map(|i| format!("U{i}\t主張 {i}\t訳 {i}\t/S/a.md\n"))
        .collect();
    fs::write(drafts.join("qv180/w/unknowns.tsv"), list).unwrap();
    (root, drafts)
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// 置き場に係 `name` の札（対象 t3-hub.89・`ago` 秒前に起こし・`ended` が真なら終えの印）と、`group` が在れば群の席の札を書く。
fn member(drafts: &Path, name: &str, ago: u64, ended: bool, group: Option<&str>) {
    let dir = drafts.join(name);
    fs::create_dir_all(&dir).unwrap();
    let end = if ended {
        format!("{}", now() - 60)
    } else {
        "null".into()
    };
    let spec = format!(
        r#"{{"name":"{name}","type":"tsuzuri:verifier","budget":150000,"build":"なし","target":"t3-hub.89","outputs":["notes.md"],"spawned":{},"agent_id":"a{name}","ended":{end}}}"#,
        now() - ago
    );
    fs::write(dir.join("spec.json"), spec).unwrap();
    if let Some(g) = group {
        let seat = format!(r#"{{"group":"{g}","i":2,"k":3,"claims":[],"paths":[]}}"#);
        fs::write(dir.join("group.json"), seat).unwrap();
    }
}

/// tz hook agent-spawn --repo <root> --drafts <drafts> に `input` を渡した結果（`bin` が在れば PATH をその dir だけにする）。
fn hook(root: &Path, input: &str, bin: Option<&Path>) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tz"));
    cmd.args(["hook", "agent-spawn", "--repo"])
        .arg(root)
        .arg("--drafts")
        .arg(root.join("drafts"));
    if let Some(bin) = bin {
        cmd.env("PATH", bin);
    }
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    child.wait_with_output().unwrap()
}

/// 席の呼びの入力（`head` は頭の欄の足し）。
fn input(head: &str, tool: &str, tool_input: &str) -> String {
    format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"/T/s.jsonl","cwd":"/W","permission_mode":"bypassPermissions",{head}"hook_event_name":"PreToolUse","tool_name":"{tool}","tool_input":{tool_input},"tool_use_id":"toolu_X"}}"#
    )
}

/// 席の検証役の起こし（名 `name`・群の行 `line`〔空なら行なし〕・モデル sonnet）。
fn spawn(name: &str, line: &str) -> String {
    let group = if line.is_empty() {
        String::new()
    } else {
        format!(r"群: {line}\n")
    };
    let prompt = format!(
        r"予算: token 150000\n組み: なし\n対象: t3-hub.89\n出す物: notes.md\n{group}\n主張を反証する。"
    );
    let tool = format!(
        r#"{{"description":"反証","prompt":"{prompt}","subagent_type":"tsuzuri:verifier","name":"{name}","model":"sonnet"}}"#
    );
    input("", "Agent", &tool)
}

/// 標準出力が deny の答えで、理由が `want` を含む。
fn denied(out: &Output, want: &str) {
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{text}");
    assert!(
        text.contains(r#""permissionDecision":"deny""#) && text.contains(want),
        "{text}"
    );
}

/// 何も出さずに通る。
fn passed(out: &Output) {
    assert_eq!(
        (out.status.code(), out.stdout.as_slice()),
        (Some(0), &b""[..]),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn aggs_seat_workflow_calls_are_refused_and_name_the_owner() {
    let (root, _) = place("workflow", r#""/S/b.rs""#, "");
    for tool_input in [
        r#"{"script":"x"}"#,
        r#"{"script":"x","plan":"g1/plan.json","ruling":"t3-hub.90.1:20261005T0900Z-1"}"#,
    ] {
        let out = hook(&root, &input("", "Workflow", tool_input), None);
        denied(
            &out,
            "席の流れの道具 Workflow の呼びは計画の file と裁定 id を持っても全部断る",
        );
        denied(&out, "次の一手 = 流れの道具で係を起こすには持ち主に問う");
    }
    passed(&hook(
        &root,
        &input(r#""agent_id":"a77","#, "Workflow", r#"{"script":"x"}"#),
        None,
    ));
}

#[test]
fn aggs_a_group_member_passes_beside_its_mates_and_writes_its_seat() {
    let (root, drafts) = place("pass", r#""/S/b.rs""#, "");
    member(&drafts, "qv182", 60, false, Some("g1"));
    passed(&hook(&root, &spawn("qv181", "g1 1/3"), None));
    let seat = fs::read_to_string(drafts.join("qv181/group.json")).unwrap();
    assert!(
        seat.contains(r#""group": "g1""#) && seat.contains(r#""U6""#) && !seat.contains(r#""U7""#),
        "{seat}"
    );
    assert!(drafts.join("qv181/spec.json").is_file());
    assert!(!drafts.join("g1/shared.jsonl").exists());
    let (root, drafts) = place("outside", r#""/S/b.rs""#, "");
    member(&drafts, "qv170", 60, false, None);
    denied(
        &hook(&root, &spawn("qv181", "g1 1/3"), None),
        "生きた係 qv170 が同じ対象 t3-hub.89 を持つ",
    );
    assert!(!drafts.join("qv181").exists());
}

#[test]
fn aggs_shared_read_paths_pass_and_are_recorded() {
    let (root, drafts) = place("shared", r#""/S/b.rs","/S/a.md""#, "");
    passed(&hook(&root, &spawn("qv181", "g1 1/3"), None));
    let log = fs::read_to_string(drafts.join("g1/shared.jsonl")).unwrap();
    assert_eq!(log.lines().count(), 1, "{log}");
    assert!(
        log.contains(r#""name":"qv181","paths":["/S/a.md"]"#),
        "{log}"
    );
}

#[test]
fn aggs_the_gate_reads_the_line_the_model_and_the_group_totals_from_files() {
    let (root, drafts) = place("files", r#""/S/b.rs""#, "");
    denied(
        &hook(&root, &spawn("qv181", ""), None),
        "群 g1 の計画が割りに持つ係 qv181 の頭に 群: の行が無い",
    );
    passed(&hook(&root, &spawn("qv189", ""), None));
    let _ = fs::remove_dir_all(drafts.join("qv189"));
    let opus = spawn("qv181", "g1 1/3").replace(r#""model":"sonnet""#, r#""model":"opus""#);
    denied(
        &hook(&root, &opus, None),
        "呼びのモデル opus が計画の値 sonnet と違う",
    );
    member(&drafts, "qv182", 60, true, Some("g1"));
    member(&drafts, "qv183", 60, true, Some("g1"));
    fs::write(drafts.join("qv182/meter.json"), r#"{"offset":0,"used":200000,"cache_read":0,"last_id":null,"last_used":0,"last_cache_read":0,"marks":[]}"#).unwrap();
    fs::write(drafts.join("qv183/meter.json"), r#"{"offset":0,"used":204999,"cache_read":0,"last_id":null,"last_used":0,"last_cache_read":0,"marks":[]}"#).unwrap();
    passed(&hook(&root, &spawn("qv181", "g1 1/3"), None));
    fs::write(drafts.join("qv183/meter.json"), r#"{"offset":0,"used":205000,"cache_read":0,"last_id":null,"last_used":0,"last_cache_read":0,"marks":[]}"#).unwrap();
    denied(
        &hook(&root, &spawn("qv181", "g1 1/3"), None),
        "群 g1 の合計（新しい量 405000・読み直し 0）が止める線に届いた",
    );
    fs::write(drafts.join("qv183/meter.json"), "{").unwrap();
    denied(
        &hook(&root, &spawn("qv181", "g1 1/3"), None),
        "群 g1 の合計が読めない",
    );
}

/// 偽の bd（引数が --readonly show t3-hub.90.1 --json の時だけ `json` を出して 0・ほかは rc 3）を置いた dir。
fn bd(root: &Path, json: &str) -> PathBuf {
    let bin = root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let script = format!(
        "#!/bin/sh\n[ \"$*\" = \"--readonly show t3-hub.90.1 --json\" ] || exit 3\nprintf '%s\\n' '{json}'\n"
    );
    let path = bin.join("bd");
    fs::write(&path, script).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    bin
}

#[test]
fn aggs_the_gate_reads_the_list_from_the_attacker_out_dir() {
    let (root, _) = place("attacker", r#""/S/b.rs""#, "");
    passed(&hook(&root, &spawn("qv181", "g1 1/3"), None));
    let missing = "群 g1 の unknowns.tsv が無い";
    let (root, drafts) = place("group-dir", r#""/S/b.rs""#, "");
    fs::rename(
        drafts.join("qv180/w/unknowns.tsv"),
        drafts.join("g1/unknowns.tsv"),
    )
    .unwrap();
    denied(&hook(&root, &spawn("qv181", "g1 1/3"), None), missing);
    assert!(!drafts.join("qv181").exists());
    let (root, drafts) = place("escape", r#""/S/b.rs""#, "");
    let plan = fs::read_to_string(drafts.join("g1/plan.json")).unwrap();
    let escaped = plan.replace(r#""attacker":"qv180""#, r#""attacker":"../qv180""#);
    assert_ne!(escaped, plan, "attacker の字");
    fs::write(drafts.join("g1/plan.json"), escaped).unwrap();
    fs::create_dir_all(root.join("qv180/w")).unwrap();
    fs::copy(
        drafts.join("qv180/w/unknowns.tsv"),
        root.join("qv180/w/unknowns.tsv"),
    )
    .unwrap();
    denied(&hook(&root, &spawn("qv181", "g1 1/3"), None), missing);
    assert!(!drafts.join("qv181").exists());
}

#[test]
fn aggs_the_seven_day_window_reads_the_ruling_from_the_ledger() {
    let ruling = format!(r#","ruling":"{RULING}""#);
    let (root, drafts) = place("window", r#""/S/b.rs""#, &ruling);
    member(&drafts, "qv170", 86_400, true, Some("g0"));
    let notes = format!("前の行\\n裁定 id = {RULING}・問い = t3-hub.90.1・逐語 = はい");
    let good = bd(
        &root,
        &format!(
            r#"[{{"id":"t3-hub.90.1","description":"検証の群（g1）を起こす","notes":"{notes}"}}]"#
        ),
    );
    passed(&hook(&root, &spawn("qv181", "g1 1/3"), Some(&good)));
    let _ = fs::remove_dir_all(drafts.join("qv181"));
    let other = bd(
        &root,
        &format!(r#"[{{"id":"t3-hub.90.1","description":"検証の群を起こす","notes":"{notes}"}}]"#),
    );
    denied(
        &hook(&root, &spawn("qv181", "g1 1/3"), Some(&other)),
        "問い t3-hub.90.1 の本文が群 g1 を語として名指さない",
    );
    let bad = bd(&root, "not json");
    denied(
        &hook(&root, &spawn("qv181", "g1 1/3"), Some(&bad)),
        "台帳の問い t3-hub.90.1 が読めない",
    );
    let empty = root.join("empty");
    fs::create_dir_all(&empty).unwrap();
    denied(
        &hook(&root, &spawn("qv181", "g1 1/3"), Some(&empty)),
        "台帳の問い t3-hub.90.1 が読めない",
    );
}
