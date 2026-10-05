//! 行 g-link-kcard の歯: 板の札は押すと吹き出しを開く button（hover の card は付けない・行 g-pipe-cards）・歯の名の置き場。
//! 札の link の先 card_href と、その歯 klink_card_opens_same_id は行 g-dead-sweep-a で消した。
#![cfg(test)]

use crate::common::read;


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
fn klink_dom_anchor_keeps_hover() {
    let src = read("src/project/pipeline.rs");
    let at = src.find("mod dom").expect("pipeline.rs に mod dom が在る");
    let dom = &src[at..];
    assert!(dom.contains("p.press(&id, Via::Card)"), "DOM が札の押しで吹き出しを開く");
    assert!(
        dom.contains("<button type=\"button\" class=card.class data-pop-card="),
        "札の外側の要素が button で class の次に吹き出しの口の印（行 g-pipe-cards）"
    );
    assert!(
        !dom.contains("use:attach=card.hover.clone()"),
        "札に hover の card が付かない（行 g-pipe-cards）"
    );
    assert!(
        !src.contains("<div class=card.class"),
        "札の外側の要素に div が残る"
    );
}

#[test]
fn klink_names_stay_apart() {
    let src = read("tests/teeth4/kcardlink.rs");
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
    assert!(names.len() >= 2, "歯の名が 2 つ以上: {names:?}");
    for name in names {
        let rest = name
            .strip_prefix("klink_")
            .unwrap_or_else(|| panic!("{name} が klink_ で始まらない"));
        for word in TAKEN {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
