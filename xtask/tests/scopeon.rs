//! 行 t-scope-on と t-scope-xtask の歯（接頭辞 scopeon_・設計ノート surface-wave29c と surface-wave29d・判断の記録 ADR-64 の決定 (5)）: 根の器の宣言
//! .vessel.toml の任意 key scope-paths が、設計の 3 つの dir（design-intent/・contracts/・docs/）の外で器の固定の面と crate-roots に入らない根の
//! 追跡する項目を全部持つ 1 行で、各項目が repo の追跡する path を名指すこと（器の追随の再 gate・着地の後の検出線・差の当たりの面が、共通 verify
//! の組みと歯と lint の入力の全部を読む）。器は宣言の項目が repo に在るかも、面の漏れも知れないので、名指しの外れと漏れはこの歯が見る。
//! 否定の見本は正しい見本から 1 句だけ替える。外の依存を使わない。
#![cfg(test)]

use std::path::PathBuf;
use std::process::Command;

/// 名乗りの 1 行の字（key と値の間は空白 1 つずつ・項目は byte の順）。
const SCOPE: &str = "scope-paths = [\".agents/\", \".beads/\", \".claude/\", \".codex/\", \".config/\", \".github/\", \".gitignore\", \".vessel\", \"LICENSE-APACHE\", \"LICENSE-MIT\", \"README.md\", \"carry-exclusions.toml\", \"clippy.toml\", \"deny.toml\", \"folio2/\", \"plugin/\", \"rust-toolchain.toml\", \"scribe2/\", \"tests/\", \"xtask/\"]";

/// 面に入れない設計の 3 つの dir（席の設計の commit が書く所・着地の後の main の確かめが読む）。
const DESIGN: [&str; 3] = ["contracts/", "design-intent/", "docs/"];

/// 器の固定の面の写し（器の land.rs の DETECTION_SCOPE と同じ字・歯 scopeon_fixed_face_copy_matches_the_vessel が照らす）。
const FIXED: [&str; 5] = [
    "crates/",
    "Cargo.toml",
    "Cargo.lock",
    "rules/",
    ".vessel.toml",
];

/// repo の根（xtask の manifest の dir の 1 つ上）。
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// 注でない行のうち key が `key` の行（前後の空白を除いた字・file の順）。
fn key_lines<'a>(text: &'a str, key: &str) -> Vec<&'a str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter(|line| line.split('=').next().map(str::trim) == Some(key))
        .collect()
}

/// 注でない行のうち key が scope-paths の行。
fn scope_lines(text: &str) -> Vec<&str> {
    key_lines(text, "scope-paths")
}

/// 行の値の配列の項目（二重引用符の中の字・順のまま）。
fn items(line: &str) -> Vec<&str> {
    line.split_once('=')
        .map(|(_, value)| value.split('"').skip(1).step_by(2).collect())
        .unwrap_or_default()
}

/// path が面の 1 項目に触れるか（末尾 / の項目は dir の接頭辞・ほかは file の完全一致・器の in_face と同じ読み）。
fn in_face(path: &str, face: &str) -> bool {
    if face.ends_with('/') {
        path.starts_with(face)
    } else {
        path == face
    }
}

/// 項目のうち、追跡する path を名指さない物（末尾 / の項目はその下に追跡する file が無い・ほかは同じ字の追跡する file が無い）。
fn untracked<'a>(declared: &[&'a str], tracked: &[String]) -> Vec<&'a str> {
    declared
        .iter()
        .copied()
        .filter(|item| !tracked.iter().any(|path| in_face(path, item)))
        .collect()
}

/// 追跡する file のうち、設計の dir・固定の面・根（roots）・面の項目（scope）のどれにも入らない物（file の順）。
fn outside<'a>(tracked: &'a [String], roots: &[&str], scope: &[&str]) -> Vec<&'a str> {
    tracked
        .iter()
        .map(String::as_str)
        .filter(|path| {
            !DESIGN
                .iter()
                .chain(FIXED.iter())
                .chain(roots.iter())
                .chain(scope.iter())
                .any(|face| in_face(path, face))
        })
        .collect()
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

/// 根の .vessel.toml の字。
fn declaration() -> String {
    std::fs::read_to_string(repo_root().join(".vessel.toml")).expect("根の .vessel.toml を読む")
}

#[test]
fn scopeon_root_declaration_names_the_vessel_scope_paths() {
    let text = declaration();
    assert_eq!(scope_lines(&text), [SCOPE]);
    let declared = items(SCOPE);
    assert_eq!(declared.len(), 20, "{declared:?}");
    assert_eq!(declared.first(), Some(&".agents/"));
    assert_eq!(declared.last(), Some(&"xtask/"));
    for want in [
        "xtask/",
        "tests/",
        "plugin/",
        "folio2/",
        "scribe2/",
        ".config/",
        "rust-toolchain.toml",
    ] {
        assert!(declared.contains(&want), "{want}");
    }
}

#[test]
fn scopeon_declared_paths_are_tracked_in_the_repo() {
    let files = tracked();
    assert_eq!(untracked(&items(SCOPE), &files), Vec::<&str>::new());
    // file の項目の照らしの対照: 根の manifest は同じ字の追跡する file。
    assert_eq!(untracked(&["Cargo.toml"], &files), Vec::<&str>::new());
    // 否定の見本: 1 字外れた dir、dir の印の無い dir、字の続く隣の dir、1 字外れた file は、どれも名指しの外れ。
    let off = ["scribe/", "scribe2", "scribe2x/", "Cargo.tom"];
    for item in off {
        assert_eq!(untracked(&[item], &files), [item], "{item}");
    }
}

#[test]
fn scopeon_one_clause_changes_are_refused() {
    assert_eq!(scope_lines(&format!("schema = 1\n{SCOPE}\n")), [SCOPE]);
    let cases: [(String, Vec<&str>); 4] = [
        (
            SCOPE.replace("\"xtask/\"", "\"xtask\""),
            items(SCOPE)
                .into_iter()
                .map(|item| if item == "xtask/" { "xtask" } else { item })
                .collect(),
        ),
        (
            SCOPE.replace(", \"xtask/\"", ""),
            items(SCOPE)
                .into_iter()
                .filter(|item| *item != "xtask/")
                .collect(),
        ),
        (format!("# {SCOPE}"), Vec::new()),
        (SCOPE.replace("scope-paths", "scope_paths"), Vec::new()),
    ];
    for (line, want) in cases {
        let text = format!("schema = 1\n{line}\n");
        let lines = scope_lines(&text);
        let got = lines.first().map(|found| items(found)).unwrap_or_default();
        assert_eq!(got, want, "{line}");
        assert_ne!(lines, [SCOPE], "{line}");
    }
    // 同じ行を 2 度書いた宣言は 1 行と読まない。
    assert_eq!(
        scope_lines(&format!("schema = 1\n{SCOPE}\n{SCOPE}\n")),
        [SCOPE, SCOPE]
    );
}

#[test]
fn scopeon_tracked_paths_outside_the_design_dirs_are_all_in_the_face() {
    let text = declaration();
    let roots_line = key_lines(&text, "crate-roots");
    assert_eq!(
        roots_line.len(),
        1,
        "crate-roots の行は 1 本: {roots_line:?}"
    );
    let roots = roots_line
        .first()
        .map(|line| items(line))
        .unwrap_or_default();
    let declared = scope_lines(&text)
        .first()
        .map(|line| items(line))
        .unwrap_or_default();
    let files = tracked();
    assert_eq!(outside(&files, &roots, &declared), Vec::<&str>::new());
    // 足す物の在る見本: 設計の dir と面の中の file は数えず、上の段の新しい file と新しい dir の file だけを名指す。
    let sample: Vec<String> = [
        "docs/a.md",
        "xtask/src/x.rs",
        "folio2/vendor/a.js",
        "new.toml",
        "newdir/a.rs",
        "crates/a/src/lib.rs",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    assert_eq!(
        outside(&sample, &roots, &declared),
        ["new.toml", "newdir/a.rs"]
    );
    // 否定の見本: 宣言から xtask/ の 1 項目だけを外すと、xtask の追跡する file が漏れに出る。
    let without: Vec<&str> = declared
        .iter()
        .copied()
        .filter(|item| *item != "xtask/")
        .collect();
    let leaked = outside(&files, &roots, &without);
    assert!(
        !leaked.is_empty() && leaked.iter().all(|path| path.starts_with("xtask/")),
        "{leaked:?}"
    );
    assert!(leaked.contains(&"xtask/Cargo.toml"), "{leaked:?}");
}

#[test]
fn scopeon_fixed_face_copy_matches_the_vessel() {
    let land = std::fs::read_to_string(repo_root().join("scribe2/crates/scribe2/src/pipe/land.rs"))
        .expect("器の land.rs を読む");
    let lines: Vec<&str> = land
        .lines()
        .filter(|line| line.starts_with("pub const DETECTION_SCOPE: &[&str] = &["))
        .collect();
    assert_eq!(lines.len(), 1, "{lines:?}");
    let found: Vec<&str> = lines
        .first()
        .map(|line| line.split('"').skip(1).step_by(2).collect())
        .unwrap_or_default();
    assert_eq!(found, FIXED);
}
