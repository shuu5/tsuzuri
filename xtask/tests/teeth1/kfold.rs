//! 段の束ね（判断の記録 ADR-32）の歯（接頭辞 kfold_）: 歯の群 tests/<群>/main.rs が群の dir の .rs を全部 1 度だけ mod の行で名指すこと
//! （名指さない file は組まれず歯が黙って消える）・群と module の数が表 GROUPS のとおりであること
//! （#[test] の fn の名の一意は行 t-fold-contract が足す・器は歯を fn の名だけで照らす）。表は畳みの行が w/fold2.py と同じ手で書き直す。
//! 外の依存を使わず、repo の根（xtask の manifest の dir の 1 つ上）からの相対の path で字を読む。
#![cfg(test)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// 歯の群（repo の根からの dir）と module の数（main.rs を除く .rs と、common のような dir の module）。
const GROUPS: &[(&str, usize)] = &[
    // kfold-groups-begin
    ("xtask/tests/teeth1", 9),
    // kfold-groups-end
];

fn repo_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map_or_else(|| manifest.join(".."), Path::to_path_buf)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

/// 根の Cargo.toml の members の dir。
fn members() -> Vec<String> {
    let text = read(&repo_root().join("Cargo.toml"));
    let start = text.find("members = [").expect("members の表");
    let end = start + text[start..].find(']').expect("members の終わり");
    text[start..end]
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// member の tests/ の直下の、main.rs を持つ dir（群・path の順）。
fn groups_of(member: &str) -> Vec<PathBuf> {
    let tests = repo_root().join(member).join("tests");
    let Ok(entries) = std::fs::read_dir(&tests) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .map(|e| e.expect("tests の項目").path())
        .filter(|p| p.join("main.rs").is_file())
        .collect();
    out.sort();
    out
}

/// 群の dir の module の名（main.rs を除く .rs の stem と、mod.rs を持つ dir の名）。
fn modules_on_disk(group: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for entry in std::fs::read_dir(group).expect("群の dir を読む") {
        let path = entry.expect("群の項目").path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("項目の名")
            .to_string();
        if path.is_dir() && path.join("mod.rs").is_file() {
            out.insert(name);
        } else if let Some(stem) = name.strip_suffix(".rs").filter(|s| *s != "main") {
            out.insert(stem.to_string());
        }
    }
    out
}

/// main.rs の行 `mod <名>;` の名（出てきた順・重なりも数える・#[path] の付いた mod は群の dir の外の file なので除く）。
fn declared(group: &Path) -> Vec<String> {
    let text = read(&group.join("main.rs"));
    let mut out = Vec::new();
    let mut pathed = false;
    for line in text.lines().map(str::trim) {
        if let Some(name) = line.strip_prefix("mod ").and_then(|r| r.strip_suffix(';')) {
            if !pathed {
                out.push(name.to_string());
            }
            pathed = false;
        } else if line.starts_with("#[path") {
            pathed = true;
        }
    }
    out
}

#[test]
fn kfold_groups_declare_every_module_once() {
    let root = repo_root();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for member in members() {
        for group in groups_of(&member) {
            let decl = declared(&group);
            let set: BTreeSet<String> = decl.iter().cloned().collect();
            assert_eq!(
                set.len(),
                decl.len(),
                "{} の mod の行が重なる",
                group.display()
            );
            assert_eq!(
                set,
                modules_on_disk(&group),
                "{} の mod の行と dir の file",
                group.display()
            );
            let rel = group
                .strip_prefix(&root)
                .expect("根の下")
                .to_string_lossy()
                .into_owned();
            seen.insert(rel, set.len());
        }
    }
    let want: BTreeMap<String, usize> =
        GROUPS.iter().map(|(g, n)| ((*g).to_string(), *n)).collect();
    assert_eq!(seen, want, "群と module の数");
}
