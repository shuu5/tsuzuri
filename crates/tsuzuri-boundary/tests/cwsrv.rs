//! board の server の相談の窓の 3 つの口の歯（接頭辞 cwsrv_・設計ノート surface-wave27b 行 cs-server）。
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）に、git init した repo（鍵 tsuzuri.draftsdir）と state dir と起草の置き場と、
//! 台帳の file を返す偽の bd と、受けた argv を記録する偽の bdw と偽の器の CLI を置き、server を同じ process の thread で
//! 127.0.0.1 の空き port に立てて素の TCP で撃つ。窓の作業場（控えと所見の file の名）は歯が直に書く。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use tsuzuri_boundary::consult::list::board;
use tsuzuri_boundary::consult::{Ctx, minute_now};
use tsuzuri_boundary::server::consult::next_request;
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::consult::{
    ConsultBoard, ConsultRequest, ConsultUnreceived, FindingId, Form, PATH, REQUEST_PATH,
    RequestId, Starter, UNRECEIVED_PATH, WindowFile, WindowId,
};
use tsuzuri_contract::ledger::{BeadId, LedgerWrite};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::lines::{Line, render, scan};

const LEDGER: &str = r#"[
{"id":"fx-s","title":"根","status":"open","issue_type":"epic","updated_at":"2026-10-03T00:00:00Z","notes":"NOTES"},
{"id":"fx-s.1","title":"控え","status":"open","issue_type":"task","labels":["intake:memo"],"parent":"fx-s","updated_at":"2026-10-03T00:00:00Z"},
{"id":"fx-s.2","title":"問い","status":"open","issue_type":"task","labels":["intake:question"],"parent":"fx-s.1","updated_at":"2026-10-03T00:00:00Z"}
]"#;

/// 頼みの行（受けの無い頼み）。
const RQ: &str = "相談の頼み = rq-20261003T1412Z-1・題 = fx-s.2・形 = 問う・model = opus・起こし手 = 持ち主の button・時刻 = 20261003T1412Z";

/// 所見 cw1-2 の受けの行。
const GOT_F: &str = "相談の受け = cw1-2・経路 = 見張り・時刻 = 20261003T1530Z";

/// 頼みの受けの行。
const GOT_RQ: &str = "相談の受け = rq-20261003T1412Z-1・経路 = 一覧・時刻 = 20261003T1531Z";

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .expect("git");
    assert!(ok.success(), "git {args:?}");
}

fn w(n: u32) -> WindowId {
    WindowId::new(n).expect("窓の id")
}

/// 歯ごとの置き場。
struct Fx {
    root: PathBuf,
    repo: PathBuf,
    state: PathBuf,
    drafts: PathBuf,
}

impl Fx {
    /// 鍵 tsuzuri.draftsdir は置かない（歯が置く）。
    fn new(name: &str) -> Fx {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cwsrv")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, state, drafts) = (root.join("repo"), root.join("state"), root.join("drafts"));
        for d in [
            &repo,
            &state,
            &drafts,
            &root.join("files"),
            &root.join("bin"),
        ] {
            fs::create_dir_all(d).expect("置き場");
        }
        fs::write(root.join("files/index.html"), "tz").expect("index.html");
        git(&repo, &["init", "-q"]);
        let r = root.display().to_string();
        let record = |log: &str| {
            format!(
                "for a in \"$@\"; do printf '%s\\037' \"$a\"; done >> '{r}/{log}'\necho >> '{r}/{log}'"
            )
        };
        script(&root.join("bin/bd"), &format!("exec cat '{r}/ledger.json'"));
        script(&root.join("bin/bdw"), &record("bdw.log"));
        script(&root.join("bin/scribe2"), &record("scribe2.log"));
        let fx = Fx {
            root,
            repo,
            state,
            drafts,
        };
        fx.notes(LEDGER, &[]);
        fx
    }

    /// 根の notes に `lines` を置いた台帳の字を書く。
    fn notes(&self, ledger: &str, lines: &[&str]) {
        let text = ledger.replace("NOTES", &lines.join("\\n"));
        fs::write(self.root.join("ledger.json"), text).expect("台帳の file");
    }

    /// 窓の作業場（`dir` は dir の名）と所見の file の名（中身は空の object）。
    fn window(&self, dir: &str, n: u32, ks: &[u32]) {
        let ws = self.drafts.join(dir);
        for d in [".consult", "findings"] {
            fs::create_dir_all(ws.join(d)).expect("作業場");
        }
        let file = WindowFile {
            id: w(n),
            form: Form::Talk,
            topic: Some("fx-s.2".to_string()),
            model: "fable".into(),
            effort: "xhigh".into(),
            starter: Starter::Seat,
            uttered: None,
            request: None,
            made: "20261003T1400Z".into(),
        };
        fs::write(
            ws.join(".consult/window.json"),
            wire::encode(&file).expect("控えの電文"),
        )
        .expect("控え");
        for k in ks {
            fs::write(ws.join(format!("findings/cw{n}-{k}.json")), "{}").expect("所見の file");
        }
    }

    /// state dir を置くか・読むだけかを選んで server を立て、口の住所を返す（席の target と器の CLI は偽）。
    fn serve(&self, with_state: bool, read_only: bool) -> SocketAddr {
        let config = Config {
            bd: self.root.join("bin/bd").into(),
            bdw: self.root.join("bin/bdw").into(),
            scribe2: self.root.join("bin/scribe2").into(),
            seat: Some("fx-seat".to_string()),
            state_dir: with_state.then(|| self.state.clone()),
            read_only,
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.root.join("files"),
            )
        };
        let server = Server::bind(&config).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }

    fn ctx(&self) -> Ctx {
        Ctx {
            repo: self.repo.clone(),
            state: self.state.clone(),
            drafts: self.drafts.clone(),
            bd: self.root.join("bin/bd").into(),
            bdw: self.root.join("bin/bdw").into(),
        }
    }

    /// 偽の program の記録の行（無ければ空）。
    fn log(&self, name: &str) -> Vec<Vec<String>> {
        fs::read_to_string(self.root.join(name))
            .unwrap_or_default()
            .lines()
            .map(|l| {
                l.split('\u{1f}')
                    .filter(|a| !a.is_empty())
                    .map(String::from)
                    .collect()
            })
            .collect()
    }
}

/// 要求を 1 つ撃ち、状態の code と本文を返す（`origin` は頭 Origin の字）。
fn call(
    addr: SocketAddr,
    method: &str,
    path: &str,
    origin: Option<&str>,
    body: &str,
) -> (u16, String) {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    let origin = origin
        .map(|o| format!("Origin: {o}\r\n"))
        .unwrap_or_default();
    s.write_all(
        format!(
            "{method} {path} HTTP/1.1\r\nHost: {addr}\r\n{origin}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
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

fn get(addr: SocketAddr, path: &str) -> String {
    let (status, body) = call(addr, "GET", path, None, "");
    assert_eq!(status, 200, "{path}: {body}");
    body
}

fn ask(topic: Option<&str>, model: &str) -> String {
    wire::encode(&ConsultRequest {
        topic: topic.map(String::from),
        form: Form::Ask,
        model: model.to_string(),
    })
    .expect("頼みの電文")
}

fn known_board(b: &ConsultBoard) -> bool {
    [
        matches!(b.windows, Reading::Known(_)),
        matches!(b.findings, Reading::Known(_)),
        matches!(b.requests, Reading::Known(_)),
        matches!(b.quota, Reading::Known(_)),
    ] == [true; 4]
}

fn unknown_board(b: &ConsultBoard) -> bool {
    [
        b.windows == Reading::Unknown,
        b.findings == Reading::Unknown,
        b.requests == Reading::Unknown,
        b.quota == Reading::Unknown,
    ] == [true; 4]
}

/// (1) 口 /api/consult の電文は、作業場の file と台帳の相談の行から tz consult list の組み（`list::board`）が作る
/// 電文と同じ字で、窓は退いた窓を含む 2 つ・処分の無い所見は cw1-1・受けの無い頼みは 1 つ。
#[test]
fn cwsrv_board_matches_list() {
    let fx = Fx::new("board");
    git(
        &fx.repo,
        &[
            "config",
            "tsuzuri.draftsdir",
            &fx.drafts.display().to_string(),
        ],
    );
    fx.window("consult-cw1", 1, &[1]);
    fx.window("retired-consult-cw2", 2, &[]);
    fx.notes(LEDGER, &[RQ]);
    let body = get(fx.serve(true, false), PATH);
    let lines = scan(RQ);
    let want = board(&fx.ctx(), &lines, &minute_now());
    assert_eq!(body, wire::encode(&want).expect("電文"));
    let got: ConsultBoard = wire::decode(&body).expect("一覧の電文");
    assert!(known_board(&got), "{body}");
    let Reading::Known(wins) = &got.windows else {
        panic!("{body}")
    };
    assert_eq!(wins.iter().map(|r| r.id).collect::<Vec<_>>(), [w(1), w(2)]);
    let Reading::Known(open) = &got.findings else {
        panic!("{body}")
    };
    assert_eq!(
        open.iter().map(|r| r.id.to_string()).collect::<Vec<_>>(),
        ["cw1-1"]
    );
    let Reading::Known(rqs) = &got.requests else {
        panic!("{body}")
    };
    assert_eq!(
        rqs.iter().map(|r| r.id.to_string()).collect::<Vec<_>>(),
        ["rq-20261003T1412Z-1"]
    );
}

/// (2) state dir の無い server・鍵 tsuzuri.draftsdir の無い repo・dir でない鍵の値・読めない台帳は、どれも
/// 一覧の 4 つの段を Unknown にし、未受けの口を Unknown にする（ほかの材料は揃えた形・揃えた対照は 4 段とも Known）。
#[test]
fn cwsrv_unknown_without_place() {
    let fx = Fx::new("unknown");
    let drafts = fx.drafts.display().to_string();
    let both = |addr: SocketAddr| {
        let b: ConsultBoard = wire::decode(&get(addr, PATH)).expect("一覧の電文");
        let u: Reading<ConsultUnreceived> =
            wire::decode(&get(addr, UNRECEIVED_PATH)).expect("未受けの電文");
        (b, u)
    };
    let (b, u) = both(fx.serve(true, false));
    assert!(unknown_board(&b) && u == Reading::Unknown, "鍵が無い");
    git(
        &fx.repo,
        &["config", "tsuzuri.draftsdir", &format!("{drafts}/none")],
    );
    let (b, u) = both(fx.serve(true, false));
    assert!(unknown_board(&b) && u == Reading::Unknown, "dir でない");
    git(&fx.repo, &["config", "tsuzuri.draftsdir", &drafts]);
    let (b, u) = both(fx.serve(false, false));
    assert!(
        unknown_board(&b) && u == Reading::Unknown,
        "state dir が無い"
    );
    let (b, u) = both(fx.serve(true, false));
    assert!(
        known_board(&b) && matches!(u, Reading::Known(_)),
        "揃えた対照"
    );
    fs::write(fx.root.join("ledger.json"), "not json").expect("読めない台帳");
    let (b, u) = both(fx.serve(true, false));
    assert!(unknown_board(&b) && u == Reading::Unknown, "読めない台帳");
}

/// (3) 口 /api/consult/unreceived の電文は、退いていない作業場の所見のうち受けの行の無い id と、受けの行の無い頼みの id
/// （退いた作業場の所見と受けた所見と受けた頼みは載らない）。
#[test]
fn cwsrv_unreceived_lists_findings_and_requests() {
    let fx = Fx::new("unreceived");
    git(
        &fx.repo,
        &[
            "config",
            "tsuzuri.draftsdir",
            &fx.drafts.display().to_string(),
        ],
    );
    fx.window("consult-cw1", 1, &[1, 2]);
    fx.window("retired-consult-cw2", 2, &[1]);
    fx.notes(LEDGER, &[RQ, GOT_F]);
    let addr = fx.serve(true, false);
    let got: Reading<ConsultUnreceived> = wire::decode(&get(addr, UNRECEIVED_PATH)).expect("電文");
    let f1 = FindingId::parse("cw1-1").expect("所見の id");
    let rq = RequestId::parse("rq-20261003T1412Z-1").expect("頼みの id");
    assert_eq!(
        got,
        Reading::Known(ConsultUnreceived {
            findings: vec![f1],
            requests: vec![rq],
        })
    );
    fx.notes(LEDGER, &[RQ, GOT_F, GOT_RQ]);
    let addr = fx.serve(true, false);
    let got: Reading<ConsultUnreceived> = wire::decode(&get(addr, UNRECEIVED_PATH)).expect("電文");
    assert_eq!(
        got,
        Reading::Known(ConsultUnreceived {
            findings: vec![f1],
            requests: vec![],
        })
    );
}

/// (4) 次の頼みの id は、同じ分の字の頼みの行の数の最大に 1 を足した数（ほかの分の字の行は数えず、無ければ 1）で、
/// 分の字が形でなければ None。
#[test]
fn cwsrv_next_request_counts_same_minute() {
    let m = "20261003T1412Z";
    let rq = |minute: &str, n: u32| Line::Request {
        id: RequestId::new(minute, n).expect("頼みの id"),
        topic: None,
        form: Form::Talk,
        model: "fable".into(),
        at: minute.into(),
    };
    let lines = [rq(m, 1), rq(m, 3), rq("20261003T1413Z", 7)];
    let next = |minute: &str| next_request(&lines, minute).map(|id| id.to_string());
    assert_eq!(next(m).as_deref(), Some("rq-20261003T1412Z-4"));
    assert_eq!(
        next("20261003T1413Z").as_deref(),
        Some("rq-20261003T1413Z-8")
    );
    assert_eq!(
        next("20261003T1414Z").as_deref(),
        Some("rq-20261003T1414Z-1")
    );
    assert_eq!(next_request(&[], m).map(|id| id.n()), Some(1));
    assert_eq!(next("20261003T1412z"), None);
}

/// (5) 口 POST /api/consult/request は、題の bead から上った memo（題が無ければ根）に、頼みの id と題と形と model と
/// 起こし手 持ち主の button と時刻の相談の頼みの行を bdw の AppendNotes で 1 本だけ足し、応答は頼みの id の電文で、
/// 器の CLI（裁定の配達）を撃たない。
#[test]
fn cwsrv_request_appends_one_line() {
    let fx = Fx::new("request");
    let addr = fx.serve(true, false);
    let origin = format!("http://{addr}");
    for (topic, home) in [(Some("fx-s.2"), "fx-s.1"), (None, "fx-s")] {
        let _ = fs::remove_file(fx.root.join("bdw.log"));
        let (status, body) = call(
            addr,
            "POST",
            REQUEST_PATH,
            Some(&origin),
            &ask(topic, "opus"),
        );
        assert_eq!(status, 200, "{body}");
        let id: RequestId = wire::decode(&body).expect("頼みの id の電文");
        let line = Line::Request {
            id: id.clone(),
            topic: topic.map(String::from),
            form: Form::Ask,
            model: "opus".into(),
            at: id.minute().to_string(),
        };
        let text = render(&line).expect("行の字");
        assert!(text.contains("・起こし手 = 持ち主の button・"), "{text}");
        let want = LedgerWrite::AppendNotes {
            id: BeadId::new(home).expect("bead の id"),
            line: text,
        };
        assert_eq!(fx.log("bdw.log"), [want.argv()]);
    }
    assert!(fx.log("scribe2.log").is_empty(), "配達を撃たない");
}

/// (6) 頼みの口は、読むだけの server（403）・別の Origin（403）・model が空の頼み（400 bad-line）・読めない台帳
/// （503 ledger-unknown）・根の無い台帳（503 no-root）を断り、どれも bdw を撃たない（ほかの欄は揃えた形）。
#[test]
fn cwsrv_request_refusals() {
    let fx = Fx::new("refuse");
    let post = |addr: SocketAddr, origin: &str, body: &str| {
        call(addr, "POST", REQUEST_PATH, Some(origin), body)
    };
    let ro = fx.serve(true, true);
    assert_eq!(
        post(ro, &format!("http://{ro}"), &ask(None, "opus")).0,
        403,
        "読むだけ"
    );
    let addr = fx.serve(true, false);
    let me = format!("http://{addr}");
    assert_eq!(
        post(addr, "http://198.51.100.7:8120", &ask(None, "opus")).0,
        403,
        "別の Origin"
    );
    assert_eq!(
        post(addr, &me, &ask(None, "")),
        (400, "bad-line".to_string())
    );
    fx.notes(&LEDGER.replace("\"epic\"", "\"task\""), &[]);
    assert_eq!(
        post(addr, &me, &ask(None, "opus")),
        (503, "no-root".to_string())
    );
    fs::write(fx.root.join("ledger.json"), "not json").expect("読めない台帳");
    assert_eq!(
        post(addr, &me, &ask(None, "opus")),
        (503, "ledger-unknown".to_string())
    );
    assert!(fx.log("bdw.log").is_empty(), "bdw を撃たない");
}
