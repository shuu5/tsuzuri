//! `folio ceiling --gate`（便 73・docs/design/delivery-73.md §1 (d)(e)）の歯。binary 経由。
//! 正本は便 38 の凍結 fixture の束の source/（tests/fixtures/ceiling/bundle/source/）を一時 dir の design-intent/ へ写したもの。
//! 印は凍結 fixture stamp-pass / stamp-fail / stamp-unknown.yaml を写し、「同じ要約値」の場合は sources の仮の値（64 字の 0）を
//! 写しの正本から sha256sum（子の処理）で測った値に置き換え、「古い」の場合は仮の値のまま置く。
//! 命令は一時 dir を今の dir にして `--dir design-intent` で撃つ（write-set は repo の根からの相対）。
//!
//! 便 126（docs/design/delivery-126.md §1 (e)）: 印の trigger の仮の値も、fresh のときは引き金の要約値の独立の実装
//! （凍結 anchor ceiling-region.txt の trigger の欄と写しの文書を yaml-rust2 で読み、正規化の json を sha256sum で測る・
//! folio の code を呼ばない）で測った値に置き換える。節点の数を見る歯は folio graph --print の節点の行から nodes の表を組み、
//! rest の仮の値と一緒に印の末尾に足す。
//! 便 129（docs/design/delivery-129.md §1 (e) の 1）: 独立の実装は憲法を凍結 anchor の trigger.constitution の scope の範囲で写す。
//!
//! 版管理の下の file は書き換えない（`--dir` は必ず一時 dir の中）。

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use yaml_rust2::{Yaml, YamlLoader};

/// fixture の印の sources の仮の値。
const PLACEHOLDER: &str =
    "sources: sha256 0000000000000000000000000000000000000000000000000000000000000000\n";
/// fixture の印の trigger の仮の値（便 126）。
const TRIGGER_PLACEHOLDER: &str =
    "trigger: sha256 0000000000000000000000000000000000000000000000000000000000000000\n";
/// 印の末尾に足す rest の仮の値（便 126）。
const REST_PLACEHOLDER: &str =
    "rest: sha256 0000000000000000000000000000000000000000000000000000000000000000\n";

/// 天井の正本の観点の reads が指す文書の file（bundle fixture の ceiling.yaml・file 形と dir 形）。
const READ_FILES: [&str; 6] = [
    "index.yaml",
    "constitution.yaml",
    "rules.yaml",
    "srs.yaml",
    "adr/",
    "design-note/",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn findings_fixture(name: &str) -> String {
    fs::read_to_string(
        repo_root()
            .join("tests/fixtures/ceiling/findings")
            .join(name),
    )
    .unwrap()
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or_else(|| {
        panic!(
            "folio が signal で終わった: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// 要約値（sha256）は命令 `sha256sum` を子の処理で撃って測る（tests/stamp.rs と同じ形）。
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

// ── 引き金の要約値の独立の実装（便 126・§1 (e)・folio の code を呼ばない）──

/// 正規化の json の値（表の鍵は byte 順）。
enum J {
    Null,
    /// 数と真偽の字面
    Raw(String),
    Str(String),
    Arr(Vec<J>),
    Obj(BTreeMap<String, J>),
}

impl J {
    fn of(y: &Yaml) -> J {
        match y {
            Yaml::Null => J::Null,
            Yaml::Boolean(b) => J::Raw(b.to_string()),
            Yaml::Integer(i) => J::Raw(i.to_string()),
            Yaml::Real(s) => J::Raw(s.clone()),
            Yaml::String(s) => J::Str(s.clone()),
            Yaml::Array(items) => J::Arr(items.iter().map(J::of).collect()),
            Yaml::Hash(h) => J::Obj(
                h.iter()
                    .map(|(k, v)| (k.as_str().expect("文字列でない鍵").to_string(), J::of(v)))
                    .collect(),
            ),
            other => panic!("写せない値: {other:?}"),
        }
    }

    /// キー順固定・空白なし・非 ASCII はそのまま（json.dumps の ensure_ascii なしと同じ escape）。
    fn text(&self, out: &mut String) {
        fn string(s: &str, out: &mut String) {
            out.push('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    '\u{8}' => out.push_str("\\b"),
                    '\u{c}' => out.push_str("\\f"),
                    c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                    c => out.push(c),
                }
            }
            out.push('"');
        }
        match self {
            J::Null => out.push_str("null"),
            J::Raw(s) => out.push_str(s),
            J::Str(s) => string(s, out),
            J::Arr(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    item.text(out);
                }
                out.push(']');
            }
            J::Obj(map) => {
                out.push('{');
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    string(k, out);
                    out.push(':');
                    v.text(out);
                }
                out.push('}');
            }
        }
    }
}

fn load_yaml(path: &Path) -> Yaml {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: 読めない: {e}", path.display()));
    YamlLoader::load_from_str(&text).unwrap().remove(0)
}

fn names(y: &Yaml) -> Vec<String> {
    y.as_vec()
        .expect("一覧でない")
        .iter()
        .map(|s| s.as_str().expect("字でない").to_string())
        .collect()
}

/// 表の欄（無ければ None）。
fn field<'a>(y: &'a Yaml, key: &str) -> Option<&'a Yaml> {
    y.as_hash()?.get(&Yaml::String(key.to_string()))
}

/// 行 1 つを欄の表に（点は入れ子を辿る・無ければ null）。
fn row_of(row: &Yaml, fields: &[String]) -> J {
    J::Obj(
        fields
            .iter()
            .map(|f| {
                let v = f.split('.').try_fold(row, |n, k| field(n, k));
                (f.clone(), v.map_or(J::Null, J::of))
            })
            .collect(),
    )
}

/// 行の一覧の節（無い・null は null）。
fn rows_of(doc: &Yaml, section: &str, fields: &[String]) -> J {
    match field(doc, section) {
        None | Some(Yaml::Null) => J::Null,
        Some(Yaml::Array(rows)) => J::Arr(rows.iter().map(|r| row_of(r, fields)).collect()),
        Some(other) => panic!("{section} が一覧でない: {other:?}"),
    }
}

/// 写しの置き場 `dir` の今の引き金の要約値（16 進 64 字）。一覧は凍結 anchor ceiling-region.txt の trigger の欄から読む。
fn trigger_hex(dir: &Path) -> String {
    let region = load_yaml(&repo_root().join("tests/fixtures/schema/ceiling-region.txt"));
    let trigger = &region["schema"]["trigger"];
    let ceiling = load_yaml(&dir.join("ceiling.yaml"));
    let documents: BTreeMap<String, String> = ceiling["documents"]
        .as_vec()
        .expect("documents が一覧でない")
        .iter()
        .map(|r| (r["id"].as_str().unwrap().to_string(), r["file"].as_str().unwrap().to_string()))
        .collect();
    let mut top: BTreeMap<String, J> = BTreeMap::new();
    for (doc, lists) in trigger.as_hash().expect("trigger が表でない") {
        let doc = doc.as_str().unwrap();
        let file = &documents[doc];
        let value = match doc {
            "constitution" => {
                let scope = names(&lists["scope"]);
                let (af, sf) = (names(&lists["articles"]), names(&lists["statements"]));
                let c = load_yaml(&dir.join(file));
                // 範囲の各節のうち articles 以外は丸ごと（便 129）
                let mut out: BTreeMap<String, J> = scope
                    .iter()
                    .filter(|s| *s != "articles")
                    .map(|s| (s.clone(), field(&c, s).map_or(J::Null, J::of)))
                    .collect();
                if !scope.iter().any(|s| s == "articles") {
                    top.insert(doc.to_string(), J::Obj(out));
                    continue;
                }
                let articles = c["articles"].as_vec().cloned().unwrap_or_default();
                let projected = articles
                    .iter()
                    .map(|a| {
                        J::Obj(
                            af.iter()
                                .map(|f| {
                                    let v = if f == "statements" {
                                        let sts = a[f.as_str()].as_vec().cloned().unwrap_or_default();
                                        J::Arr(sts.iter().map(|s| row_of(s, &sf)).collect())
                                    } else {
                                        field(a, f).map_or(J::Null, J::of)
                                    };
                                    (f.clone(), v)
                                })
                                .collect(),
                        )
                    })
                    .collect();
                out.insert("articles".to_string(), J::Arr(projected));
                J::Obj(out)
            }
            "adr" => {
                let (status, fields) = (names(&lists["status"]), names(&lists["fields"]));
                let sub = dir.join(file);
                let mut files: Vec<String> = fs::read_dir(&sub)
                    .unwrap()
                    .map(|e| e.unwrap())
                    .filter(|e| e.file_type().unwrap().is_file())
                    .map(|e| e.file_name().into_string().unwrap())
                    .filter(|n| n.ends_with(".yaml") && n != "schema.yaml")
                    .collect();
                files.sort();
                let mut out = Vec::new();
                for name in files {
                    let record = load_yaml(&sub.join(&name));
                    let effective = record["status"].as_str().is_some_and(|s| status.iter().any(|w| w == s))
                        && matches!(field(&record, "approval"), Some(Yaml::Hash(_)));
                    if effective {
                        out.push(row_of(&record, &fields));
                    }
                }
                J::Arr(out)
            }
            "rules" => {
                let (sections, fields) = (names(&lists["sections"]), names(&lists["fields"]));
                let r = load_yaml(&dir.join(file));
                J::Obj(sections.iter().map(|s| (s.clone(), rows_of(&r, s, &fields))).collect())
            }
            _ => {
                let d = load_yaml(&dir.join(file));
                let mut out = BTreeMap::new();
                for (section, list) in lists.as_hash().expect("文書の一覧が表でない") {
                    let section = section.as_str().unwrap();
                    if section == "whole" {
                        for name in names(list) {
                            let v = field(&d, &name).map_or(J::Null, J::of);
                            out.insert(name, v);
                        }
                    } else {
                        out.insert(section.to_string(), rows_of(&d, section, &names(list)));
                    }
                }
                J::Obj(out)
            }
        };
        top.insert(doc.to_string(), value);
    }
    let mut text = String::new();
    J::Obj(top).text(&mut text);
    sha256_hex(text.as_bytes()).expect("引き金の要約値を測れない")
}

/// 一時 dir（design-intent/ = 正本の写し）。
struct Repo {
    td: PathBuf,
}

impl Repo {
    fn new(case: &str) -> Repo {
        let td = std::env::temp_dir().join(format!("folio-gate-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&td);
        copy_tree(
            &repo_root().join("tests/fixtures/ceiling/bundle/source"),
            &td.join("design-intent"),
        );
        Repo { td }
    }

    fn dir(&self) -> PathBuf {
        self.td.join("design-intent")
    }

    /// 写しの正本の要約値（読む文書の file を design-intent からの相対 path の byte 順に連結した sha256）。
    fn sources_hex(&self) -> String {
        let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        for file in READ_FILES {
            let path = self.dir().join(file);
            if let Some(sub) = file.strip_suffix('/') {
                for entry in fs::read_dir(&path).unwrap() {
                    let name = entry.unwrap().file_name().into_string().unwrap();
                    if name.ends_with(".yaml") {
                        files.insert(
                            format!("{sub}/{name}"),
                            fs::read(path.join(&name)).unwrap(),
                        );
                    }
                }
            } else {
                files.insert(file.to_string(), fs::read(&path).unwrap());
            }
        }
        let bytes: Vec<u8> = files.into_values().flatten().collect();
        sha256_hex(&bytes).expect("要約値を測れない")
    }

    /// 印を置く。`fresh` なら sources を今の正本の要約値に、trigger を今の引き金の要約値（独立の実装）に合わせる。
    fn put_stamp(&self, name: &str, fresh: bool) {
        let mut text = findings_fixture(name);
        assert!(text.contains(PLACEHOLDER), "{name}: 仮の値が無い");
        assert!(text.contains(TRIGGER_PLACEHOLDER), "{name}: trigger の仮の値が無い");
        if fresh {
            text = text.replace(
                PLACEHOLDER,
                &format!("sources: sha256 {}\n", self.sources_hex()),
            );
            text = text.replace(
                TRIGGER_PLACEHOLDER,
                &format!("trigger: sha256 {}\n", trigger_hex(&self.dir())),
            );
        }
        self.write_stamp(&text);
    }

    /// fresh の合格の印に、rest の仮の値と今の節点の表（folio graph --print の節点の行の 1 列目と 4 列目）を足して置く。
    fn put_stamp_with_nodes(&self) {
        self.put_stamp("stamp-pass.yaml", true);
        let print = Command::new(env!("CARGO_BIN_EXE_folio"))
            .args(["graph", "--print", "--dir"])
            .arg(self.dir())
            .output()
            .expect("folio を起動できない");
        assert_eq!(code(&print), 0, "{}", String::from_utf8_lossy(&print.stderr));
        let mut text = self.read_stamp();
        text.push_str(REST_PLACEHOLDER);
        text.push_str("nodes:\n");
        let mut rows = 0;
        for line in stdout(&print).lines().take_while(|l| !l.starts_with("# 辺")) {
            if line.starts_with('#') {
                continue;
            }
            let cols: Vec<&str> = line.split('\t').collect();
            assert_eq!(cols.len(), 5, "節点の行の欄の数: {line}");
            text.push_str(&format!("  - {{id: {}, digest: {}}}\n", cols[0], cols[3]));
            rows += 1;
        }
        assert!(rows > 0, "節点の行が無い");
        self.write_stamp(&text);
    }

    fn read_stamp(&self) -> String {
        fs::read_to_string(self.dir().join("preview/ceiling-stamp.yaml")).unwrap()
    }

    fn write_stamp(&self, text: &str) {
        fs::create_dir_all(self.dir().join("preview")).unwrap();
        fs::write(self.dir().join("preview/ceiling-stamp.yaml"), text).unwrap();
    }

    /// 写しの正本 `file` の字 `from` を 1 か所だけ `to` に置き換える（1 か所でなければ歯を落とす）。
    fn edit(&self, file: &str, from: &str, to: &str) {
        let path = self.dir().join(file);
        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(text.matches(from).count(), 1, "{file}: 「{from}」が 1 か所でない");
        fs::write(&path, text.replacen(from, to, 1)).unwrap();
    }

    /// `folio ceiling --gate --dir design-intent --write-set <paths…>`（今の dir = 一時 dir）。
    fn gate(&self, write_set: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_folio"))
            .current_dir(&self.td)
            .args(["ceiling", "--gate", "--dir", "design-intent", "--write-set"])
            .args(write_set)
            .output()
            .expect("folio を起動できない")
    }

    fn done(self) {
        let _ = fs::remove_dir_all(&self.td);
    }
}

// ── 1. 設計文書の正本を書き換えない便 ──

#[test]
fn gate_passes_a_delivery_that_touches_no_design_intent() {
    let repo = Repo::new("no-design");
    let run = repo.gate(&[
        "crates/folio/src/gate.rs",
        "+crates/folio/tests/gate.rs",
        "docs/design/delivery-73.md",
    ]);
    repo.done();
    assert_eq!(code(&run), 0, "{}", stdout(&run));
    assert!(stdout(&run).contains("通す"), "{}", stdout(&run));
}

// ── 2. 4 観点合格・要約値が同じ ──

#[test]
fn gate_passes_when_the_stamp_is_all_pass_and_fresh() {
    let repo = Repo::new("pass");
    repo.put_stamp("stamp-pass.yaml", true);
    let run = repo.gate(&["design-intent/srs.yaml", "crates/folio/src/gate.rs"]);
    repo.done();
    assert_eq!(code(&run), 0, "{}", stdout(&run));
    let out = stdout(&run);
    assert!(out.contains("通す") && out.contains("合格"), "{out}");
}

// ── 3. 不合格の観点 ──

#[test]
fn gate_stops_on_a_failed_viewpoint() {
    let repo = Repo::new("fail");
    repo.put_stamp("stamp-fail.yaml", true);
    let run = repo.gate(&["design-intent/srs.yaml"]);
    repo.done();
    assert_eq!(code(&run), 1, "{}", stdout(&run));
    let out = stdout(&run);
    assert!(out.contains("止める") && out.contains("coherence"), "{out}");
}

// ── 4. 印が古い ──

#[test]
fn gate_is_unknown_when_the_stamp_is_stale() {
    let repo = Repo::new("stale");
    repo.put_stamp("stamp-pass.yaml", false);
    let run = repo.gate(&["design-intent/srs.yaml"]);
    repo.done();
    assert_eq!(code(&run), 2, "{}", stdout(&run));
    let out = stdout(&run);
    assert!(out.contains("まだ分からない") && out.contains("印が古い"), "{out}");
}

// ── 5. まだ分からない観点 ──

#[test]
fn gate_is_unknown_on_an_unknown_viewpoint() {
    let repo = Repo::new("unknown");
    repo.put_stamp("stamp-unknown.yaml", true);
    let run = repo.gate(&["+design-intent/adr/ADR-3.yaml"]);
    repo.done();
    assert_eq!(code(&run), 2, "{}", stdout(&run));
    let out = stdout(&run);
    assert!(out.contains("まだ分からない") && out.contains("reality"), "{out}");
}

// ── 6. 印が無い ──

#[test]
fn gate_is_unknown_without_a_stamp() {
    let repo = Repo::new("no-stamp");
    let run = repo.gate(&["design-intent/srs.yaml"]);
    repo.done();
    assert_eq!(code(&run), 2, "{}", stdout(&run));
    let out = stdout(&run);
    assert!(out.contains("まだ分からない") && out.contains("印が無い"), "{out}");
}

// ── 7. preview の生成物と retired は正本でない ──

#[test]
fn gate_ignores_preview_and_retired_paths() {
    let repo = Repo::new("preview-retired");
    let run = repo.gate(&[
        "design-intent/preview/ceiling-stamp.yaml",
        "design-intent/preview/retired/readable.html",
        "design-intent/adr/retired/ADR-0.yaml",
    ]);
    repo.done();
    assert_eq!(code(&run), 0, "{}", stdout(&run));
    let out = stdout(&run);
    assert!(
        out.contains("通す") && out.contains("設計文書の正本を書き換えない便"),
        "{out}"
    );
}

// ── 便 126: 引き金の要約値で古さを判定する（docs/design/delivery-126.md §1 (e) の 1〜6） ──

/// 要件 FR1 の平易文（引き金の外・節点 FR1 の中）。
const FR1_PLAIN: (&str, &str) = (
    "    plain: 質問が出て、どれにもおすすめが付きます。\n",
    "    plain: 質問が出て、どれにもおすすめが付く。\n",
);

#[test]
fn f126_the_gate_passes_a_plain_edit_and_counts_one_node() {
    let repo = Repo::new("f126-plain");
    repo.put_stamp_with_nodes();
    repo.edit("srs.yaml", FR1_PLAIN.0, FR1_PLAIN.1);
    let run = repo.gate(&["design-intent/srs.yaml"]);
    repo.done();
    let out = stdout(&run);
    assert_eq!(code(&run), 0, "{out}");
    assert!(
        out.contains("通す") && out.contains("引き金の外の変更") && out.contains("節点 1 個"),
        "{out}"
    );
}

#[test]
fn f126_the_gate_is_stale_on_each_normative_edit() {
    let cases: [(&str, &str, &str); 5] = [
        (
            "constitution.yaml",
            "text: 道具は検査の結果を知らせる。}",
            "text: 道具は検査の結果を必ず知らせる。}",
        ),
        (
            "srs.yaml",
            "    shall: folio は易しい質問を推奨回答つきで出す。\n",
            "    shall: folio は易しい質問を推奨回答つきで必ず出す。\n",
        ),
        (
            "rules.yaml",
            "    what: AI へ常時渡す説明文の合計\n",
            "    what: AI へ常時渡す説明文の総量\n",
        ),
        (
            "ceiling.yaml",
            "    question: 人が書いた自由文（",
            "    question: 人の書いた自由文（",
        ),
        (
            "adr/ADR-2.yaml",
            "\nstatus: proposed\n",
            "\nstatus: accepted\napproval: {who: 持ち主, date: 2026-09-07, ruling: 裁定 F-9, verbatim: 承認する, surface: R-8}\n",
        ),
    ];
    for (i, (file, from, to)) in cases.into_iter().enumerate() {
        let repo = Repo::new(&format!("f126-norm-{i}"));
        repo.put_stamp("stamp-pass.yaml", true);
        repo.edit(file, from, to);
        let run = repo.gate(&[&format!("design-intent/{file}")]);
        repo.done();
        let out = stdout(&run);
        assert_eq!(code(&run), 2, "{file}: {out}");
        assert!(
            out.contains("印が古い") && out.contains("引き金の要約値が違う"),
            "{file}: {out}"
        );
    }
}

#[test]
fn f126_the_gate_passes_edits_outside_the_trigger() {
    // （file・元の字・変えた字・読む文書の file か＝節点の数を添えるか）
    let cases: [(&str, &str, &str, bool); 9] = [
        (
            "constitution.yaml",
            "    plain: 道具は知らせるところまでで、決めるのはあなたです。\n",
            "    plain: 道具は知らせるところまでで、決めるのはあなた。\n",
            true,
        ),
        (
            "srs.yaml",
            "plain: 答えを入れると表が出る。,",
            "plain: 答えを入れると表が出ます。,",
            true,
        ),
        (
            "rules.yaml",
            "    note: 測り方を凍結してから再計測\n",
            "    note: 測り方を凍結してから測り直す\n",
            true,
        ),
        ("rules.yaml", "    ruling: 裁定 F-5\n", "    ruling: 裁定 F-6\n", true),
        ("srs.yaml", "  version: v0.3\n", "  version: v0.4\n", true),
        (
            "adr/ADR-2.yaml",
            "decision: 見本の判断の記録を 1 本置き",
            "decision: 見本の判断の記録を 2 本置き",
            true,
        ),
        (
            "design-note/full.yaml",
            "      面の骨格を 1 枚で測る。",
            "      面の骨格を 1 枚で量る。",
            true,
        ),
        (
            "ceiling.yaml",
            "{id: srs, file: srs.yaml, note: 要件書}",
            "{id: srs, file: srs.yaml, note: 要件の書}",
            false,
        ),
        ("ceiling.yaml", "    name: 忠実さ\n", "    name: 忠実性\n", false),
    ];
    for (i, (file, from, to, read)) in cases.into_iter().enumerate() {
        let repo = Repo::new(&format!("f126-outside-{i}"));
        repo.put_stamp_with_nodes();
        repo.edit(file, from, to);
        let run = repo.gate(&[&format!("design-intent/{file}")]);
        repo.done();
        let out = stdout(&run);
        assert_eq!(code(&run), 0, "{file}「{to}」: {out}");
        assert!(out.contains("通す"), "{file}: {out}");
        let word = if read { "引き金の外の変更" } else { "正本の要約値が同じ" };
        assert!(out.contains(word), "{file}「{to}」: {out}");
    }
}

#[test]
fn f126_a_stamp_without_the_trigger_is_stale() {
    let repo = Repo::new("f126-no-trigger");
    repo.put_stamp("stamp-pass.yaml", true);
    let stamp = repo.read_stamp();
    let line = stamp
        .lines()
        .find(|l| l.starts_with("trigger: "))
        .expect("印に trigger の行が無い")
        .to_string();
    repo.write_stamp(&stamp.replacen(&format!("{line}\n"), "", 1));
    let run = repo.gate(&["design-intent/srs.yaml"]);
    repo.done();
    let out = stdout(&run);
    assert_eq!(code(&run), 2, "{out}");
    assert!(
        out.contains("印が古い") && out.contains("引き金の要約値の欄が無い"),
        "{out}"
    );
}

#[test]
fn f126_the_gate_is_unknown_when_the_trigger_cannot_be_measured() {
    let repo = Repo::new("f126-unmeasurable");
    repo.put_stamp("stamp-pass.yaml", true);
    let path = repo.dir().join("constitution.yaml");
    let mut text = fs::read_to_string(&path).unwrap();
    text.push_str("\narticles: []\n");
    fs::write(&path, text).unwrap();
    let run = repo.gate(&["design-intent/constitution.yaml"]);
    repo.done();
    let out = stdout(&run);
    assert_eq!(code(&run), 2, "{out}");
    assert!(out.contains("引き金の要約値が測れない"), "{out}");
}

#[test]
fn f126_the_gate_does_not_pass_without_the_node_table() {
    let repo = Repo::new("f126-no-nodes");
    repo.put_stamp("stamp-pass.yaml", true);
    repo.edit("srs.yaml", FR1_PLAIN.0, FR1_PLAIN.1);
    let run = repo.gate(&["design-intent/srs.yaml"]);
    repo.done();
    let out = stdout(&run);
    assert_eq!(code(&run), 2, "{out}");
    assert!(out.contains("印の節点の表が読めない"), "{out}");
}

// ── 便 129: 憲法の前文と schema 節も引き金に入る（docs/design/delivery-129.md §1 (c) の 1） ──

#[test]
fn f129_the_gate_is_stale_on_a_precedence_or_schema_edit() {
    // （元の字・変えた字・終了コード・標準出力に要る字）。元の字が空なら何も変えない
    let cases: [(&str, &str, i32, [&str; 2]); 4] = [
        ("", "", 0, ["通す", "正本の要約値が同じ"]),
        (
            "  plain: 迷ったら、揃っている方を選びます。\n",
            "  plain: 迷ったら、揃っている方を選ぶ。\n",
            2,
            ["印が古い", "引き金の要約値が違う"],
        ),
        (
            "    tier: [always, ask-first, never]\n",
            "    tier: [always, never, ask-first]\n",
            2,
            ["印が古い", "引き金の要約値が違う"],
        ),
        (
            "articles: [A-1], sections: [§6]}",
            "articles: [A-1, N-1], sections: [§6]}",
            0,
            ["通す", "引き金の外の変更"],
        ),
    ];
    for (i, (from, to, want, words)) in cases.into_iter().enumerate() {
        let repo = Repo::new(&format!("f129-{i}"));
        repo.put_stamp_with_nodes();
        if !from.is_empty() {
            repo.edit("constitution.yaml", from, to);
        }
        let run = repo.gate(&["design-intent/constitution.yaml"]);
        repo.done();
        let out = stdout(&run);
        assert_eq!(code(&run), want, "写し {i}: {out}");
        assert!(words.iter().all(|w| out.contains(w)), "写し {i}: {out}");
    }
}
