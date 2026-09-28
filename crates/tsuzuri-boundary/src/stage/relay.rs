//! 端末に届かない時の落ちる先（行 i-4・要件 FR16・受入 AC13・判断の記録 ADR-15 の決定 (5)・持ち主の裁定 t3-hub.59.4）。
//! ssh で届かない端末には board の URL を渡すだけにし、席はその端末の browser を操作しない。
//! 席の目はこの server の headless の Chrome（screenshot・DOM・console）で、画面を配信する窓は作らない。
//! 席の目の Chrome の口は --remote-debugging-pipe（子の fd 3 と fd 4 だけで話し、口座の外から届かない・見積りの T3）。
//! std の Command は fd 3 と fd 4 を子に渡せないので、sh の -c の 1 行で子の標準入出力を写してから Chrome を exec する。
//! pipe の上では browser の段に繋がるので、最初の頁の target に flatten の形で付く。

use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::cdp::{self, Session};
use super::json;
use super::pipe::Pipe;
use super::terminal::Terminal;
use super::tunnel::{self, Tunnel};
use crate::server::proc;

/// 既定の席の目の Chrome（PATH で引く）。
pub const CHROME: &str = "google-chrome";

/// fd を写す shell（PATH で引く）。
pub const SHELL: &str = "sh";

/// 子の標準入力を fd 3 に、標準出力を fd 4 に写し、Chrome 自身の標準入出力と標準エラーを捨てて exec する 1 行。
pub const PIPE_SCRIPT: &str = "exec \"$0\" \"$@\" 3<&0 4>&1 </dev/null >/dev/null 2>&1";

/// 頁の target を探す method。
pub const TARGETS: &str = "Target.getTargets";

/// 頁の target に付く method。
pub const ATTACH: &str = "Target.attachToTarget";

/// 頁の target の見直しの間隔。
const POLL: Duration = Duration::from_millis(50);

/// 席の目の Chrome の引数（headless・専用の profile・pipe の口・初めの問いを出さない・最初の頁）。
pub fn eyes_argv(profile: &Path, url: &str) -> Vec<String> {
    vec![
        "--headless=new".to_string(),
        format!("--user-data-dir={}", profile.display()),
        "--remote-debugging-pipe".to_string(),
        "--no-first-run".to_string(),
        "--no-default-browser-check".to_string(),
        url.to_string(),
    ]
}

/// SHELL の後の引数（-c・PIPE_SCRIPT・Chrome の program・eyes_argv）。
pub fn spawn_argv(chrome: &OsStr, profile: &Path, url: &str) -> Vec<OsString> {
    let mut argv = vec![
        OsString::from("-c"),
        OsString::from(PIPE_SCRIPT),
        chrome.to_os_string(),
    ];
    argv.extend(eyes_argv(profile, url).into_iter().map(OsString::from));
    argv
}

/// 席の目の headless の Chrome（drop で Chrome を group ごと止めて profile の dir を中身ごと消す）。
#[derive(Debug)]
pub struct Eyes {
    dir: PathBuf,
    child: Option<Child>,
}

impl Eyes {
    /// base の下の 0700 の profile の dir で席の目の Chrome を起こし、最初の頁の target に付いた Session を返す。
    /// 形の外の URL は Chrome を撃たずに Err。付く前の Err は子を止めて dir を消す。
    pub fn open(
        chrome: &OsStr,
        base: &Path,
        url: &str,
        timeout: Duration,
    ) -> Result<(Eyes, Session), String> {
        let shaped = (url.starts_with("http://") || url.starts_with("https://"))
            && !url.chars().any(|c| c.is_whitespace() || c.is_control());
        if !shaped {
            return Err(format!(
                "URL {url:?} は http:// か https:// で始まる空白と制御の字の無い字でない（席の目の Chrome を撃たない）"
            ));
        }
        let program = chrome.to_string_lossy();
        let mut eyes = Eyes {
            dir: tunnel::socket_dir(base)?,
            child: None,
        };
        let child = Command::new(SHELL)
            .args(spawn_argv(chrome, &eyes.dir, url))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .map_err(|e| format!("席の目の Chrome {program} を撃てない: {e}"))?;
        let child = eyes.child.insert(child);
        let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
            return Err(format!("席の目の Chrome {program} の pipe が無い"));
        };
        let mut pipe = Pipe::new(stdin, stdout);
        attach(&mut pipe, timeout)
            .map_err(|e| format!("席の目の headless の Chrome {program} に付けない: {e}"))?;
        Ok((eyes, Session::attach(pipe, timeout)))
    }

    /// profile の dir。
    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

impl Drop for Eyes {
    fn drop(&mut self) {
        if let Some(child) = self.child.take() {
            proc::stop_with(child, OsStr::new(proc::KILL));
        }
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// 最初の頁の target を探して flatten の形で付き、pipe に session の id を置く。
fn attach(pipe: &mut Pipe, timeout: Duration) -> Result<(), String> {
    let start = Instant::now();
    let mut id = 1;
    let target = loop {
        let reply = request(pipe, id, TARGETS, "{}", timeout)?;
        id += 1;
        let page = json::member(&reply, "result")
            .and_then(|r| json::member(r, "targetInfos"))
            .and_then(json::items)
            .and_then(|infos| {
                infos.into_iter().find_map(|info| {
                    let kind = json::member(info, "type").and_then(json::unquote)?;
                    let target = json::member(info, "targetId").and_then(json::unquote)?;
                    (kind == "page").then_some(target)
                })
            });
        if let Some(target) = page {
            break target;
        }
        if start.elapsed() >= timeout {
            return Err(format!(
                "{} 秒の内に頁の target が無い",
                timeout.as_secs_f32()
            ));
        }
        thread::sleep(POLL);
    };
    let params = format!("{{\"targetId\":{},\"flatten\":true}}", json::escape(&target));
    let reply = request(pipe, id, ATTACH, &params, timeout)?;
    let session = json::member(&reply, "result")
        .and_then(|r| json::member(r, "sessionId"))
        .and_then(json::unquote)
        .ok_or_else(|| format!("{ATTACH}: 応答に sessionId が無い"))?;
    pipe.join(&session);
    Ok(())
}

/// 1 つの message を撃ち、同じ id の応答を timeout まで待つ（ほかの字は読み捨てる・誤りの応答は Err）。
fn request(
    pipe: &mut Pipe,
    id: u64,
    method: &str,
    params: &str,
    timeout: Duration,
) -> Result<String, String> {
    pipe.send(&cdp::message(id, method, params))
        .map_err(|e| format!("{method}: {e}"))?;
    let want = id.to_string();
    let deadline = Instant::now() + timeout;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(format!("{method}: 時間切れ"));
        }
        let text = pipe
            .recv(left)
            .map_err(|e| format!("{method}: {e}"))?
            .ok_or_else(|| format!("{method}: 相手が閉じた"))?;
        if json::member(&text, "id") != Some(want.as_str()) {
            continue;
        }
        if let Some(error) = json::member(&text, "error") {
            return Err(format!("{method}: {error}"));
        }
        return Ok(text);
    }
}

/// 端末へ届いたか、席の目に落ちたか。
pub enum Reach {
    /// 端末へ引いた tunnel（窓の在りかは撃つ側が window で見る）。
    Terminal(Tunnel),
    /// 席の目と、その頁の Session と、落ちた旨の出力の 1 行。
    Eyes {
        eyes: Eyes,
        session: Session,
        line: String,
    },
}

/// 端末へ tunnel を引き、届かなければ席の目の Chrome に落ちる（端末の browser を操作しない）。
pub fn reach(
    ssh: &OsStr,
    chrome: &OsStr,
    terminal: &Terminal,
    base: &Path,
    url: &str,
    timeout: Duration,
) -> Result<Reach, String> {
    let dir = tunnel::socket_dir(base)?;
    let far = match Tunnel::open(ssh, terminal, &dir, timeout) {
        Ok(tunnel) => return Ok(Reach::Terminal(tunnel)),
        Err(far) => far,
    };
    match Eyes::open(chrome, base, url, timeout) {
        Ok((eyes, session)) => Ok(Reach::Eyes {
            eyes,
            session,
            line: format!(
                "端末 {} に届かないので席の目（この server の headless の Chrome）に落ちた（{far}）・持ち主へは board の URL を渡し、席は端末の browser を操作しない",
                terminal.name
            ),
        }),
        Err(e) => Err(format!("{far}・{e}")),
    }
}
