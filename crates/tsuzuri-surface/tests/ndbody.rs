//! 個別の頁の本文と記録の block の歯（行 g-node-body・接頭辞 ndbody_・判断の記録 ADR-30 決定 (4)・要件 FR5）。
//! 頁の block の並びと畳める段の鍵と初めの閉じ、読む bead の決め方（台帳の bead の種類だけ）、1 本の引きの電文の本文と記録の
//! 段の列と、読めない間の測れていないの 1 行を断言し、DOM（wasm の枝）の配線の字を block の file の範囲で照らす。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::graph::{AroundDoc, NodeKind};
use tsuzuri_contract::ledger::{BeadId, LedgerItem, LedgerRow};
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::{self, Block, PageId};
use tsuzuri_surface::project::nodearound::PageState;
use tsuzuri_surface::project::nodebody::{
    self, BLANK, BLOCK, FOLDS, PART_CLASS, PART_KEYS, REASON, Texts, bead_of, item_path, kept_bead,
    texts,
};
use tsuzuri_surface::project::{Body, NO_CONTENT, NOT_READ};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::md;

const ID: &str = "t-1.2";

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// block の file の DOM（wasm の枝）の字。
fn dom() -> String {
    let src = read("src/project/nodebody.rs");
    src[src.find("mod dom {").expect("wasm の枝")..].to_string()
}

fn bead(s: &str) -> BeadId {
    BeadId::new(s).expect("bead の id")
}

/// 中心の節点の種類と id を替えた近傍の電文（fixture は着地した歯 gsum と同じ file）。
fn around(kind: NodeKind, id: &str) -> AroundDoc {
    let mut doc: AroundDoc = wire::decode(&read("../../tests/fixtures/surface/around-doc.json"))
        .expect("around-doc.json が電文として読める");
    let c = doc
        .rows
        .iter_mut()
        .find(|r| r.col == 0)
        .expect("fixture の中心の行");
    c.node.kind = kind;
    c.node.id = id.to_string();
    doc
}

fn center_bead(kind: NodeKind, id: &str) -> Option<BeadId> {
    let doc = around(kind, id);
    let c = doc.rows.iter().find(|r| r.col == 0).expect("中心の行");
    bead_of(c)
}

/// 1 本の引きの電文（id と本文と記録だけを替える）。
fn item(id: &str, description: &str, notes: &str) -> Fetched {
    let row = LedgerRow {
        id: bead(id),
        kind: "task".to_string(),
        title: format!("{id} の題"),
        status: "open".to_string(),
        updated_at: 1_790_000_000,
        parent: None,
        labels: Vec::new(),
    };
    let it = LedgerItem {
        row,
        description: description.to_string(),
        notes: notes.to_string(),
    };
    Fetched::Body(wire::encode(&it).expect("電文に書ける"))
}

/// 節点の頁の block の並びは node・around・body・timeline。本文と記録の block は id body・見出しの鍵 nb_body・class panel で、
/// 見出しと 2 つの段の語は語の辞書に在る。
#[test]
fn ndbody_page_order() {
    assert_eq!(
        frame::page(PageId::Node).block_ids(),
        vec!["node", "around", "body", "timeline"]
    );
    assert_eq!(
        BLOCK,
        Block {
            id: "body",
            heading: "nb_body",
            class: "panel",
        }
    );
    assert_eq!(PART_KEYS, ["nb_text", "nb_notes"]);
    for (k, want) in [
        ("nb_body", "本文と記録"),
        ("nb_text", "本文"),
        ("nb_notes", "記録"),
    ] {
        let t = vocab()
            .term(k)
            .unwrap_or_else(|| panic!("{k} が語の辞書に無い"));
        assert_eq!(t.label, want, "{k}");
        assert!(!t.note.is_empty(), "{k}");
    }
}

/// 畳める段は鍵 node:body と node:notes の 2 つで、どちらも表示の型によらず、初めの値はその browser の保存の値
/// （無ければ閉じ・行 g-fold-keep）で、開き閉じを畳める段の記録に書き戻す。段の class は stylesheet に在る。
#[test]
fn ndbody_folds_closed() {
    assert_eq!(FOLDS, ["node:body", "node:notes"]);
    let dom = dom();
    for key in FOLDS {
        let call = format!("fold(\"{key}\".to_string(), || store::fold_open(\"{key}\"));");
        assert_eq!(dom.matches(&call).count(), 1, "{call}");
    }
    assert_eq!(
        dom.matches("<details class=PART_CLASS prop:open=open_")
            .count(),
        2
    );
    assert_eq!(dom.matches("on:toggle=toggle_").count(), 2);
    assert!(!dom.contains("mode"), "段の開き閉じが表示の型を読む");
    assert_eq!(PART_CLASS, "fold nbody");
    assert!(read("style.css").contains("details.nbody { "));
}

/// 中心の節点が台帳の bead の種類（契約・epic・memo・問い）で id が bead の id の形の時だけ bead を返し、設計の節点・
/// 走行・決定は同じ id でも None。読む口の path は 1 本の引きの口。近傍の読み直しの間は前の bead を保つ。
#[test]
fn ndbody_reads_bead_only() {
    for kind in [
        NodeKind::Task,
        NodeKind::Epic,
        NodeKind::Memo,
        NodeKind::Question,
    ] {
        assert_eq!(center_bead(kind, ID), Some(bead(ID)), "{kind:?}");
    }
    for kind in [
        NodeKind::Run,
        NodeKind::NoteRow,
        NodeKind::Req,
        NodeKind::Adr,
        NodeKind::Ruling,
    ] {
        assert_eq!(center_bead(kind, ID), None, "{kind:?}");
    }
    assert_eq!(center_bead(NodeKind::Task, "contracts/n.toml#r-a"), None);
    assert_eq!(item_path(&bead(ID)), "/api/ledger/t-1.2");
    let before = Some(bead("t-9"));
    let doc = PageState::Doc(around(NodeKind::Memo, ID));
    assert_eq!(kept_bead(before.clone(), &doc), Some(bead(ID)));
    for st in [PageState::NotRead, PageState::Unread(NOT_READ)] {
        assert_eq!(kept_bead(before.clone(), &st), before);
    }
    assert_eq!(kept_bead(before, &PageState::NotFound), None);
    let dom = dom();
    for s in [
        "bead.with(|b| b.as_ref().map(item_path).unwrap_or_default())",
        "let item = crate::net::read_path(path);",
        "let Some(b) = bead.get() else {\n                return ().into_any();",
    ] {
        assert_eq!(dom.matches(s).count(), 1, "{s}");
    }
}

/// 読めた電文の本文と記録は部品 md の parse の段の列で、DOM は段を台帳の字の印の下に md::view で描き、空の段は 1 行。
#[test]
fn ndbody_body_parsed() {
    let desc = "## 概要\n- a `b`\n\n本文の行";
    let notes = "記録の行\n\n| x |";
    let got = texts(&item(ID, desc, notes), &bead(ID));
    assert_eq!(
        got,
        Body::Filled(Texts {
            body: md::parse(desc),
            notes: md::parse(notes),
        })
    );
    let Body::Filled(t) = got else {
        panic!("中身が無い");
    };
    assert_eq!((t.body.len(), t.notes.len()), (3, 2));
    let empty = texts(&item(ID, desc, ""), &bead(ID));
    assert!(matches!(empty, Body::Filled(Texts { notes, .. }) if notes.is_empty()));
    let dom = dom();
    assert_eq!(
        dom.matches("<div data-ledger-text=\"\">{md::view(blocks)}</div>")
            .count(),
        1
    );
    assert_eq!(dom.matches("<span>{BLANK}</span>").count(), 1);
    assert_eq!(BLANK, "書かれていない");
    assert_eq!(dom.matches("{part(t.body)}").count(), 1);
    assert_eq!(dom.matches("{part(t.notes)}").count(), 1);
}

/// 引きが読めない間は測れていないの 1 行: まだ読んでいない・口が読めない・電文が読めない・ほかの bead の電文。
#[test]
fn ndbody_unread_line() {
    let b = bead(ID);
    assert!(matches!(texts(&item(ID, "x", "y"), &b), Body::Filled(_)));
    for (f, why) in [
        (Fetched::NotRead, NOT_READ),
        (Fetched::Failed, REASON),
        (Fetched::Body("{".to_string()), NO_CONTENT),
        (item("t-9", "x", "y"), NOT_READ),
    ] {
        assert_eq!(texts(&f, &b), Body::Unmeasured(why), "{f:?}");
    }
    assert!(REASON.contains("読めない"));
    assert_eq!(
        dom()
            .matches(
                "Body::Unmeasured(reason) => section(BLOCK, ().into_any(), unmeasured(reason)),"
            )
            .count(),
        1
    );
    assert_eq!(nodebody::PATHS, [] as [&str; 0]);
}
