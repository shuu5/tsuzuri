//! 問いの一覧の口と裁定の受付の口の歯（接頭辞 server_ask_・設計ノート surface-base 便 e-ask の完了の条件）。
//! 偽の bd は作業場の out.json（既定は fixture の question-2.json）を標準出力へ出す script、
//! 偽の bdw と偽の器は受けた argv を記録の置き場（repo と state dir の外）に 1 行ずつ書く script。
//! server は同じ process の thread で 127.0.0.1 の空き port に立てる（標準 error を見る歯だけ tz を撃つ）。

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tsuzuri_boundary::server::ledger::epoch_secs;
use tsuzuri_boundary::server::{Config, Server, ledger, ruling};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BDW, BeadId, LedgerWrite};
use tsuzuri_contract::question::{QuestionCard, QuestionList};
use tsuzuri_contract::surface::{Refusal, RefusalResponse, RulingRequest, RulingResponse};
use tsuzuri_contract::wire;
use tsuzuri_core::delivery::{Route, mark_line};

const FIXTURE: &str = "surface/question-2.json";

/// 定型行を 4 つ持つ open の問い・定型行の無い open の問い・closed の問い。
const WITH_LINES: &str = "fx-ask.3";
const WITHOUT_LINES: &str = "fx-ask.2";
const CLOSED: &str = "fx-ask.1";

fn read_fixture() -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(FIXTURE),
    )
    .expect("fixture")
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 撃たれた回ごとに argv と cwd を `<log>/<name>.<回>.args|cwd` に書き、`<log>/<name>.fail` の回なら rc 1 で終わる script。
fn recorder(path: &Path, log: &Path, name: &str) {
    let log = log.display();
    script(
        path,
        &format!(
            "n=$(( $(cat '{log}/{name}.count' 2>/dev/null || echo 0) + 1 ))\n\
             echo \"$n\" > '{log}/{name}.count'\n\
             for a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{log}/{name}.'\"$n\"'.args'\n\
             pwd -P > '{log}/{name}.'\"$n\"'.cwd'\n\
             if [ \"$n\" = \"$(cat '{log}/{name}.fail' 2>/dev/null)\" ]; then echo 落ちた >&2; exit 1; fi\n\
             exit 0"
        ),
    );
}

/// 歯ごとの作業場（repo・面の file・state dir・偽の program・記録の置き場）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
    state: PathBuf,
    log: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("server_ask")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, files, state, log) = (
            root.join("repo"),
            root.join("files"),
            root.join("state"),
            root.join("log"),
        );
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::create_dir_all(&files).expect("面の file の置き場");
        fs::create_dir_all(state.join("fleet")).expect("state dir");
        fs::create_dir_all(&log).expect("記録の置き場");
        fs::write(files.join("index.html"), "tz").expect("index.html");
        for mark in ["issues.jsonl", "interactions.jsonl"] {
            fs::write(repo.join(".beads").join(mark), "{}\n").expect("印の file");
        }
        fs::write(state.join("fleet/events.jsonl"), "").expect("event log");
        script(
            &root.join("bd"),
            &format!("exec cat '{}'", root.join("out.json").display()),
        );
        recorder(&root.join("bdw"), &log, "bdw");
        recorder(&root.join("scribe2"), &log, "scribe2");
        let place = Place {
            root,
            repo,
            files,
            state,
            log,
        };
        place.bd_returns(&read_fixture());
        place
    }

    /// 偽の bd が返す字を置く。
    fn bd_returns(&self, text: &str) {
        fs::write(self.root.join("out.json"), text).expect("偽の bd の出力");
    }

    /// 偽の bd を落とす（出す file が無いので rc 1）。
    fn bd_fails(&self) {
        fs::remove_file(self.root.join("out.json")).expect("偽の bd の出力を消す");
    }

    /// 偽の program（bdw か scribe2）を `n` 回目で落とす。
    fn fail_at(&self, name: &str, n: u32) {
        fs::write(self.log.join(format!("{name}.fail")), n.to_string()).expect("落とす回");
    }

    /// 偽の program が撃たれた回ごとの（argv・cwd）。
    fn calls(&self, name: &str) -> Vec<(Vec<String>, String)> {
        let count: u32 = fs::read_to_string(self.log.join(format!("{name}.count")))
            .map_or(0, |c| c.trim().parse().expect("回の数"));
        (1..=count)
            .map(|n| {
                let read = |ext: &str| {
                    fs::read_to_string(self.log.join(format!("{name}.{n}.{ext}"))).expect("記録")
                };
                (
                    read("args").lines().map(str::to_string).collect(),
                    read("cwd"),
                )
            })
            .collect()
    }

    fn config(&self, seat: Option<&str>, state_dir: bool) -> Config {
        Config {
            bd: self.root.join("bd").into(),
            state_dir: state_dir.then(|| self.state.clone()),
            bdw: self.root.join("bdw").into(),
            seat: seat.map(str::to_string),
            scribe2: self.root.join("scribe2").into(),
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        }
    }

    fn serve_with(&self, config: &Config) -> SocketAddr {
        let server = Server::bind(config).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }

    /// 席の target も state dir も持つ server。
    fn serve(&self) -> SocketAddr {
        self.serve_with(&self.config(Some("tsuzuri:0.1"), true))
    }

    fn repo_real(&self) -> String {
        format!(
            "{}\n",
            self.repo.canonicalize().expect("repo の実体").display()
        )
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_secs()
}

struct Reply {
    status: u16,
    head: String,
    body: String,
}

/// 要求を 1 つ撃ち、接続が閉じるまで応答を読む。
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

/// 本文と足す頭の行で要求を撃つ。
fn send(addr: SocketAddr, method: &str, path: &str, heads: &str, body: &str) -> Reply {
    request(
        addr,
        format!(
            "{method} {path} HTTP/1.1\r\nHost: {addr}\r\n{heads}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        )
        .as_bytes(),
    )
}

fn ruling_body(question: &str, digest: &str, verbatim: &str) -> String {
    wire::encode(&RulingRequest {
        question: BeadId::new(question).expect("bead id"),
        seen_digest: digest.to_string(),
        verbatim: verbatim.to_string(),
    })
    .expect("要求の電文")
}

fn post(addr: SocketAddr, question: &str, digest: &str, verbatim: &str) -> Reply {
    send(
        addr,
        "POST",
        ruling::PATH,
        "",
        &ruling_body(question, digest, verbatim),
    )
}

/// 中核の関数で読んだ card（台帳の字は偽の bd が返す字と同じ）。
fn card(ledger: &str, id: &str) -> QuestionCard {
    let Reading::Known(cards) = tsuzuri_core::question::list(ledger).cards else {
        panic!("読める台帳が Unknown");
    };
    cards
        .into_iter()
        .find(|c| c.id.as_str() == id)
        .unwrap_or_else(|| panic!("{id} の card"))
}

fn recorded(reply: &Reply) -> RulingResponse {
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert!(
        reply.head.contains("Content-Type: application/json"),
        "{}",
        reply.head
    );
    wire::decode(&reply.body).unwrap_or_else(|e| panic!("応答の形でない {e}: {}", reply.body))
}

fn refused(reply: &Reply) -> Refusal {
    wire::decode::<RefusalResponse>(&reply.body)
        .unwrap_or_else(|e| panic!("断りの形でない {e}: {}", reply.body))
        .reason
}

/// id の分（`<問いの id>:<分>-<数>` の分）を epoch 秒にする（歯の側で独立に読む）。
fn minute_secs(minute: &str) -> u64 {
    let b = minute.as_bytes();
    assert!(
        b.len() == 14 && b[8] == b'T' && b[13] == b'Z',
        "分の形でない: {minute}"
    );
    let rfc = format!(
        "{}-{}-{}T{}:{}:00Z",
        &minute[0..4],
        &minute[4..6],
        &minute[6..8],
        &minute[9..11],
        &minute[11..13]
    );
    epoch_secs(&rfc).unwrap_or_else(|| panic!("分の形でない: {minute}"))
}

/// id を（問いの id・分・数）に分ける。
fn split_id(id: &str) -> (&str, &str, u32) {
    let (question, rest) = id.split_once(':').expect("id の「:」");
    let (minute, n) = rest.rsplit_once('-').expect("id の「-」");
    (question, minute, n.parse().expect("数"))
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
fn server_ask_questions_route_matches_core() {
    let place = Place::new("list");
    let addr = place.serve();
    let reply = send(addr, "GET", "/api/questions", "", "");
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert!(
        reply.head.contains("Content-Type: application/json"),
        "{}",
        reply.head
    );
    let got: QuestionList = wire::decode(&reply.body).expect("問いの一覧の形");
    assert_eq!(got, tsuzuri_core::question::list(&read_fixture()));
    let Reading::Known(cards) = &got.cards else {
        panic!("読める台帳が Unknown");
    };
    let ids: Vec<&str> = cards.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, [WITHOUT_LINES, WITH_LINES]);

    // 読めた後に偽の bd が落ちれば、最後に読めた一覧と、最後に読めた時からの秒の頭（行 e-hold）。
    // 口は見張りの読みの字を返すので、印を動かして見張りの読みを待つ（行 e-snap）。
    place.bd_fails();
    fs::write(place.repo.join(".beads/issues.jsonl"), "{}\n{}\n").expect("印の file");
    let until = Instant::now() + Duration::from_millis(2500);
    let reply = loop {
        let reply = send(addr, "GET", "/api/questions", "", "");
        let aged = reply.head.to_ascii_lowercase().contains("x-tz-read-age");
        if aged || Instant::now() >= until {
            break reply;
        }
        thread::sleep(Duration::from_millis(50));
    };
    assert!(
        reply.head.to_ascii_lowercase().contains("x-tz-read-age"),
        "{}",
        reply.head
    );
    assert_eq!(reply.status, 200, "{}", reply.body);
    let got: QuestionList = wire::decode(&reply.body).expect("問いの一覧の形");
    assert_eq!(got, tsuzuri_core::question::list(&read_fixture()));

    // 一度も読めていない server は 200 で「まだ分からない」。
    let place = Place::new("list-down");
    place.bd_fails();
    let addr = place.serve();
    let reply = send(addr, "GET", "/api/questions", "", "");
    assert_eq!(reply.status, 200, "{}", reply.body);
    let got: QuestionList = wire::decode(&reply.body).expect("問いの一覧の形");
    assert_eq!(got.cards, Reading::Unknown);
}

#[test]
fn server_ask_ruling_writes_twice() {
    let place = Place::new("write");
    let addr = place.serve();
    let digest = card(&read_fixture(), WITH_LINES).digest;
    let from = now();
    let got = recorded(&post(addr, WITH_LINES, &digest, "減らしてよい"));
    let to = now();
    assert!(
        (from..=to).contains(&got.recorded_at),
        "記帳時刻は受付の時刻: {} not in {from}..={to}",
        got.recorded_at
    );
    let id = got.ruling.as_str();
    let (question, minute, n) = split_id(id);
    assert_eq!((question, n), (WITH_LINES, 1), "{id}");
    let at = minute_secs(minute);
    assert!(
        at <= got.recorded_at && got.recorded_at < at + 60,
        "分は受付の時刻の UTC の分: {id} {}",
        got.recorded_at
    );

    let calls = place.calls("bdw");
    assert_eq!(calls.len(), 3, "偽の bdw は 3 回だけ（追記・閉じる・印）: {calls:?}");
    let q = BeadId::new(WITH_LINES).expect("bead id");
    let append = LedgerWrite::AppendNotes {
        id: q.clone(),
        line: format!("裁定 id = {id}・問い = {WITH_LINES}・逐語 = 減らしてよい"),
    };
    let close = LedgerWrite::CloseItem {
        id: q,
        reason: format!("裁定 {id}"),
    };
    assert_eq!(calls[0].0, append.argv(), "1 回目は notes への追記");
    assert_eq!(calls[1].0, close.argv(), "2 回目は問いを閉じる");
    for (_, cwd) in &calls {
        assert_eq!(cwd, &place.repo_real(), "cwd は repo の置き場");
    }
    assert_eq!(BDW, "bdw", "program の名の既定");
}

#[test]
fn server_ask_id_counts_up_in_same_minute() {
    for _ in 0..3 {
        let place = Place::new("count");
        let minute = ruling::minute(now());
        let taken = format!(
            "見本の notes の 1 行\\n裁定 id = {WITH_LINES}:{minute}-1・問い = {WITH_LINES}・逐語 = 前\\n裁定 id = {WITH_LINES}:{minute}-2・問い = {WITH_LINES}・逐語 = 前の前"
        );
        let ledger = read_fixture().replace("見本の notes の 1 行", &taken);
        assert_ne!(ledger, read_fixture(), "notes を書き換えた");
        place.bd_returns(&ledger);
        let addr = place.serve();
        let digest = card(&ledger, WITH_LINES).digest;
        let got = recorded(&post(addr, WITH_LINES, &digest, "はい"));
        let (_, got_minute, n) = split_id(got.ruling.as_str());
        if got_minute != minute {
            continue; // 分を跨いだ（撃ち直す）。
        }
        assert_eq!(n, 3, "同じ分の id の定型行が 2 つ在れば 3: {}", got.ruling);
        return;
    }
    panic!("3 回とも分を跨いだ");
}

#[test]
fn server_ask_refusals_write_nothing() {
    let place = Place::new("refuse");
    let addr = place.serve();
    let fixture = read_fixture();
    let digest = card(&fixture, WITH_LINES).digest;
    for (question, digest, verbatim, want) in [
        (
            WITH_LINES,
            digest.as_str(),
            " \n\t　",
            Refusal::EmptyVerbatim,
        ),
        (WITH_LINES, digest.as_str(), "", Refusal::EmptyVerbatim),
        (
            "fx-ask.9",
            digest.as_str(),
            "はい",
            Refusal::UnknownQuestion,
        ),
        (CLOSED, digest.as_str(), "はい", Refusal::UnknownQuestion),
        (
            WITH_LINES,
            "0000000000000000",
            "はい",
            Refusal::StaleVersion,
        ),
        (
            WITH_LINES,
            &card(&fixture, WITHOUT_LINES).digest,
            "はい",
            Refusal::StaleVersion,
        ),
    ] {
        let reply = post(addr, question, digest, verbatim);
        let status = match want {
            Refusal::EmptyVerbatim => 400,
            Refusal::UnknownQuestion => 404,
            _ => 409,
        };
        assert_eq!(
            reply.status, status,
            "{question} {verbatim:?}: {}",
            reply.body
        );
        assert_eq!(refused(&reply), want, "{question} {verbatim:?}");
    }
    // 台帳が読めなければ 503。
    place.bd_fails();
    let reply = post(addr, WITH_LINES, &digest, "はい");
    assert_eq!(reply.status, 503, "{}", reply.body);
    assert!(place.calls("bdw").is_empty(), "断りで偽の bdw を撃つ");
    assert!(place.calls("scribe2").is_empty(), "断りで偽の器を撃つ");
}

#[test]
fn server_ask_write_failures_are_502() {
    let digest = card(&read_fixture(), WITH_LINES).digest;
    // 1 回目が落ちれば 2 回目を撃たない。
    let place = Place::new("fail-1");
    place.fail_at("bdw", 1);
    let addr = place.serve();
    let reply = post(addr, WITH_LINES, &digest, "はい");
    assert_eq!(reply.status, 502, "{}", reply.body);
    assert_eq!(place.calls("bdw").len(), 1);
    assert!(
        place.calls("scribe2").is_empty(),
        "落ちた書きの後に配達する"
    );
    // 2 回目が落ちれば 502 で、応答の字に発行した id。
    let place = Place::new("fail-2");
    place.fail_at("bdw", 2);
    let addr = place.serve();
    let reply = post(addr, WITH_LINES, &digest, "はい");
    assert_eq!(reply.status, 502, "{}", reply.body);
    let calls = place.calls("bdw");
    assert_eq!(calls.len(), 2);
    let line = calls[0].0.last().expect("追記の行");
    let id = line
        .strip_prefix("--append-notes=裁定 id = ")
        .and_then(|r| r.split('・').next())
        .unwrap_or_else(|| panic!("追記の行の形でない: {line}"));
    assert!(id.starts_with("fx-ask.3:"), "{id}");
    assert!(
        reply.body.contains(id),
        "応答の字に id が無い: {}",
        reply.body
    );
    assert!(
        place.calls("scribe2").is_empty(),
        "落ちた書きの後に配達する"
    );
}

#[test]
fn server_ask_verbatim_is_one_line() {
    let place = Place::new("escape");
    let addr = place.serve();
    let digest = card(&read_fixture(), WITH_LINES).digest;
    let got = recorded(&post(
        addr,
        WITH_LINES,
        &digest,
        "一行目\n二行目 \\ 逆斜線\n",
    ));
    let calls = place.calls("bdw");
    assert_eq!(calls.len(), 3);
    assert_eq!(
        calls[0].0.len(),
        3,
        "argv の字に改行が残る: {:?}",
        calls[0].0
    );
    assert_eq!(
        calls[0].0[2],
        format!(
            "--append-notes=裁定 id = {}・問い = {WITH_LINES}・逐語 = 一行目\\n二行目 \\\\ 逆斜線\\n",
            got.ruling
        )
    );
}

#[test]
fn server_ask_guards_write_nothing() {
    let place = Place::new("guard");
    let addr = place.serve();
    let digest = card(&read_fixture(), WITH_LINES).digest;
    let body = ruling_body(WITH_LINES, &digest, "はい");
    // POST を受けるのは /api/ruling だけ。
    for path in ["/api/questions", "/api/ledger", "/api/ruling/x", "/"] {
        let reply = send(addr, "POST", path, "", &body);
        assert_eq!(reply.status, 405, "POST {path}");
        assert!(reply.head.contains("Allow: GET"), "{}", reply.head);
    }
    for method in ["PUT", "DELETE", "PATCH"] {
        let reply = send(addr, method, ruling::PATH, "", &body);
        assert_eq!(reply.status, 405, "{method}");
    }
    // 本文が 65536 byte を越えれば 413（逐語を伸ばした正しい形の要求）。
    let big = ruling_body(WITH_LINES, &digest, &"あ".repeat(22_000));
    assert!(big.len() > 65_536);
    assert_eq!(send(addr, "POST", ruling::PATH, "", &big).status, 413);
    // Origin の host と port が Host と違えば 403。
    let port = addr.port();
    for origin in [
        format!("http://evil.example:{port}"),
        format!("http://127.0.0.1:{}", port.wrapping_add(1)),
        "null".to_string(),
    ] {
        let reply = send(
            addr,
            "POST",
            ruling::PATH,
            &format!("Origin: {origin}\r\n"),
            &body,
        );
        assert_eq!(reply.status, 403, "{origin}: {}", reply.body);
    }
    // 読めない本文は 400。
    assert_eq!(send(addr, "POST", ruling::PATH, "", "{").status, 400);
    assert!(place.calls("bdw").is_empty(), "守りで偽の bdw を撃つ");
    // 同じ Origin は通る。
    let reply = send(
        addr,
        "POST",
        ruling::PATH,
        &format!("Origin: http://{addr}\r\n"),
        &body,
    );
    recorded(&reply);
    assert_eq!(place.calls("bdw").len(), 3);
}

#[test]
fn server_ask_delivers_only_with_seat_and_state_dir() {
    let digest = card(&read_fixture(), WITH_LINES).digest;
    let place = Place::new("deliver");
    let addr = place.serve();
    let from = now();
    let got = recorded(&post(addr, WITH_LINES, &digest, "はい"));
    let to = now();
    let calls = place.calls("scribe2");
    assert_eq!(calls.len(), 1, "偽の器は 1 回: {calls:?}");
    let state = place.state.display().to_string();
    assert_eq!(
        calls[0].0,
        [
            "seat",
            "deliver",
            "--state-dir",
            state.as_str(),
            "--target",
            "tsuzuri:0.1",
            "--ruling",
            got.ruling.as_str()
        ]
    );
    let bdw = place.calls("bdw");
    assert_eq!(bdw.len(), 3, "配達は書きの後・印は配達の後: {bdw:?}");
    let marks: Vec<Vec<String>> = [ruling::minute(from), ruling::minute(to)]
        .iter()
        .map(|m| {
            LedgerWrite::AppendNotes {
                id: BeadId::new(WITH_LINES).expect("bead id"),
                line: mark_line(&got.ruling, Route::Deliver, m),
            }
            .argv()
        })
        .collect();
    assert!(
        marks.contains(&bdw[2].0),
        "3 回目は配達の口の印: {:?}",
        bdw[2].0
    );
    // 片方でも無ければ撃たない。
    for (name, seat, state_dir) in [
        ("no-seat", None, true),
        ("no-state", Some("tsuzuri:0.1"), false),
        ("neither", None, false),
    ] {
        let place = Place::new(name);
        let addr = place.serve_with(&place.config(seat, state_dir));
        recorded(&post(addr, WITH_LINES, &digest, "はい"));
        assert_eq!(place.calls("bdw").len(), 2, "{name}");
        assert!(place.calls("scribe2").is_empty(), "{name}: 偽の器を撃つ");
    }
    assert_eq!(ruling::SCRIBE2, "scribe2", "program の名の既定");
}

/// tz を撃ち、標準 error の最初の行から口の住所を読む（残りの標準 error の読み手も返す）。
fn spawn_tz(
    place: &Place,
    extra: &[String],
) -> (
    std::process::Child,
    SocketAddr,
    BufReader<std::process::ChildStderr>,
) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["surface", "serve", "--bind", "127.0.0.1:0", "--repo"])
        .arg(&place.repo)
        .arg("--files")
        .arg(&place.files)
        .arg(format!("--bd={}", place.root.join("bd").display()))
        .args(extra)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("tz を撃つ");
    let mut err = BufReader::new(child.stderr.take().expect("標準 error"));
    let mut line = String::new();
    err.read_line(&mut line).expect("口の住所の行");
    let addr = line
        .trim()
        .strip_prefix("tz surface serve: http://")
        .and_then(|a| a.strip_suffix('/'))
        .and_then(|a| a.parse().ok())
        .unwrap_or_else(|| panic!("口の住所の行でない: {line}"));
    (child, addr, err)
}

#[test]
fn server_ask_failed_delivery_stays_200_with_stderr_line() {
    let place = Place::new("deliver-fail");
    place.fail_at("scribe2", 1);
    let digest = card(&read_fixture(), WITH_LINES).digest;
    let (mut child, addr, mut err) = spawn_tz(
        &place,
        &[
            format!("--bdw={}", place.root.join("bdw").display()),
            "--seat".to_string(),
            "tsuzuri:0.1".to_string(),
            format!("--state-dir={}", place.state.display()),
            format!("--scribe2={}", place.root.join("scribe2").display()),
        ],
    );
    let reply = std::panic::catch_unwind(|| post(addr, WITH_LINES, &digest, "はい"));
    let _ = child.kill();
    let _ = child.wait();
    let reply = reply.expect("応答を読めない");
    let got = recorded(&reply);
    assert_eq!(place.calls("scribe2").len(), 1);
    let mut rest = String::new();
    err.read_to_string(&mut rest).expect("標準 error");
    let lines: Vec<&str> = rest
        .lines()
        .filter(|l| l.contains(got.ruling.as_str()))
        .collect();
    assert_eq!(lines.len(), 1, "受けない配達の行が 1 行でない: {rest}");
    assert!(lines[0].contains(ruling::NOT_TAKEN), "{rest}");
    assert!(!rest.contains("配達が落ちた"), "{rest}");
    assert_eq!(place.calls("bdw").len(), 2, "受けなければ印を置かない");

    // 空の値は使い方の誤りで rc 1。
    for bad in ["--bdw=", "--seat=", "--scribe2="] {
        let out = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(["surface", "serve", "--bind", "127.0.0.1:0", "--repo"])
            .arg(&place.repo)
            .arg("--files")
            .arg(&place.files)
            .arg(bad)
            .output()
            .expect("tz を撃つ");
        assert_eq!(out.status.code(), Some(1), "{bad}: {out:?}");
    }
}

#[test]
fn server_ask_repo_and_state_bytes_unchanged() {
    let place = Place::new("bytes");
    let addr = place.serve();
    let before = (tree(&place.repo), tree(&place.state));
    let digest = card(&read_fixture(), WITH_LINES).digest;
    assert_eq!(send(addr, "GET", "/api/questions", "", "").status, 200);
    assert_eq!(
        post(addr, WITH_LINES, "0000000000000000", "はい").status,
        409
    );
    assert_eq!(post(addr, CLOSED, &digest, "はい").status, 404);
    recorded(&post(addr, WITH_LINES, &digest, "はい"));
    assert_eq!(place.calls("bdw").len(), 3);
    assert_eq!(place.calls("scribe2").len(), 1);
    assert!(
        before == (tree(&place.repo), tree(&place.state)),
        "受付と一覧の後に repo か state dir の byte が変わる"
    );
}

#[test]
fn server_ask_no_new_dependencies() {
    let deps = |krate: &str| -> Vec<String> {
        let text = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join(krate)
                .join("Cargo.toml"),
        )
        .expect("Cargo.toml");
        let mut names = Vec::new();
        let mut inside = false;
        for line in text.lines().map(str::trim) {
            if line.starts_with('[') {
                inside = line.ends_with("dependencies]");
                continue;
            }
            // `名 = …` か `名.workspace = true` の名。
            if inside
                && let Some((name, _)) = line.split_once('=')
                && !line.starts_with('#')
            {
                let name = name.trim();
                names.push(name.strip_suffix(".workspace").unwrap_or(name).to_string());
            }
        }
        names
    };
    assert_eq!(
        deps("tsuzuri-boundary"),
        ["tsuzuri-contract", "tsuzuri-core"]
    );
    assert_eq!(
        deps("tsuzuri-core"),
        ["serde", "serde_json", "tsuzuri-contract"]
    );
    // 台帳の読みは着地済みの bd の口のまま。
    assert_eq!(ledger::BD, "bd");
}
