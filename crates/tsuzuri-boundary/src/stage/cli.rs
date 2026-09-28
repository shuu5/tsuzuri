//! tz stage の口（行 i-5・要件 FR16・判断の記録 ADR-15 の決定 (6) と (7)・持ち主の裁定 t3-hub.59.3・t3-hub.59.5・t3-hub.59.7）。
//! 席は端末の一覧の名で命令し、命令ごとに決まった旗だけを受ける（端末の値を上書きする旗は持たず、撃つ program の
//! 差し替えの旗だけを持つ・条 N-3）。board の頁の上では見せるだけにし、click・入力・key を撃つ前に断る。
//! 窓を起こすのは持ち主の tz stage open だけで、席の中（環境変数 CLAUDECODE が 1）の open は断る（計画の 3 節の席の決め）。
//! open は端末ごとの錠を持って撃ち、同時の撃ちで窓を 2 つにしない。動いている Chrome に頁が無い時だけ、
//! 持ち主の頼みとして頁の target を 1 つ作る（NEW_PAGE・この口の 1 回だけ）。窓を前に出す・動かす語は持たない。
//! host の面は器の validate が rc 0 で返った後にだけ読み、自分の anchor の state dir の host.toml だけを読む。
//! 標準出力の最後の行は、board の URL を組めた後のどの終わり方でも url の line（持ち主へ渡す URL）。

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{self, Read};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant};

use super::cdp::{self, Command, Session};
use super::json;
use super::relay::{self, Reach};
use super::terminal::{self, Terminal};
use super::tunnel::{self, Tunnel, Window};
use super::url::{self, Board};
use super::ws::Socket;
use crate::acct;
use crate::server::proc;

/// 使い方の 1 行。
pub const USAGE: &str = "usage: tz stage <navigate|viewport|reload|click|type|key|scroll|wait|screenshot|dom|console|run|open> --to <端末の名> [--url <URL> | --width <n> --height <n> --scale <n> --mobile <true|false> | --x <n> --y <n> [--dy <n>] | --text <字> | --key <鍵> | --ms <n> | --out <path>] [--repo <dir>] [--ssh <program>] [--scribe2 <program>] [--git <program>] [--tailnet <program>] [--chrome <program>]";

/// 席の中の撃ちの印の環境変数（Claude Code の Bash の道具が子の process に 1 を渡す）。
pub const SEAT_ENV: &str = "CLAUDECODE";

/// 器の検めの引数（この後に state dir）。
pub const VALIDATE_ARGS: [&str; 2] = ["validate", "--state-dir"];

/// 子の process と接続の待ち。
pub const TIMEOUT: Duration = Duration::from_secs(10);

/// ほかの tz stage open の錠を待つ上限。
pub const LOCK_WAIT: Duration = Duration::from_secs(30);

/// 動いている Chrome に頁の target を 1 つ作る method（持ち主の tz stage open の頼みの時だけ撃つ）。
pub const NEW_PAGE: &str = "Target.createTarget";

/// 不合格（断り・使い方の誤り・命令の誤り）。
const FAIL: u8 = 1;

/// 錠の見直しの間隔。
const POLL: Duration = Duration::from_millis(50);

/// どの命令でも受ける旗（端末の名と、repo と、撃つ program の差し替え）。
const COMMON: [&str; 7] = [
    "--to",
    "--repo",
    "--ssh",
    "--scribe2",
    "--git",
    "--tailnet",
    "--chrome",
];

/// tz stage の命令の語。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verb {
    /// 1 つの命令と、screenshot の書き先。
    One(Command, Option<PathBuf>),
    /// 標準入力の行ごとの命令の列。
    Run,
    /// 持ち主が表示先の端末に窓を開く。
    Open,
}

/// 読んだ引数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub verb: Verb,
    pub to: String,
    pub repo: PathBuf,
    pub ssh: OsString,
    pub scribe2: OsString,
    pub git: OsString,
    pub tailnet: OsString,
    pub chrome: OsString,
}

/// 旗の列を `--名 値` か `--名=値` の組にする（-- で始まらない字・値の無い旗・空の値は Err）。
fn pairs<'a>(args: &[&'a str]) -> Result<Vec<(&'a str, &'a str)>, String> {
    let mut out = Vec::new();
    let mut it = args.iter();
    while let Some(&arg) = it.next() {
        if !arg.starts_with("--") {
            return Err(format!("旗でない字 {arg}"));
        }
        let (name, value) = match arg.split_once('=') {
            Some(pair) => pair,
            None => (
                arg,
                *it.next().ok_or_else(|| format!("{arg} の値が無い"))?,
            ),
        };
        if value.is_empty() {
            return Err(format!("{name} の値が空"));
        }
        out.push((name, value));
    }
    Ok(out)
}

/// 旗の値を数にする（外れれば旗の名を含む Err）。
fn number<T: FromStr>(name: &str, value: &str, kind: &str) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("{name} の値 {value} は {kind} の数でない"))
}

/// 1 つの命令の語と旗の組を命令にする（命令ごとに決まった旗だけを受ける）。
pub fn one(verb: &str, flags: &[(&str, &str)]) -> Result<(Command, Option<PathBuf>), String> {
    let allowed: &[&str] = match verb {
        "navigate" => &["--url"],
        "viewport" => &["--width", "--height", "--scale", "--mobile"],
        "click" => &["--x", "--y"],
        "type" => &["--text"],
        "key" => &["--key"],
        "scroll" => &["--x", "--y", "--dy"],
        "wait" => &["--ms"],
        "screenshot" => &["--out"],
        "reload" | "dom" | "console" => &[],
        _ => return Err(format!("知らない命令 {verb}")),
    };
    let mut seen: Vec<(&str, &str)> = Vec::new();
    for &(name, value) in flags {
        if !allowed.contains(&name) {
            return Err(format!("{verb} は旗 {name} を受けない"));
        }
        if seen.iter().any(|(n, _)| *n == name) {
            return Err(format!("{name} が 2 度ある"));
        }
        seen.push((name, value));
    }
    let get = |name: &str| {
        seen.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| *v)
            .ok_or_else(|| format!("{verb} に {name} が無い"))
    };
    let u32_of = |name: &str| get(name).and_then(|v| number::<u32>(name, v, "0 以上の u32"));
    let command = match verb {
        "navigate" => {
            let url = get("--url")?;
            let shaped = (url.starts_with("http://") || url.starts_with("https://"))
                && !url.chars().any(|c| c.is_whitespace() || c.is_control());
            if !shaped {
                return Err(format!(
                    "--url の値 {url:?} は http:// か https:// で始まる空白と制御の字の無い字でない（script と端末の file を URL で運ばない）"
                ));
            }
            Command::Navigate {
                url: url.to_string(),
            }
        }
        "viewport" => Command::Viewport {
            width: u32_of("--width")?,
            height: u32_of("--height")?,
            scale: u32_of("--scale")?,
            mobile: match get("--mobile")? {
                "true" => true,
                "false" => false,
                other => return Err(format!("--mobile の値 {other} は true か false でない")),
            },
        },
        "click" => Command::Click {
            x: u32_of("--x")?,
            y: u32_of("--y")?,
        },
        "type" => Command::Type {
            text: get("--text")?.to_string(),
        },
        "key" => Command::Key {
            key: get("--key")?.to_string(),
        },
        "scroll" => Command::Scroll {
            x: u32_of("--x")?,
            y: u32_of("--y")?,
            dy: get("--dy").and_then(|v| number::<i32>("--dy", v, "i32"))?,
        },
        "wait" => Command::Wait {
            ms: get("--ms").and_then(|v| number::<u64>("--ms", v, "0 以上の u64"))?,
        },
        "screenshot" => return Ok((Command::Screenshot, Some(PathBuf::from(get("--out")?)))),
        "reload" => Command::Reload,
        "dom" => Command::Dom,
        _ => Command::Console,
    };
    Ok((command, None))
}

/// tz stage の後の引数を読む（最初の字が命令の語・共通の旗は 1 度ずつ・--to は要る）。
pub fn parse(args: &[&str]) -> Result<Call, String> {
    let Some((&verb, rest)) = args.split_first() else {
        return Err("命令が無い".to_string());
    };
    let mut common: [Option<&str>; 7] = [None; 7];
    let mut own = Vec::new();
    for (name, value) in pairs(rest)? {
        match COMMON.iter().position(|c| *c == name) {
            Some(i) => {
                if common[i].replace(value).is_some() {
                    return Err(format!("{name} が 2 度ある"));
                }
            }
            None => own.push((name, value)),
        }
    }
    let [to, repo, ssh, scribe2, git, tailnet, chrome] = common;
    let to = to.ok_or_else(|| format!("{verb} に --to が無い（表示先の端末の名を渡す）"))?;
    let verb = match verb {
        "run" | "open" => {
            if let Some((name, _)) = own.first() {
                return Err(format!("{verb} は旗 {name} を受けない"));
            }
            if verb == "run" { Verb::Run } else { Verb::Open }
        }
        _ => {
            let (command, out) = one(verb, &own)?;
            Verb::One(command, out)
        }
    };
    let program = |value: Option<&str>, default: &str| OsString::from(value.unwrap_or(default));
    Ok(Call {
        verb,
        to: to.to_string(),
        repo: PathBuf::from(repo.unwrap_or(".")),
        ssh: program(ssh, "ssh"),
        scribe2: program(scribe2, "scribe2"),
        git: program(git, acct::GIT),
        tailnet: program(tailnet, url::TAILNET),
        chrome: program(chrome, relay::CHROME),
    })
}

/// 標準入力の字を命令の列にする（行ごとに JSON の字の配列・空の行は飛ばす・誤りは行の番号を名指す）。
pub fn lines(input: &str) -> Result<Vec<(Command, Option<PathBuf>)>, String> {
    let mut out = Vec::new();
    for (i, line) in input.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let n = i + 1;
        let words: Vec<String> = json::items(line)
            .ok_or_else(|| format!("{n} 行目は JSON の配列でない"))?
            .into_iter()
            .map(|item| {
                json::unquote(item).ok_or_else(|| format!("{n} 行目の要素 {item} は字でない"))
            })
            .collect::<Result<_, _>>()?;
        let Some((verb, rest)) = words.split_first() else {
            return Err(format!("{n} 行目に命令が無い"));
        };
        if matches!(verb.as_str(), "run" | "open") {
            return Err(format!("{n} 行目: 知らない命令 {verb}（行の命令にできない）"));
        }
        let rest: Vec<&str> = rest.iter().map(String::as_str).collect();
        let flags = pairs(&rest).map_err(|e| format!("{n} 行目: {e}"))?;
        out.push(one(verb, &flags).map_err(|e| format!("{n} 行目: {e}"))?);
    }
    Ok(out)
}

/// 席の中の tz stage open の断り（環境変数の値が字 1 の時だけ Some）。
pub fn seat_refusal(value: Option<&OsStr>) -> Option<String> {
    (value == Some(OsStr::new("1"))).then(|| {
        format!(
            "tz stage open は席から撃たない（環境変数 {SEAT_ENV} が 1・窓を開くのは持ち主が素の端末か board の button で撃つ）"
        )
    })
}

/// 命令が board の頁の上で断る click・入力・key か（判断の記録 ADR-15 の決定 (7)・board は見せるだけ）。
pub fn on_board(command: &Command, page: &str, board: &Board) -> bool {
    matches!(
        command,
        Command::Click { .. } | Command::Type { .. } | Command::Key { .. }
    ) && board.holds(page)
}

/// 端末ごとの錠（base の下の tzst-<名>.lock を mode 0600 で開き、wait まで取りに行く・File の drop で放れる）。
pub fn lock(base: &Path, name: &str, wait: Duration) -> Result<File, String> {
    let shaped = !name.is_empty()
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if !shaped {
        return Err(format!(
            "端末の名 {name:?} は錠の file の名にできない（英数字と - と _ と . だけで . で始まらない名）"
        ));
    }
    let path = base.join(format!("tzst-{name}.lock"));
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(&path)
        .map_err(|e| format!("錠の file {} を開けない: {e}", path.display()))?;
    let deadline = Instant::now() + wait;
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Error(e)) => {
                return Err(format!("錠の file {} を取れない: {e}", path.display()));
            }
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "端末 {name} の錠を {} 秒の内に取れない（ほかの tz stage open が窓を開いている）",
                wait.as_secs_f32()
            ));
        }
        thread::sleep(POLL);
    }
}

/// /json/version の本文の browser の target の websocket の path（/devtools/browser/ で始まる時だけ）。
pub fn browser_resource(version: &str) -> Option<String> {
    let url = json::member(version, "webSocketDebuggerUrl").and_then(json::unquote)?;
    let rest = url.strip_prefix("ws://")?;
    let path = &rest[rest.find('/')?..];
    path.starts_with("/devtools/browser/")
        .then(|| path.to_string())
}

/// 字 = の詰めの付いた標準の base64 を byte に戻す（形の外れた字は None）。
fn unbase64(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return None;
    }
    let groups = bytes.len() / 4;
    let mut out = Vec::with_capacity(groups * 3);
    for (i, chunk) in bytes.chunks(4).enumerate() {
        let pad = chunk.iter().rev().take_while(|b| **b == b'=').count();
        if pad > 2 || (pad > 0 && i + 1 != groups) {
            return None;
        }
        let mut n: u32 = 0;
        for &b in &chunk[..4 - pad] {
            let v = match b {
                b'A'..=b'Z' => b - b'A',
                b'a'..=b'z' => b - b'a' + 26,
                b'0'..=b'9' => b - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                _ => return None,
            };
            n = (n << 6) | u32::from(v);
        }
        n <<= 6 * pad as u32;
        out.extend_from_slice(&n.to_be_bytes()[1..4 - pad]);
    }
    Some(out)
}

/// 命令の語。
fn word(command: &Command) -> &'static str {
    match command {
        Command::Navigate { .. } => "navigate",
        Command::Viewport { .. } => "viewport",
        Command::Reload => "reload",
        Command::Click { .. } => "click",
        Command::Type { .. } => "type",
        Command::Key { .. } => "key",
        Command::Scroll { .. } => "scroll",
        Command::Wait { .. } => "wait",
        Command::Screenshot => "screenshot",
        Command::Dom => "dom",
        Command::Console => "console",
    }
}

/// 使い方の誤り（標準エラーに書いて FAIL）。
fn usage(what: &str) -> u8 {
    eprintln!("tz stage: {what}\n{USAGE}");
    FAIL
}

/// tz stage の後の引数を撃つ（rc は 0 か 1）。
pub fn run(args: &[&str]) -> u8 {
    let call = match parse(args) {
        Ok(call) => call,
        Err(e) => return usage(&e),
    };
    let script = match &call.verb {
        Verb::One(command, out) => vec![(command.clone(), out.clone())],
        Verb::Open => Vec::new(),
        Verb::Run => {
            let mut input = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut input) {
                return usage(&format!("標準入力を読めない: {e}"));
            }
            match lines(&input) {
                Ok(script) => script,
                Err(e) => return usage(&e),
            }
        }
    };
    let board = match url::board(&call.tailnet, &call.git, &call.repo, TIMEOUT) {
        Ok(board) => board,
        Err(e) => {
            println!("{e}");
            return FAIL;
        }
    };
    let rc = match stage(&call, &script, &board) {
        Ok(()) => 0,
        Err(e) => {
            println!("{e}");
            FAIL
        }
    };
    println!("{}", url::line(&board.url));
    rc
}

/// 席の中の open を断り、器の検めの後に端末の行を引き、open か命令の列を撃つ。
fn stage(call: &Call, script: &[(Command, Option<PathBuf>)], board: &Board) -> Result<(), String> {
    if call.verb == Verb::Open
        && let Some(line) = seat_refusal(env::var_os(SEAT_ENV).as_deref())
    {
        return Err(line);
    }
    let terminal = face(call)?;
    let base = tunnel::user_dir(&env::temp_dir())?;
    if call.verb == Verb::Open {
        return open(call, &terminal, &base, board);
    }
    match relay::reach(&call.ssh, &call.chrome, &terminal, &base, &board.url, TIMEOUT)? {
        Reach::Terminal(tunnel) => match tunnel.window(&board.url, false)? {
            Window::Page { resource, .. } => {
                let session = Session::open(tunnel.socket(), &resource, TIMEOUT)?;
                drive(session, script, board)
            }
            Window::Absent(line) => Err(line),
        },
        Reach::Eyes {
            eyes: _eyes,
            session,
            line,
        } => {
            println!("{line}");
            drive(session, script, board)
        }
    }
}

/// 自分の anchor の state dir を引き、器の validate が rc 0 で返った後にだけ host の面を読んで --to の行を引く。
fn face(call: &Call) -> Result<Terminal, String> {
    let mut args = vec![OsString::from("-C"), call.repo.as_os_str().to_os_string()];
    args.extend(acct::GIT_ARGS.iter().map(OsString::from));
    let state = proc::capture(&call.git, &args, &call.repo, TIMEOUT)
        .map(|out| String::from_utf8_lossy(&out).trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            format!(
                "{} の git config の scribe2.statedir が読めない（自分の anchor の state dir が無い）",
                call.repo.display()
            )
        })?;
    let mut validate: Vec<OsString> = VALIDATE_ARGS.iter().map(OsString::from).collect();
    validate.push(OsString::from(&state));
    if proc::capture(&call.scribe2, &validate, &call.repo, TIMEOUT).is_none() {
        return Err(format!(
            "器 {} の validate --state-dir {state} が {} 秒の内に rc 0 で返らない（host の面を読まない）",
            call.scribe2.to_string_lossy(),
            TIMEOUT.as_secs_f32()
        ));
    }
    let text = terminal::read_face(&call.repo.join(&state))?;
    terminal::lookup(&text, &call.to)
}

/// 持ち主が窓を開く（端末ごとの錠を持ったまま、窓を 1 回だけ起こすか、頁の無い Chrome に頁を 1 つ作る）。
fn open(call: &Call, terminal: &Terminal, base: &Path, board: &Board) -> Result<(), String> {
    let name = &terminal.name;
    let _held = lock(base, name, LOCK_WAIT)?;
    let dir = tunnel::socket_dir(base)?;
    let tunnel = Tunnel::open(&call.ssh, terminal, &dir, TIMEOUT)?;
    let line = match tunnel.window(&board.url, true)? {
        Window::Page { launched: true, .. } => format!("端末 {name} に表示面の窓を起こした"),
        Window::Page { .. } => format!("端末 {name} の表示面の窓は在る（起こさない）"),
        Window::Absent(_) => {
            new_page(&tunnel, &board.url)?;
            match tunnel.window(&board.url, false)? {
                Window::Page { .. } => {
                    format!("端末 {name} の表示面の Chrome に頁の窓を 1 つ開いた")
                }
                Window::Absent(line) => return Err(line),
            }
        }
    };
    println!("{line}");
    Ok(())
}

/// 動いている Chrome の browser の target に繋ぎ、頁の target を 1 つ作る message を 1 度だけ送って応答を待つ。
fn new_page(tunnel: &Tunnel, url: &str) -> Result<(), String> {
    let resource = tunnel
        .version()
        .as_deref()
        .and_then(browser_resource)
        .ok_or_else(|| "端末の Chrome の /json/version に browser の target の path が無い".to_string())?;
    let mut socket = Socket::connect(tunnel.socket(), &resource, TIMEOUT)?;
    let params = format!("{{\"url\":{}}}", json::escape(url));
    socket
        .send(&cdp::message(1, NEW_PAGE, &params))
        .map_err(|e| format!("{NEW_PAGE}: {e}"))?;
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(format!("{NEW_PAGE}: 時間切れ"));
        }
        socket.wait(left)?;
        let text = socket
            .recv()
            .map_err(|e| format!("{NEW_PAGE}: {e}"))?
            .ok_or_else(|| format!("{NEW_PAGE}: 相手が閉じた"))?;
        if json::member(&text, "id") != Some("1") {
            continue;
        }
        if let Some(error) = json::member(&text, "error") {
            return Err(format!("{NEW_PAGE}: {error}"));
        }
        break;
    }
    socket.close()
}

/// 命令を順に撃ち（どの命令の Err でもその後の命令を撃たない）、終わりに Session を閉じる。
fn drive(
    mut session: Session,
    script: &[(Command, Option<PathBuf>)],
    board: &Board,
) -> Result<(), String> {
    let done = script.iter().try_for_each(|(command, out)| {
        shoot(&mut session, command, out.as_deref(), board)
            .map_err(|e| format!("{}: {e}", word(command)))
    });
    let closed = session.close();
    done.and(closed)
}

/// 1 つの命令を撃って出力の行を書く（board の頁の上の click・入力・key は撃たずに断る）。
fn shoot(
    session: &mut Session,
    command: &Command,
    out: Option<&Path>,
    board: &Board,
) -> Result<(), String> {
    let verb = word(command);
    if matches!(
        command,
        Command::Click { .. } | Command::Type { .. } | Command::Key { .. }
    ) {
        let page = session.url()?;
        if on_board(command, &page, board) {
            return Err(format!(
                "board の頁 {page} の上では {verb} を断る（判断の記録 ADR-15 の決定 (7)・board は見せるだけ）"
            ));
        }
    }
    let from = session.events().len();
    let replies = session.run(command)?;
    let first = replies.first().map(String::as_str);
    let result = first.and_then(|r| json::member(r, "result"));
    match command {
        Command::Screenshot => {
            let bytes = result
                .and_then(|r| json::member(r, "data"))
                .and_then(json::unquote)
                .as_deref()
                .and_then(unbase64)
                .ok_or_else(|| "応答の result の data が base64 の字でない".to_string())?;
            let path = out.ok_or_else(|| "写真の書き先が無い".to_string())?;
            fs::write(path, &bytes)
                .map_err(|e| format!("写真を {} に書けない: {e}", path.display()))?;
            println!("済み screenshot {} {} byte", path.display(), bytes.len());
        }
        Command::Dom => {
            let text = result
                .and_then(|r| json::member(r, "result"))
                .and_then(|r| json::member(r, "value"))
                .and_then(json::unquote)
                .ok_or_else(|| "応答の result の result の value が字でない".to_string())?;
            println!("{text}");
            println!("済み dom");
        }
        Command::Console => {
            for event in &session.events()[from..] {
                let method = json::member(event, "method").and_then(json::unquote);
                if matches!(
                    method.as_deref(),
                    Some("Runtime.consoleAPICalled" | "Log.entryAdded")
                ) {
                    println!("{event}");
                }
            }
            println!("済み console");
        }
        _ => println!("済み {verb}"),
    }
    Ok(())
}
