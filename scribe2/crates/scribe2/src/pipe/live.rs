//! host の根の走りの札（行 xp-host-runs・判断の記録 ADR-37 の決定 (7) の行 xp-host-live の前半）。
//!
//! runner を起こしてから終わりを見届けるまで、器が選んだ口座の label を持つ札 1 枚を host の根の `runs/`
//! （[`host_runs_dir`]・受付札の置き場と同じ state dir の親から導く）に置き、終わりで外す（[`Hold`] の `Drop`）。
//! 同じ親の下の置き場（本番と実験）の走りが 1 つの dir に並ぶので、口座の選びが置き場の event の外の走りを
//! 数えられる（数える側は後の行）。札は受付札と同じく一時の印で、口座を選ばない周（親の環境の継承）は置かない。
//! 置けない周は runner を止めず、理由の 1 行を呼び手へ返す（黙って落とさない・NFR4）。

use super::admission::now_ms;
use crate::fleet::json_lite::{self, Value};
use crate::fleet::SCHEMA;
use crate::seat::{host_runs_dir, sanitize_target};
use std::fs;
use std::path::{Path, PathBuf};

/// 走りの札の拡張子。
pub const EXT: &str = "run";

/// 書きかけの札の拡張子（rename の前・読み手は [`EXT`] しか数えない）。
const PARTIAL_EXT: &str = "partial";

/// 走りの札 1 枚の中身（1 行 JSON・schema / pid / run / account / place / ts）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticket {
    /// 札を持つ process（runner の終わりを見届ける driver）。
    pub pid: u32,
    /// 便 id。
    pub run: String,
    /// 器が選んで runner に渡した口座の label。
    pub account: String,
    /// 札を置いた置き場（state dir の字）。数える側は自分の置き場の札を event で数え、札では数えない。
    pub place: String,
    /// 札を書いた時刻（UNIX epoch の ms）。
    pub ts_ms: u64,
}

impl Ticket {
    /// 札の 1 行。
    pub fn to_line(&self) -> String {
        json_lite::write_object(&[
            ("schema", Value::Num(SCHEMA)),
            ("pid", Value::Num(u64::from(self.pid))),
            ("run", Value::Str(self.run.clone())),
            ("account", Value::Str(self.account.clone())),
            ("place", Value::Str(self.place.clone())),
            ("ts", Value::Num(self.ts_ms)),
        ])
    }

    /// 札の 1 行を読む。schema 違い・数値の壊れ・key の欠け・空の口座は `None`。
    pub fn parse(text: &str) -> Option<Self> {
        let pairs = json_lite::parse_object(text.trim()).ok()?;
        let get = |key: &str| pairs.iter().find(|(found, _)| found == key).map(|(_, value)| value);
        if get("schema")?.as_num()? != SCHEMA {
            return None;
        }
        let account = get("account")?.as_str()?.to_owned();
        Some(Self {
            pid: u32::try_from(get("pid")?.as_num()?).ok()?,
            run: get("run")?.as_str()?.to_owned(),
            account: (!account.is_empty()).then_some(account)?,
            place: get("place")?.as_str()?.to_owned(),
            ts_ms: get("ts")?.as_num()?,
        })
    }
}

/// 置いた札の外し番（`Drop` で札を消す）。
#[derive(Debug)]
pub struct Hold {
    /// 札の path。
    path: PathBuf,
}

impl Hold {
    /// 札の path。
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Hold {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// runner の走りの札を置く。口座の label が無い周（親の環境の継承）は置かない（`Ok(None)`）。置けない周は理由の 1 行。
pub fn hold(state_dir: &Path, run: &str, account: Option<&str>) -> Result<Option<Hold>, String> {
    let Some(account) = account else {
        return Ok(None);
    };
    let ticket = Ticket {
        pid: std::process::id(),
        run: run.to_owned(),
        account: account.to_owned(),
        place: state_dir.display().to_string(),
        ts_ms: now_ms(),
    };
    put(&host_runs_dir(state_dir), &ticket).map(Some)
}

/// 札を `<pid>-<run>.run` に書く（dir を作り、書きかけを別名で書いて rename する＝読み手は半端を見ない）。
fn put(dir: &Path, ticket: &Ticket) -> Result<Hold, String> {
    let stem = format!("{}-{}", ticket.pid, sanitize_target(&ticket.run));
    let partial = dir.join(format!("{stem}.{PARTIAL_EXT}"));
    let path = dir.join(format!("{stem}.{EXT}"));
    fs::create_dir_all(dir)
        .and_then(|()| fs::write(&partial, format!("{}\n", ticket.to_line())))
        .and_then(|()| fs::rename(&partial, &path))
        .map_err(|err| {
            let _ = fs::remove_file(&partial);
            format!("pipe: 走りの札 {} を置けない: {err}", path.display())
        })?;
    Ok(Hold { path })
}

#[cfg(test)]
mod tests {
    use super::{hold, Ticket, EXT};
    use crate::fleet::SCHEMA;
    use crate::seat::{host_runs_dir, host_slots_dir};
    use std::path::PathBuf;

    /// temp の dir の下の置き場（`<temp>/xphost-<pid>-<tag>/state`）。host の根はその親の下に在る。
    fn place(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("xphost-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        root.join("state")
    }

    fn ticket() -> Ticket {
        Ticket { pid: 4242, run: "r-1".to_owned(), account: "acct-a".to_owned(), place: "/s/state".to_owned(), ts_ms: 1_700 }
    }

    /// 札の 1 行は口座と置き場を運び、読み返すと同じ札。schema 違い・口座の欠けか空・置き場の欠け・数でない pid は読まない。
    #[test]
    fn xphost_ticket_line_round_trips_with_account_and_place() {
        let line = ticket().to_line();
        assert_eq!(
            line,
            format!("{{\"schema\":{SCHEMA},\"pid\":4242,\"run\":\"r-1\",\"account\":\"acct-a\",\"place\":\"/s/state\",\"ts\":1700}}")
        );
        assert_eq!(Ticket::parse(&format!("{line}\n")), Some(ticket()));
        let with = |old: &str, new: &str| Ticket::parse(&line.replacen(old, new, 1));
        assert_eq!(with(&format!("\"schema\":{SCHEMA}"), "\"schema\":999"), None, "schema 違い");
        assert_eq!(with(",\"account\":\"acct-a\"", ""), None, "口座の欠け");
        assert_eq!(with("\"account\":\"acct-a\"", "\"account\":\"\""), None, "空の口座");
        assert_eq!(with(",\"place\":\"/s/state\"", ""), None, "置き場の欠け");
        assert_eq!(with("\"pid\":4242", "\"pid\":\"4242\""), None, "数でない pid");
    }

    /// 札の置き場は受付札の置き場と同じ host の根の `runs/`。
    #[test]
    fn xphost_runs_dir_is_beside_the_slots_dir() {
        let state = place("dir");
        let slots = host_slots_dir(&state);
        assert_eq!(host_runs_dir(&state), slots.parent().map(|root| root.join("runs")).unwrap_or_default());
    }

    /// 口座の label を持つ周は札を 1 枚（名は自分の pid と便 id の字・書きかけを残さない）置き、外し番を落とすと消える。
    #[test]
    fn xphost_hold_puts_one_ticket_and_drop_removes_it() {
        let state = place("hold");
        let held = hold(&state, "s2-x/1", Some("acct-a"));
        let dir = host_runs_dir(&state);
        let names = || -> Vec<String> {
            std::fs::read_dir(&dir).map(|found| found.flatten().map(|entry| entry.file_name().to_string_lossy().into_owned()).collect()).unwrap_or_default()
        };
        assert_eq!(names(), vec![format!("{}-s2-x_1.{EXT}", std::process::id())], "札 1 枚・書きかけ無し");
        let text = std::fs::read_to_string(dir.join(&names()[0])).unwrap_or_default();
        let found = Ticket::parse(&text).map(|found| (found.pid, found.run, found.account, found.place));
        assert_eq!(found, Some((std::process::id(), "s2-x/1".to_owned(), "acct-a".to_owned(), state.display().to_string())));
        assert!(held.as_ref().is_ok_and(|found| found.as_ref().is_some_and(|found| found.path() == dir.join(&names()[0]))));
        drop(held);
        assert_eq!(names(), Vec::<String>::new(), "外し番を落とすと消える");
    }

    /// 札を置くのは口座の label を持つ周だけで、口座を選ばない周（label が無い）は札も dir も作らない。
    #[test]
    fn xphost_hold_puts_a_ticket_only_with_a_label() {
        let state = place("none");
        assert!(hold(&state, "r-1", None).is_ok_and(|found| found.is_none()), "label が無い");
        assert!(!host_runs_dir(&state).exists(), "dir も作らない");
        let held = hold(&state, "r-1", Some("acct-a"));
        assert!(held.as_ref().is_ok_and(|found| found.as_ref().is_some_and(|found| found.path().is_file())), "label を持つ: {held:?}");
    }

    /// 置けない周（host の根が file）は理由の 1 行を返し、書きかけを残さない。
    #[test]
    fn xphost_unwritable_dir_returns_a_reason() {
        let state = place("file");
        let root = host_runs_dir(&state).parent().map(PathBuf::from).unwrap_or_default();
        let _ = std::fs::create_dir_all(root.parent().unwrap_or(&root));
        let _ = std::fs::write(&root, "");
        let found = hold(&state, "r-1", Some("acct-a"));
        assert!(found.as_ref().is_err_and(|reason| reason.starts_with("pipe: 走りの札 ") && reason.contains("を置けない")), "{found:?}");
    }
}
