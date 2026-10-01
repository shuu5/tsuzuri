//! 器の event log の読み（file を 1 つ読むだけ・書かない・便 e-read）。
//! file は `<state dir>/fleet/events.jsonl`。state dir を省いたとき・file が無いか読めないときは、
//! 走行の出所は読めない（None）。file は変化の印（更新時刻と長さ）としても見る。
//! 便の dir（`<state dir>/pipe/<run id>/`）の gate と審査の判定の file も読む（行 c-run-verdict）。
//! 器はその file を書いた後に段の event を足すので、変化の印は event log だけで足りる。

use std::path::{Component, Path, PathBuf};

/// event log の file（state dir の下の path の区切りの列）。
pub const EVENTS_LOG: [&str; 2] = ["fleet", "events.jsonl"];

/// 便の dir の置き場（state dir の下）。
pub const PIPE_DIR: &str = "pipe";

/// 便の dir の gate の判定の file（器の pipe gate が書く）。
pub const GATE_FILE: &str = "verdict.json";

/// 便の dir の審査の判定の file（器の pipe review が書く）。
pub const REVIEW_FILE: &str = "review.json";

/// 走行の読みの出所（event log の file と便の dir の置き場・state dir を省けば None）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Runs {
    pub log: Option<PathBuf>,
    pub pipe: Option<PathBuf>,
}

impl Runs {
    pub fn new(state_dir: Option<&Path>) -> Runs {
        Runs {
            log: state_dir.map(|dir| EVENTS_LOG.iter().fold(dir.to_path_buf(), |p, s| p.join(s))),
            pipe: state_dir.map(|dir| dir.join(PIPE_DIR)),
        }
    }

    /// 便の dir の file `name` の字（run の id が path の普通の要素 1 つでないか、読めなければ None）。
    pub fn run_file(&self, run: &str, name: &str) -> Option<String> {
        let mut parts = Path::new(run).components();
        let (Some(Component::Normal(_)), None) = (parts.next(), parts.next()) else {
            return None;
        };
        std::fs::read_to_string(self.pipe.as_ref()?.join(run).join(name)).ok()
    }

    /// event log の字（読めなければ None）。
    pub fn text(&self) -> Option<String> {
        std::fs::read_to_string(self.log.as_ref()?).ok()
    }

    /// 変化の印の file（event log の 1 つ・state dir を省けば空）。
    pub fn marks(&self) -> Vec<PathBuf> {
        self.log.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Runs;
    use std::path::{Path, PathBuf};

    #[test]
    fn server_read_events_log_path() {
        let runs = Runs::new(Some(Path::new("/s")));
        assert_eq!(runs.log, Some(PathBuf::from("/s/fleet/events.jsonl")));
        assert_eq!(runs.marks(), [PathBuf::from("/s/fleet/events.jsonl")]);
        let none = Runs::new(None);
        assert_eq!(none.text(), None);
        assert!(none.marks().is_empty());
        assert_eq!(
            Runs::new(Some(Path::new("/nonexistent/tz-state"))).text(),
            None
        );
    }
}
