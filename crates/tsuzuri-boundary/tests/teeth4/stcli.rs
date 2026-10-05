//! tz stage の口の歯（行 i-5・接頭辞 stcli_）。
//! 端末は fixture の tests/fixtures/stage/terminals.toml を偽の state dir の host.toml に写して引く（名も宛先も path も偽物）。
//! 偽の ssh・器・git・tailnet の道具・席の目の Chrome は sh の script で argv を記録し、席の目の Chrome の pipe の相手と
//! 端末の Chrome は歯の中の thread で話す（本物の ssh と Chrome と tailnet の道具と器を撃たず網に出ない）。
#![cfg(test)]

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{self, Command as Process, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::common::{LINE, LOAD, debug, err, fixture, strings, term};
use tsuzuri_boundary::stage::cdp::Command;
use tsuzuri_boundary::stage::cli::{
    self, Call, LOCK_WAIT, SEAT_ENV, TIMEOUT, USAGE, VALIDATE_ARGS, Verb,
};
use tsuzuri_boundary::stage::json;
use tsuzuri_boundary::stage::launch;
use tsuzuri_boundary::stage::memo;
use tsuzuri_boundary::stage::target::{self, Targets};
use tsuzuri_boundary::stage::terminal;
use tsuzuri_boundary::stage::tunnel::{self, Tunnel};
use tsuzuri_boundary::stage::url::{self, Board};
use tsuzuri_contract::stage::StageTargets;
use tsuzuri_contract::wire;

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

/// 節の term-a の窓が在る時の行（起こさない）。
const LIVE_A: &str =
    "端末 term-a の board http://srv-a.tailnet.invalid:4801/ の表示面の窓は在る（起こさない）";

/// 節の term-a に窓を起こした時の行。
const RAISED_A: &str = "端末 term-a に board http://srv-a.tailnet.invalid:4801/ の表示面の窓を起こした";

/// 節の term-a の印の行。
const MARKED_A: &str =
    "表示先の設定に端末 term-a の印を書いた（持ち主が閉じた後は --to を省いた撃ちで起こし直さない）";

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

/// 節の席の目の method の 18（どの命令の前にも頁の URL を読む）。
const EYES_METHODS: [&str; 18] = [
    "Target.getTargets",
    "Target.attachToTarget",
    "Page.getNavigationHistory",
    "Page.enable",
    "Page.navigate",
    "Page.getNavigationHistory",
    "Input.dispatchMouseEvent",
    "Input.dispatchMouseEvent",
    "Input.dispatchMouseEvent",
    "Page.getNavigationHistory",
    "Input.insertText",
    "Page.getNavigationHistory",
    "Page.captureScreenshot",
    "Page.getNavigationHistory",
    "Runtime.evaluate",
    "Page.getNavigationHistory",
    "Runtime.enable",
    "Log.enable",
];

/// 節の account board の頁（自分の board の port の上で query の組に board=account を持つ）。
const ACCOUNT: &str = "http://srv-a.tailnet.invalid:4801/?board=account";

/// 偽の端末の Chrome の /json/list の本文（頁はほかの project の board）。
const LIST_OTHER: &str = r#"[ {
   "id": "P1",
   "type": "page",
   "url": "http://srv-a.tailnet.invalid:4802/",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/P1"
} ]
"#;

/// 節の開発中の app の頁（自分の board の port と違う port の頁）。
const APP: &str = "http://127.0.0.1:4173/app";

/// 偽の端末の Chrome の /json/list の本文（頁は開発中の app の頁だけ）。
const LIST_APP: &str = r#"[ {
   "id": "P1",
   "type": "page",
   "url": "http://127.0.0.1:4173/app",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/P1"
} ]
"#;

/// 偽の端末の Chrome の /json/list の本文（頁は account board の頁とほかの project の board の頁）。
const LIST_FOREIGN: &str = r#"[ {
   "id": "P1",
   "type": "page",
   "url": "http://srv-a.tailnet.invalid:4801/?board=account",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/P1"
}, {
   "id": "P2",
   "type": "page",
   "url": "http://srv-a.tailnet.invalid:4802/",
   "webSocketDebuggerUrl": "ws://localhost/devtools/page/P2"
} ]
"#;

/// 命令と写真の書き先の組。
type Line = (Command, Option<PathBuf>);

/// 旗の名と値の組の列。
type Flags<'a> = &'a [(&'a str, &'a str)];

/// one の型。
type OneFn = for<'a> fn(&'a str, Flags<'a>) -> Result<Line, String>;

fn src(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/stage")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
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
        to: Some(to.to_string()),
        repo: PathBuf::from("."),
        ssh: OsString::from("ssh"),
        scribe2: OsString::from("scribe2"),
        git: OsString::from("git"),
        tailnet: OsString::from("tailscale"),
        chrome: OsString::from("google-chrome"),
        config: None,
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

/// 偽の ssh の tunnel の回の振る舞い。
#[derive(Clone, Copy)]
enum Ssh {
    Sleep,
    Fail,
}

/// 偽の場（作業場・repo・state dir・記録の置き場・2 つの FIFO・偽の program・場の一時の dir・印の file・表示先の設定）。
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
    launched: PathBuf,
    broken: PathBuf,
    config: PathBuf,
}

impl Field {
    fn new(name: &str, ssh: Ssh) -> Field {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("stcli")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (records, repo, state, input, output) = Self::dirs(&root);
        let status = root.join("status");
        fs::write(&status, STATUS).expect("場の status");
        let tmp = env::temp_dir().join(format!("stcli-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).expect("場の一時の dir");
        let socket = tmp.join("far.sock");
        let on = root.join("on");
        let launched = root.join("launched");
        let broken = root.join("broken");
        let config = root.join("cfg").join(target::FILE);
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
                "if [ \"$1\" = -N ]; then\n  {tunnel}\nfi\n: > '{}'\n: > '{}'\nexit 0\n",
                on.display(),
                launched.display()
            ),
        );
        let scribe2 = fake(
            &root,
            "scribe2",
            &format!("if [ -e '{}' ]; then exit 1; fi\nexit 0\n", broken.display()),
        );
        let (git, tailnet, chrome) =
            Self::local_fakes(root.clone(), &state, &status, &input, &output);
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
            launched,
            broken,
            config,
        }
    }

    /// 場の dir と host の面と 2 つの FIFO を作る。
    fn dirs(root: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
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
        (records, repo, state, input, output)
    }

    /// 偽の git と tailnet と chrome を置く。
    fn local_fakes(
        root: PathBuf,
        state: &Path,
        status: &Path,
        input: &Path,
        output: &Path,
    ) -> (PathBuf, PathBuf, PathBuf) {
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
        (git, tailnet, chrome)
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

    /// tz stage を偽の program と場の表示先の設定で撃つ（標準入力に input・seat なら環境変数 CLAUDECODE を 1 に置く）。
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
            .arg("--config")
            .arg(&self.config)
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

    /// 場の表示先の設定の印の表（名と epoch 秒の組）。
    fn shown(&self) -> Vec<(String, u64)> {
        target::load(&self.config)
            .expect("場の表示先の設定")
            .shown
            .into_iter()
            .collect()
    }
}

/// 今の epoch 秒。
fn epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("epoch の後")
        .as_secs()
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
            let (replies, session, next) = respond(buf, &mut got, now);
            now = next;
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

/// 受けた 1 つの message を記し、返す message の列と session と今の頁を返す。
fn respond(
    buf: Vec<u8>,
    got: &mut Vec<String>,
    mut now: String,
) -> (Vec<String>, Option<String>, String) {
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
    (replies, session, now)
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
        Far::start_after(field, list, list)
    }

    /// 頁の一覧は、窓を起こす回の偽の ssh が撃たれるまでは before・撃たれた後は after。
    fn start_after(field: &Field, before: &'static str, after: &'static str) -> Far {
        let listener = UnixListener::bind(&field.socket)
            .unwrap_or_else(|e| panic!("{}: {e}", field.socket.display()));
        listener.set_nonblocking(true).expect("待ち受けを止めない形に");
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let on = field.on.clone();
        let launched = field.launched.clone();
        let thread = thread::spawn(move || {
            let mut got = Vec::new();
            while !flag.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let list = if launched.exists() { after } else { before };
                        answer(stream, &on, list, &mut got);
                    }
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

/// 1 つの接続（印の file が無ければ何も書かずに閉じる・在れば /json/version と /json/list に答える）。
fn answer(mut stream: UnixStream, on: &Path, list: &str, got: &mut Vec<String>) {
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
        "GET /json/list" => list,
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

#[test]
fn stcli_shape_and_consts() {
    let usage: &str = USAGE;
    assert!(usage.starts_with("usage: tz stage"), "{usage}");
    let seat: &str = SEAT_ENV;
    assert_eq!(seat, "CLAUDECODE");
    let validate: [&str; 3] = VALIDATE_ARGS;
    assert_eq!(validate, ["rules", "validate", "--state-dir"]);
    let timeout: Duration = TIMEOUT;
    assert_eq!(timeout, Duration::from_secs(10));
    let wait: Duration = LOCK_WAIT;
    assert_eq!(wait, Duration::from_secs(30));
    let run: fn(&[&str]) -> u8 = cli::run;
    let parse: fn(&[&str]) -> Result<Call, String> = cli::parse;
    let one_fn: OneFn = cli::one;
    let lines: fn(&str) -> Result<Vec<Line>, String> = cli::lines;
    let seat_refusal: fn(Option<&OsStr>) -> Option<String> = cli::seat_refusal;
    let on_board: fn(&Command, &str, &Board) -> bool = cli::on_board;
    let lock: fn(&Path, &str, Duration) -> Result<File, String> = cli::lock;
    let user_dir: fn(&Path) -> Result<PathBuf, String> = tunnel::user_dir;
    let version: fn(&Tunnel) -> Option<String> = Tunnel::version;
    let _ = (run, parse, one_fn, lines, seat_refusal, on_board, lock);
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
        config,
    } = call;
    let _: (Verb, Option<String>, PathBuf) = (verb, to, repo);
    let _: [OsString; 5] = [ssh, scribe2, git, tailnet, chrome];
    let _: Option<PathBuf> = config;
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
        to: Some("term-b".to_string()),
        repo: PathBuf::from("/r"),
        ssh: OsString::from("/f/ssh"),
        scribe2: OsString::from("/f/s2"),
        git: OsString::from("/f/git"),
        tailnet: OsString::from("/f/tn"),
        chrome: OsString::from("/f/c"),
        config: None,
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
    ];
    parse_more(table);
}

/// 読みの表の中ほどの行（click から screenshot まで）を足して続ける。
fn parse_more(mut table: Vec<(Vec<&str>, Call)>) {
    let rest: Vec<(Vec<&str>, Call)> = vec![
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
    ];
    table.extend(rest);
    parse_last(table);
}

/// 読みの表の残りの行（dom から後）を足して、表の全部を撃つ。
fn parse_last(mut table: Vec<(Vec<&str>, Call)>) {
    let rest: Vec<(Vec<&str>, Call)> = vec![
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
        (
            vec!["navigate", "--url", NEXT],
            Call {
                to: None,
                ..plain(
                    one(Command::Navigate {
                        url: NEXT.to_string(),
                    }),
                    "term-a",
                )
            },
        ),
    ];
    table.extend(rest);
    for (args, want) in table {
        assert_eq!(cli::parse(&args), Ok(want), "{args:?}");
    }
    parse_refusals();
}

/// 読みの断りの表（命令と旗と値の形）。
fn parse_refusals() {
    let refusals: [(&[&str], &str); 19] = [
        (&[], "命令が無い"),
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
    for word in ["rules validate --state-dir", state.as_str(), "rc 0"] {
        assert!(out[0].contains(word), "{word}: {}", out[0]);
    }
    assert_eq!(out[1], LINE);
    assert_eq!(
        field.records("scribe2"),
        [strings(&["rules", "validate", "--state-dir", &state])]
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

/// 本物の器の形を写した偽の器の本文（rules validate --state-dir S だけ rc 0・ほかは使い方の 1 行と rc 1）。
const REAL_VESSEL: &str = "if [ \"$#\" -eq 4 ] && [ \"$1\" = rules ] && [ \"$2\" = validate ] && [ \"$3\" = --state-dir ]; then\n  if [ -e \"$4/host.toml\" ]; then host=present; else host=absent; fi\n  printf 'rules: ok rows=82 kinds=80 accounts=7 plugins=1 launch-args=1 host=%s\\n' \"$host\"\n  exit 0\nfi\nprintf 'usage: scribe2 <name|--version|doctor|account|rules|fleet|vessel|hook|host-guard|pipe|runner|lens|seat|polarity|contracts>\\n'\nexit 1\n";

#[test]
fn stcli_vessel_real_form() {
    let mut field = Field::new("vessel", Ssh::Fail);
    field.scribe2 = fake(&field.root, "scribe2", REAL_VESSEL);
    let state = Field::text(&field.state);

    let upper = Process::new(&field.scribe2)
        .args(["validate", "--state-dir", &state])
        .output()
        .expect("偽の器を撃つ");
    assert_eq!(upper.status.code(), Some(1), "上の階の validate は rc 1");
    let upper_out = String::from_utf8(upper.stdout).expect("標準出力の字");
    assert_eq!(upper_out.lines().count(), 1, "{upper_out}");
    assert!(upper_out.starts_with("usage: scribe2 <name|"), "{upper_out}");

    let shown = Process::new(env!("CARGO_BIN_EXE_tz"))
        .args(["stage", "target", "show", "--json"])
        .arg("--repo")
        .arg(&field.repo)
        .arg("--scribe2")
        .arg(&field.scribe2)
        .arg("--git")
        .arg(&field.git)
        .arg("--config")
        .arg(&field.config)
        .env_remove(SEAT_ENV)
        .output()
        .expect("tz を撃つ");
    let stderr = String::from_utf8_lossy(&shown.stderr).to_string();
    assert_eq!(shown.status.code(), Some(0), "{stderr}");
    let text = String::from_utf8(shown.stdout).expect("標準出力の字");
    assert_eq!(text.lines().count(), 1, "{text}");
    let got: StageTargets = wire::decode(text.trim_end()).expect("電文");
    assert_eq!(got.project, "repo");
    assert_eq!(got.names, ["term-a", "term-b"]);

    let (rc, out) = field.tz(&["dom", "--to", "term-z"], "", false);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out.len(), 2, "{out:?}");
    for word in ["端末 term-z", "term-a・term-b"] {
        assert!(out[0].contains(word), "{word}: {}", out[0]);
    }
    assert_eq!(out[1], LINE);

    assert_eq!(
        field.records("scribe2"),
        [
            strings(&["validate", "--state-dir", &state]),
            strings(&["rules", "validate", "--state-dir", &state]),
            strings(&["rules", "validate", "--state-dir", &state]),
        ]
    );
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
            "Page.getNavigationHistory",
            "Input.dispatchMouseEvent",
            "Page.getNavigationHistory",
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
    assert_eq!(out, [LIVE_A, LINE]);
    assert_eq!(
        far.stop(),
        ["GET /json/version", "GET /json/list", "GET /json/list"]
    );
    assert_eq!(field.records("ssh").len(), 1);
    assert_eq!(mode(&field.user().join("tzst-term-a.lock")), 0o600);
    let file = memo::path(&field.user(), "term-a").expect("覚えの path");
    let text = fs::read_to_string(&file).expect("窓の覚え");
    assert_eq!(text, "repo\t/devtools/page/P1\n");
    assert_eq!(mode(&file), 0o600);
}

#[test]
fn stcli_open_names_page() {
    let field = Field::new("named", Ssh::Sleep);
    fs::write(&field.on, "").expect("印の file");
    let base = tunnel::user_dir(&field.tmp).expect("user_dir");
    let file = memo::path(&base, "term-a").expect("覚えの path");
    fs::write(&file, "repo\t/devtools/page/P1\n").expect("窓の覚え");
    let far = Far::start(&field, LIST_APP);
    let (rc, out) = field.tz(&["open", "--to", "term-a"], "", false);
    assert_eq!(rc, 0, "{out:?}");
    let named = format!(
        "端末 term-a の board {BOARD_URL} の表示面の窓は在る（起こさない・頁は {APP}）"
    );
    assert_eq!(out, [named.as_str(), LINE]);
    assert_eq!(
        far.stop(),
        ["GET /json/version", "GET /json/list", "GET /json/list"]
    );
    assert_eq!(field.records("ssh").len(), 1);
    assert_eq!(
        fs::read_to_string(&file).expect("窓の覚え"),
        "repo\t/devtools/page/P1\n"
    );
}

#[test]
fn stcli_foreign_refused() {
    let field = Field::new("foreign", Ssh::Fail);
    let server = serve(&field);
    let input = format!(
        "[\"navigate\",\"--url\",\"{ACCOUNT}\"]\n[\"reload\"]\n[\"dom\"]\n"
    );
    let (rc, out) = field.tz(&["run", "--to", "term-b"], &input, false);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out.len(), 4, "{out:?}");
    assert_eq!(out[1], "済み navigate");
    for word in [ACCOUNT, "自分の project board でない board の頁", "を断る"] {
        assert!(out[2].contains(word), "{word}: {}", out[2]);
    }
    assert_eq!(out[3], LINE);
    let got = server.join().expect("偽の席の目の Chrome");
    assert_eq!(
        methods(&got),
        [
            "Target.getTargets",
            "Target.attachToTarget",
            "Page.getNavigationHistory",
            "Page.enable",
            "Page.navigate",
            "Page.getNavigationHistory",
        ]
    );

    let next = format!(r#"["navigate","--url","{NEXT}"]"#);
    let verbs = [
        ("foreignscroll", r#"["scroll","--x","1","--y","2","--dy","5"]"#),
        ("foreignapp", next.as_str()),
        ("foreigndom", r#"["dom"]"#),
    ];
    for (name, verb) in verbs {
        let field = Field::new(name, Ssh::Fail);
        let server = serve(&field);
        let input = format!("[\"navigate\",\"--url\",\"{ACCOUNT}\"]\n{verb}\n[\"dom\"]\n");
        let (rc, out) = field.tz(&["run", "--to", "term-b"], &input, false);
        assert_eq!(rc, 1, "{name}: {out:?}");
        assert_eq!(out.len(), 4, "{name}: {out:?}");
        assert!(out[2].contains(ACCOUNT) && out[2].contains("を断る"), "{}", out[2]);
        let got = server.join().expect("偽の席の目の Chrome");
        assert_eq!(methods(&got).len(), 6, "{name}: {got:?}");
    }
}

#[test]
fn stcli_foreign_tables() {
    let foreign: fn(&Board, &str, &[u16]) -> bool = Board::foreign;
    let shows: fn(&Board, &str) -> bool = Board::shows;
    let account_page: fn(&str) -> bool = url::account_page;
    let account_query: &str = url::ACCOUNT_QUERY;
    assert_eq!(account_query, "board=account");
    let own = own();
    let ports: &[u16] = &[4802];
    let table: [(&str, bool); 12] = [
        ("http://srv-a.tailnet.invalid:4801/", false),
        ("http://192.0.2.7:4801/x", false),
        (NEXT, false),
        ("about:blank", false),
        ("http://srv-a.tailnet.invalid:4801/#board=account", false),
        ("http://srv-a.tailnet.invalid:4801/?board=accounts", false),
        ("http://other.tailnet.invalid:4802/", false),
        ("http://srv-a.tailnet.invalid:4801/?board=account", true),
        ("http://127.0.0.1:4801/?x=1&board=account", true),
        ("http://srv-a.tailnet.invalid:4802/", true),
        ("http://srv-a:4802/p?board=account", true),
        ("http://srv-a.tailnet.invalid:4809/", false),
    ];
    for (page, want) in table {
        assert_eq!(foreign(&own, page, ports), want, "foreign {page}");
    }
    assert!(!foreign(&own, "http://srv-a.tailnet.invalid:4802/", &[]));
    assert!(foreign(&own, "http://srv-a.tailnet.invalid:4801/?board=account", &[]));
    assert!(shows(&own, "http://srv-a.tailnet.invalid:4801/x"));
    assert!(!shows(&own, "http://srv-a.tailnet.invalid:4801/?board=account"));
    assert!(!shows(&own, "http://srv-a.tailnet.invalid:4802/"));
    assert!(account_page("http://x/?a=1&board=account#f"));
    assert!(!account_page("http://x/#f?board=account"));
    assert!(!account_page("http://x/?board=account2"));
    page_url_table();
}

/// 頁の一覧の字から websocket の path の頁の URL を引く表。
fn page_url_table() {
    let list = format!(
        "[ {}, {} ]",
        r#"{"type":"page","url":"http://a/","webSocketDebuggerUrl":"ws://localhost/devtools/page/P1"}"#,
        r#"{"type":"page","url":"http://b/","webSocketDebuggerUrl":"ws://localhost/devtools/page/P2"}"#
    );
    let page_url: fn(&str, &str) -> Option<String> = launch::page_url;
    assert_eq!(page_url(&list, "/devtools/page/P2").as_deref(), Some("http://b/"));
    assert_eq!(page_url(&list, "/devtools/page/P1").as_deref(), Some("http://a/"));
    assert_eq!(page_url(&list, "/devtools/page/P3"), None);
    assert_eq!(page_url("{}", "/devtools/page/P1"), None);
}

#[test]
fn stcli_open_raises_once() {
    let field = Field::new("raise", Ssh::Sleep);
    let far = Far::start(&field, LIST);
    let (rc, out) = field.tz(&["open", "--to", "term-a"], "", false);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(out, [RAISED_A, MARKED_A, LINE]);
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
    for (name, before) in [("page", NO_PAGE), ("pageother", LIST_OTHER), ("pageforeign", LIST_FOREIGN)] {
        let field = Field::new(name, Ssh::Sleep);
        fs::write(&field.on, "").expect("印の file");
        let far = Far::start_after(&field, before, LIST);
        let (rc, out) = field.tz(&["open", "--to", "term-a"], "", false);
        assert_eq!(rc, 0, "{name}: {out:?}");
        assert_eq!(out, [RAISED_A, MARKED_A, LINE], "{name}");
        assert_eq!(
            far.stop(),
            ["GET /json/version", "GET /json/list", "GET /json/list"],
            "{name}"
        );
        let ssh = field.records("ssh");
        assert_eq!(ssh.len(), 2, "{name}: {ssh:?}");
        assert_eq!(
            ssh[1],
            launch::launch_argv(&term("term-a"), BOARD_URL).expect("launch_argv"),
            "{name}"
        );
        let file = memo::path(&field.user(), "term-a").expect("覚えの path");
        assert_eq!(
            fs::read_to_string(&file).expect("窓の覚え"),
            "repo\t/devtools/page/P1\n",
            "{name}"
        );
        assert_eq!(mode(&file), 0o600, "{name}");
    }
}

#[test]
fn stcli_other_board_untouched() {
    let field = Field::new("untouched", Ssh::Sleep);
    fs::write(&field.on, "").expect("印の file");
    let far = Far::start(&field, LIST_FOREIGN);
    let (rc, out) = field.tz(&["reload", "--to", "term-a"], "", false);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out.len(), 2, "{out:?}");
    for word in ["端末 term-a", BOARD_URL, "ほかの board の窓は使わない"] {
        assert!(out[0].contains(word), "{word}: {}", out[0]);
    }
    assert_eq!(out[1], LINE);
    assert_eq!(far.stop(), ["GET /json/version", "GET /json/list"]);
    assert_eq!(field.records("ssh").len(), 1);
    assert!(!field.launched.exists(), "起動の引数が撃たれた");
    let file = memo::path(&field.user(), "term-a").expect("覚えの path");
    assert!(!file.exists(), "窓の覚えが在る");
}

#[test]
fn stcli_memo_round_trip() {
    let a = "/devtools/page/A";
    let b = "/devtools/page/B";
    let entries = vec![("p".to_string(), a.to_string()), ("q".to_string(), b.to_string())];
    let text = memo::render(&entries);
    assert_eq!(text, format!("p\t{a}\nq\t{b}\n"));
    assert_eq!(memo::read(&text), entries);
    assert_eq!(memo::get(&entries, "q"), Some(b));
    assert_eq!(memo::get(&entries, "r"), None);
    let messy = format!(
        "p\t{a}\nno tab {a}\n\t{a}\nx\t/devtools/browser/B1\nx\ty\tz\nx\t\nq\t{b}\np\t{b}\n\n"
    );
    assert_eq!(
        memo::read(&messy),
        [("q".to_string(), b.to_string()), ("p".to_string(), b.to_string())]
    );
    assert_eq!(memo::read(""), []);

    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("stcli").join("memo");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("歯の dir");
    assert_eq!(memo::path(&root, "term-a"), Some(root.join("tzst-term-a.win")));
    for bad in ["", ".x", "a/b", "a b"] {
        assert_eq!(memo::path(&root, bad), None, "{bad:?}");
    }
    memo_puts(root, a, b);
}

/// 覚えの file へ書く・読む・権限を 0600 に保つ。
fn memo_puts(root: PathBuf, a: &str, b: &str) {
    let file = root.join("tzst-term-a.win");
    assert_eq!(memo::recall(&file, "p"), None);
    assert_eq!(memo::put(&file, "p", a), Ok(true));
    assert_eq!(mode(&file), 0o600);
    assert_eq!(fs::read_to_string(&file).expect("覚えの字"), format!("p\t{a}\n"));
    let before = fs::metadata(&file).expect("覚えの file").modified().expect("時刻");
    assert_eq!(memo::put(&file, "p", a), Ok(false));
    assert_eq!(fs::metadata(&file).expect("覚えの file").modified().expect("時刻"), before);
    assert_eq!(memo::put(&file, "q", b), Ok(true));
    assert_eq!(memo::put(&file, "p", b), Ok(true));
    assert_eq!(
        fs::read_to_string(&file).expect("覚えの字"),
        format!("q\t{b}\np\t{b}\n")
    );
    assert_eq!(memo::recall(&file, "p").as_deref(), Some(b));
    assert_eq!(memo::recall(&file, "r"), None);
    fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).expect("権限を緩める");
    assert_eq!(memo::put(&file, "r", a), Ok(true));
    assert_eq!(mode(&file), 0o600);
    memo_bad_puts(root, file, a);
}

/// 形の悪い project と頁の覚えは断り、file も dir も替えない。
fn memo_bad_puts(root: PathBuf, file: PathBuf, a: &str) {
    let text = fs::read_to_string(&file).expect("覚えの字");
    for (project, page) in [
        ("", a),
        ("a\tb", a),
        ("a\nb", a),
        ("p", "/devtools/browser/B1"),
        ("p", "/devtools/page/"),
        ("p", "/devtools/page/a b"),
        ("p", "x"),
        ("p", "/devtools/page/A\tB"),
    ] {
        assert!(memo::put(&file, project, page).is_err(), "{project:?} {page:?}");
    }
    assert_eq!(fs::read_to_string(&file).expect("覚えの字"), text);
    let names: Vec<String> = fs::read_dir(&root)
        .expect("歯の dir")
        .map(|e| e.expect("要素").file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["tzst-term-a.win"]);
}

#[test]
fn stcli_open_marks_shown() {
    let field = Field::new("marks", Ssh::Sleep);
    let far = Far::start(&field, LIST);
    let before = epoch();
    let (rc, out) = field.tz(&["open", "--to", "term-a"], "", false);
    let after = epoch();
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(out, [RAISED_A, MARKED_A, LINE]);
    let shown = field.shown();
    assert_eq!(shown.len(), 1, "{shown:?}");
    assert_eq!(shown[0].0, "term-a");
    assert!(
        (before..=after).contains(&shown[0].1),
        "{before} {after} {shown:?}"
    );
    assert_eq!(mode(&field.config), 0o600);

    let (rc, out) = field.tz(&["open", "--to", "term-a"], "", false);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(out, [LIVE_A, LINE]);
    assert_eq!(field.shown(), shown);
    far.stop();
    reload_without_window(field, shown);
}

/// 窓の無い端末への reload は断り、印を替えない。
fn reload_without_window(field: Field, shown: Vec<(String, u64)>) {
    let mut targets = target::load(&field.config).expect("場の表示先の設定");
    targets.default = Some("term-a".to_string());
    target::save(&field.config, &targets).expect("既定を置く");
    fs::remove_file(&field.on).expect("窓を閉じる");
    fs::remove_file(&field.socket).expect("場の socket の file を消す");
    let far = Far::start(&field, LIST);
    let ssh = field.records("ssh").len();
    let (rc, out) = field.tz(&["reload"], "", false);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out.len(), 2, "{out:?}");
    for word in ["端末 term-a に表示面の Chrome の窓が無い", "tz stage open"] {
        assert!(out[0].contains(word), "{word}: {}", out[0]);
    }
    assert_eq!(out[1], LINE);
    assert_eq!(far.stop(), ["GET /json/version"]);
    assert_eq!(field.records("ssh").len(), ssh + 1);
    assert!(!field.on.exists(), "窓を起こす回の印の file が在る");
    assert_eq!(field.shown(), shown);
}

#[test]
fn stcli_open_keeps_old_mark() {
    let field = Field::new("oldmark", Ssh::Sleep);
    let mut targets = Targets::default();
    targets.shown.insert("term-a".to_string(), 7);
    target::save(&field.config, &targets).expect("印を置く");
    let far = Far::start(&field, LIST);
    let (rc, out) = field.tz(&["open", "--to", "term-a"], "", false);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(out, [RAISED_A, LINE]);
    far.stop();
    assert_eq!(field.shown(), [("term-a".to_string(), 7)]);
}

#[test]
fn stcli_open_bad_config_first() {
    let field = Field::new("badcfg", Ssh::Sleep);
    let text = format!("{}\n[screen]\n", target::HEAD);
    fs::create_dir_all(field.config.parent().expect("設定の dir")).expect("設定の dir を作る");
    fs::write(&field.config, &text).expect("形の外れた設定");
    let (rc, out) = field.tz(&["open", "--to", "term-a"], "", false);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out.len(), 2, "{out:?}");
    let path = Field::text(&field.config);
    for word in ["表示先の設定", path.as_str(), "知らない表 [screen]"] {
        assert!(out[0].contains(word), "{word}: {}", out[0]);
    }
    assert_eq!(out[1], LINE);
    assert!(field.records("ssh").is_empty(), "偽の ssh が撃たれた");
    assert_eq!(fs::read_to_string(&field.config).expect("設定の字"), text);
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
    assert_eq!(out, [LIVE_A, LINE]);
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
    for word in ["createTarget", "NEW_PAGE", "fn browser_resource", "fn new_page"] {
        assert!(!cli_text.contains(word), "cli.rs が {word} を含む");
    }
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
    assert!(holders.is_empty(), "{holders:?}");
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
    assert_eq!(src("mod.rs").matches("pub mod memo;").count(), 1);
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
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth4/stcli.rs");
    let text = fs::read_to_string(&path).expect("tests/teeth4/stcli.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 26, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("stcli_")
            .unwrap_or_else(|| panic!("{name} は stcli_ で始まらない"));
        for word in &words {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
