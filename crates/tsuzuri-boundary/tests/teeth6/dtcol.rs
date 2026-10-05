//! 契約表の欄の写しを器の今の版に揃え、設計ノートの行の欄 done-teeth と code-facts を tz が受けて契約表へ写す歯（接頭辞 dtcol_・
//! 判断の記録 ADR-45 の門 H1 の teeth-check の前の 1 歩）。根の写し contracts/field-schema/schema.toml は、同じ repo の器の生成物
//! scribe2/contracts/schema.toml（器の歯が core の FIELDS と byte で照らす）と頭の 1 行（注）の後ろが byte で同じ。置き場は
//! CARGO_TARGET_TMPDIR の下に歯ごとに作り、design-intent/design-note/ に手で書いた小さな見本の設計ノートと、
//! contracts/field-schema/schema.toml に根の写し（か 1 欄を抜いた写し）を置いて tz derive と tz check を撃つ。
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

/// 見本の設計ノート（行 a は flow の引用符の形・行 b は block の裸の形で欄を書く）。
const NOTE: &str = "meta:
  id: dtcol-note
  title: 歯の欄の見本
  version: v0.1
  status: example
  generated: 2026-10-05
  profile: design-note
sections:
  - n: 1
    type: prose
    title: 目的
    body: 歯の欄を写す。
  - n: 2
    type: contract-table
    title: 契約表
    rows:
      - id: a
        title: 引用符の行
        req: [FR11]
        section: \"1\"
        verify: [\"git apply --reverse --check p.patch\", \"cargo nextest run -p toy --test dtx --no-tests=fail dtx_\"]
        size: S
        done: (1) 差が在る (2) 歯が緑
        growth: [\"src/a.rs:10\"]
        done-teeth: [\"2:dtx_new_tooth\", \"1:@1\", \"2:=dtx_kept_tooth\", \"1:!write-set\"]
        code-facts: [\"defs:src/a.rs=3\"]
      - id: b
        title: 裸の行
        req: [FR11]
        section: \"1\"
        verify: [\"cargo nextest run -p toy --test dtx --no-tests=fail dtx_\"]
        size: S
        done: (1) 歯が緑
        done-teeth:
          - 1:dtx_other_tooth
          - 1:!new-file
";

/// 見本の設計ノートの導出物（欄は器の宣言の順・欄 done-teeth と code-facts は growth の後で goal の前・要素は書いた順のまま）。
const DERIVED: &str = "schema = 1

[[contract]]
id = \"a\"
title = \"引用符の行\"
req = [\"FR11\"]
section = \"1\"
verify = [\"git apply --reverse --check p.patch\", \"cargo nextest run -p toy --test dtx --no-tests=fail dtx_\"]
size = \"S\"
done = \"(1) 差が在る (2) 歯が緑\"
growth = [\"src/a.rs:10\"]
done-teeth = [\"2:dtx_new_tooth\", \"1:@1\", \"2:=dtx_kept_tooth\", \"1:!write-set\"]
code-facts = [\"defs:src/a.rs=3\"]
goal = \"歯の欄を写す。\"

[[contract]]
id = \"b\"
title = \"裸の行\"
req = [\"FR11\"]
section = \"1\"
verify = [\"cargo nextest run -p toy --test dtx --no-tests=fail dtx_\"]
size = \"S\"
done = \"(1) 歯が緑\"
done-teeth = [\"1:dtx_other_tooth\", \"1:!new-file\"]
goal = \"歯の欄を写す。\"
";

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

/// 根の写しから欄 `name` の塊だけを抜いた字（塊はちょうど 1 度在る）。
fn copy_without(name: &str) -> String {
    let copy = read(COPY);
    assert_eq!(copy.matches(&block(name)).count(), 1, "{name}");
    copy.replacen(&block(name), "", 1)
}

/// 歯ごとの置き場（design-intent/ に正本・contracts/field-schema/ に写し・out/ に導出物）。
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

    /// 見本の設計ノート 1 本の置き場（`note` は見本の字の 1 か所を替えた字でもよい）。
    fn note(name: &str, copy: &str, note: &str) -> Place {
        let p = Place::new(name, copy);
        fs::create_dir_all(p.root.join("design-intent/design-note")).expect("正本の置き場");
        fs::write(
            p.root.join("design-intent/design-note/dtcol-note.yaml"),
            note,
        )
        .expect("正本を書く");
        p
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

    fn derive(&self) -> (i32, String) {
        self.tz(&[
            "derive",
            "--write",
            "--dir",
            "design-intent",
            "--out",
            "../out",
        ])
    }

    fn derived(&self) -> Option<String> {
        fs::read_to_string(self.root.join("out/dtcol-note.toml")).ok()
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
fn dtcol_derive_writes_the_two_fields_after_growth() {
    let p = Place::note("write", &read(COPY), NOTE);
    let (rc, out) = p.derive();
    assert_eq!(rc, 0, "{out}");
    assert!(out.contains("書いた 1 file"), "{out}");
    assert_eq!(
        p.derived().as_deref(),
        Some(DERIVED),
        "欄は growth の後・goal の前で要素は書いた順"
    );
    assert_eq!(
        p.tz(&[
            "derive",
            "--check",
            "--dir",
            "design-intent",
            "--out",
            "../out"
        ])
        .0,
        0,
        "--check は一致"
    );
}

#[test]
fn dtcol_derive_refuses_a_text_done_teeth() {
    let from = "        done-teeth:\n          - 1:dtx_other_tooth\n          - 1:!new-file\n";
    assert_eq!(NOTE.matches(from).count(), 1);
    let p = Place::note(
        "text",
        &read(COPY),
        &NOTE.replacen(from, "        done-teeth: 1:dtx_other_tooth\n", 1),
    );
    let (rc, out) = p.derive();
    assert_eq!(rc, 2, "{out}");
    assert!(
        out.contains("行 b: 欄「done-teeth」が list の形でない"),
        "{out}"
    );
    assert_eq!(p.derived(), None, "何も書かない");
}

#[test]
fn dtcol_copy_without_a_field_refuses_it() {
    for (name, rest) in [("done-teeth", "code-facts"), ("code-facts", "done-teeth")] {
        let p = Place::note(name, &copy_without(name), NOTE);
        let (rc, out) = p.derive();
        assert_eq!(rc, 2, "{name}: {out}");
        assert!(
            out.contains(&format!("欄「{name}」が器の導出 file に無い")),
            "{name}: {out}"
        );
        assert!(
            !out.contains(&format!("欄「{rest}」")),
            "{name}: 残した欄 {rest} は断らない: {out}"
        );
        assert_eq!(p.derived(), None, "{name}: 何も書かない");
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
