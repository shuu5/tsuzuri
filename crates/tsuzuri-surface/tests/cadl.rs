//! 行 g-card-adopt-c の歯: 台帳の block の epic の進みの行と一覧の項・抜けの検査の頁の名指しの項に付ける節点の card の値と、
//! DOM の付け方の字（kit の一覧の 1 項が card を受け、呼ぶ所が card を渡す）。
//! card の値は host で組み、DOM は wasm の target のときだけなので、file の字で付け方を見る。

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::{BeadAttr, GraphDoc, GraphNode, InvariantCheck, NodeKind};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::stats::LedgerStats;
use tsuzuri_contract::wire;
use tsuzuri_surface::project::ledger::{self, Group};
use tsuzuri_surface::project::{Body, Item, gaps, node_item};
use tsuzuri_surface::view::{Fetched, Screen};
use tsuzuri_surface::widgets::nodecard::card_of;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn graph_text() -> String {
    read("../../tests/fixtures/surface/graph-doc.json")
}

fn graph_doc() -> GraphDoc {
    wire::decode(&graph_text()).expect("fixture の電文が読める")
}

/// 電文にならない読み（まだ読んでいない・読めない・字 {} の本文）。
fn unread() -> [Fetched; 3] {
    [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("{}".to_string()),
    ]
}

/// fixture の指標の組 filled の epic の id を `swap` の対で替えた指標。
fn stats(swap: &[(&str, &str)]) -> LedgerStats {
    let mut sets: BTreeMap<String, LedgerStats> =
        wire::decode(&read("../../tests/fixtures/surface/ledger-stats.json"))
            .expect("fixture の組が電文として読める");
    let mut s = sets.remove("filled").expect("fixture の組 filled");
    for e in s.epics.iter_mut() {
        if let Some((_, to)) = swap.iter().find(|(from, _)| e.epic.as_str() == *from) {
            e.epic = BeadId::new(*to).expect("id");
        }
    }
    s
}

/// 指標を口の本文の形（Reading の Known で包んだ字）にする。
fn wrap(s: &LedgerStats) -> Fetched {
    Fetched::Body(wire::encode(&Reading::Known(s.clone())).expect("電文"))
}

/// (1) epic の進みの行の節点の card は、電文に在る epic の id だけを card_of の値で持つ。
#[test]
fn cadl_epic_cards_rules() {
    let doc = graph_doc();
    let graph = Fetched::Body(graph_text());
    let screen = Screen::initial();

    let s = stats(&[("bm", "t3")]);
    let Body::Filled(m) = ledger::content(&wrap(&s), &screen) else {
        panic!("替えた指標の中身が Filled でない");
    };
    let ids: Vec<&str> = m.epics.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(ids, vec!["t3", "zz.9"]);
    assert_eq!(m, ledger::panel(&s, &screen), "Metrics と EpicBar は変えない");

    let got = ledger::epic_cards(&m.epics, &graph);
    let keys: Vec<&str> = got.keys().map(String::as_str).collect();
    assert_eq!(keys, vec!["t3"]);
    let node = card_of(&doc, "t3").expect("t3 の card");
    assert_eq!(got["t3"], node);
    assert_eq!(node.title, "tsuzuri の面");
    assert_eq!(node.kind, "epic · beads · open");
    assert_eq!(node.value, "t3 要約なし");

    let Body::Filled(plain) = ledger::content(&wrap(&stats(&[])), &screen) else {
        panic!("fixture の指標の中身が Filled でない");
    };
    assert_eq!(plain.epics.len(), 2);
    assert!(ledger::epic_cards(&plain.epics, &graph).is_empty());
    assert!(ledger::epic_cards(&[], &graph).is_empty());
    for g in unread() {
        assert!(ledger::epic_cards(&m.epics, &g).is_empty(), "{g:?}");
    }
}

/// (2) 一覧の組の項の節点の card は、epic の項と下の項の id のうち電文に在るものだけを card_of の値で持つ。
#[test]
fn cadl_group_cards_rules() {
    let doc = graph_doc();
    let graph = Fetched::Body(graph_text());
    let item = |id: &str| node_item(&doc, id).unwrap_or_else(|| panic!("{id} の項"));
    let stray = Item {
        shape: "shape band-beads".to_string(),
        alert: false,
        id: "zz.9".to_string(),
        title: "電文に無い項".to_string(),
        aside: String::new(),
        stage: None,
    };
    let groups = vec![
        Group {
            head: Some(item("t3")),
            children: vec![item("t3.q10"), stray.clone()],
        },
        Group {
            head: None,
            children: vec![item("t3.q2")],
        },
    ];

    let got = ledger::group_cards(&groups, &graph);
    let keys: Vec<&str> = got.keys().map(String::as_str).collect();
    assert_eq!(keys, vec!["t3", "t3.q10", "t3.q2"]);
    for id in keys {
        assert_eq!(Some(&got[id]), card_of(&doc, id).as_ref(), "{id}");
    }
    assert_eq!(got["t3.q2"].title, "閉じた問い");

    let lone = vec![Group {
        head: None,
        children: vec![stray],
    }];
    assert!(ledger::group_cards(&lone, &graph).is_empty());
    assert!(ledger::group_cards(&[], &graph).is_empty());
    for g in unread() {
        assert!(ledger::group_cards(&groups, &g).is_empty(), "{g:?}");
    }
}

/// fixture の判定の電文の g-2 の名指しを fx-g.9 と fx-g.15 に替え、`node` なら節点 fx-g.9 を足した本文。
fn verdicts(node: bool) -> Fetched {
    let mut doc: GraphDoc = wire::decode(&read("../../tests/fixtures/surface/invariants.json"))
        .expect("fixture の判定の電文が読める");
    let check: &mut InvariantCheck = doc
        .invariants
        .iter_mut()
        .find(|c| c.id == "g-2")
        .expect("判定 g-2");
    check.ids = vec!["fx-g.9".to_string(), "fx-g.15".to_string()];
    if node {
        doc.nodes.push(GraphNode {
            id: "fx-g.9".to_string(),
            kind: NodeKind::Task,
            file: None,
            digest: None,
            title: "契約の行".to_string(),
            line: None,
            plain: None,
            eng: None,
            updated: None,
        });
        doc.beads.insert(
            "fx-g.9".to_string(),
            BeadAttr {
                kind: NodeKind::Task,
                status: "open".to_string(),
                labels: Vec::new(),
                pointers: Vec::new(),
                touches: Vec::new(),
            },
        );
    }
    Fetched::Body(wire::encode(&doc).expect("電文の字にできる"))
}

/// (3) 抜けの検査の頁の cards は、名指しの id のうち電文の節点に在るものだけを card_of の値で持つ（鍵は found と同じ）。
#[test]
fn cadl_gaps_cards_rules() {
    let fetched = verdicts(true);
    let Fetched::Body(text) = &fetched else {
        panic!("本文");
    };
    let doc: GraphDoc = wire::decode(text).expect("電文");
    let Body::Filled(g) = gaps::body(&fetched) else {
        panic!("節点を足した判定の電文が Filled でない");
    };
    let keys: Vec<&str> = g.cards.keys().map(String::as_str).collect();
    assert_eq!(keys, vec!["fx-g.9"]);
    let found: Vec<&str> = g.found.keys().map(String::as_str).collect();
    assert_eq!(found, keys, "鍵は found と同じ");
    let card = card_of(&doc, "fx-g.9").expect("fx-g.9 の card");
    assert_eq!(g.cards["fx-g.9"], card);
    assert_eq!(card.title, "契約の行");
    assert_eq!(card.kind, "task · beads · open");
    assert_eq!(Some(&g.found["fx-g.9"]), node_item(&doc, "fx-g.9").as_ref());
    assert!(g.rows.iter().any(|r| r.named.contains(&"fx-g.15".to_string())));
    assert!(!g.cards.contains_key("fx-g.15"));
    assert!(!g.found.contains_key("fx-g.15"));

    let Body::Filled(bare) = gaps::body(&verdicts(false)) else {
        panic!("判定の電文が Filled でない");
    };
    assert!(bare.found.is_empty());
    assert!(bare.cards.is_empty());
}

/// `start` から末尾までの字。
fn after<'a>(text: &'a str, start: &str) -> &'a str {
    let at = text
        .find(start)
        .unwrap_or_else(|| panic!("字 {start} が無い"));
    &text[at..]
}

/// 宣言の字から、次の 4 つの空白と閉じ波括弧だけの行までの字。
fn fn_body<'a>(text: &'a str, decl: &str) -> &'a str {
    let rest = after(text, decl);
    let end = rest.find("\n    }\n").expect("本文の終わり");
    &rest[..end]
}

/// (4) kit の一覧の 1 項が card を受けて題の a に hover の attach_some で付け、台帳の block と抜けの検査の頁が card を渡す。
#[test]
fn cadl_dom_wiring() {
    let ledger_text = read("src/project/ledger.rs");
    let dom = after(&ledger_text, "mod dom {");
    for want in [
        "let graph = crate::net::read(map::PATH);",
        "tier_view(*tier, parts, &m, screen, unref, graph)",
        "Memo::new(move |_| graph.with(|g| epic_cards(&epics, g)))",
        "epic_view(e, c.get(&e.id).cloned())",
        "use crate::widgets::hover::attach_some;",
        "let cards = graph.with(|g| group_cards(&groups, g));",
        "Some(epic) => item_view(epic, None, cards.get(&epic.id).cloned()),",
        ".map(|c| item_view(c, None, cards.get(&c.id).cloned()))",
    ] {
        assert!(dom.contains(want), "ledger.rs の mod dom に {want} が無い");
    }
    assert_eq!(dom.matches("list_view(screen, graph)").count(), 2);
    let epic = fn_body(dom, "fn epic_view(");
    assert!(epic.contains("card: Option<Card>"), "{epic}");
    assert!(epic.contains("<a href=href use:attach_some=card>"), "{epic}");

    let kit = read("src/kit.rs");
    let kdom = after(&kit, "mod dom {");
    assert!(kdom.contains("use crate::widgets::hover::{Card, attach_some};"));
    let decl =
        "\n    pub fn item_view(item: &Item, number: Option<usize>, card: Option<Card>) -> AnyView {\n";
    let body = fn_body(kdom, decl);
    assert!(
        body.contains("<a class=\"ttl\" href=href use:attach_some=card><span class=\"nid\">"),
        "{body}"
    );

    let gaps_text = read("src/project/gaps.rs");
    let gdom = after(&gaps_text, "mod dom {");
    assert!(gdom.contains("Some(item) => item_view(item, None, cards.get(&id).cloned()),"));

    for (name, text) in [
        ("ledger.rs", &ledger_text),
        ("kit.rs", &kit),
        ("gaps.rs", &gaps_text),
    ] {
        for bad in ["fn attach_node", "fn attach_some", "#[allow("] {
            assert!(!text.contains(bad), "{name} に {bad} が在る");
        }
    }
    for bad in ["fn item_view(", "use:attach"] {
        assert!(!gaps_text.contains(bad), "gaps.rs に {bad} が在る");
    }
}

/// main 7ea0462 の contracts の verify の filter の語を畳んだ語と、後の行 g-gz と同じノートの行 g-card-adopt-b の接頭辞。
const FILTERS: &[&str] = &[
    "aaround_",
    "accept_",
    "acchold_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "afocus_",
    "aord_",
    "apop_",
    "askcard_",
    "athr_",
    "batchpanel_",
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadopt_",
    "cdorm_",
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "cspk_",
    "ctick_",
    "denv_",
    "dnedge_",
    "dnrow_",
    "dnskip_",
    "ecache_",
    "epolq_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "fserve_",
    "fstop_",
    "fxpre_",
    "gapspage_",
    "gbnote_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gjst_",
    "glabel_",
    "gnav_",
    "gpface_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "hacols_",
    "harest_",
    "hasplit_",
    "hbconf_",
    "hbmark_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
    "hdchip_",
    "hfig_",
    "hnunk_",
    "hook_",
    "hruling_",
    "hsblock_",
    "hsderive_",
    "hshist_",
    "hspage_",
    "hspk_",
    "hsym_",
    "iclose_",
    "ilink_",
    "kcli_",
    "klink_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lidle_",
    "lresume_",
    "lsnap_",
    "lspark_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
    "mstore_",
    "mtree_",
    "nact_",
    "nbatch_",
    "ncard_",
    "nextstep_",
    "nodepage_",
    "nstall_",
    "nsum_",
    "nsumw_",
    "ntime_",
    "nxact_",
    "parts_",
    "pclosed_",
    "pfold_",
    "pipe_",
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
    "ptitle_",
    "punmap_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "question_",
    "relay_",
    "rhold_",
    "runsdoc_",
    "rvk_",
    "saxis_",
    "sclosed_",
    "seatblock_",
    "seatcard_",
    "server_",
    "sesplit_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "stcli_",
    "steady_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "topbar_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
    "pgz_",
    "cadq_",
];

/// (6) この file の歯の名はどれも cadl_ で始まり、残りの字は filter の語を含まない。
#[test]
fn cadl_names_stay_apart() {
    assert_eq!(FILTERS.len(), 159);
    let text = read("tests/cadl.rs");
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
            .strip_prefix("cadl_")
            .unwrap_or_else(|| panic!("{name} が cadl_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
