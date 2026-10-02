//! ほかの project の問いを問いの一覧に混ぜる歯（接頭辞 mqask_・設計ノート surface-wave22c 行 e-multi-ask）。
//! 自分の置き場の台帳は見本の surface/question-2.json（open の問い fx-ask.3 と fx-ask.2）、ほかの project の置き場
//! proj-x の台帳は歯の中の 2 行（open の問い fx-oth.2 と closed の問い fx-oth.1）で、proj-y は台帳の file を持たない。
//! 偽の bd は撃たれた cwd の ledger.json を出す script（無ければ rc が 0 でない）で、偽の bdw は撃たれた回を数える。
//! (1)〜(4) は server を同じ process の thread で立て、(5) は tz を撃つ。
#![cfg(test)]

use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::server::{Config, Server, ruling};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::question::{AllQuestions, QuestionCard, QuestionList};
use tsuzuri_contract::surface::{Refusal, RefusalResponse, RulingRequest};
use tsuzuri_contract::wire;

/// 自分の置き場の台帳の見本。
const ASK: &str = "surface/question-2.json";

/// proj-x の open の問い。
const OTHER_OPEN: &str = r#"{"id":"fx-oth.2","title":"問い — ほかの project の問い","description":"概要 = ほかの project の問い","status":"open","issue_type":"task","created_at":"2026-09-25T01:00:00Z","updated_at":"2026-09-25T01:00:00Z","labels":["intake:question"]}"#;

/// proj-x の closed の問い。
const OTHER_CLOSED: &str = r#"{"id":"fx-oth.1","title":"問い — ほかの project の答えの済んだ問い","description":"概要 = 閉じた問い","status":"closed","issue_type":"task","created_at":"2026-09-24T01:00:00Z","updated_at":"2026-09-24T02:00:00Z","closed_at":"2026-09-24T02:00:00Z","labels":["intake:question"]}"#;

/// proj-x に後から増える open の問い。
const OTHER_NEW: &str = r#"{"id":"fx-oth.3","title":"問い — 後から増えた問い","description":"概要 = 後から増えた問い","status":"open","issue_type":"task","created_at":"2026-09-26T01:00:00Z","updated_at":"2026-09-26T01:00:00Z","labels":["intake:question"]}"#;

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

/// 台帳の字を丸ごと替える（隣に書いてから移すので、偽の bd から書きかけは見えない）。
fn put(dir: &Path, text: &str) {
    let tmp = dir.join("ledger.json.tmp");
    fs::write(&tmp, text).expect("移す前の台帳");
    fs::rename(&tmp, dir.join("ledger.json")).expect("台帳を移す");
}

/// 歯ごとの作業場（自分の repo・面の file・記録の置き場・ほかの project の 2 つの置き場・偽の program）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
    log: PathBuf,
    proj_x: PathBuf,
    proj_y: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("mqask")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, files, log) = (root.join("repo"), root.join("files"), root.join("log"));
        let (proj_x, proj_y) = (root.join("proj-x"), root.join("proj-y"));
        for dir in [&repo, &proj_x, &proj_y] {
            fs::create_dir_all(dir.join(".beads")).expect("repo の置き場");
        }
        for dir in [&files, &log] {
            fs::create_dir_all(dir).expect("作業場の dir");
        }
        fs::write(files.join("index.html"), "tz").expect("index.html");
        put(&repo, &fixture(ASK));
        put(&proj_x, &format!("[{OTHER_OPEN},\n{OTHER_CLOSED}]\n"));
        script(&root.join("bd"), "exec cat ledger.json");
        let l = log.display();
        script(
            &root.join("bdw"),
            &format!(
                "n=$(( $(cat '{l}/bdw.count' 2>/dev/null || echo 0) + 1 ))\n\
                 echo \"$n\" > '{l}/bdw.count'\n\
                 exit 0"
            ),
        );
        Place {
            root,
            repo,
            files,
            log,
            proj_x,
            proj_y,
        }
    }

    /// 偽の bdw が撃たれた回の数。
    fn bdw_count(&self) -> u32 {
        fs::read_to_string(self.log.join("bdw.count"))
            .map_or(0, |c| c.trim().parse().expect("回の数"))
    }

    fn config(&self, projects: Vec<PathBuf>) -> Config {
        Config {
            bd: self.root.join("bd").into(),
            bdw: self.root.join("bdw").into(),
            projects,
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        }
    }

    /// server を同じ process の thread で立てて口の住所を返す。
    fn serve(&self, projects: Vec<PathBuf>) -> SocketAddr {
        let server = Server::bind(&self.config(projects)).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }

    /// proj-x と proj-y を引数の順に持つ server。
    fn serve_both(&self) -> SocketAddr {
        self.serve(vec![self.proj_x.clone(), self.proj_y.clone()])
    }
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

/// 口 GET /api/questions の本文。
fn questions_text(addr: SocketAddr) -> String {
    let (status, body) = send(addr, "GET", "/api/questions", "");
    assert_eq!(status, 200, "{body}");
    body
}

fn questions(addr: SocketAddr) -> AllQuestions {
    wire::decode(&questions_text(addr)).expect("全部の問いの一覧の形")
}

fn ids(cards: &Reading<Vec<QuestionCard>>) -> Vec<String> {
    match cards {
        Reading::Known(cards) => cards.iter().map(|c| c.id.to_string()).collect(),
        Reading::Unknown => panic!("card が Unknown"),
    }
}

/// 自分の台帳の見本の問いの一覧（答えを受ける server）。
fn own_list() -> QuestionList {
    tsuzuri_core::question::list(&fixture(ASK))
}

/// (1) 鍵 others は引数の順に proj-x（answerable 偽・open の問い fx-oth.2 の 1 本）と proj-y（answerable 偽・Unknown）を
/// 持ち、自分の問いの欄は今の QuestionList と一致し、同じ本文を QuestionList で読むと自分の問いの一覧になる。
#[test]
fn mqask_others_follow_args() {
    let place = Place::new("args");
    let addr = place.serve_both();
    let text = questions_text(addr);
    let all: AllQuestions = wire::decode(&text).expect("全部の問いの一覧の形");
    assert_eq!(all.own, own_list());
    assert_eq!(all.others.len(), 2, "{text}");
    let (x, y) = (&all.others[0], &all.others[1]);
    assert_eq!(x.project, "proj-x");
    assert!(!x.answerable, "proj-x の answerable が真");
    assert_eq!(ids(&x.cards), ["fx-oth.2"]);
    assert_eq!(y.project, "proj-y");
    assert!(!y.answerable, "proj-y の answerable が真");
    assert_eq!(y.cards, Reading::Unknown);
    let bare: QuestionList = wire::decode(&text).expect("問いの一覧の形");
    assert_eq!(bare, own_list());
}

/// (2) --project の無い server の本文は QuestionList の電文の字と一致し、鍵 others を持たない。
#[test]
fn mqask_no_projects_same_text() {
    let place = Place::new("bare");
    let addr = place.serve(vec![]);
    let text = questions_text(addr);
    assert_eq!(text, wire::encode(&own_list()).expect("問いの一覧の電文"));
    assert!(!text.contains("\"others\""), "{text}");
}

/// (3) 知らせの接続の後に proj-x の台帳に open の問い fx-oth.3 が増えて印が動くと、10 秒の内に ledger-changed が届き、
/// その後の口の proj-x の card の id は fx-oth.2・fx-oth.3 の順。
#[test]
fn mqask_other_change_reaches_events() {
    let place = Place::new("events");
    let addr = place.serve_both();
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_millis(200)))
        .expect("timeout");
    s.write_all(
        format!("GET /api/surface/events HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n")
            .as_bytes(),
    )
    .expect("要求を書く");
    let mut got = String::new();
    let mut read_until = |want: &str, within: Duration, got: &mut String| -> bool {
        let until = Instant::now() + within;
        let mut buf = [0u8; 4096];
        while Instant::now() < until {
            if got.contains(want) {
                return true;
            }
            match s.read(&mut buf) {
                Ok(0) => return got.contains(want),
                Ok(n) => got.push_str(&String::from_utf8_lossy(&buf[..n])),
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
                Err(e) => panic!("知らせを読めない: {e}"),
            }
        }
        got.contains(want)
    };
    assert!(
        read_until("retry: 1000\n\n", Duration::from_secs(10), &mut got),
        "知らせの頭が来ない: {got}"
    );
    // 受け手が付いた周の読みを待つ（同じ読みなら知らせない）。
    thread::sleep(Duration::from_millis(1500));
    assert!(
        !got.contains("event: ledger-changed"),
        "台帳が動く前に知らせる: {got}"
    );
    put(
        &place.proj_x,
        &format!("[{OTHER_OPEN},\n{OTHER_CLOSED},\n{OTHER_NEW}]\n"),
    );
    fs::write(place.proj_x.join(".beads/issues.jsonl"), "x\n").expect("印を動かす");
    assert!(
        read_until("event: ledger-changed\n", Duration::from_secs(10), &mut got),
        "10 秒の内に ledger-changed が来ない: {got}"
    );
    let all = questions(addr);
    assert_eq!(all.others[0].project, "proj-x");
    assert_eq!(ids(&all.others[0].cards), ["fx-oth.2", "fx-oth.3"]);
}

/// (4) ほかの project の問い fx-oth.2 への答えは状態 404・理由 UnknownQuestion の断りになり、bdw を撃たない。
#[test]
fn mqask_answer_to_other_refused() {
    let place = Place::new("answer");
    let addr = place.serve_both();
    let all = questions(addr);
    let Reading::Known(cards) = &all.others[0].cards else {
        panic!("proj-x の card が Unknown");
    };
    let card = cards
        .iter()
        .find(|c| c.id.as_str() == "fx-oth.2")
        .expect("fx-oth.2 の card");
    let body = wire::encode(&RulingRequest {
        question: BeadId::new("fx-oth.2").expect("bead id"),
        seen_digest: card.digest.clone(),
        verbatim: "はい".to_string(),
    })
    .expect("要求の電文");
    let (status, reply) = send(addr, "POST", ruling::PATH, &body);
    assert_eq!(status, 404, "{reply}");
    let refusal: RefusalResponse = wire::decode(&reply).expect("断りの電文");
    assert_eq!(refusal.reason, Refusal::UnknownQuestion);
    thread::sleep(Duration::from_millis(300));
    assert_eq!(place.bdw_count(), 0, "偽の bdw を撃つ");
}

/// tz surface serve を `extra` の引数を足して撃ち、rc と標準 error を返す。
fn tz(place: &Place, extra: &[String]) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["surface", "serve", "--bind", "127.0.0.1:0", "--repo"])
        .arg(&place.repo)
        .arg("--files")
        .arg(&place.files)
        .arg(format!("--bd={}", place.root.join("bd").display()))
        .args(extra)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .expect("tz を撃つ");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// (5) tz は --project の dir の名 same が 2 度なら、--project= と --project=/ なら、rc 1 で断りの字を標準 error に書く。
#[test]
fn mqask_tz_refuses_bad_project() {
    let place = Place::new("cli");
    let (a, b) = (place.root.join("a/same"), place.root.join("b/same"));
    let (rc, err) = tz(
        &place,
        &[
            "--project".to_string(),
            a.display().to_string(),
            format!("--project={}", b.display()),
        ],
    );
    assert_eq!(rc, Some(1), "{err}");
    assert!(err.contains("--project の名 same が 2 度ある"), "{err}");
    for bad in ["--project=", "--project=/"] {
        let (rc, err) = tz(&place, &[bad.to_string()]);
        assert_eq!(rc, Some(1), "{bad}: {err}");
        assert!(err.contains("に dir の名が無い"), "{bad}: {err}");
    }
}
