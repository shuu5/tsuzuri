//! bead の走行の時間軸の口の歯（接頭辞 runsdoc_・設計ノート surface-wave3b 行 e-runs の完了の条件）。
//! server は同じ process の thread で 127.0.0.1 の空き port に立て、GET を素の TCP で撃つ。
//! fixture の tests/fixtures/graph/real/events.jsonl（読むだけ）を state dir の fleet/events.jsonl に写す。
#![cfg(test)]

use std::fs;
use std::net::SocketAddr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;

use crate::common::{get, manifest, read};
use tsuzuri_boundary::server::route::{Key, Match};
use tsuzuri_boundary::server::{Config, Route, Server};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::runs::{PATH, RunsDoc};
use tsuzuri_contract::wire;
use tsuzuri_core::pipeline::runs_of;

/// 着地済みの行とこの波の行の verify の filter の語（この行の接頭辞は並べない）。
const FILTERS: [&str; 65] = [
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "acctframe_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "askcard_",
    "batchpanel_",
    "board_min_",
    "contract_form_",
    "frame_",
    "gapspage_",
    "gquestion_",
    "graph_",
    "gview_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hook_",
    "hsblock_",
    "hsderive_",
    "hspage_",
    "ledgerblock_",
    "mapgraph_",
    "mapview_",
    "nextstep_",
    "nodepage_",
    "parts_",
    "pipe_",
    "project_",
    "question_",
    "seatblock_",
    "seatcard_",
    "server_",
    "skeleton_",
    "stage_",
    "stats_",
    "steady_",
    "topbar_",
    "tz_",
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
    "ilink_",
    "afocus_",
    "nxact_",
    "urpanel_",
    "saxis_",
    "ticker_",
    "hfig_",
    "pmore_",
    "lspark_",
    "apop_",
    "brand_",
    "nbatch_",
];

fn real_log() -> String {
    read(&manifest("../../tests/fixtures/graph/real/events.jsonl"))
}

/// 歯ごとの作業場（repo・面の file の置き場・state dir・偽の bd）。
struct Place {
    root: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("runsdoc")
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
        fs::write(root.join("state/fleet/events.jsonl"), real_log()).expect("event log");
        let bd = root.join("bd");
        fs::write(&bd, "#!/bin/sh\necho '[]'\n").expect("偽の bd");
        fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("偽の bd の権限");
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

#[test]
fn runsdoc_get_matches_core() {
    let place = Place::new("core");
    let addr = place.serve(true);
    let (status, body) = get(addr, &format!("{PATH}?bead=t3-hub.2"));
    assert_eq!(status, 200, "{body}");
    let bead = BeadId::new("t3-hub.2").expect("bead の id");
    let want = wire::encode(&runs_of(&real_log(), &bead)).expect("電文");
    assert_eq!(body, want);
    let doc: RunsDoc = wire::decode(&body).expect("走行の電文の形");
    assert!(matches!(&doc.runs, Reading::Known(lines) if lines.len() == 3));
}

#[test]
fn runsdoc_no_state_dir_is_unknown() {
    let place = Place::new("nostate");
    let addr = place.serve(false);
    let (status, body) = get(addr, &format!("{PATH}?bead=t3-hub.2"));
    assert_eq!(status, 200, "{body}");
    let doc: RunsDoc = wire::decode(&body).expect("走行の電文の形");
    assert_eq!(doc.bead, BeadId::new("t3-hub.2").expect("bead の id"));
    assert_eq!(doc.runs, Reading::Unknown);
}

#[test]
fn runsdoc_refuses_bad_bead() {
    let place = Place::new("refuse");
    let addr = place.serve(true);
    for (path, want) in [
        (PATH.to_string(), "no-bead"),
        (format!("{PATH}?bead="), "no-bead"),
        (format!("{PATH}?bead=a%20b"), "id-shape"),
    ] {
        assert_eq!(get(addr, &path), (400, want.to_string()), "{path}");
    }
}

#[test]
fn runsdoc_route_is_listed() {
    let route = Route::ALL
        .iter()
        .find(|r| r.name() == "runs")
        .expect("Route の ALL に名 runs の口が無い");
    assert_eq!(
        route.key(),
        Key {
            method: "GET",
            path: Match::Exact(PATH),
        }
    );
}

#[test]
fn runsdoc_own_names_clean() {
    let text = read(&manifest("tests/teeth3/runsdoc.rs"));
    let mut names = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "#[test]" {
            continue;
        }
        let decl = lines.next().expect("test の属性の次の行");
        let name = decl
            .trim()
            .strip_prefix("fn ")
            .and_then(|r| r.split('(').next())
            .unwrap_or_else(|| panic!("fn の宣言でない: {decl}"));
        names.push(name.to_string());
    }
    assert!(names.len() >= 5, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("runsdoc_")
            .unwrap_or_else(|| panic!("{name} が runsdoc_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
