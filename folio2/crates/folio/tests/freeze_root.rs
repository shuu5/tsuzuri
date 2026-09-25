//! 列の根の表と始まりの凍結 `folio check --freeze-start` の歯（便 121・docs/design/delivery-121.md §1 (f)・ADR-16 決定 (2)(ア)(イ)・
//! FR23 / FR24）。folio は実行 file の crate なので命令を撃つ。
//! 凍結 anchor（P-10.1）は 3 つ: (1) 歯の中の定数 ROOT（folio2 の列の根の digest・day-1 の床の値・crate の中の表は読まない）、
//! (2) 床の凍結の土台の凍結済みの anchors/（constitution-v1.0.yaml の digest の欄と ids-v1.8.yaml の byte 列）、
//! (3) tests/fixtures/anchor/root-digest-drift/ の索引の digest の値（定数 DRIFT に写す）。
//! 写しは一時 dir に design-intent/ として作り、置き場の親の contracts/ に器の導出 file を写し、git init と 1 commit を行う
//! （tests/freeze.rs と同じ作り方・歯の終わりに消す）。版管理の履歴に anchor を残したくない置き場は、commit の前に消す。
//! 1. folio2 の床と 1 行の写し／2. 名の書き換えは床で落ちる／3. 別の中身の根は名でも行でも落ちる／
//! 4. 始まりの凍結が 2 つを書き、書いた後の床が合格／5. どちらか 1 本在れば断る／
//! 6. ほかの検査に違反か「まだ分からない」・表に無い名（--freeze-anchor も）／7. 写しは置き場の名の行だけ（folio schema）／
//! 8. tsuzuri の名は表の 2 行目を写し、床が folio2 の根をその行と照らして落とす（便 133 が旧 scribe3 の名で足し・
//!    docs/design/delivery-133.md §1 (c)2・便 134 が改名の後の名に改めた・docs/design/delivery-134.md §1 (c)2）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// folio2 の列の根の digest（憲法 第 1.0 版の凍結・tests/anchor.rs の定数と同じ値）。
const ROOT: &str = "acb52acd04b5d3a1feaf9ad5f0138f7614ce31964144b46ead914bde86e866ed";
/// tests/fixtures/anchor/root-digest-drift/anchors/index.yaml の entries[0].digest（自分自身と一致する別の中身の根）。
const DRIFT: &str = "85cbd21b680e0b8d3d2c134f11f8a03926929717a8ab172bc1679106da99a9fe";
const FLOOR_BASE: &str = "tests/fixtures/floor_base/design-intent";
const FOLIO2: &str = "folio2-constitution";
/// 表に無い名（歯のための名）。
const OTHER: &str = "renamed-constitution";
/// tsuzuri の憲法の名と列の根の digest（旧 scribe3 の版 e52d24c の始まりの凍結・便 133・版 1c93e70 の改名で名を
/// 改めた・便 134・digest は名の外で不変・crate の中の表は読まない）。
const TSUZURI: &str = "tsuzuri-constitution";
const TSUZURI_ROOT: &str = "35eb6b369f0504167571a27b50c950e1361609d9b19b71e0f1e9de832f8c5356";

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

/// 写しの一時 dir（歯の終わりに消す）。
struct Work {
    root: PathBuf,
}

impl Work {
    /// `src` を design-intent/ として写し、`contracts` なら器の導出 file を写し、`prep` を当ててから git init と 1 commit。
    fn new(case: &str, src: &str, contracts: bool, prep: impl FnOnce(&Path)) -> Work {
        let root = std::env::temp_dir().join(format!("folio-f121-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_tree(&repo_root().join(src), &root.join("design-intent"));
        if contracts {
            fs::create_dir_all(root.join("contracts")).unwrap();
            fs::copy(
                repo_root().join("contracts/schema.toml"),
                root.join("contracts/schema.toml"),
            )
            .unwrap();
        }
        prep(&root.join("design-intent"));
        git(&root, &["init", "-q"]);
        let w = Work { root };
        w.commit();
        w
    }

    fn commit(&self) {
        git(&self.root, &["add", "-A"]);
        git(&self.root, &["commit", "-q", "--allow-empty", "-m", "fixture"]);
    }

    fn dir(&self) -> PathBuf {
        self.root.join("design-intent")
    }

    fn check(&self, flags: &[&str]) -> Output {
        folio(&["check", "--dir"], &self.dir(), flags)
    }

    fn schema(&self, mode: &str) -> Output {
        folio(&["schema", "--dir"], &self.dir(), &[mode])
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn folio(head: &[&str], dir: &Path, tail: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(head)
        .arg(dir)
        .args(tail)
        .output()
        .expect("folio を起動できない")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn show(out: &Output) -> String {
    format!("{}\n{}", text(&out.stdout), text(&out.stderr))
}

/// 標準出力の違反の行（「[種別] 」で始まる）。
fn violations(out: &Output) -> Vec<String> {
    text(&out.stdout)
        .lines()
        .filter(|l| l.starts_with('['))
        .map(str::to_string)
        .collect()
}

fn edit(path: &Path, f: impl FnOnce(&str) -> String) {
    let before = fs::read_to_string(path).unwrap();
    let after = f(&before);
    assert_ne!(before, after, "変異が当たっていない: {}", path.display());
    fs::write(path, after).unwrap();
}

/// 憲法の meta.id を `name` に書き換える。
fn rename(dir: &Path, from: &str, name: &str) {
    edit(&dir.join("constitution.yaml"), |t| {
        t.replacen(
            &format!("\nmeta:\n  id: {from}\n"),
            &format!("\nmeta:\n  id: {name}\n"),
            1,
        )
    });
}

/// 欄の決まりの写しの列の根の表を空の表にする（folio2 の block の 2 行 → 1 行）。
fn empty_table(dir: &Path) {
    edit(&dir.join("adr/schema.yaml"), |t| {
        t.replacen(
            &format!("    root_digests:\n      {FOLIO2}: {ROOT}\n"),
            "    root_digests: {}\n",
            1,
        )
    });
}

fn remove(dir: &Path, files: &[&str]) {
    for f in files {
        fs::remove_file(dir.join("anchors").join(f)).unwrap();
    }
}

fn no_anchors(dir: &Path) {
    fs::remove_dir_all(dir.join("anchors")).unwrap();
}

/// anchors/ の名と byte 列（無ければ空）。
fn anchors_snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let Ok(rd) = fs::read_dir(dir.join("anchors")) else {
        return Vec::new();
    };
    let mut files: Vec<(String, Vec<u8>)> = rd
        .map(|e| {
            let e = e.unwrap();
            (
                e.file_name().to_string_lossy().into_owned(),
                fs::read(e.path()).unwrap(),
            )
        })
        .collect();
    files.sort();
    files
}

/// file の digest の欄の値（「digest: 」か「"digest": "」の行・字下げは問わない）を全部。
fn digests(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter_map(|l| {
            let l = l.trim_start().trim_start_matches("- ");
            l.strip_prefix("\"digest\": ")
                .or_else(|| l.strip_prefix("digest: "))
                .map(|v| v.trim_matches('"').to_string())
        })
        .collect()
}

/// 列の根の欄の行とその次の行（2 行）。
fn table_lines(path: &Path) -> Vec<String> {
    let text = fs::read_to_string(path).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.trim_start().starts_with("root_digests:"))
        .unwrap_or_else(|| panic!("root_digests の行が無い: {}", path.display()));
    lines[at..(at + 2).min(lines.len())]
        .iter()
        .map(|l| l.to_string())
        .collect()
}

/// 1. folio2 の床と 1 行の写し: 土台の床は合格のまま、実の生成区間と土台の写しの列の根の表は folio2 の 1 行。
#[test]
fn f121_folio2_floor_passes_and_the_copy_holds_its_own_row() {
    let w = Work::new("floor", FLOOR_BASE, true, |_| {});
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    assert!(
        text(&out.stdout).contains("合格（違反 0・まだ分からない 0）"),
        "{}",
        show(&out)
    );
    let want = [
        "    root_digests:".to_string(),
        format!("      {FOLIO2}: {ROOT}"),
    ];
    for path in [
        repo_root().join("design-intent/adr/schema.yaml"),
        repo_root().join(FLOOR_BASE).join("adr/schema.yaml"),
    ] {
        assert_eq!(table_lines(&path), want, "{}", path.display());
        let next = fs::read_to_string(&path).unwrap();
        let after = next
            .lines()
            .skip_while(|l| !l.trim_start().starts_with("root_digests:"))
            .nth(2)
            .unwrap_or_default();
        assert!(!after.starts_with("      "), "表の行が 2 つ以上: {after}");
    }
    let real = fs::read_to_string(repo_root().join("design-intent/adr/schema.yaml")).unwrap();
    assert!(!real.contains("    root_digest: "), "前の欄名 root_digest の行が残る");
}

/// 2. 名の書き換えは床で落ちる: 表に無いの違反（digest の全桁）と写しの違反の 2 件。
#[test]
fn f121_renamed_constitution_fails_the_floor_twice() {
    let w = Work::new("rename", FLOOR_BASE, true, |_| {});
    rename(&w.dir(), FOLIO2, OTHER);
    w.commit();
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    let v = violations(&out);
    assert_eq!(v.len(), 2, "{v:?}");
    assert_eq!(
        v.iter()
            .filter(|l| l.starts_with("[anchor] ")
                && l.contains("列の根")
                && l.contains(OTHER)
                && l.contains("表に無い")
                && l.contains(ROOT))
            .count(),
        1,
        "{v:?}"
    );
    assert_eq!(
        v.iter()
            .filter(|l| l.starts_with("[adr] ") && l.contains("schema.anchor.root_digests"))
            .count(),
        1,
        "{v:?}"
    );
}

/// 3. 別の中身の根は、表に無い名で落ち、folio2 の名に書き換えると表の行と違うで落ちる。
#[test]
fn f121_foreign_root_fails_by_name_and_by_row() {
    let w = Work::new("drift", "tests/fixtures/anchor/root-digest-drift", true, |_| {});
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    let v = violations(&out);
    assert_eq!(v.len(), 1, "{v:?}");
    assert!(
        v[0].contains("fixture-constitution") && v[0].contains("表に無い") && v[0].contains(DRIFT),
        "{v:?}"
    );
    rename(&w.dir(), "fixture-constitution", FOLIO2);
    w.commit();
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    let v = violations(&out);
    assert!(
        v.iter().any(|l| l.contains("列の根")
            && l.contains(&format!("列の根の表の {FOLIO2} の行"))
            && l.contains("と違う")),
        "{v:?}"
    );
    assert!(!v.iter().any(|l| l.contains("表に無い")), "{v:?}");
}

/// 4. 2 つの基準が無い置き場: 既存の 2 つの旗は互いに断り、--freeze-start が 3 file を書き、commit の後の床は合格。
#[test]
fn f121_freeze_start_writes_both_and_the_floor_passes() {
    let w = Work::new("start", FLOOR_BASE, true, no_anchors);
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(2), "{}", show(&out));
    assert!(violations(&out).is_empty(), "{}", show(&out));
    for flag in ["--freeze-anchor", "--freeze-ids"] {
        let out = w.check(&[flag]);
        assert_eq!(out.status.code(), Some(2), "{flag}: {}", show(&out));
        assert!(text(&out.stderr).contains("凍結しない"), "{flag}: {}", show(&out));
        assert!(!w.dir().join("anchors").exists(), "{flag} が anchors/ を作った");
    }
    let out = w.check(&["--freeze-start"]);
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    assert!(text(&out.stderr).contains("始まりの凍結をした"), "{}", show(&out));
    let names: Vec<String> = anchors_snapshot(&w.dir()).into_iter().map(|(n, _)| n).collect();
    assert_eq!(names, ["constitution-v1.0.yaml", "ids-v1.8.yaml", "index.yaml"]);
    let base = repo_root().join(FLOOR_BASE).join("anchors");
    assert_eq!(digests(&base.join("constitution-v1.0.yaml")), [ROOT]);
    assert_eq!(digests(&w.dir().join("anchors/constitution-v1.0.yaml")), [ROOT]);
    assert_eq!(digests(&w.dir().join("anchors/index.yaml")), [ROOT]);
    assert!(
        fs::read(w.dir().join("anchors/ids-v1.8.yaml")).unwrap()
            == fs::read(base.join("ids-v1.8.yaml")).unwrap(),
        "書いた id の一覧が土台の凍結済みの file と byte 一致しない"
    );
    w.commit();
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    assert!(
        text(&out.stdout).contains("合格（違反 0・まだ分からない 0）"),
        "{}",
        show(&out)
    );
}

/// 5. どちらか 1 本在れば、何も書かずに 1 で断る。
#[test]
fn f121_freeze_start_refuses_when_either_exists() {
    for (case, gone, present) in [
        ("chain-only", &["ids-v1.8.yaml"][..], "憲法の列"),
        (
            "ids-only",
            &["constitution-v1.0.yaml", "index.yaml"][..],
            "id の一覧",
        ),
    ] {
        let w = Work::new(case, FLOOR_BASE, true, |d| remove(d, gone));
        let before = anchors_snapshot(&w.dir());
        let out = w.check(&["--freeze-start"]);
        let se = text(&out.stderr);
        assert_eq!(out.status.code(), Some(1), "{case}: {}", show(&out));
        assert!(
            se.contains("--freeze-start") && se.contains(present) && se.contains("在る"),
            "{case}: {se}"
        );
        assert_eq!(anchors_snapshot(&w.dir()), before, "{case}: anchors/ が変わった");
    }
}

/// 6. ほかの検査に違反か「まだ分からない」・表に無い名では書かない。--freeze-anchor も表に無い名で digest の全桁を出す。
#[test]
fn f121_freeze_start_writes_nothing_on_any_finding() {
    // (1) 違反 1 つ
    let w = Work::new("violation", FLOOR_BASE, true, |d| {
        no_anchors(d);
        edit(&d.join("adr/schema.yaml"), |t| {
            t.replacen("options_rule: {min: 2, adopted: 1}", "options_rule: {min: 3, adopted: 1}", 1)
        });
    });
    let out = w.check(&["--freeze-start"]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    assert_eq!(violations(&out).len(), 1, "{}", show(&out));
    assert!(text(&out.stderr).contains("凍結しない"), "{}", show(&out));
    assert!(!w.dir().join("anchors").exists());
    drop(w);

    // (2) まだ分からない 1 つ（器の導出 file を置かない）
    let w = Work::new("unknown", FLOOR_BASE, false, no_anchors);
    let out = w.check(&["--freeze-start"]);
    let se = text(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{}", show(&out));
    assert!(violations(&out).is_empty(), "{}", show(&out));
    assert!(se.contains("contracts/schema.toml") && se.contains("凍結しない"), "{se}");
    assert!(!w.dir().join("anchors").exists());
    drop(w);

    // (3) 表に無い名（写しは空の表に揃える＝写しの違反を立てない）
    let w = Work::new("unlisted", FLOOR_BASE, true, |d| {
        no_anchors(d);
        rename(d, FOLIO2, OTHER);
        empty_table(d);
    });
    let out = w.check(&["--freeze-start"]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    let v = violations(&out);
    assert_eq!(v.len(), 1, "{v:?}");
    assert!(
        v[0].contains(OTHER) && v[0].contains("表に無い") && v[0].contains(ROOT),
        "{v:?}"
    );
    assert!(!w.dir().join("anchors").exists());
    drop(w);

    // (4) id の一覧だけが在る置き場で同じ名の書き換え → --freeze-anchor も digest の全桁を出す
    let w = Work::new("unlisted-anchor", FLOOR_BASE, true, |d| {
        remove(d, &["constitution-v1.0.yaml", "index.yaml"]);
        rename(d, FOLIO2, OTHER);
        empty_table(d);
    });
    let before = anchors_snapshot(&w.dir());
    let out = w.check(&["--freeze-anchor"]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    let v = violations(&out);
    assert!(
        v.iter().any(|l| l.contains("--freeze-anchor") && l.contains("表に無い") && l.contains(ROOT)),
        "{v:?}"
    );
    assert_eq!(anchors_snapshot(&w.dir()), before);
}

/// 7. folio schema は置き場の名の行だけを写す: 名を書き換えると check 1・write の後は空の表で check 0・meta.id が無ければ 2。
#[test]
fn f121_schema_copies_only_the_named_row() {
    let w = Work::new("schema", "design-intent", true, |_| {});
    let out = w.schema("--check");
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    rename(&w.dir(), FOLIO2, OTHER);
    let out = w.schema("--check");
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    assert!(text(&out.stderr).contains("adr/schema.yaml"), "{}", show(&out));
    let out = w.schema("--write");
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    let file = fs::read_to_string(w.dir().join("adr/schema.yaml")).unwrap();
    let rows: Vec<&str> = file
        .lines()
        .filter(|l| l.trim_start().starts_with("root_digests"))
        .collect();
    assert_eq!(rows, ["    root_digests: {}"]);
    assert!(!file.contains(ROOT), "写しに folio2 の行が残る");
    let out = w.schema("--check");
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    edit(&w.dir().join("constitution.yaml"), |t| {
        t.replacen(&format!("\nmeta:\n  id: {OTHER}\n"), "\nmeta:\n", 1)
    });
    let out = w.schema("--check");
    assert_eq!(out.status.code(), Some(2), "{}", show(&out));
    assert!(text(&out.stderr).contains("meta.id"), "{}", show(&out));
}

/// tsuzuri の名（8）: folio schema --write が列の根の欄を tsuzuri の行 1 行にし、commit の後の床は
/// 写しの folio2 の根を表の tsuzuri の行と照らして違うの違反ちょうど 1 件で 1。
#[test]
fn f134_tsuzuri_name_picks_its_row() {
    let w = Work::new("tsuzuri", "design-intent", true, |_| {});
    rename(&w.dir(), FOLIO2, TSUZURI);
    let out = w.schema("--write");
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    let path = w.dir().join("adr/schema.yaml");
    assert_eq!(
        table_lines(&path),
        [
            "    root_digests:".to_string(),
            format!("      {TSUZURI}: {TSUZURI_ROOT}"),
        ]
    );
    let file = fs::read_to_string(&path).unwrap();
    let after = file
        .lines()
        .skip_while(|l| !l.trim_start().starts_with("root_digests:"))
        .nth(2)
        .unwrap_or_default();
    assert!(!after.starts_with("      "), "表の行が 2 つ以上: {after}");
    assert!(!file.contains(ROOT), "写しに folio2 の行が残る");
    w.commit();
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    let v = violations(&out);
    assert_eq!(v.len(), 1, "{v:?}");
    assert!(
        v[0].starts_with("[anchor] ")
            && v[0].contains(&format!("列の根の表の {TSUZURI} の行"))
            && !v[0].contains("表に無い"),
        "{v:?}"
    );
}
