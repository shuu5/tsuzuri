//! xtask: `cargo run -q -p xtask -- <task>` の形で起動する（alias の file は置かない）。
//! check は workspace の build・歯の全部・clippy を順に撃ち、最初に落ちた段の rc を返す。

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// check の段。順に撃ち、最初に落ちた段で止まる。
const CHECK_STEPS: &[&[&str]] = &[
    &["build", "--workspace"],
    &["nextest", "run", "--workspace"],
    &[
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ],
];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("check") if args.len() == 1 => exit_code(check(&workspace_root())),
        _ => {
            eprintln!("usage: cargo run -q -p xtask -- check");
            ExitCode::from(2)
        }
    }
}

fn exit_code(rc: i32) -> ExitCode {
    ExitCode::from(u8::try_from(rc).unwrap_or(1))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask は workspace の root の直下に在る")
        .to_path_buf()
}

/// 段を順に撃ち、最初に落ちた段の rc を返す（全部通れば 0）。
fn check(root: &Path) -> i32 {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    for step in CHECK_STEPS {
        eprintln!("xtask check: cargo {}", step.join(" "));
        let status = Command::new(&cargo).args(*step).current_dir(root).status();
        let rc = match status {
            Ok(s) if s.success() => continue,
            Ok(s) => s.code().unwrap_or(1),
            Err(e) => {
                eprintln!("xtask check: cargo を起動できない: {e}");
                1
            }
        };
        eprintln!("xtask check: 落ちた段 cargo {} (rc {rc})", step.join(" "));
        return rc;
    }
    0
}

/// 歯が読む file の簡易な読み（依存を serde と serde_json に限るので TOML の crate を使わない）。
#[cfg(test)]
mod read {
    use std::path::Path;

    /// 行の中の `"…"` を順に取り出す（TOML の基本文字列の簡易な読み・escape は扱わない）。
    fn quoted(s: &str) -> Vec<String> {
        s.split('"')
            .skip(1)
            .step_by(2)
            .map(str::to_string)
            .collect()
    }

    /// `key = [ … ]` の配列の文字列（複数行にまたがってよい）。
    pub(super) fn string_array(text: &str, key: &str) -> Option<Vec<String>> {
        let start = text.lines().position(|l| {
            l.split_once('=')
                .is_some_and(|(k, v)| k.trim() == key && v.trim_start().starts_with('['))
        })?;
        let mut body = String::new();
        for line in text.lines().skip(start) {
            body.push_str(line);
            body.push('\n');
            if line.contains(']') {
                let (_, rest) = body.split_once('=')?;
                let inner = rest.split_once('[')?.1.rsplit_once(']')?.0;
                return Some(quoted(inner));
            }
        }
        None
    }

    /// `[section]` の中の key の名の一覧（`key = …` の行の左辺）。
    pub(super) fn section_keys(text: &str, section: &str) -> Vec<String> {
        let header = format!("[{section}]");
        let mut inside = false;
        let mut keys = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                inside = line == header;
            } else if inside
                && !line.starts_with('#')
                && let Some((k, _)) = line.split_once('=')
            {
                keys.push(k.trim().trim_matches('"').to_string());
            }
        }
        keys
    }

    /// `[section]` の中の `key = "…"` の値。
    pub(super) fn string_value(text: &str, section: &str, key: &str) -> Option<String> {
        let header = format!("[{section}]");
        let mut inside = false;
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                inside = line == header;
            } else if inside
                && let Some((k, v)) = line.split_once('=')
                && k.trim() == key
            {
                return quoted(v).into_iter().next();
            }
        }
        None
    }

    /// workspace の member の package の名（root の Cargo.toml の members から各 member の Cargo.toml を読む）。
    pub(super) fn member_names(root: &Path) -> Vec<String> {
        let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("Cargo.toml を読む");
        string_array(&manifest, "members")
            .expect("workspace の members")
            .iter()
            .map(|dir| {
                let m = std::fs::read_to_string(root.join(dir).join("Cargo.toml"))
                    .unwrap_or_else(|e| panic!("{dir}/Cargo.toml を読む: {e}"));
                string_value(&m, "package", "name")
                    .unwrap_or_else(|| panic!("{dir}/Cargo.toml の package の名"))
            })
            .collect()
    }

    /// Cargo.lock の package の (名, workspace の外か)。source の在る package が外。
    pub(super) fn lock_packages(lock: &str) -> Vec<(String, bool)> {
        lock.split("[[package]]")
            .skip(1)
            .map(|block| {
                let mut name = None;
                let mut external = false;
                for line in block.lines() {
                    if let Some((k, v)) = line.split_once('=') {
                        match k.trim() {
                            "name" => name = quoted(v).into_iter().next(),
                            "source" => external = true,
                            _ => {}
                        }
                    }
                }
                (name.expect("package の name"), external)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::read::{lock_packages, member_names, section_keys, string_array, string_value};
    use super::workspace_root;

    const MEMBERS: [&str; 5] = [
        "tsuzuri-contract",
        "tsuzuri-core",
        "tsuzuri-boundary",
        "tsuzuri-surface",
        "xtask",
    ];

    /// workspace の外の package の許可の一覧: serde と serde の導出（serde_derive）が連れて来るもの（便 a）と、
    /// serde_json とそれが連れて来るもの（便 b）。
    const ALLOWED_OUTSIDE: [&str; 11] = [
        "serde",
        "serde_core",
        "serde_derive",
        "proc-macro2",
        "quote",
        "syn",
        "unicode-ident",
        "serde_json",
        "itoa",
        "memchr",
        "zmij",
    ];

    #[test]
    fn skeleton_workspace_has_five_members() {
        let mut names = member_names(&workspace_root());
        names.sort();
        let mut want = MEMBERS.map(str::to_string).to_vec();
        want.sort();
        assert_eq!(names, want);
    }

    #[test]
    fn skeleton_common_verify_is_xtask_check() {
        let vessel =
            std::fs::read_to_string(workspace_root().join(".vessel.toml")).expect(".vessel.toml");
        assert_eq!(
            string_array(&vessel, "common-verify"),
            Some(vec!["cargo run -q -p xtask -- check".to_string()])
        );
    }

    #[test]
    fn skeleton_lock_outside_is_allowed() {
        let lock =
            std::fs::read_to_string(workspace_root().join("Cargo.lock")).expect("Cargo.lock");
        let packages = lock_packages(&lock);
        let outside: Vec<&str> = packages
            .iter()
            .filter(|(_, ext)| *ext)
            .map(|(n, _)| n.as_str())
            .collect();
        let stray: Vec<&str> = outside
            .iter()
            .copied()
            .filter(|n| !ALLOWED_OUTSIDE.contains(n))
            .collect();
        assert!(stray.is_empty(), "許可の一覧の外の package: {stray:?}");
        assert!(outside.contains(&"serde"), "serde が Cargo.lock に無い");
        assert!(
            outside.contains(&"serde_json"),
            "serde_json が Cargo.lock に無い"
        );
        let mut inside: Vec<&str> = packages
            .iter()
            .filter(|(_, ext)| !*ext)
            .map(|(n, _)| n.as_str())
            .collect();
        inside.sort_unstable();
        let mut want = MEMBERS.to_vec();
        want.sort_unstable();
        assert_eq!(inside, want);
    }

    /// 境界の crate（最小の server と tz の入口・便 e-min）は外の依存を足さない（標準 library だけ）。
    /// 直接依存は workspace の member だけで、binary の名は tz。
    #[test]
    fn skeleton_boundary_std_only_and_bin_tz() {
        let manifest =
            std::fs::read_to_string(workspace_root().join("crates/tsuzuri-boundary/Cargo.toml"))
                .expect("境界の crate の Cargo.toml");
        for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
            let stray: Vec<String> = section_keys(&manifest, section)
                .into_iter()
                .filter(|k| !MEMBERS.contains(&k.as_str()))
                .collect();
            assert!(stray.is_empty(), "[{section}] に外の依存: {stray:?}");
        }
        assert!(
            !manifest.contains("[target."),
            "target ごとの依存の節を持たない"
        );
        assert_eq!(
            section_keys(&manifest, "dependencies"),
            vec!["tsuzuri-contract".to_string()]
        );
        assert_eq!(
            string_value(&manifest, "[bin]", "name").as_deref(),
            Some("tz")
        );
    }
}
