//! 係の門の全時間の検めの歯（接頭辞 agfen_・判断の記録 ADR-59 決定 (3)(4)・条 N-2）。
//! 中核の `cargo_word` と `fenced` を直に撃ち、tz の binary の tz hook agent-guard に係の呼びの入力の形（会話の id と path は伏せた）を
//! 標準入力で渡し、歯ごとの置き場（CARGO_TARGET_TMPDIR の下）を --drafts で渡す。測りの札は置かない（使った量 0 で予算の前の段）。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_core::agent::guard::{cargo_word, fenced};

/// 歯ごとの置き場（前の撃ちの残りを消して作る）の drafts/。
fn place(name: &str) -> PathBuf {
    let drafts = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("agfen")
        .join(name)
        .join("drafts");
    let _ = fs::remove_dir_all(&drafts);
    fs::create_dir_all(&drafts).unwrap();
    drafts
}

/// 係 w218a の札（予算 1000・組み `build`・出す物 notes.md）と、係の id a77 から名への結び。
fn bind(drafts: &Path, build: &str) {
    let dir = drafts.join("w218a");
    fs::create_dir_all(&dir).unwrap();
    let spec = format!(
        r#"{{"name":"w218a","type":"tsuzuri:drafter","budget":1000,"build":"{build}","target":"t3-hub.87","outputs":["notes.md"],"spawned":1,"agent_id":"a77","ended":null}}"#
    );
    fs::write(dir.join("spec.json"), spec).unwrap();
    fs::create_dir_all(drafts.join(".agents")).unwrap();
    fs::write(drafts.join(".agents/a77"), "w218a\n").unwrap();
}

/// 係の id `head`（空なら席の呼び）の道具 `tool` の呼びの前の入力（tool_input は `input` の JSON の字）。
fn input(head: &str, tool: &str, input: &str) -> String {
    format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"/T/s.jsonl","cwd":"/W","permission_mode":"bypassPermissions",{head}"hook_event_name":"PreToolUse","tool_name":"{tool}","tool_input":{input},"tool_use_id":"toolu_X"}}"#
    )
}

/// tz hook agent-guard に `payload` を渡した結果。
fn guard(drafts: &Path, payload: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["hook", "agent-guard", "--repo"])
        .arg(drafts)
        .arg("--drafts")
        .arg(drafts)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(payload.as_bytes());
    child.wait_with_output().unwrap()
}

/// 係の呼びの頭の欄（係の id a77）。
const SUB: &str = r#""agent_id":"a77","agent_type":"tsuzuri:drafter","#;

/// deny の答えの理由の頭（PreToolUse の deny の JSON の頭と、断りの字の頭の `why`）。
fn deny_head(why: &str) -> String {
    format!(
        r#"{{"hookSpecificOutput":{{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"{why}"#
    )
}

/// 何も出さずに rc 0 で通る。
fn passed(out: &Output, what: &str) {
    assert_eq!(
        (out.status.code(), out.stdout.as_slice()),
        (Some(0), &b""[..]),
        "{what} {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// deny の答えで、理由が `why` で始まる。
fn denied(out: &Output, why: &str, what: &str) {
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{what} {text}");
    assert!(text.starts_with(&deny_head(why)), "{what} {text}");
}

/// Bash の呼びの tool_input。
fn bash(command: &str) -> String {
    format!(r#"{{"command":"{command}"}}"#)
}

/// 書きの道具の path の tool_input。
fn write(path: &Path) -> String {
    format!(r#"{{"file_path":"{}","content":"x"}}"#, path.display())
}

/// 組みの検めの command の語は、command の頭の語だけを読む。
#[test]
fn agfen_cargo_word_reads_the_command_word_only() {
    for command in [
        "cargo nextest run",
        "cd x && cargo build",
        "CARGO_TARGET_DIR=/t cargo clippy",
        "timeout 600 cargo test",
        "/h/.cargo/bin/cargo build",
        "(cargo build)",
        "bash -c 'cargo build'",
        "cargo-clippy",
    ] {
        assert!(cargo_word(command).is_some(), "{command}");
    }
    for command in [
        "grep -n cargo Cargo.toml",
        "echo cargo",
        "cat Cargo.toml",
        "tz check --dir /T/design-intent",
        "scribe2 pipe preflight --contract /T/a.toml",
    ] {
        assert_eq!(cargo_word(command), None, "{command}");
    }
}

/// 書きの置き場の検めは、出力の dir と写しの dir の下だけを通す。
#[test]
fn agfen_fenced_lets_writes_under_the_out_and_the_clone_only() {
    let (out, clone) = (Path::new("/D/w218a/w"), Path::new("/D/try-w218a"));
    for (tool, path) in [
        ("Write", "/D/w218a/w/notes.md"),
        ("Edit", "/D/try-w218a/src/a.rs"),
        ("NotebookEdit", "/D/w218a/w/n.ipynb"),
    ] {
        assert!(!fenced(tool, Some(path), out, clone), "{tool} {path}");
    }
    for path in [
        "/D/w219a/w/notes.md",
        "/D/w218a/spec.json",
        "/D/try-w218ax/a.rs",
        "/D/w218a/w/../spec.json",
        "/R/design-intent/a.yaml",
    ] {
        assert!(fenced("Write", Some(path), out, clone), "{path}");
    }
    assert!(fenced("Write", None, out, clone));
    assert!(!fenced("Bash", None, out, clone));
    assert!(!fenced("Read", Some("/R/design-intent/a.yaml"), out, clone));
}

/// 組みが なし の係の Bash の cargo だけを、予算の前でも止める。
#[test]
fn agfen_gate_denies_cargo_for_a_no_build_agent_only() {
    let none = place("none");
    bind(&none, "なし");
    let why = "係の門は止める（頼みの頭の組みが なし の係の Bash で、語 cargo は撃たない）";
    let out = guard(&none, &input(SUB, "Bash", &bash("cargo nextest run")));
    denied(&out, why, "なし の cargo");
    for command in [
        "tz check --dir /T/design-intent",
        "scribe2 pipe preflight --contract /T/a.toml",
        "grep -n cargo Cargo.toml",
    ] {
        let out = guard(&none, &input(SUB, "Bash", &bash(command)));
        passed(&out, command);
    }
    let light = place("light");
    bind(&light, "軽");
    let out = guard(&light, &input(SUB, "Bash", &bash("cargo nextest run")));
    passed(&out, "軽 の cargo");
}

/// 書きの道具は、係の出力の dir と写しの dir の下だけを通し、予算の前でも外を止める。
#[test]
fn agfen_gate_fences_writes_outside_the_out_and_the_clone() {
    let drafts = place("fence");
    bind(&drafts, "なし");
    for path in ["w218a/w/notes.md", "try-w218a/src/a.rs"] {
        let out = guard(&drafts, &input(SUB, "Write", &write(&drafts.join(path))));
        passed(&out, path);
    }
    let why = "係の門は止める（書きの道具 Write の path ";
    for path in [
        "w219a/w/notes.md",
        "w218a/spec.json",
        "try-w218ax/a.rs",
        "g1/unknowns.tsv",
    ] {
        let out = guard(&drafts, &input(SUB, "Write", &write(&drafts.join(path))));
        denied(&out, why, path);
    }
}

/// 係の id の無い席の呼びは、cargo の Bash も外への Write も通し、門は置き場に何も書かない。
#[test]
fn agfen_gate_leaves_the_seat_alone() {
    let drafts = place("seat");
    let outside = Path::new("/etc/outside.txt");
    let out = guard(&drafts, &input("", "Bash", &bash("cargo build")));
    passed(&out, "席の cargo");
    let out = guard(&drafts, &input("", "Write", &write(outside)));
    passed(&out, "席の外への Write");
    assert_eq!(fs::read_dir(&drafts).unwrap().count(), 0);
}
