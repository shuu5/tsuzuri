//! tool の呼びの組の後の配達の hook の境界の歯（行 f-deliver-tool・接頭辞 fdlt_）。
//! 偽の git・tailnet の道具・bd・bdw は作業場の sh の script（撃たれた引数を記録する）で、tz は作業場を PATH の頭に足して撃つ。
//! board は同じ process の thread で立てた読むだけの server か 127.0.0.1 の受け手で、偽の git が port を出す（tailnet の道具は落ちる）。

use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tsuzuri_boundary::hook::deliver_tool::{
    self, Args, MARK_BUDGET, UNRECEIVED_PATH, USAGE, fetch, unreceived,
};
use tsuzuri_boundary::hook::question_signal::{REACH, WAIT};
use tsuzuri_boundary::server::ledger::{BD, BD_ARGS, BD_TIMEOUT};
use tsuzuri_boundary::server::{Config, Server, ruling};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BDW, LedgerWrite};
use tsuzuri_contract::surface::RulingId;
use tsuzuri_contract::wire;
use tsuzuri_core::delivery::{
    CONTEXT_CAP, Route, Said, TOOL_EVENT, context_for, mark_line, unmarked,
};

const AGENT_INPUT: &str = r#"{"session_id":"s-1","hook_event_name":"PostToolBatch","agent_id":"a1","agent_type":"t"}"#;
const TYPE_INPUT: &str = r#"{"session_id":"s-1","hook_event_name":"PostToolBatch","agent_type":"t"}"#;
const BATCH_INPUT: &str = r#"{"session_id":"s-1","hook_event_name":"PostToolBatch","tool_calls":[]}"#;

const STAMP: &str = "2026-09-30T01:00:00Z";

const T1: &str = "fx-t.1:20260930T0101Z-1";
const T2: &str = "fx-t.2:20260930T0102Z-1";
const T3: &str = "fx-t.3:20260930T0103Z-1";

fn rid(id: &str) -> RulingId {
    RulingId::new(id).expect("裁定の id")
}

/// 台帳の bead 1 本の JSON（notes は行を改行でつなぐ）。
fn bead(id: &str, notes: &[String]) -> String {
    let text = |s: &str| wire::encode(&s).expect("字の電文");
    format!(
        "{{\"id\":{},\"title\":{},\"status\":\"closed\",\"priority\":2,\"issue_type\":\"task\",\"created_at\":\"{STAMP}\",\"updated_at\":\"{STAMP}\",\"labels\":[\"intake:question\"],\"notes\":{}}}",
        text(id),
        text(&format!("問い — {id}")),
        text(&notes.join("\n")),
    )
}

fn ruling_line(id: &str, question: &str, verbatim: &str) -> String {
    format!("裁定 id = {id}・問い = {question}・逐語 = {verbatim}")
}

fn mark(id: &str, route: Route) -> String {
    mark_line(&rid(id), route, "20260930T0110Z")
}

fn ledger(beads: Vec<String>) -> String {
    format!("[\n{}\n]\n", beads.join(",\n"))
}

/// fx-t.1 と fx-t.3 は印の無い裁定・fx-t.2 は停止の印の在る裁定の台帳（逐語は `first` が fx-t.1 の字）。
fn pending_ledger(first: &str) -> String {
    ledger(vec![
        bead("fx-t.1", &[ruling_line(T1, "fx-t.1", first)]),
        bead(
            "fx-t.2",
            &[ruling_line(T2, "fx-t.2", "二の答え"), mark(T2, Route::Stop)],
        ),
        bead("fx-t.3", &[ruling_line(T3, "fx-t.3", "三の答え")]),
    ])
}

/// fx-t.1 と fx-t.2 と fx-t.3 の全部に印が在る台帳。
fn marked_ledger() -> String {
    ledger(
        [("fx-t.1", T1), ("fx-t.2", T2), ("fx-t.3", T3)]
            .iter()
            .map(|(q, id)| bead(q, &[ruling_line(id, q, "答え"), mark(id, Route::Stop)]))
            .collect(),
    )
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 撃たれた回ごとに argv を `<log>/<name>.<回>.args` に、回の数を `<log>/<name>.count` に書き、
/// `<log>/<name>.fail` が在れば rc 1 で終わり、無ければ `then` を撃つ script。
fn recorder(path: &Path, log: &Path, name: &str, then: &str) {
    let log = log.display();
    script(
        path,
        &format!(
            "n=$(( $(cat '{log}/{name}.count' 2>/dev/null || echo 0) + 1 ))\n\
             echo \"$n\" > '{log}/{name}.count'\n\
             for a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{log}/{name}.'\"$n\"'.args'\n\
             if [ -e '{log}/{name}.fail' ]; then exit 1; fi\n\
             {then}"
        ),
    );
}

/// drop で path を消す守り（歯が通っても落ちても、worktree を模した .git を CARGO_TARGET_TMPDIR の下に残さない）。
struct Tidy(PathBuf);

impl Drop for Tidy {
    fn drop(&mut self) {
        match fs::symlink_metadata(&self.0) {
            Ok(meta) if meta.is_dir() => {
                let _ = fs::remove_dir_all(&self.0);
            }
            _ => {
                let _ = fs::remove_file(&self.0);
            }
        }
    }
}

/// 歯ごとの作業場（repo・記録の置き場・偽の git と tailnet の道具と hook の bd と bdw・server の bd）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
    log: PathBuf,
}

impl Place {
    /// hook の偽の bd と server の偽の bd は同じ `ledger` を出す。
    fn new(name: &str, ledger: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fdlt").join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, files, log) = (root.join("repo"), root.join("files"), root.join("log"));
        for dir in [repo.join(".beads"), files.clone(), log.clone()] {
            fs::create_dir_all(dir).expect("作業場の dir");
        }
        fs::write(files.join("index.html"), "tz").expect("index.html");
        let place = Place {
            root,
            repo,
            files,
            log,
        };
        recorder(
            &place.git(),
            &place.log,
            "git",
            &format!("exec cat '{}'", place.root.join("port").display()),
        );
        recorder(&place.tailnet(), &place.log, "tailscale", "exit 1");
        // 偽の bd は cwd（repo）でなく作業場の out.json を出す。
        recorder(
            &place.bd(),
            &place.log,
            "bd",
            &format!("exec cat '{}'", place.root.join("out.json").display()),
        );
        script(
            &place.root.join("server-bd"),
            &format!("exec cat '{}'", place.root.join("server-out.json").display()),
        );
        recorder(&place.bdw(), &place.log, "bdw", "exit 0");
        place.bd_returns(ledger);
        place.server_returns(ledger);
        place
    }

    /// hook の偽の bd が出す字。
    fn bd_returns(&self, text: &str) {
        fs::write(self.root.join("out.json"), text).expect("偽の bd の出力");
    }

    /// server の偽の bd が出す字。
    fn server_returns(&self, text: &str) {
        fs::write(self.root.join("server-out.json"), text).expect("server の偽の bd の出力");
    }

    fn touch(&self, name: &str) {
        fs::write(self.log.join(name), "").expect("記録の置き場の印");
    }

    fn git(&self) -> PathBuf {
        self.root.join("git")
    }

    fn tailnet(&self) -> PathBuf {
        self.root.join("tailscale")
    }

    fn bd(&self) -> PathBuf {
        self.root.join("bd")
    }

    fn bdw(&self) -> PathBuf {
        self.root.join("bdw")
    }

    /// 偽の git が出す port を置く。
    fn port(&self, port: u16) {
        fs::write(self.root.join("port"), format!("{port}\n")).expect("port");
    }

    /// 読むだけの server を同じ process の thread で立て、その port を偽の git が出すようにする。
    fn serve(&self) -> SocketAddr {
        let config = Config {
            bd: self.root.join("server-bd").into(),
            bdw: self.root.join("bdw").into(),
            read_only: true,
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        };
        let server = Server::bind(&config).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        self.port(addr.port());
        addr
    }

    /// 偽のプログラムが撃たれた回ごとの引数（回の順）。
    fn calls(&self, name: &str) -> Vec<Vec<String>> {
        let count: u32 = fs::read_to_string(self.log.join(format!("{name}.count")))
            .map_or(0, |c| c.trim().parse().expect("回の数"));
        (1..=count)
            .map(|n| {
                fs::read_to_string(self.log.join(format!("{name}.{n}.args")))
                    .expect("記録")
                    .lines()
                    .map(str::to_string)
                    .collect()
            })
            .collect()
    }

    /// hook deliver-tool に --repo と偽の --bd と --bdw を渡す引数。
    fn hook_args(&self, repo: &Path) -> Vec<OsString> {
        vec![
            "hook".into(),
            "deliver-tool".into(),
            "--repo".into(),
            repo.into(),
            "--bd".into(),
            self.bd().into(),
            "--bdw".into(),
            self.bdw().into(),
        ]
    }

    /// tz を作業場を PATH の頭に足して撃ち、標準入力に payload を書いて閉じ、終わりまで待つ。
    fn tz(&self, args: &[OsString], payload: &str) -> Output {
        let path = std::env::var_os("PATH").unwrap_or_default();
        let mut dirs = vec![self.root.clone()];
        dirs.extend(std::env::split_paths(&path));
        let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(args)
            .env("PATH", std::env::join_paths(dirs).expect("PATH"))
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

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8")
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_secs()
}

/// 1 本の要求を受け、頭を返し、`response` を書いて閉じる受け手（別 thread・受けた要求の頭を返す）。
fn receiver(response: String) -> (u16, JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("受け手");
    let port = listener.local_addr().expect("住所").port();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("受ける");
        let mut head = Vec::new();
        let mut byte = [0u8; 1];
        while !head.ends_with(b"\r\n\r\n") {
            if stream.read(&mut byte).expect("要求を読む") == 0 {
                break;
            }
            head.push(byte[0]);
        }
        let _ = stream.write_all(response.as_bytes());
        text(&head)
    });
    (port, handle)
}

fn http(status: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// 受け手を立てて偽の git に port を出させ、unreceived の答えと受け手が受けた要求の頭を返す。
fn ask(place: &Place, response: String) -> (Result<Reading<Vec<RulingId>>, String>, String) {
    let (port, handle) = receiver(response);
    place.port(port);
    let got = unreceived(&place.repo, place.git().as_os_str(), place.tailnet().as_os_str());
    (got, handle.join().expect("受け手の終わり"))
}

/// 受け手の居ない port。
fn dead_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("受け手");
    listener.local_addr().expect("住所").port()
}

type Parse = fn(&[&str]) -> Result<Args, String>;
type Run = fn(&[&str]) -> u8;

const PARSE: Parse = deliver_tool::parse;
const RUN: Run = deliver_tool::run;

#[test]
fn fdlt_parse_args() {
    assert_eq!(
        USAGE,
        "usage: tz hook deliver-tool --repo <dir> [--bd <program>] [--bdw <program>]"
    );
    assert_eq!(MARK_BUDGET, Duration::from_secs(15));
    assert_eq!(
        PARSE(&["--repo", "/r"]),
        Ok(Args {
            repo: PathBuf::from("/r"),
            bd: OsString::from(BD),
            bdw: OsString::from(BDW),
        })
    );
    assert_eq!(
        PARSE(&["--bdw=w", "--repo=/r", "--bd", "b"]),
        Ok(Args {
            repo: PathBuf::from("/r"),
            bd: OsString::from("b"),
            bdw: OsString::from("w"),
        })
    );
    for bad in [
        &[][..],
        &["--bd", "b"],
        &["--repo", "/r", "--git", "g"],
        &["--repo", "/r", "--repo", "/s"],
        &["--repo"],
        &["--repo="],
        &["--repo", "/r", "--bdw="],
        &["/r"],
    ] {
        assert!(PARSE(bad).is_err(), "{bad:?}");
    }
    // 使い方の誤りは標準入力を読まずに 1。
    assert_eq!(RUN(&[]), 1);
}

#[test]
fn fdlt_gate_reads_board() {
    assert_eq!(UNRECEIVED_PATH, "/api/unreceived");
    let place = Place::new("gate", &marked_ledger());
    let head = |port: u16| {
        format!(
            "GET /api/unreceived HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
        )
    };
    let bodies = [
        (r#"{"known":[]}"#, Reading::Known(Vec::new())),
        (
            r#"{"known":["fx-t.1:20260930T0101Z-1"]}"#,
            Reading::Known(vec![rid(T1)]),
        ),
        (r#""unknown""#, Reading::Unknown),
    ];
    for (body, want) in bodies {
        let (got, request) = ask(&place, http("200 OK", body));
        assert_eq!(got, Ok(want), "{body}");
        let port = fs::read_to_string(place.root.join("port")).expect("port");
        assert_eq!(request, head(port.trim().parse().expect("数")), "{body}");
    }
    let git = place.calls("git");
    assert_eq!(git.len(), 3);
    assert_eq!(
        git[0],
        ["-C", &place.repo.display().to_string(), "config", "--get", "tsuzuri.boardport"]
    );
    assert_eq!(place.calls("tailscale")[0], ["status", "--json"]);

    let (got, _) = ask(&place, http("200 OK", "not json"));
    assert!(got.is_err(), "{got:?}");
    let (got, _) = ask(&place, http("500 Internal Server Error", r#"{"known":[]}"#));
    assert!(got.expect_err("500 は Err").contains("500"));
    place.port(dead_port());
    assert!(
        unreceived(&place.repo, place.git().as_os_str(), place.tailnet().as_os_str()).is_err(),
        "受け手の居ない port"
    );
    place.touch("git.fail");
    let err = unreceived(&place.repo, place.git().as_os_str(), place.tailnet().as_os_str())
        .expect_err("git が落ちる");
    assert!(err.contains("tsuzuri.boardport"), "{err}");

    // 状態の行の読めない応答。
    let (port, handle) = receiver("garbage\r\n\r\n".to_string());
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().expect("住所");
    let err = fetch(addr, REACH).expect_err("状態の行が読めない");
    assert!(err.contains("状態の行"), "{err}");
    let _ = handle.join();
}

#[test]
fn fdlt_gate_on_live_route() {
    let place = Place::new("live", &pending_ledger("一の答え"));
    place.serve();
    let got = unreceived(&place.repo, place.git().as_os_str(), place.tailnet().as_os_str());
    assert_eq!(got, Ok(Reading::Known(vec![rid(T1), rid(T3)])));
}

/// 台帳の字の中核の unmarked の Known の列。
fn said_of(ledger: &str) -> Vec<Said> {
    match unmarked(ledger) {
        Reading::Known(said) => said,
        Reading::Unknown => panic!("台帳が読めない"),
    }
}

/// 印の書きの argv（その裁定・tool の呼びの経路・分）。
fn mark_argv(s: &Said, minute: &str) -> Vec<String> {
    LedgerWrite::AppendNotes {
        id: s.question.clone(),
        line: mark_line(&s.ruling, Route::Tool, minute),
    }
    .argv()
}

/// 偽の bdw の撃たれた回が、`said` の順の印の書き（同じ分・撃った前後の分のどちらか）だけか。
fn check_marks(calls: &[Vec<String>], said: &[Said], (before, after): (u64, u64)) {
    assert_eq!(calls.len(), said.len(), "偽の bdw の回: {calls:?}");
    let minutes = [ruling::minute(before), ruling::minute(after)];
    let minute = minutes
        .iter()
        .find(|m| mark_argv(&said[0], m) == calls[0])
        .unwrap_or_else(|| panic!("1 回目の argv が印の書きでない: {:?}", calls[0]));
    for (s, argv) in said.iter().zip(calls) {
        assert_eq!(argv, &mark_argv(s, minute), "全部の回が同じ分");
    }
}

#[test]
fn fdlt_run_delivers_and_marks() {
    let ledger = pending_ledger("一の答え");
    let place = Place::new("run", &ledger);
    place.serve();
    let said = said_of(&ledger);
    assert_eq!(said.len(), 2);
    let want = format!("{}\n", context_for(&said, TOOL_EVENT).expect("答え"));
    let before = now();
    let out = place.tz(&place.hook_args(&place.repo), BATCH_INPUT);
    let after = now();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(text(&out.stdout), want);
    assert!(out.stderr.is_empty(), "{out:?}");
    assert_eq!(place.calls("bd"), [BD_ARGS.map(str::to_string).to_vec()]);
    check_marks(&place.calls("bdw"), &said, (before, after));

    // 上限ちょうどの逐語の裁定は写し、次の裁定を名指し、その次の裁定には印を置かない。
    let ledger = ledger_of_three(&"あ".repeat(CONTEXT_CAP));
    let place = Place::new("run-cap", &ledger);
    place.serve();
    let said = said_of(&ledger);
    assert_eq!(said.len(), 3);
    let before = now();
    let out = place.tz(&place.hook_args(&place.repo), BATCH_INPUT);
    let after = now();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(
        text(&out.stdout),
        format!("{}\n", context_for(&said, TOOL_EVENT).expect("答え"))
    );
    check_marks(&place.calls("bdw"), &said[..2], (before, after));

    // 偽の bdw が落ちても、答えは同じで、落ちた裁定の id を含む行を標準エラーに書く。
    let ledger = pending_ledger("一の答え");
    let place = Place::new("run-bdw-fails", &ledger);
    place.serve();
    place.touch("bdw.fail");
    let said = said_of(&ledger);
    let out = place.tz(&place.hook_args(&place.repo), BATCH_INPUT);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(
        text(&out.stdout),
        format!("{}\n", context_for(&said, TOOL_EVENT).expect("答え"))
    );
    assert_eq!(place.calls("bdw").len(), 2);
    let err = text(&out.stderr);
    let lines: Vec<&str> = err.lines().collect();
    assert_eq!(lines.len(), 2, "{err}");
    assert!(lines[0].contains(T1) && lines[1].contains(T3), "{err}");

    assert!(WAIT * 2 + REACH * 3 + BD_TIMEOUT + MARK_BUDGET < Duration::from_secs(30));
}

/// fx-t.1 の逐語が `first`・fx-t.2 と fx-t.3 が小さい 3 つの印の無い裁定の台帳。
fn ledger_of_three(first: &str) -> String {
    ledger(vec![
        bead("fx-t.1", &[ruling_line(T1, "fx-t.1", first)]),
        bead("fx-t.2", &[ruling_line(T2, "fx-t.2", "二の答え")]),
        bead("fx-t.3", &[ruling_line(T3, "fx-t.3", "三の答え")]),
    ])
}

/// 静かな形の結果（rc 0・標準出力が 0 byte）を確かめ、標準エラーの行を返す。
fn quiet(place: &Place, payload: &str, label: &str) -> Vec<String> {
    let out = place.tz(&place.hook_args(&place.repo), payload);
    assert_eq!(out.status.code(), Some(0), "{label}: {out:?}");
    assert!(out.stdout.is_empty(), "{label}: {out:?}");
    text(&out.stderr).lines().map(str::to_string).collect()
}

/// 標準エラーがこの hook の 1 行だけ。
fn one_line(lines: &[String], label: &str) {
    assert_eq!(lines.len(), 1, "{label}: {lines:?}");
    assert!(lines[0].starts_with("tz hook deliver-tool: "), "{label}: {lines:?}");
}

#[test]
fn fdlt_run_quiet_paths() {
    // 本体でない入力は子 process を撃たない。
    let place = Place::new("quiet-agent", &pending_ledger("一の答え"));
    place.serve();
    for payload in [AGENT_INPUT, TYPE_INPUT, "not json", ""] {
        let lines = quiet(&place, payload, payload);
        assert!(lines.is_empty(), "{payload}: {lines:?}");
        for name in ["git", "bd", "bdw"] {
            assert!(place.calls(name).is_empty(), "{payload}: 偽の {name} を撃つ");
        }
    }

    // git の worktree（.git が file）。
    let wt = place.root.join("wt");
    fs::create_dir_all(&wt).expect("worktree の形の dir");
    let git = wt.join(".git");
    let _tidy = Tidy(git.clone());
    fs::write(&git, format!("gitdir: {}\n", place.root.join("other").display())).expect(".git の file");
    let out = place.tz(&place.hook_args(&wt), BATCH_INPUT);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty() && out.stderr.is_empty(), "{out:?}");
    for name in ["git", "bd", "bdw"] {
        assert!(place.calls(name).is_empty(), "worktree で偽の {name} を撃つ");
    }

    // 口が Known の空の列: git だけを 1 回撃つ。
    let place = Place::new("quiet-empty", &marked_ledger());
    place.serve();
    let lines = quiet(&place, BATCH_INPUT, "empty");
    assert!(lines.is_empty(), "{lines:?}");
    assert_eq!(place.calls("git").len(), 1);
    assert!(place.calls("bd").is_empty() && place.calls("bdw").is_empty());

    // 受け手の居ない port と、口が Unknown の台帳: bd を撃たず標準エラーは 1 行。
    let place = Place::new("quiet-dead", &pending_ledger("一の答え"));
    place.port(dead_port());
    let lines = quiet(&place, BATCH_INPUT, "dead");
    one_line(&lines, "dead");
    assert!(place.calls("bd").is_empty() && place.calls("bdw").is_empty());
    let place = Place::new("quiet-unknown", &pending_ledger("一の答え"));
    place.server_returns(r#"{"not":"an array"}"#);
    place.serve();
    let lines = quiet(&place, BATCH_INPUT, "unknown");
    one_line(&lines, "unknown");
    assert!(place.calls("bd").is_empty() && place.calls("bdw").is_empty());

    // hook の bd が落ちる: bd を 1 回撃ち、標準エラーは 1 行で、bdw を撃たない。
    let place = Place::new("quiet-bd-fails", &pending_ledger("一の答え"));
    place.serve();
    place.touch("bd.fail");
    let lines = quiet(&place, BATCH_INPUT, "bd-fails");
    one_line(&lines, "bd-fails");
    assert_eq!(place.calls("bd").len(), 1);
    assert!(place.calls("bdw").is_empty());

    // 口が印の無い裁定を返しても hook の読んだ台帳の全部に印が在れば、bd を 1 回撃つだけ。
    let place = Place::new("quiet-marked", &marked_ledger());
    place.server_returns(&pending_ledger("一の答え"));
    place.serve();
    let lines = quiet(&place, BATCH_INPUT, "marked");
    assert!(lines.is_empty(), "{lines:?}");
    assert_eq!(place.calls("bd").len(), 1);
    assert!(place.calls("bdw").is_empty());
}

#[test]
fn fdlt_usage_errors_are_one() {
    let place = Place::new("usage", &marked_ledger());
    let file = place.root.join("file.txt");
    fs::write(&file, "x").expect("dir でない file");
    let repo: OsString = place.repo.clone().into();
    let cases: Vec<Vec<OsString>> = vec![
        vec!["hook".into(), "deliver-tool".into()],
        vec![
            "hook".into(),
            "deliver-tool".into(),
            "--repo".into(),
            repo,
            "--nope".into(),
        ],
        vec![
            "hook".into(),
            "deliver-tool".into(),
            "--repo".into(),
            file.into(),
        ],
        vec![
            "hook".into(),
            "deliver-tool".into(),
            "--nope".into(),
            "x".into(),
        ],
    ];
    for args in &cases {
        let out = place.tz(args, BATCH_INPUT);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
        assert!(out.stdout.is_empty(), "{args:?}: {out:?}");
        let err = text(&out.stderr);
        assert!(err.starts_with("tz hook deliver-tool: "), "{args:?}: {err}");
        assert!(err.contains(USAGE), "{args:?}: {err}");
        assert!(place.calls("git").is_empty(), "{args:?}: 偽の git を撃つ");
        assert!(place.calls("bd").is_empty(), "{args:?}: 偽の bd を撃つ");
    }
}
