//! 時間切れの子 process を孫まで止める歯（接頭辞 server_reap_・設計ノート surface-base 便 e-reap の完了の条件）。
//! 歯は temp の置き場に一意の名の dir を作り、sh の script を書いて撃つ。
//! 孫が生きているかは /proc の下の pid の stat で見る（無いか、状態の字が Z なら止まった）。
#![cfg(test)]

use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tsuzuri_boundary::server::ledger::{KILL, capture, stop_with};

/// 止まったかを確かめる間隔と上限。
const POLL: Duration = Duration::from_millis(50);
const LIMIT: Duration = Duration::from_secs(2);

/// 歯ごとの一意の名の dir。
fn place(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("server_reap")
        .join(format!("{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("作業場");
    dir
}

fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("script");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("script の権限");
    path
}

/// pid の process の状態の字（/proc の stat の 3 つ目の欄・stat が無ければ None）。
fn state(pid: u32) -> Option<char> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // 2 つ目の欄（名）は空白や括弧を含みうるので、最後の ')' の後から読む。
    let rest = &stat[stat.rfind(')')? + 1..];
    rest.split_whitespace().next()?.chars().next()
}

fn gone(pid: u32) -> bool {
    matches!(state(pid), None | Some('Z'))
}

/// 50 ミリ秒ずつ 2 秒まで待ち、止まったかを返す。
fn stops_within(pid: u32) -> bool {
    let until = Instant::now() + LIMIT;
    loop {
        if gone(pid) {
            return true;
        }
        if Instant::now() >= until {
            return false;
        }
        thread::sleep(POLL);
    }
}

#[test]
fn server_reap_timeout_stops_grandchild() {
    let dir = place("timeout");
    let pidfile = dir.join("grandchild.pid");
    let sh = script(
        &dir,
        "wrap",
        &format!("sleep 30 &\necho $! > '{}'\nwait", pidfile.display()),
    );
    let started = Instant::now();
    let out = capture(
        sh.as_os_str(),
        [] as [&str; 0],
        &dir,
        Duration::from_millis(300),
    );
    let returned = Instant::now();
    assert_eq!(out, None, "時間切れ");
    assert!(
        returned - started < Duration::from_secs(2),
        "時間切れで返らない: {:?}",
        returned - started
    );
    let pid: u32 = fs::read_to_string(&pidfile)
        .expect("孫の pid の file")
        .trim()
        .parse()
        .expect("孫の pid");
    assert!(stops_within(pid), "孫 {pid} が止まらない: {:?}", state(pid));
    assert!(returned.elapsed() <= LIMIT + POLL * 2);
}

#[test]
fn server_reap_capture_returns_output_on_rc0_only() {
    let dir = place("rc");
    let ok = script(&dir, "ok", "echo ok\nexit 0");
    let bad = script(&dir, "bad", "echo ok\nexit 1");
    let timeout = Duration::from_secs(5);
    assert_eq!(
        capture(ok.as_os_str(), [] as [&str; 0], &dir, timeout).as_deref(),
        Some(&b"ok\n"[..])
    );
    assert_eq!(
        capture(bad.as_os_str(), [] as [&str; 0], &dir, timeout),
        None
    );
}

/// 新しい process group に入れて sh の sleep 30 を撃つ。
fn sleeper() -> std::process::Child {
    Command::new("sh")
        .args(["-c", "sleep 30"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .expect("sh の sleep")
}

#[test]
fn server_reap_stop_with_falls_back_without_tool() {
    let child = sleeper();
    let pid = child.id();
    let dir = place("fallback");
    let missing = dir.join("no-such-kill");
    assert!(!stop_with(child, missing.as_os_str()), "在らない道具");
    assert_eq!(state(pid), None, "子 {pid} が片付いていない");
    // 子だけを止めたので、sh が残した孫が在れば歯の後始末で止める。
    let _ = Command::new(KILL)
        .args(["-KILL", "--", &format!("-{pid}")])
        .stderr(Stdio::null())
        .status();

    let child = sleeper();
    let pid = child.id();
    assert!(
        stop_with(child, OsStr::new(KILL)),
        "道具 kill で group へ送る"
    );
    assert_eq!(state(pid), None, "子 {pid} が片付いていない");
}
