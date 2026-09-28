//! 便 g-map-keys の歯: 地図の頁の tab の上の矢印の key で移る面（端は反対の端へ回る・ほかの key は無し）・
//! DOM の keydown の受け取りの字（wasm の target のときだけなので src の字で見る）・この file の歯の名。

use std::path::PathBuf;

use tsuzuri_surface::mapview::View;
use tsuzuri_surface::project::map::tab_step;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

#[test]
fn mkeys_arrows_wrap_around() {
    use View::{Compact, Design, Graph, Ledger, List, Table};
    let right = [
        (Compact, List),
        (List, Graph),
        (Graph, Table),
        (Table, Design),
        (Design, Ledger),
        (Ledger, Compact),
    ];
    for (now, next) in right {
        assert_eq!(tab_step(now, "ArrowRight"), Some(next), "{now:?} の右");
    }
    let left = [
        (Compact, Ledger),
        (List, Compact),
        (Graph, List),
        (Table, Graph),
        (Design, Table),
        (Ledger, Design),
    ];
    for (now, prev) in left {
        assert_eq!(tab_step(now, "ArrowLeft"), Some(prev), "{now:?} の左");
    }
    let others = [
        "Enter",
        " ",
        "ArrowUp",
        "ArrowDown",
        "Tab",
        "Home",
        "End",
        "",
        "arrowright",
    ];
    for now in View::ALL {
        for key in others {
            assert_eq!(tab_step(now, key), None, "{now:?} で {key:?}");
        }
    }
}

#[test]
fn mkeys_dom_wires_arrows() {
    let text = read("src/project/map.rs");
    let at = text.find("mod dom").expect("map.rs に mod dom が在る");
    let dom = &text[at..];
    for word in ["on:keydown=", "tab_step(", "prevent_default()", ".focus()"] {
        assert!(dom.contains(word), "map.rs の mod dom から後に {word} が無い");
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
];

/// この file の test の属性の付いた fn の名。
fn test_names() -> Vec<String> {
    let text = read("tests/mapkeys.rs");
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
fn mkeys_names_stay_apart() {
    assert_eq!(FILTERS.len(), 66);
    let names = test_names();
    assert!(names.len() >= 3, "歯の名が 3 つ以上: {names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("mkeys_")
            .unwrap_or_else(|| panic!("{name} が mkeys_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
