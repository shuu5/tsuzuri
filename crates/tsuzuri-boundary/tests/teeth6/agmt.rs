//! 係の測りの歯（接頭辞 agmt_・設計ノート surface-wave29b 行 ag-meter・判断の記録 ADR-59 決定 (3)(4)）。
//! 中核の `feed` と `cross` に係の記録の行の形（assistant の行の message の id と usage）を渡し、tz の binary の tz hook agent-meter に
//! 係の呼びの門の入力の形（会話の id と path は伏せた）を標準入力で渡し、歯ごとの置き場（CARGO_TARGET_TMPDIR の下）を --drafts で渡す。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_core::agent::meter::Meter;

/// assistant の行（応答の id・input・cache_creation・output・cache_read）。
fn line(id: &str, usage: [u64; 4]) -> String {
    let [i, c, o, r] = usage;
    format!(
        r#"{{"type":"assistant","message":{{"id":"{id}","usage":{{"input_tokens":{i},"cache_creation_input_tokens":{c},"output_tokens":{o},"cache_read_input_tokens":{r}}}}}}}"#
    ) + "\n"
}

/// 歯ごとの置き場（前の撃ちの残りを消して作る）。drafts/ と、係の記録を置く親の記録の dir。
fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("agmt")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("drafts")).unwrap();
    fs::create_dir_all(root.join("s/subagents")).unwrap();
    root
}

/// 係 w209a の札（予算 1000）と、係の id a77 から名への結び。
fn bind(root: &Path) {
    let dir = root.join("drafts/w209a");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.json"),
        r#"{"name":"w209a","type":"tsuzuri:drafter","budget":1000,"build":"なし","target":"t3-hub.87","outputs":["notes.md"],"spawned":1,"agent_id":"a77","ended":null}"#,
    )
    .unwrap();
    fs::create_dir_all(root.join("drafts/.agents")).unwrap();
    fs::write(root.join("drafts/.agents/a77"), "w209a\n").unwrap();
}

/// 係の記録（親の記録 s.jsonl の dir の下の subagents/agent-a77.jsonl）に行を足す。
fn append(root: &Path, text: &str) {
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("s/subagents/agent-a77.jsonl"))
        .unwrap();
    f.write_all(text.as_bytes()).unwrap();
}

/// tz hook agent-meter に、係の id `head`（空なら席の呼び）の Bash の呼びの結果の入力を渡した結果。
fn meter(root: &Path, head: &str) -> Output {
    let parent = root.join("s.jsonl");
    let input = format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"{}","cwd":"/W","permission_mode":"bypassPermissions",{head}"hook_event_name":"PostToolUse","tool_name":"Bash","tool_input":{{"command":"echo"}},"tool_response":{{"stdout":""}},"tool_use_id":"toolu_X"}}"#,
        parent.display()
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["hook", "agent-meter", "--repo"])
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

/// 係の呼びの頭の欄（係の id a77）。
const SUB: &str = r#""agent_id":"a77","agent_type":"tsuzuri:drafter","#;

/// 測りの札の字。
fn read(root: &Path) -> Meter {
    Meter::parse(&fs::read_to_string(root.join("drafts/w209a/meter.json")).unwrap()).unwrap()
}

/// 同じ応答の id の続く 2 行は後の行の分だけを 1 度数え、cache の読みは和に入れず別の欄に足し、user の行は数えない。
#[test]
fn agmt_feed_counts_one_message_id_once_and_keeps_cache_read_apart() {
    let user = r#"{"type":"user","message":{"id":"u1","usage":{"input_tokens":7000}}}"#;
    let text = line("A", [5, 100, 10, 1000])
        + &line("A", [5, 100, 30, 1000])
        + user
        + "\n"
        + &line("B", [1, 20, 4, 2000]);
    let mut m = Meter::default();
    m.feed(text.as_bytes());
    assert_eq!((m.used, m.cache_read), (135 + 25, 3000));
    assert_eq!(m.offset, text.len() as u64);
}

/// 末の書きかけの行は読まずに offset を最後の改行の後に置き、続きを足すと全部を 1 度に読んだ時と同じ和になる（同じ id の行が境を跨いでも）。
#[test]
fn agmt_feed_reads_only_whole_lines_from_the_offset() {
    let whole =
        line("A", [5, 100, 10, 1000]) + &line("A", [5, 100, 30, 1000]) + &line("B", [1, 20, 4, 0]);
    let cut = line("A", [5, 100, 10, 1000]).len() + 12;
    let mut parts = Meter::default();
    parts.feed(&whole.as_bytes()[..cut]);
    assert_eq!((parts.offset, parts.used), (cut as u64 - 12, 115));
    parts.feed(&whole.as_bytes()[parts.offset as usize..]);
    let mut once = Meter::default();
    once.feed(whole.as_bytes());
    assert_eq!(
        (parts.offset, parts.used, parts.cache_read),
        (once.offset, once.used, once.cache_read)
    );
    assert_eq!(parts.used, 160);
}

/// 50・75・90% の印は越えた最初の 1 回だけ返し、2 つを 1 度に越えると大きい印を返して両方を札に残す。
#[test]
fn agmt_cross_returns_each_mark_once() {
    let mut m = Meter {
        used: 499,
        ..Meter::default()
    };
    assert_eq!(m.cross(1000), None);
    m.used = 500;
    assert_eq!(m.cross(1000), Some(50));
    assert_eq!(m.cross(1000), None);
    m.used = 920;
    assert_eq!(m.cross(1000), Some(90));
    assert_eq!(m.marks, vec![50, 75, 90]);
    m.used = 2000;
    assert_eq!(m.cross(1000), None);
}

/// 結んだ係の記録を測りの札の offset から読み、印を越えた周だけ残りの注ぎを 1 行出し、次の周は出さない。
#[test]
fn agmt_hook_measures_from_the_offset_and_injects_once_per_mark() {
    let root = place("measure");
    bind(&root);
    let before = line("Z", [0, 0, 900, 0]);
    append(&root, &before);
    let start = format!(
        r#"{{"offset":{},"used":0,"cache_read":0,"last_id":null,"last_used":0,"last_cache_read":0,"marks":[]}}"#,
        before.len()
    );
    fs::write(root.join("drafts/w209a/meter.json"), start).unwrap();
    append(
        &root,
        &(line("A", [5, 100, 400, 7000]) + &line("A", [5, 100, 420, 7000])),
    );
    let out = meter(&root, SUB);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.starts_with(
            r#"{"hookSpecificOutput":{"additionalContext":"係の測り: 予算 1000 の 50% を越えた"#
        ),
        "{text}"
    );
    assert!(
        text.ends_with("\",\"hookEventName\":\"PostToolUse\"}}\n"),
        "{text}"
    );
    assert!(
        text.contains("予算 1000 の 50% を越えた（使った量 525 "),
        "{text}"
    );
    assert!(
        text.contains("残り 475・cache の読み 7000 は別の欄"),
        "{text}"
    );
    assert_eq!((read(&root).used, read(&root).marks), (525, vec![50]));
    let again = meter(&root, SUB);
    assert_eq!((again.status.code(), again.stdout.len()), (Some(0), 0));
    append(&root, &line("B", [0, 0, 300, 0]));
    let third = meter(&root, SUB);
    assert!(String::from_utf8_lossy(&third.stdout).contains("の 75% を越えた（使った量 825 "));
}

/// 結びの無い係の id の呼びは通して unbound.jsonl に 1 行足し、席の呼びは何も読まず何も書かない。
#[test]
fn agmt_hook_records_an_unbound_agent_and_passes_the_seat() {
    let root = place("unbound");
    append(&root, &line("A", [0, 0, 900, 0]));
    let seat = meter(&root, "");
    assert_eq!((seat.status.code(), seat.stdout.len()), (Some(0), 0));
    assert!(!root.join("drafts/.agents").exists());
    let sub = meter(&root, SUB);
    assert_eq!((sub.status.code(), sub.stdout.len()), (Some(0), 0));
    let log = fs::read_to_string(root.join("drafts/.agents/unbound.jsonl")).unwrap();
    assert_eq!(log.lines().count(), 1);
    assert!(log.starts_with(r#"{"agent_id":"a77","at":"#), "{log}");
    assert!(
        log.ends_with("\"event\":\"PostToolUse\",\"tool\":\"Bash\"}\n"),
        "{log}"
    );
    assert!(!root.join("drafts/w209a").exists());
}
