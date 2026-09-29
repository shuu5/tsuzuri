//! 行 g-dn-group の歯: 地図の圧縮の面の design-note の帯を、ノートごとの見出し（ノートの名の自然な順）の下に
//! ノートの行の札（行の番号の順・番号の無い行は後に id の自然な順）を並べる組に分けること。
//! 組の値は host で組み、DOM は wasm の target のときだけなので、compact.rs の字で見出しの組み方を見る。

use std::path::PathBuf;

use tsuzuri_contract::graph::{GraphDoc, GraphNode, NodeKind};
use tsuzuri_contract::wire;
use tsuzuri_surface::mapview::band::Band;
use tsuzuri_surface::mapview::compact::{BandBox, Cards, NoteGroup, compact, note_of};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture() -> GraphDoc {
    wire::decode(&read("../../tests/fixtures/surface/graph-doc.json"))
        .expect("fixture が電文として読める")
}

/// design-note の帯の箱。
fn dn_box(doc: &GraphDoc) -> BandBox {
    compact(doc)
        .into_iter()
        .find(|b| b.band == Band::DesignNote)
        .expect("design-note の帯の箱")
}

/// 組の名と札の id。
fn shape(groups: &[NoteGroup]) -> Vec<(&str, Vec<&str>)> {
    groups
        .iter()
        .map(|g| {
            (
                g.note.as_str(),
                g.tags.iter().map(|t| t.id.as_str()).collect(),
            )
        })
        .collect()
}

/// file の字 `mod dom {` から後の字（wasm の target のときだけの DOM）。
fn dom_part(rel: &str) -> String {
    let src = read(rel);
    let at = src
        .find("mod dom {")
        .unwrap_or_else(|| panic!("{rel} に mod dom が無い"));
    src[at..].to_string()
}

/// (1) ノートの名は最初の「#」の前の字（「#」が無ければ字の全部）。
#[test]
fn dngrp_note_of() {
    for (id, want) in [
        ("surface-board#g-map", "surface-board"),
        ("a#b#c", "a"),
        ("surface-board", "surface-board"),
        ("#x", ""),
    ] {
        assert_eq!(note_of(id), want, "{id:?}");
    }
}

/// (2) fixture の design-note の帯は 1 つのノートの組。
#[test]
fn dngrp_fixture_one_note() {
    let b = dn_box(&fixture());
    assert_eq!(b.count, Some(2));
    let Cards::Notes(groups) = &b.cards else {
        panic!("design-note の帯が Notes でない: {:?}", b.cards);
    };
    assert_eq!(
        shape(groups),
        vec![(
            "surface-board",
            vec!["surface-board#g-frame", "surface-board#g-map"]
        )]
    );
    assert_eq!(
        b.ids(),
        vec!["surface-board#g-frame", "surface-board#g-map"]
    );
}

/// (3) 組はノートの名の自然な順・組の中は行の番号の順で、番号の無い行は後に id の自然な順。
#[test]
fn dngrp_notes_natural_rows_by_line() {
    let mut doc = fixture();
    let proto: GraphNode = doc
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::NoteRow)
        .expect("fixture の設計ノートの行")
        .clone();
    doc.nodes.retain(|n| n.kind != NodeKind::NoteRow);
    for (id, line) in [
        ("surface-wave10a#b", Some(12)),
        ("surface-wave9#z", Some(7)),
        ("surface-wave10a#a", Some(40)),
        ("surface-wave9#y", None),
        ("surface-base#row10", None),
        ("surface-base#row9", None),
        ("surface-wave10a#c", None),
    ] {
        let mut n = proto.clone();
        n.id = id.to_string();
        n.title = format!("{id} の題");
        n.line = line;
        doc.nodes.push(n);
    }
    let b = dn_box(&doc);
    assert_eq!(b.count, Some(7));
    let Cards::Notes(groups) = &b.cards else {
        panic!("design-note の帯が Notes でない: {:?}", b.cards);
    };
    assert_eq!(
        shape(groups),
        vec![
            (
                "surface-base",
                vec!["surface-base#row9", "surface-base#row10"]
            ),
            ("surface-wave9", vec!["surface-wave9#z", "surface-wave9#y"]),
            (
                "surface-wave10a",
                vec!["surface-wave10a#b", "surface-wave10a#a", "surface-wave10a#c"]
            ),
        ]
    );
    assert_eq!(
        b.ids(),
        vec![
            "surface-base#row9",
            "surface-base#row10",
            "surface-wave9#z",
            "surface-wave9#y",
            "surface-wave10a#b",
            "surface-wave10a#a",
            "surface-wave10a#c",
        ]
    );
}

/// (4) mod dom の Notes の枝は見出し（印・ノートの名・札の数）と札を class cards7 の div に入れる。
#[test]
fn dngrp_dom_text() {
    let dom = dom_part("src/mapview/compact.rs");
    let at = dom
        .find("Cards::Notes(notes) => {")
        .expect("compact.rs の mod dom に Notes の枝が無い");
    let arm = &dom[at..];
    for w in [
        "<div class=\"subh\">",
        "<span class=shape.clone() aria-hidden=\"true\"></span>",
        "<span class=\"mono\">{g.note}</span>",
        "<span class=\"num muted\">{g.tags.len()}</span>",
        "tag_view(t, mode)",
        "<div class=\"cards7\">{notes}</div>",
    ] {
        assert!(arm.contains(w), "Notes の枝に {w} が無い");
    }
}
