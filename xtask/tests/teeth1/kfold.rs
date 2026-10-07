//! 段の束ね（判断の記録 ADR-32）の歯（接頭辞 kfold_）: 歯の群 tests/<群>/main.rs が群の dir の .rs を全部 1 度だけ mod の行で名指すこと
//! （名指さない file は組まれず歯が黙って消える）・群と module の数が表 GROUPS のとおりであること
//! （#[test] の fn の名の一意は行 t-fold-contract が足す・器は歯を fn の名だけで照らす）。表は畳みの行が w/fold2.py と同じ手で書き直す。
//! 外の依存を使わず、repo の根（xtask の manifest の dir の 1 つ上）からの相対の path で字を読む。
#![cfg(test)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// 歯の群（repo の根からの dir）と module の数（main.rs を除く .rs と、mod.rs を持つ dir の module・共通の module POOL は数えない）。
const GROUPS: &[(&str, usize)] = &[
    // kfold-groups-begin
    ("crates/tsuzuri-boundary/tests/teeth1", 26),
    ("crates/tsuzuri-boundary/tests/teeth2", 27),
    ("crates/tsuzuri-boundary/tests/teeth3", 15),
    ("crates/tsuzuri-boundary/tests/teeth4", 14),
    ("crates/tsuzuri-boundary/tests/teeth5", 29),
    ("crates/tsuzuri-boundary/tests/teeth6", 10),
    ("crates/tsuzuri-contract/tests/teeth1", 14),
    ("crates/tsuzuri-core/tests/teeth1", 28),
    ("crates/tsuzuri-core/tests/teeth2", 25),
    ("crates/tsuzuri-core/tests/teeth3", 22),
    ("crates/tsuzuri-surface/tests/teeth1", 31),
    ("crates/tsuzuri-surface/tests/teeth2", 38),
    ("crates/tsuzuri-surface/tests/teeth3", 30),
    ("crates/tsuzuri-surface/tests/teeth4", 31),
    ("crates/tsuzuri-surface/tests/teeth5", 45),
    ("crates/tsuzuri-surface/tests/teeth6", 1),
    ("folio2/crates/folio/tests/teeth1", 7),
    ("folio2/crates/folio/tests/tz1", 11),
    ("folio2/crates/folio/tests/tz2", 10),
    ("folio2/crates/folio/tests/tz3", 17),
    ("folio2/crates/folio/tests/tz4", 16),
    ("folio2/crates/folio/tests/tz5", 3),
    ("xtask/tests/teeth1", 15),
    // kfold-groups-end
];

/// 群の共通の module の名（畳みの道具が字で同じ helper の写しを寄せる先・判断の記録 ADR-63 の決定 (8) の (c)）。main.rs が名指すことは
/// ほかの module と同じに見て、表 GROUPS の module の数には数えない（群を足すたびに表を書き直さずに寄せの行を並べるため）。
const POOL: &str = "common";

fn repo_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map_or_else(|| manifest.join(".."), Path::to_path_buf)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

/// 群の行の上限（群の dir の直下の .rs の行の和・main.rs を除く・判断の記録 ADR-32 の決定 (2)）。この定数と次の定数の行が 2 つの値の正本で、
/// xtask の命令 kfold-cap と席の道具は行頭からの字 `const <名>: usize = <数字と _>;` を読む（形を替えない・歯 kfold_caps_read_back_by_kfold_cap）。
const GROUP_LINES_CAP: usize = 10_000;
/// 段の file（member の tests/ の直下の .rs）の本数の上限（判断の記録 ADR-63 の決定 (8)・命令 kfold-cap は届いても落とさない）。
const STAGE_FILES_CAP: usize = 24;

#[expect(
    dead_code,
    reason = "上限の行の読みと群の探しだけを使い、命令の測りと判じは使わない"
)]
#[path = "../../src/kcap.rs"]
mod kcap;

/// 根の Cargo.toml の members の dir（命令 kfold-cap と同じ読み）。
fn members() -> Vec<String> {
    kcap::members(&repo_root()).expect("members の表")
}

/// member の tests/ の直下の、main.rs を持つ dir（群・path の順・命令 kfold-cap と同じ探し）。
fn groups_of(member: &str) -> Vec<PathBuf> {
    kcap::groups(&repo_root(), member)
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
            seen.insert(rel, set.iter().filter(|m| m.as_str() != POOL).count());
        }
    }
    let want: BTreeMap<String, usize> =
        GROUPS.iter().map(|(g, n)| ((*g).to_string(), *n)).collect();
    assert_eq!(seen, want, "群と module の数");
}

/// 歯の区間（tests/ の file は全部・src の file は行頭の #[cfg(test)] から後）の #[test] の直下の fn の名。
fn test_fns(text: &str, whole: bool) -> Vec<String> {
    let region = if whole {
        text
    } else {
        text.find("\n#[cfg(test)]").map_or("", |i| &text[i..])
    };
    let mut out = Vec::new();
    let mut pending = false;
    for line in region.lines() {
        let t = line.trim();
        if t == "#[test]" {
            pending = true;
        } else if pending && !(t.is_empty() || t.starts_with("#[") || t.starts_with("//")) {
            pending = false;
            if let Some(rest) = t.split_once("fn ").map(|(_, r)| r) {
                out.push(
                    rest.split(|c: char| !(c.is_alphanumeric() || c == '_'))
                        .next()
                        .unwrap_or("")
                        .to_string(),
                );
            }
        }
    }
    out
}

fn rs_under(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.expect("dir の項目").path();
        if path.is_dir() {
            rs_under(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn kfold_test_names_unique_in_each_crate() {
    let root = repo_root();
    let mut total = 0;
    for member in members() {
        let mut names: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (sub, whole) in [("tests", true), ("src", false)] {
            let mut files = Vec::new();
            rs_under(&root.join(&member).join(sub), &mut files);
            for file in files {
                for name in test_fns(&read(&file), whole) {
                    names.entry(name).or_default().push(
                        file.strip_prefix(&root)
                            .expect("根の下")
                            .display()
                            .to_string(),
                    );
                }
            }
        }
        let dups: Vec<(&String, &Vec<String>)> =
            names.iter().filter(|(_, v)| v.len() > 1).collect();
        assert!(dups.is_empty(), "{member} の中で重なる歯の名: {dups:?}");
        total += names.len();
    }
    assert!(total >= 2_500, "数えた歯の名が {total}");
}

/// 2 つの上限の定数の行を、命令 kfold-cap の読み（src/kcap.rs の caps）でこの file の字から読み直すと、定数の値と同じ。
#[test]
fn kfold_caps_read_back_by_kfold_cap() {
    let caps = kcap::caps(include_str!("kfold.rs")).expect("上限の行を読む");
    assert_eq!(
        (caps.group_lines, caps.stage_files),
        (GROUP_LINES_CAP, STAGE_FILES_CAP)
    );
}
