//! 行 g-link-graph の歯: グラフの面の節点から節点の頁へ移る道（固定の帯の link・2 回押す・Enter と Space・
//! 狭い幅の一覧の行の link）と、wasm の target のときだけの DOM を src/mapview/graph/dom.rs へ移した形。
//! DOM と事件の受け取りは host では撃てないので、host の関数の値と src の字を見る。

use std::path::PathBuf;

use tsuzuri_contract::graph::GraphView;
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::{self, Mode, PageId};
use tsuzuri_surface::mapview::graph::opens_node;
use tsuzuri_surface::project::nodearound::id_of;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

const HOST: &str = "src/mapview/graph.rs";
const DOM: &str = "src/mapview/graph/dom.rs";

/// 着地済みの行とこの波の行の verify の filter の語（歯の名に部分の字として含めない）。
const FILTERS: [&str; 66] = [
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
];

/// Enter と空白 1 字（Space の key の値）でだけ節点の頁へ移る（見本の ui.js の 1086 行）。
#[test]
fn gnav_enter_space_decide() {
    assert!(opens_node("Enter"));
    assert!(opens_node(" "));
    for key in [
        "Escape",
        "Tab",
        "ArrowRight",
        "",
        "enter",
        "Spacebar",
        "a",
        "  ",
    ] {
        assert!(!opens_node(key), "{key:?}");
    }
}

/// 例のグラフの全部の節点の id と 2 つの mode で、節点の頁への link は同じ id と mode の節点の頁を開く。
#[test]
fn gnav_hrefs_open_same_id() {
    let doc: GraphView = wire::decode(&read("../../tests/fixtures/surface/graph-view.json"))
        .expect("fixture が眺めの電文として読める");
    assert!(!doc.nodes.is_empty(), "例のグラフに節点が在る");
    for n in &doc.nodes {
        let id = n.node.id.as_str();
        for mode in Mode::ALL {
            let href = frame::node_href(id, mode);
            assert_eq!(PageId::from_query(&href), PageId::Node, "{href}");
            assert_eq!(id_of(&href).as_deref(), Some(id), "{href}");
            assert_eq!(Mode::from_query(&href), mode, "{href}");
        }
    }
}

/// DOM の file は節点の頁へ移る道（固定の帯の link・2 回押す・Enter と Space）を組む。
#[test]
fn gnav_dom_wires_open() {
    let text = read(DOM);
    for want in [
        "node_href(",
        "opens_node(",
        "on:dblclick=",
        "on:keydown=",
        "<a class=\"btn sm\" href=",
        "label(\"open_node\"",
    ] {
        assert!(text.contains(want), "{DOM} に {want} が在る");
    }
}

/// 狭い幅の一覧の行の題は節点の頁への link で、「まだ無い」「後の便」の字は残らない。
#[test]
fn gnav_chain_rows_are_anchors() {
    let text = format!("{}{}", read(HOST), read(DOM));
    assert!(text.contains("<a class=\"ttl\" href="));
    for gone in [
        "<span class=\"ttl\">",
        "節点の頁はまだ無い",
        "節点の頁は後の便",
    ] {
        assert!(!text.contains(gone), "{gone} が無い");
    }
}

/// DOM の中身は graph/dom.rs に在り、graph.rs は属性の行の次に mod dom の宣言の行だけを持つ。
#[test]
fn gnav_dom_moved_out() {
    let host = read(HOST);
    assert!(!host.contains("mod dom {"), "inline の mod dom が無い");
    let lines: Vec<&str> = host.split('\n').collect();
    let at = lines
        .iter()
        .position(|l| l.trim() == "mod dom;")
        .expect("mod dom の宣言の行が在る");
    assert!(at > 0);
    assert_eq!(lines[at - 1].trim(), "#[cfg(target_arch = \"wasm32\")]");
    assert!(lines.len() <= 1150, "graph.rs の行の数 {}", lines.len());
    assert!(read(DOM).contains("pub fn view("), "{DOM} に pub fn view が在る");
}

/// この file の歯の名はどれも gnav_ で始まり、残りの字は verify の filter の語を含まない。
#[test]
fn gnav_names_stay_apart() {
    let text = read("tests/graphnav.rs");
    let lines: Vec<&str> = text.lines().collect();
    let mut names = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if l.trim() != "#[test]" {
            continue;
        }
        let decl = lines[i + 1..]
            .iter()
            .find(|l| l.trim_start().starts_with("fn "))
            .expect("test の属性の後に fn が在る");
        let name = decl
            .trim_start()
            .trim_start_matches("fn ")
            .split('(')
            .next()
            .unwrap_or_default()
            .to_string();
        names.push(name);
    }
    assert!(names.len() >= 5, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("gnav_")
            .unwrap_or_else(|| panic!("{name} は gnav_ で始まる"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} は {word} を含まない");
        }
    }
}
