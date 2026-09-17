//! `folio check` の凍結 anchor の列の歯（便 7・docs/design/delivery-7.md §1）。
//! tests/fixtures/anchor/ の 2 組: `no-anchor/`（違反 0・anchors/ なし）で まだ分からない 2（凍結 anchor が 0 本）、
//! `root-digest-drift/`（自分自身と一致する digest の anchor と索引）で 不合格 1（列の根の digest が床の定数と違う・違反 1 件）。
//! 加えて main の `design-intent/anchors/constitution-v1.0.yaml` を型付きの読みと正規化で sha256 に掛け、
//! file の digest 欄と床の定数 root_digest に一致することを見る（凍結 anchor が生成側からも検査側からも独立した物差し・P-10.1）。
//! fixture の digest は歯の中では計算しない（file に書いた値＝凍結）。

#[allow(dead_code)]
#[path = "../src/sha256.rs"]
mod sha256;
#[allow(dead_code)]
#[path = "../src/yaml.rs"]
mod yaml;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 床の定数 root_digest（scripts/check_draft.py の ROOT_DIGEST）。
const ROOT_DIGEST: &str = "acb52acd04b5d3a1feaf9ad5f0138f7614ce31964144b46ead914bde86e866ed";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn folio_check(dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("check")
        .arg("--dir")
        .arg(dir)
        .output()
        .expect("folio を起動できない")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// 違反の行（`[種類] …`）だけを拾う。
fn violations(out: &Output) -> Vec<String> {
    stdout(out)
        .lines()
        .filter(|l| l.starts_with('['))
        .map(str::to_string)
        .collect()
}

fn fixture(name: &str) -> PathBuf {
    repo_root().join("tests/fixtures/anchor").join(name)
}

/// digest 欄を除く全欄の正規化の sha256（day-1 の床の digest_of と同じ式）。
fn digest_of(doc: &yaml::Value) -> String {
    let body: Vec<(yaml::Value, yaml::Value)> = doc
        .as_map()
        .expect("anchor が欄の表でない")
        .iter()
        .filter(|(k, _)| k.as_str() != Some("digest"))
        .cloned()
        .collect();
    let text = yaml::canonical(&yaml::Value::Map(body)).expect("正規化できない");
    sha256::hex(text.as_bytes())
}

#[test]
fn anchor_no_anchor_is_unknown() {
    let out = folio_check(&fixture("no-anchor"));
    assert_eq!(
        out.status.code(),
        Some(2),
        "{}{}",
        stdout(&out),
        stderr(&out)
    );
    assert!(violations(&out).is_empty(), "{:?}", violations(&out));
    assert!(
        stderr(&out)
            .lines()
            .any(|l| l.starts_with("# まだ分からない: ") && l.contains("凍結 anchor が 0 本")),
        "{}",
        stderr(&out)
    );
    let s = stdout(&out);
    assert!(
        s.contains("まだ分からない") && !s.contains("folio check: 合格"),
        "{s}"
    );
}

#[test]
fn anchor_root_digest_drift_fails() {
    let out = folio_check(&fixture("root-digest-drift"));
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}{}",
        stdout(&out),
        stderr(&out)
    );
    let v = violations(&out);
    assert_eq!(v.len(), 1, "違反は列の根の digest の 1 件だけのはず: {v:?}");
    assert!(
        v[0].starts_with("[anchor] ") && v[0].contains("列の根") && v[0].contains("床の定数"),
        "{v:?}"
    );
    assert!(stdout(&out).contains("不合格"));
}

#[test]
fn anchor_main_digest_matches_floor_constant() {
    let path = repo_root().join("design-intent/anchors/constitution-v1.0.yaml");
    let doc = yaml::parse_typed(&fs::read_to_string(path).unwrap()).expect("型付きで読めない");
    let computed = digest_of(&doc);
    assert_eq!(
        doc.get("digest").and_then(yaml::Value::as_str),
        Some(computed.as_str())
    );
    assert_eq!(computed, ROOT_DIGEST);
}
