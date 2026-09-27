//! 行 g-link-kcard の歯: 板の札は契約 bead と同じ id の節点の頁への link（hover の card は残す）・歯の名の置き場。

use std::path::PathBuf;

use tsuzuri_contract::board::{PipelineBoard, Reading};
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::{self, Mode, PageId};
use tsuzuri_surface::project::nodearound::id_of;
use tsuzuri_surface::project::pipeline::{card_href, kcard};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 着地済みの行の verify の filter の語と、この波の行の接頭辞（この行の klink_ を除く）。
const TAKEN: &[&str] = &[
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

#[test]
fn klink_card_opens_same_id() {
    let board: PipelineBoard =
        wire::decode(&read("../../tests/fixtures/surface/pipeline-board.json"))
            .expect("fixture の板が電文として読める");
    let Reading::Known(cards) = board.cards else {
        panic!("fixture の札が Unknown");
    };
    assert!(!cards.is_empty(), "fixture の札が空");
    for c in &cards {
        let k = kcard(c, &[]);
        let id = c.contract.to_string();
        for mode in Mode::ALL {
            let href = card_href(&k, mode);
            assert_eq!(href, frame::node_href(&id, mode), "{id} の札の先");
            assert_eq!(PageId::from_query(&href), PageId::Node, "{href} の頁");
            assert_eq!(id_of(&href).as_deref(), Some(id.as_str()), "{href} の id");
            assert_eq!(Mode::from_query(&href), mode, "{href} の mode");
        }
    }
}

#[test]
fn klink_dom_anchor_keeps_hover() {
    let src = read("src/project/pipeline.rs");
    let at = src.find("mod dom").expect("pipeline.rs に mod dom が在る");
    let dom = &src[at..];
    assert!(dom.contains("card_href("), "DOM が card_href を呼ぶ");
    assert!(
        dom.contains("<a class=card.class href="),
        "札の外側の要素が a で class の次に href"
    );
    assert!(
        dom.contains("use:attach=card.hover.clone()"),
        "札に hover の card が付く"
    );
    assert!(
        !src.contains("<div class=card.class"),
        "札の外側の要素に div が残る"
    );
}

#[test]
fn klink_names_stay_apart() {
    let src = read("tests/kcardlink.rs");
    let lines: Vec<&str> = src.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .filter_map(|w| {
            w[1].trim()
                .strip_prefix("fn ")
                .and_then(|r| r.split('(').next())
        })
        .collect();
    assert!(names.len() >= 3, "歯の名が 3 つ以上: {names:?}");
    for name in names {
        let rest = name
            .strip_prefix("klink_")
            .unwrap_or_else(|| panic!("{name} が klink_ で始まらない"));
        for word in TAKEN {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
