//! 知らせの種類の歯（接頭辞 evkind_・計画 surface-plan の行 c-ev-kind の完了の条件 (1)〜(4)）。
//! board-changed の data は契約の BoardChanged で、欄 kinds に印の動いた種類を載せる。
//! 印の file は作業場の put の dir に書いてから移すので、見張りから書きかけは見えない。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tsuzuri_boundary::server::design::DESIGN_DIR;
use tsuzuri_boundary::server::events::Hub;
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::surface::{BOARD_CHANGED_EVENT, BoardChanged, ChangeKind};
use tsuzuri_contract::wire;

use ChangeKind::{Account, Design, Runs, Seat};

/// 席の target。
const TARGET: &str = "proj-1:0.1";

/// 見張りの間隔（Hub::watch_board に直に渡す歯）。
const POLL: Duration = Duration::from_millis(20);

fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// 歯ごとの作業場（名に process の id と時刻を入れる・put の dir を持つ）。
fn place(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("evkind")
        .join(format!("{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(dir.join("put")).expect("作業場");
    dir
}

/// 印の file の中身を丸ごと置く（作業場の put の dir に書いてから移す）。
fn put(root: &Path, mark: &Path, text: &str) {
    let tmp = root.join("put").join(mark.file_name().expect("印の名"));
    fs::write(&tmp, text).expect("移す前の印");
    fs::rename(&tmp, mark).expect("印を移す");
}

/// frame の data の字。
fn data(frame: &str) -> &str {
    frame
        .lines()
        .find_map(|l| l.strip_prefix("data: "))
        .unwrap_or_else(|| panic!("data が無い: {frame}"))
}

/// board-changed の frame の kinds（data は欄 at から始まる）。
fn kinds(frame: &str) -> Vec<ChangeKind> {
    assert!(
        frame.contains(&format!("event: {BOARD_CHANGED_EVENT}\n")),
        "{frame}"
    );
    let data = data(frame);
    assert!(data.starts_with("{\"at\":"), "{frame}");
    wire::decode::<BoardChanged>(data)
        .unwrap_or_else(|e| panic!("BoardChanged でない {e}: {frame}"))
        .kinds
}

/// 呼びの回が `target` に届くまで待つ（5 秒で届かなければ落ちる）。
fn wait_calls(calls: &AtomicUsize, target: usize) {
    let until = Instant::now() + Duration::from_secs(5);
    while calls.load(Ordering::SeqCst) < target {
        assert!(Instant::now() < until, "5 秒で呼びが {target} 回に届かない");
        thread::sleep(Duration::from_millis(2));
    }
}

/// 受け手に届いた frame を全部取る。
fn drain(rx: &Receiver<String>) -> Vec<String> {
    let mut out = Vec::new();
    while let Ok(frame) = rx.try_recv() {
        out.push(frame);
    }
    out
}

#[test]
fn evkind_board_sends_moved_kinds() {
    let root = place("moved");
    let [s, r, d, a, both] = ["s", "r", "d", "a", "both"].map(|n| root.join(n));
    for f in [&s, &r, &d, &a, &both] {
        put(&root, f, "0");
    }
    let list = Arc::new(Mutex::new(vec![
        (Seat, s.clone()),
        (Seat, both.clone()),
        (Runs, r.clone()),
        (Design, d.clone()),
        (Account, a.clone()),
        (Account, both.clone()),
    ]));
    let hub = Arc::new(Hub::default());
    let rx = hub.subscribe();
    let calls = Arc::new(AtomicUsize::new(0));
    let (c, l) = (Arc::clone(&calls), Arc::clone(&list));
    let board = move || {
        c.fetch_add(1, Ordering::SeqCst);
        l.lock().expect("lock").clone()
    };
    Hub::watch_board(&hub, board, POLL);
    thread::sleep(Duration::from_millis(100));
    assert!(drain(&rx).is_empty(), "印が動かないのに知らせる");
    for (file, want) in [
        (&s, vec![Seat]),
        (&r, vec![Runs]),
        (&d, vec![Design]),
        (&a, vec![Account]),
        (&both, vec![Seat, Account]),
    ] {
        put(&root, file, "11");
        let frame = rx.recv_timeout(Duration::from_secs(5)).expect("1 件");
        assert_eq!(kinds(&frame), want, "{}: {frame}", file.display());
        thread::sleep(Duration::from_millis(200));
        assert!(drain(&rx).is_empty(), "{}: 印 1 回に 2 件", file.display());
    }
    // 種類をまたいで並びだけを入れ替える（どの種類の印の列も変わらない）。
    list.lock().expect("lock").rotate_left(2);
    let from = calls.load(Ordering::SeqCst);
    wait_calls(&calls, from + 13);
    assert!(drain(&rx).is_empty(), "並びだけの入れ替えで知らせる");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn evkind_frames_exact() {
    let hub = Hub::default();
    let rx = hub.subscribe();
    hub.ledger_changed(7);
    hub.board_changed(&[Seat, Account], 8);
    assert_eq!(
        rx.try_recv().expect("1 件目"),
        "id: 1\nevent: ledger-changed\ndata: {\"at\":7}\n\n"
    );
    assert_eq!(
        rx.try_recv().expect("2 件目"),
        "id: 2\nevent: board-changed\ndata: {\"at\":8,\"kinds\":[\"seat\",\"account\"]}\n\n"
    );
    assert!(rx.try_recv().is_err(), "3 件目が在る");
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// SSE の接続から次の board-changed の frame を `limit` まで待つ（ほかの frame は捨てる）。
fn next_board(s: &mut TcpStream, seen: &mut String, limit: Duration) -> Option<String> {
    let deadline = Instant::now() + limit;
    let needle = format!("event: {BOARD_CHANGED_EVENT}\n");
    let mut buf = [0u8; 4096];
    loop {
        while let Some(end) = seen.find("\n\n") {
            let frame: String = seen.drain(..end + 2).collect();
            if frame.contains(&needle) {
                return Some(frame);
            }
        }
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return None;
        }
        s.set_read_timeout(Some(left.max(Duration::from_millis(10))))
            .expect("timeout");
        match s.read(&mut buf) {
            Ok(0) => return None,
            Ok(n) => seen.push_str(&String::from_utf8_lossy(&buf[..n])),
            Err(_) => {}
        }
    }
}

#[test]
fn evkind_serve_marks_by_kind() {
    let root = place("serve");
    let (repo, files, state) = (root.join("repo"), root.join("files"), root.join("state"));
    let seat_dir = state.join("seat").join(TARGET.replace(':', "_"));
    let design = repo.join(DESIGN_DIR);
    for dir in [
        repo.join(".beads"),
        design.clone(),
        files.clone(),
        state.join("fleet"),
        seat_dir.clone(),
    ] {
        fs::create_dir_all(dir).expect("置き場");
    }
    fs::write(files.join("index.html"), "tz").expect("index.html");
    let (tick, log, events, doc) = (
        seat_dir.join("tick-last"),
        seat_dir.join("state.jsonl"),
        state.join("fleet/events.jsonl"),
        design.join("note.md"),
    );
    for f in [&tick, &log, &events, &doc] {
        put(&root, f, "0\n");
    }
    script(&root.join("bd"), "echo '[]'");
    script(&root.join("scribe2"), "exit 1");
    let config = Config {
        bd: root.join("bd").into(),
        state_dir: Some(state.clone()),
        bdw: root.join("bdw").into(),
        seat: Some(TARGET.to_string()),
        scribe2: root.join("scribe2").into(),
        ..Config::new(
            repo.clone(),
            "127.0.0.1:0".parse().expect("bind 先"),
            files.clone(),
        )
    };
    let server = Server::bind(&config).expect("起動");
    let addr: SocketAddr = server.local_addr().expect("口の住所");
    thread::spawn(move || server.run());

    let mut s = TcpStream::connect(addr).expect("接続");
    s.write_all(format!("GET /api/surface/events HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes())
        .expect("要求を書く");
    let mut seen = String::new();
    assert_eq!(
        next_board(&mut s, &mut seen, Duration::from_millis(1500)),
        None,
        "file が動かないのに知らせる"
    );
    for (file, text, want) in [
        (&tick, "ts=1790510399 decision=nudge\n", vec![Seat]),
        (&log, "{\"schema\":1}\n{\"schema\":1}\n", vec![Seat]),
        (&events, "{\"e\":1}\n{\"e\":2}\n", vec![Runs]),
        (&doc, "# 設計の字を書き換える\n", vec![Design]),
    ] {
        put(&root, file, text);
        let frame = next_board(&mut s, &mut seen, Duration::from_secs(5))
            .unwrap_or_else(|| panic!("{} の変化が 5 秒以内に届かない", file.display()));
        assert_eq!(kinds(&frame), want, "{}: {frame}", file.display());
        assert_eq!(
            next_board(&mut s, &mut seen, Duration::from_millis(1100)),
            None,
            "{}: 印 1 回に 2 件",
            file.display()
        );
    }
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn evkind_wiring_text() {
    let modrs = read(&manifest("src/server/mod.rs"));
    for w in [
        "kinded(ChangeKind::Runs, runs.marks())",
        "kinded(ChangeKind::Design, design.marks())",
        "kinded(ChangeKind::Seat, seat_marks.clone())",
        "kinded(ChangeKind::Account, lock(&held_marks).clone())",
    ] {
        assert_eq!(modrs.matches(w).count(), 1, "mod.rs の {w}");
    }
    let form = read(&manifest("src/server/form.rs"));
    assert_eq!(form.matches("board_changed(").count(), 1, "form.rs の board_changed(");
    assert_eq!(
        form.matches("hub.board_changed(&[ChangeKind::Ledger], now())")
            .count(),
        1,
        "form.rs の台帳の種類の送り"
    );
}
