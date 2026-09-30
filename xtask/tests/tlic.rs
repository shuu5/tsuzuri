//! 許しの file と manifest の許しの欄（行 t-license・裁定 t3-hub.67.6:20260929T0117Z-1・要件 NFR3）。
//! 外の依存を使わず、repo の根（xtask の manifest の dir の 1 つ上）からの相対の path で字を読む。
#![cfg(test)]

use std::path::PathBuf;

/// 5 つの member の dir（repo の根からの相対）。
const MEMBERS: [&str; 5] = [
    "crates/tsuzuri-contract",
    "crates/tsuzuri-core",
    "crates/tsuzuri-boundary",
    "crates/tsuzuri-surface",
    "xtask",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask の dir の 1 つ上")
        .to_path_buf()
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 契約の crate の fnv1a64 と同じ式（FNV-1a 64 bit）。
fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &b| {
        (hash ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// 表の見出しの行の直後から次の見出しの行までの行（前後の空白を除く）。見出しが無ければ None。
fn table_lines(text: &str, header: &str) -> Option<Vec<String>> {
    let mut lines = text.lines().map(str::trim);
    lines.find(|l| *l == header)?;
    Some(
        lines
            .take_while(|l| !l.starts_with('['))
            .map(str::to_string)
            .collect(),
    )
}

#[test]
fn tlic_license_files() {
    for (file, want) in [
        ("LICENSE-MIT", 0x00c6_fee1_4373_c683_u64),
        ("LICENSE-APACHE", 0xf52f_7dd8_cdb5_ce00_u64),
    ] {
        let got = fnv1a64(read(file).as_bytes());
        assert_eq!(got, want, "{file} の字の FNV-1a 64 bit: {got:016x}");
    }
}

#[test]
fn tlic_workspace_license() {
    let root = table_lines(&read("Cargo.toml"), "[workspace.package]")
        .expect("根の manifest の表 [workspace.package]");
    assert!(
        root.iter().any(|l| l == r#"license = "MIT OR Apache-2.0""#),
        "根の [workspace.package] に license の行: {root:?}"
    );
    for dir in MEMBERS {
        let package = table_lines(&read(&format!("{dir}/Cargo.toml")), "[package]")
            .unwrap_or_else(|| panic!("{dir} の manifest の表 [package]"));
        assert!(
            package.iter().any(|l| l == "license.workspace = true"),
            "{dir} の [package] に license.workspace の行: {package:?}"
        );
    }
}

#[test]
fn tlic_table_lines_stops_at_next_header() {
    let text = "[a]\n x = 1 \n\n[b]\ny = 2\n";
    assert_eq!(
        table_lines(text, "[a]"),
        Some(vec!["x = 1".to_string(), String::new()])
    );
    assert_eq!(table_lines(text, "[c]"), None);
}
