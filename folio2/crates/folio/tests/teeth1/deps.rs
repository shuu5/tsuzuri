//! 外の部品（外部 crate）の歯（便 212・docs/design/delivery-212.md §1 (c)）。binary を撃たない・file を読むだけ。
//! yaml-rust2 の既定の機能 encoding（字の符号を見分けて読む口 YamlDecoder だけが使う）を外し、
//! その機能だけが引いていた外の部品 encoding_rs を置き場の解いた依存の一覧（Cargo.lock）から無くす。
//! 外の部品を増やす・減らすときの確認は条 A-3.1 が持つ（ここは減らした後の形を数えるだけ）。
#![cfg(test)]

use crate::common::repo_root;
use std::fs;
use std::path::Path;

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{} が読めない: {e}", path.display()))
}

/// Cargo.toml の `[節]` の中で `yaml-rust2 =` で始まる行の値（空白を除いた字）の一覧。
fn yaml_rust2_values(manifest: &str, section: &str) -> Vec<String> {
    let mut current = String::new();
    let mut values = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            current = line.to_string();
            continue;
        }
        if current == section
            && let Some(rest) = line.strip_prefix("yaml-rust2")
            && let Some(value) = rest.trim_start().strip_prefix('=')
        {
            values.push(value.chars().filter(|c| !c.is_whitespace()).collect());
        }
    }
    values
}

/// Cargo.lock の package ごとの（名, 依存の名の一覧）。依存の字の「名 版」は名だけにする。
fn lock_packages(lock: &str) -> Vec<(String, Vec<String>)> {
    let mut packages: Vec<(String, Vec<String>)> = Vec::new();
    let mut in_deps = false;
    for line in lock.lines() {
        if line == "[[package]]" {
            packages.push((String::new(), Vec::new()));
            in_deps = false;
        } else if let Some(name) = line.strip_prefix("name = ") {
            if let Some(last) = packages.last_mut() {
                last.0 = name.trim_matches('"').to_string();
            }
        } else if line == "dependencies = [" {
            in_deps = true;
        } else if in_deps && line == "]" {
            in_deps = false;
        } else if in_deps && let Some(last) = packages.last_mut() {
            let dep = line.trim().trim_end_matches(',').trim_matches('"');
            last.1.push(dep.split(' ').next().unwrap_or("").to_string());
        }
    }
    packages
}

#[test]
fn f212_manifest_turns_off_yaml_rust2_default_features() {
    let manifest = read(&repo_root().join("crates/folio/Cargo.toml"));
    for section in ["[dependencies]", "[build-dependencies]"] {
        let values = yaml_rust2_values(&manifest, section);
        assert_eq!(values.len(), 1, "{section} の yaml-rust2 の行は 1 つのはず: {values:?}");
        let value = &values[0];
        assert!(
            value.contains("default-features=false"),
            "{section} の yaml-rust2 は既定の機能を外すはず: {value}"
        );
        assert!(
            !value.contains("\"encoding\""),
            "{section} の yaml-rust2 は機能 encoding を名指さないはず: {value}"
        );
    }
}

#[test]
fn f212_lockfile_resolves_no_encoding_rs() {
    // 行 k-join-folio: folio は tsuzuri の根の workspace の member なので、錠は folio2/ の 1 つ上の repo の根の Cargo.lock。
    let packages = lock_packages(&read(&repo_root().join("../Cargo.lock")));
    // 一覧が読めていない場合を合格にしない（P-4.1）。
    let names: Vec<&str> = packages.iter().map(|(name, _)| name.as_str()).collect();
    assert!(names.contains(&"folio"), "Cargo.lock に folio が無い: {names:?}");
    let yaml = packages
        .iter()
        .find(|(name, _)| name == "yaml-rust2")
        .unwrap_or_else(|| panic!("Cargo.lock に yaml-rust2 が無い: {names:?}"));
    assert!(
        yaml.1.contains(&"hashlink".to_string()),
        "yaml-rust2 の依存が読めていない: {:?}",
        yaml.1
    );
    assert!(
        !yaml.1.contains(&"encoding_rs".to_string()),
        "yaml-rust2 の依存に encoding_rs が残る: {:?}",
        yaml.1
    );
    assert!(
        !names.contains(&"encoding_rs"),
        "Cargo.lock に encoding_rs が残る"
    );
}
