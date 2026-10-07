//! 接頭辞 dvoff_・設計ノートの契約表から導出物を書く口を閉じる行 t-derive-off の歯（判断の記録 ADR-74 決定 (6)(8)）。
//! tz derive は設計ノート design-note/ を読まず、席の手元の写し（規則の表に欄 key が seat-bytes と seat-role-bytes の行が在る
//! 置き場だけ）を置き場の下の dir seat に書く・数える。置き場は CARGO_TARGET_TMPDIR の下に歯ごとに作り、版管理の根にして tz を撃つ。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 契約表の行を持つ設計ノートの見本（前の口なら導出物を 1 本書く）。
const NOTE: &str = "folio2/tests/fixtures/design-note/derive-anchor.yaml";

/// 器の導出 file の写し（根の contracts/field-schema/schema.toml に置く）。
const SCHEMA: &str = "folio2/contracts/schema.toml";

/// 歯ごとの置き場（根で git init する）。
struct Place {
    root: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("dvoff").join(name);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("置き場");
        let ok = Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["init", "-q"])
            .status()
            .expect("git");
        assert!(ok.success(), "git init");
        Place { root }
    }

    /// 見本の設計ノートと器の導出 file の写しを置いた置き場（正本は design-intent/）。
    fn with_note(name: &str) -> Place {
        let p = Place::new(name);
        let repo = repo();
        fs::create_dir_all(p.root.join("design-intent/design-note")).expect("正本の置き場");
        fs::copy(
            repo.join(NOTE),
            p.root.join("design-intent/design-note/derive-anchor.yaml"),
        )
        .expect("見本を写す");
        fs::create_dir_all(p.root.join("contracts/field-schema")).expect("写しの置き場");
        fs::copy(
            repo.join(SCHEMA),
            p.root.join("contracts/field-schema/schema.toml"),
        )
        .expect("写しを置く");
        p
    }

    fn tz(&self, cwd: &str, args: &[&str]) -> (i32, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(args)
            .current_dir(self.root.join(cwd))
            .output()
            .expect("tz を撃つ");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        (out.status.code().expect("終了 code"), text)
    }

    fn derive(&self, mode: &str) -> (i32, String) {
        self.tz(
            ".",
            &["derive", mode, "--dir", "design-intent", "--out", "../out"],
        )
    }
}

/// repo の根。
fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn dvoff_a_contract_table_note_writes_nothing() {
    let p = Place::with_note("note");
    let (rc, out) = p.derive("--write");
    assert_eq!(rc, 0, "{out}");
    assert!(out.contains("書いた 0 file・変わらない 0 file"), "{out}");
    let dir = p.root.join("out");
    assert!(dir.is_dir(), "out が dir として在る");
    assert_eq!(fs::read_dir(&dir).expect("out を読む").count(), 0, "中が空");
}

#[test]
fn dvoff_tables_left_in_the_out_dir_are_not_read() {
    let p = Place::with_note("left");
    let left = p.root.join("out/x.toml");
    fs::create_dir_all(p.root.join("out")).expect("out");
    fs::write(&left, "schema = 1\n").expect("手で書いた x.toml");
    let (rc, out) = p.derive("--check");
    assert_eq!(rc, 0, "{out}");
    assert!(out.contains("一致 0・差分 0"), "{out}");
    assert!(!out.contains("DRIFT"), "{out}");
    assert!(!out.contains("数えない"), "{out}");
    let (rc, out) = p.derive("--write");
    assert_eq!(rc, 0, "{out}");
    assert_eq!(fs::read(&left).expect("x.toml"), b"schema = 1\n");
}

#[test]
fn dvoff_from_root_resolves_out_from_the_repo_root() {
    let p = Place::new("root");
    fs::create_dir_all(p.root.join("docs/di")).expect("置き場");
    let (rc, out) = p.tz(
        "docs",
        &[
            "derive",
            "--dir",
            "di",
            "--out",
            "contracts",
            "--from-root",
            "--write",
        ],
    );
    assert_eq!(rc, 0, "{out}");
    assert!(p.root.join("contracts").is_dir(), "根の下の contracts");
    assert!(!p.root.join("docs/di/contracts").exists(), "置き場の下に無い");
    assert!(!p.root.join("docs/contracts").exists(), "今の dir の下に無い");
}

#[test]
fn dvoff_help_names_only_the_seat_copies() {
    let p = Place::new("help");
    let (rc, out) = p.tz(".", &["derive", "--help"]);
    assert_eq!(rc, 0, "{out}");
    assert!(!out.contains("契約表"), "{out}");
    assert!(out.contains("席の手元の写し"), "{out}");
}
