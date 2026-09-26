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
    // 便 159 (c) の 6: 置き換えの道を write_md に移したので字を縛る。
    assert!(
        stderr(&write).starts_with("folio inject: wrote "),
        "{}",
        stderr(&write)
    );
    assert!(
        stderr(&check).trim_end().ends_with(" byte 一致"),
        "{}",
        stderr(&check)
    );
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

/// 外の置き場で区間の外を数えなかったときの知らせの行の頭（便 159 (b) の 5）。
const NOTICE: &str = "folio inject: # 区間の外の規範語の行は数えていない";

fn notices(out: &Output) -> Vec<String> {
    stderr(out)
        .lines()
        .filter(|l| l.starts_with(NOTICE))
        .map(str::to_string)
        .collect()
}

/// 手書きの ok/CLAUDE.md と、その頭の 2 行（「# fixture」と空行）を除いた区間だけの byte。
fn ok_and_region() -> (Vec<u8>, Vec<u8>) {
    let ok = fs::read(fixture("ok").join("CLAUDE.md")).unwrap();
    let head = b"# fixture\n\n";
    assert!(ok.starts_with(head), "ok/CLAUDE.md の頭が変わった");
    let region = ok[head.len()..].to_vec();
    (ok, region)
}

/// 歯 1（便 159 (c) の 1）: 区間の置き場が無い 8 形に --write が区間を作り、続く --check が合格・2 回目は差が無い。
#[test]
fn f159_write_puts_the_region_where_there_is_none() {
    let dir = fixture("ok");
    let (ok, region) = ok_and_region();
    let joined = |head: &[u8], gap: &[u8]| [head, gap, &region[..]].concat();
    // (形, 元の byte〔None = 無い〕, 書いた後の期待の byte)。
    let cases = [
        ("absent", None, region.clone()),
        ("empty", Some(&b""[..]), region.clone()),
        ("newline", Some(&b"# fixture\n"[..]), ok.clone()),
        ("bare", Some(&b"# fixture"[..]), ok.clone()),
        (
            "bom",
            Some("\u{feff}# fixture\n".as_bytes()),
            joined("\u{feff}# fixture\n".as_bytes(), b"\n"),
        ),
        (
            "crlf",
            Some(&b"# fixture\r\n\r\nmemo\r\n"[..]),
            joined(b"# fixture\r\n\r\nmemo\r\n", b"\n"),
        ),
        (
            "blank-tail",
            Some(&b"# fixture\n\n"[..]),
            joined(b"# fixture\n\n", b"\n"),
        ),
        (
            "space-tail",
            Some(&b"# fixture  "[..]),
            joined(b"# fixture  ", b"\n\n"),
        ),
    ];
    for (case, before, want) in cases {
        let verb = match before {
            None => "無いので区間だけの file を作った",
            Some(_) => "marker が無いので末尾に区間を足した",
        };
        let td = temp_dir(&format!("f159-put-{case}"));
        let md = td.join("CLAUDE.md");
        if let Some(before) = before {
            fs::write(&md, before).unwrap();
        }
        let write = folio_inject(&dir, &md, "--write");
        let written = fs::read(&md).unwrap();
        let check = folio_inject(&dir, &md, "--check");
        let again = folio_inject(&dir, &md, "--write");
        let after = fs::read(&md).unwrap();
        let _ = fs::remove_dir_all(&td);
        assert_eq!(code(&write, "--write"), 0, "{case}: {}", stderr(&write));
        assert!(
            written == want,
            "{case}: 書いた byte が違う\n{}",
            String::from_utf8_lossy(&written)
        );
        assert!(
            stderr(&write).starts_with(&format!("folio inject: {}: {verb}（", md.display())),
            "{case}: {}",
            stderr(&write)
        );
        assert_eq!(code(&check, "--check"), 0, "{case}: {}", stderr(&check));
        assert_eq!(code(&again, "--write 2 回目"), 0, "{case}: {}", stderr(&again));
        assert!(stderr(&again).contains("差が無い"), "{case}: {}", stderr(&again));
        assert_eq!(written, after, "{case}: 2 回目で byte が動いた");
    }

    // 相対の --claude-md でも字は作った file の絶対の path。
    let td = temp_dir("f159-put-relative");
    let out = Command::new(env!("CARGO_BIN_EXE_folio"))
        .current_dir(&td)
        .args(["inject", "--dir"])
        .arg(&dir)
        .args(["--claude-md", "CLAUDE.md", "--write"])
        .output()
        .unwrap();
    let abs = fs::canonicalize(&td).unwrap().join("CLAUDE.md");
    let made = fs::read(&abs).unwrap();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&out, "--write"), 0, "{}", stderr(&out));
    assert_eq!(
        stderr(&out).lines().next(),
        Some(
            format!(
                "folio inject: {}: 無いので区間だけの file を作った（4 行 / {} byte）",
                abs.display(),
                region.len() - "<!-- constitution:begin -->\n\n<!-- constitution:end -->\n".len()
            )
            .as_str()
        ),
    );
    assert_eq!(made, region);

    // folio2 の写しへの --write は CLAUDE.md を変えない。
    let (write, md) = pinned("f159-folio2", "--write", 0, |_| {});
    assert_eq!(md, fs::read(repo_root().join("CLAUDE.md")).unwrap());
    assert!(stderr(&write).contains("差が無い"), "{}", stderr(&write));
}

/// 歯 2（便 159 (c) の 2）: marker が在って 1 対でない 4 形は --write でも 2 で byte 不変。
#[test]
fn f159_write_still_refuses_markers_that_are_not_one_pair() {
    let dir = fixture("ok");
    let b = "<!-- constitution:begin -->\n";
    let e = "<!-- constitution:end -->\n";
    for (case, text) in [
        ("begin-only", format!("# x\n\n{b}")),
        ("end-only", format!("# x\n\n{e}")),
        ("reversed", format!("# x\n\n{e}{b}")),
        ("two-pairs", format!("# x\n\n{b}{e}{b}{e}")),
    ] {
        let td = temp_dir(&format!("f159-pair-{case}"));
        let md = td.join("CLAUDE.md");
        fs::write(&md, &text).unwrap();
        let out = folio_inject(&dir, &md, "--write");
        let after = fs::read_to_string(&md).unwrap();
        let _ = fs::remove_dir_all(&td);
        assert_eq!(code(&out, "--write"), 2, "{case}: {}", stderr(&out));
        assert!(
            stderr(&out).contains("marker が 1 対でない"),
            "{case}: {}",
            stderr(&out)
        );
        assert_eq!(after, text, "{case}: byte が動いた");
    }
}

/// 歯 3（便 159 (c) の 3）: 作ってはいけない所では作らず、上書きもしない。
#[test]
fn f159_write_makes_nothing_it_must_not() {
    let dir = fixture("ok");
    let td = temp_dir("f159-nothing");

    // UTF-8 でない file は 読めない の 2 で byte 不変。
    let bad = td.join("bad.md");
    fs::write(&bad, [0xff, 0xfe, b'a', b'\n']).unwrap();
    let out = folio_inject(&dir, &bad, "--write");
    assert_eq!(code(&out, "--write"), 2, "{}", stderr(&out));
    assert!(stderr(&out).contains("読めない"), "{}", stderr(&out));
    assert_eq!(fs::read(&bad).unwrap(), [0xff, 0xfe, b'a', b'\n']);

    // 無い CLAUDE.md に --check は 2・--print は 0 で、どちらも作らない。
    let none = td.join("none.md");
    let check = folio_inject(&dir, &none, "--check");
    assert_eq!(code(&check, "--check"), 2, "{}", stderr(&check));
    assert!(stderr(&check).contains("読めない"), "{}", stderr(&check));
    let print = folio_inject(&dir, &none, "--print");
    assert_eq!(code(&print, "--print"), 0, "{}", stderr(&print));
    assert!(!none.exists(), "--check か --print が作った");

    // 上限を超える正本では --write も作らない。
    let over = folio_inject(&fixture("over-limit"), &none, "--write");
    assert_eq!(code(&over, "--write"), 1, "{}", stderr(&over));
    assert!(!none.exists(), "上限を超えたのに作った");

    // 無い dir の下へは 書けない の 2（親を作らない）。
    let deep = td.join("no-such-dir").join("CLAUDE.md");
    let out = folio_inject(&dir, &deep, "--write");
    assert_eq!(code(&out, "--write"), 2, "{}", stderr(&out));
    assert!(stderr(&out).contains("書けない"), "{}", stderr(&out));
    assert!(!td.join("no-such-dir").exists(), "親の dir を作った");

    // 行き先の無い symlink は 読めない の 2 で、行き先を作らない。
    let target = td.join("target.md");
    let link = td.join("link.md");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let out = folio_inject(&dir, &link, "--write");
    let made = target.exists();
    let still_link = fs::symlink_metadata(&link)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false);
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&out, "--write"), 2, "{}", stderr(&out));
    assert!(stderr(&out).contains("読めない"), "{}", stderr(&out));
    assert!(!made, "行き先の無い symlink の先を作った");
    assert!(still_link, "symlink を上書きした");
}

/// 歯 5（便 159 (c) の 5）: 区間の外の規範語は folio2 の置き場でだけ数え、外の置き場は判定の外の 1 行で知らせる。
/// 区間と正本の差分は外の置き場でも数える。
#[test]
fn f159_outside_lines_count_only_in_folio2s_own_place() {
    // ① folio init の骨格（名 未記入）。
    let td = temp_dir("f159-scope-skeleton");
    let place = td.join("design-intent");
    let init = Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(["init", "--dir"])
        .arg(&place)
        .output()
        .unwrap();
    assert_eq!(code(&init, "folio init"), 0, "{}", stderr(&init));
    let md = td.join("CLAUDE.md");
    let user = "回答は日本語でする。\n";
    fs::write(&md, user).unwrap();
    let write = folio_inject(&place, &md, "--write");
    let written = fs::read_to_string(&md).unwrap();
    let check = folio_inject(&place, &md, "--check");
    edit(&md, |text| text.replacen("P-1.1: ", "P-1.9: ", 1));
    let drift = folio_inject(&place, &md, "--check");
    fs::write(
        &md,
        format!("{user}\n<!-- constitution:begin -->\n<!-- constitution:end -->\n"),
    )
    .unwrap();
    let empty = folio_inject(&place, &md, "--check");
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&write, "--write"), 0, "{}", stderr(&write));
    assert!(written.starts_with(user), "{written}");
    assert_eq!(code(&check, "--check"), 0, "{}", stderr(&check));
    let seen = notices(&check);
    assert_eq!(seen.len(), 1, "{}", stderr(&check));
    assert!(seen[0].contains("「未記入」"), "{}", seen[0]);
    assert_eq!(code(&drift, "区間の 1 字の差"), 1, "{}", stderr(&drift));
    assert_eq!(notices(&drift).len(), 1, "{}", stderr(&drift));
    assert_eq!(code(&empty, "空の区間"), 2, "{}", stderr(&empty));
    assert_eq!(notices(&empty).len(), 1, "{}", stderr(&empty));

    // ② folio2 の写しの名を表の 2 行目に替えると、区間の外の規範語の行が在っても 0 と知らせ。
    let line = |td: &Path| {
        edit(&td.join("CLAUDE.md"), |text| {
            format!("{text}これは規範とする。\n")
        })
    };
    let (renamed, _) = pinned("f159-scope-renamed", "--check", 0, |td| {
        edit(&td.join("constitution.yaml"), |text| {
            text.replacen("id: folio2-constitution", "id: tsuzuri-constitution", 1)
        });
        line(td);
    });
    let seen = notices(&renamed);
    assert_eq!(seen.len(), 1, "{}", stderr(&renamed));
    assert!(seen[0].contains("「tsuzuri-constitution」"), "{}", seen[0]);

    // ③ 名が folio2 のままなら数えて 1・知らせなし。
    let (own, _) = pinned("f159-scope-own", "--check", 1, line);
    assert!(notices(&own).is_empty(), "{}", stderr(&own));

    // ④ 行を足さない folio2 の写しは 0 で標準エラーは 1 行。
    let (plain, _) = pinned("f159-scope-plain", "--check", 0, |_| {});
    assert_eq!(stderr(&plain).lines().count(), 1, "{}", stderr(&plain));
}
