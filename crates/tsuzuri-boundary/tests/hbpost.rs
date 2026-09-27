//! POST の 4 つの口を routes の下の file に移す歯
//! （接頭辞 hbpost_・設計ノート surface-hub 行 hb-post の完了の条件・判断の記録 ADR-13）。
//! mod.rs と自分の file は CARGO_MANIFEST_DIR から読む。

use std::fs;
use std::path::Path;

use tsuzuri_boundary::server::route::{Key, Match};
use tsuzuri_boundary::server::{Route, batch, policy, ruling};
use tsuzuri_contract::account::HEARTBEAT_PATH;

/// 着地済みの行とこの文書の行の verify の filter の語。
const FILTERS: [&str; 49] = [
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
    "hook_",
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
    "hbproc_",
    "hbconf_",
    "hbroute_",
    "hbpost_",
    "hsblock_",
    "hspage_",
    "hsderive_",
];

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn hbpost_four_posts_in_route_all() {
    for (name, path) in [
        ("ruling", ruling::PATH),
        ("batch", batch::PATH),
        ("policy", policy::PATH),
        ("heartbeat", HEARTBEAT_PATH),
    ] {
        let route = Route::ALL
            .iter()
            .find(|r| r.name() == name)
            .unwrap_or_else(|| panic!("Route の ALL に {name} が無い"));
        let key = Key {
            method: "POST",
            path: Match::Exact(path),
        };
        assert_eq!(route.key(), key, "{name} の鍵");
        assert!(route.key().matches("POST", path), "{name} の鍵が {path} に当たらない");
        assert!(!route.key().matches("GET", path), "{name} の鍵が GET に当たる");
    }
}

#[test]
fn hbpost_mod_rs_drops_post_fns() {
    let module = read("src/server/mod.rs");
    for word in [
        "fn post_ruling",
        "fn post_batch",
        "fn post_policy",
        "fn post_heartbeat",
        "HEARTBEAT_PATH",
    ] {
        assert!(!module.contains(word), "mod.rs に {word} が残る");
    }
}

#[test]
fn hbpost_teeth_names_stay_apart() {
    let text = read("tests/hbpost.rs");
    let mut names = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "#[test]" {
            continue;
        }
        let head = lines.next().expect("test の属性の後の行");
        let name = head
            .trim()
            .strip_prefix("fn ")
            .and_then(|rest| rest.split_once('('))
            .map(|(name, _)| name)
            .unwrap_or_else(|| panic!("test の属性の後が fn でない: {head}"));
        names.push(name.to_string());
    }
    assert!(!names.is_empty(), "test の fn が無い");
    for name in names {
        let rest = name
            .strip_prefix("hbpost_")
            .unwrap_or_else(|| panic!("{name} が hbpost_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
