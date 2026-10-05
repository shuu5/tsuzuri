//! 走行の時間軸の口が便の dir の判定の file を読む歯（接頭辞 bvvroute_・設計ノート surface-wave26a 行 c-run-verdict の完了の条件）。
//! server は同じ process の thread で 127.0.0.1 の空き port に立て、GET を素の TCP で撃つ。
//! state dir に小さな event log と便の dir（pipe/<run id>/ の verdict.json と review.json）を歯の中の字で置く。
#![cfg(test)]

use std::fs;
use std::net::SocketAddr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;

use crate::common::get;
use tsuzuri_boundary::server::runs::{GATE_FILE, PIPE_DIR, REVIEW_FILE, Runs};
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::runs::{PATH, RunLine, RunsDoc};
use tsuzuri_contract::wire;
use tsuzuri_core::pipeline::{gate_of, review_of};

const RUN_A: &str = "fx-r.1-20261001T000000Z";
const RUN_B: &str = "fx-r.1-20261001T010000Z";

/// bead fx-r.1 の 2 つの走行。
const LOG: &str = concat!(
    r#"{"ts":"2026-10-01T00:00:00Z","kind":"RunCreated","run":"fx-r.1-20261001T000000Z","bead":"fx-r.1","stage":"Intake"}"#,
    "\n",
    r#"{"ts":"2026-10-01T00:01:00Z","kind":"RunStage","run":"fx-r.1-20261001T000000Z","bead":"fx-r.1","stage":"Reviewed","detail":"verdict:FAIL kind:other"}"#,
    "\n",
    r#"{"ts":"2026-10-01T01:00:00Z","kind":"RunCreated","run":"fx-r.1-20261001T010000Z","bead":"fx-r.1","stage":"Intake"}"#,
    "\n",
    r#"{"ts":"2026-10-01T01:20:00Z","kind":"RunStage","run":"fx-r.1-20261001T010000Z","bead":"fx-r.1","stage":"Gated","detail":"verdict:PASS"}"#,
    "\n",
);

const GATE_PASS: &str = r#"{"schema":1,"run":"fx-r.1-20261001T010000Z","verdict":"PASS","evidence":"ok","verify_red":0,"diff_bytes":512,"tree":"0000000000000000000000000000000000000000","findings":"contract-fit:0,teeth-nonvacuous:0,constitution:0,delete:0,stdlib:0,native:0,yagni:0,shrink:1","population":"files:2,lines:40","ts":"2026-10-01T01:19:00Z"}"#;

const REVIEW_FAIL: &str = r#"{"schema":1,"run":"fx-r.1-20261001T000000Z","verdict":"FAIL","evidence":"goal の外の字","kind":"other","at":"§2","ts":"2026-10-01T00:00:55Z"}"#;

const REVIEW_PASS: &str = r#"{"schema":1,"run":"fx-r.1-20261001T010000Z","verdict":"PASS","evidence":"ok","ts":"2026-10-01T01:01:00Z"}"#;

/// 歯ごとの作業場（repo・面の file の置き場・state dir・偽の bd）。
struct Place {
    root: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("bvvroute")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        for dir in [
            root.join("repo/.beads"),
            root.join("files"),
            root.join("state/fleet"),
        ] {
            fs::create_dir_all(dir).expect("置き場");
        }
        fs::write(root.join("files/index.html"), "tz").expect("index.html");
        fs::write(root.join("state/fleet/events.jsonl"), LOG).expect("event log");
        let bd = root.join("bd");
        fs::write(&bd, "#!/bin/sh\necho '[]'\n").expect("偽の bd");
        fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("偽の bd の権限");
        Place { root }
    }

    fn state(&self) -> PathBuf {
        self.root.join("state")
    }

    /// 便の dir に判定の file を 1 つ置く。
    fn put(&self, run: &str, name: &str, body: &str) {
        let dir = self.state().join(PIPE_DIR).join(run);
        fs::create_dir_all(&dir).expect("便の dir");
        fs::write(dir.join(name), format!("{body}\n")).expect("判定の file");
    }

    /// server を立て、口の住所を返す。
    fn serve(&self) -> SocketAddr {
        let config = Config {
            bd: self.root.join("bd").into(),
            state_dir: Some(self.state()),
            ..Config::new(
                self.root.join("repo"),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.root.join("files"),
            )
        };
        let server = Server::bind(&config).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }
}

/// 口の走行の列（Known でなければ落とす）。
fn lines(addr: SocketAddr) -> Vec<RunLine> {
    let (status, body) = get(addr, &format!("{PATH}?bead=fx-r.1"));
    assert_eq!(status, 200, "{body}");
    let doc: RunsDoc = wire::decode(&body).expect("走行の電文の形");
    match doc.runs {
        Reading::Known(lines) => lines,
        Reading::Unknown => panic!("runs が Unknown"),
    }
}

/// 口は走行ごとに state dir の pipe/<run id>/ の verdict.json と review.json を読み、中核の読みと同じ判定を置く。
#[test]
fn bvvroute_reads_run_dir_files() {
    let place = Place::new("read");
    place.put(RUN_A, REVIEW_FILE, REVIEW_FAIL);
    place.put(RUN_B, GATE_FILE, GATE_PASS);
    place.put(RUN_B, REVIEW_FILE, REVIEW_PASS);
    let got = lines(place.serve());
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].run, RUN_A);
    assert_eq!(got[0].gate, Reading::Unknown);
    assert_eq!(got[0].review, review_of(REVIEW_FAIL));
    assert!(matches!(got[0].review, Reading::Known(_)));
    assert_eq!(got[1].run, RUN_B);
    assert_eq!(got[1].gate, gate_of(GATE_PASS));
    assert!(matches!(got[1].gate, Reading::Known(_)));
    assert_eq!(got[1].review, review_of(REVIEW_PASS));
}

/// file の無い便の dir と、schema の違う file と、壊れた file の走行の判定は Unknown。
#[test]
fn bvvroute_missing_files_unknown() {
    let place = Place::new("missing");
    place.put(
        RUN_A,
        REVIEW_FILE,
        r#"{"schema":2,"verdict":"PASS","evidence":"ok"}"#,
    );
    place.put(RUN_B, GATE_FILE, "{broken");
    let got = lines(place.serve());
    for line in &got {
        assert_eq!(line.gate, Reading::Unknown, "{}", line.run);
        assert_eq!(line.review, Reading::Unknown, "{}", line.run);
    }
}

/// 便の dir の file の読みは、run の id が path の普通の要素 1 つのときだけ読む（.. や区切りを持つ id は読まない）。
#[test]
fn bvvroute_bad_run_id_reads_nothing() {
    let place = Place::new("id");
    place.put(RUN_B, GATE_FILE, GATE_PASS);
    fs::write(place.state().join(GATE_FILE), GATE_PASS).expect("state dir の直下の file");
    let runs = Runs::new(Some(&place.state()));
    assert_eq!(runs.pipe, Some(place.state().join("pipe")));
    assert_eq!(
        runs.run_file(RUN_B, GATE_FILE).as_deref(),
        Some(format!("{GATE_PASS}\n").as_str())
    );
    for run in ["", ".", "..", "../pipe", "x/..", "/tmp", "a/b"] {
        assert_eq!(runs.run_file(run, GATE_FILE), None, "{run:?}");
    }
    assert_eq!(runs.run_file(RUN_A, GATE_FILE), None);
    assert_eq!(Runs::new(None).run_file(RUN_B, GATE_FILE), None);
}
