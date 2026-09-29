//! 外の読みの持ち回しの表（行 e-held-design・判断の記録 ADR-23 の決定 (1)(2)）。
//! 読みの鍵ごとに、撃った時刻と撃つ前に取った印（file の更新時刻と長さ）と読めた字（読めなければ None）を持つ。
//! 同じ鍵の次の読みは、印が撃つ前と同じで上限の内なら読みの関数を撃たずに持っていた字を返し、
//! 印が動いたか上限を過ぎていれば読みの関数を撃ち直す。読めなかった読み（None）は上限と `FAILED_HOLD` の短い方だけ持つ
//! （持たないと読めない子を要求ごとに撃ち直す）。走っている撃ちへの合流は読みの関数の内側（`Coalesce`）に残す。
//! 表の clone は同じ表を分け合う。読みの関数を撃つ間は表の錠を持たない。

use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime};

use super::events::stamp;

/// 読めなかった読み（None）を持つ上限（読みの上限がこれより長くても、これで打ち切る）。
pub const FAILED_HOLD: Duration = Duration::from_secs(5);

/// 読みの鍵（program と引数の列と cwd を並べたもの）。
pub type Key = Vec<OsString>;

/// 印の file 1 つの更新時刻と長さ（無ければ None）。
type Stamps = Vec<(PathBuf, Option<(SystemTime, u64)>)>;

/// 1 つの鍵の持ち分。
struct Row {
    /// 読みを撃つ前の時刻。
    at: Instant,
    /// 読みを撃つ前に取った印。
    stamps: Stamps,
    /// 読めた字（読めなければ None）。
    text: Option<String>,
}

/// 読みの鍵ごとの持ち回しの表（clone は同じ表を分け合う）。
#[derive(Clone, Default)]
pub struct Held {
    rows: Arc<Mutex<HashMap<Key, Row>>>,
}

impl fmt::Debug for Held {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Held")
    }
}

impl Held {
    pub fn new() -> Held {
        Held::default()
    }

    /// 鍵の読みを返す。印の file が撃った前と同じで、撃ってから `ceiling`（読めなかった読みは `FAILED_HOLD` とのうち短い方）
    /// の内なら、`read` を撃たず持っていた字を返す。ほかは `read` を撃って、その結果を撃つ前に取った印と組で持つ。
    pub fn get(
        &self,
        key: &[OsString],
        marks: &[PathBuf],
        ceiling: Duration,
        read: impl FnOnce() -> Option<String>,
    ) -> Option<String> {
        let stamps: Stamps = marks.iter().map(|m| (m.clone(), stamp(m))).collect();
        if let Some(row) = lock(&self.rows).get(key) {
            let limit = match row.text {
                Some(_) => ceiling,
                None => ceiling.min(FAILED_HOLD),
            };
            if row.at.elapsed() < limit && row.stamps == stamps {
                return row.text.clone();
            }
        }
        let at = Instant::now();
        let text = read();
        lock(&self.rows).insert(
            key.to_vec(),
            Row {
                at,
                stamps,
                text: text.clone(),
            },
        );
        text
    }
}

/// dir の下の file を集める（symlink の dir はたどらない・読めない dir は飛ばす）。
pub fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        match entry.file_type() {
            Ok(t) if t.is_dir() => walk(&path, out),
            Ok(_) => out.push(path),
            Err(_) => {}
        }
    }
}

/// repo の git の置き場（`.git` が dir ならそれ・file なら gitdir の行の指す dir）と、共有の置き場（commondir の指す dir）。
/// 読めなければ空。
pub fn git_dirs(repo: &Path) -> Vec<PathBuf> {
    let dot = repo.join(".git");
    let gitdir = if dot.is_dir() {
        dot
    } else {
        let Ok(text) = std::fs::read_to_string(&dot) else {
            return Vec::new();
        };
        let Some(line) = text.lines().find_map(|l| l.strip_prefix("gitdir:")) else {
            return Vec::new();
        };
        repo.join(line.trim())
    };
    let mut dirs = vec![gitdir.clone()];
    if let Ok(text) = std::fs::read_to_string(gitdir.join("commondir")) {
        dirs.push(gitdir.join(text.trim()));
    }
    dirs
}

/// git の refs の印の file（HEAD と packed-refs と refs の dir の下の全 file・置き場ごと）。
pub fn git_marks(repo: &Path, out: &mut Vec<PathBuf>) {
    for dir in git_dirs(repo) {
        out.push(dir.join("HEAD"));
        out.push(dir.join("packed-refs"));
        walk(&dir.join("refs"), out);
    }
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}
