//! CDP の口と命令の列（行 i-3・要件 FR16・判断の記録 ADR-5 の決定 (1)）。
//! 命令の語彙 `Command` を CDP の method の列 `Step` に写す純粋な `steps` と、列を撃って応答を待つ `Session` を持つ。
//! 任意の式を撃つ口を持たない（見積りの T6・条 P-12.1）: Command は式を運ぶ欄を持たず、Runtime.evaluate は
//! Dom の決まった式と受入の測りの決まった式だけで、Session は method と params を直に受ける口を持たない
//! （撃てるのは steps の列と measure だけ）。
//! 窓の置き場は持ち主に任せ（持ち主の裁定 t3-hub.59.5・判断の記録 ADR-15 の決定 (6)）、語彙は同じ窓の中の操作だけを持つ。
//! session は tz の 1 回の撃ちごとに繋いで閉じ、常駐しない（繋ぐ先は行 i-2 が張る 0700 の dir の中の unix socket）。
//! 行 i-4 で 2 つ目の運び手（席の目の headless の Chrome の pipe に付く `attach`・ws の上の振る舞いは変えない）を足す。
//! 行 i-4 で頁の今の URL の読み `url` と、それが撃つ script でない決まった 1 つの method `HISTORY` を足す。
//! 行 j-runner で受入 12 条の測り `measure`（決まった式 `MEASURE_EXPRESSION` を 1 度撃つ・Command に枝を足さない）を足す。

use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use super::json;
use super::pipe::Pipe;
use super::ws::Socket;

/// DOM の命令で評価する唯一の式。
pub const DOM_EXPRESSION: &str = "document.documentElement.outerHTML";

/// 受入 12 条の測りの式（measure だけが撃つ・頁を書き換えない 1 つの式・事実の JSON の字を返す）。
pub const MEASURE_EXPRESSION: &str = include_str!("measure.js");

/// 頁の読み込みの終わりの event。
pub const LOAD_EVENT: &str = "Page.loadEventFired";

/// 頁の今の URL を読む method（script を撃たない）。
pub const HISTORY: &str = "Page.getNavigationHistory";

/// 席の命令の語彙（要件 FR16 の navigate・viewport・reload・click・入力・key・scroll・待ち・screenshot・DOM・console）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Navigate {
        url: String,
    },
    Viewport {
        width: u32,
        height: u32,
        scale: u32,
        mobile: bool,
    },
    Reload,
    Click {
        x: u32,
        y: u32,
    },
    Type {
        text: String,
    },
    Key {
        key: String,
    },
    Scroll {
        x: u32,
        y: u32,
        dy: i32,
    },
    Wait {
        ms: u64,
    },
    Screenshot,
    Dom,
    Console,
}

/// 命令の 1 歩（Call は method と params の JSON の object の字・Await は待つ event の名・Pause は眠る間）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Call {
        method: &'static str,
        params: String,
    },
    Await(&'static str),
    Pause(Duration),
}

/// params の値（空白を挟まない JSON の object の字に組む）。
#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a str),
    Int(i64),
    Bool(bool),
}

fn object(pairs: &[(&str, Value)]) -> String {
    let body: Vec<String> = pairs
        .iter()
        .map(|(key, value)| {
            let value = match value {
                Value::Text(text) => json::escape(text),
                Value::Int(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
            };
            format!("{}:{value}", json::escape(key))
        })
        .collect();
    format!("{{{}}}", body.join(","))
}

fn call(method: &'static str, pairs: &[(&str, Value)]) -> Step {
    Step::Call {
        method,
        params: object(pairs),
    }
}

/// 命令を CDP の歩の列に写す。
pub fn steps(command: &Command) -> Vec<Step> {
    use Value::{Bool, Int, Text};
    match command {
        Command::Navigate { url } => vec![
            call("Page.enable", &[]),
            call("Page.navigate", &[("url", Text(url))]),
            Step::Await(LOAD_EVENT),
        ],
        Command::Viewport {
            width,
            height,
            scale,
            mobile,
        } => viewport(*width, *height, *scale, *mobile),
        Command::Reload => vec![
            call("Page.enable", &[]),
            call("Page.reload", &[]),
            Step::Await(LOAD_EVENT),
        ],
        Command::Click { x, y } => click(*x, *y),
        Command::Type { text } => vec![call("Input.insertText", &[("text", Text(text))])],
        Command::Key { key } => ["keyDown", "keyUp"]
            .into_iter()
            .map(|kind| {
                call(
                    "Input.dispatchKeyEvent",
                    &[("type", Text(kind)), ("key", Text(key))],
                )
            })
            .collect(),
        Command::Scroll { x, y, dy } => vec![call(
            "Input.dispatchMouseEvent",
            &[
                ("type", Text("mouseWheel")),
                ("x", Int(i64::from(*x))),
                ("y", Int(i64::from(*y))),
                ("deltaX", Int(0)),
                ("deltaY", Int(i64::from(*dy))),
            ],
        )],
        Command::Wait { ms } => vec![Step::Pause(Duration::from_millis(*ms))],
        Command::Screenshot => vec![call("Page.captureScreenshot", &[("format", Text("png"))])],
        Command::Dom => vec![call(
            "Runtime.evaluate",
            &[
                ("expression", Text(DOM_EXPRESSION)),
                ("returnByValue", Bool(true)),
            ],
        )],
        Command::Console => vec![call("Runtime.enable", &[]), call("Log.enable", &[])],
    }
}

/// 窓の大きさの命令の歩（大きさと倍率と mobile の印を写し、touch の真似を mobile の印に合わせる）。
fn viewport(width: u32, height: u32, scale: u32, mobile: bool) -> Vec<Step> {
    use Value::{Bool, Int};
    vec![
        call(
            "Emulation.setDeviceMetricsOverride",
            &[
                ("width", Int(i64::from(width))),
                ("height", Int(i64::from(height))),
                ("deviceScaleFactor", Int(i64::from(scale))),
                ("mobile", Bool(mobile)),
            ],
        ),
        call("Emulation.setTouchEmulationEnabled", &[("enabled", Bool(mobile))]),
    ]
}

/// click の命令の歩（点へ動かし、左の button を押して離す）。
fn click(x: u32, y: u32) -> Vec<Step> {
    use Value::{Int, Text};
    let (x, y) = (Int(i64::from(x)), Int(i64::from(y)));
    let press = |kind| {
        call(
            "Input.dispatchMouseEvent",
            &[
                ("type", Text(kind)),
                ("x", x),
                ("y", y),
                ("button", Text("left")),
                ("clickCount", Int(1)),
            ],
        )
    };
    vec![
        call(
            "Input.dispatchMouseEvent",
            &[("type", Text("mouseMoved")), ("x", x), ("y", y)],
        ),
        press("mousePressed"),
        press("mouseReleased"),
    ]
}

/// CDP の 1 つの message の字（鍵 id・method・params の順・空白を挟まない）。
pub fn message(id: u64, method: &str, params: &str) -> String {
    format!(
        "{{\"id\":{id},\"method\":{},\"params\":{params}}}",
        json::escape(method)
    )
}

/// session の運び手（行 i-3 の websocket か、行 i-4 の席の目の Chrome の pipe）。
enum Link {
    Ws(Socket),
    Pipe(Pipe),
}

impl Link {
    fn send(&mut self, text: &str) -> Result<(), String> {
        match self {
            Link::Ws(socket) => socket.send(text),
            Link::Pipe(pipe) => pipe.send(text),
        }
    }

    /// 残りの時間 left を上限にして次の字を読む（相手が閉じれば None）。
    fn recv(&mut self, left: Duration) -> Result<Option<String>, String> {
        match self {
            Link::Ws(socket) => {
                socket.wait(left)?;
                socket.recv()
            }
            Link::Pipe(pipe) => pipe.recv(left),
        }
    }

    fn close(self) -> Result<(), String> {
        match self {
            Link::Ws(socket) => socket.close(),
            Link::Pipe(pipe) => pipe.close(),
        }
    }
}

/// 頁の target への 1 回の接続（id は 1 から増え、応答でない字は event として受けた順に貯める）。
pub struct Session {
    link: Link,
    next: u64,
    events: Vec<String>,
    timeout: Duration,
}

impl Session {
    /// 頁の target の websocket に繋ぐ。
    pub fn open(path: &Path, resource: &str, timeout: Duration) -> Result<Session, String> {
        Ok(Session {
            link: Link::Ws(Socket::connect(path, resource, timeout)?),
            next: 1,
            events: Vec::new(),
            timeout,
        })
    }

    /// 頁の session に付いた pipe の上の session（付くのは relay の Eyes の open）。
    pub fn attach(pipe: Pipe, timeout: Duration) -> Session {
        Session {
            link: Link::Pipe(pipe),
            next: 1,
            events: Vec::new(),
            timeout,
        }
    }

    /// 頁の今の URL（HISTORY の応答の currentIndex の番の entries の url）。
    pub fn url(&mut self) -> Result<String, String> {
        let reply = self.call(HISTORY, "{}")?;
        let result = json::member(&reply, "result");
        let index = result
            .and_then(|r| json::member(r, "currentIndex"))
            .and_then(|n| n.parse::<usize>().ok());
        let entries = result
            .and_then(|r| json::member(r, "entries"))
            .and_then(json::items);
        index
            .zip(entries)
            .and_then(|(i, entries)| entries.get(i).copied())
            .and_then(|entry| json::member(entry, "url"))
            .and_then(json::unquote)
            .ok_or_else(|| format!("{HISTORY}: 応答の currentIndex の番の entries の url が読めない"))
    }

    /// 命令の歩を順に行い、Call の歩の応答の字を順に返す（どの歩の Err でもその後の歩は撃たない）。
    pub fn run(&mut self, command: &Command) -> Result<Vec<String>, String> {
        let from = self.events.len();
        let mut replies = Vec::new();
        for step in steps(command) {
            match step {
                Step::Call { method, params } => replies.push(self.call(method, &params)?),
                Step::Await(name) => self.await_event(name, from)?,
                Step::Pause(span) => thread::sleep(span),
            }
        }
        Ok(replies)
    }

    /// 測りの式を 1 度撃ち、返った字の値をそのまま返す（字の値が無ければ Err）。
    /// 式は DevTools の console の命令 getEventListeners を使うので includeCommandLineAPI を真にする。
    pub fn measure(&mut self) -> Result<String, String> {
        const METHOD: &str = "Runtime.evaluate";
        let params = object(&[
            ("expression", Value::Text(MEASURE_EXPRESSION)),
            ("returnByValue", Value::Bool(true)),
            ("includeCommandLineAPI", Value::Bool(true)),
        ]);
        let reply = self.call(METHOD, &params)?;
        json::member(&reply, "result")
            .and_then(|r| json::member(r, "result"))
            .and_then(|r| json::member(r, "value"))
            .and_then(json::unquote)
            .ok_or_else(|| format!("{METHOD}: 測りの式の応答に字の値が無い"))
    }

    /// session で貯めた event の字（受けた順）。
    pub fn events(&self) -> &[String] {
        &self.events
    }

    /// websocket か pipe を閉じる。
    pub fn close(self) -> Result<(), String> {
        self.link.close()
    }

    fn call(&mut self, method: &'static str, params: &str) -> Result<String, String> {
        let id = self.next;
        self.next += 1;
        self.link
            .send(&message(id, method, params))
            .map_err(|e| format!("{method}: {e}"))?;
        let want = id.to_string();
        let deadline = Instant::now() + self.timeout;
        loop {
            let text = self.read(deadline).map_err(|e| format!("{method}: {e}"))?;
            let Some(got) = json::member(&text, "id") else {
                self.events.push(text);
                continue;
            };
            if got != want {
                return Err(format!("{method}: 別の id {got} の応答"));
            }
            if let Some(error) = json::member(&text, "error") {
                let said = json::member(error, "message")
                    .and_then(json::unquote)
                    .unwrap_or_else(|| error.to_string());
                return Err(format!("{method}: {said}"));
            }
            return Ok(text);
        }
    }

    /// run の始めから貯めた event に名 name の event が在るまで読み進める（待ちは始めから timeout まで）。
    fn await_event(&mut self, name: &'static str, from: usize) -> Result<(), String> {
        let named = |text: &str| json::member(text, "method").and_then(json::unquote).as_deref() == Some(name);
        if self.events.iter().skip(from).any(|e| named(e)) {
            return Ok(());
        }
        let deadline = Instant::now() + self.timeout;
        loop {
            let text = self.read(deadline).map_err(|e| format!("{name} を待つ: {e}"))?;
            let hit = named(&text);
            self.events.push(text);
            if hit {
                return Ok(());
            }
        }
    }

    /// deadline までの残りを上限にして次の字を読む。
    fn read(&mut self, deadline: Instant) -> Result<String, String> {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err("時間切れ".to_string());
        }
        self.link
            .recv(left)?
            .ok_or_else(|| "相手が閉じた".to_string())
    }
}
