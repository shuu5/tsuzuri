//! tz consult stamp <作業場>（窓の会話の印の hook・判断の記録 ADR-55 決定 (2)）。
//! 標準入力の hook の JSON を中核の `consult::stamp::read` で読み、作業場の `.consult/stamps.jsonl` に印の 1 行を足す。
//! 書くのは、渡した path が「/」で始まり、名が `consult-cw<n>` で、窓の控え `.consult/window.json` が在る作業場の、その 1 file だけ。
//! hook は囲いの外で持ち主の権限で走り、turn の終わりの hook の断りは窓の終わりを止めるので、どの周も rc 0 で返す
//! （書かなかった周は理由を標準エラーに 1 行・作業場の symlink と普通でない file も `plain` で断る）。起動の設定は `timeout 4 <tz> consult stamp <作業場> || true` で撃つ。

use std::io::Read;
use std::path::Path;

use tsuzuri_contract::consult::WindowId;
use tsuzuri_contract::wire;
use tsuzuri_core::consult::stamp::read;

use super::WINDOW_FILE;
use super::plain::{append_plain, plain_file};
use crate::out::emit_err;
use crate::server::events::now;

/// 印の file（作業場からの相対）。
pub const STAMPS: &str = ".consult/stamps.jsonl";

/// tz consult stamp の残りの引数を受けて終了 code を返す（いつも 0）。
pub fn run(rest: &[&str]) -> u8 {
    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        payload.clear();
    }
    if let Err(why) = stamp(rest, &payload) {
        emit_err(&format!("tz consult stamp: {why}（書かない）"));
    }
    0
}

/// 作業場を照らして印の 1 行を足す。
fn stamp(rest: &[&str], payload: &str) -> Result<(), String> {
    let [ws] = rest else {
        return Err("作業場の絶対 path を 1 つ渡す".to_string());
    };
    let ws = Path::new(ws);
    let named = ws
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.strip_prefix("consult-"))
        .is_some_and(|id| WindowId::parse(id).is_ok());
    if !ws.is_absolute() || !named || !plain_file(ws, WINDOW_FILE) {
        return Err(format!("{} は窓の作業場でない", ws.display()));
    }
    let line = read(payload, now()).ok_or("hook の入力の事か会話の id の形でない")?;
    let text = wire::encode(&line).map_err(|e| e.to_string())?;
    append_plain(ws, STAMPS, format!("{text}\n").as_bytes())
        .map_err(|e| format!("印の file を書けない: {e}"))
}
