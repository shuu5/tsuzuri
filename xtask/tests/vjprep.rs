//! 行 v-join-prep の歯: 器 scribe2 の開発の道具の部品（xtask）の package の名と dir の名を同じ新しい名 scribe2-xtask に揃え、
//! 器の cargo の設定の別名 xtask を新しい名へ向けたこと（判断の記録 ADR-33 の決定 (11)・根の workspace に入れる前の行）。
//! 根の部品の名と器の部品の名が重ならないことを見る（根の workspace に入れる時の名の衝突を先に除く）。外の依存を使わず repo の根からの相対の path で読む。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::PathBuf;

/// 揃えた後の器の開発の道具の部品の名（package の名と crates/ の下の dir の名が同じ字）。
const RUNNER: &str = "scribe2-xtask";
/// 揃える前の名（器の crates/ の下に dir として残らない）。
const OLD: &str = "xtask";
/// 器の cargo の設定の別名 xtask の値（cargo xtask の字の意味を保つ）。
const ALIAS: &str = "run -q -p scribe2-xtask --";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask の dir の 1 つ上")
        .to_path_buf()
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読めない: {e}"))
}

/// manifest の行 members = [ から ] までの二重引用符の中の字（順のまま）。
fn members(manifest: &str) -> Vec<String> {
    let Some(at) = manifest.find("members = [") else {
        return Vec::new();
    };
    let tail = &manifest[at + "members = [".len()..];
    let list = &tail[..tail.find(']').unwrap_or(tail.len())];
    list.split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// manifest の節 [package] の行 name = の二重引用符の中の字。
fn package_name(manifest: &str) -> Option<String> {
    let at = manifest.find("[package]")?;
    manifest[at..]
        .lines()
        .skip(1)
        .take_while(|l| !l.starts_with('['))
        .find_map(|l| l.strip_prefix("name = \""))
        .and_then(|v| v.strip_suffix('"'))
        .map(str::to_string)
}

/// 錠の [[package]] の塊のうち、行頭が字 source = の行を持たない塊（workspace の部品）の名。
fn local_names(lock: &str) -> BTreeSet<String> {
    lock.split("[[package]]")
        .skip(1)
        .filter(|b| !b.lines().any(|l| l.starts_with("source = ")))
        .filter_map(|b| b.lines().find_map(|l| l.strip_prefix("name = \"")))
        .filter_map(|v| v.strip_suffix('"'))
        .map(str::to_string)
        .collect()
}

/// cargo の設定の節 [alias] の鍵 key の二重引用符の中の字。
fn alias(config: &str, key: &str) -> Option<String> {
    let at = config.find("[alias]")?;
    let head = format!("{key} = \"");
    config[at..]
        .lines()
        .skip(1)
        .take_while(|l| !l.starts_with('['))
        .find_map(|l| l.strip_prefix(head.as_str()))
        .and_then(|v| v.strip_suffix('"'))
        .map(str::to_string)
}

/// workspace の members の各 dir の Cargo.toml の package の名。
fn names_of(base: &str, manifest: &str) -> BTreeSet<String> {
    members(manifest)
        .iter()
        .map(|m| {
            let rel = if base.is_empty() {
                format!("{m}/Cargo.toml")
            } else {
                format!("{base}/{m}/Cargo.toml")
            };
            package_name(&read(&rel)).unwrap_or_else(|| panic!("{rel} に package の名が無い"))
        })
        .collect()
}

#[test]
fn vjprep_runner_dir_and_name_match() {
    let vessel = read("scribe2/Cargo.toml");
    assert_eq!(
        members(&vessel),
        [
            "crates/scribe2",
            "crates/scribe2-boundary",
            "crates/scribe2-xtask"
        ],
        "器の members は 3 つでこの順"
    );
    assert!(
        !root().join("scribe2/crates").join(OLD).exists(),
        "器の crates/ の下に dir xtask が残らない"
    );
    let manifest = read(&format!("scribe2/crates/{RUNNER}/Cargo.toml"));
    assert_eq!(
        package_name(&manifest).as_deref(),
        Some(RUNNER),
        "package の名は dir の名と同じ字"
    );
    // 否定の見本: 名だけを替えて dir を替えない members の字は、揃えた後の members と違う。
    let half = "[workspace]\nmembers = [\"crates/scribe2\", \"crates/scribe2-boundary\", \"crates/xtask\"]\n";
    assert_ne!(
        members(half),
        members(&vessel),
        "dir を替えない members の字は通さない"
    );
    // 否定の見本: dir だけを替えて名を替えない manifest の名は RUNNER でない。
    assert_eq!(
        package_name("[package]\nname = \"xtask\"\n").as_deref(),
        Some(OLD)
    );
}

#[test]
fn vjprep_lock_names_disjoint_from_root() {
    let vessel = names_of("scribe2", &read("scribe2/Cargo.toml"));
    let rooted = names_of("", &read("Cargo.toml"));
    assert_eq!(
        local_names(&read("scribe2/Cargo.lock")),
        vessel,
        "器の錠の workspace の部品の名は器の members の名と同じ"
    );
    assert!(
        vessel.contains(RUNNER) && !vessel.contains(OLD),
        "器の部品の名に scribe2-xtask が在り xtask が無い"
    );
    assert!(rooted.contains(OLD), "根の開発の道具の名は xtask のまま");
    assert_eq!(
        vessel.intersection(&rooted).count(),
        0,
        "根の部品の名と器の部品の名は重ならない"
    );
    // 否定の見本: 器の錠の名を 1 つだけ揃える前に戻すと、根の名と重なる。
    let lock = "[[package]]\nname = \"scribe2\"\n\n[[package]]\nname = \"xtask\"\n\n[[package]]\nname = \"serde\"\nsource = \"registry+x\"\n";
    let old: BTreeSet<String> = local_names(lock);
    assert_eq!(
        old.iter().map(String::as_str).collect::<Vec<_>>(),
        ["scribe2", "xtask"],
        "source の在る塊は数えない"
    );
    assert_eq!(
        old.intersection(&rooted).count(),
        1,
        "揃える前の名は根の名と 1 つ重なる"
    );
}

#[test]
fn vjprep_alias_points_to_runner() {
    let config = read("scribe2/.cargo/config.toml");
    assert_eq!(
        alias(&config, "xtask").as_deref(),
        Some(ALIAS),
        "別名 xtask は新しい名の部品を撃つ"
    );
    // 否定の見本: 別名の値の -p の語だけを揃える前に戻した字は通さない。
    let old = "[alias]\nxtask = \"run -q -p xtask --\"\n";
    assert_ne!(alias(old, "xtask").as_deref(), Some(ALIAS));
    // 否定の見本: 節 [alias] の外の同じ字は別名と読まない。
    assert_eq!(
        alias("[build]\nxtask = \"run -q -p scribe2-xtask --\"\n", "xtask"),
        None
    );
}
