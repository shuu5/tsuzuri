//! 行 g-map-tree の歯: 地図の 6 つの面と URL・木の面の語彙の予算・親子の辺の型の表・fixture の設計の木と台帳の木・
//! 項の値と開き閉じの鍵・木の親子にしない辺・読めない出所と空の電文・DOM の字（wasm の target のときだけなので src の字で見る）・
//! この file の歯の名。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::graph::{
    EdgeEnd, EdgeType, GraphDoc, GraphEdge, GraphSource, NodeKind, basis_end, title36,
};
use tsuzuri_contract::wire;
use tsuzuri_surface::kit::Item;
use tsuzuri_surface::mapview::band::{Band, unread_reason};
use tsuzuri_surface::mapview::tree::{
    Branch, Forest, Head, ISSUE_KINDS, NESTS, Tree, fold_key, forest, is_closed, parent_end,
};
use tsuzuri_surface::mapview::{View, open_question, shape_class, with_view};
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::help::{Inline, note};
use tsuzuri_surface::widgets::nodecard::card_of;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture() -> GraphDoc {
    wire::decode(&read("../../tests/fixtures/surface/graph-doc.json"))
        .expect("fixture が電文として読める")
}

fn edge(from: &str, to: &str, edge_type: EdgeType) -> GraphEdge {
    GraphEdge {
        from: from.to_string(),
        to: to.to_string(),
        edge_type,
    }
}

/// fixture に辺を足した電文。
fn with_edges(edges: &[(&str, &str, EdgeType)]) -> GraphDoc {
    let mut doc = fixture();
    doc.edges
        .extend(edges.iter().map(|(f, t, e)| edge(f, t, *e)));
    doc
}

/// AC-3 から FR14 の verifies と FR2 から AC-3 の verify.ac を足した電文。
fn with_ac() -> GraphDoc {
    with_edges(&[
        ("AC-3", "FR14", EdgeType::Verifies),
        ("FR2", "AC-3", EdgeType::VerifyAc),
    ])
}

/// 項の名（帯は band:名:数・設計ノートは note:名:数・節点は id）。
fn name(head: &Head) -> String {
    match head {
        Head::Band { band, count } => format!("band:{}:{count}", band.name()),
        Head::Note { note, count } => format!("note:{note}:{count}"),
        Head::Retired(count) => format!("retired:{count}"),
        Head::Node(item) => item.id.clone(),
    }
}

/// 項を前から順にたどった (深さ・名・open・kids の数) の列。
fn outline(items: &[Branch]) -> Vec<(usize, String, bool, usize)> {
    fn go(items: &[Branch], depth: usize, out: &mut Vec<(usize, String, bool, usize)>) {
        for b in items {
            out.push((depth, name(&b.head), b.open, b.kids.len()));
            go(&b.kids, depth + 1, out);
        }
    }
    let mut out = Vec::new();
    go(items, 0, &mut out);
    out
}

fn rows(expected: &[(usize, &str, bool, usize)]) -> Vec<(usize, String, bool, usize)> {
    expected
        .iter()
        .map(|(d, n, o, k)| (*d, n.to_string(), *o, *k))
        .collect()
}

/// 全部の項（前から順）。
fn all(items: &[Branch]) -> Vec<&Branch> {
    fn go<'a>(items: &'a [Branch], out: &mut Vec<&'a Branch>) {
        for b in items {
            out.push(b);
            go(&b.kids, out);
        }
    }
    let mut out = Vec::new();
    go(items, &mut out);
    out
}

fn item<'a>(f: &'a Forest, id: &str) -> &'a Item {
    all(&f.items)
        .into_iter()
        .find_map(|b| match &b.head {
            Head::Node(i) if i.id == id => Some(i),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{id} の項が在る"))
}

fn branch<'a>(f: &'a Forest, id: &str) -> &'a Branch {
    all(&f.items)
        .into_iter()
        .find(|b| matches!(&b.head, Head::Node(i) if i.id == id))
        .unwrap_or_else(|| panic!("{id} の項が在る"))
}

const DESIGN_REST: &[(usize, &str, bool, usize)] = &[
    (0, "band:rules:3", true, 3),
    (1, "D-3", false, 0),
    (1, "R-4", false, 0),
    (1, "R-25", false, 0),
    (0, "band:ADR:3", true, 3),
    (1, "ADR-2", false, 0),
    (1, "ADR-7", false, 0),
    (1, "ADR-10", false, 0),
];

const DESIGN_CONSTITUTION: &[(usize, &str, bool, usize)] = &[
    (0, "band:constitution:9", true, 4),
    (1, "A-2", true, 1),
    (2, "A-2.1", false, 0),
    (1, "N-1", true, 2),
    (2, "N-1.2", false, 0),
    (2, "N-1.10", false, 0),
    (1, "P-1", true, 2),
    (2, "P-1.1", false, 0),
    (2, "P-1.2", false, 0),
    (1, "P-9.1", false, 0),
];

const DESIGN_NOTE: &[(usize, &str, bool, usize)] = &[
    (0, "band:design-note:2", true, 1),
    (1, "note:surface-board:2", true, 2),
    (2, "surface-board#g-frame", false, 0),
    (2, "surface-board#g-map", false, 0),
];

fn design_outline(srs: &[(usize, &str, bool, usize)]) -> Vec<(usize, String, bool, usize)> {
    [DESIGN_CONSTITUTION, DESIGN_REST, srs, DESIGN_NOTE]
        .into_iter()
        .flat_map(rows)
        .collect()
}

const LEDGER: &[(usize, &str, bool, usize)] = &[
    (0, "t3", true, 4),
    (1, "t3-hub.2", false, 1),
    (2, "r-1", false, 0),
    (1, "t3-hub.9", false, 0),
    (1, "t3-hub.15", true, 2),
    (2, "r-3", false, 0),
    (2, "r-12", false, 0),
    (1, "t3.q10", false, 0),
    (0, "t3.m1", false, 0),
    (0, "t3.q2", false, 1),
    (1, "t3.q2#r", false, 0),
    (0, "t3.p", false, 0),
];

#[test]
fn mtree_views_six_in_url() {
    use View::{Compact, Design, Graph, Ledger, List, Table};
    assert_eq!(View::ALL, [Compact, List, Graph, Table, Design, Ledger]);
    let names: Vec<&str> = View::ALL.iter().map(|v| v.name()).collect();
    assert_eq!(
        names,
        ["compact", "list", "graph", "table", "design", "ledger"]
    );
    let keys: Vec<&str> = View::ALL.iter().map(|v| v.key()).collect();
    assert_eq!(
        keys,
        [
            "view_compact",
            "view_list",
            "view_graph",
            "view_table",
            "view_design",
            "view_ledger"
        ]
    );
    assert_eq!(View::from_query("?page=map&view=design"), Design);
    assert_eq!(View::from_query("?view=ledger"), Ledger);
    for s in ["?view=tree", "?view=", ""] {
        assert_eq!(View::from_query(s), Compact, "{s:?}");
    }
    assert_eq!(
        with_view("?page=map&mode=expert", Design),
        "?page=map&mode=expert&view=design"
    );
    for v in View::ALL {
        assert_eq!(View::from_query(&with_view("?page=map", v)), v, "{v:?}");
    }
    assert_eq!(Tree::ALL, [Tree::Design, Tree::Ledger]);
    assert_eq!(Tree::Design.view(), Design);
    assert_eq!(Tree::Ledger.view(), Ledger);
    assert_eq!(
        Tree::Design.bands(),
        [
            Band::Constitution,
            Band::Rules,
            Band::Adr,
            Band::Srs,
            Band::DesignNote
        ]
    );
    assert_eq!(Tree::Ledger.bands(), [Band::Beads, Band::Pipeline]);
    let joined: Vec<Band> = Tree::ALL
        .iter()
        .flat_map(|t| t.bands().iter().copied())
        .collect();
    assert_eq!(joined, Band::ALL);
}

fn text(parts: &[Inline]) -> String {
    parts
        .iter()
        .map(|p| match p {
            Inline::Text(s) | Inline::Code(s) | Inline::Sym(s) => s.as_str(),
        })
        .collect()
}

fn no_sym(parts: &[Inline], what: &str) {
    assert!(
        !parts.iter().any(|p| matches!(p, Inline::Sym(_))),
        "{what} に記号の見本が在る"
    );
}

#[test]
fn mtree_vocab_budget() {
    let v = vocab();
    let term = |k: &str| v.term(k).unwrap_or_else(|| panic!("鍵 {k} が在る"));
    let design = term("view_design");
    assert_eq!(design.label, "設計の木");
    assert_eq!(
        design.note,
        [
            "設計の文書を入れ子の一覧で見る",
            "・ 帯 → 条 → 規範文",
            "・ 要件 → 受入基準",
            "・ 設計ノート → 行",
            "→ 押すと項目の頁"
        ]
        .join("\n")
    );
    assert_eq!(
        design.internal,
        "view=design\n親子 = in-article、verify.ac、verifies"
    );
    let ledger = term("view_ledger");
    assert_eq!(ledger.label, "台帳の木");
    assert_eq!(
        ledger.note,
        [
            "台帳の項目を入れ子の一覧で見る",
            "・ epic → task → run",
            "・ question → あなたの決定",
            "・ 閉じた項目は畳んでおく",
            "→ 押すと項目の頁"
        ]
        .join("\n")
    );
    assert_eq!(
        ledger.internal,
        "view=ledger\n親子 = parent-child、run_of、answers"
    );
    let views = term("views");
    assert_eq!(views.label, "眺め");
    assert_eq!(views.note, "同じ中身を 6 つの形で見る\n・ 切り替えは URL に残る");
    assert_eq!(
        views.internal,
        "?view=compact|list|graph|table|design|ledger"
    );
    let labels: Vec<String> = View::ALL.iter().map(|x| term(x.key()).label.clone()).collect();
    for (i, l) in labels.iter().enumerate() {
        assert!(!l.is_empty(), "{:?} の語が空", View::ALL[i]);
        assert!(
            !labels[..i].contains(l),
            "{:?} の語 {l} が前の面と同じ",
            View::ALL[i]
        );
    }
    for key in ["view_design", "view_ledger", "views"] {
        let n = note(key).unwrap_or_else(|| panic!("{key} の注釈"));
        assert!(text(&n.head).chars().count() <= 40, "{key} の 1 行目");
        assert!(n.items.len() <= 4, "{key} の項の数");
        for it in &n.items {
            assert!(text(&it.text).chars().count() <= 20, "{key} の項 {:?}", it.text);
            no_sym(&it.sym, key);
            no_sym(&it.text, key);
        }
        assert!(n.more.is_empty(), "{key} の詳しく");
        no_sym(&n.head, key);
        for l in &n.internal {
            no_sym(l, key);
        }
        let t = term(key);
        for line in t.note.split('\n').chain(t.internal.split('\n')) {
            assert!(line.matches('・').count() < 2, "{key} の行 {line}");
            assert!(!line.contains(['（', '(']), "{key} の行 {line}");
        }
    }
}

#[test]
fn mtree_nests_table_closed() {
    use NodeKind::*;
    assert_eq!(ISSUE_KINDS, [Epic, Task, Memo, Question]);
    let table: Vec<(EdgeType, Vec<NodeKind>, Vec<NodeKind>)> = NESTS
        .iter()
        .map(|n| (n.edge_type, n.parents.to_vec(), n.children.to_vec()))
        .collect();
    let issues = ISSUE_KINDS.to_vec();
    assert_eq!(
        table,
        [
            (EdgeType::InArticle, vec![Article], vec![Norm]),
            (EdgeType::VerifyAc, vec![Req, Nfr], vec![Ac]),
            (EdgeType::Verifies, vec![Req, Nfr], vec![Ac]),
            (EdgeType::ParentChild, issues.clone(), issues.clone()),
            (EdgeType::Answers, vec![Question], vec![Ruling]),
            (EdgeType::RunOf, issues, vec![Run]),
        ]
    );
    let cases = [
        (EdgeType::InArticle, Norm, Article, Some(EdgeEnd::To)),
        (EdgeType::InArticle, Article, Norm, Some(EdgeEnd::From)),
        (EdgeType::InArticle, Article, Article, None),
        (EdgeType::Verifies, Ac, Req, Some(EdgeEnd::To)),
        (EdgeType::Verifies, Ac, Nfr, Some(EdgeEnd::To)),
        (EdgeType::VerifyAc, Req, Ac, Some(EdgeEnd::From)),
        (EdgeType::VerifyAc, Req, Goal, None),
        (EdgeType::ParentChild, Task, Epic, Some(EdgeEnd::To)),
        (EdgeType::ParentChild, Epic, Epic, Some(EdgeEnd::To)),
        (EdgeType::ParentChild, Policy, Epic, None),
        (EdgeType::Answers, Ruling, Question, Some(EdgeEnd::To)),
        (EdgeType::Answers, Ruling, Task, None),
        (EdgeType::RunOf, Run, Task, Some(EdgeEnd::To)),
    ];
    for (t, from, to, want) in cases {
        assert_eq!(parent_end(t, from, to), want, "{t:?} {from:?} → {to:?}");
    }
    for t in EdgeType::ALL {
        for from in NodeKind::ALL {
            for to in NodeKind::ALL {
                let end = basis_end(t, from, to);
                let (p, c) = match end {
                    EdgeEnd::From => (from, to),
                    EdgeEnd::To => (to, from),
                };
                let hit = NESTS
                    .iter()
                    .any(|n| n.edge_type == t && n.parents.contains(&p) && n.children.contains(&c));
                let want = hit.then_some(end);
                assert_eq!(parent_end(t, from, to), want, "{t:?} {from:?} → {to:?}");
            }
        }
    }
}

#[test]
fn mtree_design_on_fixture() {
    let f = forest(&fixture(), Tree::Design);
    assert!(f.unread.is_empty());
    let srs = [
        (0, "band:SRS:5", true, 5),
        (1, "G-1", false, 0),
        (1, "FR2", false, 0),
        (1, "FR14", false, 0),
        (1, "NFR2", false, 0),
        (1, "AC-3", false, 0),
    ];
    assert_eq!(outline(&f.items), design_outline(&srs));
    let heads: Vec<Band> = f
        .items
        .iter()
        .map(|b| match b.head {
            Head::Band { band, .. } => band,
            ref h => panic!("1 段目が帯でない: {h:?}"),
        })
        .collect();
    assert_eq!(heads, Tree::Design.bands());

    let g = forest(&with_ac(), Tree::Design);
    let srs = [
        (0, "band:SRS:5", true, 4),
        (1, "G-1", false, 0),
        (1, "FR2", true, 1),
        (2, "AC-3", false, 0),
        (1, "FR14", true, 1),
        (2, "AC-3", false, 0),
        (1, "NFR2", false, 0),
    ];
    assert_eq!(outline(&g.items), design_outline(&srs));
    for i in [0, 1, 2, 4] {
        assert_eq!(g.items[i], f.items[i], "帯 {i} は足す前と同じ");
    }
}

#[test]
fn mtree_ledger_on_fixture() {
    let doc = fixture();
    let f = forest(&doc, Tree::Ledger);
    assert!(f.unread.is_empty());
    assert_eq!(outline(&f.items), rows(LEDGER));
    let t3 = item(&f, "t3");
    assert_eq!(t3.shape, "shape band-beads");
    assert!(!t3.alert);
    assert_eq!(t3.title, "tsuzuri の面");
    let q10 = item(&f, "t3.q10");
    assert_eq!(q10.shape, "shape band-beads");
    assert!(q10.alert);
    let hub2 = item(&f, "t3-hub.2");
    assert_eq!(hub2.shape, "shape band-beads fill");
    assert!(!hub2.alert);
    let r1 = item(&f, "r-1");
    assert_eq!(r1.shape, "shape band-pipeline fill");
    assert_eq!(r1.title, "t3-hub.2 の 1 回目");

    for s in ["closed", "rejected", "superseded"] {
        assert!(is_closed(Some(s)), "{s}");
    }
    for s in ["open", "in_progress", "blocked", "deferred", "Landed"] {
        assert!(!is_closed(Some(s)), "{s}");
    }
    assert!(!is_closed(None));

    let mut blocked = doc.clone();
    blocked
        .beads
        .get_mut("t3")
        .expect("t3 の属性")
        .status = "blocked".to_string();
    let b = forest(&blocked, Tree::Ledger);
    let t3b = branch(&b, "t3");
    assert!(t3b.open);
    assert_eq!(t3b.kids, branch(&f, "t3").kids);

    let mut closed = doc;
    closed
        .beads
        .get_mut("t3-hub.15")
        .expect("t3-hub.15 の属性")
        .status = "closed".to_string();
    let c = forest(&closed, Tree::Ledger);
    assert!(!branch(&c, "t3-hub.15").open);
}

#[test]
fn mtree_items_and_keys() {
    let plain = fixture();
    let ac = with_ac();
    let sets = [
        (forest(&plain, Tree::Design), &plain),
        (forest(&ac, Tree::Design), &ac),
        (forest(&plain, Tree::Ledger), &plain),
    ];
    for (f, doc) in &sets {
        let mut keys = Vec::new();
        for b in all(&f.items) {
            if !b.kids.is_empty() {
                let k = fold_key(&b.head);
                assert!(!keys.contains(&k), "開き閉じの鍵 {k} が 2 つ");
                keys.push(k);
            }
            let Head::Node(it) = &b.head else {
                continue;
            };
            let node = doc
                .nodes
                .iter()
                .find(|n| n.id == it.id)
                .unwrap_or_else(|| panic!("{} が電文の節点", it.id));
            assert_eq!(it.title, title36(&node.title), "{}", it.id);
            assert_eq!(it.shape, shape_class(doc, node), "{}", it.id);
            assert_eq!(it.alert, open_question(doc, node), "{}", it.id);
            assert_eq!(it.aside, "", "{}", it.id);
            assert!(card_of(doc, &it.id).is_some(), "{} の card", it.id);
        }
    }
    assert_eq!(
        item(&sets[0].0, "R-4").title,
        "abcdefghij abcdefghij abcdefghij abc"
    );
    assert_eq!(
        fold_key(&Head::Band {
            band: Band::Constitution,
            count: 9
        }),
        "tree:band:constitution"
    );
    assert_eq!(
        fold_key(&Head::Band {
            band: Band::DesignNote,
            count: 0
        }),
        "tree:band:design-note"
    );
    assert_eq!(
        fold_key(&Head::Note {
            note: "surface-board".to_string(),
            count: 2
        }),
        "tree:note:surface-board"
    );
    assert_eq!(
        fold_key(&Head::Node(item(&sets[2].0, "t3").clone())),
        "tree:node:t3"
    );
}

#[test]
fn mtree_odd_edges_keep_shape() {
    let odd = with_edges(&[
        ("P-1", "P-1.1", EdgeType::InArticle),
        ("P-1", "P-1.2", EdgeType::InArticle),
        ("A-2", "A-2.1", EdgeType::InArticle),
        ("N-1", "N-1.2", EdgeType::InArticle),
        ("N-1", "N-1.10", EdgeType::InArticle),
        ("N-1", "A-2", EdgeType::InArticle),
        ("t3", "t3-hub.9", EdgeType::ParentChild),
        ("t3.p", "t3", EdgeType::ParentChild),
        ("t3.m1", "nowhere", EdgeType::ParentChild),
        ("FR2", "G-1", EdgeType::VerifyAc),
    ]);
    let plain = fixture();
    for tree in Tree::ALL {
        assert_eq!(forest(&odd, tree), forest(&plain, tree), "{tree:?}");
    }
}

/// 走行の項を除き、子が無くなった項を畳む。
fn without_runs(items: &[Branch]) -> Vec<Branch> {
    items
        .iter()
        .filter(|b| !matches!(&b.head, Head::Node(i) if i.id.starts_with("r-")))
        .map(|b| {
            let kids = without_runs(&b.kids);
            Branch {
                head: b.head.clone(),
                open: b.open && !kids.is_empty(),
                kids,
            }
        })
        .collect()
}

#[test]
fn mtree_unread_and_empty() {
    let plain = fixture();
    let unread = |sources: &[GraphSource]| {
        let mut d = fixture();
        d.unread = sources.to_vec();
        d
    };
    let design = forest(&plain, Tree::Design);
    let ledger = forest(&plain, Tree::Ledger);

    let d = unread(&[GraphSource::Design]);
    let f = forest(&d, Tree::Design);
    assert_eq!(f.unread, [unread_reason(GraphSource::Design)]);
    assert!(f.items.is_empty());
    assert_eq!(forest(&d, Tree::Ledger), ledger);

    let d = unread(&[GraphSource::Ledger]);
    let f = forest(&d, Tree::Ledger);
    assert_eq!(f.unread, [unread_reason(GraphSource::Ledger)]);
    assert_eq!(
        outline(&f.items),
        rows(&[
            (0, "r-1", false, 0),
            (0, "r-3", false, 0),
            (0, "r-12", false, 0)
        ])
    );
    assert_eq!(forest(&d, Tree::Design), design);

    let d = unread(&[GraphSource::Runs]);
    let f = forest(&d, Tree::Ledger);
    assert_eq!(f.unread, [unread_reason(GraphSource::Runs)]);
    assert_eq!(f.items, without_runs(&ledger.items));
    assert!(!branch(&f, "t3-hub.2").open);
    assert!(branch(&f, "t3-hub.15").kids.is_empty());

    let d = unread(&[GraphSource::Ledger, GraphSource::Runs]);
    let f = forest(&d, Tree::Ledger);
    assert_eq!(
        f.unread,
        [
            unread_reason(GraphSource::Ledger),
            unread_reason(GraphSource::Runs)
        ]
    );
    assert!(f.items.is_empty());

    let mut empty = fixture();
    empty.nodes.clear();
    empty.edges.clear();
    empty.beads.clear();
    empty.runs.clear();
    empty.unread.clear();
    let f = forest(&empty, Tree::Design);
    assert!(f.unread.is_empty());
    let want: Vec<Branch> = Tree::Design
        .bands()
        .iter()
        .map(|&band| Branch {
            head: Head::Band { band, count: 0 },
            open: false,
            kids: Vec::new(),
        })
        .collect();
    assert_eq!(f.items, want);
    let f = forest(&empty, Tree::Ledger);
    assert!(f.unread.is_empty());
    assert!(f.items.is_empty());
}

/// src の file を字「mod dom {」の最初の所の前と後に分ける。
fn split_dom(rel: &str) -> (String, String) {
    let text = read(rel);
    let at = text
        .find("mod dom {")
        .unwrap_or_else(|| panic!("{rel} に mod dom が在る"));
    (text[..at].to_string(), text[at..].to_string())
}

#[test]
fn mtree_dom_text() {
    let (head, dom) = split_dom("src/mapview/tree.rs");
    assert!(head.contains("pub use dom::view;"));
    for word in [
        "pub fn view(",
        "forest(",
        "delegate()",
        "card_of(",
        ".show(&ev, card)",
        "leaves(",
        ".leave(&ev)",
        "<ul class=\"items\" on:mouseover=over on:mouseout=out>",
        "<li class=\"nest\">",
        "<details class=\"fold\" prop:open=open on:toggle=toggle>",
        "fold(fold_key(",
        "shown.get().then(",
        "<a class=\"ttl\" href=href data-key=",
        "node_href(",
        "band_chip(",
        "unmeasured(",
    ] {
        assert!(dom.contains(word), "tree.rs の mod dom から後に {word} が無い");
    }
    assert!(!read("src/mapview/tree.rs").contains("use:attach"));
    let (_, map) = split_dom("src/project/map.rs");
    for word in [
        "View::Design => tree::view(d, Tree::Design, search)",
        "View::Ledger => tree::view(d, Tree::Ledger, search)",
    ] {
        assert!(map.contains(word), "map.rs の mod dom から後に {word} が無い");
    }
}

/// 着地済みの行と、この波の行の verify の filter の語（歯の名が当たると、その行の歯の置き場が増える）。
const FILTERS: &[&str] = &[
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "acctframe_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "askcard_",
    "batchpanel_",
    "board_min_",
    "contract_form_",
    "frame_",
    "gapspage_",
    "gquestion_",
    "graph_",
    "gview_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hook_",
    "hsblock_",
    "hsderive_",
    "hspage_",
    "ledgerblock_",
    "mapgraph_",
    "mapview_",
    "nextstep_",
    "nodepage_",
    "parts_",
    "pipe_",
    "project_",
    "question_",
    "seatblock_",
    "seatcard_",
    "server_",
    "skeleton_",
    "stage_",
    "stats_",
    "steady_",
    "topbar_",
    "tz_",
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
    "ilink_",
    "afocus_",
    "nxact_",
    "urpanel_",
    "saxis_",
    "ticker_",
    "hfig_",
    "pmore_",
    "lspark_",
    "apop_",
    "brand_",
    "runsdoc_",
    "nbatch_",
    "kcli_",
    "qgate_",
    "hcard_",
    "ntime_",
    "ptitle_",
    "ncard_",
    "hsym_",
    "smore_",
    "lcard_",
    "aaround_",
    "hruling_",
    "sxaxis_",
    "plimit_",
    "bport_",
    "fstop_",
    "nsum_",
    "nsumw_",
    "fmark_",
    "fserve_",
    "cadopt_",
    "tipx_",
    "hcsess_",
    "hcproj_",
    "sesplit_",
    "mstore_",
    "athr_",
    "qblock_",
    "wsteady_",
    "gsum_",
    "nstall_",
    "pfold_",
    "uword_",
    "cround_",
    "cgdom_",
    "csled_",
    "lhome_",
    "shb_",
    "ghb_",
    "nact_",
    "aord_",
];

/// この file の test の属性の付いた fn の名。
fn test_names() -> Vec<String> {
    let text = read("tests/mtree.rs");
    let mut names = Vec::new();
    let mut marked = false;
    for line in text.lines().map(str::trim) {
        if line == "#[test]" {
            marked = true;
        } else if marked && let Some(rest) = line.strip_prefix("fn ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            names.push(name);
            marked = false;
        }
    }
    names
}

#[test]
fn mtree_names_stay_apart() {
    assert_eq!(FILTERS.len(), 106);
    let names = test_names();
    assert!(names.len() >= 10, "歯の名が 10 以上: {names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("mtree_")
            .unwrap_or_else(|| panic!("{name} が mtree_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
