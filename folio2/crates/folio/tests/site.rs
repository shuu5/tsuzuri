//! `folio build` の歯（便 17・docs/design/delivery-17.md §1 (d)）。binary 経由。
//! - 凍結 fixture（tests/fixtures/face/）との byte 一致（P-10.1）
//! - check の 3 値（一致 0・DRIFT 1・無い 2・空の配信先 2）
//! - 実の正本で `folio parts --check` と 各面（設計ノートの面を含む）の `folio face --check` に合格する（AC2 の機構）
//! - 全部か無しか（導出できなければ配信先の dir も作らない）・親 dir が無い・配信先の他の file を消さない
//!
//! 版管理の `design-intent/preview/` は書き換えない（`--out` は必ず一時 dir の中）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture() -> PathBuf {
    repo_root().join("tests/fixtures/face")
}

fn design_intent() -> PathBuf {
    repo_root().join("design-intent")
}

fn vendor() -> PathBuf {
    repo_root().join("vendor/archify")
}

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-site-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    fs::create_dir_all(&td).unwrap();
    td
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

/// 凍結 fixture の正本 6 file と adr/・design-note/ を一時 dir の src/ へ写し、src/preview/ に最小の様式 2 本を、
/// src/ の親 dir に器の導出 file と図の道具（vendor/archify・便 31 の設計ノートの面が図ごとに撃つ）を置く。
/// 戻り値 = (一時 dir, 正本の写し)。支度表 intake-sheet.yaml と期待の面 3 本は写さない。
fn fixture_copy(case: &str) -> (PathBuf, PathBuf) {
    let td = temp_dir(case);
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
    ] {
        fs::copy(fixture().join(name), work.join(name)).unwrap();
    }
    // 判断の記録の面も出す（便 26）ので、写す記録は欄の揃った便 25 の 1 本
    fs::copy(
        fixture().join("adr/ADR-2.yaml"),
        work.join("adr/ADR-2.yaml"),
    )
    .unwrap();
    // 設計ノートの面も出す（便 29）ので、写す設計ノートは欄の揃った便 28 の 1 本。
    // その面は契約表の節を持つので、器の導出 file を写しの src/ の親 dir へも置く
    fs::copy(
        fixture().join("design-note/full.yaml"),
        work.join("design-note/full.yaml"),
    )
    .unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        td.join("contracts/schema.toml"),
    )
    .unwrap();
    for name in ["folio.css", "folio-ui.js"] {
        fs::copy(fixture().join(name), work.join("preview").join(name)).unwrap();
    }
    copy_dir(&vendor(), &td.join("vendor/archify"));
    (td, work)
}

fn folio_build(dir: &Path, out: &Path, mode: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("build")
        .arg("--dir")
        .arg(dir)
        .arg("--out")
        .arg(out)
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

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// 配信先へ出る 7 本（順もこのとおり・凍結 fixture の写しは判断の記録 1 本と設計ノート 1 本）。
const SITE_FILES: [&str; 7] = [
    "index.html",
    "constitution.html",
    "srs.html",
    "folio.css",
    "folio-ui.js",
    "adr-2.html",
    "note-full.html",
];

// ── 凍結 fixture ──

#[test]
fn site_write_matches_the_frozen_fixture() {
    let (td, work) = fixture_copy("anchor");
    let site = td.join("site");
    let run = folio_build(&work, &site, "--write");
    let got: Vec<Vec<u8>> = SITE_FILES
        .iter()
        .map(|name| fs::read(site.join(name)).unwrap_or_default())
        .collect();
    let want: Vec<Vec<u8>> = [
        "expected-index.html",
        "expected.html",
        "expected-srs.html",
        "folio.css",
        "folio-ui.js",
        // 組み立ての写しは ADR-2 だけなので、便 25 の expected-adr.html とは byte が違う（独立 anchor）
        "expected-site-adr-2.html",
        // 設計ノートの面の入力は正本と憲法・rules・要件書・器の導出 file だけなので、便 28 の凍結と byte 一致
        "expected-note.html",
    ]
    .iter()
    .map(|name| fs::read(fixture().join(name)).unwrap())
    .collect();
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&run, "folio build --write"), 0, "{}", stderr(&run));
    let total: usize = got.iter().map(Vec::len).sum();
    assert_eq!(
        stdout(&run),
        format!("folio build: 書いた（7 file・{total} byte）\n")
    );
    for (i, name) in SITE_FILES.iter().enumerate() {
        assert!(!got[i].is_empty(), "{name} が配信先に無い");
        assert_eq!(
            got[i].len(),
            want[i].len(),
            "{name} の大きさが凍結 fixture と違う"
        );
        assert!(got[i] == want[i], "{name} が凍結 fixture と byte で違う");
    }
}

// ── check の 3 値 ──

#[test]
fn site_check_has_three_values() {
    let (td, work) = fixture_copy("check");
    let site = td.join("site");

    let write = folio_build(&work, &site, "--write");
    let ok = folio_build(&work, &site, "--check");
    for name in ["srs.html", "adr-2.html", "note-full.html"] {
        let mut bytes = fs::read(site.join(name)).unwrap();
        bytes[0] ^= 0x20;
        fs::write(site.join(name), &bytes).unwrap();
    }
    let drift = folio_build(&work, &site, "--check");
    let rewrite = folio_build(&work, &site, "--write");
    fs::remove_file(site.join("folio.css")).unwrap();
    let missing = folio_build(&work, &site, "--check");
    let empty = td.join("empty");
    fs::create_dir(&empty).unwrap();
    let nothing = folio_build(&work, &empty, "--check");
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&write, "write"), 0, "{}", stderr(&write));
    assert_eq!(code(&ok, "check（一致）"), 0, "{}", stderr(&ok));
    assert!(
        stdout(&ok).starts_with("folio build: OK — 配信先は正本と一致（7 file・"),
        "{}",
        stdout(&ok)
    );
    assert_eq!(code(&drift, "check（不一致）"), 1, "{}", stderr(&drift));
    assert!(stderr(&drift).contains("DRIFT"), "{}", stderr(&drift));
    assert!(stderr(&drift).contains("srs.html"), "{}", stderr(&drift));
    assert!(stderr(&drift).contains("adr-2.html"), "{}", stderr(&drift));
    assert!(
        stderr(&drift).contains("note-full.html"),
        "{}",
        stderr(&drift)
    );
    assert_eq!(code(&rewrite, "write（再）"), 0, "{}", stderr(&rewrite));
    assert_eq!(code(&missing, "check（無い）"), 2, "{}", stderr(&missing));
    assert!(
        stderr(&missing).contains("配信先に無い"),
        "{}",
        stderr(&missing)
    );
    assert!(
        stderr(&missing).contains("folio.css"),
        "{}",
        stderr(&missing)
    );
    assert_eq!(
        code(&nothing, "check（空の配信先）"),
        2,
        "{}",
        stderr(&nothing)
    );
}

// ── 実の正本（AC2 の機構）──

/// 実の正本の判断の記録の数（`adr/` の下の `ADR-<数>.yaml` の本数）。
fn real_records() -> usize {
    fs::read_dir(design_intent().join("adr"))
        .unwrap()
        .filter(|e| {
            let name = e.as_ref().unwrap().file_name();
            let name = name.to_string_lossy();
            name.starts_with("ADR-") && name.ends_with(".yaml")
        })
        .count()
}

/// 実の正本の設計ノートの id（`design-note/` の下の `.yaml` から欄の決まりを除いた stem・字の昇順）。
fn real_notes() -> Vec<String> {
    let mut ids: Vec<String> = fs::read_dir(design_intent().join("design-note"))
        .unwrap()
        .filter_map(|e| {
            let name = e.unwrap().file_name().to_string_lossy().into_owned();
            let id = name.strip_suffix(".yaml")?;
            (name != "schema.yaml").then(|| id.to_string())
        })
        .collect();
    ids.sort();
    ids
}

#[test]
fn site_on_the_real_sources_passes_parts_check_and_face_check() {
    let td = temp_dir("real");
    let site = td.join("site");
    let build = folio_build(&design_intent(), &site, "--write");
    let records = real_records();
    let notes = real_notes();
    // 3 面 + 様式 2 本 + 判断の記録の面（記録の数だけ）+ 設計ノートの面（設計ノートの数だけ）
    let total = 3 + 2 + records + notes.len();
    let adr_pages: Vec<PathBuf> = (1..=records)
        .map(|n| site.join(format!("adr-{n}.html")))
        .collect();
    let all_adr = adr_pages.iter().all(|p| p.is_file());
    let all_notes = notes
        .iter()
        .all(|id| site.join(format!("note-{id}.html")).is_file());
    let last = format!("ADR-{records}");
    let last_note = notes.last().cloned().expect("設計ノートが 1 本も無い");
    let mut pages = Command::new(env!("CARGO_BIN_EXE_folio"));
    pages
        .arg("parts")
        .arg("--check")
        .arg("--dir")
        .arg(design_intent());
    for (face, name) in [
        ("index", "index.html".to_string()),
        ("constitution", "constitution.html".to_string()),
        ("srs", "srs.html".to_string()),
        ("adr", format!("adr-{records}.html")),
        ("note", format!("note-{last_note}.html")),
    ] {
        pages
            .arg("--page")
            .arg(format!("{face}={}", site.join(name).display()));
    }
    let parts = pages.output().unwrap();
    let mut faces: Vec<(&str, Output)> = [
        ("index", "index.html"),
        ("constitution", "constitution.html"),
        ("srs", "srs.html"),
    ]
    .iter()
    .map(|(face, name)| {
        let out = Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("face")
            .arg("--face")
            .arg(face)
            .arg("--dir")
            .arg(design_intent())
            .arg("--out")
            .arg(site.join(name))
            .arg("--check")
            .output()
            .unwrap();
        (*face, out)
    })
    .collect();
    faces.push((
        "adr",
        Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("face")
            .arg("--face")
            .arg("adr")
            .arg("--id")
            .arg(&last)
            .arg("--dir")
            .arg(design_intent())
            .arg("--out")
            .arg(site.join(format!("adr-{records}.html")))
            .arg("--check")
            .output()
            .unwrap(),
    ));
    for id in &notes {
        faces.push((
            "note",
            Command::new(env!("CARGO_BIN_EXE_folio"))
                .arg("face")
                .arg("--face")
                .arg("note")
                .arg("--id")
                .arg(id)
                .arg("--dir")
                .arg(design_intent())
                .arg("--out")
                .arg(site.join(format!("note-{id}.html")))
                .arg("--check")
                .output()
                .unwrap(),
        ));
    }
    let _ = fs::remove_dir_all(&td);

    assert_eq!(
        code(&build, "folio build（実の正本）"),
        0,
        "{}",
        stderr(&build)
    );
    assert!(
        stdout(&build).starts_with(&format!("folio build: 書いた（{total} file・")),
        "{}",
        stdout(&build)
    );
    assert!(all_adr, "判断の記録の面が {records} 枚そろっていない");
    assert!(
        all_notes,
        "設計ノートの面が {} 枚そろっていない",
        notes.len()
    );
    assert_eq!(
        code(&parts, "folio parts --check"),
        0,
        "{}{}",
        stdout(&parts),
        stderr(&parts)
    );
    assert!(stdout(&parts).contains("違反 0"), "{}", stdout(&parts));
    for (face, out) in &faces {
        assert_eq!(
            code(out, "folio face --check"),
            0,
            "{face}: {}{}",
            stdout(out),
            stderr(out)
        );
        assert!(
            stdout(out).contains("folio face: OK"),
            "{face}: {}",
            stdout(out)
        );
    }
}

// ── 全部か無しか ──

#[test]
fn site_writes_nothing_when_a_face_cannot_be_derived() {
    let (td, work) = fixture_copy("all-or-nothing");
    // 便 16 の歯と同じ変異（counts を 1 ずらす）
    let path = work.join("constitution.yaml");
    let before = fs::read_to_string(&path).unwrap();
    let after = before.replacen(
        "{always: 1, ask-first: 1, never: 1}",
        "{always: 2, ask-first: 1, never: 1}",
        1,
    );
    assert_ne!(before, after, "変異が当たっていない");
    fs::write(&path, after).unwrap();
    let site = td.join("site");
    let run = folio_build(&work, &site, "--write");
    let exists = site.exists();
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&run, "folio build"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("まだ分からない"), "{}", stderr(&run));
    assert!(!exists, "導出できないのに配信先の dir を作った");
}

#[test]
fn site_writes_nothing_when_a_note_face_cannot_be_derived() {
    let (td, work) = fixture_copy("all-or-nothing-note");
    // 設計ノートの面の名札の表に無い状態（便 29）
    let path = work.join("design-note/full.yaml");
    let before = fs::read_to_string(&path).unwrap();
    let after = before.replacen("status: draft", "status: final", 1);
    assert_ne!(before, after, "変異が当たっていない");
    fs::write(&path, after).unwrap();
    let site = td.join("site");
    let run = folio_build(&work, &site, "--write");
    let exists = site.exists();
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&run, "folio build"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("まだ分からない"), "{}", stderr(&run));
    assert!(!exists, "導出できないのに配信先の dir を作った");
}

#[test]
fn site_is_unknown_when_the_out_parent_dir_is_missing() {
    let (td, work) = fixture_copy("out-parent");
    let site = td.join("no-such-dir/site");
    let run = folio_build(&work, &site, "--write");
    let exists = site.exists();
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&run, "folio build"), 2, "{}", stderr(&run));
    assert!(
        stderr(&run).contains("配信先の親 dir が無い"),
        "{}",
        stderr(&run)
    );
    assert!(!exists);
}

// ── 消さない（N-1.1）──

#[test]
fn site_write_keeps_the_other_files_in_the_out_dir() {
    let (td, work) = fixture_copy("keep");
    let site = td.join("site");
    fs::create_dir(&site).unwrap();
    fs::write(site.join("extra.txt"), "手で置いた file\n").unwrap();
    let run = folio_build(&work, &site, "--write");
    let extra = fs::read_to_string(site.join("extra.txt")).unwrap_or_default();
    let all = SITE_FILES.iter().all(|name| site.join(name).is_file());
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&run, "folio build --write"), 0, "{}", stderr(&run));
    assert_eq!(extra, "手で置いた file\n", "配信先の他の file を消した");
    assert!(all, "7 本が揃っていない");
}
