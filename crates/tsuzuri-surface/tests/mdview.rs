//! 本文の書式（Markdown）の一部の読み手の歯（行 g-md-view・接頭辞 mdview_・判断の記録 ADR-30 決定 (4)・規則の行 R-25）。
//! 見出し・箇条・番号つきの箇条・段落・等幅の字を段の列に読み、山括弧の字と描けない形（表の行・囲いの code）を字のまま残し、
//! 部品の file が inner_html の字を持たないことを断言する。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::widgets::md::{Block, FENCE, Inline, parse};

fn text(s: &str) -> Inline {
    Inline::Text(s.to_string())
}

fn code(s: &str) -> Inline {
    Inline::Code(s.to_string())
}

fn para(lines: &[&str]) -> Block {
    Block::Para(lines.iter().map(|l| vec![text(l)]).collect())
}

fn item(depth: u8, ordered: Option<&str>, body: &str) -> Block {
    Block::Item {
        depth,
        ordered: ordered.map(str::to_string),
        body: vec![text(body)],
    }
}

/// 行頭の # が 1〜4 個と空白の行は見出しの段（深さは # の数）。# が 5 個の行と空白の無い行は段落。
#[test]
fn mdview_heads() {
    assert_eq!(
        parse("# a\n## b\n### c\n#### d"),
        [1, 2, 3, 4]
            .into_iter()
            .zip(["a", "b", "c", "d"])
            .map(|(n, t)| Block::Head(n, vec![text(t)]))
            .collect::<Vec<_>>()
    );
    assert_eq!(parse("##### e"), [para(&["##### e"])]);
    assert_eq!(parse("#f"), [para(&["#f"])]);
}

/// 「- 」「* 」の箇条は字下げ 2 字ごとに深さ 1、数字と「. 」「) 」は番号つきの箇条で番号の字を持つ。
#[test]
fn mdview_items_depth() {
    let got = parse("- a\n  - b\n    * c\n1. x\n12) y\n  3. z");
    assert_eq!(
        got,
        [
            item(0, None, "a"),
            item(1, None, "b"),
            item(2, None, "c"),
            item(0, Some("1."), "x"),
            item(0, Some("12)"), "y"),
            item(1, Some("3."), "z"),
        ]
    );
    assert_eq!(parse("-a\n1.x"), [para(&["-a", "1.x"])]);
}

/// 空の行で段落を分け、続く行は 1 つの段落の中の行として残す（行末の \r は外す）。
#[test]
fn mdview_paragraphs() {
    let want = [para(&["a", "b"]), para(&["c"])];
    assert_eq!(parse("a\nb\n\nc"), want);
    assert_eq!(parse("a\r\nb\r\n\r\nc\r\n"), want);
    assert_eq!(
        parse("a\n## h\nb"),
        [para(&["a"]), Block::Head(2, vec![text("h")]), para(&["b"])]
    );
}

/// 逆引用符の対の中は等幅の片、対にならない逆引用符は字のまま。
#[test]
fn mdview_inline_code() {
    assert_eq!(
        parse("x `y` z `w"),
        [Block::Para(vec![vec![
            text("x "),
            code("y"),
            text(" z `w")
        ]])]
    );
    assert_eq!(
        parse("- `a`"),
        [Block::Item {
            depth: 0,
            ordered: None,
            body: vec![code("a")]
        }]
    );
}

/// 山括弧の字は字の片のまま（HTML として読まない）。
#[test]
fn mdview_angle_text() {
    let s = "<k> と <b>太</b> と &amp;";
    assert_eq!(parse(s), [para(&[s])]);
}

/// 「|」で始まる続く行は 1 つの字のままの段、囲いの code は閉じる行まで（閉じなければ末まで）字のままの段。
#[test]
fn mdview_unknown_pre() {
    assert_eq!(FENCE, "```");
    assert_eq!(
        parse("| a | b |\n| - | - |\n\n| c |"),
        [
            Block::Pre("| a | b |\n| - | - |".to_string()),
            Block::Pre("| c |".to_string())
        ]
    );
    assert_eq!(
        parse("```\n# x\n- y\n```\nz"),
        [Block::Pre("```\n# x\n- y\n```".to_string()), para(&["z"])]
    );
    assert_eq!(
        parse("```rust\nfn f() {}"),
        [Block::Pre("```rust\nfn f() {}".to_string())]
    );
}

/// 部品の file は inner_html の字を持たず、DOM（wasm の枝）は字を字の node で出す。
#[test]
fn mdview_no_inner_html() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/widgets/md.rs");
    let src = std::fs::read_to_string(&path).expect("md.rs が読める");
    assert!(!src.contains("inner_html"));
    let dom = &src[src.find("mod dom {").expect("wasm の枝")..];
    for s in [
        "Inline::Text(t) => view! { <span>{t}</span> }",
        "Inline::Code(t) => view! { <code>{t}</code> }",
        "Block::Pre(t) => view! { <pre class=pre>{t}</pre> }",
    ] {
        assert_eq!(dom.matches(s).count(), 1, "{s}");
    }
}
