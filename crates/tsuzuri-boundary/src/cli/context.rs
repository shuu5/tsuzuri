//! tz context（判断の記録 ADR-51 の決定 (4) と帰結）: 設計ノートの行 1 本の近い仕様（設計の層の塊）を
//! 標準出力へ出す。書かない。席と係が起草の時に、器が便に渡す前の材料と同じ組みの字を引く口。
//! tz context --row <ノート>#<行> [--dir <dir>]（--dir は設計の置き場・省けば撃った場所からの design-intent）。
//! 組みは folio の lib の入口 `folio::entry::context`で、塊の字はそのまま出す。
//! 終了 code は 組めた 0・使い方の誤りと名指しの誤り（ノートか行が無い）1・正本が読めない 2（まだ分からない）。

use std::path::Path;

use crate::out::{emit, emit_err};

pub const USAGE: &str = "usage: tz context --row <ノート>#<行> [--dir <dir>]";

/// 設計の置き場の既定（撃った場所からの相対）。
pub const DIR: &str = "design-intent";

/// 不合格（使い方の誤り）。
const FAIL: u8 = 1;

pub fn run(rest: &[&str]) -> u8 {
    let (row, dir) = match parse(rest) {
        Ok(args) => args,
        Err(e) => {
            emit_err(&format!("tz context: {e}\n{USAGE}"));
            return FAIL;
        }
    };
    match ::folio::entry::context(Path::new(dir), row) {
        Ok(text) => {
            if let Some(body) = text.strip_suffix('\n') {
                emit(body);
            }
            0
        }
        Err((code, why)) => {
            emit_err(&format!("tz context: {why}"));
            code
        }
    }
}

/// `--名 値` か `--名=値` の --row（要る）と --dir（省ける）を読む（空の値と 2 度の引数と知らない引数は断る）。
fn parse<'a>(rest: &[&'a str]) -> Result<(&'a str, &'a str), String> {
    let (mut row, mut dir) = (None, None);
    let mut it = rest.iter();
    while let Some(arg) = it.next() {
        let (name, value) = match arg.split_once('=') {
            Some((n, v)) => (n, Some(v)),
            None => (*arg, None),
        };
        let slot = match name {
            "--row" => &mut row,
            "--dir" => &mut dir,
            _ => return Err(format!("知らない引数 {arg}")),
        };
        let value = match value {
            Some(v) => v,
            None => *it.next().ok_or_else(|| format!("{name} の値が無い"))?,
        };
        if value.is_empty() {
            return Err(format!("{name} の値が空"));
        }
        if slot.replace(value).is_some() {
            return Err(format!("{name} が 2 度ある"));
        }
    }
    let row = row.ok_or_else(|| "--row が無い".to_string())?;
    Ok((row, dir.unwrap_or(DIR)))
}
