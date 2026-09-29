//! 行 g-map-tips の歯: 地図の面の 0 件の SRS と pipeline の帯の card（見本の srs0 と run0）と、
//! グラフの数の行と表の行列の数の cell の経験者だけの注釈（見本の data-tip-expert）。
//! card の値は host で組み、DOM は wasm の target のときだけなので、3 つの file の字で付け方を見る。

use std::path::PathBuf;

use tsuzuri_surface::mapview::band::Band;
use tsuzuri_surface::mapview::compact::{zero_card, zero_key};
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::hover::Card;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// file の字 `mod dom {` から後の字（wasm の target のときだけの DOM）。
fn dom_part(rel: &str) -> String {
    let src = read(rel);
    let at = src
        .find("mod dom {")
        .unwrap_or_else(|| panic!("{rel} に mod dom が無い"));
    src[at..].to_string()
}

/// card を持たない 5 つの帯。
const PLAIN: [Band; 5] = [
    Band::Constitution,
    Band::Rules,
    Band::Adr,
    Band::DesignNote,
    Band::Beads,
];

#[test]
fn mtips_zero_keys() {
    assert_eq!(zero_key(Band::Srs), "k:要件");
    assert_eq!(zero_key(Band::Pipeline), "run");
    for band in PLAIN {
        assert_eq!(zero_key(band), band.key(), "{band:?}");
    }
    for band in Band::ALL {
        assert!(
            vocab().term(zero_key(band)).is_some(),
            "{} が語の辞書に無い",
            zero_key(band)
        );
    }
}

#[test]
fn mtips_zero_cards() {
    assert_eq!(
        zero_card(Band::Srs),
        Some(Card {
            title: "requirement 0 本".to_string(),
            kind: "requirement · SRS".to_string(),
            value: "✓ 0 と測れた = 要件の行が無い".to_string(),
            src: "design-intent/srs.yaml".to_string(),
            more: Vec::new(),
        })
    );
    assert_eq!(
        zero_card(Band::Pipeline),
        Some(Card {
            title: "run 0 件".to_string(),
            kind: "run · pipeline".to_string(),
            value: "✓ 0 と測れた = RunCreated が無い".to_string(),
            src: "…/fleet/events.jsonl".to_string(),
            more: vec!["<state dir>/fleet/events.jsonl".to_string()],
        })
    );
    for band in PLAIN {
        assert_eq!(zero_card(band), None, "{band:?}");
    }
}

#[test]
fn mtips_dom_text() {
    let compact = dom_part("src/mapview/compact.rs");
    let span = "<span>{label(zero_key(band))}</span>";
    for w in [
        "Cards::Empty => match zero_card(band) {".to_string(),
        format!("<div class=\"empty\" tabindex=\"0\" use:attach=card>{span}"),
        format!("<div class=\"empty\">{span}"),
    ] {
        assert!(compact.contains(&w), "compact.rs の mod dom に {w} が無い");
    }
    assert!(!compact.contains("{label(band.key())}"));

    let table = dom_part("src/mapview/table.rs");
    assert!(
        table.contains(
            "<td use:expert_tip=c.types_text()>{pair_button(c, search, \"\")}</td>"
        ),
        "table.rs の行列の cell に経験者の注釈が無い"
    );
    assert!(!table.contains("title="), "table.rs の mod dom に title= が在る");

    let graph = read("src/mapview/graph/dom.rs");
    assert!(graph.contains(
        "<div class=\"cutline num\" tabindex=\"0\" data-term=\"cut\" use:expert_tip=line>{count}</div>"
    ));
    assert!(graph.contains("let line = expert_line(&v);"));
    for w in ["expert_row", "shows_internal"] {
        assert!(!graph.contains(w), "graph/dom.rs に {w} が在る");
    }
}
