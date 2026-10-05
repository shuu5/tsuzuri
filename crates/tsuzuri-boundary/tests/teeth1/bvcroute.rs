//! 器の局面の出力の口の歯（接頭辞 bvcroute_・設計ノート surface-wave26a 行 c-case-read の完了の条件）。
//! server は同じ process の thread で 127.0.0.1 の空き port に立て、GET を素の TCP で撃つ。
//! fixture の tests/fixtures/case/lifecycle.json と lifecycle.stale（読むだけ）を state dir の fleet の下に写す。
#![cfg(test)]

use std::fs;
use std::net::SocketAddr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;

use crate::common::{get, manifest};
use tsuzuri_boundary::server::cases::Cases;
use tsuzuri_boundary::server::route::{Key, Match};
use tsuzuri_boundary::server::{Config, Route, Server};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::case::{CaseDoc, PATH};
use tsuzuri_contract::wire;
use tsuzuri_core::case::cases_of;

fn fixture(name: &str) -> String {
    let path = manifest("../../tests/fixtures/case").join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// 歯ごとの作業場（repo・面の file の置き場・state dir・偽の bd）。
struct Place {
    root: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("bvcroute")
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
        Place { root }
    }

    /// state dir の fleet の下に file を置く。
    fn put(&self, name: &str, text: &str) {
        fs::write(self.root.join("state/fleet").join(name), text).expect("fleet の file");
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

/// 口の電文を読む（200 でなければ落ちる）。
fn doc_of(addr: SocketAddr) -> CaseDoc {
    let (status, body) = get(addr, PATH);
    assert_eq!(status, 200, "{body}");
    wire::decode(&body).expect("局面の出力の電文の形")
}

#[test]
fn bvcroute_get_matches_core() {
    let place = Place::new("core");
    place.put("lifecycle.json", &fixture("lifecycle.json"));
    place.put("lifecycle.stale", &fixture("lifecycle.stale"));
    let addr = place.serve(true);
    let (status, body) = get(addr, PATH);
    assert_eq!(status, 200, "{body}");
    let want = cases_of(
        &fixture("lifecycle.json"),
        Some(&fixture("lifecycle.stale")),
    );
    assert_eq!(body, wire::encode(&want).expect("電文"));
    let doc: CaseDoc = wire::decode(&body).expect("局面の出力の電文の形");
    assert!(matches!(&doc.parts, Reading::Known(parts) if parts.len() == 9));
    assert_eq!(doc.stale, ["ledger-gate"]);
}

#[test]
fn bvcroute_no_state_dir_is_unknown() {
    let place = Place::new("nostate");
    place.put("lifecycle.json", &fixture("lifecycle.json"));
    let doc = doc_of(place.serve(false));
    assert_eq!(doc.parts, Reading::Unknown);
    assert!(doc.stale.is_empty());
}

#[test]
fn bvcroute_absent_files() {
    let place = Place::new("absent");
    let addr = place.serve(true);
    assert_eq!(doc_of(addr).parts, Reading::Unknown, "出力が無い");
    place.put("lifecycle.json", &fixture("lifecycle.json"));
    let doc = doc_of(addr);
    assert!(
        matches!(&doc.parts, Reading::Known(parts) if parts.len() == 9),
        "古さの印の file が無くても読む"
    );
    assert!(doc.stale.is_empty(), "古さの印の file が無ければ古くない");
}

#[test]
fn bvcroute_unreadable_stale_is_unknown() {
    let place = Place::new("stale");
    place.put("lifecycle.json", &fixture("lifecycle.json"));
    place.put("lifecycle.stale", "not json");
    let addr = place.serve(true);
    assert_eq!(doc_of(addr).parts, Reading::Unknown, "読めない印の file");
    let stale = place.root.join("state/fleet/lifecycle.stale");
    fs::remove_file(&stale).expect("印の file を外す");
    fs::create_dir(&stale).expect("印の名の dir");
    assert_eq!(
        doc_of(addr).parts,
        Reading::Unknown,
        "在って読めない印（無いのでない）"
    );
}

#[test]
fn bvcroute_paths_under_state_dir() {
    let cases = Cases::new(Some(Path::new("/s")));
    assert_eq!(cases.json, Some(PathBuf::from("/s/fleet/lifecycle.json")));
    assert_eq!(cases.stale, Some(PathBuf::from("/s/fleet/lifecycle.stale")));
    assert_eq!(Cases::new(None).texts(), (String::new(), None));
}

#[test]
fn bvcroute_route_is_listed() {
    let route = Route::ALL
        .iter()
        .find(|r| r.name() == "cases")
        .expect("Route の ALL に名 cases の口が無い");
    assert_eq!(
        route.key(),
        Key {
            method: "GET",
            path: Match::Exact(PATH),
        }
    );
}
