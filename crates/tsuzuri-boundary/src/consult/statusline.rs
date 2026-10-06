//! tz consult statusline <作業場>（窓の状態の 1 行の口・判断の記録 ADR-57 決定 (1)(コ)・(4)）。
//! Claude Code が設定の statusLine の命令として撃つ（標準入力に session の JSON）。窓の控え `.consult/window.json`・
//! findings/ の所見の数・束の要約値 bundle/digest・標準入力の文脈の割合を中核の `status::render` で 1 行にして標準出力に出す。
//! 口は囲いの外で持ち主の権限で走り、材料は窓が書ける作業場の file なので symlink を辿らない（作業場から file までの
//! どの段も lstat で dir か普通の file かを見て、開いた後の fstat で同じ file かを照らす・助けは `plain`）。普通の file でない物（fifo など）は
//! 開かず、読む字に上限を置く。読めない物は欄を出さない。口は file を書かない（口座の命令の書きは命令のまま）。席の口でないので窓の守りは通す。
//! 渡した path が「/」で始まらないか名が `consult-cw<n>` でないか symlink でない dir でなければ、何も出さずに rc 1。
//! 1 行の前に、窓を起こした口座（環境の CLAUDE_CONFIG_DIR・起動の口が窓に渡す）の設定 file の statusLine の命令を、同じ標準入力を
//! 渡して作業場で殻に撃った出力の行を出す（中核の `status::stanza`・判断の記録 ADR-67）。命令は新しい process group で撃ち、上限
//! `ACCOUNT_WAIT` を越えたら group ごと止める。設定が無い・読めない・鍵が無い・落ちた・時間切れ・入れ子の周は、1 行の末に訳の語を添える。

use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_contract::consult::{WindowFile, WindowId};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::status::{
    Miss, account_command, account_lines, context_percent, render, stanza,
};

use super::launch::ACCOUNT_ENV;
use super::plain::{plain_dir, plain_path, same_file};
use super::{FAIL, WINDOW_FILE};
use crate::out::{emit, emit_err};
use crate::server::proc::{KILL, stop_with};

/// 作業場の file から読む字の上限（byte）。
pub const FILE_MAX: u64 = 16 * 1024;

/// 標準入力から読む字の上限（byte）。
pub const STDIN_MAX: u64 = 256 * 1024;

/// 束の要約値の file（作業場からの相対）。
pub const DIGEST: &str = "bundle/digest";

/// 口座の設定 file（口座の置き場からの相対）。
pub const SETTINGS: &str = "settings.json";

/// 口座の設定 file から読む字の上限（byte）。
pub const SETTINGS_MAX: u64 = 1 << 20;

/// 口座の命令を撃つ殻。
pub const SHELL: &str = "sh";

/// 口座の命令を待つ上限（命令の字の `timeout 2` の内に 1 行を出す）。
pub const ACCOUNT_WAIT: Duration = Duration::from_millis(1500);

/// 口座の命令の出力から読む字の上限（byte）。
pub const ACCOUNT_OUT_MAX: u64 = 64 * 1024;

/// 口座の命令の子に置く印の環境変数（印の在る口は口座の命令を撃たない）。
pub const NESTED_ENV: &str = "TZ_CONSULT_STATUSLINE";

/// 口座の命令の終わりを見る間。
const STEP: Duration = Duration::from_millis(5);

/// tz consult statusline の残りの引数を受けて終了 code を返す。
pub fn run(rest: &[&str]) -> u8 {
    let [ws] = rest else {
        emit_err("tz consult statusline: 作業場の絶対 path を 1 つ渡す");
        return FAIL;
    };
    let ws = Path::new(ws);
    let named = ws
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.strip_prefix("consult-"))
        .is_some_and(|id| WindowId::parse(id).is_ok());
    if !ws.is_absolute() || !named || !plain_dir(ws) {
        emit_err(&format!(
            "tz consult statusline: {} は窓の作業場でない",
            ws.display()
        ));
        return FAIL;
    }
    let mut input = String::new();
    if std::io::stdin()
        .take(STDIN_MAX)
        .read_to_string(&mut input)
        .is_err()
    {
        input.clear();
    }
    let window = read_plain(ws, WINDOW_FILE).and_then(|t| wire::decode::<WindowFile>(&t).ok());
    let digest = read_plain(ws, DIGEST);
    let line = render(
        window.as_ref(),
        count(ws, "findings"),
        digest.as_deref(),
        context_percent(&input),
    );
    emit(&stanza(account(ws, &input), &line));
    0
}

/// 環境の口座の置き場の設定 file の字（置き場が無い・読めない・UTF-8 でなければ None・頭の `SETTINGS_MAX` byte まで）。
pub fn account_settings() -> Option<String> {
    let dir = std::env::var_os(ACCOUNT_ENV)?;
    let file = File::open(Path::new(&dir).join(SETTINGS)).ok()?;
    let mut text = String::new();
    file.take(SETTINGS_MAX).read_to_string(&mut text).ok()?;
    Some(text)
}

/// 口座の設定の statusLine の命令を作業場で撃った出力の行（出せなければ訳）。
fn account(ws: &Path, input: &str) -> Result<Vec<String>, Miss> {
    if std::env::var_os(NESTED_ENV).is_some() {
        return Err(Miss::Nested);
    }
    let command = account_command(&account_settings().ok_or(Miss::Unread)?)?;
    let out = shoot(&command, ws, input)?;
    Ok(account_lines(&String::from_utf8_lossy(&out)))
}

/// 命令を殻で撃ち（cwd は作業場・標準入力に `input`・標準エラーは捨てる・子に `NESTED_ENV`）、rc 0 で上限の内に返した
/// 標準出力（頭の `ACCOUNT_OUT_MAX` byte）を返す。子は新しい process group に入れ、時間切れには group ごと止める。
fn shoot(command: &str, ws: &Path, input: &str) -> Result<Vec<u8>, Miss> {
    let deadline = Instant::now() + ACCOUNT_WAIT;
    let mut child = Command::new(SHELL)
        .args(["-c", command])
        .current_dir(ws)
        .env(NESTED_ENV, "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map_err(|_| Miss::Failed)?;
    let (Some(mut stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
        stop_with(child, OsStr::new(KILL));
        return Err(Miss::Failed);
    };
    let input = input.as_bytes().to_vec();
    thread::spawn(move || {
        let _ = stdin.write_all(&input);
    });
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut out = Vec::new();
        let read = stdout.take(ACCOUNT_OUT_MAX).read_to_end(&mut out);
        let _ = tx.send(read.map(|_| out));
    });
    let out = match rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
        Ok(Ok(out)) => out,
        Ok(Err(_)) => {
            stop_with(child, OsStr::new(KILL));
            return Err(Miss::Failed);
        }
        Err(_) => {
            stop_with(child, OsStr::new(KILL));
            return Err(Miss::TimedOut);
        }
    };
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(out),
            Ok(Some(_)) => return Err(Miss::Failed),
            Ok(None) if Instant::now() < deadline => thread::sleep(STEP),
            _ => {
                stop_with(child, OsStr::new(KILL));
                return Err(Miss::TimedOut);
            }
        }
    }
}

/// 作業場からの相対 `rel` の普通の file の字（`plain` の照らし・頭の `FILE_MAX` byte まで・UTF-8 でなければ None）。
pub fn read_plain(ws: &Path, rel: &str) -> Option<String> {
    let path = plain_path(ws, rel)?;
    let before = fs::symlink_metadata(&path)
        .ok()
        .filter(|m| m.file_type().is_file())?;
    let file = File::open(&path).ok().filter(|f| same_file(f, &before))?;
    let mut text = String::new();
    file.take(FILE_MAX).read_to_string(&mut text).ok()?;
    Some(text)
}

/// 作業場の dir `rel`（symlink でない dir）の直下の、名が `.json` で終わる普通の file の数（symlink は数えない）。
fn count(ws: &Path, rel: &str) -> Option<usize> {
    let dir = ws.join(rel);
    if !plain_dir(&dir) {
        return None;
    }
    let n = fs::read_dir(&dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter(|e| e.file_name().to_str().is_some_and(|n| n.ends_with(".json")))
        .count();
    Some(n)
}
