//! 天井の名札（部品 ceiling-stamp・便 40・docs/design/delivery-40.md §1 (c)〜(f)・ADR-8 決定 (4)・FR18）の歯。binary 経由。
//! 束は便 38 の凍結 fixture（tests/fixtures/ceiling/bundle/）から `folio ceiling --write` で一時 dir に組み、
//! 所見 file は便 39 の凍結 fixture（tests/fixtures/ceiling/findings/）を <観点>/findings.yaml に写してから面を撃つ。
//! - 1. 逐語（合格 ×4・不合格・まだ分からない・観点の dir なし）・2. 未実施（--ceiling なし）・3. 5 面で同じ字・
//!   4. --check の一致と DRIFT・5. 部品目録（--print の byte 一致・--check 5 面）・6. 面の凍結 fixture 7 本の byte 一致
//!
//! 凍結 fixture の更新は生成器の出力を写す形になるので、(c) の字面の逐語（1・2）を独立の物差しとして置く（P-10.2）。
//! 版管理の下の file は書き換えない（`--out` と束の置き場は必ず一時 dir の中）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const VIEWPOINTS: [&str; 4] = ["fidelity", "readability", "coherence", "reality"];

/// (c) の 1 形目（便 38 の束 + 便 39 の pass-<観点>.yaml 4 本）。
const STAMP_PASS: &str = "<span data-component=\"ceiling-stamp\">天井 <b>忠実さ 合格</b> · <b>読みやすさ 合格</b> · <b>文書どうしの整合 合格</b> · <b>実態との整合 合格</b>（2026-09-19T05:00:00Z・束 2bd67637/ded1548e/91ca924d/24ce1b87）</span>";

/// (c) の 2 形目（--ceiling なし）。
const STAMP_NONE: &str = "<span data-component=\"ceiling-stamp\">天井 <b>忠実さ まだ分からない</b> · <b>読みやすさ まだ分からない</b> · <b>文書どうしの整合 まだ分からない</b> · <b>実態との整合 まだ分からない</b>（未実施）</span>";

const STAMP_OPEN: &str = "<span data-component=\"ceiling-stamp\">";
const FRESHNESS_OPEN: &str = "<span data-component=\"freshness-stamp\">";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// 面の凍結 fixture（正本・様式・期待の面 7 本・天井の正本 ceiling.yaml）。
fn fixture() -> PathBuf {
    repo_root().join("tests/fixtures/face")
}

/// 便 38 の凍結 fixture（source/ = 最小の正本・faces/ = 面の写しの見本）。
fn bundle_fixture() -> PathBuf {
    repo_root().join("tests/fixtures/ceiling/bundle")
}

/// 便 39 の所見 file の凍結 fixture。
fn findings_fixture(name: &str) -> String {
    fs::read_to_string(
        repo_root()
            .join("tests/fixtures/ceiling/findings")
            .join(name),
    )
    .unwrap()
}

fn design_intent() -> PathBuf {
    repo_root().join("design-intent")
}

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-badge-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    fs::create_dir_all(&td).unwrap();
    td
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

/// 組み立てた binary を子の処理で撃つ。
fn folio(args: &[&dyn AsRef<std::ffi::OsStr>]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_folio"));
    for a in args {
        cmd.arg(a.as_ref());
    }
    cmd.output().expect("folio を起動できない")
}

fn code(out: &Output, what: &str) -> i32 {
    out.status.code().unwrap_or_else(|| {
        panic!(
            "{what} が signal で終わった: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn assert_code(out: &Output, what: &str, want: i32) {
    assert_eq!(
        code(out, what),
        want,
        "{what}: stdout:\n{}stderr:\n{}",
        stdout(out),
        stderr(out)
    );
}

/// 便 38 の fixture から組んだ束の置き場（4 観点に便 39 の pass-<観点>.yaml を写した状態）。
struct Bundle {
    out: PathBuf,
}

impl Bundle {
    fn passing(td: &Path, tag: &str) -> Bundle {
        let src = td.join(format!("bundle-{tag}-src"));
        let faces = td.join(format!("bundle-{tag}-faces"));
        copy_tree(&bundle_fixture().join("source"), &src);
        copy_tree(&bundle_fixture().join("faces"), &faces);
        let out = td.join(format!("bundle-{tag}"));
        let run = folio(&[
            &"ceiling", &"--dir", &src, &"--faces", &faces, &"--out", &out, &"--write",
        ]);
        assert_code(&run, "folio ceiling --write", 0);
        let bundle = Bundle { out };
        for id in VIEWPOINTS {
            bundle.put(id, &findings_fixture(&format!("pass-{id}.yaml")));
        }
        bundle
    }

    /// <観点>/findings.yaml に書く。
    fn put(&self, id: &str, text: &str) {
        fs::write(self.out.join(id).join("findings.yaml"), text).unwrap();
    }
}

/// 面の凍結 fixture の写し（`tests/site.rs` の fixture_copy と同じ集合 + 天井の正本）: 正本 6 file と ceiling.yaml・
/// adr/ADR-2.yaml・design-note/full.yaml を src/ へ、様式 2 本を src/preview/ へ、器の導出 file と図の道具を親 dir へ。
fn face_copy(td: &Path) -> PathBuf {
    let work = td.join("src");
    fs::create_dir_all(work.join("preview")).unwrap();
    fs::create_dir_all(work.join("adr")).unwrap();
    fs::create_dir_all(work.join("design-note")).unwrap();
    fs::create_dir_all(td.join("contracts")).unwrap();
    for name in [
        "constitution.yaml",
        "rules.yaml",
        "vocabulary.yaml",
        "srs.yaml",
        "index.yaml",
        "intake.yaml",
        "ceiling.yaml",
        "adr/ADR-2.yaml",
        "design-note/full.yaml",
    ] {
        fs::copy(fixture().join(name), work.join(name)).unwrap();
    }
    for name in ["folio.css", "folio-ui.js"] {
        fs::copy(fixture().join(name), work.join("preview").join(name)).unwrap();
    }
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        td.join("contracts/schema.toml"),
    )
    .unwrap();
    copy_tree(
        &repo_root().join("vendor/archify"),
        &td.join("vendor/archify"),
    );
    work
}

/// 実の正本の写し（design-intent/ と器の導出 file と図の道具）。戻り値 = 正本の写し。
fn real_copy(td: &Path) -> PathBuf {
    let dir = td.join("design-intent");
    copy_tree(&design_intent(), &dir);
    fs::create_dir_all(td.join("contracts")).unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        td.join("contracts/schema.toml"),
    )
    .unwrap();
    copy_tree(
        &repo_root().join("vendor/archify"),
        &td.join("vendor/archify"),
    );
    dir
}

/// `folio face --face <face> [--id <id>] --dir <dir> --out <out> [--ceiling <bundle>] --write` → 面の本文。
fn write_face(
    face: &str,
    id: Option<&str>,
    dir: &Path,
    out: &Path,
    ceiling: Option<&Path>,
) -> String {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_folio"));
    cmd.arg("face").arg("--face").arg(face);
    if let Some(id) = id {
        cmd.arg("--id").arg(id);
    }
    cmd.arg("--dir").arg(dir).arg("--out").arg(out);
    if let Some(c) = ceiling {
        cmd.arg("--ceiling").arg(c);
    }
    let run = cmd.arg("--write").output().expect("folio を起動できない");
    assert_code(&run, &format!("folio face --face {face} --write"), 0);
    fs::read_to_string(out).unwrap()
}

/// `folio build --dir <dir> --out <out> [--ceiling <bundle>] <mode>`。
fn build(dir: &Path, out: &Path, ceiling: Option<&Path>, mode: &str) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_folio"));
    cmd.arg("build").arg("--dir").arg(dir).arg("--out").arg(out);
    if let Some(c) = ceiling {
        cmd.arg("--ceiling").arg(c);
    }
    cmd.arg(mode).output().expect("folio を起動できない")
}

/// 面の中の ceiling-stamp の span（ちょうど 1 つ）。
fn stamp_of(html: &str, what: &str) -> String {
    assert_eq!(
        html.matches(STAMP_OPEN).count(),
        1,
        "{what}: ceiling-stamp が 1 つでない"
    );
    let start = html.find(STAMP_OPEN).unwrap();
    let end = html[start..]
        .find("</span>")
        .unwrap_or_else(|| panic!("{what}: ceiling-stamp が閉じない"));
    html[start..start + end + "</span>".len()].to_string()
}

/// `html` に `verbatim` が逐語で 1 度在る。
fn assert_once(html: &str, verbatim: &str, what: &str) {
    assert_eq!(
        html.matches(verbatim).count(),
        1,
        "{what}: 逐語が 1 度でない: {verbatim}\n--- 面の名札\n{}",
        stamp_of(html, what)
    );
}

// ── 1. 逐語 ──

#[test]
fn badge_stamp_is_verbatim_for_pass_fail_unknown_and_missing_bundle() {
    let td = temp_dir("verbatim");
    let bundle = Bundle::passing(&td, "a");
    let work = face_copy(&td);
    let out = td.join("index.html");

    // 合格 ×4・日付・束の要約値
    let html = write_face("index", None, &work, &out, Some(&bundle.out));
    assert_once(&html, STAMP_PASS, "合格 ×4");

    // 不合格（止める が支持のまま残る）
    bundle.put("fidelity", &findings_fixture("stop-upheld.yaml"));
    let html = write_face("index", None, &work, &out, Some(&bundle.out));
    assert_once(
        &html,
        &STAMP_PASS.replacen("忠実さ 合格", "忠実さ 不合格", 1),
        "不合格",
    );

    // まだ分からない（起動の記録に at が無い）→ 日付は残り 3 観点の at
    bundle.put("fidelity", &findings_fixture("missing-field.yaml"));
    let html = write_face("index", None, &work, &out, Some(&bundle.out));
    assert_once(
        &html,
        &STAMP_PASS.replacen("忠実さ 合格", "忠実さ まだ分からない", 1),
        "まだ分からない",
    );

    // 観点の dir が無い → まだ分からない・束の先頭が --------
    fs::remove_dir_all(bundle.out.join("fidelity")).unwrap();
    let html = write_face("index", None, &work, &out, Some(&bundle.out));
    assert_once(
        &html,
        &STAMP_PASS
            .replacen("忠実さ 合格", "忠実さ まだ分からない", 1)
            .replacen("束 2bd67637/", "束 --------/", 1),
        "観点の dir なし",
    );
    let _ = fs::remove_dir_all(&td);
}

// ── 2. 未実施 ──

#[test]
fn badge_stamp_says_not_run_without_the_ceiling_flag() {
    let td = temp_dir("none");
    let work = face_copy(&td);
    let html = write_face("index", None, &work, &td.join("index.html"), None);
    let _ = fs::remove_dir_all(&td);
    assert_once(&html, STAMP_NONE, "--ceiling なし");
    assert!(html.contains("（未実施）"), "「未実施」が無い");
}

// ── 3. 5 面 ──

const FIVE_FACES: [&str; 5] = [
    "index.html",
    "constitution.html",
    "srs.html",
    "adr-1.html",
    "note-example.html",
];

#[test]
fn badge_five_faces_carry_the_same_stamp_right_after_the_freshness_stamp() {
    let td = temp_dir("five");
    let bundle = Bundle::passing(&td, "a");
    let dir = real_copy(&td);
    let site = td.join("site");
    let run = build(&dir, &site, Some(&bundle.out), "--write");
    let pages: Vec<(&str, String)> = FIVE_FACES
        .iter()
        .map(|name| {
            (
                *name,
                fs::read_to_string(site.join(name)).unwrap_or_default(),
            )
        })
        .collect();
    let _ = fs::remove_dir_all(&td);

    assert_code(&run, "folio build --ceiling --write", 0);
    let mut stamps = Vec::new();
    for (name, html) in &pages {
        assert!(!html.is_empty(), "{name} が配信先に無い");
        let stamp = stamp_of(html, name);
        assert!(stamp.starts_with(STAMP_OPEN), "{name}: {stamp}");
        assert!(
            stamp.contains("束 2bd67637/"),
            "{name}: 束の要約値が無い: {stamp}"
        );
        // freshness-stamp の直後（間は改行だけ）
        let fresh = html
            .find(FRESHNESS_OPEN)
            .unwrap_or_else(|| panic!("{name}: freshness-stamp が無い"));
        let fresh_end = fresh + html[fresh..].find("</span>").unwrap() + "</span>".len();
        let between = &html[fresh_end..html.find(STAMP_OPEN).unwrap()];
        assert!(
            between.chars().all(char::is_whitespace),
            "{name}: ceiling-stamp が freshness-stamp の直後でない: {between:?}"
        );
        stamps.push(stamp);
    }
    for (name, stamp) in FIVE_FACES.iter().zip(&stamps) {
        assert_eq!(stamp, &stamps[0], "{name} の名札が index.html と違う");
    }
}

// ── 4. 一致と DRIFT ──

#[test]
fn badge_check_matches_the_same_ceiling_and_drifts_on_another() {
    let td = temp_dir("check");
    let same = Bundle::passing(&td, "a");
    let other = Bundle::passing(&td, "b");
    other.put("fidelity", &findings_fixture("stop-upheld.yaml"));
    let dir = real_copy(&td);
    let site = td.join("site");
    let write = build(&dir, &site, Some(&same.out), "--write");
    let ok = build(&dir, &site, Some(&same.out), "--check");
    let drift = build(&dir, &site, Some(&other.out), "--check");
    let none = build(&dir, &site, None, "--check");
    let _ = fs::remove_dir_all(&td);

    assert_code(&write, "folio build --write", 0);
    assert_code(&ok, "folio build --check（同じ束）", 0);
    assert!(
        stdout(&ok).starts_with("folio build: OK"),
        "{}",
        stdout(&ok)
    );
    assert_code(&drift, "folio build --check（別の束）", 1);
    assert!(stderr(&drift).contains("DRIFT"), "{}", stderr(&drift));
    for name in FIVE_FACES {
        assert!(stderr(&drift).contains(name), "{name}: {}", stderr(&drift));
    }
    assert_code(&none, "folio build --check（--ceiling なし）", 1);
    assert!(stderr(&none).contains("DRIFT"), "{}", stderr(&none));
}

// ── 5. 目録 ──

#[test]
fn badge_parts_catalog_lists_the_stamp_and_the_five_faces_pass_parts_check() {
    let print = folio(&[&"parts", &"--print"]);
    assert_code(&print, "folio parts --print", 0);
    let frozen = fs::read(repo_root().join("tests/fixtures/floor/parts-catalog.json")).unwrap();
    assert!(
        print.stdout == frozen,
        "parts --print が凍結目録と byte で違う:\n{}",
        stdout(&print)
    );
    assert!(
        stdout(&print)
            .contains("\"ceiling-stamp\":[\"index\",\"constitution\",\"srs\",\"adr\",\"note\"]"),
        "{}",
        stdout(&print)
    );

    let td = temp_dir("parts");
    let bundle = Bundle::passing(&td, "a");
    let dir = real_copy(&td);
    let site = td.join("site");
    let write = build(&dir, &site, Some(&bundle.out), "--write");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_folio"));
    cmd.arg("parts").arg("--check").arg("--dir").arg(&dir);
    for (face, name) in ["index", "constitution", "srs", "adr", "note"]
        .iter()
        .zip(FIVE_FACES)
    {
        cmd.arg("--page")
            .arg(format!("{face}={}", site.join(name).display()));
    }
    let check = cmd.output().expect("folio を起動できない");
    let _ = fs::remove_dir_all(&td);

    assert_code(&write, "folio build --write", 0);
    assert_code(&check, "folio parts --check（5 面）", 0);
    assert!(
        stdout(&check).contains("folio parts: 合格（違反 0・まだ分からない 0）"),
        "{}",
        stdout(&check)
    );
}

// ── 6. 凍結 ──

#[test]
fn badge_faces_without_the_flag_match_the_seven_frozen_fixtures() {
    let td = temp_dir("frozen");
    let work = face_copy(&td);
    let same = |got: &str, expected: &str| {
        let frozen = fs::read_to_string(fixture().join(expected)).unwrap();
        if got != frozen {
            let at = got
                .bytes()
                .zip(frozen.bytes())
                .position(|(a, b)| a != b)
                .unwrap_or(got.len().min(frozen.len()));
            let show = |s: &str| {
                String::from_utf8_lossy(
                    &s.as_bytes()[at.saturating_sub(120)..(at + 200).min(s.len())],
                )
                .into_owned()
            };
            panic!(
                "{expected} と違う（{} byte ≠ {} byte・最初の差 {at} byte 目）\n--- folio\n{}\n--- 凍結\n{}",
                got.len(),
                frozen.len(),
                show(got),
                show(&frozen)
            );
        }
    };
    // 写しは ADR-2 だけ・支度表なし
    same(
        &write_face("index", None, &work, &td.join("index.html"), None),
        "expected-index.html",
    );
    same(
        &write_face(
            "constitution",
            None,
            &work,
            &td.join("constitution.html"),
            None,
        ),
        "expected.html",
    );
    same(
        &write_face("srs", None, &work, &td.join("srs.html"), None),
        "expected-srs.html",
    );
    same(
        &write_face(
            "note",
            Some("full"),
            &work,
            &td.join("note-full.html"),
            None,
        ),
        "expected-note.html",
    );
    same(
        &write_face("adr", Some("ADR-2"), &work, &td.join("adr-2.html"), None),
        "expected-site-adr-2.html",
    );
    // 支度表を置くと便 20 の期待
    fs::copy(
        fixture().join("intake-sheet.yaml"),
        work.join("intake-sheet.yaml"),
    )
    .unwrap();
    same(
        &write_face("index", None, &work, &td.join("index.html"), None),
        "expected-index-sheet.html",
    );
    // ADR-1 も置くと便 25 の期待（`tests/face_adr.rs` の写しと同じ集合・fixture の ADR-1 は欄が欠けるので入口の面の後）
    fs::copy(
        fixture().join("adr/ADR-1.yaml"),
        work.join("adr/ADR-1.yaml"),
    )
    .unwrap();
    same(
        &write_face("adr", Some("ADR-2"), &work, &td.join("adr-2.html"), None),
        "expected-adr.html",
    );
    let _ = fs::remove_dir_all(&td);
}
