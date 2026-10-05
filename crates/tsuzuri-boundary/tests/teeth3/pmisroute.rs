//! 行 c-pipe-misfit と行 c-misfit-pair の歯（境界）: 台帳の見張りの読みの周の台帳の字で器の doctor の台帳の形の行を撃ち
//! （`form::Form`）、その字と台帳の形の行を組（`form::Kept`）で持ち、口 /api/pipeline は札を今の台帳の字から、
//! 形の崩れの一覧を持った組から写す。知らせの接続も撃ちを許す（歯の名は中核の tests/teeth2/pmisfit.rs と同じ接頭辞 pmisfit_）。
//! 偽の bd は撃たれるたびに記録の file bd に 1 行を足して作業場の ledger.json の字を出し、偽の器は受けた argv を
//! 記録の file argv に 1 行ずつ足し、argv の頭が doctor なら作業場の sleep の字の秒だけ待ってから out-doctor の字を出す
//! （ほかと file の無い出力は rc 1）。作業場は CARGO_TARGET_TMPDIR の下に歯ごとに作る。
//! 知らせの接続の読みは助けの `reread` で撃ち、Interrupted だけを同じ締め切りの中で撃ち直す（行 c-sse-eintr）。
#![cfg(test)]

use std::fs;
use std::io::{self, ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::common::script;
use tsuzuri_boundary::server::events::{Hub, Subscription};
use tsuzuri_boundary::server::form::{
    FORM_ARGS, FORM_TIMEOUT, Form, Kept, REPO_ARGS, form_lines, misfits,
};
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::{Misfit, MisfitBead, PipelineBoard, Reading, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::wire;
use tsuzuri_core::pipeline::{board, board_with_doctor};

/// 節の席の target。
const TARGET: &str = "proj-1:0.1";

/// 節の台帳（b.1 は印無し・b.2 は控えの印と設計の参照・b.3 は設計の参照）。
const LEDGER: &str = r#"[
{"id":"b.1","title":"t b.1","status":"open","issue_type":"task","created_at":"2026-09-27T07:39:00Z","updated_at":"2026-09-27T07:39:00Z"},
{"id":"b.2","title":"t b.2","status":"open","issue_type":"task","labels":["intake:memo"],"acceptance_criteria":"design = contracts/x.toml#b","created_at":"2026-09-27T07:39:00Z","updated_at":"2026-09-27T07:39:00Z"},
{"id":"b.3","title":"t b.3","status":"open","issue_type":"task","acceptance_criteria":"design = contracts/x.toml#a","created_at":"2026-09-27T07:39:00Z","updated_at":"2026-09-27T07:39:00Z"}
]
"#;

/// 節の席の行。
const SEAT_LINE: &str = "seat: role=orchestrator anchor=/srv/proj-1 target=proj-1:0.1 account=acct-1 model=opus pane=%1";

/// 節の台帳の形の行。
const FORM_LINE: &str = "ledger-form: open=3 memos=1 no-source=1:b.2 no-observation=1:b.2 no-candidate=1:b.2 no-promotion=1:b.2 shaped=3 both=1:b.2 neither=1:b.1 contracts=1 undiscovered=0 unlanded=0 drift=0 settled=0";

/// 走行を持たない event log の 1 行（札の列を Known にする）。
const EVENT: &str = r#"{"schema":1,"ts":"2026-09-27T11:10:00Z","kind":"SeatSpawned","host":"host-1","actor":"machine","detail":""}
"#;

/// 待つ上限。
const WAIT: Duration = Duration::from_secs(5);

fn doctor() -> String {
    format!("{SEAT_LINE}\n{FORM_LINE}\n")
}

/// 節の台帳に b.4（設計の参照を持つ task）を足した字。
fn grown() -> String {
    LEDGER.replace(
        "\n]\n",
        ",\n{\"id\":\"b.4\",\"title\":\"t b.4\",\"status\":\"open\",\"issue_type\":\"task\",\"acceptance_criteria\":\"design = contracts/x.toml#c\",\"created_at\":\"2026-09-27T07:39:00Z\",\"updated_at\":\"2026-09-27T07:39:00Z\"}\n]\n",
    )
}

/// `grown` の b.1 の題を t b.1 new に替えた字。
fn renamed() -> String {
    grown().replace("\"t b.1\"", "\"t b.1 new\"")
}

/// 札の列の id と段（札が Unknown なら空）。
fn stages(b: &PipelineBoard) -> Vec<(String, Stage)> {
    match &b.cards {
        Reading::Known(cards) => cards
            .iter()
            .map(|c| (c.contract.as_str().to_string(), c.stage))
            .collect(),
        Reading::Unknown => Vec::new(),
    }
}

/// 節の組（節の台帳の字と節の台帳の形の行）。
fn pair() -> Kept {
    Kept {
        ledger: LEDGER.to_string(),
        form: FORM_LINE.to_string(),
    }
}

/// 歯ごとの作業場（repo・面の file・state dir・記録の dir）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
    state: PathBuf,
    log: PathBuf,
    /// 印を動かした回（印の字の長さを替える）。
    moves: std::cell::Cell<usize>,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("pmisroute")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let place = Place {
            repo: root.join("repo"),
            files: root.join("files"),
            state: root.join("state"),
            log: root.join("log"),
            root,
            moves: std::cell::Cell::new(0),
        };
        for dir in [
            place.repo.join(".beads"),
            place.files.clone(),
            place.state.join("fleet"),
            place.log.clone(),
            place.root.join("put"),
        ] {
            fs::create_dir_all(dir).expect("置き場");
        }
        fs::write(place.repo.join(".beads/issues.jsonl"), "1\n").expect("印");
        fs::write(place.files.join("index.html"), "tz").expect("index.html");
        fs::write(place.state.join("fleet/events.jsonl"), "").expect("event log");
        fs::write(place.root.join("ledger.json"), LEDGER).expect("台帳の字");
        place.doctor_returns(&doctor());
        let (log, root) = (place.log.display(), place.root.display());
        script(
            &place.root.join("bd"),
            &format!("echo x >> '{log}/bd'\nexec cat '{root}/ledger.json'"),
        );
        script(
            &place.root.join("scribe2"),
            &format!(
                "printf '%s\\n' \"$*\" >> '{log}/argv'\n\
                 if [ \"$1\" != doctor ]; then exit 1; fi\n\
                 if [ -e '{root}/sleep' ]; then sleep \"$(cat '{root}/sleep')\"; fi\n\
                 exec cat '{root}/out-doctor'"
            ),
        );
        place
    }

    fn scribe2(&self) -> PathBuf {
        self.root.join("scribe2")
    }

    /// 偽の器の doctor の出力の字を置く。
    fn doctor_returns(&self, text: &str) {
        fs::write(self.root.join("out-doctor"), text).expect("doctor の字");
    }

    /// 偽の bd の出す台帳の字を置く。
    fn ledger_returns(&self, text: &str) {
        fs::write(self.root.join("ledger.json"), text).expect("台帳の字");
    }

    /// 偽の器の doctor の出力を落とす（file が無いので rc 1）。
    fn doctor_fails(&self) {
        fs::remove_file(self.root.join("out-doctor")).expect("doctor の字を消す");
    }

    /// 偽の器が doctor で待つ秒（None は待たない）。
    fn sleep(&self, secs: Option<&str>) {
        let path = self.root.join("sleep");
        match secs {
            Some(s) => fs::write(path, s).expect("待ちの字"),
            None => {
                let _ = fs::remove_file(path);
            }
        }
    }

    /// 偽の bd が撃たれた回。
    fn bd_calls(&self) -> usize {
        fs::read_to_string(self.log.join("bd"))
            .unwrap_or_default()
            .lines()
            .count()
    }

    /// 偽の器が受けた argv（1 回 1 行）。
    fn argv(&self) -> Vec<String> {
        fs::read_to_string(self.log.join("argv"))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    /// 台帳の印（issues.jsonl）を動かす（読みは待たない）。
    fn shift_mark(&self) {
        let n = self.moves.get() + 1;
        self.moves.set(n);
        let tmp = self.root.join("put/issues.jsonl");
        fs::write(&tmp, "1".repeat(n + 1)).expect("移す前の印");
        fs::rename(&tmp, self.repo.join(".beads/issues.jsonl")).expect("印を移す");
    }

    /// 台帳の印を動かし、口 /api/ledger を読んで読み（偽の bd の回が増える）を待つ
    /// （知らせの受け手が居ない間は見張りは読まず、口の読みが印の遅れを見て読む）。
    fn move_mark(&self, addr: SocketAddr) {
        let before = self.bd_calls();
        self.shift_mark();
        assert_eq!(get(addr, "/api/ledger").0, 200);
        until("口の読み", || self.bd_calls() > before);
    }

    fn config(&self, seat: bool, state_dir: bool) -> Config {
        Config {
            bd: self.root.join("bd").into(),
            folio: self.root.join("no-such-folio").into(),
            state_dir: state_dir.then(|| self.state.clone()),
            bdw: self.root.join("bdw").into(),
            seat: seat.then(|| TARGET.to_string()),
            scribe2: self.scribe2().into(),
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        }
    }

    fn serve(&self, seat: bool, state_dir: bool) -> SocketAddr {
        let server = Server::bind(&self.config(seat, state_dir)).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }
}

/// 条件が真になるまで 10 ミリ秒ごとに見る（上限 `WAIT`）。
fn until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = Instant::now() + WAIT;
    while !ok() {
        assert!(Instant::now() < deadline, "{what} を待ちきれない");
        thread::sleep(Duration::from_millis(10));
    }
}

/// 読みを 1 回撃ち、Interrupted なら `deadline` の前に限って撃ち直す（ほかの誤りと時間切れはそのまま返す）。
/// read の timeout を付けた socket の読みは、process が止められて再開した周に signal の手が無くても
/// Interrupted を返しうる（`Read::read` は撃ち直さない）。
fn reread(r: &mut impl Read, buf: &mut [u8], deadline: Instant) -> io::Result<usize> {
    loop {
        match r.read(buf) {
            Err(e) if e.kind() == ErrorKind::Interrupted && Instant::now() < deadline => {}
            got => return got,
        }
    }
}

/// GET を 1 つ撃ち、（状態の code・本文）を返す。
fn get(addr: SocketAddr, path: &str) -> (u16, String) {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    s.write_all(format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n").as_bytes())
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

/// 口 /api/pipeline の板。
fn pipeline(addr: SocketAddr) -> PipelineBoard {
    let (status, body) = get(addr, "/api/pipeline");
    assert_eq!(status, 200, "{body}");
    wire::decode(&body).expect("PipelineBoard の形")
}

/// 口の欄 misfits が `ok` になるまで口を撃ち直す。
fn board_until(
    addr: SocketAddr,
    what: &str,
    ok: impl Fn(&Reading<Vec<MisfitBead>>) -> bool,
) -> PipelineBoard {
    let deadline = Instant::now() + WAIT;
    loop {
        let b = pipeline(addr);
        if ok(&b.misfits) {
            return b;
        }
        assert!(Instant::now() < deadline, "{what} を待ちきれない: {b:?}");
        thread::sleep(Duration::from_millis(10));
    }
}

fn want() -> Reading<Vec<MisfitBead>> {
    Reading::Known(vec![
        MisfitBead {
            bead: BeadId::new("b.1").expect("id"),
            title: "t b.1".into(),
            misfit: Misfit::Neither,
        },
        MisfitBead {
            bead: BeadId::new("b.2").expect("id"),
            title: "t b.2".into(),
            misfit: Misfit::Both,
        },
    ])
}

/// 受け手に届いた board-changed の知らせの数（`wait` の間に届いた分）。
fn frames(rx: &std::sync::mpsc::Receiver<String>, wait: Duration) -> usize {
    thread::sleep(wait);
    let mut n = 0;
    while let Ok(frame) = rx.try_recv() {
        assert!(frame.contains("event: board-changed\n"), "{frame}");
        n += 1;
    }
    n
}

/// (6) 許しの後の kick だけが器を撃ち、台帳の字と台帳の形の行を組で持ち、組から写した一覧が変われば知らせ、
/// 台帳の読みか器が落ちた周と台帳の形の行の無い周は None、上限を越えれば止める。
#[test]
fn pmisfit_form_kick_rules() {
    let place = Place::new("kick");
    assert_eq!(form_lines(&doctor()), FORM_LINE);
    assert_eq!(form_lines(SEAT_LINE), "");
    assert_eq!(FORM_ARGS, ["doctor"]);
    assert_eq!(REPO_ARGS, ["--repo", "."]);
    assert_eq!(FORM_TIMEOUT, Duration::from_secs(30));
    let form = Form::with_timeout(
        place.scribe2(),
        &place.state,
        &place.repo,
        Duration::from_secs(1),
    );
    let state = place.state.display().to_string();
    assert_eq!(
        form.argv(),
        ["doctor", "--state-dir", state.as_str(), "--repo", "."]
    );
    let hub = Arc::new(Hub::default());
    let rx = hub.subscribe();
    form.notify(&hub);

    // 許しの前は撃たない。
    assert!(!form.armed());
    form.kick(Some(LEDGER.to_string()));
    thread::sleep(Duration::from_millis(300));
    assert!(place.argv().is_empty(), "許しの前に撃つ");
    assert_eq!(form.kept(), None);
    assert_eq!(misfits(None, 0), Reading::Unknown);
    armed_kicks(place, form, state, rx);
}

/// 許しの後の撃ちと組と知らせを、同じ組の周・台帳の字だけが変わる周・題が変わる周で見る。
fn armed_kicks(place: Place, form: Form, state: String, rx: Subscription) {
    // 許しの後は 1 度撃ち、組を持ち、知らせを 1 件。
    form.arm();
    assert!(form.armed());
    form.kick(Some(LEDGER.to_string()));
    until("最初の撃ち", || form.kept().is_some());
    let line = format!("doctor --state-dir {state} --repo .");
    assert_eq!(place.argv(), [line.as_str()]);
    assert_eq!(form.kept(), Some(pair()));
    assert_eq!(misfits(Some(&pair()), 0), want());
    let frame = rx.recv_timeout(WAIT).expect("知らせ");
    assert!(frame.contains("event: board-changed\n"), "{frame}");
    assert_eq!(
        frames(&rx, Duration::from_millis(200)),
        0,
        "撃ち 1 回に 2 件"
    );

    // 同じ組は知らせない。
    form.kick(Some(LEDGER.to_string()));
    until("2 度目の撃ち", || place.argv().len() == 2);
    assert_eq!(
        frames(&rx, Duration::from_millis(300)),
        0,
        "同じ組で知らせる"
    );
    assert_eq!(form.kept(), Some(pair()));

    // 台帳の字だけが変わって一覧が同じ周は、組の台帳の字を替えて知らせない。
    form.kick(Some(grown()));
    until("足した台帳の撃ち", || {
        form.kept().map(|k| k.ledger) == Some(grown())
    });
    let kept = form.kept().expect("足した台帳の組");
    assert_eq!(kept.form, FORM_LINE);
    assert_eq!(misfits(Some(&kept), 0), want());
    assert_eq!(
        frames(&rx, Duration::from_millis(300)),
        0,
        "同じ一覧で知らせる"
    );

    // 一覧の題が変わる周は知らせる。
    form.kick(Some(renamed()));
    until("題を替えた台帳の撃ち", || {
        form.kept().map(|k| k.ledger) == Some(renamed())
    });
    rx.recv_timeout(WAIT).expect("題を替えた周の知らせ");
    failed_rounds(place, form, rx);
}

/// 台帳の読みか器が落ちた周・形の行の無い周・上限を越えた周の組と知らせと、戻った周を見る。
fn failed_rounds(place: Place, form: Form, rx: Subscription) {
    // 台帳の読みが落ちた周は器を撃たず None で知らせる。
    let shots = place.argv().len();
    form.kick(None);
    until("台帳の落ちた周", || form.kept().is_none());
    thread::sleep(Duration::from_millis(300));
    assert_eq!(place.argv().len(), shots, "台帳の落ちた周に撃つ");
    rx.recv_timeout(WAIT).expect("台帳の落ちた周の知らせ");

    // 節の台帳の字に戻すと組に戻る。
    form.kick(Some(LEDGER.to_string()));
    until("戻った台帳の撃ち", || form.kept().is_some());
    assert_eq!(form.kept(), Some(pair()));
    rx.recv_timeout(WAIT).expect("戻った台帳の周の知らせ");

    // 器の落ちた周は None（持ち回さない）で知らせる。
    place.doctor_fails();
    form.kick(Some(LEDGER.to_string()));
    until("落ちた撃ち", || form.kept().is_none());
    rx.recv_timeout(WAIT).expect("落ちた周の知らせ");

    // 台帳の形の行の無い周は撃っても None のままで知らせない。
    place.doctor_returns(&format!("{SEAT_LINE}\n"));
    let shots = place.argv().len();
    form.kick(Some(LEDGER.to_string()));
    until("形の行の無い撃ち", || {
        place.argv().len() == shots + 1
    });
    assert_eq!(
        frames(&rx, Duration::from_millis(300)),
        0,
        "形の行の無い周で知らせる"
    );
    assert_eq!(form.kept(), None);

    // 置き直した周は組に戻る。
    place.doctor_returns(&doctor());
    form.kick(Some(LEDGER.to_string()));
    until("戻った撃ち", || form.kept().is_some());
    assert_eq!(form.kept(), Some(pair()));
    rx.recv_timeout(WAIT).expect("戻った周の知らせ");

    // 上限（1 秒）を越えた周は None。
    place.sleep(Some("3"));
    let started = Instant::now();
    form.kick(Some(LEDGER.to_string()));
    until("上限を越えた撃ち", || form.kept().is_none());
    assert!(
        started.elapsed() < Duration::from_millis(2500),
        "{:?}",
        started.elapsed()
    );
    place.sleep(None);
}

/// (6) 撃ちは同時に 1 本で、走っている間の kick は終わった後に最後の周の台帳の字で 1 回だけ撃つ。
#[test]
fn pmisfit_form_one_at_a_time() {
    let place = Place::new("once");
    place.sleep(Some("0.4"));
    let form = Form::new(place.scribe2(), &place.state, &place.repo);
    form.arm();
    let started = Instant::now();
    form.kick(Some("a".to_string()));
    form.kick(Some("b".to_string()));
    form.kick(Some("c".to_string()));
    assert!(
        started.elapsed() < Duration::from_millis(200),
        "{:?}",
        started.elapsed()
    );
    until("2 度目の撃ち", || place.argv().len() == 2);
    until("2 度目の撃ちの組", || {
        form.kept().map(|k| k.ledger).as_deref() == Some("c")
    });
    thread::sleep(Duration::from_millis(1000));
    assert_eq!(place.argv().len(), 2, "{:?}", place.argv());
    assert_eq!(
        form.kept(),
        Some(Kept {
            ledger: "c".to_string(),
            form: FORM_LINE.to_string(),
        })
    );
}

/// (7) 口の最初の要求の後の見張りの読みの周だけ器を撃ち、口は札を今の台帳の字から、一覧を持った組から写す。
#[test]
fn pmisfit_route_reads_kept_form() {
    let place = Place::new("route");
    fs::write(place.state.join("fleet/events.jsonl"), EVENT).expect("event log");
    let addr = place.serve(true, true);
    assert_eq!(place.bd_calls(), 1, "起動の読み");

    // 口の要求の前は、見張りの読みの周でも器を撃たない。
    place.move_mark(addr);
    thread::sleep(Duration::from_millis(300));
    assert!(place.argv().is_empty(), "口の要求の前に撃つ");

    // 最初の要求は許すだけで撃たない（一覧は Unknown）。
    assert_eq!(pipeline(addr).misfits, Reading::Unknown);
    thread::sleep(Duration::from_millis(300));
    assert!(place.argv().is_empty(), "口が撃つ");

    // その後の見張りの読みの周に撃ち、口は持った字から写す。
    place.move_mark(addr);
    until("口の読みの周の撃ち", || place.argv().len() == 1);
    let state = place.state.display();
    assert_eq!(
        place.argv(),
        [format!("doctor --state-dir {state} --repo .")]
    );
    let got = board_until(addr, "Known の一覧", |m| matches!(m, Reading::Known(_)));
    assert_eq!(got.misfits, want());
    assert_eq!(stages(&got), [("b.3".to_string(), Stage::Queued)]);
    let doctor = doctor();
    assert_eq!(
        got,
        board_with_doctor(LEDGER, EVENT, Some(&doctor), 0).board
    );
    route_rounds(place, addr, doctor);
}

/// 撃った後の口の要求と、撃っている間・撃ちを終えた後・落ちた周に口が写す札と一覧を見る。
fn route_rounds(place: Place, addr: SocketAddr, doctor: String) {
    // 口の要求は器を撃たない。
    for _ in 0..3 {
        assert_eq!(pipeline(addr).misfits, want());
    }
    assert_eq!(place.argv().len(), 1, "口の要求で撃つ");

    // 器を撃っている間の口は、札を今の台帳の字から組み、一覧は前の組のまま。
    place.sleep(Some("2"));
    let now = renamed();
    place.ledger_returns(&now);
    place.move_mark(addr);
    until("今の台帳の字", || {
        get(addr, "/api/ledger").1.contains("t b.1 new")
    });
    let during = pipeline(addr);
    assert_eq!(
        stages(&during),
        [
            ("b.3".to_string(), Stage::Queued),
            ("b.4".to_string(), Stage::Queued)
        ]
    );
    assert_eq!(during.cards, board(&now, EVENT, 0).board.cards);
    assert_eq!(during.misfits, want());

    // 撃ちを終えた後の口は、その周の台帳の字と台帳の形の行の組から写す。
    let after = board_with_doctor(&now, EVENT, Some(&doctor), 0).board;
    let done = board_until(addr, "撃ちを終えた一覧", |m| *m == after.misfits);
    assert_eq!(done, after);
    place.sleep(None);

    // 落ちた周は Unknown、置き直した周は戻る。
    place.doctor_fails();
    place.move_mark(addr);
    board_until(addr, "落ちた周の Unknown", |m| *m == Reading::Unknown);
    place.doctor_returns(&doctor);
    place.move_mark(addr);
    let back = board_until(addr, "戻った一覧", |m| *m == after.misfits);
    assert_eq!(back.misfits, after.misfits);
}

/// (9) 知らせの接続は撃ちを許し、受け手の付いた周の見張りの読みが器を撃ち、一覧が変われば接続に知らせる。
#[test]
fn pmisfit_sse_arms_and_tells() {
    let place = Place::new("sse");
    let addr = place.serve(true, true);
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(WAIT)).expect("timeout");
    s.write_all(b"GET /api/surface/events HTTP/1.1\r\nHost: x\r\n\r\n")
        .expect("要求を書く");
    let mut seen = String::new();
    let mut read_until = |what: &str, want: &str| {
        let deadline = Instant::now() + WAIT;
        let mut buf = [0u8; 4096];
        while !seen.contains(want) {
            assert!(Instant::now() < deadline, "{what} を待ちきれない: {seen}");
            let n = reread(&mut s, &mut buf, deadline).expect("接続を読む");
            assert!(n > 0, "接続が閉じた: {seen}");
            seen.push_str(&String::from_utf8_lossy(&buf[..n]));
        }
    };
    read_until("頭の retry の行", "retry: 1000\n\n");

    // 口を撃たず印も動かさずに、受け手の付いた周の読みが器を撃つ。
    read_until("board-changed の知らせ", "event: board-changed\n");
    let state = place.state.display();
    assert_eq!(
        place.argv(),
        [format!("doctor --state-dir {state} --repo .")]
    );
    assert!(place.bd_calls() >= 2, "受け手の付いた周の読み");
    assert_eq!(pipeline(addr).misfits, want());
}

/// (8) 席の target か state dir が無い server は器を撃たず、台帳の形の行の無い字は Unknown。
#[test]
fn pmisfit_route_without_seat_or_line() {
    for (name, seat, state_dir) in [("no-seat", false, true), ("no-state", true, false)] {
        let place = Place::new(name);
        let addr = place.serve(seat, state_dir);
        assert_eq!(pipeline(addr).misfits, Reading::Unknown, "{name}");
        place.move_mark(addr);
        thread::sleep(Duration::from_millis(300));
        assert!(place.argv().is_empty(), "{name} が撃つ: {:?}", place.argv());
        assert_eq!(pipeline(addr).misfits, Reading::Unknown, "{name}");
    }

    let place = Place::new("no-line");
    place.doctor_returns(&format!("{SEAT_LINE}\n"));
    let addr = place.serve(true, true);
    assert_eq!(pipeline(addr).misfits, Reading::Unknown);
    place.move_mark(addr);
    until("口の読みの周の撃ち", || place.argv().len() == 1);
    thread::sleep(Duration::from_millis(300));
    assert_eq!(pipeline(addr).misfits, Reading::Unknown);
}

/// (10) 知らせの口を開いたままの server では、口を読まずに台帳の印を動かすと見張りの読みの周に器が撃たれ、
/// 一覧は Known・器の撃ちを落とした周は Unknown・置き直した周は戻る。
#[test]
fn pmisfit_watch_kicks_form_while_listening() {
    let place = Place::new("listen");
    let addr = place.serve(true, true);
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(WAIT)).expect("timeout");
    s.write_all(b"GET /api/surface/events HTTP/1.1\r\nHost: x\r\n\r\n")
        .expect("要求を書く");
    // 受け手の付いた周の読みが 1 回撃つ。
    until("受け手の付いた周の撃ち", || place.argv().len() == 1);
    let done = board_until(addr, "Known の一覧", |m| matches!(m, Reading::Known(_)));
    assert_eq!(done.misfits, want());
    let state = place.state.display();
    let line = format!("doctor --state-dir {state} --repo .");
    assert_eq!(place.argv(), [line.as_str()]);
    // 口を読まずに印を動かす（見張りの読みが撃つ）。
    place.shift_mark();
    until("見張りの周の撃ち", || place.argv().len() == 2);
    assert_eq!(pipeline(addr).misfits, want());
    // 落ちた周は Unknown、置き直した周は戻る。
    place.doctor_fails();
    place.shift_mark();
    board_until(addr, "落ちた周の Unknown", |m| *m == Reading::Unknown);
    place.doctor_returns(&doctor());
    place.shift_mark();
    board_until(addr, "戻った一覧", |m| *m == want());
    drop(s);
}

/// 撃たれるたびに頭の 1 つを返す偽の読み手（尽きたら 0 byte・撃たれた数を数える）。
struct Steps {
    steps: Vec<Result<&'static [u8], ErrorKind>>,
    reads: usize,
}

impl Steps {
    fn new(steps: Vec<Result<&'static [u8], ErrorKind>>) -> Steps {
        Steps { steps, reads: 0 }
    }
}

impl Read for Steps {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reads += 1;
        if self.steps.is_empty() {
            return Ok(0);
        }
        match self.steps.remove(0) {
            Ok(bytes) => {
                buf[..bytes.len()].copy_from_slice(bytes);
                Ok(bytes.len())
            }
            Err(kind) => Err(kind.into()),
        }
    }
}

/// 行 c-sse-eintr: 最初の 1 回だけ Interrupted を返す読み手は、助けの読みが撃ち直して字を読む。
#[test]
fn pmisfit_reread_after_interrupt() {
    let mut r = Steps::new(vec![Err(ErrorKind::Interrupted), Ok(b"retry: 1000\n\n")]);
    let mut buf = [0u8; 64];
    let n = reread(&mut r, &mut buf, Instant::now() + WAIT).expect("撃ち直して読む");
    assert_eq!(&buf[..n], b"retry: 1000\n\n");
    assert_eq!(r.reads, 2);
}

/// 行 c-sse-eintr: Interrupted でない誤り（時間切れを含む）と締め切りの後の Interrupted は、撃ち直さずに返す。
#[test]
fn pmisfit_reread_other_errors_fall() {
    let mut buf = [0u8; 64];
    for kind in [
        ErrorKind::WouldBlock,
        ErrorKind::TimedOut,
        ErrorKind::ConnectionReset,
    ] {
        let mut r = Steps::new(vec![Err(kind), Ok(b"x")]);
        let e = reread(&mut r, &mut buf, Instant::now() + WAIT).expect_err("落ちる");
        assert_eq!(e.kind(), kind);
        assert_eq!(r.reads, 1, "{kind:?} を撃ち直す");
    }
    let mut r = Steps::new(vec![Err(ErrorKind::Interrupted), Ok(b"x")]);
    let e = reread(&mut r, &mut buf, Instant::now()).expect_err("締め切りの後は落ちる");
    assert_eq!(e.kind(), ErrorKind::Interrupted);
    assert_eq!(r.reads, 1, "締め切りの後に撃ち直す");
}
