//! 席の「見て」の知らせの記録を読む server の歯（接頭辞 ntc_・行 i-11 の完了の条件 (1)〜(4)）。
//! 記録は tz stage notify と同じ `notify::save` で書き、口 GET /api/notices と変化の知らせ（SSE）を本物の server で撃つ。
#![cfg(test)]

use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_boundary::stage::notify::{self, EXT, Record};
use tsuzuri_contract::notice::{Notice, Notices, PATH};
use tsuzuri_contract::surface::{BOARD_CHANGED_EVENT, BoardChanged, ChangeKind};
use tsuzuri_contract::wire;

fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// 歯ごとの作業場（名に process の id と時刻を入れる）。
fn place(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ntcsrv")
        .join(format!("{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("作業場");
    dir
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_secs()
}

/// 記録を 1 つ組む。
fn record(at: u64, project: &str, title: &str, url: &str) -> Record {
    Record {
        at,
        project: project.to_string(),
        title: title.to_string(),
        url: url.to_string(),
    }
}

/// dir の下の project の名の記録の file の path。
fn file(dir: &Path, project: &str) -> PathBuf {
    dir.join(format!("{project}{EXT}"))
}

/// 記録を tz stage notify と同じ `save` で書く。
fn save(dir: &Path, r: &Record) {
    notify::save(&file(dir, &r.project), r).unwrap_or_else(|e| panic!("{e}"));
}

/// path の最後の名の列。
fn names(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .expect("名")
                .to_string()
        })
        .collect()
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// repo の dir の名が `project` の server を起こす（台帳の bd は空の一覧を返す偽の program・記録の dir は `notify`）。
fn serve(root: &Path, project: &str, notify: Option<PathBuf>) -> SocketAddr {
    let (repo, files) = (root.join(project), root.join("files"));
    for dir in [repo.join(".beads"), files.clone()] {
        fs::create_dir_all(dir).expect("置き場");
    }
    fs::write(files.join("index.html"), "tz").expect("index.html");
    script(&root.join("bd"), "echo '[]'");
    let config = Config {
        bd: root.join("bd").into(),
        notify,
        ..Config::new(repo, "127.0.0.1:0".parse().expect("bind 先"), files)
    };
    let server = Server::bind(&config).expect("起動");
    let addr = server.local_addr().expect("口の住所");
    thread::spawn(move || server.run());
    addr
}

/// GET を 1 つ撃ち、（状態の code・本文）を返す。
fn get(addr: SocketAddr, path: &str) -> (u16, String) {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    s.write_all(format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes())
        .expect("要求を書く");
    let mut out = Vec::new();
    s.read_to_end(&mut out).expect("応答を読む");
    let text = String::from_utf8(out).expect("応答の字");
    let (head, body) = text.split_once("\r\n\r\n").expect("頭と本文");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|c| c.parse().ok())
        .expect("状態の code");
    (status, body.to_string())
}

/// 口を撃って電文に読む（200 でなければ落ちる）。
fn notices(addr: SocketAddr) -> Notices {
    let (status, body) = get(addr, PATH);
    assert_eq!(status, 200, "{body}");
    wire::decode(&body).unwrap_or_else(|e| panic!("Notices でない {e}: {body}"))
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

/// board-changed の frame の kinds。
fn kinds(frame: &str) -> Vec<ChangeKind> {
    let data = frame
        .lines()
        .find_map(|l| l.strip_prefix("data: "))
        .unwrap_or_else(|| panic!("data が無い: {frame}"));
    wire::decode::<BoardChanged>(data)
        .unwrap_or_else(|e| panic!("BoardChanged でない {e}: {frame}"))
        .kinds
}

/// (1) 記録の dir の読み: dir の表・path・EXT・files・gather。
#[test]
fn ntc_dir_files_gather() {
    let os = |s: &'static str| Some(OsStr::new(s));
    let table: [(Option<&OsStr>, Option<&OsStr>, Option<&str>); 5] = [
        (os("/x/state"), os("/home/u"), Some("/x/state/tsuzuri/notify")),
        (
            os("x/state"),
            os("/home/u"),
            Some("/home/u/.local/state/tsuzuri/notify"),
        ),
        (None, os("/home/u"), Some("/home/u/.local/state/tsuzuri/notify")),
        (os("x/state"), os("u"), None),
        (None, None, None),
    ];
    for (xdg, home, want) in table {
        let got = notify::dir(xdg, home);
        assert_eq!(got, want.map(PathBuf::from), "{xdg:?} {home:?}");
        let path = notify::path(xdg, home, "proj-a");
        assert_eq!(
            path,
            want.map(|d| Path::new(d).join("proj-a.json")),
            "{xdg:?} {home:?}"
        );
    }
    assert_eq!(EXT, ".json");

    let root = place("gather");
    let dir = root.join("notify");
    assert_eq!(notify::files(&dir).expect("無い dir"), Vec::<PathBuf>::new());
    assert_eq!(
        notify::gather(&dir).expect("無い dir"),
        (Vec::new(), Vec::new())
    );
    let (a, b, c) = (
        record(1_790_000_100, "proj-a", "頁 a", "http://srv-a.tailnet.invalid:8120/?page=a"),
        record(1_790_000_200, "proj-b", "頁 b", "http://srv-a.tailnet.invalid:8121/?page=b"),
        record(1_790_000_100, "proj-c", "頁 c", "http://srv-a.tailnet.invalid:8122/?page=c"),
    );
    for r in [&a, &b, &c] {
        save(&dir, r);
    }
    fs::write(
        file(&dir, "proj-d"),
        "{\"at\":1790000300,\"project\":\"proj-d\"}\n",
    )
    .expect("欠けた記録");
    fs::write(
        file(&dir, "proj-e"),
        notify::render(&record(1_790_000_300, "proj-x", "頁 x", "http://x.invalid/")),
    )
    .expect("名の違う記録");
    fs::write(dir.join(".notify.77"), notify::render(&a)).expect("一時の file");
    fs::write(dir.join("proj-g.txt"), notify::render(&a)).expect("拡張子の違う file");
    fs::write(dir.join(".json"), notify::render(&a)).expect("名の無い file");
    fs::create_dir_all(dir.join("proj-h.json")).expect("dir");

    let got = notify::files(&dir).expect("files");
    assert_eq!(
        names(&got),
        ["proj-a.json", "proj-b.json", "proj-c.json", "proj-d.json", "proj-e.json"]
    );
    assert!(got.iter().all(|p| p.parent() == Some(dir.as_path())), "{got:?}");
    let (records, unread) = notify::gather(&dir).expect("gather");
    assert_eq!(records, [b, a, c]);
    assert_eq!(unread, ["proj-d", "proj-e"]);

    // 読めない dir（dir の代わりの file）は Err。
    let flat = root.join("flat");
    fs::write(&flat, "x").expect("file");
    assert!(notify::files(&flat).is_err());
    assert!(notify::gather(&flat).is_err());
    let _ = fs::remove_dir_all(&root);
}

/// (2) 口 GET /api/notices は記録の dir を読むだけで、dir が無ければ空・記録を置けば新しい順・読めない dir は 503。
#[test]
fn ntc_route_reads_records() {
    let root = place("route");
    let dir = root.join("state/tsuzuri/notify");
    let addr = serve(&root, "proj-kiri", Some(dir.clone()));

    let empty = notices(addr);
    assert_eq!(empty.project, "proj-kiri");
    assert!(empty.latest.is_empty() && empty.unread.is_empty(), "{empty:?}");
    assert!(!dir.exists(), "口が記録の dir を作った");

    let kiri = record(
        1_790_000_400,
        "proj-kiri",
        "設計の頁を見て",
        "http://srv-a.tailnet.invalid:8120/?board=proj-kiri",
    );
    let other = record(
        1_790_000_300,
        "proj-b",
        "地図を見て",
        "http://srv-a.tailnet.invalid:8120/?board=proj-b",
    );
    save(&dir, &other);
    save(&dir, &kiri);
    fs::write(file(&dir, "proj-z"), "not json").expect("読めない記録");

    let before = now();
    let got = notices(addr);
    assert!(
        got.at >= before && got.at <= before + 5,
        "at {} は {before} から 5 秒の内でない",
        got.at
    );
    assert_eq!(got.project, "proj-kiri");
    let notice = |r: &Record| Notice {
        at: r.at,
        project: r.project.clone(),
        title: r.title.clone(),
        url: r.url.clone(),
    };
    assert_eq!(got.latest, [notice(&kiri), notice(&other)]);
    assert_eq!(got.unread, ["proj-z"]);
    let mut left: Vec<String> = fs::read_dir(&dir)
        .expect("記録の dir")
        .map(|e| e.expect("項").file_name().to_string_lossy().into_owned())
        .collect();
    left.sort();
    assert_eq!(left, ["proj-b.json", "proj-kiri.json", "proj-z.json"]);

    // 記録の dir の無い server は 503 no-notify-dir・dir の代わりに file が在れば 503 notify-unread。
    let bare = serve(&place("route-none"), "proj-kiri", None);
    assert_eq!(get(bare, PATH), (503, "no-notify-dir".to_string()));
    let flat_root = place("route-flat");
    let flat = flat_root.join("notify");
    fs::write(&flat, "x").expect("file");
    let broken = serve(&flat_root, "proj-kiri", Some(flat));
    assert_eq!(get(broken, PATH), (503, "notify-unread".to_string()));
    let _ = fs::remove_dir_all(&root);
}

/// (3) 変化の知らせは記録を足した時と替えた時に kinds が notice だけの board-changed を送り、一時の file では送らない。
#[test]
fn ntc_board_marks_notice() {
    let root = place("board");
    let dir = root.join("notify");
    let first = record(
        1_790_000_100,
        "proj-a",
        "問いを見て",
        "http://srv-a.tailnet.invalid:8120/?page=ask",
    );
    save(&dir, &first);
    let addr = serve(&root, "proj-kiri", Some(dir.clone()));

    let mut s = TcpStream::connect(addr).expect("接続");
    s.write_all(format!("GET /api/surface/events HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes())
        .expect("要求を書く");
    let mut seen = String::new();
    assert_eq!(
        next_board(&mut s, &mut seen, Duration::from_millis(1500)),
        None,
        "記録が動かないのに知らせる"
    );
    let added = record(
        1_790_000_200,
        "proj-b",
        "地図を見て",
        "http://srv-a.tailnet.invalid:8121/?page=map",
    );
    let replaced = record(
        1_790_000_300,
        "proj-a",
        "設計の頁を見て（替えた）",
        "http://srv-a.tailnet.invalid:8120/?page=map",
    );
    for (what, r) in [("足した", &added), ("替えた", &replaced)] {
        save(&dir, r);
        let frame = next_board(&mut s, &mut seen, Duration::from_secs(5))
            .unwrap_or_else(|| panic!("記録を{what}時の知らせが 5 秒の内に届かない"));
        assert_eq!(kinds(&frame), [ChangeKind::Notice], "{what}: {frame}");
    }
    fs::write(dir.join(".notify.99"), notify::render(&added)).expect("一時の file");
    assert_eq!(
        next_board(&mut s, &mut seen, Duration::from_millis(1500)),
        None,
        "一時の file で知らせる"
    );
    let _ = fs::remove_dir_all(&root);
}

/// (4) 字の見張り: 口の file は記録を書かず子 process を撃たない・server と main と notify の繋ぎの字。
#[test]
fn ntc_wiring_text() {
    let route = read(&manifest("src/server/routes/notices.rs"));
    for word in [
        "fs::write",
        "File::create",
        "OpenOptions",
        "remove_",
        "rename",
        "notify::save",
        "Command",
        "window",
    ] {
        assert!(!route.contains(word), "notices.rs が {word} を含む");
    }
    for word in ["notify::gather(", "cli::project("] {
        assert_eq!(route.matches(word).count(), 1, "notices.rs の {word}");
    }
    let module = read(&manifest("src/server/mod.rs"));
    for word in ["kinded(ChangeKind::Notice,", "map(notify::files)"] {
        assert_eq!(module.matches(word).count(), 1, "mod.rs の {word}");
    }
    let main = read(&manifest("src/main.rs"));
    assert_eq!(main.matches("notify::dir(").count(), 1, "main.rs の notify::dir(");
    let own = read(&manifest("src/stage/notify.rs"));
    for word in ["pub fn dir(", "dir(xdg_state_home, home)?"] {
        assert_eq!(own.matches(word).count(), 1, "notify.rs の {word}");
    }
}
