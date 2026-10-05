//! 会話の印の口の歯（接頭辞 cwstp_・設計ノート surface-wave29b 行 cs-acct-mark・判断の記録 ADR-55 決定 (2)）。
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）に窓の作業場を置き、tz consult stamp に hook の入力の JSON を渡して、
//! 作業場の `.consult/stamps.jsonl` に 1 行ずつ足すことと、窓の作業場でない path と読めない入力では書かずに rc 0 で返すことを見る。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_contract::consult::StampEvent;
use tsuzuri_core::consult::stamp::lines;

const SID: &str = "5e1d0c2a-3b4f-4a6e-9d8c-7b6a5f4e3d21";
const SID2: &str = "772c5b4c-0000-4000-8000-00000000abcd";

/// 歯ごとの置き場（前の撃ちの残りを消して作る）。
fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("cwstp-{name}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// 窓の控えを持つ作業場を作る。
fn workspace(root: &Path, dir: &str) -> PathBuf {
    let ws = root.join(dir);
    fs::create_dir_all(ws.join(".consult")).unwrap();
    fs::write(ws.join(".consult/window.json"), "{}").unwrap();
    ws
}

/// 歯の置き場を cwd にして口を撃つ（相対の path が置き場の作業場を指す）。
fn stamp(cwd: &Path, args: &[&str], payload: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .current_dir(cwd)
        .args(["consult", "stamp"])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(payload.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn payload(event: &str, sid: &str) -> String {
    format!(
        r#"{{"hook_event_name":"{event}","session_id":"{sid}","source":"startup","cwd":"/elsewhere"}}"#
    )
}

fn quiet_ok(o: &Output) {
    assert_eq!(o.status.code(), Some(0));
    assert!(o.stdout.is_empty());
    assert!(
        o.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
}

fn refused_ok(o: &Output) {
    assert_eq!(o.status.code(), Some(0));
    assert!(o.stdout.is_empty());
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.starts_with("tz consult stamp: ") && err.ends_with("（書かない）\n"),
        "{err}"
    );
}

#[test]
fn cwstp_appends_one_line_per_event() {
    let root = place("append");
    let ws = workspace(&root, "consult-cw2");
    let at = ws.display().to_string();
    for (event, sid) in [
        ("SessionStart", SID),
        ("UserPromptSubmit", SID),
        ("Stop", SID),
        ("SessionStart", SID2),
    ] {
        quiet_ok(&stamp(&root, &[&at], &payload(event, sid)));
    }
    let got = lines(&fs::read_to_string(ws.join(".consult/stamps.jsonl")).unwrap());
    let seen: Vec<(StampEvent, &str, Option<&str>)> = got
        .iter()
        .map(|s| (s.event, s.sid.as_str(), s.source.as_deref()))
        .collect();
    assert_eq!(
        seen,
        [
            (StampEvent::Start, SID, Some("startup")),
            (StampEvent::Prompt, SID, None),
            (StampEvent::Stop, SID, None),
            (StampEvent::Start, SID2, Some("startup")),
        ]
    );
    assert!(got.windows(2).all(|w| w[0].at <= w[1].at));
    assert_eq!(fs::read_dir(ws.join(".consult")).unwrap().count(), 2);
}

#[test]
fn cwstp_ignores_foreign_paths_and_payloads() {
    let root = place("foreign");
    let good = workspace(&root, "consult-cw3");
    let other = workspace(&root, "work");
    let named = workspace(&root, "consult-x");
    let bare = root.join("consult-cw4");
    fs::create_dir_all(bare.join(".consult")).unwrap();
    let ok = payload("Stop", SID);
    let s = |p: &Path| p.display().to_string();
    for args in [
        vec![s(&other)],
        vec![s(&named)],
        vec![s(&bare)],
        vec!["consult-cw3".to_string()],
        vec![],
        vec![s(&good), s(&good)],
    ] {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        refused_ok(&stamp(&root, &args, &ok));
    }
    for p in [
        payload("PreToolUse", SID),
        payload("Stop", "x"),
        "{".to_string(),
    ] {
        refused_ok(&stamp(&root, &[&s(&good)], &p));
    }
    for ws in [&good, &other, &named, &bare] {
        assert!(
            !ws.join(".consult/stamps.jsonl").exists(),
            "{}",
            ws.display()
        );
    }
}

/// 対照には 1 行を足し、.consult・印の file・控えの 1 つを外の作業場の同じ物への symlink にした作業場は断りの 1 行で返し、
/// 外の印の file を空のままにし、控えの symlink の作業場に印の file を作らない。
#[test]
fn cwstp_does_not_follow_symlinks() {
    let (root, at) = (place("links"), |w: &Path| w.display().to_string());
    let out = workspace(&root, "outside");
    let (stamps, window) = (".consult/stamps.jsonl", ".consult/window.json");
    fs::write(out.join(stamps), "").unwrap();
    let checks: [fn(&Output); 2] = [refused_ok, quiet_ok];
    for (i, link) in ["", ".consult", stamps, window].into_iter().enumerate() {
        let ws = workspace(&root, &format!("consult-cw{}", i + 1));
        if !link.is_empty() {
            let _ = fs::remove_dir_all(ws.join(link)).or(fs::remove_file(ws.join(link)));
            symlink(out.join(link), ws.join(link)).unwrap();
        }
        checks[usize::from(link.is_empty())](&stamp(&root, &[&at(&ws)], &payload("Stop", SID)));
    }
    let text = fs::read_to_string(root.join("consult-cw1").join(stamps)).unwrap();
    assert_eq!(lines(&text).len(), 1);
    assert_eq!(fs::read_to_string(out.join(stamps)).unwrap(), "");
    assert!(!root.join("consult-cw4").join(stamps).exists());
}
