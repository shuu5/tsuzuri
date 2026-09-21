//! `folio ceiling --gate`（便 73・docs/design/delivery-73.md §1 (d)(e)）の歯。binary 経由。
//! 正本は便 38 の凍結 fixture の束の source/（tests/fixtures/ceiling/bundle/source/）を一時 dir の design-intent/ へ写したもの。
//! 印は凍結 fixture stamp-pass / stamp-fail / stamp-unknown.yaml を写し、「同じ要約値」の場合は sources の仮の値（64 字の 0）を
//! 写しの正本から sha256sum（子の処理）で測った値に置き換え、「古い」の場合は仮の値のまま置く。
//! 命令は一時 dir を今の dir にして `--dir design-intent` で撃つ（write-set は repo の根からの相対）。
//!
//! 版管理の下の file は書き換えない（`--dir` は必ず一時 dir の中）。

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// fixture の印の sources の仮の値。
const PLACEHOLDER: &str =
    "sources: sha256 0000000000000000000000000000000000000000000000000000000000000000\n";

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

    /// 印を置く。`fresh` なら sources を今の正本の要約値に合わせる。
    fn put_stamp(&self, name: &str, fresh: bool) {
        let mut text = findings_fixture(name);
        assert!(text.contains(PLACEHOLDER), "{name}: 仮の値が無い");
        if fresh {
            text = text.replace(
                PLACEHOLDER,
                &format!("sources: sha256 {}\n", self.sources_hex()),
            );
        }
        fs::create_dir_all(self.dir().join("preview")).unwrap();
        fs::write(self.dir().join("preview/ceiling-stamp.yaml"), text).unwrap();
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
