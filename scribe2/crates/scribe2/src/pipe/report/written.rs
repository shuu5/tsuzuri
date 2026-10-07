//! 便 1 本の装置への正味の書きを段ごとに読む口。

use crate::fleet::json_lite::{self, Value};
use crate::fleet::{CostSource, Event, EventKind};
use crate::pipe::confine::io::UNMEASURED;
use crate::pipe::{run_dir, verify_log_path};
use std::io::{self, ErrorKind};
use std::path::Path;

/// 段の記録が無い字（record の file も、runner の消費の event と起こしも無い段）。
pub const ABSENT: &str = "-";

/// 囲いの外で数えない書きの名（行の末の語 `uncounted=` の値・作業木の作り・着地の git・審査役の claude の囲い）。
pub const UNCOUNTED: &str = "worktree,land-git,reviewer-claude";

/// 終わりの門の record の file の名（`pipe::spawn` の書き手と同じ字・run dir の直下）。
const END_GATE_FILE: &str = "end-gate.jsonl";

/// 着地の確かめの record の file の名（`pipe::land` の書き手と同じ字・run dir の直下）。
const MAIN_FILE: &str = "verify-main.jsonl";

/// 消費の event の detail の書きの語の頭（`confine::io::detail` の字）。
const WRITE_WORD: &str = "write:";

/// 段の値（閉じた 3 値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// 測った正味の byte の和。
    Bytes(u64),
    /// 測れない（数でない語・読めない record・起こした runner より少ない消費の event・和の桁あふれ）。
    Unmeasured,
    /// 段の記録が無い。
    Absent,
}

impl Part {
    /// 行の値の字。
    pub fn word(self) -> String {
        match self {
            Self::Bytes(bytes) => bytes.to_string(),
            Self::Unmeasured => UNMEASURED.to_owned(),
            Self::Absent => ABSENT.to_owned(),
        }
    }
}

/// 便 1 本の 4 段の書き。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Written {
    /// runner の囲い（消費の event の語 `write:` の和）。
    pub runner: Part,
    /// gate の全部の周と列の候補（`verify.jsonl`）。
    pub verify: Part,
    /// 終わりの門（`end-gate.jsonl`）。
    pub end_gate: Part,
    /// 着地の確かめと着地後の検出（`verify-main.jsonl`）。
    pub main: Part,
}

impl Written {
    /// 4 段の和: どれかが測れない周は測れない・4 段とも記録が無い周は無い・ほかは記録の無い段を 0 とした和（桁あふれは測れない）。
    pub fn total(self) -> Part {
        let parts = [self.runner, self.verify, self.end_gate, self.main];
        if parts.contains(&Part::Unmeasured) {
            return Part::Unmeasured;
        }
        let mut sum: Option<u64> = None;
        for part in parts {
            if let Part::Bytes(bytes) = part {
                match sum.unwrap_or(0).checked_add(bytes) {
                    Some(found) => sum = Some(found),
                    None => return Part::Unmeasured,
                }
            }
        }
        sum.map_or(Part::Absent, Part::Bytes)
    }

    /// `pipe show --write` の 1 行（`write: runner=<v> verify=<v> end-gate=<v> main=<v> total=<v> uncounted=<名>`）。
    pub fn line(self) -> String {
        format!(
            "write: runner={} verify={} end-gate={} main={} total={} uncounted={UNCOUNTED}",
            self.runner.word(),
            self.verify.word(),
            self.end_gate.word(),
            self.main.word(),
            self.total().word()
        )
    }
}

/// 置き場の便 1 本の書きを読む（`events` は呼び手が読んだ置き場の全行・record の file は run dir から読む）。
pub fn written_of(state_dir: &Path, events: &[Event], id: &str) -> Written {
    let dir = run_dir(state_dir, id);
    Written {
        runner: runner_part(events, id),
        verify: record_part(std::fs::read_to_string(verify_log_path(state_dir, id))),
        end_gate: record_part(std::fs::read_to_string(dir.join(END_GATE_FILE))),
        main: record_part(std::fs::read_to_string(dir.join(MAIN_FILE))),
    }
}

/// runner の段: 便の source `runner` の消費の event の detail の語 `write:` の和。語が無いか数でない event が 1 つでも在る周と、
/// 便の `SeatSpawned` の数より消費の event が少ない周（runner が走り中・止めた・6 値の揃わない終わり）は測れない。消費の event も
/// `SeatSpawned` も無い便は記録が無い。
fn runner_part(events: &[Event], id: &str) -> Part {
    let own: Vec<&Event> = events.iter().filter(|event| event.run == id).collect();
    let spawned = own.iter().filter(|event| event.kind == EventKind::SeatSpawned).count();
    let costs: Vec<Option<u64>> = own
        .iter()
        .filter(|event| event.kind == EventKind::RunCost)
        .filter(|event| event.cost.is_some_and(|cost| cost.source == CostSource::Runner))
        .map(|event| write_of(event.detail.as_deref()))
        .collect();
    if costs.is_empty() && spawned == 0 {
        return Part::Absent;
    }
    if costs.len() < spawned {
        return Part::Unmeasured;
    }
    costs
        .into_iter()
        .try_fold(0_u64, |sum, part| sum.checked_add(part?))
        .map_or(Part::Unmeasured, Part::Bytes)
}

/// detail の語 `write:<10 進>` の byte（語が無い・数でない・桁あふれは `None`）。
fn write_of(detail: Option<&str>) -> Option<u64> {
    detail?.split_whitespace().find_map(|word| word.strip_prefix(WRITE_WORD)).and_then(digits)
}

/// ASCII の 10 進だけの字の数（空・符号・ほかの字・桁あふれは `None`）。
fn digits(text: &str) -> Option<u64> {
    (!text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())).then(|| text.parse().ok()).flatten()
}

/// record の段: file の無い周は記録が無い・読めない周は測れない・ほかは撃った行（`secs` を持つ行）の `write_bytes` の和で、
/// 撃った行の `write_bytes` が欠けるか数でない周と、object として読めない行の在る周は測れない。撃った行の無い file は 0。
fn record_part(text: io::Result<String>) -> Part {
    let text = match text {
        Ok(found) => found,
        Err(err) if err.kind() == ErrorKind::NotFound => return Part::Absent,
        Err(_) => return Part::Unmeasured,
    };
    let mut sum = 0_u64;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(pairs) = json_lite::parse_object(line) else {
            return Part::Unmeasured;
        };
        if !pairs.iter().any(|(key, _)| key == "secs") {
            continue;
        }
        let bytes = pairs.iter().find(|(key, _)| key == "write_bytes").and_then(|(_, value)| bytes_of(value));
        match bytes.and_then(|found| sum.checked_add(found)) {
            Some(found) => sum = found,
            None => return Part::Unmeasured,
        }
    }
    Part::Bytes(sum)
}

/// record の `write_bytes` の値の byte（10 進の字か数・ほかは `None`）。
fn bytes_of(value: &Value) -> Option<u64> {
    match value {
        Value::Str(text) => digits(text),
        Value::Num(found) => Some(*found),
        Value::Bool(_) | Value::Null => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{record_part, runner_part, written_of, Part, Written, ABSENT, UNCOUNTED};
    use crate::fleet::{Cost, CostSource, Event, EventKind, Usage};
    use crate::pipe::confine::io::UNMEASURED;
    use crate::pipe::fixture::{event, scratch};
    use crate::pipe::run_dir;
    use std::io::{Error, ErrorKind};

    /// 歯の便 id。
    const RUN: &str = "r-vrunw";

    /// ほかの便の id（同じ置き場に在って数えない）。
    const OTHER: &str = "r-other";

    /// 消費の event（source と detail は呼び手が選ぶ・6 値は決まった数）。
    fn cost(run: &str, source: CostSource, detail: Option<&str>) -> Event {
        let usage = Usage { input: 1, output: 2, cache_read: 3, cache_create: 4, turns: 5, wall_ms: 6 };
        Event { cost: Some(Cost { source, usage }), ..event(run, EventKind::RunCost, None, None, detail) }
    }

    /// runner の起こしの event。
    fn spawned(run: &str) -> Event {
        event(run, EventKind::SeatSpawned, None, Some(run), None)
    }

    /// 正しい event の列: runner を 2 回起こし、runner の消費が 2 件（語 write: 100 と 20）、lens の語 write: 9000 と review の
    /// 語の無い消費、ほかの便の runner の起こしと消費（語 write: 7）。
    fn good_events() -> Vec<Event> {
        vec![
            spawned(RUN),
            cost(RUN, CostSource::Runner, Some("write:100 build:abc model:m")),
            cost(RUN, CostSource::Lens, Some("write:9000 build:abc")),
            cost(RUN, CostSource::Review, Some("build:abc")),
            spawned(RUN),
            cost(RUN, CostSource::Runner, Some("write:20 build:abc")),
            spawned(OTHER),
            cost(OTHER, CostSource::Runner, Some("write:7")),
        ]
    }

    /// 正しい verify の record: 撃たない write-set の行（`secs` なし・write_bytes は字 unmeasured）・撃った行 2 本（字 1000 と
    /// 列の候補の印 train を持つ字 200）・撃たなかった段の行（`secs` なし）。
    const GOOD_VERIFY: &str = concat!(
        r#"{"schema":1,"n":1,"rc":0,"cmd":"write-set","write_bytes":"unmeasured","kind":"write-set"}"#,
        "\n",
        r#"{"schema":1,"n":2,"rc":0,"cmd":"c","write_bytes":"1000","kind":"verify","secs":3}"#,
        "\n",
        r#"{"schema":1,"n":3,"rc":0,"cmd":"t","write_bytes":"200","kind":"verify","train":"2","secs":0}"#,
        "\n",
        r#"{"schema":1,"n":4,"skipped":"verify","reason":"red-before"}"#,
        "\n",
    );

    /// 撃った行の 1 本の正しい record（`write_bytes` の字は呼び手が選ぶ）。
    fn fired(bytes: &str) -> String {
        format!(r#"{{"schema":1,"n":2,"rc":0,"cmd":"c","write_bytes":"{bytes}","kind":"verify","secs":3}}"#)
    }

    /// 和は runner の消費の語の和（ほかの便・lens・review を数えない）と、撃った行の write_bytes の和。
    #[test]
    fn vrunw_sums_runner_words_and_fired_records_by_stage() {
        assert_eq!(runner_part(&good_events(), RUN), Part::Bytes(120), "runner は 100 + 20（lens の 9000・ほかの便の 7 を足さない）");
        assert_eq!(runner_part(&good_events(), OTHER), Part::Bytes(7), "便ごとに分ける");
        assert_eq!(record_part(Ok(GOOD_VERIFY.to_owned())), Part::Bytes(1200), "撃った 2 行（列の候補を含む）だけ");
        let end_gate = format!("{}\n{}\n", fired("33"), r#"{"end_gate":1,"result":"green"}"#);
        assert_eq!(record_part(Ok(end_gate)), Part::Bytes(33), "門の要約の行は数えない");
        let landed = format!("{}\n{}\n", fired("4000"), fired("5").replace(r#""kind":"verify""#, r#""kind":"detection","landed":"s""#));
        assert_eq!(record_part(Ok(landed)), Part::Bytes(4005), "着地後の検出の撃った行も足す");
        let written = Written { runner: Part::Bytes(120), verify: Part::Bytes(1200), end_gate: Part::Bytes(33), main: Part::Bytes(4005) };
        assert_eq!(written.total(), Part::Bytes(5358), "和は 4 段の和");
    }

    /// runner の段は、正しい列から 1 つだけ崩すと測れないか記録が無いに替わる。
    #[test]
    fn vrunw_runner_is_unmeasured_without_a_number_for_every_spawn() {
        let with = |at: usize, replaced: Event| {
            let mut events = good_events();
            if let Some(slot) = events.get_mut(at) {
                *slot = replaced;
            }
            runner_part(&events, RUN)
        };
        assert_eq!(with(1, cost(RUN, CostSource::Runner, Some("write:unmeasured build:abc"))), Part::Unmeasured, "語が字 unmeasured");
        assert_eq!(with(1, cost(RUN, CostSource::Runner, Some("build:abc"))), Part::Unmeasured, "語 write: が無い");
        assert_eq!(with(1, cost(RUN, CostSource::Runner, None)), Part::Unmeasured, "detail が無い");
        assert_eq!(with(1, cost(RUN, CostSource::Runner, Some("write:1x0"))), Part::Unmeasured, "数でない");
        assert_eq!(with(1, cost(RUN, CostSource::Runner, Some("rewrite:100"))), Part::Unmeasured, "語の頭が違う");
        assert_eq!(with(5, spawned(RUN)), Part::Unmeasured, "起こしが 3 回で消費が 1 件");
        assert_eq!(with(5, cost(RUN, CostSource::Lens, Some("write:20"))), Part::Unmeasured, "lens の消費は runner の数に入らない");
        let mut events = good_events();
        events.push(spawned(RUN));
        assert_eq!(runner_part(&events, RUN), Part::Unmeasured, "起こしが消費より多い（走り中）");
        let overflow = cost(RUN, CostSource::Runner, Some(&format!("write:{}", u64::MAX)));
        assert_eq!(with(5, overflow), Part::Unmeasured, "和の桁あふれ");
        let lone: Vec<Event> = good_events().into_iter().filter(|found| found.run == OTHER).collect();
        assert_eq!(runner_part(&lone, RUN), Part::Absent, "起こしも消費も無い便は記録が無い");
        let costs_only: Vec<Event> = good_events().into_iter().filter(|found| found.kind != EventKind::SeatSpawned).collect();
        assert_eq!(runner_part(&costs_only, RUN), Part::Bytes(120), "起こしの記帳の無い便も消費の語の和");
    }

    /// record の段は、正しい record から 1 つだけ崩すと測れないか記録が無いか 0 に替わる。
    #[test]
    fn vrunw_record_stage_is_unmeasured_or_absent_by_its_file() {
        let swap = |from: &str, to: &str| {
            assert_eq!(GOOD_VERIFY.matches(from).count(), 1, "崩す字は 1 つ: {from}");
            record_part(Ok(GOOD_VERIFY.replacen(from, to, 1)))
        };
        assert_eq!(swap(r#""write_bytes":"1000""#, r#""write_bytes":"unmeasured""#), Part::Unmeasured, "撃った行の字 unmeasured");
        assert_eq!(swap(r#""write_bytes":"1000","#, ""), Part::Unmeasured, "撃った行の write_bytes の欠け");
        assert_eq!(swap(r#""write_bytes":"1000""#, r#""write_bytes":"-1000""#), Part::Unmeasured, "符号つき");
        assert_eq!(swap(r#""write_bytes":"1000""#, r#""write_bytes":1000"#), Part::Bytes(1200), "数の値も 10 進と同じ");
        assert_eq!(swap(r#""write_bytes":"1000""#, r#""write_bytes":true"#), Part::Unmeasured, "真偽");
        assert_eq!(swap(r#""kind":"verify","secs":3}"#, r#""kind":"verify","secs":3"#), Part::Unmeasured, "object として読めない行");
        assert_eq!(swap(r#","secs":3}"#, "}"), Part::Bytes(200), "secs の無い行は撃っていない");
        let overflow = format!(r#""write_bytes":"{}""#, u64::MAX);
        assert_eq!(swap(r#""write_bytes":"1000""#, &overflow), Part::Unmeasured, "和の桁あふれ");
        let unfired = GOOD_VERIFY.lines().filter(|line| !line.contains("secs")).collect::<Vec<_>>().join("\n");
        assert_eq!(record_part(Ok(unfired)), Part::Bytes(0), "撃った行の無い file は 0");
        assert_eq!(record_part(Err(Error::from(ErrorKind::NotFound))), Part::Absent, "file が無い");
        assert_eq!(record_part(Err(Error::from(ErrorKind::PermissionDenied))), Part::Unmeasured, "file を読めない");
    }

    /// 和と行の字: 測れない段が 1 つでも在れば和も測れない・4 段とも記録が無い周は字 -・記録の無い段は 0 として足す。
    #[test]
    fn vrunw_total_and_line_keep_unmeasured_and_absent_apart() {
        let base = Written { runner: Part::Bytes(1), verify: Part::Bytes(20), end_gate: Part::Absent, main: Part::Bytes(300) };
        assert_eq!(base.total(), Part::Bytes(321), "記録の無い段は 0");
        assert_eq!(
            base.line(),
            format!("write: runner=1 verify=20 end-gate={ABSENT} main=300 total=321 uncounted={UNCOUNTED}"),
            "行の語の順と字"
        );
        for (at, written) in [
            (0, Written { runner: Part::Unmeasured, ..base }),
            (1, Written { verify: Part::Unmeasured, ..base }),
            (2, Written { end_gate: Part::Unmeasured, ..base }),
            (3, Written { main: Part::Unmeasured, ..base }),
        ] {
            assert_eq!(written.total(), Part::Unmeasured, "段 {at} が測れない");
            assert!(written.line().contains(&format!("total={UNMEASURED} ")), "{}", written.line());
        }
        let none = Written { runner: Part::Absent, verify: Part::Absent, end_gate: Part::Absent, main: Part::Absent };
        assert_eq!(none.total(), Part::Absent, "4 段とも記録が無い");
        assert_eq!(Written { main: Part::Bytes(0), ..none }.total(), Part::Bytes(0), "測った 0 は記録が無いと違う");
        assert_eq!(Written { runner: Part::Bytes(u64::MAX), ..base }.total(), Part::Unmeasured, "和の桁あふれ");
        assert_eq!(UNCOUNTED, "worktree,land-git,reviewer-claude", "数えない物の名");
    }

    /// 置き場から読む口: run dir の 3 つの file を段に当て、file の無い段は記録が無い・dir の在る名は読めない。
    #[test]
    fn vrunw_reads_the_three_records_from_the_run_dir() {
        let state = scratch("vrunw-reads");
        let dir = run_dir(&state, RUN);
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("verify.jsonl"), GOOD_VERIFY);
        let _ = std::fs::write(dir.join("end-gate.jsonl"), format!("{}\n", fired("33")));
        let found = written_of(&state, &good_events(), RUN);
        let expected = Written { runner: Part::Bytes(120), verify: Part::Bytes(1200), end_gate: Part::Bytes(33), main: Part::Absent };
        assert_eq!(found, expected, "verify と end-gate は file から・main は file が無い");
        let _ = std::fs::create_dir_all(dir.join("verify-main.jsonl"));
        assert_eq!(written_of(&state, &good_events(), RUN).main, Part::Unmeasured, "読めない（dir）");
        assert_eq!(written_of(&state, &good_events(), "r-none").total(), Part::Absent, "記録の無い便");
        let _ = std::fs::remove_dir_all(&state);
    }
}
