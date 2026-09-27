//! 設計の索引の読み（設計の道具を子 process で 1 本撃つだけ・書かない・便 e-read）。
//! 撃つ形は `<program> graph --print --dir <repo>/design-intent`（cwd は repo の置き場・標準入力は空・標準エラーは捨てる）。
//! 起動できない・rc が 0 でない・UTF-8 でない・5 秒を超えて返さない、のどれでも設計の出所は読めない（None）。
//! 設計文書の dir の下の全 file は変化の印（更新時刻と長さ）として見るだけで、中身は読まない。

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::ledger::capture;

/// 既定の program の名（引数 --folio で替える）。
pub const FOLIO: &str = "folio";

/// 設計文書の dir（repo の置き場の下）。
pub const DESIGN_DIR: &str = "design-intent";

/// 設計の道具に渡す引数の頭（この後に設計文書の dir の path が続く）。
pub const FOLIO_ARGS: [&str; 3] = ["graph", "--print", "--dir"];

/// 設計の道具が返すまでの上限（要件 NFR2 の上限・台帳の読みと同じ）。越えれば止めて読めない。
pub const FOLIO_TIMEOUT: Duration = Duration::from_secs(5);

/// 設計の索引の読みの出所（repo の置き場と設計の道具の program）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Design {
    pub repo: PathBuf,
    pub folio: OsString,
}

impl Design {
    pub fn new(repo: impl Into<PathBuf>, folio: impl Into<OsString>) -> Design {
        Design {
            repo: repo.into(),
            folio: folio.into(),
        }
    }

    /// 設計文書の dir。
    pub fn dir(&self) -> PathBuf {
        self.repo.join(DESIGN_DIR)
    }

    /// 設計の道具に渡す引数の列。
    pub fn args(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = FOLIO_ARGS.iter().map(OsString::from).collect();
        args.push(self.dir().into_os_string());
        args
    }

    /// 設計の道具を撃ち、標準出力の字を返す（読めなければ None）。
    pub fn text(&self) -> Option<String> {
        let out = capture(&self.folio, self.args(), &self.repo, FOLIO_TIMEOUT)?;
        String::from_utf8(out).ok()
    }

    /// 変化の印の file（設計文書の dir の下の全 file・path の順・dir が無ければ空）。
    pub fn marks(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        walk(&self.dir(), &mut out);
        out.sort();
        out
    }
}

/// dir の下の file を集める（symlink の dir はたどらない・読めない dir は飛ばす）。
fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        match entry.file_type() {
            Ok(t) if t.is_dir() => walk(&path, out),
            Ok(_) => out.push(path),
            Err(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Design;
    use std::path::PathBuf;

    #[test]
    fn server_read_folio_args() {
        let design = Design::new("/r", "folio");
        assert_eq!(design.dir(), PathBuf::from("/r/design-intent"));
        assert_eq!(
            design.args(),
            ["graph", "--print", "--dir", "/r/design-intent"].map(std::ffi::OsString::from)
        );
    }

    #[test]
    fn server_read_unstartable_folio_is_unread() {
        let design = Design::new(std::env::temp_dir(), "/nonexistent/tz-no-such-folio");
        assert_eq!(design.text(), None);
    }

    #[test]
    fn server_read_marks_walk_the_dir() {
        let root = std::env::temp_dir().join(format!("tz-design-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("design-intent");
        std::fs::create_dir_all(dir.join("adr")).expect("置き場");
        std::fs::write(dir.join("rules.yaml"), "r").expect("file");
        std::fs::write(dir.join("adr/ADR-7.yaml"), "a").expect("file");
        let design = Design::new(&root, "folio");
        assert_eq!(
            design.marks(),
            [dir.join("adr/ADR-7.yaml"), dir.join("rules.yaml")]
        );
        assert!(Design::new(root.join("none"), "folio").marks().is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }
}
