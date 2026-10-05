//! 席の hook の相談の拾いの歯（行 cs-hooks・接頭辞 cwhok_・判断の記録 ADR-29 決定 (8)）。
//! 偽の git は鍵 tsuzuri.draftsdir に作業場の drafts.txt を、鍵 tsuzuri.boardport に port の file を出し（無ければ rc 1）、
//! 偽の tailnet の道具は落ちる（住所は 127.0.0.1 だけ）。board は同じ process の thread の受け手で、path ごとに決めた本文を返す。
//! 偽の bd は作業場の out.json を出し、偽の bdw は撃たれた回の argv を記録する。tz は作業場を PATH の頭に足して撃つ。
#![cfg(test)]

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use crate::common::TZW;
use tsuzuri_boundary::consult::watch::{ALIVE, FRESH};
use tsuzuri_boundary::hook::consult::{PICK_BUDGET, pick};
use tsuzuri_boundary::hook::question_signal::{REACH, WAIT};
use tsuzuri_boundary::hook::{deliver_tool, stop};
use tsuzuri_boundary::server::ledger::BD_TIMEOUT;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::consult::{ConsultUnreceived, FindingId, RequestId};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::surface::RulingId;
use tsuzuri_contract::wire;
use tsuzuri_core::consult::pickup::{block_with, context_with};
use tsuzuri_core::delivery::{Pending, block, context_for};

const STOP: &str = r#"{"session_id":"s-1","hook_event_name":"Stop","stop_hook_active":false}"#;
const STOP_AGAIN: &str = r#"{"session_id":"s-1","hook_event_name":"Stop","stop_hook_active":true}"#;
const PROMPT: &str =
    r#"{"session_id":"s-1","hook_event_name":"UserPromptSubmit","prompt":"続けて"}"#;
const BATCH: &str = r#"{"session_id":"s-1","hook_event_name":"PostToolBatch"}"#;
const FINDING_LINE: &str = "相談: 窓 cw1 の所見 cw1-1 が届いた（/x/tzw consult show cw1-1 --via hook）・「/x/tzw consult watch」を背景で置き直す";
const REQUEST_LINE: &str = "相談: 頼み rq-20261003T1412Z-1 が届いた（/x/tzw consult open --request rq-20261003T1412Z-1 --by button）・「/x/tzw consult watch」を背景で置き直す";
const MISSING_LINE: &str = "相談: 見張りが居ない（「/x/tzw consult watch」を背景で置く）";

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

fn read_fixture(rel: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(rel),
    )
    .unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 受けの無い所見 cw1-1 と頼み rq-20261003T1412Z-1。
fn waiting() -> ConsultUnreceived {
    ConsultUnreceived {
        findings: vec![FindingId::parse("cw1-1").expect("所見の id")],
        requests: vec![RequestId::parse("rq-20261003T1412Z-1").expect("頼みの id")],
    }
}

fn known(w: ConsultUnreceived) -> String {
    wire::encode(&Reading::Known(w)).expect("電文")
}

fn empty() -> String {
    known(ConsultUnreceived {
        findings: Vec::new(),
        requests: Vec::new(),
    })
}

/// 歯ごとの作業場（repo・起草の置き場・記録の置き場・偽の道具・受け手の要求の頭の列）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    drafts: PathBuf,
    heads: Arc<Mutex<Vec<String>>>,
}

impl Place {
    /// 起草の置き場を鍵に置き、board は相談の未受けの口に `body`・裁定の未受けの口に空の列を返す。
    fn new(name: &str, body: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cwhok")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, drafts) = (root.join("repo"), root.join("drafts"));
        for dir in [&repo, &drafts, &root.join("log")] {
            fs::create_dir_all(dir).expect("作業場の dir");
        }
        let r = root.display();
        script(
            &root.join("git"),
            &format!(
                "echo \"$*\" >> '{r}/log/git'\ncase \"$*\" in\n*tsuzuri.draftsdir*) exec cat '{r}/drafts.txt' ;;\n*tsuzuri.boardport*) exec cat '{r}/port' ;;\nesac\nexit 1"
            ),
        );
        script(
            &root.join("tailscale"),
            &format!("echo \"$*\" >> '{r}/log/tailscale'\nexit 1"),
        );
        script(&root.join("bd"), &format!("exec cat '{r}/out.json'"));
        script(
            &root.join("bdw"),
            &format!("echo \"$*\" >> '{r}/log/bdw'\nexit 0"),
        );
        let place = Place {
            root,
            repo,
            drafts,
            heads: Arc::new(Mutex::new(Vec::new())),
        };
        place.key(&place.drafts.display().to_string());
        place.bd_returns(&read_fixture("stop/ledger-done.json"));
        place.serve(body);
        place
    }

    /// 偽の git が鍵 tsuzuri.draftsdir に出す字。
    fn key(&self, value: &str) {
        fs::write(self.root.join("drafts.txt"), format!("{value}\n")).expect("鍵の字");
    }

    fn bd_returns(&self, text: &str) {
        fs::write(self.root.join("out.json"), text).expect("偽の bd の出力");
    }

    /// 受け手を立て、相談の未受けの口に `body` を返す（裁定の未受けの口は Known の空の列）。
    fn serve(&self, body: &str) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("受け手");
        let port = listener.local_addr().expect("住所").port();
        fs::write(self.root.join("port"), format!("{port}\n")).expect("port");
        let (heads, body) = (Arc::clone(&self.heads), body.to_string());
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut head = Vec::new();
                let mut byte = [0u8; 1];
                while !head.ends_with(b"\r\n\r\n") && stream.read(&mut byte).unwrap_or(0) == 1 {
                    head.push(byte[0]);
                }
                let head = String::from_utf8_lossy(&head).into_owned();
                let text = if head.starts_with("GET /api/consult/unreceived ") {
                    body.clone()
                } else {
                    r#"{"known":[]}"#.to_string()
                };
                let reply = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                    text.len()
                );
                let _ = stream.write_all(reply.as_bytes());
                heads.lock().expect("頭の列").push(head);
            }
        });
    }

    /// 受け手が受けた要求の 1 行目の列。
    fn asked(&self) -> Vec<String> {
        let heads = self.heads.lock().expect("頭の列");
        heads
            .iter()
            .map(|h| h.lines().next().unwrap_or_default().to_string())
            .collect()
    }

    /// 偽の道具が撃たれた回の引数の行。
    fn calls(&self, name: &str) -> Vec<String> {
        fs::read_to_string(self.root.join("log").join(name))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    /// 見張りの生きている印を `age` 前の更新時刻で置く。
    fn alive(&self, age: Duration) {
        let path = self.drafts.join(ALIVE);
        fs::write(&path, "pid = 1・始まり = 20261003T1412Z\n").expect("印");
        let file = fs::File::options()
            .write(true)
            .open(&path)
            .expect("印を開く");
        file.set_modified(SystemTime::now() - age)
            .expect("印の時刻");
    }

    /// tz hook <口> を作業場を PATH の頭に足して撃つ（TZ_WRAPPER は /x/tzw）。
    fn hook(&self, verb: &str, payload: &str) -> Output {
        let path = std::env::var_os("PATH").unwrap_or_default();
        let mut dirs = vec![self.root.clone()];
        dirs.extend(std::env::split_paths(&path));
        let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(["hook", verb, "--repo"])
            .arg(&self.repo)
            .args(["--bd", "bd"])
            .args(if verb == "deliver" {
                &[][..]
            } else {
                &["--bdw", "bdw"][..]
            })
            .env("PATH", std::env::join_paths(dirs).expect("PATH"))
            .env("TZ_WRAPPER", TZW)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("tz を撃つ");
        let mut stdin = child.stdin.take().expect("標準入力");
        let _ = stdin.write_all(payload.as_bytes());
        drop(stdin);
        child.wait_with_output().expect("tz の終わり")
    }
}

/// 停止の hook の答え（鍵 decision と reason）。
type Block = BTreeMap<String, String>;

/// 入力の時と道具の周の hook の答え（鍵 hookSpecificOutput の下の hookEventName と additionalContext）。
type Context = BTreeMap<String, BTreeMap<String, String>>;

/// 標準出力の字（rc 0 を確かめる・改行を除く）。
fn stdout(out: &Output) -> String {
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    String::from_utf8(out.stdout.clone())
        .expect("UTF-8")
        .trim_end()
        .to_string()
}

/// 停止の答えの JSON（標準出力が空なら None）。
fn block_of(out: &Output) -> Option<Block> {
    let text = stdout(out);
    (!text.is_empty()).then(|| wire::decode(&text).expect("答えは JSON"))
}

/// 文脈の答えの JSON（標準出力が空なら None）。
fn context_of(text: &str) -> Option<Context> {
    (!text.is_empty()).then(|| wire::decode(text).expect("答えは JSON"))
}

fn reason_lines(b: &Block) -> Vec<String> {
    b["reason"].lines().map(str::to_string).collect()
}

/// 停止の答えの形（decision の字 block と reason）。
fn blocked(reason: &str) -> Option<Block> {
    Some(Block::from([
        ("decision".to_string(), "block".to_string()),
        ("reason".to_string(), reason.to_string()),
    ]))
}

/// 文脈の答えの形。
fn context(event: &str, text: &str) -> Option<Context> {
    let inner = BTreeMap::from([
        ("additionalContext".to_string(), text.to_string()),
        ("hookEventName".to_string(), event.to_string()),
    ]);
    Some(Context::from([("hookSpecificOutput".to_string(), inner)]))
}

/// 裁定の 3 つの停止の答え。
fn ruling_block() -> Block {
    wire::decode(&block(&three()).expect("裁定の答え")).expect("JSON")
}

/// 台帳 stop/ledger.json の未配達の 3 つの裁定。
fn three() -> Vec<Pending> {
    [
        ("fx-s.2", "fx-s.2:20260928T0101Z-1"),
        ("fx-s.3", "fx-s.3:20260928T0102Z-1"),
        ("fx-s.4", "fx-s.4:20260928T0105Z-1"),
    ]
    .iter()
    .map(|(q, r)| Pending {
        question: BeadId::new(*q).expect("bead の id"),
        ruling: RulingId::new(*r).expect("裁定の id"),
    })
    .collect()
}

#[test]
fn cwhok_stop_appends_after_rulings() {
    let place = Place::new("stop-rulings", &known(waiting()));
    place.bd_returns(&read_fixture("stop/ledger.json"));
    let got = block_of(&place.hook("stop", STOP)).expect("答え");
    assert_eq!(got["decision"], "block");
    let mut want = reason_lines(&ruling_block());
    want.extend([FINDING_LINE.to_string(), REQUEST_LINE.to_string()]);
    assert_eq!(
        reason_lines(&got),
        want,
        "裁定の見出しと行の後ろに相談の 2 行"
    );
    assert_eq!(place.calls("bdw").len(), 3, "印は未配達の裁定の 3 つだけ");
    assert_eq!(place.asked(), ["GET /api/consult/unreceived HTTP/1.1"]);
}

#[test]
fn cwhok_stop_nudges_when_nothing_waits() {
    let place = Place::new("stop-empty", &empty());
    assert_eq!(block_of(&place.hook("stop", STOP)), blocked(MISSING_LINE));
    let place = Place::new("stop-unknown", r#""unknown""#);
    assert_eq!(block_of(&place.hook("stop", STOP)), blocked(MISSING_LINE));
    let place = Place::new("stop-noport", &empty());
    fs::remove_file(place.root.join("port")).expect("port を消す");
    assert_eq!(block_of(&place.hook("stop", STOP)), blocked(MISSING_LINE));
    assert!(place.calls("bdw").is_empty(), "印を置かない");
}

/// 見張りの印が古ければ拾い、新しければ拾わない（ほかは対照と同じ）。
fn quiet_with_fresh_mark() {
    let place = Place::new("quiet-stale", &known(waiting()));
    place.alive(FRESH + Duration::from_secs(30));
    assert_eq!(
        block_of(&place.hook("stop", STOP)),
        blocked(&format!("{FINDING_LINE}\n{REQUEST_LINE}"))
    );
    let place = Place::new("quiet-fresh", &known(waiting()));
    place.alive(Duration::from_secs(1));
    assert_eq!(block_of(&place.hook("stop", STOP)), None, "見張りが居る");
    assert!(place.asked().is_empty(), "見張りが居る間に board を撃つ");
    place.bd_returns(&read_fixture("stop/ledger.json"));
    let got = stdout(&place.hook("stop", STOP));
    assert_eq!(
        got,
        block(&three()).expect("裁定の答え"),
        "裁定の答えは字のまま"
    );
}

#[test]
fn cwhok_quiet_with_watch_or_without_key() {
    quiet_with_fresh_mark();
    let place = Place::new("quiet-nokey", &known(waiting()));
    fs::remove_file(place.root.join("drafts.txt")).expect("鍵を消す");
    assert_eq!(block_of(&place.hook("stop", STOP)), None, "鍵の無い repo");
    assert_eq!(place.calls("git").len(), 1, "鍵の読みのほかに git を撃つ");
    assert!(place.calls("tailscale").is_empty() && place.asked().is_empty());
    let place = Place::new("quiet-file", &known(waiting()));
    let file = place.root.join("not-dir");
    fs::write(&file, "x").expect("file");
    place.key(&file.display().to_string());
    assert_eq!(
        block_of(&place.hook("stop", STOP)),
        None,
        "鍵の値が dir でない"
    );
    assert!(place.asked().is_empty());
    let place = Place::new("quiet-again", &known(waiting()));
    assert_eq!(
        block_of(&place.hook("stop", STOP_AGAIN)),
        None,
        "続けた後の停止"
    );
    assert!(place.calls("git").is_empty() && place.asked().is_empty());
}

#[test]
fn cwhok_prompt_adds_context() {
    let place = Place::new("prompt", &known(waiting()));
    let got = context_of(&stdout(&place.hook("deliver", PROMPT)));
    assert_eq!(
        got,
        context(
            "UserPromptSubmit",
            &format!("{FINDING_LINE}\n{REQUEST_LINE}")
        )
    );
    let place = Place::new("prompt-empty", &empty());
    let got = context_of(&stdout(&place.hook("deliver", PROMPT)));
    assert_eq!(got, context("UserPromptSubmit", MISSING_LINE));
    let place = Place::new("prompt-fresh", &known(waiting()));
    place.alive(Duration::from_secs(1));
    assert_eq!(stdout(&place.hook("deliver", PROMPT)), "", "見張りが居る");
}

#[test]
fn cwhok_tool_adds_only_waiting() {
    let place = Place::new("tool", &known(waiting()));
    let got = context_of(&stdout(&place.hook("deliver-tool", BATCH)));
    assert_eq!(
        got,
        context("PostToolBatch", &format!("{FINDING_LINE}\n{REQUEST_LINE}"))
    );
    assert!(
        place
            .asked()
            .contains(&"GET /api/consult/unreceived HTTP/1.1".to_string())
    );
    for (name, body) in [
        ("tool-empty", empty()),
        ("tool-unknown", r#""unknown""#.to_string()),
    ] {
        let place = Place::new(name, &body);
        let out = place.hook("deliver-tool", BATCH);
        assert_eq!(
            stdout(&out),
            "",
            "{name}: 見張りを置かせる行を道具の周ごとに足す"
        );
        assert!(out.stderr.is_empty(), "{name}: {out:?}");
    }
}

#[test]
fn cwhok_merge_keeps_ruling_answer() {
    let extra = vec![FINDING_LINE.to_string(), REQUEST_LINE.to_string()];
    let ruling = block(&three());
    assert_eq!(
        block_with(ruling.clone(), &[]),
        ruling,
        "相談の行が無ければ裁定の答えのまま"
    );
    assert_eq!(block_with(None, &[]), None);
    let merged: Block = wire::decode(&block_with(ruling, &extra).expect("答え")).expect("JSON");
    assert_eq!(
        merged,
        blocked(&format!(
            "{}\n{FINDING_LINE}\n{REQUEST_LINE}",
            ruling_block()["reason"]
        ))
        .expect("形")
    );
    assert_eq!(
        block_with(None, &extra[1..]),
        Some(r#"{"decision":"block","reason":"_"}"#.replace('_', REQUEST_LINE))
    );
    let said = Vec::new();
    assert_eq!(context_for(&said, "PostToolBatch"), None);
    let only = context_of(&context_with(None, &extra, "PostToolBatch").expect("答え"));
    assert_eq!(
        only,
        context("PostToolBatch", &format!("{FINDING_LINE}\n{REQUEST_LINE}"))
    );
    let ctx = r#"{"hookSpecificOutput":{"additionalContext":"席に届いた裁定","hookEventName":"UserPromptSubmit"}}"#;
    assert_eq!(
        context_with(Some(ctx.to_string()), &[], "UserPromptSubmit").as_deref(),
        Some(ctx)
    );
    let both = context_of(
        &context_with(Some(ctx.to_string()), &extra[..1], "UserPromptSubmit").expect("答え"),
    );
    assert_eq!(
        both,
        context(
            "UserPromptSubmit",
            &format!("席に届いた裁定\n{FINDING_LINE}")
        )
    );
}

#[test]
fn cwhok_pick_within_budget() {
    assert!(
        PICK_BUDGET + BD_TIMEOUT + stop::MARK_BUDGET < Duration::from_secs(30),
        "停止の hook の timeout 30"
    );
    assert!(
        PICK_BUDGET + BD_TIMEOUT < Duration::from_secs(10),
        "入力の時の hook の timeout 10"
    );
    assert!(
        WAIT * 2 + REACH * 3 + BD_TIMEOUT + deliver_tool::MARK_BUDGET + PICK_BUDGET
            < Duration::from_secs(30),
        "道具の周の hook の timeout 30"
    );
    let place = Place::new("budget", &empty());
    script(&place.root.join("tailscale"), "sleep 10");
    let start = Instant::now();
    let got = pick(
        &place.repo,
        place.root.join("git").as_os_str(),
        place.root.join("tailscale").as_os_str(),
    );
    assert!(
        start.elapsed() < PICK_BUDGET + Duration::from_secs(1),
        "{:?}",
        start.elapsed()
    );
    assert!(got.is_some(), "見張りが居ないので Some");
    let got = pick(
        &place.repo,
        OsStr::new("/nonexistent/git"),
        place.root.join("tailscale").as_os_str(),
    );
    assert_eq!(got, None, "git が撃てなければ鍵の無い repo と同じ");
}
