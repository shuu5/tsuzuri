//! 端末に届かない時の落ちる先と board の URL の歯（行 i-4・接頭辞 relay_）。
//! 端末は fixture の tests/fixtures/stage/terminals.toml の行を lookup で引く（名も宛先も path も偽物）。
//! 偽の ssh・tailnet の道具・git・席の目の Chrome は sh の script で argv を記録する。偽の Chrome の pipe の相手は
//! 歯の中の thread で、mkfifo の 2 つの FIFO で話す（本物の ssh と Chrome と tailnet の道具を撃たず網に出ない）。
#![cfg(test)]

use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt::Debug;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{ChildStdin, ChildStdout, Command as Process};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use tsuzuri_boundary::stage::cdp::{Command, HISTORY, Session};
use tsuzuri_boundary::stage::json;
use tsuzuri_boundary::stage::launch;
use tsuzuri_boundary::stage::pipe::Pipe;
use tsuzuri_boundary::stage::relay::{
    self, ATTACH, CHROME, Eyes, PIPE_SCRIPT, Reach, SHELL, TARGETS,
};
use tsuzuri_boundary::stage::terminal::{self, Terminal};
use tsuzuri_boundary::stage::tunnel::Tunnel;
use tsuzuri_boundary::stage::url::{self, Board, LOOPBACK, STATUS_ARGS, TAILNET};

/// contracts の verify の filter の語のうち、ほかの語を部分の字として含まない最小の語（この行の接頭辞 relay_ は並べない）。
const FILTER_WORDS: &str = concat!(
    "aaround_ accept_ account_ acctcore_ acctdoc_ accthb_ accthome_ acctled_ acctlook_ acctpcore_ ",
    "acctproj_ acctsess_ acctwin_ acctwire_ afocus_ aord_ apop_ askcard_ athr_ batchpanel_ ",
    "bhalf_ board_min_ bport_ brand_ btuck_ cadopt_ cgdom_ cmark_ contract_form_ cround_ ",
    "csled_ denv_ ecache_ flight_ fmark_ frame_ fserve_ fstop_ gapspage_ gfresh_ ",
    "ghb_ glabel_ gnav_ graph_ gsum_ gtuck_ gview_ hbconf_ hbpost_ hbproc_ ",
    "hbroute_ hcard_ hcled_ hcnx_ hcproj_ hcsess_ hfig_ hnunk_ hook_ hruling_ ",
    "hsblock_ hsderive_ hspage_ hsym_ iclose_ ilink_ kcli_ klink_ launch_ lcard_ ",
    "ledgerblock_ lhome_ lspark_ mapview_ mkeys_ mlink_ mstore_ mtree_ nact_ nbatch_ ",
    "ncard_ nextstep_ nodepage_ nstall_ nsum_ nsumw_ ntime_ nxact_ parts_ pclosed_ ",
    "pfold_ pipe_ plimit_ pmore_ pquest_ project_ ptitle_ punmap_ pwhole_ qblock_ ",
    "qgate_ qkey_ question_ rhold_ runsdoc_ saxis_ sclosed_ seatblock_ seatcard_ server_ ",
    "sesplit_ shb_ skeleton_ smore_ stage_ stats_ steady_ sxaxis_ ticker_ tipx_ ",
    "topbar_ tz_ urpanel_ uword_ wstrip_",
);

/// 節の URL（shell の特別な字 ? と & を持つ）。
const URL: &str = "http://127.0.0.1:4173/?a=1&b=2";

/// 節の移り先の URL。
const NEXT: &str = "http://127.0.0.1:4173/next";

/// 節の board の URL（名は偽物・予約の頂の invalid）。
const BOARD_URL: &str = "http://srv-a.tailnet.invalid:4801/";

/// 節の写真の base64（8 byte の PNG の頭と字 fake）。
const PNG: &str = "iVBORw0KGgpmYWtl";

/// 節の DOM の字。
const DOM: &str = "<html><body>eyes</body></html>";

/// 節の偽の tailnet の道具の出力（住所は文書の例の住所で tailnet の住所でない）。
const STATUS: &str = r#"{
  "BackendState": "Running",
  "Self": {
    "HostName": "srv-a",
    "DNSName": "SRV-A.tailnet.invalid.",
    "TailscaleIPs": [
      "192.0.2.7",
      "2001:DB8::7"
    ]
  },
  "Peer": {}
}
"#;

/// 節の status の表（self_name の値）。
const NAMES: [(&str, Option<&str>); 8] = [
    (STATUS, Some("srv-a.tailnet.invalid")),
    (r#"{"Self":{"DNSName":"b-2.x"}}"#, Some("b-2.x")),
    (r#"{"Self":{"DNSName":""}}"#, None),
    (r#"{"Self":{"DNSName":"a..b."}}"#, None),
    (r#"{"Self":{"DNSName":"a b.x."}}"#, None),
    (r#"{"Self":{"HostName":"srv-a"}}"#, None),
    (r#"{"Peer":{"n1":{"DNSName":"srv-b.tailnet.invalid."}}}"#, None),
    ("srv-a.tailnet.invalid", None),
];

/// 節の HOSTS。
const HOSTS: [&str; 6] = [
    "srv-a.tailnet.invalid",
    "srv-a",
    "192.0.2.7",
    "2001:db8::7",
    "127.0.0.1",
    "localhost",
];

/// 節の hosts の表（self_hosts の値）。
const HOSTS_TABLE: [(&str, Option<&[&str]>); 5] = [
    (STATUS, Some(&HOSTS)),
    (
        r#"{"Self":{"DNSName":"b-2.x."}}"#,
        Some(&["b-2.x", "b-2", "127.0.0.1", "localhost"]),
    ),
    (
        r#"{"Self":{"DNSName":"localhost.","TailscaleIPs":["127.0.0.1"]}}"#,
        Some(&["localhost", "127.0.0.1"]),
    ),
    (
        r#"{"Self":{"DNSName":"b-2.x.","TailscaleIPs":["192.0.2.7/32"]}}"#,
        None,
    ),
    (
        r#"{"Self":{"DNSName":"b-2.x.","TailscaleIPs":"192.0.2.7"}}"#,
        None,
    ),
];

/// 節の holds の表。
const HOLDS: [(&str, bool); 12] = [
    (BOARD_URL, true),
    ("http://srv-a:4801/x", true),
    ("http://192.0.2.7:4801/", true),
    ("http://[2001:DB8::7]:4801/q", true),
    ("http://127.0.0.1:4801/", true),
    ("http://LOCALHOST:4801/", true),
    ("https://srv-a.tailnet.invalid:4801/", true),
    ("http://srv-a.tailnet.invalid:4802/", false),
    ("http://srv-b:4801/", false),
    ("http://127.0.0.1:4173/", false),
    ("http://[::1]:4801/", false),
    ("about:blank", false),
];

/// 節の origin の表。
const ORIGINS: [(&str, Option<&str>); 12] = [
    (BOARD_URL, Some("http://srv-a.tailnet.invalid:4801")),
    (
        "http://SRV-A.Tailnet.Invalid.:4801/x/y?q=1#top",
        Some("http://srv-a.tailnet.invalid:4801"),
    ),
    (
        "HTTP://me:pw@srv-a.tailnet.invalid:4801/",
        Some("http://srv-a.tailnet.invalid:4801"),
    ),
    (
        "http://srv-a.tailnet.invalid/",
        Some("http://srv-a.tailnet.invalid:80"),
    ),
    (
        "https://srv-a.tailnet.invalid/",
        Some("https://srv-a.tailnet.invalid:443"),
    ),
    (URL, Some("http://127.0.0.1:4173")),
    ("http://[::1]:4801/", Some("http://[::1]:4801")),
    ("http://[::1]/", Some("http://[::1]:80")),
    ("file:///fake/x.html", None),
    ("javascript:alert(1)", None),
    ("http://:4801/", None),
    ("http://srv-a:99999/", None),
];

/// 節の 5 つの形の外の URL。
const BAD_URLS: [&str; 5] = [
    "javascript:alert(1)",
    "--no-sandbox",
    "",
    "http://a b/",
    "file:///fake/x.html",
];

/// 節の付き方の 6 つの字。
const ATTACHED: [&str; 6] = [
    r#"{"id":1,"method":"Target.getTargets","params":{}}"#,
    r#"{"id":2,"method":"Target.attachToTarget","params":{"targetId":"T1","flatten":true}}"#,
    r#"{"sessionId":"S1","id":1,"method":"Page.enable","params":{}}"#,
    r#"{"sessionId":"S1","id":2,"method":"Page.navigate","params":{"url":"http://127.0.0.1:4173/next"}}"#,
    r#"{"sessionId":"S1","id":3,"method":"Page.captureScreenshot","params":{"format":"png"}}"#,
    r#"{"sessionId":"S1","id":4,"method":"Page.getNavigationHistory","params":{}}"#,
];

/// 読み込みの終わりの event の字。
const LOAD: &str = r#"{"method":"Page.loadEventFired","params":{"timestamp":1}}"#;

/// console の event の字。
const CONSOLE: &str = r#"{"method":"Runtime.consoleAPICalled","params":{"type":"log","args":[]}}"#;

fn fixture() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/stage/terminals.toml");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn term(name: &str) -> Terminal {
    terminal::lookup(&fixture(), name).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn src(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/stage")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

/// 節の自分の board。
fn own() -> Board {
    Board {
        url: BOARD_URL.to_string(),
        hosts: strings(&HOSTS),
        port: 4801,
    }
}

/// open の Err の字（Ok なら落ちる）。
fn refused(got: Result<(Eyes, Session), String>) -> String {
    match got {
        Ok(_) => panic!("open が Ok"),
        Err(e) => e,
    }
}

/// /proc の下の pid が 2 秒の内に居なくなるか状態 Z になるか。
fn gone(pid: &str) -> bool {
    let start = Instant::now();
    loop {
        let state = fs::read_to_string(format!("/proc/{pid}/stat"))
            .ok()
            .map(|stat| {
                stat.rsplit_once(')')
                    .and_then(|(_, rest)| rest.trim_start().chars().next())
            });
        if matches!(state, None | Some(Some('Z'))) {
            return true;
        }
        if start.elapsed() > Duration::from_secs(2) {
            return false;
        }
        thread::sleep(Duration::from_millis(20));
    }
}

/// 偽の ssh の tunnel の回の振る舞い。
#[derive(Clone, Copy)]
enum Ssh {
    Sleep,
    Fail,
}

/// 偽の Chrome の頁の target の出方。
#[derive(Clone, Copy)]
enum Page {
    First,
    Second,
    Never,
}

/// 偽の場（記録の置き場・2 つの FIFO・偽の program・場の字の file）。
struct Field {
    root: PathBuf,
    records: PathBuf,
    input: PathBuf,
    output: PathBuf,
    ssh: PathBuf,
    tailnet: PathBuf,
    git: PathBuf,
    chrome: PathBuf,
    status: PathBuf,
    port: PathBuf,
    mark: PathBuf,
}

impl Field {
    fn new(name: &str, ssh: Ssh) -> Field {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("strelay")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let records = root.join("records");
        fs::create_dir_all(&records).expect("記録の置き場");
        let input = root.join("in.fifo");
        let output = root.join("out.fifo");
        for fifo in [&input, &output] {
            let made = Process::new("mkfifo")
                .arg(fifo)
                .status()
                .expect("mkfifo を撃つ");
            assert!(made.success(), "mkfifo {}", fifo.display());
        }
        let status = root.join("status");
        fs::write(&status, STATUS).expect("場の status");
        let port = root.join("port");
        fs::write(&port, "4801").expect("場の port");
        let mark = root.join("mark");
        let tunnel = match ssh {
            Ssh::Sleep => {
                "for a in \"$@\"; do\n    if [ \"$prev\" = -L ]; then : > \"${a%%:*}\"; fi\n    prev=$a\n  done\n  exec sleep 30"
            }
            Ssh::Fail => "exit 255",
        };
        let ssh = fake(
            &root,
            "ssh",
            &format!("if [ \"$1\" = -N ]; then\n  {tunnel}\nfi\nexit 0\n"),
        );
        let tailnet = fake(
            &root,
            "tailnet",
            &format!(
                "if [ -e '{}' ]; then exit 1; fi\nexec cat '{}'\n",
                mark.display(),
                status.display()
            ),
        );
        let git = fake(&root, "git", &format!("exec cat '{}'\n", port.display()));
        let chrome = fake(
            &root,
            "chrome",
            &format!(
                "printf '%s\\n' \"$$\" > \"$rec/chrome.pid\"\ncat <&3 > '{}' &\nprintf '%s\\n' \"$!\" > \"$rec/cat.pid\"\nexec cat '{}' >&4\n",
                input.display(),
                output.display()
            ),
        );
        Field {
            root,
            records,
            input,
            output,
            ssh,
            tailnet,
            git,
            chrome,
            status,
            port,
            mark,
        }
    }

    /// 偽の program の argv の記録（撃たれた順）。
    fn records(&self, name: &str) -> Vec<Vec<String>> {
        let count: usize = fs::read_to_string(self.records.join(format!("{name}.count")))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        (1..=count)
            .map(|n| {
                let path = self.records.join(format!("{name}-{n}.args"));
                fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
                    .lines()
                    .map(str::to_string)
                    .collect()
            })
            .collect()
    }

    /// 記録の置き場の 1 行の字（pid）。
    fn pid(&self, name: &str) -> String {
        let path = self.records.join(name);
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
            .trim()
            .to_string()
    }
}

/// argv を記録する偽の program（sh の script・権限 0755）。
fn fake(root: &Path, name: &str, body: &str) -> PathBuf {
    let script = format!(
        "#!/bin/sh\nrec='{}'\nn=$(( $(cat \"$rec/{name}.count\" 2>/dev/null || echo 0) + 1 ))\nfor a in \"$@\"; do printf '%s\\n' \"$a\"; done > \"$rec/{name}-$n.args\"\nprintf '%s' \"$n\" > \"$rec/{name}.count\"\n{body}",
        root.join("records").display()
    );
    let path = root.join(name);
    fs::write(&path, script).expect("偽の program");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
    path
}

/// 偽の Chrome の pipe の相手（in.fifo を読み out.fifo へ書き、受けた message を順に記す）。
fn serve(field: &Field, page: Page) -> JoinHandle<Vec<String>> {
    let input = field.input.clone();
    let output = field.output.clone();
    thread::spawn(move || {
        let mut reader = BufReader::new(File::open(&input).expect("in.fifo を開く"));
        let mut writer = OpenOptions::new()
            .write(true)
            .open(&output)
            .expect("out.fifo を開く");
        let mut got = Vec::new();
        let mut now = URL.to_string();
        let mut asks = 0;
        loop {
            let mut buf = Vec::new();
            match reader.read_until(0, &mut buf) {
                Ok(n) if n > 0 && buf.last() == Some(&0) => {
                    buf.pop();
                }
                _ => break,
            }
            let text = String::from_utf8(buf).expect("字の message");
            got.push(text.clone());
            let id = json::member(&text, "id").expect("鍵 id").to_string();
            let method = json::member(&text, "method")
                .and_then(json::unquote)
                .expect("鍵 method");
            let session = json::member(&text, "sessionId").map(str::to_string);
            let ok = format!(r#"{{"id":{id},"result":{{}}}}"#);
            let replies = match method.as_str() {
                "Target.getTargets" => {
                    asks += 1;
                    let shown = match page {
                        Page::First => true,
                        Page::Second => asks >= 2,
                        Page::Never => false,
                    };
                    let mut infos = vec![
                        r#"{"targetId":"W1","type":"service_worker","url":"chrome-extension://x/bg.js"}"#
                            .to_string(),
                    ];
                    if shown {
                        infos.push(format!(
                            r#"{{"targetId":"T1","type":"page","url":{}}}"#,
                            json::escape(&now)
                        ));
                    }
                    vec![format!(
                        r#"{{"id":{id},"result":{{"targetInfos":[{}]}}}}"#,
                        infos.join(",")
                    )]
                }
                "Target.attachToTarget" => vec![
                    r#"{"method":"Target.attachedToTarget","params":{"sessionId":"S1","targetInfo":{"targetId":"T1","type":"page"},"waitingForDebugger":false}}"#
                        .to_string(),
                    format!(r#"{{"id":{id},"result":{{"sessionId":"S1"}}}}"#),
                ],
                "Page.navigate" => {
                    now = json::member(&text, "params")
                        .and_then(|p| json::member(p, "url"))
                        .and_then(json::unquote)
                        .expect("移り先の url");
                    vec![ok, LOAD.to_string()]
                }
                "Page.captureScreenshot" => {
                    vec![format!(r#"{{"id":{id},"result":{{"data":"{PNG}"}}}}"#)]
                }
                "Runtime.evaluate" => vec![format!(
                    r#"{{"id":{id},"result":{{"result":{{"type":"string","value":{}}}}}}}"#,
                    json::escape(DOM)
                )],
                "Runtime.enable" => vec![CONSOLE.to_string(), ok],
                "Page.getNavigationHistory" => vec![format!(
                    r#"{{"id":{id},"result":{{"currentIndex":0,"entries":[{{"id":1,"url":{},"title":""}}]}}}}"#,
                    json::escape(&now)
                )],
                _ => vec![ok],
            };
            for reply in replies {
                let reply = match &session {
                    Some(s) => format!("{{\"sessionId\":{s},{}", &reply[1..]),
                    None => reply,
                };
                let mut bytes = reply.into_bytes();
                bytes.push(0);
                if writer.write_all(&bytes).and_then(|()| writer.flush()).is_err() {
                    return got;
                }
            }
        }
        got
    })
}

/// 受けた message の method の列。
fn methods(got: &[String]) -> Vec<String> {
    got.iter()
        .map(|m| {
            json::member(m, "method")
                .and_then(json::unquote)
                .unwrap_or_default()
        })
        .collect()
}

/// Reach の変種の名と line（wildcard の腕を持たない・変種が足されると組めない）。
fn seen(reach: &Reach) -> &str {
    match reach {
        Reach::Terminal(tunnel) => {
            let _: &Tunnel = tunnel;
            "terminal"
        }
        Reach::Eyes {
            eyes,
            session,
            line,
        } => {
            let _: (&Eyes, &Session) = (eyes, session);
            line
        }
    }
}

/// open の返り。
type Opened = Result<(Eyes, Session), String>;

/// reach の型。
type ReachFn = fn(&OsStr, &OsStr, &Terminal, &Path, &str, Duration) -> Result<Reach, String>;

/// board の型。
type BoardFn = fn(&OsStr, &OsStr, &Path, Duration) -> Result<Board, String>;

fn debug<T: Debug>() {}

#[test]
fn relay_shape_and_consts() {
    let chrome: &str = CHROME;
    assert_eq!(chrome, "google-chrome");
    let shell: &str = SHELL;
    assert_eq!(shell, "sh");
    let script: &str = PIPE_SCRIPT;
    assert_eq!(
        script,
        r#"exec "$0" "$@" 3<&0 4>&1 </dev/null >/dev/null 2>&1"#
    );
    let targets: &str = TARGETS;
    assert_eq!(targets, "Target.getTargets");
    let attach_method: &str = ATTACH;
    assert_eq!(attach_method, "Target.attachToTarget");
    let history: &str = HISTORY;
    assert_eq!(history, "Page.getNavigationHistory");
    let tailnet: &str = TAILNET;
    assert_eq!(tailnet, "tailscale");
    let status_args: [&str; 2] = STATUS_ARGS;
    assert_eq!(status_args, ["status", "--json"]);
    let loopback: [&str; 2] = LOOPBACK;
    assert_eq!(loopback, ["127.0.0.1", "localhost"]);

    let eyes_argv: fn(&Path, &str) -> Vec<String> = relay::eyes_argv;
    let spawn_argv: fn(&OsStr, &Path, &str) -> Vec<OsString> = relay::spawn_argv;
    let reach: ReachFn = relay::reach;
    let open: fn(&OsStr, &Path, &str, Duration) -> Opened = Eyes::open;
    let dir: for<'a> fn(&'a Eyes) -> &'a Path = Eyes::dir;
    let new: fn(ChildStdin, ChildStdout) -> Pipe = Pipe::new;
    let join: fn(&mut Pipe, &str) = Pipe::join;
    let send: fn(&mut Pipe, &str) -> Result<(), String> = Pipe::send;
    let recv: fn(&mut Pipe, Duration) -> Result<Option<String>, String> = Pipe::recv;
    let close: fn(Pipe) -> Result<(), String> = Pipe::close;
    let attach: fn(Pipe, Duration) -> Session = Session::attach;
    let page_url: fn(&mut Session) -> Result<String, String> = Session::url;
    let self_name: fn(&str) -> Option<String> = url::self_name;
    let self_hosts: fn(&str) -> Option<Vec<String>> = url::self_hosts;
    let board_url: fn(&str, u16) -> String = url::board_url;
    let board: BoardFn = url::board;
    let line: fn(&str) -> String = url::line;
    let origin: fn(&str) -> Option<String> = url::origin;
    let holds: fn(&Board, &str) -> bool = Board::holds;
    let see: for<'a> fn(&'a Reach) -> &'a str = seen;
    let _ = (eyes_argv, spawn_argv, reach, open, dir, new, join, send, recv, close);
    let _ = (attach, page_url, self_name, self_hosts, board_url, board, line, origin);
    let _ = (holds, see);
    debug::<Eyes>();
    debug::<Board>();
    let own = own();
    assert_eq!(own.clone(), own);
    let Board { url, hosts, port } = own;
    let _: (String, Vec<String>, u16) = (url, hosts, port);
}

#[test]
fn relay_board_url() {
    for (status, want) in NAMES {
        assert_eq!(url::self_name(status).as_deref(), want, "{status}");
    }
    assert_eq!(url::board_url("srv-a.tailnet.invalid", 4801), BOARD_URL);
    assert_eq!(url::line(BOARD_URL), format!("board の URL {BOARD_URL}"));

    let field = Field::new("board", Ssh::Fail);
    let repo = field.root.clone();
    let repo_text = repo.to_str().expect("repo の dir の字").to_string();
    let five = Duration::from_secs(5);
    let shot = || url::board(field.tailnet.as_os_str(), field.git.as_os_str(), &repo, five);
    assert_eq!(shot(), Ok(own()));
    assert_eq!(field.records("tailnet"), [strings(&["status", "--json"])]);
    assert_eq!(
        field.records("git"),
        [strings(&[
            "-C",
            &repo_text,
            "config",
            "--get",
            "tsuzuri.boardport"
        ])]
    );
    for (port, named) in [(Some("0"), true), (Some("abc"), true), (None, false)] {
        match port {
            Some(port) => fs::write(&field.port, port).expect("場の port"),
            None => fs::remove_file(&field.port).expect("場の port を消す"),
        }
        let err = shot().expect_err("port が読めなければ Err");
        assert!(err.contains("tsuzuri.boardport"), "{port:?}: {err}");
        if named {
            assert!(err.contains(&repo_text), "{port:?}: {err}");
        }
    }
    fs::write(&field.port, "4801").expect("場の port");
    fs::write(&field.status, r#"{"Self":{"HostName":"srv-a"}}"#).expect("場の status");
    let err = shot().expect_err("DNSName が無ければ Err");
    assert!(err.contains("DNSName"), "{err}");
    fs::write(&field.status, STATUS).expect("場の status");
    fs::write(&field.mark, "").expect("印の file");
    let err = shot().expect_err("tailnet の道具が rc 1 なら Err");
    assert!(
        err.contains("tailnet の道具") && err.contains("status --json"),
        "{err}"
    );
}

#[test]
fn relay_origin_table() {
    for (page, want) in ORIGINS {
        assert_eq!(url::origin(page).as_deref(), want, "{page}");
    }
    for (status, want) in HOSTS_TABLE {
        assert_eq!(url::self_hosts(status), want.map(strings), "{status}");
    }
    let own = own();
    for (page, want) in HOLDS {
        assert_eq!(own.holds(page), want, "{page}");
    }
}

#[test]
fn relay_eyes_argv_snapshot() {
    let profile = Path::new("/p/tzst-1-2");
    let want = [
        "--headless=new",
        "--user-data-dir=/p/tzst-1-2",
        "--remote-debugging-pipe",
        "--no-first-run",
        "--no-default-browser-check",
        URL,
    ];
    assert_eq!(relay::eyes_argv(profile, URL), want);
    let mut spawn = vec!["-c", PIPE_SCRIPT, "/fake/bin/eyes"];
    spawn.extend(want);
    let spawn: Vec<OsString> = spawn.into_iter().map(OsString::from).collect();
    assert_eq!(spawn.len(), 9);
    assert_eq!(
        relay::spawn_argv(OsStr::new("/fake/bin/eyes"), profile, URL),
        spawn
    );
}

#[test]
fn relay_eyes_attach_sequence() {
    let field = Field::new("attach", Ssh::Fail);
    let server = serve(&field, Page::First);
    let (eyes, mut session) = Eyes::open(
        field.chrome.as_os_str(),
        &env::temp_dir(),
        URL,
        Duration::from_secs(5),
    )
    .expect("open");
    let dir = eyes.dir().to_path_buf();
    let meta = fs::metadata(&dir).expect("profile の dir");
    assert!(meta.is_dir(), "{}", dir.display());
    assert_eq!(meta.permissions().mode() & 0o777, 0o700);
    session
        .run(&Command::Navigate {
            url: NEXT.to_string(),
        })
        .expect("Navigate");
    let shot = session.run(&Command::Screenshot).expect("Screenshot");
    assert_eq!(shot.len(), 1, "{shot:?}");
    assert!(
        shot[0].contains(PNG) && shot[0].contains(r#""sessionId":"S1""#),
        "{}",
        shot[0]
    );
    assert_eq!(session.url(), Ok(NEXT.to_string()));
    assert!(
        session.events().iter().any(|e| e.contains("Page.loadEventFired")),
        "{:?}",
        session.events()
    );
    assert_eq!(
        field.records("chrome"),
        [relay::eyes_argv(&dir, URL)]
    );
    drop(eyes);
    assert!(!dir.exists(), "drop の後の dir: {}", dir.display());
    drop(session);
    assert_eq!(server.join().expect("偽の Chrome"), ATTACHED);
}

#[test]
fn relay_eyes_waits_for_page() {
    let field = Field::new("waits", Ssh::Fail);
    let server = serve(&field, Page::Second);
    let (eyes, mut session) = Eyes::open(
        field.chrome.as_os_str(),
        &env::temp_dir(),
        URL,
        Duration::from_secs(5),
    )
    .expect("open");
    let dom = session.run(&Command::Dom).expect("Dom");
    assert_eq!(dom.len(), 1, "{dom:?}");
    assert!(dom[0].contains(&json::escape(DOM)), "{}", dom[0]);
    assert_eq!(session.url(), Ok(URL.to_string()));
    drop(session);
    drop(eyes);
    let got = server.join().expect("偽の Chrome");
    assert_eq!(
        methods(&got),
        [
            "Target.getTargets",
            "Target.getTargets",
            "Target.attachToTarget",
            "Runtime.evaluate",
            "Page.getNavigationHistory",
        ]
    );
}

#[test]
fn relay_falls_back_when_unreachable() {
    let field = Field::new("fall", Ssh::Fail);
    let server = serve(&field, Page::First);
    let b = term("term-b");
    let reached = relay::reach(
        field.ssh.as_os_str(),
        field.chrome.as_os_str(),
        &b,
        &env::temp_dir(),
        URL,
        Duration::from_secs(5),
    )
    .expect("reach");
    let Reach::Eyes {
        eyes,
        mut session,
        line,
    } = reached
    else {
        panic!("届かない端末で Eyes に落ちない");
    };
    for word in ["端末 term-b", "255", "席の目", "board の URL"] {
        assert!(line.contains(word), "{word}: {line}");
    }
    let ssh = field.records("ssh");
    assert_eq!(ssh.len(), 1, "{ssh:?}");
    assert_eq!(ssh[0].len(), 13, "{ssh:?}");
    let socket = ssh[0][10].split_once(':').map_or("", |(path, _)| path);
    let socket = Path::new(socket);
    assert_eq!(ssh[0], launch::tunnel_argv(&b, socket));
    let tunnel_dir = socket.parent().expect("socket の dir");
    assert!(!tunnel_dir.exists(), "{}", tunnel_dir.display());
    assert_eq!(
        field.records("chrome"),
        [relay::eyes_argv(eyes.dir(), URL)]
    );
    let dom = session.run(&Command::Dom).expect("Dom");
    assert!(dom.iter().any(|r| r.contains("eyes")), "{dom:?}");
    drop(session);
    drop(eyes);
    let got = server.join().expect("偽の Chrome");
    assert_eq!(got.len(), 3, "{got:?}");
}

#[test]
fn relay_reachable_stays_on_terminal() {
    let field = Field::new("stay", Ssh::Sleep);
    let a = term("term-a");
    let reached = relay::reach(
        field.ssh.as_os_str(),
        field.chrome.as_os_str(),
        &a,
        &env::temp_dir(),
        URL,
        Duration::from_secs(5),
    )
    .expect("reach");
    let Reach::Terminal(tunnel) = reached else {
        panic!("届く端末で Terminal でない");
    };
    assert_eq!(
        field.records("ssh"),
        [launch::tunnel_argv(&a, tunnel.socket())]
    );
    let dir = tunnel.socket().parent().expect("socket の dir").to_path_buf();
    drop(tunnel);
    assert!(!dir.exists(), "drop の後の dir: {}", dir.display());
    assert!(field.records("chrome").is_empty(), "偽の Chrome が撃たれた");
}

#[test]
fn relay_eyes_cleanup() {
    let field = Field::new("cleanup", Ssh::Fail);
    let server = serve(&field, Page::First);
    let (eyes, session) = Eyes::open(
        field.chrome.as_os_str(),
        &env::temp_dir(),
        URL,
        Duration::from_secs(5),
    )
    .expect("open");
    let dir = eyes.dir().to_path_buf();
    let chrome = field.pid("chrome.pid");
    let cat = field.pid("cat.pid");
    drop(eyes);
    assert!(!dir.exists(), "drop の後の dir: {}", dir.display());
    assert!(gone(&chrome), "偽の Chrome {chrome} が残る");
    assert!(gone(&cat), "偽の Chrome の子 {cat} が残る");
    drop(session);
    server.join().expect("偽の Chrome");
}

#[test]
fn relay_eyes_refusals() {
    let field = Field::new("refuse", Ssh::Fail);
    let five = Duration::from_secs(5);
    for bad in BAD_URLS {
        let err = refused(Eyes::open(
            field.chrome.as_os_str(),
            &env::temp_dir(),
            bad,
            five,
        ));
        assert!(err.contains("URL"), "{bad:?}: {err}");
    }
    assert!(field.records("chrome").is_empty(), "偽の Chrome が撃たれた");

    let start = Instant::now();
    let err = refused(Eyes::open(
        OsStr::new("/fake/none/chrome"),
        &env::temp_dir(),
        URL,
        five,
    ));
    assert!(start.elapsed() < five, "{:?}", start.elapsed());
    assert!(
        err.contains("/fake/none/chrome") && err.contains("席の目"),
        "{err}"
    );

    let field = Field::new("nopage", Ssh::Fail);
    let server = serve(&field, Page::Never);
    let start = Instant::now();
    let err = refused(Eyes::open(
        field.chrome.as_os_str(),
        &env::temp_dir(),
        URL,
        Duration::from_secs(1),
    ));
    assert!(start.elapsed() < five, "{:?}", start.elapsed());
    assert!(err.contains("頁の target"), "{err}");
    let got = server.join().expect("偽の Chrome");
    assert!(got.len() >= 2, "{got:?}");
    assert!(got.iter().all(|m| m.contains("Target.getTargets")), "{got:?}");
}

#[test]
fn relay_source_guards() {
    let print = ["print", "!"].concat();
    let println = ["println", "!"].concat();
    for file in ["pipe.rs", "relay.rs", "url.rs"] {
        let text = src(file);
        for word in [
            "bringToFront",
            "activateTarget",
            "createTarget",
            "setWindowBounds",
            "json/new",
            "json/activate",
            "Runtime.evaluate",
            "remote-debugging-port",
            "ts.net",
            "unsafe",
            &print,
            &println,
        ] {
            assert!(!text.contains(word), "{file} が {word} を含む");
        }
    }
    let cdp = src("cdp.rs");
    for word in ["enum Link", "Page.getNavigationHistory"] {
        assert_eq!(cdp.matches(word).count(), 1, "cdp.rs の {word}");
    }
    let module = src("mod.rs");
    for line in ["pub mod pipe;", "pub mod relay;", "pub mod url;"] {
        assert_eq!(module.matches(line).count(), 1, "{line}");
    }
}

#[test]
fn relay_own_names_clean() {
    let words: Vec<&str> = FILTER_WORDS.split_whitespace().collect();
    assert_eq!(words.len(), 125, "filter の語の数");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/strelay.rs");
    let text = fs::read_to_string(&path).expect("tests/strelay.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 12, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("relay_")
            .unwrap_or_else(|| panic!("{name} は relay_ で始まらない"));
        for word in &words {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
