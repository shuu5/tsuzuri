//! 根の crate の src の module の頭の doc に行の id を書かない決まり（判断の記録 ADR-63 の決定 (2) の (b)）の lint の歯（接頭辞 mdid_）:
//! crates/ の直下の crate ごとに、src の下の .rs の内側の doc の行（字 //! で始まる行・前の空白は除く）のうち、
//! 行の id の字を持つ行を数え、crate ごとの上限の定数を越えたら落ちる。行の id の正本は契約と commit が持つ。
//! 行の id の字は、正規表現で書くと次の 2 つのどちらか（id の後と bead の id の前の字の照らしは、英数字・_・.・-・/ の続きの中の字を拾わないため）。
//! 行か便の字の後に空白 0〜1 つで `(?:[a-z][a-z0-9]*(?:-[a-z0-9]+)+|[a-z])(?![A-Za-z0-9_.-])`、
//! または台帳の bead の id `(?<![A-Za-z0-9_./-])[a-z][a-z0-9]*-[a-z0-9]+(?:\.[0-9]+)+`。
//! 外の依存を使わず、repo の根（xtask の manifest の dir の 1 つ上）からの相対の path で字を読む。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// 境界の crate の上限（行の id を外す後の行が下げ、最後に 0 にする）。
const CAP_BOUNDARY: usize = 232;

/// 契約の crate の上限。
const CAP_CONTRACT: usize = 0;

/// 中核の crate の上限。
const CAP_CORE: usize = 0;

/// 面の crate の上限（行の id を外す後の行が下げ、最後に 0 にする）。
const CAP_SURFACE: usize = 199;

/// crates/ の直下の src を持つ dir の名と上限（名の順・crate を足す行はこの表も足す）。
const CAPS: [(&str, usize); 4] = [
    ("tsuzuri-boundary", CAP_BOUNDARY),
    ("tsuzuri-contract", CAP_CONTRACT),
    ("tsuzuri-core", CAP_CORE),
    ("tsuzuri-surface", CAP_SURFACE),
];

fn repo_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map_or_else(|| manifest.join(".."), Path::to_path_buf)
}

/// 行の id の中の字（英小文字と数字）。
fn id_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit()
}

/// id の後に続けば id でなくなる字（英数字・_・.・-）。
fn word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-')
}

/// 字の列 `run`（英数字・_・.・- の続きの全部）が、1 字の英小文字か、- で繋いだ 2 つ以上の部分
/// （どの部分も英小文字と数字の 1 字以上・最初の部分は英小文字で始まる）か。
fn row_id_shape(run: &[char]) -> bool {
    let text: String = run.iter().collect();
    let parts: Vec<&str> = text.split('-').collect();
    let lower_head = run.first().is_some_and(char::is_ascii_lowercase);
    let single = run.len() == 1 && lower_head;
    let joined = parts.len() >= 2
        && lower_head
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(id_char));
    single || joined
}

/// 位置 i の字が行か便で、その後（空白 0〜1 つ）に行の id の形の字の続きが在るか。
fn row_at(chars: &[char], i: usize) -> bool {
    if !matches!(chars.get(i), Some('行' | '便')) {
        return false;
    }
    let start = if chars.get(i + 1) == Some(&' ') {
        i + 2
    } else {
        i + 1
    };
    let rest = chars.get(start..).unwrap_or_default();
    let len = rest.iter().take_while(|c| word_char(**c)).count();
    row_id_shape(rest.get(..len).unwrap_or_default())
}

/// 位置 i から、i から先の英小文字と数字の続きの長さ。
fn id_run(chars: &[char], i: usize) -> usize {
    chars
        .get(i..)
        .unwrap_or_default()
        .iter()
        .take_while(|c| id_char(**c))
        .count()
}

/// 位置 i から台帳の bead の id（英小文字で始まる字と数字・-・字と数字・. と数字の 1 つ以上）が始まるか。
fn bead_at(chars: &[char], i: usize) -> bool {
    let before_ok = i == 0
        || chars
            .get(i - 1)
            .is_some_and(|c| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-')));
    if !before_ok || !chars.get(i).is_some_and(char::is_ascii_lowercase) {
        return false;
    }
    let hyphen = i + id_run(chars, i);
    if chars.get(hyphen) != Some(&'-') {
        return false;
    }
    let tail = id_run(chars, hyphen + 1);
    let dot = hyphen + 1 + tail;
    tail >= 1
        && chars.get(dot) == Some(&'.')
        && chars.get(dot + 1).is_some_and(char::is_ascii_digit)
}

/// 1 行が行の id の字を持つか。
fn has_id(line: &str) -> bool {
    let chars: Vec<char> = line.chars().collect();
    (0..chars.len()).any(|i| row_at(&chars, i) || bead_at(&chars, i))
}

/// 字の中の内側の doc の行のうち、行の id の字を持つ行の番号（1 から）。
fn id_lines(text: &str) -> Vec<usize> {
    text.lines()
        .enumerate()
        .filter(|(_, l)| l.trim_start().starts_with("//!") && has_id(l))
        .map(|(i, _)| i + 1)
        .collect()
}

/// dir の下の字 .rs で終わる file（path の順・下の dir も）。
fn rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let entries =
        std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{} を読む: {e}", dir.display()));
    for entry in entries {
        let path = entry.expect("dir の項目").path();
        if path.is_dir() {
            out.extend(rs_files(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out.sort();
    out
}

/// 根の下の crates/ の直下の src を持つ dir ごとに、行の id の字を持つ行（根からの path と行の番号の字）。
fn count_tree(root: &Path) -> BTreeMap<String, Vec<String>> {
    let crates = root.join("crates");
    let entries =
        std::fs::read_dir(&crates).unwrap_or_else(|e| panic!("{} を読む: {e}", crates.display()));
    let mut out = BTreeMap::new();
    for entry in entries {
        let dir = entry.expect("dir の項目").path();
        let src = dir.join("src");
        if !src.is_dir() {
            continue;
        }
        let mut hits = Vec::new();
        for path in rs_files(&src) {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string();
            hits.extend(id_lines(&text).into_iter().map(|n| format!("{rel}:{n}")));
        }
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        out.insert(name, hits);
    }
    out
}

/// toy の木（名に pid と字 tag）に、根からの相対の path と字の組を置く。
fn toy(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("tsuzuri-mdid-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for (rel, body) in files {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().expect("親の dir")).expect(rel);
        std::fs::write(&path, body).expect(rel);
    }
    root
}

#[test]
fn mdid_root_src_heads_stay_under_caps() {
    let found = count_tree(&repo_root());
    let names: Vec<&str> = found.keys().map(String::as_str).collect();
    let table: Vec<&str> = CAPS.iter().map(|(n, _)| *n).collect();
    assert_eq!(
        names, table,
        "crates/ の直下の src を持つ dir と表 CAPS の名が違う"
    );
    let counts: Vec<String> = CAPS
        .iter()
        .map(|(name, cap)| format!("{name} {} 行（上限 {cap}）", found[*name].len()))
        .collect();
    let over: Vec<&String> = CAPS
        .iter()
        .filter(|(name, cap)| found[*name].len() > *cap)
        .flat_map(|(name, _)| found[*name].iter().take(20))
        .collect();
    assert!(
        CAPS.iter().all(|(name, cap)| found[*name].len() <= *cap),
        "src の内側の doc に行の id の字の行が上限を越えた: {counts:?} {over:?}"
    );
}

#[test]
fn mdid_id_forms_count_and_one_clause_off_does_not() {
    // 正しい見本（数える）と、測る句を 1 つだけ外した見本（数えない）の対。
    let pairs = [
        (
            "//! 札を出す（行 g-pipe-cards）。",
            "//! 札を出す（g-pipe-cards）。",
        ),
        (
            "//! 札を出す（行g-pipe-cards）。",
            "//! 札を出す（行  g-pipe-cards）。",
        ),
        (
            "//! 板の 3 段は便 c が置く。",
            "//! 板の 3 段は便 cd が置く。",
        ),
        (
            "//! 板の 3 段は便 c が置く。",
            "//! 板の 3 段は便 C が置く。",
        ),
        (
            "//! 札を出す（行 g-pipe-cards）。",
            "//! 札を出す（行 g-pipe-cards.rs）。",
        ),
        (
            "//! 札を出す（行 g-pipe-cards）。",
            "//! 札を出す（行 G-pipe-cards）。",
        ),
        (
            "//! 札を出す（行 g-pipe-cards）。",
            "//! 札を出す（行 g--cards）。",
        ),
        ("//! 裁定 t3-hub.52.29 の案。", "//! 裁定 t3-hub の案。"),
        (
            "//! 裁定 t3-hub.52.29 の案。",
            "//! 裁定 x/t3-hub.52.29 の案。",
        ),
        ("//! 裁定 s2-07l.738 の案。", "//! 裁定 s2-07l.x738 の案。"),
    ];
    for (yes, no) in pairs {
        assert_eq!(id_lines(yes), vec![1], "数えない: {yes}");
        assert_eq!(id_lines(no), Vec::<usize>::new(), "数える: {no}");
    }
}

#[test]
fn mdid_only_inner_doc_lines_count() {
    let id = "行 g-pipe-cards";
    let text = format!(
        "//! 頭の 1 行。\n//! 札（{id}）。\n\n/// 札（{id}）。\n// 札（{id}）。\nfn f() {{}}\n\nmod m {{\n    //! 内の module（{id}）。\n}}\nconst S: &str = \"{id}\";\n"
    );
    assert_eq!(id_lines(&text), vec![2, 9]);
    let moved = text.replacen("//! 札", "/// 札", 1);
    assert_eq!(id_lines(&moved), vec![9], "頭の行を /// にしても数えた");
    let inner = text.replacen("    //! 内", "    // 内", 1);
    assert_eq!(
        id_lines(&inner),
        vec![2],
        "内の module の行を // にしても数えた"
    );
}

#[test]
fn mdid_only_root_crate_src_files_count() {
    let line = "//! 札（行 g-pipe-cards）。\n";
    let places = [
        "crates/a/src/lib.rs",
        "crates/a/src/x/deep.rs",
        "crates/a/tests/t.rs",
        "crates/a/build.rs",
        "crates/b/src/x.txt",
        "folio2/crates/f/src/lib.rs",
        "scribe2/crates/s/src/lib.rs",
        "xtask/src/main.rs",
    ];
    let files: Vec<(&str, &str)> = places.iter().map(|p| (*p, line)).collect();
    let root = toy("places", &files);
    let found = count_tree(&root);
    let _ = std::fs::remove_dir_all(&root);
    let want: BTreeMap<String, Vec<String>> = [
        (
            "a".to_string(),
            vec![
                "crates/a/src/lib.rs:1".to_string(),
                "crates/a/src/x/deep.rs:1".to_string(),
            ],
        ),
        ("b".to_string(), Vec::new()),
    ]
    .into_iter()
    .collect();
    assert_eq!(found, want);
}
