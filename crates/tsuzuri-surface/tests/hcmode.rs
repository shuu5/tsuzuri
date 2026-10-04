//! hover の card の要約を表示の型の 1 つにする歯（行 g-card-mode・接頭辞 hcmode_・判断の記録 ADR-30 決定 (2)）。
//! card の値は host の純な関数で撃ち、DOM の呼び手（wasm の枝）は字を読む。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::graph::{GraphNode, NodeKind};
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::nodecard::{card_for, card_in, gist_in};
use tsuzuri_surface::widgets::sumpick::{ENG_KEY, PLAIN_KEY};

const P: &str = "P字やさしい";
const E: &str = "E字かたい";

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn node(plain: Option<&str>, eng: Option<&str>) -> GraphNode {
    GraphNode {
        id: "FR1".to_string(),
        kind: NodeKind::Req,
        file: Some("design-intent/srs.yaml".to_string()),
        digest: None,
        title: "題".to_string(),
        line: Some(3),
        plain: plain.map(str::to_string),
        eng: eng.map(str::to_string),
        updated: None,
    }
}

/// (1) card の値の行と詳しくの概要の字は、初心者は非エンジニア向け・経験者はエンジニア向けから作り、もう一方の字を含まない。
#[test]
fn hcmode_value_by_mode() {
    let long_p = format!("{P}、{}", "あ".repeat(30));
    let long_e = format!("{E}、{}", "い".repeat(30));
    let n = node(Some(&long_p), Some(&long_e));
    let b = card_in(&n, None, Mode::Beginner);
    assert_eq!(b.value, format!("FR1 {P}、{}…", "あ".repeat(22)));
    assert!(b.more.iter().any(|r| r.starts_with(P)));
    let e = card_in(&n, None, Mode::Expert);
    assert_eq!(e.value, format!("FR1 {E}、{}…", "い".repeat(23)));
    assert!(e.more.iter().any(|r| r.starts_with(E)));
    for (c, other) in [(&b, "E字"), (&e, "P字")] {
        assert!(!c.value.contains(other));
        assert!(c.more.iter().all(|r| !r.contains(other)), "{:?}", c.more);
    }
}

/// (2) 表示の型の側が無い時はもう一方の字を出し、詳しくの頭にその語の鍵の語を置く。表示の型の側が在れば印の行は無い。
#[test]
fn hcmode_mark_row() {
    let only_e = node(None, Some(E));
    let b = card_in(&only_e, None, Mode::Beginner);
    assert_eq!(b.value, format!("FR1 {E}"));
    assert_eq!(b.more.first(), Some(&vocab().label(ENG_KEY)));
    let only_p = node(Some(P), None);
    let e = card_in(&only_p, None, Mode::Expert);
    assert_eq!(e.more.first(), Some(&vocab().label(PLAIN_KEY)));
    let both = node(Some(P), Some(E));
    for mode in Mode::ALL {
        let c = card_in(&both, None, mode);
        assert!(!c.more.contains(&vocab().label(ENG_KEY)), "{mode:?}");
        assert!(!c.more.contains(&vocab().label(PLAIN_KEY)), "{mode:?}");
    }
    assert_eq!(
        gist_in(&only_e, Mode::Beginner).map(|p| p.marked),
        Some(true)
    );
}

/// (3) 表示の型を受けない card_for は初心者の card_in と同じ値（着地した呼び手と歯の包み）。
#[test]
fn hcmode_beginner_wrapper() {
    for (p, e) in [
        (Some(P), Some(E)),
        (None, Some(E)),
        (Some(P), None),
        (None, None),
    ] {
        let n = node(p, e);
        assert_eq!(
            card_for(&n, Some("open")),
            card_in(&n, Some("open"), Mode::Beginner)
        );
    }
}

/// (4) DOM の呼び手（近傍の図と一覧・走行の行・決定の歴・抜けの検査）は表示の型の版を呼ぶ。
#[test]
fn hcmode_callers_pass_mode() {
    for (file, want) in [
        (
            "src/project/nodearound.rs",
            "let cards = view_cards_in(&gv.nodes, mode());",
        ),
        (
            "src/project/timeline.rs",
            "let card = run_card_in(&around, &r.run, mode());",
        ),
        (
            "src/project/askpage.rs",
            "hist_cards_in(&entries, g, mode())",
        ),
        (
            "src/project/gaps.rs",
            "fetched.with(|f| body_in(f, mode()))",
        ),
    ] {
        let text = read(file);
        let dom = &text[text
            .find("mod dom {")
            .or_else(|| text.find("pub fn inner("))
            .expect("DOM の部分")..];
        assert!(dom.contains(want), "{file} の DOM に {want} が無い");
    }
}
