//! `folio graph --print --summary`（便 180・docs/design/delivery-180.md §1 (c)・要件 FR14 第 1.52 版）の歯。folio は
//! 実行 file の crate なので命令を撃つ。
//! 1. 実の正本で、節点の表の各行に 1 行ずつ同じ順・同じ id・種類・file・題で出て、line の行にその id が書かれている。
//! 2. 凍結した土台の写しの 8 節点の行が、歯の側に手で書いた字（凍結 anchor・P-10.1）と一致する（受入基準の技術の要約は
//!    題の全文）。
//! 3. タブ・改行・引用符・逆斜線・制御の字と `|` の塊（複数行・末尾の改行）を持つ欄が JSON の escape で 1 行に収まり、
//!    技術の要約の欄の順（shall・text・what・decision・空の値は飛ばす）と条の「最初の規範文」と受入基準の題の全文
//!    （畳まず切らない）が守られる。期待の字はどれも歯の側の手書き（凍結 anchor・P-10.1 / P-10.2）。
//! 4. 組めない置き場では 1 行も出さずに まだ分からない（終了コード 2）。--summary は --print と一緒のときだけ。
//! 5. --summary の口が在っても、--summary の無い --print と --digest の出力は凍結 anchor のまま（土台の写し）。

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const EDGES_HEAD: &str = "# 辺（1 行 = 端 / 端 / 型・タブ区切り）";
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

/// 凍結した土台 tests/fixtures/floor_base/design-intent の写しの一時 dir（歯の終わりに消す）。
struct Work {
    root: PathBuf,
}

impl Work {
    fn base(case: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-graph-summary-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_tree(&repo_root().join(FLOOR_BASE), &root.join("design-intent"));
        Work { root }
    }

    /// 写しの file の中の `from`（ちょうど 1 か所）を `to` に替える。
    fn replace(&self, file: &str, from: &str, to: &str) {
        let path = self.dir().join(file);
        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(text.matches(from).count(), 1, "{file}: 「{from}」が 1 か所でない");
        fs::write(&path, text.replacen(from, to, 1)).unwrap();
    }

    fn dir(&self) -> PathBuf {
        self.root.join("design-intent")
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn graph(dir: &Path, flags: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("graph")
        .args(flags)
        .arg("--dir")
        .arg(dir)
        .output()
        .expect("folio を起動できない")
}

fn passed(out: Output) -> String {
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).expect("出力が UTF-8 でない")
}

fn summary(dir: &Path) -> String {
    passed(graph(dir, &["--print", "--summary"]))
}

/// 歯の側の JSON の字の書き方（引用符・逆斜線・制御の字だけを逃がす・非 ASCII はそのまま）。
fn quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// 出力の中の id の行（ちょうど 1 行）。
fn line_of<'a>(text: &'a str, id: &str) -> &'a str {
    let head = format!("{{\"id\":{},", quoted(id));
    let found: Vec<&str> = text.lines().filter(|l| l.starts_with(&head)).collect();
    assert_eq!(found.len(), 1, "{id} の行の数");
    found[0]
}

/// 要約値（sha256）は命令 `sha256sum` を子の処理で撃って測る（歯は crate の中を読めない・tests/graph.rs と同じ形）。
fn sha256_hex(bytes: &[u8]) -> String {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("sha256sum を起動できない");
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    let out = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string()
}

#[test]
fn f180_each_node_of_the_print_has_one_line() {
    let dir = repo_root().join("design-intent");
    let print = passed(graph(&dir, &["--print"]));
    let text = summary(&dir);
    assert!(text.ends_with('\n') && !text.contains(['\t', '\r']), "JSON Lines でない");
    let rows: Vec<&str> = print.lines().skip(1).take_while(|l| *l != EDGES_HEAD).collect();
    let lines: Vec<&str> = text.lines().collect();
    assert!(!rows.is_empty());
    assert_eq!(lines.len(), rows.len(), "節点の表の行の数と 1 行ずつでない");
    for (row, line) in rows.iter().zip(&lines) {
        let cols: Vec<&str> = row.split('\t').collect();
        let (id, kind, file, title) = (cols[0], cols[1], cols[2], cols[4]);
        let head = format!("{{\"id\":{},\"kind\":{},\"file\":{},\"line\":", quoted(id), quoted(kind), quoted(file));
        let rest = line.strip_prefix(&head).unwrap_or_else(|| panic!("表の {row} と頭が違う: {line}"));
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        let n: usize = digits.parse().unwrap_or_else(|_| panic!("line が数でない: {line}"));
        let next = format!(",\"title\":{},\"plain\":", quoted(title));
        assert!(rest[digits.len()..].starts_with(&next), "題が表と違う: {line}");
        assert!(line.contains(",\"eng\":") && line.ends_with('}'), "{line}");
        let source = fs::read_to_string(dir.join(file)).unwrap();
        let at = source.lines().nth(n.checked_sub(1).expect("line が 0")).expect("line が file の外");
        // 設計ノートの行（便 185）の id は「文書 id#行 id」で、書かれているのは行 id
        let key = format!("id: {}", id.rsplit('#').next().unwrap_or(id));
        let written = at.match_indices(&key).any(|(i, _)| {
            matches!(at[i + key.len()..].chars().next(), None | Some(',' | '}' | ' '))
        });
        assert!(written, "{file} の {n} 行目に {key} が無い: {at}");
    }
}

#[test]
fn f180_the_frozen_base_lines_are_the_hand_written_ones() {
    let text = summary(&repo_root().join(FLOOR_BASE));
    for want in [
        r#"{"id":"N-3","kind":"条","file":"constitution.yaml","line":463,"title":"例外の仕組みを作らない","plain":"「今回だけ特別」のスイッチを足す変更は、機械が止めます。変えたいときは正式に改訂します。","eng":"変更が規則の例外機構（無効化の旗・「今回だけ」の口）を足すなら、それを拒む。"}"#,
        r#"{"id":"P-6.2","kind":"規範文","file":"constitution.yaml","line":201,"title":"生成物を手で直さない。","plain":null,"eng":"生成物を手で直さない。"}"#,
        r#"{"id":"R-5","kind":"規則行","file":"rules.yaml","line":33,"title":"密度 profile と図の型の数","plain":null,"eng":"密度 profile と図の型の数"}"#,
        r#"{"id":"FR8","kind":"要件","file":"srs.yaml","line":195,"title":"途中で足す窓口","plain":"あとから「やっぱりこの文書も」となっても、最初からやり直さず、差分だけ相談します。","eng":"folio は差分だけを対象に intake を再実行し、支度表を更新する。"}"#,
        r#"{"id":"AC12","kind":"受入基準","file":"srs.yaml","line":409,"title":"検査を通らない図は生成されず、前の生成物が残る","plain":"わざと崩れた図の記述を入れて走らせ、図が作られず前の図がそのまま残ることを見せる。","eng":"検査を通らない図は生成されず、前の生成物が残る"}"#,
        r#"{"id":"AC11","kind":"受入基準","file":"srs.yaml","line":406,"title":"印の無い commit の契約は「まだ分からない」と出て「未着地」と出な","plain":"便が入ったかを示す印が無いとき、「まだ分からない」と表示され「入っていない」とは表示されないことを見せる。","eng":"印の無い commit の契約は「まだ分からない」と出て「未着地」と出ない"}"#,
        r#"{"id":"folio-v2","kind":"登場人物","file":"srs.yaml","line":92,"title":"folio v2","plain":null,"eng":null}"#,
    ] {
        let id = want.split('"').nth(3).unwrap();
        assert_eq!(line_of(&text, id), want);
    }
    // 判断の記録は file の頭の注の後の id の行・技術の要約は決定の欄
    let adr = line_of(&text, "ADR-9");
    let head = r#"{"id":"ADR-9","kind":"判断の記録","file":"adr/ADR-9.yaml","line":5,"title":"欄の決まりの file（判断の記録・設計ノート）の schema 節は床","plain":"「判断の記録の書き方」と"#;
    assert!(adr.starts_with(head), "{adr}");
    assert!(adr.contains(r#","eng":"欄の決まりの file 2 本（adr/schema.yaml・design-note/schema.yaml）のうち schema 節"#), "{adr}");
}

#[test]
fn f180_text_fields_are_escaped_into_one_json_line() {
    let work = Work::base("escape");
    work.replace(
        "srs.yaml",
        "    shall: folio は差分だけを対象に intake を再実行し、支度表を更新する。\n    plain: あとから「やっぱりこの文書も」となっても、最初からやり直さず、差分だけ相談します。\n",
        "    shall: folio は差分だけを対象に intake を再実行し、支度表を更新する。\n    text: 本文の欄（規範文の欄が先）\n    plain: \"タブ\\tと改行\\nと \\\"引用符\\\" と \\\\ 逆斜線と \\u0001 制御の字\"\n",
    );
    work.replace(
        "srs.yaml",
        "  - {id: folio-v2, name: folio v2, role: 道具}\n",
        "  - {id: folio-v2, name: folio v2, role: 道具, shall: ~, decision: 決定の欄, what: \"what の欄 \\\"x\\\"\", plain: ~}\n",
    );
    work.replace(
        "constitution.yaml",
        "      - {id: N-3.1, ",
        "      - {pattern: unwanted, text: id の無い行は規範文でない}\n      - {id: N-3.1, ",
    );
    work.replace(
        "srs.yaml",
        "  - {id: AC12, title: 検査を通らない図は生成されず、前の生成物が残る, ",
        "  - {id: AC12, title: \"検査を通らない図は  生成されず、\\n前の生成物が残る（題を 36 字で切らず、空白も畳まない全文）\", ",
    );
    work.replace(
        "constitution.yaml",
        "    plain: 「今回だけ特別」のスイッチを足す変更は、機械が止めます。変えたいときは正式に改訂します。\n",
        "    plain: |\n      「今回だけ特別」のスイッチを足す変更は、\n      機械が止めます。\n",
    );
    work.replace("rules.yaml", "  - {id: R-5, article: P-2, what: ", "  - {id: R-5, article: P-2, text: 本文が what より先, what: ");
    let text = summary(&work.dir());
    let print = passed(graph(&work.dir(), &["--print"]));
    let rows = print.lines().skip(1).take_while(|l| *l != EDGES_HEAD).count();
    assert_eq!(text.lines().count(), rows, "字の中の改行で行が割れた");
    assert!(!text.contains('\t'), "字の中のタブが逃がされていない");
    assert_eq!(
        line_of(&text, "FR8"),
        r#"{"id":"FR8","kind":"要件","file":"srs.yaml","line":195,"title":"途中で足す窓口","plain":"タブ\tと改行\nと \"引用符\" と \\ 逆斜線と \u0001 制御の字","eng":"folio は差分だけを対象に intake を再実行し、支度表を更新する。"}"#
    );
    assert_eq!(
        line_of(&text, "folio-v2"),
        r#"{"id":"folio-v2","kind":"登場人物","file":"srs.yaml","line":92,"title":"folio v2","plain":null,"eng":"what の欄 \"x\""}"#
    );
    // `|` の塊の平易文は複数行と末尾の改行ごと・条の技術の要約は id を持つ最初の規範文の字
    assert_eq!(
        line_of(&text, "N-3"),
        r#"{"id":"N-3","kind":"条","file":"constitution.yaml","line":463,"title":"例外の仕組みを作らない","plain":"「今回だけ特別」のスイッチを足す変更は、\n機械が止めます。\n","eng":"変更が規則の例外機構（無効化の旗・「今回だけ」の口）を足すなら、それを拒む。"}"#
    );
    // 本文の欄 text は what より先
    assert_eq!(
        line_of(&text, "R-5"),
        r#"{"id":"R-5","kind":"規則行","file":"rules.yaml","line":33,"title":"密度 profile と図の型の数","plain":null,"eng":"本文が what より先"}"#
    );
    assert!(
        line_of(&text, "AC12").ends_with(r#","title":"検査を通らない図は 生成されず、 前の生成物が残る（題を 36 字で切ら","plain":"わざと崩れた図の記述を入れて走らせ、図が作られず前の図がそのまま残ることを見せる。","eng":"検査を通らない図は  生成されず、\n前の生成物が残る（題を 36 字で切らず、空白も畳まない全文）"}"#),
        "受入基準の技術の要約が題の全文でない"
    );
}

#[test]
fn f180_an_unbuildable_index_prints_no_line() {
    let gone = Work::base("gone");
    fs::remove_file(gone.dir().join("srs.yaml")).unwrap();
    let broken = Work::base("disagree");
    broken.replace("constitution.yaml", "      - {id: P-1.1, ", "      -  {id: P-1.1, ");
    for (dir, why) in [(gone.dir(), "srs.yaml を読めない"), (broken.dir(), "食い違う")] {
        let out = graph(&dir, &["--print", "--summary"]);
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(2), "{err}");
        assert!(out.stdout.is_empty(), "行が出た: {}", String::from_utf8_lossy(&out.stdout));
        assert!(err.contains("まだ分からない") && err.contains(why), "{err}");
    }
    let dir = repo_root().join(FLOOR_BASE);
    for flags in [&["--summary"][..], &["--digest", "--summary"]] {
        let out = graph(&dir, flags);
        assert_eq!(out.status.code(), Some(2), "{flags:?}");
        assert!(out.stdout.is_empty(), "{flags:?}");
    }
    assert_eq!(passed(graph(&dir, &["--summary", "--print"])), summary(&dir), "旗の順で出力が変わった");
}

#[test]
fn f180_the_summary_leaves_the_print_and_the_digest_on_their_anchors() {
    let dir = repo_root().join(FLOOR_BASE);
    assert!(summary(&dir).starts_with("{\"id\":"), "--summary が節点ごとの行を出さない");
    let anchor = fs::read_to_string(repo_root().join("tests/fixtures/schema/graph-anchor.txt")).unwrap();
    let whole = anchor
        .lines()
        .find_map(|l| l.strip_prefix("出力全体 sha256 = "))
        .expect("anchor に出力全体の行が無い");
    let print = passed(graph(&dir, &["--print"]));
    assert_eq!(sha256_hex(print.as_bytes()), whole[..64], "--print の出力が anchor と違う");
    let digest = fs::read(repo_root().join("tests/fixtures/schema/graph-digest-anchor.txt")).unwrap();
    assert!(passed(graph(&dir, &["--digest"])).as_bytes() == digest.as_slice(), "--digest の出力が anchor と違う");
}
