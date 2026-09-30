//! CDP の口の歯（行 i-3・接頭辞 stage_cdp_）。
//! 偽の server と偽の Chrome は歯の中の thread で、temp_dir の下の短い名の unix socket で待ち受け、歯の終わりに消す。
//! 偽の側は自分の小さな frame の読み書き（mask を解く読みと mask の無い書き）を持ち、握手の応答の鍵だけ crate の accept で組む。
#![cfg(test)]

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use tsuzuri_boundary::stage::cdp::{
    self, Command, DOM_EXPRESSION, LOAD_EVENT, MEASURE_EXPRESSION, Session, Step,
};
use tsuzuri_boundary::stage::json;
use tsuzuri_boundary::stage::ws::{self, GUID, MAX_MESSAGE, Socket};

/// 着地済みの行と波の行の verify の filter の語（この行の接頭辞 stage_cdp_ は並べない）。
const FILTER_WORDS: [&str; 111] = [
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "acctframe_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "askcard_",
    "batchpanel_",
    "board_min_",
    "contract_form_",
    "frame_",
    "gapspage_",
    "gquestion_",
    "graph_",
    "gview_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hook_",
    "hsblock_",
    "hsderive_",
    "hspage_",
    "ledgerblock_",
    "mapgraph_",
    "mapview_",
    "nextstep_",
    "nodepage_",
    "parts_",
    "pipe_",
    "project_",
    "question_",
    "seatblock_",
    "seatcard_",
    "server_",
    "skeleton_",
    "stage_",
    "stats_",
    "steady_",
    "topbar_",
    "tz_",
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
    "ilink_",
    "afocus_",
    "nxact_",
    "urpanel_",
    "saxis_",
    "ticker_",
    "hfig_",
    "pmore_",
    "lspark_",
    "apop_",
    "brand_",
    "runsdoc_",
    "nbatch_",
    "kcli_",
    "hcard_",
    "qgate_",
    "nsum_",
    "ntime_",
    "ptitle_",
    "ncard_",
    "hsym_",
    "smore_",
    "lcard_",
    "aaround_",
    "hruling_",
    "sxaxis_",
    "plimit_",
    "bport_",
    "fmark_",
    "fstop_",
    "fserve_",
    "nsumw_",
    "cadopt_",
    "tipx_",
    "hcsess_",
    "hcproj_",
    "sesplit_",
    "mstore_",
    "athr_",
    "qblock_",
    "wsteady_",
    "gsum_",
    "nstall_",
    "pfold_",
    "uword_",
    "cround_",
    "cgraph_",
    "csled_",
    "lhome_",
    "shb_",
    "ghb_",
    "nact_",
    "aord_",
    "mtree_",
    "cgdom_",
    "rhold_",
    "qkey_",
    "stage_term_",
];

/// 台本の Navigate の url。
const URL: &str = "http://127.0.0.1:1/?a=1&b=2";

/// 台本の Type の text（日本の 2 字・引用符・逆斜線・改行・水平タブ・U+0001）。
const TYPE_TEXT: &str = "日本\"\\\n\t\u{1}";

/// 台本の Type の text を escape した字。
const TYPE_JSON: &str = r#""日本\"\\\n\t\u0001""#;

/// 節の DOM の字。
const DOM: &str = "<html></html>";

/// 写真の data の字。
const PNG: &str = "iVBORw0KGgo=";

/// 節の DOM の params の字。
const DOM_PARAMS: &str = r#"{"expression":"document.documentElement.outerHTML","returnByValue":true}"#;

/// 節の 12 の method。
const METHODS: [&str; 12] = [
    "Page.enable",
    "Page.navigate",
    "Page.reload",
    "Emulation.setDeviceMetricsOverride",
    "Emulation.setTouchEmulationEnabled",
    "Input.dispatchMouseEvent",
    "Input.insertText",
    "Input.dispatchKeyEvent",
    "Page.captureScreenshot",
    "Runtime.evaluate",
    "Runtime.enable",
    "Log.enable",
];

/// 節の 17 の message（台本の順・id は 1 から）。
const MESSAGES: [&str; 17] = [
    r#"{"id":1,"method":"Page.enable","params":{}}"#,
    r#"{"id":2,"method":"Page.navigate","params":{"url":"http://127.0.0.1:1/?a=1&b=2"}}"#,
    r#"{"id":3,"method":"Emulation.setDeviceMetricsOverride","params":{"width":390,"height":844,"deviceScaleFactor":3,"mobile":true}}"#,
    r#"{"id":4,"method":"Emulation.setTouchEmulationEnabled","params":{"enabled":true}}"#,
    r#"{"id":5,"method":"Page.enable","params":{}}"#,
    r#"{"id":6,"method":"Page.reload","params":{}}"#,
    r#"{"id":7,"method":"Input.dispatchMouseEvent","params":{"type":"mouseWheel","x":195,"y":422,"deltaX":0,"deltaY":-600}}"#,
    r#"{"id":8,"method":"Input.dispatchMouseEvent","params":{"type":"mouseMoved","x":10,"y":20}}"#,
    r#"{"id":9,"method":"Input.dispatchMouseEvent","params":{"type":"mousePressed","x":10,"y":20,"button":"left","clickCount":1}}"#,
    r#"{"id":10,"method":"Input.dispatchMouseEvent","params":{"type":"mouseReleased","x":10,"y":20,"button":"left","clickCount":1}}"#,
    r#"{"id":11,"method":"Input.insertText","params":{"text":"日本\"\\\n\t\u0001"}}"#,
    r#"{"id":12,"method":"Input.dispatchKeyEvent","params":{"type":"keyDown","key":"Enter"}}"#,
    r#"{"id":13,"method":"Input.dispatchKeyEvent","params":{"type":"keyUp","key":"Enter"}}"#,
    r#"{"id":14,"method":"Page.captureScreenshot","params":{"format":"png"}}"#,
    r#"{"id":15,"method":"Runtime.evaluate","params":{"expression":"document.documentElement.outerHTML","returnByValue":true}}"#,
    r#"{"id":16,"method":"Runtime.enable","params":{}}"#,
    r#"{"id":17,"method":"Log.enable","params":{}}"#,
];

/// 読み込みの終わりの event の字。
const LOAD: &str = r#"{"method":"Page.loadEventFired","params":{"timestamp":1}}"#;

/// console の event の字。
const CONSOLE: &str = r#"{"method":"Runtime.consoleAPICalled","params":{"type":"log","args":[]}}"#;

/// 台本の命令ごとの応答の数。
const COUNTS: [usize; 11] = [2, 2, 2, 1, 3, 1, 2, 0, 1, 1, 2];

/// Command の変種を台本の値に写す（wildcard の腕を持たない・変種が足されると組めない）。
fn scripted(command: &Command) -> Command {
    match command {
        Command::Navigate { .. } => Command::Navigate {
            url: URL.to_string(),
        },
        Command::Viewport { .. } => Command::Viewport {
            width: 390,
            height: 844,
            scale: 3,
            mobile: true,
        },
        Command::Reload => Command::Reload,
        Command::Scroll { .. } => Command::Scroll {
            x: 195,
            y: 422,
            dy: -600,
        },
        Command::Click { .. } => Command::Click { x: 10, y: 20 },
        Command::Type { .. } => Command::Type {
            text: TYPE_TEXT.to_string(),
        },
        Command::Key { .. } => Command::Key {
            key: "Enter".to_string(),
        },
        Command::Wait { .. } => Command::Wait { ms: 10 },
        Command::Screenshot => Command::Screenshot,
        Command::Dom => Command::Dom,
        Command::Console => Command::Console,
    }
}

/// 節の台本の 11 の命令（Command の 11 の変種の全部を台本の順に）。
fn script() -> Vec<Command> {
    [
        Command::Navigate { url: String::new() },
        Command::Viewport {
            width: 0,
            height: 0,
            scale: 0,
            mobile: false,
        },
        Command::Reload,
        Command::Scroll { x: 0, y: 0, dy: 0 },
        Command::Click { x: 0, y: 0 },
        Command::Type {
            text: String::new(),
        },
        Command::Key { key: String::new() },
        Command::Wait { ms: 0 },
        Command::Screenshot,
        Command::Dom,
        Command::Console,
    ]
    .iter()
    .map(scripted)
    .collect()
}

fn named(script: &[Command], pick: fn(&Command) -> bool) -> Command {
    script.iter().find(|c| pick(c)).cloned().expect("台本の命令")
}

/// 歯の unix socket の置き場（process の id と歯の番号を持つ短い名・終わりに消す）。
struct Spot(PathBuf);

impl Spot {
    fn new(n: u32) -> Spot {
        let path = env::temp_dir().join(format!("tzcdp-{}-{n}", process::id()));
        let _ = fs::remove_file(&path);
        Spot(path)
    }

    fn listen(&self) -> UnixListener {
        UnixListener::bind(&self.0).unwrap_or_else(|e| panic!("{}: {e}", self.0.display()))
    }
}

impl Drop for Spot {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// 偽の側が読んだ frame（payload は mask を解いた字）。
#[derive(Debug)]
struct Frame {
    fin: bool,
    opcode: u8,
    masked: bool,
    mark: u8,
    mask: [u8; 4],
    payload: Vec<u8>,
}

/// 偽の server の 1 つの接続。
struct Peer(UnixStream);

impl Peer {
    /// 接続を受け、頭を CR と LF の 2 組の並びまで読む。
    fn open(listener: &UnixListener) -> (Peer, String) {
        let (stream, _) = listener.accept().expect("接続を受ける");
        stream
            .set_read_timeout(Some(Duration::from_secs(20)))
            .expect("読みの上限");
        let mut peer = Peer(stream);
        let mut head = Vec::new();
        while !head.ends_with(b"\r\n\r\n") {
            let mut b = [0u8; 1];
            peer.0.read_exact(&mut b).expect("頭を読む");
            head.push(b[0]);
        }
        (peer, String::from_utf8(head).expect("頭の字"))
    }

    /// 接続を受けて正しい 101 を返す。
    fn agreed(listener: &UnixListener) -> (Peer, String) {
        let (mut peer, head) = Peer::open(listener);
        let answer = ws::accept(&header(&head, "sec-websocket-key").expect("鍵"));
        peer.write(
            format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {answer}\r\n\r\n")
                .as_bytes(),
        );
        (peer, head)
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.write_all(bytes).expect("偽の側が書く");
    }

    fn text(&mut self, text: &str) {
        self.write(&frame(true, 1, text.as_bytes()));
    }

    /// 次の frame（接続の終わりか誤りで None）。
    fn next(&mut self) -> Option<Frame> {
        let mut head = [0u8; 2];
        self.0.read_exact(&mut head).ok()?;
        let mark = head[1] & 0x7f;
        let len = match mark {
            126 => {
                let mut b = [0u8; 2];
                self.0.read_exact(&mut b).ok()?;
                u64::from(u16::from_be_bytes(b))
            }
            127 => {
                let mut b = [0u8; 8];
                self.0.read_exact(&mut b).ok()?;
                u64::from_be_bytes(b)
            }
            n => u64::from(n),
        };
        let masked = head[1] & 0x80 != 0;
        let mut mask = [0u8; 4];
        if masked {
            self.0.read_exact(&mut mask).ok()?;
        }
        let mut payload = vec![0u8; usize::try_from(len).ok()?];
        self.0.read_exact(&mut payload).ok()?;
        for (i, b) in payload.iter_mut().enumerate() {
            *b ^= mask[i % 4];
        }
        Some(Frame {
            fin: head[0] & 0x80 != 0,
            opcode: head[0] & 0x0f,
            masked,
            mark,
            mask,
            payload,
        })
    }

    /// 接続の終わりまでの frame の全部。
    fn rest(&mut self) -> Vec<Frame> {
        std::iter::from_fn(|| self.next()).collect()
    }
}

/// mask の無い frame の byte（長さは最も短い形）。
fn frame(fin: bool, opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![if fin { 0x80 | opcode } else { opcode }];
    match payload.len() {
        n @ 0..=125 => out.push(n as u8),
        n @ 126..=65535 => {
            out.push(126);
            out.extend_from_slice(&(n as u16).to_be_bytes());
        }
        n => {
            out.push(127);
            out.extend_from_slice(&(n as u64).to_be_bytes());
        }
    }
    out.extend_from_slice(payload);
    out
}

/// 頭の欄の値（名は大文字と小文字を区別しない・前後の空白を除く）。
fn header(head: &str, name: &str) -> Option<String> {
    head.split("\r\n")
        .skip(1)
        .filter_map(|l| l.split_once(':'))
        .find(|(n, _)| n.trim().eq_ignore_ascii_case(name))
        .map(|(_, v)| v.trim().to_string())
}

/// 字 = の詰めを持つ base64 を byte に戻す。
fn unbase64(text: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    if !text.len().is_multiple_of(4) {
        return None;
    }
    let body = text.trim_end_matches('=');
    if text.len() - body.len() > 2 {
        return None;
    }
    let mut bits = 0u32;
    let mut count = 0;
    let mut out = Vec::new();
    for c in body.bytes() {
        bits = (bits << 6) | ALPHABET.iter().position(|a| *a == c)? as u32;
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push((bits >> count) as u8);
        }
    }
    Some(out)
}

fn connect(spot: &Spot) -> Result<Socket, String> {
    Socket::connect(&spot.0, "/devtools/page/P1", Duration::from_secs(5))
}

/// 偽の Chrome の振る舞い。
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Plain,
    Refuse,
    Mute,
    NoLoad,
    NoValue,
}

fn ok(id: &str) -> String {
    format!(r#"{{"id":{id},"result":{{}}}}"#)
}

/// 偽の Chrome（受けた字の message を payload のまま記録して返す）。
fn chrome(listener: UnixListener, mode: Mode) -> JoinHandle<Vec<String>> {
    thread::spawn(move || {
        let (mut peer, _) = Peer::agreed(&listener);
        let mut got = Vec::new();
        while let Some(f) = peer.next() {
            if f.opcode == 8 {
                peer.write(&frame(true, 8, &f.payload));
                break;
            }
            let text = String::from_utf8(f.payload).expect("字の message");
            got.push(text.clone());
            let id = json::member(&text, "id").expect("鍵 id").to_string();
            let method = json::member(&text, "method")
                .and_then(json::unquote)
                .expect("鍵 method");
            let kind = json::member(&text, "params")
                .and_then(|p| json::member(p, "type"))
                .and_then(json::unquote);
            let refused = mode == Mode::Refuse
                && (method == "Input.insertText"
                    || (method == "Input.dispatchMouseEvent"
                        && kind.as_deref() == Some("mousePressed")));
            if refused {
                peer.text(&format!(
                    r#"{{"id":{id},"error":{{"code":-32000,"message":"no focus"}}}}"#
                ));
                continue;
            }
            match method.as_str() {
                "Page.captureScreenshot" if mode == Mode::Mute => {}
                "Page.captureScreenshot" => {
                    let reply = format!(r#"{{"id":{id},"result":{{"data":"{PNG}"}}}}"#);
                    let (a, rest) = reply.split_at(reply.len() / 3);
                    let (b, c) = rest.split_at(rest.len() / 2);
                    peer.write(&frame(false, 1, a.as_bytes()));
                    peer.write(&frame(false, 0, b.as_bytes()));
                    peer.write(&frame(true, 0, c.as_bytes()));
                }
                "Runtime.evaluate" if mode == Mode::NoValue => peer.text(&format!(
                    r#"{{"id":{id},"result":{{"result":{{"type":"undefined"}}}}}}"#
                )),
                "Runtime.evaluate" => peer.text(&format!(
                    r#"{{"id":{id},"result":{{"result":{{"type":"string","value":"{DOM}"}}}}}}"#
                )),
                "Page.navigate" | "Page.reload" => {
                    peer.text(&ok(&id));
                    if mode != Mode::NoLoad {
                        peer.text(LOAD);
                    }
                }
                "Runtime.enable" => {
                    peer.text(CONSOLE);
                    peer.text(&ok(&id));
                }
                _ => peer.text(&ok(&id)),
            }
        }
        got
    })
}

fn src(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/stage")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn call_methods(command: &Command) -> Vec<&'static str> {
    cdp::steps(command)
        .into_iter()
        .filter_map(|s| match s {
            Step::Call { method, .. } => Some(method),
            _ => None,
        })
        .collect()
}

#[test]
fn stage_cdp_shape_and_consts() {
    assert_eq!(GUID, "258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    assert_eq!(MAX_MESSAGE, 67_108_864);
    assert_eq!(DOM_EXPRESSION, "document.documentElement.outerHTML");
    assert_eq!(LOAD_EVENT, "Page.loadEventFired");
    let accept: fn(&str) -> String = ws::accept;
    let connect: fn(&Path, &str, Duration) -> Result<Socket, String> = Socket::connect;
    let send: fn(&mut Socket, &str) -> Result<(), String> = Socket::send;
    let recv: fn(&mut Socket) -> Result<Option<String>, String> = Socket::recv;
    let close: fn(Socket) -> Result<(), String> = Socket::close;
    let escape: fn(&str) -> String = json::escape;
    let member: for<'a> fn(&'a str, &str) -> Option<&'a str> = json::member;
    let unquote: fn(&str) -> Option<String> = json::unquote;
    let steps: fn(&Command) -> Vec<Step> = cdp::steps;
    let message: fn(u64, &str, &str) -> String = cdp::message;
    let open: fn(&Path, &str, Duration) -> Result<Session, String> = Session::open;
    let run: fn(&mut Session, &Command) -> Result<Vec<String>, String> = Session::run;
    let events: fn(&Session) -> &[String] = Session::events;
    let end: fn(Session) -> Result<(), String> = Session::close;
    let _ = (accept, connect, send, recv, close, escape, member, unquote);
    let _ = (steps, message, open, run, events, end);
    let _ = [
        Step::Call {
            method: "Page.enable",
            params: "{}".to_string(),
        },
        Step::Await(LOAD_EVENT),
        Step::Pause(Duration::from_millis(1)),
    ];
    let script = script();
    assert_eq!(script.len(), 11);
    for (i, command) in script.iter().enumerate() {
        assert_eq!(&scripted(command), command, "台本の値");
        for other in &script[i + 1..] {
            assert_ne!(
                std::mem::discriminant(command),
                std::mem::discriminant(other),
                "変種が重なる"
            );
        }
    }
}

#[test]
fn stage_cdp_key_answer_vectors() {
    assert_eq!(
        ws::accept("dGhlIHNhbXBsZSBub25jZQ=="),
        "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
    );
    assert_eq!(
        ws::accept("x3JJHMbDL1EzLkh9GBhXDw=="),
        "HSmrc0sMlYUkAGmm5OPpG2HaGWk="
    );
}

#[test]
fn stage_cdp_handshake_request_form() {
    let spot = Spot::new(3);
    let listener = spot.listen();
    let server = thread::spawn(move || {
        (0..2)
            .map(|_| Peer::agreed(&listener).1)
            .collect::<Vec<String>>()
    });
    connect(&spot).expect("1 回目の connect");
    connect(&spot).expect("2 回目の connect");
    let heads = server.join().expect("偽の server");
    for head in &heads {
        assert_eq!(
            head.split("\r\n").next(),
            Some("GET /devtools/page/P1 HTTP/1.1"),
            "{head}"
        );
        assert_eq!(header(head, "host").as_deref(), Some("localhost"), "{head}");
        assert!(
            header(head, "upgrade").is_some_and(|v| v.eq_ignore_ascii_case("websocket")),
            "{head}"
        );
        assert!(
            header(head, "connection").is_some_and(|v| v.contains("Upgrade")),
            "{head}"
        );
        assert_eq!(
            header(head, "sec-websocket-version").as_deref(),
            Some("13"),
            "{head}"
        );
        let key = header(head, "sec-websocket-key").expect("鍵");
        assert_eq!(key.len(), 24, "{key}");
        assert_eq!(unbase64(&key).map(|b| b.len()), Some(16), "{key}");
        for absent in ["origin", "sec-websocket-extensions", "sec-websocket-protocol"] {
            assert_eq!(header(head, absent), None, "{absent}: {head}");
        }
    }
    assert_ne!(
        header(&heads[0], "sec-websocket-key"),
        header(&heads[1], "sec-websocket-key")
    );
}

#[test]
fn stage_cdp_handshake_refusals() {
    let spot = Spot::new(4);
    let listener = spot.listen();
    let server = thread::spawn(move || {
        for case in 0..4 {
            let (mut peer, head) = Peer::open(&listener);
            let key = header(&head, "sec-websocket-key").expect("鍵");
            match case {
                0 => peer.write(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n"),
                1 => peer.write(
                    format!(
                        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
                        ws::accept(&format!("{key}x"))
                    )
                    .as_bytes(),
                ),
                2 => peer.write(
                    b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\r\n",
                ),
                _ => {}
            }
        }
    });
    for case in 0..4 {
        assert!(connect(&spot).is_err(), "場 {case}");
    }
    server.join().expect("偽の server");
    let nobody = Spot::new(40);
    assert!(connect(&nobody).is_err(), "待ち受けの無い path");
}

#[test]
fn stage_cdp_client_masks_each_text() {
    let spot = Spot::new(5);
    let listener = spot.listen();
    let server = thread::spawn(move || Peer::agreed(&listener).0.rest());
    let texts = ["a".repeat(5), "b".repeat(300), "c".repeat(70000)];
    let mut socket = connect(&spot).expect("connect");
    for text in &texts {
        socket.send(text).expect("send");
    }
    for _ in 0..8 {
        socket.send("same").expect("send");
    }
    drop(socket);
    let frames = server.join().expect("偽の server");
    assert_eq!(frames.len(), 11, "frame の数");
    for ((f, text), mark) in frames.iter().zip(&texts).zip([5u8, 126, 127]) {
        assert!(f.fin && f.opcode == 1 && f.masked, "{}", text.len());
        assert_eq!(f.mark, mark, "{} の長さの形", text.len());
        assert_eq!(f.payload, text.as_bytes(), "{}", text.len());
    }
    let masks: Vec<[u8; 4]> = frames[3..].iter().map(|f| f.mask).collect();
    assert!(masks.iter().any(|m| *m != masks[0]), "{masks:?}");
}

#[test]
fn stage_cdp_joins_fragments_and_pongs() {
    let spot = Spot::new(6);
    let listener = spot.listen();
    let big = "z".repeat(70000);
    let sent = big.clone();
    let server = thread::spawn(move || {
        let (mut peer, _) = Peer::agreed(&listener);
        peer.write(&frame(false, 1, b"ab"));
        peer.write(&frame(true, 9, b"p1"));
        peer.write(&frame(false, 0, b"cd"));
        peer.write(&frame(true, 0, b"ef"));
        let pong = peer.next().expect("pong");
        peer.write(&frame(true, 10, b""));
        let whole = frame(true, 1, sent.as_bytes());
        assert_eq!(whole[1], 127, "印 127 の長さ");
        peer.write(&whole);
        (pong, peer.rest())
    });
    let mut socket = connect(&spot).expect("connect");
    assert_eq!(socket.recv(), Ok(Some("abcdef".to_string())));
    assert_eq!(socket.recv(), Ok(Some(big)));
    drop(socket);
    let (pong, rest) = server.join().expect("偽の server");
    assert!(pong.fin && pong.opcode == 10 && pong.masked, "{pong:?}");
    assert_eq!(pong.payload, b"p1");
    assert!(rest.is_empty(), "pong の後の frame: {rest:?}");
}

#[test]
fn stage_cdp_refuses_odd_input() {
    let mut huge = vec![0x81, 127];
    huge.extend_from_slice(&(1u64 << 62).to_be_bytes());
    let cases: Vec<Vec<u8>> = vec![
        vec![0x81, 0x81, 1, 2, 3, 4, b'x' ^ 1],
        vec![0x82, 1, 0],
        vec![0x80, 1, b'x'],
        vec![0xC1, 1, b'x'],
        vec![0x81, 2, 0xC3, 0x28],
        vec![0x01, 1, b'a', 0x81, 1, b'b'],
        huge,
    ];
    let n = cases.len();
    let spot = Spot::new(7);
    let listener = spot.listen();
    let server = thread::spawn(move || {
        for bytes in cases {
            let (mut peer, _) = Peer::agreed(&listener);
            peer.write(&bytes);
            peer.rest();
        }
    });
    for case in 1..=n {
        let mut socket = connect(&spot).expect("connect");
        let got = socket.recv();
        assert!(got.is_err(), "場 ({case}): {got:?}");
    }
    server.join().expect("偽の server");
}

#[test]
fn stage_cdp_close_both_ways() {
    let spot = Spot::new(8);
    let listener = spot.listen();
    let server = thread::spawn(move || {
        let (mut peer, _) = Peer::agreed(&listener);
        let first = peer.next().expect("close");
        peer.write(&frame(true, 8, &[3, 232]));
        peer.rest();
        let (mut peer, _) = Peer::agreed(&listener);
        peer.write(&frame(true, 8, &[3, 232]));
        (first, peer.rest())
    });
    let socket = connect(&spot).expect("connect");
    assert_eq!(socket.close(), Ok(()));
    let mut socket = connect(&spot).expect("connect");
    assert_eq!(socket.recv(), Ok(None));
    drop(socket);
    let (first, rest) = server.join().expect("偽の server");
    assert!(first.fin && first.opcode == 8 && first.masked, "{first:?}");
    assert_eq!(first.payload, [3, 232]);
    assert_eq!(rest.len(), 1, "{rest:?}");
    assert!(rest[0].fin && rest[0].opcode == 8 && rest[0].masked, "{rest:?}");
    assert_eq!(rest[0].payload, [3, 232]);
}

#[test]
fn stage_cdp_json_helpers() {
    assert_eq!(json::escape("abc"), "\"abc\"");
    assert_eq!(json::escape(TYPE_TEXT), TYPE_JSON);
    assert_eq!(json::escape("\u{1f}"), "\"\\u001f\"");

    const O: &str = r#"{"id":7,"result":{"id":9,"note":"say \"id\":3"},"method":"M","list":[1,{"id":4}]}"#;
    const RESULT: &str = r#"{"id":9,"note":"say \"id\":3"}"#;
    assert_eq!(json::member(O, "id"), Some("7"));
    assert_eq!(json::member(O, "result"), Some(RESULT));
    assert_eq!(json::member(RESULT, "id"), Some("9"));
    assert_eq!(json::member(O, "method"), Some("\"M\""));
    assert_eq!(json::member(O, "list"), Some(r#"[1,{"id":4}]"#));
    assert_eq!(json::member(O, "nope"), None);
    assert_eq!(json::member(r#"{"a":"\"id\":1"}"#, "id"), None);
    const SPACED: &str = r#"{ "id" : 5 , "x" : true }"#;
    assert_eq!(json::member(SPACED, "id"), Some("5"));
    assert_eq!(json::member(SPACED, "x"), Some("true"));
    assert_eq!(json::member("[1]", "id"), None);
    assert_eq!(json::member("7", "id"), None);

    assert_eq!(
        json::unquote(r#""a\u00e9\ud83d\ude00\"\\\/\n""#).as_deref(),
        Some("a\u{e9}\u{1F600}\"\\/\n")
    );
    assert_eq!(json::unquote("7"), None);
    assert_eq!(json::unquote(r#""\x""#), None);
    assert_eq!(json::unquote(r#""\ud83d""#), None);
}

#[test]
fn stage_cdp_steps_table() {
    fn call(method: &'static str, params: &str) -> Step {
        Step::Call {
            method,
            params: params.to_string(),
        }
    }
    let enable = call("Page.enable", "{}");
    let want: [Vec<Step>; 11] = [
        vec![
            enable.clone(),
            call("Page.navigate", r#"{"url":"http://127.0.0.1:1/?a=1&b=2"}"#),
            Step::Await(LOAD_EVENT),
        ],
        vec![
            call(
                "Emulation.setDeviceMetricsOverride",
                r#"{"width":390,"height":844,"deviceScaleFactor":3,"mobile":true}"#,
            ),
            call("Emulation.setTouchEmulationEnabled", r#"{"enabled":true}"#),
        ],
        vec![
            enable,
            call("Page.reload", "{}"),
            Step::Await(LOAD_EVENT),
        ],
        vec![call(
            "Input.dispatchMouseEvent",
            r#"{"type":"mouseWheel","x":195,"y":422,"deltaX":0,"deltaY":-600}"#,
        )],
        vec![
            call(
                "Input.dispatchMouseEvent",
                r#"{"type":"mouseMoved","x":10,"y":20}"#,
            ),
            call(
                "Input.dispatchMouseEvent",
                r#"{"type":"mousePressed","x":10,"y":20,"button":"left","clickCount":1}"#,
            ),
            call(
                "Input.dispatchMouseEvent",
                r#"{"type":"mouseReleased","x":10,"y":20,"button":"left","clickCount":1}"#,
            ),
        ],
        vec![call(
            "Input.insertText",
            &format!(r#"{{"text":{TYPE_JSON}}}"#),
        )],
        vec![
            call(
                "Input.dispatchKeyEvent",
                r#"{"type":"keyDown","key":"Enter"}"#,
            ),
            call("Input.dispatchKeyEvent", r#"{"type":"keyUp","key":"Enter"}"#),
        ],
        vec![Step::Pause(Duration::from_millis(10))],
        vec![call("Page.captureScreenshot", r#"{"format":"png"}"#)],
        vec![call("Runtime.evaluate", DOM_PARAMS)],
        vec![call("Runtime.enable", "{}"), call("Log.enable", "{}")],
    ];
    for (command, want) in script().iter().zip(want) {
        assert_eq!(cdp::steps(command), want, "{command:?}");
    }
    assert_eq!(
        cdp::message(3, "Page.enable", "{}"),
        r#"{"id":3,"method":"Page.enable","params":{}}"#
    );
    assert_eq!(
        cdp::message(12, "Page.navigate", r#"{"url":"u"}"#),
        r#"{"id":12,"method":"Page.navigate","params":{"url":"u"}}"#
    );
}

#[test]
fn stage_cdp_session_sequence() {
    let spot = Spot::new(11);
    let chrome = chrome(spot.listen(), Mode::Plain);
    let mut session =
        Session::open(&spot.0, "/devtools/page/P1", Duration::from_secs(5)).expect("open");
    let mut replies = Vec::new();
    for (command, count) in script().iter().zip(COUNTS) {
        let got = session.run(command).unwrap_or_else(|e| panic!("{command:?}: {e}"));
        assert_eq!(got.len(), count, "{command:?}");
        replies.push(got);
    }
    let shot = &replies[8][0];
    let data = json::member(shot, "result")
        .and_then(|r| json::member(r, "data"))
        .and_then(json::unquote);
    assert_eq!(data.as_deref(), Some(PNG), "{shot}");
    let dom = &replies[9][0];
    let value = json::member(dom, "result")
        .and_then(|r| json::member(r, "result"))
        .and_then(|r| json::member(r, "value"))
        .and_then(json::unquote);
    assert_eq!(value.as_deref(), Some(DOM), "{dom}");
    assert_eq!(session.events(), [LOAD, LOAD, CONSOLE]);
    assert_eq!(session.close(), Ok(()));
    assert_eq!(chrome.join().expect("偽の Chrome"), MESSAGES);
}

#[test]
fn stage_cdp_error_reply_stops() {
    let script = script();
    let spot = Spot::new(12);
    let chrome = chrome(spot.listen(), Mode::Refuse);
    let mut session =
        Session::open(&spot.0, "/devtools/page/P1", Duration::from_secs(5)).expect("open");
    let typed = named(&script, |c| matches!(c, Command::Type { .. }));
    let err = session.run(&typed).expect_err("Type は Err");
    assert!(err.contains("Input.insertText") && err.contains("no focus"), "{err}");
    let click = named(&script, |c| matches!(c, Command::Click { .. }));
    let err = session.run(&click).expect_err("Click は Err");
    assert!(err.contains("Input.dispatchMouseEvent"), "{err}");
    let key = named(&script, |c| matches!(c, Command::Key { .. }));
    assert_eq!(session.run(&key).map(|r| r.len()), Ok(2));
    assert_eq!(session.close(), Ok(()));
    let got = chrome.join().expect("偽の Chrome");
    let seen: Vec<(String, String, String)> = got
        .iter()
        .map(|m| {
            let id = json::member(m, "id").unwrap_or("").to_string();
            let method = json::member(m, "method")
                .and_then(json::unquote)
                .unwrap_or_default();
            let kind = json::member(m, "params")
                .and_then(|p| json::member(p, "type"))
                .and_then(json::unquote)
                .unwrap_or_default();
            (id, method, kind)
        })
        .collect();
    let want: Vec<(String, String, String)> = [
        ("1", "Input.insertText", ""),
        ("2", "Input.dispatchMouseEvent", "mouseMoved"),
        ("3", "Input.dispatchMouseEvent", "mousePressed"),
        ("4", "Input.dispatchKeyEvent", "keyDown"),
        ("5", "Input.dispatchKeyEvent", "keyUp"),
    ]
    .iter()
    .map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string()))
    .collect();
    assert_eq!(seen, want, "{got:?}");
}

#[test]
fn stage_cdp_waits_are_bounded() {
    let script = script();
    let cases = [
        (
            Mode::Mute,
            named(&script, |c| matches!(c, Command::Screenshot)),
            "Page.captureScreenshot",
        ),
        (
            Mode::NoLoad,
            named(&script, |c| matches!(c, Command::Navigate { .. })),
            "Page.loadEventFired",
        ),
    ];
    for (n, (mode, command, word)) in (130..).zip(cases) {
        let spot = Spot::new(n);
        let chrome = chrome(spot.listen(), mode);
        let mut session =
            Session::open(&spot.0, "/devtools/page/P1", Duration::from_secs(1)).expect("open");
        let start = Instant::now();
        let err = session.run(&command).expect_err("待ちは Err");
        assert!(start.elapsed() < Duration::from_secs(20), "{word}");
        assert!(err.contains(word), "{word}: {err}");
        drop(session);
        chrome.join().expect("偽の Chrome");
    }
}

#[test]
fn stage_cdp_no_script_port() {
    let script = script();
    let mut evaluates = Vec::new();
    for command in &script {
        for step in cdp::steps(command) {
            if let Step::Call { method, params } = step {
                assert!(METHODS.contains(&method), "{method}");
                if method == "Runtime.evaluate" {
                    evaluates.push((command.clone(), params));
                }
            }
        }
    }
    assert_eq!(evaluates, [(Command::Dom, DOM_PARAMS.to_string())]);
    let print = ["print", "!"].concat();
    let println = ["println", "!"].concat();
    for file in ["ws.rs", "json.rs", "cdp.rs", "measure.js"] {
        let text = src(file);
        for word in [
            "unsafe",
            &print,
            &println,
            "callFunctionOn",
            "compileScript",
            "runScript",
            "addScriptToEvaluateOnNewDocument",
            "Debugger.",
        ] {
            assert!(!text.contains(word), "{file} が {word} を含む");
        }
    }
    let text = src("cdp.rs");
    let mut names: Vec<&str> = text
        .match_indices("pub fn ")
        .map(|(at, word)| {
            let rest = &text[at + word.len()..];
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            &rest[..end]
        })
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "attach", "close", "events", "measure", "message", "open", "run", "steps", "url"
        ]
    );
}

#[test]
fn stage_cdp_measure_one_fixed_expression() {
    assert_eq!(MEASURE_EXPRESSION, src("measure.js"));
    assert!(
        MEASURE_EXPRESSION.starts_with("// 受入 12 条の測りの式"),
        "測りの式の頭"
    );
    let params = format!(
        r#"{{"expression":{},"returnByValue":true,"includeCommandLineAPI":true}}"#,
        json::escape(MEASURE_EXPRESSION)
    );
    for (n, mode) in [(14, Mode::Plain), (15, Mode::NoValue)] {
        let spot = Spot::new(n);
        let fake = chrome(spot.listen(), mode);
        let mut session =
            Session::open(&spot.0, "/devtools/page/P1", Duration::from_secs(5)).expect("open");
        let got = session.measure();
        if mode == Mode::Plain {
            assert_eq!(got.as_deref(), Ok(DOM));
        } else {
            let err = got.expect_err("字の値の無い応答は Err");
            assert!(err.contains("Runtime.evaluate"), "{err}");
        }
        assert!(session.events().is_empty(), "{:?}", session.events());
        assert_eq!(session.close(), Ok(()));
        assert_eq!(
            fake.join().expect("偽の Chrome"),
            [cdp::message(1, "Runtime.evaluate", &params)]
        );
    }
}

#[test]
fn stage_cdp_window_never_raised() {
    let words = ["bringToFront", "activateTarget", "createTarget", "setWindowBounds"];
    for command in script() {
        for method in call_methods(&command) {
            for word in words {
                assert!(!method.contains(word), "{command:?}: {method}");
            }
        }
    }
    for file in ["ws.rs", "json.rs", "cdp.rs"] {
        let text = src(file);
        for word in words {
            assert!(!text.contains(word), "{file} が {word} を含む");
        }
    }
}

#[test]
fn stage_cdp_own_names_clean() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/stcdp.rs");
    let text = fs::read_to_string(&path).expect("tests/stcdp.rs を読む");
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
            .strip_prefix("stage_cdp_")
            .unwrap_or_else(|| panic!("{name} は stage_cdp_ で始まらない"));
        for word in FILTER_WORDS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
