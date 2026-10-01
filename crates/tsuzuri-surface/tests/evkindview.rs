//! 合図の種類で読み直す口を決める歯（接頭辞 evkind_・計画 surface-plan の行 c-ev-kind の完了の条件 (5)〜(8)）。
//! 口の path の字は面の block の module の定数と契約の定数から引く。
#![cfg(test)]

use std::fs;
use std::path::Path;

use tsuzuri_contract::ledger::{ITEM_PATH, LEDGER_CHANGED_EVENT};
use tsuzuri_contract::surface::{BOARD_CHANGED_EVENT, ChangeKind};
use tsuzuri_contract::{account, project, runs};
use tsuzuri_surface::project::{ask, ledger, map, next, nodearound, pipeline, seat};
use tsuzuri_surface::view::{changed_kinds, path_kinds, reloads};

use ChangeKind::{Account, Design, Ledger, Runs, Seat};

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// 字の中の fn の本文（fn の頭から次の行頭の fn・pub fn・async fn・pub async fn の前まで）。
fn function<'a>(text: &'a str, name: &str) -> &'a str {
    let start = text
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("fn {name} が在る"));
    let rest = &text[start..];
    let end = ["\nfn ", "\npub fn ", "\nasync fn ", "\npub async fn "]
        .iter()
        .filter_map(|m| rest[1..].find(m).map(|i| i + 1))
        .min()
        .unwrap_or(rest.len());
    &rest[..end]
}

/// 面が読む口の 13 の path（query を付けて読む口は読む形の query を付けた字・グラフの眺めの口は行 m-map-graph で外した）。
fn paths() -> [String; 13] {
    [
        seat::PATH.to_string(),
        next::PATH.to_string(),
        ledger::PATH.to_string(),
        ledger::METRICS_PATH.to_string(),
        ledger::UNREF_PATH.to_string(),
        format!("{ITEM_PATH}fx-a.1"),
        ask::PATH.to_string(),
        pipeline::PATH.to_string(),
        map::PATH.to_string(),
        format!("{}?id=fx-a.1&k=1&fold=", nodearound::PATH),
        format!("{}?bead=fx-a.1", runs::PATH),
        account::PATH.to_string(),
        project::PATH.to_string(),
    ]
}

#[test]
fn evkind_path_kinds_table() {
    let want: [&[ChangeKind]; 13] = [
        &[Seat],
        &[Ledger, Runs, Seat],
        &[Ledger],
        &[Ledger],
        &[Ledger],
        &[Ledger],
        &[Ledger],
        &[Ledger, Runs],
        &[Ledger, Runs, Design],
        &[Ledger, Runs, Design],
        &[Runs],
        &[Account, Ledger],
        &[],
    ];
    for (path, want) in paths().iter().zip(want) {
        assert_eq!(path_kinds(path), want, "{path}");
    }
    for path in ["/api/other", "/api/seatx", "/api/ledgers?x=1"] {
        assert_eq!(path_kinds(path), ChangeKind::ALL, "{path}");
    }
}

#[test]
fn evkind_seat_reloads_two() {
    let seat_only = changed_kinds(BOARD_CHANGED_EVENT, Some("{\"at\":8,\"kinds\":[\"seat\"]}"));
    assert_eq!(seat_only, [Seat]);
    let paths = paths();
    let hit: Vec<&str> = paths
        .iter()
        .filter(|p| reloads(p, &seat_only))
        .map(String::as_str)
        .collect();
    assert_eq!(hit, [seat::PATH, next::PATH]);

    let ledger_kinds = changed_kinds(LEDGER_CHANGED_EVENT, Some("{\"at\":7}"));
    let miss: Vec<&str> = paths
        .iter()
        .filter(|p| !reloads(p, &ledger_kinds))
        .map(String::as_str)
        .collect();
    let runs_path = format!("{}?bead=fx-a.1", runs::PATH);
    assert_eq!(miss, [seat::PATH, runs_path.as_str(), project::PATH]);
}

#[test]
fn evkind_changed_kinds() {
    for data in [None, Some("x"), Some("{\"at\":7}")] {
        assert_eq!(changed_kinds(LEDGER_CHANGED_EVENT, data), [Ledger], "{data:?}");
    }
    assert_eq!(
        changed_kinds(
            BOARD_CHANGED_EVENT,
            Some("{\"at\":8,\"kinds\":[\"runs\",\"design\"]}")
        ),
        [Runs, Design]
    );
    for data in [
        None,
        Some("not json"),
        Some("{\"at\":8}"),
        Some("{\"at\":8,\"kinds\":[]}"),
    ] {
        assert_eq!(
            changed_kinds(BOARD_CHANGED_EVENT, data),
            ChangeKind::ALL,
            "{data:?}"
        );
    }
    assert!(changed_kinds("open", Some("{\"at\":8,\"kinds\":[\"seat\"]}")).is_empty());
}

#[test]
fn evkind_net_wiring_text() {
    let net = read("src/net.rs");
    let changed = function(&net, "reload_changed");
    for w in [
        "changed_kinds(",
        "event.type_()",
        "event.data()",
        "reload_if(|path| reloads(path, &kinds))",
    ] {
        assert!(changed.contains(w), "reload_changed に {w} が無い: {changed}");
    }
    let all = function(&net, "reload_all");
    assert!(all.contains("reload_if(|_| true)"), "{all}");
    let connect = function(&net, "connect");
    assert_eq!(
        connect
            .matches("Closure::<dyn FnMut(MessageEvent)>::new(reload_changed)")
            .count(),
        1,
        "{connect}"
    );
    assert!(connect.contains("for name in RELOAD_EVENTS {"), "{connect}");
}
