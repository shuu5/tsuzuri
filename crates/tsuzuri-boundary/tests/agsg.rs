//! 係の終える前の門の歯（接頭辞 agsg_・設計ノート surface-wave29b 行 ag-stop・判断の記録 ADR-59 決定 (4)）。
//! 中核の `lacks` を直に撃ち、tz の binary の tz hook agent-stop に係の終わりの門の入力の形（会話の id と path は伏せた）を
//! 標準入力で渡し、歯ごとの置き場（CARGO_TARGET_TMPDIR の下）を --drafts で渡す。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_core::agent::spec::Spec;
use tsuzuri_core::agent::stop::lacks;

/// 係の記録の assistant の行（道具 `tool` の tool_use で、input の宛先 `to`）。
fn call(tool: &str, to: &str) -> String {
    format!(
        r#"{{"type":"assistant","message":{{"id":"A","content":[{{"type":"text","text":"済み"}},{{"type":"tool_use","id":"toolu_1","name":"{tool}","input":{{"to":"{to}","message":"済み"}}}}]}}}}"#
    ) + "\n"
}

/// 席への知らせの在る係の記録（頼みの行の後に team-lead への SendMessage）。
fn told() -> String {
    r#"{"type":"user","message":{"content":"頼み"}}"#.to_string()
        + "\n"
        + &call("SendMessage", "team-lead")
}

/// 席への知らせの欠けの字。
const UNTOLD: &str = "係の記録に席（team-lead）への SendMessage の知らせが無い";

/// 中核の欠けの数え: 席への知らせは content の tool_use の SendMessage で宛先が team-lead の呼びだけ。
#[test]
fn agsg_lacks_finds_only_a_send_message_to_the_lead() {
    let (out, last) = ("/D/w218a/w", "出す物は /D/w218a/w に在る");
    let outs = vec!["notes.md".to_string()];
    assert!(lacks(told().as_bytes(), &outs, |_| true, last, out).is_empty());
    for record in [
        call("SendMessage", "w219a"),
        call("Write", "team-lead"),
        String::new(),
    ] {
        assert_eq!(
            lacks(record.as_bytes(), &outs, |_| true, last, out),
            vec![UNTOLD.to_string()]
        );
    }
}

/// 中核の欠けの数え: 無い出す物は 1 つずつ、最後の文の path の欠けは末に、席への知らせ・出す物・path の順で並ぶ。
#[test]
fn agsg_lacks_lists_each_missing_output_and_the_path_in_order() {
    let (out, last) = ("/D/w218a/w", "出す物は /D/w218a/w に在る");
    let outs = vec!["notes.md".to_string(), "a.patch".to_string()];
    let record = told();
    let only = lacks(record.as_bytes(), &outs, |o| o == "notes.md", last, out);
    assert_eq!(
        only,
        vec!["出す物 a.patch が /D/w218a/w/ に無い".to_string()]
    );
    let path = lacks(record.as_bytes(), &outs, |_| true, "済み", out);
    assert_eq!(
        path,
        vec!["最後の文に出力の dir の path /D/w218a/w が無い".to_string()]
    );
    let all = lacks(b"", &outs, |_| false, "済み", out);
    assert_eq!(
        all,
        vec![
            UNTOLD.to_string(),
            "出す物 notes.md が /D/w218a/w/ に無い".to_string(),
            "出す物 a.patch が /D/w218a/w/ に無い".to_string(),
            "最後の文に出力の dir の path /D/w218a/w が無い".to_string(),
        ]
    );
}

/// 歯ごとの置き場（前の撃ちの残りを消して作る）。drafts/ と、係の記録を置く親の記録の dir。
fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("agsg")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("drafts")).unwrap();
    fs::create_dir_all(root.join("s/subagents")).unwrap();
    root
}

/// 欠けの無い終わりの置き場: 係 w218a の札（出す物 notes.md）と係の id a77 の結び・席への知らせの在る係の記録・w/notes.md。
fn whole(name: &str) -> PathBuf {
    let root = place(name);
    let dir = root.join("drafts/w218a");
    fs::create_dir_all(dir.join("w")).unwrap();
    fs::write(
        dir.join("spec.json"),
        r#"{"name":"w218a","type":"tsuzuri:drafter","budget":1000,"build":"なし","target":"t3-hub.87","outputs":["notes.md"],"spawned":1,"agent_id":"a77","ended":null}"#,
    )
    .unwrap();
    fs::write(dir.join("w/notes.md"), "分かった所\n").unwrap();
    fs::create_dir_all(root.join("drafts/.agents")).unwrap();
    fs::write(root.join("drafts/.agents/a77"), "w218a\n").unwrap();
    fs::write(root.join("s/subagents/agent-a77.jsonl"), told()).unwrap();
    root
}

/// 出力の dir の path の字。
fn w(root: &Path) -> String {
    root.join("drafts/w218a/w").display().to_string()
}

/// 最後の文（出力の dir の path を持つ）。
fn said(root: &Path) -> String {
    format!("済み。出す物は {} に在る", w(root))
}

/// tz hook agent-stop に、係の id `head`（空なら席）の終わりの入力（止めた後の終わりか `again`・最後の文 `last`）を渡した結果。
fn stop(root: &Path, head: &str, again: bool, last: &str) -> Output {
    let input = format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"{}","cwd":"/W","permission_mode":"bypassPermissions",{head}"hook_event_name":"SubagentStop","stop_hook_active":{again},"agent_transcript_path":"{}","last_assistant_message":"{last}"}}"#,
        root.join("s.jsonl").display(),
        root.join("s/subagents/agent-a77.jsonl").display()
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

/// 係の終わりの頭の欄（係の id a77）。
const SUB: &str = r#""agent_id":"a77","agent_type":"tsuzuri:drafter","#;

/// 係 w218a の札。
fn spec(root: &Path) -> Spec {
    Spec::parse(&fs::read_to_string(root.join("drafts/w218a/spec.json")).unwrap()).unwrap()
}

/// 欠けの無い終わりから 1 句だけ外した 3 つは、1 度目の終わりを rc 2 と標準エラーの理由で止め、札と w/ を書かない。
#[test]
fn agsg_hook_holds_the_first_stop_with_one_hole() {
    let untold = whole("untold");
    fs::write(
        untold.join("s/subagents/agent-a77.jsonl"),
        call("SendMessage", "w219a"),
    )
    .unwrap();
    let bare = whole("bare");
    fs::remove_file(bare.join("drafts/w218a/w/notes.md")).unwrap();
    let mute = whole("mute");
    for (root, last, hole) in [
        (&untold, said(&untold), UNTOLD.to_string()),
        (
            &bare,
            said(&bare),
            format!("出す物 notes.md が {}/ に無い", w(&bare)),
        ),
        (
            &mute,
            "済み".to_string(),
            format!("最後の文に出力の dir の path {} が無い", w(&mute)),
        ),
    ] {
        let out = stop(root, SUB, false, &last);
        assert_eq!((out.status.code(), out.stdout.len()), (Some(2), 0));
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.starts_with(&format!("係の終える前の門は止める（{hole}） 次の一手 = ")),
            "{err}"
        );
        assert_eq!(spec(root).ended, None);
        assert!(!root.join("drafts/w218a/w/STOP-GATE.txt").exists());
    }
}

/// 欠けの無い 1 度目の終わりは何も出さずに rc 0 で通し、札に終えの印（今の時刻）を書く。
#[test]
fn agsg_hook_passes_a_whole_stop_and_marks_it_ended() {
    let root = whole("pass");
    let out = stop(&root, SUB, false, &said(&root));
    assert_eq!(
        (out.status.code(), out.stdout.len(), out.stderr.len()),
        (Some(0), 0, 0)
    );
    assert!(spec(&root).ended.is_some_and(|t| t > 1_700_000_000));
    assert!(!root.join("drafts/w218a/w/STOP-GATE.txt").exists());
}

/// 止めた後の 2 度目の終わりは、欠けが在っても rc 0 で通し、w/STOP-GATE.txt に欠けを書き、札に終えの印を書く。
#[test]
fn agsg_hook_lets_the_second_stop_go_and_writes_the_holes() {
    let root = whole("again");
    fs::remove_file(root.join("drafts/w218a/w/notes.md")).unwrap();
    let out = stop(&root, SUB, true, &said(&root));
    assert_eq!((out.status.code(), out.stdout.len()), (Some(0), 0));
    let gate = fs::read_to_string(root.join("drafts/w218a/w/STOP-GATE.txt")).unwrap();
    assert_eq!(gate, format!("出す物 notes.md が {}/ に無い\n", w(&root)));
    assert!(spec(&root).ended.is_some());
}

/// 席の終わりは何も読まず何も書かず、結びの無い係の id の終わりは通して unbound.jsonl に 1 行だけ足す。
#[test]
fn agsg_hook_passes_the_seat_and_records_an_unbound_agent() {
    let root = place("unbound");
    let seat = stop(&root, "", false, "済み");
    assert_eq!((seat.status.code(), seat.stdout.len()), (Some(0), 0));
    assert!(!root.join("drafts/.agents").exists());
    let sub = stop(&root, SUB, false, "済み");
    assert_eq!((sub.status.code(), sub.stdout.len()), (Some(0), 0));
    let log = fs::read_to_string(root.join("drafts/.agents/unbound.jsonl")).unwrap();
    assert_eq!(log.lines().count(), 1);
    assert!(log.starts_with(r#"{"agent_id":"a77","at":"#), "{log}");
    assert!(
        log.ends_with("\"event\":\"SubagentStop\",\"tool\":\"\"}\n"),
        "{log}"
    );
    assert!(!root.join("drafts/w218a").exists());
}
