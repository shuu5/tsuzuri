//! 答えた決定の取り消しの歯（接頭辞 rvk_・設計ノート surface-wave12d 行 e-revoke・要件 FR7 と FR9）。
//! 偽の bd は作業場の out.json を標準出力へ出す script（無ければ rc 1）、
//! 偽の bdw と偽の器は撃たれた回ごとの argv と cwd を記録の置き場（repo と state dir の外）に書き、決めた回で rc 1 にする script。
//! 台帳の字は歯の中で組む bd の出力の形の JSON の配列。server は同じ process の thread で 127.0.0.1 の空き port に立てる。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::server::ledger::Source;
use tsuzuri_boundary::server::route::{Key, Match};
use tsuzuri_boundary::server::ruling::{
    Delivery, LINE_PREFIX, Revoked, Writer, deliver_argv, minute, revoke, revoke_line,
};
use tsuzuri_boundary::server::{Config, Route, Server, events};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::{EdgeType, NodeKind};
use tsuzuri_contract::ledger::{BeadId, ITEM_PATH, LedgerItem, LedgerRow, LedgerWrite};
use tsuzuri_contract::surface::{
    CLOSED_STATUS, QUESTION_FIELD, REVOKE_PATH, REVOKES, RULING_LINE, Refusal, RefusalResponse,
    RevokeRequest, RevokeResponse, RulingId, VERBATIM, latest_ruling, pending_reopen, revocable,
};
use tsuzuri_contract::wire;
use tsuzuri_core::delivery::{Pending, RULING_PREFIX, Route as MarkRoute, mark_line, undelivered};
use tsuzuri_core::graph::Inputs;
use tsuzuri_core::graph::build::{TYPED_LINES, build};

/// 器の配達の口の target。
const TARGET: &str = "tsuzuri:0.1";

/// done (5) と (6) の受付の時刻（2026-09-28T04:41:00Z）。
const AT: u64 = 1_790_570_460;

/// done (7) の受付の時刻（2026-09-28T04:42:00Z）。
const AT_RETRY: u64 = 1_790_570_520;

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("bead id")
}

fn rid(id: &str) -> RulingId {
    RulingId::new(id).expect("裁定の id")
}

/// 裁定の行（`裁定 id = <id>・問い = <問い>・逐語 = はい`）。
fn rline(id: &str, question: &str) -> String {
    format!("裁定 id = {id}・問い = {question}・逐語 = はい")
}

/// 取り消しの行（境界の `revoke_line` の字）。
fn rvline(id: &str, question: &str, revokes: &str, verbatim: &str) -> String {
    revoke_line(&rid(id), &bead(question), &rid(revokes), verbatim)
}

/// bd の出力の形の 1 本（問いなら label intake:question の 1 つ・notes は行を改行でつなぐ）。
fn bd_bead(id: &str, status: &str, question: bool, notes: &[String]) -> String {
    let s = |t: &str| wire::encode(&t.to_string()).expect("字の電文");
    let labels = if question { r#"["intake:question"]"# } else { "[]" };
    format!(
        r#"{{"id":{},"title":{},"status":{},"updated_at":"2026-09-28T04:41:00Z","issue_type":"task","labels":{labels},"notes":{}}}"#,
        s(id),
        s(&format!("題 {id}")),
        s(status),
        s(&notes.join("\n"))
    )
}

fn bd_list(beads: &[String]) -> String {
    format!("[{}]", beads.join(","))
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
             if [ \"$n\" = \"$(cat '{log}/{name}.fail' 2>/dev/null)\" ]; then echo refused >&2; exit 1; fi\n\
             exit 0"
        ),
    );
}

/// 場ごとの作業場（repo・面の file・state dir・記録の置き場・out.json・偽の program）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
    state: PathBuf,
    log: PathBuf,
}

impl Place {
    fn new(tooth: &str, scene: &str, ledger: Option<&str>) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("rvk")
            .join(tooth)
            .join(scene);
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
        if let Some(text) = ledger {
            place.bd_returns(text);
        }
        place
    }

    fn bd_returns(&self, text: &str) {
        fs::write(self.root.join("out.json"), text).expect("偽の bd の出力");
    }

    /// 偽の bd を落とす（出す file が無いので rc 1）。
    fn bd_fails(&self) {
        let _ = fs::remove_file(self.root.join("out.json"));
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

    fn argvs(&self, name: &str) -> Vec<Vec<String>> {
        self.calls(name).into_iter().map(|(argv, _)| argv).collect()
    }

    /// 偽の program の `n` 回目の cwd の記録が空でなくなるまで 10 秒まで待つ（配達は応答の後の thread）。
    fn wait(&self, name: &str, n: u32) {
        let path = self.log.join(format!("{name}.{n}.cwd"));
        let until = Instant::now() + Duration::from_secs(10);
        while !fs::read(&path).is_ok_and(|b| !b.is_empty()) && Instant::now() < until {
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn delivery(&self) -> Delivery {
        Delivery {
            program: self.root.join("scribe2").into(),
            state_dir: self.state.clone(),
            target: TARGET.to_string(),
        }
    }

    /// 配達の先つきの Writer（`deliver` が false なら配達の先は無し）。
    fn writer(&self, deliver: bool) -> Writer {
        Writer {
            repo: self.repo.clone(),
            bdw: self.root.join("bdw").into(),
            delivery: deliver.then(|| self.delivery()),
        }
    }

    fn source(&self) -> Source {
        Source::new(self.repo.clone(), self.root.join("bd"))
    }

    /// 取り消しを直に受ける。
    fn revoke(&self, req: &RevokeRequest, deliver: bool, now: u64) -> Revoked {
        revoke(req, &self.source(), &self.writer(deliver), now)
    }

    fn deliver_argv(&self, id: &str) -> Vec<String> {
        deliver_argv(&self.delivery(), &rid(id))
            .into_iter()
            .map(|a| a.into_string().expect("argv の字"))
            .collect()
    }

    /// 偽の器と偽の bdw の cwd はどれも repo の置き場。
    fn cwd_is_repo(&self) {
        let real = format!(
            "{}\n",
            self.repo.canonicalize().expect("repo の実体").display()
        );
        for name in ["scribe2", "bdw"] {
            for (_, cwd) in self.calls(name) {
                assert_eq!(cwd, real, "{name} の cwd は repo の置き場");
            }
        }
    }

    fn nothing_shot(&self, what: &str) {
        assert!(self.calls("bdw").is_empty(), "{what}: 偽の bdw を撃つ");
        assert!(self.calls("scribe2").is_empty(), "{what}: 偽の器を撃つ");
    }

    /// 配達の先の無い server（席の target と state dir は無し）。
    fn serve(&self) -> SocketAddr {
        let config = Config {
            bd: self.root.join("bd").into(),
            bdw: self.root.join("bdw").into(),
            scribe2: self.root.join("scribe2").into(),
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        };
        let server = Server::bind(&config).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }
}

fn request(question: &str, ruling: &str, verbatim: &str) -> RevokeRequest {
    RevokeRequest {
        question: bead(question),
        ruling: rid(ruling),
        verbatim: verbatim.to_string(),
    }
}

fn reopen_argv(question: &str, id: &str, revokes: &str) -> Vec<String> {
    LedgerWrite::ReopenItem {
        id: bead(question),
        reason: format!("裁定 {id} 取り消す = {revokes}"),
    }
    .argv()
}

/// 経路 配達の口 の印の追記（頭の 2 語と 3 語目の頭の字 `--append-notes=配達 = <id>・`）。
fn is_mark(argv: &[String], question: &str, id: &str) -> bool {
    argv.len() == 3
        && argv[0] == "update"
        && argv[1] == question
        && argv[2].starts_with(&format!("--append-notes=配達 = {id}・"))
        && argv[2].contains("経路 = 配達の口")
}

struct Reply {
    status: u16,
    head: String,
    body: String,
}

/// 要求を 1 つ撃ち、接続が閉じるまで応答を読む。
fn send(addr: SocketAddr, method: &str, path: &str, heads: &str, body: &str) -> Reply {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    s.write_all(
        format!(
            "{method} {path} HTTP/1.1\r\nHost: {addr}\r\n{heads}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
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
    Reply {
        status,
        head: head.to_string(),
        body: body.to_string(),
    }
}

fn post(addr: SocketAddr, req: &RevokeRequest) -> Reply {
    send(
        addr,
        "POST",
        REVOKE_PATH,
        "",
        &wire::encode(req).expect("要求の電文"),
    )
}

fn refused(reply: &Reply) -> Refusal {
    wire::decode::<RefusalResponse>(&reply.body)
        .unwrap_or_else(|e| panic!("断りの形でない {e}: {}", reply.body))
        .reason
}

/// 字 `start` から次の行頭の閉じ括弧 1 字だけの行までの字。
fn body_of<'a>(text: &'a str, start: &str) -> &'a str {
    let at = text
        .find(start)
        .unwrap_or_else(|| panic!("{start} の定義"));
    let rest = &text[at..];
    let end = rest.find("\n}\n").expect("本文の終わり");
    &rest[..end]
}

#[test]
fn rvk_reopen_argv_and_wire() {
    let w = LedgerWrite::ReopenItem {
        id: bead("rv.1"),
        reason: "裁定 rv.1:20260928T0441Z-1・取り消す = rv.1:20260928T0100Z-1".into(),
    };
    assert_eq!(
        w.argv(),
        [
            "reopen",
            "rv.1",
            "--reason=裁定 rv.1:20260928T0441Z-1・取り消す = rv.1:20260928T0100Z-1"
        ]
    );
    assert!(!w.creates_child());
    let flag = LedgerWrite::ReopenItem {
        id: bead("rv.1"),
        reason: "--notes".into(),
    };
    assert_eq!(flag.argv(), ["reopen", "rv.1", "--reason=--notes"]);
    let text = wire::encode(&w).expect("電文");
    assert!(text.contains(r#""op":"reopen-item""#), "{text}");
    assert_eq!(wire::decode::<LedgerWrite>(&text).expect("読み戻す"), w);
    assert!(wire::decode::<LedgerWrite>(r#"{"op":"reopen-item","reason":"x"}"#).is_err());
    assert_eq!(ITEM_PATH, "/api/ledger/");
}

#[test]
fn rvk_wire_and_path() {
    assert_eq!(REVOKE_PATH, "/api/revoke");
    assert_eq!(RULING_LINE, "裁定 id = ");
    assert_eq!(QUESTION_FIELD, "問い = ");
    assert_eq!(REVOKES, "取り消す = ");
    assert_eq!(VERBATIM, "逐語 = ");
    assert_eq!(CLOSED_STATUS, "closed");
    let req = request("rv.1", "rv.1:20260928T0441Z-1", "待つ");
    let text = wire::encode(&req).expect("要求の電文");
    assert_eq!(
        text,
        r#"{"question":"rv.1","ruling":"rv.1:20260928T0441Z-1","verbatim":"待つ"}"#
    );
    assert_eq!(wire::decode::<RevokeRequest>(&text).expect("読み戻す"), req);
    let res = RevokeResponse {
        ruling: rid("rv.1:20260928T0441Z-1"),
        recorded_at: AT,
        reopened_only: true,
    };
    let text = wire::encode(&res).expect("応答の電文");
    assert_eq!(
        text,
        r#"{"ruling":"rv.1:20260928T0441Z-1","recorded_at":1790570460,"reopened_only":true}"#
    );
    assert_eq!(wire::decode::<RevokeResponse>(&text).expect("読み戻す"), res);
    for bad in [
        r#"{"question":"rv.1","ruling":"a b","verbatim":"x"}"#,
        r#"{"question":"rv.1","ruling":"rv.1:20260928T0441Z-1"}"#,
        r#"{"question":"-x","ruling":"rv.1:20260928T0441Z-1","verbatim":"x"}"#,
    ] {
        assert!(wire::decode::<RevokeRequest>(bad).is_err(), "{bad} を読む");
    }
    let key = |name: &str| {
        Route::ALL
            .iter()
            .find(|r| r.name() == name)
            .unwrap_or_else(|| panic!("口 {name}"))
            .key()
    };
    assert_eq!(
        key("revoke"),
        Key {
            method: "POST",
            path: Match::Exact(REVOKE_PATH),
        }
    );
    assert_eq!(
        key("item"),
        Key {
            method: "GET",
            path: Match::Prefix(ITEM_PATH),
        }
    );
}

/// 節の表の行（q.1 の問いの裁定）。
struct Table {
    a: String,
    b: String,
    r: String,
    ra: String,
    r2: String,
    c: String,
    f: String,
    mark_b: String,
    mark_c: String,
}

const A: &str = "q.1:20260928T0100Z-1";
const B: &str = "q.1:20260928T0200Z-1";
const R: &str = "q.1:20260928T0300Z-1";
const C: &str = "q.1:20260928T0400Z-1";
const F: &str = "q.1:20260928T0500Z-1";

fn table() -> Table {
    Table {
        a: rline(A, "q.1"),
        b: format!("裁定 id = {B}・問い = q.1・束 = batch:20260928T0200Z-1・逐語 = はい"),
        r: rvline(R, "q.1", B, "待つ"),
        ra: rvline("q.1:20260928T0300Z-2", "q.1", A, "待つ"),
        r2: format!("裁定 id = {R}・問い = q.2・取り消す = {B}・逐語 = 待つ"),
        c: rline(C, "q.1"),
        f: format!("裁定 id = {F}・問い = q.1・逐語 = はい・取り消す = {C}"),
        mark_b: mark_line(&rid(B), MarkRoute::Deliver, "20260928T0201Z"),
        mark_c: mark_line(&rid(C), MarkRoute::Stop, "20260928T0401Z"),
    }
}

fn item(status: &str, question: bool, notes: &[&String]) -> LedgerItem {
    LedgerItem {
        row: LedgerRow {
            id: bead("q.1"),
            kind: "task".into(),
            title: "題 q.1".into(),
            status: status.into(),
            updated_at: AT,
            parent: None,
            labels: if question {
                vec!["intake:question".into()]
            } else {
                vec![]
            },
        },
        description: String::new(),
        notes: notes
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

#[test]
fn rvk_latest_and_revocable_table() {
    let t = table();
    let j = |lines: &[&String]| {
        lines
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    };
    let empty_id = "裁定 id = ・問い = q.1・逐語 = はい".to_string();
    let spaced = format!(" {}", rline("q.1:20260928T0600Z-1", "q.1"));
    let lead = "前置き\n受け id = q.1:20260928T0100Z-1・席 = x".to_string();
    let crlf = format!("{}\r\n{}\r\n{}\r\n", t.a, t.b, t.mark_b);
    let latest = [
        (String::new(), None),
        (lead, None),
        (j(&[&t.a]), Some(A)),
        (crlf, Some(B)),
        (j(&[&t.a, &t.b, &t.r]), Some(A)),
        (j(&[&t.a, &t.ra]), None),
        (j(&[&t.a, &t.b, &t.r, &t.c, &t.mark_c]), Some(C)),
        (j(&[&t.c, &t.f]), Some(F)),
        (j(&[&t.a, &empty_id]), Some(A)),
        (j(&[&t.a, &spaced]), Some(A)),
    ];
    for (notes, want) in &latest {
        assert_eq!(latest_ruling(notes), *want, "{notes:?}");
    }

    let q1 = bead("q.1");
    let q2 = bead("q.2");
    let pending = [
        (j(&[&t.a, &t.b, &t.r]), &q1, B, Some(R)),
        (j(&[&t.a, &t.b, &t.r, &t.mark_b]), &q1, B, Some(R)),
        (j(&[&t.a, &t.b, &t.r]), &q1, A, None),
        (j(&[&t.a, &t.b, &t.r]), &q2, B, None),
        (j(&[&t.a, &t.b, &t.r2]), &q1, B, None),
        (j(&[&t.a, &t.b, &t.r, &t.c]), &q1, B, None),
        (j(&[&t.b, &t.f]), &q1, C, None),
        (j(&[&t.a, &t.b]), &q1, B, None),
        (String::new(), &q1, B, None),
    ];
    for (notes, question, ruling, want) in &pending {
        assert_eq!(
            pending_reopen(notes, question, &rid(ruling)),
            *want,
            "{notes:?} {question} {ruling}"
        );
    }
    revocable_rows(t);
}

/// 取り消せる問いの真と偽の表と、裁定の行の頭の字の一致を見る。
fn revocable_rows(t: Table) {
    let truthy = [
        (item("closed", true, &[&t.a, &t.b]), B),
        (item("closed", true, &[&t.a, &t.b, &t.r]), B),
        (item("closed", true, &[&t.a, &t.b, &t.r]), A),
    ];
    for (it, ruling) in &truthy {
        assert!(revocable(it, &rid(ruling)), "{it:?} {ruling}");
    }
    let falsy = [
        (item("open", true, &[&t.a, &t.b]), B),
        (item("in_progress", true, &[&t.a, &t.b]), B),
        (item("closed", false, &[&t.a, &t.b]), B),
        (item("closed", true, &[&t.a, &t.b]), A),
        (item("open", true, &[&t.a, &t.b, &t.r]), B),
        (item("closed", true, &[&t.a, &t.b, &t.r, &t.c]), B),
        (item("closed", true, &[&t.a, &t.b, &t.r2]), B),
        (item("closed", true, &[]), A),
    ];
    for (it, ruling) in &falsy {
        assert!(!revocable(it, &rid(ruling)), "{it:?} {ruling}");
    }

    assert_eq!(RULING_LINE, LINE_PREFIX);
    assert_eq!(RULING_LINE, RULING_PREFIX);
    assert_eq!(TYPED_LINES[0], (RULING_LINE, NodeKind::Ruling));
}

#[test]
fn rvk_writes_line_then_reopens() {
    let tooth = "writes_line_then_reopens";
    for _ in 0..3 {
        let now = events::now();
        let m = minute(now);
        let first = format!("rv.1:{m}-1");
        let second = format!("rv.1:{m}-2");
        let notes = [
            "前置き".to_string(),
            rline(&first, "rv.1"),
            mark_line(&rid(&first), MarkRoute::Deliver, &m),
        ];
        let ledger = bd_list(&[bd_bead("rv.1", "closed", true, &notes)]);
        let req = request("rv.1", &first, "やはり待つ\n理由");

        let place = Place::new(tooth, "deliver", Some(&ledger));
        let got = place.revoke(&req, true, now);
        let after = minute(events::now());
        if after != m {
            continue; // 分を跨いだ（組み直して撃ち直す）。
        }
        assert_eq!(
            got,
            Revoked::Recorded(RevokeResponse {
                ruling: rid(&second),
                recorded_at: now,
                reopened_only: false,
            })
        );
        place.wait("bdw", 3);
        let bdw = place.argvs("bdw");
        assert_eq!(bdw.len(), 3, "{bdw:?}");
        assert_eq!(
            bdw[0],
            LedgerWrite::AppendNotes {
                id: bead("rv.1"),
                line: format!(
                    "裁定 id = {second}・問い = rv.1・取り消す = {first}・逐語 = やはり待つ\\n理由"
                ),
            }
            .argv()
        );
        assert_eq!(bdw[1], reopen_argv("rv.1", &second, &first));
        assert!(is_mark(&bdw[2], "rv.1", &second), "{bdw:?}");
        assert_eq!(place.argvs("scribe2"), [place.deliver_argv(&second)]);
        place.cwd_is_repo();

        // 配達の先の無い Writer は配達しない。
        let place = Place::new(tooth, "no-delivery", Some(&ledger));
        let got = place.revoke(&req, false, now);
        assert!(
            matches!(&got, Revoked::Recorded(r) if r.ruling.as_str() == second && !r.reopened_only),
            "{got:?}"
        );
        let bdw = place.argvs("bdw");
        assert_eq!(bdw.len(), 2, "{bdw:?}");
        assert_eq!(bdw[1], reopen_argv("rv.1", &second, &first));
        assert!(place.calls("scribe2").is_empty(), "配達の先が無いのに器を撃つ");
        place.cwd_is_repo();
        return;
    }
    panic!("3 回とも分を跨いだ");
}

/// done (5) の台帳（rv.1 から rv.5）。
fn five() -> String {
    bd_list(&[
        bd_bead(
            "rv.1",
            "closed",
            true,
            &[rline("rv.1:20260928T0100Z-1", "rv.1")],
        ),
        bd_bead(
            "rv.2",
            "closed",
            false,
            &[rline("rv.2:20260928T0100Z-1", "rv.2")],
        ),
        bd_bead(
            "rv.3",
            "open",
            true,
            &[rline("rv.3:20260928T0100Z-1", "rv.3")],
        ),
        bd_bead(
            "rv.4",
            "closed",
            true,
            &[
                rline("rv.4:20260928T0100Z-1", "rv.4"),
                rline("rv.4:20260928T0101Z-1", "rv.4"),
            ],
        ),
        bd_bead(
            "rv.5",
            "closed",
            true,
            &[
                rline("rv.5:20260928T0100Z-1", "rv.5"),
                rvline(
                    "rv.5:20260928T0102Z-1",
                    "rv.5",
                    "rv.5:20260928T0100Z-1",
                    "はい",
                ),
                rline("rv.5:20260928T0103Z-1", "rv.5"),
            ],
        ),
    ])
}

#[test]
fn rvk_refusals_write_nothing() {
    let tooth = "refusals_write_nothing";
    let place = Place::new(tooth, "refuse", Some(&five()));
    for (q, r, verbatim, want) in [
        (
            "rv.1",
            "rv.1:20260928T0100Z-1",
            " \n\t\n",
            Refusal::EmptyVerbatim,
        ),
        (
            "rv.9",
            "rv.9:20260928T0100Z-1",
            "はい",
            Refusal::UnknownQuestion,
        ),
        (
            "rv.2",
            "rv.2:20260928T0100Z-1",
            "はい",
            Refusal::UnknownQuestion,
        ),
        (
            "rv.3",
            "rv.3:20260928T0100Z-1",
            "はい",
            Refusal::StaleVersion,
        ),
        (
            "rv.4",
            "rv.4:20260928T0100Z-1",
            "はい",
            Refusal::StaleVersion,
        ),
        (
            "rv.5",
            "rv.5:20260928T0100Z-1",
            "はい",
            Refusal::StaleVersion,
        ),
        (
            "rv.5",
            "rv.5:20260928T0102Z-1",
            "はい",
            Refusal::StaleVersion,
        ),
    ] {
        let got = place.revoke(&request(q, r, verbatim), true, AT);
        assert_eq!(got, Revoked::Refused(want), "{q} {r} {verbatim:?}");
    }
    assert_eq!(Refusal::StaleVersion.http_status(), 409);
    place.nothing_shot("断り");

    let place = Place::new(tooth, "bd-fails", None);
    let got = place.revoke(&request("rv.1", "rv.1:20260928T0100Z-1", "はい"), true, AT);
    assert_eq!(got, Revoked::LedgerUnknown);
    place.nothing_shot("読めない台帳");
}

#[test]
fn rvk_write_failures() {
    let tooth = "write_failures";
    let ledger = bd_list(&[bd_bead(
        "rv.1",
        "closed",
        true,
        &[rline("rv.1:20260928T0100Z-1", "rv.1")],
    )]);
    let req = request("rv.1", "rv.1:20260928T0100Z-1", "待つ");

    let place = Place::new(tooth, "append", Some(&ledger));
    place.fail_at("bdw", 1);
    assert_eq!(place.revoke(&req, true, AT), Revoked::AppendFailed);
    assert_eq!(place.calls("bdw").len(), 1);
    assert!(place.calls("scribe2").is_empty(), "落ちた書きの後に配達する");

    let place = Place::new(tooth, "reopen", Some(&ledger));
    place.fail_at("bdw", 2);
    assert_eq!(
        place.revoke(&req, true, AT),
        Revoked::ReopenFailed(rid("rv.1:20260928T0441Z-1"))
    );
    let bdw = place.argvs("bdw");
    assert_eq!(bdw.len(), 2, "{bdw:?}");
    assert_eq!(bdw[1][0], "reopen", "{bdw:?}");
    assert!(place.calls("scribe2").is_empty(), "落ちた書きの後に配達する");
}

#[test]
fn rvk_retry_reopens_only() {
    let tooth = "retry_reopens_only";
    let (old, id) = ("rv.1:20260928T0100Z-1", "rv.1:20260928T0441Z-1");
    let lines = [rline(old, "rv.1"), rvline(id, "rv.1", old, "待つ")];
    let ledger = bd_list(&[bd_bead("rv.1", "closed", true, &lines)]);
    let req = request("rv.1", old, "もう一度");
    let want = Revoked::Recorded(RevokeResponse {
        ruling: rid(id),
        recorded_at: AT_RETRY,
        reopened_only: true,
    });

    // 行を足さず開き直しだけを撃ち、配達して印を置く。
    let place = Place::new(tooth, "deliver", Some(&ledger));
    assert_eq!(place.revoke(&req, true, AT_RETRY), want);
    place.wait("bdw", 2);
    let bdw = place.argvs("bdw");
    assert_eq!(bdw.len(), 2, "{bdw:?}");
    assert_eq!(bdw[0], reopen_argv("rv.1", id, old));
    assert!(is_mark(&bdw[1], "rv.1", id), "{bdw:?}");
    for argv in &bdw {
        assert!(
            !argv.iter().any(|a| a.contains("もう一度")),
            "撃ち直しの逐語を書く {argv:?}"
        );
    }
    assert_eq!(place.argvs("scribe2"), [place.deliver_argv(id)]);
    place.cwd_is_repo();

    // 配達の先が無ければ開き直しの 1 回だけ。
    let place = Place::new(tooth, "no-delivery", Some(&ledger));
    assert_eq!(place.revoke(&req, false, AT_RETRY), want);
    assert_eq!(place.argvs("bdw"), [reopen_argv("rv.1", id, old)]);
    assert!(place.calls("scribe2").is_empty());

    // 開き直しがまた落ちれば同じ id で ReopenFailed。
    let place = Place::new(tooth, "fail", Some(&ledger));
    place.fail_at("bdw", 1);
    assert_eq!(
        place.revoke(&req, true, AT_RETRY),
        Revoked::ReopenFailed(rid(id))
    );
    assert_eq!(place.calls("bdw").len(), 1);
    assert!(place.calls("scribe2").is_empty(), "落ちた書きの後に配達する");
    stale_and_http(tooth, (old, id), lines, ledger, req);
}

/// 撃ち直しが古い版で断られる台帳と、口の server での開き直しだけの撃ち直しを見る。
fn stale_and_http(
    tooth: &str,
    (old, id): (&str, &str),
    lines: [String; 2],
    ledger: String,
    req: RevokeRequest,
) {
    // 答え直した後・問いの欄が違う・open の台帳は 409 で何も撃たない。
    let answered = [
        lines[0].clone(),
        lines[1].clone(),
        rline("rv.1:20260928T0500Z-1", "rv.1"),
    ];
    let other = [lines[0].clone(), rvline(id, "rv.2", old, "待つ")];
    for (scene, text) in [
        (
            "answered",
            bd_list(&[bd_bead("rv.1", "closed", true, &answered)]),
        ),
        (
            "other-question",
            bd_list(&[bd_bead("rv.1", "closed", true, &other)]),
        ),
        ("open", bd_list(&[bd_bead("rv.1", "open", true, &lines)])),
    ] {
        let place = Place::new(tooth, scene, Some(&text));
        assert_eq!(
            place.revoke(&req, true, AT_RETRY),
            Revoked::Refused(Refusal::StaleVersion),
            "{scene}"
        );
        place.nothing_shot(scene);
    }

    // 口の server でも同じ（200 の本文の reopened_only は真）。
    let place = Place::new(tooth, "http", Some(&ledger));
    let addr = place.serve();
    let reply = post(addr, &req);
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert!(reply.body.contains(r#""reopened_only":true"#), "{}", reply.body);
    let got: RevokeResponse = wire::decode(&reply.body).expect("応答の形");
    assert_eq!(got.ruling, rid(id));
    assert!(got.reopened_only);
    assert_eq!(place.argvs("bdw"), [reopen_argv("rv.1", id, old)]);
    assert!(place.calls("scribe2").is_empty());
}

#[test]
fn rvk_http_guard_and_reply() {
    let tooth = "http_guard_and_reply";
    let ledger = bd_list(&[
        bd_bead(
            "rv.1",
            "closed",
            true,
            &[rline("rv.1:20260928T0100Z-1", "rv.1")],
        ),
        bd_bead(
            "rv.4",
            "closed",
            true,
            &[
                rline("rv.4:20260928T0100Z-1", "rv.4"),
                rline("rv.4:20260928T0101Z-1", "rv.4"),
            ],
        ),
    ]);
    let good = request("rv.1", "rv.1:20260928T0100Z-1", "待つ");
    let body = wire::encode(&good).expect("要求の電文");

    let place = Place::new(tooth, "reply", Some(&ledger));
    let addr = place.serve();
    // 1 本の引きの口は取り消せる問いを返す。
    let reply = send(addr, "GET", &format!("{ITEM_PATH}rv.1"), "", "");
    assert_eq!(reply.status, 200, "{}", reply.body);
    let item: LedgerItem = wire::decode(&reply.body).expect("1 本の形");
    assert!(revocable(&item, &rid("rv.1:20260928T0100Z-1")), "{item:?}");
    // 守り。
    let port = addr.port();
    let reply = send(
        addr,
        "POST",
        REVOKE_PATH,
        &format!("Origin: http://evil.example:{port}\r\n"),
        &body,
    );
    assert_eq!((reply.status, reply.body.as_str()), (403, "origin"));
    let reply = send(addr, "POST", REVOKE_PATH, "", "{");
    assert_eq!((reply.status, reply.body.as_str()), (400, "bad-body"));
    // 断り。
    for (req, status, want) in [
        (
            request("rv.4", "rv.4:20260928T0100Z-1", "待つ"),
            409,
            Refusal::StaleVersion,
        ),
        (
            request("rv.9", "rv.9:20260928T0100Z-1", "待つ"),
            404,
            Refusal::UnknownQuestion,
        ),
        (
            request("rv.1", "rv.1:20260928T0100Z-1", " "),
            400,
            Refusal::EmptyVerbatim,
        ),
    ] {
        let reply = post(addr, &req);
        assert_eq!(reply.status, status, "{req:?}: {}", reply.body);
        assert_eq!(refused(&reply), want, "{req:?}");
    }
    assert!(place.calls("bdw").is_empty(), "守りと断りで偽の bdw を撃つ");
    written_and_failed(tooth, ledger, good, place, addr);
}

/// 書けた時の応答と、書きが落ちた時・台帳が読めない時の応答を見る。
fn written_and_failed(
    tooth: &str,
    ledger: String,
    good: RevokeRequest,
    place: Place,
    addr: SocketAddr,
) {
    // 書いた。
    let reply = post(addr, &good);
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert!(
        reply.head.contains("Content-Type: application/json"),
        "{}",
        reply.head
    );
    let got: RevokeResponse = wire::decode(&reply.body).expect("応答の形");
    assert!(got.ruling.as_str().starts_with("rv.1:"), "{got:?}");
    assert!(!got.reopened_only);
    assert_eq!(place.calls("bdw").len(), 2);
    assert!(place.calls("scribe2").is_empty(), "配達の先が無いのに器を撃つ");

    // 書きが落ちた。
    let place = Place::new(tooth, "append", Some(&ledger));
    place.fail_at("bdw", 1);
    let reply = post(place.serve(), &good);
    assert_eq!((reply.status, reply.body.as_str()), (502, "ledger-append"));
    let place = Place::new(tooth, "reopen", Some(&ledger));
    place.fail_at("bdw", 2);
    let reply = post(place.serve(), &good);
    assert_eq!(reply.status, 502, "{}", reply.body);
    let id = reply
        .body
        .strip_prefix("ledger-reopen ")
        .unwrap_or_else(|| panic!("字 ledger-reopen で始まらない: {}", reply.body));
    assert!(id.starts_with("rv.1:"), "{id}");
    assert!(place.argvs("bdw")[0][2].contains(id), "応答の id が取り消しの行の id でない");
    // 台帳が読めない。
    let place = Place::new(tooth, "unknown", Some(&ledger));
    let addr = place.serve();
    place.bd_fails();
    let reply = post(addr, &good);
    assert_eq!((reply.status, reply.body.as_str()), (503, "ledger-unknown"));
    place.nothing_shot("読めない台帳");
}

#[test]
fn rvk_line_reads_as_ruling() {
    let (old, id) = ("rv.1:20260928T0100Z-1", "rv.1:20260928T0441Z-1");
    let ledger = bd_list(&[bd_bead(
        "rv.1",
        "open",
        true,
        &[
            rline(old, "rv.1"),
            mark_line(&rid(old), MarkRoute::Deliver, "20260928T0101Z"),
            rvline(id, "rv.1", old, "待つ"),
        ],
    )]);
    assert_eq!(
        undelivered(&ledger),
        Reading::Known(vec![Pending {
            question: bead("rv.1"),
            ruling: rid(id),
        }])
    );
    let g = build(&Inputs {
        design_index: "",
        ledger: &ledger,
        events: "",
    });
    assert!(
        g.nodes
            .iter()
            .any(|n| n.id == id && n.kind == NodeKind::Ruling),
        "{:?}",
        g.nodes
    );
    assert!(
        g.edges
            .iter()
            .any(|e| e.from == id && e.to == "rv.1" && e.edge_type == EdgeType::Answers),
        "{:?}",
        g.edges
    );
}

#[test]
fn rvk_ruling_rs_text() {
    let read = |rel: &str| fs::read_to_string(manifest().join(rel)).expect("src を読む");
    let ruling_rs = read("src/server/ruling.rs");
    assert_eq!(ruling_rs.matches("deliver_argv(").count(), 2);
    let revoke_body = body_of(&ruling_rs, "pub fn revoke(");
    for word in [
        "reread(",
        "parse_bd(",
        "revocable(",
        "pending_reopen(",
        "next_id(",
        "reopen(",
    ] {
        assert!(revoke_body.contains(word), "revoke に {word} が無い");
    }
    for word in [".got()", ".text()", ".read()", "CloseItem", "deliver_argv("] {
        assert!(!revoke_body.contains(word), "revoke に {word}");
    }
    let reopen_body = body_of(&ruling_rs, "fn reopen(");
    for word in ["ReopenItem", "deliver("] {
        assert!(reopen_body.contains(word), "reopen に {word} が無い");
    }
    for word in ["AppendNotes", "CloseItem", "deliver_argv(", "next_id("] {
        assert!(!reopen_body.contains(word), "reopen に {word}");
    }
    let route = read("src/server/routes/revoke.rs");
    assert!(route.contains("guarded("), "口が guarded を通らない");
    assert!(!route.contains("\"/api/"), "口の file に /api/ の字");
}

/// filter の語（contracts の verify の filter の語を畳んだ語と、後の行 g-gz の接頭辞）。
const FILTER: [&str; 129] = [
    "aaround_",
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "afocus_",
    "aord_",
    "apop_",
    "askcard_",
    "athr_",
    "batchpanel_",
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadopt_",
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "denv_",
    "ecache_",
    "flight_",
    "fmark_",
    "frame_",
    "fserve_",
    "fstop_",
    "gapspage_",
    "gfresh_",
    "ghb_",
    "glabel_",
    "gnav_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "hacols_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
    "hfig_",
    "hnunk_",
    "hook_",
    "hruling_",
    "hsblock_",
    "hsderive_",
    "hspage_",
    "hsym_",
    "iclose_",
    "ilink_",
    "kcli_",
    "klink_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lspark_",
    "mapview_",
    "mkeys_",
    "mlink_",
    "mstore_",
    "mtree_",
    "nact_",
    "nbatch_",
    "ncard_",
    "nextstep_",
    "nodepage_",
    "nstall_",
    "nsum_",
    "nsumw_",
    "ntime_",
    "nxact_",
    "parts_",
    "pclosed_",
    "pfold_",
    "pipe_",
    "plimit_",
    "pmore_",
    "pquest_",
    "project_",
    "ptitle_",
    "punmap_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "question_",
    "relay_",
    "rhold_",
    "runsdoc_",
    "saxis_",
    "sclosed_",
    "seatblock_",
    "seatcard_",
    "server_",
    "sesplit_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "stcli_",
    "steady_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "topbar_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
    "pgz_",
];

#[test]
fn rvk_own_names_clean() {
    let text = fs::read_to_string(manifest().join("tests/teeth3/rvk.rs")).expect("歯の file");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            w[1].trim()
                .strip_prefix("fn ")
                .and_then(|r| r.split('(').next())
                .unwrap_or_else(|| panic!("test の属性の後の行が fn でない: {}", w[1]))
        })
        .collect();
    assert!(names.len() >= 11, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("rvk_")
            .unwrap_or_else(|| panic!("rvk_ で始まらない: {name}"));
        for word in FILTER {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
