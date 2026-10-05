//! 口座の置き場の設定 file に作業場 1 つだけの信頼の印を置く書き手（行 cs-trust・判断の記録 ADR-55 決定 (3)）。
//! 字は中核の `trust::trusted` が組み、ここは読みと書きだけを持つ（器が席を起こす前に anchor の信頼の印を置く形と同じ:
//! 同じ dir の一時 file に書いて読み直し、印が真なら rename で置き換える・mode を保つ・既に真なら書かない・lock は持たない）。
//! 読めない・JSON でない・書けない置き場は rc 2 で断る（直してから撃ち直す）。

use std::fs;
use std::path::Path;

use tsuzuri_core::consult::trust::{TRUST_FILE, trusted};

use super::{Refused, UNKNOWN};

/// 口座の置き場 `account` の設定 file に作業場 `workspace` だけの信頼の印を置く（既に真なら書かない・file が無ければ作る・
/// 同じ dir の一時 file に書いて読み直し、印が真なら rename で置き換える・mode を保つ）。
pub fn place_trust(account: &Path, workspace: &str) -> Result<(), Refused> {
    let path = account.join(TRUST_FILE);
    let unknown = |why: String| {
        (
            UNKNOWN,
            format!("{} {why}（直してから撃ち直す）", path.display()),
        )
    };
    let text = match fs::read_to_string(&path) {
        Ok(text) => Some(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(unknown(format!("が読めない: {e}"))),
    };
    let body = match trusted(text.as_deref(), workspace) {
        Ok(Some(body)) => body,
        Ok(None) => return Ok(()),
        Err(why) => return Err(unknown(format!("の字に信頼の印を置けない: {why}"))),
    };
    let staged = account.join(format!("{TRUST_FILE}.{}.tz-staged", std::process::id()));
    let mode = fs::metadata(&path).map(|m| m.permissions());
    let written = fs::write(&staged, body)
        .and_then(|()| mode.map_or(Ok(()), |m| fs::set_permissions(&staged, m)));
    let settled = fs::read_to_string(&staged).ok();
    if written.is_ok()
        && trusted(settled.as_deref(), workspace) == Ok(None)
        && fs::rename(&staged, &path).is_ok()
    {
        return Ok(());
    }
    let _ = fs::remove_file(&staged);
    Err(unknown("に信頼の印を書けない".to_string()))
}
