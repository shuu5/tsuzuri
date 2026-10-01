//! 未反映の口と指標の口が state dir の局面の出力を読む歯（接頭辞 bvuroute_・設計ノート surface-wave26a 行 c-unref-lc の完了の条件）。
//! server は同じ process の thread で 127.0.0.1 の空き port に立て、GET を素の TCP で撃つ。
//! fixture の tests/fixtures/case/unref.json と lifecycle.stale（読むだけ）を state dir の fleet の下に写す。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use tsuzuri_boundary::server::board::list;
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::stats::{LedgerStats, UnreflectedCount, UnreflectedKind, UnreflectedList};
use tsuzuri_contract::wire;
use tsuzuri_core::ledger::unreflected;

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/case")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// 歯ごとの作業場（repo・面の file の置き場・state dir・空の台帳を返す偽の bd）。
struct Place {
    root: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("bvuroute")
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
        let bd = root.join("bd");
        fs::write(&bd, "#!/bin/sh\necho '[]'\n").expect("偽の bd");
        fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("偽の bd の権限");
        for (to, from) in [
            ("lifecycle.json", "unref.json"),
            ("lifecycle.stale", "lifecycle.stale"),
        ] {
            fs::write(root.join("state/fleet").join(to), fixture(from)).expect("fleet の file");
        }
        Place { root }
    }

    /// state dir を置くか選んで server を立て、口の住所を返す。
    fn serve(&self, with_state: bool) -> SocketAddr {
        let config = Config {
            bd: self.root.join("bd").into(),
            state_dir: with_state.then(|| self.root.join("state")),
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

/// GET を 1 つ撃ち、本文を返す（200 でなければ落ちる）。
fn get(addr: SocketAddr, path: &str) -> String {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    s.write_all(format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes())
        .expect("要求を書く");
    let mut out = Vec::new();
    s.read_to_end(&mut out).expect("応答を読む");
    let text = String::from_utf8(out).expect("応答の字");
    let (head, body) = text.split_once("\r\n\r\n").expect("頭と本文");
    assert!(head.starts_with("HTTP/1.1 200"), "{path}: {head}");
    body.to_string()
}

/// (1) 口 /api/unreflected の電文は、state dir の出力と古さの印と台帳の字から中核の unreflected が組む一覧の電文と
/// 同じ字で、memo 2・裁定 1・発話 1 と古さの印 ledger-gate を持つ。
#[test]
fn bvuroute_list_matches_core() {
    let place = Place::new("list");
    let body = get(place.serve(true), "/api/unreflected");
    let want = list(&unreflected(
        "[]",
        &fixture("unref.json"),
        Some(&fixture("lifecycle.stale")),
    ));
    assert_eq!(body, wire::encode(&want).expect("電文"));
    let got: UnreflectedList = wire::decode(&body).expect("未反映の電文の形");
    let len = |r: &Reading<Vec<_>>| match r {
        Reading::Known(rows) => Some(rows.len()),
        Reading::Unknown => None,
    };
    assert_eq!(
        (len(&got.memos), len(&got.rulings), len(&got.utterances)),
        (Some(2), Some(1), Some(1))
    );
    assert_eq!(got.stale, ["ledger-gate"]);
}

/// (2) 口 /api/metrics の指標の未反映の 3 欄は state dir の出力から数えた件数（memo 2・裁定 1・発話 1・和 4）。
#[test]
fn bvuroute_metrics_counts() {
    let place = Place::new("metrics");
    let body = get(place.serve(true), "/api/metrics");
    let Reading::Known(stats) = wire::decode::<Reading<LedgerStats>>(&body).expect("指標の電文")
    else {
        panic!("空の台帳は読める: {body}");
    };
    let count = |kind, count| UnreflectedCount { kind, count };
    assert_eq!(
        stats.unreflected_kinds,
        vec![
            count(UnreflectedKind::Memo, 2),
            count(UnreflectedKind::Ruling, 1),
            count(UnreflectedKind::Utterance, 1),
        ]
    );
    assert_eq!(stats.unreflected, 4);
    assert!(stats.unreflected_unknown.is_empty());
}

/// (3) state dir を省いた server は 2 つの口とも 3 種を「まだ分からない」にし、古さの印を持たず、数は 0。
#[test]
fn bvuroute_no_state_dir_unknown() {
    let place = Place::new("nostate");
    let addr = place.serve(false);
    let got: UnreflectedList =
        wire::decode(&get(addr, "/api/unreflected")).expect("未反映の電文の形");
    assert_eq!(
        (got.memos, got.rulings, got.utterances),
        (Reading::Unknown, Reading::Unknown, Reading::Unknown)
    );
    assert!(got.stale.is_empty());
    let body = get(addr, "/api/metrics");
    let Reading::Known(stats) = wire::decode::<Reading<LedgerStats>>(&body).expect("指標の電文")
    else {
        panic!("空の台帳は読める: {body}");
    };
    assert_eq!(stats.unreflected, 0);
    assert_eq!(stats.unreflected_unknown, UnreflectedKind::ALL.to_vec());
}
