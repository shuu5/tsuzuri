//! 台帳の読みの出所の歯（接頭辞 server_src_・設計ノート surface-base 便 e-src の完了の条件）。
//! 偽の bd（shell の script）を歯ごとの一時の dir に置き、server は同じ process の thread で
//! 127.0.0.1 の空き port に立てる。偽の bd は撃たれるたびに引数・cwd・標準入力を calls/ の下に残す。
#![cfg(test)]

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::server::ledger::{BD, BD_ARGS, BD_TIMEOUT, BD_WAIT};
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{LedgerItem, LedgerList, LedgerRow};
use tsuzuri_contract::wire;

/// 偽の bd の振る舞い（記録の後の最後の行）。
enum Fake {
    /// out.json を返して rc 0。
    Ok,
    /// out.json を返すが rc 1。
    Rc1,
    /// JSON でない字を返して rc 0。
    NotJson,
    /// 返さない（`BD_TIMEOUT` を越えて眠る）。
    Hang,
}

/// 歯ごとの作業場（repo の置き場・面の file の置き場・偽の bd と、その出力と記録）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
}

/// bead 8 本の fixture（bd の読み取りの口の出力の形）。
fn fixture() -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/ledger/bd-list-8.json"),
    )
    .expect("fixture")
}

impl Place {
    fn new(name: &str, fake: Fake) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("server_src")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let files = root.join("files");
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::create_dir_all(&files).expect("面の file の置き場");
        fs::create_dir_all(root.join("calls")).expect("記録の置き場");
        fs::write(files.join("index.html"), "tz").expect("index.html");
        for mark in ["issues.jsonl", "interactions.jsonl"] {
            fs::write(repo.join(".beads").join(mark), "{}\n").expect("印の file");
        }
        let (calls, out) = (root.join("calls"), root.join("out.json"));
        let (calls, out) = (calls.display(), out.display());
        let last = match fake {
            Fake::Ok => format!("exec cat '{out}'"),
            Fake::Rc1 => format!("cat '{out}'\nexit 1"),
            Fake::NotJson => "echo 'not json'".to_string(),
            Fake::Hang => "exec sleep 30".to_string(),
        };
        let script = format!(
            "#!/bin/sh\n\
             for a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{calls}'/$$.args\n\
             pwd -P > '{calls}'/$$.cwd\n\
             cat > '{calls}'/$$.stdin\n\
             echo '標準エラーの字' >&2\n\
             {last}\n"
        );
        let bd = root.join("bd");
        fs::write(&bd, script).expect("偽の bd");
        fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("偽の bd の権限");
        let place = Place { root, repo, files };
        place.bd_returns(&fixture());
        place
    }

    fn bd(&self) -> PathBuf {
        self.root.join("bd")
    }

    /// 偽の bd が返す字を置く（書きかけの字を読ませないよう、別の名で書いてから置き換える）。
    fn bd_returns(&self, text: &str) {
        let tmp = self.root.join("out.json.tmp");
        fs::write(&tmp, text).expect("偽の bd の出力");
        fs::rename(&tmp, self.root.join("out.json")).expect("偽の bd の出力を置く");
    }

    /// 印の file（issues.jsonl か interactions.jsonl）に 1 行を足す。
    fn touch(&self, mark: &str) {
        OpenOptions::new()
            .append(true)
            .open(self.repo.join(".beads").join(mark))
            .expect("印の file を開く")
            .write_all(b"{}\n")
            .expect("印の file を書く");
    }

    fn config(&self, bd: impl Into<std::ffi::OsString>) -> Config {
        Config {
            bd: bd.into(),
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        }
    }

    /// 偽の bd で server を立てて口の住所を返す。
    fn serve(&self) -> SocketAddr {
        serve(&self.config(self.bd()))
    }

    /// 偽の bd が撃たれた回ごとの（引数の列・cwd・標準入力）。
    fn calls(&self) -> Vec<(Vec<String>, String, String)> {
        let dir = self.root.join("calls");
        let mut pids: Vec<String> = fs::read_dir(&dir)
            .expect("記録の置き場")
            .filter_map(|e| {
                let name = e.expect("entry").file_name().into_string().ok()?;
                name.strip_suffix(".args").map(str::to_string)
            })
            .collect();
        pids.sort();
        pids.into_iter()
            .map(|pid| {
                let read = |ext: &str| {
                    fs::read_to_string(dir.join(format!("{pid}.{ext}"))).unwrap_or_default()
                };
                (
                    read("args").lines().map(str::to_string).collect(),
                    read("cwd"),
                    read("stdin"),
                )
            })
            .collect()
    }
}

fn serve(config: &Config) -> SocketAddr {
    let server = Server::bind(config).expect("起動");
    let addr = server.local_addr().expect("口の住所");
    thread::spawn(move || server.run());
    addr
}

/// GET を 1 つ撃ち、接続が閉じるまで応答の（状態の code・本文）を読む（bd の上限を越えて待てる）。
fn get(addr: SocketAddr, path: &str) -> (u16, String) {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(BD_TIMEOUT * 3)).expect("timeout");
    s.write_all(format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n").as_bytes())
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

fn list(addr: SocketAddr) -> Reading<Vec<LedgerRow>> {
    let (status, body) = get(addr, "/api/ledger");
    assert_eq!(status, 200, "{body}");
    let list: LedgerList = wire::decode(&body).expect("契約の型の形");
    list.rows
}

/// 一覧の 1 行の（id・状態・親・label）。
fn brief(r: &LedgerRow) -> (String, String, Option<String>, Vec<String>) {
    (
        r.id.to_string(),
        r.status.clone(),
        r.parent.as_ref().map(ToString::to_string),
        r.labels.clone(),
    )
}

#[test]
fn server_src_lists_eight_from_bd() {
    let place = Place::new("list", Fake::Ok);
    let addr = place.serve();
    let Reading::Known(rows) = list(addr) else {
        panic!("8 本の台帳が Unknown");
    };
    let got: Vec<_> = rows.iter().map(brief).collect();
    let hub = || Some("fx-hub".to_string());
    let q = || vec!["intake:question".to_string()];
    let want = vec![
        ("fx-hub.7", "open", hub(), vec![]),
        ("fx-hub.6", "in_progress", hub(), vec![]),
        ("fx-hub.5", "closed", hub(), vec![]),
        ("fx-hub.4", "open", hub(), q()),
        ("fx-hub.3", "open", hub(), q()),
        ("fx-hub.2", "closed", hub(), q()),
        ("fx-hub.1", "open", hub(), vec!["intake:memo".to_string()]),
        ("fx-hub", "open", None, vec![]),
    ]
    .into_iter()
    .map(|(id, st, p, l)| (id.to_string(), st.to_string(), p, l))
    .collect::<Vec<_>>();
    assert_eq!(got, want);
    // closed の 2 本。
    let closed: Vec<&str> = rows
        .iter()
        .filter(|r| r.status == "closed")
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(closed, ["fx-hub.5", "fx-hub.2"]);
    // 問いと memo は label で見分ける。
    let questions: Vec<&str> = rows
        .iter()
        .filter(|r| r.is_question())
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(questions, ["fx-hub.4", "fx-hub.3", "fx-hub.2"]);
    let memos: Vec<&str> = rows
        .iter()
        .filter(|r| r.is_memo())
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(memos, ["fx-hub.1"]);
    // 種類と題と更新時刻。
    assert_eq!(rows[7].kind, "epic");
    assert_eq!(rows[2].title, "便 1 — 着地した便");
    assert_eq!(rows[2].updated_at, 1_789_956_000);
    // 1 本の口も bd の読みから引く。
    let (status, body) = get(addr, "/api/ledger/fx-hub.2");
    assert_eq!(status, 200, "{body}");
    let item: LedgerItem = wire::decode(&body).expect("契約の型の形");
    assert_eq!(item.row.status, "closed");
    assert_eq!(item.notes, "見本の notes の 1 行");
    assert_eq!(item.description, "fixture の bead（本文は見本の字）");
    assert_eq!(get(addr, "/api/ledger/fx-hub.9").0, 404);
}

#[test]
fn server_src_bd_args_and_cwd() {
    let place = Place::new("args", Fake::Ok);
    let addr = place.serve();
    assert!(matches!(list(addr), Reading::Known(r) if r.len() == 8));
    let calls = place.calls();
    // 起動の最初の読みの 1 回以上（一覧の口は bd を撃たず見張りの読みの字を返す・行 e-snap）。
    assert!(!calls.is_empty(), "{calls:?}");
    let repo = format!(
        "{}\n",
        place.repo.canonicalize().expect("repo の実体").display()
    );
    for (args, cwd, stdin) in &calls {
        assert_eq!(args, &BD_ARGS.map(String::from), "引数の列");
        assert_eq!(
            args.join(" "),
            "--readonly list --all --limit 0 --json",
            "引数の列"
        );
        assert_eq!(cwd, &repo, "cwd は repo の置き場");
        assert_eq!(stdin, "", "標準入力は空");
    }
    assert_eq!(BD, "bd", "program の名の既定");
}

#[test]
fn server_src_bd_failures_are_unknown() {
    for (name, fake) in [("rc1", Fake::Rc1), ("not-json", Fake::NotJson)] {
        let place = Place::new(name, fake);
        let addr = place.serve();
        assert_eq!(list(addr), Reading::Unknown, "{name}");
        assert_eq!(get(addr, "/api/ledger/fx-hub").0, 503, "{name}");
        assert!(
            !place.calls().is_empty(),
            "{name}: 偽の bd が撃たれていない"
        );
    }
    // 起動できない program。
    let place = Place::new("no-program", Fake::Ok);
    let addr = serve(&place.config(place.root.join("no-such-bd")));
    assert_eq!(list(addr), Reading::Unknown);
}

#[test]
fn server_src_bd_timeout_is_unknown() {
    let place = Place::new("hang", Fake::Hang);
    // 起動の読みが上限で止まる（Server::bind は最初の読みを取ってから戻る）。
    let started = Instant::now();
    let addr = place.serve();
    let took = started.elapsed();
    assert!(took >= BD_TIMEOUT - Duration::from_millis(100), "{took:?}");
    assert!(took < BD_TIMEOUT + Duration::from_secs(2), "{took:?}");
    // 一覧の口は見張りの読みの結果を返す（読みの途中なら、その終わりを BD_WAIT まで待つ）。
    let started = Instant::now();
    assert_eq!(list(addr), Reading::Unknown);
    let took = started.elapsed();
    assert!(took < BD_WAIT + Duration::from_secs(1), "{took:?}");
}

/// SSE の口を開き、頭の後まで読む。
fn subscribe(addr: SocketAddr) -> (TcpStream, String) {
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
    (s, buf)
}

/// SSE の口から `until` まで（か `stop` が立つまで）読む。
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

fn events(b: &str) -> usize {
    b.matches("event: ledger-changed\n").count()
}

/// fixture の 1 本の状態を替えた出力（器が便を閉じた見立て）。
fn closed_fixture(id: &str) -> String {
    let text = fixture();
    let at = text
        .find(&format!("\"id\": \"{id}\""))
        .expect("fixture の bead");
    let (head, tail) = text.split_at(at);
    format!(
        "{head}{}",
        tail.replacen("\"status\": \"open\"", "\"status\": \"closed\"", 1)
    )
}

#[test]
fn server_src_marks_change_within_1500ms() {
    let place = Place::new("marks", Fake::Ok);
    let addr = place.serve();
    let (mut s, mut buf) = subscribe(addr);
    assert_eq!(events(&buf), 0, "変化の前に知らせが出る: {buf}");
    // 後の印（器の close を含む状態の変更で動く file）が動く。
    for (n, (mark, id)) in [
        ("interactions.jsonl", "fx-hub.7"),
        ("issues.jsonl", "fx-hub.4"),
    ]
    .into_iter()
    .enumerate()
    {
        let changed = Instant::now();
        place.bd_returns(&closed_fixture(id));
        place.touch(mark);
        read_until(
            &mut s,
            &mut buf,
            changed + Duration::from_millis(1500),
            |b| events(b) > n && b.ends_with("\n\n"),
        );
        let took = changed.elapsed();
        assert_eq!(
            events(&buf),
            n + 1,
            "{mark}: 1.5 秒以内に 1 件でない（{took:?}）: {buf}"
        );
        assert!(took < Duration::from_millis(1500), "{mark}: {took:?}");
        let Reading::Known(rows) = list(addr) else {
            panic!("{mark}: 一覧が Unknown");
        };
        let row = rows.iter().find(|r| r.id.as_str() == id).expect("bead");
        assert_eq!(row.status, "closed", "{mark}: 知らせの後の一覧が古い");
    }
    // 印が動いても読みが同じなら知らせない。
    place.touch("issues.jsonl");
    read_until(
        &mut s,
        &mut buf,
        Instant::now() + Duration::from_millis(1100),
        |b| events(b) >= 3,
    );
    assert_eq!(events(&buf), 2, "同じ読みで知らせる: {buf}");
}

#[test]
fn server_src_rereads_every_5s_without_marks() {
    let place = Place::new("reread", Fake::Ok);
    let addr = place.serve();
    let bound = Instant::now();
    let (mut s, mut buf) = subscribe(addr);
    // 受け手が付いた周の読みを待ってから、印を動かさずに中身だけを替える（印の取りこぼしの見立て）。
    thread::sleep(Duration::from_secs(1));
    place.bd_returns(&closed_fixture("fx-hub.3"));
    read_until(&mut s, &mut buf, bound + Duration::from_secs(4), |b| {
        events(b) >= 1
    });
    assert_eq!(
        events(&buf),
        0,
        "印が動かないのに 5 秒より前に知らせる: {buf}"
    );
    read_until(&mut s, &mut buf, bound + Duration::from_secs(7), |b| {
        events(b) >= 1 && b.ends_with("\n\n")
    });
    assert_eq!(
        events(&buf),
        1,
        "5 秒ごとの読み直しで 1 件でない（{:?}）: {buf}",
        bound.elapsed()
    );
    let Reading::Known(rows) = list(addr) else {
        panic!("一覧が Unknown");
    };
    let row = rows
        .iter()
        .find(|r| r.id.as_str() == "fx-hub.3")
        .expect("bead");
    assert_eq!(row.status, "closed");
}

#[test]
fn server_src_tz_takes_bd_flag() {
    let place = Place::new("flag", Fake::Ok);
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["surface", "serve", "--bind", "127.0.0.1:0", "--repo"])
        .arg(&place.repo)
        .arg("--files")
        .arg(&place.files)
        .arg(format!("--bd={}", place.bd().display()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("tz を撃つ");
    let mut line = String::new();
    BufReader::new(child.stderr.take().expect("標準エラー"))
        .read_line(&mut line)
        .expect("口の住所の行");
    let addr: SocketAddr = line
        .trim()
        .strip_prefix("tz surface serve: http://")
        .and_then(|a| a.strip_suffix('/'))
        .and_then(|a| a.parse().ok())
        .unwrap_or_else(|| panic!("口の住所の行でない: {line}"));
    let got = list(addr);
    let _ = child.kill();
    let _ = child.wait();
    assert!(matches!(got, Reading::Known(r) if r.len() == 8));
    // 空の --bd は使い方の誤りで rc 1。
    let out = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["surface", "serve", "--bind", "127.0.0.1:0", "--repo"])
        .arg(&place.repo)
        .arg("--files")
        .arg(&place.files)
        .arg("--bd=")
        .output()
        .expect("tz を撃つ");
    assert_eq!(out.status.code(), Some(1), "{out:?}");
}
