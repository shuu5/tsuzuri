//! turn の終わりの古さの止めの歯（接頭辞 `wipst_`・設計 docs/design/dialogue-surface.md §12・台帳の記録の節 25 / 26）。
//! 書いた turn の stop が作業中の bead の定型の行の古さを 1 度だけ止めることを、純関数の表と偽の bd を置いた stop の外形から測る。

use super::*;
use vessel::hook::role_guard::PathKind;
use vessel::hook::turn_end::{stale_of, wrote_of, Staleness, Wrote};

/// 古い頭の段（どの turn の最初の書きより前）。
const OLD_HEAD: &str = "[席 2000-01-01T00:00Z]";

/// payload の頭の波括弧の後ろに session の id を差す。
fn with_sid(payload: &str, sid: &str) -> String {
    format!("{{\"session_id\":{},{}", json_lite::quote(sid), payload.strip_prefix('{').unwrap_or(payload))
}

/// `pre-tool-use` を session `sid` で撃つ。
fn pre_tool(args: &[&str], sid: &str, payload: &str) -> Output {
    let mut all = vec!["pre-tool-use"];
    all.extend(args);
    run_hook_args(&all, &with_sid(payload, sid))
}

/// 台帳の書き（`bdw update w-1 --priority 1`）を通す（rc 0 を要求する）。
fn write_ledger(repo: &Path, args: &[&str], sid: &str) {
    let out = pre_tool(args, sid, &bash_payload(repo, "bdw update w-1 --priority 1"));
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "台帳の書きは通る: {}", stderr_text(&out));
}

/// 撃たれるたびに `calls` へ 1 行を足し、`extra`（exit か sleep の行・無ければ空）の後に `body` を stdout へ出す偽の台帳 client。
#[expect(clippy::expect_used, reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く")]
fn calls_bd(dir: &Path, body: &str, extra: &str) -> String {
    use std::os::unix::fs::PermissionsExt;
    let (json, calls, path) = (dir.join("bd.json"), dir.join("calls"), dir.join("bd"));
    fs::write(&json, body).expect("台帳の fixture を書ける");
    let script = format!("#!/bin/sh\necho called >> \"{}\"\n{extra}\ncat \"{}\"\n", calls.display(), json.display());
    fs::write(&path, script).expect("偽の bd を書ける");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("偽の bd を実行可能にできる");
    path.display().to_string()
}

/// 偽の bd が撃たれた回数。
fn calls_of(dir: &Path) -> usize {
    fs::read_to_string(dir.join("calls")).map_or(0, |text| text.lines().count())
}

/// 台帳の JSON（鍵 id・status・title・notes の要素の配列）。
fn ledger_json(beads: &[(&str, &str, &str)]) -> String {
    let items: Vec<String> = beads
        .iter()
        .map(|(id, status, notes)| {
            format!("{{\"id\":{},\"status\":{},\"title\":\"作業\",\"notes\":{}}}", json_lite::quote(id), json_lite::quote(status), json_lite::quote(notes))
        })
        .collect();
    format!("[{}]", items.join(","))
}

/// 今の分の頭の札（`[席 <今の分>Z]`・分を跨ぐ揺れを避けるため、秒が 45 を過ぎていれば次の分まで待つ）。
fn this_minute_head() -> String {
    let now = vessel::seat::state::now_secs();
    if now % 60 >= 45 {
        std::thread::sleep(std::time::Duration::from_secs(61 - now % 60));
    }
    let stamp = vessel::fleet::cli::format_utc(vessel::seat::state::now_secs());
    format!("[席 {}Z]", stamp.get(..16).unwrap_or_default())
}

/// stale の止めの外形: rc 2・stdout 0 byte・stderr はちょうど 1 行で、`turn-end stale` と `wip=<wip>` を持つ。
fn assert_stale(out: &Output, wip: &str, why: &str) {
    assert_eq!(out.status.code(), Some(i32::from(RC_BROKEN)), "{why}: rc 2: {}", stderr_text(out));
    assert!(out.stdout.is_empty(), "{why}: stdout 0 byte");
    let text = stderr_text(out);
    assert_eq!(text.lines().count(), 1, "{why}: stderr は 1 行: {text}");
    for needle in ["turn-end stale", &format!("wip={wip}"), "wrote="] {
        assert!(text.contains(needle), "{why}: {needle}: {text}");
    }
}

/// (2) 書きの読みの表: Bash の台帳の書き（subcommand が WRITES の片）は Ledger、設計の種別の編集の道具は Design、ほかは None。
#[test]
fn wipst_wrote_of_reads_ledger_writes_and_design_kinds_purely() {
    let table = [
        ("Bash", Some("bdw update w-1 --priority 1"), None, Some(Wrote::Ledger)),
        ("Bash", Some("bd --readonly list --json"), None, None),
        ("Bash", Some("git status && bdw close x --reason r"), None, Some(Wrote::Ledger)),
        ("Bash", Some("echo bdw update w-1"), None, None),
        ("Bash", Some("scripts/bdw note w-1 x"), None, Some(Wrote::Ledger)),
        ("Edit", None, Some(PathKind::DesignDoc), Some(Wrote::Design)),
        ("Write", None, Some(PathKind::DesignIntent), Some(Wrote::Design)),
        ("Write", None, Some(PathKind::Code), None),
        ("Read", None, Some(PathKind::DesignIntent), None),
    ];
    assert_eq!(table.len(), 9, "母集団");
    for (tool, command, kind, expected) in table {
        assert_eq!(wrote_of(tool, command, kind), expected, "{tool} {command:?} {kind:?}");
    }
    assert_eq!((Wrote::Ledger.as_str(), Wrote::Design.as_str()), ("ledger", "design"));
}

/// (3) 古さの表: 作業中の bead のどれか 1 本に、最初の書きの分以上の頭の段の定型の行が在れば Fresh、無ければ（0 本も）Stale。
#[test]
fn wipst_stale_of_needs_one_fresh_wip_line_purely() {
    let since = vessel::fleet::epoch_of("2026-10-06T11:21:10Z").unwrap_or_default();
    let bead = |id: &str, status: &str, notes: &str| recent::Bead { id: id.into(), status: status.into(), title: String::new(), updated: None, notes: notes.into() };
    let stale = |ids: &[&str]| Staleness::Stale { wip: ids.iter().map(|id| (*id).to_owned()).collect() };
    let table = [
        ("作業中 0 本", vec![bead("o-1", "open", "[席 2026-10-06T11:22Z]\n計画: x")], stale(&[])),
        ("今の分の段", vec![bead("w-1", "in_progress", "[席 2026-10-06T11:21Z]\n計画: x")], Staleness::Fresh),
        ("前の分の段", vec![bead("w-1", "in_progress", "[席 2026-10-06T11:20Z]\n計画: x")], stale(&["w-1"])),
        (
            "新しい段に定型の行が無い",
            vec![bead("w-1", "in_progress", "[席 2026-10-06T11:20Z]\n計画: x\n[席 2026-10-06T11:22Z]\n着地した")],
            stale(&["w-1"]),
        ),
        (
            "2 本のうち 1 本が新しい",
            vec![bead("w-1", "in_progress", "[席 2026-10-06T11:20Z]\n計画: x"), bead("w-2", "in_progress", "[席 2026-10-06T11:22Z]\n次の手: y")],
            Staleness::Fresh,
        ),
        ("頭の後が空白だけ", vec![bead("w-1", "in_progress", "[席 2026-10-06T11:22Z]\n計画:   ")], stale(&["w-1"])),
        ("10 分の形は下の端", vec![bead("w-1", "in_progress", "[席 2026-10-06T11:2xZ]\n計画: x")], stale(&["w-1"])),
        ("段の頭が無い", vec![bead("w-1", "in_progress", "計画: x")], stale(&["w-1"])),
    ];
    assert_eq!(table.len(), 8, "母集団");
    for (name, beads, expected) in table {
        assert_eq!(stale_of(since, &beads), expected, "{name}");
    }
}

/// (4) 古い頭の定型の行を持つ作業中の bead が在る turn は 1 度だけ止める。再入は通し、次の書きでまた止める。
#[test]
fn wipst_blocks_once_per_written_turn_when_the_lines_are_old() {
    let (repo, aux) = (git_repo(), tmp());
    let state = linked(&repo);
    let bd = calls_bd(&aux, &ledger_json(&[("w-1", "in_progress", &format!("{OLD_HEAD}\n計画: 古い計画"))]), "");
    let flag = state.display().to_string();
    let args = ["--state-dir", flag.as_str(), "--bd", bd.as_str()];
    start_session(&repo, &args, "sid-w");
    write_ledger(&repo, &args, "sid-w");
    assert_stale(&stop_hook(&repo, &args, Some("sid-w"), false), "w-1", "古い段");
    assert_passes(&stop_hook(&repo, &args, Some("sid-w"), true), "再入は通す");
    assert_passes(&stop_hook(&repo, &args, Some("sid-w"), false), "印は空になり、続く再入でない stop は通す");
    write_ledger(&repo, &args, "sid-w");
    assert_stale(&stop_hook(&repo, &args, Some("sid-w"), false), "w-1", "もう 1 度の書き");
    clean(&[&repo, &aux, &state]);
}

/// (5) 台帳は書きの後の stop でだけ 1 回読む。今の分の頭の段が在れば通し、次の stop は読まない。
#[test]
fn wipst_reads_the_ledger_only_after_a_write() {
    let (repo, aux) = (git_repo(), tmp());
    let state = linked(&repo);
    let head = this_minute_head();
    let bd = calls_bd(&aux, &ledger_json(&[("w-1", "in_progress", &format!("{head}\n計画: 今の計画"))]), "");
    let flag = state.display().to_string();
    let args = ["--state-dir", flag.as_str(), "--bd", bd.as_str()];
    start_session(&repo, &args, "sid-r");
    assert_passes(&stop_hook(&repo, &args, Some("sid-r"), false), "書きの無い turn");
    assert_eq!(calls_of(&aux), 0, "書きの無い turn は台帳を読まない");
    write_ledger(&repo, &args, "sid-r");
    assert_passes(&stop_hook(&repo, &args, Some("sid-r"), false), "今の分の段が在る");
    assert_eq!(calls_of(&aux), 1, "書きの後の stop は台帳を 1 回読む");
    assert_passes(&stop_hook(&repo, &args, Some("sid-r"), false), "次の stop");
    assert_eq!(calls_of(&aux), 1, "印は空になり、次の stop は台帳を読まない");
    clean(&[&repo, &aux, &state]);
}

/// (6) 設計の書きは宣言の種別で測り、設計でない file・repo の外・runner は印に載せない。作業中の bead が 0 本の turn も止める（wip=-）。
#[test]
fn wipst_marks_follow_the_declared_design_kinds_and_skip_runners() {
    let (repo, aux) = (git_repo(), tmp());
    let declaration = paths_declaration("design-intent-paths = [\"spec/\"]\n");
    fs::write(repo.join(DECL_FILE), declaration).unwrap_or_else(|err| panic!("宣言を書ける: {err}"));
    git(&repo, &["add", DECL_FILE]);
    git(&repo, &["commit", "-q", "-m", "declare"]);
    let state = linked(&repo);
    let bd = calls_bd(&aux, "[]", "");
    let flag = state.display().to_string();
    let args = ["--state-dir", flag.as_str(), "--bd", bd.as_str()];
    start_session(&repo, &args, "sid-k");
    let runner = state.join("pipe").join("x").display().to_string();
    let outside = aux.join("x.yaml").display().to_string();
    let inside = |rel: &str| repo.join(rel).display().to_string();
    let skipped = [
        ("宣言の外の design-intent/", pre_tool(&args, "sid-k", &tool_payload(&repo, "Write", &inside("design-intent/x.yaml")))),
        ("repo の外", pre_tool(&args, "sid-k", &tool_payload(&repo, "Write", &outside))),
        (
            "runner",
            pre_tool(&[args.as_slice(), &["--plugin-root", runner.as_str()]].concat(), "sid-k", &tool_payload(&repo, "Write", &inside("spec/x.yaml"))),
        ),
    ];
    for (name, out) in skipped {
        assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{name}: 書きは通る: {}", stderr_text(&out));
        assert_passes(&stop_hook(&repo, &args, Some("sid-k"), false), name);
        assert_eq!(calls_of(&aux), 0, "{name}: 台帳を読まない");
    }
    let out = pre_tool(&args, "sid-k", &tool_payload(&repo, "Write", &inside("spec/x.yaml")));
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "宣言の設計の書きは通る: {}", stderr_text(&out));
    assert_stale(&stop_hook(&repo, &args, Some("sid-k"), false), "-", "宣言の設計の書き（作業中 0 本）");
    assert_eq!(calls_of(&aux), 1, "台帳を 1 回読む");
    clean(&[&repo, &aux, &state]);
}

/// (7) 台帳を読めない周（子が rc 1）と待ち上限を越えた周は止めず、reason ledger-unreadable と ledger-timeout を 1 件ずつ記帳する。
#[test]
fn wipst_ledger_unreadable_and_timeout_pass_and_are_recorded() {
    let (repo, aux) = (git_repo(), tmp());
    let state = linked(&repo);
    let flag = state.display().to_string();
    let rules = aux.join("rules.toml");
    let row = "[[rule]]\nid = \"seat.ledger_timeout_s\"\nkind = \"LedgerTimeoutS\"\nvalue = 1\nenabled = true\nruling = \"r\"\nruled_at = \"d\"\n";
    fs::write(&rules, format!("schema = 1\n\n{row}")).unwrap_or_else(|err| panic!("rules: {err}"));
    let (failing, sleeping) = (tmp(), tmp());
    let bd_fail = calls_bd(&failing, "[]", "exit 1");
    let bd_slow = calls_bd(&sleeping, "[]", "sleep 3");
    let pre = ["--state-dir", flag.as_str()];
    start_session(&repo, &pre, "sid-u");
    start_session(&repo, &pre, "sid-t");
    write_ledger(&repo, &pre, "sid-u");
    let out = stop_hook(&repo, &["--state-dir", flag.as_str(), "--bd", bd_fail.as_str()], Some("sid-u"), false);
    assert_eq!(unjudged_after(&state, &out, "子が rc 1"), [(Some("sid-u".to_owned()), "ledger-unreadable".to_owned())]);
    write_ledger(&repo, &pre, "sid-t");
    let slow = ["--state-dir", flag.as_str(), "--bd", bd_slow.as_str(), "--rules", &rules.display().to_string()];
    let out = stop_hook(&repo, &slow, Some("sid-t"), false);
    let expected = [(Some("sid-u".to_owned()), "ledger-unreadable".to_owned()), (Some("sid-t".to_owned()), "ledger-timeout".to_owned())];
    assert_eq!(unjudged_after(&state, &out, "待ち上限 1 秒で 3 秒眠る"), expected);
    clean(&[&repo, &aux, &state, &failing, &sleeping]);
}

/// (8) 未仕分けの止めが先で印を消さず、再入の後の再入でない stop が古さの止めになる。
#[test]
fn wipst_unsorted_block_comes_first_and_keeps_the_marks() {
    let (repo, aux) = (git_repo(), tmp());
    let state = linked(&repo);
    let bd = calls_bd(&aux, &ledger_json(&[("w-1", "in_progress", &format!("{OLD_HEAD}\n計画: 古い計画"))]), "");
    let flag = state.display().to_string();
    let args = ["--state-dir", flag.as_str(), "--bd", bd.as_str()];
    start_session(&repo, &args, "sid-b");
    let ts = say_in(&repo, "sid-b", "未仕分けの発話");
    write_ledger(&repo, &args, "sid-b");
    let first = stop_hook(&repo, &args, Some("sid-b"), false);
    assert_blocked(&first, &[&ts], "未仕分けが先");
    assert!(stderr_text(&first).contains("turn-end block"), "未仕分けの止めの行: {}", stderr_text(&first));
    assert_eq!(calls_of(&aux), 0, "未仕分けの止めは台帳を読まない");
    assert!(session_files(&state).contains(&"sid-b/wrote".to_owned()), "印は消えない: {:?}", session_files(&state));
    assert_passes(&stop_hook(&repo, &args, Some("sid-b"), true), "再入");
    assert_stale(&stop_hook(&repo, &args, Some("sid-b"), false), "w-1", "古さの止め");
    clean(&[&repo, &aux, &state]);
}
