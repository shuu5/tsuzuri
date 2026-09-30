//! 問いの起票の門の hook の plugin の登録の歯（行 f-gate-install・接頭辞 gins_）。
//! workspace の根の plugin/hooks/hooks.json を JSON として読み、PreToolUse の command を sh で撃つ
//! （境界の crate は serde_json に直に依存しないので、この歯は中核の crate に置く）。

use std::io::{ErrorKind, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::Value;

/// hooks.json の PreToolUse の command の語（二重引用符を除いた字）。
const COMMAND_WORDS: [&str; 10] = [
    "[",
    "-e",
    "$CLAUDE_PLUGIN_ROOT/scribe2-runner",
    "]",
    "||",
    "$CLAUDE_PROJECT_DIR/target/debug/tz",
    "hook",
    "question-gate",
    "--repo",
    "$CLAUDE_PROJECT_DIR",
];

/// 名の字を持つ環境変数（command の中で二重引用符に挟む）。
const PROJECT_DIR: &str = "$CLAUDE_PROJECT_DIR";

/// runner の印の字（command の中で二重引用符に挟む）。
const RUNNER_MARK: &str = "$CLAUDE_PLUGIN_ROOT/scribe2-runner";

/// runner の印の file の名。
const MARK_NAME: &str = "scribe2-runner";

/// 席の Bash の呼びの標準入力（問いの起票でない command）。
const PAYLOAD: &str = r#"{"tool_name":"Bash","tool_input":{"command":"ls"}}"#;

/// 偽の tz が標準出力に出す字。
const FAKE_OUT: &str = "deny-line\n";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_hooks() -> Value {
    let path = root().join("plugin/hooks/hooks.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("hooks.json を読む: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("hooks.json は JSON でない: {e}"))
}

/// object の鍵を並べ替えた列。
fn keys(v: &Value) -> Vec<&str> {
    let mut keys: Vec<&str> = v
        .as_object()
        .unwrap_or_else(|| panic!("object でない: {v}"))
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    keys
}

/// 要素 1 つの配列のその要素。
fn only(v: &Value) -> &Value {
    match v.as_array().map(Vec::as_slice) {
        Some([one]) => one,
        _ => panic!("要素 1 つの配列でない: {v}"),
    }
}

/// 空白で分けて二重引用符を除いた語。
fn words(text: &str) -> Vec<String> {
    text.split_whitespace().map(|w| w.replace('"', "")).collect()
}

/// PreToolUse の要素の hook の command の字。
fn gate_command(hooks: &Value) -> String {
    let entry = only(&hooks["hooks"]["PreToolUse"]);
    let hook = only(&entry["hooks"]);
    hook["command"]
        .as_str()
        .expect("command は字")
        .to_owned()
}

#[test]
fn gins_pretool_bash_entry() {
    let hooks = read_hooks();
    assert_eq!(keys(&hooks), ["hooks"]);
    assert_eq!(
        keys(&hooks["hooks"]),
        [
            "PostToolBatch",
            "PostToolUse",
            "PreToolUse",
            "Stop",
            "UserPromptSubmit"
        ]
    );
    let entry = only(&hooks["hooks"]["PreToolUse"]);
    assert_eq!(keys(entry), ["hooks", "matcher"]);
    assert_eq!(entry["matcher"], "Bash");
    let hook = only(&entry["hooks"]);
    assert_eq!(keys(hook), ["command", "timeout", "type"]);
    assert_eq!(hook["type"], "command");
    assert_eq!(hook["timeout"].as_u64(), Some(10), "timeout は数 10");
    let command = hook["command"].as_str().expect("command は字");
    assert_eq!(words(command), COMMAND_WORDS, "{command}");
    let quoted_dir = format!("\"{PROJECT_DIR}\"");
    let quoted_mark = format!("\"{RUNNER_MARK}\"");
    assert_eq!(command.matches(PROJECT_DIR).count(), 2, "{command}");
    assert_eq!(command.matches(&quoted_dir).count(), 2, "{command}");
    assert_eq!(command.matches(&quoted_mark).count(), 1, "{command}");
}

/// dir の下を入れ子までたどり、見た中身の数を数え、印の名の中身が無いことを見る。
fn walk(dir: &Path, seen: &mut usize) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{} を読む: {e}", dir.display())) {
        let entry = entry.expect("dir の中身を読む");
        *seen += 1;
        assert_ne!(
            entry.file_name(),
            MARK_NAME,
            "plugin の dir の下に印が在る: {}",
            entry.path().display()
        );
        let kind = entry.file_type().expect("中身の種を読む");
        if kind.is_dir() {
            walk(&entry.path(), seen);
        }
    }
}

#[test]
fn gins_repo_plugin_has_no_runner_mark() {
    let mut seen = 0;
    walk(&root().join("plugin"), &mut seen);
    assert!(seen >= 4, "見た中身が {seen} で 4 より少ない");
}

/// command を sh -c で撃ち、payload を標準入力に書いて子の終わりを待つ。
fn run_sh(command: &str, project: &Path, plugin: &Path) -> Output {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .env("CLAUDE_PROJECT_DIR", project)
        .env("CLAUDE_PLUGIN_ROOT", plugin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("sh を撃つ");
    let mut stdin = child.stdin.take().expect("標準入力");
    match stdin.write_all(PAYLOAD.as_bytes()) {
        Ok(()) => {}
        // sh が標準入力を読まずに先に終わると書きは Broken pipe を返しうる。
        Err(e) if e.kind() == ErrorKind::BrokenPipe => {}
        Err(e) => panic!("payload を書く: {e}"),
    }
    drop(stdin);
    child.wait_with_output().expect("sh の終わりを待つ")
}

#[test]
fn gins_command_under_sh() {
    let command = gate_command(&read_hooks());
    let work = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("gins-{}", std::process::id()));
    if work.exists() {
        std::fs::remove_dir_all(&work).expect("前の作業場を消す");
    }
    let project = work.join("project");
    let bin = project.join("target/debug");
    std::fs::create_dir_all(&bin).expect("project の dir を作る");
    let args_rec = work.join("args.rec");
    let stdin_rec = work.join("stdin.rec");
    let script = format!(
        "#!/bin/sh\nfor a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{}'\ncat > '{}'\nprintf '%s' '{FAKE_OUT}'\n",
        args_rec.display(),
        stdin_rec.display(),
    );
    let tz = bin.join("tz");
    std::fs::write(&tz, script).expect("偽の tz を書く");
    std::fs::set_permissions(&tz, std::fs::Permissions::from_mode(0o755)).expect("偽の tz を撃てる形にする");

    let plain = work.join("plugin-plain");
    std::fs::create_dir_all(&plain).expect("印の無い plugin の dir を作る");
    let marked = work.join("plugin-marked");
    std::fs::create_dir_all(&marked).expect("印の在る plugin の dir を作る");
    std::fs::write(marked.join(MARK_NAME), "").expect("印を置く");
    let bare = work.join("project-bare");
    std::fs::create_dir_all(&bare).expect("tz の無い project の dir を作る");

    // 席: 印が無く tz が在る。
    let out = run_sh(&command, &project, &plain);
    assert_eq!(out.status.code(), Some(0), "席の rc: {out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout), FAKE_OUT, "席の標準出力");
    let args = std::fs::read_to_string(&args_rec).expect("tz が撃たれて引数を記録した");
    let want = format!("hook\nquestion-gate\n--repo\n{}\n", project.display());
    assert_eq!(args, want, "tz の引数");
    let stdin = std::fs::read_to_string(&stdin_rec).expect("tz が標準入力を記録した");
    assert_eq!(stdin, PAYLOAD, "tz の標準入力");

    // runner: 印が在る。
    std::fs::remove_file(&args_rec).expect("引数の記録を消す");
    std::fs::remove_file(&stdin_rec).expect("標準入力の記録を消す");
    let out = run_sh(&command, &project, &marked);
    assert_eq!(out.status.code(), Some(0), "runner の rc: {out:?}");
    assert!(out.stdout.is_empty(), "runner の標準出力: {out:?}");
    assert!(!args_rec.exists(), "runner で tz が撃たれた");
    assert!(!stdin_rec.exists(), "runner で tz が撃たれた");

    // tz が無い: 止めない誤り。
    let out = run_sh(&command, &bare, &plain);
    let code = out.status.code();
    assert!(code != Some(0) && code != Some(2), "tz の無い場の rc: {out:?}");
    assert!(out.stdout.is_empty(), "tz の無い場の標準出力: {out:?}");

    std::fs::remove_dir_all(&work).expect("作業場を消す");
}
