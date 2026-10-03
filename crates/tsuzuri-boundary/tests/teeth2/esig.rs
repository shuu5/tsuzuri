//! 問いの合図で台帳の見張りの周期の待ちを終わらせる歯（接頭辞 esig_・設計ノート surface-wave21d 行 e-signal の完了の条件・
//! 要件 NFR2・規則の行 R-21）。作業場は CARGO_TARGET_TMPDIR の下に歯ごとに作る。server の歯の偽の bd は撃たれるたびに
//! 記録の file に 1 行を足してから台帳の file を cat する script で、server は同じ process の thread で 127.0.0.1 の空き port に立てる。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::server::events::{Hub, NUDGE_PATH, NUDGED, Timing};
use tsuzuri_boundary::server::route::{Key, Match};
use tsuzuri_boundary::server::{Config, Route, Server};

/// 合図から知らせまでの上限（規則の行 R-21）。
const WITHIN: Duration = Duration::from_millis(200);

/// 知らせも読みも無いことを見る間。
const QUIET: Duration = Duration::from_millis(300);

/// 歯ごとの作業場（前の歯の残りを消して作り直す）。
fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("esig").join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("作業場");
    root
}

/// file の中身を丸ごと替える（隣の file に書いてから rename で移すので、見張りから書きかけは見えない）。
fn put(path: &Path, text: &str) {
    let name = path.file_name().expect("file の名").to_string_lossy();
    let tmp = path.with_file_name(format!("{name}.tmp"));
    fs::write(&tmp, text).expect("隣の file");
    fs::rename(&tmp, path).expect("移す");
}

#[test]
fn esig_route_key_and_words() {
    assert_eq!(NUDGE_PATH, "/api/surface/questions");
    assert_eq!(NUDGED, "nudged");
    let route = Route::ALL
        .iter()
        .find(|r| r.name() == "nudge")
        .expect("名が nudge の口が無い");
    let key = route.key();
    assert_eq!(
        key,
        Key {
            method: "POST",
            path: Match::Exact(NUDGE_PATH),
        }
    );
    assert!(!key.matches("GET", NUDGE_PATH), "GET に当たる");
    let posts = Route::ALL
        .iter()
        .filter(|r| r.key().matches("POST", NUDGE_PATH))
        .count();
    assert_eq!(posts, 1, "POST {NUDGE_PATH} に当たる口の数");
}

#[test]
fn esig_nudge_ends_the_wait() {
    let root = place("wait");
    let mark = root.join("mark");
    put(&mark, "1");
    let content = Arc::new(Mutex::new(0u32));
    let reads = Arc::new(AtomicUsize::new(0));
    let (c, r) = (Arc::clone(&content), Arc::clone(&reads));
    let read = move || {
        r.fetch_add(1, Ordering::SeqCst);
        *c.lock().expect("lock")
    };
    let timing = Timing {
        poll: Duration::from_secs(60),
        reread: Duration::from_secs(60),
        store_reread: Duration::from_secs(60),
    };
    let hub = Hub::watch(vec![mark.clone()], read, timing);
    let n = || reads.load(Ordering::SeqCst);
    assert_eq!(n(), 1, "最初の読みは戻る前");
    let rx = hub.subscribe();
    // 周期が 60 秒なので、印と値が動いても合図が無ければ待ちは終わらない。
    put(&mark, "22");
    *content.lock().expect("lock") = 1;
    thread::sleep(QUIET);
    assert!(rx.try_recv().is_err(), "合図の前に知らせる");
    assert_eq!(n(), 1, "合図の前に読む");
    let at = Instant::now();
    assert!(hub.nudge(), "見張りの在る Hub の合図が偽");
    let frame = rx.recv_timeout(WITHIN).expect("合図から 200 ミリ秒以内の知らせ");
    assert!(at.elapsed() <= WITHIN, "{:?}", at.elapsed());
    assert!(frame.contains("event: ledger-changed\n"), "{frame}");
    assert_eq!(n(), 2, "合図の周の読み");
    // 印が動かなければ、合図が何回来ても読まない。
    *content.lock().expect("lock") = 2;
    for _ in 0..20 {
        assert!(hub.nudge(), "合図が偽");
    }
    thread::sleep(QUIET);
    assert!(rx.try_recv().is_err(), "印が動かないのに知らせる");
    assert_eq!(n(), 2, "印が動かないのに読む");
    nudge_after_mark_moves(root, mark, hub, n, rx);
}

/// 印が動いた後の合図で知らせが 1 件だけ届き、見張りの無い Hub の合図は偽。
fn nudge_after_mark_moves(
    root: PathBuf,
    mark: PathBuf,
    hub: Arc<Hub>,
    n: impl Fn() -> usize,
    rx: tsuzuri_boundary::server::events::Subscription,
) {
    // 印が動いた後の合図でまた 1 件。
    put(&mark, "333");
    let at = Instant::now();
    assert!(hub.nudge(), "合図が偽");
    let frame = rx.recv_timeout(WITHIN).expect("合図から 200 ミリ秒以内の知らせ");
    assert!(at.elapsed() <= WITHIN, "{:?}", at.elapsed());
    assert!(frame.contains("event: ledger-changed\n"), "{frame}");
    thread::sleep(QUIET);
    assert!(rx.try_recv().is_err(), "合図 1 回に 2 件");
    assert_eq!(n(), 3, "印の動いた合図の周の読み");
    assert!(!Hub::default().nudge(), "見張りの無い Hub の合図が真");
    let _ = fs::remove_dir_all(&root);
}

/// 偽の bd の記録の file の行の数。
fn calls(root: &Path) -> usize {
    fs::read_to_string(root.join("calls"))
        .unwrap_or_default()
        .lines()
        .count()
}

/// 台帳の file の字（bead を `n` 本）。
fn ledger(n: u32) -> String {
    let beads: Vec<String> = (1..=n)
        .map(|i| {
            format!(
                "{{\"id\":\"fx-n.{i}\",\"title\":\"合図の問い {i}\",\"status\":\"open\",\"updated_at\":\"2026-09-29T00:00:0{i}Z\"}}"
            )
        })
        .collect();
    format!("[{}]\n", beads.join(","))
}

/// 台帳の file と印の issues.jsonl を移しで替える（台帳が先・印が後）。
fn swap(root: &Path, issues: &Path, n: u32) {
    put(&root.join("ledger"), &ledger(n));
    put(issues, &"{}\n".repeat(n as usize + 1));
}

/// SSE の接続から次の ledger-changed の frame を `limit` まで待ち、届いた時を返す（ほかの frame は捨てる）。
fn next_ledger(s: &mut TcpStream, seen: &mut String, limit: Duration) -> Option<Instant> {
    let deadline = Instant::now() + limit;
    let mut buf = [0u8; 4096];
    loop {
        while let Some(end) = seen.find("\n\n") {
            let frame: String = seen.drain(..end + 2).collect();
            if frame.contains("event: ledger-changed\n") {
                return Some(Instant::now());
            }
        }
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return None;
        }
        s.set_read_timeout(Some(left.max(Duration::from_millis(5))))
            .expect("timeout");
        match s.read(&mut buf) {
            Ok(0) => return None,
            Ok(n) => seen.push_str(&String::from_utf8_lossy(&buf[..n])),
            Err(_) => {}
        }
    }
}

/// 合図の口に POST を撃ち、状態と本文を返す（`origin` が在れば頭 Origin を付ける）。
fn post(addr: SocketAddr, body: &str, origin: Option<&str>) -> (u16, String) {
    let mut s = TcpStream::connect(addr).expect("接続");
    let origin = origin.map_or(String::new(), |o| format!("Origin: {o}\r\n"));
    s.write_all(
        format!(
            "POST {NUDGE_PATH} HTTP/1.1\r\nHost: x\r\n{origin}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .as_bytes(),
    )
    .expect("要求");
    s.set_read_timeout(Some(Duration::from_secs(5)))
        .expect("timeout");
    let mut raw = Vec::new();
    s.read_to_end(&mut raw).expect("応答");
    let text = String::from_utf8_lossy(&raw).into_owned();
    let (head, rest) = text.split_once("\r\n\r\n").expect("応答の頭");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|c| c.parse().ok())
        .unwrap_or_else(|| panic!("状態の行: {head}"));
    (status, rest.to_string())
}

#[test]
fn esig_post_wakes_the_server() {
    let root = place("post");
    let (repo, files) = (root.join("repo"), root.join("files"));
    fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
    fs::create_dir_all(&files).expect("面の file の置き場");
    fs::write(files.join("index.html"), "tz").expect("index.html");
    let issues = repo.join(".beads").join("issues.jsonl");
    swap(&root, &issues, 0);
    let bd = root.join("bd");
    fs::write(
        &bd,
        format!(
            "#!/bin/sh\necho x >> '{0}/calls'\ncat '{0}/ledger'\n",
            root.display()
        ),
    )
    .expect("偽の bd");
    fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("偽の bd の権限");
    let config = Config {
        bd: bd.into(),
        ..Config::new(
            repo.clone(),
            "127.0.0.1:0".parse().expect("bind 先"),
            files.clone(),
        )
    };
    let server = Server::bind(&config).expect("起動");
    let addr = server.local_addr().expect("口の住所");
    thread::spawn(move || server.run());
    assert_eq!(calls(&root), 1, "起動の読み");
    let mut s = TcpStream::connect(addr).expect("接続");
    s.write_all(
        b"GET /api/surface/events HTTP/1.1\r\nHost: x\r\nAccept: text/event-stream\r\n\r\n",
    )
    .expect("要求");
    let mut seen = String::new();
    // 受け手が付いた周の読み（字は同じなので知らせない）を待つ。
    let until = Instant::now() + Duration::from_secs(2);
    while calls(&root) < 2 {
        assert!(Instant::now() < until, "受け手が付いて 2 秒以内に読まない");
        thread::sleep(Duration::from_millis(5));
    }
    // 周期の読みの知らせ。届いた時の直後が見張りの周期の頭。
    swap(&root, &issues, 1);
    next_ledger(&mut s, &mut seen, Duration::from_secs(2)).expect("周期の読みの知らせ");
    thread::sleep(Duration::from_millis(50));
    // 周期の読みなら約 450 ミリ秒の後なので、200 ミリ秒以内の知らせは合図の道でしか届かない。
    swap(&root, &issues, 2);
    let at = Instant::now();
    let reply = post(addr, "{\"question\":\"fx-n.2\"}", None);
    assert_eq!(reply, (202, NUDGED.to_string()), "合図の応答");
    let got = next_ledger(&mut s, &mut seen, WITHIN).expect("合図から 200 ミリ秒以内の知らせ");
    assert!(got - at <= WITHIN, "{:?}", got - at);
    nudge_without_reading(root, addr, s);
}

/// 印が動かない合図と断る合図は見張りを起こさず、bd を撃たない。
fn nudge_without_reading(root: PathBuf, addr: SocketAddr, s: TcpStream) {
    // 印が動かなければ、合図を受けても bd を撃たない。
    let before = calls(&root);
    for _ in 0..3 {
        let reply = post(addr, "{\"question\":\"fx-n.2\"}", None);
        assert_eq!(reply, (202, NUDGED.to_string()), "合図の応答");
    }
    // 読めない本文と別の住所の Origin は断り、見張りを起こさない。
    for (body, origin, want) in [
        ("{}", None, (400, "bad-body")),
        ("{\"question\":\"fx n.2\"}", None, (400, "bad-body")),
        ("not json", None, (400, "bad-body")),
        (
            "{\"question\":\"fx-n.2\"}",
            Some("http://elsewhere.example"),
            (403, "origin"),
        ),
    ] {
        let reply = post(addr, body, origin);
        assert_eq!(reply, (want.0, want.1.to_string()), "{body} {origin:?}");
    }
    thread::sleep(QUIET);
    assert_eq!(calls(&root), before, "印が動かない合図か断りの後に bd を撃つ");
    drop(s);
    let _ = fs::remove_dir_all(&root);
}
