//! tsuzuri-boundary の歯の群 teeth4 の共通の手（群の 2 つ以上の module が字で同じ写しを持っていた helper を 1 つだけ置く・畳みの道具 fold2.py が寄せる・判断の記録 ADR-63 の決定 (8) の (c)）。
//! 写しを持っていた module は use crate::common::… で呼ぶ。字が 1 文字でも違う物はここへ寄せない。
#![cfg(test)]

use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};
use tsuzuri_boundary::stage::terminal::Terminal;
use tsuzuri_boundary::stage::terminal;

pub(crate) const EVENTS: &str = "graph/real/events.jsonl";

pub(crate) const INDEX: &str = "graph/real/design-index.tsv";

pub(crate) const LEDGER: &str = "ledger/bd-list-8.json";

/// 節の URL の行。
pub(crate) const LINE: &str = "board の URL http://srv-a.tailnet.invalid:4801/";

/// 読み込みの終わりの event の字。
pub(crate) const LOAD: &str = r#"{"method":"Page.loadEventFired","params":{"timestamp":1}}"#;

pub(crate) fn debug<T: Debug + Clone + PartialEq + Eq>() {}

/// Err の字（Ok なら落ちる）。
pub(crate) fn err<T: Debug>(got: Result<T, String>, what: &str) -> String {
    match got {
        Ok(v) => panic!("{what}: Ok {v:?}"),
        Err(e) => e,
    }
}

pub(crate) fn fixture() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/stage/terminals.toml");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_secs()
}

pub(crate) fn refused(status: u16, text: &str) -> (u16, String) {
    (status, text.to_string())
}

pub(crate) fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

pub(crate) fn term(name: &str) -> Terminal {
    terminal::lookup(&fixture(), name).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// dir の中の file の path と byte の一覧。
pub(crate) fn tree(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).expect("dir を読む") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                out.push((path.clone(), Vec::new()));
                stack.push(path);
            } else {
                out.push((path.clone(), fs::read(&path).expect("file を読む")));
            }
        }
    }
    out.sort();
    out
}
