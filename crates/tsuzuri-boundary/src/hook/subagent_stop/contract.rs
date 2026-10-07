//! 終える前の門の契約の file の撃ち直し（判断の記録 ADR-72 決定 (2)）。
//! 出力の dir の子 `contract` の直下の .toml を名の順に、係の dir の子 `pf-state` の空の state dir で、
//! file ごとに器の preflight を 1 本撃ち、通らない file ごとに欠けの 1 行を返す。
//! 撃ちの上限は契約 1 本ごとに `EACH` で、全部で `TOTAL` である。

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tsuzuri_core::agent::stop::BODIES;

use crate::server::proc::{self, Failed};

/// 係の dir の子の空の state dir の名。
const STATE: &str = "pf-state";
/// 器の道具の名（PATH で解く）。
const PROGRAM: &str = "scribe2";
/// 契約 1 本の撃ちの上限（測りの 1 本の上の 2.8 秒の約 3 倍）。
const EACH: Duration = Duration::from_secs(8);
/// 全部の撃ちの上限（hooks.json の終える前の門の timeout 60 秒から散文の門と記録の読みの 10 秒を残す）。
const TOTAL: Duration = Duration::from_secs(50);

/// 撃ちの始めからの経過 `elapsed` での 1 本の撃ちの上限（全部の上限を使い切っていれば None）。
pub fn shot_limit(elapsed: Duration) -> Option<Duration> {
    (elapsed < TOTAL).then(|| EACH.min(TOTAL - elapsed))
}

/// 出力の dir `out` の子 contract の直下の .toml の file（名の順・読めなければ空）。
fn files(out: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(out.join(BODIES)) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "toml"))
        .collect();
    found.sort();
    found
}

/// 契約の file の撃ち直しの欠け（file が無ければ空・state dir が空でなければその 1 行だけ・通らない file ごとに 1 行）。
pub fn holes(agent: &Path, out: &Path, repo: &Path, bead: &str) -> Vec<String> {
    let found = files(out);
    if found.is_empty() {
        return Vec::new();
    }
    let state = agent.join(STATE);
    let empty = fs::create_dir_all(&state)
        .and_then(|()| fs::read_dir(&state))
        .is_ok_and(|mut entries| entries.next().is_none());
    if !empty {
        return vec![format!(
            "preflight の空の state dir {} が無い（作れないか空でない）",
            state.display()
        )];
    }
    let begun = Instant::now();
    let mut lacks = Vec::new();
    for file in found {
        let name = format!(
            "{BODIES}/{}",
            file.file_name().unwrap_or_default().to_string_lossy()
        );
        let shot = if let Some(limit) = shot_limit(begun.elapsed()) {
            let args: [&OsStr; 10] = [
                OsStr::new("pipe"),
                OsStr::new("preflight"),
                OsStr::new("--contract"),
                file.as_os_str(),
                OsStr::new("--bead"),
                OsStr::new(bead),
                OsStr::new("--repo"),
                repo.as_os_str(),
                OsStr::new("--state-dir"),
                state.as_os_str(),
            ];
            proc::run(OsStr::new(PROGRAM), args, repo, limit)
        } else {
            Err(Failed::TimedOut)
        };
        if let Err(failed) = shot {
            lacks.push(format!(
                "契約の file {name} の preflight が通らない（{}）",
                failed.word()
            ));
        }
    }
    lacks
}
