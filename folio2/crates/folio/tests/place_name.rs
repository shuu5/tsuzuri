//! 面と生成物の置き場の名の歯（便 154・docs/design/delivery-154.md §1 (c) の 2・FR7 / FR4 / FR1 / FR24）。binary 経由。
//! 置き場の名は憲法の meta.id の `<名>-constitution` の <名>（導けなければ名を出さない）。期待の字は歯の中の手書きで、
//! 生成器の関数は呼ばない（P-10.1）。
//! 外の置き場は一時 dir の根で git の init → `folio init --dir design-intent` → 憲法と入口の meta.id を替える →
//! 様式 3 本を repo の design-intent/preview/ から写す → commit、の後に撃つ（§1 (a) の 2）。
//! 1. 名を替えた置き場は自分の名を出し folio2 を出さない／2. 骨格のままなら名を出さない／
//! 3. folio2 自身の面・支度表・始まりの凍結の 3 file は folio2 の名のまま／4. 名の無い置き場の 5 種の面に名札も名も無い。
//!
//! 版管理の下の file は書き換えない（`--dir` の写しと `--out` は必ず一時 dir の中）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 様式 3 本（repo の design-intent/preview/ から写す）。
const STYLES: [&str; 3] = ["folio.css", "folio-ui.js", "parts.json"];
/// 外の置き場の面 4 枚（配信先の file 名・骨格の判断の記録は ADR-1）。
const FACES: [&str; 4] = ["index.html", "constitution.html", "srs.html", "adr-1.html"];
/// 外の置き場の面と支度表に 1 つも在ってはならない字。
const FORBIDDEN: [&str; 3] = ["folio2", "f2-", ">f2<"];
const FLOOR_BASE: &str = "tests/fixtures/floor_base/design-intent";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &to);
        } else {
            fs::copy(entry.path(), &to).unwrap();
        }
    }
}

/// git を呼ぶ。環境変数 GIT_* は継承しない。
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

fn folio(args: &[&str], dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(args)
        .arg("--dir")
        .arg(dir)
        .output()
        .expect("folio を起動できない")
}

fn both(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn edit(path: &Path, from: &str, to: &str) {
    let text = fs::read_to_string(path).unwrap();
    assert_eq!(text.matches(from).count(), 1, "{}: 「{from}」が 1 か所でない", path.display());
    fs::write(path, text.replacen(from, to, 1)).unwrap();
}

/// 一時 dir（歯の終わりに消す）。
struct Work {
    root: PathBuf,
}

impl Work {
    fn empty(case: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-f154-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        Work { root }
    }

    /// 外の置き場: git の init → init → `name` が在れば憲法と入口の meta.id を替える → 様式 3 本 → commit。
    fn outer(case: &str, name: Option<&str>) -> Work {
        let w = Work::empty(case);
        git(&w.root, &["init", "-q"]);
        let init = folio(&["init"], &w.place());
        assert_eq!(init.status.code(), Some(0), "{}", both(&init));
        if let Some(n) = name {
            edit(
                &w.place().join("constitution.yaml"),
                "\nmeta:\n  id: 未記入\n",
                &format!("\nmeta:\n  id: {n}-constitution\n"),
            );
            edit(
                &w.place().join("index.yaml"),
                "\nmeta:\n  id: index\n",
                &format!("\nmeta:\n  id: {n}-index\n"),
            );
        }
        fs::create_dir_all(w.place().join("preview")).unwrap();
        for s in STYLES {
            fs::copy(
                repo_root().join("design-intent/preview").join(s),
                w.place().join("preview").join(s),
            )
            .unwrap();
        }
        git(&w.root, &["add", "-A"]);
        git(&w.root, &["commit", "-q", "-m", "place"]);
        w
    }

    fn place(&self) -> PathBuf {
        self.root.join("design-intent")
    }

    fn site(&self) -> PathBuf {
        self.root.join("site")
    }

    /// `build --write` を撃ち、終了コードを確かめて配信先の面 `faces` を読む。
    fn build(&self, dir: &Path, code: i32, faces: &[&str]) -> Vec<(String, String)> {
        let out = folio(
            &["build", "--write", "--out", self.site().to_str().unwrap()],
            dir,
        );
        assert_eq!(out.status.code(), Some(code), "{}", both(&out));
        faces
            .iter()
            .map(|f| (f.to_string(), fs::read_to_string(self.site().join(f)).unwrap()))
            .collect()
    }

    /// `intake --write` を撃って支度表を読む。
    fn sheet(&self, dir: &Path) -> String {
        let out = folio(&["intake", "--write"], dir);
        assert_eq!(out.status.code(), Some(0), "{}", both(&out));
        fs::read_to_string(dir.join("intake-sheet.yaml")).unwrap()
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn no_forbidden(file: &str, text: &str) {
    for f in FORBIDDEN {
        assert!(!text.contains(f), "{file} に「{f}」が在る");
    }
}

/// 歯 1: 名を kumo に替えた外の置き場: 面 4 枚の題・名札・表紙と入口と憲法の見出し、支度表の頭の注と id が kumo。
#[test]
fn f154_a_renamed_place_shows_its_own_name_and_never_folio2() {
    let w = Work::outer("kumo", Some("kumo"));
    for (file, html) in w.build(&w.place(), 2, &FACES) {
        assert!(html.contains("<title>kumo — "), "{file}");
        assert!(
            html.contains("<span class=\"brand\"><span class=\"long\">kumo</span><span class=\"short\">k</span></span>"),
            "{file}"
        );
        assert!(html.contains("<span>kumo — "), "{file}");
        no_forbidden(&file, &html);
        match file.as_str() {
            "index.html" => assert!(html.contains("<h1>kumo — "), "{file}"),
            "constitution.html" => assert!(html.contains("<h1>kumo の憲法 — "), "{file}"),
            _ => {}
        }
    }
    let sheet = w.sheet(&w.place());
    assert!(sheet.starts_with("# kumo 支度表（"), "{sheet}");
    assert!(sheet.contains("\n  \"id\": \"kumo-intake-sheet\"\n"), "{sheet}");
    no_forbidden("intake-sheet.yaml", &sheet);
}

/// 歯 2: 骨格のまま（憲法の meta.id は 未記入）: 題と表紙は名なし・名札なし・憲法の見出しは「憲法 — 」・支度表の id は intake-sheet。
#[test]
fn f154_the_skeleton_shows_no_name() {
    let w = Work::outer("skeleton", None);
    for (file, html) in w.build(&w.place(), 2, &FACES) {
        let title = match file.as_str() {
            "index.html" => "<title>設計文書の入口（",
            "constitution.html" => "<title>憲法（",
            "srs.html" => "<title>要件書（",
            _ => "<title>判断の記録 ADR-1（",
        };
        assert!(html.contains(title), "{file}: {title}");
        assert!(!html.contains("class=\"brand\""), "{file}");
        assert!(!html.contains("<span>未記入"), "{file}");
        no_forbidden(&file, &html);
        if file == "constitution.html" {
            assert!(html.contains("<h1>憲法 — "), "{file}");
        }
    }
    let sheet = w.sheet(&w.place());
    assert!(sheet.starts_with("# 支度表（"), "{sheet}");
    assert!(sheet.contains("\n  \"id\": \"intake-sheet\"\n"), "{sheet}");
    no_forbidden("intake-sheet.yaml", &sheet);
}

/// 歯 3（folio2 自身）:repo の design-intent の組み立ては 0 で面 4 枚が folio2 / f2・写しの支度表は folio2・
/// 床の凍結の土台の写しの始まりの凍結の 3 file の頭の注は「# folio2 」（tests/freeze_root.rs の歯 4 と同じ作り方）。
#[test]
fn f154_folio2_keeps_its_own_name() {
    let w = Work::empty("folio2");
    let real = repo_root().join("design-intent");
    for (file, html) in w.build(&real, 0, &FACES) {
        assert!(html.contains("<title>folio2 — "), "{file}");
        assert!(
            html.contains("<span class=\"brand\"><span class=\"long\">folio2</span><span class=\"short\">f2</span></span>"),
            "{file}"
        );
    }

    let copy = w.root.join("copy");
    copy_tree(&real, &copy);
    let sheet = w.sheet(&copy);
    assert!(sheet.starts_with("# folio2 支度表（"), "{sheet}");
    assert!(sheet.contains("\n  \"id\": \"folio2-intake-sheet\"\n"), "{sheet}");

    let place = w.root.join("start/design-intent");
    copy_tree(&repo_root().join(FLOOR_BASE), &place);
    fs::remove_dir_all(place.join("anchors")).unwrap();
    fs::create_dir_all(w.root.join("start/contracts")).unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        w.root.join("start/contracts/schema.toml"),
    )
    .unwrap();
    let start = w.root.join("start");
    git(&start, &["init", "-q"]);
    git(&start, &["add", "-A"]);
    git(&start, &["commit", "-q", "-m", "fixture"]);
    let out = folio(&["check", "--freeze-start"], &place);
    assert_eq!(out.status.code(), Some(0), "{}", both(&out));
    for name in ["constitution-v1.0.yaml", "index.yaml", "ids-v1.8.yaml"] {
        let text = fs::read_to_string(place.join("anchors").join(name)).unwrap();
        assert!(text.starts_with("# folio2 "), "{name}: {text}");
    }
}

/// 歯 4: 面の凍結 fixture の写しの憲法の meta.id を 未記入 にした置き場: 5 種の面の題が名で始まらず、名札が無く、
/// 字 >folio2< と「folio2 — 」と「fixture — 」が無い。
#[test]
fn f154_every_face_of_an_unnamed_place_shows_no_name() {
    let w = Work::empty("unnamed");
    let fixture = repo_root().join("tests/fixtures/face");
    let src = w.root.join("src");
    for dir in ["preview", "adr", "design-note"] {
        fs::create_dir_all(src.join(dir)).unwrap();
    }
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
        fs::copy(fixture.join(name), src.join(name)).unwrap();
    }
    for name in ["folio.css", "folio-ui.js"] {
        fs::copy(fixture.join(name), src.join("preview").join(name)).unwrap();
    }
    fs::create_dir_all(w.root.join("contracts")).unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        w.root.join("contracts/schema.toml"),
    )
    .unwrap();
    copy_tree(&repo_root().join("vendor/archify"), &w.root.join("vendor/archify"));
    edit(
        &src.join("constitution.yaml"),
        "\n  id: fixture-constitution\n",
        "\n  id: 未記入\n",
    );
    let faces = [
        ("index.html", "<title>設計文書の入口（"),
        ("constitution.html", "<title>憲法（"),
        ("srs.html", "<title>要件書（"),
        ("adr-2.html", "<title>判断の記録 ADR-2（"),
        ("note-full.html", "<title>設計ノート full（"),
    ];
    let names: Vec<&str> = faces.iter().map(|(f, _)| *f).collect();
    for ((file, html), (_, title)) in w.build(&src, 2, &names).into_iter().zip(faces) {
        assert!(html.contains(title), "{file}: {title}");
        assert!(!html.contains("class=\"brand\""), "{file}");
        for bad in [">folio2<", "folio2 — ", "fixture — "] {
            assert!(!html.contains(bad), "{file} に「{bad}」が在る");
        }
    }
}
