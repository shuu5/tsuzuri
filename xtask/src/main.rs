//! xtask: `cargo run -q -p xtask -- <task>` の形で起動する（alias の file は置かない）。
//! check は workspace の build・歯の全部・clippy（host と面の wasm）・面の組み立てを順に撃ち、最初に落ちた段の rc を返す。
//! surface-build は面の crate の dir で trunk を呼び、dist に index.html と wasm の file を出す（便 g-min）。

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
    // 面の DOM と通信（board と net）は wasm の target のときだけ組み立てるので、host の clippy に載らない分をここで見る。
    &[
        "clippy",
        "-p",
        "tsuzuri-surface",
        "--target",
        WASM_TARGET,
        "--",
        "-D",
        "warnings",
    ],
];

/// 面の crate の組み立て先（rust-toolchain.toml の targets と同じ）。
const WASM_TARGET: &str = "wasm32-unknown-unknown";

/// 面の crate の dir（workspace の root から）。trunk はここの Trunk.toml と index.html を読む。
const SURFACE_DIR: &str = "crates/tsuzuri-surface";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("check") if args.len() == 1 => exit_code(check(&workspace_root())),
        Some("surface-build") if args.len() == 1 => exit_code(surface_build(&workspace_root())),
        _ => {
            eprintln!("usage: cargo run -q -p xtask -- <check|surface-build>");
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
    eprintln!("xtask check: surface-build");
    let rc = surface_build(root);
    if rc != 0 {
        eprintln!("xtask check: 落ちた段 surface-build (rc {rc})");
    }
    rc
}

/// 面の crate の dir で `trunk build` を撃ち（設定は Trunk.toml）、dist に index.html と wasm の file が在るかを見る。
fn surface_build(root: &Path) -> i32 {
    let dir = root.join(SURFACE_DIR);
    eprintln!("xtask surface-build: trunk build ({SURFACE_DIR})");
    let rc = match Command::new("trunk")
        .arg("build")
        .current_dir(&dir)
        .status()
    {
        Ok(s) if s.success() => 0,
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("xtask surface-build: trunk を起動できない: {e}");
            1
        }
    };
    if rc != 0 {
        return rc;
    }
    let dist = dir.join("dist");
    match dist_missing(&dist) {
        None => 0,
        Some(what) => {
            eprintln!("xtask surface-build: {} に {what} が無い", dist.display());
            1
        }
    }
}

/// dist に足りない物（index.html か wasm の file）の名。揃っていれば None。
fn dist_missing(dist: &Path) -> Option<&'static str> {
    if !dist.join("index.html").is_file() {
        return Some("index.html");
    }
    let has_wasm = std::fs::read_dir(dist).is_ok_and(|entries| {
        entries
            .flatten()
            .any(|e| e.path().extension().is_some_and(|x| x == "wasm") && e.path().is_file())
    });
    (!has_wasm).then_some("wasm の file")
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

    /// 依存の節か（`dependencies`・`dev-dependencies`・`build-dependencies` と、その target ごとの節）。
    fn is_dependency_table(table: &str) -> bool {
        const KINDS: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];
        KINDS.contains(&table)
            || (table.starts_with("target.")
                && KINDS.iter().any(|k| table.ends_with(&format!(".{k}"))))
    }

    /// manifest の直接依存の名（どの依存の節か・target ごとの節も含めて集める・重複なし・名の順）。
    /// `name.workspace = true` の形は `name`、`[dependencies.name]` の節は `name` と読む。
    pub(super) fn dependency_names(text: &str) -> Vec<String> {
        let mut inside = false;
        let mut names = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if let Some(table) = line.strip_prefix('[') {
                let table = table.trim_start_matches('[').trim_end_matches(']').trim();
                inside = is_dependency_table(table);
                if !inside
                    && let Some((head, name)) = table.rsplit_once('.')
                    && is_dependency_table(head)
                {
                    names.push(name.trim().trim_matches('"').to_string());
                }
            } else if inside
                && !line.starts_with('#')
                && let Some((k, _)) = line.split_once('=')
            {
                let key = k.trim().trim_matches('"');
                names.push(key.split('.').next().unwrap_or(key).to_string());
            }
        }
        names.sort();
        names.dedup();
        names
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
    use super::read::{
        dependency_names, lock_packages, member_names, section_keys, string_array, string_value,
    };
    use super::{SURFACE_DIR, workspace_root};

    const MEMBERS: [&str; 5] = [
        "tsuzuri-contract",
        "tsuzuri-core",
        "tsuzuri-boundary",
        "tsuzuri-surface",
        "xtask",
    ];

    /// crate ごとの直接依存の名の一覧（member の dir と、依存の全部の節（target ごとの節も）の名・名の順）。
    /// 外の部品は便ごとに足した名だけ: serde（便 a）・serde_json（便 b・中核は便 c）・Leptos の一式（便 g-min・規則の行 R-25）。
    /// 名を足す便はこの一覧を直す。一覧に無い名が manifest に在れば落ちる。
    const DIRECT_DEPS: [(&str, &[&str]); 5] = [
        ("crates/tsuzuri-contract", &["serde", "serde_json"]),
        (
            "crates/tsuzuri-core",
            &["serde", "serde_json", "tsuzuri-contract"],
        ),
        ("crates/tsuzuri-boundary", &["tsuzuri-contract"]),
        (
            SURFACE_DIR,
            &[
                "leptos",
                "tsuzuri-boundary",
                "tsuzuri-contract",
                "wasm-bindgen-futures",
                "web-sys",
            ],
        ),
        ("xtask", &[]),
    ];

    /// 面の crate の直接依存の上限（規則の行 R-25 の値・要件 NFR3）。rules の file の行 R-25 の字と照らす。
    const SURFACE_DIRECT_MAX: usize = 8;

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

    /// 各 member の manifest の直接依存の名が一覧と同じ（面の crate の外の crate の名は増えない）。
    #[test]
    fn skeleton_direct_deps_match_list() {
        let root = workspace_root();
        let mut dirs: Vec<&str> = DIRECT_DEPS.iter().map(|(dir, _)| *dir).collect();
        dirs.sort_unstable();
        let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("Cargo.toml");
        let mut members = string_array(&manifest, "members").expect("workspace の members");
        members.sort();
        assert_eq!(members, dirs, "一覧は workspace の member の全部を持つ");
        for (dir, want) in DIRECT_DEPS {
            let text = std::fs::read_to_string(root.join(dir).join("Cargo.toml"))
                .unwrap_or_else(|e| panic!("{dir}/Cargo.toml を読む: {e}"));
            assert_eq!(dependency_names(&text), want.to_vec(), "{dir} の直接依存");
        }
    }

    /// 面の crate の直接依存は規則の行 R-25 の上限（8 本）以下。
    #[test]
    fn skeleton_surface_direct_deps_within_budget() {
        let (_, surface) = DIRECT_DEPS
            .iter()
            .find(|(dir, _)| *dir == SURFACE_DIR)
            .expect("面の crate の行");
        assert!(
            surface.len() <= SURFACE_DIRECT_MAX,
            "面の crate の直接依存 {} 本が上限 {SURFACE_DIRECT_MAX} 本を越える: {surface:?}",
            surface.len()
        );
    }

    /// 歯の上限の定数が rules の file の行 R-25 の字（直接依存 N 本）と同じ。
    #[test]
    fn skeleton_surface_budget_matches_rule_r25() {
        let rules = std::fs::read_to_string(workspace_root().join("design-intent/rules.yaml"))
            .expect("design-intent/rules.yaml");
        let row = rules
            .lines()
            .find(|l| l.contains("id: R-25,"))
            .expect("rules の file に行 R-25 が在る");
        assert_eq!(rule_direct_max(row), Some(SURFACE_DIRECT_MAX), "{row}");
    }

    /// 行の value の字の「直接依存 N 本」の N（無いか 2 つ以上なら None）。
    fn rule_direct_max(row: &str) -> Option<usize> {
        let value = row.split_once("value: \"")?.1.split_once('"')?.0;
        let mut found = value.split("直接依存").skip(1).filter_map(|rest| {
            let n = rest.trim_start().split_once('本')?.0.trim();
            n.parse::<usize>().ok()
        });
        let n = found.next()?;
        found.next().is_none().then_some(n)
    }

    #[test]
    fn skeleton_rule_direct_max_reads_value() {
        let row = r#"  - {id: R-25, what: "x", value: "直接依存 8 本 以下（面の crate）・以後は 1 便 1 本", note: "直接依存 3 本"}"#;
        assert_eq!(rule_direct_max(row), Some(8));
        assert_eq!(rule_direct_max(&row.replace("8 本", "9 本")), Some(9));
        assert_eq!(rule_direct_max(&row.replace("8 本", "八本")), None);
        assert_eq!(
            rule_direct_max(&row.replace("8 本 以下", "8 本 以下・直接依存 7 本")),
            None
        );
    }

    #[test]
    fn skeleton_dependency_names_reads_every_table() {
        let text = r#"
[package]
name = "x"

[dependencies]
a = { path = "../a" }
b.workspace = true
# c = "1"

[target.'cfg(target_arch = "wasm32")'.dependencies]
d = { version = "0.3", features = ["E"] }

[dev-dependencies]
a = { path = "../a" }

[build-dependencies.e]
version = "1"

[[bin]]
name = "y"
"#;
        assert_eq!(dependency_names(text), vec!["a", "b", "d", "e"]);
    }

    /// Cargo.lock の workspace の中の package は member だけ。
    #[test]
    fn skeleton_lock_inside_is_members() {
        let lock =
            std::fs::read_to_string(workspace_root().join("Cargo.lock")).expect("Cargo.lock");
        let packages = lock_packages(&lock);
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
