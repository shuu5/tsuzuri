//! 答えの口の読みの撃ち直し・応答の後の配達・断りの log の歯（接頭辞 eretry_・設計ノート surface-wave19f 行 e-ruling-retry）。
//! 受付は server を立てず、`ruling::accept`・`ruling::revoke`・`batch::accept` を受付の時刻 `NOW` で直に撃つ
//! （server の台帳の見張りの読みが偽の bd の回を数えに混ぜないように）。
//! 偽の bd は作業場の bd.fails の数の回までは rc 1 で落ち、その後は out.json を出す script。
//! 偽の bdw と偽の器は回ごとに argv を記録し、argv を書き終えたら回ごとの done の file を置く script で、
//! 偽の器は答えの口の回（引数の頭が seat と ruling と answer）に標準入力を記録して裁定 id を出し、
//! 配達の回（引数の頭が seat と deliver）に門の file gate を 9 秒まで待ってから終わりの印 scribe2.end を置く。
//! log の歯だけ tz を撃つ（127.0.0.1 の空き port・tz を止めてから標準 error を読む）。
#![cfg(test)]

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::common::{TARGET, bead, script};
use tsuzuri_boundary::server::batch;
use tsuzuri_boundary::server::ledger::Source;
use tsuzuri_boundary::server::ruling::{
    self, Delivery, Outcome, PATH, READ_TRIES, REFUSED_LOG, RETRY_STEP, Revoked, Vessel,
    WRITE_TIMEOUT, Writer,
};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::surface::{BatchItem, BatchRequest, RevokeRequest, RulingId, RulingRequest};
use tsuzuri_contract::wire;

/// 受付の時刻（2026-09-28T04:41:30Z）。
const NOW: u64 = 1_790_570_490;

/// NOW の分。
const MINUTE: &str = "20260928T0441Z";

/// 1 問の見本（open の問い・closed の問いとその裁定）。
const ASK: &str = "surface/question-2.json";
const OPEN: &str = "fx-ask.3";
const CLOSED: &str = "fx-ask.1";
const CLOSED_RULING: &str = "fx-ask.1:20260924T0200Z-1";

/// 束の見本（open の問い 2 本）。
const BATCH: &str = "surface/question-batch.json";
const ROWS: [&str; 2] = ["fx-b.2", "fx-b.3"];

/// log に出ない逐語。
const SECRET: &str = "秘めた言葉は外へ出さない";

fn fixture(rel: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(rel),
    )
    .expect("fixture")
}

/// 撃たれた回ごとに argv を `<log>/<name>.<回>.args` に書き、書き終えたら `<log>/<name>.<回>.done` を置き、
/// `then` を撃って rc 0 で終わる script。
fn recorder(path: &Path, log: &Path, name: &str, then: &str) {
    let log = log.display();
    script(
        path,
        &format!(
            "n=$(( $(cat '{log}/{name}.count' 2>/dev/null || echo 0) + 1 ))\n\
             echo \"$n\" > '{log}/{name}.count'\n\
             for a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{log}/{name}.'\"$n\"'.args'\n\
             : > '{log}/{name}.'\"$n\"'.done'\n\
             {then}\n\
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
    /// 偽の器の program。
    vessel: PathBuf,
}

impl Place {
    fn new(name: &str, ledger: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("eretry")
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
        fs::write(root.join("out.json"), ledger).expect("偽の bd の出力");
        let r = root.display();
        script(
            &root.join("bd"),
            &format!(
                "n=$(( $(cat '{r}/bd.count' 2>/dev/null || echo 0) + 1 ))\n\
                 echo \"$n\" > '{r}/bd.count'\n\
                 if [ \"$n\" -le \"$(cat '{r}/bd.fails' 2>/dev/null || echo 0)\" ]; then exit 1; fi\n\
                 exec cat '{r}/out.json'"
            ),
        );
        recorder(&root.join("bdw"), &log, "bdw", ":");
        recorder(
            &root.join("scribe2"),
            &log,
            "scribe2",
            &format!(
                "if [ \"$1 $2 $3\" = 'seat ruling answer' ]; then\n\
                   cat > '{log}/scribe2.'\"$n\"'.stdin'\n\
                   if [ -e '{log}/junk' ]; then echo not-an-id; else echo \"$9:{MINUTE}-1\"; fi\n\
                 fi\n\
                 if [ \"$1 $2\" = 'seat deliver' ]; then\n\
                   i=0\n\
                   while [ ! -e '{r}/gate' ] && [ \"$i\" -lt 90 ]; do sleep 0.1; i=$((i + 1)); done\n\
                   : > '{r}/scribe2.end'\n\
                 fi",
                log = log.display()
            ),
        );
        let vessel = root.join("scribe2");
        Place {
            root,
            repo,
            files,
            state,
            log,
            vessel,
        }
    }

    /// 偽の bd を `n` 回目まで落とす。
    fn bd_fails(&self, n: u32) {
        fs::write(self.root.join("bd.fails"), n.to_string()).expect("落とす回");
    }

    /// 偽の bd が撃たれた回の数。
    fn bd_count(&self) -> u32 {
        fs::read_to_string(self.root.join("bd.count"))
            .map_or(0, |c| c.trim().parse().expect("回の数"))
    }

    /// 偽の program（bdw か scribe2）が撃たれた回ごとの argv。
    fn argvs(&self, name: &str) -> Vec<Vec<String>> {
        let count: u32 = fs::read_to_string(self.log.join(format!("{name}.count")))
            .map_or(0, |c| c.trim().parse().expect("回の数"));
        (1..=count)
            .map(|n| {
                fs::read_to_string(self.log.join(format!("{name}.{n}.args")))
                    .expect("記録")
                    .lines()
                    .map(str::to_string)
                    .collect()
            })
            .collect()
    }

    /// 偽の program の `n` 回目の done の file が在るまで 10 秒まで待つ（在れば true）。
    fn wait_done(&self, name: &str, n: u32) -> bool {
        let path = self.log.join(format!("{name}.{n}.done"));
        let until = Instant::now() + Duration::from_secs(10);
        while !path.exists() && Instant::now() < until {
            thread::sleep(Duration::from_millis(20));
        }
        path.exists()
    }

    /// 偽の器の門を開ける。
    fn open_gate(&self) {
        fs::write(self.root.join("gate"), "").expect("門の file");
    }

    fn ended(&self) -> bool {
        self.root.join("scribe2.end").exists()
    }

    fn source(&self) -> Source {
        Source::new(self.repo.clone(), self.root.join("bd"))
    }

    /// 答えの口の撃ち先（偽の器の program と置き場）。
    fn vessel(&self) -> Vessel<'_> {
        (self.vessel.as_os_str(), Some(self.state.as_path()))
    }

    /// 偽の器の答えの口の 1 回の argv（束の id が在れば末に旗つき）。
    fn answer_words(&self, question: &str, batch: Option<&str>) -> Vec<String> {
        let mut words: Vec<String> = [
            "seat",
            "ruling",
            "answer",
            "--repo",
            &self.repo.display().to_string(),
            "--state-dir",
            &self.state.display().to_string(),
            "--question",
            question,
        ]
        .map(str::to_string)
        .to_vec();
        if let Some(batch) = batch {
            words.extend(["--batch".to_string(), batch.to_string()]);
        }
        words
    }

    /// 配達の先つきの Writer（`deliver` が false なら配達の先は無し）。
    fn writer(&self, deliver: bool) -> Writer {
        Writer {
            repo: self.repo.clone(),
            bdw: self.root.join("bdw").into(),
            delivery: deliver.then(|| Delivery {
                program: self.root.join("scribe2").into(),
                state_dir: self.state.clone(),
                target: TARGET.to_string(),
            }),
        }
    }
}

fn rid(id: &str) -> RulingId {
    RulingId::new(id).expect("記帳 id")
}

/// 中核の問いの一覧で読んだ card の digest（見た版の要約値）。
fn digest(ledger: &str, id: &str) -> String {
    let Reading::Known(cards) = tsuzuri_core::question::list(ledger).cards else {
        panic!("見本の台帳が Unknown");
    };
    cards
        .into_iter()
        .find(|c| c.id.as_str() == id)
        .unwrap_or_else(|| panic!("{id} の card"))
        .digest
}

fn ruling_request(ledger: &str, verbatim: &str) -> RulingRequest {
    RulingRequest {
        question: bead(OPEN),
        seen_digest: digest(ledger, OPEN),
        verbatim: verbatim.to_string(),
    }
}

fn batch_request(ledger: &str) -> BatchRequest {
    BatchRequest {
        items: ROWS
            .iter()
            .map(|q| BatchItem {
                question: bead(q),
                seen_digest: digest(ledger, q),
                verbatim: None,
            })
            .collect(),
        verbatim: "はい".to_string(),
    }
}

/// (1) 読みが落ちれば 1 秒を空けて 3 回まで撃ち直し、読めた後に 1 度ずつ書く。
#[test]
fn eretry_read_retries_then_writes() {
    assert_eq!(READ_TRIES, 3);
    assert_eq!(RETRY_STEP, Duration::from_secs(1));
    assert_eq!(WRITE_TIMEOUT, Duration::from_secs(120));

    // 2 回落ちて 3 回目に読めれば記録し、器の答えの口を 1 回だけ撃つ（偽の bdw は撃たない）。
    let ledger = fixture(ASK);
    let place = Place::new("accept", &ledger);
    place.bd_fails(2);
    let got = ruling::accept(
        &ruling_request(&ledger, "はい"),
        &place.source(),
        &place.writer(false),
        place.vessel(),
        NOW,
    );
    let id = rid(&format!("{OPEN}:{MINUTE}-1"));
    let Outcome::Recorded(response) = &got else {
        panic!("記録しない: {got:?}");
    };
    assert_eq!(response.ruling, id);
    assert_eq!(place.bd_count(), 3, "偽の bd は 3 回");
    assert!(place.argvs("bdw").is_empty(), "偽の bdw は撃たれない");
    assert_eq!(place.argvs("scribe2"), [place.answer_words(OPEN, None)]);

    // 3 回とも落ちれば LedgerUnknown で、書きも器も撃たない。
    let place = Place::new("accept-down", &ledger);
    place.bd_fails(3);
    let got = ruling::accept(
        &ruling_request(&ledger, "はい"),
        &place.source(),
        &place.writer(false),
        place.vessel(),
        NOW,
    );
    assert_eq!(got, Outcome::LedgerUnknown);
    assert_eq!(place.bd_count(), 3, "偽の bd は 3 回");
    assert!(place.argvs("bdw").is_empty(), "読めないのに偽の bdw を撃つ");
    assert!(place.argvs("scribe2").is_empty(), "読めないのに偽の器を撃つ");
    revoke_and_batch_retry(ledger);
}

/// 取り消しと束も、読みが 2 回落ちた後の 3 回目で記録する。
fn revoke_and_batch_retry(ledger: String) {
    // 取り消しも 3 回目の読みで記録する。
    let place = Place::new("revoke", &ledger);
    place.bd_fails(2);
    let got = ruling::revoke(
        &RevokeRequest {
            question: bead(CLOSED),
            ruling: rid(CLOSED_RULING),
            verbatim: "待つ".to_string(),
        },
        &place.source(),
        &place.writer(false),
        NOW,
    );
    assert!(matches!(got, Revoked::Recorded(_)), "{got:?}");
    assert_eq!(place.bd_count(), 3, "偽の bd は 3 回");

    // 束も 3 回目の読みで記録する。
    let ledger = fixture(BATCH);
    let place = Place::new("batch", &ledger);
    place.bd_fails(2);
    let got = batch::accept(
        &batch_request(&ledger),
        &place.source(),
        &place.writer(false),
        place.vessel(),
        NOW,
    );
    assert!(matches!(got, batch::Outcome::Recorded(_)), "{got:?}");
    assert_eq!(place.bd_count(), 3, "偽の bd は 3 回");
}

/// (2) 配達の先を持つ受付は、偽の器が門で止まっている間に記録を返し、門を開けた後に印を置く。
#[test]
fn eretry_deliver_after_reply() {
    // 1 問。
    let ledger = fixture(ASK);
    let place = Place::new("deliver-one", &ledger);
    let got = ruling::accept(
        &ruling_request(&ledger, "はい"),
        &place.source(),
        &place.writer(true),
        place.vessel(),
        NOW,
    );
    let Outcome::Recorded(response) = &got else {
        panic!("記録しない: {got:?}");
    };
    assert!(!place.ended(), "偽の器が終わってから返す");
    assert!(place.argvs("bdw").is_empty(), "応答の時に偽の bdw を撃つ");
    assert!(place.wait_done("scribe2", 2), "配達を撃たない");
    assert!(!place.ended(), "門の前に偽の器が終わる");
    assert!(place.argvs("bdw").is_empty(), "配達の前に印を置く");
    place.open_gate();
    assert!(place.wait_done("bdw", 1), "門の後に印を置かない");
    let scribe2 = place.argvs("scribe2");
    assert_eq!(scribe2.len(), 2, "答えと配達: {scribe2:?}");
    assert_eq!(scribe2[0], place.answer_words(OPEN, None));
    assert_eq!(
        scribe2[1].last().map(String::as_str),
        Some(response.ruling.as_str())
    );
    assert_eq!(place.argvs("bdw").len(), 1, "印の 1 回だけ");
    deliver_batch_after_reply();
}

/// 束の配達も、偽の器が門で止まっている間に記録を返し、門を開けた後に印を置く。
fn deliver_batch_after_reply() {
    // 束の 2 行。
    let ledger = fixture(BATCH);
    let place = Place::new("deliver-batch", &ledger);
    let got = batch::accept(
        &batch_request(&ledger),
        &place.source(),
        &place.writer(true),
        place.vessel(),
        NOW,
    );
    let batch::Outcome::Recorded(response) = &got else {
        panic!("記録しない: {got:?}");
    };
    assert!(!place.ended(), "偽の器が終わってから返す");
    assert!(place.argvs("bdw").is_empty(), "応答の時に偽の bdw を撃つ");
    assert!(place.wait_done("scribe2", 3), "配達を撃たない");
    assert!(!place.ended(), "門の前に偽の器が終わる");
    assert!(place.argvs("bdw").is_empty(), "配達の前に印を置く");
    place.open_gate();
    assert!(place.wait_done("bdw", 2), "門の後に印を置かない");
    let scribe2 = place.argvs("scribe2");
    assert_eq!(scribe2.len(), 3, "行ごとの答えと配達: {scribe2:?}");
    for (row, question) in ROWS.iter().enumerate() {
        assert_eq!(
            scribe2[row],
            place.answer_words(question, Some(response.batch.as_str()))
        );
    }
    assert_eq!(
        scribe2[2].last().map(String::as_str),
        Some(response.batch.as_str())
    );
    assert_eq!(place.argvs("bdw").len(), 2, "印は行ごとの 2 回");
}

/// 要求を 1 つ撃ち、接続が閉じるまで応答を読む（状態の code と本文）。
fn send(addr: SocketAddr, body: &str) -> (u16, String) {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(30)))
        .expect("timeout");
    s.write_all(
        format!(
            "POST {PATH} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
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

/// (3) 口が断れば、状態の code と本文と問いの id の 1 行を標準 error に書き、逐語は書かない。
#[test]
fn eretry_refusal_logged() {
    let ledger = fixture(ASK);
    let place = Place::new("log", &ledger);
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["surface", "serve", "--bind", "127.0.0.1:0", "--repo"])
        .arg(&place.repo)
        .arg("--files")
        .arg(&place.files)
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
    let addr: SocketAddr = line
        .trim()
        .strip_prefix("tz surface serve: http://")
        .and_then(|a| a.strip_suffix('/'))
        .and_then(|a| a.parse().ok())
        .unwrap_or_else(|| panic!("口の住所の行でない: {line}"));
    let replies = std::panic::catch_unwind(|| {
        let stale = RulingRequest {
            seen_digest: "0000000000000000".to_string(),
            ..ruling_request(&ledger, SECRET)
        };
        let first = send(addr, &wire::encode(&stale).expect("要求の電文"));
        place.bd_fails(u32::MAX);
        let second = send(
            addr,
            &wire::encode(&ruling_request(&ledger, SECRET)).expect("要求の電文"),
        );
        (first, second)
    });
    let _ = child.kill();
    let _ = child.wait();
    let ((s1, b1), (s2, b2)) = replies.expect("応答を読めない");
    assert_eq!(s1, 409, "{b1}");
    assert_eq!(s2, 503, "{b2}");
    let mut rest = String::new();
    err.read_to_string(&mut rest).expect("標準 error");

    let lines: Vec<&str> = rest.lines().filter(|l| l.starts_with(REFUSED_LOG)).collect();
    assert_eq!(lines.len(), 2, "断りの行が 2 行でない: {rest}");
    assert!(
        lines[0].starts_with(&format!("{REFUSED_LOG}: /api/ruling 409 ")),
        "{}",
        lines[0]
    );
    assert!(lines[0].ends_with(&format!("・問い {OPEN}")), "{}", lines[0]);
    assert_eq!(
        lines[1],
        format!("{REFUSED_LOG}: /api/ruling 503 ledger-unknown・問い {OPEN}")
    );
    assert!(!rest.contains(SECRET), "標準 error に逐語: {rest}");
}
