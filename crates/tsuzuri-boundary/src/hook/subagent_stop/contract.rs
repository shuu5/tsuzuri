//! 終える前の門の契約の file の撃ち直し（判断の記録 ADR-72 決定 (2)）。
//! 出力の dir の子 `contract` の直下の .toml を名の順に、係の dir の子 `pf-state` の空の state dir で、
//! file ごとに器の preflight を 1 本撃ち、通らない file ごとに欠けの 1 行を返す。

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
/// 全部の撃ちの上限（hooks.json の終える前の門の timeout の内）。
const LIMIT: Duration = Duration::from_secs(8);

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
    let deadline = Instant::now() + LIMIT;
    let mut lacks = Vec::new();
    for file in found {
        let name = format!(
            "{BODIES}/{}",
            file.file_name().unwrap_or_default().to_string_lossy()
        );
        let left = deadline.saturating_duration_since(Instant::now());
        let shot = if left.is_zero() {
            Err(Failed::TimedOut)
        } else {
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
            proc::run(OsStr::new(PROGRAM), args, repo, left)
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
