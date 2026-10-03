//! 行 c-case-columns の歯（境界・接頭辞 bvqsrv_）: 口 GET /api/pipeline が state dir の器の局面の出力を要求のたびに読み、
//! 走行の無い札の段と理由と since を契約の部品で決める（読めなければ台帳の blocks の割り）。
//! server は同じ process の thread で 127.0.0.1 の空き port に立て、GET を素の TCP で撃つ。台帳は偽の bd が返す字。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::{PipelineBoard, Reading, Stage};
use tsuzuri_contract::wire;

/// 部品の since（2026-09-27T11:30:00Z の epoch 秒）。
const SINCE_SECS: u64 = 1_790_508_600;

/// 配れる open の task の字（acceptance に設計 pointer の行・blocker を選ぶ）。
fn task(id: &str, blocker: Option<&str>) -> String {
    let deps = blocker.map_or(String::new(), |on| {
        format!(r#","dependencies":[{{"issue_id":"{id}","depends_on_id":"{on}","type":"blocks"}}]"#)
    });
    format!(
        r#"{{"id":"{id}","title":"t {id}","status":"open","issue_type":"task","acceptance_criteria":"design = contracts/x.toml#{id}"{deps}}}"#
    )
}

/// 節の局面の出力（qs.1 は dependency・qs.2 は host-busy・qs.3 の部品は無い）。
fn lifecycle() -> String {
    let part = |id: &str, reason: &str| {
        format!(
            r#"{{"part":"contract","id":"{id}","phase":"contract-queued","turn":"vessel","since":"2026-09-27T11:30:00Z","reason":"{reason}","closed":false,"overdue":null,"links":{{"on":[]}}}}"#
        )
    };
    format!(
        r#"{{"version":1,"generated_at":"2026-09-27T11:59:00Z","parts":[{},{}]}}"#,
        part("qs.1", "dependency"),
        part("qs.2", "host-busy")
    )
}

/// 歯ごとの作業場（repo・面の file の置き場・state dir・偽の bd）。
struct Place {
    root: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("bvqsrv")
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
        let ledger = [
            task("qs.1", None),
            task("qs.2", Some("qs.3")),
            task("qs.3", None),
        ];
        fs::write(root.join("ledger.json"), format!("[{}]", ledger.join(","))).expect("台帳の字");
        let run = r#"{"schema":1,"ts":"2026-09-27T11:00:00Z","kind":"RunCreated","run":"qs.9-20260927T110000Z","bead":"qs.9","stage":"Intake"}"#;
        fs::write(root.join("state/fleet/events.jsonl"), format!("{run}\n")).expect("event log");
        let bd = root.join("bd");
        let body = format!(
            "#!/bin/sh\nexec cat '{}'\n",
            root.join("ledger.json").display()
        );
        fs::write(&bd, body).expect("偽の bd");
        fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("偽の bd の権限");
        Place { root }
    }

    /// state dir の fleet の下に局面の出力を置く。
    fn put_lifecycle(&self) {
        fs::write(self.root.join("state/fleet/lifecycle.json"), lifecycle()).expect("局面の出力");
    }

    fn serve(&self) -> SocketAddr {
        let config = Config {
            bd: self.root.join("bd").into(),
            state_dir: Some(self.root.join("state")),
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

/// GET /api/pipeline を 1 つ撃ち、板を読む（200 でなければ落ちる）。
fn pipeline(addr: SocketAddr) -> PipelineBoard {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    s.write_all(format!("GET /api/pipeline HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes())
        .expect("要求を書く");
    let mut out = Vec::new();
    s.read_to_end(&mut out).expect("応答を読む");
    let text = String::from_utf8(out).expect("応答の字");
    let (head, body) = text.split_once("\r\n\r\n").expect("頭と本文");
    assert!(head.starts_with("HTTP/1.1 200"), "{head}");
    wire::decode(body).expect("板の電文の形")
}

/// 札の段と理由と since（札が無ければ落ちる）。
fn row(b: &PipelineBoard, id: &str) -> (Stage, Option<String>, Option<u64>) {
    let Reading::Known(cards) = &b.cards else {
        panic!("札が Unknown");
    };
    let c = cards
        .iter()
        .find(|c| c.contract.as_str() == id)
        .unwrap_or_else(|| panic!("{id} の札が無い"));
    (c.stage, c.reason.clone(), c.since)
}

#[test]
fn bvqsrv_columns_from_lifecycle() {
    let place = Place::new("cased");
    place.put_lifecycle();
    let b = pipeline(place.serve());
    assert_eq!(
        row(&b, "qs.1"),
        (Stage::Blocked, Some("dependency".into()), Some(SINCE_SECS))
    );
    assert_eq!(
        row(&b, "qs.2"),
        (Stage::Queued, Some("host-busy".into()), Some(SINCE_SECS))
    );
    assert_eq!(row(&b, "qs.3"), (Stage::Queued, None, None), "部品の無い札");
}

#[test]
fn bvqsrv_unreadable_falls_back_then_reads() {
    let place = Place::new("fallback");
    let addr = place.serve();
    let b = pipeline(addr);
    assert_eq!(row(&b, "qs.1"), (Stage::Queued, None, None));
    assert_eq!(
        row(&b, "qs.2"),
        (Stage::Blocked, None, None),
        "台帳の blocks"
    );
    place.put_lifecycle();
    let b = pipeline(addr);
    assert_eq!(row(&b, "qs.1").0, Stage::Blocked, "要求のたびに読む");
    assert_eq!(row(&b, "qs.2").0, Stage::Queued);
}
