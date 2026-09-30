//! 起動の引数を `Config::new` と残りの欄の埋めで組む歯（接頭辞 hbconf_・設計ノート surface-hub 行 hb-config の完了の条件）。
//! 欄の歯は `Config::new` の値の欄を 1 つずつ比べ、Config を struct の字で組まない（欄が増えても歯を直さずに済む）。
//! 定義の置き場と、main.rs と 9 つの歯の file の組み方は src と tests の字を読んで見る。
#![cfg(test)]

use std::ffi::OsString;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use tsuzuri_boundary::server::{Config, design, ledger, ruling};
use tsuzuri_contract::ledger::BDW;

/// Config を struct の字で組む 9 つの歯の file。
const TEETH: [&str; 9] = [
    "server_src.rs",
    "server_seat.rs",
    "server_coalesce.rs",
    "server_read.rs",
    "server_ask.rs",
    "server_min.rs",
    "server_batch.rs",
    "server_view.rs",
    "server_acctwire.rs",
];

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// `fn config` の開き括弧から、次の 4 つの空白と閉じ括弧だけの行までの本文。
fn config_body(file: &str) -> String {
    let text = read(&format!("tests/{file}"));
    let start = text
        .find("fn config(")
        .unwrap_or_else(|| panic!("{file} に fn config が無い"));
    let open = start
        + text[start..]
            .find('{')
            .unwrap_or_else(|| panic!("{file} の fn config に開き括弧が無い"));
    let mut body = String::new();
    for line in text[open..].lines() {
        if line == "    }" {
            return body;
        }
        body.push_str(line);
        body.push('\n');
    }
    panic!("{file} の fn config に閉じ括弧の行が無い")
}

#[test]
fn hbconf_config_moves_to_config_rs() {
    let (module, config) = (read("src/server/mod.rs"), read("src/server/config.rs"));
    assert!(!module.contains("pub struct Config"), "mod.rs に Config が残る");
    assert!(module.contains("mod config;"), "mod.rs に mod config が無い");
    assert!(
        module.contains("pub use self::config::Config;"),
        "mod.rs が Config を再公開しない"
    );
    for def in [
        "#[derive(Debug, Clone, PartialEq, Eq)]\npub struct Config {",
        "pub repo: PathBuf,",
        "pub bind: SocketAddr,",
        "pub files: PathBuf,",
        "pub bd: OsString,",
        "pub state_dir: Option<PathBuf>,",
        "pub folio: OsString,",
        "pub bdw: OsString,",
        "pub seat: Option<String>,",
        "pub scribe2: OsString,",
        "pub fn new(repo: PathBuf, bind: SocketAddr, files: PathBuf) -> Config",
    ] {
        assert!(config.contains(def), "config.rs に {def} が無い");
    }
}

#[test]
fn hbconf_new_fills_defaults() {
    let repo = PathBuf::from("/tmp/hbconf-repo");
    let bind: SocketAddr = "127.0.0.1:0".parse().expect("bind 先");
    let files = PathBuf::from("/tmp/hbconf-files");
    let config = Config::new(repo.clone(), bind, files.clone());
    assert_eq!(config.repo, repo);
    assert_eq!(config.bind, bind);
    assert_eq!(config.files, files);
    assert_eq!(config.bd, OsString::from(ledger::BD));
    assert_eq!(config.state_dir, None);
    assert_eq!(config.folio, OsString::from(design::FOLIO));
    assert_eq!(config.bdw, OsString::from(BDW));
    assert_eq!(config.seat, None);
    assert_eq!(config.scribe2, OsString::from(ruling::SCRIBE2));
}

#[test]
fn hbconf_teeth_fill_the_rest_with_new() {
    for file in TEETH {
        let body = config_body(file);
        assert!(body.contains("Config::new"), "{file} の fn config が Config::new を呼ばない");
        if body.contains("Config {") {
            assert!(
                body.contains("..Config::new"),
                "{file} の fn config が残りの欄を Config::new で埋めない"
            );
        }
    }
}

#[test]
fn hbconf_main_parse_fills_the_rest_with_new() {
    let main = read("src/main.rs");
    assert!(main.contains("Config::new"), "main.rs が Config::new を呼ばない");
    assert_eq!(
        main.matches("Config {").count(),
        main.matches("..Config::new").count(),
        "main.rs に Config::new で埋めない struct の字が在る"
    );
}
