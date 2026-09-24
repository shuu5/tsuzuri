//! `folio schema`（便 45・docs/design/delivery-45.md §1 (f)）の歯。folio は実行 file の crate なので命令を撃つ。
//! 1. 実の正本: design-intent の写しに --check → 0・「一致」・「22263 byte」。生成区間を sha256sum で測り直して (d) の値。
//! 2. ずれ: 生成区間の 1 byte を書き換えて --check → 1。
//! 3. 印: begin を消す・end を 2 本に・begin と end を入れ替える → 2。
//! 4. 書き直し: ずれた写しに --write → 0・file 全体が元と byte 一致。もう 1 度 → 0・「変わらない」。
//! 5. 旗: --write と --check の両方・どちらも無し → 2。
//! 6. file が無い置き場 → 2・「読めない」。
//! 7. 床は印を見ない: 印 2 本を消した写しに folio check → 合格（git init 済みの写し・tests/ceiling.rs の Work と同じ作り方）。
//!
//! 便 46（docs/design/delivery-46.md §1 (d)）: 命令は 2 本目の file design-note/schema.yaml も順に見る（合格の標準出力は 2 行）。
//! 8. 設計ノートの側の実の正本: --check → 0・2 行目に「design-note/schema.yaml」「15305 byte」・生成区間の要約値が (c) の値。
//! 9. 設計ノートの側のずれ: 生成区間の 1 byte を書き換えて --check → 1。
//! 10. 設計ノートの側の印: begin を消す → 2。
//! 11. 設計ノートの側の書き直し: ずれた写しに --write → 0・file 全体が元と byte 一致。
//!
//! 便 57（docs/design/delivery-57.md §1 (c)）: 設計ノートの側の注 4 つに「未実装である」を足した（8 の定数を 15305 byte と新しい要約値に）。
//! 20. 設計ノートの側の実の生成区間が凍結 anchor tests/fixtures/schema/note-region.txt と byte 一致 ∧ 注 folio_check_note の行に「未実装である」を含む。
//!     便 119（docs/design/delivery-119.md §1 (c)(f)）: 注 3 つは「未実装である」を含まず字 folio derive --check を含む形に改め、名を
//!     schema_design_note_region_matches_the_frozen_anchor_and_names_the_derive_command に（8 の定数を 16721 byte と新しい要約値に）。
//!
//! 便 58（docs/design/delivery-58.md §1 (e)）: 承認者の値域に orchestrator 席 を足した（1 の定数を 22254 byte と新しい要約値に）。
//! 21. 判断の記録の側の実の生成区間が凍結 anchor tests/fixtures/schema/adr-region.txt と byte 一致 ∧ approver の行に orchestrator 席 を含む。
//!
//! 便 69（docs/design/delivery-69.md §1 (d)）: 注 prose_note の R-9 の母集団に 語彙 を足した（1 の定数を 22263 byte と新しい要約値に）。
//! 22. design-intent の写しに --check → 0 ∧ 実の生成区間の prose_note の行が「憲法・rules・要件書・語彙〕」を含み「憲法・rules・要件書〕」を含まない。
//! 23. 凍結 anchor の自己検査: adr-region.txt が 117 行・22263 byte・(d) の要約値（測れなければ落とす）。
//!
//! 便 121（docs/design/delivery-121.md §1 (d)(g)）: 列の根を憲法の名で引く表 root_digests に替えた（1 の定数を 25326 byte と新しい要約値に）。
//!
//! 便 89（docs/design/delivery-89.md §1）: 残る 6 本の正本（天井の正本・規則の表・入口の正本・要件書・語彙・相談窓口）の側の歯は tests/schema_docs.rs へ移した（字は 1 字も変えていない）。

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// (d) 凍結 anchor: planner が独立の Python で組んだ生成区間の実測（便 58 (b) で承認者の値域に
/// orchestrator 席 を足し、便 69 (b) で注 prose_note の母集団に 語彙 を足し、便 92 (d) で帰結の欄 produced と
/// その注を、便 101 (d) で改訂の欄 revises を足した後の値・tests/fixtures/schema/adr-region.txt と同じ byte）。
/// 便 121 (d): 列の根の欄を表 root_digests の 2 行（folio2 の行）にし、注 anchor_note の 2 項の書き換えと 1 項の挿入・
/// 注 limits_note の末項を直した後の値（anchor は字面の置き換えで作り、導出と byte 一致を adr.rs の単体の歯が見る）。
const REGION_LINES: usize = 138;
const REGION_BYTES: usize = 25326;
const REGION_SHA256: &str = "7a500d7a811bd698dac4f8feb10756785e449e973f3e9f434e737767f16f2ee1";

/// 便 46 (c) → 便 57 (b) 凍結 anchor: design-note/schema.yaml の生成区間（設計判断の席が独立の実装で組んだ・
/// tests/fixtures/schema/note-region.txt と同じ byte・便 103 で索引の節を指す欄 4 つに改め、便 119 で導出物の検査の命令の名と
/// 注 3 つを直した後の値）。
const NOTE_REGION_LINES: usize = 137;
const NOTE_REGION_BYTES: usize = 16721;
const NOTE_REGION_SHA256: &str = "d10d261b77f94b3410fdda4f4e3672f94ba11647d1e14ff222685b4e895ed83b";

/// 命令が見る file の数（合格の標準出力の行数・判断の記録 → 設計ノート → 天井の正本 → 規則の表 → 入口の正本
/// → 要件書 → 語彙 → 相談窓口 → 索引の欄の決まり）。
const TARGETS: usize = 9;

const BEGIN: &str = "# folio:schema:begin — 生成区間・手で直さない・正本は実装の定数（folio schema --write が書く）";
const END: &str = "# folio:schema:end";

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

/// 要約値（sha256）は命令 `sha256sum` を子の処理で撃って測る（歯は crate の中を読めない・tests/bundle.rs と同じ形）。
fn sha256_hex(bytes: &[u8]) -> Result<String, String> {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("sha256sum を起動できない: {e}"))?;
    child
        .stdin
        .take()
        .ok_or_else(|| "sha256sum の標準入力が無い".to_string())?
        .write_all(bytes)
        .map_err(|e| format!("sha256sum へ書けない: {e}"))?;
    let out = child
        .wait_with_output()
        .map_err(|e| format!("sha256sum を待てない: {e}"))?;
    if !out.status.success() {
        return Err("sha256sum が失敗した".to_string());
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let hex = text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    if hex.len() != 64 {
        return Err(format!("sha256sum の出力が 16 進 64 字でない: {text}"));
    }
    Ok(hex)
}

/// design-intent の写しの一時 dir（歯の終わりに消す）。git init + 1 commit 済み（歯 7 の folio check のため）。
struct Work {
    root: PathBuf,
}

impl Work {
    fn new(case: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-schema-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_tree(
            &repo_root().join("design-intent"),
            &root.join("design-intent"),
        );
        fs::create_dir_all(root.join("contracts")).unwrap();
        fs::copy(
            repo_root().join("contracts/schema.toml"),
            root.join("contracts/schema.toml"),
        )
        .unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-q", "-m", "fixture"]);
        Work { root }
    }

    fn dir(&self) -> PathBuf {
        self.root.join("design-intent")
    }

    fn schema_yaml(&self) -> PathBuf {
        self.dir().join("adr/schema.yaml")
    }

    fn note_schema_yaml(&self) -> PathBuf {
        self.dir().join("design-note/schema.yaml")
    }

    fn read(&self) -> String {
        fs::read_to_string(self.schema_yaml()).unwrap()
    }

    fn read_note(&self) -> String {
        fs::read_to_string(self.note_schema_yaml()).unwrap()
    }

    /// 写しの adr/schema.yaml の字面の変異（1 か所だけ）。
    fn mutate(&self, from: &str, to: &str) {
        mutate_file(&self.schema_yaml(), from, to);
    }

    /// 写しの design-note/schema.yaml の字面の変異（1 か所だけ）。
    fn mutate_note(&self, from: &str, to: &str) {
        mutate_file(&self.note_schema_yaml(), from, to);
    }

    fn schema(&self, flags: &[&str]) -> Output {
        folio(&["schema", "--dir"], &self.dir(), flags)
    }

    fn check(&self) -> Output {
        folio(&["check", "--dir"], &self.dir(), &[])
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// file の字面の変異（当て先は 1 か所だけ）。
fn mutate_file(path: &Path, from: &str, to: &str) {
    let before = fs::read_to_string(path).unwrap();
    assert_eq!(
        before.matches(from).count(),
        1,
        "変異の当て先が 1 か所でない: {from:?}"
    );
    fs::write(path, before.replacen(from, to, 1)).unwrap();
}

fn folio(head: &[&str], dir: &Path, tail: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(head)
        .arg(dir)
        .args(tail)
        .output()
        .expect("folio を起動できない")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// 終了コードと、標準出力（0 のとき）か標準エラー（それ以外）に含む語。合格の標準出力は file ごとの 1 行（`TARGETS` 行）。
fn assert_outcome(out: &Output, code: i32, words: &[&str]) {
    let shown = format!("{}{}", stdout(out), stderr(out));
    assert_eq!(out.status.code(), Some(code), "{shown}");
    let stream = if code == 0 { stdout(out) } else { stderr(out) };
    for word in words {
        assert!(stream.contains(word), "「{word}」が無い: {shown}");
    }
    if code != 0 {
        assert!(stdout(out).is_empty(), "標準出力は空のはず: {shown}");
        assert!(stderr(out).starts_with("folio schema: "), "{shown}");
    }
    assert_eq!(
        stdout(out).lines().count(),
        if code == 0 { TARGETS } else { 0 },
        "{shown}"
    );
}

/// 生成区間（begin の行の次から end の行の手前まで）。
fn region(text: &str) -> &str {
    let b = text.find(&format!("{BEGIN}\n")).expect("begin が無い") + BEGIN.len() + 1;
    let e = text.find(&format!("\n{END}\n")).expect("end が無い") + 1;
    &text[b..e]
}

// ── 1. 実の正本 ──

#[test]
fn schema_check_matches_the_real_file_and_its_frozen_digest() {
    let w = Work::new("real");
    assert_outcome(
        &w.schema(&["--check"]),
        0,
        &["一致", "adr/schema.yaml", &format!("{REGION_BYTES} byte")],
    );
    let text = w.read();
    let cur = region(&text);
    assert_eq!(cur.len(), REGION_BYTES, "生成区間の byte 数");
    assert_eq!(cur.lines().count(), REGION_LINES, "生成区間の行数");
    assert!(cur.starts_with("schema:\n"));
    match sha256_hex(cur.as_bytes()) {
        Ok(hex) => assert_eq!(hex, REGION_SHA256, "sha256sum で測り直した要約値"),
        // 測る道具が無いときは合格にしない代わりに理由を出す（panic で落とさない・FR5 の形）
        Err(why) => eprintln!("# まだ分からない: 要約値を測れない: {why}"),
    }
    // 検査は file を書かない
    assert_eq!(w.read(), text);
}

// ── 2. ずれ ──

#[test]
fn schema_check_fails_on_one_byte_drift_inside_the_region() {
    let w = Work::new("drift");
    w.mutate(
        "\n  options_rule: {min: 2, adopted: 1}\n",
        "\n  options_rule: {min: 3, adopted: 1}\n",
    );
    assert_outcome(
        &w.schema(&["--check"]),
        1,
        &[
            "adr/schema.yaml",
            "生成区間",
            "≠ 導出",
            &format!("{REGION_BYTES} byte"),
        ],
    );
}

// ── 3. 印 ──

#[test]
fn schema_check_is_unknown_without_the_begin_marker() {
    let w = Work::new("no-begin");
    w.mutate(&format!("{BEGIN}\n"), "");
    assert_outcome(
        &w.schema(&["--check"]),
        2,
        &["adr/schema.yaml", "印が 1 対でない"],
    );
    assert_outcome(&w.schema(&["--write"]), 2, &["印が 1 対でない"]);
}

#[test]
fn schema_check_is_unknown_with_two_end_markers() {
    let w = Work::new("two-ends");
    w.mutate(&format!("\n{END}\n"), &format!("\n{END}\n{END}\n"));
    assert_outcome(&w.schema(&["--check"]), 2, &["印が 1 対でない"]);
}

#[test]
fn schema_check_is_unknown_when_markers_are_reversed() {
    let w = Work::new("reversed");
    w.mutate(&format!("{BEGIN}\n"), &format!("{END}\n"));
    w.mutate(
        &format!("\n{END}\n\nplain:"),
        &format!("\n{BEGIN}\n\nplain:"),
    );
    let text = w.read();
    assert!(text.find(END).unwrap() < text.find(BEGIN).unwrap());
    assert_outcome(&w.schema(&["--check"]), 2, &["印が 1 対でない"]);
}

// ── 4. 書き直し ──

#[test]
fn schema_write_restores_the_region_and_is_idempotent() {
    let w = Work::new("write");
    let original = w.read();
    w.mutate(
        "\n  options_rule: {min: 2, adopted: 1}\n",
        "\n  options_rule: {min: 3, adopted: 1}\n",
    );
    assert_ne!(w.read(), original);
    assert_outcome(
        &w.schema(&["--write"]),
        0,
        &["書いた", "adr/schema.yaml", &format!("{REGION_BYTES} byte")],
    );
    assert_eq!(
        w.read(),
        original,
        "file 全体が元と byte 一致（生成区間の外も不変）"
    );
    assert_outcome(
        &w.schema(&["--write"]),
        0,
        &["変わらない", &format!("{REGION_BYTES} byte")],
    );
    assert_eq!(w.read(), original);
    assert_outcome(&w.schema(&["--check"]), 0, &["一致"]);
}

/// 生成区間の外（頭の注・meta・plain）を変えても --write は触らない。
#[test]
fn schema_write_leaves_bytes_outside_the_region_alone() {
    let w = Work::new("outside");
    w.mutate("\nplain: ", "\nplain: 外の変更 ");
    w.mutate("\n  options_rule_note: ", "\n  options_rule_note: ずれ ");
    let outside = w.read();
    assert_outcome(&w.schema(&["--write"]), 0, &["書いた"]);
    let after = w.read();
    assert!(after.contains("\nplain: 外の変更 "));
    assert!(!after.contains("options_rule_note: ずれ "));
    assert_eq!(region(&after).len(), REGION_BYTES);
    // 区間の外は 1 byte も変わらない
    let (head_before, _) = outside.split_once(BEGIN).unwrap();
    let (head_after, _) = after.split_once(BEGIN).unwrap();
    assert_eq!(head_before, head_after);
    let (_, tail_before) = outside.rsplit_once(END).unwrap();
    let (_, tail_after) = after.rsplit_once(END).unwrap();
    assert_eq!(tail_before, tail_after);
}

// ── 5. 旗 ──

#[test]
fn schema_refuses_both_flags_and_no_flag() {
    let w = Work::new("flags");
    let out = w.schema(&["--write", "--check"]);
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    let out = w.schema(&[]);
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert_eq!(
        w.read(),
        fs::read_to_string(repo_root().join("design-intent/adr/schema.yaml")).unwrap()
    );
}

// ── 6. file が無い置き場 ──

#[test]
fn schema_is_unknown_when_the_file_is_missing() {
    let w = Work::new("missing");
    fs::remove_file(w.schema_yaml()).unwrap();
    assert_outcome(&w.schema(&["--check"]), 2, &["adr/schema.yaml", "読めない"]);
    assert_outcome(&w.schema(&["--write"]), 2, &["読めない"]);
    assert!(!w.schema_yaml().exists(), "--write は無い file を作らない");
    let nowhere = w.root.join("nowhere");
    let out = folio(&["schema", "--dir"], &nowhere, &["--check"]);
    assert_outcome(&out, 2, &["読めない"]);
}

// ── 7. 床は印を見ない ──

#[test]
fn schema_markers_are_invisible_to_folio_check() {
    let w = Work::new("no-markers");
    w.mutate(&format!("{BEGIN}\n"), "");
    w.mutate(&format!("{END}\n"), "");
    assert!(!w.read().contains("folio:schema:"));
    let out = w.check();
    let shown = format!("{}{}", stdout(&out), stderr(&out));
    assert_eq!(out.status.code(), Some(0), "{shown}");
    assert!(
        stdout(&out).contains("folio check: 合格（違反 0・まだ分からない 0）"),
        "{shown}"
    );
    // 印が無いので folio schema は「まだ分からない」＝床の検査とは別の口
    assert_outcome(&w.schema(&["--check"]), 2, &["印が 1 対でない"]);
}

// ── 便 46: 設計ノートの側 ──

/// 設計ノートの側の生成区間の変異（1 byte・n_rule の start）。
const NOTE_DRIFT_FROM: &str = "\n    n_rule: {start: 1, order: ascending,";
const NOTE_DRIFT_TO: &str = "\n    n_rule: {start: 2, order: ascending,";

// ── 8. 設計ノートの側の実の正本 ──

#[test]
fn schema_check_matches_the_real_design_note_file_and_its_frozen_digest() {
    let w = Work::new("note-real");
    let out = w.schema(&["--check"]);
    assert_outcome(
        &out,
        0,
        &[
            "一致",
            "design-note/schema.yaml",
            &format!("{NOTE_REGION_BYTES} byte"),
        ],
    );
    let lines: Vec<String> = stdout(&out).lines().map(str::to_string).collect();
    assert_eq!(lines.len(), TARGETS, "{lines:?}");
    assert!(lines[0].contains("adr/schema.yaml"), "{lines:?}");
    assert!(lines[1].contains("design-note/schema.yaml"), "{lines:?}");
    assert!(
        lines[1].contains(&format!("{NOTE_REGION_BYTES} byte")),
        "{lines:?}"
    );
    let text = w.read_note();
    let cur = region(&text);
    assert_eq!(cur.len(), NOTE_REGION_BYTES, "生成区間の byte 数");
    assert_eq!(cur.lines().count(), NOTE_REGION_LINES, "生成区間の行数");
    assert!(cur.starts_with("schema:\n"));
    match sha256_hex(cur.as_bytes()) {
        Ok(hex) => assert_eq!(hex, NOTE_REGION_SHA256, "sha256sum で測り直した要約値"),
        Err(why) => eprintln!("# まだ分からない: 要約値を測れない: {why}"),
    }
    // 検査は file を書かない
    assert_eq!(w.read_note(), text);
}

// ── 9. 設計ノートの側のずれ ──

#[test]
fn schema_check_fails_on_one_byte_drift_inside_the_design_note_region() {
    let w = Work::new("note-drift");
    let adr = w.read();
    w.mutate_note(NOTE_DRIFT_FROM, NOTE_DRIFT_TO);
    assert_outcome(
        &w.schema(&["--check"]),
        1,
        &[
            "design-note/schema.yaml",
            "生成区間",
            "≠ 導出",
            &format!("{NOTE_REGION_BYTES} byte"),
        ],
    );
    // 1 本目（合格）の行は、2 本目で落ちたときは出さない
    assert_eq!(w.read(), adr);
}

// ── 10. 設計ノートの側の印 ──

#[test]
fn schema_check_is_unknown_without_the_design_note_begin_marker() {
    let w = Work::new("note-no-begin");
    w.mutate_note(&format!("{BEGIN}\n"), "");
    assert_outcome(
        &w.schema(&["--check"]),
        2,
        &["design-note/schema.yaml: 印が 1 対でない"],
    );
    assert_outcome(
        &w.schema(&["--write"]),
        2,
        &["design-note/schema.yaml: 印が 1 対でない"],
    );
}

// ── 11. 設計ノートの側の書き直し ──

#[test]
fn schema_write_restores_the_design_note_region_and_is_idempotent() {
    let w = Work::new("note-write");
    let original = w.read_note();
    let adr = w.read();
    w.mutate_note(NOTE_DRIFT_FROM, NOTE_DRIFT_TO);
    assert_ne!(w.read_note(), original);
    assert_outcome(
        &w.schema(&["--write"]),
        0,
        &[
            "変わらない",
            "adr/schema.yaml",
            "書いた",
            "design-note/schema.yaml",
            &format!("{NOTE_REGION_BYTES} byte"),
        ],
    );
    assert_eq!(
        w.read_note(),
        original,
        "file 全体が元と byte 一致（対応表と平易文も不変）"
    );
    assert_eq!(w.read(), adr, "判断の記録の側は触らない");
    assert_outcome(
        &w.schema(&["--write"]),
        0,
        &["変わらない", &format!("{NOTE_REGION_BYTES} byte")],
    );
    assert_eq!(w.read_note(), original);
    assert_outcome(&w.schema(&["--check"]), 0, &["一致"]);
}

// ── 便 57 → 便 119: 設計ノートの側の注は導出物の命令を名指す ──

// ── 20. 実の生成区間は凍結 anchor と byte 一致・注 3 つが folio derive --check を名指し「未実装である」を含まない ──

#[test]
fn schema_design_note_region_matches_the_frozen_anchor_and_names_the_derive_command() {
    let text =
        fs::read_to_string(repo_root().join("design-intent/design-note/schema.yaml")).unwrap();
    let cur = region(&text);
    let anchor =
        fs::read_to_string(repo_root().join("tests/fixtures/schema/note-region.txt")).unwrap();
    assert_eq!(cur, anchor, "実の生成区間が凍結 anchor と byte 一致");
    assert_eq!(cur.len(), NOTE_REGION_BYTES);
    // 注 3 つ（folio_check_note・derived_note・guards_note）は「未実装である」を含まず、導出物の命令を名指す（便 119）
    for key in ["folio_check_note", "derived_note", "guards_note"] {
        let line = cur
            .lines()
            .find(|l| l.trim_start().starts_with(&format!("{key}: ")))
            .unwrap_or_else(|| panic!("注 {key} の行が無い"));
        assert!(!line.contains("未実装である"), "{key}: {line}");
        assert!(line.contains("folio derive --check"), "{key}: {line}");
    }
}

// ── 便 58: 承認者の値域に orchestrator 席 ──

// ── 21. 実の生成区間は凍結 anchor と byte 一致・値域 approver に orchestrator 席 ──

#[test]
fn schema_adr_region_matches_the_frozen_anchor_and_lists_the_orchestrator_seat() {
    let text = fs::read_to_string(repo_root().join("design-intent/adr/schema.yaml")).unwrap();
    let cur = region(&text);
    let anchor =
        fs::read_to_string(repo_root().join("tests/fixtures/schema/adr-region.txt")).unwrap();
    assert_eq!(cur, anchor, "実の生成区間が凍結 anchor と byte 一致");
    assert_eq!(cur.len(), REGION_BYTES);
    let line = cur
        .lines()
        .find(|l| l.trim_start().starts_with("approver: "))
        .expect("値域 approver の行が無い");
    assert!(line.contains("orchestrator 席"), "{line}");
}

// ── 便 69: 注 prose_note の R-9 の母集団に 語彙 ──

// ── 22. 実の生成区間の注 prose_note は母集団に 語彙 を含む ──

#[test]
fn r9_population_names_the_vocabulary() {
    let w = Work::new("r9-population");
    assert_outcome(&w.schema(&["--check"]), 0, &["一致", "adr/schema.yaml"]);
    let text = w.read();
    let line = region(&text)
        .lines()
        .find(|l| l.trim_start().starts_with("prose_note: "))
        .expect("注 prose_note の行が無い");
    assert!(line.contains("憲法・rules・要件書・語彙〕"), "{line}");
    assert!(!line.contains("憲法・rules・要件書〕"), "{line}");
}

// ── 23. 凍結 anchor の自己検査（要約値を測れなければ落とす）──

#[test]
fn r9_population_anchor_holds() {
    let anchor =
        fs::read_to_string(repo_root().join("tests/fixtures/schema/adr-region.txt")).unwrap();
    assert_eq!(anchor.lines().count(), REGION_LINES, "anchor の行数");
    assert_eq!(anchor.len(), REGION_BYTES, "anchor の byte 数");
    let hex = sha256_hex(anchor.as_bytes())
        .unwrap_or_else(|why| panic!("要約値を測れない（素通りにしない）: {why}"));
    assert_eq!(hex, REGION_SHA256, "sha256sum で測った anchor の要約値");
}
