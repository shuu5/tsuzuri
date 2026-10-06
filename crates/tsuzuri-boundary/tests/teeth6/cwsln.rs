//! 窓の状態の 1 行の口の歯（接頭辞 cwsln_・設計ノート surface-wave29b 行 cs-status-verb・判断の記録 ADR-57 決定 (1)(コ)・(4)）。
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）に窓の作業場を置き、tz consult statusline に session の JSON を渡して、
//! 1 行の字と、symlink を辿らないこと・fifo を開かないこと・制御の字を出さないこと・窓の作業場でない path を断ることを見る。
//! 口座は statusLine の命令が何も出さない偽の置き場にする（口座の行は出ず 1 行だけ・行 cs-status-seat）。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use tsuzuri_contract::consult::{Form, Starter, WindowFile, WindowId};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::status::SEP;

const INPUT: &str = r#"{"context_window":{"used_percentage":42.4}}"#;

/// 歯ごとの置き場（前の撃ちの残りを消して作る）。
fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("cwsln-{name}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn window_json(topic: &str) -> String {
    let w = WindowFile {
        id: WindowId::parse("cw3").unwrap(),
        form: Form::Talk,
        topic: Some(topic.into()),
        model: "fable".into(),
        effort: "xhigh".into(),
        starter: Starter::Chat,
        uttered: Some("20261004T2207Z".into()),
        request: None,
        made: "20261004T2207Z".into(),
    };
    wire::encode(&w).unwrap()
}

/// 控え・所見 2 つ・束の要約値を持つ作業場 `consult-cw3` を作る。
fn workspace(root: &Path) -> PathBuf {
    let ws = root.join("consult-cw3");
    for dir in [".consult", "findings", "bundle"] {
        fs::create_dir_all(ws.join(dir)).unwrap();
    }
    fs::write(ws.join(".consult/window.json"), window_json("t3-hub.78")).unwrap();
    fs::write(ws.join("findings/cw3-1.json"), "{}").unwrap();
    fs::write(ws.join("findings/cw3-2.json"), "{}").unwrap();
    fs::write(ws.join("findings/notes.txt"), "x").unwrap();
    fs::write(ws.join("bundle/digest"), "3766ac7797cdf78f\n").unwrap();
    ws
}

/// statusLine の命令が何も出さない口座の置き場（歯の process ごとの名で書いて rename で置く・並んだ歯が半端な字を読まない）。
fn quiet_account() -> PathBuf {
    let acct = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cwsln-quiet");
    fs::create_dir_all(&acct).unwrap();
    let staged = acct.join(format!("settings.{}", std::process::id()));
    fs::write(
        &staged,
        r#"{"statusLine":{"type":"command","command":"true"}}"#,
    )
    .unwrap();
    fs::rename(&staged, acct.join("settings.json")).unwrap();
    acct
}

fn statusline(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .current_dir(env!("CARGO_TARGET_TMPDIR"))
        .env("CLAUDE_CONFIG_DIR", quiet_account())
        .env_remove("TZ_CONSULT_STATUSLINE")
        .args(["consult", "statusline"])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // 断る口は標準入力を読まずに終わるので、書きの誤り（壊れた pipe）は見ない。
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    child.wait_with_output().unwrap()
}

fn line(ws: &Path) -> String {
    let out = statusline(&[ws.to_str().unwrap()], INPUT);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .unwrap()
        .trim_end()
        .to_string()
}

#[test]
fn cwsln_prints_the_line() {
    let ws = workspace(&place("line"));
    let want = [
        "窓 cw3",
        "題 t3-hub.78",
        "fable・xhigh",
        "所見 2",
        "束 3766ac77",
        "文脈 42%",
    ];
    assert_eq!(line(&ws), want.join(SEP));
}

/// 控え・束の dir・束の file・所見の dir・所見の file のどれかを外の物への symlink に替えると、その欄を出さない。
#[test]
fn cwsln_does_not_follow_symlinks() {
    let root = place("links");
    let outside = root.join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("window.json"), window_json("SECRET")).unwrap();
    fs::write(outside.join("digest"), "5ec2e7aa5ec2e7aa").unwrap();
    fs::write(outside.join("x.json"), "{}").unwrap();
    let cases: [(&str, &str, &str, &str); 5] = [
        (".consult/window.json", "window.json", "窓 ?", "SECRET"),
        ("bundle", "", "所見 2", "5ec2e7aa"),
        ("bundle/digest", "digest", "所見 2", "5ec2e7aa"),
        ("findings", "", "fable・xhigh", "所見"),
        ("findings/cw3-2.json", "x.json", "所見 1", "所見 2"),
    ];
    for (at, target, has, hidden) in cases {
        let ws = workspace(&root.join(at.replace('/', "-")));
        let path = ws.join(at);
        if path.is_dir() {
            fs::remove_dir_all(&path).unwrap();
        } else {
            fs::remove_file(&path).unwrap();
        }
        symlink(outside.join(target), &path).unwrap();
        let got = line(&ws);
        assert!(got.contains(has) && !got.contains(hidden), "{at}: {got}");
    }
}

/// 束の要約値が fifo なら開かずにすぐ返し、その欄を出さない。
#[test]
fn cwsln_skips_fifo() {
    let ws = workspace(&place("fifo"));
    let digest = ws.join("bundle/digest");
    fs::remove_file(&digest).unwrap();
    let made = Command::new("mkfifo").arg(&digest).status().unwrap();
    assert!(made.success());
    let start = Instant::now();
    let got = line(&ws);
    assert!(
        start.elapsed() < Duration::from_secs(2),
        "{:?}",
        start.elapsed()
    );
    assert_eq!(
        got,
        [
            "窓 cw3",
            "題 t3-hub.78",
            "fable・xhigh",
            "所見 2",
            "文脈 42%"
        ]
        .join(SEP)
    );
}

/// 窓が控えの題に端末を操る字を書いても、出す字に制御の字は無い。
#[test]
fn cwsln_drops_control_chars() {
    let ws = workspace(&place("ctl"));
    let evil = "\u{1b}]52;c;QUFB\u{7}題\u{9b}2J";
    fs::write(ws.join(".consult/window.json"), window_json(evil)).unwrap();
    let out = statusline(&[ws.to_str().unwrap()], INPUT);
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.trim_end().contains("題 ]52;c;QUFB題2J"), "{text:?}");
    assert!(!text.trim_end().chars().any(char::is_control), "{text:?}");
}

/// 相対の path（cwd からは在る作業場を指す）・名が窓の作業場でない path・symlink の作業場・引数の数の違いは、何も出さずに rc 1。
#[test]
fn cwsln_refuses_non_workspace() {
    let root = place("bad");
    let ws = workspace(&root);
    let other = root.join("notes");
    fs::create_dir_all(&other).unwrap();
    let link = root.join("consult-cw4");
    symlink(&ws, &link).unwrap();
    let (ws, other, link) = (
        ws.to_str().unwrap(),
        other.to_str().unwrap(),
        link.to_str().unwrap(),
    );
    let cases: [&[&str]; 5] = [
        &["cwsln-bad/consult-cw3"],
        &[other],
        &[link],
        &[],
        &[ws, ws],
    ];
    for args in cases {
        let out = statusline(args, INPUT);
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        assert!(out.stdout.is_empty(), "{args:?}");
    }
}
