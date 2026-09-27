//! header の題を project の名にする面の歯（接頭辞 brand_・設計ノート surface-wave3b 行 g-brand の完了の条件）。
//! 名の進め方と題の字は view の純粋な関数を host で撃つ。board.rs と index.html と自分の file の字は
//! CARGO_MANIFEST_DIR から読む（題の span の中身と set_title の呼びは wasm の target のときだけ組む）。

use std::fs;
use std::path::{Path, PathBuf};

use tsuzuri_surface::frame::BRAND;
use tsuzuri_surface::view::{BOARD_WORDS, Fetched, board_title, brand, kept_name};

/// 着地済みの行とこの波の行の verify の filter の語（この行の接頭辞は並べない）。
const FILTERS: [&str; 65] = [
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
    "runsdoc_",
    "nbatch_",
];

fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn body(text: &str) -> Fetched {
    Fetched::Body(text.to_string())
}

fn some(name: &str) -> Option<String> {
    Some(name.to_string())
}

#[test]
fn brand_keeps_last_read_name() {
    let cases = [
        (None, body(r#"{"name":"proj-kiri"}"#), some("proj-kiri")),
        (some("a"), Fetched::Failed, some("a")),
        (some("a"), Fetched::NotRead, some("a")),
        (some("a"), body("{}"), some("a")),
        (some("a"), body(r#"{"name":""}"#), some("a")),
        (some("a"), body(r#"{"name":"b"}"#), some("b")),
    ];
    for (before, fetched, want) in cases {
        assert_eq!(
            kept_name(before.clone(), &fetched),
            want,
            "前の名 {before:?} と {fetched:?}"
        );
    }
}

#[test]
fn brand_title_joins_board_words() {
    assert_eq!(BOARD_WORDS, "project board");
    assert_eq!(brand(None), BRAND);
    assert_eq!(brand(Some("proj-kiri")), "proj-kiri");
    assert_eq!(board_title(None), "tsuzuri \u{2014} project board");
    assert_eq!(board_title(Some("proj-kiri")), "proj-kiri \u{2014} project board");
    let html = read(&manifest("index.html"));
    let title = format!("<title>{}</title>", board_title(None));
    assert!(html.contains(&title), "index.html に {title} が無い");
}

#[test]
fn brand_top_reads_name_and_sets_title() {
    let text = read(&manifest("src/board.rs"));
    for word in [
        "tsuzuri_contract::project::PATH as PROJECT_PATH",
        "net::read(PROJECT_PATH)",
        "kept_name(",
        "set_title(",
        "board_title(",
    ] {
        assert!(text.contains(word), "board.rs に {word} が無い");
    }
    let open = r#"<span class="name">"#;
    let at = text.find(open).expect("class の値が name の span が無い");
    let rest = &text[at..];
    let end = rest.find("</span>").expect("span の閉じの tag が無い");
    assert!(
        rest[..end].contains("brand("),
        "題の span の中身に brand( が無い: {}",
        &rest[..end]
    );
    assert!(!text.contains("{BRAND}"), "board.rs に {{BRAND}} が残る");
}

#[test]
fn brand_own_names_clean() {
    let text = read(&manifest("tests/brand.rs"));
    let mut names = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "#[test]" {
            continue;
        }
        let decl = lines.next().expect("test の属性の次の行");
        let name = decl
            .trim()
            .strip_prefix("fn ")
            .and_then(|r| r.split('(').next())
            .unwrap_or_else(|| panic!("fn の宣言でない: {decl}"));
        names.push(name.to_string());
    }
    assert!(names.len() >= 4, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("brand_")
            .unwrap_or_else(|| panic!("{name} が brand_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
