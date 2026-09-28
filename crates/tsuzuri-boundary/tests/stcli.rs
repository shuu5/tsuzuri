//! tz stage の口の歯（行 i-5・接頭辞 stcli_）。
//! 端末は fixture の tests/fixtures/stage/terminals.toml を偽の state dir の host.toml に写して引く（名も宛先も path も偽物）。
//! 偽の ssh・器・git・tailnet の道具・席の目の Chrome は sh の script で argv を記録し、席の目の Chrome の pipe の相手と
//! 端末の Chrome は歯の中の thread で話す（本物の ssh と Chrome と tailnet の道具と器を撃たず網に出ない）。

use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt::Debug;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{self, Command as Process, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use tsuzuri_boundary::stage::cdp::Command;
use tsuzuri_boundary::stage::cli::{
    self, Call, LOCK_WAIT, NEW_PAGE, SEAT_ENV, TIMEOUT, USAGE, VALIDATE_ARGS, Verb,
};
use tsuzuri_boundary::stage::json;
use tsuzuri_boundary::stage::launch;
use tsuzuri_boundary::stage::terminal::{self, Terminal};
use tsuzuri_boundary::stage::tunnel::{self, Tunnel};
use tsuzuri_boundary::stage::url::{self, Board};
use tsuzuri_boundary::stage::ws;

/// contracts の verify の filter の語のうち、ほかの語を部分の字として含まない最小の語と行 i-4 の接頭辞 relay_
/// （この行の接頭辞 stcli_ は並べない）。
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
    "topbar_ tz_ urpanel_ uword_ wstrip_ relay_",
);

/// 節の board の URL（名は偽物・予約の頂の invalid）。
const BOARD_URL: &str = "http://srv-a.tailnet.invalid:4801/";

/// 節の URL の行。
const LINE: &str = "board の URL http://srv-a.tailnet.invalid:4801/";

/// 節の開発中の app の URL。
const NEXT: &str = "http://127.0.0.1:4173/next";

/// 節の board の住所の URL（文書の例の住所）。
const ADDRESS: &str = "http://192.0.2.7:4801/x";

/// 節の写真の base64 と、その 12 byte（PNG の頭の 8 byte と字 fake）。
const PNG: &str = "iVBORw0KGgpmYWtl";
const PNG_BYTES: &[u8; 12] = b"\x89PNG\r\n\x1a\nfake";

/// 節の DOM の字。
const DOM: &str = "<html><body>eyes</body></html>";

/// 偽の席の目の Chrome が Runtime.enable の前に送る console の event と、sessionId の付いたその字。
const CONSOLE: &str = r#"{"method":"Runtime.consoleAPICalled","params":{"type":"log","args":["hello"]}}"#;
const CONSOLE_LINE: &str = r#"{"sessionId":"S1","method":"Runtime.consoleAPICalled","params":{"type":"log","args":["hello"]}}"#;

/// 読み込みの終わりの event の字。
const LOAD: &str = r#"{"method":"Page.loadEventFired","params":{"timestamp":1}}"#;

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

/// 節の status の self_hosts。
const HOSTS: [&str; 6] = [
    "srv-a.tailnet.invalid",
    "srv-a",
    "192.0.2.7",
    "2001:db8::7",
    "127.0.0.1",
    "localhost",
];

/// 偽の端末の Chrome の /json/version の本文。
const VERSION: &str = r#"{"Browser":"Chrome/146.0.0.0","Protocol-Version":"1.3","webSocketDebuggerUrl":"ws://localhost/devtools/browser/B1"}"#;

/// 偽の端末の Chrome の /json/list の本文（service_worker の後に頁）。
const LIST: &str = r#"[ {
   "id": "S1",
   "type": "service_worker",
   "url": "chrome-extension://x/bg.js",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/S1"
}, {
   "id": "P1",
   "type": "page",
   "url": "http://srv-a.tailnet.invalid:4801/",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/P1"
} ]
"#;

/// 偽の端末の Chrome の頁の無い /json/list の本文。
const NO_PAGE: &str = r#"[ {
   "id": "S1",
   "type": "service_worker",
   "url": "chrome-extension://x/bg.js",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/S1"
} ]
"#;

/// 節の頁を作る 7 つの字。
const MADE: [&str; 7] = [
    "GET /json/version",
    "GET /json/list",
    "GET /json/version",
    "GET /devtools/browser/B1",
    r#"{"id":1,"method":"Target.createTarget","params":{"url":"http://srv-a.tailnet.invalid:4801/"}}"#,
    "GET /json/version",
    "GET /json/list",
];

/// 節の席の目の method の 14。
const EYES_METHODS: [&str; 14] = [
    "Target.getTargets",
    "Target.attachToTarget",
    "Page.enable",
    "Page.navigate",
    "Page.getNavigationHistory",
    "Input.dispatchMouseEvent",
    "Input.dispatchMouseEvent",
    "Input.dispatchMouseEvent",
    "Page.getNavigationHistory",
    "Input.insertText",
    "Page.captureScreenshot",
    "Runtime.evaluate",
    "Runtime.enable",
    "Log.enable",
];

/// 命令と写真の書き先の組。
type Line = (Command, Option<PathBuf>);

/// 旗の名と値の組の列。
type Flags<'a> = &'a [(&'a str, &'a str)];

/// one の型。
type OneFn = for<'a> fn(&'a str, Flags<'a>) -> Result<Line, String>;

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

/// 節の自分の board（hosts は節の status の self_hosts）。
fn own() -> Board {
    let hosts = url::self_hosts(STATUS).expect("節の status の self_hosts");
    assert_eq!(hosts, HOSTS);
    Board {
        url: BOARD_URL.to_string(),
        hosts,
        port: 4801,
    }
}

/// 既定の program と repo の Call。
fn plain(verb: Verb, to: &str) -> Call {
    Call {
        verb,
        to: to.to_string(),
        repo: PathBuf::from("."),
        ssh: OsString::from("ssh"),
        scribe2: OsString::from("scribe2"),
        git: OsString::from("git"),
        tailnet: OsString::from("tailscale"),
        chrome: OsString::from("google-chrome"),
    }
}

/// 1 つの命令の Verb（写真の書き先の無いもの）。
fn one(command: Command) -> Verb {
    Verb::One(command, None)
}

/// この process の持ち主の uid。
fn uid() -> u32 {
    fs::metadata("/proc/self").expect("/proc/self").uid()
}

/// 権限の下 9 bit。
fn mode(path: &Path) -> u32 {
    fs::metadata(path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .permissions()
        .mode()
        & 0o777
}

/// Err の字（Ok なら落ちる）。
fn err<T: Debug>(got: Result<T, String>, what: &str) -> String {
    match got {
        Ok(v) => panic!("{what}: Ok {v:?}"),
        Err(e) => e,
    }
}

/// 偽の ssh の tunnel の回の振る舞い。
#[derive(Clone, Copy)]
enum Ssh {
    Sleep,
    Fail,
}

/// 偽の場（作業場・repo・state dir・記録の置き場・2 つの FIFO・偽の program・場の一時の dir・印の file）。
struct Field {
    root: PathBuf,
    repo: PathBuf,
    state: PathBuf,
    records: PathBuf,
    input: PathBuf,
    output: PathBuf,
    ssh: PathBuf,
    scribe2: PathBuf,
    git: PathBuf,
    tailnet: PathBuf,
    chrome: PathBuf,
    tmp: PathBuf,
    socket: PathBuf,
    on: PathBuf,
    broken: PathBuf,
}

impl Field {
    fn new(name: &str, ssh: Ssh) -> Field {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("stcli")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let records = root.join("records");
        let repo = root.join("repo");
        let state = root.join("state");
        for dir in [&records, &repo, &state] {
            fs::create_dir_all(dir).expect("場の dir");
        }
        fs::write(state.join(terminal::FACE), fixture()).expect("host の面");
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
        let tmp = env::temp_dir().join(format!("stcli-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).expect("場の一時の dir");
        let socket = tmp.join("far.sock");
        let on = root.join("on");
        let broken = root.join("broken");
        let tunnel = match ssh {
            Ssh::Sleep => format!(
                "for a in \"$@\"; do\n    if [ \"$prev\" = -L ]; then ln -s '{}' \"${{a%%:*}}\"; fi\n    prev=$a\n  done\n  exec sleep 30",
                socket.display()
            ),
            Ssh::Fail => "exit 255".to_string(),
        };
        let ssh = fake(
            &root,
            "ssh",
            &format!(
                "if [ \"$1\" = -N ]; then\n  {tunnel}\nfi\n: > '{}'\nexit 0\n",
                on.display()
            ),
        );
        let scribe2 = fake(
            &root,
            "scribe2",
            &format!("if [ -e '{}' ]; then exit 1; fi\nexit 0\n", broken.display()),
        );
        let git = fake(
            &root,
            "git",
            &format!(
                "case \"$5\" in\n  scribe2.statedir) printf '%s\\n' '{}' ;;\n  tsuzuri.boardport) printf '4801\\n' ;;\n  *) exit 1 ;;\nesac\n",
                state.display()
            ),
        );
        let tailnet = fake(&root, "tailnet", &format!("exec cat '{}'\n", status.display()));
        let chrome = fake(
            &root,
            "chrome",
            &format!(
                "cat <&3 > '{}' &\nexec cat '{}' >&4\n",
                input.display(),
                output.display()
            ),
        );
        Field {
            root,
            repo,
            state,
            records,
            input,
            output,
            ssh,
            scribe2,
            git,
            tailnet,
            chrome,
            tmp,
            socket,
            on,
            broken,
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

    /// 場の一時の dir の下の口座の dir。
    fn user(&self) -> PathBuf {
        self.tmp.join(format!("tzst-u{}", uid()))
    }

    fn text(path: &Path) -> String {
        path.to_str().expect("path の字").to_string()
    }

    /// tz stage を偽の program で撃つ（標準入力に input・seat なら環境変数 CLAUDECODE を 1 に置く）。
    /// rc と標準出力の行を返す。
    fn tz(&self, args: &[&str], input: &str, seat: bool) -> (i32, Vec<String>) {
        let mut shot = Process::new(env!("CARGO_BIN_EXE_tz"));
        shot.arg("stage")
            .args(args)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--ssh")
            .arg(&self.ssh)
            .arg("--scribe2")
            .arg(&self.scribe2)
            .arg("--git")
            .arg(&self.git)
            .arg("--tailnet")
            .arg(&self.tailnet)
            .arg("--chrome")
            .arg(&self.chrome)
            .env_remove(SEAT_ENV)
            .env("TMPDIR", &self.tmp)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if seat {
            shot.env(SEAT_ENV, "1");
        }
        let mut child = shot.spawn().expect("tz を撃つ");
        let mut stdin = child.stdin.take().expect("tz の標準入力");
        stdin.write_all(input.as_bytes()).expect("tz の標準入力に書く");
        drop(stdin);
        let out = child.wait_with_output().expect("tz を待つ");
        let text = String::from_utf8(out.stdout).expect("tz の標準出力の字");
        (
            out.status.code().unwrap_or(-1),
            text.lines().map(str::to_string).collect(),
        )
    }
}

impl Drop for Field {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.tmp);
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

/// 偽の席の目の Chrome の pipe の相手（in.fifo を読み out.fifo へ書き、受けた message を順に記す・最初の頁は board）。
fn serve(field: &Field) -> JoinHandle<Vec<String>> {
    let input = field.input.clone();
    let output = field.output.clone();
    thread::spawn(move || {
        let mut reader = BufReader::new(File::open(&input).expect("in.fifo を開く"));
        let mut writer = OpenOptions::new()
            .write(true)
            .open(&output)
            .expect("out.fifo を開く");
        let mut got = Vec::new();
        let mut now = BOARD_URL.to_string();
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
                "Target.getTargets" => vec![format!(
                    r#"{{"id":{id},"result":{{"targetInfos":[{{"targetId":"T1","type":"page","url":{}}}]}}}}"#,
                    json::escape(&now)
                )],
                "Target.attachToTarget" => {
                    vec![format!(r#"{{"id":{id},"result":{{"sessionId":"S1"}}}}"#)]
                }
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

/// 偽の端末の Chrome（場の socket で待ち受け、受けた要求の行と websocket の字を記す thread）。
struct Far {
    stop: Arc<AtomicBool>,
    thread: JoinHandle<Vec<String>>,
}

impl Far {
    fn start(field: &Field, list: &'static str) -> Far {
        let listener = UnixListener::bind(&field.socket)
            .unwrap_or_else(|e| panic!("{}: {e}", field.socket.display()));
        listener.set_nonblocking(true).expect("待ち受けを止めない形に");
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let on = field.on.clone();
        let thread = thread::spawn(move || {
            let mut got = Vec::new();
            let mut list = list;
            while !flag.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => answer(stream, &on, &mut list, &mut got),
                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => panic!("接続を受ける: {e}"),
                }
            }
            got
        });
        Far { stop, thread }
    }

    /// thread を止めて受けた字を返す。
    fn stop(self) -> Vec<String> {
        self.stop.store(true, Ordering::SeqCst);
        self.thread.join().expect("偽の端末の Chrome")
    }
}

/// 1 つの接続（印の file が無ければ何も書かずに閉じる・在れば /json/version と /json/list に答え、
/// browser の target の websocket では id 1 の応答を返してその後の /json/list を LIST にする）。
fn answer(mut stream: UnixStream, on: &Path, list: &mut &'static str, got: &mut Vec<String>) {
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
    got.push(line.to_string());
    if !on.exists() {
        return;
    }
    let body = match line {
        "GET /json/version" => VERSION,
        "GET /json/list" => *list,
        "GET /devtools/browser/B1" => {
            let key = head
                .split("\r\n")
                .filter_map(|l| l.split_once(':'))
                .find(|(n, _)| n.trim().eq_ignore_ascii_case("Sec-WebSocket-Key"))
                .map(|(_, v)| v.trim().to_string())
                .expect("Sec-WebSocket-Key");
            let reply = format!(
                "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
                ws::accept(&key)
            );
            stream.write_all(reply.as_bytes()).expect("握手の応答");
            let Some((1, payload)) = frame(&mut stream) else {
                got.push("字の frame が無い".to_string());
                return;
            };
            let text = String::from_utf8(payload).expect("字の frame");
            got.push(text.clone());
            if text.contains(NEW_PAGE) {
                *list = LIST;
            }
            let id = json::member(&text, "id").unwrap_or("0");
            let reply = format!(r#"{{"id":{id},"result":{{"targetId":"P2"}}}}"#);
            let mut out = vec![0x81, reply.len() as u8];
            out.extend_from_slice(reply.as_bytes());
            stream.write_all(&out).expect("応答の frame");
            if matches!(frame(&mut stream), Some((8, _))) {
                let _ = stream.write_all(&[0x88, 2, 0x03, 0xe8]);
            }
            return;
        }
        _ => return,
    };
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=UTF-8\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    if stream.write_all(head.as_bytes()).is_err() || stream.write_all(body.as_bytes()).is_err() {
        return;
    }
    let mut rest = [0u8; 64];
    while matches!(stream.read(&mut rest), Ok(n) if n > 0) {}
}

/// client の mask つきの 1 つの frame（opcode と mask を解いた payload）。
fn frame(stream: &mut UnixStream) -> Option<(u8, Vec<u8>)> {
    let mut head = [0u8; 2];
    stream.read_exact(&mut head).ok()?;
    let len = match head[1] & 0x7f {
        126 => {
            let mut b = [0u8; 2];
            stream.read_exact(&mut b).ok()?;
            usize::from(u16::from_be_bytes(b))
        }
        127 => {
            let mut b = [0u8; 8];
            stream.read_exact(&mut b).ok()?;
            usize::try_from(u64::from_be_bytes(b)).ok()?
        }
        n => usize::from(n),
    };
    let mut key = [0u8; 4];
    stream.read_exact(&mut key).ok()?;
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).ok()?;
    for (i, b) in payload.iter_mut().enumerate() {
        *b ^= key[i % 4];
    }
    Some((head[0] & 0x0f, payload))
}

/// Verb の変種の名（wildcard の腕を持たない・変種が足されると組めない）。
fn verb_name(verb: &Verb) -> &'static str {
    match verb {
        Verb::One(command, out) => {
            let _: (&Command, &Option<PathBuf>) = (command, out);
            "one"
        }
        Verb::Run => "run",
        Verb::Open => "open",
    }
}

fn debug<T: Debug + Clone + PartialEq + Eq>() {}

#[test]
fn stcli_shape_and_consts() {
    let usage: &str = USAGE;
    assert!(usage.starts_with("usage: tz stage"), "{usage}");
    let seat: &str = SEAT_ENV;
    assert_eq!(seat, "CLAUDECODE");
    let validate: [&str; 2] = VALIDATE_ARGS;
    assert_eq!(validate, ["validate", "--state-dir"]);
    let timeout: Duration = TIMEOUT;
    assert_eq!(timeout, Duration::from_secs(10));
    let wait: Duration = LOCK_WAIT;
    assert_eq!(wait, Duration::from_secs(30));
    let new_page: &str = NEW_PAGE;
    assert_eq!(new_page, "Target.createTarget");

    let run: fn(&[&str]) -> u8 = cli::run;
    let parse: fn(&[&str]) -> Result<Call, String> = cli::parse;
    let one_fn: OneFn = cli::one;
    let lines: fn(&str) -> Result<Vec<Line>, String> = cli::lines;
    let seat_refusal: fn(Option<&OsStr>) -> Option<String> = cli::seat_refusal;
    let on_board: fn(&Command, &str, &Board) -> bool = cli::on_board;
    let lock: fn(&Path, &str, Duration) -> Result<File, String> = cli::lock;
    let browser_resource: fn(&str) -> Option<String> = cli::browser_resource;
    let user_dir: fn(&Path) -> Result<PathBuf, String> = tunnel::user_dir;
    let version: fn(&Tunnel) -> Option<String> = Tunnel::version;
    let _ = (run, parse, one_fn, lines, seat_refusal, on_board, lock, browser_resource);
    let _ = (user_dir, version);
    debug::<Verb>();
    debug::<Call>();
    let verbs = [
        Verb::One(Command::Screenshot, Some(PathBuf::from("/o/p.png"))),
        Verb::Run,
        Verb::Open,
    ];
    let names: Vec<&str> = verbs.iter().map(verb_name).collect();
    assert_eq!(names, ["one", "run", "open"]);
    let call = plain(Verb::Run, "term-a");
    assert_eq!(call.clone(), call);
    let Call {
        verb,
        to,
        repo,
        ssh,
        scribe2,
        git,
        tailnet,
        chrome,
    } = call;
    let _: (Verb, String, PathBuf) = (verb, to, repo);
    let _: [OsString; 5] = [ssh, scribe2, git, tailnet, chrome];
}

#[test]
fn stcli_parse_table() {
    let full = Call {
        verb: one(Command::Viewport {
            width: 390,
            height: 844,
            scale: 2,
            mobile: true,
        }),
        to: "term-b".to_string(),
        repo: PathBuf::from("/r"),
        ssh: OsString::from("/f/ssh"),
        scribe2: OsString::from("/f/s2"),
        git: OsString::from("/f/git"),
        tailnet: OsString::from("/f/tn"),
        chrome: OsString::from("/f/c"),
    };
    let table: Vec<(Vec<&str>, Call)> = vec![
        (
            vec!["navigate", "--to", "term-a", "--url", NEXT],
            plain(
                one(Command::Navigate {
                    url: NEXT.to_string(),
                }),
                "term-a",
            ),
        ),
        (
            vec![
                "viewport",
                "--to=term-b",
                "--width",
                "390",
                "--height=844",
                "--scale",
                "2",
                "--mobile",
                "true",
                "--repo",
                "/r",
                "--ssh",
                "/f/ssh",
                "--scribe2",
                "/f/s2",
                "--git",
                "/f/git",
                "--tailnet",
                "/f/tn",
                "--chrome",
                "/f/c",
            ],
            full,
        ),
        (
            vec!["reload", "--to", "term-a"],
            plain(one(Command::Reload), "term-a"),
        ),
        (
            vec!["click", "--to", "term-a", "--x", "10", "--y", "20"],
            plain(one(Command::Click { x: 10, y: 20 }), "term-a"),
        ),
        (
            vec!["type", "--to", "term-a", "--text", "a b"],
            plain(
                one(Command::Type {
                    text: "a b".to_string(),
                }),
                "term-a",
            ),
        ),
        (
            vec!["key", "--to", "term-a", "--key", "Enter"],
            plain(
                one(Command::Key {
                    key: "Enter".to_string(),
                }),
                "term-a",
            ),
        ),
        (
            vec![
                "scroll", "--to", "term-a", "--x", "1", "--y", "2", "--dy", "-300",
            ],
            plain(
                one(Command::Scroll {
                    x: 1,
                    y: 2,
                    dy: -300,
                }),
                "term-a",
            ),
        ),
        (
            vec!["wait", "--to", "term-a", "--ms", "250"],
            plain(one(Command::Wait { ms: 250 }), "term-a"),
        ),
        (
            vec!["screenshot", "--to", "term-a", "--out", "/o/p.png"],
            plain(
                Verb::One(Command::Screenshot, Some(PathBuf::from("/o/p.png"))),
                "term-a",
            ),
        ),
        (
            vec!["dom", "--to", "term-a"],
            plain(one(Command::Dom), "term-a"),
        ),
        (
            vec!["console", "--to", "term-a"],
            plain(one(Command::Console), "term-a"),
        ),
        (
            vec![
                "viewport", "--to", "term-a", "--width", "1", "--height", "2", "--scale", "1",
                "--mobile", "false",
            ],
            plain(
                one(Command::Viewport {
                    width: 1,
                    height: 2,
                    scale: 1,
                    mobile: false,
                }),
                "term-a",
            ),
        ),
        (vec!["run", "--to", "term-a"], plain(Verb::Run, "term-a")),
        (vec!["open", "--to", "term-a"], plain(Verb::Open, "term-a")),
    ];
    for (args, want) in table {
        assert_eq!(cli::parse(&args), Ok(want), "{args:?}");
    }

    let refusals: [(&[&str], &str); 20] = [
        (&[], "命令が無い"),
        (&["navigate", "--url", NEXT], "--to"),
        (&["jump", "--to", "term-a"], "知らない命令 jump"),
        (&["click", "--to", "term-a", "--x", "10"], "--y"),
        (
            &["click", "--to", "term-a", "--x", "1", "--x", "2", "--y", "3"],
            "--x が 2 度",
        ),
        (
            &["navigate", "--to", "term-a", "--url", "javascript:alert(1)"],
            "--url",
        ),
        (
            &["navigate", "--to", "term-a", "--url", "file:///fake/x.html"],
            "--url",
        ),
        (&["open", "--to", "term-a", "--url", NEXT], "open は旗 --url"),
        (&["run", "--to", "term-a", "--x", "1"], "run は旗 --x"),
        (
            &["reload", "--to", "term-a", "--display", ":1"],
            "reload は旗 --display",
        ),
        (
            &["reload", "--to", "term-a", "--profile-dir", "/p"],
            "reload は旗 --profile-dir",
        ),
        (
            &["reload", "--to", "term-a", "--port", "9224"],
            "reload は旗 --port",
        ),
        (&["dom", "--to", "term-a", "--to", "term-b"], "--to が 2 度"),
        (&["dom", "--to"], "--to の値が無い"),
        (&["dom", "--to="], "--to の値が空"),
        (&["dom", "term-a"], "旗でない字 term-a"),
        (
            &[
                "viewport", "--to", "term-a", "--width", "1", "--height", "2", "--scale", "1",
                "--mobile", "yes",
            ],
            "--mobile",
        ),
        (&["click", "--to", "term-a", "--x", "-1", "--y", "2"], "--x"),
        (&["wait", "--to", "term-a", "--ms", "x"], "--ms"),
        (&["screenshot", "--to", "term-a"], "--out"),
    ];
    for (args, word) in refusals {
        let e = err(cli::parse(args), &format!("{args:?}"));
        assert!(e.contains(word), "{args:?}: {e}");
    }
}

#[test]
fn stcli_lines_table() {
    let input = format!(
        "[\"navigate\",\"--url\",\"{NEXT}\"]\n\n  [\"click\", \"--x\", \"10\", \"--y\", \"20\"]  \n[\"screenshot\",\"--out=/o/a.png\"]\n[\"dom\"]\n"
    );
    let want: Vec<Line> = vec![
        (
            Command::Navigate {
                url: NEXT.to_string(),
            },
            None,
        ),
        (Command::Click { x: 10, y: 20 }, None),
        (Command::Screenshot, Some(PathBuf::from("/o/a.png"))),
        (Command::Dom, None),
    ];
    assert_eq!(cli::lines(&input), Ok(want));
    assert_eq!(cli::lines(""), Ok(Vec::new()));
    let refusals: [(&str, &str); 7] = [
        (r#"{"verb":"dom"}"#, "1 行目"),
        (r#"["dom",1]"#, "1 行目"),
        ("[]", "1 行目"),
        (r#"["open"]"#, "知らない命令 open"),
        (r#"["run"]"#, "知らない命令 run"),
        (r#"["dom","--to","term-a"]"#, "dom は旗 --to"),
        ("[\"dom\"]\n[\"click\",\"--x\",\"1\"]", "2 行目"),
    ];
    for (text, word) in refusals {
        let e = err(cli::lines(text), text);
        assert!(e.contains(word), "{text}: {e}");
    }
}

#[test]
fn stcli_guard_tables() {
    for value in [None, Some("0"), Some("")] {
        assert_eq!(cli::seat_refusal(value.map(OsStr::new)), None, "{value:?}");
    }
    let refused = cli::seat_refusal(Some(OsStr::new("1"))).expect("字 1 は Some");
    for word in ["tz stage open", "CLAUDECODE"] {
        assert!(refused.contains(word), "{word}: {refused}");
    }

    let click = Command::Click { x: 1, y: 2 };
    let typed = Command::Type {
        text: "a".to_string(),
    };
    let key = Command::Key {
        key: "Enter".to_string(),
    };
    let table: [(&Command, &str, bool); 12] = [
        (&click, BOARD_URL, true),
        (&click, ADDRESS, true),
        (&typed, "http://srv-a:4801/", true),
        (&typed, "http://srv-a.tailnet.invalid:4801/p/q?x=1", true),
        (&key, "http://SRV-A.TAILNET.INVALID.:4801/", true),
        (&key, "http://127.0.0.1:4801/", true),
        (&Command::Scroll { x: 1, y: 2, dy: -3 }, BOARD_URL, false),
        (
            &Command::Navigate {
                url: NEXT.to_string(),
            },
            BOARD_URL,
            false,
        ),
        (&Command::Screenshot, BOARD_URL, false),
        (&click, NEXT, false),
        (&click, "http://srv-a.tailnet.invalid:4802/", false),
        (&click, "about:blank", false),
    ];
    let own = own();
    for (command, page, want) in table {
        assert_eq!(cli::on_board(command, page, &own), want, "{command:?} {page}");
    }

    assert_eq!(
        cli::browser_resource(VERSION).as_deref(),
        Some("/devtools/browser/B1")
    );
    assert_eq!(
        cli::browser_resource(r#"{"webSocketDebuggerUrl":"ws://localhost/devtools/page/P1"}"#),
        None
    );
    assert_eq!(cli::browser_resource("{}"), None);
}

#[test]
fn stcli_lock_waits() {
    let field = Field::new("lock", Ssh::Fail);
    let second = Duration::from_secs(1);
    let held = cli::lock(&field.tmp, "term-a", second).expect("始めの錠");
    assert_eq!(mode(&field.tmp.join("tzst-term-a.lock")), 0o600);
    let start = Instant::now();
    let e = err(cli::lock(&field.tmp, "term-a", second), "持たれた錠");
    let took = start.elapsed();
    assert!(
        took > Duration::from_millis(900) && took < Duration::from_secs(5),
        "{took:?}"
    );
    assert!(e.contains("端末 term-a"), "{e}");
    let other = cli::lock(&field.tmp, "term-b", second).expect("別の端末の錠");
    drop(held);
    let again = cli::lock(&field.tmp, "term-a", second).expect("放した後の錠");
    drop((other, again));
    for bad in ["../x", ".hidden", "", "a b"] {
        err(cli::lock(&field.tmp, bad, second), bad);
    }
}

#[test]
fn stcli_open_refused_in_seat() {
    let field = Field::new("seat", Ssh::Sleep);
    let (rc, out) = field.tz(&["open", "--to", "term-a"], "", true);
    assert_eq!(rc, 1, "{out:?}");
    let refused = cli::seat_refusal(Some(OsStr::new("1"))).expect("字 1 は Some");
    assert_eq!(out, [refused.as_str(), LINE]);
    assert!(field.records("ssh").is_empty(), "偽の ssh が撃たれた");
    assert!(field.records("scribe2").is_empty(), "偽の器が撃たれた");
    let repo = Field::text(&field.repo);
    assert_eq!(
        field.records("git"),
        [strings(&["-C", &repo, "config", "--get", "tsuzuri.boardport"])]
    );
    assert_eq!(field.records("tailnet"), [strings(&["status", "--json"])]);
}

#[test]
fn stcli_validate_gate() {
    let field = Field::new("gate", Ssh::Fail);
    fs::write(&field.broken, "").expect("印の file");
    let state = Field::text(&field.state);
    let (rc, out) = field.tz(&["dom", "--to", "term-b"], "", false);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out.len(), 2, "{out:?}");
    for word in ["validate --state-dir", state.as_str(), "rc 0"] {
        assert!(out[0].contains(word), "{word}: {}", out[0]);
    }
    assert_eq!(out[1], LINE);
    assert_eq!(
        field.records("scribe2"),
        [strings(&["validate", "--state-dir", &state])]
    );
    assert!(field.records("ssh").is_empty(), "偽の ssh が撃たれた");
    assert!(field.records("chrome").is_empty(), "偽の席の目の Chrome が撃たれた");

    fs::remove_file(&field.broken).expect("印の file を消す");
    let (rc, out) = field.tz(&["dom", "--to", "term-z"], "", false);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out.len(), 2, "{out:?}");
    for word in ["端末 term-z", "term-a・term-b"] {
        assert!(out[0].contains(word), "{word}: {}", out[0]);
    }
    assert_eq!(out[1], LINE);
    assert!(field.records("ssh").is_empty(), "偽の ssh が撃たれた");
}

#[test]
fn stcli_eyes_run() {
    let field = Field::new("eyes", Ssh::Fail);
    let server = serve(&field);
    let shot = field.root.join("shot.png");
    let shot_text = Field::text(&shot);
    let input = format!(
        "[\"navigate\",\"--url\",\"{NEXT}\"]\n[\"click\",\"--x\",\"10\",\"--y\",\"20\"]\n[\"type\",\"--text\",\"abc\"]\n[\"screenshot\",\"--out\",{}]\n[\"dom\"]\n[\"console\"]\n",
        json::escape(&shot_text)
    );
    let (rc, out) = field.tz(&["run", "--to", "term-b"], &input, false);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(out.len(), 10, "{out:?}");
    for word in ["端末 term-b", "255", "席の目"] {
        assert!(out[0].contains(word), "{word}: {}", out[0]);
    }
    let screenshot = format!("済み screenshot {shot_text} 12 byte");
    let want = [
        "済み navigate",
        "済み click",
        "済み type",
        screenshot.as_str(),
        DOM,
        "済み dom",
        CONSOLE_LINE,
        "済み console",
        LINE,
    ];
    assert_eq!(out[1..], want);
    assert_eq!(fs::read(&shot).expect("写真の file"), PNG_BYTES);
    let ssh = field.records("ssh");
    assert_eq!(ssh.len(), 1, "{ssh:?}");
    let socket = ssh[0]
        .get(10)
        .and_then(|v| v.split_once(':'))
        .map_or("", |(path, _)| path);
    assert_eq!(ssh[0], launch::tunnel_argv(&term("term-b"), Path::new(socket)));
    let chrome = field.records("chrome");
    assert_eq!(chrome.len(), 1, "{chrome:?}");
    assert_eq!(chrome[0].get(5).map(String::as_str), Some(BOARD_URL));
    let got = server.join().expect("偽の席の目の Chrome");
    assert_eq!(methods(&got), EYES_METHODS);
}

#[test]
fn stcli_board_refused() {
    let field = Field::new("board", Ssh::Fail);
    let server = serve(&field);
    let input = format!(
        "[\"scroll\",\"--x\",\"1\",\"--y\",\"2\",\"--dy\",\"100\"]\n[\"navigate\",\"--url\",\"{ADDRESS}\"]\n[\"click\",\"--x\",\"10\",\"--y\",\"20\"]\n[\"dom\"]\n"
    );
    let (rc, out) = field.tz(&["run", "--to", "term-b"], &input, false);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out.len(), 5, "{out:?}");
    assert_eq!(out[1], "済み scroll");
    assert_eq!(out[2], "済み navigate");
    for word in ["board の頁", ADDRESS, "の上では click を断る"] {
        assert!(out[3].contains(word), "{word}: {}", out[3]);
    }
    assert_eq!(out[4], LINE);
    let got = server.join().expect("偽の席の目の Chrome");
    assert_eq!(
        methods(&got),
        [
            "Target.getTargets",
            "Target.attachToTarget",
            "Input.dispatchMouseEvent",
            "Page.enable",
            "Page.navigate",
            "Page.getNavigationHistory",
        ]
    );

    let field = Field::new("boardkey", Ssh::Fail);
    let server = serve(&field);
    let (rc, out) = field.tz(&["key", "--to", "term-b", "--key", "Enter"], "", false);
    assert_eq!(rc, 1, "{out:?}");
    assert!(
        out.get(1).is_some_and(|l| l.contains("の上では key を断る")),
        "{out:?}"
    );
    let got = server.join().expect("偽の席の目の Chrome");
    let methods = methods(&got);
    assert!(
        methods.iter().all(|m| !m.starts_with("Input")),
        "{methods:?}"
    );
}

#[test]
fn stcli_open_reuses_window() {
    let field = Field::new("reuse", Ssh::Sleep);
    fs::write(&field.on, "").expect("印の file");
    let far = Far::start(&field, LIST);
    let (rc, out) = field.tz(&["open", "--to", "term-a"], "", false);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(out, ["端末 term-a の表示面の窓は在る（起こさない）", LINE]);
    assert_eq!(far.stop(), ["GET /json/version", "GET /json/list"]);
    assert_eq!(field.records("ssh").len(), 1);
    assert_eq!(mode(&field.user().join("tzst-term-a.lock")), 0o600);
}

#[test]
fn stcli_open_raises_once() {
    let field = Field::new("raise", Ssh::Sleep);
    let far = Far::start(&field, LIST);
    let (rc, out) = field.tz(&["open", "--to", "term-a"], "", false);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(out, ["端末 term-a に表示面の窓を起こした", LINE]);
    let ssh = field.records("ssh");
    assert_eq!(ssh.len(), 2, "{ssh:?}");
    assert_eq!(
        ssh[1],
        launch::launch_argv(&term("term-a"), BOARD_URL).expect("launch_argv")
    );
    let got = far.stop();
    assert!(got.len() >= 3, "{got:?}");
    assert_eq!(got[..2], ["GET /json/version", "GET /json/version"]);
    assert!(got[2..].iter().all(|l| l == "GET /json/list"), "{got:?}");
}

#[test]
fn stcli_open_makes_one_page() {
    let field = Field::new("page", Ssh::Sleep);
    fs::write(&field.on, "").expect("印の file");
    let far = Far::start(&field, NO_PAGE);
    let (rc, out) = field.tz(&["open", "--to", "term-a"], "", false);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(out, ["端末 term-a の表示面の Chrome に頁の窓を 1 つ開いた", LINE]);
    let got = far.stop();
    assert_eq!(got, MADE);
    assert_eq!(got.iter().filter(|l| l.contains(NEW_PAGE)).count(), 1);
    assert_eq!(field.records("ssh").len(), 1);
}

#[test]
fn stcli_navigate_keeps_closed() {
    let field = Field::new("closed", Ssh::Sleep);
    let far = Far::start(&field, LIST);
    let (rc, out) = field.tz(&["navigate", "--to", "term-a", "--url", NEXT], "", false);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out.len(), 2, "{out:?}");
    for word in ["端末 term-a", "tz stage open"] {
        assert!(out[0].contains(word), "{word}: {}", out[0]);
    }
    assert_eq!(out[1], LINE);
    assert_eq!(far.stop(), ["GET /json/version"]);
    assert_eq!(field.records("ssh").len(), 1);
    assert!(!field.on.exists(), "窓を起こす回の印の file が在る");
}

#[test]
fn stcli_open_waits_for_lock() {
    let field = Field::new("wait", Ssh::Sleep);
    fs::write(&field.on, "").expect("印の file");
    let far = Far::start(&field, LIST);
    let base = tunnel::user_dir(&field.tmp).expect("user_dir");
    let held = cli::lock(&base, "term-a", Duration::from_secs(1)).expect("歯の錠");
    let ((rc, out), took) = thread::scope(|s| {
        let shot = s.spawn(|| {
            let start = Instant::now();
            let got = field.tz(&["open", "--to", "term-a"], "", false);
            (got, start.elapsed())
        });
        thread::sleep(Duration::from_secs(1));
        assert!(field.records("ssh").is_empty(), "錠を持つ間に偽の ssh が撃たれた");
        drop(held);
        shot.join().expect("tz の thread")
    });
    assert_eq!(rc, 0, "{out:?}");
    assert!(took > Duration::from_secs(1), "{took:?}");
    assert_eq!(out, ["端末 term-a の表示面の窓は在る（起こさない）", LINE]);
    far.stop();
}

#[test]
fn stcli_private_dir_checks() {
    let field = Field::new("private", Ssh::Sleep);
    let dir = field.user();
    assert_eq!(tunnel::user_dir(&field.tmp), Ok(dir.clone()));
    assert_eq!(mode(&dir), 0o700);
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).expect("0755 にする");
    assert_eq!(tunnel::user_dir(&field.tmp), Ok(dir.clone()));
    assert_eq!(mode(&dir), 0o700);

    fs::remove_dir(&dir).expect("口座の dir を消す");
    symlink(&field.root, &dir).expect("symlink を置く");
    let text = Field::text(&dir);
    let e = err(tunnel::user_dir(&field.tmp), "symlink");
    for word in [text.as_str(), "symlink"] {
        assert!(e.contains(word), "{word}: {e}");
    }
    let (rc, out) = field.tz(&["open", "--to", "term-a"], "", false);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out.len(), 2, "{out:?}");
    assert!(out[0].contains(&text), "{}", out[0]);
    assert_eq!(out[1], LINE);
    assert!(field.records("ssh").is_empty(), "偽の ssh が撃たれた");

    fs::remove_file(&dir).expect("symlink を消す");
    fs::write(&dir, "").expect("空の file を置く");
    err(tunnel::user_dir(&field.tmp), "空の file");
}

/// dir の下の rs の file の path（深さの順）。
fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    for entry in entries {
        let path = entry.expect("dir の要素").path();
        if path.is_dir() {
            rs_files(&path, out);
        } else if path.extension() == Some(OsStr::new("rs")) {
            out.push(path);
        }
    }
}

#[test]
fn stcli_source_guards() {
    let cli_text = src("cli.rs");
    assert_eq!(cli_text.matches("Target.createTarget").count(), 1);
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    for name in [
        "tsuzuri-boundary",
        "tsuzuri-core",
        "tsuzuri-contract",
        "tsuzuri-surface",
    ] {
        rs_files(&crates.join(name).join("src"), &mut files);
    }
    assert!(files.len() > 20, "{files:?}");
    let holders: Vec<PathBuf> = files
        .into_iter()
        .filter(|p| fs::read_to_string(p).is_ok_and(|t| t.contains("createTarget")))
        .collect();
    let cli_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/stage/cli.rs");
    assert_eq!(holders.len(), 1, "{holders:?}");
    assert_eq!(
        fs::canonicalize(&holders[0]).expect("持つ file"),
        fs::canonicalize(&cli_path).expect("cli.rs")
    );
    for word in [
        "bringToFront",
        "activateTarget",
        "setWindowBounds",
        "json/new",
        "json/activate",
        "Runtime.evaluate",
        "remote-debugging-port",
        "unsafe",
    ] {
        assert!(!cli_text.contains(word), "cli.rs が {word} を含む");
    }
    for file in [
        "cdp.rs",
        "json.rs",
        "launch.rs",
        "pipe.rs",
        "relay.rs",
        "terminal.rs",
        "tunnel.rs",
        "url.rs",
        "ws.rs",
    ] {
        assert!(!src(file).contains("CLAUDECODE"), "{file} が CLAUDECODE を含む");
    }
    assert_eq!(src("mod.rs").matches("pub mod cli;").count(), 1);
    assert_eq!(
        src("tunnel.rs")
            .matches("pub fn version(&self) -> Option<String>")
            .count(),
        1
    );
}

#[test]
fn stcli_own_names_clean() {
    let words: Vec<&str> = FILTER_WORDS.split_whitespace().collect();
    assert_eq!(words.len(), 126, "filter の語の数");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/stcli.rs");
    let text = fs::read_to_string(&path).expect("tests/stcli.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 17, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("stcli_")
            .unwrap_or_else(|| panic!("{name} は stcli_ で始まらない"));
        for word in &words {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
