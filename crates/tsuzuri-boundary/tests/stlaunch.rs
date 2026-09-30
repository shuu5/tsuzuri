//! 起動の引数と tunnel と窓を 1 回だけ起こすことの歯（行 i-2・接頭辞 launch_）。
//! 端末は fixture の tests/fixtures/stage/terminals.toml の行を lookup で引く（名も宛先も path も偽物）。
//! 偽の ssh は sh の script で argv を記録し、偽の Chrome は歯の中の thread で tunnel の socket に待ち受ける
//! （本物の ssh と Chrome を撃たず網に出ない）。

use std::env;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use tsuzuri_boundary::stage::json;
use tsuzuri_boundary::stage::launch::{self, PORT, WINDOW_SIZE};
use tsuzuri_boundary::stage::terminal::{self, Os, Terminal};
use tsuzuri_boundary::stage::tunnel::{self, SOCKET, Tunnel, Window};
use tsuzuri_boundary::stage::url::Board;

/// contracts の verify の filter の語のうち、ほかの語を部分の字として含まない最小の語（この行の接頭辞 launch_ は並べない）。
const FILTER_WORDS: &str = concat!(
    "aaround_ accept_ account_ acctcore_ acctdoc_ accthb_ accthome_ acctled_ acctlook_ acctpcore_ ",
    "acctproj_ acctsess_ acctwin_ acctwire_ afocus_ aord_ apop_ askcard_ athr_ batchpanel_ ",
    "board_min_ bport_ brand_ btuck_ cadopt_ cgdom_ cmark_ contract_form_ cround_ csled_ ",
    "denv_ ecache_ flight_ fmark_ frame_ fserve_ fstop_ gapspage_ gfresh_ ghb_ ",
    "glabel_ gnav_ graph_ gsum_ gtuck_ gview_ hbconf_ hbpost_ hbproc_ hbroute_ ",
    "hcard_ hcled_ hcnx_ hcproj_ hcsess_ hfig_ hook_ hruling_ hsblock_ hsderive_ ",
    "hspage_ hsym_ iclose_ ilink_ kcli_ klink_ lcard_ ledgerblock_ lhome_ lspark_ ",
    "mapview_ mkeys_ mlink_ mstore_ mtree_ nact_ nbatch_ ncard_ nextstep_ nodepage_ ",
    "nstall_ nsum_ nsumw_ ntime_ nxact_ parts_ pclosed_ pfold_ pipe_ plimit_ ",
    "pmore_ pquest_ project_ ptitle_ pwhole_ qblock_ qgate_ qkey_ question_ rhold_ ",
    "runsdoc_ saxis_ sclosed_ seatblock_ seatcard_ server_ sesplit_ shb_ skeleton_ smore_ ",
    "stage_ stats_ steady_ sxaxis_ ticker_ tipx_ topbar_ tz_ urpanel_ uword_ ",
    "wstrip_",
);

/// 節の URL（shell の特別な字 ? と & を持つ）。
const URL: &str = "http://127.0.0.1:4173/?a=1&b=2";

/// 遠くの shell の字の末（前の空白を含む）。
const TAIL: &str = " </dev/null >/dev/null 2>&1";

/// 節の printf の書式。
const PRINTF: &str = r"printf '%s\0' ";

/// 節の Chrome の argv（term-a）。
const CHROME_A: [&str; 7] = [
    "/fake/bin/chrome",
    "--app=http://127.0.0.1:4173/?a=1&b=2",
    "--user-data-dir=/fake/profile-a",
    "--remote-debugging-port=9224",
    "--no-first-run",
    "--no-default-browser-check",
    "--window-size=900,760",
];

/// 節の起こす argv の遠くの shell の字（term-a）。
const REMOTE_A: &str = r#"'setsid' '-f' 'env' 'DISPLAY=:0' 'GTK_IM_MODULE=fcitx' 'XMODIFIERS=@im=fcitx' '/fake/bin/chrome' '--app=http://127.0.0.1:4173/?a=1&b=2' '--user-data-dir=/fake/profile-a' '--remote-debugging-port=9224' '--no-first-run' '--no-default-browser-check' '--window-size=900,760' </dev/null >/dev/null 2>&1"#;

/// 節の起こす argv の遠くの shell の字（term-b）。
const REMOTE_B: &str = r#"'setsid' '-f' 'env' 'WAYLAND_DISPLAY=wayland-1' 'XDG_RUNTIME_DIR=/fake/run-b' '/fake/bin/chrome' '--app=http://127.0.0.1:4173/?a=1&b=2' '--user-data-dir=/fake/profile-b' '--remote-debugging-port=9224' '--no-first-run' '--no-default-browser-check' '--window-size=900,760' </dev/null >/dev/null 2>&1"#;

/// 節の tunnel の argv（path /s/cdp.sock と term-a）。
const TUNNEL_A: [&str; 13] = [
    "-N",
    "-o",
    "ExitOnForwardFailure=yes",
    "-o",
    "BatchMode=yes",
    "-o",
    "ConnectTimeout=10",
    "-o",
    "ControlPath=none",
    "-L",
    "/s/cdp.sock:127.0.0.1:9224",
    "--",
    "me@term-a",
];

/// 節の HOME の語。
const HOME_WORD: &str = r#"'--user-data-dir='"$HOME"'/.cache/tsuzuri-stage'"#;

/// 節の quote の表。
const QUOTES: [(&str, &str); 5] = [
    ("abc", "'abc'"),
    ("", "''"),
    ("a b", "'a b'"),
    ("it's", r"'it'\''s'"),
    ("$HOME;`id`&|<>*", "'$HOME;`id`&|<>*'"),
];

/// 節の 5 つの形の外の URL。
const BAD_URLS: [&str; 5] = [
    "javascript:alert(1)",
    "file:///fake/x.html",
    "http://a b/",
    "http://a/\n",
    "",
];

/// 偽の Chrome の /json/version の本文。
const VERSION: &str = r#"{"Browser":"Chrome/146.0.0.0","Protocol-Version":"1.3","webSocketDebuggerUrl":"ws://localhost/devtools/browser/B1"}"#;

/// 偽の Chrome の /json/list の本文（service_worker の後に頁）。
const LIST: &str = r#"[ {
   "description": "",
   "id": "S1",
   "type": "service_worker",
   "url": "chrome-extension://x/bg.js",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/S1"
}, {
   "description": "",
   "id": "P1",
   "title": "tsuzuri",
   "type": "page",
   "url": "http://127.0.0.1:4173/?a=1&b=2",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/P1"
} ]
"#;

/// 偽の Chrome の頁の無い /json/list の本文（LIST の service_worker の要素だけ）。
const NO_PAGE: &str = r#"[ {
   "description": "",
   "id": "S1",
   "type": "service_worker",
   "url": "chrome-extension://x/bg.js",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/S1"
} ]
"#;

/// 偽の Chrome の頁の一覧（頁は 4 つ。P1 は account board の頁・P2 はほかの port の頁・P3 は開発中の app の頁・P4 は board の頁）。
const LIST_MANY: &str = r#"[ {
   "id": "S1",
   "type": "service_worker",
   "url": "http://127.0.0.1:4173/",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/S1"
}, {
   "id": "P1",
   "type": "page",
   "url": "http://127.0.0.1:4173/?board=account#x",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/P1"
}, {
   "id": "P2",
   "type": "page",
   "url": "http://127.0.0.1:4802/",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/P2"
}, {
   "id": "P3",
   "type": "page",
   "url": "http://127.0.0.1:8080/app",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/P3"
}, {
   "id": "P4",
   "type": "page",
   "url": "http://localhost:4173/board?x=1#board=account",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/P4"
} ]
"#;

/// 偽の Chrome の頁の一覧（ほかの board の頁と account board の頁だけ）。
const LIST_OTHER: &str = r#"[ {
   "id": "P1",
   "type": "page",
   "url": "http://127.0.0.1:4173/?board=account",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/P1"
}, {
   "id": "P2",
   "type": "page",
   "url": "http://127.0.0.1:4802/",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/P2"
} ]
"#;

/// 窓を足した後の偽の Chrome の頁の一覧（LIST_OTHER の後ろに board の頁）。
const LIST_ADDED: &str = r#"[ {
   "id": "P1",
   "type": "page",
   "url": "http://127.0.0.1:4173/?board=account",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/P1"
}, {
   "id": "P2",
   "type": "page",
   "url": "http://127.0.0.1:4802/",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/P2"
}, {
   "id": "P5",
   "type": "page",
   "url": "http://127.0.0.1:4173/?a=1&b=2",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/P5"
} ]
"#;

/// 節の board（URL は URL・host は 127.0.0.1 と localhost・port は 4173）。
fn board() -> Board {
    Board {
        url: URL.to_string(),
        hosts: strings(&["127.0.0.1", "localhost"]),
        port: 4173,
    }
}

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

fn pairs(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// 節の起こす argv（宛先と遠くの shell の字）。
fn raise_argv(ssh: &str, remote: &str) -> Vec<String> {
    let mut argv = strings(&TUNNEL_A[3..9]);
    argv.push("--".to_string());
    argv.push(ssh.to_string());
    argv.push(remote.to_string());
    argv
}

/// 節の printf の書式に続けて words を sh の -c で撃ち、標準出力を NUL で分けた字の列（末の NUL の後の空の字は数えない）。
fn printed(words: &str, home: Option<&str>) -> Vec<String> {
    let mut sh = Command::new("sh");
    sh.arg("-c").arg(format!("{PRINTF}{words}"));
    if let Some(home) = home {
        sh.env("HOME", home);
    }
    let out = sh.output().expect("sh を撃つ");
    assert!(out.status.success(), "{words}: {out:?}");
    let mut fields: Vec<String> = out
        .stdout
        .split(|b| *b == 0)
        .map(|f| String::from_utf8(f.to_vec()).expect("UTF-8 の字"))
        .collect();
    assert_eq!(fields.pop().as_deref(), Some(""), "末の NUL: {words}");
    fields
}

/// 偽の ssh の tunnel の回の振る舞い。
#[derive(Clone, Copy)]
enum Reach {
    Sleep,
    Fail,
}

/// 偽の場（記録の置き場の dir・偽の ssh・印の file・窓を足した印の file）。
struct Field {
    records: PathBuf,
    ssh: PathBuf,
    mark: PathBuf,
    added: PathBuf,
}

impl Field {
    /// 窓を起こす回の偽の ssh は raise が真なら印の file を置き、いつも窓を足した印の file を置く。
    /// marked が真なら印の file を始めから置く。
    fn new(name: &str, reach: Reach, raise: bool, marked: bool) -> Field {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("stlaunch")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let records = root.join("records");
        fs::create_dir_all(&records).expect("記録の置き場");
        let mark = root.join("mark");
        let added = root.join("added");
        if marked {
            fs::write(&mark, "").expect("印の file");
        }
        let tunnel = match reach {
            Reach::Sleep => "exec sleep 30",
            Reach::Fail => "exit 255",
        };
        let raise = if raise {
            format!(": > '{}'", mark.display())
        } else {
            String::new()
        };
        let script = format!(
            "#!/bin/sh\nrec='{}'\nn=$(( $(cat \"$rec/count\" 2>/dev/null || echo 0) + 1 ))\nfor a in \"$@\"; do printf '%s\\n' \"$a\"; done > \"$rec/$n.args\"\nprintf '%s' \"$n\" > \"$rec/count\"\nif [ \"$1\" = -N ]; then\n  {tunnel}\nfi\n{raise}\n: > '{}'\nexit 0\n",
            records.display(),
            added.display()
        );
        let ssh = root.join("ssh");
        fs::write(&ssh, script).expect("偽の ssh");
        fs::set_permissions(&ssh, fs::Permissions::from_mode(0o755)).expect("偽の ssh の権限");
        Field {
            records,
            ssh,
            mark,
            added,
        }
    }

    /// 偽の ssh の argv の記録（count が want になるまで 5 秒まで待つ）。
    fn records(&self, want: usize) -> Vec<Vec<String>> {
        let start = Instant::now();
        loop {
            let count: usize = fs::read_to_string(self.records.join("count"))
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);
            if count >= want || start.elapsed() > Duration::from_secs(5) {
                return (1..=count)
                    .map(|n| {
                        let path = self.records.join(format!("{n}.args"));
                        fs::read_to_string(&path)
                            .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
                            .lines()
                            .map(str::to_string)
                            .collect()
                    })
                    .collect();
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
}

/// 偽の Chrome（受けた要求の行を記す thread）。
struct Chrome {
    stop: Arc<AtomicBool>,
    thread: JoinHandle<Vec<String>>,
}

/// 偽の Chrome の頁の一覧（窓を足した印の file が在れば after・無ければ before）。
#[derive(Clone, Copy)]
struct Lists {
    before: &'static str,
    after: &'static str,
}

impl Chrome {
    fn start(socket: &Path, mark: &Path, added: &Path, lists: Lists) -> Chrome {
        let listener =
            UnixListener::bind(socket).unwrap_or_else(|e| panic!("{}: {e}", socket.display()));
        listener.set_nonblocking(true).expect("待ち受けを止めない形に");
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let mark = mark.to_path_buf();
        let added = added.to_path_buf();
        let thread = thread::spawn(move || {
            let mut got = Vec::new();
            while !flag.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let list = if added.exists() { lists.after } else { lists.before };
                        serve(stream, &mark, list, &mut got);
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => panic!("接続を受ける: {e}"),
                }
            }
            got
        });
        Chrome { stop, thread }
    }

    /// thread を止めて受けた要求の行を返す。
    fn stop(self) -> Vec<String> {
        self.stop.store(true, Ordering::SeqCst);
        self.thread.join().expect("偽の Chrome")
    }
}

/// 1 つの接続（印の file が無ければ何も書かずに閉じる・在れば 200 を 2 回に分けて書き、相手が閉じるまで待つ）。
fn serve(mut stream: UnixStream, mark: &Path, list: &str, got: &mut Vec<String>) {
    stream.set_nonblocking(false).expect("読みを待つ形に");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("読みの上限");
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        match stream.read(&mut byte) {
            Ok(1) => head.push(byte[0]),
            _ => {
                got.push("頭が途中で切れた".to_string());
                return;
            }
        }
    }
    let head = String::from_utf8(head).expect("頭の字");
    let first = head.split("\r\n").next().unwrap_or("");
    let line = first.strip_suffix("HTTP/1.1").unwrap_or(first).trim_end();
    let host = head
        .split("\r\n")
        .skip(1)
        .filter_map(|l| l.split_once(':'))
        .any(|(n, v)| n.trim().eq_ignore_ascii_case("host") && v.trim() == "localhost");
    got.push(if host {
        line.to_string()
    } else {
        format!("{line}（Host の欄が localhost でない）")
    });
    if !mark.exists() {
        return;
    }
    let (head, body) = match line {
        "GET /json/version" => (
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=UTF-8\r\nContent-Length:{}\r\n\r\n",
                VERSION.len()
            ),
            VERSION,
        ),
        "GET /json/list" => (
            format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json; charset=UTF-8\r\ncontent-length: {}\r\n\r\n",
                list.len()
            ),
            list,
        ),
        _ => return,
    };
    if stream.write_all(head.as_bytes()).is_err() {
        return;
    }
    thread::sleep(Duration::from_millis(20));
    if stream.write_all(body.as_bytes()).is_err() {
        return;
    }
    let mut rest = [0u8; 64];
    while matches!(stream.read(&mut rest), Ok(n) if n > 0) {}
}

/// 偽の場の 1 つの撃ち（tunnel の dir・偽の Chrome・開いた Tunnel）。
struct Scene {
    terminal: Terminal,
    dir: PathBuf,
    chrome: Chrome,
    tunnel: Tunnel,
}

impl Scene {
    fn open(field: &Field, name: &str, list: &'static str, timeout: Duration) -> Scene {
        let lists = Lists {
            before: list,
            after: list,
        };
        Scene::open_lists(field, name, lists, timeout)
    }

    fn open_lists(field: &Field, name: &str, lists: Lists, timeout: Duration) -> Scene {
        let terminal = term(name);
        let dir = tunnel::socket_dir(&env::temp_dir()).expect("socket_dir");
        let chrome = Chrome::start(&dir.join(SOCKET), &field.mark, &field.added, lists);
        let tunnel = Tunnel::open(field.ssh.as_os_str(), &terminal, &dir, timeout).expect("open");
        assert_eq!(tunnel.socket(), dir.join(SOCKET));
        let first = field.records(1);
        assert_eq!(
            first,
            [launch::tunnel_argv(&terminal, tunnel.socket())],
            "tunnel の回の記録"
        );
        Scene {
            terminal,
            dir,
            chrome,
            tunnel,
        }
    }

    /// Tunnel を drop して dir が無いことを見てから、偽の Chrome を止めて受けた要求の行を返す。
    fn finish(self) -> Vec<String> {
        let Scene {
            dir,
            chrome,
            tunnel,
            ..
        } = self;
        drop(tunnel);
        assert!(!dir.exists(), "drop の後の dir: {}", dir.display());
        chrome.stop()
    }
}

/// env の KEY と VALUE の組。
type Pair = (String, String);

fn debug<T: Debug>() {}

#[test]
fn launch_shape_and_consts() {
    let port: u16 = PORT;
    assert_eq!(port, 9224);
    let size: &str = WINDOW_SIZE;
    assert_eq!(size, "900,760");
    let socket_name: &str = SOCKET;
    assert_eq!(socket_name, "cdp.sock");
    let quote: fn(&str) -> String = launch::quote;
    let launch_env: fn(&Terminal) -> Result<Vec<Pair>, String> = launch::launch_env;
    let chrome_argv: fn(&Terminal, &str) -> Vec<String> = launch::chrome_argv;
    let launch_argv: fn(&Terminal, &str) -> Result<Vec<String>, String> = launch::launch_argv;
    let tunnel_argv: fn(&Terminal, &Path) -> Vec<String> = launch::tunnel_argv;
    let page_resource: fn(&str) -> Option<String> = launch::page_resource;
    let items: for<'a> fn(&'a str) -> Option<Vec<&'a str>> = json::items;
    let socket_dir: fn(&Path) -> Result<PathBuf, String> = tunnel::socket_dir;
    let open: fn(&OsStr, &Terminal, &Path, Duration) -> Result<Tunnel, String> = Tunnel::open;
    let socket: for<'a> fn(&'a Tunnel) -> &'a Path = Tunnel::socket;
    let window: fn(&Tunnel, &Board, Option<&str>, bool) -> Result<Window, String> = Tunnel::window;
    let board_page: fn(&str, &Board) -> Option<String> = launch::board_page;
    let pick: fn(&str, Option<&str>, &Board) -> Option<String> = launch::pick;
    let _ = (quote, launch_env, chrome_argv, launch_argv, tunnel_argv, page_resource);
    let _ = (items, socket_dir, open, socket, window, board_page, pick);
    debug::<Tunnel>();
    let windows = [
        Window::Page {
            resource: "/devtools/page/P1".to_string(),
            launched: true,
        },
        Window::Absent("端末 term-a".to_string()),
    ];
    for window in &windows {
        match window {
            Window::Page { resource, launched } => {
                assert_eq!(resource, "/devtools/page/P1");
                assert!(*launched);
            }
            Window::Absent(line) => assert_eq!(line, "端末 term-a"),
        }
    }
    assert_ne!(windows[0], windows[1].clone());
}

#[test]
fn launch_quote_round_trip() {
    for (word, want) in QUOTES {
        assert_eq!(launch::quote(word), want, "{word}");
    }
    let words: Vec<&str> = QUOTES
        .iter()
        .map(|(w, _)| *w)
        .chain(["日本", "''", "a\"b\\c", URL])
        .collect();
    assert_eq!(words.len(), 9);
    let line: Vec<String> = words.iter().map(|w| launch::quote(w)).collect();
    assert_eq!(printed(&line.join(" "), None), words);
}

#[test]
fn launch_env_pairs_in_order() {
    let a = term("term-a");
    let b = term("term-b");
    assert_eq!(
        launch::launch_env(&a),
        Ok(pairs(&[
            ("DISPLAY", ":0"),
            ("GTK_IM_MODULE", "fcitx"),
            ("XMODIFIERS", "@im=fcitx"),
        ]))
    );
    assert_eq!(
        launch::launch_env(&b),
        Ok(pairs(&[
            ("WAYLAND_DISPLAY", "wayland-1"),
            ("XDG_RUNTIME_DIR", "/fake/run-b"),
        ]))
    );
    let mut ime = a.ime_env.clone();
    ime.push(("DISPLAY".to_string(), ":1".to_string()));
    let doubled_a = Terminal {
        ime_env: ime,
        ..a.clone()
    };
    let doubled_b = Terminal {
        ime_env: pairs(&[("XDG_RUNTIME_DIR", "/x")]),
        ..b.clone()
    };
    for (terminal, key) in [(&doubled_a, "DISPLAY"), (&doubled_b, "XDG_RUNTIME_DIR")] {
        let err = launch::launch_env(terminal).expect_err("重なった KEY は Err");
        assert!(err.contains(&terminal.name) && err.contains(key), "{err}");
    }
}

#[test]
fn launch_argv_snapshot() {
    let a = term("term-a");
    let b = term("term-b");
    assert_eq!(launch::chrome_argv(&a, URL), CHROME_A);
    assert_eq!(
        launch::launch_argv(&a, URL),
        Ok(raise_argv("me@term-a", REMOTE_A))
    );
    assert_eq!(
        launch::launch_argv(&b, URL),
        Ok(raise_argv("me@term-b", REMOTE_B))
    );
    for argv in [raise_argv("me@term-a", REMOTE_A), raise_argv("me@term-b", REMOTE_B)] {
        assert_eq!(argv.len(), 9);
    }
    let socket = Path::new("/s/cdp.sock");
    assert_eq!(launch::tunnel_argv(&a, socket), TUNNEL_A);
    let mut tunnel_b = strings(&TUNNEL_A);
    tunnel_b[12] = "me@term-b".to_string();
    assert_eq!(launch::tunnel_argv(&b, socket), tunnel_b);
}

#[test]
fn launch_remote_line_round_trip() {
    let a = term("term-a");
    let b = term("term-b");
    let home = Terminal {
        profile_dir: "~/.cache/tsuzuri-stage".to_string(),
        ..a.clone()
    };
    let cases = [
        (&a, "--user-data-dir=/fake/profile-a"),
        (&b, "--user-data-dir=/fake/profile-b"),
        (&home, "--user-data-dir=/fake/home/.cache/tsuzuri-stage"),
    ];
    for (terminal, profile) in cases {
        let argv = launch::launch_argv(terminal, URL).expect("launch_argv");
        assert_eq!(argv.len(), 9, "{argv:?}");
        let last = &argv[8];
        if terminal.profile_dir.starts_with("~/") {
            assert!(last.contains(HOME_WORD), "{last}");
        }
        let words = last
            .strip_suffix(TAIL)
            .unwrap_or_else(|| panic!("末の字: {last}"));
        let mut want = strings(&["setsid", "-f", "env"]);
        want.extend(
            launch::launch_env(terminal)
                .expect("launch_env")
                .iter()
                .map(|(k, v)| format!("{k}={v}")),
        );
        let mut chrome = launch::chrome_argv(terminal, URL);
        chrome[2] = profile.to_string();
        want.extend(chrome);
        assert_eq!(printed(words, Some("/fake/home")), want, "{}", terminal.name);
    }
}

#[test]
fn launch_refusals_named() {
    let a = term("term-a");
    let b = term("term-b");
    let socket = Path::new("/s/cdp.sock");
    for os in [Os::Macos, Os::Windows] {
        let other = Terminal { os, ..a.clone() };
        let err = launch::launch_argv(&other, URL).expect_err("linux でない端末は Err");
        assert!(
            err.contains("term-a") && err.contains("[os]") && err.contains(os.word()),
            "{err}"
        );
        assert_eq!(launch::tunnel_argv(&other, socket), TUNNEL_A);
    }
    let blind = Terminal {
        display: None,
        ..a.clone()
    };
    assert!(blind.display_env.is_empty());
    let err = launch::launch_argv(&blind, URL).expect_err("画面の欄の無い行は Err");
    assert!(
        err.contains("term-a") && err.contains("[display]") && err.contains("[display-env]"),
        "{err}"
    );
    for profile in ["profile-a", "~other/x", "~"] {
        let odd = Terminal {
            profile_dir: profile.to_string(),
            ..a.clone()
        };
        let err = launch::launch_argv(&odd, URL).expect_err("形の外の profile-dir は Err");
        assert!(
            err.contains("term-a") && err.contains("[profile-dir]"),
            "{profile}: {err}"
        );
    }
    for terminal in [&a, &b] {
        for url in BAD_URLS {
            let err = launch::launch_argv(terminal, url).expect_err("形の外の URL は Err");
            assert!(err.contains("URL"), "{url:?}: {err}");
        }
    }
}

#[test]
fn launch_socket_dir_private() {
    let dir = tunnel::socket_dir(&env::temp_dir()).expect("socket_dir");
    let name = dir.file_name().and_then(OsStr::to_str).unwrap_or("");
    assert!(name.starts_with("tzst-"), "{}", dir.display());
    let mode = fs::metadata(&dir).expect("dir の権限").permissions().mode();
    assert_eq!(mode & 0o777, 0o700, "{mode:o}");
    let socket = dir.join(SOCKET);
    assert!(socket.as_os_str().len() <= 107, "{}", socket.display());
    fs::remove_dir(&dir).expect("作った dir を消す");

    let colon = env::temp_dir().join(format!("tz:{}", process::id()));
    let long = env::temp_dir().join("a".repeat(120));
    for made in [&colon, &long] {
        fs::create_dir_all(made).expect("歯が先に作る dir");
    }
    let absent = env::temp_dir().join(format!("tzst-none-{}", process::id())).join("x");
    for base in [&colon, &long, &absent] {
        assert!(tunnel::socket_dir(base).is_err(), "{}", base.display());
    }
    for made in [&colon, &long] {
        fs::remove_dir_all(made).expect("歯が作った dir を消す");
    }
}

#[test]
fn launch_page_from_list() {
    let lists: [(&str, Option<&str>); 8] = [
        (LIST, Some("/devtools/page/P1")),
        (
            r#"[{"type":"page","webSocketDebuggerUrl":"ws://localhost/devtools/page/P1"},{"type":"page","webSocketDebuggerUrl":"ws://localhost/devtools/page/P2"}]"#,
            Some("/devtools/page/P1"),
        ),
        (
            r#"[{"type":"page","webSocketDebuggerUrl":"ws://127.0.0.1:9224/devtools/page/P9"}]"#,
            Some("/devtools/page/P9"),
        ),
        (NO_PAGE, None),
        ("[]", None),
        ("{}", None),
        (
            r#"[{"type":"page","webSocketDebuggerUrl":"ws://localhost/devtools/browser/B1"}]"#,
            None,
        ),
        (r#"[{"type":"page","url":"http://127.0.0.1:4173/"}]"#, None),
    ];
    for (list, want) in lists {
        assert_eq!(launch::page_resource(list).as_deref(), want, "{list}");
    }
    let own = board();
    let boards: [(&str, Option<&str>); 8] = [
        (LIST, Some("/devtools/page/P1")),
        (LIST_MANY, Some("/devtools/page/P4")),
        (LIST_OTHER, None),
        (NO_PAGE, None),
        ("[]", None),
        ("{}", None),
        (
            r#"[{"type":"page","url":"http://127.0.0.1:4173/","webSocketDebuggerUrl":"ws://localhost/devtools/browser/B1"}]"#,
            None,
        ),
        (
            r#"[{"type":"page","url":"http://127.0.0.1:4173/?a=1&board=account"},{"type":"page","url":"http://localhost:4173/x","webSocketDebuggerUrl":"ws://localhost/devtools/page/P7"}]"#,
            Some("/devtools/page/P7"),
        ),
    ];
    for (list, want) in boards {
        assert_eq!(launch::board_page(list, &own).as_deref(), want, "{list}");
    }
    let arrays: [(&str, Option<Vec<&str>>); 9] = [
        (
            r#" [ 1 , {"a":[2,3]} , "x" , [4] , true ] "#,
            Some(vec!["1", r#"{"a":[2,3]}"#, r#""x""#, "[4]", "true"]),
        ),
        (" [ \n ] ", Some(Vec::new())),
        ("{}", None),
        ("[1,]", None),
        ("[,1]", None),
        ("[1,,2]", None),
        ("[1] x", None),
        ("[1", None),
        ("", None),
    ];
    for (array, want) in arrays {
        assert_eq!(json::items(array), want, "{array:?}");
    }
}

#[test]
fn launch_running_chrome_reused() {
    let field = Field::new("running", Reach::Sleep, true, true);
    let scene = Scene::open(&field, "term-a", LIST, Duration::from_secs(5));
    assert_eq!(
        scene.tunnel.window(&board(), None, true),
        Ok(Window::Page {
            resource: "/devtools/page/P1".to_string(),
            launched: false,
        })
    );
    let want = [launch::tunnel_argv(&scene.terminal, scene.tunnel.socket())];
    assert_eq!(field.records(1), want);
    assert_eq!(scene.finish(), ["GET /json/version", "GET /json/list"]);
}

#[test]
fn launch_memo_page_reused() {
    let field = Field::new("memo", Reach::Sleep, true, true);
    let scene = Scene::open(&field, "term-a", LIST_MANY, Duration::from_secs(5));
    let own = board();
    let page = |resource: &str| {
        Ok(Window::Page {
            resource: resource.to_string(),
            launched: false,
        })
    };
    assert_eq!(launch::pick(LIST_MANY, None, &own).as_deref(), Some("/devtools/page/P4"));
    assert_eq!(
        launch::pick(LIST_MANY, Some("/devtools/page/P3"), &own).as_deref(),
        Some("/devtools/page/P3")
    );
    assert_eq!(
        launch::pick(LIST_MANY, Some("/devtools/page/S1"), &own).as_deref(),
        Some("/devtools/page/P4")
    );
    assert_eq!(
        launch::pick(LIST_MANY, Some("/devtools/page/P9"), &own).as_deref(),
        Some("/devtools/page/P4")
    );
    assert_eq!(launch::pick(LIST_OTHER, Some("/devtools/page/P9"), &own), None);
    // 覚えた頁が開発中の app の頁でも、may_open が偽でも、その頁を起こさずに使う。
    for may_open in [false, true] {
        assert_eq!(
            scene.tunnel.window(&own, Some("/devtools/page/P3"), may_open),
            page("/devtools/page/P3"),
            "{may_open}"
        );
    }
    assert_eq!(scene.tunnel.window(&own, None, false), page("/devtools/page/P4"));
    // 頁でない項の path を覚えていても選ばない。
    assert_eq!(
        scene.tunnel.window(&own, Some("/devtools/page/S1"), false),
        page("/devtools/page/P4")
    );
    let want = [launch::tunnel_argv(&scene.terminal, scene.tunnel.socket())];
    assert_eq!(field.records(1), want);
    assert_eq!(scene.finish().len(), 8);

    // 覚えた path が一覧に無く board の頁も無ければ、board の URL を含む Absent。
    let field = Field::new("memo-none", Reach::Sleep, true, true);
    let scene = Scene::open(&field, "term-a", LIST_OTHER, Duration::from_secs(5));
    match scene.tunnel.window(&own, Some("/devtools/page/P9"), false) {
        Ok(Window::Absent(line)) => assert!(line.contains(URL), "{line}"),
        other => panic!("Absent でない: {other:?}"),
    }
    scene.finish();
}

#[test]
fn launch_closed_stays_closed() {
    let field = Field::new("closed", Reach::Sleep, true, false);
    let scene = Scene::open(&field, "term-a", LIST, Duration::from_secs(5));
    match scene.tunnel.window(&board(), None, false) {
        Ok(Window::Absent(line)) => {
            assert!(line.contains("term-a") && line.contains("tz stage open"), "{line}");
        }
        other => panic!("Absent でない: {other:?}"),
    }
    let want = [launch::tunnel_argv(&scene.terminal, scene.tunnel.socket())];
    assert_eq!(field.records(1), want);
    assert_eq!(scene.finish(), ["GET /json/version"]);
}

#[test]
fn launch_raise_once_when_asked() {
    for (name, ssh, remote) in [
        ("term-a", "me@term-a", REMOTE_A),
        ("term-b", "me@term-b", REMOTE_B),
    ] {
        let field = Field::new(&format!("raise-{name}"), Reach::Sleep, true, false);
        let scene = Scene::open(&field, name, LIST, Duration::from_secs(5));
        assert_eq!(
            scene.tunnel.window(&board(), None, true),
            Ok(Window::Page {
                resource: "/devtools/page/P1".to_string(),
                launched: true,
            }),
            "{name}"
        );
        let want = vec![
            launch::tunnel_argv(&scene.terminal, scene.tunnel.socket()),
            launch::launch_argv(&scene.terminal, URL).expect("launch_argv"),
        ];
        assert_eq!(want[1], raise_argv(ssh, remote), "{name}");
        assert_eq!(field.records(2), want, "{name}");
        let got = scene.finish();
        assert!(got.len() >= 3, "{name}: {got:?}");
        assert_eq!(got[..2], ["GET /json/version", "GET /json/version"], "{name}");
        assert!(got[2..].iter().all(|l| l == "GET /json/list"), "{name}: {got:?}");
    }
}

#[test]
fn launch_no_second_raise() {
    let field = Field::new("once", Reach::Sleep, false, false);
    let scene = Scene::open(&field, "term-a", LIST, Duration::from_secs(2));
    let start = Instant::now();
    let err = scene
        .tunnel
        .window(&board(), None, true)
        .expect_err("起こした Chrome が答えなければ Err");
    assert!(start.elapsed() < Duration::from_secs(10), "{:?}", start.elapsed());
    assert!(err.contains("term-a"), "{err}");
    let want = vec![
        launch::tunnel_argv(&scene.terminal, scene.tunnel.socket()),
        raise_argv("me@term-a", REMOTE_A),
    ];
    assert_eq!(field.records(2), want);
    let got = scene.finish();
    assert!(got.len() >= 4, "{got:?}");
    assert_eq!(got[..2], ["GET /json/version", "GET /json/version"]);
    assert!(got[2..].iter().all(|l| l == "GET /json/list"), "{got:?}");
}

#[test]
fn launch_no_page_no_new_window() {
    for (name, list) in [("nopage", NO_PAGE), ("otherpage", LIST_OTHER)] {
        let field = Field::new(name, Reach::Sleep, true, true);
        let scene = Scene::open(&field, "term-a", list, Duration::from_secs(5));
        for hint in [None, Some("/devtools/page/S1"), Some("/devtools/page/P9")] {
            match scene.tunnel.window(&board(), hint, false) {
                Ok(Window::Absent(line)) => {
                    assert!(line.contains("term-a") && line.contains(URL), "{name}: {line}");
                    assert!(line.contains("ほかの board の窓は使わない"), "{name}: {line}");
                }
                other => panic!("{name}: Absent でない: {other:?}"),
            }
        }
        let want = [launch::tunnel_argv(&scene.terminal, scene.tunnel.socket())];
        assert_eq!(field.records(1), want, "{name}");
        assert!(!field.added.exists(), "{name}: 起動の引数が撃たれた");
        let got = scene.finish();
        assert_eq!(got.len(), 6, "{name}: {got:?}");
        for pair in got.chunks(2) {
            assert_eq!(pair, ["GET /json/version", "GET /json/list"], "{name}");
        }
    }
}

#[test]
fn launch_other_board_adds_window() {
    let field = Field::new("adds", Reach::Sleep, false, true);
    let lists = Lists {
        before: LIST_OTHER,
        after: LIST_ADDED,
    };
    let scene = Scene::open_lists(&field, "term-a", lists, Duration::from_secs(5));
    assert_eq!(
        scene.tunnel.window(&board(), Some("/devtools/page/P9"), true),
        Ok(Window::Page {
            resource: "/devtools/page/P5".to_string(),
            launched: true,
        })
    );
    let want = vec![
        launch::tunnel_argv(&scene.terminal, scene.tunnel.socket()),
        launch::launch_argv(&scene.terminal, URL).expect("launch_argv"),
    ];
    assert_eq!(field.records(2), want);
    let got = scene.finish();
    assert!(got.len() >= 3, "{got:?}");
    assert_eq!(got[..2], ["GET /json/version", "GET /json/list"]);
    assert!(got[2..].iter().all(|l| l == "GET /json/list"), "{got:?}");

    // 覚えた頁が在れば、頼まれても窓を足さない。
    let field = Field::new("adds-memo", Reach::Sleep, false, true);
    let lists = Lists {
        before: LIST_ADDED,
        after: LIST_ADDED,
    };
    let scene = Scene::open_lists(&field, "term-a", lists, Duration::from_secs(5));
    assert_eq!(
        scene.tunnel.window(&board(), Some("/devtools/page/P2"), true),
        Ok(Window::Page {
            resource: "/devtools/page/P2".to_string(),
            launched: false,
        })
    );
    assert_eq!(field.records(1).len(), 1);
    scene.finish();
}

#[test]
fn launch_unreachable_named() {
    let b = term("term-b");
    let cases = [
        ("unreachable-fail", Reach::Fail, 5, 5, Some("255")),
        ("unreachable-sleep", Reach::Sleep, 1, 10, None),
    ];
    for (name, reach, timeout, limit, word) in cases {
        let field = Field::new(name, reach, true, false);
        let dir = tunnel::socket_dir(&env::temp_dir()).expect("socket_dir");
        let start = Instant::now();
        let err = Tunnel::open(
            field.ssh.as_os_str(),
            &b,
            &dir,
            Duration::from_secs(timeout),
        )
        .expect_err("届かない端末は Err");
        assert!(
            start.elapsed() < Duration::from_secs(limit),
            "{name}: {:?}",
            start.elapsed()
        );
        assert!(err.contains("term-b"), "{name}: {err}");
        if let Some(word) = word {
            assert!(err.contains(word), "{name}: {err}");
        }
        assert!(!dir.exists(), "{name}: {}", dir.display());
    }
}

#[test]
fn launch_source_guards() {
    let print = ["print", "!"].concat();
    let println = ["println", "!"].concat();
    for file in ["launch.rs", "tunnel.rs"] {
        let text = src(file);
        for word in [
            "bringToFront",
            "activateTarget",
            "createTarget",
            "setWindowBounds",
            "json/new",
            "json/activate",
            "unsafe",
            &print,
            &println,
            "tsuzuri-stage",
        ] {
            assert!(!text.contains(word), "{file} が {word} を含む");
        }
    }
    let text = src("launch.rs");
    assert_eq!(text.matches("std::").count(), 1, "launch.rs の std::");
    let uses: Vec<&str> = text.lines().filter(|l| l.starts_with("use std")).collect();
    assert_eq!(uses, ["use std::path::Path;"]);
    let module = src("mod.rs");
    for line in ["pub mod launch;", "pub mod tunnel;"] {
        assert_eq!(module.matches(line).count(), 1, "{line}");
    }
}

#[test]
fn launch_own_names_clean() {
    let words: Vec<&str> = FILTER_WORDS.split_whitespace().collect();
    assert_eq!(words.len(), 121, "filter の語の数");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/stlaunch.rs");
    let text = fs::read_to_string(&path).expect("tests/stlaunch.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 18, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("launch_")
            .unwrap_or_else(|| panic!("{name} は launch_ で始まらない"));
        for word in &words {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
