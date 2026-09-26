//! `folio build` の歯（便 17・docs/design/delivery-17.md §1 (d)）。binary 経由。
//! - 凍結 fixture（tests/fixtures/face/）との byte 一致（P-10.1）
//! - check の 3 値（一致 0・DRIFT 1・無い 2・空の配信先 2）
//! - 実の正本で `folio parts --check` と 各面（設計ノートの面を含む）の `folio face --check` に合格する（AC2 の機構）
//! - 全部か無しか（導出できなければ配信先の dir も作らない）・親 dir が無い・配信先の他の file を消さない
//! - 構造の床（便 56・delivery-56.md §1 (b)・FR5）: `--write` は最初に床を回す。合格の写し（git の 1 commit）は 0・
//!   不合格の写しは書かず 1・既に在る配信先は不変・版管理の無い写しは書いて 2・`--check` は床と無関係
//!
//! 凍結 fixture（tests/fixtures/face/）の写しは正本が揃っていない（adr/schema.yaml 等が無い）ので床は「まだ分からない」
//! ＝ `--write` は書いて 2。版管理の `design-intent/preview/` は書き換えない（`--out` は必ず一時 dir の中）。

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

    // 凍結 fixture の写しは床が「まだ分からない」（便 56）= 書いて 2・1 行目は床の行
    assert_eq!(code(&run, "folio build --write"), 2, "{}", stderr(&run));
    let total: usize = got.iter().map(Vec::len).sum();
    let out = stdout(&run);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "{out}");
    assert!(
        lines[0].starts_with("folio build: 床 = まだ分からない（"),
        "{out}"
    );
    assert_eq!(
        lines[1],
        format!("folio build: 書いた（7 file・{total} byte）")
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

    // 凍結 fixture の写しは床が「まだ分からない」（便 56）= 書いて 2
    assert_eq!(code(&write, "write"), 2, "{}", stderr(&write));
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
    assert_eq!(code(&rewrite, "write（再）"), 2, "{}", stderr(&rewrite));
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

/// `adr/` の下に実在する判断の記録の番号（`ADR-<数>.yaml` の数・昇順）。
/// 廃止は状態で表し番号を空けたままにする（P-7.2）ので、連番（1..=本数）とは仮定しない。
fn record_ids(adr: &Path) -> Vec<u32> {
    let mut ids: Vec<u32> = fs::read_dir(adr)
        .unwrap()
        .filter_map(|e| {
            let name = e.unwrap().file_name().to_string_lossy().into_owned();
            name.strip_prefix("ADR-")?
                .strip_suffix(".yaml")?
                .parse()
                .ok()
        })
        .collect();
    ids.sort_unstable();
    ids
}

/// 判断の記録の面の期待の file 名（実在する番号ごとに `adr-<数>.html`・番号の昇順）。
fn adr_page_names(ids: &[u32]) -> Vec<String> {
    ids.iter().map(|n| format!("adr-{n}.html")).collect()
}

/// 実の正本の判断の記録の番号（`design-intent/adr/` の下）。
fn real_records() -> Vec<u32> {
    record_ids(&design_intent().join("adr"))
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
    let ids = real_records();
    let records = ids.len();
    let notes = real_notes();
    // 3 面 + 様式 2 本 + 判断の記録の面（記録の数だけ）+ 設計ノートの面（設計ノートの数だけ）
    let total = 3 + 2 + records + notes.len();
    let all_adr = adr_page_names(&ids)
        .iter()
        .all(|name| site.join(name).is_file());
    let last_id = *ids.last().expect("判断の記録が 1 本も無い");
    let all_notes = notes
        .iter()
        .all(|id| site.join(format!("note-{id}.html")).is_file());
    let last = format!("ADR-{last_id}");
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
        ("adr", format!("adr-{last_id}.html")),
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
            .arg(site.join(format!("adr-{last_id}.html")))
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
    // 実の正本は版管理の中に在り床が合格する（便 56）= 1 行目が床の合格・2 行目が「書いた」
    let out = stdout(&build);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "{out}");
    assert_eq!(
        lines[0],
        "folio build: 床 = 合格（違反 0・まだ分からない 0）"
    );
    assert!(
        lines[1].starts_with(&format!("folio build: 書いた（{total} file・")),
        "{out}"
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

/// 欠番のある写し（ADR-1 と ADR-3 だけ・ADR-2 を欠く）でも、面の期待は実在する番号の集合から組まれ、
/// 組み立ての写しはその期待どおりの面を出す（P-7.2・f2-648.175）。
#[test]
fn site_expects_adr_faces_from_the_existing_ids_not_a_sequence() {
    let (td, work) = fixture_copy("gap");
    let adr = work.join("adr");
    let base = fs::read_to_string(adr.join("ADR-2.yaml")).unwrap();
    assert_eq!(
        base.matches("ADR-2").count(),
        1,
        "写しの元の番号の出現が id の 1 か所でない"
    );
    fs::remove_file(adr.join("ADR-2.yaml")).unwrap();
    for n in [1, 3] {
        fs::write(
            adr.join(format!("ADR-{n}.yaml")),
            base.replace("id: ADR-2", &format!("id: ADR-{n}")),
        )
        .unwrap();
    }
    let ids = record_ids(&adr);
    let names = adr_page_names(&ids);
    let site = td.join("site");
    let run = folio_build(&work, &site, "--write");
    let present: Vec<bool> = names.iter().map(|n| site.join(n).is_file()).collect();
    let missing_gap = !site.join("adr-2.html").exists();
    let _ = fs::remove_dir_all(&td);

    assert_eq!(ids, vec![1, 3]);
    assert_eq!(names, vec!["adr-1.html", "adr-3.html"]);
    // 凍結 fixture の写しは床が「まだ分からない」（便 56）= 書いて 2
    assert_eq!(code(&run, "folio build --write"), 2, "{}", stderr(&run));
    assert_eq!(present, vec![true, true], "{}", stdout(&run));
    assert!(missing_gap, "欠番の面 adr-2.html が出ている");
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

    // 凍結 fixture の写しは床が「まだ分からない」（便 56）= 書いて 2
    assert_eq!(code(&run, "folio build --write"), 2, "{}", stderr(&run));
    assert_eq!(extra, "手で置いた file\n", "配信先の他の file を消した");
    assert!(all, "7 本が揃っていない");
}

// ── 構造の床（便 56・FR5）──

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

/// 実の設計文書の置き場 design-intent と器の導出 file と図の道具を一時 dir へ写す。`commit` なら git の 1 commit にする
/// （床が合格する写し・tests/schema.rs の Work と同じ形）。戻り値 = (一時 dir, 正本の写し)。
fn real_copy(case: &str, commit: bool) -> (PathBuf, PathBuf) {
    let td = temp_dir(case);
    let dir = td.join("design-intent");
    copy_dir(&design_intent(), &dir);
    fs::create_dir_all(td.join("contracts")).unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        td.join("contracts/schema.toml"),
    )
    .unwrap();
    copy_dir(&vendor(), &td.join("vendor/archify"));
    if commit {
        git(&td, &["init", "-q"]);
        git(&td, &["add", "-A"]);
        git(&td, &["commit", "-q", "-m", "fixture"]);
    }
    (td, dir)
}

/// 写しの設計ノートの散文に、規範の印（R-16 の marks）を持ち参照 id の無い文を 1 つ足して commit する。
fn add_prose_violation(td: &Path, dir: &Path) {
    let path = dir.join("design-note/example.yaml");
    let before = fs::read_to_string(&path).unwrap();
    let needle =
        "      検査を通らなかった図は生成せず、前の生成物も上書きしない（FR15・P-4.1）。\n";
    assert_eq!(
        before.matches(needle).count(),
        1,
        "変異の当て先が 1 か所でない"
    );
    let after = before.replacen(
        needle,
        &format!("{needle}      検査を通らなかった図の生成物は消去しなければならない。\n"),
        1,
    );
    fs::write(&path, after).unwrap();
    git(td, &["add", "-A"]);
    git(td, &["commit", "-q", "-m", "violation"]);
}

fn folio_check(dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("check")
        .arg("--dir")
        .arg(dir)
        .output()
        .expect("folio を起動できない")
}

/// dir の直下の全 file の名と中身（名の順）。dir が無ければ空。
fn site_tree(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<(String, Vec<u8>)> = entries
        .map(|e| {
            let e = e.unwrap();
            (
                e.file_name().to_string_lossy().into_owned(),
                fs::read(e.path()).unwrap(),
            )
        })
        .collect();
    out.sort();
    out
}

#[test]
fn site_floor_pass_writes_and_exits_zero() {
    let (td, dir) = real_copy("floor-pass", true);
    let site = td.join("site");
    let run = folio_build(&dir, &site, "--write");
    let index = site.join("index.html").is_file();
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&run, "folio build --write"), 0, "{}", stderr(&run));
    let out = stdout(&run);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "{out}");
    assert!(lines[0].starts_with("folio build: 床 = 合格"), "{out}");
    assert!(lines[1].starts_with("folio build: 書いた"), "{out}");
    assert!(index, "面が書かれていない");
}

#[test]
fn site_floor_fail_writes_nothing_and_exits_one() {
    let (td, dir) = real_copy("floor-fail", true);
    add_prose_violation(&td, &dir);
    let check = folio_check(&dir);
    let site = td.join("site");
    let run = folio_build(&dir, &site, "--write");
    let exists = site.exists();
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&check, "folio check"), 1, "{}", stdout(&check));
    assert_eq!(code(&run, "folio build --write"), 1, "{}", stderr(&run));
    let out = stdout(&run);
    assert_eq!(out.lines().count(), 1, "{out}");
    assert!(out.contains("床 = 不合格"), "{out}");
    assert!(out.contains("書かない"), "{out}");
    assert!(!exists, "床が不合格なのに配信先の dir を作った");
}

#[test]
fn site_floor_fail_leaves_an_existing_site_untouched() {
    let (td, dir) = real_copy("floor-keep", true);
    let site = td.join("site");
    let first = folio_build(&dir, &site, "--write");
    let before = site_tree(&site);
    add_prose_violation(&td, &dir);
    let second = folio_build(&dir, &site, "--write");
    let after = site_tree(&site);
    let _ = fs::remove_dir_all(&td);

    assert_eq!(
        code(&first, "folio build --write（合格）"),
        0,
        "{}",
        stderr(&first)
    );
    assert!(!before.is_empty(), "合格の配信先が空");
    assert_eq!(
        code(&second, "folio build --write（不合格）"),
        1,
        "{}",
        stderr(&second)
    );
    assert!(
        stdout(&second).contains("床 = 不合格"),
        "{}",
        stdout(&second)
    );
    assert!(
        before == after,
        "床が不合格なのに既に在る配信先の file を変えた"
    );
}

#[test]
fn site_floor_unknown_without_version_control_writes_and_exits_two() {
    let (td, dir) = real_copy("floor-nogit", false);
    let site = td.join("site");
    let run = folio_build(&dir, &site, "--write");
    let index = site.join("index.html").is_file();
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&run, "folio build --write"), 2, "{}", stderr(&run));
    let out = stdout(&run);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "{out}");
    assert!(
        lines[0].starts_with("folio build: 床 = まだ分からない"),
        "{out}"
    );
    assert!(lines[1].starts_with("folio build: 書いた"), "{out}");
    assert!(index, "面が書かれていない");
}

#[test]
fn site_floor_check_does_not_run_the_floor() {
    let (td, dir) = real_copy("floor-check", true);
    let site = td.join("site");
    let write = folio_build(&dir, &site, "--write");
    let ok = folio_build(&dir, &site, "--check");
    add_prose_violation(&td, &dir);
    let drift = folio_build(&dir, &site, "--check");
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&write, "folio build --write"), 0, "{}", stderr(&write));
    assert_eq!(
        code(&ok, "folio build --check（一致）"),
        0,
        "{}",
        stderr(&ok)
    );
    assert!(
        stdout(&ok).starts_with("folio build: OK"),
        "{}",
        stdout(&ok)
    );
    assert!(!stdout(&ok).contains("床"), "{}", stdout(&ok));
    // 正本が変わったので不一致の 1（床の 3 値とは無関係・出力にも床の行は無い）
    assert_eq!(
        code(&drift, "folio build --check（不一致）"),
        1,
        "{}",
        stderr(&drift)
    );
    assert!(stderr(&drift).contains("DRIFT"), "{}", stderr(&drift));
    assert!(
        stderr(&drift).contains("note-example.html"),
        "{}",
        stderr(&drift)
    );
    assert!(stdout(&drift).is_empty(), "{}", stdout(&drift));
    assert!(!stderr(&drift).contains("床"), "{}", stderr(&drift));
}

/// 便 136（docs/design/delivery-136.md §1 (c) の 3）: 規則の表の行 R-1 の key と id を二重引用符つきにして commit した写しは、
/// 索引が組めないので build の床が不合格で何も書かない（配信先の dir を作らない）。
#[test]
fn f136_quoted_rule_row_stops_the_build_floor() {
    let (td, dir) = real_copy("f136-rule-row", false);
    let path = dir.join("rules.yaml");
    let before = fs::read_to_string(&path).unwrap();
    let from = "  - {id: R-1, ";
    assert_eq!(before.matches(from).count(), 1, "変異の当て先が 1 か所でない");
    fs::write(&path, before.replacen(from, "  - {\"id\": \"R-1\", ", 1)).unwrap();
    git(&td, &["init", "-q"]);
    git(&td, &["add", "-A"]);
    git(&td, &["commit", "-q", "-m", "fixture"]);
    let site = td.join("site");
    let run = folio_build(&dir, &site, "--write");
    let exists = site.exists();
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&run, "folio build --write"), 1, "{}", stderr(&run));
    assert!(stdout(&run).contains("床 = 不合格（違反 1・"), "{}", stdout(&run));
    assert!(!exists, "床が不合格なのに配信先の dir を作った");
}

/// 便 153（docs/design/delivery-153.md §1 (c) の 2・ADR-27 決定 (2)）: 置き場の様式が在ればその字を出し、無いときだけ
/// 組み立て時に焼いた folio2 の様式を出す。在るのに読めない様式（dir・壊れた symlink・途中が file）は まだ分からない で
/// 配信先を作らない。
#[test]
fn f153_build_falls_back_to_the_baked_style_only_when_the_place_has_none() {
    let (td, work) = fixture_copy("f153-baked");
    let preview = work.join("preview");
    let repo_style = |name: &str| fs::read(design_intent().join("preview").join(name)).unwrap();

    // 1. 置き場の様式（凍結 fixture の最小の様式・repo の正本と違う字）が在れば、その字を出す
    let site = td.join("site-own");
    let run = folio_build(&work, &site, "--write");
    assert_eq!(code(&run, "folio build --write"), 2, "{}", stderr(&run));
    for name in ["folio.css", "folio-ui.js"] {
        let own = fs::read(fixture().join(name)).unwrap();
        assert!(own != repo_style(name), "{name}: fixture が repo の正本と同じ");
        assert!(
            fs::read(site.join(name)).unwrap() == own,
            "{name}: 置き場の様式を出していない"
        );
    }

    // 2. 2 本を消すと 7 file を書き、2 本とも repo の正本と byte で同じ・--check は 0
    for name in ["folio.css", "folio-ui.js"] {
        fs::remove_file(preview.join(name)).unwrap();
    }
    let site = td.join("site-baked");
    let run = folio_build(&work, &site, "--write");
    assert_eq!(code(&run, "folio build --write"), 2, "{}", stderr(&run));
    assert!(
        stdout(&run).contains("folio build: 書いた（7 file・"),
        "{}{}",
        stdout(&run),
        stderr(&run)
    );
    for name in ["folio.css", "folio-ui.js"] {
        assert!(
            fs::read(site.join(name)).unwrap() == repo_style(name),
            "{name}: 焼いた様式が repo の正本と byte で違う"
        );
    }
    let check = folio_build(&work, &site, "--check");
    assert_eq!(code(&check, "folio build --check"), 0, "{}", stderr(&check));

    // 3・4. folio.css が dir か壊れた symlink なら 2 で「folio.css: 読めない」・配信先を作らない
    let css = preview.join("folio.css");
    fs::create_dir(&css).unwrap();
    let dir_site = td.join("site-dir");
    let dir_run = folio_build(&work, &dir_site, "--write");
    fs::remove_dir(&css).unwrap();
    std::os::unix::fs::symlink(td.join("no-such.css"), &css).unwrap();
    let link_site = td.join("site-link");
    let link_run = folio_build(&work, &link_site, "--write");
    fs::remove_file(&css).unwrap();
    for (case, run, out) in [("dir", &dir_run, &dir_site), ("symlink", &link_run, &link_site)] {
        assert_eq!(code(run, "folio build --write"), 2, "{case}: {}", stderr(run));
        assert!(
            stderr(run).contains("folio.css: 読めない"),
            "{case}: {}",
            stderr(run)
        );
        assert!(!out.exists(), "{case}: 読めない様式で配信先を作った");
    }

    // 5. preview が file（途中が file）なら 2 で配信先を作らない
    fs::remove_dir_all(&preview).unwrap();
    fs::write(&preview, "file\n").unwrap();
    let file_site = td.join("site-file");
    let file_run = folio_build(&work, &file_site, "--write");
    let file_exists = file_site.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        code(&file_run, "folio build --write"),
        2,
        "{}",
        stderr(&file_run)
    );
    assert!(!file_exists, "preview が file なのに配信先を作った");
}
