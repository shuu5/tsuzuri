//! 読む側の口の歯（接頭辞 server_read_・設計ノート surface-base 便 e-read の完了の条件）。
//! 偽の bd（bead 8 本の fixture を返す script）と偽の設計の道具（実物の索引の fixture を返す script）を
//! 歯ごとの一時の dir に置き、event log の fixture を一時の state dir に写す。
//! server は同じ process の thread で 127.0.0.1 の空き port に立てる。

use std::fmt::Debug;
use std::fs;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tsuzuri_boundary::server::design::{DESIGN_DIR, FOLIO, FOLIO_TIMEOUT};
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::{PipelineBoard, Reading};
use tsuzuri_contract::graph::{GraphDoc, GraphSource, NodeKind, Verdict};
use tsuzuri_contract::stats::{LedgerStats, NextStep};
use tsuzuri_contract::wire;
use tsuzuri_core::graph::{self, Graph, Inputs};

/// 偽の設計の道具の振る舞い。
#[derive(Clone, Copy)]
enum Folio {
    /// 索引の fixture を返して rc 0（撃たれるたびに引数と cwd を calls/ に残す）。
    Ok,
    /// 索引の fixture を返すが rc 1。
    Rc1,
    /// 返さない（`FOLIO_TIMEOUT` を越えて眠る）。
    Hang,
    /// program が無い。
    Missing,
}

/// event log の置き方。
#[derive(Clone, Copy)]
enum Log {
    /// state dir に fixture を写す。
    Fixture,
    /// state dir は在るが event log の file が無い。
    NoFile,
    /// --state-dir を省く。
    NoStateDir,
}

fn fixture(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(rel)
}

fn read_fixture(rel: &str) -> String {
    fs::read_to_string(fixture(rel)).expect("fixture")
}

const LEDGER: &str = "ledger/bd-list-8.json";
const INDEX: &str = "graph/real/design-index.tsv";
const EVENTS: &str = "graph/real/events.jsonl";

/// 歯ごとの作業場（repo の置き場・面の file の置き場・state dir・偽の bd と偽の設計の道具）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
    state: PathBuf,
    folio: Folio,
    log: Log,
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

impl Place {
    fn new(name: &str, folio: Folio, log: Log) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("server_read")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let files = root.join("files");
        let state = root.join("state");
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::create_dir_all(repo.join(DESIGN_DIR).join("adr")).expect("設計文書の dir");
        fs::create_dir_all(&files).expect("面の file の置き場");
        fs::create_dir_all(root.join("calls")).expect("記録の置き場");
        fs::write(files.join("index.html"), "tz").expect("index.html");
        for mark in ["issues.jsonl", "interactions.jsonl"] {
            fs::write(repo.join(".beads").join(mark), "{}\n").expect("印の file");
        }
        fs::write(repo.join(DESIGN_DIR).join("rules.yaml"), "rules: []\n").expect("設計文書");
        fs::write(repo.join(DESIGN_DIR).join("adr/ADR-7.yaml"), "id: ADR-7\n").expect("設計文書");
        match log {
            Log::Fixture => {
                fs::create_dir_all(state.join("fleet")).expect("state dir");
                fs::write(state.join("fleet/events.jsonl"), read_fixture(EVENTS))
                    .expect("event log");
            }
            Log::NoFile => fs::create_dir_all(&state).expect("state dir"),
            Log::NoStateDir => {}
        }
        script(
            &root.join("bd"),
            &format!("exec cat '{}'", fixture(LEDGER).display()),
        );
        let (calls, index) = (root.join("calls"), fixture(INDEX));
        let (calls, index) = (calls.display(), index.display());
        let last = match folio {
            Folio::Ok | Folio::Missing => format!("exec cat '{index}'"),
            Folio::Rc1 => format!("cat '{index}'\nexit 1"),
            Folio::Hang => "exec sleep 30".to_string(),
        };
        script(
            &root.join("folio"),
            &format!(
                "for a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{calls}'/$$.args\n\
                 pwd -P > '{calls}'/$$.cwd\n\
                 echo '標準エラーの字' >&2\n\
                 {last}"
            ),
        );
        Place {
            root,
            repo,
            files,
            state,
            folio,
            log,
        }
    }

    fn folio_program(&self) -> PathBuf {
        match self.folio {
            Folio::Missing => self.root.join("no-such-folio"),
            _ => self.root.join("folio"),
        }
    }

    fn events_log(&self) -> PathBuf {
        self.state.join("fleet/events.jsonl")
    }

    fn config(&self) -> Config {
        Config {
            bd: self.root.join("bd").into(),
            state_dir: match self.log {
                Log::NoStateDir => None,
                _ => Some(self.state.clone()),
            },
            folio: self.folio_program().into(),
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        }
    }

    fn serve(&self) -> SocketAddr {
        let server = Server::bind(&self.config()).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }

    /// 偽の設計の道具が撃たれた回ごとの（引数の列・cwd）。
    fn folio_calls(&self) -> Vec<(Vec<String>, String)> {
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
                )
            })
            .collect()
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_secs()
}

/// 要求を 1 つ撃ち、接続が閉じるまで応答の（状態の code・頭・本文）を読む。
fn request(addr: SocketAddr, raw: &str) -> (u16, String, String) {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(FOLIO_TIMEOUT * 3))
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
    (status, head.to_string(), body.to_string())
}

/// 口の電文を契約の型で読む（型は受ける側が決める）。
macro_rules! decode {
    ($body:expr) => {
        wire::decode(&$body).unwrap_or_else(|e| panic!("契約の型の形でない {e}: {}", $body))
    };
}

/// GET を撃ち、200 と JSON を確かめて本文を返す。前後の時刻（epoch 秒）も返す。
fn get(addr: SocketAddr, path: &str) -> (String, u64, u64) {
    let from = now();
    let (status, head, body) = request(addr, &format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n"));
    let to = now();
    assert_eq!(status, 200, "{path}: {body}");
    assert!(
        head.contains("Content-Type: application/json"),
        "{path}: {head}"
    );
    (body, from, to)
}

fn pipeline(addr: SocketAddr) -> (PipelineBoard, u64, u64) {
    let (body, from, to) = get(addr, "/api/pipeline");
    (decode!(body), from, to)
}

fn metrics(addr: SocketAddr) -> (Reading<LedgerStats>, u64, u64) {
    let (body, from, to) = get(addr, "/api/metrics");
    (decode!(body), from, to)
}

fn next(addr: SocketAddr) -> (NextStep, u64, u64) {
    let (body, from, to) = get(addr, "/api/next");
    (decode!(body), from, to)
}

fn graph_doc(addr: SocketAddr) -> GraphDoc {
    decode!(get(addr, "/api/graph").0)
}

/// 口の値が、要求の前後のどれかの時刻で中核の関数を直に呼んだ値と一致する。
fn same_at_some_now<T: PartialEq + Debug>(
    what: &str,
    got: &T,
    from: u64,
    to: u64,
    want: impl Fn(u64) -> T,
) {
    assert!(
        (from..=to).any(|t| &want(t) == got),
        "{what}: 中核の関数の値と違う（{from}..={to}）\n口: {got:?}\n中核: {:?}",
        want(from)
    );
}

/// 口のグラフが中核の crate の Graph と check の値と一致する。
fn assert_graph_matches(doc: &GraphDoc, g: &Graph) {
    assert_eq!(doc.nodes, g.nodes, "節点");
    assert_eq!(doc.edges, g.edges, "辺");
    let unread: Vec<String> = g
        .unread
        .iter()
        .map(|s| wire::encode(s).expect("出所の語"))
        .collect();
    let got: Vec<String> = doc
        .unread
        .iter()
        .map(|s| wire::encode(s).expect("出所の語"))
        .collect();
    assert_eq!(got, unread, "読めなかった出所");
    assert_eq!(
        doc.beads.keys().collect::<Vec<_>>(),
        g.beads.keys().collect::<Vec<_>>(),
        "bead の属性の id"
    );
    for (id, b) in &g.beads {
        let d = &doc.beads[id];
        assert_eq!(
            (&d.kind, &d.status, &d.labels, &d.pointers, &d.touches),
            (&b.kind, &b.status, &b.labels, &b.pointers, &b.touches),
            "{id} の属性"
        );
    }
    assert_eq!(
        doc.runs.keys().collect::<Vec<_>>(),
        g.runs.keys().collect::<Vec<_>>(),
        "走行の属性の id"
    );
    for (id, r) in &g.runs {
        let d = &doc.runs[id];
        assert_eq!(
            (&d.stage, &d.account),
            (&r.stage, &r.account),
            "{id} の属性"
        );
    }
    let invariants = graph::check(g);
    assert_eq!(doc.invariants.len(), 12);
    for (d, i) in doc.invariants.iter().zip(&invariants) {
        let (verdict, ids) = match &i.verdict {
            graph::Verdict::Pass => (Verdict::Pass, vec![]),
            graph::Verdict::Violation(ids) => (Verdict::Violation, ids.clone()),
            graph::Verdict::Unknown => (Verdict::Unknown, vec![]),
        };
        assert_eq!(
            (d.id.as_str(), d.verdict, d.violations as usize, &d.ids),
            (i.id, verdict, ids.len(), &ids),
            "不変条件 {}",
            i.id
        );
    }
    assert_eq!(
        (doc.skipped.design as usize, doc.skipped.ledger as usize),
        (g.skipped.design_edges, g.skipped.ledger_edges),
        "組まずに数えた辺"
    );
}

/// 出所ごとの節点の数（設計・台帳・走行）。
fn per_source(doc: &GraphDoc) -> (usize, usize, usize) {
    let count = |s: graph::Source| {
        doc.nodes
            .iter()
            .filter(|n| s.kinds().contains(&n.kind))
            .count()
    };
    (
        count(graph::Source::Design),
        count(graph::Source::Ledger),
        count(graph::Source::Runs),
    )
}

#[test]
fn server_read_four_routes_match_core() {
    let place = Place::new("match", Folio::Ok, Log::Fixture);
    let addr = place.serve();
    let (ledger, index, events) = (
        read_fixture(LEDGER),
        read_fixture(INDEX),
        read_fixture(EVENTS),
    );

    let (board, from, to) = pipeline(addr);
    assert!(
        matches!(&board.cards, Reading::Known(c) if !c.is_empty()),
        "{board:?}"
    );
    same_at_some_now("pipeline", &board, from, to, |t| {
        tsuzuri_core::pipeline::board(&ledger, &events, t).board
    });

    let (body, from, to) = get(addr, "/api/metrics");
    let stats: Reading<LedgerStats> = decode!(body);
    assert!(matches!(stats, Reading::Known(_)), "{stats:?}");
    // 小数は電文の字を読み直すと最後の 1 桁がずれることが在るので、読んだ値でなく電文の字で比べる。
    same_at_some_now("metrics", &body, from, to, |t| {
        wire::encode(&tsuzuri_core::ledger::stats(&ledger, t)).expect("指標の電文")
    });

    let (step, from, to) = next(addr);
    same_at_some_now("next", &step, from, to, |t| {
        tsuzuri_core::next_step::next_step(&ledger, &events, t)
    });

    let doc = graph_doc(addr);
    let g = graph::build(&Inputs {
        design_index: &index,
        ledger: &ledger,
        events: &events,
    });
    assert!(g.unread.is_empty(), "{:?}", g.unread);
    assert!(doc.unread.is_empty(), "{:?}", doc.unread);
    assert_graph_matches(&doc, &g);
    let (design, beads, runs) = per_source(&doc);
    assert!(
        design > 0 && beads > 0 && runs > 0,
        "{design} {beads} {runs}"
    );
    // 3 つの出所が読めているので、材料の在る 9 本は「まだ分からない」でない。
    let unknown: Vec<&str> = doc
        .invariants
        .iter()
        .filter(|i| i.verdict == Verdict::Unknown)
        .map(|i| i.id.as_str())
        .collect();
    assert_eq!(unknown, graph::check::UNMEASURED.to_vec());
}

#[test]
fn server_read_folio_argv_and_cwd() {
    let place = Place::new("argv", Folio::Ok, Log::Fixture);
    let addr = place.serve();
    let doc = graph_doc(addr);
    assert!(doc.unread.is_empty(), "{:?}", doc.unread);
    let calls = place.folio_calls();
    assert_eq!(
        calls.len(),
        2,
        "グラフの口 1 回に設計の道具 2 回（索引と要約）: {calls:?}"
    );
    let dir = place.repo.join(DESIGN_DIR).display().to_string();
    let repo = format!(
        "{}\n",
        place.repo.canonicalize().expect("repo の実体").display()
    );
    // pid の順は撃った順と限らないので、引数の列を並べ替えて比べる。
    let mut argv: Vec<&Vec<String>> = calls.iter().map(|(args, _)| args).collect();
    argv.sort();
    assert_eq!(
        argv,
        [
            &["graph", "--print", "--dir", dir.as_str()].map(str::to_string).to_vec(),
            &["graph", "--print", "--summary", "--dir", dir.as_str()]
                .map(str::to_string)
                .to_vec(),
        ],
        "引数の列"
    );
    for (_, cwd) in &calls {
        assert_eq!(cwd, &repo, "cwd は repo の置き場");
    }
    // 口は要求のたびに組み直す（設計の道具を撃ち直す）。
    graph_doc(addr);
    assert_eq!(place.folio_calls().len(), 4);
    // 板と指標と次の一手は設計の索引も要約も使わない。
    pipeline(addr);
    metrics(addr);
    next(addr);
    assert_eq!(place.folio_calls().len(), 4);
    assert_eq!(FOLIO, "folio", "program の名の既定");
}

#[test]
fn server_read_runs_unread_without_log() {
    for (name, log) in [("no-state-dir", Log::NoStateDir), ("no-log", Log::NoFile)] {
        let place = Place::new(name, Folio::Ok, log);
        let addr = place.serve();
        assert_eq!(pipeline(addr).0.cards, Reading::Unknown, "{name}");
        let doc = graph_doc(addr);
        assert_eq!(doc.unread, vec![GraphSource::Runs], "{name}");
        let (design, beads, runs) = per_source(&doc);
        assert!(design > 0 && beads > 0, "{name}: {design} {beads}");
        assert_eq!(runs, 0, "{name}");
        assert!(doc.runs.is_empty(), "{name}");
        let g = graph::build(&Inputs {
            design_index: &read_fixture(INDEX),
            ledger: &read_fixture(LEDGER),
            events: "",
        });
        assert_graph_matches(&doc, &g);
        // 指標は台帳だけで数える。次の一手は読めない出所が在っても 200 で返す。
        assert!(matches!(metrics(addr).0, Reading::Known(_)), "{name}");
        next(addr);
    }
}

#[test]
fn server_read_design_unread_on_folio_failure() {
    for (name, folio) in [
        ("folio-rc1", Folio::Rc1),
        ("folio-missing", Folio::Missing),
        ("folio-hang", Folio::Hang),
    ] {
        let place = Place::new(name, folio, Log::Fixture);
        let addr = place.serve();
        let started = Instant::now();
        let doc = graph_doc(addr);
        let took = started.elapsed();
        assert_eq!(doc.unread, vec![GraphSource::Design], "{name}");
        let (design, beads, runs) = per_source(&doc);
        assert_eq!(design, 0, "{name}");
        assert!(beads > 0 && runs > 0, "{name}: {beads} {runs}");
        assert!(
            doc.nodes.iter().all(|n| n.kind != NodeKind::Article),
            "{name}"
        );
        let g = graph::build(&Inputs {
            design_index: "",
            ledger: &read_fixture(LEDGER),
            events: &read_fixture(EVENTS),
        });
        assert_graph_matches(&doc, &g);
        if matches!(folio, Folio::Hang) {
            assert!(
                took >= FOLIO_TIMEOUT - Duration::from_millis(100),
                "{name}: {took:?}"
            );
            assert!(
                took < FOLIO_TIMEOUT + Duration::from_secs(2),
                "{name}: {took:?}"
            );
        }
        if !matches!(folio, Folio::Missing) {
            assert!(
                !place.folio_calls().is_empty(),
                "{name}: 偽の設計の道具が撃たれていない"
            );
        }
    }
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

fn board_events(b: &str) -> usize {
    b.matches("event: board-changed\n").count()
}

/// 板の印の file の中身を丸ごと替える（作業場の直下に書いてから移すので、見張りから書きかけは見えない）。
fn put(root: &Path, path: &Path, text: &str) {
    let tmp = root.join("put.tmp");
    fs::write(&tmp, text).expect("移す前の印");
    fs::rename(&tmp, path).expect("印を移す");
}

#[test]
fn server_read_board_changed_within_5s() {
    let place = Place::new("sse", Folio::Ok, Log::Fixture);
    let addr = place.serve();
    let (mut s, mut buf) = subscribe(addr);
    read_until(
        &mut s,
        &mut buf,
        Instant::now() + Duration::from_millis(1100),
        |b| board_events(b) >= 1,
    );
    assert_eq!(board_events(&buf), 0, "変化の前に知らせが出る: {buf}");
    let design_file = place.repo.join(DESIGN_DIR).join("adr/ADR-7.yaml");
    for (n, what) in ["event log に 1 行", "設計文書の書き換え"]
        .into_iter()
        .enumerate()
    {
        let changed = Instant::now();
        if n == 0 {
            let log = fs::read_to_string(place.events_log()).expect("event log");
            let text = format!("{log}{{\"kind\":\"RunCost\",\"run\":\"x\"}}\n");
            put(&place.root, &place.events_log(), &text);
        } else {
            put(&place.root, &design_file, "id: ADR-7\ntitle: 書き換え\n");
        }
        read_until(&mut s, &mut buf, changed + Duration::from_secs(5), |b| {
            board_events(b) > n && b.ends_with("\n\n")
        });
        let took = changed.elapsed();
        assert_eq!(
            board_events(&buf),
            n + 1,
            "{what}: 5 秒以内に 1 件でない（{took:?}）: {buf}"
        );
        assert!(took < Duration::from_secs(5), "{what}: {took:?}");
    }
    let frame = buf
        .split("\n\n")
        .find(|f| f.contains("event: board-changed"))
        .expect("event");
    let data = frame
        .lines()
        .find_map(|l| l.strip_prefix("data: "))
        .expect("data の行");
    assert!(data.starts_with("{\"at\":"), "{data}");
    // 変化が無ければ次の知らせは出ない。
    read_until(
        &mut s,
        &mut buf,
        Instant::now() + Duration::from_millis(1100),
        |b| board_events(b) >= 3,
    );
    assert_eq!(board_events(&buf), 2, "変化 1 回に知らせが 2 件以上: {buf}");
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

const ROUTES: [&str; 4] = ["/api/pipeline", "/api/metrics", "/api/next", "/api/graph"];

#[test]
fn server_read_routes_write_nothing() {
    let place = Place::new("bytes", Folio::Ok, Log::Fixture);
    let addr = place.serve();
    let before = (tree(&place.repo), tree(&place.state));
    for path in ROUTES {
        let (status, _, body) = request(addr, &format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n"));
        assert_eq!(status, 200, "{path}: {body}");
    }
    assert!(
        before == (tree(&place.repo), tree(&place.state)),
        "口を読んだ後に repo か state dir の byte が変わる"
    );
}

#[test]
fn server_read_non_get_405_writes_nothing() {
    let place = Place::new("method", Folio::Ok, Log::Fixture);
    let addr = place.serve();
    let before = (tree(&place.repo), tree(&place.state));
    for path in ROUTES {
        for method in ["POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"] {
            let body = r#"{"cards":"unknown"}"#;
            let (status, head, _) = request(
                addr,
                &format!(
                    "{method} {path} HTTP/1.1\r\nHost: x\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                    body.len()
                ),
            );
            assert_eq!(status, 405, "{method} {path}");
            assert!(head.contains("Allow: GET"), "{head}");
        }
    }
    assert!(
        before == (tree(&place.repo), tree(&place.state)),
        "405 の後に repo か state dir の byte が変わる"
    );
    assert!(place.folio_calls().is_empty(), "405 で設計の道具を撃つ");
}

/// tz を撃ち、標準エラーの最初の行から口の住所を読む。
fn spawn_tz(place: &Place, extra: &[String]) -> (std::process::Child, SocketAddr) {
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
    let mut line = String::new();
    BufReader::new(child.stderr.take().expect("標準エラー"))
        .read_line(&mut line)
        .expect("口の住所の行");
    let addr = line
        .trim()
        .strip_prefix("tz surface serve: http://")
        .and_then(|a| a.strip_suffix('/'))
        .and_then(|a| a.parse().ok())
        .unwrap_or_else(|| panic!("口の住所の行でない: {line}"));
    (child, addr)
}

#[test]
fn server_read_tz_takes_state_dir_and_folio() {
    let place = Place::new("flags", Folio::Ok, Log::Fixture);
    let (mut child, addr) = spawn_tz(
        &place,
        &[
            "--state-dir".to_string(),
            place.state.display().to_string(),
            format!("--folio={}", place.root.join("folio").display()),
        ],
    );
    let got = std::panic::catch_unwind(|| (graph_doc(addr), pipeline(addr).0));
    let _ = child.kill();
    let _ = child.wait();
    let (doc, board) = got.expect("口を読めない");
    assert!(doc.unread.is_empty(), "{:?}", doc.unread);
    assert!(matches!(board.cards, Reading::Known(_)));

    // 省いても起動し、走行の出所は読めない扱い。
    let (mut child, addr) = spawn_tz(&place, &[]);
    let got = std::panic::catch_unwind(|| pipeline(addr).0);
    let _ = child.kill();
    let _ = child.wait();
    assert_eq!(got.expect("口を読めない").cards, Reading::Unknown);

    // 空の値は使い方の誤りで rc 1。
    for bad in ["--state-dir=", "--folio="] {
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
