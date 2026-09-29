//! 設計の索引の読み（設計の道具を子 process で 1 本撃つだけ・書かない・便 e-read）。
//! 撃つ形は `<program> graph --print --dir <repo>/design-intent`（cwd は repo の置き場・標準入力は空・標準エラーは捨てる）。
//! 起動できない・rc が 0 でない・UTF-8 でない・5 秒を超えて返さない、のどれでも設計の出所は読めない（None）。
//! 設計文書の dir の下の全 file は変化の印（更新時刻と長さ）として見るだけで、中身は読まない。
//! 同じ `Design` とその clone の読みは、走っている 1 本の子 process を分け合う（`coalesce`・便 e-coalesce）。
//! 要約の読み（行 c-summary-wire）は `<program> graph --print --summary --dir <repo>/design-intent` を同じ形で撃ち、
//! 索引の読みとは別の場で合流する。--summary を知らない folio では要約だけが読めない（索引は読める）。
//! 裁定の書き出しの読み（行 c-g3g7）は `<program> check --emit-rulings --dir <repo>/design-intent` を同じ形で撃ち、
//! 別の場で合流する。rc が 0 でない書き出しは全数でない（床がまだ分からないか不合格）ので読めない（None）。
//! 3 つの読みは持ち回しの表（`Held`・行 e-held-design）を通り、設計文書の dir の下の全 file の印が撃つ前と同じで
//! `DESIGN_HOLD` の内なら撃ち直さない（裁定の書き出しは contracts の dir と git の印も見る・`rulings_marks`）。

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use super::coalesce::{Coalesce, GRACE};
use super::held::{Held, git_marks, walk};
use super::ledger::capture;

/// 既定の program の名（引数 --folio で替える）。
pub const FOLIO: &str = "folio";

/// 設計文書の dir（repo の置き場の下）。
pub const DESIGN_DIR: &str = "design-intent";

/// 設計の道具に渡す引数の頭（この後に設計文書の dir の path が続く）。
pub const FOLIO_ARGS: [&str; 3] = ["graph", "--print", "--dir"];

/// 要約の読みに渡す引数の頭（この後に設計文書の dir の path が続く・行 c-summary-wire）。
pub const SUMMARY_ARGS: [&str; 4] = ["graph", "--print", "--summary", "--dir"];

/// 裁定の書き出しの読みに渡す引数の頭（この後に設計文書の dir の path が続く・行 c-g3g7）。
pub const RULINGS_ARGS: [&str; 3] = ["check", "--emit-rulings", "--dir"];

/// 設計の道具が返すまでの上限（要件 NFR2 の上限・台帳の読みと同じ）。越えれば止めて読めない。
pub const FOLIO_TIMEOUT: Duration = Duration::from_secs(5);

/// 走っている読みに合流した呼び出しが待つ上限（`FOLIO_TIMEOUT` に 1 秒を足す・便 e-coalesce）。
pub const FOLIO_WAIT: Duration = FOLIO_TIMEOUT.saturating_add(GRACE);

/// 設計の道具の読みを持ち回す上限（判断の記録 ADR-23・印が動けば上限の内でも撃ち直す）。
pub const DESIGN_HOLD: Duration = Duration::from_secs(300);

/// 契約の dir（repo の置き場の下・裁定の書き出しが読む）。
const CONTRACTS_DIR: &str = "contracts";

/// 設計の索引の読みの出所（repo の置き場と設計の道具の program）。
/// clone は読みの合流の場を分け合う（比べるのは repo と folio だけ）。
#[derive(Debug, Clone)]
pub struct Design {
    pub repo: PathBuf,
    pub folio: OsString,
    shared: Coalesce<String>,
    summary_shared: Coalesce<String>,
    rulings_shared: Coalesce<String>,
    held: Held,
}

impl PartialEq for Design {
    fn eq(&self, other: &Design) -> bool {
        (&self.repo, &self.folio) == (&other.repo, &other.folio)
    }
}

impl Eq for Design {}

impl Design {
    pub fn new(repo: impl Into<PathBuf>, folio: impl Into<OsString>) -> Design {
        Design {
            repo: repo.into(),
            folio: folio.into(),
            shared: Coalesce::new(),
            summary_shared: Coalesce::new(),
            rulings_shared: Coalesce::new(),
            held: Held::new(),
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
    /// 走っている読みが在れば新しく撃たず、その終わりを `FOLIO_WAIT` まで待って同じ結果を返す（便 e-coalesce）。
    /// 設計文書の dir の印が撃つ前と同じで `DESIGN_HOLD` の内なら、撃たず前の字を返す（行 e-held-design）。
    pub fn text(&self) -> Option<String> {
        let args = self.args();
        self.held
            .get(&self.key(&args), &self.marks(), DESIGN_HOLD, || {
                self.shared.share(FOLIO_WAIT, || {
                    let out = capture(&self.folio, args.clone(), &self.repo, FOLIO_TIMEOUT)?;
                    String::from_utf8(out).ok()
                })
            })
    }

    /// 読みの鍵（program と引数の列と cwd）。
    fn key(&self, args: &[OsString]) -> Vec<OsString> {
        let mut key = vec![self.folio.clone()];
        key.extend(args.iter().cloned());
        key.push(self.repo.clone().into_os_string());
        key
    }

    /// 要約の読みに渡す引数の列。
    pub fn summary_args(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = SUMMARY_ARGS.iter().map(OsString::from).collect();
        args.push(self.dir().into_os_string());
        args
    }

    /// 設計の道具を要約の引数で撃ち、標準出力の字を返す（読めなければ None・行 c-summary-wire）。
    /// 索引の読みとは別の場で合流し、合流した呼び出しは `FOLIO_WAIT` まで待つ。
    pub fn summary(&self) -> Option<String> {
        let args = self.summary_args();
        self.held
            .get(&self.key(&args), &self.marks(), DESIGN_HOLD, || {
                self.summary_shared.share(FOLIO_WAIT, || {
                    let out = capture(&self.folio, args.clone(), &self.repo, FOLIO_TIMEOUT)?;
                    String::from_utf8(out).ok()
                })
            })
    }

    /// 裁定の書き出しの読みに渡す引数の列（行 c-g3g7）。
    pub fn rulings_args(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = RULINGS_ARGS.iter().map(OsString::from).collect();
        args.push(self.dir().into_os_string());
        args
    }

    /// 設計の道具を裁定の書き出しの引数で撃ち、標準出力の字を返す（rc が 0 でなければ読めず None・行 c-g3g7）。
    /// 索引と要約の読みとは別の場で合流し、合流した呼び出しは `FOLIO_WAIT` まで待つ。
    pub fn rulings(&self) -> Option<String> {
        let args = self.rulings_args();
        self.held
            .get(&self.key(&args), &self.rulings_marks(), DESIGN_HOLD, || {
                self.rulings_shared.share(FOLIO_WAIT, || {
                    let out = capture(&self.folio, args.clone(), &self.repo, FOLIO_TIMEOUT)?;
                    String::from_utf8(out).ok()
                })
            })
    }

    /// 変化の印の file（設計文書の dir の下の全 file・path の順・dir が無ければ空）。
    pub fn marks(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        walk(&self.dir(), &mut out);
        out.sort();
        out
    }

    /// 裁定の書き出しの持ち回しの印の file（`marks` に contracts の dir の下の全 file と、git の HEAD と packed-refs と
    /// refs の dir の下の全 file を足す・`.git` が file なら gitdir と commondir をたどる）。
    pub fn rulings_marks(&self) -> Vec<PathBuf> {
        let mut out = self.marks();
        walk(&self.repo.join(CONTRACTS_DIR), &mut out);
        git_marks(&self.repo, &mut out);
        out.sort();
        out.dedup();
        out
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
