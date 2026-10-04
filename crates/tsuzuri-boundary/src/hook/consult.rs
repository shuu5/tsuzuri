//! 席の hook の相談の拾い（設計ノート surface-wave27b 行 cs-hooks・判断の記録 ADR-29 決定 (8)）。
//! repo の git config の鍵 `tsuzuri.draftsdir` の起草の置き場に見張りの生きている印（`watch::fresh`）が無い間だけ、
//! 自分の board の未受けの口（GET /api/consult/unreceived・住所の列は裁定の未受けの口と同じ解き方）を撃つ。
//! 鍵が無いか値が dir でない repo（相談の窓を使わない project）と、見張りの居る間は、鍵の読みのほかに何も撃たない。
//! 鍵の読みと口の撃ちの待ちの全部は `PICK_BUDGET` まで（hooks.json の timeout の内に収める）。hook は file を書かない。

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::consult::{ConsultUnreceived, UNRECEIVED_PATH};
use tsuzuri_contract::wire;

use super::deliver_tool::fetch_path;
use super::question_signal::{REACH, WAIT, places};
use crate::acct::{self, GIT};
use crate::consult::watch::fresh;
use crate::consult::{DRAFTS_ARGS, tzw};
use crate::server::proc;
use crate::stage::url::{STATUS_ARGS, TAILNET};

/// 1 回の拾い（鍵の読み・port の読み・tailnet の status・口の撃ち）の待ちの全部の上限。
pub const PICK_BUDGET: Duration = Duration::from_secs(3);

/// `-C <repo>` を頭に付けた git の引数。
fn config_args<'a>(repo: &'a Path, tail: &'a [&'a str]) -> Vec<&'a OsStr> {
    let mut args = vec![OsStr::new("-C"), repo.as_os_str()];
    args.extend(tail.iter().map(OsStr::new));
    args
}

/// 見張りの居ない起草の置き場（鍵が無い・値が dir でない・見張りの印が新しい時は None）。
pub fn unwatched(repo: &Path, git: &OsStr, wait: Duration) -> Option<PathBuf> {
    let out = proc::capture(git, config_args(repo, &DRAFTS_ARGS), repo, wait)?;
    let drafts = PathBuf::from(String::from_utf8(out).ok()?.trim());
    (drafts.is_dir() && !fresh(&drafts)).then_some(drafts)
}

/// 見張りが居なければ、自分の board の未受けの口の答え（口が撃てない・読めなければ Unknown）。
/// 見張りが居るか鍵の無い repo なら None。
pub fn pick(repo: &Path, git: &OsStr, tailnet: &OsStr) -> Option<Reading<ConsultUnreceived>> {
    let end = Instant::now() + PICK_BUDGET;
    let left = |cap: Duration| end.saturating_duration_since(Instant::now()).min(cap);
    unwatched(repo, git, left(WAIT))?;
    let Some(port) = proc::capture(git, config_args(repo, &acct::BOARD_ARGS), repo, left(WAIT))
        .and_then(|out| acct::board_port(&String::from_utf8_lossy(&out)))
    else {
        return Some(Reading::Unknown);
    };
    let status = proc::capture(tailnet, STATUS_ARGS, repo, left(WAIT))
        .map(|out| String::from_utf8_lossy(&out).into_owned());
    for addr in places(status.as_deref(), port) {
        if let Ok((code, body)) = fetch_path(addr, UNRECEIVED_PATH, left(REACH)) {
            let reading = (code == 200).then(|| wire::decode(&body).ok()).flatten();
            return Some(reading.unwrap_or(Reading::Unknown));
        }
    }
    Some(Reading::Unknown)
}

/// 停止と入力の時の hook が足す相談の行（見張りが居るか鍵の無い repo なら空）。
pub fn nudge(repo: &Path) -> Vec<String> {
    pick(repo, OsStr::new(GIT), OsStr::new(TAILNET))
        .map(|w| tsuzuri_core::consult::pickup::nudge(&w, &tzw()))
        .unwrap_or_default()
}

/// 道具の周の hook が足す相談の行（受けの無い物の行だけ・無いか読めなければ空）。
pub fn items(repo: &Path) -> Vec<String> {
    match pick(repo, OsStr::new(GIT), OsStr::new(TAILNET)) {
        Some(Reading::Known(w)) => tsuzuri_core::consult::pickup::items(&w, &tzw()),
        _ => Vec::new(),
    }
}
