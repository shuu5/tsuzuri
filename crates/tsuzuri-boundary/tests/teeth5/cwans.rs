//! 窓の側の所見の口と守りの hook の歯（接頭辞 cwans_・設計ノート surface-wave27a 行 cs-answer・受入 AC16 の断りと file）。
//! 歯ごとの作業場（CARGO_TARGET_TMPDIR の下・窓の控えだけを置く）で、tz consult answer を cwd = 作業場・
//! 環境の TZ_CONSULT_ID で撃つ。草稿は fixture tests/fixtures/consult/finding-2/ の complete.json と missing.json。
//! 守りの hook は tz consult guard に標準入力の hook の JSON を渡して撃ち、中核の判じも直に撃つ。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_boundary::server::{events, ruling};
use tsuzuri_contract::consult::{Finding, FindingId, Form, Starter, WindowFile, WindowId};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::finding::check;
use tsuzuri_core::consult::guard::{SEAT_VERBS, judge};

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/consult/finding-2")
        .join(name);
    fs::read_to_string(path).unwrap()
}

/// 窓 cw3 の作業場（窓の控えと findings/ と work/ の添え物 2 つ）。
fn workspace(name: &str) -> PathBuf {
    let ws = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("cwans")
        .join(name)
        .join("consult-cw3");
    let _ = fs::remove_dir_all(&ws);
    for d in [".consult", "findings", "work", "drafts"] {
        fs::create_dir_all(ws.join(d)).unwrap();
    }
    let window = WindowFile {
        id: WindowId::new(3).unwrap(),
        form: Form::Ask,
        topic: None,
        model: "fable".into(),
        effort: "xhigh".into(),
        starter: Starter::Seat,
        uttered: None,
        request: None,
        made: "20261003T1412Z".into(),
    };
    fs::write(
        ws.join(".consult/window.json"),
        wire::encode(&window).unwrap(),
    )
    .unwrap();
    fs::write(ws.join("work/report.md"), "レポート\n").unwrap();
    fs::write(ws.join("work/probe.py"), "print(1)\n").unwrap();
    fs::write(ws.join("drafts/complete.json"), fixture("complete.json")).unwrap();
    fs::write(ws.join("drafts/missing.json"), fixture("missing.json")).unwrap();
    ws
}

fn answer(ws: &Path, id: Option<&str>, args: &[&str]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_tz"));
    c.args(["consult", "answer"]).args(args).current_dir(ws);
    match id {
        Some(id) => c.env("TZ_CONSULT_ID", id),
        None => c.env_remove("TZ_CONSULT_ID"),
    };
    c.output().unwrap()
}

fn guard(args: &[&str], payload: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["consult", "guard"])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // 口は引数で断る時に stdin を読まずに抜けるので、書きの BrokenPipe だけは許す（ほかの誤りは落とす）。
    if let Err(e) = child.stdin.take().unwrap().write_all(payload.as_bytes()) {
        assert_eq!(e.kind(), std::io::ErrorKind::BrokenPipe, "{e}");
    }
    child.wait_with_output().unwrap()
}

fn rc(o: &Output) -> i32 {
    o.status.code().unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8(o.stdout.clone()).unwrap()
}

fn err(o: &Output) -> String {
    String::from_utf8(o.stderr.clone()).unwrap()
}

/// findings/ の file の名（字の順）。
fn names(ws: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(ws.join("findings"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    v.sort();
    v
}

fn fid(k: u32) -> FindingId {
    FindingId::new(WindowId::new(3).unwrap(), k).unwrap()
}

fn read_finding(ws: &Path, k: u32) -> Finding {
    let text = fs::read_to_string(ws.join(format!("findings/cw3-{k}.json"))).unwrap();
    wire::decode(&text).unwrap()
}

fn bash(command: &str) -> String {
    format!("{{\"tool_name\":\"Bash\",\"tool_input\":{{\"command\":{command:?}}}}}")
}

#[test]
fn cwans_complete_written() {
    let ws = workspace("complete");
    let before = ruling::minute(events::now());
    let o = answer(&ws, Some("cw3"), &["drafts/complete.json"]);
    let after = ruling::minute(events::now());
    assert_eq!(rc(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "所見 cw3-1 を置いた\n");
    let got = read_finding(&ws, 1);
    assert!(before <= got.date && got.date <= after, "{}", got.date);
    let want = Finding::new(fid(1), &got.date, check(&fixture("complete.json")).unwrap());
    assert_eq!(got, want);
    assert_eq!((got.id, got.window), (fid(1), WindowId::new(3).unwrap()));
    let o = answer(&ws, Some("cw3"), &["drafts/complete.json"]);
    assert_eq!(out(&o), "所見 cw3-2 を置いた\n");
    assert_eq!(read_finding(&ws, 2).id, fid(2));
    assert_eq!(names(&ws), ["cw3-1.json", "cw3-2.json"]);
}

#[test]
fn cwans_missing_refused() {
    let ws = workspace("missing");
    let o = answer(&ws, Some("cw3"), &["drafts/missing.json"]);
    assert_eq!(rc(&o), 1);
    assert_eq!(out(&o), "欠けた欄: bundle_digest\n");
    let complete = fixture("complete.json");
    let bad = [
        (
            complete.replacen('{', "{\"score\": 1,", 1),
            "知らない欄: score\n",
        ),
        (
            complete.replace("\"rejected\"", "\"adopted\""),
            "候補は 2 つ以上で、採る（adopted）のは 1 つ\n",
        ),
        ("[]".to_string(), "所見の草稿が JSON の object でない\n"),
        (
            complete.replace("work/probe.py", "../probe.py"),
            "添え物の path が作業場からの相対でない: ../probe.py\n",
        ),
    ];
    for (text, want) in bad {
        fs::write(ws.join("drafts/bad.json"), text).unwrap();
        let o = answer(&ws, Some("cw3"), &["drafts/bad.json"]);
        assert_eq!((rc(&o), out(&o)), (1, want.to_string()));
    }
    assert!(names(&ws).is_empty());
}

#[test]
fn cwans_attachments_under_workspace() {
    let ws = workspace("attach");
    fs::remove_file(ws.join("work/report.md")).unwrap();
    let o = answer(&ws, Some("cw3"), &["drafts/complete.json"]);
    assert_eq!(rc(&o), 1);
    assert_eq!(
        out(&o),
        "添え物が作業場の下の file でない: work/report.md\n"
    );
    fs::write(ws.join("work/report.md"), "レポート\n").unwrap();
    fs::remove_file(ws.join("work/probe.py")).unwrap();
    let o = answer(&ws, Some("cw3"), &["drafts/complete.json"]);
    assert_eq!(rc(&o), 1);
    assert_eq!(out(&o), "添え物が作業場の下の file でない: work/probe.py\n");
    std::os::unix::fs::symlink("/etc/hostname", ws.join("work/probe.py")).unwrap();
    let o = answer(&ws, Some("cw3"), &["drafts/complete.json"]);
    assert_eq!((rc(&o), out(&o).contains("work/probe.py")), (1, true));
    fs::remove_file(ws.join("work/probe.py")).unwrap();
    fs::create_dir(ws.join("work/probe.py")).unwrap();
    assert_eq!(rc(&answer(&ws, Some("cw3"), &["drafts/complete.json"])), 1);
    fs::remove_dir(ws.join("work/probe.py")).unwrap();
    std::os::unix::fs::symlink(ws.join("work/report.md"), ws.join("work/probe.py")).unwrap();
    let o = answer(&ws, Some("cw3"), &["drafts/complete.json"]);
    assert_eq!(rc(&o), 0, "作業場の下を指す symlink は通す: {}", out(&o));
    assert_eq!(names(&ws), ["cw3-1.json"]);
}

#[test]
fn cwans_window_env_and_numbers() {
    let ws = workspace("env");
    let draft = ["drafts/complete.json"];
    assert_eq!(rc(&answer(&ws, None, &draft)), 1);
    assert_eq!(rc(&answer(&ws, Some("cw4"), &draft)), 1);
    assert_eq!(rc(&answer(&ws, Some("3"), &draft)), 1);
    let o = answer(&ws.join("work"), Some("cw3"), &["../drafts/complete.json"]);
    assert_eq!((rc(&o), out(&o)), (1, String::new()));
    assert!(
        err(&o).contains("cwd が窓 cw3 の作業場でない"),
        "{}",
        err(&o)
    );
    assert_eq!(rc(&answer(&ws, Some("cw3"), &[])), 1);
    assert_eq!(rc(&answer(&ws, Some("cw3"), &["drafts/none.json"])), 1);
    assert_eq!(
        rc(&answer(&ws, Some("cw3"), &["drafts/complete.json", "x"])),
        1
    );
    assert!(names(&ws).is_empty());
    // 在る番号は 2 と 10（間が空き、字の順の最大と数の順の最大が違う）。次は数の最大の次。
    fs::write(ws.join("findings/cw3-2.json"), "前の所見\n").unwrap();
    fs::write(ws.join("findings/cw3-10.json"), "前の所見\n").unwrap();
    fs::write(ws.join("findings/cw9-4.json"), "ほかの窓\n").unwrap();
    let o = answer(&ws, Some("cw3"), &draft);
    assert_eq!(out(&o), "所見 cw3-11 を置いた\n");
    for kept in ["findings/cw3-2.json", "findings/cw3-10.json"] {
        assert_eq!(fs::read_to_string(ws.join(kept)).unwrap(), "前の所見\n");
    }
    let o = answer(&ws, Some("cw3"), &draft);
    assert_eq!(out(&o), "所見 cw3-12 を置いた\n");
    assert_eq!(
        names(&ws),
        [
            "cw3-10.json",
            "cw3-11.json",
            "cw3-12.json",
            "cw3-2.json",
            "cw9-4.json"
        ]
    );
}

/// 席の口の 8 つ（字のまま並べる）を、どれも口の名を添えて断り、一続きの命令の中でも断る。
fn refuses_seat_verbs() {
    let verbs = [
        "open", "launch", "watch", "show", "dispose", "list", "close", "guard",
    ];
    for verb in verbs {
        let err = judge(&bash(&format!("/p/tz consult {verb} cw1"))).unwrap_err();
        assert!(err.contains(&format!("tz consult {verb}")), "{err}");
    }
    for bad in [
        "echo a; tz consult launch cw1",
        "tz consult close cw1; echo a",
        "sh -c 'tz consult close cw1'",
        "x && tzw consult show cw1-1",
    ] {
        assert!(judge(&bash(bad)).is_err(), "{bad}");
    }
    assert_eq!(SEAT_VERBS, verbs);
}

#[test]
fn cwans_guard_tools_and_verbs() {
    for tool in [
        "Read",
        "Grep",
        "Glob",
        "Edit",
        "Write",
        "WebSearch",
        "WebFetch",
    ] {
        assert_eq!(
            judge(&format!("{{\"tool_name\":\"{tool}\"}}")),
            Ok(()),
            "{tool}"
        );
    }
    for ok in [
        "ls -la work",
        "/p/tz consult answer drafts/a.json",
        "tzw consult bundle cw3 --topic x",
        "echo consult",
        "git show HEAD",
    ] {
        assert_eq!(judge(&bash(ok)), Ok(()), "{ok}");
    }
    for tool in ["Task", "mcp__x__y", "NotebookEdit", "bash", "Web"] {
        assert!(
            judge(&format!("{{\"tool_name\":\"{tool}\"}}")).is_err(),
            "{tool}"
        );
    }
    refuses_seat_verbs();
    let unsandboxed = "{\"tool_name\":\"Bash\",\"tool_input\":{\"command\":\"ls\",\"dangerouslyDisableSandbox\":true}}";
    assert!(judge(unsandboxed).is_err());
    assert_eq!(judge(&unsandboxed.replace("true", "false")), Ok(()));
    for bad in ["", "not json", "[]", "{}", "{\"tool_name\":3}"] {
        assert!(judge(bad).is_err(), "{bad}");
    }
}

#[test]
fn cwans_guard_exit_codes() {
    let o = guard(&[], &bash("tz consult answer drafts/a.json"));
    assert_eq!(
        (rc(&o), out(&o), err(&o)),
        (0, String::new(), String::new())
    );
    let o = guard(&[], &bash("tz consult open --by seat"));
    assert_eq!(rc(&o), 2);
    assert!(
        err(&o).starts_with("相談の窓の守り: 窓から tz consult open は撃てない"),
        "{}",
        err(&o)
    );
    assert_eq!(rc(&guard(&[], "{\"tool_name\":\"Task\"}")), 2);
    assert_eq!(rc(&guard(&[], "")), 2);
    assert_eq!(rc(&guard(&["--x"], "{\"tool_name\":\"Read\"}")), 2);
}
