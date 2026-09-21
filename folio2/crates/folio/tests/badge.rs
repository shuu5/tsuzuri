//! 天井の名札（部品 ceiling-stamp・便 40・docs/design/delivery-40.md §1 (c)〜(f)・ADR-8 決定 (4)・FR18）の歯。binary 経由。
//! 便 83（docs/design/delivery-83.md §1 (e)）で名札の出所は天井の印 `<dir>/preview/ceiling-stamp.yaml` の 1 つになり、
//! 旗 `--ceiling` は無くなった。歯は写しの置き場に印を手で書いてから面を撃つ。
//! - 1. 逐語（合格 ×4・不合格・まだ分からない）・2. 未実施（印なし）・3. 5 面で同じ字・
//!   4. --check の一致と DRIFT・5. 部品目録（--print の byte 一致・--check 5 面）・6. 面の凍結 fixture 7 本の byte 一致
//! - f83_: 印の値との一致・未実施・壊れた印・正本と食い違う印・旗が無いこと・5 面の小窓
//!
//! 凍結 fixture の更新は生成器の出力を写す形になるので、(c) の字面の逐語（1・2）を独立の物差しとして置く（P-10.2）。
//! 版管理の下の file は書き換えない（`--out` と印は必ず一時 dir の中）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 天井の正本の観点（id・名）。正本の順。
const VIEWPOINTS: [(&str, &str); 4] = [
    ("fidelity", "忠実さ"),
    ("readability", "読みやすさ"),
    ("coherence", "文書どうしの整合"),
    ("reality", "実態との整合"),
];

/// 歯の印の観点ごとの要約値（先頭 8 字）。
const BUNDLES: [&str; 4] = ["2bd67637", "ded1548e", "91ca924d", "24ce1b87"];

/// (c) の 1 形目（印の 4 観点とも 合格）。小窓の前まで。
const STAMP_PASS: &str = "<span data-component=\"ceiling-stamp\">天井 <b>忠実さ 合格</b> · <b>読みやすさ 合格</b> · <b>文書どうしの整合 合格</b> · <b>実態との整合 合格</b>（2026-09-19T05:00:00Z・束 2bd67637/ded1548e/91ca924d/24ce1b87）<span class=\"hint\">";

/// (c) の 2 形目（印なし）。小窓の前まで。
const STAMP_NONE: &str = "<span data-component=\"ceiling-stamp\">天井 <b>忠実さ まだ分からない</b> · <b>読みやすさ まだ分からない</b> · <b>文書どうしの整合 まだ分からない</b> · <b>実態との整合 まだ分からない</b>（未実施）<span class=\"hint\">";

const STAMP_OPEN: &str = "<span data-component=\"ceiling-stamp\">";
const FRESHNESS_OPEN: &str = "<span data-component=\"freshness-stamp\">";
const HINT_OPEN: &str = "<span class=\"hint\">";
const HINT_BODY_OPEN: &str = "<span class=\"hint-body\">";

/// 印の置き場（`--dir` からの相対）。
const MARK: &str = "preview/ceiling-stamp.yaml";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// 面の凍結 fixture（正本・様式・期待の面 7 本・天井の正本 ceiling.yaml）。
fn fixture() -> PathBuf {
    repo_root().join("tests/fixtures/face")
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

/// 印の全文（便 72 の欄の決まり・観点の行は（id・3 値・要約値・at））。
fn mark_text(at: &str, rows: &[(&str, &str, &str, &str)]) -> String {
    let mut text = format!(
        "# folio2 天井の印 — 生成物（folio ceiling --stamp が書く・手で直さない・P-6.2）\nround: fixture\nat: {at}\nverdict: 合格\nsources: sha256 00\nfaces: sha256 00\nviewpoints:\n"
    );
    for (id, verdict, bundle, at) in rows {
        text.push_str(&format!(
            "  - {{id: {id}, verdict: {verdict}, findings: 0, stops: 0, bundle: {bundle}, model: opus, effort: default, at: {at}}}\n"
        ));
    }
    text.push_str("refutes: []\nreads: [srs]\n");
    text
}

/// 4 観点の印（`verdicts` は正本の順）。top-level の at は 2026-09-19T05:00:00Z・観点の at はそれより前。
fn mark_with(verdicts: [&str; 4]) -> String {
    let rows: Vec<(&str, &str, &str, &str)> = VIEWPOINTS
        .iter()
        .zip(verdicts)
        .zip(BUNDLES)
        .map(|(((id, _), v), b)| (*id, v, b, "2026-09-19T04:00:00Z"))
        .collect();
    mark_text("2026-09-19T05:00:00Z", &rows)
}

/// `<dir>/preview/ceiling-stamp.yaml` に書く。
fn put_mark(dir: &Path, text: &str) {
    fs::create_dir_all(dir.join("preview")).unwrap();
    fs::write(dir.join(MARK), text).unwrap();
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

/// git を呼ぶ。環境変数 GIT_* は継承しない（tests/schema.rs と同じ形）。
fn git(cwd: &Path, args: &[&str]) {
    let mut cmd = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(key);
        }
    }
    let out = cmd
        .current_dir(cwd)
        .args([
            "-c",
            "user.email=fx@example",
            "-c",
            "user.name=fx",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .expect("git を起動できない");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// 実の正本の写し（design-intent/ と器の導出 file と図の道具）に歯の印 `mark` を置き、git の 1 commit にする
/// （folio build の --write は最初に構造の床を回す＝便 56・FR5・版管理の無い写しは「まだ分からない」の 2 になる）。
/// 戻り値 = 正本の写し。
fn real_copy(td: &Path, mark: &str) -> PathBuf {
    let dir = td.join("design-intent");
    copy_tree(&design_intent(), &dir);
    put_mark(&dir, mark);
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
    git(td, &["init", "-q"]);
    git(td, &["add", "-A"]);
    git(td, &["commit", "-q", "-m", "fixture"]);
    dir
}

/// `folio face --face <face> [--id <id>] --dir <dir> --out <out> --write` の出力。
fn run_face(face: &str, id: Option<&str>, dir: &Path, out: &Path) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_folio"));
    cmd.arg("face").arg("--face").arg(face);
    if let Some(id) = id {
        cmd.arg("--id").arg(id);
    }
    cmd.arg("--dir").arg(dir).arg("--out").arg(out);
    cmd.arg("--write").output().expect("folio を起動できない")
}

/// `folio face ... --write` → 面の本文。
fn write_face(face: &str, id: Option<&str>, dir: &Path, out: &Path) -> String {
    let run = run_face(face, id, dir, out);
    assert_code(&run, &format!("folio face --face {face} --write"), 0);
    fs::read_to_string(out).unwrap()
}

/// `folio build --dir <dir> --out <out> <mode>`。
fn build(dir: &Path, out: &Path, mode: &str) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_folio"));
    cmd.arg("build").arg("--dir").arg(dir).arg("--out").arg(out);
    cmd.arg(mode).output().expect("folio を起動できない")
}

/// `html[start..]` の span の開きから、入れ子を数えて対の閉じまで。
fn balanced_span(html: &str, start: usize, what: &str) -> String {
    let mut depth = 0usize;
    let mut i = start;
    loop {
        let rest = &html[i..];
        let open = rest.find("<span");
        let close = rest
            .find("</span>")
            .unwrap_or_else(|| panic!("{what}: span が閉じない"));
        match open {
            Some(o) if o < close => {
                depth += 1;
                i += o + "<span".len();
            }
            _ => {
                depth -= 1;
                i += close + "</span>".len();
                if depth == 0 {
                    return html[start..i].to_string();
                }
            }
        }
    }
}

/// 面の中の ceiling-stamp の span（ちょうど 1 つ・中の小窓を含む）。
fn stamp_of(html: &str, what: &str) -> String {
    assert_eq!(
        html.matches(STAMP_OPEN).count(),
        1,
        "{what}: ceiling-stamp が 1 つでない"
    );
    balanced_span(html, html.find(STAMP_OPEN).unwrap(), what)
}

/// 名札の中の小窓の本体の中身（小窓はちょうど 1 つ）。
fn hint_body_of(stamp: &str, what: &str) -> String {
    assert_eq!(
        stamp.matches(HINT_OPEN).count(),
        1,
        "{what}: 名札の中の小窓が 1 つでない: {stamp}"
    );
    let at = stamp
        .find(HINT_BODY_OPEN)
        .unwrap_or_else(|| panic!("{what}: 小窓の本体が無い: {stamp}"));
    let body = balanced_span(stamp, at, what);
    body[HINT_BODY_OPEN.len()..body.len() - "</span>".len()].to_string()
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
fn badge_stamp_is_verbatim_for_pass_fail_unknown() {
    let td = temp_dir("verbatim");
    let work = face_copy(&td);
    let out = td.join("index.html");

    // 合格 ×4・日付・束の要約値
    put_mark(&work, &mark_with(["合格", "合格", "合格", "合格"]));
    let html = write_face("index", None, &work, &out);
    assert_once(&html, STAMP_PASS, "合格 ×4");

    // 不合格
    put_mark(&work, &mark_with(["不合格", "合格", "合格", "合格"]));
    let html = write_face("index", None, &work, &out);
    assert_once(
        &html,
        &STAMP_PASS.replacen("忠実さ 合格", "忠実さ 不合格", 1),
        "不合格",
    );

    // まだ分からない
    put_mark(&work, &mark_with(["まだ分からない", "合格", "合格", "合格"]));
    let html = write_face("index", None, &work, &out);
    assert_once(
        &html,
        &STAMP_PASS.replacen("忠実さ 合格", "忠実さ まだ分からない", 1),
        "まだ分からない",
    );
    let _ = fs::remove_dir_all(&td);
}

// ── 2. 未実施 ──

#[test]
fn badge_stamp_says_not_run_without_the_mark() {
    let td = temp_dir("none");
    let work = face_copy(&td);
    let html = write_face("index", None, &work, &td.join("index.html"));
    let _ = fs::remove_dir_all(&td);
    assert_once(&html, STAMP_NONE, "印なし");
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
    let dir = real_copy(&td, &mark_with(["合格", "合格", "合格", "合格"]));
    let site = td.join("site");
    let run = build(&dir, &site, "--write");
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

    assert_code(&run, "folio build --write", 0);
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
fn badge_check_matches_the_same_mark_and_drifts_on_another() {
    let td = temp_dir("check");
    let dir = real_copy(&td, &mark_with(["合格", "合格", "合格", "合格"]));
    let site = td.join("site");
    let write = build(&dir, &site, "--write");
    let ok = build(&dir, &site, "--check");
    put_mark(&dir, &mark_with(["不合格", "合格", "合格", "合格"]));
    let drift = build(&dir, &site, "--check");
    fs::remove_file(dir.join(MARK)).unwrap();
    let none = build(&dir, &site, "--check");
    let _ = fs::remove_dir_all(&td);

    assert_code(&write, "folio build --write", 0);
    assert_code(&ok, "folio build --check（同じ印）", 0);
    assert!(
        stdout(&ok).starts_with("folio build: OK"),
        "{}",
        stdout(&ok)
    );
    assert_code(&drift, "folio build --check（別の印）", 1);
    assert!(stderr(&drift).contains("DRIFT"), "{}", stderr(&drift));
    for name in FIVE_FACES {
        assert!(stderr(&drift).contains(name), "{name}: {}", stderr(&drift));
    }
    assert_code(&none, "folio build --check（印なし）", 1);
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
    let dir = real_copy(&td, &mark_with(["合格", "合格", "合格", "合格"]));
    let site = td.join("site");
    let write = build(&dir, &site, "--write");
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
fn badge_faces_without_the_mark_match_the_seven_frozen_fixtures() {
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
        &write_face("index", None, &work, &td.join("index.html")),
        "expected-index.html",
    );
    same(
        &write_face("constitution", None, &work, &td.join("constitution.html")),
        "expected.html",
    );
    same(
        &write_face("srs", None, &work, &td.join("srs.html")),
        "expected-srs.html",
    );
    same(
        &write_face("note", Some("full"), &work, &td.join("note-full.html")),
        "expected-note.html",
    );
    same(
        &write_face("adr", Some("ADR-2"), &work, &td.join("adr-2.html")),
        "expected-site-adr-2.html",
    );
    // 支度表を置くと便 20 の期待
    fs::copy(
        fixture().join("intake-sheet.yaml"),
        work.join("intake-sheet.yaml"),
    )
    .unwrap();
    same(
        &write_face("index", None, &work, &td.join("index.html")),
        "expected-index-sheet.html",
    );
    // ADR-1 も置くと便 25 の期待（`tests/face_adr.rs` の写しと同じ集合・fixture の ADR-1 は欄が欠けるので入口の面の後）
    fs::copy(
        fixture().join("adr/ADR-1.yaml"),
        work.join("adr/ADR-1.yaml"),
    )
    .unwrap();
    same(
        &write_face("adr", Some("ADR-2"), &work, &td.join("adr-2.html")),
        "expected-adr.html",
    );
    let _ = fs::remove_dir_all(&td);
}

// ── 便 83（delivery-83.md §1 (e)）──

/// 5 面（入口・憲法・要件書・判断の記録 ADR-2・設計ノート full）を書き、面の名と本文を返す。
fn five_faces(work: &Path, td: &Path) -> Vec<(&'static str, String)> {
    [
        ("index", None, "index.html"),
        ("constitution", None, "constitution.html"),
        ("srs", None, "srs.html"),
        ("adr", Some("ADR-2"), "adr-2.html"),
        ("note", Some("full"), "note-full.html"),
    ]
    .into_iter()
    .map(|(face, id, name)| (name, write_face(face, id, work, &td.join(name))))
    .collect()
}

#[test]
fn f83_stamp_comes_from_the_mark() {
    let td = temp_dir("f83-mark");
    let work = face_copy(&td);
    // top-level の at は観点の at のどれとも違う値（名札の日付が印の top-level の at そのままであることを見る）
    let rows: Vec<(&str, &str, &str, &str)> = VIEWPOINTS
        .iter()
        .zip(["8dead31e", "22aa40dc", "af59959e", "53588227"])
        .map(|((id, _), b)| (*id, "合格", b, "2026-09-21T14:00:00Z"))
        .collect();
    put_mark(&work, &mark_text("2026-09-21T14:08:10Z", &rows));
    let pages = five_faces(&work, &td);
    let _ = fs::remove_dir_all(&td);

    let cells: Vec<String> = VIEWPOINTS
        .iter()
        .map(|(_, name)| format!("<b>{name} 合格</b>"))
        .collect();
    let digests: Vec<&str> = rows.iter().map(|r| r.2).collect();
    let want = format!(
        "{STAMP_OPEN}天井 {}（2026-09-21T14:08:10Z・束 {}）{HINT_OPEN}",
        cells.join(" · "),
        digests.join("/")
    );
    for (name, html) in &pages {
        assert!(
            stamp_of(html, name).starts_with(&want),
            "{name}: 名札が印の値と違う\n--- 期待\n{want}\n--- 面\n{}",
            stamp_of(html, name)
        );
    }
}

#[test]
fn f83_stamp_says_not_run_without_the_mark() {
    let td = temp_dir("f83-none");
    let work = face_copy(&td);
    assert!(!work.join(MARK).exists());
    let pages = five_faces(&work, &td);
    let _ = fs::remove_dir_all(&td);
    for (name, html) in &pages {
        assert!(
            stamp_of(html, name).starts_with(STAMP_NONE),
            "{name}: {}",
            stamp_of(html, name)
        );
    }
}

#[test]
fn f83_stamp_is_unknown_when_the_mark_is_broken() {
    let td = temp_dir("f83-broken");
    let work = face_copy(&td);
    let text = mark_with(["合格", "合格", "合格", "合格"]).replacen(
        "{id: readability, verdict: 合格, ",
        "{id: readability, ",
        1,
    );
    assert!(!text.contains("{id: readability, verdict"));
    put_mark(&work, &text);
    let out = td.join("index.html");
    let run = run_face("index", None, &work, &out);
    let wrote = out.exists();
    let _ = fs::remove_dir_all(&td);

    assert_code(&run, "folio face（verdict の欠けた印）", 2);
    assert!(!wrote, "まだ分からないのに面を書いた");
    let err = stderr(&run);
    assert!(err.contains(MARK), "印の相対 path が無い: {err}");
    assert!(err.contains("verdict"), "読めなかった欄の名が無い: {err}");
}

#[test]
fn f83_stamp_refuses_a_mark_that_disagrees_with_the_source() {
    let td = temp_dir("f83-skew");
    let work = face_copy(&td);
    let text = mark_with(["合格", "合格", "合格", "合格"]).replacen(
        "{id: readability,",
        "{id: clarity,",
        1,
    );
    put_mark(&work, &text);
    let out = td.join("index.html");
    let run = run_face("index", None, &work, &out);
    let _ = fs::remove_dir_all(&td);

    assert_code(&run, "folio face（正本と食い違う印）", 2);
    assert!(
        stderr(&run).contains("clarity"),
        "食い違った id が無い: {}",
        stderr(&run)
    );
}

#[test]
fn f83_the_ceiling_flag_is_gone() {
    let td = temp_dir("f83-flag");
    let work = face_copy(&td);
    let face = folio(&[
        &"face",
        &"--face",
        &"index",
        &"--dir",
        &work,
        &"--out",
        &td.join("index.html"),
        &"--ceiling",
        &td,
        &"--write",
    ]);
    let build = folio(&[
        &"build",
        &"--dir",
        &work,
        &"--out",
        &td.join("site"),
        &"--ceiling",
        &td,
        &"--write",
    ]);
    let _ = fs::remove_dir_all(&td);
    for (what, run) in [("face", &face), ("build", &build)] {
        assert_ne!(code(run, what), 0, "{what}: --ceiling を受け付けた");
        let err = stderr(run);
        assert!(
            err.contains("unexpected argument") && err.contains("--ceiling"),
            "{what}: 未知の引数の旨が無い: {err}"
        );
    }
}

#[test]
fn f83_five_faces_carry_the_same_hint() {
    let td = temp_dir("f83-hint");
    let work = face_copy(&td);
    put_mark(&work, &mark_with(["合格", "合格", "合格", "合格"]));
    let pages = five_faces(&work, &td);
    let _ = fs::remove_dir_all(&td);

    // 語彙の付録の章 = 憲法の面で crumb が「用語集」の section の id（生成器の付録の表と独立に面から引く）
    let constitution = &pages
        .iter()
        .find(|(name, _)| *name == "constitution.html")
        .unwrap()
        .1;
    let chapter = constitution
        .split("<section id=\"")
        .skip(1)
        .find(|chunk| {
            chunk
                .split("</p>")
                .next()
                .is_some_and(|head| head.contains(" 用語集 "))
        })
        .and_then(|chunk| chunk.split('"').next())
        .expect("憲法の面に用語集の章が無い")
        .to_string();
    let link = format!("<a href=\"constitution.html#{chapter}\">");

    let bodies: Vec<String> = pages
        .iter()
        .map(|(name, html)| hint_body_of(&stamp_of(html, name), name))
        .collect();
    for ((name, _), body) in pages.iter().zip(&bodies) {
        assert_eq!(body, &bodies[0], "{name}: 小窓の本体が index.html と違う");
    }
    let body = &bodies[0];
    for word in ["合格", "不合格", "まだ分からない"] {
        assert!(body.contains(word), "小窓に「{word}」が無い: {body}");
    }
    assert!(
        body.contains(&link),
        "小窓に用語集への導線 {link} が無い: {body}"
    );
}
