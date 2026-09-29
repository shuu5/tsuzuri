//! 読みの合流の歯（接頭辞 server_coalesce_・設計ノート surface-base 便 e-coalesce の完了の条件）。
//! 偽の bd と偽の設計の道具は、撃たれるたびに作業場の file に 1 行を足してから 500 ミリ秒眠り、
//! fixture を標準出力へ出す script。偽の bd が出す字は作業場に写した台帳の file で、歯の途中で替える。
//! 作業場に fail の file が在る間、偽の bd は眠った後に何も出さず rc 1 で落ちる。
//! server は同じ process の thread で 127.0.0.1 の空き port に立て、同時の要求は thread を並べて撃つ。
//! 変化の見張りの周期の読み（起動の後 5 秒ごと）と重ならないよう、数える歯は起動の直後に撃ち終える。

use std::fmt::Debug;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tsuzuri_boundary::server::coalesce::{Coalesce, GRACE};
use tsuzuri_boundary::server::design::{DESIGN_DIR, FOLIO_TIMEOUT, FOLIO_WAIT};
use tsuzuri_boundary::server::ledger::{BD_TIMEOUT, BD_WAIT};
use tsuzuri_boundary::server::{Config, Server, ruling};
use tsuzuri_contract::board::{PipelineBoard, Reading};
use tsuzuri_contract::graph::{GraphDoc, GraphSource, GraphView};
use tsuzuri_contract::ledger::{BeadId, LedgerItem, LedgerList};
use tsuzuri_contract::stats::{LedgerStats, NextStep};
use tsuzuri_contract::surface::RulingRequest;
use tsuzuri_contract::wire;

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

/// fixture の台帳に在る bead の id。
const BEAD: &str = "fx-hub.3";

/// 歯ごとの作業場（repo の置き場・面の file の置き場・state dir・偽の bd と偽の設計の道具）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
    state: PathBuf,
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("server_coalesce")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let files = root.join("files");
        let state = root.join("state");
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::create_dir_all(repo.join(DESIGN_DIR).join("adr")).expect("設計文書の dir");
        fs::create_dir_all(&files).expect("面の file の置き場");
        fs::write(files.join("index.html"), "tz").expect("index.html");
        for mark in ["issues.jsonl", "interactions.jsonl"] {
            fs::write(repo.join(".beads").join(mark), "{}\n").expect("印の file");
        }
        fs::write(repo.join(DESIGN_DIR).join("rules.yaml"), "rules: []\n").expect("設計文書");
        fs::create_dir_all(state.join("fleet")).expect("state dir");
        fs::write(state.join("fleet/events.jsonl"), read_fixture(EVENTS)).expect("event log");
        fs::write(root.join("ledger.json"), read_fixture(LEDGER)).expect("台帳の写し");
        let (r, index) = (root.display(), fixture(INDEX));
        script(
            &root.join("bd"),
            &format!(
                "echo bd >> '{r}/bd.calls'\nsleep 0.5\n\
                 if [ -e '{r}/fail' ]; then exit 1; fi\n\
                 exec cat '{r}/ledger.json'"
            ),
        );
        // 要約の読み（引数に --summary が在る回）と裁定の書き出しの読み（引数に --emit-rulings が在る回）は
        // 別の記録の file に書く。
        script(
            &root.join("folio"),
            &format!(
                "log=folio\nfor a in \"$@\"; do [ \"$a\" = --summary ] && log=summary; \
                 [ \"$a\" = --emit-rulings ] && log=rulings; done\n\
                 echo folio >> '{r}'/$log.calls\nsleep 0.5\nexec cat '{}'",
                index.display()
            ),
        );
        Place {
            root,
            repo,
            files,
            state,
        }
    }

    fn config(&self) -> Config {
        Config {
            bd: self.root.join("bd").into(),
            state_dir: Some(self.state.clone()),
            folio: self.root.join("folio").into(),
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

    /// 偽の program が撃たれた回数（記録の file の行の数）。
    fn calls(&self, program: &str) -> usize {
        fs::read_to_string(self.root.join(format!("{program}.calls")))
            .map_or(0, |t| t.lines().count())
    }

    fn bd_calls(&self) -> usize {
        self.calls("bd")
    }

    /// 偽の設計の道具の索引の読みの回数（引数に --summary も --emit-rulings も無い回）。
    fn folio_calls(&self) -> usize {
        self.calls("folio")
    }

    /// 偽の設計の道具の要約の読みの回数（引数に --summary が在る回）。
    fn summary_calls(&self) -> usize {
        self.calls("summary")
    }

    /// 偽の設計の道具の裁定の書き出しの読みの回数（引数に --emit-rulings が在る回）。
    fn rulings_calls(&self) -> usize {
        self.calls("rulings")
    }

    /// 印を動かす（issues.jsonl に 1 行を足した字を隣の file に書いてから置き替える）。
    fn touch(&self) {
        let beads = self.repo.join(".beads");
        let text = fs::read_to_string(beads.join("issues.jsonl")).expect("印の file");
        fs::write(beads.join("issues.jsonl.tmp"), format!("{text}{{}}\n")).expect("印の file");
        fs::rename(beads.join("issues.jsonl.tmp"), beads.join("issues.jsonl"))
            .expect("印の file を置き替える");
    }

    /// 偽の bd を落とす（true）か戻す（false）。
    fn fail(&self, on: bool) {
        let flag = self.root.join("fail");
        if on {
            fs::write(flag, "").expect("fail の印");
        } else {
            fs::remove_file(flag).expect("fail の印を消す");
        }
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_secs()
}

/// 応答（状態の code・本文）と前後の時刻（epoch 秒）。
struct Reply {
    status: u16,
    body: String,
    from: u64,
    to: u64,
}

/// 要求を 1 つ撃ち、接続が閉じるまで応答を読む。
fn request(addr: SocketAddr, raw: &str) -> Reply {
    let from = now();
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
    Reply {
        status,
        body: body.to_string(),
        from,
        to: now(),
    }
}

fn get(addr: SocketAddr, path: &str) -> Reply {
    request(addr, &format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n"))
}

/// GET を thread に並べて同時に撃ち、撃った順に応答を返す。
fn get_all(addr: SocketAddr, paths: &[String]) -> Vec<Reply> {
    thread::scope(|s| {
        let handles: Vec<_> = paths
            .iter()
            .map(|path| s.spawn(move || get(addr, path)))
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("要求の thread"))
            .collect()
    })
}

/// 口の電文を契約の型で読む（型は受ける側が決める）。
macro_rules! decode {
    ($body:expr) => {
        wire::decode(&$body).unwrap_or_else(|e| panic!("契約の型の形でない {e}: {}", $body))
    };
}

/// 口の値が、要求の前後のどれかの時刻で中核の関数を直に呼んだ値と一致する。
fn same_at_some_now<T: PartialEq + Debug>(what: &str, got: &T, r: &Reply, want: impl Fn(u64) -> T) {
    assert!(
        (r.from..=r.to).any(|t| &want(t) == got),
        "{what}: 中核の関数の値と違う（{}..={}）\n口: {got:?}\n中核: {:?}",
        r.from,
        r.to,
        want(r.from)
    );
}

/// 台帳の一覧の口の行の数（読めなければ None）。
fn ledger_rows(r: &Reply) -> Option<usize> {
    assert_eq!(r.status, 200, "{}", r.body);
    let list: LedgerList = decode!(r.body);
    match list.rows {
        Reading::Known(rows) => Some(rows.len()),
        Reading::Unknown => None,
    }
}

/// 数える歯は起動の直後に撃つ（変化の見張りの周期の読みは起動の読みから 5 秒の後）。
fn fresh(place: &Place) -> (SocketAddr, Instant) {
    let addr = place.serve();
    (addr, Instant::now())
}

/// 数えた区間が変化の見張りの周期の読みより前に終わった。
fn before_reread(started: Instant) {
    assert!(
        started.elapsed() < Duration::from_secs(4),
        "歯が遅く、変化の見張りの読みと重なりうる: {:?}",
        started.elapsed()
    );
}

#[test]
fn server_coalesce_seven_routes_share_one_bd() {
    let place = Place::new("seven");
    let (addr, started) = fresh(&place);
    let (ledger, events) = (read_fixture(LEDGER), read_fixture(EVENTS));
    let before = place.bd_calls();
    let paths: Vec<String> = [
        "/api/ledger",
        "/api/pipeline",
        "/api/metrics",
        "/api/next",
        "/api/graph",
        "/api/questions",
        &format!("/api/ledger/{BEAD}"),
    ]
    .map(str::to_string)
    .to_vec();
    let replies = get_all(addr, &paths);
    let shot = place.bd_calls() - before;
    before_reread(started);
    for (path, r) in paths.iter().zip(&replies) {
        assert_eq!(r.status, 200, "{path}: {}", r.body);
    }
    // 口は bd を撃たず、変化の見張りの最後の読みの字を返す（行 e-snap）。
    assert_eq!(shot, 0, "7 つの口の同時の要求に偽の bd が {shot} 回");

    assert_eq!(ledger_rows(&replies[0]), Some(8), "{}", replies[0].body);
    let board: PipelineBoard = decode!(replies[1].body);
    assert!(
        matches!(&board.cards, Reading::Known(c) if !c.is_empty()),
        "{board:?}"
    );
    same_at_some_now("pipeline", &board, &replies[1], |t| {
        tsuzuri_core::pipeline::board(&ledger, &events, t).board
    });
    let stats: Reading<LedgerStats> = decode!(replies[2].body);
    assert!(matches!(stats, Reading::Known(_)), "{stats:?}");
    same_at_some_now("metrics", &replies[2].body, &replies[2], |t| {
        wire::encode(&tsuzuri_core::ledger::stats(&ledger, t)).expect("指標の電文")
    });
    let step: NextStep = decode!(replies[3].body);
    same_at_some_now("next", &step, &replies[3], |t| {
        tsuzuri_core::next_step::next_step(&ledger, &events, t)
    });
    let doc: GraphDoc = decode!(replies[4].body);
    assert!(doc.unread.is_empty(), "{:?}", doc.unread);
    let questions = tsuzuri_core::question::list(&ledger);
    assert!(matches!(questions.cards, Reading::Known(_)));
    assert_eq!(
        replies[5].body,
        wire::encode(&questions).expect("問いの一覧の電文"),
        "問いの一覧"
    );
    let item: LedgerItem = decode!(replies[6].body);
    assert_eq!(item.row.id, BeadId::new(BEAD).expect("bead id"));
}

#[test]
fn server_coalesce_graph_routes_share_one_folio() {
    let place = Place::new("folio");
    let (addr, started) = fresh(&place);
    let (before, before_summary, before_rulings) = (
        place.folio_calls(),
        place.summary_calls(),
        place.rulings_calls(),
    );
    let paths = ["/api/graph", "/api/graph", "/api/graph/view"].map(str::to_string);
    let replies = get_all(addr, &paths);
    let shot = place.folio_calls() - before;
    let summary_shot = place.summary_calls() - before_summary;
    let rulings_shot = place.rulings_calls() - before_rulings;
    before_reread(started);
    let unread: Vec<Vec<GraphSource>> = replies
        .iter()
        .zip(&paths)
        .map(|(r, path)| {
            assert_eq!(r.status, 200, "{path}: {}", r.body);
            if path == "/api/graph/view" {
                let view: GraphView = decode!(r.body);
                view.unread
            } else {
                let doc: GraphDoc = decode!(r.body);
                doc.unread
            }
        })
        .collect();
    for (u, path) in unread.iter().zip(&paths) {
        assert!(!u.contains(&GraphSource::Design), "{path}: {u:?}");
    }
    assert!(
        (1..=2).contains(&shot),
        "3 つの口の同時の要求に偽の設計の道具の索引の読みが {shot} 回"
    );
    assert!(
        (1..=2).contains(&summary_shot),
        "3 つの口の同時の要求に偽の設計の道具の要約の読みが {summary_shot} 回"
    );
    assert!(
        (1..=2).contains(&rulings_shot),
        "3 つの口の同時の要求に偽の設計の道具の裁定の書き出しの読みが {rulings_shot} 回"
    );
}

#[test]
fn server_coalesce_no_carry_over() {
    let place = Place::new("carry");
    let (addr, started) = fresh(&place);
    let replies = get_all(
        addr,
        &["/api/ledger".to_string(), "/api/ledger".to_string()],
    );
    for r in &replies {
        assert_eq!(ledger_rows(r), Some(8), "{}", r.body);
    }
    // 印の file は動かさず、偽の bd の出力だけを替える（1 本に絞る）。
    let one = read_fixture(LEDGER);
    let first = &one[..one.find("\n  },\n").expect("1 本目の終わり") + "\n  }".len()];
    fs::write(place.root.join("ledger.json"), format!("{first}\n]\n")).expect("出力を替える");
    let before = place.bd_calls();
    let r = get(addr, "/api/ledger");
    // 口は見張りの最後の読みの字を返す（印が動かなければ替える前の字・行 e-snap）。
    assert_eq!(ledger_rows(&r), Some(8), "印の動かない後の出力: {}", r.body);
    assert_eq!(place.bd_calls() - before, 0, "口は bd を撃たない");
    // 印を動かせば見張りが読み、口は替えた後の字を返す。
    let before = place.bd_calls();
    place.touch();
    let rows = until_rows(addr, Some(1));
    let shot = place.bd_calls() - before;
    before_reread(started);
    assert_eq!(rows, Some(1), "替えた後の出力");
    assert_eq!(shot, 1, "見張りの読み");
}

/// /api/ledger の行の数が `want` になるまで（2.5 秒まで）50 ミリ秒ごとに撃ち、最後の行の数を返す。
fn until_rows(addr: SocketAddr, want: Option<usize>) -> Option<usize> {
    let until = Instant::now() + Duration::from_millis(2500);
    loop {
        let rows = ledger_rows(&get(addr, "/api/ledger"));
        if rows == want || Instant::now() >= until {
            return rows;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn server_coalesce_shared_failure_then_new_read() {
    let place = Place::new("fails");
    place.fail(true);
    let (addr, started) = fresh(&place);
    let before = place.bd_calls();
    let paths: Vec<String> = [
        "/api/ledger",
        "/api/ledger",
        "/api/metrics",
        &format!("/api/ledger/{BEAD}"),
    ]
    .map(str::to_string)
    .to_vec();
    let replies = get_all(addr, &paths);
    let shot = place.bd_calls() - before;
    assert_eq!(shot, 0, "{shot} 回");
    for r in &replies[..2] {
        assert_eq!(ledger_rows(r), None, "{}", r.body);
    }
    assert_eq!(replies[2].status, 200, "{}", replies[2].body);
    let stats: Reading<LedgerStats> = decode!(replies[2].body);
    assert_eq!(stats, Reading::Unknown);
    assert_eq!(
        (replies[3].status, replies[3].body.as_str()),
        (503, "ledger-unknown")
    );
    // 口は bd を撃たない（戻しても、見張りが読むまでは落ちた読みの結果・行 e-snap）。
    place.fail(false);
    let before = place.bd_calls();
    let r = get(addr, "/api/ledger");
    assert_eq!(place.bd_calls() - before, 0, "口は bd を撃たない");
    assert_eq!(ledger_rows(&r), None, "{}", r.body);
    // 印を動かせば見張りが読み、戻った bd の出力を返す。
    let before = place.bd_calls();
    place.touch();
    let rows = until_rows(addr, Some(8));
    let shot = place.bd_calls() - before;
    before_reread(started);
    assert_eq!(rows, Some(8), "戻った後の出力");
    assert_eq!(shot, 1, "見張りの読み");
}

#[test]
fn server_coalesce_ruling_reads_alone() {
    let place = Place::new("ruling");
    let (addr, started) = fresh(&place);
    let before = place.bd_calls();
    let body = wire::encode(&RulingRequest {
        question: BeadId::new("fx-no-such-question").expect("bead id"),
        seen_digest: "x".to_string(),
        verbatim: "はい".to_string(),
    })
    .expect("要求の電文");
    // 印を動かし、見張りの読みが走り始める（偽の bd が記録の行を足す）まで待つ。
    place.touch();
    let until = Instant::now() + Duration::from_secs(2);
    while place.bd_calls() == before && Instant::now() < until {
        thread::sleep(Duration::from_millis(5));
    }
    let (listed, ruled, during) = thread::scope(|s| {
        // 口は走っている見張りの読みの終わりを待つ（bd を撃たない）。
        let listed = s.spawn(|| get(addr, "/api/ledger"));
        let during = place.bd_calls() - before;
        let ruled = request(
            addr,
            &format!(
                "POST {} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                ruling::PATH,
                body.len()
            ),
        );
        (listed.join().expect("要求の thread"), ruled, during)
    });
    before_reread(started);
    assert_eq!(during, 1, "読みの途中");
    assert_eq!(
        place.bd_calls() - before,
        2,
        "裁定の受付は走っている読みを分け合わない"
    );
    assert_eq!(ledger_rows(&listed), Some(8), "{}", listed.body);
    assert_ne!(ruled.status, 503, "受付の読みが落ちた: {}", ruled.body);
    assert_eq!(ruled.status, 404, "問いが無い: {}", ruled.body);
}

#[test]
fn server_coalesce_wait_is_limit_plus_one_second() {
    assert_eq!(GRACE, Duration::from_secs(1));
    assert_eq!(BD_WAIT, BD_TIMEOUT + Duration::from_secs(1));
    assert_eq!(FOLIO_WAIT, FOLIO_TIMEOUT + Duration::from_secs(1));

    // 合流して待つ呼び出しは、渡した上限を越えたら読めないを返す（読みの終わりを待たない）。
    let shared: Coalesce<String> = Coalesce::new();
    let wait = Duration::from_millis(200);
    let (lead, joined, took) = thread::scope(|s| {
        let lead = s.spawn(|| {
            shared.share(wait, || {
                thread::sleep(Duration::from_millis(800));
                Some("読めた".to_string())
            })
        });
        thread::sleep(Duration::from_millis(100));
        let started = Instant::now();
        let joined = shared.clone().share(wait, || Some("別の読み".to_string()));
        let took = started.elapsed();
        (lead.join().expect("読みの thread"), joined, took)
    });
    assert_eq!(lead.as_deref(), Some("読めた"));
    assert_eq!(joined, None, "上限を越えた呼び出し");
    assert!(
        took >= wait && took < Duration::from_millis(700),
        "{took:?}"
    );
}

#[test]
fn server_coalesce_share_within_one_place() {
    let shared: Coalesce<String> = Coalesce::new();
    let other: Coalesce<String> = Coalesce::new();
    let long = Duration::from_secs(5);
    let (lead, joined, apart) = thread::scope(|s| {
        let lead = s.spawn(|| {
            shared.share(long, || {
                thread::sleep(Duration::from_millis(400));
                Some("1 本目".to_string())
            })
        });
        thread::sleep(Duration::from_millis(100));
        let joined = s.spawn(|| {
            shared
                .clone()
                .share(long, || Some("合流しない".to_string()))
        });
        // 別に new で作った場とは分け合わない。
        let apart = other.share(long, || Some("別の場".to_string()));
        (
            lead.join().expect("読みの thread"),
            joined.join().expect("合流の thread"),
            apart,
        )
    });
    assert_eq!(lead.as_deref(), Some("1 本目"));
    assert_eq!(joined.as_deref(), Some("1 本目"), "走っている読みの結果");
    assert_eq!(apart.as_deref(), Some("別の場"));
    // 読みが終われば結果は持ち回さない。
    assert_eq!(
        shared.share(long, || Some("2 本目".to_string())).as_deref(),
        Some("2 本目")
    );
    // 落ちた読みを分け合った呼び出しは読めない。次の呼び出しは新しい読みを始める。
    let (lead, joined) = thread::scope(|s| {
        let lead = s.spawn(|| {
            shared.share(long, || {
                thread::sleep(Duration::from_millis(300));
                None
            })
        });
        thread::sleep(Duration::from_millis(100));
        let joined = shared.share(long, || Some("合流しない".to_string()));
        (lead.join().expect("読みの thread"), joined)
    });
    assert_eq!((lead, joined), (None, None));
    assert_eq!(
        shared.share(long, || Some("3 本目".to_string())).as_deref(),
        Some("3 本目")
    );
}
