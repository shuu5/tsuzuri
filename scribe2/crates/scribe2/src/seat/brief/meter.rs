//! 要の写しを名乗った席の SessionStart の出力ごとの記録と役割の行の上限の検出（tsuzuri の判断の記録 ADR-38 の決定 (4)・撤退の条件 (5)・
//! 行 v-brief-meter）: 写しの行と役割の行を別の記録に書き、役割の行の byte（tsuzuri の規則の行 R-41 が数える量）が要の写しの隣の
//! 上限の file（[`CAP_FILE`]・tsuzuri の tz derive が R-41 の値から導く・行 t-seatcap）の 2 行目の数を越えた周（ちょうどは越えない）と
//! file を読めない周を、記録と stderr の 1 行で名指す（出力は止めない・読めない周を越えないに倒さない）。1 回の出力の字の数は SessionStart の終わりの記録 1 行（[`total`]）が持つ。数は記録の `what` の字に載せ、記録の
//! schema は上げない。key の無い席は今の記録（指示文の 1 件）のまま。越えと読めないの記録の読み手は doctor の 1 行（[`doctor_line`]）で、
//! 席ごとに最新の名乗った SessionStart の測りを読む（同じ記録の決定 (4) の検出の event を記録の行と doctor の行で満たす）。

use super::copy::{Copy, NEXT_STEP};
use crate::fleet::json_tree::{self, Tree};
use crate::name::NAME;
use crate::pipe::declaration::SeatConstitution;
use std::collections::BTreeMap;
use std::path::Path;

/// 役割の行の byte の上限の file の名（宣言が名乗る要の写しと同じ dir・頭の 1 行と 10 進の数の 1 行・器は 2 行目だけを読む）。
pub const CAP_FILE: &str = "role-max-bytes.txt";
/// 宣言を読めず上限の file の path が分からない周の path の字。
const UNKNOWN_PATH: &str = "-";
/// 記録の `what`（写しを出した周）。
pub const WHAT_COPY: &str = "session-start-constitution";
/// 記録の `what`（写しの代わりに断りの 1 行を出した周）。
pub const WHAT_COPY_REFUSED: &str = "session-start-constitution-refused";
/// 記録の `what` の頭（1 回の出力の全部・後ろに ` chars=<字の数>`）。
pub const WHAT_TOTAL: &str = "session-start-total";
/// 記録の `what` の頭（役割の行が上限を越えた周・後ろに ` bytes=<n> cap=<m>`）。
pub const WHAT_OVER: &str = "seat-role-over";
/// 記録の `what` の頭（上限の file を読めない周・後ろに ` path=<file の repo 相対の path>`）。
pub const WHAT_UNMEASURED: &str = "seat-role-unmeasured";
/// doctor の行の鍵（`seat-role=<ok|over|unmeasured|unreadable>`）。
const DOCTOR_KEY: &str = "seat-role";

/// 記録 1 件の中身（`what` と `bytes`・残りの欄は hook の記録の組み手が埋める）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// 何を出したか（数を字に載せる）。
    pub what: String,
    /// byte 数（越えと読めないの記録は 0）。
    pub bytes: u64,
}

/// 役割の行の上限の読み（上限の file の path と、読めた数）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cap {
    /// 上限の file の repo 相対の path（宣言を読めない周は字 `-`）。
    pub path: String,
    /// file の 2 行目の数（file が無い・2 行でない・2 行目が 10 進の数字だけでない周は `None`＝越えないに倒さない）。
    pub value: Option<u64>,
}

/// 出した行の byte（出力層が行ごとに足す改行 1 byte を含む）。
pub fn printed(lines: &[String]) -> u64 {
    lines.iter().map(|line| u64::try_from(line.len()).unwrap_or(u64::MAX).saturating_add(1)).fold(0, u64::saturating_add)
}

/// 役割の行の上限（`root` は anchor・宣言が名乗る要の写しの隣の [`CAP_FILE`] の 2 行目・頭の行は読まない）。
pub fn cap_of(root: &Path, declared: &SeatConstitution) -> Cap {
    let SeatConstitution::Declared(copy) = declared else {
        return Cap { path: UNKNOWN_PATH.to_owned(), value: None };
    };
    let path = Path::new(copy).with_file_name(CAP_FILE).display().to_string();
    let value = std::fs::read_to_string(root.join(&path)).ok().as_deref().and_then(value_of);
    Cap { path, value }
}

/// 上限の file の字の数（ちょうど 2 行で、2 行目が 10 進の数字だけの周だけ）。
fn value_of(text: &str) -> Option<u64> {
    let mut lines = text.lines();
    let (_head, number, rest) = (lines.next()?, lines.next()?, lines.next());
    if rest.is_some() || !number.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    number.parse().ok()
}

/// 名乗った席の brief の記録（写し → 役割の行 → 越えか読めないの順）と stderr の行。key の無い席は `None`（今の 1 件のまま）。
/// `brief` は穴を埋めた雛形の 12 行、`role_what` は役割の行の記録の `what`。
pub fn brief_entries(copy: &Copy, brief: &str, cap: &Cap, role_what: &str) -> Option<(Vec<Entry>, Vec<String>)> {
    let (head, role) = copy.parts(brief)?;
    let what = if matches!(copy, Copy::Refused(_)) { WHAT_COPY_REFUSED } else { WHAT_COPY };
    let bytes = printed(&role);
    let mut entries = vec![Entry { what: what.to_owned(), bytes: printed(&head) }, Entry { what: role_what.to_owned(), bytes }];
    let mut alarms = Vec::new();
    let path = &cap.path;
    match cap.value {
        Some(cap) if bytes > cap => {
            entries.push(Entry { what: format!("{WHAT_OVER} bytes={bytes} cap={cap}"), bytes: 0 });
            alarms.push(format!("{NAME}: 役割の行が上限を越えた bytes={bytes} cap={cap} path={path}"));
        }
        Some(_) => {}
        None => {
            entries.push(Entry { what: format!("{WHAT_UNMEASURED} path={path}"), bytes: 0 });
            alarms.push(format!("{NAME}: 役割の行の上限を読めない path={path}（次の 1 手: {NEXT_STEP}・越えないに倒さない）"));
        }
    }
    Some((entries, alarms))
}

/// 1 回の出力の全部の記録（`what` は Unicode の字の数・`bytes` は byte・どちらも行ごとの改行を含む）。
pub fn total(lines: &[String]) -> Entry {
    let chars = lines.iter().map(|line| line.chars().count().saturating_add(1)).fold(0_usize, usize::saturating_add);
    Entry { what: format!("{WHAT_TOTAL} chars={chars}"), bytes: printed(lines) }
}

/// 席 1 つの最新の名乗った SessionStart の測り（越えと読めないは記録の `what` の頭の後ろの字を持つ）。
enum Last {
    /// 上限の内（写しの記録の後に越えも読めないも無い）。
    Within,
    /// 越えた。
    Over(String),
    /// 上限の行を読めない。
    Unmeasured(String),
}

/// doctor の 1 行（記録の置き場の `inject.jsonl` から・file が無い周と名乗った席の記録が 1 件も無い周は `None`＝出さない・読めない
/// file は `seat-role=unreadable`）。
pub fn doctor_line(state_dir: &Path) -> Option<String> {
    match std::fs::read_to_string(crate::hook::inject_path(state_dir)) {
        Ok(text) => doctor_word(&text),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(_) => Some(format!("{DOCTOR_KEY}=unreadable")),
    }
}

/// 記録 1 行の席と測り（読めない行・席の無い行・測りでない記録は `None`）。
fn measure_of(line: &str) -> Option<(String, Last)> {
    let tree = json_tree::parse(line).ok()?;
    let (what, seat) = (tree.get("what").and_then(Tree::as_str)?, tree.get("seat").and_then(Tree::as_str)?);
    let rest = |head: &str| what.strip_prefix(head).and_then(|rest| rest.strip_prefix(' ')).map(str::to_owned);
    let found = if what == WHAT_COPY || what == WHAT_COPY_REFUSED {
        Last::Within
    } else if let Some(found) = rest(WHAT_OVER) {
        Last::Over(found)
    } else {
        Last::Unmeasured(rest(WHAT_UNMEASURED)?)
    };
    Some((seat.to_owned(), found))
}

/// 記録の字から doctor の 1 行を組む（pure）: 席ごとに、写しの記録（[`WHAT_COPY`] か [`WHAT_COPY_REFUSED`]）が名乗った周の始まりで、
/// その後の越えか読めないの記録が周の結果（後の周が前の周を上書きする＝直った席の古い越えを出さない）。語は越え → 読めない → ok の
/// 順に強く、越えか読めないの周は数と、最後に記録された越えか読めないの席とその字を名指す。読めない行と席の無い行は飛ばす。
pub fn doctor_word(text: &str) -> Option<String> {
    let mut last: BTreeMap<String, (usize, Last)> = BTreeMap::new();
    for (at, (seat, found)) in text.lines().enumerate().filter_map(|(at, line)| measure_of(line).map(|pair| (at, pair))) {
        last.insert(seat, (at, found));
    }
    if last.is_empty() {
        return None;
    }
    let over = last.values().filter(|(_, found)| matches!(found, Last::Over(_))).count();
    let unmeasured = last.values().filter(|(_, found)| matches!(found, Last::Unmeasured(_))).count();
    let latest = last
        .iter()
        .filter_map(|(seat, (at, found))| match found {
            Last::Over(rest) | Last::Unmeasured(rest) => Some((*at, seat, rest)),
            Last::Within => None,
        })
        .max_by_key(|(at, _, _)| *at);
    let word = if over > 0 { "over" } else if unmeasured > 0 { "unmeasured" } else { "ok" };
    Some(match latest {
        None => format!("{DOCTOR_KEY}={word} seats={}", last.len()),
        Some((_, seat, rest)) => format!("{DOCTOR_KEY}={word} seats={} over={over} unmeasured={unmeasured} last={seat} {rest}", last.len()),
    })
}

#[cfg(test)]
mod tests {
    use super::{brief_entries, cap_of, doctor_line, doctor_word, printed, total, Cap, Entry, CAP_FILE, WHAT_COPY, WHAT_COPY_REFUSED};
    use crate::pipe::declaration::SeatConstitution;
    use crate::seat::brief::copy::{Copy, NEXT_STEP};
    use crate::seat::brief::{role_lines, template};
    use crate::seat::role::Role;

    /// 写しの見本（多字節・末尾の改行）。
    const SAMPLE: &str = "生成物・手で直さない v9.9\n順位 甲  乙\n";
    /// 役割の行の記録の `what`（hook の WHAT_BRIEF と同じ字）。
    const ROLE: &str = "session-start-brief";
    /// 宣言が名乗る要の写しの path と、その隣の上限の file の path。
    const COPY: &str = "contracts/seat/brief.txt";
    const PATH: &str = "contracts/seat/role-max-bytes.txt";

    /// 雛形の役割の 7 行の byte（行ごとの改行を含む）。
    fn role_bytes() -> u64 {
        printed(&role_lines(template(Role::Orchestrator)).map(str::to_owned).collect::<Vec<String>>())
    }

    /// 記録 1 件。
    fn entry(what: &str, bytes: u64) -> Entry {
        Entry { what: what.to_owned(), bytes }
    }

    /// 上限の読み（path は見本の上限の file）。
    fn cap(value: Option<u64>) -> Cap {
        Cap { path: PATH.to_owned(), value }
    }

    /// 名乗った写しは写しの記録（写しの byte）と役割の行の記録（7 行の byte）の 2 件に分かれ、上限の内なら stderr は 0 行。読めない周は
    /// 写しの記録の what が refused で byte は断りの 1 行。
    #[test]
    fn vbmeter_keyed_seat_records_the_copy_and_the_role_lines_apart() {
        let brief = template(Role::Orchestrator);
        let read = brief_entries(&Copy::Read(SAMPLE.to_owned()), brief, &cap(Some(u64::MAX)), ROLE);
        let want = vec![entry(WHAT_COPY, SAMPLE.len() as u64), entry(ROLE, role_bytes())];
        assert_eq!(read, Some((want, Vec::new())), "写しと役割の行の 2 件");
        let line = "断りの 1 行".to_owned();
        let refused = brief_entries(&Copy::Refused(line.clone()), brief, &cap(Some(u64::MAX)), ROLE);
        let want = vec![entry(WHAT_COPY_REFUSED, line.len() as u64 + 1), entry(ROLE, role_bytes())];
        assert_eq!(refused, Some((want, Vec::new())), "断りの 1 行と役割の行の 2 件");
    }

    /// key の無い席は記録を分けない（指示文の 1 件のまま・`None`）。
    #[test]
    fn vbmeter_absent_key_keeps_the_single_brief_record() {
        assert_eq!(brief_entries(&Copy::Absent, template(Role::Orchestrator), &cap(Some(1)), ROLE), None);
    }

    /// 役割の行の byte が上限ちょうどは越えず（3 件目なし）、1 byte 下の上限は越えの記録（bytes 0）と stderr の 1 行が byte と上限と
    /// 上限の file の path を名指す。
    #[test]
    fn vbmeter_role_lines_over_the_cap_are_named_and_exactly_the_cap_is_not() {
        let brief = template(Role::Orchestrator);
        let copy = Copy::Read(SAMPLE.to_owned());
        let bytes = role_bytes();
        let exact = brief_entries(&copy, brief, &cap(Some(bytes)), ROLE).unwrap_or_default();
        assert_eq!((exact.0.len(), exact.1.len()), (2, 0), "ちょうどは越えない: {exact:?}");
        let (entries, alarms) = brief_entries(&copy, brief, &cap(Some(bytes - 1)), ROLE).unwrap_or_default();
        assert_eq!(entries.get(2), Some(&entry(&format!("seat-role-over bytes={bytes} cap={}", bytes - 1), 0)), "{entries:?}");
        assert_eq!(alarms.len(), 1, "{alarms:?}");
        let alarm = alarms.first().cloned().unwrap_or_default();
        assert!(alarm.contains(&format!("bytes={bytes} cap={} path={PATH}", bytes - 1)), "{alarm}");
    }

    /// 上限は宣言が名乗る写しの隣の file の 2 行目で、頭の行は読まない。file が無い・2 行目が数でない・行の形が違う（1 行・3 行）周と
    /// 宣言を読めない周は読めない（`None`）で、読めないの記録と stderr の 1 行が file の path と次の 1 手を名指し、越えないに倒さない。
    #[test]
    fn vbmeter_unreadable_cap_file_is_unmeasured_not_within() {
        let root = crate::pipe::fixture::scratch("vbmeter-cap");
        let declared = SeatConstitution::Declared(COPY.to_owned());
        let _ = std::fs::create_dir_all(root.join("contracts/seat"));
        let read = |text: Option<&str>| {
            let _ = std::fs::remove_file(root.join(PATH));
            if let Some(text) = text {
                let _ = std::fs::write(root.join(PATH), text);
            }
            cap_of(&root, &declared)
        };
        assert_eq!(read(Some("頭は読まない 9\n2000\n")), cap(Some(2000)), "2 行目の数");
        assert_eq!(read(None), cap(None), "file が無い");
        assert_eq!(read(Some("頭\n2,000\n")), cap(None), "2 行目が数でない");
        assert_eq!(read(Some("2000\n")), cap(None), "1 行");
        assert_eq!(read(Some("頭\n2000\n3\n")), cap(None), "3 行");
        let unknown = cap_of(&root, &SeatConstitution::Unreadable);
        assert_eq!((unknown.path.as_str(), unknown.value), ("-", None), "宣言を読めない");
        assert!(PATH.ends_with(CAP_FILE), "写しの隣の名");
        let (entries, alarms) = brief_entries(&Copy::Read(SAMPLE.to_owned()), template(Role::Orchestrator), &cap(None), ROLE).unwrap_or_default();
        assert_eq!(entries.get(2), Some(&entry(&format!("seat-role-unmeasured path={PATH}"), 0)), "{entries:?}");
        assert!(alarms.len() == 1 && alarms.iter().all(|line| line.contains(&format!("path={PATH}")) && line.contains(NEXT_STEP)), "{alarms:?}");
    }

    /// 1 回の出力の記録は Unicode の字の数（改行を含む）を what に、byte を bytes に持つ。
    #[test]
    fn vbmeter_total_counts_chars_and_bytes_with_newlines() {
        let lines = vec!["甲乙".to_owned(), "ab".to_owned()];
        assert_eq!(total(&lines), entry("session-start-total chars=6", 10));
    }

    /// 記録 1 行（`what` と `seat`・ほかの欄は hook の記録と同じ形）。
    fn rec(what: &str, seat: &str) -> String {
        format!("{{\"schema\":1,\"who\":\"hook:session-start\",\"what\":\"{what}\",\"when\":\"SessionStart\",\"bytes\":0,\"tokens\":null,\"wall_ms\":0,\"seat\":\"{seat}\",\"ts\":1}}\n")
    }

    /// doctor の行は席ごとに最新の名乗った SessionStart の測りを読む: 名乗った記録が無ければ出さず、全部の席が上限の内なら ok と席の数、
    /// 越えか読めないの席が在れば語（越えが強い）と数と最後に記録された席とその字。同じ席の後の周の上限の内は前の越えを消し、ほかの
    /// 席の周は消さない。file が無い周は出さず、読めない file は unreadable。
    #[test]
    fn vbmeter_doctor_line_reads_the_latest_measure_of_each_seat() {
        let (copy, refused) = (rec(WHAT_COPY, "a"), rec(WHAT_COPY_REFUSED, "b"));
        let over = rec("seat-role-over bytes=2101 cap=2000", "a");
        let unmeasured = rec(&format!("seat-role-unmeasured path={PATH}"), "b");
        assert_eq!(doctor_word(&rec("session-start-brief", "a")), None, "名乗った記録が無ければ出さない");
        assert_eq!(doctor_word(&format!("{copy}{refused}")), Some("seat-role=ok seats=2".to_owned()), "全部の席が上限の内");
        let want = format!("seat-role=over seats=2 over=1 unmeasured=1 last=b path={PATH}");
        assert_eq!(doctor_word(&format!("{copy}{over}{refused}{unmeasured}")), Some(want), "越えが強く最後は b");
        let want = format!("seat-role=unmeasured seats=2 over=0 unmeasured=1 last=b path={PATH}");
        assert_eq!(doctor_word(&format!("{copy}{refused}{unmeasured}")), Some(want), "読めないだけ");
        assert_eq!(doctor_word(&format!("{copy}{over}{copy}")).as_deref(), Some("seat-role=ok seats=1"), "後の周が前の越えを消す");
        let want = "seat-role=over seats=2 over=1 unmeasured=0 last=a bytes=2101 cap=2000";
        assert_eq!(doctor_word(&format!("{copy}{over}{refused}")).as_deref(), Some(want), "ほかの席の周は消さない");
        assert_eq!(doctor_word(&format!("{copy}{over}not json\n")).as_deref(), Some("seat-role=over seats=1 over=1 unmeasured=0 last=a bytes=2101 cap=2000"));
        let state = crate::pipe::fixture::scratch("vbmeter-doctor");
        assert_eq!(doctor_line(&state), None, "file が無ければ出さない");
        let _ = std::fs::create_dir_all(crate::hook::inject_path(&state));
        assert_eq!(doctor_line(&state).as_deref(), Some("seat-role=unreadable"), "読めない file");
    }
}
