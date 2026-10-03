//! 面と生成物の置き場の名の歯（便 154・docs/design/delivery-154.md §1 (c) の 2・FR7 / FR4 / FR1 / FR24）。binary 経由。
//! 置き場の名は憲法の meta.id の `<名>-constitution` の <名>（導けなければ名を出さない）。期待の字は歯の中の手書きで、
//! 生成器の関数は呼ばない（P-10.1）。
//! 外の置き場は一時 dir の根で git の init → `folio init --dir design-intent` → 憲法と入口の meta.id を替える →
//! 様式 3 本を repo の design-intent/preview/ から写す → commit、の後に撃つ（§1 (a) の 2）。
//! 1. 名を替えた置き場は自分の名を出し folio2 を出さない／2. 骨格のままなら名を出さない／
//! 3. folio2 自身の面・支度表・始まりの凍結の 3 file は folio2 の名のまま／4. 名の無い置き場の 5 種の面に名札も名も無い。
//!
//! 便 174（docs/design/delivery-174.md §1 (c)・ADR-16 決定 (2)(オ)）: 5. 外の置き場（骨格の名を替え、規則の表の R-7 が別の意味・
//! R-13 が無い）の 9 本の生成区間は行 R-8・R-16 のほかの folio2 の番号を名指さない／6. folio2 自身の置き場の生成区間は番号を持ったまま。
//!
//! 便 175（docs/design/delivery-175.md §1 (c) の 2・ADR-30 決定 (5)(6)）: 7. folio2・骨格・外の置き場の 9 本の生成区間は、周の引き金の
//! 仕掛け（引き金の一覧・印と門が同じ関数で測る要約値）と、欄の決まりに無い改訂の欄を言わない。
//! 便 177（docs/design/delivery-177.md §1 (c) の 1）: 8. 同じ 3 つの置き場の生成区間は、印が持たなくなった節点の表と残差を言わない。
//!
//! 版管理の下の file は書き換えない（`--dir` の写しと `--out` は必ず一時 dir の中）。
#![cfg(test)]

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
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../folio2")
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
    Command::new(env!("CARGO_BIN_EXE_tz"))
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
    fs::create_dir_all(w.root.join("start/contracts/field-schema")).unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        w.root.join("start/contracts/field-schema/schema.toml"),
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
    fs::create_dir_all(w.root.join("contracts/field-schema")).unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        w.root.join("contracts/field-schema/schema.toml"),
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

// ── 便 174: 生成区間は外の置き場で folio2 の番号を名指さない ──

/// 外の置き場でも名指してよい行（ADR-16 決定 (2)(オ)）。
const F174_RESERVED: [&str; 2] = ["R-8", "R-16"];
/// 生成区間を持つ 9 本。
const F174_FILES: [&str; 9] = [
    "adr/schema.yaml",
    "design-note/schema.yaml",
    "ceiling.yaml",
    "rules.yaml",
    "index.yaml",
    "srs.yaml",
    "vocabulary.yaml",
    "intake.yaml",
    "graph.yaml",
];
/// folio2 の置き場にだけ在る設計ノートの欄の決まりの 5 欄（値が folio2 の規則の表の行）。
const F174_HOME_ROWS: [&str; 5] = [
    "    body_classes_rules_row: R-3\n",
    "    quality_rules_row: R-14\n",
    "    tool_version_rules_row: R-15\n",
    "    retry_rules_row: R-7\n",
    "    p18_4_judged_by: R-13\n",
];

/// 生成区間の印の間の字。
fn f174_region(text: &str) -> String {
    text.lines()
        .skip_while(|l| !l.starts_with("# folio:schema:begin"))
        .skip(1)
        .take_while(|l| !l.starts_with("# folio:schema:end"))
        .map(|l| format!("{l}\n"))
        .collect()
}

/// id の形の語（前の字が英字でないもの・歯の中の手書き）: ADR-n・NFR/GOAL/CON/FR/AC と数・P-/N-/A- と数（. と数が続いてよい）・
/// R-/D- と数・便 と数。
fn f174_ids(text: &str) -> Vec<String> {
    let c: Vec<char> = text.chars().collect();
    let num = |at: usize| c.iter().skip(at).take_while(|x| x.is_ascii_digit()).count();
    let mut out = Vec::new();
    let mut i = 0;
    while i < c.len() {
        let head: String = c.iter().skip(i).take(4).collect();
        let prefix = ["ADR-", "NFR", "GOAL", "CON", "FR", "AC"]
            .iter()
            .find(|p| head.starts_with(**p))
            .map(|p| p.chars().count())
            .or_else(|| {
                (matches!(c[i], 'P' | 'N' | 'A' | 'R' | 'D') && c.get(i + 1) == Some(&'-')).then_some(2)
            })
            .or_else(|| (c[i] == '便').then(|| 1 + c.iter().skip(i + 1).take_while(|x| **x == ' ').count()));
        let fresh = i == 0 || !c[i - 1].is_ascii_alphabetic();
        match prefix {
            Some(p) if fresh && num(i + p) > 0 => {
                let mut len = p + num(i + p);
                if matches!(c[i], 'P' | 'N' | 'A') && p == 2 && c.get(i + len) == Some(&'.') && num(i + len + 1) > 0 {
                    len += 1 + num(i + len + 1);
                }
                out.push(c[i..i + len].iter().filter(|x| **x != ' ').collect());
                i += len;
            }
            _ => i += 1,
        }
    }
    out
}

/// 置き場の 9 本の生成区間: 予約の外の id と判断の記録の決定の番号を持たず、予約の 2 行は名指す。
fn f174_regions_name_only_reserved_rows(place: &Path) {
    let mut seen = Vec::new();
    for file in F174_FILES {
        let region = f174_region(&fs::read_to_string(place.join(file)).unwrap());
        assert!(!region.is_empty(), "{file}: 生成区間が無い");
        let ids = f174_ids(&region);
        let left: Vec<&String> = ids.iter().filter(|i| !F174_RESERVED.contains(&i.as_str())).collect();
        assert!(left.is_empty(), "{file} の生成区間が folio2 の番号を名指す: {left:?}");
        assert!(!region.contains("決定 ("), "{file} の生成区間が判断の記録の決定を名指す");
        seen.extend(ids);
    }
    for id in F174_RESERVED {
        assert!(seen.iter().any(|i| i == id), "予約の行 {id} を名指さない");
    }
}

/// 歯 5: 外の置き場（骨格を tsuzuri の名に替え、規則の表に別の意味の R-7 を足す・R-13 は無い）の 9 本の生成区間は予約の 2 行の
/// ほかの番号を名指さず（骨格の命令の字も --write の後の字も）、folio2 にだけ在る 5 欄を書かず、--check 0 で、床は設計ノートの
/// 写しを落とさない。
#[test]
fn f174_a_tsuzuri_shaped_place_names_no_folio2_number() {
    let w = Work::outer("f174-tsuzuri", Some("tsuzuri"));
    edit(
        &w.place().join("rules.yaml"),
        "  - id: R-8\n",
        "  - id: R-7\n    article: P-1\n    what: 席の停止の検知条件と再起動先・新規投入の線\n    value: 未定\n    kind: deny\n    status: 未定\n    ruling: 未記入\n    ruled_at: 未記入\n    stage: in-loop\n  - id: R-8\n",
    );
    // 骨格の命令が書いた字（名は未記入）と、tsuzuri の名で書き直した字（列の根の表の tsuzuri の行が入る）の両方
    f174_regions_name_only_reserved_rows(&w.place());
    for flag in ["--write", "--check"] {
        let out = folio(&["schema", flag], &w.place());
        assert_eq!(out.status.code(), Some(0), "schema {flag}: {}", both(&out));
    }
    f174_regions_name_only_reserved_rows(&w.place());
    let note = fs::read_to_string(w.place().join("design-note/schema.yaml")).unwrap();
    for row in F174_HOME_ROWS {
        assert!(!note.contains(row), "外の置き場に folio2 の欄 {row}");
    }
    let check = both(&folio(&["check"], &w.place()));
    assert!(!check.contains("床の定数と違う"), "{check}");
}

/// 歯 6: folio2 自身の置き場（repo の正本の憲法と 9 本の写し）の生成区間は番号を持ったまま --check 0。
#[test]
fn f174_folio2_keeps_its_numbers() {
    let w = Work::empty("f174-folio2");
    let real = repo_root().join("design-intent");
    for file in F174_FILES.iter().chain(["constitution.yaml"].iter()) {
        let to = w.place().join(file);
        fs::create_dir_all(to.parent().unwrap()).unwrap();
        fs::copy(real.join(file), &to).unwrap();
    }
    let out = folio(&["schema", "--check"], &w.place());
    assert_eq!(out.status.code(), Some(0), "{}", both(&out));
    let note = f174_region(&fs::read_to_string(w.place().join("design-note/schema.yaml")).unwrap());
    for row in F174_HOME_ROWS {
        assert!(note.contains(row), "folio2 の欄 {row} が無い");
    }
    assert!(note.contains("（JSON の 5 型の欄の決まりそのまま・ADR-4 決定 (1)）"), "{note}");
}

// ── 便 175: 生成区間は周の引き金の仕掛けを言わない ──

/// 生成区間に在ってはならない字（手書き・ADR-30 決定 (2)(5)(6) の後の実装と違う主張・印の節点の表の字は便 177）。
const F175_FALSE: [&str; 4] = ["引き金", "trigger", "印と門が同じ関数で測る", "revises"];

fn f175_no_false_claims(place: &Path, label: &str, words: &[&str]) {
    for file in F174_FILES {
        let region = f174_region(&fs::read_to_string(place.join(file)).unwrap());
        for word in words {
            assert!(!region.contains(word), "{label}: {file} の生成区間に「{word}」");
        }
    }
}

/// 歯 7: folio2 自身の置き場・骨格の命令が書いた置き場・tsuzuri の名で書き直した置き場の 9 本の生成区間。
#[test]
fn f175_no_region_claims_the_round_trigger() {
    f175_no_false_claims(&repo_root().join("design-intent"), "folio2", &F175_FALSE);
    let w = Work::outer("f175-tsuzuri", Some("tsuzuri"));
    f175_no_false_claims(&w.place(), "骨格", &F175_FALSE);
    let out = folio(&["schema", "--write"], &w.place());
    assert_eq!(out.status.code(), Some(0), "schema --write: {}", both(&out));
    f175_no_false_claims(&w.place(), "tsuzuri", &F175_FALSE);
}

// ── 便 177: 生成区間は印の節点の表を言わない ──

/// 生成区間に在ってはならない字（手書き・印は節点の表と残差の要約値を書かない）。
const F177_FALSE: [&str; 2] = ["天井の印はこの要約値の表", "残差"];

/// 歯 8: folio2 自身の置き場・骨格の命令が書いた置き場・tsuzuri の名で書き直した置き場の 9 本の生成区間。
#[test]
fn f177_no_region_claims_the_stamp_node_table() {
    f175_no_false_claims(&repo_root().join("design-intent"), "folio2", &F177_FALSE);
    let w = Work::outer("f177-tsuzuri", Some("tsuzuri"));
    f175_no_false_claims(&w.place(), "骨格", &F177_FALSE);
    let out = folio(&["schema", "--write"], &w.place());
    assert_eq!(out.status.code(), Some(0), "schema --write: {}", both(&out));
    f175_no_false_claims(&w.place(), "tsuzuri", &F177_FALSE);
}

// ── 便 194（docs/design/delivery-194.md §1 (c)・台帳 f2-648.261）: 外の生成区間に番号を落とした跡の字の壊れを残さない ──

/// 和字（平仮名・片仮名の字と長音・漢字・歯の中の手書き）。
fn f194_wa(ch: char) -> bool {
    matches!(ch, '\u{3041}'..='\u{3096}' | '\u{30a1}'..='\u{30fa}' | 'ー' | '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}')
}

/// 英数字と和字が空白なしで接する 2 字の並び（順不同）の集合。
fn f194_joins(text: &str) -> Vec<String> {
    let c: Vec<char> = text.chars().collect();
    let mut out: Vec<String> = c
        .windows(2)
        .filter(|w| (w[0].is_ascii_alphanumeric() && f194_wa(w[1])) || (f194_wa(w[0]) && w[1].is_ascii_alphanumeric()))
        .map(|w| w.iter().collect())
        .collect();
    out.sort();
    out.dedup();
    out
}

#[test]
fn f194_abroad_regions_leave_no_broken_joins() {
    // 外の置き場（tsuzuri の名）の 9 本の生成区間: 英数字と和字の詰まりは folio2 の同じ file の生成区間に在る並びだけ
    let w = Work::outer("f194-tsuzuri", Some("tsuzuri"));
    let out = folio(&["schema", "--write"], &w.place());
    assert_eq!(out.status.code(), Some(0), "{}", both(&out));
    let home = repo_root().join("design-intent");
    for file in F174_FILES {
        let there = f174_region(&fs::read_to_string(w.place().join(file)).unwrap());
        let here = f174_joins_in(&home.join(file));
        let extra: Vec<String> = f194_joins(&there).into_iter().filter(|j| !here.contains(j)).collect();
        assert!(extra.is_empty(), "{file}: 外にだけ在る詰まり {extra:?}");
    }
    let adr = f174_region(&fs::read_to_string(w.place().join("adr/schema.yaml")).unwrap());
    assert!(adr.contains("folio2 の他の id と同じくゼロ詰めしない。"), "{adr}");
    // 注で何も残らない欄は欄ごと書かない（便 174 の検証の V7）
    assert!(!adr.contains("grill_note:"), "{adr}");
    let note = f174_region(&fs::read_to_string(w.place().join("design-note/schema.yaml")).unwrap());
    // 落とした文を指す「どちらも」は語だけが落ちて文の中身は残り、出所の台帳の id を落とした括弧は「持ち主の裁定」ごと落ちる
    // （ほかの注の括弧の中の「どちらも」は指す語でないので、見るのは derived_note の行だけ）
    let derived = note.lines().find(|l| l.trim_start().starts_with("derived_note:")).unwrap_or_default();
    assert!(derived.contains("導出物の置き場") && !derived.contains("どちらも"), "{derived}");
    assert!(
        note.contains("のはそのため。面の生成器も様式の file も呼ばず、導出物の置き場（--out）は消費側が宣言する（既定なし）。値に二重引用符"),
        "{note}"
    );
    assert!(note.contains("図の対＝設計ノートの図は"), "{note}");
    assert!(!note.contains("（持ち主の裁定 2026-09-19）"), "{note}");
}

/// folio2 の置き場の file の生成区間の詰まりの並び。
fn f174_joins_in(path: &Path) -> Vec<String> {
    f194_joins(&f174_region(&fs::read_to_string(path).unwrap()))
}
