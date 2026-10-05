//! tz consult statusline <作業場>（窓の状態の 1 行の口・判断の記録 ADR-57 決定 (1)(コ)・(4)）。
//! Claude Code が設定の statusLine の命令として撃つ（標準入力に session の JSON）。窓の控え `.consult/window.json`・
//! findings/ の所見の数・束の要約値 bundle/digest・標準入力の文脈の割合を中核の `status::render` で 1 行にして標準出力に出す。
//! 口は囲いの外で持ち主の権限で走り、材料は窓が書ける作業場の file なので symlink を辿らない（作業場から file までの
//! どの段も lstat で dir か普通の file かを見て、開いた後の fstat で同じ file かを照らす・助けは `plain`）。普通の file でない物（fifo など）は
//! 開かず、読む字に上限を置く。読めない物は欄を出さない。何も書かない。席の口でないので窓の守りは通す。
//! 渡した path が「/」で始まらないか名が `consult-cw<n>` でないか symlink でない dir でなければ、何も出さずに rc 1。

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use tsuzuri_contract::consult::{WindowFile, WindowId};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::status::{context_percent, render};

use super::plain::{plain_dir, plain_path, same_file};
use super::{FAIL, WINDOW_FILE};
use crate::out::{emit, emit_err};

/// 作業場の file から読む字の上限（byte）。
pub const FILE_MAX: u64 = 16 * 1024;

/// 標準入力から読む字の上限（byte）。
pub const STDIN_MAX: u64 = 256 * 1024;

/// 束の要約値の file（作業場からの相対）。
pub const DIGEST: &str = "bundle/digest";

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
    emit(&line);
    0
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
