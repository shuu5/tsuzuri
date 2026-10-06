//! 作業場の file を symlink を辿らずに見て書く助け（判断の記録 ADR-55）。
//! 窓の hook と状態の 1 行の口は囲いの外で持ち主の権限で走り、窓は作業場の中に symlink と fifo を作れるので、作業場から file までの
//! どの段も lstat で見て、作業場と途中の段は symlink でない dir、file は普通の file だけを受ける。在る file は開いた後の fstat で
//! lstat と同じ dev と ino かを照らし、無い file は create_new（O_EXCL・symlink を辿らない）で作る。
//! lstat と開く間に途中の dir を替える競りは残る（std に段ごとの O_NOFOLLOW の開きが無い）。

use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read, Write};
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

/// 途中の段の断りの字。
pub const MID: &str = "途中の段が symlink でない dir でない";

/// 普通でない file の断りの字。
pub const NOT_PLAIN: &str = "普通の file でない";

/// 開いた後の照らしの違いの断りの字。
pub const NOT_SAME: &str = "開いた file が照らした file と違う";

/// 上限を越える file の断りの字。
pub const TOO_BIG: &str = "上限を越える";

/// hard link の在る file の断りの字（書けば作業場の外の名の file の字も替わる）。
pub const LINKED: &str = "link の数が 1 でない";

/// 作業場の下の相対 path でない字（空の段・`.`・`..`・「/」で始まる字）の断りの字。
pub const NOT_UNDER: &str = "作業場の下の相対 path でない";

/// 相対 path `rel` の段の列（空の段・`.`・`..` を断る＝作業場の外へ出ない）。
fn steps(rel: &str) -> io::Result<Vec<&str>> {
    let parts: Vec<&str> = rel.split('/').collect();
    if parts
        .iter()
        .any(|p| p.is_empty() || *p == "." || *p == "..")
    {
        return Err(io::Error::other(NOT_UNDER));
    }
    Ok(parts)
}

/// 作業場 `ws` の下の dir の段 `dirs` の path（作業場と各段を lstat で見る・無い段は NotFound・symlink でない dir でない段は断る）。
fn dir_path(ws: &Path, dirs: &[&str]) -> io::Result<PathBuf> {
    let mut path = ws.to_path_buf();
    for dir in std::iter::once("").chain(dirs.iter().copied()) {
        if !dir.is_empty() {
            path.push(dir);
        }
        if !fs::symlink_metadata(&path)?.file_type().is_dir() {
            return Err(io::Error::other(MID));
        }
    }
    Ok(path)
}

/// 作業場 `ws` からの相対 `rel` の path（`steps` と `dir_path` の照らしの後・file 自身は見ない）。
fn walk(ws: &Path, rel: &str) -> io::Result<PathBuf> {
    let parts = steps(rel)?;
    let (name, dirs) = parts
        .split_last()
        .ok_or_else(|| io::Error::other(NOT_UNDER))?;
    Ok(dir_path(ws, dirs)?.join(name))
}

/// 作業場からの相対 `rel` が在るのに `plain` の読み書きが断る形（途中の段の symlink・file の symlink・fifo・dir）なら、その断りの字
/// （無い file と無い段は None）。
pub fn odd(ws: &Path, rel: &str) -> Option<String> {
    match walk(ws, rel).map(fs::symlink_metadata) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => Some(e.to_string()),
        Ok(Err(_)) => None,
        Ok(Ok(m)) => (!m.file_type().is_file()).then(|| NOT_PLAIN.to_string()),
    }
}

/// 作業場の dir `rel` が在るのに作業場か途中の段か dir 自身が symlink でない dir でなければ、その断りの字（無い dir は None）。
pub fn odd_dir(ws: &Path, rel: &str) -> Option<String> {
    match steps(rel).and_then(|dirs| dir_path(ws, &dirs)) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => Some(e.to_string()),
        Ok(_) => None,
    }
}

/// 作業場からの相対 `rel` の在る普通の file を読む形で開き、開く前の lstat の値と返す（無ければ NotFound）。途中の段の symlink・
/// 普通でない file・開いた後の照らしの違いは開かずに誤りを返す（fifo は開く前に断るので止まらない）。
pub fn open_plain(ws: &Path, rel: &str) -> io::Result<(File, Metadata)> {
    let path = walk(ws, rel)?;
    let before = fs::symlink_metadata(&path)?;
    if !before.file_type().is_file() {
        return Err(io::Error::other(NOT_PLAIN));
    }
    let file = File::open(&path)?;
    if !same_file(&file, &before) {
        return Err(io::Error::other(NOT_SAME));
    }
    Ok((file, before))
}

/// 作業場からの相対 `rel` の普通の file の字（無い file と無い段は None）。`open_plain` の断りと、上限 `max` byte を越える file と
/// UTF-8 でない字は誤りを返す。
pub fn read_text(ws: &Path, rel: &str, max: u64) -> io::Result<Option<String>> {
    let (file, before) = match open_plain(ws, rel) {
        Ok(opened) => opened,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    if before.len() > max {
        return Err(io::Error::other(TOO_BIG));
    }
    let mut text = String::new();
    file.take(max).read_to_string(&mut text)?;
    Ok(Some(text))
}

/// 作業場からの相対 `rel` に、中身を空にした普通の file を書く形で開く（無ければ create_new で作る・在る普通の file は開いた後の
/// 照らしの後に切り詰める）。途中の段の symlink・file の symlink・普通でない file・link の数が 1 でない file・開いた後の照らしの
/// 違いは開かずに誤りを返す（作業場の外の file を切り詰めない・fifo は開く前に断るので止まらない）。
pub fn create_plain(ws: &Path, rel: &str) -> io::Result<File> {
    let path = walk(ws, rel)?;
    match fs::symlink_metadata(&path) {
        Ok(m) if m.file_type().is_file() => {
            if m.nlink() != 1 {
                return Err(io::Error::other(LINKED));
            }
            let file = OpenOptions::new().write(true).open(&path)?;
            if !same_file(&file, &m) {
                return Err(io::Error::other(NOT_SAME));
            }
            file.set_len(0)?;
            Ok(file)
        }
        Ok(_) => Err(io::Error::other(NOT_PLAIN)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            OpenOptions::new().write(true).create_new(true).open(&path)
        }
        Err(e) => Err(e),
    }
}

/// 作業場からの相対 `rel` の普通の file を `bytes` に書き替える（`create_plain` で開いて全部を書く）。
pub fn write_plain(ws: &Path, rel: &str, bytes: &[u8]) -> io::Result<()> {
    create_plain(ws, rel)?.write_all(bytes)
}

/// 作業場の dir `rel`（作業場と途中の段と dir 自身が symlink でない dir）の直下の普通の file の名（名の順・dir が無いか
/// 断る形なら空）。
pub fn plain_names(ws: &Path, rel: &str) -> Vec<String> {
    entries(ws, rel)
        .into_iter()
        .filter_map(|(name, plain)| plain.then_some(name))
        .collect()
}

/// 作業場の dir `rel` の直下の名と、普通の file か（lstat の型・名の順・dir が無いか断る形なら空）。
pub fn entries(ws: &Path, rel: &str) -> Vec<(String, bool)> {
    let Ok(dir) = steps(rel).and_then(|dirs| dir_path(ws, &dirs)) else {
        return Vec::new();
    };
    let mut out: Vec<(String, bool)> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let plain = e.file_type().is_ok_and(|t| t.is_file());
            Some((e.file_name().into_string().ok()?, plain))
        })
        .collect();
    out.sort();
    out
}
