//! 行 t-index-on の歯（接頭辞 ixon_・設計ノート surface-wave29d・器の設計 reverse-index.md の 4 節 形 1・器の判断の記録 ADR-0105）: 根の器の宣言
//! .vessel.toml が器の任意 key index-scip と index-roles をそれぞれ 1 行だけで名乗り、字が器の宣言 scribe2/.vessel.toml に倣い（規則の file の path の頭
//! scribe2/ だけが違う）、名指す規則の file が追跡されて役の 9 語の規則を順に持ち、index-scip の 1 本が scribe2/ の外の全部の manifest を覆い、
//! rust-toolchain.toml が component rust-analyzer を 1 度だけ名乗ること。注の行を除いて key の字の行を数える。否定の見本は正しい見本から 1 句だけ替える。
#![cfg(test)]

use std::path::PathBuf;
use std::process::Command;

/// 索引を作る command の名乗りの行の字（器の宣言と同じ字）。
const SCIP_ON: &str = "index-scip = [\"rust-analyzer scip {tree} --output {out}\"]";

/// 構文の役の一致を出す command の名乗りの行の字（器の宣言の規則の file の path に頭 scribe2/ を足した字）。
const ROLES_ON: &str =
    "index-roles = [\"ast-grep scan --rule scribe2/.config/index-roles.yml --json=stream {tree}\"]";

/// 名指す規則の file（repo の根からの相対）。
const RULES: &str = "scribe2/.config/index-roles.yml";

/// 役の 9 語（器の設計 reverse-index.md の 5 節の順）。
const ROLES: [&str; 9] = [
    "literal", "pattern", "call", "use", "reexport", "test", "doclink", "capture", "vis",
];

/// 器の宣言の規則の file の頭と、根の宣言の頭。
const VESSEL_RULE: &str = "--rule .config/";
const ROOT_RULE: &str = "--rule scribe2/.config/";

/// repo の根。
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// repo の根からの相対 path の file の字。
fn read(path: &str) -> String {
    std::fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|err| panic!("{path} を読む: {err}"))
}

/// repo の追跡する file（repo の根からの相対）。
fn tracked() -> Vec<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo_root())
        .args(["ls-files", "-z"])
        .output()
        .expect("git ls-files を撃つ");
    assert!(out.status.success(), "git ls-files の rc");
    String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect()
}

/// 注でない行のうち key が key の行（前後の空白を除いた字・file の順）。
fn key_lines<'a>(text: &'a str, key: &str) -> Vec<&'a str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter(|line| line.split('=').next().map(str::trim) == Some(key))
        .collect()
}

/// 根の 2 行が器の宣言の 2 key の行に倣うか（index-scip は同じ字・index-roles は規則の file の頭だけを scribe2/ に替えた字）。
fn follows(vessel: &str) -> bool {
    let roles: Vec<String> = key_lines(vessel, "index-roles")
        .iter()
        .map(|line| line.replace(VESSEL_RULE, ROOT_RULE))
        .collect();
    key_lines(vessel, "index-scip") == vec![SCIP_ON] && roles == vec![ROLES_ON.to_owned()]
}

/// 規則の file の頭の段の rule id（行頭の字 id: の後ろ・file の順）。
fn rule_ids(text: &str) -> Vec<&str> {
    text.lines()
        .filter_map(|line| line.strip_prefix("id: "))
        .map(str::trim)
        .collect()
}

/// 根の manifest の [workspace] の members の項目（file の順）。
fn members(text: &str) -> Vec<&str> {
    let after = text.split_once("members = [").map_or("", |(_, rest)| rest);
    let body = after.split_once(']').map_or("", |(body, _)| body);
    body.split(',')
        .map(|item| item.trim().trim_matches('"'))
        .filter(|item| !item.is_empty())
        .collect()
}

/// scribe2/ の外の追跡する manifest のうち、根の manifest でも members の dir でもない物の dir。
fn uncovered<'a>(manifests: &[&'a str], members: &[&str]) -> Vec<&'a str> {
    manifests
        .iter()
        .copied()
        .filter(|path| !path.starts_with("scribe2/"))
        .filter_map(|path| path.strip_suffix("/Cargo.toml"))
        .filter(|dir| !members.contains(dir))
        .collect()
}

/// rust-toolchain.toml の components の行に字 "rust-analyzer" が現れる数。
fn analyzer_count(text: &str) -> usize {
    key_lines(text, "components")
        .iter()
        .map(|line| line.matches("\"rust-analyzer\"").count())
        .sum()
}

#[test]
fn ixon_root_declaration_names_both_index_lines_once() {
    let text = read(".vessel.toml");
    assert_eq!(key_lines(&text, "index-scip"), vec![SCIP_ON]);
    assert_eq!(key_lines(&text, "index-roles"), vec![ROLES_ON]);
}

#[test]
fn ixon_lines_follow_the_vessel_declaration() {
    let vessel = read("scribe2/.vessel.toml");
    assert!(follows(&vessel), "根の 2 行は器の宣言に倣う");
    let cases = [
        (
            "scip の旗を 1 つ足す",
            vessel.replace(
                "--output {out}\"]",
                "--output {out} --exclude-vendored-libraries\"]",
            ),
        ),
        (
            "roles の出力の形を替える",
            vessel.replace("--json=stream", "--json=compact"),
        ),
        (
            "roles の規則の file の名を替える",
            vessel.replace("index-roles.yml", "roles.yml"),
        ),
        (
            "scip の行を外す",
            vessel.replace("index-scip = ", "# index-scip = "),
        ),
    ];
    for (label, text) in cases {
        assert_ne!(text, vessel, "{label}: 見本が替わっていない");
        assert!(!follows(&text), "{label} を通す");
    }
}

#[test]
fn ixon_rule_file_is_tracked_with_the_nine_roles() {
    assert!(
        tracked().iter().any(|path| path == RULES),
        "{RULES} は追跡される"
    );
    assert_eq!(rule_ids(&read(RULES)), ROLES.to_vec());
    let base = "id: literal\n---\nid: pattern\n";
    assert_eq!(rule_ids(base), vec!["literal", "pattern"]);
    assert_eq!(
        rule_ids("  id: literal\n---\nid: pattern\n"),
        vec!["pattern"]
    );
    assert_eq!(
        rule_ids("id: literal\n---\nid: literal\n"),
        vec!["literal", "literal"]
    );
}

#[test]
fn ixon_one_scip_line_covers_every_manifest_outside_the_vessel() {
    let paths = tracked();
    let manifests: Vec<&str> = paths
        .iter()
        .map(String::as_str)
        .filter(|path| path.ends_with("Cargo.toml"))
        .collect();
    let root = read("Cargo.toml");
    assert!(
        manifests.contains(&"Cargo.toml"),
        "根の manifest は追跡される"
    );
    assert_eq!(uncovered(&manifests, &members(&root)), Vec::<&str>::new());
    let fixture = [
        "Cargo.toml",
        "crates/a/Cargo.toml",
        "folio2/crates/folio/Cargo.toml",
        "scribe2/Cargo.toml",
        "scribe2/crates/x/Cargo.toml",
    ];
    let full = "[workspace]\nmembers = [\n    \"crates/a\",\n    \"folio2/crates/folio\",\n]\n";
    assert_eq!(uncovered(&fixture, &members(full)), Vec::<&str>::new());
    let short = "[workspace]\nmembers = [\n    \"crates/a\",\n]\n";
    assert_eq!(
        uncovered(&fixture, &members(short)),
        vec!["folio2/crates/folio"]
    );
    let near = "[workspace]\nmembers = [\n    \"crates/a\",\n    \"folio2/crates/foli\",\n]\n";
    assert_eq!(
        uncovered(&fixture, &members(near)),
        vec!["folio2/crates/folio"]
    );
}

#[test]
fn ixon_toolchain_names_rust_analyzer_once() {
    assert_eq!(analyzer_count(&read("rust-toolchain.toml")), 1);
    assert_eq!(
        analyzer_count("[toolchain]\ncomponents = [\"clippy\", \"rust-analyzer\"]\n"),
        1
    );
    let cases = [
        "[toolchain]\ncomponents = [\"clippy\", \"rustfmt\"]\n",
        "[toolchain]\n# components = [\"clippy\", \"rust-analyzer\"]\ncomponents = [\"clippy\"]\n",
        "[toolchain]\ncomponents = [\"clippy\", \"rust-analyzer-preview\"]\n",
        "[toolchain]\ncomponents = [\"rust-analyzer\", \"rust-analyzer\"]\n",
    ];
    for text in cases {
        assert_ne!(analyzer_count(text), 1, "{text}");
    }
}

#[test]
fn ixon_one_clause_changes_are_refused() {
    let good = format!("schema = 1\n{SCIP_ON}\n{ROLES_ON}\n");
    assert_eq!(key_lines(&good, "index-scip"), vec![SCIP_ON]);
    assert_eq!(key_lines(&good, "index-roles"), vec![ROLES_ON]);
    let cases = [
        ("index-scip", good.replace(" --output {out}", "")),
        ("index-scip", good.replace("scip {tree}", "scip .")),
        (
            "index-scip",
            good.replace("rust-analyzer scip", "rust-analyzr scip"),
        ),
        (
            "index-scip",
            good.replace("index-scip = ", "# index-scip = "),
        ),
        ("index-scip", good.replace("index-scip = ", "index_scip = ")),
        ("index-scip", format!("{good}{SCIP_ON}\n")),
        (
            "index-roles",
            good.replace("--rule scribe2/.config/", "--rule .config/"),
        ),
        ("index-roles", good.replace(" --json=stream", "")),
        ("index-roles", good.replace(" {tree}\"]", " .\"]")),
        (
            "index-roles",
            good.replace("index-roles = ", "# index-roles = "),
        ),
    ];
    for (key, text) in cases {
        let want = if key == "index-scip" {
            SCIP_ON
        } else {
            ROLES_ON
        };
        assert_ne!(key_lines(&text, key), vec![want], "{text}");
    }
}
