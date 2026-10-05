//! stylesheet の部品の歯（面・接頭辞 stparts_・設計ノート surface-wave29b 行 g-style-parts・判断の記録 ADR-58）。
//! 部品の表は style の dir の .css の file の名の順・つないだ字は名の注と中身・selector の字は基と部品の全部で 1 つの file だけ・
//! wasm は面を描く前に部品を入れる。否定の見本は 1 欄だけ替える。
#![cfg(test)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use tsuzuri_surface::style::{PARTS, joined};

fn crate_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read(rel: &str) -> String {
    fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// file の字の selector（注を除き空白を詰め `,` で割った 1 つずつ・@media などの中も数え・@keyframes の中は数えない）。
fn selectors(css: &str) -> Vec<String> {
    let mut text = String::new();
    let mut rest = css;
    while let Some(i) = rest.find("/*") {
        text.push_str(&rest[..i]);
        rest = rest[i..].find("*/").map_or("", |j| &rest[i + j + 2..]);
    }
    text.push_str(rest);
    let mut out = Vec::new();
    // 開いた塊の型（true = 規則の中身か @keyframes の中で、selector を読まない）。
    let mut stack: Vec<bool> = Vec::new();
    let mut head = String::new();
    for c in text.chars() {
        let skip = stack.last().copied().unwrap_or(false);
        match c {
            '{' => {
                let prelude = head.split_whitespace().collect::<Vec<_>>().join(" ");
                if !skip && !prelude.starts_with('@') {
                    out.extend(
                        prelude
                            .split(',')
                            .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" ")),
                    );
                }
                stack.push(skip || !prelude.starts_with('@') || prelude.starts_with("@keyframes"));
                head.clear();
            }
            '}' => {
                stack.pop();
                head.clear();
            }
            ';' => head.clear(),
            _ => head.push(c),
        }
    }
    out
}

/// selector を 2 つ以上の file が持てば、selector と file の名を書いた Err（同じ file の中の 2 度は数えない）。
fn owned_once(files: &[(&str, &str)]) -> Result<(), String> {
    let mut owner: BTreeMap<String, &str> = BTreeMap::new();
    for (name, css) in files {
        for sel in selectors(css) {
            if let Some(first) = owner.insert(sel.clone(), name).filter(|f| f != name) {
                return Err(format!("selector {sel} が {first} と {name} の両方に在る"));
            }
        }
    }
    Ok(())
}

/// (1) 部品の表は style の dir の .css の file の名の byte の順で、名と中身の字が file と同じ。
#[test]
fn stparts_table_follows_dir() {
    let mut names: Vec<String> = fs::read_dir(crate_dir().join("style"))
        .expect("style の dir を読む")
        .map(|e| {
            e.expect("dir の項")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|n| n.ends_with(".css"))
        .collect();
    names.sort();
    assert!(names.contains(&"000-about.css".to_string()), "{names:?}");
    let table: Vec<&str> = PARTS.iter().map(|(n, _)| *n).collect();
    let want: Vec<String> = names.iter().map(|n| format!("style/{n}")).collect();
    assert_eq!(table, want);
    for (name, text) in PARTS {
        assert_eq!(text, read(name), "{name} の中身");
    }
}

/// (2) つないだ字は表の順に、部品ごとに名の注の 1 行と中身（末尾の改行を足す）を並べた字。
#[test]
fn stparts_joined_in_name_order() {
    let mut want = String::new();
    for (name, text) in PARTS {
        want.push_str(&format!("/* {name} */\n{text}"));
        if !text.ends_with('\n') {
            want.push('\n');
        }
    }
    assert_eq!(joined(), want);
    assert!(joined().starts_with("/* style/000-about.css */\n/* 面の stylesheet の部品の dir"));
}

/// (3) 基と部品の全部の selector は 1 つの file だけに在る・部品が基の selector を書き直す見本は名指して断る。
#[test]
fn stparts_selectors_owned_once() {
    let base = read("style.css");
    let mut files = vec![("style.css", base.as_str())];
    files.extend(PARTS.iter().copied());
    assert_eq!(owned_once(&files), Ok(()));
    let sels = selectors(&base);
    assert!(sels.len() > 1000, "基の selector が少ない: {}", sels.len());
    for s in [".plk code", ".freshw", "details.nbody > summary"] {
        assert!(sels.iter().any(|x| x == s), "基に {s} が在る");
    }
    let part = ".plk  code { color: red; }\n";
    let bad = [("style.css", base.as_str()), ("style/x.css", part)];
    assert_eq!(
        owned_once(&bad),
        Err("selector .plk code が style.css と style/x.css の両方に在る".to_string())
    );
    let ok = [
        ("style.css", base.as_str()),
        ("style/x.css", ".plk-new code { color: red; }\n"),
    ];
    assert_eq!(owned_once(&ok), Ok(()));
    let media = [
        ("a.css", "@media (max-width: 600px) { .m1, .m2 { x: 1; } }"),
        ("b.css", ".m2 { y: 2; }"),
    ];
    assert_eq!(
        owned_once(&media),
        Err("selector .m2 が a.css と b.css の両方に在る".to_string())
    );
    let frames = [
        ("a.css", "@keyframes k { from { x: 0; } to { x: 1; } }"),
        ("b.css", "@keyframes j { from { x: 0; } }"),
    ];
    assert_eq!(owned_once(&frames), Ok(()));
}

/// (5) wasm の main は面を描く前（2 つの mount の呼びの前）に部品を 1 度だけ入れる。
#[test]
fn stparts_injected_before_mount() {
    let main = read("src/main.rs");
    let body = main
        .split("#[cfg(target_arch = \"wasm32\")]\nfn main() {")
        .nth(1)
        .and_then(|b| b.split("\n}\n").next())
        .expect("wasm の main");
    assert_eq!(
        body.matches("tsuzuri_surface::style::inject();").count(),
        1,
        "{body}"
    );
    let at = body
        .find("tsuzuri_surface::style::inject();")
        .expect("入れる呼び");
    for mount in [
        "tsuzuri_surface::account::board::mount();",
        "tsuzuri_surface::board::mount();",
    ] {
        assert!(
            body.find(mount).is_some_and(|m| at < m),
            "{mount} の前に入れる"
        );
    }
}

/// (6) frame の stylesheet_classes は基と部品をつないだ字から class を集める。
#[test]
fn stparts_frame_reads_parts() {
    let frame = read("tests/teeth2/frame.rs");
    let body = frame
        .split("fn stylesheet_classes() -> BTreeSet<String> {")
        .nth(1)
        .and_then(|b| b.split("\n}\n").next())
        .expect("stylesheet_classes");
    assert_eq!(
        body.matches("tsuzuri_surface::style::joined()").count(),
        1,
        "{body}"
    );
    assert!(body.contains("let css = read(\"style.css\") + &tsuzuri_surface::style::joined();"));
}
