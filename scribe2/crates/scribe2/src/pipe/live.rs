//! host の根の走りの札。
//! 置けない周は runner を止めず、理由の 1 行を呼び手へ返す。
//! 出所: 判断の記録 ADR-37 判断の記録 ADR-41

use super::admission::now_ms;
use crate::fleet::json_lite::{self, Value};
use crate::fleet::store::started_ms;
use crate::fleet::SCHEMA;
use crate::seat::{host_runs_dir, sanitize_target};
use std::collections::BTreeSet;
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

/// 札の dir の判じ（[`scan`] の返り・消さない）。
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Scan {
    /// ほかの置き場の生きた札が持つ口座の label。
    pub elsewhere: BTreeSet<String>,
    /// 回収する札（持ち主が死んだ札か pid の再利用・読めない札）。
    pub doomed: Vec<PathBuf>,
}

/// 札の dir を判じる（pure に `started` を受ける・消さない）。`started` は pid の起動時刻（epoch ms・無い process は `None`）で、
/// 生きた札 = process が在り、起動時刻が札の ts 以前（受付札の判じ [`super::admission::judge`] と同じ形）。自分の置き場
/// `place` の札は数えない（その走りは置き場の event の便数で数える）。拡張子 [`EXT`] でない file は見ない。dir を読めない周は空。
pub fn scan(dir: &Path, place: &str, started: impl Fn(u32) -> Option<u64>) -> Scan {
    let mut found = Scan::default();
    let Ok(entries) = fs::read_dir(dir) else {
        return found;
    };
    for path in entries.flatten().map(|entry| entry.path()) {
        if path.extension().is_none_or(|ext| ext != EXT) {
            continue;
        }
        match fs::read_to_string(&path).ok().as_deref().and_then(Ticket::parse) {
            Some(ticket) if started(ticket.pid).is_some_and(|ms| ms <= ticket.ts_ms) => {
                if ticket.place != place {
                    found.elsewhere.insert(ticket.account);
                }
            }
            Some(_) | None => found.doomed.push(path),
        }
    }
    found
}

/// ほかの置き場の生きた走りの札を持つ口座（便用の選定の頭の鍵・行 xp-host-live）。置き場 `state_dir` の host の根の札を
/// [`scan`] で判じ、死んだ札と読めない札を消して回収する（別の読み手が先に消した札は黙って飛ばす）。
pub fn elsewhere(state_dir: &Path) -> BTreeSet<String> {
    let found = scan(&host_runs_dir(state_dir), &state_dir.display().to_string(), |pid| started_ms(pid).started());
    for path in &found.doomed {
        let _ = fs::remove_file(path);
    }
    found.elsewhere
}

#[cfg(test)]
mod tests {
    use super::{elsewhere, hold, put, scan, Ticket, EXT, PARTIAL_EXT};
    use crate::fleet::SCHEMA;
    use crate::seat::{host_runs_dir, host_slots_dir};
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};

    /// temp の dir の下の置き場（`<temp>/xphost-<pid>-<tag>/state`）。host の根はその親の下に在る。
    fn place(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("xphost-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        crate::pipe::fixture::held(root).join("state")
    }

    /// 置き場の根（返した path の親）は歯の thread の終わりに消える（[`crate::pipe::fixture::held`]・memo t3-hub.74.49.10）。
    #[test]
    fn vschd_live_place_root_is_gone_after_the_thread_ends() {
        let (inside, gone, root) = crate::pipe::fixture::made_in_thread(|| {
            let state = place("vschd-gone");
            let _ = std::fs::create_dir_all(&state);
            state.parent().map(Path::to_path_buf).unwrap_or_default()
        });
        assert!(inside, "thread の中では根が在り file を置ける: {}", root.display());
        assert!(gone, "join の後は根が無い: {}", root.display());
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

    /// 札の dir に札 1 枚を `<stem>.<ext>` の名で書く。
    fn write(dir: &Path, stem: &str, ext: &str, line: &str) -> PathBuf {
        let path = dir.join(format!("{stem}.{ext}"));
        let _ = std::fs::create_dir_all(dir);
        let _ = std::fs::write(&path, format!("{line}\n"));
        path
    }

    /// ほかの置き場（`/s/other`）の生きた札（pid 11・ts 1700）から口座の label と 1 つの欄だけを替えた札。
    fn other(account: &str, pid: u32, place: &str) -> String {
        Ticket { pid, account: account.to_owned(), place: place.to_owned(), ..ticket() }.to_line()
    }

    /// 偽の起動時刻: pid 11 は札の ts ちょうど・12 は無い process・13 は ts の 1 ms 後（pid の再利用）・ほかは ts より前。
    fn started(pid: u32) -> Option<u64> {
        match pid {
            11 => Some(1_700),
            12 => None,
            13 => Some(1_701),
            _ => Some(1_699),
        }
    }

    /// 自分の置き場を `/s/state` とする札の dir の見本（生きた札 2 枚・自分の置き場の札・死んだ札・pid の再利用・壊れた札・
    /// 書きかけ）と、回収に回る札の path（名の順）。
    fn fixture(tag: &str) -> (PathBuf, Vec<PathBuf>) {
        let dir = place(tag).join("runs");
        write(&dir, "11-live", EXT, &other("live", 11, "/s/other"));
        write(&dir, "4242-early", EXT, &other("early", 4242, "/s/other"));
        write(&dir, "11-own", EXT, &other("own", 11, "/s/state"));
        write(&dir, "11-partial", PARTIAL_EXT, &other("partial", 11, "/s/other"));
        let broken = other("broken", 11, "/s/other").replacen(&format!("\"schema\":{SCHEMA}"), "\"schema\":999", 1);
        let doomed = vec![
            write(&dir, "11-broken", EXT, &broken),
            write(&dir, "12-dead", EXT, &other("dead", 12, "/s/other")),
            write(&dir, "13-reused", EXT, &other("reused", 13, "/s/other")),
        ];
        (dir, doomed)
    }

    /// 数えるのはほかの置き場の生きた札（起動時刻が札の ts 以前・ちょうども生）の口座だけで、自分の置き場の札と拡張子
    /// run でない file は数えない。dir が無い周は空。
    #[test]
    fn xplive_scan_names_accounts_of_live_tickets_of_other_places() {
        let (dir, _) = fixture("scan-live");
        let found = scan(&dir, "/s/state", started);
        let want: BTreeSet<String> = ["early".to_owned(), "live".to_owned()].into_iter().collect();
        assert_eq!(found.elsewhere, want, "生きた札 2 枚の口座だけ");
        assert_eq!(scan(&dir, "/s/other", started).elsewhere, BTreeSet::from(["own".to_owned()]), "置き場の字で除く");
        assert_eq!(scan(&dir.join("none"), "/s/state", started), super::Scan::default(), "dir が無い");
    }

    /// 持ち主の process が無い札・起動時刻が札の ts より後の札（pid の再利用）・読めない札は回収に回し、口座を数えない。
    /// 生きた札・自分の置き場の札・書きかけは回収に回さない。
    #[test]
    fn xplive_scan_dooms_dead_reused_and_broken_tickets() {
        let (dir, want) = fixture("scan-doom");
        let mut found = scan(&dir, "/s/state", started).doomed;
        found.sort();
        assert_eq!(found, want, "壊れた札・死んだ札・pid の再利用の 3 枚だけ");
        assert!(dir.join(format!("12-dead.{EXT}")).is_file(), "scan は消さない");
    }

    /// 置き場の host の根の札を読み、ほかの置き場の生きた札の口座を返し、回収に回る札を消す（生きた札は残す）。自分の
    /// 置き場の札は数えない。持ち主の判じは本物の起動時刻（自分の pid・札の ts 0 は起動より前＝pid の再利用）。
    #[test]
    fn xplive_elsewhere_reads_the_host_root_and_reclaims_doomed_tickets() {
        let mine = place("reclaim");
        let theirs = mine.with_file_name("other");
        let held = hold(&theirs, "r-1", Some("acct-b"));
        let live = held.as_ref().ok().and_then(Option::as_ref).map(|found| found.path().to_path_buf()).unwrap_or_default();
        let place = theirs.display().to_string();
        let stale = Ticket { pid: std::process::id(), run: "r-2".to_owned(), account: "acct-c".to_owned(), place, ts_ms: 0 };
        let doomed = put(&host_runs_dir(&mine), &stale);
        let gone = doomed.as_ref().map(|found| found.path().to_path_buf()).unwrap_or_default();
        assert!(live.is_file() && gone.is_file(), "札 2 枚を置ける: {held:?} {doomed:?}");
        assert_eq!(elsewhere(&mine), BTreeSet::from(["acct-b".to_owned()]), "ほかの置き場の生きた札の口座");
        assert!(!gone.exists(), "死んだ札は消す");
        assert!(live.is_file(), "生きた札は残す");
        assert_eq!(elsewhere(&theirs), BTreeSet::new(), "自分の置き場の札は数えない");
        assert!(live.is_file(), "自分の置き場の札も残す");
    }
}
