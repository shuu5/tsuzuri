//! 索引の設計ノートの行（便 185・docs/design/delivery-185.md §1 (c)・判断の記録 ADR-32・要件 FR14 第 1.54 版）の歯。folio は
//! 実行 file の crate なので命令を撃つ。期待の字はどれも歯の側の手書き（凍結 anchor・P-10.1）で、要約値は手で書いた本文の
//! sha256 を命令 sha256sum で測る（歯は crate の中を読めない・tests/tz3/graph.rs と同じ形）。
//! 1. 凍結した土台の契約表の行 example#a が、節点の行・req の辺・--summary の行・--digest の 3 表で手書きの字のとおりに出る。
//! 2. 塊の形の行（外の利用者の形）: id は meta の id（file 名でない）と行 id、題は行の section が指す節の題、eng は行の題の全文、
//!    line は行 id の行、要約値は req と depends の行を落とした塊。depends は同じノートの行への辺、行の無い depends は数えるだけ。
//!    行 id は欄の順を問わない（塊の形の 2 行目・流れの形の 2 つ目の欄）。契約表でない表の行（部品表・計画の行の索引）は節点にしない。
//! 3. 辺の欄（req・depends）だけの変更は要約値を動かさず、行の題の変更はその行の要約値だけを動かす。
//! 4. 設計ノートの外の行（節点・辺・--summary の行・--digest の行）は、設計ノートの置き場を消した写しと byte で同じ。
//! 5. 行の逐語で切れない行（引用符つきの行 id）・読めない設計ノート・dir でない置き場では表を出さずに まだ分からない（2）、
//!    床は引用符つきの行 id を種類 索引の節点 の違反 1 件に数える。
//! 6. 1 本のノートの 2 つの契約表に同じ行 id が在れば、索引のどの口（--print・--summary・--digest・folio hello）も
//!    組めず（2）、床は 索引の節点 の違反に数える。
//! 7. 退役の設計ノートの行も節点になる（状態で絞らない・判断の記録 ADR-32 決定 (1)）。
//! 8. 便 208（docs/design/delivery-208.md §1 (c)・判断の記録 ADR-35 決定 (2)・要件 FR31）: --summary の設計ノートの行の
//!    status はノートの meta.status の字そのまま（draft・effective・retired・example）で、状態の欄が無いか字でなければ null。
//! 9. 外の置き場（骨格 folio init に外の利用者の形のノートを足した写し）でも同じ形: 判断の記録と設計ノートの行だけが字を持つ。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const EDGES_HEAD: &str = "# 辺（1 行 = 端 / 端 / 型・タブ区切り）";
const FLOOR_BASE: &str = "tests/fixtures/floor_base/design-intent";
const KIND: &str = "設計ノートの行";

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

/// 凍結した土台の写しの一時 dir（歯の終わりに消す）。
struct Work {
    root: PathBuf,
}

impl Work {
    fn base(case: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-graph-notes-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_tree(&repo_root().join(FLOOR_BASE), &root.join("design-intent"));
        Work { root }
    }

    fn dir(&self) -> PathBuf {
        self.root.join("design-intent")
    }

    fn write(&self, file: &str, text: &str) {
        fs::write(self.dir().join(file), text).unwrap();
    }

    /// 写しの file の中の `from`（ちょうど 1 か所）を `to` に替える。
    fn replace(&self, file: &str, from: &str, to: &str) {
        let path = self.dir().join(file);
        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(text.matches(from).count(), 1, "{file}: 「{from}」が 1 か所でない");
        fs::write(&path, text.replacen(from, to, 1)).unwrap();
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn folio(args: &[&str], dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(args)
        .arg("--dir")
        .arg(dir)
        .output()
        .expect("folio を起動できない")
}

fn passed(out: Output) -> String {
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).expect("出力が UTF-8 でない")
}

fn print(dir: &Path) -> String {
    passed(folio(&["graph", "--print"], dir))
}

/// 節点の表の行と辺の表の行と要約の 1 行。
fn tables(text: &str) -> (Vec<&str>, Vec<&str>, &str) {
    let lines: Vec<&str> = text.lines().collect();
    let at = lines.iter().position(|l| *l == EDGES_HEAD).expect("辺の表の見出しが無い");
    (lines[1..at].to_vec(), lines[at + 1..lines.len() - 1].to_vec(), lines[lines.len() - 1])
}

/// 表の行のうち 1 列目（節点の id か辺の端）が設計ノートの行のもの。
fn note_rows<'a>(rows: &[&'a str]) -> Vec<&'a str> {
    rows.iter().copied().filter(|l| l.split('\t').next().is_some_and(|id| id.contains('#'))).collect()
}

/// 手で書いた本文の sha256 の先頭 8 字（命令 sha256sum で測る）。
fn digest_of(body: &str) -> String {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("sha256sum を起動できない");
    child.stdin.take().unwrap().write_all(body.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&out.stdout)[..8].to_string()
}

#[test]
fn f185_the_frozen_base_row_is_a_node_with_its_req_edge() {
    let dir = repo_root().join(FLOOR_BASE);
    let text = print(&dir);
    let (nodes, edges, _) = tables(&text);
    // 要約値の本文 = 流れの形の行から req と depends の対を落とした字（手書き）
    let body = "      - {id: a, title: 図の生成の口 3 つを 1 便で置く, section: \"1\", verify: [cargo nextest run -p folio figure], size: M, done: 型付き記述の見本 1 本が仕上がりの段を通り図の本体が部品として埋まる}\n";
    assert_eq!(digest_of(body), "4914ab68", "手書きの本文の要約値");
    assert_eq!(note_rows(&nodes), ["example#a\t設計ノートの行\tdesign-note/example.yaml\t4914ab68\t目的"]);
    assert_eq!(note_rows(&edges), ["example#a\tFR15\treq"]);
    let summary = passed(folio(&["graph", "--print", "--summary"], &dir));
    let lines: Vec<&str> = summary.lines().filter(|l| l.contains("#a\"")).collect();
    assert_eq!(
        lines,
        [r#"{"id":"example#a","kind":"設計ノートの行","file":"design-note/example.yaml","line":55,"title":"目的","plain":null,"eng":"図の生成の口 3 つを 1 便で置く","status":"example"}"#]
    );
    let digest = passed(folio(&["graph", "--digest"], &dir));
    for want in ["\n設計ノートの行\t1\n", "\nreq\t1\t0\n", "\ndepends\t0\t0\n", "\ndesign-note/example.yaml\t1\n"] {
        assert!(digest.contains(want), "--digest に「{}」が無い:\n{digest}", want.trim());
    }
}

/// 外の利用者の形の設計ノート（塊の形の行・file 名は meta の id と違う・契約表でない表の行〔部品表と計画の行の索引〕を持つ）。
const WAVE: &str = "meta:
  id: wave
  title: 塊の形の行の見本
  version: v0.1
  status: draft
  generated: 2026-09-28
  profile: design-note
sections:
  - n: 1
    type: prose
    title: 行 p — 一つ目の行の見出し
    body: |
      一つ目。
  - n: 2
    type: prose
    title: 行 q — 二つ目の行の見出しは三十六字を越えるので、索引の表の題では後ろが切れる
    body: |
      二つ目。
  - n: 3
    type: contract-table
    title: 契約表
    rows:
      - id: p
        title: 一つ目の行の題
        req: [FR1]
        section: \"1\"
        size: S
        depends: []
      - title: 二つ目の行の題は技術の要約として全文が出て、表の題の三十六字では切られないことを見る
        id: q
        req:
          - FR2
        section: \"2\"
        depends: [p, zz]
        size: M
      - {section: \"1\", id: r, req: [FR3], title: 三つ目の行は流れの形で id が先頭でない}
  - n: 4
    type: parts-table
    title: 部品
    rows:
      - {id: part, name: 部品, role: 節点にならない表の行}
  - n: 5
    type: row-index
    title: 行の索引
    rows:
      # folio:rows:begin — 生成区間・手で直さない・正本は置き場の契約表（folio derive --write が書く）
      - {id: s, doc: other}
      # folio:rows:end
";

#[test]
fn f185_block_rows_keep_the_meta_id_the_section_title_and_depends() {
    let work = Work::base("block");
    work.write("design-note/wave-file.yaml", WAVE);
    let text = print(&work.dir());
    let (nodes, edges, summary) = tables(&text);
    let p = digest_of("      - id: p\n        title: 一つ目の行の題\n        section: \"1\"\n        size: S\n");
    let q = digest_of("      - title: 二つ目の行の題は技術の要約として全文が出て、表の題の三十六字では切られないことを見る\n        id: q\n        section: \"2\"\n        size: M\n");
    let r = digest_of("      - {section: \"1\", id: r, title: 三つ目の行は流れの形で id が先頭でない}\n");
    assert_eq!(
        note_rows(&nodes),
        [
            "example#a\t設計ノートの行\tdesign-note/example.yaml\t4914ab68\t目的".to_string(),
            format!("wave#p\t{KIND}\tdesign-note/wave-file.yaml\t{p}\t行 p — 一つ目の行の見出し"),
            format!("wave#q\t{KIND}\tdesign-note/wave-file.yaml\t{q}\t行 q — 二つ目の行の見出しは三十六字を越えるので、索引の表の題では後"),
            format!("wave#r\t{KIND}\tdesign-note/wave-file.yaml\t{r}\t行 p — 一つ目の行の見出し"),
        ]
    );
    assert_eq!(
        note_rows(&edges),
        ["example#a\tFR15\treq", "wave#p\tFR1\treq", "wave#q\tFR2\treq", "wave#q\twave#p\tdepends", "wave#r\tFR3\treq"]
    );
    // 行の無い depends（wave#zz）は辺の表に出さず、端が節点でない参照に数える（土台の 26 に 1 を足す）
    assert_eq!(summary, "# 節点 197・辺 589・型 12・端が節点でない参照 27");
    let lines = passed(folio(&["graph", "--print", "--summary"], &work.dir()));
    let wave: Vec<&str> = lines.lines().filter(|l| l.starts_with("{\"id\":\"wave#")).collect();
    assert_eq!(
        wave,
        [
            r#"{"id":"wave#p","kind":"設計ノートの行","file":"design-note/wave-file.yaml","line":23,"title":"行 p — 一つ目の行の見出し","plain":null,"eng":"一つ目の行の題","status":"draft"}"#,
            r#"{"id":"wave#q","kind":"設計ノートの行","file":"design-note/wave-file.yaml","line":30,"title":"行 q — 二つ目の行の見出しは三十六字を越えるので、索引の表の題では後","plain":null,"eng":"二つ目の行の題は技術の要約として全文が出て、表の題の三十六字では切られないことを見る","status":"draft"}"#,
            r#"{"id":"wave#r","kind":"設計ノートの行","file":"design-note/wave-file.yaml","line":36,"title":"行 p — 一つ目の行の見出し","plain":null,"eng":"三つ目の行は流れの形で id が先頭でない","status":"draft"}"#,
        ]
    );
}

#[test]
fn f185_an_edge_only_change_moves_no_digest_and_a_title_moves_one() {
    let work = Work::base("digest");
    work.write("design-note/wave-file.yaml", WAVE);
    let before = note_rows(&tables(&print(&work.dir())).0).join("\n");
    work.replace("design-note/wave-file.yaml", "        req: [FR1]\n", "        req: [FR1, FR3]\n");
    work.replace("design-note/wave-file.yaml", "        depends: [p, zz]\n", "        depends: []\n");
    work.replace("design-note/wave-file.yaml", "          - FR2\n", "          - FR2\n          - FR4\n");
    let edges_only = note_rows(&tables(&print(&work.dir())).0).join("\n");
    assert_eq!(before, edges_only, "req と depends だけの変更で節点の行が動いた");
    work.replace("design-note/wave-file.yaml", "        title: 一つ目の行の題\n", "        title: 一つ目の行の題を直した\n");
    let after = print(&work.dir());
    let after = note_rows(&tables(&after).0);
    let moved: Vec<&str> = before
        .lines()
        .zip(&after)
        .filter(|(b, a)| b != *a)
        .map(|(b, _)| b.split('\t').next().unwrap())
        .collect();
    assert_eq!(moved, ["wave#p"], "動いた節点の行");
}

#[test]
fn f185_the_lines_outside_the_notes_do_not_move() {
    let with = Work::base("with");
    with.write("design-note/wave-file.yaml", WAVE);
    let without = Work::base("without");
    fs::remove_dir_all(without.dir().join("design-note")).unwrap();
    let (a, b) = (print(&with.dir()), print(&without.dir()));
    let ((an, ae, _), (bn, be, bs)) = (tables(&a), tables(&b));
    assert_eq!(note_rows(&an).len(), 4, "設計ノートの行の数");
    let outside = |rows: &[&str]| -> Vec<String> {
        rows.iter().filter(|l| !l.split('\t').next().unwrap().contains('#')).map(|l| l.to_string()).collect()
    };
    assert_eq!(outside(&an), bn, "設計ノートの外の節点の行");
    assert_eq!(outside(&ae), be, "設計ノートの外の辺の行");
    assert_eq!(bs, "# 節点 193・辺 584・型 10・端が節点でない参照 26", "設計ノートの無い写しの要約の 1 行");
    let summary = |dir: &Path| passed(folio(&["graph", "--print", "--summary"], dir));
    let (sa, sb) = (summary(&with.dir()), summary(&without.dir()));
    let kept: Vec<&str> = sa.lines().filter(|l| !l.starts_with("{\"id\":\"") || !l.split('"').nth(3).unwrap().contains('#')).collect();
    assert_eq!(kept, sb.lines().collect::<Vec<_>>(), "--summary の設計ノートの外の行");
    let digest = |dir: &Path| passed(folio(&["graph", "--digest"], dir));
    let (da, db) = (digest(&with.dir()), digest(&without.dir()));
    let only = |x: &str, y: &str| -> Vec<String> {
        x.lines().filter(|l| !y.lines().any(|m| m == *l)).map(str::to_string).collect()
    };
    assert_eq!(
        only(&da, &db),
        [
            "設計ノートの行\t4",
            "req\t4\t0",
            "depends\t1\t1",
            "design-note/example.yaml\t1",
            "design-note/wave-file.yaml\t3",
            "# 節点 197・辺 589・型 12・端が節点でない参照 27",
        ],
        "--digest で設計ノートの在る写しだけに在る行"
    );
    assert_eq!(
        only(&db, &da),
        ["設計ノートの行\t0", "req\t0\t0", "depends\t0\t0", "# 節点 193・辺 584・型 10・端が節点でない参照 26"],
        "--digest で設計ノートの無い写しだけに在る行"
    );
    assert_eq!(da.lines().count(), db.lines().count() + 2, "--digest の行の数（file の行 2 つ）");
}

#[test]
fn f185_an_unscannable_or_unreadable_note_is_inconclusive() {
    let quoted = Work::base("quoted");
    quoted.replace("design-note/example.yaml", "      - {id: a, title:", "      - {id: \"a\", title:");
    let broken = Work::base("broken");
    broken.write("design-note/broken.yaml", "meta: [\n");
    let file = Work::base("file");
    fs::remove_dir_all(file.dir().join("design-note")).unwrap();
    fs::write(file.dir().join("design-note"), "").unwrap();
    for (dir, why) in [(quoted.dir(), "食い違う"), (broken.dir(), "design-note/broken.yaml"), (file.dir(), "design-note/ が dir でない")] {
        let out = folio(&["graph", "--print"], &dir);
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(2), "{err}");
        assert!(out.stdout.is_empty(), "表が出た: {}", String::from_utf8_lossy(&out.stdout));
        assert!(err.contains("まだ分からない") && err.contains(why), "{err}");
    }
    let check = folio(&["check"], &quoted.dir());
    let text = format!("{}{}", String::from_utf8_lossy(&check.stdout), String::from_utf8_lossy(&check.stderr));
    let hits: Vec<&str> = text.lines().filter(|l| l.starts_with("[索引の節点]")).collect();
    assert_eq!(hits.len(), 1, "{text}");
    assert!(hits[0].contains("design-note/example.yaml") && hits[0].contains("example#a"), "{}", hits[0]);
}

#[test]
fn f185_a_row_id_twice_in_one_note_is_inconclusive() {
    let work = Work::base("twice");
    let twice = format!(
        "{WAVE}  - n: 6\n    type: contract-table\n    title: 二つ目の契約表\n    rows:\n      - {{id: p, title: 同じ行 id, req: [FR5], section: \"1\"}}\n"
    );
    work.write("design-note/wave-file.yaml", &twice);
    // 重なった行を黙って 1 つに数える口を残さない（--digest と folio hello は行の逐語を走らせない・P-4.1）
    for args in [&["graph", "--print"][..], &["graph", "--print", "--summary"], &["graph", "--digest"]] {
        let out = folio(args, &work.dir());
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(2), "{args:?}: {err}");
        assert!(out.stdout.is_empty(), "{args:?}: 表が出た");
        assert!(err.contains("まだ分からない") && err.contains("wave#p") && err.contains("2 度"), "{args:?}: {err}");
    }
    let hello = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["hello", "--dir"])
        .arg(work.dir())
        .arg("--state")
        .arg(work.root.join("state"))
        .output()
        .expect("folio を起動できない");
    let err = String::from_utf8_lossy(&hello.stderr);
    assert_eq!(hello.status.code(), Some(2), "{err}");
    assert!(err.contains("まだ分からない") && err.contains("wave#p"), "{err}");
    let check = folio(&["check"], &work.dir());
    let text = format!("{}{}", String::from_utf8_lossy(&check.stdout), String::from_utf8_lossy(&check.stderr));
    assert!(
        text.lines().any(|l| l.starts_with("[索引の節点] design-note/wave-file.yaml") && l.contains("2 度")),
        "{text}"
    );
}

/// 退役の設計ノート（状態 retired・契約表の行 1 つ）。
const OLD: &str = "meta:
  id: old
  title: 退役した設計ノート
  version: v1.0
  status: retired
  generated: 2026-09-28
  profile: design-note
sections:
  - n: 1
    type: prose
    title: 行 z — 退役した便の見出し
    body: |
      退役。
  - n: 2
    type: contract-table
    title: 契約表
    rows:
      - {id: z, title: 退役した便, req: [FR1], section: \"1\"}
";

#[test]
fn f185_a_retired_note_row_is_still_a_node() {
    let work = Work::base("retired");
    work.write("design-note/old.yaml", OLD);
    let text = print(&work.dir());
    let (nodes, edges, _) = tables(&text);
    let z = digest_of("      - {id: z, title: 退役した便, section: \"1\"}\n");
    assert_eq!(
        note_rows(&nodes),
        [
            "example#a\t設計ノートの行\tdesign-note/example.yaml\t4914ab68\t目的".to_string(),
            format!("old#z\t{KIND}\tdesign-note/old.yaml\t{z}\t行 z — 退役した便の見出し"),
        ]
    );
    assert_eq!(note_rows(&edges), ["example#a\tFR15\treq", "old#z\tFR1\treq"]);
}

/// --summary の行の id と最後の欄 status の字（status の欄がちょうど最後に 1 つ在ること）。
fn states(text: &str) -> Vec<(String, String)> {
    text.lines()
        .map(|line| {
            let at = line.rfind(",\"status\":").unwrap_or_else(|| panic!("status の欄が無い: {line}"));
            assert_eq!(line.matches(",\"status\":").count(), 1, "{line}");
            let tail = line[at + 10..].strip_suffix('}').unwrap_or_else(|| panic!("{line}"));
            (line.split('"').nth(3).unwrap().to_string(), tail.to_string())
        })
        .collect()
}

#[test]
fn f208_a_note_line_carries_the_note_state_word() {
    let work = Work::base("state");
    work.write("design-note/wave-file.yaml", WAVE);
    work.write("design-note/old.yaml", OLD);
    let eff = OLD.replace("id: old", "id: eff").replace("status: retired", "status: effective");
    work.write("design-note/eff.yaml", &eff);
    work.write("design-note/bare.yaml", &OLD.replace("id: old", "id: bare").replace("  status: retired\n", ""));
    work.write("design-note/list.yaml", &OLD.replace("id: old", "id: list").replace("status: retired", "status: [retired]"));
    let text = passed(folio(&["graph", "--print", "--summary"], &work.dir()));
    let notes: Vec<(String, String)> = states(&text).into_iter().filter(|(id, _)| id.contains('#')).collect();
    let want = [
        ("bare#z", "null"),
        ("eff#z", "\"effective\""),
        ("example#a", "\"example\""),
        ("list#z", "null"),
        ("old#z", "\"retired\""),
        ("wave#p", "\"draft\""),
        ("wave#q", "\"draft\""),
        ("wave#r", "\"draft\""),
    ];
    assert_eq!(notes, want.map(|(a, b)| (a.to_string(), b.to_string())));
}

#[test]
fn f208_an_outside_place_has_the_same_shape() {
    let root = std::env::temp_dir().join(format!("folio-graph-notes-outside-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let dir = root.join("design-intent");
    let init = Command::new(env!("CARGO_BIN_EXE_tz")).args(["init", "--dir"]).arg(&dir).output().unwrap();
    assert_eq!(init.status.code(), Some(0), "{}", String::from_utf8_lossy(&init.stderr));
    fs::write(dir.join("design-note/wave-file.yaml"), WAVE).unwrap();
    let text = passed(folio(&["graph", "--print", "--summary"], &dir));
    let _ = fs::remove_dir_all(&root);
    let keys = ["{\"id\":", ",\"kind\":", ",\"file\":", ",\"line\":", ",\"title\":", ",\"plain\":", ",\"eng\":", ",\"status\":"];
    for line in text.lines() {
        let mut at = 0;
        for key in keys {
            at += line[at..].find(key).unwrap_or_else(|| panic!("{key} が順に無い: {line}")) + key.len();
        }
    }
    let got = states(&text);
    let named: Vec<(&str, &str)> =
        got.iter().filter(|(_, st)| st != "null").map(|(id, st)| (id.as_str(), st.as_str())).collect();
    assert_eq!(
        named,
        [("ADR-1", "\"proposed\""), ("wave#p", "\"draft\""), ("wave#q", "\"draft\""), ("wave#r", "\"draft\"")]
    );
    assert_eq!(got.len(), 10, "骨格の 7 節点と外の利用者の形のノートの 3 行");
}
