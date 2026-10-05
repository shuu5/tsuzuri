//! tool の呼びの組の後の配達の hook の中核の歯（行 f-deliver-tool・接頭辞 fdlt_）。
//! 台帳は歯の中で組む（問い fx-t.1〜fx-t.8）。hooks.json は workspace の根の plugin/hooks/hooks.json を読む。
//! 境界の tests/teeth2/fdlt.rs の歯の名もこの file が数える（fdlt_own_names_clean）。
#![cfg(test)]

use std::io::{ErrorKind, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::common::{
    FAKE_OUT, MARK_NAME, bead, declared, only, plugin_dir, read_hooks, rid, root, said_of,
};
use serde_json::Value;
use tsuzuri_contract::board::Reading;
use tsuzuri_core::delivery::{
    AGENT_KEYS, BATCH_FIELD, CONTEXT_CAP, Route, Said, TOOL_EVENT, context, context_for,
    main_thread, mark_line, named, unmarked,
};

/// 起草の時の main 666ccd51 の契約表の verify の filter の語を、ほかの語を部分の字として含まない語に畳んだ語。
const FILTERS: &[&str] = &[
    "aaround_",
    "abss_",
    "abst_",
    "accept_",
    "acchold_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "aface_",
    "afocus_",
    "alean_",
    "aord_",
    "aown_",
    "apark_",
    "apop_",
    "areread_",
    "askcard_",
    "athr_",
    "batchpanel_",
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadl_",
    "cadopt_",
    "cadq_",
    "cdorm_",
    "cexcl_",
    "cfsplit_",
    "cg9_",
    "cgdom_",
    "check",
    "cmark_",
    "cnote_",
    "cnret_",
    "contract_form_",
    "cround_",
    "csled_",
    "cspk_",
    "ctick_",
    "cupd_",
    "cupdlist_",
    "denv_",
    "dnedge_",
    "dngrp_",
    "dnrow_",
    "dnskip_",
    "dretry_",
    "dstg_",
    "ecache_",
    "eheld_acct_",
    "eheld_design_",
    "eheld_held_",
    "eheld_vessel_",
    "elazy_",
    "epolq_",
    "eretry_",
    "esig_",
    "evkind_",
    "fdlv_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "fserve_",
    "fstop_",
    "fundl_",
    "fxpre_",
    "g3g7_",
    "gacct_",
    "gapspage_",
    "gatt_",
    "gbnote_",
    "gchip_",
    "gcoach_",
    "gext_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gins_",
    "gjst_",
    "glabel_",
    "gmret_",
    "gmretw_",
    "gnav_",
    "gpface_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "gwv_",
    "hacols_",
    "harest_",
    "hasplit_",
    "hbconf_",
    "hbmark_",
    "hbon_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
    "hdchip_",
    "hfig_",
    "hnunk_",
    "hook_",
    "hruling_",
    "hsblock_",
    "hsderive_",
    "hshist_",
    "hspage_",
    "hspk_",
    "hsym_",
    "hthr_",
    "http_",
    "hwstore_",
    "iclose_",
    "ilink_",
    "jcount_",
    "jrun_",
    "kcli_",
    "kg9_",
    "kindlab_",
    "klink_",
    "ksum_",
    "lateface_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lgrp_",
    "lhome_",
    "lidle_",
    "lkind_",
    "lresume_",
    "lsnap_",
    "lspark_",
    "lstg_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
    "mqask_",
    "mqface_",
    "mstore_",
    "mtips_",
    "mtree_",
    "nact_",
    "nbatch_",
    "ncard_",
    "nextstep_",
    "nodepage_",
    "nstall_",
    "nsum_",
    "nsumw_",
    "ntc_",
    "ntime_",
    "nxact_",
    "nxorg_",
    "parts_",
    "pci_",
    "pclosed_",
    "pfold_",
    "pgsw_",
    "pgz_",
    "pipe_",
    "pkac_",
    "pkview_",
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
    "ptitle_",
    "pubfp_",
    "pubscan_",
    "punmap_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "qsig_",
    "question_",
    "rbusy_",
    "relay_",
    "rhold_",
    "runsdoc_",
    "rvk_",
    "saxis_",
    "sclosed_",
    "seatblock_",
    "seatcard_",
    "server_",
    "sesplit_",
    "sgrace_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "stbp_",
    "stcli_",
    "steady_",
    "sthr_",
    "stnfy_",
    "stskill_",
    "sttgt_",
    "sxaxis_",
    "tgall_",
    "tgown_",
    "ticker_",
    "tipx_",
    "tkad_",
    "tlic_",
    "topbar_",
    "topfit_",
    "tz_",
    "udacct_",
    "udash_",
    "unow_",
    "urpanel_",
    "uword_",
    "warnings",
    "wstrip_",
];

/// この行の歯の名の接頭辞。
const PREFIX: &str = "fdlt_";

/// hooks.json の PostToolBatch の command の字。
const COMMAND: &str = "[ -e \"$CLAUDE_PLUGIN_ROOT/scribe2-runner\" ] || \"$CLAUDE_PLUGIN_ROOT\"/bin/tzw hook deliver-tool --repo \"$CLAUDE_PROJECT_DIR\"";

/// 席の本体の PostToolBatch の入力の字。
const BATCH_INPUT: &str = r#"{"session_id":"s-1","hook_event_name":"PostToolBatch","tool_calls":[{"tool_name":"Bash"}]}"#;

const T1: &str = "fx-t.1:20260930T0101Z-1";
const T2: &str = "fx-t.2:20260930T0102Z-1";
const T3: &str = "fx-t.3:20260930T0103Z-1";
const T4: &str = "fx-t.4:20260930T0104Z-1";
const T5: &str = "fx-t.5:20260930T0105Z-1";
const T6: &str = "fx-t.6:20260930T0106Z-1";
const T7A: &str = "fx-t.7:20260930T0107Z-1";
const T7B: &str = "fx-t.7:20260930T0107Z-2";
const BATCH: &str = "batch:20260930T0103Z-1";

/// 裁定の行（逐語の欄は escape 済みの字を渡す・束の id が空なら束の欄を書かない）。
fn ruling_line(id: &str, question: &str, batch: &str, verbatim: &str) -> String {
    let batch = if batch.is_empty() {
        String::new()
    } else {
        format!("・{BATCH_FIELD}{batch}")
    };
    format!("裁定 id = {id}・問い = {question}{batch}・逐語 = {verbatim}")
}

/// 印の行。
fn mark(id: &str, route: Route) -> String {
    mark_line(&rid(id), route, "20260930T0110Z")
}

/// 問い fx-t.1〜fx-t.8 の台帳の字。
fn ledger() -> String {
    let r1 = ruling_line(T1, "fx-t.1", "", r"一行目\n二行目\\nの字");
    let r7b = ruling_line(T7B, "fx-t.7", "", "七の答え");
    Value::Array(vec![
        bead("fx-t.1", true, "closed", &[&r1]),
        bead(
            "fx-t.2",
            true,
            "closed",
            &[
                &ruling_line(T2, "fx-t.2", BATCH, "二の答え"),
                &mark(T2, Route::Tool),
            ],
        ),
        bead(
            "fx-t.3",
            true,
            "closed",
            &[&ruling_line(T3, "fx-t.3", BATCH, "三の答え")],
        ),
        bead(
            "fx-t.4",
            true,
            "closed",
            &[
                &ruling_line(T4, "fx-t.4", BATCH, "四の答え"),
                &mark(T4, Route::Stop),
            ],
        ),
        bead(
            "fx-t.5",
            false,
            "closed",
            &[&ruling_line(T5, "fx-t.5", "", "五の答え")],
        ),
        bead(
            "fx-t.6",
            true,
            "tombstone",
            &[&ruling_line(T6, "fx-t.6", "", "六の答え")],
        ),
        bead(
            "fx-t.7",
            true,
            "closed",
            &[
                &format!("裁定 id = {T7A}・問い = fx-t.7"),
                &r7b,
                &r7b,
            ],
        ),
        bead("fx-t.8", true, "closed", &[&r1, &mark(T1, Route::Stop)]),
    ])
    .to_string()
}

fn keys(v: &Value) -> Vec<&str> {
    let mut keys: Vec<&str> = v
        .as_object()
        .unwrap_or_else(|| panic!("object でない: {v}"))
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    keys
}

#[test]
fn fdlt_main_thread_keys() {
    assert_eq!(TOOL_EVENT, "PostToolBatch");
    assert_eq!(AGENT_KEYS, ["agent_id", "agent_type"]);
    for payload in [
        BATCH_INPUT,
        "{}",
        r#"{"tool_calls":[{"agent_id":"a1","agent_type":"t"}],"nested":{"agent_id":"a1"}}"#,
    ] {
        assert!(main_thread(payload), "{payload}");
    }
    for payload in [
        r#"{"agent_id":"a1"}"#,
        r#"{"agent_type":"t"}"#,
        r#"{"session_id":"s-1","agent_id":"a1","agent_type":"t"}"#,
        r#"{"agent_id":null}"#,
        "[]",
        r#""agent_id""#,
        "not json",
        "",
    ] {
        assert!(!main_thread(payload), "{payload}");
    }
}

fn known(ledger: &str) -> Vec<Said> {
    match unmarked(ledger) {
        Reading::Known(found) => found,
        Reading::Unknown => panic!("台帳が読めない"),
    }
}

#[test]
fn fdlt_unmarked_verbatims() {
    assert_eq!(
        known(&ledger()),
        [
            said_of("fx-t.1", T1, "一行目\n二行目\\nの字"),
            said_of("fx-t.3", T3, "三の答え"),
            said_of("fx-t.7", T7B, "七の答え"),
        ]
    );
    assert!(known("[]").is_empty());
    for text in ["not json", ""] {
        assert!(matches!(unmarked(text), Reading::Unknown), "{text:?}");
    }
}

/// 小さい Said（逐語は 1 字）。
fn small(n: u32) -> Said {
    said_of(&format!("fx-s.{n}"), &format!("fx-s.{n}:20260930T0101Z-1"), "x")
}

/// 逐語が字数ちょうどの Said。
fn sized(n: u32, chars: usize) -> Said {
    let mut s = small(n);
    s.verbatim = "あ".repeat(chars);
    s
}

fn additional(answer: &str) -> String {
    let answer: Value = serde_json::from_str(answer).expect("JSON");
    answer["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("字")
        .to_string()
}

/// 名指しの行の字（裁定 <id>（問い <問いの id>））。
fn naming(s: &Said) -> String {
    format!("裁定 {}（問い {}）", s.ruling, s.question)
}

#[test]
fn fdlt_tool_answer_and_named() {
    assert!(context_for(&[], TOOL_EVENT).is_none());
    let two = [small(1), small(2)];
    let answer: Value = serde_json::from_str(&context_for(&two, TOOL_EVENT).expect("答え"))
        .expect("JSON");
    assert_eq!(keys(&answer), ["hookSpecificOutput"]);
    let inner = &answer["hookSpecificOutput"];
    assert_eq!(keys(inner), ["additionalContext", "hookEventName"]);
    assert_eq!(inner["hookEventName"], "PostToolBatch");
    let prompt = context(&two).expect("答え");
    assert_eq!(inner["additionalContext"], additional(&prompt).as_str());
    let prompt_answer: Value = serde_json::from_str(&prompt).expect("JSON");
    assert_eq!(
        prompt_answer["hookSpecificOutput"]["hookEventName"],
        "UserPromptSubmit"
    );
    assert_eq!(context_for(&two, "UserPromptSubmit"), Some(prompt));

    assert_eq!(named(&[]), 0);
    assert_eq!(named(&two), 2);
    let shapes = [
        (vec![sized(1, CONTEXT_CAP), small(2), small(3)], 2),
        (vec![sized(1, CONTEXT_CAP + 1), small(2)], 1),
        (vec![small(1), sized(2, CONTEXT_CAP), small(3)], 2),
    ];
    for (said, want) in &shapes {
        assert_eq!(named(said), *want, "{said:?}");
        let text = additional(&context_for(said, TOOL_EVENT).expect("答え"));
        for (i, s) in said.iter().enumerate() {
            assert_eq!(text.contains(&naming(s)), i < *want, "{i}: {text}");
        }
    }

    tool_mark_line();
}

/// tool の呼びの経路の語と、その経路の配達の印の行の字。
fn tool_mark_line() {
    assert_eq!(Route::Tool.word(), "tool の呼び");
    assert_eq!(
        mark_line(&rid(T1), Route::Tool, "20260930T0102Z"),
        "配達 = fx-t.1:20260930T0101Z-1・経路 = tool の呼び・時刻 = 20260930T0102Z"
    );
}

#[test]
fn fdlt_hooks_batch_entry() {
    let hooks = read_hooks();
    let entry = only(&hooks["hooks"]["PostToolBatch"]);
    assert_eq!(keys(entry), ["hooks"], "matcher を持たない");
    let hook = only(&entry["hooks"]);
    assert_eq!(keys(hook), ["command", "timeout", "type"]);
    assert_eq!(hook["type"], "command");
    assert_eq!(hook["timeout"].as_u64(), Some(30), "timeout は数 30");
    assert_eq!(hook["command"], COMMAND);
}

/// command を sh -c で撃ち、標準入力に BATCH_INPUT を書いて子の終わりを待つ。
fn run_sh(project: &Path, plugin: &Path) -> Output {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(COMMAND)
        .env("CLAUDE_PROJECT_DIR", project)
        .env("CLAUDE_PLUGIN_ROOT", plugin)
        .env("PATH", "/usr/bin:/bin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("sh を撃つ");
    let mut stdin = child.stdin.take().expect("標準入力");
    match stdin.write_all(BATCH_INPUT.as_bytes()) {
        Ok(()) => {}
        // sh が標準入力を読まずに先に終わると書きは Broken pipe を返しうる。
        Err(e) if e.kind() == ErrorKind::BrokenPipe => {}
        Err(e) => panic!("入力を書く: {e}"),
    }
    drop(stdin);
    child.wait_with_output().expect("sh の終わりを待つ")
}

#[test]
fn fdlt_command_under_sh() {
    let work = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("fdlt-{}", std::process::id()));
    if work.exists() {
        std::fs::remove_dir_all(&work).expect("前の作業場を消す");
    }
    let project = work.join("project");
    let bin = work.join("target/debug");
    std::fs::create_dir_all(&bin).expect("build の dir を作る");
    declared(&project);
    let args_rec = work.join("args.rec");
    let stdin_rec = work.join("stdin.rec");
    let script = format!(
        "#!/bin/sh\nfor a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{}'\ncat > '{}'\nprintf '%s' '{FAKE_OUT}'\n",
        args_rec.display(),
        stdin_rec.display(),
    );
    let tz = bin.join("tz");
    std::fs::write(&tz, script).expect("偽の tz を書く");
    std::fs::set_permissions(&tz, std::fs::Permissions::from_mode(0o755))
        .expect("偽の tz を撃てる形にする");
    let plain = work.join("plugin-plain");
    plugin_dir(&plain);
    let marked = work.join("plugin-marked");
    std::fs::create_dir_all(&marked).expect("印の在る plugin の dir を作る");
    std::fs::write(marked.join(MARK_NAME), "").expect("印を置く");

    // 席: 印が無く tz が在る。
    let out = run_sh(&project, &plain);
    assert_eq!(out.status.code(), Some(0), "席の rc: {out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout), FAKE_OUT, "席の標準出力");
    let args = std::fs::read_to_string(&args_rec).expect("tz が撃たれて引数を記録した");
    assert_eq!(
        args,
        format!("hook\ndeliver-tool\n--repo\n{}\n", project.display()),
        "tz の引数"
    );
    let stdin = std::fs::read_to_string(&stdin_rec).expect("tz が標準入力を記録した");
    assert_eq!(stdin, BATCH_INPUT, "tz の標準入力");

    // runner: 印が在る。
    std::fs::remove_file(&args_rec).expect("引数の記録を消す");
    std::fs::remove_file(&stdin_rec).expect("標準入力の記録を消す");
    let out = run_sh(&project, &marked);
    assert_eq!(out.status.code(), Some(0), "runner の rc: {out:?}");
    assert!(out.stdout.is_empty(), "runner の標準出力: {out:?}");
    assert!(!args_rec.exists(), "runner で tz が撃たれた");
    assert!(!stdin_rec.exists(), "runner で tz が撃たれた");

    std::fs::remove_dir_all(&work).expect("作業場を消す");
}

/// tests/teeth1/fdlt.rs の test の fn の名（#[test] の次の行の fn）。
fn test_names(src: &str) -> Vec<String> {
    let lines: Vec<&str> = src.lines().collect();
    lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest).to_string()
        })
        .collect()
}

#[test]
fn fdlt_own_names_clean() {
    let core = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth1/fdlt.rs");
    let boundary = root().join("crates/tsuzuri-boundary/tests/teeth2/fdlt.rs");
    let mut names = Vec::new();
    for path in [core, boundary] {
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
        names.extend(test_names(&src));
    }
    assert_eq!(names.len(), 12, "歯の数: {names:?}");
    assert_eq!(FILTERS.len(), 239, "filter の語の数");
    for name in &names {
        let rest = name
            .strip_prefix(PREFIX)
            .unwrap_or_else(|| panic!("{name} は {PREFIX} で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
    for word in FILTERS {
        assert!(!PREFIX.contains(word), "接頭辞が {word} を含む");
        assert!(!word.contains(PREFIX), "{word} が接頭辞を含む");
    }
}
