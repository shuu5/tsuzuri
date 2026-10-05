//! `pipe show --write` の歯（接頭辞 `vrunw_`・行 v-run-write）: 便 1 本の装置への書きを段ごとと和の 1 行で読む口。
//!
//! 共有の helper は親 module（`tests/e2e/pipe/gate.rs`）と `tests/e2e/pipe.rs` に在り、`use super::*` で使う。置き場の行は
//! 本物の便の行の形を写した字で置き、toy の便を着地まで通す歯は書き手が置いた 3 つの record を読み手が拾うことを測る。

use super::*;
use vessel::fleet::json_lite::{self, Value};

/// 置いた便の id。
const RUN: &str = "r-vrunw-1";

/// 置いた便の event の行: 起こしが 2 回・runner の消費が 2 件（語 write: 100 と 20）・lens の語 write: 9000・review の消費・
/// ほかの便の runner の消費（語 write: 7）。
const EVENTS: &[&str] = &[
    r#"{"schema":1,"ts":"2026-10-04T00:00:00Z","kind":"RunCreated","run":"r-vrunw-1","bead":"b-1","host":"h","actor":"machine","stage":"Intake","detail":"classes:"}"#,
    r#"{"schema":1,"ts":"2026-10-04T00:00:01Z","kind":"SeatSpawned","run":"r-vrunw-1","bead":"b-1","host":"h","actor":"machine","seat":"r-vrunw-1","pid":1}"#,
    r#"{"schema":1,"ts":"2026-10-04T00:00:02Z","kind":"RunCost","run":"r-vrunw-1","bead":"b-1","source":"runner","usage":"in:1,out:2,cache_read:3,cache_create:4","turns":5,"wall_ms":6,"host":"h","actor":"machine","detail":"write:100 build:abc"}"#,
    r#"{"schema":1,"ts":"2026-10-04T00:00:03Z","kind":"RunCost","run":"r-vrunw-1","bead":"b-1","source":"lens","usage":"in:1,out:2,cache_read:3,cache_create:4","turns":5,"wall_ms":6,"host":"h","actor":"machine","detail":"write:9000 build:abc"}"#,
    r#"{"schema":1,"ts":"2026-10-04T00:00:04Z","kind":"RunCost","run":"r-vrunw-1","bead":"b-1","source":"review","usage":"in:1,out:2,cache_read:3,cache_create:4","turns":5,"wall_ms":6,"host":"h","actor":"machine","detail":"build:abc"}"#,
    r#"{"schema":1,"ts":"2026-10-04T00:00:05Z","kind":"SeatSpawned","run":"r-vrunw-1","bead":"b-1","host":"h","actor":"machine","seat":"r-vrunw-1","pid":2}"#,
    r#"{"schema":1,"ts":"2026-10-04T00:00:06Z","kind":"RunCost","run":"r-vrunw-1","bead":"b-1","source":"runner","usage":"in:1,out:2,cache_read:3,cache_create:4","turns":5,"wall_ms":6,"host":"h","actor":"machine","detail":"write:20 build:abc"}"#,
    r#"{"schema":1,"ts":"2026-10-04T00:00:07Z","kind":"RunCreated","run":"r-vrunw-2","bead":"b-2","host":"h","actor":"machine","stage":"Intake","detail":"classes:"}"#,
    r#"{"schema":1,"ts":"2026-10-04T00:00:08Z","kind":"RunCost","run":"r-vrunw-2","bead":"b-2","source":"runner","usage":"in:1,out:2,cache_read:3,cache_create:4","turns":5,"wall_ms":6,"host":"h","actor":"machine","detail":"write:7 build:abc"}"#,
];

/// 置いた便の gate の record: 撃たない write-set の行・撃った行 2 本（字 1000 と列の候補の印を持つ字 200）。
const VERIFY: &str = concat!(
    r#"{"schema":1,"n":1,"rc":0,"cmd":"write-set","write_bytes":"unmeasured","kind":"write-set"}"#,
    "\n",
    r#"{"schema":1,"n":2,"rc":0,"cmd":"c","write_bytes":"1000","kind":"verify","secs":3}"#,
    "\n",
    r#"{"schema":1,"n":3,"rc":0,"cmd":"t","write_bytes":"200","kind":"verify","train":"2","secs":1}"#,
    "\n",
);

/// 置いた便の終わりの門の record: 撃った行 1 本（字 33）と周の要約の行。
const END_GATE: &str = concat!(
    r#"{"schema":1,"n":2,"rc":0,"cmd":"c","write_bytes":"33","kind":"verify","secs":2}"#,
    "\n",
    r#"{"end_gate":1,"result":"green"}"#,
    "\n",
);

/// 置いた便の書きの行（着地の確かめの record は置かない＝字 -）。
const LINE: &str = "write: runner=120 verify=1200 end-gate=33 main=- total=1353 uncounted=worktree,land-git,reviewer-claude";

/// 便 1 本の event と run dir の 2 つの record を置いた置き場（tmp の root の 1 段下）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn placed_state() -> PathBuf {
    let state = tmp().join(STATE_LEAF);
    fs::create_dir_all(state.join("fleet")).expect("fleet dir を作れる");
    fs::write(state.join("fleet").join("events.jsonl"), format!("{}\n", EVENTS.join("\n"))).expect("event log を置ける");
    let dir = run_dir(&state, RUN);
    fs::create_dir_all(&dir).expect("run dir を作れる");
    fs::write(dir.join("verify.jsonl"), VERIFY).expect("gate の record を置ける");
    fs::write(dir.join("end-gate.jsonl"), END_GATE).expect("門の record を置ける");
    state
}

/// `pipe show` を `extra` の旗を足して撃つ。
fn show_with(state: &Path, extra: &[&str]) -> Output {
    let state_arg = state.display().to_string();
    let mut args = vec!["show", "--run", RUN, "--state-dir", &state_arg];
    args.extend_from_slice(extra);
    run_pipe(&args)
}

/// 置き場の event log と run dir の file の byte（名の順）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn place_bytes(state: &Path) -> Vec<(String, Vec<u8>)> {
    let dir = run_dir(state, RUN);
    let mut names: Vec<String> = fs::read_dir(&dir)
        .expect("run dir を読める")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let mut found = vec![("events.jsonl".to_owned(), fs::read(state.join("fleet").join("events.jsonl")).expect("event log を読める"))];
    found.extend(names.into_iter().map(|name| {
        let bytes = fs::read(dir.join(&name)).unwrap_or_default();
        (name, bytes)
    }));
    found
}

/// 旗 `--write` の周は、旗の無い周の行を全部同じ字と順で出した後に書きの 1 行を足し、rc は 0 で stderr は同じ。行の値は
/// runner が runner の消費の語の和（lens・review・ほかの便を数えない）・verify と end-gate が撃った行の和・main は記録が無い字 -。
#[test]
fn vrunw_show_write_adds_the_stage_line_after_the_plain_lines() {
    let state = placed_state();
    let plain = show_with(&state, &[]);
    let with = show_with(&state, &["--write"]);
    assert_eq!(plain.status.code(), Some(i32::from(RC_OK)), "旗の無い周: {}", stderr_of(&plain));
    assert_eq!(with.status.code(), Some(i32::from(RC_OK)), "旗の在る周: {}", stderr_of(&with));
    let plain_lines: Vec<String> = stdout_of(&plain).lines().map(str::to_owned).collect();
    let with_lines: Vec<String> = stdout_of(&with).lines().map(str::to_owned).collect();
    assert!(plain_lines.first().is_some_and(|line| line.starts_with(&format!("run={RUN} "))), "1 行目は便の段: {plain_lines:?}");
    let mut expected = plain_lines.clone();
    expected.push(LINE.to_owned());
    assert_eq!(with_lines, expected, "旗の無い周の行の後に 1 行だけ足す");
    assert_eq!(stderr_of(&with), stderr_of(&plain), "stderr は同じ");
    clean(&[&state]);
}

/// 旗 `--write` の無い周は書きの行を出さず、旗の有無に依らず event log と run dir の file の byte は撃つ前と同じ。
#[test]
fn vrunw_show_without_the_flag_has_no_write_line_and_writes_nothing() {
    let state = placed_state();
    let before = place_bytes(&state);
    let plain = show_with(&state, &[]);
    assert_eq!(plain.status.code(), Some(i32::from(RC_OK)), "旗の無い周: {}", stderr_of(&plain));
    assert!(!stdout_of(&plain).lines().any(|line| line.starts_with("write:")), "書きの行を出さない: {}", stdout_of(&plain));
    assert_eq!(place_bytes(&state), before, "旗の無い周は置き場を替えない");
    let with = show_with(&state, &["--write"]);
    assert_eq!(with.status.code(), Some(i32::from(RC_OK)), "旗の在る周: {}", stderr_of(&with));
    assert_eq!(place_bytes(&state), before, "旗の在る周も置き場を替えない");
    clean(&[&state]);
}

/// record の file の撃った行（`secs` を持つ行）の `write_bytes` の和の字（数でない値が在れば字 unmeasured）と、撃った行の数。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn fired_sum(path: &Path) -> (String, usize) {
    let text = fs::read_to_string(path).expect("書き手が置いた record を読める");
    let rows: Vec<Vec<(String, Value)>> =
        text.lines().map(|line| json_lite::parse_object(line).expect("record の行は object")).collect();
    let fired: Vec<Option<u64>> = rows
        .iter()
        .filter(|row| row.iter().any(|(key, _)| key == "secs"))
        .map(|row| row.iter().find(|(key, _)| key == "write_bytes").and_then(|(_, value)| value.as_str()?.parse().ok()))
        .collect();
    let sum = fired.iter().try_fold(0_u64, |sum, part| sum.checked_add((*part)?));
    (sum.map_or_else(|| "unmeasured".to_owned(), |found| found.to_string()), fired.len())
}

/// 書き手の置いた record の file の末に、`write_bytes` が `bytes` の撃った行を 1 本足す（file が無ければ歯が落ちる）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn append_fired(path: &Path, bytes: u64) {
    let text = fs::read_to_string(path).expect("書き手が置いた record が在る");
    let row = format!(r#"{{"schema":1,"n":99,"rc":0,"cmd":"marker","write_bytes":"{bytes}","kind":"verify","secs":1}}"#);
    fs::write(path, format!("{text}{row}\n")).expect("record に行を足せる");
}

/// toy の便を gate の PASS から着地まで通した後の `--write` の行は、書き手が run dir に置いた 3 つの record（gate・門・着地の確かめ）
/// を段ごとに拾う: 歯が 3 つの file に違う byte の撃った行を 1 本ずつ足した後、verify と end-gate と main はそれぞれの file の撃った
/// 行の和で、偽の runner は消費を書かないので runner と和は字 unmeasured。
#[test]
fn vrunw_show_write_reads_the_records_a_landed_run_wrote() {
    let (repo, state) = repo_with_state();
    let design = write_contract(&repo, &[], &[]);
    let id = gated_pass(&repo, &state, &design, &state.join("lens-ran"));
    let out = land_once(&repo, &state, &id);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "land は rc 0: {}", stderr_of(&out));
    let dir = run_dir(&state, &id);
    let (_, gate_rows) = fired_sum(&dir.join("verify.jsonl"));
    assert!(gate_rows > 0, "gate の record は書き手の撃った行を持つ（{gate_rows}）");
    for (name, bytes) in [("verify.jsonl", 300), ("end-gate.jsonl", 7_000), ("verify-main.jsonl", 90_000)] {
        append_fired(&dir.join(name), bytes);
    }
    let (verify, _) = fired_sum(&dir.join("verify.jsonl"));
    let (end_gate, _) = fired_sum(&dir.join("end-gate.jsonl"));
    let (main, _) = fired_sum(&dir.join("verify-main.jsonl"));
    assert!(end_gate != main, "段を分ける値: {end_gate}・{main}");
    let shown = run_pipe(&["show", "--run", &id, "--state-dir", &state.display().to_string(), "--write"]);
    assert_eq!(shown.status.code(), Some(i32::from(RC_OK)), "{}", stderr_of(&shown));
    let line = stdout_of(&shown).lines().last().map(str::to_owned).unwrap_or_default();
    assert_eq!(
        line,
        format!("write: runner=unmeasured verify={verify} end-gate={end_gate} main={main} total=unmeasured uncounted=worktree,land-git,reviewer-claude"),
        "3 つの record を段ごとに拾う"
    );
    clean(&[&repo, &state]);
}
