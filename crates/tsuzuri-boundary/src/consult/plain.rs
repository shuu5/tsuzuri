//! 作業場の file を symlink を辿らずに見て書く助け（判断の記録 ADR-55）。
//! 窓の hook と状態の 1 行の口は囲いの外で持ち主の権限で走り、窓は作業場の中に symlink と fifo を作れるので、作業場から file までの
//! どの段も lstat で見て、作業場と途中の段は symlink でない dir、file は普通の file だけを受ける。在る file は開いた後の fstat で
//! lstat と同じ dev と ino かを照らし、無い file は create_new（O_EXCL・symlink を辿らない）で作る。
//! lstat と開く間に途中の dir を替える競りは残る（std に段ごとの O_NOFOLLOW の開きが無い）。

use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// lstat で symlink でない dir か。
pub fn plain_dir(p: &Path) -> bool {
    fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_dir())
}

/// 作業場 `ws` からの相対 `rel` の path（作業場と途中の段が全部 symlink でない dir の時だけ）。
pub fn plain_path(ws: &Path, rel: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = rel.split('/').collect();
    let (name, dirs) = parts.split_last()?;
    let mut path = ws.to_path_buf();
    if !plain_dir(&path) {
        return None;
    }
    for dir in dirs {
        path.push(dir);
        if !plain_dir(&path) {
            return None;
        }
    }
    path.push(name);
    Some(path)
}

/// 開いた file が、開く前の lstat の普通の file と同じ dev と ino の普通の file か。
pub fn same_file(file: &File, before: &Metadata) -> bool {
    file.metadata()
        .is_ok_and(|m| m.is_file() && (m.dev(), m.ino()) == (before.dev(), before.ino()))
}

/// 作業場からの相対 `rel` が普通の file として在るか（途中の段も lstat で見る）。
pub fn plain_file(ws: &Path, rel: &str) -> bool {
    plain_path(ws, rel)
        .and_then(|p| fs::symlink_metadata(p).ok())
        .is_some_and(|m| m.file_type().is_file())
}

/// 作業場からの相対 `rel` の普通の file の末に `bytes` を足す（無ければ create_new で作る）。途中の段の symlink・
/// file の symlink・普通でない file・開いた後の照らしの違いは書かずに誤りを返す。
pub fn append_plain(ws: &Path, rel: &str, bytes: &[u8]) -> io::Result<()> {
    let refuse = |why: &str| io::Error::other(why.to_string());
    let path = plain_path(ws, rel).ok_or_else(|| refuse("途中の段が symlink でない dir でない"))?;
    let mut file = match fs::symlink_metadata(&path) {
        Ok(m) if m.file_type().is_file() => {
            let f = OpenOptions::new().append(true).open(&path)?;
            if !same_file(&f, &m) {
                return Err(refuse("開いた file が照らした file と違う"));
            }
            f
        }
        Ok(_) => return Err(refuse("普通の file でない")),
        Err(e) if e.kind() == io::ErrorKind::NotFound => OpenOptions::new()
            .append(true)
            .create_new(true)
            .open(&path)?,
        Err(e) => return Err(e),
    };
    file.write_all(bytes)
}
