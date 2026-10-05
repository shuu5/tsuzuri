//! 行 c-cases-watch の歯（接頭辞 bvlcw_）: 器の局面の出力（state dir の fleet/lifecycle.json と lifecycle.stale）の見張り。
//! 面が読む中身（古さの印の種類と部品）が動いた周だけ局面の出力の種類の board-changed を 1 件送り、生成の時刻と入力の
//! 印だけが動く書き直しでは送らない。印の file は作業場の put の dir に書いてから移す（器の書き手と同じ置き替え）。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::common::{manifest, script};
use tsuzuri_boundary::server::cases::Cases;
use tsuzuri_boundary::server::events::Hub;
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::surface::{BOARD_CHANGED_EVENT, BoardChanged, ChangeKind};
use tsuzuri_contract::wire;

/// 見張りの間隔（`Cases::watch` に直に渡す歯）。
const POLL: Duration = Duration::from_millis(20);

/// 歯ごとの作業場（名に process の id と時刻を入れる・put の dir と state dir の fleet の dir を持つ）。
fn place(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("bvlcw")
        .join(format!("{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(dir.join("put")).expect("作業場");
    fs::create_dir_all(dir.join("state/fleet")).expect("fleet の dir");
    dir
}

/// file の中身を丸ごと置く（作業場の put の dir に書いてから移す）。
fn put(root: &Path, file: &Path, text: &str) {
    let tmp = root.join("put").join(file.file_name().expect("file の名"));
    fs::write(&tmp, text).expect("移す前の file");
    fs::rename(&tmp, file).expect("file を移す");
}

/// 局面の出力の字（部品 1 つ・`generated` は生成の時刻・`len` は入力の event log の長さ）。
fn output(generated: &str, len: u64, phase: &str) -> String {
    format!(
        r#"{{"version":1,"generated_at":"{generated}","scope":"partial","inputs":{{"events":{{"len":{len}}}}},"parts":[{{"part":"contract","id":"fx-w.1","phase":"{phase}","turn":"vessel","since":"2026-10-02T00:00:00Z","reason":"dependency","closed":false,"links":{{"on":["fx-w.2"],"runs":[]}}}}]}}"#
    )
}

/// 古さの印の字（種類 1 つ）。
fn stale(kind: &str) -> String {
    format!(r#"{{"version":1,"marks":[{{"kind":"{kind}","at":"2026-10-02T00:00:00Z"}}]}}"#)
}

/// board-changed の frame の kinds。
fn kinds(frame: &str) -> Vec<ChangeKind> {
    assert!(
        frame.contains(&format!("event: {BOARD_CHANGED_EVENT}\n")),
        "{frame}"
    );
    let data = frame
        .lines()
        .find_map(|l| l.strip_prefix("data: "))
        .unwrap_or_else(|| panic!("data が無い: {frame}"));
    wire::decode::<BoardChanged>(data)
        .unwrap_or_else(|e| panic!("BoardChanged でない {e}: {frame}"))
        .kinds
}

/// 受け手に届いた frame を全部取る。
fn drain(rx: &Receiver<String>) -> Vec<String> {
    let mut out = Vec::new();
    while let Ok(frame) = rx.try_recv() {
        out.push(frame);
    }
    out
}

/// 次の frame を 5 秒まで待ち、種類が局面の出力だけで、その後 10 周の間に 2 件目が来ないことを見る。
fn one(rx: &Receiver<String>, what: &str) {
    let frame = rx
        .recv_timeout(Duration::from_secs(5))
        .unwrap_or_else(|_| panic!("{what}: 5 秒以内に届かない"));
    assert_eq!(kinds(&frame), [ChangeKind::Cases], "{what}");
    thread::sleep(POLL * 10);
    assert_eq!(drain(rx), Vec::<String>::new(), "{what}: 1 回の変化に 2 件");
}

/// (2) 中身が動いた周だけ局面の出力の種類の board-changed を 1 件送り、生成の時刻と入力の印だけが動く書き直しでは
/// 送らない（部品の局面・古さの印の出入り・読めない出力とその戻り）。state dir を省いた読みは見張らない。
#[test]
fn bvlcw_watch_sends_on_sight_change() {
    let root = place("sight");
    let state = root.join("state");
    let json = state.join("fleet/lifecycle.json");
    put(
        &root,
        &json,
        &output("2026-10-02T00:00:00Z", 10, "contract-queued"),
    );
    let cases = Cases::new(Some(&state));
    let hub = Arc::new(Hub::default());
    let rx = hub.subscribe();
    cases.watch(&hub, POLL);
    assert_eq!(Arc::weak_count(&hub), 1, "見張りが Hub を弱く持つ");
    thread::sleep(POLL * 10);
    assert_eq!(drain(&rx), Vec::<String>::new(), "file が動かないのに送る");
    same_sight_rewrites(&root, &json, &rx);
    sight_changes(&root, &state, &rx);
    let bare = Arc::new(Hub::default());
    Cases::new(None).watch(&bare, POLL);
    assert_eq!(
        Arc::weak_count(&bare),
        0,
        "state dir を省いた読みは見張らない"
    );
    drop(hub);
    let _ = fs::remove_dir_all(&root);
}

/// 生成の時刻と入力の印だけが動く書き直し（更新時刻は動く）3 回では送らない。
fn same_sight_rewrites(root: &Path, json: &Path, rx: &Receiver<String>) {
    for (len, at) in [
        (11, "2026-10-02T00:00:15Z"),
        (12, "2026-10-02T00:00:30Z"),
        (13, "2026-10-02T00:05:00Z"),
    ] {
        put(root, json, &output(at, len, "contract-queued"));
        thread::sleep(POLL * 10);
        assert_eq!(
            drain(rx),
            Vec::<String>::new(),
            "中身の変わらない書き直し {at}"
        );
    }
}

/// 部品の局面・古さの印の出入り・読めない出力とその戻り・出力が消える周と、出力が無い周から読めない出力への行き来
/// （どちらも部品は Unknown で、読めない版かだけが動く）で 1 件ずつ送る。
fn sight_changes(root: &Path, state: &Path, rx: &Receiver<String>) {
    let (json, marks) = (
        state.join("fleet/lifecycle.json"),
        state.join("fleet/lifecycle.stale"),
    );
    put(
        root,
        &json,
        &output("2026-10-02T00:06:00Z", 20, "contract-running"),
    );
    one(rx, "部品の局面");
    put(root, &marks, &stale("ledger"));
    one(rx, "古さの印が来る");
    fs::remove_file(&marks).expect("古さの印を消す");
    one(rx, "古さの印が去る");
    put(root, &json, "not json");
    one(rx, "読めない出力");
    put(
        root,
        &json,
        &output("2026-10-02T00:07:00Z", 21, "contract-running"),
    );
    one(rx, "読める出力に戻る");
    fs::remove_file(&json).expect("出力を消す");
    one(rx, "出力が消える");
    put(root, &json, "not json");
    one(rx, "出力が無い周から読めない出力");
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

/// (3) server の知らせの口（GET /api/surface/events）は、state dir の局面の出力が現れた時と中身が動いた時に種類 cases
/// だけを載せた board-changed を 1 件送り、生成の時刻だけが動く書き直しでは送らない。
#[test]
fn bvlcw_serve_sends_cases_kind() {
    let root = place("serve");
    let (repo, files, state) = (root.join("repo"), root.join("files"), root.join("state"));
    for dir in [repo.join(".beads"), files.clone()] {
        fs::create_dir_all(dir).expect("置き場");
    }
    fs::write(files.join("index.html"), "tz").expect("index.html");
    script(&root.join("bd"), "echo '[]'");
    let config = Config {
        bd: root.join("bd").into(),
        state_dir: Some(state.clone()),
        bdw: root.join("bdw").into(),
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
    let json = state.join("fleet/lifecycle.json");
    assert_eq!(
        next_board(&mut s, &mut seen, Duration::from_millis(1500)),
        None,
        "file が動かないのに知らせる"
    );
    for (text, want) in [
        (output("2026-10-02T00:00:00Z", 10, "contract-queued"), true),
        (output("2026-10-02T00:00:15Z", 11, "contract-queued"), false),
        (output("2026-10-02T00:00:30Z", 12, "run-intake"), true),
    ] {
        put(&root, &json, &text);
        let frame = next_board(
            &mut s,
            &mut seen,
            Duration::from_secs(if want { 5 } else { 2 }),
        );
        match (frame, want) {
            (Some(frame), true) => assert_eq!(kinds(&frame), [ChangeKind::Cases], "{frame}"),
            (None, false) => {}
            (got, _) => panic!("{text}: 送るか {want} で {got:?}"),
        }
        assert_eq!(
            next_board(&mut s, &mut seen, Duration::from_millis(1100)),
            None,
            "{text}: 1 回の変化に 2 件"
        );
    }
    let _ = fs::remove_dir_all(&root);
}

/// mod.rs の impl の中の関数の字（頭から 4 字下げの閉じの前まで）。
fn fn_of<'a>(src: &'a str, head: &str) -> &'a str {
    let at = src
        .find(head)
        .unwrap_or_else(|| panic!("mod.rs に {head} が無い"));
    let rest = &src[at..];
    &rest[..rest.find("\n    }\n").expect("関数の閉じ")]
}

/// bind_with は局面の出力の読みを作ってから start_hub に貸して、Shared の欄 cases にはその束縛そのものを移す。
fn bind_shares_one(modrs: &str) {
    let bind = fn_of(modrs, "pub fn bind_with(");
    let make = "let cases = Cases::new(config.state_dir.as_deref());";
    let pass = "let hub = Server::start_hub(&sources, &cases, seat_marks, held_marks, notify_dir);";
    let (Some(m), Some(p)) = (bind.find(make), bind.find(pass)) else {
        panic!("bind_with に {make} と {pass} が無い");
    };
    assert!(
        m < p,
        "bind_with は局面の出力の読みを作ってから start_hub に渡す"
    );
    let shared = &bind[bind
        .find("shared: Arc::new(Shared {")
        .expect("Shared の組み")..];
    let shared = &shared[..shared
        .find("\n            }),\n")
        .expect("Shared の組みの閉じ")];
    let fields: Vec<&str> = shared
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("cases"))
        .collect();
    assert_eq!(
        fields,
        ["cases,"],
        "Shared の局面の出力の読みは bind の束縛そのもの"
    );
}

/// start_hub は局面の出力の読みを借り、板の印の見張りを始めた後に、本文の一番上の段（枝や閉包の中でない
/// 8 字下げの 1 行）で局面の出力の見張りを始め、見張りを掛けた Hub を返す。
fn start_hub_watches(modrs: &str) {
    let hub = fn_of(modrs, "fn start_hub(");
    assert!(
        hub.contains("\n        cases: &Cases,\n"),
        "start_hub は局面の出力の読みを借りる"
    );
    let lines: Vec<&str> = hub.lines().collect();
    let start = lines
        .iter()
        .position(|l| l.starts_with("        let hub = Hub::start("));
    let watch = lines
        .iter()
        .position(|l| *l == "        cases.watch(&hub, events::POLL);");
    let (Some(start), Some(watch)) = (start, watch) else {
        panic!("start_hub の本文の一番上の段に板の印の見張りと局面の出力の見張りが無い");
    };
    assert!(start < watch, "局面の出力の見張りは板の印の見張りの後");
    assert_eq!(
        lines.last(),
        Some(&"        hub"),
        "start_hub は見張りを掛けた Hub を返す"
    );
}

/// (4) bind は局面の出力の読みを 1 つだけ作り、同じ値を start_hub と Shared に渡す。start_hub は見張りを板の印と
/// 同じ間隔で、その本文の一番上の段で始める（bind や枝の中では始めない）。送る種類は局面の出力の種類だけ。
#[test]
fn bvlcw_wiring_text() {
    let modrs = fs::read_to_string(manifest("src/server/mod.rs")).expect("mod.rs を読む");
    for (want, n) in [("Cases::new(", 1), ("cases.watch(", 1), ("start_hub(", 2)] {
        assert_eq!(modrs.matches(want).count(), n, "mod.rs の {want} の数");
    }
    bind_shares_one(&modrs);
    start_hub_watches(&modrs);
    let casesrs = fs::read_to_string(manifest("src/server/cases.rs")).expect("cases.rs を読む");
    assert_eq!(
        casesrs.matches("board_changed(").count(),
        1,
        "cases.rs の board_changed("
    );
    assert_eq!(
        casesrs
            .matches("hub.board_changed(&[ChangeKind::Cases], now());")
            .count(),
        1,
        "cases.rs の局面の出力の種類の送り"
    );
}
