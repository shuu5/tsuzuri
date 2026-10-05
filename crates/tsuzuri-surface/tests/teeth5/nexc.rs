//! 個別の頁の概要の箱に本文の頭の 1 行を出す歯（行 g-node-excerpt・接頭辞 nexc_・判断の記録 ADR-30 決定 (2)）。
//! 1 本の引きの電文から本文の頭の 1 行の 3 値を読み、2 つの概要が両方無い時だけ「本文から」の印つきで箱に出すことを断言し、
//! 本文と記録の block と同じ 1 本の引きの読みを使う DOM（wasm の枝）の配線の字を照らす。
#![cfg(test)]

use crate::common::{ID, bead, read, span};
use tsuzuri_contract::graph::{AroundDoc, NodeKind};
use tsuzuri_contract::ledger::{LedgerItem, LedgerRow};
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::node::{
    self, Excerpt, NO_SUMMARY, SUMMARY_MARKED, SUMMARY_NONE, excerpt_of, summary_in,
};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::sumpick::{BODY_KEY, ENG_KEY, PLAIN_KEY};

/// file の wasm の枝の字。
fn dom(rel: &str) -> String {
    let src = read(rel);
    src[src.find("mod dom {").expect("wasm の枝")..].to_string()
}

/// 中心の節点を memo の bead にし、2 つの概要を替えた近傍の電文（fixture は着地した歯 gsum と同じ file）。
fn around(plain: Option<&str>, eng: Option<&str>) -> AroundDoc {
    let mut doc: AroundDoc = wire::decode(&read("../../tests/fixtures/surface/around-doc.json"))
        .expect("around-doc.json が電文として読める");
    let c = doc
        .rows
        .iter_mut()
        .find(|r| r.col == 0)
        .expect("fixture の中心の行");
    c.node.kind = NodeKind::Memo;
    c.node.id = ID.to_string();
    c.node.plain = plain.map(str::to_string);
    c.node.eng = eng.map(str::to_string);
    doc
}

/// 1 本の引きの電文（id と本文だけを替える）。
fn item(id: &str, description: &str) -> Fetched {
    let it = LedgerItem {
        row: LedgerRow {
            id: bead(id),
            kind: "task".to_string(),
            title: format!("{id} の題"),
            status: "open".to_string(),
            updated_at: 1_790_000_000,
            parent: None,
            labels: Vec::new(),
        },
        description: description.to_string(),
        notes: String::new(),
    };
    Fetched::Body(wire::encode(&it).expect("電文に書ける"))
}

const BODY: &str = "## 見出し\n\n本文の頭の行\n次の行";

#[test]
fn nexc_excerpt_states() {
    let b = bead(ID);
    assert_eq!(
        excerpt_of(&item(ID, BODY), Some(&b)),
        Excerpt::Read(Some("本文の頭の行".to_string()))
    );
    assert_eq!(
        excerpt_of(&item(ID, "# 見出しだけ\n"), Some(&b)),
        Excerpt::Read(None)
    );
    assert_eq!(
        excerpt_of(&item(ID, BODY), None),
        Excerpt::Never,
        "bead でない節点は読まない"
    );
    for f in [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("{".to_string()),
        item("t-9", BODY),
    ] {
        assert_eq!(excerpt_of(&f, Some(&b)), Excerpt::Unread, "{f:?}");
    }
}

#[test]
fn nexc_box_body_line() {
    let line = Excerpt::Read(Some("本文の頭の行".to_string()));
    let doc = around(None, None);
    let c = node::center(&doc).expect("中心の行");
    for mode in [Mode::Beginner, Mode::Expert] {
        let b = summary_in(c, mode, &line);
        assert_eq!(
            (b.key, b.class, b.text.as_str()),
            (BODY_KEY, SUMMARY_MARKED, "本文の頭の行"),
            "{mode:?}"
        );
        assert_eq!(b.other, None);
    }
    assert_eq!(vocab().label(BODY_KEY), "本文から");
    // 表示の型の側の概要が在れば本文の頭の 1 行は出さない（見本の違いは非エンジニア向けの概要の有無だけ）。
    let doc = around(Some("平の概要"), None);
    let c = node::center(&doc).expect("中心の行");
    let b = summary_in(c, Mode::Beginner, &line);
    assert_eq!(
        (b.key, b.class, b.text.as_str()),
        (PLAIN_KEY, "sumbox", "平の概要")
    );
    // もう一方の概要だけが在ればもう一方を印つきで出す（本文の頭の 1 行より先）。
    let b = summary_in(c, Mode::Expert, &line);
    assert_eq!(
        (b.key, b.class, b.text.as_str()),
        (PLAIN_KEY, SUMMARY_MARKED, "平の概要")
    );
}

#[test]
fn nexc_box_unread_none() {
    let doc = around(None, None);
    let c = node::center(&doc).expect("中心の行");
    let unread = summary_in(c, Mode::Beginner, &Excerpt::Unread);
    assert_eq!(
        (unread.key, unread.class, unread.text.as_str()),
        (PLAIN_KEY, SUMMARY_NONE, "まだ分からない"),
        "本文を読めていない間は要約なしと見せない"
    );
    for body in [Excerpt::Read(None), Excerpt::Never] {
        let b = summary_in(c, Mode::Expert, &body);
        assert_eq!(
            (b.key, b.class, b.text.as_str()),
            (ENG_KEY, SUMMARY_NONE, NO_SUMMARY),
            "{body:?}"
        );
    }
    assert_eq!(
        node::summary(c, Mode::Beginner),
        summary_in(c, Mode::Beginner, &Excerpt::Never)
    );
}

#[test]
fn nexc_one_item_read() {
    let node_dom = dom("src/project/node.rs");
    for want in [
        "let body = item_source();",
        "s.item.with(|(f, _)| excerpt_of(f, b.as_ref()))",
        "let boxes = sum_view(summary_in(c, mode(), &excerpt));",
    ] {
        assert_eq!(
            node_dom.matches(want).count(),
            1,
            "node.rs の DOM の {want}"
        );
    }
    assert_eq!(
        node_dom.matches("crate::net::read_path(").count(),
        1,
        "節点の block の 1 本の引きは決定の頁の問いの 1 つだけ"
    );
    let body_dom = dom("src/project/nodebody.rs");
    assert_eq!(body_dom.matches("crate::net::read_path(").count(), 1);
    let source = span(&body_dom, "pub fn item_source()", "\n    }\n");
    for want in [
        "let item = crate::net::read_path(path);",
        "if let Some(s) = ITEM.get().filter(|s| s.near == near) {",
        "ITEM.set(Some(s));",
    ] {
        assert!(source.contains(want), "item_source に {want} が無い");
    }
    assert!(
        body_dom.contains("let Some(ItemSource { bead, item, .. }) = item_source() else {"),
        "本文と記録の block も item_source の読みを使う"
    );
}
