//! 行 g-pipe-fold の歯: 開いた列の畳む button（出す列・URL の col から外す字・button の字・DOM の字）と歯の名。

use std::path::PathBuf;

use tsuzuri_contract::board::{PipelineBoard, PipelineCard, PipelineColumn, Reading};
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::pipeline::{
    CLOSE, Column, LANES, SHOW, columns, open_columns, with_closed, with_open,
};
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture_cards() -> Vec<PipelineCard> {
    let text = read("../../tests/fixtures/surface/pipeline-board.json");
    let board: PipelineBoard = wire::decode(&text).expect("fixture の板が電文として読める");
    let Reading::Known(cards) = board.cards else {
        panic!("fixture の札が Unknown");
    };
    cards
}

/// 着地済みの pipe_ の歯と同じ今（UTC の日の正午）。
const NOW: u64 = 1_790_510_400;

fn board() -> Vec<Column> {
    columns(&fixture_cards(), &[], NOW)
}

fn ids(col: &Column, open: bool) -> Vec<String> {
    col.shown(open).iter().map(|c| c.id.clone()).collect()
}

/// (1) 畳む button は開いた列で札が 3 枚を越えるときだけ・着地済みの shown と more の値は変わらない。
#[test]
fn pfold_closable_rules() {
    assert_eq!(SHOW, 3);
    let cols = board();
    let counts: Vec<usize> = cols.iter().map(|c| c.cards.len()).collect();
    assert_eq!(counts, vec![2, 2, 3, 4]);
    let land = &cols[3];
    assert_eq!(land.lane.column, PipelineColumn::Landed);
    assert_eq!(ids(land, true), vec!["px.12", "px.9", "px.10", "px.8"]);
    assert!(land.closable(true));
    assert!(!land.closable(false));
    assert_eq!(land.more(false), Some(1));
    assert_eq!(land.more(true), None);
    assert_eq!(land.shown(true).len(), 4);
    assert_eq!(land.shown(false).len(), 3);
    for col in &cols[..3] {
        for open in [true, false] {
            assert!(!col.closable(open), "{} の列 open={open}", col.lane.name);
            assert_eq!(col.more(open), None);
        }
    }
    // 「+n」と畳む button は同じ列に同時には出ない。
    for col in &cols {
        for open in [true, false] {
            assert!(!(col.closable(open) && col.more(open).is_some()));
        }
    }
}

/// (2) 列を畳んだ後の URL の query（残りの列は板の順・残りが無ければ col の片を外す・空の字は返さない）。
#[test]
fn pfold_with_closed_rules() {
    let land = PipelineColumn::Landed;
    let cases = [
        ("?mode=expert&col=wait,land", "?mode=expert&col=wait"),
        ("?mode=expert&col=land", "?mode=expert"),
        ("?col=land&page=home", "?page=home"),
        ("?col=land", "?"),
        ("?mode=expert&col=wait", "?mode=expert&col=wait"),
        ("?col=bogus,land&page=home", "?page=home"),
    ];
    let mut returned = Vec::new();
    for (from, to) in cases {
        let got = with_closed(from, land);
        assert_eq!(got, to, "{from} から");
        returned.push(got);
    }
    for lane in LANES {
        let opened = with_open("?mode=expert", lane.column);
        assert_eq!(open_columns(&opened), vec![lane.column]);
        let back = with_closed(&opened, lane.column);
        assert_eq!(back, "?mode=expert", "{} の列", lane.name);
        returned.push(back);

        let rest = with_closed("?mode=expert&col=wait,run,stop,land", lane.column);
        let want: Vec<PipelineColumn> = LANES
            .into_iter()
            .map(|l| l.column)
            .filter(|c| *c != lane.column)
            .collect();
        assert_eq!(open_columns(&rest), want, "{} の列", lane.name);
        returned.push(rest);
    }
    for got in returned.iter().filter(|q| q.contains("mode=expert")) {
        assert_eq!(Mode::from_query(got), Mode::Expert, "{got}");
    }
}

/// (3) 畳む button の字は見出しの語でない file の定数の字。
#[test]
fn pfold_close_text() {
    assert_eq!(CLOSE, "畳む");
    assert!(!CLOSE.is_ascii());
    for key in vocab().keys() {
        let term = vocab().term(key).expect("鍵の語");
        assert_ne!(term.label, CLOSE, "鍵 {key} の見出しの語");
    }
}

/// (4) DOM の部分の字（畳む関数・button の字・「+n」の字は残る）。
#[test]
fn pfold_dom_wiring() {
    let text = read("src/project/pipeline.rs");
    let (_, dom) = text.split_once("mod dom {").expect("字 mod dom {");
    for want in [
        "fn close_column(",
        "with_closed(&search(), column)",
        ".closable(is_open())",
        "{CLOSE}",
        "<button type=\"button\" class=\"more\" on:click=move |_| close_column(open, column)>",
        "on:click=move |_| open_column(open, column)",
        "class=\"more num\"",
    ] {
        assert!(dom.contains(want), "DOM の部分に {want} が無い");
    }
}

/// 着地済みの行の verify の filter の語（106 語）。
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
    "fmark_",
    "fstop_",
    "fserve_",
    "nsumw_",
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
    "uword_",
    "cround_",
    "cgdom_",
    "csled_",
    "lhome_",
    "shb_",
    "ghb_",
    "nact_",
    "aord_",
    "mtree_",
];

/// (6) この file の歯の名は pfold_ で始まり、filter の語を部分の字として含まない。
#[test]
fn pfold_names_stay_apart() {
    assert_eq!(FILTERS.len(), 106);
    let text = read("tests/pfold.rs");
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
            .strip_prefix("pfold_")
            .unwrap_or_else(|| panic!("{name} が pfold_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
