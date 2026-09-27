//! 口を 1 口 1 file に割り、口の列を組み立ての script が dir から生成する歯
//! （接頭辞 hbroute_・設計ノート surface-hub 行 hb-route の完了の条件・判断の記録 ADR-13）。
//! dir と mod.rs と build.rs と測りの記録は CARGO_MANIFEST_DIR から、生成の字は OUT_DIR から読む。

use std::fs;
use std::path::Path;

use tsuzuri_boundary::server::Route;
use tsuzuri_boundary::server::route::{Key, Match};

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

fn manifest(rel: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// src/server/routes の下の拡張子 rs の file の名（拡張子を除く・byte の順）。
fn dir_names() -> Vec<String> {
    let dir = manifest("src/server/routes");
    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| entry.expect("dir の項").path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|e| e == "rs"))
        .map(|path| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .expect("file の名")
                .to_string()
        })
        .collect();
    names.sort();
    names
}

/// 同じ method の 2 つの鍵が、どの path でも両方には当たらない（`Key::matches` の決まり）。
fn apart(a: &Key, b: &Key) -> bool {
    if a.method != b.method {
        return true;
    }
    match (a.path, b.path) {
        (Match::Exact(x), Match::Exact(y)) => x != y,
        (Match::Exact(x), Match::Prefix(p)) | (Match::Prefix(p), Match::Exact(x)) => {
            !x.starts_with(p)
        }
        (Match::Prefix(x), Match::Prefix(y)) => !x.starts_with(y) && !y.starts_with(x),
    }
}

#[test]
fn hbroute_all_names_follow_dir() {
    let names: Vec<&str> = Route::ALL.iter().map(|r| r.name()).collect();
    assert_eq!(names, dir_names(), "Route の ALL の名が routes の dir の file と違う");
    let mut unique = names.clone();
    unique.dedup();
    assert_eq!(unique, names, "Route の ALL の名が重なる");
}

#[test]
fn hbroute_keys_cover_twelve_gets() {
    let keys: Vec<Key> = Route::ALL.iter().map(|r| r.key()).collect();
    let wanted = [
        Match::Exact("/api/ledger"),
        Match::Exact("/api/pipeline"),
        Match::Exact("/api/metrics"),
        Match::Exact("/api/next"),
        Match::Exact("/api/seat"),
        Match::Exact("/api/account"),
        Match::Exact("/api/graph"),
        Match::Exact("/api/graph/view"),
        Match::Exact("/api/around"),
        Match::Exact("/api/unreflected"),
        Match::Exact("/api/questions"),
        Match::Prefix("/api/ledger/"),
    ];
    for path in wanted {
        let key = Key {
            method: "GET",
            path,
        };
        assert!(keys.contains(&key), "口の鍵 {key:?} が無い");
    }
}

#[test]
fn hbroute_keys_never_overlap() {
    for (i, a) in Route::ALL.iter().enumerate() {
        for b in &Route::ALL[i + 1..] {
            assert!(
                apart(&a.key(), &b.key()),
                "{} と {} の鍵が重なる",
                a.name(),
                b.name()
            );
        }
    }
}

#[test]
fn hbroute_key_matches_rule() {
    let exact = Key {
        method: "GET",
        path: Match::Exact("/api/ledger"),
    };
    assert!(exact.matches("GET", "/api/ledger"));
    assert!(!exact.matches("POST", "/api/ledger"));
    assert!(!exact.matches("GET", "/api/ledger/x"));
    let prefix = Key {
        method: "GET",
        path: Match::Prefix("/api/ledger/"),
    };
    assert!(prefix.matches("GET", "/api/ledger/x"));
    assert!(prefix.matches("GET", "/api/ledger/"));
    assert!(!prefix.matches("GET", "/api/ledger"));
    assert!(!prefix.matches("POST", "/api/ledger/x"));
}

#[test]
fn hbroute_mod_rs_keeps_only_events() {
    let module = read(&manifest("src/server/mod.rs"));
    for def in ["fn route(", "fn around(", "fn account("] {
        assert!(!module.contains(def), "mod.rs に {def} が残る");
    }
    let quoted: Vec<&str> = module
        .match_indices("\"/api/")
        .map(|(at, _)| {
            let rest = &module[at + 1..];
            &rest[..rest.find('"').expect("閉じの引用符")]
        })
        .collect();
    assert_eq!(quoted, ["/api/surface/events"], "mod.rs の引用符の /api/ の字");
}

#[test]
fn hbroute_build_script_writes_paths() {
    let script = read(&manifest("build.rs"));
    for line in [
        "cargo:rerun-if-changed=src/server/routes",
        "cargo:rerun-if-changed=build.rs",
    ] {
        assert!(script.contains(line), "build.rs に {line} が無い");
    }
    let generated = read(&Path::new(env!("OUT_DIR")).join("routes.rs"));
    let mut from = 0;
    for name in dir_names() {
        let path = format!("{}/src/server/routes/{name}.rs", env!("CARGO_MANIFEST_DIR"));
        let decl = format!("#[path = {path:?}]\nmod {name};");
        let at = generated[from..]
            .find(&decl)
            .unwrap_or_else(|| panic!("生成の字に {decl} が名の順に無い"));
        from += at + decl.len();
    }
    for line in generated.lines() {
        assert!(!line.trim_start().starts_with("//!"), "生成の字に内側の doc: {line}");
        assert!(!line.trim_start().starts_with("#!"), "生成の字に内側の属性: {line}");
    }
}

#[test]
fn hbroute_measure_r2_recorded() {
    let text = read(&manifest("../../docs/measure/r2-boundary-build.txt"));
    let value = |key: &str| -> Option<&str> {
        text.lines().find_map(|line| {
            let (k, v) = line.split_once('=')?;
            (k.trim() == key).then(|| v.trim())
        })
    };
    let secs = |key: &str| -> f64 {
        value(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| panic!("{key} の秒の数が無い"))
    };
    let (before, after) = (secs("before_s"), secs("after_s"));
    if after - before > 0.3 {
        assert!(
            value("reason").is_some_and(|r| !r.is_empty()),
            "差が 0.3 秒を越えるのに reason が無い"
        );
    }
}

#[test]
fn hbroute_teeth_names_stay_apart() {
    let text = read(&manifest("tests/hbroute.rs"));
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
            .strip_prefix("hbroute_")
            .unwrap_or_else(|| panic!("{name} が hbroute_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
