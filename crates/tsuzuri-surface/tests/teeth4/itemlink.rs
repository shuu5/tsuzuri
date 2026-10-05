//! 行 g-link-items の歯: 電文の節点から一覧の 1 項を引く node_item・抜けの検査の頁の名指しの found・
//! 一覧の 1 項の題を節点の頁への link にする item_view の字・gaps の mod dom が found を item_view で描く字・
//! この file の歯の名。
#![cfg(test)]

use std::collections::BTreeMap;

use crate::common::read;
use tsuzuri_contract::graph::{BeadAttr, GraphDoc, GraphNode, NodeKind};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::{self, Body, Item, gaps};
use tsuzuri_surface::view::Fetched;

const FIXTURE: &str = "../../tests/fixtures/surface/invariants.json";

/// 着地済みの fixture のままの電文（節点 0）。
fn plain() -> GraphDoc {
    wire::decode(&read(FIXTURE)).expect("fixture が電文として読める")
}

fn node(id: &str, kind: NodeKind, title: &str) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        kind,
        file: None,
        digest: None,
        title: title.to_string(),
        line: None,
        plain: None,
        eng: None,
        updated: None,
    }
}

fn bead(kind: NodeKind, status: &str) -> BeadAttr {
    BeadAttr {
        kind,
        status: status.to_string(),
        labels: Vec::new(),
        pointers: Vec::new(),
        touches: Vec::new(),
    }
}

/// fixture に節の 4 つの節点と台帳の属性を足した電文。
fn doc() -> GraphDoc {
    let mut d = plain();
    for (id, kind, title, status) in [
        ("t3-hub.9", NodeKind::Task, "契約の行", Some("open")),
        ("q.1", NodeKind::Question, "色を決める", Some("open")),
        ("t3-hub.20", NodeKind::Task, "閉じた契約", Some("closed")),
        ("FR1", NodeKind::Req, "面は 2 つ", None),
    ] {
        d.nodes.push(node(id, kind, title));
        if let Some(s) = status {
            d.beads.insert(id.to_string(), bead(kind, s));
        }
    }
    d
}

fn fetched(d: &GraphDoc) -> Fetched {
    Fetched::Body(wire::encode(d).expect("電文を字にする"))
}

fn want(shape: &str, alert: bool, id: &str, title: &str) -> Item {
    Item {
        shape: shape.to_string(),
        alert,
        id: id.to_string(),
        title: title.to_string(),
        aside: String::new(),
    }
}

/// (1) node_item は節の 4 つの節点で節の値の Item を返し、節点に無い id で None を返す。
#[test]
fn ilink_node_row_from_doc() {
    let d = doc();
    let cases = [
        want("shape band-beads", false, "t3-hub.9", "契約の行"),
        want("shape band-beads", true, "q.1", "色を決める"),
        want("shape band-beads fill", false, "t3-hub.20", "閉じた契約"),
        want("shape band-srs fill", false, "FR1", "面は 2 つ"),
    ];
    for w in cases {
        assert_eq!(project::node_item(&d, &w.id), Some(w.clone()), "{}", w.id);
        assert_eq!(tsuzuri_surface::kit::node_item(&d, &w.id), Some(w));
    }
    assert_eq!(project::node_item(&d, "t3-hub.15"), None);
    assert_eq!(project::node_item(&plain(), "t3-hub.9"), None);
}

fn filled(d: &GraphDoc) -> gaps::Gaps {
    match gaps::body(&fetched(d)) {
        Body::Filled(g) => g,
        other => panic!("電文が中身にならない: {other:?}"),
    }
}

/// (2) found の鍵は名指しの id のうち節点に在るもの・値は node_item の値・tiles と rows は invariants から。
#[test]
fn ilink_gaps_rows_found() {
    let d = doc();
    let g = filled(&d);
    let keys: Vec<&str> = g.found.keys().map(String::as_str).collect();
    assert_eq!(keys, ["t3-hub.9"]);
    let named: Vec<&String> = g.rows.iter().flat_map(|r| &r.named).collect();
    let expect: BTreeMap<String, Item> = named
        .iter()
        .filter_map(|id| project::node_item(&d, id).map(|i| ((*id).clone(), i)))
        .collect();
    assert_eq!(g.found, expect);
    assert_eq!(g.tiles, gaps::tiles(&d.invariants));
    assert_eq!(g.rows, gaps::rows(&d.invariants));

    let p = plain();
    let g = filled(&p);
    assert!(g.found.is_empty(), "{:?}", g.found);
    assert_eq!(g.tiles, gaps::tiles(&p.invariants));
    assert_eq!(g.rows, gaps::rows(&p.invariants));
}

/// 宣言の行から、次の 4 つの空白と閉じ波括弧だけの行までの字。
fn fn_body<'a>(text: &'a str, decl: &str) -> &'a str {
    let start = text.find(decl).unwrap_or_else(|| panic!("{decl} が無い"));
    let rest = &text[start..];
    let end = rest.find("\n    }\n").expect("本文の終わり");
    &rest[..end]
}

const DOM: &str = "#[cfg(target_arch = \"wasm32\")]\nmod dom {";

/// (3) item_view の宣言の行は今と同じ・本文は li を 1 つと class ttl の a を持ち class ttl の span を持たない・
/// mod dom は node_href と HelpCtx の context と Mode の from_query を呼ぶ。
#[test]
fn ilink_shared_row_links() {
    let text = read("src/kit.rs");
    let dom = &text[text.find(DOM).expect("kit の mod dom")..];
    let decl =
        "\n    pub fn item_view(item: &Item, number: Option<usize>, card: Option<Card>) -> AnyView {\n";
    assert!(dom.contains(decl), "item_view の宣言の行");
    let body = fn_body(dom, decl);
    assert!(body.contains("<a class=\"ttl\""), "{body}");
    assert!(!body.contains("<span class=\"ttl\""), "{body}");
    assert_eq!(body.matches("<li").count(), 1, "{body}");
    for word in ["node_href(", "use_context::<HelpCtx>()", "Mode::from_query("] {
        assert!(dom.contains(word), "kit の mod dom に {word} が無い");
    }
}

/// (4)(5) gaps の mod dom は found を item_view で描き、無い id の字を残す・節点の列を自分でたどらない。
#[test]
fn ilink_gaps_src_uses_row() {
    let text = read("src/project/gaps.rs");
    let dom = &text[text.find(DOM).expect("gaps の mod dom")..];
    for word in ["item_view(", "found", "<span class=\"ttl\"><span class=\"nid\">"] {
        assert!(dom.contains(word), "gaps の mod dom に {word} が無い");
    }
    for word in [".nodes", ".edges", ".beads", ".runs", "\"/api/"] {
        assert!(!text.contains(word), "gaps.rs に {word} が在る");
    }
}

/// 着地済みの行の filter の語と、同じ波の行の接頭辞。
const FILTERS: [&str; 65] = [
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
    "hook_",
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
    "hbproc_",
    "hbconf_",
    "hbroute_",
    "hbpost_",
    "hsblock_",
    "hspage_",
    "hsderive_",
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
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
];

/// (7) この file の歯の名はどれも ilink_ で始まり、残りの字は filter の語を含まない。
#[test]
fn ilink_own_names_clean() {
    let text = read("tests/teeth4/itemlink.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == "#[test]")
        .map(|(i, _)| {
            lines[i + 1..]
                .iter()
                .find_map(|l| l.strip_prefix("fn "))
                .and_then(|rest| rest.split('(').next())
                .expect("test の属性の後の fn")
        })
        .collect();
    assert!(names.len() >= 5, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("ilink_")
            .unwrap_or_else(|| panic!("{name} が ilink_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
