//! 席に届いていない裁定の口の歯（接頭辞 fundl_・設計ノート surface-wave23b 行 f-undelivered・要件 FR9）。
//! 台帳は歯の中で組み（逐語は作った字）、偽の bd が作業場の out.json を出し、server を同じ process の thread で立てて
//! 口 GET /api/unreceived を撃つ。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::surface::RulingId;
use tsuzuri_contract::wire;
use tsuzuri_core::delivery::{Route, mark_line};

const STAMP: &str = "2026-09-29T10:00:00Z";
const QUESTION: &str = "intake:question";

fn ruling(id: &str) -> RulingId {
    RulingId::new(id).expect("裁定の id")
}

/// 裁定の行（問いの欄と逐語の欄だけ）。
fn ruling_line(id: &str, question: &str) -> String {
    format!("裁定 id = {id}・問い = {question}・逐語 = はい")
}

/// 印の行（配達の口か停止の経路）。
fn mark(id: &str, route: Route) -> String {
    mark_line(&ruling(id), route, "20260929T1010Z")
}

/// 台帳の bead 1 本の JSON（notes は行を改行でつなぐ）。
fn bead(id: &str, status: &str, labels: &[&str], notes: &[String]) -> String {
    let text = |s: &str| wire::encode(&s).expect("字の電文");
    let labels: Vec<String> = labels.iter().map(|l| text(l)).collect();
    format!(
        "{{\"id\":{},\"title\":{},\"status\":{},\"priority\":2,\"issue_type\":\"task\",\"created_at\":\"{STAMP}\",\"updated_at\":\"{STAMP}\",\"labels\":[{}],\"notes\":{}}}",
        text(id),
        text(&format!("問い — {id}")),
        text(status),
        labels.join(","),
        text(&notes.join("\n")),
    )
}

fn ledger(beads: &[String]) -> String {
    format!("[\n{}\n]\n", beads.join(",\n"))
}

/// 節の混ぜた台帳（fx-u.1 は配達の口の印・fx-u.2 は印なし・fx-u.3 は停止の印と印の無い取り消しの行・
/// fx-u.4 は label の無い bead・fx-u.5 は印の無い束の行）。
fn mixed() -> String {
    let (u1, u2) = ("fx-u.1:20260929T1000Z-1", "fx-u.2:20260929T1001Z-1");
    let (u3, u3r) = ("fx-u.3:20260929T1002Z-1", "fx-u.3:20260929T1003Z-1");
    let (u4, u5) = ("fx-u.4:20260929T1003Z-2", "fx-u.5:20260929T1004Z-1");
    ledger(&[
        bead(
            "fx-u.1",
            "closed",
            &[QUESTION],
            &[ruling_line(u1, "fx-u.1"), mark(u1, Route::Deliver)],
        ),
        bead("fx-u.2", "closed", &[QUESTION], &[ruling_line(u2, "fx-u.2")]),
        bead(
            "fx-u.3",
            "open",
            &[QUESTION],
            &[
                ruling_line(u3, "fx-u.3"),
                mark(u3, Route::Stop),
                format!("{}・取り消す = {u3}", ruling_line(u3r, "fx-u.3")),
            ],
        ),
        bead("fx-u.4", "closed", &[], &[ruling_line(u4, "fx-u.4")]),
        bead(
            "fx-u.5",
            "closed",
            &[QUESTION],
            &[format!(
                "裁定 id = {u5}・問い = fx-u.5・束 = batch:20260929T1004Z-1・逐語 = はい"
            )],
        ),
    ])
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 歯ごとの作業場（repo・面の file・偽の bd と bdw）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
}

impl Place {
    /// `out` を偽の bd が出す字にした作業場。
    fn new(name: &str, out: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fundl").join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, files) = (root.join("repo"), root.join("files"));
        for dir in [repo.join(".beads"), files.clone()] {
            fs::create_dir_all(dir).expect("作業場の dir");
        }
        fs::write(files.join("index.html"), "tz").expect("index.html");
        fs::write(root.join("out.json"), out).expect("out.json");
        script(&root.join("bd"), &format!("exec cat '{}/out.json'", root.display()));
        script(&root.join("bdw"), "exit 0");
        Place { root, repo, files }
    }

    /// server を同じ process の thread で立てて口の住所を返す。
    fn serve(&self, read_only: bool) -> SocketAddr {
        let config = Config {
            bd: self.root.join("bd").into(),
            bdw: self.root.join("bdw").into(),
            read_only,
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

/// 口 GET /api/unreceived を撃ち、状態の code と本文を返す。
fn get(addr: SocketAddr) -> (u16, String) {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(30))).expect("timeout");
    s.write_all(
        format!("GET /api/unreceived HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n")
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

fn reading(addr: SocketAddr) -> Reading<Vec<RulingId>> {
    let (status, body) = get(addr);
    assert_eq!(status, 200, "{body}");
    wire::decode(&body).expect("Reading の電文")
}

/// (1) 混ぜた台帳の口は 200 と、印の無い裁定の id を台帳の順に Known で返す（読むだけの server も同じ）。
#[test]
fn fundl_route_lists_unmarked() {
    let want = Reading::Known(vec![
        ruling("fx-u.2:20260929T1001Z-1"),
        ruling("fx-u.3:20260929T1003Z-1"),
        ruling("fx-u.5:20260929T1004Z-1"),
    ]);
    for (name, read_only) in [("mixed", false), ("mixed-ro", true)] {
        let place = Place::new(name, &mixed());
        assert_eq!(reading(place.serve(read_only)), want, "{name}");
    }
}

/// (2) 台帳の字が JSON の配列でなければ 200 と Unknown・停止の印の在る裁定だけの台帳なら 200 と Known の空の列。
#[test]
fn fundl_route_unknown_and_empty() {
    let place = Place::new("unknown", "{\"not\":\"an array\"}\n");
    assert_eq!(reading(place.serve(false)), Reading::Unknown);
    let id = "fx-u.6:20260929T1005Z-1";
    let stopped = ledger(&[bead(
        "fx-u.6",
        "closed",
        &[QUESTION],
        &[ruling_line(id, "fx-u.6"), mark(id, Route::Stop)],
    )]);
    let place = Place::new("empty", &stopped);
    assert_eq!(reading(place.serve(false)), Reading::Known(Vec::new()));
}
