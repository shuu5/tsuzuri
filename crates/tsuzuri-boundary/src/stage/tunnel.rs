//! tunnel と窓の頁の選びと窓を 1 回だけ起こすこと（要件 FR16・判断の記録 ADR-5 の決定 (1)・ADR-24 の決定 (2) と (4)）。
//! 端末の Chrome の remote debugging の口を、この server の 0700 の dir の中の unix socket へ ssh の -L で引く
//! （見積りの T3・127.0.0.1 の TCP の転送だとほかの口座の process も持ち主の Chrome を操れる）。
//! tz の 1 回の撃ちごとに繋いで閉じ、常駐しない（見積りの T1・drop で ssh を止めて dir を消す・Chrome は端末に残る）。
//! 撃つ HTTP の GET は /json/version と /json/list だけで、窓を前に出す・動かす・足す口を撃たない。
//! 窓を起こすのは呼ぶ側が起こしてよいと渡した時の 1 回だけで、
//! 撃つのは起動の引数だけ（動いている Chrome があればその Chrome が app の窓を足す）。

use std::ffi::{OsStr, OsString};
use std::fs::{self, DirBuilder, Permissions};
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{self, Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::launch;
use super::terminal::Terminal;
use super::url::Board;
use crate::server::proc;

/// tunnel の socket の名（socket_dir の dir の下）。
pub const SOCKET: &str = "cdp.sock";

/// unix socket の path の上限（108 byte から終わりの NUL を除く）。
const SOCKET_MAX: usize = 107;

/// HTTP の応答の上限。
const REPLY_MAX: usize = 1 << 20;

/// 見直しの間隔。
const POLL: Duration = Duration::from_millis(50);

/// base の下に tunnel の socket を置く dir を 0700 で新しく作る（名は tzst- と process の id と時刻の nanosecond）。
/// socket の path が上限を越えるか、コロンか空白を含めば作らずに Err。在る dir と無い base も Err。
pub fn socket_dir(base: &Path) -> Result<PathBuf, String> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    let dir = base.join(format!("tzst-{}-{nanos}", process::id()));
    let socket = dir.join(SOCKET);
    let fits = socket.to_str().is_some_and(|s| {
        s.len() <= SOCKET_MAX && !s.contains(':') && !s.contains(char::is_whitespace)
    });
    if !fits {
        return Err(format!(
            "tunnel の socket の path {} は {SOCKET_MAX} byte を越えるかコロンか空白を含む（短い dir を渡す）",
            socket.display()
        ));
    }
    DirBuilder::new()
        .mode(0o700)
        .create(&dir)
        .map_err(|e| format!("tunnel の dir {} を作れない: {e}", dir.display()))?;
    if let Err(e) = fs::set_permissions(&dir, Permissions::from_mode(0o700)) {
        let _ = fs::remove_dir(&dir);
        return Err(format!("tunnel の dir {} を 0700 にできない: {e}", dir.display()));
    }
    Ok(dir)
}

/// base の下の口座ごとの 0700 の dir（名は tzst-u と /proc/self の持ち主の uid・在れば使う・行 i-5）。
/// tunnel の dir・席の目の profile の dir・錠の file をこの下に置き、ほかの口座が先に作れる共有の名に置かない。
/// symlink か dir でない path・持ち主の違う dir・コロンか空白を含む path は Err（どれも path の字を含む）。
/// 使う前に権限を 0700 に置き直す（socket_dir と同じ・umask によらない）。
pub fn user_dir(base: &Path) -> Result<PathBuf, String> {
    let uid = fs::metadata("/proc/self")
        .map_err(|e| format!("/proc/self の持ち主を読めない（口座の dir を作らない）: {e}"))?
        .uid();
    let dir = base.join(format!("tzst-u{uid}"));
    let shaped = dir
        .to_str()
        .is_some_and(|s| !s.contains(':') && !s.contains(char::is_whitespace));
    if !shaped {
        return Err(format!(
            "口座の dir の path {} はコロンか空白を含む（短い dir を渡す）",
            dir.display()
        ));
    }
    match DirBuilder::new().mode(0o700).create(&dir) {
        Ok(()) => {}
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
        Err(e) => return Err(format!("口座の dir {} を作れない: {e}", dir.display())),
    }
    let meta = fs::symlink_metadata(&dir)
        .map_err(|e| format!("口座の dir {} を見られない: {e}", dir.display()))?;
    if !meta.file_type().is_dir() {
        return Err(format!(
            "口座の dir {} は symlink か dir でない（使わない）",
            dir.display()
        ));
    }
    if meta.uid() != uid {
        return Err(format!(
            "口座の dir {} はほかの口座（uid {}）が先に作った（使わない）",
            dir.display(),
            meta.uid()
        ));
    }
    fs::set_permissions(&dir, Permissions::from_mode(0o700))
        .map_err(|e| format!("口座の dir {} を 0700 にできない: {e}", dir.display()))?;
    Ok(dir)
}

/// 端末の窓の在りか。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Window {
    /// 頁の target の websocket の path と、この撃ちで窓を起こしたか（行 i-7 が初めて見せた印を書くのに使う）。
    Page { resource: String, launched: bool },
    /// 窓が無いことを名指す出力の 1 行。
    Absent(String),
}

/// 端末の Chrome の口へ引いた tunnel（drop で ssh を group ごと止めて dir を中身ごと消す）。
#[derive(Debug)]
pub struct Tunnel {
    ssh: OsString,
    terminal: Terminal,
    dir: PathBuf,
    socket: PathBuf,
    timeout: Duration,
    child: Option<Child>,
}

impl Tunnel {
    /// dir の下の SOCKET へ tunnel を引き、socket が出来るまで待つ。
    /// 撃てない・ssh が先に終わる・timeout を越える、はどれも端末に届かない印の Err（子を止めて dir を消す）。
    pub fn open(
        ssh: &OsStr,
        terminal: &Terminal,
        dir: &Path,
        timeout: Duration,
    ) -> Result<Tunnel, String> {
        let name = &terminal.name;
        let mut tunnel = Tunnel {
            ssh: ssh.to_os_string(),
            terminal: terminal.clone(),
            dir: dir.to_path_buf(),
            socket: dir.join(SOCKET),
            timeout,
            child: None,
        };
        let child = Command::new(ssh)
            .args(launch::tunnel_argv(terminal, &tunnel.socket))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .map_err(|e| format!("端末 {name} へ tunnel の ssh を撃てない: {e}"))?;
        let child = tunnel.child.insert(child);
        let deadline = Instant::now() + timeout;
        loop {
            if tunnel.socket.exists() {
                return Ok(tunnel);
            }
            match child.try_wait() {
                Ok(None) => {}
                Ok(Some(status)) => {
                    return Err(format!(
                        "端末 {name} に届かない（tunnel の ssh が先に終わった: {status}）"
                    ));
                }
                Err(e) => return Err(format!("端末 {name} の tunnel の ssh を見られない: {e}")),
            }
            if Instant::now() >= deadline {
                return Err(format!(
                    "端末 {name} に届かない（{} 秒の内に tunnel の socket が出来ない）",
                    timeout.as_secs_f32()
                ));
            }
            thread::sleep(POLL);
        }
    }

    /// tunnel の socket の path（行 i-3 の Session の open に渡す）。
    pub fn socket(&self) -> &Path {
        &self.socket
    }

    /// board の窓の頁を見つける（行 i-board-win）。動いている Chrome の頁の一覧に、覚えた path の頁か board の頁が在ればそれを使う。
    /// 無ければ may_open の時だけ、起動の引数を 1 回撃つ（動いている Chrome があれば、その Chrome が app の窓を足す）。
    /// 一覧の GET が 1 度落ちたら 1 度だけ撃ち直し、2 度とも落ちれば窓を起こさずに Err（在る窓を無いと見て 2 つ目を起こさない）。
    pub fn window(
        &self,
        board: &Board,
        remembered: Option<&str>,
        may_open: bool,
    ) -> Result<Window, String> {
        let name = &self.terminal.name;
        let mut running = self.get("/json/version").is_some();
        if !running {
            if !may_open {
                return Ok(Window::Absent(format!(
                    "端末 {name} に表示面の Chrome の窓が無い（起こしてよいと渡されていないので窓を起こさない・持ち主が閉じた窓は開き直さない・開くのは tz stage open）"
                )));
            }
            // 一時の不通で、動いている Chrome に 2 つ目の窓を起こさないため撃ち直す。
            thread::sleep(POLL);
            running = self.get("/json/version").is_some();
        }
        if running {
            let list = self
                .get("/json/list")
                .or_else(|| {
                    thread::sleep(POLL);
                    self.get("/json/list")
                })
                .ok_or_else(|| {
                    format!(
                        "端末 {name} の表示面の Chrome の頁の一覧を読めない（在る窓を無いと見て 2 つ目を起こさない）"
                    )
                })?;
            if let Some(resource) = launch::pick(&list, remembered, board) {
                return Ok(Window::Page {
                    resource,
                    launched: false,
                });
            }
            if !may_open {
                return Ok(Window::Absent(format!(
                    "端末 {name} の表示面の Chrome は動いているが board {} の窓が無い（ほかの board の窓は使わない・窓を足さない・窓を足すのは tz stage open）",
                    board.url
                )));
            }
        }
        self.launch_window(board, remembered)
    }

    /// 起動の引数を 1 回撃ち、上限の内に board の頁が一覧に出るまで待つ（起こし直さない）。
    fn launch_window(&self, board: &Board, remembered: Option<&str>) -> Result<Window, String> {
        let name = &self.terminal.name;
        let argv = launch::launch_argv(&self.terminal, &board.url)?;
        if proc::capture(&self.ssh, &argv, &self.dir, self.timeout).is_none() {
            return Err(format!(
                "端末 {name} で Chrome の窓を起こせない（端末の setsid と Chrome の場所と画面の env を確かめる）"
            ));
        }
        let deadline = Instant::now() + self.timeout;
        loop {
            let page = self
                .get("/json/list")
                .and_then(|list| launch::pick(&list, remembered, board));
            if let Some(resource) = page {
                return Ok(Window::Page {
                    resource,
                    launched: true,
                });
            }
            if Instant::now() >= deadline {
                return Err(format!(
                    "端末 {name} で起こした Chrome が {} 秒の内に board {} の頁を答えない（起こし直さない）",
                    self.timeout.as_secs_f32(),
                    board.url
                ));
            }
            thread::sleep(POLL);
        }
    }

    /// /json/version の本文（行 i-5 の tz stage open が browser の target の path を読む・答えなければ None）。
    pub fn version(&self) -> Option<String> {
        self.get("/json/version")
    }

    /// 頁の target の今の URL（/json/list の本文の、websocket の path が resource の項の url・読めなければ None・行 i-stage-guard）。
    pub fn page_url(&self, resource: &str) -> Option<String> {
        self.get("/json/list")
            .as_deref()
            .and_then(|list| launch::page_url(list, resource))
    }

    /// socket の先の Chrome の HTTP の口へ GET を撃ち、状態 200 で長さの在る本文を返す（ほかは None）。
    /// 相手が閉じるのを待たない（Chrome の HTTP の口は応答の後も接続を保ちうる）。
    fn get(&self, path: &str) -> Option<String> {
        let mut stream = UnixStream::connect(&self.socket).ok()?;
        stream.set_read_timeout(Some(self.timeout)).ok()?;
        stream.set_write_timeout(Some(self.timeout)).ok()?;
        let request =
            format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).ok()?;
        let mut head = Vec::new();
        let mut byte = [0u8; 1];
        while !head.ends_with(b"\r\n\r\n") {
            if head.len() > REPLY_MAX || stream.read(&mut byte).ok()? == 0 {
                return None;
            }
            head.push(byte[0]);
        }
        let head = String::from_utf8(head).ok()?;
        let mut lines = head.split("\r\n");
        if lines.next()?.split_whitespace().nth(1) != Some("200") {
            return None;
        }
        let length: usize = lines
            .filter_map(|l| l.split_once(':'))
            .find(|(n, _)| n.eq_ignore_ascii_case("content-length"))?
            .1
            .trim()
            .parse()
            .ok()?;
        if length > REPLY_MAX {
            return None;
        }
        let mut body = vec![0u8; length];
        stream.read_exact(&mut body).ok()?;
        String::from_utf8(body).ok()
    }
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        if let Some(child) = self.child.take() {
            proc::stop_with(child, OsStr::new(proc::KILL));
        }
        let _ = fs::remove_dir_all(&self.dir);
    }
}
