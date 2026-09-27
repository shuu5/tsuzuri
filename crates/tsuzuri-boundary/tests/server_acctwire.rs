//! account board の口をつなぐ歯（接頭辞 acctwire_・行 h-wire の完了の条件 (1)〜(4) と (7)）。
//! 偽の器・偽の git・偽の bd は、受けた argv を記録の置き場（repo と state dir の外）に 1 行ずつ足し、
//! 作業場の out・git・bd の下の決めた字を標準出力へ出す script（file が無ければ rc 1）。
//! 器の `seat heartbeat` は何も出さずに rc 0 で返る。
//! 引数の state dir は state-h で、anchor proj-a の state dir は git が返す state-a（行 D-4: path は実行の時に組む）。
//! server は同じ process の thread で 127.0.0.1 の空き port に立てる。

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tsuzuri_boundary::acct::Acct;
use tsuzuri_boundary::accthb::{self, NO_PROJECT};
use tsuzuri_boundary::server::{ACCT_MARKS_EVERY, Config, GIT, Server};
use tsuzuri_contract::account::{
    AccountDoc, HEARTBEAT_PATH, Heartbeat, HeartbeatResponse, PATH,
};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::wire;

const TICK_A: &str = "seat tick status: target=proj-a:0.1 last=1790510390 age=10 healthy=yes heartbeat=on step=5 next=1790510395\n";

const USAGE: &str = "usage: account=acct-1 five_hour=5% resets=none seven_day=30% resets=none model=opus:0% resets=none\n";

const GROUP_LINE: &str =
    "group=g-a accounts=acct-1 anchors=1 seat-accounts=acct-1 current=acct-1 next=none refused=-\n";

const STATE_A: &str =
    "{\"schema\":1,\"state\":\"busy\",\"event\":\"prompt\",\"ts\":1790500000,\"sid\":\"s-1\"}\n";

const EVENTS_A: &str = "{\"schema\":1,\"ts\":\"2026-09-27T11:00:00Z\",\"kind\":\"RunCreated\",\"run\":\"r.1-20260927T110000Z\",\"bead\":\"r.1\",\"host\":\"host-1\",\"actor\":\"machine\",\"stage\":\"Intake\",\"detail\":\"classes:\"}\n";

const LEDGER_A: &str = "[{\"id\":\"r.1\",\"title\":\"t\",\"status\":\"open\",\"issue_type\":\"task\",\"updated_at\":\"2026-09-27T07:39:00Z\"}]\n";

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 歯ごとの作業場。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
    work: PathBuf,
    states: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("server_acctwire")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let place = Place {
            repo: root.join("repo"),
            files: root.join("files"),
            work: root.join("work"),
            states: root.join("states"),
            root,
        };
        for dir in ["bin", "out", "git", "bd", "log"] {
            fs::create_dir_all(place.root.join(dir)).expect("置き場");
        }
        for dir in [
            place.repo.join(".beads"),
            place.files.clone(),
            place.work.join("proj-a"),
            place.state("state-h").join("fleet"),
            place.states.join("scribe2-host/groups/history"),
        ] {
            fs::create_dir_all(dir).expect("置き場");
        }
        fs::write(place.files.join("index.html"), "tz").expect("index.html");
        let root = place.root.display().to_string();
        script(
            &place.program("scribe2"),
            &"printf '%s\\n' \"$*\" >> 'ROOT/log/scribe2'\n\
              for a in \"$@\"; do last=\"$a\"; done\n\
              if [ \"$1 $2\" = 'seat heartbeat' ]; then exit 0; fi\n\
              case \"$1\" in seat) f=\"tick-${last##*/}\" ;; doctor) f=\"doctor-${last##*/}\" ;; \
              fleet) f=\"usage-${last##*/}\" ;; rules) f=grace ;; *) exit 2 ;; esac\n\
              exec cat 'ROOT/out/'\"$f\""
                .replace("ROOT", &root),
        );
        script(
            &place.program("git"),
            &"printf '%s\\n' \"$*\" >> 'ROOT/log/git'\n\
              f=\"${2%/}\"; f=\"${f##*/}\"\n\
              exec cat 'ROOT/git/'\"$f\""
                .replace("ROOT", &root),
        );
        script(
            &place.program("bd"),
            &"d=\"$(pwd -P)\"; d=\"${d##*/}\"\n\
              if [ -e 'ROOT/bd/'\"$d\" ]; then exec cat 'ROOT/bd/'\"$d\"; fi\n\
              echo '[]'"
                .replace("ROOT", &root),
        );
        place.lay();
        place
    }

    fn program(&self, name: &str) -> PathBuf {
        self.root.join("bin").join(name)
    }

    fn state(&self, name: &str) -> PathBuf {
        self.states.join(name)
    }

    fn seat_dir(&self) -> PathBuf {
        self.state("state-a").join("seat/proj-a_0.1")
    }

    fn put(&self, path: PathBuf, text: &str) {
        fs::create_dir_all(path.parent().expect("親")).expect("親の dir");
        fs::write(&path, text).expect("file を置く");
    }

    /// 字を組んで置く（anchor の path は実行の時の作業場から）。
    fn lay(&self) {
        let a = self.work.join("proj-a").display().to_string();
        let root = self.root.display();
        self.put(
            self.state("state-h").join("host.toml"),
            &format!(
                "[[account]]\nlabel = \"acct-1\"\n\n\
                 [[account-group]]\nname = \"g-a\"\nanchors = [\"{a}\"]\naccounts = [\"acct-1\"]\n"
            ),
        );
        let host = format!(
            "doctor: state dir ok\n{GROUP_LINE}account=acct-1 dir={root}/accounts/acct-1 retired=no\n"
        );
        self.put(self.root.join("out/doctor-state-h"), &host);
        self.put(
            self.root.join("out/doctor-state-a"),
            &format!(
                "{host}seat: role=orchestrator anchor={a} target=proj-a:0.1 account=acct-1 model=opus\n"
            ),
        );
        self.put(self.root.join("out/tick-state-a"), TICK_A);
        self.put(self.root.join("out/usage-state-h"), USAGE);
        self.put(self.root.join("out/grace"), "1800\n");
        self.put(
            self.root.join("git/proj-a"),
            &format!("{}\n", self.state("state-a").display()),
        );
        self.put(self.root.join("bd/proj-a"), LEDGER_A);
        self.put(self.seat_dir().join("state.jsonl"), STATE_A);
        self.put(
            self.seat_dir().join("tick-last"),
            "ts=1790510390 decision=noop reason=seat-busy\n",
        );
        self.put(self.state("state-a").join("fleet/events.jsonl"), EVENTS_A);
        self.put(self.state("state-h").join("fleet/events.jsonl"), "");
        self.put(
            self.states.join("scribe2-host/groups/g-a.account"),
            "account=acct-1\nts=2026-09-27T11:50:00Z\nreason=initial\n",
        );
    }

    fn config(&self, state_dir: bool) -> Config {
        Config {
            bd: self.program("bd").into(),
            state_dir: state_dir.then(|| self.state("state-h")),
            bdw: self.program("bdw").into(),
            scribe2: self.program("scribe2").into(),
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        }
    }

    /// 偽の git で server を立てる（`state_dir` が偽なら --state-dir の無い server）。
    fn serve(&self, state_dir: bool) -> SocketAddr {
        let server =
            Server::bind_with(&self.config(state_dir), self.program("git").as_os_str())
                .expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }

    /// server と同じ出所の読み。
    fn acct(&self) -> Acct {
        Acct::new(
            self.program("scribe2"),
            self.program("git"),
            self.program("bd"),
            self.state("state-h"),
            &self.repo,
        )
    }

    /// 偽の program が受けた argv（1 回 1 行・撃った順）。
    fn calls(&self, program: &str) -> Vec<String> {
        fs::read_to_string(self.root.join("log").join(program))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    fn clear_calls(&self) {
        for program in ["scribe2", "git"] {
            let _ = fs::remove_file(self.root.join("log").join(program));
        }
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_secs()
}

/// 応答（状態の code・頭・本文）。
struct Reply {
    status: u16,
    head: String,
    body: String,
}

fn request(addr: SocketAddr, raw: &[u8]) -> Reply {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    s.write_all(raw).expect("要求を書く");
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
    request(
        addr,
        format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes(),
    )
}

fn post(addr: SocketAddr, heads: &str, body: &str) -> Reply {
    request(
        addr,
        format!(
            "POST {HEARTBEAT_PATH} HTTP/1.1\r\nHost: {addr}\r\n{heads}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        )
        .as_bytes(),
    )
}

fn account(addr: SocketAddr) -> AccountDoc {
    let reply = get(addr, PATH);
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert!(
        reply.head.contains("Content-Type: application/json"),
        "{}",
        reply.head
    );
    wire::decode(&reply.body).unwrap_or_else(|e| panic!("電文の形でない {e}: {}", reply.body))
}

fn hb_body(project: &str, to: &str) -> String {
    format!("{{\"project\":\"{project}\",\"to\":\"{to}\"}}")
}

/// (1) GET の口は、同じ偽の器と偽の git と state dir で読みの関数を直に呼んだ値と at を除いて同じ電文。
#[test]
fn acctwire_get_matches_acct() {
    let place = Place::new("get");
    let addr = place.serve(true);
    let from = now();
    let got = account(addr);
    assert!((from..=now()).contains(&got.at), "組んだ時刻は今: {}", got.at);
    let want = place.acct().doc(got.at);
    assert_eq!(got, want);
    // 電文の字も wire の encode の字と同じ。
    let body = get(addr, PATH).body;
    let again: AccountDoc = wire::decode(&body).expect("電文");
    assert_eq!(body, wire::encode(&again).expect("字"));
    // 空の電文でない（出所が読めている）。
    let names: Vec<&str> = got.projects.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, ["proj-a"]);
    assert!(got.projects[0].state_dir_known);
    assert!(matches!(got.accounts, Reading::Known(_)));
    assert!(matches!(got.groups, Reading::Known(_)));
    assert!(!place.calls("git").is_empty(), "偽の git を撃つ");
    assert_eq!(GIT, "git");
}

/// (3) --state-dir が無いとき、GET の口は 200 で空の電文を返し、器も git も撃たない。
#[test]
fn acctwire_get_without_state_dir() {
    let place = Place::new("no-state");
    let addr = place.serve(false);
    let from = now();
    let got = account(addr);
    assert!((from..=now()).contains(&got.at), "組んだ時刻は今: {}", got.at);
    assert_eq!(got.accounts, Reading::Unknown);
    assert_eq!(got.groups, Reading::Unknown);
    assert_eq!(got.moves, Reading::Unknown);
    assert!(got.projects.is_empty());
    assert!(got.sessions.is_empty());
    assert!(got.dormant.is_empty());
    thread::sleep(Duration::from_millis(300));
    assert!(place.calls("scribe2").is_empty(), "{:?}", place.calls("scribe2"));
    assert!(place.calls("git").is_empty(), "{:?}", place.calls("git"));
}

/// (2) POST の口の守り（403・413 は器を撃たない）と受付の関数の状態の数と本文。
#[test]
fn acctwire_post_heartbeat() {
    let place = Place::new("post");
    let addr = place.serve(true);
    // Origin が Host と違えば 403 origin。
    let port = addr.port();
    for origin in [
        format!("http://evil.example:{port}"),
        format!("http://127.0.0.1:{}", port.wrapping_add(1)),
        "null".to_string(),
    ] {
        let reply = post(addr, &format!("Origin: {origin}\r\n"), &hb_body("proj-a", "off"));
        assert_eq!((reply.status, reply.body.as_str()), (403, "origin"), "{origin}");
    }
    // 本文が 65536 byte を越えれば 413 too-large。
    let big = format!(
        "{{\"project\":\"proj-a\",\"to\":\"off\",\"pad\":\"{}\"}}",
        "x".repeat(65_536)
    );
    let reply = post(addr, "", &big);
    assert_eq!((reply.status, reply.body.as_str()), (413, "too-large"));
    assert!(place.calls("scribe2").is_empty(), "守りで器を撃つ");
    assert!(place.calls("git").is_empty(), "守りで git を撃つ");
    // 通った本文は受付の関数の状態の数と本文をそのまま返す。
    for (heads, text) in [
        (format!("Origin: http://{addr}\r\n"), hb_body("proj-a", "off")),
        (String::new(), hb_body("proj-a", "on")),
        (String::new(), hb_body("proj-x", "off")),
        (String::new(), "{".to_string()),
    ] {
        let reply = post(addr, &heads, &text);
        let want = accthb::accept(&place.acct(), &text);
        assert_eq!((reply.status, reply.body.clone()), want, "{text}");
        let kind = if reply.status == 200 {
            "application/json"
        } else {
            "text/plain"
        };
        assert!(reply.head.contains(kind), "{text}: {}", reply.head);
    }
    let reply = post(addr, "", &hb_body("proj-a", "off"));
    let got: HeartbeatResponse = wire::decode(&reply.body).expect("応答の電文");
    assert_eq!(
        got,
        HeartbeatResponse {
            target: "proj-a:0.1".to_string(),
            to: Heartbeat::Off
        }
    );
    let sa = place.state("state-a").display().to_string();
    assert_eq!(
        place.calls("scribe2").last().map(String::as_str),
        Some(format!("seat heartbeat off --state-dir {sa} --target proj-a:0.1").as_str())
    );
}

/// (2) state dir の無い server は受付の関数を呼ばずに 404 no-project。
#[test]
fn acctwire_post_without_state_dir() {
    let place = Place::new("post-no-state");
    let addr = place.serve(false);
    let reply = post(addr, "", &hb_body("proj-a", "off"));
    assert_eq!((reply.status, reply.body.as_str()), (404, NO_PROJECT));
    assert_eq!(NO_PROJECT, "no-project");
    let reply = post(
        addr,
        &format!("Origin: http://evil.example:{}\r\n", addr.port()),
        &hb_body("proj-a", "off"),
    );
    assert_eq!(reply.status, 403);
    assert!(place.calls("scribe2").is_empty());
    assert!(place.calls("git").is_empty());
}

/// SSE の接続から `event` の frame を `limit` まで待つ。
fn wait_event(s: &mut TcpStream, seen: &mut String, event: &str, limit: Duration) -> bool {
    let deadline = Instant::now() + limit;
    let needle = format!("event: {event}\n");
    let mut buf = [0u8; 4096];
    while Instant::now() < deadline {
        if let Some(at) = seen.find(&needle) {
            seen.drain(..at + needle.len());
            return true;
        }
        let left = deadline.saturating_duration_since(Instant::now());
        s.set_read_timeout(Some(left.max(Duration::from_millis(10))))
            .expect("timeout");
        match s.read(&mut buf) {
            Ok(0) => return false,
            Ok(n) => seen.push_str(&String::from_utf8_lossy(&buf[..n])),
            Err(_) => {}
        }
    }
    seen.contains(&needle)
}

fn append(path: &Path, line: &str) {
    let mut text = fs::read_to_string(path).unwrap_or_default();
    text.push_str(line);
    fs::write(path, text).expect("印の file を書き換える");
}

/// (4) GET の口を撃つまでは acct のために器を撃たず、撃った後は acct の印の file の変化が 5 秒以内に届く。
#[test]
fn acctwire_marks_send_board_changed() {
    let place = Place::new("sse");
    let addr = place.serve(true);
    let mut s = TcpStream::connect(addr).expect("接続");
    s.write_all(format!("GET /api/surface/events HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes())
        .expect("要求を書く");
    let mut seen = String::new();
    let state_log = place.seat_dir().join("state.jsonl");
    // GET の口を撃つ前は、acct の印の file が動いても知らせず、器も git も撃たない。
    append(&state_log, STATE_A);
    assert!(
        !wait_event(&mut s, &mut seen, "board-changed", Duration::from_millis(1500)),
        "acct の印を GET の前から見る"
    );
    assert!(place.calls("scribe2").is_empty(), "{:?}", place.calls("scribe2"));
    assert!(place.calls("git").is_empty(), "{:?}", place.calls("git"));
    // GET の口を撃つと一覧が取り直される（一覧の増えも板の変化）。
    account(addr);
    assert!(
        wait_event(&mut s, &mut seen, "board-changed", Duration::from_secs(5)),
        "一覧の取り直しが届かない"
    );
    wait_event(&mut s, &mut seen, "board-changed", Duration::from_millis(1000));
    seen.clear();
    // 印の file の字が変わると 5 秒以内に届く。
    for path in [
        state_log,
        place.seat_dir().join("tick-last"),
        place.state("state-a").join("fleet/events.jsonl"),
        place.states.join("scribe2-host/groups/g-a.account"),
    ] {
        append(&path, "\n");
        assert!(
            wait_event(&mut s, &mut seen, "board-changed", Duration::from_secs(5)),
            "{} の変化が 5 秒以内に届かない",
            path.display()
        );
    }
    // 取り直しは 60 秒ごと（最初の要求の後の 1 回だけで、要求ごとに撃ち直さない）。
    assert_eq!(ACCT_MARKS_EVERY, Duration::from_secs(60));
    place.clear_calls();
    thread::sleep(Duration::from_secs(6));
    assert!(
        place.calls("scribe2").is_empty(),
        "印の見張りが器を撃ち続ける: {:?}",
        place.calls("scribe2")
    );
}

/// (7) 足す外の依存は 0 本。
#[test]
fn acctwire_no_new_dependencies() {
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("Cargo.toml");
    let mut names = Vec::new();
    let mut inside = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line.ends_with("dependencies]");
            continue;
        }
        if inside
            && let Some((name, _)) = line.split_once('=')
            && !line.starts_with('#')
        {
            let name = name.trim();
            names.push(name.strip_suffix(".workspace").unwrap_or(name).to_string());
        }
    }
    assert_eq!(names, ["tsuzuri-contract", "tsuzuri-core"]);
}
