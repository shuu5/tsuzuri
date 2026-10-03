//! 行 g-card-graph の歯: 眺めの節点の card（view_cards）の鍵と値。
//! グラフの面の図の data-key と一覧の行の id が鍵に在ることと、DOM の付け方の字の歯は行 m-map-graph で消した。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use tsuzuri_contract::graph::GraphView;
use tsuzuri_contract::wire;
use tsuzuri_surface::widgets::nodecard::{card_for, view_cards};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 例のグラフの眺め（契約の型の GraphView として読めなければ落ちる）。
fn fixture() -> GraphView {
    wire::decode(&read("../../tests/fixtures/surface/graph-view.json"))
        .expect("graph-view.json が眺めの電文として読める")
}

/// (1) 眺めの節点の card の鍵は 8 つの節点の id で、値は card_for の値。
#[test]
fn cgdom_view_cards_cover() {
    let v = fixture();
    let cards = view_cards(&v.nodes);

    let ids: BTreeSet<String> = v.nodes.iter().map(|n| n.node.id.clone()).collect();
    assert_eq!(ids.len(), 8);
    let keys: BTreeSet<String> = cards.keys().cloned().collect();
    assert_eq!(keys, ids);
    for n in &v.nodes {
        assert_eq!(
            cards[&n.node.id],
            card_for(&n.node, n.status.as_deref()),
            "{}",
            n.node.id
        );
    }
    for (id, title, kind) in [
        ("P-1", "根拠の条", "条 · constitution · 状態なし"),
        ("R-1", "規則の行", "rule · rules · 状態なし"),
        ("FR1", "要件の行", "requirement · SRS · 状態なし"),
        ("AC1", "受入基準の行", "acceptance · SRS · 状態なし"),
        ("ADR-1", "判断の記録", "ADR · ADR · 状態なし"),
        ("e", "見本の epic", "epic · beads · open"),
        ("e.1", "見本の契約", "task · beads · open"),
        ("e.1-20260927T000000Z", "見本の走行", "run · pipeline · Landed"),
    ] {
        let c = &cards[id];
        assert_eq!(c.title, title, "{id} の題");
        assert_eq!(c.kind, kind, "{id} の種類");
    }
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
    "cround_",
    "csled_",
    "lhome_",
    "shb_",
    "ghb_",
    "nact_",
    "aord_",
    "mtree_",
];

/// (4) この file の歯の名はどれも cgdom_ で始まり、名の字の全部は filter の語を含まない。
#[test]
fn cgdom_names_clean() {
    let text = read("tests/teeth2/cgdom.rs");
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
    assert!(names.len() >= 2, "歯の数 {}", names.len());
    for name in names {
        assert!(name.starts_with("cgdom_"), "{name} が cgdom_ で始まらない");
        for word in FILTERS {
            assert!(!name.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
