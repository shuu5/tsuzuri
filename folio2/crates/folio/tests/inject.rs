//! `folio inject` の歯（便 2・docs/design/delivery-2.md §1）。
//! 正本と今の CLAUDE.md で --check 合格、tests/fixtures/inject/ の 5 組で §1 の期待の終了コード、drift の写しで --write の往復、
//! 正本の写しに変異を 1 つ当てる 6 入力（終了コードを便 2 の §1 の値で pin する・script は呼ばない・docs/design/delivery-3.md §1 (a)）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(name: &str) -> PathBuf {
    repo_root().join("tests/fixtures/inject").join(name)
}

fn folio_inject(dir: &Path, claude_md: &Path, mode: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("inject")
        .arg("--dir")
        .arg(dir)
        .arg("--claude-md")
        .arg(claude_md)
        .arg(mode)
        .output()
        .expect("folio を起動できない")
}

fn code(out: &Output, what: &str) -> i32 {
    out.status.code().unwrap_or_else(|| {
        panic!(
            "{what} が signal で終わった: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-inject-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    fs::create_dir_all(&td).unwrap();
    td
}

fn fixture_check(name: &str, expected: i32) {
    let dir = fixture(name);
    let out = folio_inject(&dir, &dir.join("CLAUDE.md"), "--check");
    assert_eq!(
        code(&out, "folio inject"),
        expected,
        "{name}: {}",
        stderr(&out)
    );
}

#[test]
fn inject_canonical_claude_md_passes() {
    let root = repo_root();
    let out = folio_inject(
        &root.join("design-intent"),
        &root.join("CLAUDE.md"),
        "--check",
    );
    assert_eq!(code(&out, "folio inject"), 0, "{}", stderr(&out));
}

#[test]
fn inject_fixture_ok_passes() {
    fixture_check("ok", 0);
}

#[test]
fn inject_fixture_drift_fails() {
    fixture_check("drift", 1);
}

#[test]
fn inject_fixture_no_marker_is_unknown() {
    fixture_check("no-marker", 2);
}

#[test]
fn inject_fixture_outside_fails() {
    fixture_check("outside", 1);
}

#[test]
fn inject_fixture_over_limit_fails_and_does_not_write() {
    fixture_check("over-limit", 1);
    let dir = fixture("over-limit");
    let td = temp_dir("over-limit");
    let md = td.join("CLAUDE.md");
    fs::write(
        &md,
        "<!-- constitution:begin -->\n<!-- constitution:end -->\n",
    )
    .unwrap();
    let before = fs::read(&md).unwrap();
    let out = folio_inject(&dir, &md, "--write");
    let after = fs::read(&md).unwrap();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&out, "folio inject"), 1, "{}", stderr(&out));
    assert_eq!(before, after, "上限を超えたのに書いた");
}

#[test]
fn inject_write_round_trip_from_drift() {
    let td = temp_dir("round-trip");
    let md = td.join("CLAUDE.md");
    fs::copy(fixture("drift").join("CLAUDE.md"), &md).unwrap();
    let dir = fixture("drift");
    let write = folio_inject(&dir, &md, "--write");
    let check = folio_inject(&dir, &md, "--check");
    let written = fs::read(&md).unwrap();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        code(&write, "folio inject --write"),
        0,
        "{}",
        stderr(&write)
    );
    assert_eq!(
        code(&check, "folio inject --check"),
        0,
        "{}",
        stderr(&check)
    );
    assert_eq!(written, fs::read(fixture("ok").join("CLAUDE.md")).unwrap());
}

#[test]
fn inject_mode_is_exactly_one() {
    let dir = fixture("ok");
    let md = dir.join("CLAUDE.md");
    for args in [&["--check", "--print"][..], &[][..]] {
        let out = Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("inject")
            .arg("--dir")
            .arg(&dir)
            .arg("--claude-md")
            .arg(&md)
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}: {}", stderr(&out));
    }
}

/// 1 入力: 正本 2 file と CLAUDE.md を一時 dir へ写し、変異を 1 つ当て、folio を掛けて §1 の終了コードと比べる。
/// 戻り値は folio の出力と、掛けた後の写しの CLAUDE.md の byte 列。
fn pinned(case: &str, mode: &str, expected: i32, mutate: impl FnOnce(&Path)) -> (Output, Vec<u8>) {
    let root = repo_root();
    let td = temp_dir(&format!("pinned-{case}"));
    for name in ["constitution.yaml", "rules.yaml"] {
        fs::copy(root.join("design-intent").join(name), td.join(name)).unwrap();
    }
    fs::copy(root.join("CLAUDE.md"), td.join("CLAUDE.md")).unwrap();
    mutate(&td);
    let md = td.join("CLAUDE.md");
    let folio = folio_inject(&td, &md, mode);
    let md_bytes = fs::read(&md).unwrap();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        code(&folio, "folio inject"),
        expected,
        "{case}: folio（期待 {expected}）\n{}",
        stderr(&folio)
    );
    (folio, md_bytes)
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn edit(path: &Path, f: impl FnOnce(&str) -> String) {
    let before = fs::read_to_string(path).unwrap();
    let after = f(&before);
    assert_ne!(before, after, "変異が当たっていない: {}", path.display());
    fs::write(path, after).unwrap();
}

/// (1) 変異なし --check。
#[test]
fn inject_pinned_unmutated_passes() {
    pinned("unmutated", "--check", 0, |_| {});
}

/// (2) 変異なし --print。区間の中身 = 改行 + 本文 + 改行・--print = 本文 + 改行。
#[test]
fn inject_pinned_print_matches_region() {
    let (folio, md) = pinned("print", "--print", 0, |_| {});
    assert!(!folio.stdout.is_empty());
    let begin = b"<!-- constitution:begin -->";
    let end = b"<!-- constitution:end -->";
    let start = find(&md, begin).expect("begin の印が無い") + begin.len();
    let stop = start + find(&md[start..], end).expect("end の印が無い");
    let mut expected = b"\n".to_vec();
    expected.extend_from_slice(&folio.stdout);
    assert_eq!(&md[start..stop], &expected[..]);
}

/// (3) 区間の本文の 1 文字を変える。
#[test]
fn inject_pinned_region_char_changed_fails() {
    pinned("region-char", "--check", 1, |td| {
        edit(&td.join("CLAUDE.md"), |text| {
            text.replacen("P-1.1: ", "P-1.9: ", 1)
        });
    });
}

/// (4) end の marker を消す。
#[test]
fn inject_pinned_end_marker_removed_is_unknown() {
    pinned("end-marker", "--check", 2, |td| {
        edit(&td.join("CLAUDE.md"), |text| {
            text.replacen("<!-- constitution:end -->", "", 1)
        });
    });
}

/// (5) P-1 の最初の規範文の strength を maybe にする。
#[test]
fn inject_pinned_strength_maybe_is_unknown() {
    pinned("strength-maybe", "--check", 2, |td| {
        edit(&td.join("constitution.yaml"), |text| {
            text.replacen(
                "{id: P-1.1, pattern: ubiquitous, strength: must,",
                "{id: P-1.1, pattern: ubiquitous, strength: maybe,",
                1,
            )
        });
    });
}

/// (6) 区間の外の末尾に規範語で終わる 1 行を足す。
#[test]
fn inject_pinned_normative_line_outside_fails() {
    pinned("outside", "--check", 1, |td| {
        edit(&td.join("CLAUDE.md"), |text| {
            format!("{text}これは規範とする。\n")
        });
    });
}
