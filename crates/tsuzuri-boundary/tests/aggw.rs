//! 群の係の測りと係の門の歯（接頭辞 aggw_・設計ノート surface-wave29b 行 ag-gwatch・判断の記録 ADR-61 決定 (4)(5)(7)）。
//! 中核の `inside` を直に撃ち、tz の binary の tz hook agent-meter と agent-guard に係の呼びの門の入力の形（会話の id と path は伏せた）を
//! 標準入力で渡し、歯ごとの置き場（CARGO_TARGET_TMPDIR の下）の drafts/ を --drafts で渡す。群の席の札は行 ag-gspawn が書く形
//! （群 g1・番 1・数 3・割りの読む path /S/a.md と dir /S/d）を歯が置く。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_core::agent::meter::Meter;
use tsuzuri_core::agent::meter::group::inside;

/// 歯ごとの置き場（前の撃ちの残りを消して作る）。drafts/ と群 g1 の dir と、係の記録を置く親の記録の dir。
fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("aggw")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("drafts/g1")).unwrap();
    fs::create_dir_all(root.join("s/subagents")).unwrap();
    root
}

/// 係 `name` の札（予算 1000）と係の id `id` から名への結びと、`seat` なら群 g1 の席の札。
fn bind(root: &Path, name: &str, id: &str, seat: bool) {
    let dir = root.join("drafts").join(name);
    fs::create_dir_all(&dir).unwrap();
    let spec = format!(
        r#"{{"name":"{name}","type":"tsuzuri:verifier","budget":1000,"build":"なし","target":"t3-hub.89","outputs":["notes.md"],"spawned":1,"agent_id":"{id}","ended":null}}"#
    );
    fs::write(dir.join("spec.json"), spec).unwrap();
    fs::create_dir_all(root.join("drafts/.agents")).unwrap();
    fs::write(root.join("drafts/.agents").join(id), format!("{name}\n")).unwrap();
    if seat {
        let seat = r#"{"group":"g1","i":1,"k":3,"claims":["U1"],"paths":["/S/a.md","/S/d"]}"#;
        fs::write(dir.join("group.json"), seat).unwrap();
    }
}

/// 係 `name` の測りの札（使った量 `used`・読み直し `read`）。
fn metered(root: &Path, name: &str, used: u64, read: u64) {
    let meter = format!(
        r#"{{"offset":0,"used":{used},"cache_read":{read},"last_id":null,"last_used":0,"last_cache_read":0,"marks":[]}}"#
    );
    fs::write(root.join("drafts").join(name).join("meter.json"), meter).unwrap();
}

/// 係の id `id` の係の記録に assistant の行（応答の id・output・cache_read）を足す。
fn append(root: &Path, id: &str, msg: &str, output: u64, read: u64) {
    let line = format!(
        r#"{{"type":"assistant","message":{{"id":"{msg}","usage":{{"input_tokens":0,"cache_creation_input_tokens":0,"output_tokens":{output},"cache_read_input_tokens":{read}}}}}}}"#
    );
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join(format!("s/subagents/agent-{id}.jsonl")))
        .unwrap();
    writeln!(f, "{line}").unwrap();
}

/// tz hook `verb` に、係の id `id` の道具 `tool`（tool_input は `body` の JSON の字）の入力（係の門は呼びの前・測りの口は呼びの後）を渡した結果。
fn hook(root: &Path, verb: &str, id: &str, tool: &str, body: &str) -> Output {
    let event = if verb == "agent-guard" {
        "PreToolUse"
    } else {
        "PostToolUse"
    };
    let input = format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"{}","cwd":"/W","permission_mode":"bypassPermissions","agent_id":"{id}","agent_type":"tsuzuri:verifier","hook_event_name":"{event}","tool_name":"{tool}","tool_input":{body},"tool_use_id":"toolu_X"}}"#,
        root.join("s.jsonl").display()
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["hook", verb, "--repo"])
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

/// 係の門に係 a81 の呼びの前の入力を渡し、rc と標準出力の字を返す。
fn guard(root: &Path, tool: &str, body: &str) -> (Option<i32>, String) {
    let out = hook(root, "agent-guard", "a81", tool, body);
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// 測りの口に係の id `id` の呼びの結果の入力を渡し、rc と標準出力の字を返す。
fn meter(root: &Path, id: &str, tool: &str, body: &str) -> (Option<i32>, String) {
    let out = hook(root, "agent-meter", id, tool, body);
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// 係 `name` の測りの札。
fn read(root: &Path, name: &str) -> Meter {
    let text = fs::read_to_string(root.join("drafts").join(name).join("meter.json")).unwrap();
    Meter::parse(&text).unwrap()
}

/// deny の答えの頭。
const DENY: &str = r#"{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"係の門は止める（"#;

/// 割りの読む path と同じ path と dir の下の path だけが中で、名の続く path・`..` を持つ path・相対の path・割りの無い時は外。
#[test]
fn aggw_inside_reads_a_seat_path_or_a_path_under_its_dir_only() {
    let paths = vec!["/S/a.md".to_string(), "/S/d".to_string()];
    for p in ["/S/a.md", "/S/d", "/S/d/", "/S/d/x.md", "/S/d/e/y.md"] {
        assert!(inside(p, &paths), "{p}");
    }
    for p in [
        "/S/a.mdx",
        "/S/dx/y.md",
        "/S/b.md",
        "/S/d/../b.md",
        "/S/a.md/..",
        "S/a.md",
        "/S",
    ] {
        assert!(!inside(p, &paths), "{p}");
    }
    assert!(!inside("/S/a.md", &[]));
}

/// 群の係の読み直しは上限 2500000 の 50・75・90% の印を越えた最初の 1 回だけ注ぎ、新しい量と同じ回に越えた印は 1 行に並べる。
#[test]
fn aggw_hook_injects_each_read_mark_of_a_member_once() {
    let root = place("marks");
    bind(&root, "qv181", "a81", true);
    append(&root, "a81", "A", 10, 1_249_999);
    assert_eq!(meter(&root, "a81", "Bash", "{}"), (Some(0), String::new()));
    append(&root, "a81", "B", 490, 1);
    let (rc, text) = meter(&root, "a81", "Bash", "{}");
    assert_eq!(rc, Some(0));
    assert!(
        text.starts_with(r#"{"hookSpecificOutput":{"additionalContext":"係の測り（群の係）: 新しい量 50%・読み直し 50% を越えた（新しい量 500 は予算 1000 の残り 500・読み直し 1250000 は上限 2500000 の残り 1250000・"#),
        "{text}"
    );
    append(&root, "a81", "C", 0, 625_000);
    let (_, text) = meter(&root, "a81", "Bash", "{}");
    assert!(
        text.contains(r#""係の測り（群の係）: 読み直し 75% を越えた（新しい量 500 "#),
        "{text}"
    );
    append(&root, "a81", "D", 0, 375_000);
    let (_, text) = meter(&root, "a81", "Bash", "{}");
    assert!(
        text.contains(r#""係の測り（群の係）: 読み直し 90% を越えた（"#),
        "{text}"
    );
    assert_eq!(meter(&root, "a81", "Bash", "{}"), (Some(0), String::new()));
    let m = read(&root, "qv181");
    assert_eq!(
        (m.cache_read, m.marks, m.read_marks),
        (2_250_000, vec![50], vec![50, 75, 90])
    );
}

/// 群の係の読みの道具の割りの外の path と係を起こす道具の呼びは断り、割りの中の読みと命令と検索の道具の読みは通す。
#[test]
fn aggw_guard_fences_a_member_read_outside_its_paths_and_the_agent_tool() {
    let root = place("fence");
    bind(&root, "qv181", "a81", true);
    for (tool, body) in [
        ("Read", r#"{"file_path":"/S/a.md"}"#),
        ("Read", r#"{"file_path":"/S/d/x.md","limit":20}"#),
        ("Bash", r#"{"command":"cat /S/b.md"}"#),
        ("Grep", r#"{"pattern":"x","path":"/S/b.md"}"#),
        ("Glob", r#"{"pattern":"*.md","path":"/S"}"#),
    ] {
        assert_eq!(
            guard(&root, tool, body),
            (Some(0), String::new()),
            "{tool} {body}"
        );
    }
    let (rc, text) = guard(&root, "Read", r#"{"file_path":"/S/b.md"}"#);
    assert_eq!(rc, Some(0));
    assert!(
        text.starts_with(&format!(
            "{DENY}群 g1 の係の読みの道具 Read の path /S/b.md は割りの読む path の外・判断の記録 ADR-61 の決定 (4)） 次の一手 = "
        )),
        "{text}"
    );
    assert!(
        text.contains("分からない（U）のまま要る path を w/ に書いて席に返す"),
        "{text}"
    );
    let (rc, text) = guard(&root, "Agent", r#"{"description":"x","prompt":"y"}"#);
    assert_eq!(rc, Some(0));
    assert!(
        text.starts_with(&format!(
            "{DENY}群 g1 の係は係を起こす道具 Agent を呼ばない（入れ子）・"
        )),
        "{text}"
    );
}

/// 群の係の読み直しが上限 2500000 以上なら、自分の w/ への書きと SendMessage のほかを断り、2499999 なら通す。
#[test]
fn aggw_guard_enters_the_last_stage_when_a_member_reads_up_to_the_limit() {
    let root = place("spent");
    bind(&root, "qv181", "a81", true);
    metered(&root, "qv181", 0, 2_500_000);
    let own = root.join("drafts/qv181/w/notes.md");
    let write = format!(r#"{{"file_path":"{}","content":"x"}}"#, own.display());
    for (tool, body) in [
        ("Write", write.as_str()),
        ("SendMessage", r#"{"to":"team-lead","message":"済み"}"#),
    ] {
        assert_eq!(guard(&root, tool, body), (Some(0), String::new()), "{tool}");
    }
    for (tool, body) in [
        ("Bash", r#"{"command":"echo"}"#),
        ("Read", r#"{"file_path":"/S/a.md"}"#),
    ] {
        let (rc, text) = guard(&root, tool, body);
        assert_eq!(rc, Some(0));
        assert!(
            text.starts_with(&format!(
                "{DENY}群の係の読み直し 2500000 が上限 2500000 以上の書き終えの段で、道具 {tool} は通さない・"
            )),
            "{text}"
        );
    }
    metered(&root, "qv181", 0, 2_499_999);
    assert_eq!(
        guard(&root, "Bash", r#"{"command":"echo"}"#),
        (Some(0), String::new())
    );
}

/// 群の席の札の無い係は、読み直しが上限を越えても注がず断らず、割りの外の読みと係を起こす道具の呼びも通し、読みを記帳しない。
#[test]
fn aggw_hooks_leave_a_non_member_to_the_adr59_gates() {
    let root = place("loner");
    bind(&root, "qv181", "a81", false);
    metered(&root, "qv181", 0, 3_000_000);
    for (tool, body) in [
        ("Bash", r#"{"command":"echo"}"#),
        ("Read", r#"{"file_path":"/S/b.md"}"#),
        ("Agent", r#"{"description":"x","prompt":"y"}"#),
    ] {
        assert_eq!(guard(&root, tool, body), (Some(0), String::new()), "{tool}");
    }
    append(&root, "a81", "A", 10, 2_500_000);
    let body = r#"{"file_path":"/S/a.md"}"#;
    assert_eq!(meter(&root, "a81", "Read", body), (Some(0), String::new()));
    assert!(read(&root, "qv181").read_marks.is_empty());
    assert!(!root.join("drafts/g1/reads.jsonl").exists());
}

/// 群の係の読みの道具の path は群の dir の reads.jsonl に足し、同じ群のほかの係が先に読んだ path を初めて読んだ時だけ twice.jsonl に足し、断らない。
#[test]
fn aggw_hook_records_a_path_read_by_two_members_without_refusing() {
    let root = place("twice");
    bind(&root, "qv181", "a81", true);
    bind(&root, "qv182", "a82", true);
    let (a, c) = (r#"{"file_path":"/S/a.md"}"#, r#"{"file_path":"/S/d/c.md"}"#);
    let quiet = (Some(0), String::new());
    assert_eq!(meter(&root, "a81", "Read", a), quiet);
    assert!(!root.join("drafts/g1/twice.jsonl").exists());
    assert_eq!(meter(&root, "a82", "Read", a), quiet);
    assert_eq!(meter(&root, "a82", "Read", a), quiet);
    assert_eq!(meter(&root, "a81", "Read", c), quiet);
    assert_eq!(
        meter(&root, "a81", "Bash", r#"{"command":"cat /S/a.md"}"#),
        quiet
    );
    let edit = r#"{"file_path":"/S/d/c.md","old_string":"a","new_string":"b"}"#;
    assert_eq!(meter(&root, "a82", "Edit", edit), quiet);
    let reads = fs::read_to_string(root.join("drafts/g1/reads.jsonl")).unwrap();
    assert_eq!(reads.lines().count(), 4);
    assert!(
        reads
            .lines()
            .nth(1)
            .unwrap()
            .ends_with(r#","name":"qv182","path":"/S/a.md"}"#),
        "{reads}"
    );
    let twice = fs::read_to_string(root.join("drafts/g1/twice.jsonl")).unwrap();
    assert_eq!(twice.lines().count(), 1, "{twice}");
    assert!(twice.starts_with(r#"{"at":"#), "{twice}");
    assert!(
        twice.ends_with(",\"first\":\"qv181\",\"name\":\"qv182\",\"path\":\"/S/a.md\"}\n"),
        "{twice}"
    );
}
