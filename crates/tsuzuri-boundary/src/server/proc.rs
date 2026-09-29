//! 子 process を撃つ共通の部品（行 hb-proc が台帳の読みから割った）。
//! 子は新しい process group に入れ、時間切れには孫まで group ごと止める（便 e-reap）。

use std::ffi::OsStr;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// 子 process の終わりを確かめる間隔。
const WAIT_STEP: Duration = Duration::from_millis(5);

/// 子の process group の全体へ KILL の signal を送る道具の名（便 e-reap）。
pub const KILL: &str = "kill";

/// 子 process を 1 本撃ち、rc 0 で `timeout` の内に返した標準出力を返す（それ以外は None）。
/// cwd は `cwd`・標準入力は空・標準エラーは捨てる。
/// 子は新しい process group に入れ（group の id は子の pid）、止めるときは孫まで group ごと止める（便 e-reap）。
pub fn capture<I, S>(program: &OsStr, args: I, cwd: &Path, timeout: Duration) -> Option<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let deadline = Instant::now() + timeout;
    let mut child = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .ok()?;
    let Some(mut stdout) = child.stdout.take() else {
        stop(child);
        return None;
    };
    // 標準出力は別の thread で読み切る（pipe が詰まって子が止まらないように）。
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut out = Vec::new();
        let _ = tx.send(stdout.read_to_end(&mut out).map(|_| out));
    });
    let Ok(Ok(out)) = rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) else {
        stop(child);
        return None;
    };
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => thread::sleep(WAIT_STEP),
            _ => {
                stop(child);
                return None;
            }
        }
    };
    status.success().then_some(out)
}

/// `run` が標準出力を返せなかったわけ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failed {
    /// 子を起動できない。
    Unstartable,
    /// 上限の内に終わらない（group ごと止めた）。
    TimedOut,
    /// rc が 0 でない（子の標準エラーの最後の空でない行の字・読みきれなければ空の字）。
    Exit(String),
}

impl Failed {
    /// log の字（`起動できない`・`時間切れ`・`rc が 0 でない: <行>`）。
    pub fn word(&self) -> String {
        match self {
            Failed::Unstartable => "起動できない".to_string(),
            Failed::TimedOut => "時間切れ".to_string(),
            Failed::Exit(line) => format!("rc が 0 でない: {line}"),
        }
    }
}

/// 子 process を `capture` と同じ撃ちで 1 本撃ち、rc 0 で `timeout` の内に返した標準出力を返す。
/// 標準エラーも別の thread で読み、rc が 0 でなければ最後の空でない行を `Failed::Exit` に持たせる
/// （子が終わった後に上限の残りまで読みを待ち、読みきれなければ空の字）。
pub fn run<I, S>(program: &OsStr, args: I, cwd: &Path, timeout: Duration) -> Result<Vec<u8>, Failed>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let deadline = Instant::now() + timeout;
    let mut child = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .map_err(|_| Failed::Unstartable)?;
    let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take()) else {
        stop(child);
        return Err(Failed::Unstartable);
    };
    // 標準出力と標準エラーはそれぞれ別の thread で読み切る（pipe が詰まって子が止まらないように）。
    let out_rx = read_all(stdout);
    let err_rx = read_all(stderr);
    let Ok(Ok(out)) = out_rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) else {
        stop(child);
        return Err(Failed::TimedOut);
    };
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => thread::sleep(WAIT_STEP),
            _ => {
                stop(child);
                return Err(Failed::TimedOut);
            }
        }
    };
    if status.success() {
        return Ok(out);
    }
    let line = match err_rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
        Ok(Ok(err)) => String::from_utf8_lossy(&err)
            .lines()
            .map(str::trim)
            .rfind(|l| !l.is_empty())
            .unwrap_or_default()
            .to_string(),
        _ => String::new(),
    };
    Err(Failed::Exit(line))
}

/// 読み口を別の thread で終わりまで読み、読めた字を送る。
fn read_all<R: Read + Send + 'static>(mut from: R) -> mpsc::Receiver<std::io::Result<Vec<u8>>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = tx.send(from.read_to_end(&mut buf).map(|_| buf));
    });
    rx
}

/// 子 process を group ごと止めて片付ける。
fn stop(child: Child) {
    stop_with(child, OsStr::new(KILL));
}

/// 子の process group（id は子の pid）の全体へ道具 `kill` で KILL の signal を送り、子を待って片付ける。
/// group へ送れたら true。道具が撃てないか失敗したら子だけを止めて false。
/// 孫は待たない（親が居ないので OS の側が片付ける）。
pub fn stop_with(mut child: Child, kill: &OsStr) -> bool {
    let group = format!("-{}", child.id());
    let sent = Command::new(kill)
        .args(["-KILL", "--", &group])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !sent {
        let _ = child.kill();
    }
    let _ = child.wait();
    sent
}
