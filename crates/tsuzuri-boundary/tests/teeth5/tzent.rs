//! tz の口の folio 由来の 11 subcommand と索引の形の tz graph の歯（行 k-tz-entry・接頭辞 tzent_）。
//! tz の binary を子の処理で撃ち、folio の lib の入口 folio::entry::run の書いた字と照らす。
#![cfg(test)]

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::common::{FILTER_WORDS, root};
use tsuzuri_boundary::cli::folio::{GRAPH_FLAGS, INDEX_FLAGS, NAMES};

/// 本物の設計文書の置き場。
fn design() -> String {
    root().join("folio2/design-intent").display().to_string()
}

/// tz の binary を撃つ（終了 code・標準出力・標準エラー）。
fn tz(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(args)
        .current_dir(root())
        .output()
        .expect("tz を撃つ");
    (
        out.status.code().expect("終了 code"),
        String::from_utf8(out.stdout).expect("標準出力は UTF-8"),
        String::from_utf8(out.stderr).expect("標準エラーは UTF-8"),
    )
}

/// folio の lib の入口を撃つ（終了 code・標準出力・標準エラー）。
fn entry(args: &[&str]) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let rc = folio::entry::run(args.iter().copied(), &mut out, &mut err);
    (
        rc,
        String::from_utf8(out).expect("標準出力は UTF-8"),
        String::from_utf8(err).expect("標準エラーは UTF-8"),
    )
}

#[test]
fn tzent_names_are_the_eleven_mouths() {
    assert_eq!(
        NAMES,
        [
            "check", "schema", "derive", "ceiling", "init", "parts", "face", "figure", "build",
            "intake", "hello"
        ]
    );
    assert_eq!(INDEX_FLAGS, ["--print", "--digest", "--summary"]);
    for name in NAMES {
        let (rc, out, err) = tz(&[name, "--help"]);
        assert_eq!(rc, 0, "{name}: {err}");
        assert!(out.contains(&format!("Usage: tz {name} ")), "{name}: {out}");
        assert!(!out.contains("Usage: folio"), "{name}: {out}");
    }
}

#[test]
fn tzent_dropped_mouths_unknown() {
    let dir = design();
    for args in [
        vec!["inject", "--print", "--dir", dir.as_str()],
        vec!["serve", "--dir", dir.as_str(), "--host", "127.0.0.1"],
    ] {
        let (rc, out, err) = tz(&args);
        assert_eq!(rc, 1, "{args:?}: {err}");
        assert_eq!(out, "", "{args:?}");
        assert!(err.starts_with("tz: subcommand\n"), "{args:?}: {err}");
    }
}

#[test]
fn tzent_index_view_reaches_folio() {
    let dir = design();
    for flags in [
        vec!["--print"],
        vec!["--digest"],
        vec!["--print", "--summary"],
    ] {
        let mut mine = vec!["graph"];
        mine.extend(&flags);
        mine.extend(["--dir", dir.as_str()]);
        let (rc, out, err) = tz(&mine);
        let mut theirs = mine.clone();
        theirs.insert(0, "folio");
        let (want_rc, want_out, want_err) = entry(&theirs);
        assert_eq!(rc, 0, "{flags:?}: {err}");
        assert_eq!(i32::from(want_rc), rc, "{flags:?}");
        assert!(!out.is_empty(), "{flags:?}");
        assert_eq!(out, want_out, "{flags:?}");
        assert_eq!(err, want_err, "{flags:?}");
    }
}

#[test]
fn tzent_index_view_mix_refused() {
    assert_eq!(
        GRAPH_FLAGS,
        [
            "--check",
            "--design",
            "--repo",
            "--bd",
            "--folio",
            "--state-dir",
            "--project"
        ]
    );
    for args in [
        vec!["graph", "--print", "--check"],
        vec!["graph", "--design", "--digest"],
        vec!["graph", "--summary", "--print", "--repo=/nowhere"],
        vec!["graph", "--digest", "--project", "/nowhere"],
    ] {
        let (rc, out, err) = tz(&args);
        assert_eq!(rc, 1, "{args:?}: {err}");
        assert_eq!(out, "", "{args:?}");
        assert!(err.contains("は混ぜない"), "{args:?}: {err}");
        assert!(err.contains("usage: tz graph"), "{args:?}: {err}");
    }
}

#[test]
fn tzent_entry_writes_to_given_writers() {
    let dup = root().join("folio2/tests/fixtures/check/dup-key");
    let dup = dup.display().to_string();
    let (rc, out, err) = entry(&["folio", "check", "--dir", dup.as_str()]);
    assert_eq!(rc, 1, "{err}");
    assert!(
        out.lines().any(|l| l.starts_with("folio check: 不合格（違反 ")),
        "{out}"
    );
    assert!(err.contains("まだ分からない"), "{err}");

    let (rc, out, err) = entry(&["folio", "check", "--no-such-flag"]);
    assert_eq!(rc, 2);
    assert_eq!(out, "");
    assert!(err.contains("--no-such-flag"), "{err}");
    assert!(err.contains("Usage: folio check"), "{err}");

    let (rc, out, err) = entry(&["folio", "check", "--help"]);
    assert_eq!(rc, 0);
    assert_eq!(err, "");
    assert!(out.contains("Usage: folio check"), "{out}");
}

#[test]
fn tzent_own_names_clean() {
    let words: Vec<&str> = FILTER_WORDS.split_whitespace().collect();
    assert_eq!(words.len(), 325, "filter の語の数");
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth5/tzent.rs"))
        .expect("tests/teeth5/tzent.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 6, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("tzent_")
            .unwrap_or_else(|| panic!("{name} は tzent_ で始まらない"));
        for word in &words {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
