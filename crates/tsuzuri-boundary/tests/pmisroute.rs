//! 行 c-pipe-misfit の歯（境界）: 台帳の見張りの読みの後に器の doctor の台帳の形の行を撃ち（`form::Form`）、
//! 口 /api/pipeline は持った字から形の崩れの一覧を写す（歯の名は中核の tests/pmisfit.rs と同じ接頭辞 pmisfit_）。
//! 偽の bd は撃たれるたびに記録の file bd に 1 行を足して作業場の ledger.json の字を出し、偽の器は受けた argv を
//! 記録の file argv に 1 行ずつ足し、argv の頭が doctor なら作業場の sleep の字の秒だけ待ってから out-doctor の字を出す
//! （ほかと file の無い出力は rc 1）。作業場は CARGO_TARGET_TMPDIR の下に歯ごとに作る。

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::server::events::Hub;
use tsuzuri_boundary::server::form::{FORM_ARGS, FORM_TIMEOUT, Form, REPO_ARGS, form_lines};
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::{Misfit, MisfitBead, PipelineBoard, Reading};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::wire;
use tsuzuri_core::pipeline::board_with_doctor;

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

/// 待つ上限。
const WAIT: Duration = Duration::from_secs(5);

fn doctor() -> String {
    format!("{SEAT_LINE}\n{FORM_LINE}\n")
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
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

    /// 台帳の印（issues.jsonl）を動かし、見張りの読み（偽の bd の回が増える）を待つ。
    fn move_mark(&self) {
        let before = self.bd_calls();
        let n = self.moves.get() + 1;
        self.moves.set(n);
        let tmp = self.root.join("put/issues.jsonl");
        fs::write(&tmp, "1".repeat(n + 1)).expect("移す前の印");
        fs::rename(&tmp, self.repo.join(".beads/issues.jsonl")).expect("印を移す");
        until("見張りの読み", || self.bd_calls() > before);
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
fn board_until(addr: SocketAddr, what: &str, ok: impl Fn(&Reading<Vec<MisfitBead>>) -> bool) -> PipelineBoard {
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

/// (6) 許しの後の kick だけが器を撃ち、字が変われば知らせ、落ちた周は None、上限を越えれば止める。
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
    form.kick();
    thread::sleep(Duration::from_millis(300));
    assert!(place.argv().is_empty(), "許しの前に撃つ");
    assert_eq!(form.text(), None);

    // 許しの後は 1 度撃ち、台帳の形の行を持ち、知らせを 1 件。
    form.arm();
    assert!(form.armed());
    form.kick();
    until("最初の撃ち", || form.text().is_some());
    let line = format!("doctor --state-dir {state} --repo .");
    assert_eq!(place.argv(), [line.as_str()]);
    assert_eq!(form.text().as_deref(), Some(FORM_LINE));
    let frame = rx.recv_timeout(WAIT).expect("知らせ");
    assert!(frame.contains("event: board-changed\n"), "{frame}");
    assert_eq!(frames(&rx, Duration::from_millis(200)), 0, "撃ち 1 回に 2 件");

    // 同じ出力は知らせない。
    form.kick();
    until("2 度目の撃ち", || place.argv().len() == 2);
    assert_eq!(frames(&rx, Duration::from_millis(300)), 0, "同じ字で知らせる");
    assert_eq!(form.text().as_deref(), Some(FORM_LINE));

    // 落ちた周は None（持ち回さない）で知らせる。
    place.doctor_fails();
    form.kick();
    until("落ちた撃ち", || form.text().is_none());
    rx.recv_timeout(WAIT).expect("落ちた周の知らせ");

    // 置き直した周は字に戻る。
    place.doctor_returns(&doctor());
    form.kick();
    until("戻った撃ち", || form.text().is_some());
    assert_eq!(form.text().as_deref(), Some(FORM_LINE));
    rx.recv_timeout(WAIT).expect("戻った周の知らせ");

    // 上限（1 秒）を越えた周は None。
    place.sleep(Some("3"));
    let started = Instant::now();
    form.kick();
    until("上限を越えた撃ち", || form.text().is_none());
    assert!(
        started.elapsed() < Duration::from_millis(2500),
        "{:?}",
        started.elapsed()
    );
    place.sleep(None);
}

/// (6) 撃ちは同時に 1 本で、走っている間の kick は終わった後に 1 回だけ撃つ。
#[test]
fn pmisfit_form_one_at_a_time() {
    let place = Place::new("once");
    place.sleep(Some("0.4"));
    let form = Form::new(place.scribe2(), &place.state, &place.repo);
    form.arm();
    let started = Instant::now();
    form.kick();
    form.kick();
    form.kick();
    assert!(
        started.elapsed() < Duration::from_millis(200),
        "{:?}",
        started.elapsed()
    );
    until("2 度目の撃ち", || place.argv().len() == 2);
    until("2 度目の撃ちの字", || form.text().is_some());
    thread::sleep(Duration::from_millis(1000));
    assert_eq!(place.argv().len(), 2, "{:?}", place.argv());
    assert_eq!(form.text().as_deref(), Some(FORM_LINE));
}

/// (7) 口の最初の要求の後の見張りの読みの周だけ器を撃ち、口は持った字から一覧を写す。
#[test]
fn pmisfit_route_reads_kept_form() {
    let place = Place::new("route");
    let addr = place.serve(true, true);
    assert_eq!(place.bd_calls(), 1, "起動の読み");

    // 口の要求の前は、見張りの読みの周でも器を撃たない。
    place.move_mark();
    thread::sleep(Duration::from_millis(300));
    assert!(place.argv().is_empty(), "口の要求の前に撃つ");

    // 最初の要求は許すだけで撃たない（一覧は Unknown）。
    assert_eq!(pipeline(addr).misfits, Reading::Unknown);
    thread::sleep(Duration::from_millis(300));
    assert!(place.argv().is_empty(), "口が撃つ");

    // その後の見張りの読みの周に撃ち、口は持った字から写す。
    place.move_mark();
    until("見張りの周の撃ち", || place.argv().len() == 1);
    let state = place.state.display();
    assert_eq!(
        place.argv(),
        [format!("doctor --state-dir {state} --repo .")]
    );
    let got = board_until(addr, "Known の一覧", |m| matches!(m, Reading::Known(_)));
    assert_eq!(got.misfits, want());
    let doctor = doctor();
    assert_eq!(
        got,
        board_with_doctor(LEDGER, "", Some(&doctor), 0).board
    );

    // 口の要求は器を撃たない。
    for _ in 0..3 {
        assert_eq!(pipeline(addr).misfits, want());
    }
    assert_eq!(place.argv().len(), 1, "口の要求で撃つ");

    // 落ちた周は Unknown、置き直した周は戻る。
    place.doctor_fails();
    place.move_mark();
    board_until(addr, "落ちた周の Unknown", |m| *m == Reading::Unknown);
    place.doctor_returns(&doctor);
    place.move_mark();
    let back = board_until(addr, "戻った一覧", |m| *m == want());
    assert_eq!(back.misfits, want());
}

/// (8) 席の target か state dir が無い server は器を撃たず、台帳の形の行の無い字は Unknown。
#[test]
fn pmisfit_route_without_seat_or_line() {
    for (name, seat, state_dir) in [("no-seat", false, true), ("no-state", true, false)] {
        let place = Place::new(name);
        let addr = place.serve(seat, state_dir);
        assert_eq!(pipeline(addr).misfits, Reading::Unknown, "{name}");
        place.move_mark();
        thread::sleep(Duration::from_millis(300));
        assert!(place.argv().is_empty(), "{name} が撃つ: {:?}", place.argv());
        assert_eq!(pipeline(addr).misfits, Reading::Unknown, "{name}");
    }

    let place = Place::new("no-line");
    place.doctor_returns(&format!("{SEAT_LINE}\n"));
    let addr = place.serve(true, true);
    assert_eq!(pipeline(addr).misfits, Reading::Unknown);
    place.move_mark();
    until("見張りの周の撃ち", || place.argv().len() == 1);
    thread::sleep(Duration::from_millis(300));
    assert_eq!(pipeline(addr).misfits, Reading::Unknown);
}
