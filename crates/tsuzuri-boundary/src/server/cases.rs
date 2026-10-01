//! 器の局面の出力の読み（file を 2 つ読むだけ・書かない・行 c-case-read）。
//! file は `<state dir>/fleet/lifecycle.json`（出力）と同じ dir の `lifecycle.stale`（古さの印・器の case-lifecycle §5.1）。
//! 器の読み手と同じく古さの印 → 出力の順に読む。state dir を省いたときは、出力は空の字で古さの印は無い。
//! 出力が無いか読めなければ空の字、古さの印は無ければ None・在って読めなければ空の字（中核の `cases_of` の入力の形）。

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// 局面の出力の file（state dir の下の path の区切りの列）。
pub const LIFECYCLE_JSON: [&str; 2] = ["fleet", "lifecycle.json"];

/// 古さの印の file（state dir の下の path の区切りの列）。
pub const LIFECYCLE_STALE: [&str; 2] = ["fleet", "lifecycle.stale"];

/// 局面の出力の読みの出所（state dir を省けばどちらも None）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cases {
    pub json: Option<PathBuf>,
    pub stale: Option<PathBuf>,
}

impl Cases {
    pub fn new(state_dir: Option<&Path>) -> Cases {
        let under = |parts: [&str; 2]| {
            state_dir.map(|dir| parts.iter().fold(dir.to_path_buf(), |p, s| p.join(s)))
        };
        Cases {
            json: under(LIFECYCLE_JSON),
            stale: under(LIFECYCLE_STALE),
        }
    }

    /// 出力の字（無いか読めなければ空の字）と古さの印の字（無ければ None・在って読めなければ空の字）。
    pub fn texts(&self) -> (String, Option<String>) {
        let stale = self
            .stale
            .as_ref()
            .and_then(|path| match std::fs::read_to_string(path) {
                Ok(text) => Some(text),
                Err(e) if e.kind() == ErrorKind::NotFound => None,
                Err(_) => Some(String::new()),
            });
        let json = self
            .json
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok());
        (json.unwrap_or_default(), stale)
    }
}
