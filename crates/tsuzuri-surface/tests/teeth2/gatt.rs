//! 行 g-attach-acct の歯: Option の card の directive の口 attach_some を hover.rs の 1 つに寄せ、
//! account board の projects.rs と home.rs は私的な写しを持たずに hover の口を use で引く。
//! DOM は wasm の target のときだけなので、file の字で見る。
#![cfg(test)]

use std::path::{Path, PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn squeeze(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// dir の下の rs の file を名の順に全部集める（src からの相対の path と字）。
fn rs_files(dir: &Path, rel: &str, out: &mut Vec<(String, String)>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{} を読む: {e}", dir.display()))
        .map(|e| e.expect("dir の項").path())
        .collect();
    entries.sort();
    for path in entries {
        let name = path.file_name().expect("名").to_string_lossy().into_owned();
        let sub = if rel.is_empty() { name } else { format!("{rel}/{name}") };
        if path.is_dir() {
            rs_files(&path, &sub, out);
        } else if path.extension().is_some_and(|x| x == "rs") {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
            out.push((sub, text));
        }
    }
}

/// file の字 `mod dom {` 以後。
fn dom_part(rel: &str) -> String {
    let text = read(rel);
    let at = text
        .find("mod dom {")
        .unwrap_or_else(|| panic!("{rel} に字 mod dom {{ が無い"));
    text[at..].to_string()
}

/// (1) 字 fn attach_some を持つ file は widgets/hover.rs の 1 つだけで、その宣言は pub の 1 つ。
#[test]
fn gatt_one_attach_some() {
    let mut files = Vec::new();
    rs_files(&crate_dir().join("src"), "", &mut files);
    assert!(files.iter().any(|(p, _)| p == "widgets/hover.rs"), "src の下に widgets/hover.rs が無い");
    let having: Vec<&str> = files
        .iter()
        .filter(|(_, t)| t.contains("fn attach_some"))
        .map(|(p, _)| p.as_str())
        .collect();
    assert_eq!(having, ["widgets/hover.rs"], "字 fn attach_some を持つ file");

    let hover = read("src/widgets/hover.rs");
    assert_eq!(hover.matches("fn attach_some").count(), 1, "hover.rs の字 fn attach_some の数");
    assert_eq!(
        squeeze(&hover)
            .matches("pubfnattach_some(el:web_sys::Element,card:Option<Card>){")
            .count(),
        1,
        "hover.rs の pub fn attach_some の宣言の形の数"
    );
}

/// (2) projects.rs と home.rs の mod dom が hover の attach_some を use で引き、directive の字は変わらない。
#[test]
fn gatt_files_use_shared() {
    for (rel, line, uses) in [
        ("src/account/projects.rs", "usecrate::widgets::hover::{Card,attach,attach_some};", 5),
        ("src/account/home.rs", "usecrate::widgets::hover::{attach,attach_some};", 1),
    ] {
        assert_eq!(squeeze(&dom_part(rel)).matches(line).count(), 1, "{rel} の mod dom の {line} の数");
        assert_eq!(read(rel).matches("use:attach_some=").count(), uses, "{rel} の字 use:attach_some= の数");
    }
}

/// (3) この file の test の属性の付いた fn は 3 つで、名はどれも gatt_ で始まる。
#[test]
fn gatt_own_names() {
    let text = read("tests/teeth2/gatt.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0] == "#[test]")
        .map(|w| {
            let rest = w[1].strip_prefix("fn ").unwrap_or_else(|| panic!("test の属性の後が fn でない: {}", w[1]));
            &rest[..rest.find('(').unwrap_or(rest.len())]
        })
        .collect();
    assert_eq!(names.len(), 3, "test の fn {names:?}");
    for n in &names {
        assert!(n.starts_with("gatt_"), "test の fn {n} の名が gatt_ で始まらない");
    }
}
