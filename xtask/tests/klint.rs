//! 行 k-lint-base の歯: 規則の行 R-10 の書き方を workspace の lint の表で deny にする機構（条 P-24.1）と、
//! 規則の行 R-4 の関数の粒度の値を写した clippy.toml と、member の manifest が表を継ぐこと（持ち込んだ folio は除外の表が外す）。
//! 外の依存を使わず、repo の根（xtask の manifest の dir の 1 つ上）からの相対の path で字を読む。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::PathBuf;

/// 表に置いてよい 18 の lint（lint・表・level・拠りの語）。拠りの語は R-10 の 13 は規則の行 R-10 の value の語、
/// 条 P-24.3 の 2 は P-24.3、R-4 の 3 は R-4。
const MAP: [(&str, &str, &str, &str); 18] = [
    ("unwrap_used", "clippy", "deny", "unwrap"),
    ("expect_used", "clippy", "deny", "expect"),
    ("panic", "clippy", "deny", "panic"),
    ("todo", "clippy", "deny", "todo"),
    ("unimplemented", "clippy", "deny", "unimplemented"),
    ("unreachable", "clippy", "deny", "unreachable"),
    ("exit", "clippy", "deny", "process の exit"),
    ("indexing_slicing", "clippy", "deny", "slice の添字"),
    ("dbg_macro", "clippy", "deny", "debug の macro（dbg）"),
    ("print_stdout", "clippy", "deny", "直接の stdout と stderr の print"),
    ("print_stderr", "clippy", "deny", "直接の stdout と stderr の print"),
    ("unused_must_use", "rust", "deny", "使わない must-use の値"),
    ("unsafe_code", "rust", "forbid", "unsafe（forbid）"),
    ("allow_attributes", "clippy", "deny", "P-24.3"),
    ("allow_attributes_without_reason", "clippy", "deny", "P-24.3"),
    ("too_many_lines", "clippy", "forbid", "R-4"),
    ("cognitive_complexity", "clippy", "forbid", "R-4"),
    ("too_many_arguments", "clippy", "forbid", "R-4"),
];

/// 今の表の lint（表に足す行がこの一覧も直す）。
const ENABLED: [&str; 7] = [
    "unused_must_use",
    "unwrap_used",
    "panic",
    "todo",
    "unimplemented",
    "exit",
    "dbg_macro",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask の dir の 1 つ上")
        .to_path_buf()
}

fn read(rel: &str) -> String {
    let path = repo_root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// trim した行が角括弧で始まるなら、最初の角括弧の中の字（表の見出し）。
fn heading(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix('[')?;
    let end = rest.find(']').expect("見出しの閉じの角括弧");
    Some(&rest[..end])
}

/// 井桁で始まる行と空の行を除いた行（trim）。
fn body(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
}

/// 見出し `name` の後から次の見出しの前までの、井桁で始まる行と空の行を除いた行（trim）。
fn section<'a>(text: &'a str, name: &str) -> Vec<&'a str> {
    let mut lines = text.lines();
    lines
        .by_ref()
        .find(|l| heading(l) == Some(name))
        .unwrap_or_else(|| panic!("見出し {name} が無い"));
    lines
        .take_while(|l| heading(l).is_none())
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

/// 等号の左の字と右の字（どちらも trim）。
fn key_value(line: &str) -> (&str, &str) {
    let (k, v) = line
        .split_once('=')
        .unwrap_or_else(|| panic!("等号の無い行: {line}"));
    (k.trim(), v.trim())
}

/// 二重引用符 2 つで囲んだ値の中の字。
fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .unwrap_or_else(|| panic!("引用符で囲まれない値: {value}"))
}

/// 根の Cargo.toml の members の dir（根からの相対）。
fn members() -> Vec<String> {
    let text = read("Cargo.toml");
    let mut lines = text.lines();
    lines
        .by_ref()
        .find(|l| l.trim() == "members = [")
        .expect("members の始まり");
    lines
        .take_while(|l| l.trim() != "]")
        .map(|l| l.trim().trim_end_matches(',').trim_matches('"').to_string())
        .collect()
}

/// 規則の file の行 `id` の value の字（二重引用符の中）。
fn rule_value(id: &str) -> String {
    let text = read("design-intent/rules.yaml");
    let mark = format!("{{id: {id},");
    let mut hits = text.lines().filter(|l| l.contains(&mark));
    let line = hits.next().unwrap_or_else(|| panic!("行 {id} が無い"));
    assert!(hits.next().is_none(), "行 {id} が 2 つ在る");
    let (_, rest) = line
        .split_once(" value: \"")
        .unwrap_or_else(|| panic!("行 {id} に value が無い"));
    let end = rest.find('"').expect("value の閉じの引用符");
    rest[..end].to_string()
}

/// R-4 の value の字の中で、語の後の空白の次の十進（字 , は飛ばす）。数の無い語と 2 度在る語は読まない。
fn r4_number(value: &str, word: &str) -> Option<u32> {
    if value.matches(word).count() != 1 {
        return None;
    }
    let (_, rest) = value.split_once(word)?;
    let rest = rest.strip_prefix(char::is_whitespace)?;
    let digits: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == ',')
        .filter(|c| *c != ',')
        .collect();
    digits.parse().ok()
}

/// 除外の表 carry-exclusions.toml の規則 R-10 の行の path（末尾の字 / を除く）。
fn excluded_r10() -> Vec<String> {
    let text = read("carry-exclusions.toml");
    let mut rows: Vec<(String, String)> = Vec::new();
    for line in body(&text) {
        if line == "[[exclusion]]" {
            rows.push((String::new(), String::new()));
            continue;
        }
        let (k, v) = key_value(line);
        let row = rows.last_mut().expect("表の行の前の鍵");
        match k {
            "rule" => row.0 = unquote(v).to_string(),
            "path" => row.1 = unquote(v).to_string(),
            _ => {}
        }
    }
    rows.into_iter()
        .filter(|(rule, _)| rule == "R-10")
        .map(|(_, path)| path.trim_end_matches('/').to_string())
        .collect()
}

#[test]
fn klint_workspace_table() {
    let names: BTreeSet<&str> = MAP.iter().map(|m| m.0).collect();
    assert_eq!(names.len(), MAP.len(), "MAP の lint が重なる");
    let enabled: BTreeSet<&str> = ENABLED.into_iter().collect();
    assert_eq!(enabled.len(), ENABLED.len(), "ENABLED の lint が重なる");

    let text = read("Cargo.toml");
    let heads: Vec<&str> = text
        .lines()
        .filter(|l| l.starts_with("[workspace.lints"))
        .collect();
    assert_eq!(heads, ["[workspace.lints.rust]", "[workspace.lints.clippy]"]);

    let mut want: Vec<String> = Vec::new();
    let mut have: Vec<String> = Vec::new();
    for table in ["rust", "clippy"] {
        for name in ENABLED {
            if let Some((lint, _, level, _)) = MAP.iter().find(|m| m.0 == name && m.1 == table) {
                want.push(format!("{table}.{lint} = {level}"));
            }
        }
        for line in section(&text, &format!("workspace.lints.{table}")) {
            let (k, v) = key_value(line);
            have.push(format!("{table}.{k} = {}", unquote(v)));
        }
    }
    for name in ENABLED {
        assert!(names.contains(name), "ENABLED の {name} が MAP に無い");
    }
    assert_eq!(have, want);
}

#[test]
fn klint_r10_words_match_rule() {
    let value = rule_value("R-10");
    let rule: Vec<&str> = value.split('・').collect();
    let words: BTreeSet<&str> = rule.iter().copied().collect();
    assert_eq!(rule.len(), 12, "{rule:?}");
    assert_eq!(words.len(), 12, "R-10 の語が重なる: {rule:?}");
    let map: BTreeSet<&str> = MAP
        .iter()
        .map(|m| m.3)
        .filter(|w| !w.starts_with("P-") && !w.starts_with("R-"))
        .collect();
    assert_eq!(map, words);
}

#[test]
fn klint_clippy_conf_copies_rule_r4() {
    let r4 = rule_value("R-4");
    let lines = r4_number(&r4, "関数").expect("R-4 の関数の行");
    let complexity = r4_number(&r4, "複雑度").expect("R-4 の複雑度");
    let mut want: Vec<(String, String)> = vec![
        ("too-many-lines-threshold".to_string(), lines.to_string()),
        (
            "cognitive-complexity-threshold".to_string(),
            complexity.to_string(),
        ),
    ];
    for key in [
        "allow-unwrap-in-tests",
        "allow-expect-in-tests",
        "allow-panic-in-tests",
        "allow-indexing-slicing-in-tests",
        "allow-print-in-tests",
    ] {
        want.push((key.to_string(), "true".to_string()));
    }
    let text = read("clippy.toml");
    let have: Vec<(String, String)> = body(&text)
        .map(|l| {
            let (k, v) = key_value(l);
            (k.to_string(), v.to_string())
        })
        .collect();
    assert_eq!(have, want);

    let root = repo_root();
    assert!(!root.join(".clippy.toml").exists(), "根に .clippy.toml が在る");
    let members = members();
    assert_eq!(members.len(), 6, "{members:?}");
    for member in &members {
        let mut dir = PathBuf::from(member);
        while !dir.as_os_str().is_empty() {
            for name in ["clippy.toml", ".clippy.toml"] {
                let path = root.join(&dir).join(name);
                assert!(!path.exists(), "{} が在る", path.display());
            }
            dir.pop();
        }
    }
}

#[test]
fn klint_r4_number_reads_value() {
    let sample = "字 1 module 1,500 行 以下・関数 60 行 以下と複雑度 15 以下と引数 5 以下";
    assert_eq!(r4_number(sample, "関数"), Some(60));
    assert_eq!(r4_number(sample, "複雑度"), Some(15));
    assert_eq!(r4_number(sample, "引数"), Some(5));
    assert_eq!(r4_number(sample, "module"), Some(1500));
    // 数の無い語・2 度在る語・空白の無い語・無い語は読まない。
    assert_eq!(r4_number("関数 行 以下", "関数"), None);
    assert_eq!(r4_number("関数 60 行・関数 70 行", "関数"), None);
    assert_eq!(r4_number("関数60行", "関数"), None);
    assert_eq!(r4_number(sample, "深さ"), None);

    let real = rule_value("R-4");
    assert_eq!(r4_number(&real, "関数"), Some(60));
    assert_eq!(r4_number(&real, "複雑度"), Some(15));
    assert_eq!(r4_number(&real, "引数"), Some(5));
    assert_eq!(r4_number(&real, "module"), Some(1500));
}

#[test]
fn klint_members_follow_carry_table() {
    let members = members();
    assert_eq!(members.len(), 6, "{members:?}");
    let excluded = excluded_r10();
    assert!(!excluded.is_empty(), "除外の表に R-10 の行が無い");
    for path in &excluded {
        assert!(members.contains(path), "{path} が members に無い");
    }
    for member in &members {
        let text = read(&format!("{member}/Cargo.toml"));
        let heads: Vec<&str> = text.lines().filter(|l| l.starts_with("[lints")).collect();
        if excluded.contains(member) {
            assert!(heads.is_empty(), "{member} が [lints を持つ: {heads:?}");
        } else {
            assert_eq!(heads, ["[lints]"], "{member}");
            assert_eq!(section(&text, "lints"), ["workspace = true"], "{member}");
        }
    }
}

/// member の tests/ の直下の .rs と tests/<dir>/main.rs（path の順）。
fn test_roots(member: &str) -> Vec<PathBuf> {
    let dir = repo_root().join(member).join("tests");
    let mut roots = Vec::new();
    let entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{} を読む: {e}", dir.display()));
    for entry in entries {
        let path = entry.expect("tests の項目").path();
        if path.is_dir() {
            let main = path.join("main.rs");
            if main.is_file() {
                roots.push(main);
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            roots.push(path);
        }
    }
    roots.sort();
    roots
}

#[test]
fn klint_test_roots_cfg_test() {
    let members = members();
    let excluded = excluded_r10();
    let mut seen = 0;
    for member in &members {
        if excluded.contains(member) {
            continue;
        }
        for path in test_roots(member) {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
            let lines: Vec<&str> = text.lines().collect();
            let head = lines.iter().take_while(|l| l.starts_with("//!")).count();
            assert_eq!(
                lines.get(head).copied(),
                Some("#![cfg(test)]"),
                "{} の頭の //! の続きの直後が #![cfg(test)] でない",
                path.display()
            );
            let count = lines.iter().filter(|l| l.trim() == "#![cfg(test)]").count();
            assert_eq!(count, 1, "{} の #![cfg(test)] が {count} 行", path.display());
            seen += 1;
        }
    }
    assert!(seen >= 1, "見た file が無い");
}

#[test]
fn klint_own_names_clean() {
    let text = read("xtask/tests/klint.rs");
    let mut names = Vec::new();
    let mut rest = text.as_str();
    // 属性の行と次の行の `fn `（この字の literal は逆斜線の escape で改行を持たず当たらない）。
    let mark = "\n#[test]\nfn ";
    while let Some(i) = rest.find(mark) {
        let after = &rest[i + mark.len()..];
        let end = after.find('(').expect("fn の名の終わり");
        names.push(after[..end].trim().to_string());
        rest = &after[end..];
    }
    assert_eq!(names.len(), 7, "{names:?}");

    let list = read("xtask/tests/filter-words.txt");
    let words: Vec<&str> = body(&list).collect();
    let unique: BTreeSet<&str> = words.iter().copied().collect();
    assert_eq!(words.len(), 325);
    assert_eq!(unique.len(), 325, "filter の語が重なる");
    for name in &names {
        let rest = name
            .strip_prefix("klint_")
            .unwrap_or_else(|| panic!("{name} が klint_ で始まらない"));
        for w in &words {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
