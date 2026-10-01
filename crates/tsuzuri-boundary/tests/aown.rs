//! 読むだけの server の歯（接頭辞 aown_・設計ノート surface-wave19h 行 e-ask-own-only）。
//! 見本は 1 問の台帳（surface/question-2.json・open の問い fx-ask.3 と、closed の問い fx-ask.1 とその裁定）。
//! 偽の bd は撃たれた回を bd.count に数えて見本を出す script。偽の bdw と偽の器は撃たれた回を数え、
//! 回ごとに argv を記録する script。
//! (1) は server を同じ process の thread で立て、(2) は tz を撃つ（127.0.0.1 の空き port・tz を止めてから標準 error を読む）。
#![cfg(test)]

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use tsuzuri_boundary::server::ruling::{self, READ_ONLY, REFUSED_LOG};
use tsuzuri_boundary::server::{Config, Server, batch, policy};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::question::QuestionList;
use tsuzuri_contract::surface::{
    BatchItem, BatchRequest, PolicyRequest, REVOKE_PATH, RevokeRequest, RulingId, RulingRequest,
};
use tsuzuri_contract::wire;

/// 1 問の見本（open の問い・closed の問いとその裁定）。
const ASK: &str = "surface/question-2.json";
const OPEN: &str = "fx-ask.3";
const CLOSED: &str = "fx-ask.1";
const CLOSED_RULING: &str = "fx-ask.1:20260924T0200Z-1";

/// 器の配達の口の target。
const TARGET: &str = "tsuzuri:0.1";

fn fixture(rel: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(rel),
    )
    .expect("fixture")
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 撃たれた回を `<log>/<name>.count` に数え、回ごとに argv を `<log>/<name>.<回>.args` に書いて rc 0 で終わる script。
fn recorder(path: &Path, log: &Path, name: &str) {
    let log = log.display();
    script(
        path,
        &format!(
            "n=$(( $(cat '{log}/{name}.count' 2>/dev/null || echo 0) + 1 ))\n\
             echo \"$n\" > '{log}/{name}.count'\n\
             for a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{log}/{name}.'\"$n\"'.args'\n\
             exit 0"
        ),
    );
}

/// 歯ごとの作業場（repo・面の file・state dir・記録の置き場・偽の program）。
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
            .join("aown")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, files, state, log) = (
            root.join("repo"),
            root.join("files"),
            root.join("state"),
            root.join("log"),
        );
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        for dir in [&files, &state, &log] {
            fs::create_dir_all(dir).expect("作業場の dir");
        }
        fs::write(files.join("index.html"), "tz").expect("index.html");
        fs::write(root.join("out.json"), fixture(ASK)).expect("偽の bd の出力");
        // 偽の bd は数えた後に見本を出す。
        let r = root.display();
        let l = log.display();
        script(
            &root.join("bd"),
            &format!(
                "n=$(( $(cat '{l}/bd.count' 2>/dev/null || echo 0) + 1 ))\n\
                 echo \"$n\" > '{l}/bd.count'\n\
                 exec cat '{r}/out.json'"
            ),
        );
        recorder(&root.join("bdw"), &log, "bdw");
        recorder(&root.join("scribe2"), &log, "scribe2");
        Place {
            root,
            repo,
            files,
            state,
            log,
        }
    }

    /// 偽の program（bd・bdw・scribe2）が撃たれた回の数。
    fn count(&self, name: &str) -> u32 {
        fs::read_to_string(self.log.join(format!("{name}.count")))
            .map_or(0, |c| c.trim().parse().expect("回の数"))
    }

    /// 席の target と state dir の両方を持つ（受ければ配達する）server の起動の引数。
    fn config(&self, read_only: bool) -> Config {
        Config {
            bd: self.root.join("bd").into(),
            state_dir: Some(self.state.clone()),
            bdw: self.root.join("bdw").into(),
            seat: Some(TARGET.to_string()),
            scribe2: self.root.join("scribe2").into(),
            read_only,
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        }
    }

    /// server を同じ process の thread で立てて口の住所を返す。
    fn serve(&self, read_only: bool) -> SocketAddr {
        let server = Server::bind(&self.config(read_only)).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }
}

fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("bead id")
}

/// 中核の問いの一覧で読んだ card の digest（見た版の要約値）。
fn digest(id: &str) -> String {
    let Reading::Known(cards) = tsuzuri_core::question::list(&fixture(ASK)).cards else {
        panic!("見本の台帳が Unknown");
    };
    cards
        .into_iter()
        .find(|c| c.id.as_str() == id)
        .unwrap_or_else(|| panic!("{id} の card"))
        .digest
}

fn ruling_request(seen_digest: String) -> String {
    wire::encode(&RulingRequest {
        question: bead(OPEN),
        seen_digest,
        verbatim: "はい".to_string(),
    })
    .expect("要求の電文")
}

/// 要求を 1 つ撃ち、接続が閉じるまで応答を読む（状態の code と本文）。
fn send(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, String) {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(30)))
        .expect("timeout");
    s.write_all(
        format!(
            "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .as_bytes(),
    )
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

/// 口 GET /api/questions の電文。
fn questions(addr: SocketAddr) -> QuestionList {
    let (status, body) = send(addr, "GET", "/api/questions", "");
    assert_eq!(status, 200, "{body}");
    wire::decode(&body).expect("問いの一覧の形")
}

/// (1) 読むだけの server は答えと方針の 4 つの口を 403 の read-only で断り、台帳の読みも書きも配達も撃たず、
/// 問いの一覧に答えを受けないと書く。読むだけでない server は答えを受けると書き、見た版の違う答えに 409 を返す。
#[test]
fn aown_read_only_refuses_without_writes() {
    assert_eq!(READ_ONLY, "read-only");
    let place = Place::new("in-process");
    assert!(
        !Config::new(
            place.repo.clone(),
            "127.0.0.1:0".parse().expect("bind 先"),
            place.files.clone()
        )
        .read_only,
        "Config::new の read_only が真"
    );

    let addr = place.serve(true);
    let list = questions(addr);
    assert!(!list.answerable, "読むだけの server の answerable が真");
    let Reading::Known(cards) = &list.cards else {
        panic!("問いの一覧が Unknown");
    };
    assert!(cards.iter().any(|c| c.id.as_str() == OPEN), "{cards:?}");
    let reads = place.count("bd");
    assert!(reads >= 1, "偽の bd を撃たない");
    refuse_then_answer(place, addr, reads);
}

/// 読むだけの server が 4 つの口を断ることと、読むだけでない server が答えを受けることを見る。
fn refuse_then_answer(place: Place, addr: SocketAddr, reads: u32) {
    let posts = [
        (ruling::PATH, ruling_request(digest(OPEN))),
        (
            batch::PATH,
            wire::encode(&BatchRequest {
                items: vec![BatchItem {
                    question: bead(OPEN),
                    seen_digest: digest(OPEN),
                    verbatim: None,
                }],
                verbatim: "はい".to_string(),
            })
            .expect("要求の電文"),
        ),
        (
            REVOKE_PATH,
            wire::encode(&RevokeRequest {
                question: bead(CLOSED),
                ruling: RulingId::new(CLOSED_RULING).expect("記帳 id"),
                verbatim: "待つ".to_string(),
            })
            .expect("要求の電文"),
        ),
        (
            policy::PATH,
            wire::encode(&PolicyRequest {
                scope: "all".to_string(),
                verbatim: "はい".to_string(),
            })
            .expect("要求の電文"),
        ),
    ];
    for (path, body) in &posts {
        let (status, reply) = send(addr, "POST", path, body);
        assert_eq!(status, 403, "{path}: {reply}");
        assert_eq!(reply.trim(), READ_ONLY, "{path}");
    }
    // 配達は別の thread なので、撃つなら撃ち終わる間を待ってから数える。
    thread::sleep(Duration::from_millis(300));
    assert_eq!(place.count("bd"), reads, "断りの前後で偽の bd の回が変わる");
    assert_eq!(place.count("bdw"), 0, "偽の bdw を撃つ");
    assert_eq!(place.count("scribe2"), 0, "偽の器を撃つ");

    // 読むだけでない server は答えを受け、見た版の違う答えに 409 を返す。
    let place = Place::new("answers");
    let addr = place.serve(false);
    assert!(
        questions(addr).answerable,
        "読むだけでない server の answerable が偽"
    );
    let (status, reply) = send(
        addr,
        "POST",
        ruling::PATH,
        &ruling_request("0000000000000000".to_string()),
    );
    assert_eq!(status, 409, "{reply}");
    assert_eq!(place.count("bdw"), 0, "見た版の違う答えで偽の bdw を撃つ");
}

/// tz surface serve を `extra` の引数を足して撃つ（標準 error は pipe）。
fn tz(place: &Place, extra: &[&str]) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tz"));
    cmd.args(["surface", "serve", "--bind", "127.0.0.1:0", "--repo"])
        .arg(&place.repo)
        .arg("--files")
        .arg(&place.files)
        .arg(format!("--bd={}", place.root.join("bd").display()))
        .arg(format!("--bdw={}", place.root.join("bdw").display()))
        .args(extra)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    cmd
}

/// (2) 引数 --read-only の tz は口 POST /api/ruling に 403 を返し、断りの 1 行を標準 error に書き、偽の bdw を撃たない。
/// --read-only を 2 度付けた tz は rc 1 で終わる。
#[test]
fn aown_cli_flag_and_log() {
    let place = Place::new("cli");
    let mut child = tz(&place, &["--read-only"]).spawn().expect("tz を撃つ");
    let mut err = BufReader::new(child.stderr.take().expect("標準 error"));
    let mut line = String::new();
    err.read_line(&mut line).expect("口の住所の行");
    let addr: Option<SocketAddr> = line
        .trim()
        .strip_prefix("tz surface serve: http://")
        .and_then(|a| a.strip_suffix('/'))
        .and_then(|a| a.parse().ok());
    let reply = addr.map(|addr| {
        std::panic::catch_unwind(|| send(addr, "POST", ruling::PATH, &ruling_request(digest(OPEN))))
    });
    let _ = child.kill();
    let _ = child.wait();
    let (status, body) = reply
        .unwrap_or_else(|| panic!("口の住所の行でない: {line}"))
        .expect("応答を読めない");
    assert_eq!(status, 403, "{body}");
    assert_eq!(body.trim(), READ_ONLY);
    let mut rest = String::new();
    err.read_to_string(&mut rest).expect("標準 error");
    let want = format!("{REFUSED_LOG}: /api/ruling 403 read-only・問い {OPEN}");
    let lines: Vec<&str> = rest.lines().filter(|l| *l == want).collect();
    assert_eq!(lines.len(), 1, "断りの行が 1 行でない: {rest}");
    assert_eq!(place.count("bdw"), 0, "偽の bdw を撃つ");

    let twice = tz(&place, &["--read-only", "--read-only"])
        .stderr(Stdio::null())
        .status()
        .expect("tz を撃つ");
    assert_eq!(twice.code(), Some(1), "--read-only を 2 度付けた tz の rc");
}
