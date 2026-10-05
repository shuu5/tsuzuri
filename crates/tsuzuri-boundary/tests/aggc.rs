//! 群の係の終える前の門の主張の表の歯（接頭辞 aggc_・設計ノート surface-wave29b 行 ag-gstop・判断の記録 ADR-61 決定 (4)(8)）。
//! 中核の `claim_lacks` を直に撃ち、tz の binary の tz hook agent-stop に係の終わりの入力の形（会話の id と path は伏せた）を標準入力で渡し、
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）の drafts/ を --drafts で渡す。群の席の札は行 ag-gspawn が書く形（群 g1・割りの主張 U1）を歯が置く。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_core::agent::spec::Spec;
use tsuzuri_core::agent::stop::claim_lacks;

/// 主張 U1〜U4 の表（印 V・D・I・U・U2 の主張は `\|` を持つ）の見本。
const TABLE: &str = "# 判じ\n\n| id | 主張 | 印 | 証拠 |\n|---|---|---|---|\n\
                     | U1 | 数は合う | V | /S/a.md |\n| U2 | 式 a \\| b は真 | D | rg x /S/a.md |\n\
                     | U3 | 見込みは 2 倍 | I | /S/b.md |\n| U4 | 割りの外 | U | /S/c.md が要る |\n";

/// `TABLE` の U2 の行を `row` に替えた字。
fn swap(row: &str) -> Vec<String> {
    vec![TABLE.replace("| U2 | 式 a \\| b は真 | D | rg x /S/a.md |", row)]
}

/// 割りの主張 U1〜U4。
fn claims() -> Vec<String> {
    ["U1", "U2", "U3", "U4"].map(String::from).to_vec()
}

/// 主張の表の 4 つの印と `\|` の欄は欠けが無く、U2 の行から 1 句だけ外すと、その句の欠けだけを返し、割りに無い主張の行は見ない。
#[test]
fn aggc_claim_lacks_names_each_missing_row_mark_and_evidence() {
    assert!(claim_lacks(&[TABLE.to_string()], &claims()).is_empty());
    let mark = "主張 U2 の行に確かさの印（V・D・I・U）が無い";
    let proof = "主張 U2 の行に証拠の path か命令が無い";
    for (row, want) in [
        ("| U2 | 式 a \\| b は真 | X | rg x /S/a.md |", mark),
        ("| U2 | 式 a \\| b は真 | d | rg x /S/a.md |", mark),
        ("| U2 | 式 a \\| b は真 |  | rg x /S/a.md |", mark),
        ("| U2 | 式 a | b は真 | D | rg x /S/a.md |", mark),
        ("| U2 | 式 a \\| b は真 | D |  |", proof),
        ("| U2 | 式 a \\| b は真 | D |", proof),
        (
            "| U9 | 式 a \\| b は真 | D | rg x /S/a.md |",
            "割りの主張 U2 の表の行が出す物に無い",
        ),
        (
            "U2 | 式 a \\| b は真 | D | rg x /S/a.md |",
            "割りの主張 U2 の表の行が出す物に無い",
        ),
    ] {
        assert_eq!(
            claim_lacks(&swap(row), &claims()),
            vec![want.to_string()],
            "{row}"
        );
    }
    let two = vec![TABLE.to_string(), "| U2 | もう 1 行 | V |\n".to_string()];
    assert_eq!(claim_lacks(&two, &claims()), vec![proof.to_string()]);
    let bare = vec!["| U9 | 割りに無い |  |  |\n".to_string()];
    assert!(claim_lacks(&bare, &[]).is_empty());
}

/// 欠けの無い群の係の終わりの置き場: 係 qv181 の札（出す物 notes.md）と係の id a81 の結び・席への知らせの在る係の記録・
/// w/notes.md の字 `notes`・`member` なら群 g1 の席の札（割りの主張 U1）。
fn site(name: &str, notes: &str, member: bool) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("aggc")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    let dir = root.join("drafts/qv181");
    fs::create_dir_all(dir.join("w")).unwrap();
    fs::create_dir_all(root.join("s/subagents")).unwrap();
    fs::write(
        dir.join("spec.json"),
        r#"{"name":"qv181","type":"tsuzuri:verifier","budget":150000,"build":"なし","target":"t3-hub.89","outputs":["notes.md"],"spawned":1,"agent_id":"a81","ended":null}"#,
    )
    .unwrap();
    if member {
        let seat = r#"{"group":"g1","i":1,"k":3,"claims":["U1"],"paths":["/S/a.md"]}"#;
        fs::write(dir.join("group.json"), seat).unwrap();
    }
    fs::write(dir.join("w/notes.md"), notes).unwrap();
    fs::create_dir_all(root.join("drafts/.agents")).unwrap();
    fs::write(root.join("drafts/.agents/a81"), "qv181\n").unwrap();
    let told = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"SendMessage","input":{"to":"team-lead","message":"済み"}}]}}"#;
    fs::write(
        root.join("s/subagents/agent-a81.jsonl"),
        format!("{told}\n"),
    )
    .unwrap();
    root
}

/// tz hook agent-stop に係 a81 の終わりの入力（止めた後の終わりか `again`・最後の文は出力の dir の path を持つ）を渡した結果。
fn end_of(root: &Path, again: bool) -> Output {
    let w = root.join("drafts/qv181/w");
    let input = format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"{}","cwd":"/W","permission_mode":"bypassPermissions","agent_id":"a81","agent_type":"tsuzuri:verifier","hook_event_name":"SubagentStop","stop_hook_active":{again},"agent_transcript_path":"{}","last_assistant_message":"済み。出す物は {} に在る"}}"#,
        root.join("s.jsonl").display(),
        root.join("s/subagents/agent-a81.jsonl").display(),
        w.display()
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["hook", "agent-stop", "--repo"])
        .arg(root)
        .arg("--drafts")
        .arg(root.join("drafts"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    child.wait_with_output().unwrap()
}

/// 係 qv181 の札の終えの印。
fn ended(root: &Path) -> Option<u64> {
    let text = fs::read_to_string(root.join("drafts/qv181/spec.json")).unwrap();
    Spec::parse(&text).unwrap().ended
}

/// 群の係の主張の表の行の印か証拠が欠けた 1 度目の終わりは rc 2 と標準エラーの理由で止め、欠けの無い表の終わりは通して終えの印を書く。
#[test]
fn aggc_hook_holds_the_first_stop_of_a_member_lacking_a_mark_or_evidence() {
    for (name, notes, hole) in [
        (
            "mark",
            "| U1 | 数は合う |  | /S/a.md |\n",
            "主張 U1 の行に確かさの印（V・D・I・U）が無い",
        ),
        (
            "proof",
            "| U1 | 数は合う | V |  |\n",
            "主張 U1 の行に証拠の path か命令が無い",
        ),
    ] {
        let root = site(name, notes, true);
        let out = end_of(&root, false);
        assert_eq!(
            (out.status.code(), out.stdout.len()),
            (Some(2), 0),
            "{name}"
        );
        let why = String::from_utf8_lossy(&out.stderr);
        assert!(
            why.starts_with(&format!("係の終える前の門は止める（{hole}） 次の一手 = ")),
            "{why}"
        );
        assert_eq!(ended(&root), None);
    }
    let root = site("whole", "| U1 | 数は合う | V | /S/a.md |\n", true);
    let out = end_of(&root, false);
    assert_eq!((out.status.code(), out.stderr.len()), (Some(0), 0));
    assert!(ended(&root).is_some());
}

/// 群の席の札の無い係は表が無くても通し、群の係の 2 度目の終わりは表の欠けを STOP-GATE.txt に書いて通す。
#[test]
fn aggc_hook_passes_a_non_member_and_records_the_holes_at_the_second_stop() {
    let loner = site("loner", "分かった所\n", false);
    let out = end_of(&loner, false);
    assert_eq!((out.status.code(), out.stderr.len()), (Some(0), 0));
    assert!(ended(&loner).is_some());
    let member = site("again", "分かった所\n", true);
    let out = end_of(&member, true);
    assert_eq!((out.status.code(), out.stderr.len()), (Some(0), 0));
    let gate = fs::read_to_string(member.join("drafts/qv181/w/STOP-GATE.txt")).unwrap();
    assert_eq!(gate, "割りの主張 U1 の表の行が出す物に無い\n");
    assert!(ended(&member).is_some());
}
