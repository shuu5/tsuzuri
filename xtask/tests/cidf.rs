//! 契約 id の凍結の file の歯（接頭辞 cidf_）: docs/design/contract-ids.txt は、契約表の行の id を 1 行に 1 つ、字の昇順で持つ。
//! 母集団は、contracts/ の直下の .toml の全部の行と、docs/design/ の直下の .md の区間 contracts:begin から contracts:end の中の行で、
//! 行頭の字 [[contract]] の後の最初の行頭 id = の字の値を集める。凍結の file の集まりがそれを包む（⊇）ので、
//! contracts/ の導出物を外した後も緑のまま残る。外の依存を使わず、repo の根（xtask の manifest の dir の 1 つ上）からの相対の path で字を読む。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// 凍結の file の根からの相対の path。
const FROZEN: &str = "docs/design/contract-ids.txt";

fn repo_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map_or_else(|| manifest.join(".."), Path::to_path_buf)
}

/// dir の直下の拡張子 `ext` の file（path の順・下の dir は読まない）。
fn files_with(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let entries =
        std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{} を読む: {e}", dir.display()));
    let mut out: Vec<PathBuf> = entries
        .map(|entry| entry.expect("dir の項目").path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|e| e == ext))
        .collect();
    out.sort();
    out
}

/// 字の中の、行頭の字 [[contract]] の後の最初の行頭 id = の字の値（引用符の中）。
fn ids_in(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut waiting = false;
    for line in text.lines() {
        if line.starts_with("[[contract]]") {
            waiting = true;
        } else if waiting && line.starts_with("id = ") {
            waiting = false;
            let value = line["id = ".len()..].trim().trim_matches('"');
            out.push(value.to_owned());
        }
    }
    out
}

/// .md の字の区間 contracts:begin から contracts:end の中の字。
fn region_of(text: &str) -> String {
    let mut inside = false;
    let mut out = String::new();
    for line in text.lines() {
        if line.contains("contracts:end") {
            inside = false;
        }
        if inside {
            out.push_str(line);
            out.push('\n');
        }
        if line.contains("contracts:begin") {
            inside = true;
        }
    }
    out
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

/// 根の木の契約表の行の id の全部。
fn table_ids(root: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for path in files_with(&root.join("contracts"), "toml") {
        out.extend(ids_in(&read(&path)));
    }
    for path in files_with(&root.join("docs/design"), "md") {
        out.extend(ids_in(&region_of(&read(&path))));
    }
    out
}

#[test]
fn cidf_frozen_ids_cover_the_tables() {
    let root = repo_root();
    let frozen: BTreeSet<String> = read(&root.join(FROZEN)).lines().map(str::to_owned).collect();
    let missing: Vec<String> = table_ids(&root).difference(&frozen).cloned().collect();
    assert!(
        missing.is_empty(),
        "凍結の file {FROZEN} に無い契約表の行の id: {missing:?}"
    );
}

#[test]
fn cidf_frozen_ids_are_sorted_and_unique() {
    let text = read(&repo_root().join(FROZEN));
    let lines: Vec<&str> = text.lines().collect();
    assert!(!lines.is_empty(), "凍結の file が空");
    let blank: Vec<&&str> = lines
        .iter()
        .filter(|line| line.is_empty() || line.trim() != **line)
        .collect();
    assert!(blank.is_empty(), "空の行か前後に空白を持つ行: {blank:?}");
    let strict = lines.windows(2).all(|pair| pair[0] < pair[1]);
    assert!(strict, "字の昇順でないか同じ行を 2 度持つ");
}
