//! 行 g-link-map の歯: 地図の圧縮の面の札と一覧の面の行の題が節点の頁への link になる。
//! link の字は frame の node_href が組み、節点の頁は同じ id と mode で開く（fixture の全部の id で見る）。
//! DOM は wasm の target のときだけなので、2 つの file の字の在る無しで a の要素と href を見る。

use std::path::PathBuf;

use tsuzuri_contract::graph::GraphDoc;
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::{self, Mode, PageId};
use tsuzuri_surface::mapview::compact::compact;
use tsuzuri_surface::mapview::list::{Query, listing};
use tsuzuri_surface::project::nodearound::id_of;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture() -> GraphDoc {
    wire::decode(&read("../../tests/fixtures/surface/graph-doc.json"))
        .expect("fixture が電文として読める")
}

/// 字 mod dom の最初の所から後の字。
fn dom_part(text: &str) -> &str {
    let at = text.find("mod dom").expect("字 mod dom が在る");
    &text[at..]
}

/// (1) 圧縮の面と一覧の面の全部の id の link は、節点の頁を同じ id と mode で開く。
#[test]
fn mlink_hrefs_open_same_id() {
    let doc = fixture();
    let mut ids: Vec<String> = compact(&doc)
        .iter()
        .flat_map(|b| b.ids().into_iter().map(str::to_string).collect::<Vec<_>>())
        .collect();
    let rows = listing(&doc, &Query::from_search("")).rows;
    assert!(!rows.is_empty(), "一覧の面の行が在る");
    ids.extend(rows.iter().map(|r| r.id.clone()));
    assert!(!ids.is_empty(), "圧縮の面の id が在る");
    for id in &ids {
        for mode in Mode::ALL {
            let href = frame::node_href(id, mode);
            let search = href.trim_start_matches('?');
            assert_eq!(PageId::from_query(search), PageId::Node, "{href}");
            assert_eq!(id_of(search).as_deref(), Some(id.as_str()), "{href}");
            assert_eq!(Mode::from_query(search), mode, "{href}");
        }
    }
}

/// (2) 圧縮の面の札・条の札の頭・規範文の子は a の要素で、href は node_href が組む。
#[test]
fn mlink_compact_tags_are_anchors() {
    let text = read("src/mapview/compact.rs");
    let dom = dom_part(&text);
    for want in [
        "node_href(",
        "<a class=t.class.clone() href=",
        "{tag_head(&a.tag)}</a>",
        "{id}</a>",
    ] {
        assert!(dom.contains(want), "mod dom に {want} が無い");
    }
    for gone in [
        "<div class=t.class.clone()>",
        "<span>{id}</span>",
        "節点の頁はまだ無い",
    ] {
        assert!(!text.contains(gone), "compact.rs に {gone} が残る");
    }
}

/// (3) 一覧の面の行の題は class ttl の a の要素で、href は node_href が組む。
#[test]
fn mlink_list_title_is_anchor() {
    let text = read("src/mapview/list.rs");
    let dom = dom_part(&text);
    for want in ["node_href(", "<a class=\"ttl\" href="] {
        assert!(dom.contains(want), "mod dom に {want} が無い");
    }
    for gone in ["<span class=\"ttl\">", "節点の頁はまだ無い"] {
        assert!(!text.contains(gone), "list.rs に {gone} が残る");
    }
}

/// 着地済みの行とこの波の行の verify の filter の語。
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
];

/// (5) この file の歯の名はどれも mlink_ で始まり、残りの字は filter の語を含まない。
#[test]
fn mlink_names_stay_apart() {
    assert_eq!(FILTERS.len(), 66);
    let text = read("tests/maplink.rs");
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
    assert!(names.len() >= 4, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("mlink_")
            .unwrap_or_else(|| panic!("{name} が mlink_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
