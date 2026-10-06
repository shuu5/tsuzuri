//! 行 v-retire の歯: 持ち込んだ器 scribe2 の中で tsuzuri の作業中に働く 2 つの file を止めたこと（判断の記録 ADR-33 の決定 (9)・(10)）。
//! 台帳の置き場の印 scribe2/.beads の 5 本は退役の置き場 scribe2/retired/beads/ へ移り、AI への指示の file scribe2/CLAUDE.md は
//! 元の path に残って、根の Claude Code の設定 .claude/settings.json の claudeMdExcludes が読ませない。外の依存を使わず repo の根からの相対の path で読む。
//! 行 t-prime-own の歯（名の頭 vretb_prime_）: 根の .beads/PRIME.md が bd prime の既定の手引きを差し替える。
#![cfg(test)]

use crate::common::root;
use std::collections::BTreeSet;

/// 移した 5 つの対（元・先）。名が字 . で始まる .gitignore は退役の置き場で働かないよう名を替える（folio2 の前例 folio2/retired/beads/gitignore.txt）。
const MOVED: [(&str, &str); 5] = [
    (
        "scribe2/.beads/.gitignore",
        "scribe2/retired/beads/gitignore.txt",
    ),
    ("scribe2/.beads/PRIME.md", "scribe2/retired/beads/PRIME.md"),
    (
        "scribe2/.beads/README.md",
        "scribe2/retired/beads/README.md",
    ),
    (
        "scribe2/.beads/config.yaml",
        "scribe2/retired/beads/config.yaml",
    ),
    (
        "scribe2/.beads/metadata.json",
        "scribe2/retired/beads/metadata.json",
    ),
];
const BEADS: &str = "scribe2/.beads";
const RETIRED: &str = "scribe2/retired";
/// 器の検査（器の xtask の check）が読む指示の file（動かさない）。
const CLAUDE_MD: &str = "scribe2/CLAUDE.md";
/// 根の Claude Code の設定と、読ませない CLAUDE.md の glob（絶対の path と照らすので頭は字 **/）。
const SETTINGS: &str = ".claude/settings.json";
const KEY: &str = "\"claudeMdExcludes\": [";
const PATTERN: &str = "**/scribe2/CLAUDE.md";
/// 器の検査の門 paths-clean と private-clean が見る private な path の形（器の limits.rs の PRIVATE_PATH_MARKS と同じ字）。
const MARKS: [&str; 2] = [concat!("/", "home", "/"), concat!("~", "/")];

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// dir（根からの相対）の下の file の path（根からの相対・名の順・下の dir も辿る）。
fn files(dir: &str, out: &mut BTreeSet<String>) {
    let Ok(entries) = std::fs::read_dir(root().join(dir)) else {
        return;
    };
    for entry in entries.flatten() {
        let rel = format!("{dir}/{}", entry.file_name().to_string_lossy());
        if entry.path().is_dir() {
            files(&rel, out);
        } else {
            out.insert(rel);
        }
    }
}

/// 設定の字の claudeMdExcludes の項（鍵の行から字 ] の行までの、二重引用符で囲んだ字）。鍵が無ければ None。
fn excludes(settings: &str) -> Option<Vec<String>> {
    let at = settings.find(KEY)?;
    let rest = &settings[at + KEY.len()..];
    let body = &rest[..rest.find(']')?];
    Some(
        body.split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    )
}

/// 項の判じ: 鍵が在り、項は 1 つで、字 **/ で始まり字 /scribe2/CLAUDE.md で終わる。
fn judge(settings: &str) -> Result<(), String> {
    let Some(list) = excludes(settings) else {
        return Err(format!("{SETTINGS} に鍵 claudeMdExcludes が無い"));
    };
    match list.as_slice() {
        [one] if one.starts_with("**/") && one.ends_with("/scribe2/CLAUDE.md") => Ok(()),
        _ => Err(format!(
            "claudeMdExcludes の項が {PATTERN} の 1 つでない: {list:?}"
        )),
    }
}

#[test]
fn vretb_beads_moved_to_retired() {
    for (from, to) in MOVED {
        assert!(!root().join(from).exists(), "{from} が元の path に在る");
        assert!(root().join(to).is_file(), "{to} が file として無い");
    }
    assert!(!root().join(BEADS).exists(), "{BEADS} が在る");
    let mut all = BTreeSet::new();
    files(RETIRED, &mut all);
    let want: BTreeSet<String> = MOVED.iter().map(|(_, to)| to.to_string()).collect();
    assert_eq!(all, want, "{RETIRED} の下の file は移した 5 本だけ");
}

#[test]
fn vretb_retired_has_no_private_marks() {
    for (_, to) in MOVED {
        let text = read(to);
        for mark in MARKS {
            assert!(!text.contains(mark), "{to} が private な path の形を持つ");
        }
    }
    let config = read("scribe2/retired/beads/config.yaml");
    let home: Vec<&str> = config.lines().filter(|l| l.contains("$HOME/")).collect();
    assert_eq!(home.len(), 2, "字 $HOME/ に替えた注の行: {home:?}");
    assert!(
        home.iter().all(|l| l.trim_start().starts_with('#')),
        "替えた行は注の行だけ: {home:?}"
    );
}

#[test]
fn vretb_claude_md_excluded() {
    assert!(
        root().join(CLAUDE_MD).is_file(),
        "{CLAUDE_MD} は元の path に残る"
    );
    let settings = read(SETTINGS);
    judge(&settings).expect("根の設定の claudeMdExcludes");
    assert_eq!(excludes(&settings), Some(vec![PATTERN.to_string()]));
    assert_eq!(
        settings.matches("claudeMdExcludes").count(),
        1,
        "鍵は 1 度だけ"
    );
    let good = format!("{{\n  {KEY}\n    \"{PATTERN}\"\n  ],\n  \"hooks\": {{}}\n}}\n");
    judge(&good).expect("良い見本");
    for (label, text) in [
        (
            "鍵の無い設定",
            good.replace("claudeMdExcludes", "claudeMdIncludes"),
        ),
        (
            "頭の字 **/ の無い項",
            good.replace(PATTERN, "a/scribe2/CLAUDE.md"),
        ),
        (
            "ほかの dir の CLAUDE.md の項",
            good.replace(PATTERN, "**/folio2/CLAUDE.md"),
        ),
        (
            "項が 2 つ",
            good.replace(
                &format!("\"{PATTERN}\""),
                &format!("\"{PATTERN}\",\n    \"**/a/CLAUDE.md\""),
            ),
        ),
    ] {
        assert!(judge(&text).is_err(), "{label} を通す");
    }
}

#[test]
fn vretb_prime_replaces_the_default_guide() {
    let rel = ".beads/PRIME.md";
    assert!(root().join(rel).is_file(), "{rel} が file として無い");
    let text = read(rel);
    assert!(text.lines().count() <= 8, "{rel} は 8 行以下");
    for want in ["bd --readonly", "bdw", "in_progress"] {
        assert!(text.contains(want), "{rel} に字 {want} が無い");
    }
    for command in ["bd create", "bd close", "bd remember", "bd dolt push"] {
        assert!(!text.contains(command), "{rel} が命令の字 {command} を持つ");
    }
    for mark in MARKS {
        assert!(!text.contains(mark), "{rel} が private な path の形を持つ");
    }
}
