//! 便 h-popstate の歯: account board の tab の押しは頁を読み直さず tab_url の URL を履歴に積み、戻ると進むで tab と mode を戻す。
//! board.rs は wasm の target のときだけなので src の字を読んで見る（DOM は xtask の surface-build で組めることで見る）。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::account::{Tab, ledger, projects, selects, session, tab_url};
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::seat::span_of;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 節の 4 つの query（A・B・C・D）。
const QUERIES: [&str; 4] = [
    "?board=account&tab=session&sort=stage&lsort=net&psort=group&span=6h&mode=expert",
    "?board=account&mode=beginner",
    "?board=account&tab=home&sort=elapsed&span=3h",
    "",
];

#[test]
fn apop_tab_url_sets_the_tab_param() {
    assert_eq!(
        tab_url(QUERIES[0], Tab::Projects),
        "?board=account&tab=projects&sort=stage&lsort=net&psort=group&span=6h&mode=expert"
    );
    assert_eq!(
        tab_url(QUERIES[0], Tab::Session),
        "?board=account&tab=session&sort=stage&lsort=net&psort=group&span=6h&mode=expert"
    );
    assert_eq!(
        tab_url(QUERIES[1], Tab::Session),
        "?board=account&mode=beginner&tab=session"
    );
    assert_eq!(
        tab_url(QUERIES[2], Tab::Home),
        "?board=account&tab=home&sort=elapsed&span=3h"
    );
    assert_eq!(tab_url(QUERIES[3], Tab::Home), "?tab=home");
}

#[test]
fn apop_tab_url_keeps_every_other_reading() {
    for q in QUERIES {
        for tab in Tab::ALL {
            let url = tab_url(q, tab);
            assert_eq!(Tab::from_query(&url), tab, "{q} → {url}");
            assert_eq!(selects(&url), selects(q), "{q} → {url}");
            assert_eq!(session::sort_of(&url), session::sort_of(q), "{q} → {url}");
            assert_eq!(ledger::sort_of(&url), ledger::sort_of(q), "{q} → {url}");
            assert_eq!(projects::psort_of(&url), projects::psort_of(q), "{q} → {url}");
            assert_eq!(span_of(&url), span_of(q), "{q} → {url}");
            assert_eq!(Mode::from_query(&url), Mode::from_query(q), "{q} → {url}");
        }
    }
}

#[test]
fn apop_dom_wiring_in_source() {
    let text = read("src/account/board.rs");
    for want in [
        "window_event_listener(ev::popstate",
        "push_state_with_url(",
        "tab_url(",
        "prevent_default()",
        "ctrl_key()",
        "meta_key()",
        "shift_key()",
        "scroll_to_with_x_and_y(",
        "aria-current",
        "move || page_view(",
    ] {
        assert!(text.contains(want), "board.rs に {want} の字が無い");
    }
    assert!(
        text.matches("Mode::from_query(").count() >= 2,
        "board.rs の Mode::from_query( は頁の初めと popstate の 2 度以上"
    );
}

/// 着地済みの filter の語（49）と同じ波の行の接頭辞（16）。
const WORDS: [&str; 65] = [
    "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
    "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
    "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
    "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_", "server_",
    "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_", "gnav_", "mkeys_",
    "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_", "ticker_", "hfig_", "pmore_",
    "lspark_", "brand_", "runsdoc_", "nbatch_",
];

#[test]
fn apop_names_keep_own_prefix() {
    let text = read("tests/acctpop.rs");
    let mut names = Vec::new();
    let mut rest = text.as_str();
    while let Some(at) = rest.find("#[test]\nfn ") {
        let after = &rest[at + "#[test]\nfn ".len()..];
        let end = after.find('(').expect("fn の名の後に (");
        names.push(&after[..end]);
        rest = &after[end..];
    }
    assert_eq!(names.len(), 4, "{names:?}");
    for name in names {
        let tail = name
            .strip_prefix("apop_")
            .unwrap_or_else(|| panic!("{name} は apop_ で始まる"));
        for w in WORDS {
            assert!(!tail.contains(w), "{name} が filter の語 {w} を含む");
        }
    }
}
