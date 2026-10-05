//! 作業場の file を symlink を辿らずに見て書く助けの歯（接頭辞 cwpln_・設計ノート surface-wave29b 行 cs-plain-open・判断の記録 ADR-55）。
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）に作業場と外の file を置き、作業場か途中の段か file が symlink か fifo の時に、
//! 見る助けが偽か None を返し、足す助けが書かずに誤りを返して外の file を替えないことと、普通の file に足し、無い file を作ることを見る。
#![cfg(test)]

use std::fs::{self, File};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;

use tsuzuri_boundary::consult::plain::{
    append_plain, plain_dir, plain_file, plain_path, same_file,
};

/// 歯ごとの置き場（前の撃ちの残りを消して作る）に、`.consult/window.json` を持つ作業場 ws と外の dir out を作る。
fn place(name: &str) -> (PathBuf, PathBuf) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("cwpln-{name}"));
    let _ = fs::remove_dir_all(&root);
    let (ws, out) = (root.join("ws"), root.join("out"));
    fs::create_dir_all(ws.join(".consult")).unwrap();
    fs::create_dir_all(&out).unwrap();
    fs::write(ws.join(".consult/window.json"), "{}").unwrap();
    fs::write(out.join("f"), "外").unwrap();
    (ws, out)
}

#[test]
fn cwpln_reads_only_plain_dirs_and_files() {
    let (ws, out) = place("see");
    assert!(plain_dir(&ws) && plain_file(&ws, ".consult/window.json"));
    assert_eq!(plain_path(&ws, ".consult/x"), Some(ws.join(".consult/x")));
    symlink(&out, ws.join("link")).unwrap();
    symlink(out.join("f"), ws.join(".consult/f")).unwrap();
    assert!(!plain_dir(&ws.join("link")) && plain_path(&ws, "link/f").is_none());
    assert!(!plain_file(&ws, ".consult/f") && !plain_file(&ws, ".consult"));
    let made = Command::new("mkfifo")
        .arg(ws.join(".consult/p"))
        .status()
        .unwrap();
    assert!(made.success() && !plain_file(&ws, ".consult/p"));
    let wsl = ws.with_file_name("wsl");
    symlink(&ws, &wsl).unwrap();
    assert!(plain_path(&wsl, ".consult/window.json").is_none());
    let (a, b) = (ws.join(".consult/window.json"), out.join("f"));
    let lst = fs::symlink_metadata(&a).unwrap();
    assert!(same_file(&File::open(&a).unwrap(), &lst));
    assert!(!same_file(&File::open(&b).unwrap(), &lst));
}

#[test]
fn cwpln_appends_without_following() {
    let (ws, out) = place("add");
    append_plain(&ws, ".consult/s", b"a\n").unwrap();
    append_plain(&ws, ".consult/s", b"b\n").unwrap();
    assert_eq!(fs::read_to_string(ws.join(".consult/s")).unwrap(), "a\nb\n");
    symlink(out.join("f"), ws.join(".consult/l")).unwrap();
    symlink(&out, ws.join("d")).unwrap();
    let made = Command::new("mkfifo")
        .arg(ws.join(".consult/p"))
        .status()
        .unwrap();
    assert!(made.success());
    for rel in [".consult/l", "d/f", ".consult/p", ".consult"] {
        assert!(append_plain(&ws, rel, b"x").is_err(), "{rel}");
    }
    assert_eq!(fs::read_to_string(out.join("f")).unwrap(), "外");
}
