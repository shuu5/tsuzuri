//! 命令 folio schema が見る 9 本のうち、欄の決まりの 2 本を除いた 7 本の正本（ceiling.yaml・rules.yaml・index.yaml・srs.yaml・vocabulary.yaml・intake.yaml・graph.yaml）の側の歯。
//! 便 89（docs/design/delivery-89.md §1 (b)）で tests/schema.rs から 1 字も変えずに移した（歯の関数名も凍結の定数の値も不変）。
//! helper（repo_root から region まで）は tests/schema.rs の写し（歯の file どうしは互いに use できない）。
//! 以下は移す前の頭の注釈（便ごとの歯の一覧）。
//!
//! 便 48（docs/design/delivery-48.md §1 (c)(d)）: 命令は 3 本目の file ceiling.yaml（天井の正本・生成区間は末尾）も順に見る（合格の標準出力は 3 行）。
//! 12. 天井の正本の側の実の正本: --check → 0・3 行目に「ceiling.yaml」「3176 byte」・生成区間の要約値が (c) の値・行数 27（便 102 で 24 行・2915 byte から）。
//! 13. 天井の正本の側のずれ: 生成区間の 1 byte を書き換えて --check → 1。
//! 14. 天井の正本の側の印: begin を消す → 2。
//! 15. 天井の正本の側の書き直し: ずれた写しに --write → 0・file 全体が元と byte 一致（人が書く節も不変）。
//!
//! 便 53（docs/design/delivery-53.md §1 (c)）: 命令は 4 本目の file rules.yaml（規則の表・生成区間は先頭の注釈の次）も順に見る（合格の標準出力は 4 行）。
//! 16. 規則の表の側の実の正本: --check → 0・4 行目に「rules.yaml」「1833 byte」・生成区間の要約値が (b) の値・行数 27。
//! 17. 規則の表の側のずれ: 生成区間の 1 byte を書き換えて --check → 1。
//! 18. 規則の表の側の印: begin を消す → 2。
//! 19. 規則の表の側の書き直し: ずれた写しに --write → 0・file 全体が元と byte 一致（人が書く行 thresholds・discipline と先頭の注釈も不変）。
//!
//! 便 76（docs/design/delivery-76.md §1 (c)(h)）: 命令は 5 本目の file index.yaml（入口の正本・生成区間は末尾）も順に見る（合格の標準出力は 5 行）。
//! f76_ 3. 入口の正本の側の実の正本: --check → 0・5 行目に「index.yaml」「860 byte」・要約値が (b) の値・行数 9・印の前は相談窓口の節・印の後は file の終わり。
//! f76_ 4. 入口の正本の側のずれ: 生成区間の 1 byte を書き換えて --check → 1・先の 4 本は触らない。
//! f76_ 5. 入口の正本の側の印: begin を消す → --check も --write も 2。
//! f76_ 6. 入口の正本の側の書き直し: ずれた写しに --write → 0・file 全体が元と byte 一致・もう 1 度で変わらない。
//!
//! 便 77（docs/design/delivery-77.md §1 (e)）: 命令は 6〜8 本目の file srs.yaml・vocabulary.yaml・intake.yaml（生成区間は末尾）も
//! 順に見る（合格の標準出力は 8 行）。
//! f77_ 1. --check → 0・8 行・6〜8 行目が 3 file で byte 数が anchor の byte 長と同じ。
//! f77_ 2. 3 file の生成区間が凍結 anchor と byte 一致・anchor の自己検査（要約値を測れなければ落とす）。
//! f77_ 3. 3 file それぞれの生成区間の 1 byte を書き換えて --check → 1。
//! f77_ 4. 3 file それぞれの begin を消す → 2。
//! f77_ 5. 3 file をずらした写しに --write → 0・3 file 全体が元と byte 一致・もう 1 度で 8 行とも変わらない。
//!
//! 便 78（docs/design/delivery-78.md §1 (d)）: 要件書の注 top_level_note から scope_m1 の誤った一文を落とした
//! （F77_REGIONS の srs.yaml を 1168 byte と新しい要約値に・便 86 で要件の行の枝を足した後の値）。
//! f78_ 1. --check → 0 ∧ 要件書の生成区間に「今の正本には無い」が無い ∧ 最上位に scope_m1 の節が在る。
//!
//! 便 85（docs/design/delivery-85.md §1 (d)）: 種別 deny の意味を下限の不足と固定の値との違いにも当たる字に直した
//! （RULES_REGION_* を 1833 byte と新しい要約値に）。
//! f85_ 1. --check → 0・8 行・rules.yaml の行が 1833 byte ∧ 生成区間が凍結 anchor と byte 一致 ∧ anchor の自己検査。
//! f85_ 2. deny の意味が 値域の外・上限の超過・下限の不足・固定の値との違い を持ち 超過なら落とす が無い ∧ R-13 / R-14 の値と種別は不変。
//!
//! 便 86（docs/design/delivery-86.md §1 (f)）: 要件書の生成区間の末尾に要件の行の欄の閉じた一覧を足した
//! （F77_REGIONS の srs.yaml を 29 行・1168 byte と新しい要約値に）。
//! f86_ 1. --check → 0・8 行・srs.yaml の行が 1168 byte ∧ 生成区間が凍結 anchor と byte 一致 ∧ anchor の自己検査。
//! f86_ 2. 正本の要件の行に現れる欄の集合が生成区間の requirement_row の 4 群 + verify の中に過不足なく収まる。
//!
//! 便 95（docs/design/delivery-95.md §1 (f)）: 命令は 9 本目の file graph.yaml（索引の欄の決まり・生成区間は末尾）も
//! 順に見る（合格の標準出力は 9 行）。
//! f95_ 1. graph.yaml の生成区間が 35 行・3607 byte（便 99 で 28 行・2106 byte から）で凍結 anchor と byte 一致・anchor の要約値。
//! f95_ 2. --check → 0・9 行・9 行目が graph.yaml。
//! f95_ 3. 生成区間の 1 byte を書き換えて --check → 1・理由に graph.yaml・--write で元の byte に戻る。
//! f95_ 4. 生成区間の node_kinds と edge_types が folio graph --print の出す種類と型を漏れなく覆う。
//!
//! 便 117（docs/design/delivery-117.md §1 (b)(d)）: 要件書の最上位の節の閉じた一覧に scope_m3 を足した（生成区間は導出・anchor は手で 1 行）。
//! F77_REGIONS の srs.yaml を wc と sha256sum で測り直した 30 行・1189 byte と要約値に・F86_SRS_* はその行を指す形に寄せた。

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use yaml_rust2::{Yaml, YamlLoader};

/// 便 48 (c) 凍結 anchor: ceiling.yaml の生成区間（設計判断の席が独立の実装で組んだ・tests/fixtures/schema/ceiling-region.txt と同じ byte）。
const CEILING_REGION_LINES: usize = 27;
const CEILING_REGION_BYTES: usize = 3176;
const CEILING_REGION_SHA256: &str =
    "1cc1401cc474f16b272e02cfa434e4fb5b00aead4b803b1d0adcb2fe9e4f5386";

/// 便 53 (b) 凍結 anchor: rules.yaml の生成区間（設計判断の席が独立の実装で組んだ・tests/fixtures/schema/rules-region.txt と同じ byte）。
const RULES_REGION_LINES: usize = 30;
const RULES_REGION_BYTES: usize = 2269;
const RULES_REGION_SHA256: &str =
    "d3f7f85d910a08c767cfe908219b1a89ceb906982a49dc2fb45313cbe9f6cf7f";

/// 便 76 (b) 凍結 anchor: index.yaml の生成区間（設計判断の席が独立に組んだ・tests/fixtures/schema/index-region.txt と同じ byte）。
const INDEX_REGION_LINES: usize = 9;
const INDEX_REGION_BYTES: usize = 860;
const INDEX_REGION_SHA256: &str =
    "01604d6003dde61fe970af89194215b5e0517d32610fd9cc0cc4554801f7f9ee";

/// 便 77 (b) 凍結 anchor 3 本（file 名・anchor・行数・byte 数・要約値）。設計判断の席が独立に組んだ。
const F77_REGIONS: [(&str, &str, usize, usize, &str); 3] = [
    (
        "srs.yaml",
        "tests/fixtures/schema/srs-region.txt",
        30,
        1189,
        "0a06c5630693cc4157376d034838d92c48635433d8948248b94b71023a45afc8",
    ),
    (
        "vocabulary.yaml",
        "tests/fixtures/schema/vocabulary-region.txt",
        3,
        238,
        "2746b140abdf5bfafa2b3b907b2bce91c9ee21e3af5488ba09420006f0170161",
    ),
    (
        "intake.yaml",
        "tests/fixtures/schema/intake-region.txt",
        3,
        262,
        "38fd3ff44e9aa6a398344d3385cb4281c77daf62d3839410e5da3814183aee8b",
    ),
];

/// 便 77 の 3 file の生成区間の変異（1 byte ずつ・F77_REGIONS と同じ順）。
const F77_DRIFTS: [(&str, &str); 3] = [
    ("\n    - glossary_pointer\n", "\n    - glossary_pointeR\n"),
    (
        "\n  top_level: [terms, field_terms, identifiers, schema]\n",
        "\n  top_level: [terms, field_terms, identifierS, schema]\n",
    ),
    (
        "\n  top_level: [meta, answers, targets, questions, sheet, schema]\n",
        "\n  top_level: [meta, answers, targets, questions, sheeT, schema]\n",
    ),
];

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

    fn ceiling_yaml(&self) -> PathBuf {
        self.dir().join("ceiling.yaml")
    }

    fn read_ceiling(&self) -> String {
        fs::read_to_string(self.ceiling_yaml()).unwrap()
    }

    fn rules_yaml(&self) -> PathBuf {
        self.dir().join("rules.yaml")
    }

    fn read_rules(&self) -> String {
        fs::read_to_string(self.rules_yaml()).unwrap()
    }

    fn index_yaml(&self) -> PathBuf {
        self.dir().join("index.yaml")
    }

    fn read_index(&self) -> String {
        fs::read_to_string(self.index_yaml()).unwrap()
    }

    /// 写しの index.yaml の字面の変異（1 か所だけ）。
    fn mutate_index(&self, from: &str, to: &str) {
        mutate_file(&self.index_yaml(), from, to);
    }

    /// 写しの ceiling.yaml の字面の変異（1 か所だけ）。
    fn mutate_ceiling(&self, from: &str, to: &str) {
        mutate_file(&self.ceiling_yaml(), from, to);
    }

    /// 写しの rules.yaml の字面の変異（1 か所だけ）。
    fn mutate_rules(&self, from: &str, to: &str) {
        mutate_file(&self.rules_yaml(), from, to);
    }

    fn schema(&self, flags: &[&str]) -> Output {
        folio(&["schema", "--dir"], &self.dir(), flags)
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

// ── 便 48: 天井の正本の側 ──

/// 天井の正本の側の生成区間の変異（1 byte・束の要約値の規則の名）。
const CEILING_DRIFT_FROM: &str = "\n    digest: sha256-files-1\n";
const CEILING_DRIFT_TO: &str = "\n    digest: sha256-files-2\n";

// ── 12. 天井の正本の側の実の正本 ──

#[test]
fn schema_check_matches_the_real_ceiling_file_and_its_frozen_digest() {
    let w = Work::new("ceiling-real");
    let out = w.schema(&["--check"]);
    assert_outcome(
        &out,
        0,
        &[
            "一致",
            "ceiling.yaml",
            &format!("{CEILING_REGION_BYTES} byte"),
        ],
    );
    let lines: Vec<String> = stdout(&out).lines().map(str::to_string).collect();
    assert_eq!(lines.len(), TARGETS, "{lines:?}");
    assert!(lines[0].contains("adr/schema.yaml"), "{lines:?}");
    assert!(lines[1].contains("design-note/schema.yaml"), "{lines:?}");
    assert!(lines[2].contains("ceiling.yaml"), "{lines:?}");
    assert!(
        lines[2].contains(&format!("{CEILING_REGION_BYTES} byte")),
        "{lines:?}"
    );
    let text = w.read_ceiling();
    let cur = region(&text);
    assert_eq!(cur.len(), CEILING_REGION_BYTES, "生成区間の byte 数");
    assert_eq!(cur.lines().count(), CEILING_REGION_LINES, "生成区間の行数");
    assert!(cur.starts_with("schema:\n"));
    match sha256_hex(cur.as_bytes()) {
        Ok(hex) => assert_eq!(hex, CEILING_REGION_SHA256, "sha256sum で測り直した要約値"),
        Err(why) => eprintln!("# まだ分からない: 要約値を測れない: {why}"),
    }
    // 生成区間は file の末尾（end の印の行で終わる）
    assert!(text.ends_with(&format!("\n{END}\n")), "{text}");
    // 検査は file を書かない
    assert_eq!(w.read_ceiling(), text);
}

// ── 13. 天井の正本の側のずれ ──

#[test]
fn schema_check_fails_on_one_byte_drift_inside_the_ceiling_region() {
    let w = Work::new("ceiling-drift");
    let adr = w.read();
    let note = w.read_note();
    w.mutate_ceiling(CEILING_DRIFT_FROM, CEILING_DRIFT_TO);
    assert_outcome(
        &w.schema(&["--check"]),
        1,
        &[
            "ceiling.yaml",
            "生成区間",
            "≠ 導出",
            &format!("{CEILING_REGION_BYTES} byte"),
        ],
    );
    // 先の 2 本（合格）の行は、3 本目で落ちたときは出さない・file も触らない
    assert_eq!(w.read(), adr);
    assert_eq!(w.read_note(), note);
}

// ── 14. 天井の正本の側の印 ──

#[test]
fn schema_check_is_unknown_without_the_ceiling_begin_marker() {
    let w = Work::new("ceiling-no-begin");
    w.mutate_ceiling(&format!("{BEGIN}\n"), "");
    assert_outcome(
        &w.schema(&["--check"]),
        2,
        &["ceiling.yaml: 印が 1 対でない"],
    );
    assert_outcome(
        &w.schema(&["--write"]),
        2,
        &["ceiling.yaml: 印が 1 対でない"],
    );
}

// ── 15. 天井の正本の側の書き直し ──

#[test]
fn schema_write_restores_the_ceiling_region_and_is_idempotent() {
    let w = Work::new("ceiling-write");
    let original = w.read_ceiling();
    let adr = w.read();
    let note = w.read_note();
    w.mutate_ceiling(CEILING_DRIFT_FROM, CEILING_DRIFT_TO);
    assert_ne!(w.read_ceiling(), original);
    assert_outcome(
        &w.schema(&["--write"]),
        0,
        &[
            "変わらない",
            "adr/schema.yaml",
            "design-note/schema.yaml",
            "書いた",
            "ceiling.yaml",
            &format!("{CEILING_REGION_BYTES} byte"),
        ],
    );
    assert_eq!(
        w.read_ceiling(),
        original,
        "file 全体が元と byte 一致（人が書く節 meta・weights・documents・viewpoints も不変）"
    );
    assert_eq!(w.read(), adr, "判断の記録の側は触らない");
    assert_eq!(w.read_note(), note, "設計ノートの側は触らない");
    assert_outcome(
        &w.schema(&["--write"]),
        0,
        &["変わらない", &format!("{CEILING_REGION_BYTES} byte")],
    );
    assert_eq!(w.read_ceiling(), original);
    assert_outcome(&w.schema(&["--check"]), 0, &["一致"]);
}

// ── 便 53: 規則の表の側 ──

/// 規則の表の側の生成区間の変異（1 byte・値域 stage の 2 つ目の値）。
const RULES_DRIFT_FROM: &str = "\n    stage: [in-loop, post]\n";
const RULES_DRIFT_TO: &str = "\n    stage: [in-loop, past]\n";

// ── 16. 規則の表の側の実の正本 ──

#[test]
fn schema_check_matches_the_real_rules_file_and_its_frozen_digest() {
    let w = Work::new("rules-real");
    let out = w.schema(&["--check"]);
    assert_outcome(
        &out,
        0,
        &["一致", "rules.yaml", &format!("{RULES_REGION_BYTES} byte")],
    );
    let lines: Vec<String> = stdout(&out).lines().map(str::to_string).collect();
    assert_eq!(lines.len(), TARGETS, "{lines:?}");
    assert!(lines[0].contains("adr/schema.yaml"), "{lines:?}");
    assert!(lines[1].contains("design-note/schema.yaml"), "{lines:?}");
    assert!(lines[2].contains("ceiling.yaml"), "{lines:?}");
    assert!(lines[3].contains("rules.yaml"), "{lines:?}");
    assert!(
        lines[3].contains(&format!("{RULES_REGION_BYTES} byte")),
        "{lines:?}"
    );
    let text = w.read_rules();
    let cur = region(&text);
    assert_eq!(cur.len(), RULES_REGION_BYTES, "生成区間の byte 数");
    assert_eq!(cur.lines().count(), RULES_REGION_LINES, "生成区間の行数");
    assert!(cur.starts_with("schema:\n"));
    match sha256_hex(cur.as_bytes()) {
        Ok(hex) => assert_eq!(hex, RULES_REGION_SHA256, "sha256sum で測り直した要約値"),
        Err(why) => eprintln!("# まだ分からない: 要約値を測れない: {why}"),
    }
    // 生成区間は file の先頭の注釈の次（begin の前は注釈と空行だけ・人が書く行は end の後）
    let (head, _) = text.split_once(BEGIN).unwrap();
    assert!(
        head.lines().all(|l| l.is_empty() || l.starts_with('#')),
        "{head}"
    );
    let (_, tail) = text.split_once(&format!("\n{END}\n")).unwrap();
    assert!(tail.contains("\nthresholds:\n"), "{tail}");
    assert!(tail.contains("\ndiscipline:\n"), "{tail}");
    // 検査は file を書かない
    assert_eq!(w.read_rules(), text);
}

// ── 17. 規則の表の側のずれ ──

#[test]
fn schema_check_fails_on_one_byte_drift_inside_the_rules_region() {
    let w = Work::new("rules-drift");
    let adr = w.read();
    let note = w.read_note();
    let ceiling = w.read_ceiling();
    w.mutate_rules(RULES_DRIFT_FROM, RULES_DRIFT_TO);
    assert_outcome(
        &w.schema(&["--check"]),
        1,
        &[
            "rules.yaml",
            "生成区間",
            "≠ 導出",
            &format!("{RULES_REGION_BYTES} byte"),
        ],
    );
    // 先の 3 本（合格）の行は、4 本目で落ちたときは出さない・file も触らない
    assert_eq!(w.read(), adr);
    assert_eq!(w.read_note(), note);
    assert_eq!(w.read_ceiling(), ceiling);
}

// ── 18. 規則の表の側の印 ──

#[test]
fn schema_check_is_unknown_without_the_rules_begin_marker() {
    let w = Work::new("rules-no-begin");
    w.mutate_rules(&format!("{BEGIN}\n"), "");
    assert_outcome(&w.schema(&["--check"]), 2, &["rules.yaml: 印が 1 対でない"]);
    assert_outcome(&w.schema(&["--write"]), 2, &["rules.yaml: 印が 1 対でない"]);
}

// ── 19. 規則の表の側の書き直し ──

#[test]
fn schema_write_restores_the_rules_region_and_is_idempotent() {
    let w = Work::new("rules-write");
    let original = w.read_rules();
    let adr = w.read();
    let note = w.read_note();
    let ceiling = w.read_ceiling();
    w.mutate_rules(RULES_DRIFT_FROM, RULES_DRIFT_TO);
    assert_ne!(w.read_rules(), original);
    assert_outcome(
        &w.schema(&["--write"]),
        0,
        &[
            "変わらない",
            "adr/schema.yaml",
            "design-note/schema.yaml",
            "ceiling.yaml",
            "書いた",
            "rules.yaml",
            &format!("{RULES_REGION_BYTES} byte"),
        ],
    );
    assert_eq!(
        w.read_rules(),
        original,
        "file 全体が元と byte 一致（人が書く行 thresholds・discipline と先頭の注釈も不変）"
    );
    assert_eq!(w.read(), adr, "判断の記録の側は触らない");
    assert_eq!(w.read_note(), note, "設計ノートの側は触らない");
    assert_eq!(w.read_ceiling(), ceiling, "天井の正本の側は触らない");
    assert_outcome(
        &w.schema(&["--write"]),
        0,
        &["変わらない", &format!("{RULES_REGION_BYTES} byte")],
    );
    assert_eq!(w.read_rules(), original);
    assert_outcome(&w.schema(&["--check"]), 0, &["一致"]);
}

// ── 便 76: 入口の正本の側 ──

/// 入口の正本の側の生成区間の変異（1 byte・凡例の 1 つ目の id）。
const INDEX_DRIFT_FROM: &str = "\n    legend: [readable, absent, binds, inside]\n";
const INDEX_DRIFT_TO: &str = "\n    legend: [readablE, absent, binds, inside]\n";

// ── f76_ 3. 入口の正本の側の実の正本 ──

#[test]
fn f76_schema_check_matches_the_real_index_file_and_its_frozen_digest() {
    let w = Work::new("index-real");
    let out = w.schema(&["--check"]);
    assert_outcome(
        &out,
        0,
        &["一致", "index.yaml", &format!("{INDEX_REGION_BYTES} byte")],
    );
    let lines: Vec<String> = stdout(&out).lines().map(str::to_string).collect();
    assert_eq!(lines.len(), TARGETS, "{lines:?}");
    assert!(lines[3].contains("rules.yaml"), "{lines:?}");
    assert!(lines[4].contains("index.yaml"), "{lines:?}");
    assert!(
        lines[4].contains(&format!("{INDEX_REGION_BYTES} byte")),
        "{lines:?}"
    );
    let text = w.read_index();
    let cur = region(&text);
    assert_eq!(cur.len(), INDEX_REGION_BYTES, "生成区間の byte 数");
    assert_eq!(cur.lines().count(), INDEX_REGION_LINES, "生成区間の行数");
    assert!(cur.starts_with("schema:\n"));
    match sha256_hex(cur.as_bytes()) {
        Ok(hex) => assert_eq!(hex, INDEX_REGION_SHA256, "sha256sum で測り直した要約値"),
        Err(why) => eprintln!("# まだ分からない: 要約値を測れない: {why}"),
    }
    // 印の前に相談窓口の節が在り、印の後は file の終わり
    let (head, _) = text.split_once(BEGIN).unwrap();
    assert!(head.contains("\nintake:\n"), "{head}");
    assert!(text.ends_with(&format!("\n{END}\n")), "{text}");
    // 検査は file を書かない
    assert_eq!(w.read_index(), text);
}

// ── f76_ 4. 入口の正本の側のずれ ──

#[test]
fn f76_schema_check_fails_on_one_byte_drift_inside_the_index_region() {
    let w = Work::new("index-drift");
    let adr = w.read();
    let note = w.read_note();
    let ceiling = w.read_ceiling();
    let rules = w.read_rules();
    w.mutate_index(INDEX_DRIFT_FROM, INDEX_DRIFT_TO);
    assert_outcome(
        &w.schema(&["--check"]),
        1,
        &[
            "index.yaml",
            "生成区間",
            "≠ 導出",
            &format!("{INDEX_REGION_BYTES} byte"),
        ],
    );
    // 先の 4 本（合格）の行は、5 本目で落ちたときは出さない・file も触らない
    assert_eq!(w.read(), adr);
    assert_eq!(w.read_note(), note);
    assert_eq!(w.read_ceiling(), ceiling);
    assert_eq!(w.read_rules(), rules);
}

// ── f76_ 5. 入口の正本の側の印 ──

#[test]
fn f76_schema_check_is_unknown_without_the_index_begin_marker() {
    let w = Work::new("index-no-begin");
    w.mutate_index(&format!("{BEGIN}\n"), "");
    assert_outcome(&w.schema(&["--check"]), 2, &["index.yaml: 印が 1 対でない"]);
    assert_outcome(&w.schema(&["--write"]), 2, &["index.yaml: 印が 1 対でない"]);
}

// ── f76_ 6. 入口の正本の側の書き直し ──

#[test]
fn f76_schema_write_restores_the_index_region_and_is_idempotent() {
    let w = Work::new("index-write");
    let original = w.read_index();
    let adr = w.read();
    let note = w.read_note();
    let ceiling = w.read_ceiling();
    let rules = w.read_rules();
    w.mutate_index(INDEX_DRIFT_FROM, INDEX_DRIFT_TO);
    assert_ne!(w.read_index(), original);
    assert_outcome(
        &w.schema(&["--write"]),
        0,
        &[
            "変わらない",
            "rules.yaml",
            "書いた",
            "index.yaml",
            &format!("{INDEX_REGION_BYTES} byte"),
        ],
    );
    assert_eq!(
        w.read_index(),
        original,
        "file 全体が元と byte 一致（人が書く節 meta・audience・shelf・lanes・intake と承認欄と注釈も不変）"
    );
    assert_eq!(w.read(), adr, "判断の記録の側は触らない");
    assert_eq!(w.read_note(), note, "設計ノートの側は触らない");
    assert_eq!(w.read_ceiling(), ceiling, "天井の正本の側は触らない");
    assert_eq!(w.read_rules(), rules, "規則の表の側は触らない");
    assert_outcome(
        &w.schema(&["--write"]),
        0,
        &["変わらない", &format!("{INDEX_REGION_BYTES} byte")],
    );
    assert_eq!(w.read_index(), original);
    assert_outcome(&w.schema(&["--check"]), 0, &["一致"]);
}

// ── 便 77: 要件書・語彙・相談窓口の側 ──

impl Work {
    fn read_file(&self, file: &str) -> String {
        fs::read_to_string(self.dir().join(file)).unwrap()
    }
}

// ── f77_ 1. 8 行の一致と byte 数 ──

#[test]
fn f77_check_covers_the_three_files() {
    let w = Work::new("f77-check");
    let out = w.schema(&["--check"]);
    assert_outcome(&out, 0, &["一致"]);
    let lines: Vec<String> = stdout(&out).lines().map(str::to_string).collect();
    assert_eq!(lines.len(), TARGETS, "{lines:?}");
    assert!(lines[4].contains("index.yaml"), "{lines:?}");
    for (i, (file, anchor, _, bytes, _)) in F77_REGIONS.iter().enumerate() {
        let line = &lines[5 + i];
        assert!(line.contains(&format!("（{file}・")), "{line}");
        assert!(line.contains(&format!("{bytes} byte")), "{line}");
        let len = fs::read(repo_root().join(anchor)).unwrap().len();
        assert!(line.contains(&format!("・{len} byte）")), "anchor の byte 長 {len}: {line}");
    }
}

// ── f77_ 2. 凍結 anchor との byte 一致と anchor の自己検査 ──

#[test]
fn f77_regions_match_the_frozen_anchors() {
    let w = Work::new("f77-anchors");
    for (file, anchor, lines, bytes, sha) in F77_REGIONS {
        let text = w.read_file(file);
        let anchor_text = fs::read_to_string(repo_root().join(anchor)).unwrap();
        assert_eq!(region(&text), anchor_text, "{file} の生成区間が {anchor} と byte 一致");
        assert_eq!(anchor_text.lines().count(), lines, "{anchor} の行数");
        assert_eq!(anchor_text.len(), bytes, "{anchor} の byte 数");
        let hex = sha256_hex(anchor_text.as_bytes())
            .unwrap_or_else(|why| panic!("要約値を測れない（素通りにしない）: {why}"));
        assert_eq!(hex, sha, "sha256sum で測った {anchor} の要約値");
        // 印の後は file の終わり（生成区間は末尾）
        assert!(text.ends_with(&format!("\n{END}\n")), "{file}");
    }
}

// ── f77_ 3. 3 file のずれ ──

#[test]
fn f77_drift_in_each_region_fails() {
    for ((file, _, _, bytes, _), (from, to)) in F77_REGIONS.iter().zip(F77_DRIFTS) {
        let w = Work::new(&format!("f77-drift-{file}"));
        mutate_file(&w.dir().join(file), from, to);
        assert_outcome(
            &w.schema(&["--check"]),
            1,
            &[&format!("{file}: 生成区間"), "≠ 導出", &format!("{bytes} byte")],
        );
    }
}

// ── f77_ 4. 3 file の印 ──

#[test]
fn f77_missing_marker_in_each_file_is_unknown() {
    for (file, ..) in F77_REGIONS {
        let w = Work::new(&format!("f77-no-begin-{file}"));
        mutate_file(&w.dir().join(file), &format!("{BEGIN}\n"), "");
        assert_outcome(
            &w.schema(&["--check"]),
            2,
            &[&format!("{file}: 印が 1 対でない")],
        );
    }
}

// ── f77_ 5. 書き直しと冪等 ──

#[test]
fn f77_write_restores_the_three_regions() {
    let w = Work::new("f77-write");
    let originals: Vec<String> = F77_REGIONS.iter().map(|r| w.read_file(r.0)).collect();
    for ((file, ..), (from, to)) in F77_REGIONS.iter().zip(F77_DRIFTS) {
        mutate_file(&w.dir().join(file), from, to);
    }
    let out = w.schema(&["--write"]);
    assert_outcome(&out, 0, &["書いた"]);
    for ((file, ..), original) in F77_REGIONS.iter().zip(&originals) {
        assert!(
            stdout(&out).contains(&format!("書いた（{file}・")),
            "{}",
            stdout(&out)
        );
        assert_eq!(
            &w.read_file(file),
            original,
            "{file} 全体が元と byte 一致（人が書く節・meta・承認欄・注釈も不変）"
        );
    }
    let again = w.schema(&["--write"]);
    assert_outcome(&again, 0, &[]);
    assert!(
        stdout(&again).lines().all(|l| l.contains("変わらない")),
        "{}",
        stdout(&again)
    );
    for ((file, ..), original) in F77_REGIONS.iter().zip(&originals) {
        assert_eq!(&w.read_file(file), original, "{file}");
    }
}

// ── 便 78: 要件書の生成区間の注は scope_m1 を「無い」と言わない ──

// ── f78_ 1. --check → 0 ∧ 注に「今の正本には無い」が無い ∧ 最上位に scope_m1 の節が在る ──

#[test]
fn f78_srs_note_does_not_claim_scope_m1_is_absent() {
    let w = Work::new("f78-srs-note");
    assert_outcome(&w.schema(&["--check"]), 0, &["一致", "srs.yaml"]);
    let text = w.read_file("srs.yaml");
    assert!(!region(&text).contains("今の正本には無い"), "{}", region(&text));
    assert!(
        text.lines().any(|l| l.starts_with("scope_m1:")),
        "要件書の最上位に scope_m1 の節が無い"
    );
}

// ── 便 85: 種別 deny の意味は上限・下限・固定の値のどれにも当たる ──

/// 便 85 (b) 凍結 anchor の置き場。
const F85_RULES_ANCHOR: &str = "tests/fixtures/schema/rules-region.txt";

// ── f85_ 1. 生成区間が新しい凍結 anchor と byte 一致・anchor の自己検査 ──

#[test]
fn f85_rules_region_matches_the_new_anchor() {
    let w = Work::new("f85-anchor");
    let out = w.schema(&["--check"]);
    assert_outcome(&out, 0, &["一致", "rules.yaml・2269 byte"]);
    let anchor_text = fs::read_to_string(repo_root().join(F85_RULES_ANCHOR)).unwrap();
    let text = w.read_rules();
    assert_eq!(region(&text), anchor_text, "rules.yaml の生成区間が anchor と byte 一致");
    assert_eq!(anchor_text.lines().count(), 30, "anchor の行数");
    assert_eq!(anchor_text.len(), 2269, "anchor の byte 数");
    let hex = sha256_hex(anchor_text.as_bytes())
        .unwrap_or_else(|why| panic!("要約値を測れない（素通りにしない）: {why}"));
    assert_eq!(
        hex, "d3f7f85d910a08c767cfe908219b1a89ceb906982a49dc2fb45313cbe9f6cf7f",
        "sha256sum で測った anchor の要約値"
    );
}

// ── f85_ 2. deny の意味の字・R-13 / R-14 の値と種別は不変 ──

#[test]
fn f85_deny_meaning_names_the_lower_bound_and_the_fixed_value() {
    let w = Work::new("f85-deny");
    let text = w.read_rules();
    let cur = region(&text);
    let deny = cur
        .lines()
        .find(|l| l.starts_with("    deny: "))
        .expect("kind_meaning の deny の行が無い");
    for word in ["値域の外", "上限の超過", "下限の不足", "固定の値との違い"] {
        assert!(deny.contains(word), "「{word}」が無い: {deny}");
    }
    assert_eq!(cur.matches("超過なら落とす").count(), 0, "{cur}");
    let row = |id: &str| {
        text.lines()
            .find(|l| l.starts_with(&format!("  - {{id: {id}, ")))
            .unwrap_or_else(|| panic!("行 {id} が無い"))
            .to_string()
    };
    let r13 = row("R-13");
    assert!(r13.contains(", value: \"1 本以上\", kind: deny, "), "{r13}");
    let r14 = row("R-14");
    assert!(
        r14.contains(", value: 最上段（showcase）固定, kind: deny, "),
        "{r14}"
    );
}

// ── 便 86: 要件の行の欄の閉じた一覧を要件書の生成区間へ導出する ──

/// 便 86 (c) 凍結 anchor の置き場と自己検査の値（設計判断の席が独立に組んだ）。
/// 便 117（docs/design/delivery-117.md §1 (b)(d)）: 値は数を書き直さず F77_REGIONS の srs.yaml の行（先頭の組）を指す
/// （anchor の値を pin する場所を 1 か所に寄せた）。
const F86_SRS_ANCHOR: &str = "tests/fixtures/schema/srs-region.txt";
const F86_SRS_LINES: usize = F77_REGIONS[0].2;
const F86_SRS_BYTES: usize = F77_REGIONS[0].3;
const F86_SRS_SHA256: &str = F77_REGIONS[0].4;

// ── f86_ 1. 生成区間が新しい凍結 anchor と byte 一致・anchor の自己検査 ──

#[test]
fn f86_srs_region_matches_the_new_anchor() {
    let w = Work::new("f86-anchor");
    let out = w.schema(&["--check"]);
    assert_outcome(&out, 0, &["一致", &format!("srs.yaml・{F86_SRS_BYTES} byte")]);
    let anchor_text = fs::read_to_string(repo_root().join(F86_SRS_ANCHOR)).unwrap();
    let text = w.read_file("srs.yaml");
    assert_eq!(region(&text), anchor_text, "srs.yaml の生成区間が anchor と byte 一致");
    assert_eq!(anchor_text.lines().count(), F86_SRS_LINES, "anchor の行数");
    assert_eq!(anchor_text.len(), F86_SRS_BYTES, "anchor の byte 数");
    let hex = sha256_hex(anchor_text.as_bytes())
        .unwrap_or_else(|why| panic!("要約値を測れない（素通りにしない）: {why}"));
    assert_eq!(hex, F86_SRS_SHA256, "sha256sum で測った anchor の要約値");
}

// ── f86_ 2. 正本の要件の行の欄の集合が生成区間の群に過不足なく収まる ──

/// 表の鍵の字（字でない鍵は落とす）。
fn keys(node: &Yaml) -> Vec<String> {
    node.as_hash()
        .map(|h| h.keys().filter_map(|k| k.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

/// 字の一覧（一覧でない・字でない項は歯を落とす）。
fn strs(node: &Yaml, at: &str) -> Vec<String> {
    node.as_vec()
        .unwrap_or_else(|| panic!("{at} が一覧でない"))
        .iter()
        .map(|v| v.as_str().unwrap_or_else(|| panic!("{at} の項が字でない")).to_string())
        .collect()
}

#[test]
fn f86_region_lists_every_group_of_the_row() {
    let w = Work::new("f86-groups");
    let text = w.read_file("srs.yaml");
    let reg = YamlLoader::load_from_str(region(&text)).unwrap().remove(0);
    let row = &reg["schema"]["requirement_row"];
    let text_keys = strs(&row["required_text"], "required_text");
    let list_keys = strs(&row["required_list"], "required_list");
    let optional = strs(&row["optional"], "optional");
    let verify_text = strs(&row["verify"]["required_text"], "verify.required_text");
    let verify_list = strs(&row["verify"]["required_list"], "verify.required_list");
    assert_eq!(
        [text_keys.len(), list_keys.len(), optional.len(), verify_text.len(), verify_list.len()],
        [7, 3, 4, 2, 1],
        "{row:?}"
    );
    assert_eq!(keys(row), ["required_text", "required_list", "optional", "verify"]);
    assert_eq!(keys(&row["verify"]), ["required_text", "required_list"]);

    // 生成区間が言う行の欄（群どうしは重ならない）
    let mut declared: Vec<String> = [&text_keys, &list_keys, &optional].into_iter().flatten().cloned().collect();
    declared.push("verify".to_string());
    let mut inner: Vec<String> = verify_text.iter().chain(&verify_list).cloned().collect();
    let (n, m) = (declared.len(), inner.len());
    declared.sort();
    declared.dedup();
    inner.sort();
    inner.dedup();
    assert_eq!((declared.len(), inner.len()), (n, m), "群が重なる");

    // 正本の要件の行（requirements と nonfunctional の全行）に実際に現れる欄の和集合
    let doc = YamlLoader::load_from_str(&text).unwrap().remove(0);
    let mut seen: Vec<String> = Vec::new();
    let mut seen_inner: Vec<String> = Vec::new();
    let mut count = 0;
    for section in ["requirements", "nonfunctional"] {
        for item in doc[section].as_vec().unwrap_or_else(|| panic!("{section} が一覧でない")) {
            count += 1;
            seen.extend(keys(item));
            seen_inner.extend(keys(&item["verify"]));
        }
    }
    assert!(count > 0, "要件の行が無い");
    seen.sort();
    seen.dedup();
    seen_inner.sort();
    seen_inner.dedup();
    assert_eq!(seen, declared, "正本の要件の行の欄の集合 = 生成区間の 4 群");
    assert_eq!(seen_inner, inner, "正本の verify の中の欄の集合 = 生成区間の verify の 2 群");
}

// ── 便 89: 切り出しの後の行数と、移した定義の在り処 ──

/// 器の行数の式: 空行を含む全行を数え、字数（Unicode の字の数）が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す。
fn f89_measure(text: &str) -> usize {
    text.lines()
        .map(|line| line.chars().count().div_ceil(120).max(1))
        .sum()
}

/// 移した定義の頭 8 本（行の先頭に在ることを見る）。
const F89_MOVED_HEADS: [&str; 8] = [
    "fn schema_check_matches_the_real_ceiling_file_and_its_frozen_digest(",
    "fn schema_check_matches_the_real_rules_file_and_its_frozen_digest(",
    "fn f76_schema_check_matches_the_real_index_file_and_its_frozen_digest(",
    "fn f77_regions_match_the_frozen_anchors(",
    "fn f86_srs_region_matches_the_new_anchor(",
    "const CEILING_REGION_BYTES:",
    "const RULES_REGION_SHA256:",
    "const F77_REGIONS:",
];

#[test]
fn f89_schema_teeth_are_split_and_under_the_cap() {
    let tests = repo_root().join("crates/folio/tests");
    let old = fs::read_to_string(tests.join("schema.rs")).expect("tests/schema.rs を読めない");
    let new =
        fs::read_to_string(tests.join("schema_docs.rs")).expect("tests/schema_docs.rs を読めない");
    let old_size = f89_measure(&old);
    let new_size = f89_measure(&new);
    assert!(old_size <= 700, "tests/schema.rs が器の式で {old_size} 行（700 以下のはず）");
    assert!(new_size <= 1200, "tests/schema_docs.rs が器の式で {new_size} 行（1200 以下のはず）");
    for head in F89_MOVED_HEADS {
        assert!(
            new.lines().any(|line| line.starts_with(head)),
            "tests/schema_docs.rs に「{head}」が無い"
        );
        assert!(
            !old.lines().any(|line| line.starts_with(head)),
            "tests/schema.rs に「{head}」が残っている"
        );
    }
}

// ── 便 95: 索引の欄の決まりの正本 graph.yaml ──

/// 便 95 (c) 凍結 anchor の置き場と自己検査の値（設計判断の席が独立の実装で組んだ）。
const F95_GRAPH_ANCHOR: &str = "tests/fixtures/schema/graph-region.txt";
/// 便 99 で node の digest と edge_fields・edge_fields_note・digest_note を足した値（docs/design/delivery-99.md §1 (f)）。
const F95_GRAPH_LINES: usize = 35;
const F95_GRAPH_BYTES: usize = 3607;
const F95_GRAPH_SHA256: &str = "7e2515a7727d84e4df0449d54577f842778f4e244701622986f3098c51eb947b";

/// 生成区間の変異（node_kinds の行の 判断の記録 の末尾の 1 字）。
const F95_DRIFT_FROM: &str = ", 判断の記録]\n";
const F95_DRIFT_TO: &str = ", 判断の記禄]\n";

// ── f95_ 1. 生成区間が凍結 anchor と byte 一致・anchor の自己検査 ──

#[test]
fn f95_the_graph_schema_region_matches_the_anchor() {
    let w = Work::new("f95-anchor");
    let text = w.read_file("graph.yaml");
    let anchor_text = fs::read_to_string(repo_root().join(F95_GRAPH_ANCHOR)).unwrap();
    assert_eq!(region(&text), anchor_text, "graph.yaml の生成区間が anchor と byte 一致");
    assert_eq!(anchor_text.lines().count(), F95_GRAPH_LINES, "anchor の行数");
    assert_eq!(anchor_text.len(), F95_GRAPH_BYTES, "anchor の byte 数");
    let hex = sha256_hex(anchor_text.as_bytes())
        .unwrap_or_else(|why| panic!("要約値を測れない（素通りにしない）: {why}"));
    assert_eq!(hex, F95_GRAPH_SHA256, "sha256sum で測った anchor の要約値");
    // 印の後は file の終わり（生成区間は末尾）
    assert!(text.ends_with(&format!("\n{END}\n")), "graph.yaml");
}

// ── f95_ 2. --check → 0・9 行・9 行目が graph.yaml ──

#[test]
fn f95_the_command_now_sees_nine_files() {
    let w = Work::new("f95-check");
    let out = w.schema(&["--check"]);
    assert_outcome(&out, 0, &["一致"]);
    let lines: Vec<String> = stdout(&out).lines().map(str::to_string).collect();
    assert_eq!(lines.len(), 9, "{lines:?}");
    assert_eq!(
        lines[8],
        format!("folio schema: 一致（graph.yaml・{F95_GRAPH_BYTES} byte）"),
        "{lines:?}"
    );
}

// ── f95_ 3. ずれ → 1・--write で元の byte に戻る ──

#[test]
fn f95_a_drift_in_the_graph_region_fails() {
    let w = Work::new("f95-drift");
    let original = w.read_file("graph.yaml");
    mutate_file(&w.dir().join("graph.yaml"), F95_DRIFT_FROM, F95_DRIFT_TO);
    assert_outcome(
        &w.schema(&["--check"]),
        1,
        &["graph.yaml: 生成区間", "≠ 導出", &format!("{F95_GRAPH_BYTES} byte")],
    );
    let out = w.schema(&["--write"]);
    assert_outcome(&out, 0, &["書いた（graph.yaml・"]);
    assert_eq!(w.read_file("graph.yaml"), original, "graph.yaml 全体が元と byte 一致");
    assert_outcome(&w.schema(&["--check"]), 0, &["一致"]);
}

// ── f95_ 4. 閉じた一覧 2 本が索引の出す種類と型を覆う ──

#[test]
fn f95_the_closed_lists_are_the_same_as_the_index() {
    let w = Work::new("f95-cover");
    let text = w.read_file("graph.yaml");
    let reg = YamlLoader::load_from_str(region(&text)).unwrap().remove(0);
    let kinds = strs(&reg["schema"]["node_kinds"], "node_kinds");
    let types = strs(&reg["schema"]["edge_types"], "edge_types");
    assert_eq!((kinds.len(), types.len()), (11, 17), "{reg:?}");

    let out = folio(&["graph", "--print", "--dir"], &w.dir(), &[]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let printed = stdout(&out);
    let mut part = 0;
    let (mut seen_kinds, mut seen_types) = (Vec::new(), Vec::new());
    for line in printed.lines() {
        if line.starts_with('#') {
            part += 1;
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        match part {
            1 => seen_kinds.push(cols[1].to_string()),
            2 => seen_types.push(cols[2].to_string()),
            _ => panic!("表の外の行: {line}"),
        }
    }
    assert!(!seen_kinds.is_empty() && !seen_types.is_empty(), "{printed}");
    for kind in &seen_kinds {
        assert!(kinds.contains(kind), "種類 {kind} が node_kinds の外");
    }
    for ty in &seen_types {
        assert!(types.contains(ty), "型 {ty} が edge_types の外");
    }
}
