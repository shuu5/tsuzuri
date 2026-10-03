//! 便 h-board-url の面の歯（接頭辞 bport_）: 電文の board は port だけ・開く URL の host の後ろの字・
//! 頁の location の protocol と hostname から URL を組む字・DOM の open の字の順。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::windows::{board_href, open_url};
use tsuzuri_surface::frame::Mode;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture() -> AccountDoc {
    wire::decode(&read("../../tests/fixtures/account/acct-doc.json"))
        .expect("fixture が AccountDoc として読める")
}

#[test]
fn bport_fixture_ports() {
    let doc = fixture();
    let got: Vec<(&str, Option<u16>)> = doc
        .projects
        .iter()
        .map(|p| (p.name.as_str(), p.board))
        .collect();
    assert_eq!(
        got,
        [
            ("proj-a", Some(40001)),
            ("proj-b", Some(40002)),
            ("proj-c", None)
        ]
    );
}

#[test]
fn bport_open_url_tail_from_port() {
    let doc = fixture();
    assert_eq!(open_url(&doc.projects[2], Mode::Expert), None);
    let mut row = doc.projects[0].clone();
    row.board = Some(1);
    assert_eq!(
        open_url(&row, Mode::Expert).as_deref(),
        Some(":1/?mode=expert")
    );
}

#[test]
fn bport_href_from_location() {
    let doc = fixture();
    for mode in Mode::ALL {
        let tail = open_url(&doc.projects[0], mode).expect("proj-a は port を持つ");
        assert_eq!(
            board_href("http:", "host-1", &tail),
            format!("http://host-1:40001/?mode={}", mode.key())
        );
    }
    assert_eq!(
        board_href("https:", "host-1", ":65535/?mode=expert"),
        "https://host-1:65535/?mode=expert"
    );
}

/// windows.rs の mod dom の open の本体（`pub fn open(` から、その後で最初の 4 つの空白と閉じの波括弧だけの行まで）。
fn open_body() -> String {
    let text = read("src/account/windows.rs");
    let dom = &text[text.find("mod dom {").expect("mod dom")..];
    let head = &dom[dom.find("pub fn open(").expect("mod dom の open")..];
    let mut body = String::new();
    for line in head.lines() {
        body.push_str(line);
        body.push('\n');
        if line == "    }" {
            return body;
        }
    }
    panic!("open の閉じの行が無い");
}

#[test]
fn bport_dom_open_composes() {
    let body = open_body();
    assert!(body.contains("protocol()"), "{body}");
    assert!(body.contains("hostname()"), "{body}");
    assert!(!body.contains(".host()"), "{body}");
    assert!(!body.contains(".port()"), "{body}");
    let bare: String = body.chars().filter(|c| !c.is_whitespace()).collect();
    let compose = bare
        .find("leturl=&board_href(&protocol,&hostname,url);")
        .expect("board_href で url を覆い直す 1 文");
    let set = bare.find("set_href(url)").expect("set_href(url)");
    assert!(compose < set, "覆い直しは set_href(url) より前");
}

/// 着地済みの歯と別の波の行の接頭辞（新しい歯の名はこれを部分の字として含まない）。
const FILTER_WORDS: [&str; 80] = [
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
];

#[test]
fn bport_names_filtered() {
    let text = read("tests/teeth1/bport.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0] == "#[test]")
        .map(|w| {
            let rest = w[1].strip_prefix("fn ").expect("test の属性の次は fn");
            &rest[..rest.find('(').expect("fn の名")]
        })
        .collect();
    assert_eq!(names.len(), 5, "{names:?}");
    for name in names {
        let tail = name
            .strip_prefix("bport_")
            .unwrap_or_else(|| panic!("接頭辞: {name}"));
        for word in FILTER_WORDS {
            assert!(!tail.contains(word), "{name} が {word} を含む");
        }
    }
}
