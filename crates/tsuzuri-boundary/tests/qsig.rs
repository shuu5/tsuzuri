//! 問いの合図の送り手の hook の歯（行 e-signal-send・接頭辞 qsig_・要件 NFR2・規則の行 R-21）。
//! 入力の読みと送り先の列は関数を直に撃つ。送りは作業場（CARGO_TARGET_TMPDIR の下の qsig）に置く sh の script を
//! 偽の git と偽の tailnet の道具にし、127.0.0.1 の空き port の受け手の thread か実物の server に向ける。
//! status の住所は文書用の住所（どこにも届かない）だけを書く。hooks.json は stage の json で読む（serde_json を使わない）。

use std::ffi::OsStr;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use tsuzuri_boundary::hook::question_signal::{
    CREATED, REACH, USAGE, WAIT, parse, places, question, send, signal,
};
use tsuzuri_boundary::server::events::{NUDGE_PATH, NUDGED};
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_boundary::stage::json;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::surface::QuestionNudge;
use tsuzuri_contract::wire;

/// hooks.json の PostToolUse の command の字（門の command の question-gate を question-signal に替えた字）。
const COMMAND: &str = "[ -e \"$CLAUDE_PLUGIN_ROOT/scribe2-runner\" ] || \"$CLAUDE_PROJECT_DIR\"/target/debug/tz hook question-signal --repo \"$CLAUDE_PROJECT_DIR\"";

/// 問いの起票の command。
const ASK: &str = "bdw create --title 'q' --labels intake:question";

/// 合図の返りの上限。
const WITHIN: Duration = Duration::from_secs(3);

/// runner の印の file の名。
const MARK_NAME: &str = "scribe2-runner";

/// 歯ごとの作業場（前の歯の残りを消して作り直す）。
fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("qsig").join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("作業場");
    root
}

/// PostToolUse の hook の入力（stdout が None なら tool_response を持たない）。
fn payload(tool: &str, command: &str, stdout: Option<&str>) -> String {
    let response = stdout.map_or(String::new(), |s| {
        format!(
            ",\"tool_response\":{{\"stdout\":{},\"stderr\":\"\",\"interrupted\":false,\"isImage\":false}}",
            json::escape(s)
        )
    });
    format!(
        "{{\"session_id\":\"s\",\"hook_event_name\":\"PostToolUse\",\"tool_name\":{},\"tool_input\":{{\"command\":{}}}{response}}}",
        json::escape(tool),
        json::escape(command)
    )
}

/// 起票の行。
fn created(id: &str) -> String {
    format!("✓ {CREATED}{id}\n")
}

/// 撃たれると引数を 1 行ずつ `name.rec` に足し、`out` を出して `rc` で終わる sh の script。
fn tool(dir: &Path, name: &str, out: &str, rc: u8) -> PathBuf {
    let path = dir.join(name);
    let rec = dir.join(format!("{name}.rec"));
    fs::write(
        &path,
        format!(
            "#!/bin/sh\nfor a in \"$@\"; do printf '%s\\n' \"$a\" >> '{}'; done\nprintf '%s' '{out}'\nexit {rc}\n",
            rec.display()
        ),
    )
    .expect("偽の道具");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("偽の道具の権限");
    path
}

/// 道具の引数の記録（撃たれていなければ None）。
fn rec(dir: &Path, name: &str) -> Option<String> {
    fs::read_to_string(dir.join(format!("{name}.rec"))).ok()
}

/// tailnet の道具の status（TailscaleIPs が `ips`）。
fn status(ips: &[&str]) -> String {
    let ips: Vec<String> = ips.iter().map(|ip| json::escape(ip)).collect();
    format!(
        "{{\"Self\":{{\"DNSName\":\"seat.example.\",\"TailscaleIPs\":[{}]}}}}",
        ips.join(",")
    )
}

/// 127.0.0.1 の空き port で要求を 1 本読み、`code` と本文 nudged で答える受け手（要求の字を返す）。
fn receiver(code: u16) -> (u16, JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("受け手");
    let port = listener.local_addr().expect("受け手の住所").port();
    let handle = thread::spawn(move || {
        let (mut s, _) = listener.accept().expect("受ける");
        s.set_read_timeout(Some(Duration::from_secs(5)))
            .expect("timeout");
        let mut raw = Vec::new();
        let mut buf = [0u8; 1024];
        loop {
            let text = String::from_utf8_lossy(&raw).into_owned();
            if let Some((head, body)) = text.split_once("\r\n\r\n") {
                let len = head
                    .lines()
                    .find_map(|l| l.strip_prefix("Content-Length: "))
                    .and_then(|n| n.parse::<usize>().ok())
                    .unwrap_or(0);
                if body.len() >= len {
                    break;
                }
            }
            match s.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => raw.extend_from_slice(&buf[..n]),
                Err(e) => panic!("要求を読む: {e}"),
            }
        }
        let reply = format!(
            "HTTP/1.1 {code} X\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{NUDGED}",
            NUDGED.len()
        );
        s.write_all(reply.as_bytes()).expect("答える");
        String::from_utf8_lossy(&raw).into_owned()
    });
    (port, handle)
}

/// 受け手の居ない 127.0.0.1 の port。
fn dead_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("空き port");
    listener.local_addr().expect("住所").port()
}

#[test]
fn qsig_id_from_payload() {
    assert_eq!(CREATED, "Created issue: ");
    let id = |s: &str| Some(BeadId::new(s).expect("id"));
    let two = format!("{}{}", created("fx-q.1"), created("fx-q.2"));
    let bad_first = format!("{}{}", created("fx/q.1"), created("fx-q.2"));
    assert_eq!(question(&payload("Bash", ASK, Some(&created("fx-q.1")))), id("fx-q.1"));
    assert_eq!(question(&payload("Bash", ASK, Some(&two))), id("fx-q.1"));
    assert_eq!(question(&payload("Bash", ASK, Some(&bad_first))), id("fx-q.2"));
    let memo = "bdw create --title 'm' --labels intake:memo";
    for (name, input) in [
        ("stdout が空", payload("Bash", ASK, Some(""))),
        ("id の無い行", payload("Bash", ASK, Some("✓ Created issue:\n"))),
        ("memo の起票", payload("Bash", memo, Some(&created("fx-q.1")))),
        ("echo", payload("Bash", "echo hi", Some(&created("fx-q.1")))),
        ("Read", payload("Read", ASK, Some(&created("fx-q.1")))),
        ("tool_response が無い", payload("Bash", ASK, None)),
        ("JSON でない", "not json".to_string()),
    ] {
        assert_eq!(question(&input), None, "{name}: {input}");
    }
}

#[test]
fn qsig_places_order() {
    let at = |ip: &str| SocketAddr::new(ip.parse().expect("住所"), 8120);
    let both = status(&["192.0.2.9", "2001:db8::9"]);
    assert_eq!(
        places(Some(&both), 8120),
        [at("192.0.2.9"), at("2001:db8::9"), at("127.0.0.1")]
    );
    let lo = status(&["127.0.0.1"]);
    assert_eq!(places(Some(&lo), 8120), [at("127.0.0.1")]);
    let nameless = "{\"Self\":{\"TailscaleIPs\":[\"192.0.2.9\"]}}";
    for s in [None, Some("not json"), Some(nameless)] {
        assert_eq!(places(s, 8120), [at("127.0.0.1")], "{s:?}");
    }
}

#[test]
fn qsig_sends_to_board() {
    let root = place("send");
    let repo = root.join("repo");
    fs::create_dir_all(&repo).expect("repo");
    let (port, handle) = receiver(202);
    let git = tool(&root, "git", &port.to_string(), 0);
    let tailnet = tool(&root, "tailscale", &status(&["192.0.2.9"]), 0);
    let input = payload("Bash", ASK, Some(&created("fx-q.1")));
    let at = Instant::now();
    let got = signal(&repo, git.as_os_str(), tailnet.as_os_str(), &input);
    assert!(at.elapsed() < WITHIN, "{:?}", at.elapsed());
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().expect("住所");
    assert_eq!(got, Ok(Some((addr, 202))));
    let body = "{\"question\":\"fx-q.1\"}";
    let want = format!(
        "POST {NUDGE_PATH} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: 21\r\nConnection: close\r\n\r\n{body}"
    );
    assert_eq!(handle.join().expect("受け手"), want);
    let nudge = QuestionNudge {
        question: BeadId::new("fx-q.1").expect("id"),
    };
    assert_eq!(wire::encode(&nudge).expect("電文"), body);
    assert_eq!(
        rec(&root, "git").expect("git が撃たれない"),
        format!("-C\n{}\nconfig\n--get\ntsuzuri.boardport\n", repo.display())
    );
    assert_eq!(
        rec(&root, "tailscale").expect("tailnet の道具が撃たれない"),
        "status\n--json\n"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn qsig_live_board_accepts() {
    let root = place("live");
    let (repo, files) = (root.join("repo"), root.join("files"));
    fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
    fs::create_dir_all(&files).expect("面の file の置き場");
    fs::write(files.join("index.html"), "tz").expect("index.html");
    let bd = tool(&root, "bd", "[]", 0);
    let config = Config {
        bd: bd.into(),
        ..Config::new(repo, "127.0.0.1:0".parse().expect("bind 先"), files)
    };
    let server = Server::bind(&config).expect("起動");
    let addr = server.local_addr().expect("口の住所");
    thread::spawn(move || server.run());
    let nudge = QuestionNudge {
        question: BeadId::new("fx-q.1").expect("id"),
    };
    let body = wire::encode(&nudge).expect("電文");
    assert_eq!(send(addr, &body, Duration::from_secs(2)), Ok(202));
    assert_eq!(NUDGED, "nudged");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn qsig_quiet_paths() {
    assert_eq!(WAIT, Duration::from_secs(2));
    assert_eq!(REACH, Duration::from_millis(500));
    let root = place("quiet");
    let repo = root.join("repo");
    fs::create_dir_all(&repo).expect("repo");
    let timed = |git: &Path, tailnet: &Path, input: &str| {
        let at = Instant::now();
        let got = signal(&repo, git.as_os_str(), tailnet.as_os_str(), input);
        assert!(at.elapsed() < WITHIN, "{:?}", at.elapsed());
        got
    };
    let ask = payload("Bash", ASK, Some(&created("fx-q.1")));

    // 起票でない入力と起票の行の無い入力は子 process を撃たない。
    let git = tool(&root, "git", "8120", 0);
    let tailnet = tool(&root, "tailscale", &status(&[]), 0);
    for input in [
        payload("Bash", "ls", Some(&created("fx-q.1"))),
        payload("Bash", ASK, Some("done\n")),
    ] {
        assert_eq!(timed(&git, &tailnet, &input), Ok(None), "{input}");
    }
    assert_eq!(rec(&root, "git"), None, "git が撃たれた");
    assert_eq!(rec(&root, "tailscale"), None, "tailnet の道具が撃たれた");

    // boardport が読めない。
    for (out, rc) in [("", 1), ("0", 0), ("x", 0)] {
        let git = tool(&root, "git", out, rc);
        let got = timed(&git, &tailnet, &ask);
        let e = got.expect_err("boardport が読めないのに Ok");
        assert!(e.contains("tsuzuri.boardport"), "{out} {rc}: {e}");
    }

    // 受け手の無い port。
    let git = tool(&root, "git", &dead_port().to_string(), 0);
    assert!(timed(&git, &tailnet, &ask).is_err(), "受け手が無いのに Ok");

    // 受け手が 400 で断る。
    let (port, handle) = receiver(400);
    let git = tool(&root, "git", &port.to_string(), 0);
    let e = timed(&git, &tailnet, &ask).expect_err("400 なのに Ok");
    assert!(e.contains("400"), "{e}");
    handle.join().expect("受け手");
    let _ = fs::remove_dir_all(&root);
}

/// tz の binary を PATH が `path` だけの場で撃ち、`input` を標準入力に書いて終わりを待つ。
fn tz(args: &[&OsStr], path: &Path, input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(args)
        .env("PATH", path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("tz を撃つ");
    let mut stdin = child.stdin.take().expect("標準入力");
    match stdin.write_all(input.as_bytes()) {
        Ok(()) => {}
        Err(e) if e.kind() == ErrorKind::BrokenPipe => {}
        Err(e) => panic!("標準入力を書く: {e}"),
    }
    drop(stdin);
    child.wait_with_output().expect("tz の終わりを待つ")
}

#[test]
fn qsig_exit_codes() {
    assert_eq!(USAGE, "usage: tz hook question-signal --repo <dir>");
    let d = PathBuf::from("d");
    assert_eq!(parse(&["--repo", "d"]), Ok(d.clone()));
    assert_eq!(parse(&["--repo=d"]), Ok(d));
    for rest in [
        &[][..],
        &["--repo"][..],
        &["--repo="][..],
        &["--repo", ""][..],
        &["--x", "d"][..],
        &["--repo", "d", "--repo", "d"][..],
    ] {
        assert!(parse(rest).is_err(), "{rest:?}");
    }

    let root = place("exit");
    let (bin, repo) = (root.join("bin"), root.join("repo"));
    fs::create_dir_all(&bin).expect("bin");
    fs::create_dir_all(&repo).expect("repo");
    tool(&bin, "git", "", 1);
    tool(&bin, "tailscale", "", 1);
    let head = [OsStr::new("hook"), OsStr::new("question-signal")];
    let with_repo = [head[0], head[1], OsStr::new("--repo"), repo.as_os_str()];
    let ask = payload("Bash", ASK, Some(&created("fx-q.1")));

    let out = tz(&with_repo, &bin, &ask);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("tsuzuri.boardport"),
        "{out:?}"
    );

    let out = tz(&head, &bin, &ask);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains(USAGE), "{out:?}");

    let out = tz(&with_repo, &bin, "");
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
    let _ = fs::remove_dir_all(&root);
}

/// hooks.json の PostToolUse の要素 1 つの hook の object の字。
fn post_hook() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugin/hooks/hooks.json");
    let text = fs::read_to_string(&path).expect("hooks.json を読む");
    let hooks = json::member(&text, "hooks").expect("鍵 hooks");
    let post = json::member(hooks, "PostToolUse").expect("鍵 PostToolUse");
    let [entry] = json::items(post).expect("PostToolUse は配列")[..] else {
        panic!("PostToolUse は要素 1 つの配列でない: {post}");
    };
    let matcher = json::member(entry, "matcher").and_then(json::unquote);
    assert_eq!(matcher.as_deref(), Some("Bash"));
    let inner = json::member(entry, "hooks").expect("要素の hooks");
    let [hook] = json::items(inner).expect("要素の hooks は配列")[..] else {
        panic!("要素の hooks は要素 1 つの配列でない: {inner}");
    };
    hook.to_string()
}

#[test]
fn qsig_hooks_post_entry() {
    let hook = post_hook();
    let field = |key: &str| json::member(&hook, key);
    assert_eq!(field("type").and_then(json::unquote).as_deref(), Some("command"));
    assert_eq!(field("async"), Some("true"));
    assert_eq!(field("timeout"), None, "鍵 timeout が在る");
    assert_eq!(field("command").and_then(json::unquote).as_deref(), Some(COMMAND));
}

/// COMMAND を sh -c で撃ち、`input` を標準入力に書いて子の終わりを待つ。
fn run_sh(project: &Path, plugin: &Path, input: &str) -> Output {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(COMMAND)
        .env("CLAUDE_PROJECT_DIR", project)
        .env("CLAUDE_PLUGIN_ROOT", plugin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("sh を撃つ");
    let mut stdin = child.stdin.take().expect("標準入力");
    match stdin.write_all(input.as_bytes()) {
        Ok(()) => {}
        // sh が標準入力を読まずに先に終わると書きは Broken pipe を返しうる。
        Err(e) if e.kind() == ErrorKind::BrokenPipe => {}
        Err(e) => panic!("標準入力を書く: {e}"),
    }
    drop(stdin);
    child.wait_with_output().expect("sh の終わりを待つ")
}

#[test]
fn qsig_command_under_sh() {
    let command = json::unquote(json::member(&post_hook(), "command").expect("鍵 command"));
    assert_eq!(command.as_deref(), Some(COMMAND));
    let root = place("sh");
    let project = root.join("project");
    let bin = project.join("target/debug");
    fs::create_dir_all(&bin).expect("project の dir");
    let (args_rec, stdin_rec) = (root.join("args.rec"), root.join("stdin.rec"));
    let tz = bin.join("tz");
    fs::write(
        &tz,
        format!(
            "#!/bin/sh\nfor a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{}'\ncat > '{}'\n",
            args_rec.display(),
            stdin_rec.display()
        ),
    )
    .expect("偽の tz");
    fs::set_permissions(&tz, fs::Permissions::from_mode(0o755)).expect("偽の tz の権限");
    let plain = root.join("plugin-plain");
    fs::create_dir_all(&plain).expect("印の無い plugin の dir");
    let marked = root.join("plugin-marked");
    fs::create_dir_all(&marked).expect("印の在る plugin の dir");
    fs::write(marked.join(MARK_NAME), "").expect("印");
    let input = payload("Bash", ASK, Some(&created("fx-q.1")));

    // 席: 印が無く tz が在る。
    let out = run_sh(&project, &plain, &input);
    assert_eq!(out.status.code(), Some(0), "席の rc: {out:?}");
    assert!(out.stdout.is_empty(), "席の標準出力: {out:?}");
    let args = fs::read_to_string(&args_rec).expect("tz が撃たれて引数を記録した");
    assert_eq!(
        args,
        format!("hook\nquestion-signal\n--repo\n{}\n", project.display()),
        "tz の引数"
    );
    let stdin = fs::read_to_string(&stdin_rec).expect("tz が標準入力を記録した");
    assert_eq!(stdin, input, "tz の標準入力");

    // runner: 印が在る。
    fs::remove_file(&args_rec).expect("引数の記録を消す");
    fs::remove_file(&stdin_rec).expect("標準入力の記録を消す");
    let out = run_sh(&project, &marked, &input);
    assert_eq!(out.status.code(), Some(0), "runner の rc: {out:?}");
    assert!(out.stdout.is_empty(), "runner の標準出力: {out:?}");
    assert!(!args_rec.exists(), "runner で tz が撃たれた");
    assert!(!stdin_rec.exists(), "runner で tz が撃たれた");
    let _ = fs::remove_dir_all(&root);
}
