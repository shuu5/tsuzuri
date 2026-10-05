//! 行 g-card-around の歯: hover の委ねの判定（over_shows と leaves）・節点と状態の字から組む card（card_for）と
//! 眺めの節点の id ごとの card（view_cards）・近傍の図と一覧が引く card の鍵。
//! card の値は host で組み、委ねの口と近傍の図の DOM は wasm の target のときだけなので、2 つの file の字で付け方を見る。
#![cfg(test)]

use std::collections::BTreeSet;

use crate::common::read;
use tsuzuri_contract::graph::{AroundDoc, GraphDoc};
use tsuzuri_contract::wire;
use tsuzuri_surface::mapview::around::{as_view, chain, layout, svg};
use tsuzuri_surface::mapview::state;
use tsuzuri_surface::widgets::hover::{Card, leaves, over_shows};
use tsuzuri_surface::widgets::nodecard::{card_for, node_card, view_cards};

fn graph_fixture() -> GraphDoc {
    wire::decode(&read("../../tests/fixtures/surface/graph-doc.json"))
        .expect("graph-doc.json が電文として読める")
}

fn around_fixture() -> AroundDoc {
    wire::decode(&read("../../tests/fixtures/surface/around-doc.json"))
        .expect("around-doc.json が電文として読める")
}

/// id の節点（無ければ落ちる）。
fn node<'a>(doc: &'a GraphDoc, id: &str) -> &'a tsuzuri_contract::graph::GraphNode {
    doc.nodes
        .iter()
        .find(|n| n.id == id)
        .unwrap_or_else(|| panic!("fixture に {id} が無い"))
}

/// 字の `start` から後で最初に出る `end` の前までの区間（どちらかが無ければ落ちる）。
fn span<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    let from = text
        .find(start)
        .unwrap_or_else(|| panic!("字 {start} が無い"));
    let rest = &text[from..];
    let to = rest[start.len()..]
        .find(end)
        .unwrap_or_else(|| panic!("字 {start} の後に字 {end} が無い"));
    &rest[..start.len() + to]
}

/// 字「mod dom {」より後の字（DOM の部分）。
fn dom_part(text: &str) -> &str {
    let at = text.find("mod dom {").expect("字 mod dom { が在る");
    &text[at..]
}

/// (1) 委ねで指が入ったときは出している card と同じ値でなければ出し、出たときは節点を離れたときだけ猶予に入る。
#[test]
fn cround_hover_steps() {
    let c = Card {
        title: "題".to_string(),
        kind: "task · beads · open".to_string(),
        ..Card::default()
    };
    let mut other = c.clone();
    other.title = "別の題".to_string();
    let same = c.clone();
    assert!(over_shows(None, &c));
    assert!(!over_shows(Some(&same), &c));
    assert!(over_shows(Some(&other), &c));

    assert!(leaves(Some("A"), None));
    assert!(leaves(Some("A"), Some("B")));
    assert!(!leaves(Some("A"), Some("A")));
    assert!(!leaves(None, None));
    assert!(!leaves(None, Some("A")));
}

/// (2) node_card は card_for に節点と電文の状態を渡した値と同じで、card_for は受けた状態の字をそのまま種類の行に置く。
#[test]
fn cround_card_split_same() {
    let doc = graph_fixture();
    for n in &doc.nodes {
        assert_eq!(node_card(&doc, n), card_for(n, state(&doc, n)), "{}", n.id);
    }

    let mut changed = doc.clone();
    {
        let p = changed
            .nodes
            .iter_mut()
            .find(|n| n.id == "P-1")
            .expect("fixture に P-1 が在る");
        p.plain = Some("やさしい".to_string());
        p.eng = Some("かたい".to_string());
        p.line = Some(12);
    }
    changed
        .beads
        .get_mut("t3-hub.9")
        .expect("fixture の beads に t3-hub.9 が在る")
        .status = "closed".to_string();
    changed
        .runs
        .get_mut("r-12")
        .expect("fixture の runs に r-12 が在る")
        .stage = Some("Landed".to_string());
    for n in &changed.nodes {
        assert_eq!(
            node_card(&changed, n),
            card_for(n, state(&changed, n)),
            "変えた電文の {}",
            n.id
        );
    }
    assert_eq!(state(&changed, node(&changed, "t3-hub.9")), Some("closed"));
    assert_eq!(state(&changed, node(&changed, "r-12")), Some("Landed"));

    let task = node(&doc, "t3-hub.9");
    assert_eq!(
        card_for(task, Some("in_progress")).kind,
        "task · beads · in_progress"
    );
    assert_eq!(card_for(task, None).kind, "task · beads · 状態なし");
    assert_eq!(
        card_for(node(&doc, "P-1"), Some("x")).kind,
        "条 · constitution · x"
    );
    assert_eq!(
        card_for(node(&doc, "r-1"), Some("Failed")).kind,
        "run · pipeline · Failed"
    );
}

/// (3) 近傍の眺めの節点の card の鍵は rows の id の全部で、図の data-key と一覧の行の id はどれも鍵に在る。
#[test]
fn cround_around_cards() {
    let doc = around_fixture();
    let gv = as_view(&doc);
    let cards = view_cards(&gv.nodes);

    let rows: BTreeSet<String> = doc.rows.iter().map(|r| r.node.id.clone()).collect();
    assert_eq!(rows.len(), 6);
    let keys: BTreeSet<String> = cards.keys().cloned().collect();
    assert_eq!(keys, rows);
    for r in &doc.rows {
        assert_eq!(
            cards[&r.node.id],
            card_for(&r.node, r.status.as_deref()),
            "{}",
            r.node.id
        );
    }
    for (id, title, kind) in [
        (
            "P-1",
            "面は測れないことを 0 と書かない",
            "条 · constitution · 状態なし",
        ),
        ("ADR-1", "導出グラフを面の正本にする", "ADR · ADR · 状態なし"),
        (
            "FR1",
            "project board を 1 枚の頁で見る",
            "requirement · SRS · 状態なし",
        ),
        (
            "AC1",
            "頁の枠が見本と同じ並びになる",
            "acceptance · SRS · 状態なし",
        ),
        ("e.2", "近傍の段数を URL に残すか", "question · beads · open"),
        (
            "e.2:20260927T0000Z-1",
            "段数は URL の k に残す",
            "あなたの決定 · beads · 状態なし",
        ),
    ] {
        let c = &cards[id];
        assert_eq!(c.title, title, "{id} の題");
        assert_eq!(c.kind, kind, "{id} の種類");
    }

    // 同じ id が後に在っても前の節点の値のまま。
    let mut nodes = gv.nodes.clone();
    let mut dup = nodes
        .iter()
        .find(|n| n.node.id == "e.2")
        .expect("眺めに e.2 が在る")
        .clone();
    dup.status = Some("closed".to_string());
    nodes.push(dup);
    let twice = view_cards(&nodes);
    assert_eq!(twice.len(), 6);
    assert_eq!(twice["e.2"].kind, "question · beads · open");

    assert!(view_cards(&[]).is_empty());
    drawn_keys(doc, cards);
}

/// 図の data-key の値と一覧の行と入れ子の行の id が card の鍵に在る。
fn drawn_keys(doc: AroundDoc, cards: std::collections::BTreeMap<String, Card>) {
    // 図の data-key の値と一覧の行と入れ子の行の id は、どれも鍵に在る。
    let picture = svg(&doc, &layout(&doc));
    let marker = "data-key=\"";
    let data_keys: Vec<&str> = picture
        .match_indices(marker)
        .map(|(i, _)| {
            let rest = &picture[i + marker.len()..];
            &rest[..rest.find('"').expect("data-key の値の閉じ引用符")]
        })
        .collect();
    assert_eq!(data_keys.len(), 6, "{data_keys:?}");
    for k in data_keys {
        assert!(cards.contains_key(k), "図の data-key {k} が鍵に無い");
    }
    let mut listed = 0;
    for side in chain(&doc) {
        for item in &side.items {
            assert!(cards.contains_key(&item.id), "一覧の {} が鍵に無い", item.id);
            listed += 1;
            for n in &item.nest {
                assert!(cards.contains_key(&n.id), "入れ子の {} が鍵に無い", n.id);
                listed += 1;
            }
        }
    }
    assert!(listed > 0);
}

/// (4)(5) 委ねの口が attach と同じ層の show と leave を通り、近傍の図の委ねと一覧の題の a が card を出す字。
#[test]
fn cround_dom_text() {
    let hover = read("src/widgets/hover.rs");
    let dom = dom_part(&hover);
    for want in [
        "pub struct Delegate",
        "pub fn delegate() -> Delegate",
        "pub fn show(self",
        "pub fn leave(self",
        "&web_sys::MouseEvent",
        "pub fn attach(",
    ] {
        assert!(dom.contains(want), "hover.rs の DOM の部分に {want} が無い");
    }
    assert!(span(dom, "pub fn delegate()", "impl Delegate").contains("use_context::<HoverCtx>()"));
    let show = span(dom, "pub fn show(self", "pub fn leave(self");
    for want in ["over_shows(", ".show(card, point(ev))", ".hold()"] {
        assert!(show.contains(want), "Delegate の show に {want} が無い");
    }
    assert!(span(dom, "pub fn leave(self", "fn card_view(").contains(".leave(point(ev))"));
    let reexport = span(&hover, "pub use dom::{", "};");
    for want in ["CardLayer", "Delegate", "HoverCtx", "attach", "delegate"] {
        assert!(reexport.contains(want), "hover.rs の再出しに {want} が無い");
    }
    around_dom_text();
}

/// 近傍の図の委ねと一覧の題の a が card を出す字（nodearound.rs の DOM の部分）。
fn around_dom_text() {
    let around = read("src/project/nodearound.rs");
    let dom = dom_part(&around);
    for want in [
        "view_cards_in(&gv.nodes, mode())",
        "delegate()",
        ".get(&item.id)",
        "on:mouseover=over on:mouseout=out",
        "on:focusin=move |ev| hover(ev.target())",
    ] {
        assert!(dom.contains(want), "nodearound.rs の DOM の部分に {want} が無い");
    }
    assert!(!dom.contains("on:mouseover=move |ev| hover(ev.target())"));
    let over = span(dom, "let over = move |ev", "let out = move |ev");
    for want in ["cards.get(", ".show(&ev, card)", "hover(ev.target())"] {
        assert!(over.contains(want), "over に {want} が無い");
    }
    assert!(!over.contains("pin."), "over が固定を見る");
    let out = span(dom, "let out = move |ev", "let click = move |ev");
    assert!(out.contains(".leave(&ev)"));
    let at_leaves = out.find("leaves(").expect("out に leaves( が在る");
    let at_pin = out
        .find("pin.with_untracked(")
        .expect("out に pin.with_untracked( が在る");
    assert!(at_leaves < at_pin, "out の leaves( が固定の見張りより後");
    let open = "<a class=\"ttl\"";
    assert_eq!(dom.matches(open).count(), 1);
    let tag = span(dom, open, ">");
    assert_eq!(
        format!("{tag}>"),
        "<a class=\"ttl\" href=href use:attach=card>"
    );
}

/// 着地済みの歯の名の filter の語（106 語・この file の接頭辞は並べない）。
const FILTERS: [&str; 106] = [
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
    "hcard_",
    "qgate_",
    "nsum_",
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
    "cgdom_",
    "csled_",
    "lhome_",
    "shb_",
    "ghb_",
    "nact_",
    "aord_",
    "mtree_",
];

/// (7) この file の歯の名はどれも cround_ で始まり、名の字の全部は filter の語を含まない。
#[test]
fn cround_names_clean() {
    let text = read("tests/teeth2/cround.rs");
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
        assert!(name.starts_with("cround_"), "{name} が cround_ で始まらない");
        for word in FILTERS {
            assert!(!name.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
