//! `folio figure`（便 30・docs/design/delivery-30.md §1 (e)）の歯。binary 経由で図の道具（vendor/archify・
//! rules 行 R-15）を実際に撃つ。
//! - 凍結 anchor（tests/fixtures/figure/anchor/body.svg）との byte 一致（P-10.1）と決定的な再実行
//! - 道具の検査を通らない図は生成せず前の生成物も上書きしない（AC12）・診断の頭が持ち主に見える
//! - --check の 3 値・型と id と鍵と spec の「まだ分からない」・道具と node の不在
//! - 実の正本（example）の fig-1 の体裁（行内の様式 0・色の直書き 0・意味 class は部品目録の一覧の中）
//! - 版の固定（写しの version と要約値が R-15 の value に在る）
//! - 凍結 anchor の実行時の照合（便 60・P-10.3）: 道具の写しを改変すると --write / --check / 設計ノートの面が
//!   「まだ分からない」（2）に落ち、出力も前の生成物も書かない・改変しない写しでは保たれる
//!
//! 版管理の下の file は書き換えない（`--out` は必ず一時 dir の中・道具の改変は写しにだけ当てる）。

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use yaml_rust2::{Yaml, YamlLoader};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// 設計ノートの正本の写しの置き場（便 28 の fixture）。
fn face_fixture() -> PathBuf {
    repo_root().join("tests/fixtures/face")
}

/// 図の凍結 fixture の置き場。
fn figure_fixture() -> PathBuf {
    repo_root().join("tests/fixtures/figure")
}

fn design_intent() -> PathBuf {
    repo_root().join("design-intent")
}

fn vendor() -> PathBuf {
    repo_root().join("vendor/archify")
}

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-figure-t-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    fs::create_dir_all(&td).unwrap();
    td
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

/// 凍結 fixture の型付き記述を、設計ノートの figures の 1 行として書く。JSON の本文を 6 字下げて `spec:` の
/// 下に置く（JSON は YAML の流れの表として読める・実測 2026-09-19）。
fn fig_row(id: &str, spec: &Path) -> String {
    let body: String = fs::read_to_string(spec)
        .unwrap()
        .lines()
        .map(|line| format!("      {line}\n"))
        .collect();
    format!(
        "  - id: {id}\n    type: archify-architecture\n    caption: {id} の図\n    spec:\n{body}"
    )
}

/// fixture の正本 4 file と design-note/full.yaml を一時 dir の下の src/ へ、repo の vendor/archify/ を
/// 親 dir の vendor/archify/ へ写す。写しの figures に凍結 fixture の 2 行（fig-anchor・fig-fails）を足す。
/// 戻り値 = (一時 dir, 写し)。
fn fixture_copy(case: &str) -> (PathBuf, PathBuf) {
    let td = temp_dir(case);
    let work = td.join("src");
    fs::create_dir_all(work.join("design-note")).unwrap();
    for name in [
        "constitution.yaml",
        "rules.yaml",
        "vocabulary.yaml",
        "srs.yaml",
        "design-note/full.yaml",
    ] {
        fs::copy(face_fixture().join(name), work.join(name)).unwrap();
    }
    copy_dir(&vendor(), &td.join("vendor/archify"));
    let note = work.join("design-note/full.yaml");
    let text = fs::read_to_string(&note).unwrap();
    let rows = format!(
        "{}{}sources: []",
        fig_row("fig-anchor", &figure_fixture().join("anchor/spec.json")),
        fig_row("fig-fails", &figure_fixture().join("fails-showcase.json"))
    );
    let after = text.replacen("sources: []", &rows, 1);
    assert_ne!(text, after, "figures に凍結 fixture の行を足せていない");
    fs::write(&note, after).unwrap();
    (td, work)
}

/// `folio figure --doc <doc> --id <id> --dir <dir> --out <out> <mode>`。
fn folio_figure(doc: &str, id: &str, dir: &Path, out: &Path, mode: &str) -> Output {
    folio_figure_env(doc, id, dir, out, mode, None)
}

/// 環境変数 PATH を差し替えられる形（道具の実行環境の不在を測る）。
fn folio_figure_env(
    doc: &str,
    id: &str,
    dir: &Path,
    out: &Path,
    mode: &str,
    path_env: Option<&Path>,
) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_folio"));
    cmd.arg("figure")
        .arg("--doc")
        .arg(doc)
        .arg("--id")
        .arg(id)
        .arg("--dir")
        .arg(dir)
        .arg("--out")
        .arg(out)
        .arg(mode);
    if let Some(p) = path_env {
        cmd.env("PATH", p);
    }
    cmd.output().expect("folio を起動できない")
}

fn code(out: &Output, what: &str) -> i32 {
    out.status.code().unwrap_or_else(|| {
        panic!(
            "{what} が signal で終わった: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn frozen_body() -> Vec<u8> {
    fs::read(figure_fixture().join("anchor/body.svg")).unwrap()
}

/// 凍結の写し（道具の生の出力）に便 67 の置き換えを当てた byte 列 = folio が書く図の本体。
/// 置き換えは 2 か所（凡例の見出しの要素の中身と 1 つ目の開始タグの言語の宣言）だけで、他の byte は動かない
/// （docs/design/delivery-67.md §1 (c) 4・置き換えの字面は歯の側で持ち、生成側の関数を呼ばない）。
fn frozen_body_ja() -> Vec<u8> {
    let raw = String::from_utf8(frozen_body()).expect("凍結の写しが UTF-8 でない");
    let head = raw.find('>').expect("凍結の写しに開始タグの閉じが無い") + 1;
    assert_eq!(
        raw[..head].matches("lang=\"en\"").count(),
        1,
        "凍結の写しの 1 つ目の開始タグに lang=en が 1 つでない"
    );
    assert_eq!(
        raw.matches(">Legend<").count(),
        1,
        "凍結の写しに凡例の見出しが 1 つでない"
    );
    let swapped = raw
        .replacen("lang=\"en\"", "lang=\"ja\"", 1)
        .replacen(">Legend<", ">凡例<", 1);
    // 置き換えは字数を変えない（見出しは 6 字 → 2 字で byte は同じ 6・言語の宣言は 2 字）
    assert_eq!(swapped.len(), raw.len(), "置き換えで byte 数が動いた");
    swapped.into_bytes()
}

/// 写しに変異を当てる（当たっていなければ落とす）。
fn edit(path: &Path, f: impl FnOnce(&str) -> String) {
    let before = fs::read_to_string(path).unwrap();
    let after = f(&before);
    assert_ne!(before, after, "変異が当たっていない: {}", path.display());
    fs::write(path, after).unwrap();
}

/// 写しに変異を当てて fig-1 を --write し、結果と出力先の中身を返す。
fn mutated(case: &str, mutate: impl FnOnce(&str) -> String) -> (Output, bool) {
    let (td, work) = fixture_copy(case);
    edit(&work.join("design-note/full.yaml"), mutate);
    let out = td.join("fig.svg");
    let run = folio_figure("full", "fig-1", &work, &out, "--write");
    let wrote = out.exists();
    let _ = fs::remove_dir_all(&td);
    (run, wrote)
}

/// 導出できない入力は 2 ∧「まだ分からない」∧ 文言 ∧ 出力先に書かない。
fn unknown(case: &str, run: &Output, wrote: bool, wording: &str) {
    assert_eq!(code(run, case), 2, "{case}: {}", stderr(run));
    assert!(
        stderr(run).contains("まだ分からない"),
        "{case}: {}",
        stderr(run)
    );
    assert!(
        stderr(run).contains(wording),
        "{case}: 「{wording}」が無い: {}",
        stderr(run)
    );
    assert!(!wrote, "{case}: 導出できないのに出力先に書いた");
}

// ── 1. 凍結 anchor ──

#[test]
fn figure_write_matches_the_frozen_anchor() {
    let (td, work) = fixture_copy("anchor");
    let out = td.join("fig.svg");
    let run = folio_figure("full", "fig-anchor", &work, &out, "--write");
    let written = fs::read(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio figure --write"), 0, "{}", stderr(&run));
    // 比べる相手は凍結の写しに便 67 の置き換えを当てた byte 列（写しとの差は 2 か所だけ）
    let frozen = frozen_body_ja();
    if written != frozen {
        let at = written
            .iter()
            .zip(&frozen)
            .position(|(a, b)| a != b)
            .unwrap_or(written.len().min(frozen.len()));
        let show = |b: &[u8]| {
            String::from_utf8_lossy(&b[at.saturating_sub(120)..(at + 200).min(b.len())])
                .into_owned()
        };
        panic!(
            "anchor/body.svg（置き換えの後）と違う（{} byte ≠ {} byte・最初の差 {at} byte 目）\n--- folio\n{}\n--- 凍結\n{}",
            written.len(),
            frozen.len(),
            show(&written),
            show(&frozen)
        );
    }
    assert_eq!(stdout(&run), "folio figure: 書いた（16804 byte）\n");
}

// ── 2. 決定的 ──

#[test]
fn figure_write_twice_yields_the_same_bytes() {
    let (td, work) = fixture_copy("twice");
    let out = td.join("fig.svg");
    let first = folio_figure("full", "fig-anchor", &work, &out, "--write");
    let a = fs::read(&out).unwrap_or_default();
    let second = folio_figure("full", "fig-anchor", &work, &out, "--write");
    let b = fs::read(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&first, "1 回目"), 0, "{}", stderr(&first));
    assert_eq!(code(&second, "2 回目"), 0, "{}", stderr(&second));
    assert!(!a.is_empty());
    assert_eq!(a, b, "2 度撃つと byte が違う（決定的でない）");
}

// ── 3. 検査を通らない図（AC12）──

#[test]
fn figure_refuses_a_spec_that_fails_the_showcase_check_and_keeps_the_previous_body() {
    let (td, work) = fixture_copy("fails");
    let out = td.join("fig.svg");
    let before = "<svg>前の生成物</svg>".as_bytes().to_vec();
    fs::write(&out, &before).unwrap();
    let run = folio_figure("full", "fig-fails", &work, &out, "--write");
    let after = fs::read(&out).unwrap();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio figure --write"), 2, "{}", stderr(&run));
    let err = stderr(&run);
    assert!(err.contains("図の道具の検査を通らない"), "{err}");
    assert!(err.contains("labelDy"), "診断の中身が見えない: {err}");
    assert_eq!(after, before, "検査を通らないのに前の生成物を上書きした");
}

// ── 4. --check の 3 値 ──

#[test]
fn figure_check_has_three_values() {
    let (td, work) = fixture_copy("check");
    let out = td.join("fig.svg");

    let missing = folio_figure("full", "fig-anchor", &work, &out, "--check");
    let write = folio_figure("full", "fig-anchor", &work, &out, "--write");
    let ok = folio_figure("full", "fig-anchor", &work, &out, "--check");
    let mut bytes = fs::read(&out).unwrap();
    bytes[0] ^= 0x20;
    fs::write(&out, &bytes).unwrap();
    let drift = folio_figure("full", "fig-anchor", &work, &out, "--check");
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&missing, "check（未生成）"), 2, "{}", stderr(&missing));
    assert!(
        stderr(&missing).contains("図が無い"),
        "{}",
        stderr(&missing)
    );
    assert_eq!(code(&write, "write"), 0, "{}", stderr(&write));
    assert_eq!(code(&ok, "check（一致）"), 0, "{}", stderr(&ok));
    assert!(stdout(&ok).contains("folio figure: OK"), "{}", stdout(&ok));
    assert_eq!(code(&drift, "check（不一致）"), 1, "{}", stderr(&drift));
    assert!(stderr(&drift).contains("DRIFT"), "{}", stderr(&drift));
}

// ── 5. 導出できない入力 ──

#[test]
fn figure_unknown_when_the_type_is_not_a_tool_type() {
    let (run, wrote) = mutated("type", |t| {
        t.replacen("type: archify-architecture", "type: pipeline-rail", 1)
    });
    unknown("図の型", &run, wrote, "図の道具の型でない");
}

#[test]
fn figure_unknown_when_the_id_is_not_in_the_figures() {
    let (td, work) = fixture_copy("no-id");
    let out = td.join("fig.svg");
    let run = folio_figure("full", "fig-9", &work, &out, "--write");
    let wrote = out.exists();
    let _ = fs::remove_dir_all(&td);
    unknown("無い図 id", &run, wrote, "figures に無い");
}

#[test]
fn figure_unknown_when_the_doc_id_is_not_in_shape() {
    let (td, work) = fixture_copy("doc-shape");
    let out = td.join("fig.svg");
    let run = folio_figure("Full", "fig-anchor", &work, &out, "--write");
    let wrote = out.exists();
    let _ = fs::remove_dir_all(&td);
    unknown("大文字始まりの --doc", &run, wrote, "id の形でない");
}

#[test]
fn figure_unknown_when_a_spec_key_is_not_a_string() {
    let (run, wrote) = mutated("spec-key", |t| {
        t.replacen("      schema_version: 1", "      1: x", 1)
    });
    unknown("数字の鍵", &run, wrote, "文字列でない");
}

#[test]
fn figure_unknown_when_the_spec_is_not_a_map() {
    let (run, wrote) = mutated("spec-scalar", |t| {
        let at = t.find("    spec:\n").expect("fig-1 の spec が無い");
        let end = t[at..].find("\n  - id: fig-anchor").expect("次の図が無い");
        format!("{}    spec: 表でない{}", &t[..at], &t[at + end..])
    });
    unknown("文字列の spec", &run, wrote, "型付き記述（spec）が表でない");
}

// ── 6. 道具と実行環境の不在 ──

#[test]
fn figure_unknown_when_the_tool_or_node_is_absent() {
    let (td, work) = fixture_copy("no-tool");
    let out = td.join("fig.svg");
    let empty = td.join("empty-path");
    fs::create_dir_all(&empty).unwrap();
    let no_node = folio_figure_env("full", "fig-anchor", &work, &out, "--write", Some(&empty));
    let wrote_no_node = out.exists();
    fs::remove_file(td.join("vendor/archify/bin/archify.mjs")).unwrap();
    let no_tool = folio_figure("full", "fig-anchor", &work, &out, "--write");
    let wrote_no_tool = out.exists();
    let _ = fs::remove_dir_all(&td);
    unknown("node が無い", &no_node, wrote_no_node, "起動できない");
    unknown("道具が無い", &no_tool, wrote_no_tool, "図の道具が無い");
}

// ── 7. 実の正本 ──

/// 図の本体の class の語（`class="…"` の中身を空白で割る）。
fn classes(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(at) = rest.find("class=\"") {
        let tail = &rest[at + "class=\"".len()..];
        let end = tail.find('"').unwrap_or(tail.len());
        for word in tail[..end].split_whitespace() {
            out.push(word.to_string());
        }
        rest = &tail[end.min(tail.len())..];
    }
    out
}

/// 部品目録の図の本体の意味 class の全一覧（class の群だけ・data 属性の一覧は class でない）。
fn parts_figure_classes() -> Vec<String> {
    let text = fs::read_to_string(design_intent().join("preview/parts.json")).unwrap();
    let doc = YamlLoader::load_from_str(&text)
        .expect("parts.json を読めない")
        .remove(0);
    let table = &doc["figure_body_classes"];
    let mut out = Vec::new();
    for group in ["node_kind", "edge_kind", "arrowhead", "text_role", "sigil"] {
        let list = table[group]
            .as_vec()
            .unwrap_or_else(|| panic!("figure_body_classes.{group} が一覧でない"));
        for item in list {
            out.push(item.as_str().expect("class が文字列でない").to_string());
        }
    }
    assert!(out.len() > 20, "class の一覧が短すぎる: {out:?}");
    out
}

#[test]
fn figure_on_the_real_source_has_only_semantic_classes() {
    let td = temp_dir("real");
    let out = td.join("fig.svg");
    let run = folio_figure("example", "fig-1", &design_intent(), &out, "--write");
    let body = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio figure --write"), 0, "{}", stderr(&run));

    assert_eq!(body.matches("<svg").count(), 1, "図の本体が 1 つでない");
    assert_eq!(body.matches("style=\"").count(), 0, "行内の様式が在る");
    assert_eq!(body.matches("fill=\"#").count(), 0, "色の直書きが在る");

    let allowed = parts_figure_classes();
    for word in classes(&body) {
        assert!(
            allowed.contains(&word),
            "class「{word}」が部品目録の figure_body_classes に無い"
        );
    }
}

// ── 8. 凍結 anchor の実行時の照合（便 60・P-10.3）──

/// 写した道具を改変する（図の本体に出る class の字 m-default → m-defaultx・凍結 anchor の body.svg に実在する
/// 字で、道具の出力が変わる最小の改変）。repo の vendor は触らない。
fn drift_tool(td: &Path) {
    edit(&td.join("vendor/archify/renderers/shared/utils.mjs"), |t| {
        t.replacen("class=\"m-default\"", "class=\"m-defaultx\"", 1)
    });
}

const ANCHOR_DRIFT: &str = "凍結 anchor が落ちた";

#[test]
fn anchor_drift_makes_figure_write_unknown() {
    let (td, work) = fixture_copy("anchor-drift-write");
    drift_tool(&td);
    let out = td.join("fig.svg");
    let run = folio_figure("full", "fig-anchor", &work, &out, "--write");
    let wrote = out.exists();
    let _ = fs::remove_dir_all(&td);
    unknown("改変した道具で --write", &run, wrote, ANCHOR_DRIFT);
}

#[test]
fn anchor_drift_keeps_the_previous_output() {
    let (td, work) = fixture_copy("anchor-drift-keep");
    drift_tool(&td);
    let out = td.join("fig.svg");
    let before = "<svg>前の生成物</svg>".as_bytes().to_vec();
    fs::write(&out, &before).unwrap();
    let run = folio_figure("full", "fig-anchor", &work, &out, "--write");
    let after = fs::read(&out).unwrap();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio figure --write"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains(ANCHOR_DRIFT), "{}", stderr(&run));
    assert_eq!(
        after, before,
        "凍結 anchor が落ちたのに前の生成物を上書きした"
    );
}

#[test]
fn anchor_drift_makes_figure_check_unknown() {
    let (td, work) = fixture_copy("anchor-drift-check");
    let out = td.join("fig.svg");
    // 改変の前に書いた生成物を置き、--check が不合格（1）でなく「まだ分からない」（2）に落ちることを測る
    let write = folio_figure("full", "fig-anchor", &work, &out, "--write");
    drift_tool(&td);
    let check = folio_figure("full", "fig-anchor", &work, &out, "--check");
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&write, "write"), 0, "{}", stderr(&write));
    assert_eq!(
        code(&check, "check（改変した道具）"),
        2,
        "{}",
        stderr(&check)
    );
    assert!(
        stderr(&check).contains("まだ分からない"),
        "{}",
        stderr(&check)
    );
    assert!(stderr(&check).contains(ANCHOR_DRIFT), "{}", stderr(&check));
}

#[test]
fn anchor_drift_makes_the_note_face_unknown() {
    // 設計ノートの面の歯（tests/face_note.rs）と同じ写し: 天井の正本と器の導出 file を足す
    let (td, work) = fixture_copy("anchor-drift-note");
    fs::copy(
        face_fixture().join("ceiling.yaml"),
        work.join("ceiling.yaml"),
    )
    .unwrap();
    fs::create_dir_all(td.join("contracts")).unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        td.join("contracts/schema.toml"),
    )
    .unwrap();
    drift_tool(&td);
    let out = td.join("note-full.html");
    let run = Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("face")
        .arg("--face")
        .arg("note")
        .arg("--id")
        .arg("full")
        .arg("--dir")
        .arg(&work)
        .arg("--out")
        .arg(&out)
        .arg("--write")
        .output()
        .expect("folio を起動できない");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --write"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("まだ分からない"), "{}", stderr(&run));
    assert!(stderr(&run).contains(ANCHOR_DRIFT), "{}", stderr(&run));
    assert!(!exists, "凍結 anchor が落ちたのに面を書いた");
}

#[test]
fn anchor_holds_on_the_untouched_copy() {
    let (td, work) = fixture_copy("anchor-holds");
    let out = td.join("fig.svg");
    let run = folio_figure("full", "fig-anchor", &work, &out, "--write");
    let written = fs::read(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio figure --write"), 0, "{}", stderr(&run));
    assert!(stderr(&run).is_empty(), "{}", stderr(&run));
    assert_eq!(
        written,
        frozen_body_ja(),
        "改変しない写しの出力が anchor/body.svg（置き換えの後）と違う"
    );
}

// ── 9. 版の固定（R-15）──

/// 版管理の下の写しの全 file を path の byte 順に連結した byte 列。
fn vendor_bytes() -> (Vec<u8>, usize) {
    let mut files: Vec<PathBuf> = Vec::new();
    collect(&vendor(), &mut files);
    files.sort_by(|a, b| {
        a.as_os_str()
            .as_encoded_bytes()
            .cmp(b.as_os_str().as_encoded_bytes())
    });
    let mut bytes = Vec::new();
    for file in &files {
        bytes.extend_from_slice(&fs::read(file).unwrap());
    }
    (bytes, files.len())
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            collect(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// 要約値（sha256）は命令 `sha256sum` を子の処理で撃って measures する（歯は crate の中を読めない）。
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

/// rules 行 R-15 の value（歯の中で yaml で読む）。
fn rule_r15() -> String {
    let text = fs::read_to_string(design_intent().join("rules.yaml")).unwrap();
    let doc = YamlLoader::load_from_str(&text)
        .expect("rules.yaml を読めない")
        .remove(0);
    let rows = doc["thresholds"].as_vec().expect("thresholds が一覧でない");
    let row = rows
        .iter()
        .find(|r| r["id"].as_str() == Some("R-15"))
        .expect("R-15 が無い");
    match &row["value"] {
        Yaml::String(s) => s.clone(),
        other => panic!("R-15 の value が文字列でない: {other:?}"),
    }
}

#[test]
fn figure_tool_version_and_digest_are_frozen_in_the_rules() {
    let value = rule_r15();
    let text = fs::read_to_string(vendor().join("package.json")).unwrap();
    let pkg = YamlLoader::load_from_str(&text)
        .expect("package.json を読めない")
        .remove(0);
    let version = pkg["version"].as_str().expect("version が無い");
    assert!(
        value.contains(version),
        "写しの版「{version}」が R-15 の value に無い: {value}"
    );

    let (bytes, count) = vendor_bytes();
    assert_eq!(count, 43, "写しの file の数が R-15 の 43 file と違う");
    match sha256_hex(&bytes) {
        Ok(hex) => assert!(
            value.contains(&hex),
            "写しの要約値 {hex}（{count} file・{} byte）が R-15 の value に無い: {value}",
            bytes.len()
        ),
        // 測る道具が無いときは合格にしない代わりに理由を出す（panic で落とさない・FR5 の形）
        Err(why) => eprintln!("# まだ分からない: 要約値を測れない: {why}"),
    }
}

// ── 10. 図の本体の日本語（便 67・docs/design/delivery-67.md §1 (c)・天井の 11 周目の読みやすさ F-3）──

/// 凍結 anchor の図（凡例を持つ）を --write し、結果と書いた図の本体を返す。
fn anchor_write(case: &str) -> (Output, String) {
    let (td, work) = fixture_copy(case);
    let out = td.join("fig.svg");
    let run = folio_figure("full", "fig-anchor", &work, &out, "--write");
    let body = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    (run, body)
}

#[test]
fn legend_ja_replaces_the_heading_word() {
    let (run, body) = anchor_write("legend-ja-word");
    assert_eq!(code(&run, "folio figure --write"), 0, "{}", stderr(&run));
    assert_eq!(
        body.matches(">凡例<").count(),
        1,
        "凡例の見出しの要素の中身が日本語でない"
    );
    assert_eq!(
        body.matches(">Legend<").count(),
        0,
        "英語の見出しが残っている"
    );
    // 要素の中身でない Legend（注釈）は触らない
    assert_eq!(
        body.matches("<!-- Legend -->").count(),
        1,
        "要素の中身でない Legend まで置き換えた"
    );
}

#[test]
fn legend_ja_sets_the_language_to_ja() {
    let (run, body) = anchor_write("legend-ja-lang");
    assert_eq!(code(&run, "folio figure --write"), 0, "{}", stderr(&run));
    let head = &body[..body.find('>').expect("開始タグの閉じが無い") + 1];
    assert!(
        head.starts_with("<svg "),
        "1 つ目の要素が svg でない: {head}"
    );
    assert!(
        head.contains("lang=\"ja\""),
        "1 つ目の svg の開始タグの言語の宣言が ja でない: {head}"
    );
    assert_eq!(
        body.matches("lang=\"en\"").count(),
        0,
        "lang=en が残っている"
    );
}

#[test]
fn legend_ja_keeps_the_anchor_check_green() {
    // 照合は置き換えの前の値（道具の生の出力）で写しと比べるので、置き換えを掛けても anchor は落ちない
    // （便 60 の歯 anchor_holds_on_the_untouched_copy と同じ判定）
    let (run, body) = anchor_write("legend-ja-anchor");
    assert_eq!(code(&run, "folio figure --write"), 0, "{}", stderr(&run));
    assert!(
        !stderr(&run).contains(ANCHOR_DRIFT),
        "置き換えの後の値で照合している: {}",
        stderr(&run)
    );
    assert!(stderr(&run).is_empty(), "{}", stderr(&run));
    assert!(!body.is_empty(), "改変しない写しなのに図を書いていない");
}
