//! 契約表の欄の写しを器の今の版に揃え、設計ノートの行の欄 done-teeth と code-facts を tz の床が受ける歯（接頭辞 dtcol_・
//! 判断の記録 ADR-45 の門 H1 の teeth-check の前の 1 歩）。根の写し contracts/field-schema/schema.toml は、同じ repo の器の生成物
//! scribe2/contracts/schema.toml（器の歯が core の FIELDS と byte で照らす）と頭の 1 行（注）の後ろが byte で同じ。置き場は
//! CARGO_TARGET_TMPDIR の下に歯ごとに作り、床の見本の写しに根の写しを置いて tz check を撃つ。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 根の写し（repo の根からの相対）。
const COPY: &str = "contracts/field-schema/schema.toml";

/// 器の生成物（repo の根からの相対）。
const VESSEL: &str = "scribe2/contracts/schema.toml";

/// tz check の床の見本の置き場（repo の根からの相対）。
const FLOOR_BASE: &str = "folio2/tests/fixtures/floor_base/design-intent";

/// repo の根。
fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo().join(rel)).expect(rel)
}

/// 写しの中の欄 1 つの塊（任意の list）。
fn block(name: &str) -> String {
    format!("[[field]]\nname = \"{name}\"\nneed = \"optional\"\nshape = \"list\"\n\n")
}

/// 歯ごとの置き場（design-intent/ に正本・contracts/field-schema/ に写し）。
struct Place {
    root: PathBuf,
}

impl Place {
    fn new(name: &str, copy: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("dtcol")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("contracts/field-schema")).expect("写しの置き場");
        fs::write(root.join(COPY), copy).expect("写しを書く");
        let p = Place { root };
        // 版管理の根を置き場にする（無ければ tz は置き場を包む repo の根の写しを読む）
        p.git(&["init", "-q"]);
        p
    }

    fn git(&self, args: &[&str]) {
        let ok = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(args)
            .status()
            .expect("git");
        assert!(ok.success(), "git {args:?}");
    }

    /// 床の見本の写しを置いて版管理に commit した置き場（床は commit の無い置き場を「まだ分からない」と読む）。
    fn floor(name: &str) -> Place {
        let p = Place::new(name, &read(COPY));
        copy_tree(&repo().join(FLOOR_BASE), &p.root.join("design-intent"));
        p.git(&["add", "-A"]);
        p.git(&[
            "-c",
            "user.name=dtcol",
            "-c",
            "user.email=dtcol@example.invalid",
            "commit",
            "-qm",
            "floor",
        ]);
        p
    }

    /// 床の見本の行 a の末に字 `extra` を足す（元の字はちょうど 1 度在る）。
    fn swap_row(&self, extra: &str) {
        let path = self.root.join("design-intent/design-note/example.yaml");
        let text = fs::read_to_string(&path).expect("見本を読む");
        let from = "depends: []}";
        assert_eq!(text.matches(from).count(), 1, "{from}");
        fs::write(
            &path,
            text.replacen(from, &format!("depends: [], {extra}}}"), 1),
        )
        .expect("見本を書く");
    }

    fn tz(&self, args: &[&str]) -> (i32, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("tz を撃つ");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        (out.status.code().expect("終了 code"), text)
    }
}

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("写す先");
    for entry in fs::read_dir(src).expect("写す元") {
        let entry = entry.expect("entry");
        let to = dst.join(entry.file_name());
        if entry.file_type().expect("種類").is_dir() {
            copy_tree(&entry.path(), &to);
        } else {
            fs::copy(entry.path(), &to).expect("写す");
        }
    }
}

#[test]
fn dtcol_copy_matches_the_vessel_schema() {
    let (copy, vessel) = (read(COPY), read(VESSEL));
    let head = copy.lines().next().expect("頭の行");
    assert!(
        head.starts_with("# ") && head.contains(VESSEL),
        "頭の 1 行は注で写した元を名指す: {head}"
    );
    assert!(vessel.starts_with("# "), "器の生成物の頭は注の 1 行");
    let body = |text: &str| text.split_once('\n').map(|(_, rest)| rest.to_owned());
    assert_eq!(
        body(&copy),
        body(&vessel),
        "頭の 1 行の後ろが器の生成物と byte で同じ"
    );
    for name in ["done-teeth", "code-facts"] {
        assert_eq!(
            copy.matches(&block(name)).count(),
            1,
            "写しは欄 {name} を任意の list で 1 度持つ"
        );
    }
}

#[test]
fn dtcol_check_accepts_the_lists_and_refuses_a_text() {
    let p = Place::floor("check");
    p.swap_row("done-teeth: [\"1:@1\", \"1:=f_kept\"], code-facts: [\"defs:src/a.rs=3\"]");
    let (rc, out) = p.tz(&["check", "--dir", "design-intent"]);
    assert_eq!(rc, 0, "{out}");
    let q = Place::floor("check-text");
    q.swap_row("done-teeth: \"1:@1\", code-facts: [\"defs:src/a.rs=3\"]");
    let (rc, out) = q.tz(&["check", "--dir", "design-intent"]);
    assert_eq!(rc, 1, "{out}");
    assert!(
        out.contains("行 a: 欄「done-teeth」が list（文字列の一覧） の形でない"),
        "{out}"
    );
    assert!(
        !out.contains("code-facts"),
        "list の欄 code-facts は断らない: {out}"
    );
}
