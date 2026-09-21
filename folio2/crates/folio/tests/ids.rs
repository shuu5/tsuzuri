//! 要件・判断・受入基準の id の消失と改番の歯（便 88・docs/design/delivery-88.md §1 (e)）。
//! 土台は凍結した写し（tests/fixtures/floor_base/design-intent/・要件書 v1.8・id の一覧の anchor ids-v1.8.yaml）を一時 dir に作り、
//! 器の導出 file を写しの根に置き、git init と 1 commit を行う（tests/freeze.rs の Work と同じ作り）。
//! (1) FR の行を消す → 1・P-7／(2) FR の番号だけを替える → 1・P-7.1／(3) FR を足すだけ → 0／(4) ids- の anchor を外す → 2／
//! (5) (4) で --freeze-ids → 0・commit して素の床 → 0・二度目は断り／(6) 実の anchor の id 53 本と FR1 の要約値の凍結 literal。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// FR1 の shall の字面を OS の道具 sha256sum に末尾の改行なしで与えて出した値（席の実測 2026-09-22・生成器と独立）。
const FR1_SUM: &str = "01391bf27709cf47e6424f5b3f4e363a877c2d1fcc069f6da753b1fab650fa2c";
const BASE_IDS: &str = "anchors/ids-v1.8.yaml";

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
    /// `mutate` は git init の前に写しへ当てる変異。
    fn new(case: &str, mutate: impl FnOnce(&Path)) -> Work {
        let root = std::env::temp_dir().join(format!("folio-ids-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_tree(
            &repo_root().join("tests/fixtures/floor_base/design-intent"),
            &root.join("design-intent"),
        );
        fs::create_dir_all(root.join("contracts")).unwrap();
        fs::copy(
            repo_root().join("contracts/schema.toml"),
            root.join("contracts/schema.toml"),
        )
        .unwrap();
        mutate(&root.join("design-intent"));
        git(&root, &["init", "-q"]);
        let w = Work { root };
        w.commit();
        w
    }

    fn commit(&self) {
        git(&self.root, &["add", "-A"]);
        git(
            &self.root,
            &["commit", "-q", "--allow-empty", "-m", "fixture"],
        );
    }

    fn dir(&self) -> PathBuf {
        self.root.join("design-intent")
    }

    fn check(&self, flags: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("check")
            .arg("--dir")
            .arg(self.dir())
            .args(flags)
            .output()
            .expect("folio を起動できない")
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn both(out: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn edit_srs(dir: &Path, f: impl FnOnce(&str) -> String) {
    let path = dir.join("srs.yaml");
    let before = fs::read_to_string(&path).unwrap();
    let after = f(&before);
    assert_ne!(before, after, "変異が当たっていない");
    fs::write(&path, after).unwrap();
}

/// FR19 の行（次の節の見出しの手前まで）を消す。
fn drop_fr19(t: &str) -> String {
    let start = t.find("  - id: FR19\n").expect("FR19 が無い");
    let end = t.find("\nnonfunctional:\n").expect("nonfunctional が無い") + 1;
    format!("{}{}", &t[..start], &t[end..])
}

/// (1) FR の行を 1 本まるごと消すと P-7 で落ちる。
#[test]
fn f88_lost_id_is_a_violation() {
    let w = Work::new("lost", |d| edit_srs(d, drop_fr19));
    let out = w.check(&[]);
    let s = both(&out);
    assert_eq!(out.status.code(), Some(1), "{s}");
    assert!(s.contains("[P-7] FR19 が消えた"), "{s}");
    assert!(!s.contains("[P-7.1]"), "{s}");
}

/// (2) FR の番号だけを未使用の番号に替える（shall は 1 字も変えない）と P-7.1 で落ち、両方の番号が出る。
#[test]
fn f88_renumbered_id_is_a_violation() {
    let w = Work::new("renumber", |d| {
        edit_srs(d, |t| t.replacen("  - id: FR19\n", "  - id: FR90\n", 1))
    });
    let out = w.check(&[]);
    let s = both(&out);
    assert_eq!(out.status.code(), Some(1), "{s}");
    assert!(s.contains("[P-7.1] FR19 の本文が FR90 へ付け替えられた"), "{s}");
    assert!(!s.contains("[P-7] "), "{s}");
}

/// (3) 欄のそろった FR を 1 本足すだけなら合格。
#[test]
fn f88_added_id_is_green() {
    let w = Work::new("added", |d| {
        edit_srs(d, |t| {
            t.replacen(
                "\nnonfunctional:\n",
                "\n  - id: FR90\n    title: 検査の行を足す\n    pattern: ubiquitous\n    strength: must\n    when: つねに\n    shall: 床は行を数える。\n    plain: 行を数えます。\n    basis: [P-1]\n    verify: {method: test, how: 歯で見る, ac: [AC1]}\n    goals: [GOAL1]\n    figures: []\nnonfunctional:\n",
                1,
            )
        })
    });
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", both(&out));
}

fn drop_ids_anchor(d: &Path) {
    fs::remove_file(d.join(BASE_IDS)).unwrap();
}

/// (4) ids- の anchor が 1 本も無ければ「まだ分からない」。
#[test]
fn f88_without_an_anchor_the_result_is_unknown() {
    let w = Work::new("no-anchor", drop_ids_anchor);
    let out = w.check(&[]);
    let s = both(&out);
    assert_eq!(out.status.code(), Some(2), "{s}");
    assert!(
        s.contains("# まだ分からない: 要件・判断・受入基準の id の消失と改番は baseline")
            && s.contains("測れない"),
        "{s}"
    );
}

/// (5) (4) の写しで --freeze-ids → 0・commit して素の床 → 0・二度目は上書きしないで断る。
#[test]
fn f88_freeze_writes_the_anchor_once() {
    let w = Work::new("freeze", drop_ids_anchor);
    let out = w.check(&["--freeze-ids"]);
    let s = both(&out);
    assert_eq!(out.status.code(), Some(0), "{s}");
    assert!(s.contains("凍結した: ") && s.contains("（id 49 本）"), "{s}");
    let written = w.dir().join(BASE_IDS);
    assert!(written.is_file(), "{s}");
    w.commit();
    let again = w.check(&[]);
    assert_eq!(again.status.code(), Some(0), "{}", both(&again));
    let twice = w.check(&["--freeze-ids"]);
    let s = both(&twice);
    assert_eq!(twice.status.code(), Some(1), "{s}");
    assert!(s.contains("上書きしない"), "{s}");
    // 凍結した木は土台の写しの anchor と 1 byte も違わない（生成物）
    assert_eq!(
        fs::read(&written).unwrap(),
        fs::read(repo_root().join("tests/fixtures/floor_base/design-intent").join(BASE_IDS))
            .unwrap()
    );
}

/// (6) 凍結 anchor: 実の ids-v1.24.yaml の id は 53 本（FR 20・NFR 3・AC 18・ADR 12）で、FR1 の要約値は独立に出した literal と一致する。
#[test]
fn f88_the_frozen_literal_pins_the_summary() {
    let text = fs::read_to_string(repo_root().join("design-intent/anchors/ids-v1.24.yaml"))
        .expect("ids-v1.24.yaml が無い");
    let lines: Vec<&str> = text.lines().collect();
    let ids: Vec<&str> = lines
        .iter()
        .filter_map(|l| l.strip_prefix("  - \"id\": \""))
        .filter_map(|l| l.strip_suffix('"'))
        .collect();
    assert_eq!(ids.len(), 53, "{ids:?}");
    let count = |p: &str| ids.iter().filter(|i| i.starts_with(p)).count();
    assert_eq!(
        (count("FR"), count("NFR"), count("AC"), count("ADR-")),
        (20, 3, 18, 12)
    );
    let at = lines
        .iter()
        .position(|l| *l == "  - \"id\": \"FR1\"")
        .expect("FR1 の行が無い");
    assert_eq!(lines[at + 1], "    \"section\": \"requirements\"");
    assert_eq!(lines[at + 2], format!("    \"sum\": \"{FR1_SUM}\""));
    // 土台の写しの側でも同じ字面の FR1 は同じ値
    let base = fs::read_to_string(
        repo_root()
            .join("tests/fixtures/floor_base/design-intent")
            .join(BASE_IDS),
    )
    .unwrap();
    assert!(
        base.contains(&format!(
            "  - \"id\": \"FR1\"\n    \"section\": \"requirements\"\n    \"sum\": \"{FR1_SUM}\"\n"
        )),
        "{base}"
    );
}
