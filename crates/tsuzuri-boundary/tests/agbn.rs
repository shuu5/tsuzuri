//! 係の札の結びの道と出す物の字の形の歯（接頭辞 agbn_・設計ノート surface-wave29c 行 ag-bind・判断の記録 ADR-59 決定 (4) と帰結の (1)・ADR-63）。
//! tz の binary の tz hook agent-spawn・agent-bind・agent-guard・agent-meter・agent-stop に、Claude Code の門の入力の形を標準入力で渡す
//! （team の形の起こしの結果・係の呼びの入力の欄・meta.json の欄は 2026-10-05 に試しの窓で記した形・会話の id と path は伏せた）。
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）の drafts/ を --drafts で、親の記録の path をその下の T/s.jsonl で渡す。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_core::agent::spec::outs::{astray, mend};

/// 係の名。
const NAME: &str = "w241x";

/// 係の呼びの入力の係の id（team の形の起こし・名を付けない形の結果の agentId と同じ形）。
const TEAM_ID: &str = "aw241x-0123456789abcdef";
const PLAIN_ID: &str = "a0123456789abcdef";

/// w/ からの相対の出す物。
const GOOD: &str = "notes.md,patch/ag.patch";

/// team の形の起こしの結果（係の呼びの入力の係の id を持たない）。
const TEAM_RESULT: &str = r#","tool_response":{"status":"teammate_spawned","teammate_id":"w241x@session-0","agent_id":"w241x@session-0","agent_type":"tsuzuri:drafter","model":"opus","name":"w241x","color":"purple","tmux_session_name":"in-process","tmux_window_name":"in-process","tmux_pane_id":"in-process","team_name":"session-0","is_splitpane":false,"plan_mode_required":false}"#;

/// team でない形の起こしの結果（agentId が係の呼びの入力の係の id と同じ）。
const PLAIN_RESULT: &str = r#","tool_response":{"isAsync":true,"status":"async_launched","agentId":"a0123456789abcdef","description":"起草","resolvedModel":"claude-opus-5-5","outputFile":"/T/out","canReadOutputFile":true}"#;

/// 係の記録の隣の meta.json（team の形・team でない形）。
const TEAM_META: &str = r#"{"agentType":"w241x","description":"起草","name":"w241x","spawnDepth":0,"requestShape":"background","requestNonInteractive":true,"model":"opus","taskKind":"in_process_teammate","teamName":"session-0","color":"purple","planModeRequired":false,"customAgentType":"tsuzuri:drafter","permissionMode":"bypassPermissions"}"#;
const PLAIN_META: &str = r#"{"agentType":"tsuzuri:drafter","description":"起草","name":"w241x","toolUseId":"toolu_X","spawnDepth":1,"requestShape":"background","requestNonInteractive":true}"#;

/// 歯ごとの置き場（前の撃ちの残りを消して作る）と、その下の空の drafts/。
fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("agbn")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("drafts")).unwrap();
    root
}

/// tz hook <verb> --repo <root> --drafts <root>/drafts に `input` を渡した結果。
fn hook(verb: &str, root: &Path, input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["hook", verb, "--repo"])
        .arg(root)
        .arg("--drafts")
        .arg(root.join("drafts"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    child.wait_with_output().unwrap()
}

/// 置き場の下の file の字。
fn read(root: &Path, path: &str) -> String {
    fs::read_to_string(root.join(path)).unwrap()
}

/// 席の Agent の呼びの入力（`outputs` は頼みの頭の出す物の値・`tail` は末の欄の足し）。
fn seat(event: &str, outputs: &str, tail: &str) -> String {
    let prompt = format!(
        r"予算: token 250000\n組み: なし\n対象: t3-hub.87.35\n出す物: {outputs}\n\n行 ag-bind を起草する。"
    );
    format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"/T/seat.jsonl","cwd":"/W","permission_mode":"bypassPermissions","hook_event_name":"{event}","tool_name":"Agent","tool_input":{{"description":"起草","prompt":"{prompt}","subagent_type":"tsuzuri:drafter","name":"{NAME}","run_in_background":true}},"tool_use_id":"toolu_X"{tail}}}"#
    )
}

/// 係の道具の呼びの入力（`event` は PreToolUse か PostToolUse・親の記録は置き場の下の T/s.jsonl）。
fn call_in(root: &Path, event: &str, id: &str) -> String {
    format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"{}","cwd":"/W","permission_mode":"bypassPermissions","agent_id":"{id}","agent_type":"{NAME}","hook_event_name":"{event}","tool_name":"Bash","tool_input":{{"command":"echo"}},"tool_use_id":"toolu_Y"}}"#,
        root.join("T/s.jsonl").display()
    )
}

/// 係の終わりの入力（1 度目の終わり・係の記録は無い path）。
fn stop_in(root: &Path, id: &str) -> String {
    format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"{}","cwd":"/W","permission_mode":"bypassPermissions","agent_id":"{id}","agent_type":"{NAME}","hook_event_name":"SubagentStop","stop_hook_active":false,"agent_transcript_path":"{}","last_assistant_message":"済み"}}"#,
        root.join("T/s.jsonl").display(),
        root.join("T/none.jsonl").display()
    )
}

/// 係の記録の隣の meta.json を書く。
fn meta(root: &Path, id: &str, body: &str) {
    let dir = root.join("T/s/subagents");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("agent-{id}.meta.json")), body).unwrap();
}

/// 起こしの門を通して係の札を書き、札の字を返す（札の係の id は無い）。
fn spawned(root: &Path) -> String {
    let out = hook("agent-spawn", root, &seat("PreToolUse", GOOD, ""));
    assert_eq!((out.status.code(), out.stdout.len()), (Some(0), 0));
    let spec = read(root, "drafts/w241x/spec.json");
    assert!(spec.contains(r#""agent_id": null"#), "{spec}");
    spec
}

/// 札の係の id が `id` に替わり、ほかの字は前の札のままで、`.agents/<id>` に名が在る。
fn bound_to(root: &Path, spec: &str, id: &str) {
    let want = spec.replace(r#""agent_id": null"#, &format!(r#""agent_id": "{id}""#));
    assert_eq!(read(root, "drafts/w241x/spec.json"), want);
    assert_eq!(read(root, &format!("drafts/.agents/{id}")), "w241x\n");
}

/// 結べない時の標準エラーの 1 行。
fn line(id: &str, cause: &str) -> String {
    format!("tz hook agent: 係の id {id} を係の札に結べない（{cause}・門は通す）\n")
}

/// team の形の起こしの結果は結びの口に何も書かせず、係の 1 度目の呼びで係の門が meta.json の名の札に係の id を結ぶ。
/// 結んだ後の測りは結びの無い呼びを記帳しない。
#[test]
fn agbn_team_spawn_binds_through_the_meta_name() {
    let root = place("team");
    let spec = spawned(&root);
    let out = hook("agent-bind", &root, &seat("PostToolUse", GOOD, TEAM_RESULT));
    assert_eq!(
        (out.status.code(), out.stdout.len(), out.stderr.len()),
        (Some(0), 0, 0)
    );
    assert_eq!(read(&root, "drafts/w241x/spec.json"), spec);
    assert!(!root.join("drafts/.agents").exists());
    meta(&root, TEAM_ID, TEAM_META);
    let out = hook("agent-guard", &root, &call_in(&root, "PreToolUse", TEAM_ID));
    assert_eq!(
        (out.status.code(), out.stdout.len(), out.stderr.len()),
        (Some(0), 0, 0)
    );
    bound_to(&root, &spec, TEAM_ID);
    let out = hook(
        "agent-meter",
        &root,
        &call_in(&root, "PostToolUse", TEAM_ID),
    );
    assert_eq!((out.status.code(), out.stderr.len()), (Some(0), 0));
    assert!(!root.join("drafts/.agents/unbound.jsonl").exists());
}

/// team でない形の起こしの結果は結びの口が agentId で結び、結びの口が撃たれなかった係は測りが team でない形の meta.json の名で結ぶ。
#[test]
fn agbn_plain_spawn_binds_by_the_result_and_by_the_meta() {
    let root = place("plain");
    let spec = spawned(&root);
    let out = hook(
        "agent-bind",
        &root,
        &seat("PostToolUse", GOOD, PLAIN_RESULT),
    );
    assert_eq!(
        (out.status.code(), out.stdout.len(), out.stderr.len()),
        (Some(0), 0, 0)
    );
    bound_to(&root, &spec, PLAIN_ID);
    let late = place("plain-late");
    let spec = spawned(&late);
    meta(&late, PLAIN_ID, PLAIN_META);
    let out = hook(
        "agent-meter",
        &late,
        &call_in(&late, "PostToolUse", PLAIN_ID),
    );
    assert_eq!((out.status.code(), out.stderr.len()), (Some(0), 0));
    bound_to(&late, &spec, PLAIN_ID);
    assert!(!late.join("drafts/.agents/unbound.jsonl").exists());
}

/// 結べない 1 つの形を作り、測りが通して（rc 0・標準出力なし）訳の 1 行を標準エラーに出し、訳を記帳し、札と結びを書かないことを見る。
/// `meta_body` は meta.json の字（None は書かない）・`edit` は起こしの門が書いた札の直し。
fn miss(case: &str, meta_body: Option<&str>, edit: fn(&str) -> String, cause: &str) {
    let root = place(case);
    let spec = edit(&spawned(&root));
    fs::write(root.join("drafts/w241x/spec.json"), &spec).unwrap();
    if let Some(body) = meta_body {
        meta(&root, TEAM_ID, body);
    }
    let out = hook(
        "agent-meter",
        &root,
        &call_in(&root, "PostToolUse", TEAM_ID),
    );
    assert_eq!(
        (out.status.code(), out.stdout.len()),
        (Some(0), 0),
        "{case}"
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        line(TEAM_ID, cause),
        "{case}"
    );
    let log = read(&root, "drafts/.agents/unbound.jsonl");
    assert_eq!(log.lines().count(), 1, "{case}");
    assert!(
        log.contains(&format!(r#""cause":"{cause}""#)),
        "{case} {log}"
    );
    assert_eq!(read(&root, "drafts/w241x/spec.json"), spec, "{case}");
    assert!(
        !root.join(format!("drafts/.agents/{TEAM_ID}")).exists(),
        "{case}"
    );
}

/// meta.json が無い・meta.json に名が無い・名の札が無い・札が別の係の id に結ばれている・札に終えの印が在る、の 5 つは結ばずに 1 行を出す。
#[test]
fn agbn_unbound_call_prints_one_line_and_records_the_cause() {
    let same = |s: &str| s.to_string();
    let nameless = TEAM_META.replace(r#""name":"w241x","#, "");
    let other = TEAM_META.replace(r#""name":"w241x""#, r#""name":"w999x""#);
    miss("no-meta", None, same, "係の記録の隣の meta.json が読めない");
    miss("nameless", Some(&nameless), same, "meta.json に名が無い");
    miss("no-spec", Some(&other), same, "名 w999x の札が無い");
    miss(
        "taken",
        Some(TEAM_META),
        |s| {
            s.replace(
                r#""agent_id": null"#,
                r#""agent_id": "aw241x-ffffffffffffffff""#,
            )
        },
        "名 w241x の札は別の係の id aw241x-ffffffffffffffff に結ばれている",
    );
    miss(
        "ended",
        Some(TEAM_META),
        |s| s.replace(r#""ended": null"#, r#""ended": 1"#),
        "名 w241x の札に終えの印が在る",
    );
}

/// team の形の係の 1 度目の終わりは、終える前の門が meta.json の名の札に結んで欠けで止め（rc 2）、
/// meta.json の無い係の終わりは通して（rc 0）訳の 1 行を出し、門の事 SubagentStop の行を記帳する。
#[test]
fn agbn_stop_holds_a_team_agent_and_passes_an_unbound_one_with_a_line() {
    let root = place("stop-team");
    let spec = spawned(&root);
    meta(&root, TEAM_ID, TEAM_META);
    let out = hook("agent-stop", &root, &stop_in(&root, TEAM_ID));
    assert_eq!(out.status.code(), Some(2));
    bound_to(&root, &spec, TEAM_ID);
    let root = place("stop-loose");
    let spec = spawned(&root);
    let out = hook("agent-stop", &root, &stop_in(&root, TEAM_ID));
    assert_eq!((out.status.code(), out.stdout.len()), (Some(0), 0));
    let cause = "係の記録の隣の meta.json が読めない";
    assert_eq!(String::from_utf8_lossy(&out.stderr), line(TEAM_ID, cause));
    let log = read(&root, "drafts/.agents/unbound.jsonl");
    assert_eq!(log.lines().count(), 1);
    assert!(
        log.contains(&format!(r#""cause":"{cause}","event":"SubagentStop""#)),
        "{log}"
    );
    assert_eq!(read(&root, "drafts/w241x/spec.json"), spec);
}

/// 出す物が w/ からの相対の頼みは通し、w/ で始まる字・/ で始まる字・.. の段を持つ字を 1 つ持つ頼みは、
/// 断った字と、ほかの 3 行のままで出す物の行だけを直した頭の見本で断り、何も書かない。
#[test]
fn agbn_spawn_refuses_outputs_outside_w() {
    let good = place("outs-good");
    let out = hook("agent-spawn", &good, &seat("PreToolUse", GOOD, ""));
    assert_eq!((out.status.code(), out.stdout.len()), (Some(0), 0));
    assert!(good.join("drafts/w241x/spec.json").is_file());
    let fixed = r#"\n予算: token 250000\n組み: なし\n対象: t3-hub.87.35\n出す物: notes.md,patch/ag.patch"}}"#;
    for (case, bad) in [
        ("outs-w", "w/notes.md"),
        ("outs-abs", "/d/w241x/w/notes.md"),
        ("outs-up", "../w236a/w/notes.md"),
    ] {
        let root = place(case);
        let out = hook(
            "agent-spawn",
            &root,
            &seat("PreToolUse", &format!("{bad},patch/ag.patch"), ""),
        );
        let text = String::from_utf8_lossy(&out.stdout).into_owned();
        assert_eq!(out.status.code(), Some(0), "{case}");
        assert!(
            text.contains(r#""permissionDecision":"deny""#),
            "{case} {text}"
        );
        let what = format!("（出す物の字 {bad} が出力の dir w/ からの相対でない");
        assert!(text.contains(&what), "{case} {text}");
        assert!(text.trim_end().ends_with(fixed), "{case} {text}");
        assert_eq!(
            fs::read_dir(root.join("drafts")).unwrap().count(),
            0,
            "{case}"
        );
    }
}

/// w/ からの相対の字はそのまま、w/ で始まる字は頭の w/ を外し、/ で始まるか .. の段を持つ字は最後の /w/ の後か末の段にする。
#[test]
fn agbn_mend_keeps_relative_items_and_fixes_each_stray_form() {
    let items: Vec<String> = [
        "notes.md",
        "patch/x.patch",
        "x/w/y.md",
        "w/notes.md",
        "w/w/a.md",
        "/d/w241x/w/patch/x.patch",
        "/etc/hosts",
        "../w236a/w/notes.md",
        "a/../b.md",
        "/d/w/../c.md",
    ]
    .map(String::from)
    .to_vec();
    assert_eq!(astray(&items), items[3..].to_vec());
    let want = [
        "notes.md",
        "patch/x.patch",
        "x/w/y.md",
        "notes.md",
        "a.md",
        "patch/x.patch",
        "hosts",
        "notes.md",
        "b.md",
        "c.md",
    ];
    for (item, want) in items.iter().zip(want) {
        assert_eq!(mend(item), want, "{item}");
    }
}
