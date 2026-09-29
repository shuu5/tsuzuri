//! 最小の server の歯（接頭辞 server_min_・設計ノート surface-base 便 e-min の完了の条件）。
//! server は同じ process の thread で 127.0.0.1 の空き port に立て、要求は素の TCP で撃つ。
//! 台帳は偽の bd（作業場の out.json を返す shell の script・便 e-src）が返す。

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::server::{Config, Server, StartError, bind_allowed};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadId, LedgerChanged, LedgerItem, LedgerList};
use tsuzuri_contract::wire;

const INDEX: &str = "<!doctype html><title>tz</title>";
const SECRET: &str = "置き場の外の秘密";

/// 歯ごとの作業場（repo の置き場と面の file の置き場と、置き場の外の file と偽の bd）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
}

/// bead 5 本の fixture（bd の 1 本 1 行の形）。
fn fixture() -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/ledger/min-5.jsonl"),
    )
    .expect("fixture")
}

/// 1 本 1 行の字を bd の読み取りの口の出力の形（JSON の配列）にする。
fn bd_array(lines: &str) -> String {
    let lines: Vec<&str> = lines
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    format!("[\n{}\n]\n", lines.join(",\n"))
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("server_min")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let files = root.join("files");
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::create_dir_all(files.join("sub")).expect("面の file の置き場");
        fs::write(files.join("index.html"), INDEX).expect("index.html");
        fs::write(files.join("sub/app.js"), "console.log(1)").expect("app.js");
        fs::write(root.join("secret.txt"), SECRET).expect("置き場の外の file");
        let bd = root.join("bd");
        fs::write(
            &bd,
            format!(
                "#!/bin/sh\nexec cat '{}'\n",
                root.join("out.json").display()
            ),
        )
        .expect("偽の bd");
        fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("偽の bd の権限");
        Place { root, repo, files }
    }

    /// 変化の印（席の書きで動く file）。
    fn ledger(&self) -> PathBuf {
        self.repo.join(".beads/issues.jsonl")
    }

    /// 偽の bd が返す字を置く（書きかけの字を読ませないよう、別の名で書いてから置き換える）。
    fn bd_returns(&self, text: &str) {
        let tmp = self.root.join("out.json.tmp");
        fs::write(&tmp, text).expect("偽の bd の出力");
        fs::rename(&tmp, self.root.join("out.json")).expect("偽の bd の出力を置く");
    }

    /// bead 5 本の fixture を偽の bd に返させ、印の file も置く。
    fn with_fixture(self) -> Place {
        self.bd_returns(&bd_array(&fixture()));
        fs::write(self.ledger(), fixture()).expect("印の file");
        self
    }

    fn config(&self, bind: &str) -> Config {
        Config {
            bd: self.root.join("bd").into(),
            ..Config::new(
                self.repo.clone(),
                bind.parse().expect("bind 先"),
                self.files.clone(),
            )
        }
    }

    /// server を立てて口の住所を返す。
    fn serve(&self) -> SocketAddr {
        let server = Server::bind(&self.config("127.0.0.1:0")).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }
}

struct Reply {
    status: u16,
    head: String,
    body: String,
}

/// 要求を 1 つ撃ち、接続が閉じるまで応答を読む。
fn request(addr: SocketAddr, raw: &str) -> Reply {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(5)))
        .expect("timeout");
    s.write_all(raw.as_bytes()).expect("要求を書く");
    let mut out = Vec::new();
    s.read_to_end(&mut out).expect("応答を読む");
    let text = String::from_utf8(out).expect("応答の字");
    let (head, body) = text.split_once("\r\n\r\n").expect("頭と本文");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|c| c.parse().ok())
        .expect("状態の code");
    Reply {
        status,
        head: head.to_string(),
        body: body.to_string(),
    }
}

fn get(addr: SocketAddr, path: &str) -> Reply {
    request(addr, &format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n"))
}

fn bead(s: &str) -> BeadId {
    BeadId::new(s).expect("bead id")
}

/// 一覧の 1 行（id・種類・題・状態・更新時刻）。
type Row = (String, String, String, String, u64);

fn rows(reply: &Reply) -> Reading<Vec<Row>> {
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert!(
        reply.head.contains("Content-Type: application/json"),
        "{}",
        reply.head
    );
    let list: LedgerList = wire::decode(&reply.body).expect("契約の型の形");
    match list.rows {
        Reading::Known(rows) => Reading::Known(
            rows.into_iter()
                .map(|r| (r.id.to_string(), r.kind, r.title, r.status, r.updated_at))
                .collect(),
        ),
        Reading::Unknown => Reading::Unknown,
    }
}

#[test]
fn server_min_ledger_list_returns_five() {
    let place = Place::new("list").with_fixture();
    let addr = place.serve();
    let got = rows(&get(addr, "/api/ledger"));
    let want = [
        (
            "fx-min",
            "epic",
            "最小の server の見本の根",
            "open",
            1_790_494_740,
        ),
        (
            "fx-min.1",
            "task",
            "台帳の一覧を読む",
            "in_progress",
            1_790_494_740,
        ),
        (
            "fx-min.2",
            "task",
            "変化を SSE で知らせる",
            "open",
            1_790_494_800,
        ),
        (
            "fx-min.3",
            "task",
            "面の file を配る",
            "closed",
            1_790_494_860,
        ),
        ("fx-min.4", "question", "問いの見本", "open", 1_790_496_720),
    ]
    .map(|(a, b, c, d, e)| (a.into(), b.into(), c.into(), d.into(), e))
    .to_vec();
    assert_eq!(got, Reading::Known(want));
    // 親と label（bd が省いた欄は空）。
    let list: LedgerList = wire::decode(&get(addr, "/api/ledger").body).expect("契約の型の形");
    let Reading::Known(full) = list.rows else {
        panic!("一覧が Unknown");
    };
    let tails: Vec<(Option<String>, Vec<String>)> = full
        .into_iter()
        .map(|r| (r.parent.map(|p| p.to_string()), r.labels))
        .collect();
    assert_eq!(tails[0], (None, vec![]));
    assert_eq!(
        tails[1],
        (Some("fx-min".to_string()), vec!["surface".to_string()])
    );
    assert!(tails[2..].iter().all(|t| t == &(None, vec![])), "{tails:?}");
    // 絞りの引数は今は読まない（一覧は同じ）。
    assert!(
        matches!(rows(&get(addr, "/api/ledger?status=open")), Reading::Known(r) if r.len() == 5)
    );
}

#[test]
fn server_min_ledger_item_by_id() {
    let place = Place::new("item").with_fixture();
    let addr = place.serve();
    let reply = get(addr, "/api/ledger/fx-min.1");
    assert_eq!(reply.status, 200, "{}", reply.body);
    let item: LedgerItem = wire::decode(&reply.body).expect("契約の型の形");
    assert_eq!(item.row.id, bead("fx-min.1"));
    assert_eq!(item.row.title, "台帳の一覧を読む");
    assert_eq!(item.description, "一覧の口");
    assert_eq!(item.notes, "裁定 fx-min.1:20260927T0739Z-1");
    let item: LedgerItem =
        wire::decode(&get(addr, "/api/ledger/fx-min.4").body).expect("契約の型の形");
    assert_eq!(item.row.id, bead("fx-min.4"));
    assert_eq!(item.notes, "");
    assert_eq!(get(addr, "/api/ledger/fx-min.9").status, 404);
    assert_eq!(get(addr, "/api/ledger/-x").status, 400);
    assert_eq!(get(addr, "/api/ledger/a%20b").status, 400);
}

#[test]
fn server_min_ledger_unreadable_is_unknown() {
    // 偽の bd が返す字が無い（cat が rc 1 で終わる）。
    let place = Place::new("unknown");
    let addr = place.serve();
    assert_eq!(rows(&get(addr, "/api/ledger")), Reading::Unknown);
    assert_eq!(get(addr, "/api/ledger/fx-min").status, 503);
    // 壊れた台帳（切れた配列・更新時刻の形の悪い bead・id の形の悪い bead・配列でない字）。
    // 口は見張りの最後の読みの字を返すので、字ごとにその字を出す偽の bd で server を起こす（起動の読みが最後の読み）。
    let good = fixture();
    for (i, broken) in [
        bd_array(&good).replace("\n]\n", ",{\"id\":"),
        bd_array(&good.replace("2026-09-27T07:40:00Z", "昨日")),
        bd_array(&format!(
            "{good}{{\"id\":\"-bad\",\"title\":\"t\",\"status\":\"open\",\"updated_at\":\"2026-09-27T07:39:00Z\"}}\n"
        )),
        good.clone(),
        String::new(),
    ]
    .iter()
    .enumerate()
    {
        let other = Place::new(&format!("unknown-{i}"));
        other.bd_returns(broken);
        let at = other.serve();
        assert_eq!(rows(&get(at, "/api/ledger")), Reading::Unknown, "{broken}");
        assert_eq!(get(at, "/api/ledger/fx-min").status, 503);
    }
    // 印の file の中身は読まない（印だけが在っても台帳は Unknown のまま）。
    fs::write(place.ledger(), good).expect("印の file");
    thread::sleep(Duration::from_millis(1500));
    assert_eq!(rows(&get(addr, "/api/ledger")), Reading::Unknown);
    // 空の台帳は 0 件で、読めない台帳と区別する。
    let empty = Place::new("unknown-empty");
    empty.bd_returns("[]\n");
    let at = empty.serve();
    assert_eq!(rows(&get(at, "/api/ledger")), Reading::Known(vec![]));
}

#[test]
fn server_min_bind_judge() {
    for ok in ["127.0.0.1", "127.10.20.30", "::1", "::ffff:127.0.0.1"] {
        assert!(bind_allowed(ok.parse::<IpAddr>().expect(ok)), "{ok} を断る");
    }
    // tailnet の住所は数の配列から組む（字で書くと xtask の pub-scan の走査に当たる・行 t-pub-scan）。
    for ok in [
        IpAddr::from([100, 64, 0, 1]),
        IpAddr::from([100, 100, 100, 100]),
        IpAddr::from([100, 127, 255, 254]),
        IpAddr::from([0xfd7a, 0x115c, 0xa1e0, 0, 0, 0, 0, 1]),
        IpAddr::from([0xfd7a, 0x115c, 0xa1e0, 0xab12, 0x4843, 0xcd96, 0x6258, 0xb240]),
    ] {
        assert!(bind_allowed(ok), "{ok} を断る");
    }
    for no in [
        "0.0.0.0",
        "::",
        "192.168.1.10",
        "10.0.0.1",
        "172.16.0.1",
        "8.8.8.8",
        "100.63.255.255",
        "100.128.0.1",
        "169.254.1.1",
        "fd7a:115c:a1e1::1",
        "fe80::1",
        "2001:db8::1",
        "::ffff:8.8.8.8",
    ] {
        assert!(
            !bind_allowed(no.parse::<IpAddr>().expect(no)),
            "{no} を通す"
        );
    }
}

#[test]
fn server_min_bind_refused_rc1() {
    let place = Place::new("refuse");
    for bind in ["0.0.0.0:0", "192.168.1.10:0", "[::]:0", "8.8.8.8:8080"] {
        assert!(
            matches!(
                Server::bind(&place.config(bind)),
                Err(StartError::BindRefused(_))
            ),
            "{bind} で起動する"
        );
        let out = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(["surface", "serve", "--repo"])
            .arg(&place.repo)
            .args(["--bind", bind, "--files"])
            .arg(&place.files)
            .output()
            .expect("tz を撃つ");
        assert_eq!(out.status.code(), Some(1), "{bind}: {out:?}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("N-6"),
            "{bind}: {out:?}"
        );
    }
    // 使い方の誤りも rc 1。
    for args in [
        &["surface"][..],
        &["surface", "serve", "--bind", "127.0.0.1:0"],
        &[
            "surface",
            "serve",
            "--bind",
            "localhost",
            "--repo",
            ".",
            "--files",
            ".",
        ],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(args)
            .output()
            .expect("tz を撃つ");
        assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
    }
}

/// SSE の口から今まで届いた字を読む（`until` まで・届いた分を返す）。
fn read_until(s: &mut TcpStream, buf: &mut String, until: Instant, stop: impl Fn(&str) -> bool) {
    let mut chunk = [0u8; 4096];
    while !stop(buf) {
        let left = until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return;
        }
        s.set_read_timeout(Some(left)).expect("timeout");
        match s.read(&mut chunk) {
            Ok(0) => return,
            Ok(n) => buf.push_str(std::str::from_utf8(&chunk[..n]).expect("SSE の字")),
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => return,
            Err(e) => panic!("SSE を読む: {e}"),
        }
    }
}

#[test]
fn server_min_sse_change_within_1500ms() {
    let place = Place::new("sse").with_fixture();
    let addr = place.serve();
    let mut s = TcpStream::connect(addr).expect("接続");
    s.write_all(
        b"GET /api/surface/events HTTP/1.1\r\nHost: x\r\nAccept: text/event-stream\r\n\r\n",
    )
    .expect("要求");
    let mut buf = String::new();
    read_until(
        &mut s,
        &mut buf,
        Instant::now() + Duration::from_secs(5),
        |b| b.contains("retry: 1000\n\n"),
    );
    assert!(buf.starts_with("HTTP/1.1 200 OK\r\n"), "{buf}");
    assert!(buf.contains("Content-Type: text/event-stream\r\n"), "{buf}");
    let events = |b: &str| b.matches("event: ledger-changed\n").count();
    assert_eq!(events(&buf), 0, "変化の前に知らせが出る: {buf}");

    // 台帳に 1 本を足す（bd の出力と、席の書きで動く印の file の両方）。
    let added = "{\"id\":\"fx-min.5\",\"title\":\"t\",\"status\":\"open\",\"updated_at\":\"2026-09-27T07:43:00Z\"}\n";
    let changed = Instant::now();
    place.bd_returns(&bd_array(&format!("{}{added}", fixture())));
    OpenOptions::new()
        .append(true)
        .open(place.ledger())
        .expect("印の file を開く")
        .write_all(added.as_bytes())
        .expect("印の file を書き換える");
    read_until(
        &mut s,
        &mut buf,
        changed + Duration::from_millis(1500),
        |b| events(b) >= 1 && b.ends_with("\n\n"),
    );
    let took = changed.elapsed();
    assert_eq!(
        events(&buf),
        1,
        "1.5 秒以内に 1 件でない（{took:?}）: {buf}"
    );
    assert!(took < Duration::from_millis(1500), "{took:?}");
    let frame = buf
        .split("\n\n")
        .find(|f| f.contains("event: ledger-changed"))
        .expect("event");
    let data = frame
        .lines()
        .find_map(|l| l.strip_prefix("data: "))
        .expect("data の行");
    let note: LedgerChanged = wire::decode(data).expect("契約の型の形");
    assert!(note.at > 1_790_000_000, "{data}");
    // 変化が無ければ次の知らせは出ない（周期の読み 2 回分を待つ）。
    read_until(
        &mut s,
        &mut buf,
        Instant::now() + Duration::from_millis(1100),
        |b| events(b) >= 2,
    );
    assert_eq!(events(&buf), 1, "変化 1 回に知らせが 2 件以上: {buf}");
    // 知らせの後の一覧は新しい行を含む。
    assert!(matches!(rows(&get(addr, "/api/ledger")), Reading::Known(r) if r.len() == 6));
}

#[test]
fn server_min_files_refuse_outside() {
    let place = Place::new("files");
    let addr = place.serve();
    let index = get(addr, "/");
    assert_eq!(index.status, 200);
    assert_eq!(index.body, INDEX);
    assert!(
        index.head.contains("Content-Type: text/html"),
        "{}",
        index.head
    );
    assert_eq!(get(addr, "/index.html").body, INDEX);
    let js = get(addr, "/sub/app.js?v=1");
    assert_eq!(js.status, 200);
    assert!(
        js.head.contains("Content-Type: text/javascript"),
        "{}",
        js.head
    );
    assert_eq!(get(addr, "/sub/").status, 404);
    assert_eq!(get(addr, "/nope.wasm").status, 404);
    // 置き場の外の file を指す symlink。
    std::os::unix::fs::symlink(
        place.files.join("../secret.txt"),
        place.files.join("leak.txt"),
    )
    .expect("symlink");
    for outside in [
        "/../secret.txt",
        "/sub/../../secret.txt",
        "/%2e%2e/secret.txt",
        "/%2E%2E%2Fsecret.txt",
        "/sub/..%2f..%2fsecret.txt",
        "/./index.html",
        "/..\\secret.txt",
        "/%00",
        "/%zz",
        "/leak.txt",
    ] {
        let reply = get(addr, outside);
        assert_eq!(reply.status, 403, "{outside}: {}", reply.body);
        assert!(!reply.body.contains(SECRET), "{outside} が外の file を返す");
    }
}

/// dir の中の file の path と byte の一覧（書かれていないことを比べる）。
fn tree(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).expect("dir を読む") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                out.push((path.clone(), Vec::new()));
                stack.push(path);
            } else {
                out.push((path.clone(), fs::read(&path).expect("file を読む")));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn server_min_non_get_405_writes_nothing() {
    let place = Place::new("method").with_fixture();
    let addr = place.serve();
    let before = (tree(&place.repo), tree(&place.files));
    for (method, path) in [
        ("POST", "/api/ledger"),
        ("PUT", "/api/ledger/fx-min.1"),
        ("DELETE", "/api/ledger/fx-min.1"),
        ("PATCH", "/api/ledger/fx-min.1"),
        ("POST", "/api/surface/events"),
        ("PUT", "/index.html"),
        ("PUT", "/new.html"),
        ("HEAD", "/"),
        ("OPTIONS", "/"),
    ] {
        let body = r#"{"id":"fx-min.1","title":"書き換え","status":"closed"}"#;
        let reply = request(
            addr,
            &format!(
                "{method} {path} HTTP/1.1\r\nHost: x\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            ),
        );
        assert_eq!(reply.status, 405, "{method} {path}");
        assert!(reply.head.contains("Allow: GET"), "{}", reply.head);
    }
    assert!(
        before == (tree(&place.repo), tree(&place.files)),
        "405 の後に置き場の byte が変わる"
    );
    assert!(!place.files.join("new.html").exists());
    // 形の悪い要求は 400。
    assert_eq!(request(addr, "GARBAGE\r\n\r\n").status, 400);
}
