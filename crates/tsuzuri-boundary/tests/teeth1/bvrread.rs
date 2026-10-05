//! 方針の口が台帳の読みの落ちで断る前の読み直しと断りの log の歯（接頭辞 bvrread_・設計ノート surface-wave26b 行 c-ruling-reread）。
//! 偽の bd は撃たれた回を数え、作業場の bd.fails の数の回までは標準エラーに 1 行 `database is locked` を書いて rc 1 で終わり、
//! その後の回は out.json（fixture tests/fixtures/ledger/bd-list-8.json の写し）を出す script。
//! 偽の bdw は撃たれた回を数えて rc 1 で終わる（どの歯も書きの前で終わる）。
//! 方針の受付は `policy::accept` を直に撃ち、口の断りの log は tz の binary を撃って標準エラーを読む。
#![cfg(test)]

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::common::{fixture, script};
use tsuzuri_boundary::server::ledger::Source;
use tsuzuri_boundary::server::policy::{self, Outcome};
use tsuzuri_boundary::server::ruling::{READ_TRIES, REFUSED_LOG, UNREAD, Writer};
use tsuzuri_contract::surface::PolicyRequest;
use tsuzuri_contract::wire;

/// 偽の bd の出力の見本。
const LEDGER: &str = "ledger/bd-list-8.json";

/// 落ちる回の偽の bd が標準エラーに書く 1 行。
const LOCKED: &str = "database is locked";

/// 落ち続ける偽の bd の落ちる回の数。
const ALWAYS: u32 = 99;

/// 歯ごとの作業場（repo・面の file の置き場・偽の bd と bdw）。
struct Place {
    root: PathBuf,
}

impl Place {
    /// 最初の `fails` 回は落ちる偽の bd の作業場。
    fn new(name: &str, fails: u32) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("bvrread")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        for dir in [root.join("repo/.beads"), root.join("files")] {
            fs::create_dir_all(dir).expect("置き場");
        }
        fs::write(root.join("files/index.html"), "tz").expect("index.html");
        fs::write(root.join("out.json"), fixture(LEDGER)).expect("偽の bd の出力");
        fs::write(root.join("bd.fails"), fails.to_string()).expect("落ちる回の数");
        let r = root.display();
        script(
            &root.join("bd"),
            &format!(
                "n=$(( $(cat '{r}/bd.count' 2>/dev/null || echo 0) + 1 ))\n\
                 echo \"$n\" > '{r}/bd.count'\n\
                 if [ \"$n\" -le \"$(cat '{r}/bd.fails')\" ]; then echo '{LOCKED}' >&2; exit 1; fi\n\
                 exec cat '{r}/out.json'"
            ),
        );
        script(
            &root.join("bdw"),
            &format!(
                "n=$(( $(cat '{r}/bdw.count' 2>/dev/null || echo 0) + 1 ))\n\
                 echo \"$n\" > '{r}/bdw.count'\n\
                 exit 1"
            ),
        );
        Place { root }
    }

    fn source(&self) -> Source {
        Source::new(self.root.join("repo"), self.root.join("bd"))
    }

    fn writer(&self) -> Writer {
        Writer {
            repo: self.root.join("repo"),
            bdw: self.root.join("bdw").into(),
            delivery: None,
        }
    }

    /// 偽の program が撃たれた回の数。
    fn count(&self, name: &str) -> u32 {
        fs::read_to_string(self.root.join(format!("{name}.count")))
            .map_or(0, |c| c.trim().parse().expect("回の数"))
    }
}

/// POST を 1 つ撃ち、（状態の code・本文）を返す。
fn post(addr: SocketAddr, path: &str, body: &str) -> (u16, String) {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(60)))
        .expect("timeout");
    s.write_all(
        format!(
            "POST {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
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

/// 範囲が open の問いでも all でもない方針の要求（台帳が読めれば書かずに BadScope で断る）。
fn bad_scope() -> PolicyRequest {
    PolicyRequest {
        scope: "fx-none".to_string(),
        verbatim: "方針".to_string(),
    }
}

/// (3) 方針の受付は 2 回落ちる台帳の読みを撃ち直して 3 回目に読み、台帳が読めた後の判じ（BadScope）まで進む。
/// 落ち続ける台帳は READ_TRIES 回撃ってから LedgerUnknown で断る。どちらも偽の bdw を撃たない。
#[test]
fn bvrread_policy_rereads_before_refusing() {
    let place = Place::new("policy-late", 2);
    let got = policy::accept(&bad_scope(), &place.source(), &place.writer(), 0);
    assert_eq!(got, Outcome::BadScope);
    assert_eq!(place.count("bd"), 3, "偽の bd は 3 回");
    assert_eq!(place.count("bdw"), 0, "書きを撃たない");

    let place = Place::new("policy-down", ALWAYS);
    let got = policy::accept(&bad_scope(), &place.source(), &place.writer(), 0);
    assert_eq!(got, Outcome::LedgerUnknown);
    assert_eq!(place.count("bd"), READ_TRIES, "偽の bd は READ_TRIES 回");
    assert_eq!(place.count("bdw"), 0, "書きを撃たない");
}

/// (4) 落ち続ける台帳の tz に撃った口 POST /api/policy は 503 と本文 ledger-unknown を返し、標準エラーに
/// 回ごとの落ちた訳を並べた読みの落ちの 1 行と、問いの id の無い断りの 1 行を 1 行ずつ書き、偽の bdw を撃たない。
#[test]
fn bvrread_policy_refusal_logged() {
    let place = Place::new("policy-log", ALWAYS);
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["surface", "serve", "--bind", "127.0.0.1:0", "--repo"])
        .arg(place.root.join("repo"))
        .arg("--files")
        .arg(place.root.join("files"))
        .arg(format!("--bd={}", place.root.join("bd").display()))
        .arg(format!("--bdw={}", place.root.join("bdw").display()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("tz を撃つ");
    let mut err = BufReader::new(child.stderr.take().expect("標準 error"));
    let mut line = String::new();
    err.read_line(&mut line).expect("口の住所の行");
    let addr: Option<SocketAddr> = line
        .trim()
        .strip_prefix("tz surface serve: http://")
        .and_then(|a| a.strip_suffix('/'))
        .and_then(|a| a.parse().ok());
    let body = wire::encode(&PolicyRequest {
        scope: "all".to_string(),
        verbatim: "方針".to_string(),
    })
    .expect("要求の電文");
    let reply = addr.map(|addr| std::panic::catch_unwind(|| post(addr, policy::PATH, &body)));
    let _ = child.kill();
    let _ = child.wait();
    let (status, text) = reply
        .unwrap_or_else(|| panic!("口の住所の行でない: {line}"))
        .expect("応答を読めない");
    assert_eq!((status, text.trim()), (503, "ledger-unknown"));
    let mut rest = String::new();
    err.read_to_string(&mut rest).expect("標準 error");
    let down = format!("rc が 0 でない: {LOCKED}");
    let unread = format!("tz surface serve: {UNREAD}: {down}・{down}・{down}");
    let refused = format!("{REFUSED_LOG}: /api/policy 503 ledger-unknown・問い ");
    for want in [&unread, &refused] {
        let lines = rest.lines().filter(|l| l == want).count();
        assert_eq!(lines, 1, "{want} が 1 行でない: {rest}");
    }
    assert_eq!(place.count("bdw"), 0, "偽の bdw を撃つ");
}
