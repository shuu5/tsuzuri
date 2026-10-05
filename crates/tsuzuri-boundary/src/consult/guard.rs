//! tz consult guard（窓の PreToolUse の守りの hook・判断の記録 ADR-29 決定 (7)）。
//! 標準入力の hook の JSON を中核の `consult::guard::judge` で判じ、通すなら何も出さずに rc 0、断るなら理由を
//! 標準エラーに 1 行書いて rc 2（Claude Code は PreToolUse の rc 2 を道具の断りと読み、標準エラーを窓に渡す）。
//! 引数を取らない（在れば断る）。hook は囲いの外で持ち主の権限で走るので、読むだけで file を書かない。
//! 起動の設定は `timeout 4 <tz> consult guard || exit 2` で撃つので、落ちても時間切れでも断る側に倒れる。

use std::io::Read;

use tsuzuri_core::consult::guard::judge;

use crate::out::emit_err;

/// 断る時の終了 code（PreToolUse の hook の断り）。
pub const DENY: u8 = 2;

/// tz consult guard の残りの引数を受けて終了 code を返す。
pub fn run(rest: &[&str]) -> u8 {
    if !rest.is_empty() {
        emit_err("相談の窓の守り: 引数を取らない");
        return DENY;
    }
    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        payload.clear();
    }
    match judge(&payload) {
        Ok(()) => 0,
        Err(why) => {
            emit_err(&format!("相談の窓の守り: {why}"));
            DENY
        }
    }
}
