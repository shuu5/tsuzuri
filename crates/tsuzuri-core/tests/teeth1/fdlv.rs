//! 配達の hook の中核の歯（行 f-deliver・接頭辞 fdlv_）。
//! 台帳は歯の中で組む（問い fx-d.1〜fx-d.6）。hooks.json は workspace の根の plugin/hooks/hooks.json を読む。
//! 境界の tests/teeth2/fdlv.rs の歯の名もこの file が数える（fdlv_own_names_clean）。
#![cfg(test)]

use std::io::{ErrorKind, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::surface::RulingId;
use tsuzuri_core::delivery::{
    BATCH_FIELD, CONTEXT_CAP, POINTER_HEAD, POINTER_TAIL, Said, context, pointed, said,
};

/// 起草の時の main 11affd91 の契約表の verify の filter の語を、ほかの語を部分の字として含まない語に畳んだ語。
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
    "epolq_",
    "eretry_",
    "esig_",
    "evkind_",
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
    "wstrip_",
];

/// この行の歯の名の接頭辞。
const PREFIX: &str = "fdlv_";

/// hooks.json の UserPromptSubmit の command の字。
const COMMAND: &str = "[ -e \"$CLAUDE_PLUGIN_ROOT/scribe2-runner\" ] || \"$CLAUDE_PLUGIN_ROOT\"/bin/tzw hook deliver --repo \"$CLAUDE_PROJECT_DIR\"";

/// runner の印の file の名。
const MARK_NAME: &str = "scribe2-runner";

/// 席の prompt を含む hook の入力の字（指し示しの行は器の名を持たない字でも拾う）。
const PROMPT_INPUT: &str = r#"{"session_id":"s-1","hook_event_name":"UserPromptSubmit","prompt":"ls"}"#;

/// 偽の tz が標準出力に出す字。
const FAKE_OUT: &str = "answer-line\n";

const D1: &str = "fx-d.1:20260930T0101Z-1";
const D2: &str = "fx-d.2:20260930T0110Z-1";
const D3: &str = "fx-d.3:20260930T0110Z-1";
const D6: &str = "fx-d.6:20260930T0120Z-1";
const BATCH: &str = "batch:20260930T0110Z-1";

/// D1 の逐語（改行と逆斜線と区切りの字を含む）。
const V1: &str = "一行目\n二行目\\nの字・終わり";

fn rid(id: &str) -> RulingId {
    RulingId::new(id).expect("裁定の id")
}

fn said_of(question: &str, id: &str, verbatim: &str) -> Said {
    Said {
        question: BeadId::new(question).expect("bead の id"),
        ruling: rid(id),
        verbatim: verbatim.to_string(),
    }
}

/// 歯の台帳の bead 1 本（label は問いの label か無し）。
fn bead(id: &str, question_label: bool, status: &str, notes: &[&str]) -> Value {
    let labels: Vec<&str> = if question_label {
        vec!["intake:question"]
    } else {
        vec!["other"]
    };
    json!({"id": id, "labels": labels, "status": status, "notes": notes.join("\n")})
}

/// 問い fx-d.1〜fx-d.6 の台帳の字。
fn ledger() -> String {
    let batch_line = |id: &str, q: &str, v: &str| {
        format!("裁定 id = {id}・問い = {q}・{BATCH_FIELD}{BATCH}・逐語 = {v}")
    };
    Value::Array(vec![
        bead(
            "fx-d.1",
            true,
            "closed",
            &[
                r"裁定 id = fx-d.1:20260930T0101Z-1・問い = fx-d.1・逐語 = 一行目\n二行目\\nの字・終わり",
                "配達 = fx-d.1:20260930T0101Z-1・経路 = 配達の口・時刻 = 20260930T0102Z",
            ],
        ),
        bead("fx-d.2", true, "closed", &[batch_line(D2, "fx-d.2", "束の答え").as_str()]),
        bead("fx-d.3", true, "closed", &[batch_line(D3, "fx-d.3", "束の答え").as_str()]),
        bead(
            "fx-d.4",
            false,
            "closed",
            &[batch_line("fx-d.4:20260930T0110Z-1", "fx-d.4", "束の答え").as_str()],
        ),
        bead(
            "fx-d.5",
            true,
            "tombstone",
            &[batch_line("fx-d.5:20260930T0110Z-1", "fx-d.5", "束の答え").as_str()],
        ),
        bead(
            "fx-d.6",
            true,
            "closed",
            &["裁定 id = fx-d.6:20260930T0120Z-1・問い = fx-d.6"],
        ),
    ])
    .to_string()
}

fn known(ledger: &str, ids: &[&str]) -> Vec<Said> {
    let ids: Vec<RulingId> = ids.iter().map(|i| rid(i)).collect();
    match said(ledger, &ids) {
        Reading::Known(found) => found,
        Reading::Unknown => panic!("台帳が読めない: {ids:?}"),
    }
}

fn prompt_input(prompt: &str) -> String {
    json!({"session_id": "s-1", "prompt": prompt}).to_string()
}

fn pointer_line(name: &str, id: &str) -> String {
    format!("{name} {POINTER_HEAD}{id}{POINTER_TAIL}")
}

fn ids_of(payload: &str) -> Vec<String> {
    pointed(payload).iter().map(ToString::to_string).collect()
}

#[test]
fn fdlv_pointer_ids_from_prompt() {
    assert_eq!(POINTER_HEAD, "seat: 裁定 ");
    assert_eq!(POINTER_TAIL, " が届いた（在りかは裁定面の記帳）");
    // 器の名は見ない・行の順・重なりを除く・束の id も拾う。
    let two = format!(
        "{}\nふつうの行\n{}\n{}\n{}",
        pointer_line("scribe2", D1),
        pointer_line("ほかの名", BATCH),
        pointer_line("scribe2", D1),
        pointer_line("", D2).trim_start(),
    );
    assert_eq!(ids_of(&prompt_input(&two)), [D1, BATCH, D2]);
    // 停止の hook の答えの形・ふつうの字・空白を含む id・空の id・尾の無い行は拾わない。
    let none = [
        format!("裁定 {D1} が届いた（問い fx-d.1）"),
        "ふつうの字".to_string(),
        pointer_line("scribe2", "a b"),
        pointer_line("scribe2", ""),
        format!("scribe2 {POINTER_HEAD}{D1}"),
        String::new(),
    ];
    for prompt in &none {
        assert!(ids_of(&prompt_input(prompt)).is_empty(), "{prompt}");
    }
    for payload in [
        r#"{"prompt":5}"#,
        r#"{"prompt":["seat: 裁定 x が届いた（在りかは裁定面の記帳）"]}"#,
        r#"{"session_id":"s-1"}"#,
        "[]",
        "not json",
        "",
    ] {
        assert!(pointed(payload).is_empty(), "{payload}");
    }
}

#[test]
fn fdlv_said_by_ruling_and_batch() {
    assert_eq!(BATCH_FIELD, "束 = ");
    let ledger = ledger();
    let d1 = said_of("fx-d.1", D1, V1);
    let d2 = said_of("fx-d.2", D2, "束の答え");
    let d3 = said_of("fx-d.3", D3, "束の答え");
    assert_eq!(known(&ledger, &[D1]), std::slice::from_ref(&d1));
    assert_eq!(known(&ledger, &[BATCH]), [d2.clone(), d3.clone()]);
    assert_eq!(known(&ledger, &[D3, D1]), [d1, d3]);
    assert_eq!(known(&ledger, &[D2]), [d2]);
    assert!(known(&ledger, &["fx-d.9:20260930T0101Z-1"]).is_empty());
    assert!(known(&ledger, &[D6]).is_empty());
    for text in ["not json", ""] {
        assert!(
            matches!(said(text, &[rid(D1)]), Reading::Unknown),
            "{text:?}"
        );
    }
}

#[test]
fn fdlv_context_shape_and_cap() {
    assert!(context(&[]).is_none());
    const { assert!(CONTEXT_CAP < 10_000) };
    let two = [said_of("fx-d.1", D1, V1), said_of("fx-d.2", D2, "答え 2")];
    let answer: Value = serde_json::from_str(&context(&two).expect("答え")).expect("JSON");
    let outer = answer.as_object().expect("object");
    assert_eq!(keys(&answer), ["hookSpecificOutput"]);
    let inner = &outer["hookSpecificOutput"];
    assert_eq!(keys(inner), ["additionalContext", "hookEventName"]);
    assert_eq!(inner["hookEventName"], "UserPromptSubmit");
    let want = format!(
        "席に届いた持ち主の裁定の逐語（2 件・台帳の問いの notes の裁定の行の写し）:\n\
         裁定 {D1}（問い fx-d.1）の逐語:\n{V1}\n\
         裁定 {D2}（問い fx-d.2）の逐語:\n答え 2"
    );
    assert_eq!(inner["additionalContext"], want.as_str());

    // 上限ちょうどの逐語は写し、その後の逐語は写さずに在りかの行で終える。
    let full = "あ".repeat(CONTEXT_CAP);
    let one = context(&[said_of("fx-d.1", D1, &full)]).expect("答え");
    assert!(one.contains(&full));
    assert!(!one.contains("写さない"), "{one}");
    let over = context(&[said_of("fx-d.1", D1, &full), said_of("fx-d.2", D2, "第二の答え")])
        .expect("答え");
    let answer: Value = serde_json::from_str(&over).expect("JSON");
    let text = answer["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("字");
    assert!(text.contains(&full), "1 つ目の逐語を写す");
    assert!(!text.contains("第二の答え"), "2 つ目の逐語は写さない");
    let last = text.lines().last().expect("最後の行");
    assert!(last.starts_with(&format!("裁定 {D2}（問い fx-d.2）")), "{last}");
    assert!(last.ends_with("（在りかは台帳の問いの notes の裁定の行）"), "{last}");
    assert!(text.chars().count() < 10_000);
}

/// plugin の形の dir（workspace の根の plugin の bin/tzw と plugin.json を写す・tzw は dir の親の target/debug/tz を引く）。
fn plugin_dir(dir: &Path) {
    for rel in ["bin/tzw", ".claude-plugin/plugin.json"] {
        let to = dir.join(rel);
        std::fs::create_dir_all(to.parent().expect("親の dir")).expect("plugin の dir を作る");
        std::fs::copy(root().join("plugin").join(rel), &to).expect("plugin の file を写す");
    }
}

/// project の宣言（git config の scribe2.statedir）の在る repo の dir を作る。
fn declared(project: &Path) {
    std::fs::create_dir_all(project).expect("project の dir を作る");
    for args in [
        &["init", "-q"][..],
        &["config", "scribe2.statedir", "/nonexistent/state"],
    ] {
        let ok = Command::new("git")
            .arg("-C")
            .arg(project)
            .args(args)
            .status()
            .expect("git");
        assert!(ok.success(), "git {args:?}");
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_hooks() -> Value {
    let path = root().join("plugin/hooks/hooks.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("hooks.json を読む: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("hooks.json は JSON でない: {e}"))
}

/// object の鍵を並べ替えた列。
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

/// 要素 1 つの配列のその要素。
fn only(v: &Value) -> &Value {
    match v.as_array().map(Vec::as_slice) {
        Some([one]) => one,
        _ => panic!("要素 1 つの配列でない: {v}"),
    }
}

#[test]
fn fdlv_hooks_prompt_entry() {
    let hooks = read_hooks();
    let entry = only(&hooks["hooks"]["UserPromptSubmit"]);
    assert_eq!(keys(entry), ["hooks"], "matcher を持たない");
    let hook = only(&entry["hooks"]);
    assert_eq!(keys(hook), ["command", "timeout", "type"]);
    assert_eq!(hook["type"], "command");
    assert_eq!(hook["timeout"].as_u64(), Some(10), "timeout は数 10");
    assert_eq!(hook["command"], COMMAND);
}

/// command を sh -c で撃ち、標準入力に PROMPT_INPUT を書いて子の終わりを待つ。
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
    match stdin.write_all(PROMPT_INPUT.as_bytes()) {
        Ok(()) => {}
        // sh が標準入力を読まずに先に終わると書きは Broken pipe を返しうる。
        Err(e) if e.kind() == ErrorKind::BrokenPipe => {}
        Err(e) => panic!("入力を書く: {e}"),
    }
    drop(stdin);
    child.wait_with_output().expect("sh の終わりを待つ")
}

#[test]
fn fdlv_command_under_sh() {
    let work = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("fdlv-{}", std::process::id()));
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
        format!("hook\ndeliver\n--repo\n{}\n", project.display()),
        "tz の引数"
    );
    let stdin = std::fs::read_to_string(&stdin_rec).expect("tz が標準入力を記録した");
    assert_eq!(stdin, PROMPT_INPUT, "tz の標準入力");

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

/// tests/teeth1/fdlv.rs の test の fn の名（#[test] の次の行の fn）。
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
fn fdlv_own_names_clean() {
    let core = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth1/fdlv.rs");
    let boundary = root().join("crates/tsuzuri-boundary/tests/teeth2/fdlv.rs");
    let mut names = Vec::new();
    for path in [core, boundary] {
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
        names.extend(test_names(&src));
    }
    assert_eq!(names.len(), 11, "歯の数: {names:?}");
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
