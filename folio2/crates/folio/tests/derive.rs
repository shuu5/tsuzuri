//! 便 119（docs/design/delivery-119.md §1 (e)・要件書 FR11・AC9・判断の記録 ADR-16 決定 (5)）: `folio derive` の歯。
//! folio は実行 file の crate なので命令を撃つ。凍結 anchor は tests/fixtures/design-note/ の手書きの対
//! （derive-anchor.yaml = 契約表を持つ設計ノート・derive-anchor.toml = 期待する導出物）で、生成器から独立している（P-10.1）。
//! 1. 対の設計ノートから書いた導出物が期待の file と byte 一致し、--check が 0。
//! 2. 導出物の 1 byte の差・置き場に無い file は --check が 1。導出元の無い .toml は差分に数えず名を出す（ほかの拡張子は数えない）。
//!    --write は置き場の他の file を消さない。置き場が無ければ --check は 2・置き場の親 dir が無ければ --write は 2。
//! 3. 書けない行 9 通り（二重引用符・逆斜線・改行・器の導出 file に無い欄・同じ行 id・散文でない節・必須の欄の欠け・形の違い・
//!    一覧の要素が字でない）と、器の導出 file が無い置き場・設計ノートの置き場が symlink の置き場は 2 で、書ける設計ノートが
//!    同じ置き場に在っても何も書かない（全部か無しか）。
//! 4. 章の上限を超えて面が導出できない設計ノートでも導出でき、様式の file の無い置き場でも同じ byte を書く（面の生成器も様式も呼ばない）。
//! 5. 設計ノートの欄の決まりの床の定数（生成区間の check.command）の命令の名は folio derive --check で、--dir と --out を足せば撃てて 0。
//! 6. 実の置き場から導出でき（導出元の設計ノートを持つ file だけ・各 file は行の数と goal の数が等しい）、版管理の導出物（contracts/ の下）は
//!    その集合と一致して --check が 0（実の置き場には書かない）。
//! 7. 便 120（docs/design/delivery-120.md §1 (d) の 4）: 凍結の対 need-conditional.yaml と need-conditional-schema.toml
//!    （verify と done が要否 conditional）から導出した行 a は verify と done を持ち、行 b は持たない（行の値のまま写す）。
//!    導出物に要否の字は写さず、--check が 0。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ANCHOR_NOTE: &str = "tests/fixtures/design-note/derive-anchor.yaml";
const ANCHOR_TOML: &str = "tests/fixtures/design-note/derive-anchor.toml";
const OUT_NAME: &str = "derive-anchor.toml";

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

/// 一時 dir（`<tmp>/design-intent/design-note/` と `<tmp>/contracts/schema.toml`）。落ちても消す。
struct Work {
    root: PathBuf,
}

impl Work {
    /// 凍結の対の設計ノートだけを置いた置き場。
    fn anchor(case: &str) -> Work {
        let w = Work::empty(case);
        fs::create_dir_all(w.dir().join("design-note")).unwrap();
        fs::copy(
            repo_root().join(ANCHOR_NOTE),
            w.dir().join("design-note/derive-anchor.yaml"),
        )
        .unwrap();
        w
    }

    /// 実の design-intent の写し全部（面の生成器も撃てる置き場）。
    fn real(case: &str) -> Work {
        let w = Work::empty(case);
        copy_tree(&repo_root().join("design-intent"), &w.dir());
        w
    }

    fn empty(case: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-derive-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("contracts")).unwrap();
        fs::copy(
            repo_root().join("contracts/schema.toml"),
            root.join("contracts/schema.toml"),
        )
        .unwrap();
        Work { root }
    }

    fn dir(&self) -> PathBuf {
        self.root.join("design-intent")
    }

    fn out(&self) -> PathBuf {
        self.root.join("out")
    }

    fn derive(&self, flag: &str) -> Output {
        derive_at(&self.dir(), &self.out(), &[flag])
    }

    /// 写しの設計ノートの字面の変異（1 か所だけ）。
    fn replace(&self, note: &str, from: &str, to: &str) {
        let path = self.dir().join("design-note").join(note);
        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(
            text.matches(from).count(),
            1,
            "変異の元の字面が 1 か所でない: {from}"
        );
        fs::write(&path, text.replacen(from, to, 1)).unwrap();
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn derive_at(dir: &Path, out: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("derive")
        .args(args)
        .arg("--dir")
        .arg(dir)
        .arg("--out")
        .arg(out)
        .output()
        .expect("folio を起動できない")
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn assert_code(out: &Output, want: i32, words: &[&str]) {
    let all = text(out);
    assert_eq!(out.status.code(), Some(want), "{all}");
    for w in words {
        assert!(all.contains(w), "「{w}」が無い: {all}");
    }
}

fn anchor_toml() -> Vec<u8> {
    fs::read(repo_root().join(ANCHOR_TOML)).unwrap()
}

/// 置き場の直下の file 名（名の昇順）。
fn listing(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .map(|it| {
            it.filter_map(Result::ok)
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

// ── 1. 凍結 anchor と byte 一致 ──

#[test]
fn f119_write_matches_the_frozen_anchor_and_check_passes() {
    let w = Work::anchor("anchor");
    assert_code(&w.derive("--write"), 0, &["書いた 1 file"]);
    assert_eq!(listing(&w.out()), [OUT_NAME], "書いた file は 1 本だけ");
    assert_eq!(
        fs::read(w.out().join(OUT_NAME)).unwrap(),
        anchor_toml(),
        "導出物が手書きの期待の file と byte 一致"
    );
    assert_code(
        &w.derive("--write"),
        0,
        &["書いた 0 file", "変わらない 1 file"],
    );
    assert_code(&w.derive("--check"), 0, &["一致"]);
}

// ── 2. 1 byte の差・無い・導出元の無い file ──

#[test]
fn f119_one_byte_drift_and_missing_are_nonzero_and_sourceless_files_are_not_counted() {
    let w = Work::anchor("drift");
    assert_code(&w.derive("--check"), 2, &["置き場が無い"]);
    fs::create_dir_all(w.out()).unwrap();
    let path = w.out().join(OUT_NAME);
    fs::write(&path, anchor_toml()).unwrap();
    assert_code(&w.derive("--check"), 0, &["一致"]);
    // 1 byte だけ変える（行 a の id の字）
    let mut bytes = anchor_toml();
    let at = bytes
        .windows(8)
        .position(|x| x == b"id = \"a\"")
        .expect("行 a の id の字面");
    bytes[at + 6] = b'c';
    fs::write(&path, &bytes).unwrap();
    assert_code(&w.derive("--check"), 1, &["DRIFT", OUT_NAME]);
    fs::remove_file(&path).unwrap();
    assert_code(&w.derive("--check"), 1, &["置き場に無い", OUT_NAME]);
    // 導出元の無い .toml（器の導出 file の写しを含む）は差分に数えず、名を 1 行ずつ出す。ほかの拡張子は名も出さない
    fs::write(&path, anchor_toml()).unwrap();
    fs::write(w.out().join("old.toml"), "schema = 1\n").unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        w.out().join("schema.toml"),
    )
    .unwrap();
    fs::write(w.out().join("keep.txt"), "toml でない file は数えない\n").unwrap();
    let checked = w.derive("--check");
    assert_code(
        &checked,
        0,
        &[
            "一致",
            "数えない: old.toml（導出元なし）",
            "数えない: schema.toml（導出元なし）",
        ],
    );
    assert!(!text(&checked).contains("keep.txt"), "{}", text(&checked));
    // 終了コードは導出元を持つ file だけで決まる
    fs::write(&path, &bytes).unwrap();
    assert_code(&w.derive("--check"), 1, &["DRIFT", "数えない: old.toml"]);
    assert_code(
        &w.derive("--write"),
        0,
        &["書いた 1 file", "数えない: old.toml"],
    );
    assert_eq!(
        listing(&w.out()),
        [OUT_NAME, "keep.txt", "old.toml", "schema.toml"],
        "--write は置き場の他の file を消さない"
    );
    assert_eq!(fs::read(&path).unwrap(), anchor_toml());
    // 置き場の親 dir が無ければ作らずに 2
    let nowhere = w.root.join("no/such/out");
    assert_code(
        &derive_at(&w.dir(), &nowhere, &["--write"]),
        2,
        &["親 dir が無い"],
    );
    assert!(!w.root.join("no").exists(), "親 dir を作らない");
}

// ── 3. 書けない行は 2・何も書かない ──

#[test]
fn f119_rows_it_cannot_write_are_unknown_and_nothing_is_written() {
    let cases: [(&str, &str, &str, &str); 9] = [
        (
            "quote",
            "title: 導出物を組む口,",
            "title: '導出物を\"組む\"口',",
            "書けない字",
        ),
        (
            "backslash",
            "title: 導出物を組む口,",
            "title: 導出物を\\組む口,",
            "書けない字",
        ),
        (
            "newline",
            "title: 導出物を組む口,",
            "title: \"導出物を\\n組む口\",",
            "書けない字",
        ),
        (
            "unknown-field",
            "id: b, depends: [a],",
            "id: b, extra: [x], depends: [a],",
            "器の導出 file に無い",
        ),
        (
            "same-id",
            "id: b, depends: [a],",
            "id: a, depends: [a],",
            "2 度在る",
        ),
        (
            "not-prose",
            "section: \"1\",",
            "section: \"3\",",
            "prose の節でない",
        ),
        ("no-size", "size: S, ", "", "必須の欄「size」"),
        ("text-as-list", "size: M,", "size: [M],", "text の形でない"),
        (
            "list-item-not-text",
            "req: [FR11],",
            "req: [{x: y}],",
            "要素が文字列でない",
        ),
    ];
    for (case, from, to, why) in cases {
        let w = Work::anchor(&format!("refuse-{case}"));
        // 名の昇順で先に来る、書ける設計ノート（全部か無しかを見る）
        fs::copy(
            repo_root().join(ANCHOR_NOTE),
            w.dir().join("design-note/a-ok.yaml"),
        )
        .unwrap();
        w.replace("derive-anchor.yaml", from, to);
        fs::create_dir_all(w.out()).unwrap();
        fs::write(w.out().join("keep.txt"), "前から在る\n").unwrap();
        assert_code(&w.derive("--write"), 2, &["まだ分からない", why]);
        assert_eq!(
            listing(&w.out()),
            ["keep.txt"],
            "{case}: 置き場に何も書かない"
        );
        assert_code(&w.derive("--check"), 2, &[why]);
    }
    let w = Work::anchor("refuse-no-schema");
    fs::remove_file(w.root.join("contracts/schema.toml")).unwrap();
    assert_code(
        &w.derive("--write"),
        2,
        &["contracts/schema.toml", "読めない"],
    );
    assert!(!w.out().exists(), "置き場を作らない");
    // 設計ノートの置き場が symlink なら読まずに 2
    let w = Work::anchor("refuse-symlink");
    let real = w.root.join("real-notes");
    fs::rename(w.dir().join("design-note"), &real).unwrap();
    std::os::unix::fs::symlink(&real, w.dir().join("design-note")).unwrap();
    assert_code(&w.derive("--write"), 2, &["design-note/ が dir として無い"]);
    assert!(!w.out().exists(), "置き場を作らない");
}

// ── 4. 面の生成器も様式の file も呼ばない ──

#[test]
fn f119_derive_does_not_need_the_face_generator_or_the_style_files() {
    let w = Work::real("no-face");
    let sections: String = (1..=13)
        .map(|i| format!("  - {{n: {i}, type: prose, title: 節 {i}, body: 本文 {i}。}}\n"))
        .collect();
    let big = format!(
        "meta:\n  id: big\n  title: 章の上限を超える設計ノート\n  version: v0.1\n  status: example\n  generated: 2026-09-24\n  profile: design-note\n\nsections:\n{sections}  - n: 14\n    type: contract-table\n    title: 契約表\n    rows:\n      - {{id: a, title: 13 節目の便, req: [FR11], section: \"13\", verify: [cargo nextest run -p folio --test derive f119_], size: S, done: 導出できる}}\n"
    );
    fs::write(w.dir().join("design-note/big.yaml"), big).unwrap();
    let face = Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(["face", "--face", "note", "--id", "big", "--write", "--dir"])
        .arg(w.dir())
        .arg("--out")
        .arg(w.root.join("big.html"))
        .output()
        .expect("folio を起動できない");
    assert_code(&face, 2, &["上限 12"]);
    assert_code(&w.derive("--write"), 0, &["書いた"]);
    assert!(listing(&w.out()).contains(&"big.toml".to_string()));
    let first = fs::read(w.out().join("big.toml")).unwrap();
    assert!(
        String::from_utf8_lossy(&first).ends_with("section = \"13\"\nverify = [\"cargo nextest run -p folio --test derive f119_\"]\nsize = \"S\"\ndone = \"導出できる\"\ngoal = \"本文 13。\"\n"),
        "{}",
        String::from_utf8_lossy(&first)
    );
    // 様式の file（preview/）を置き場から外しても同じ byte を書く
    fs::remove_dir_all(w.dir().join("preview")).unwrap();
    fs::remove_dir_all(w.out()).unwrap();
    assert_code(&w.derive("--write"), 0, &["書いた"]);
    assert_eq!(fs::read(w.out().join("big.toml")).unwrap(), first);
}

// ── 5. 床の定数が名指す命令 ──

#[test]
fn f119_the_floor_names_the_derive_command_and_it_runs() {
    let schema =
        fs::read_to_string(repo_root().join("design-intent/design-note/schema.yaml")).unwrap();
    let line = schema
        .lines()
        .map(str::trim_start)
        .find(|l| l.starts_with("check: {command: "))
        .expect("生成区間の check の行が無い");
    let command = line
        .trim_start_matches("check: {command: ")
        .split(',')
        .next()
        .unwrap();
    assert_eq!(command, "folio derive --check", "{line}");
    let words: Vec<&str> = command.split(' ').collect();
    assert_eq!(words[..2], ["folio", "derive"]);
    // --dir と --out は消費側が足す（命令の名そのものには無い）
    let w = Work::anchor("floor-command");
    fs::create_dir_all(w.out()).unwrap();
    fs::write(w.out().join(OUT_NAME), anchor_toml()).unwrap();
    assert_code(&derive_at(&w.dir(), &w.out(), &words[2..]), 0, &["一致"]);
}

// ── 6. 実の置き場と版管理の導出物 ──

#[test]
fn f119_the_real_design_intent_derives_and_its_committed_copy_matches() {
    let out = std::env::temp_dir().join(format!("folio-derive-real-{}", std::process::id()));
    let _ = fs::remove_dir_all(&out);
    let dir = repo_root().join("design-intent");
    assert_code(&derive_at(&dir, &out, &["--write"]), 0, &["書いた"]);
    let names = listing(&out);
    assert!(names.contains(&"example.toml".to_string()), "{names:?}");
    for name in &names {
        let stem = name.strip_suffix(".toml").expect("導出物は .toml");
        assert!(
            dir.join("design-note")
                .join(format!("{stem}.yaml"))
                .is_file(),
            "導出元の設計ノートが無い: {name}"
        );
        let body = fs::read_to_string(out.join(name)).unwrap();
        assert!(
            body.starts_with("schema = 1\n\n[[contract]]\n"),
            "{name}: {body}"
        );
        let rows = body.matches("[[contract]]").count();
        let goals = body.lines().filter(|l| l.starts_with("goal = ")).count();
        assert!(rows >= 1 && rows == goals, "{name}: {body}");
    }
    assert_code(&derive_at(&dir, &out, &["--check"]), 0, &["一致"]);
    let _ = fs::remove_dir_all(&out);
    // 版管理の導出物（contracts/ の下）は導出元を持つ file の集合と一致し、器の導出 file の写しは数えない
    let committed = repo_root().join("contracts");
    let mut tracked: Vec<String> = listing(&committed)
        .into_iter()
        .filter(|n| n.ends_with(".toml") && n != "schema.toml")
        .collect();
    tracked.sort();
    assert_eq!(tracked, names, "contracts/ の導出物の集合");
    assert_code(
        &derive_at(&dir, &committed, &["--check"]),
        0,
        &["一致", "数えない: schema.toml（導出元なし）"],
    );
}

// ── 7. 要否 conditional の欄は行の値のまま写す（便 120） ──

#[test]
fn f120_conditional_fields_are_copied_as_the_row_has_them() {
    let w = Work::empty("f120-conditional");
    let fixtures = repo_root().join("tests/fixtures/design-note");
    fs::copy(
        fixtures.join("need-conditional-schema.toml"),
        w.root.join("contracts/schema.toml"),
    )
    .unwrap();
    fs::create_dir_all(w.dir().join("design-note")).unwrap();
    fs::copy(
        fixtures.join("need-conditional.yaml"),
        w.dir().join("design-note/need-conditional.yaml"),
    )
    .unwrap();
    assert_code(&w.derive("--write"), 0, &["書いた 1 file"]);
    let body = fs::read_to_string(w.out().join("need-conditional.toml")).unwrap();
    let block = |id: &str| -> Vec<String> {
        body.split("[[contract]]")
            .find(|b| b.lines().any(|l| l == format!("id = \"{id}\"")))
            .unwrap_or_else(|| panic!("行 {id} が無い: {body}"))
            .lines()
            .map(str::to_string)
            .collect()
    };
    let has = |lines: &[String], key: &str| lines.iter().any(|l| l.starts_with(&format!("{key} = ")));
    let a = block("a");
    let b = block("b");
    for key in ["verify", "done", "goal"] {
        assert!(has(&a, key), "行 a に {key} が無い: {body}");
    }
    for key in ["verify", "done"] {
        assert!(!has(&b, key), "行 b に {key} が在る: {body}");
    }
    assert!(has(&b, "goal"), "行 b に goal が無い: {body}");
    assert!(!body.contains("conditional"), "要否の字を写した: {body}");
    assert_code(&w.derive("--check"), 0, &["一致"]);
}
