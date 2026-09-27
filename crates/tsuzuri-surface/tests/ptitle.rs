//! 頁の題を project の名と頁の語にする面の歯（接頭辞 ptitle_・設計ノート surface-wave5a 行 g-title の完了の条件）。
//! 題の字と節点の題の進め方は view と節点の module の純粋な関数を host で撃つ。board.rs と node.rs と
//! 自分の file の字は CARGO_MANIFEST_DIR から読む（題を置く効果は wasm の target のときだけ組む）。

use std::fs;
use std::path::{Path, PathBuf};

use tsuzuri_surface::frame::PageId;
use tsuzuri_surface::project::node::kept_subject;
use tsuzuri_surface::project::nodearound::{self, PageState, state};
use tsuzuri_surface::view::{Fetched, PageSubject, SUBJECT_CHARS, doc_title};
use tsuzuri_surface::vocab::label;

/// 着地済みの行とこの波の行の verify の filter の語（この行の接頭辞は並べない）。
const FILTERS: [&str; 80] = [
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
    "ncard_",
    "hsym_",
    "smore_",
    "lcard_",
    "aaround_",
    "hruling_",
    "sxaxis_",
    "plimit_",
    "bport_",
];

const FIXTURE: &str = "../../tests/fixtures/surface/around-doc.json";

/// fixture の中心の行の題。
const CENTER_TITLE: &str = "project board を 1 枚の頁で見る";

fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(rel: &str) -> String {
    let path = manifest(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn some(s: &str) -> Option<String> {
    Some(s.to_string())
}

/// fixture の本文を 200 で読んだ頁の状態（電文）。
fn fixture_doc() -> PageState {
    let st = state(&Fetched::Body(read(FIXTURE)), Some(200));
    assert!(matches!(st, PageState::Doc(_)), "fixture が電文に読めない: {st:?}");
    st
}

/// fixture の電文の欄を書き換えた頁の状態。
fn edited(edit: impl FnOnce(&mut tsuzuri_contract::graph::AroundDoc)) -> PageState {
    match fixture_doc() {
        PageState::Doc(mut doc) => {
            edit(&mut doc);
            PageState::Doc(doc)
        }
        other => panic!("電文でない: {other:?}"),
    }
}

/// (1) doc_title の組（名・見出しの語の鍵・subject）。
#[test]
fn ptitle_joins_name_and_word() {
    assert_eq!(SUBJECT_CHARS, 24);
    assert_eq!(PageSubject::default(), PageSubject(None));
    let name = Some("proj-kiri");
    assert_eq!(doc_title(None, "home", None), "tsuzuri \u{2014} ホーム");
    for (key, want) in [
        ("questions", "proj-kiri \u{2014} 質問"),
        ("map", "proj-kiri \u{2014} 地図"),
        ("gaps", "proj-kiri \u{2014} 抜けの検査"),
        ("nb_self", "proj-kiri \u{2014} この節点"),
    ] {
        assert_eq!(doc_title(name, key, None), want, "鍵 {key}");
    }
    assert_eq!(CENTER_TITLE.chars().count(), 24);
    assert_eq!(
        doc_title(name, "nb_self", Some(CENTER_TITLE)),
        format!("proj-kiri \u{2014} {CENTER_TITLE}")
    );
    let a24 = "あ".repeat(24);
    assert_eq!(
        doc_title(name, "nb_self", Some(&a24)),
        format!("proj-kiri \u{2014} {a24}")
    );
    let a25 = "あ".repeat(25);
    assert_eq!(
        doc_title(name, "nb_self", Some(&a25)),
        format!("proj-kiri \u{2014} {}\u{2026}", "あ".repeat(23))
    );
    for blank in ["", "   "] {
        assert_eq!(
            doc_title(name, "nb_self", Some(blank)),
            "proj-kiri \u{2014} この節点",
            "subject {blank:?}"
        );
    }
}

/// (2) 生成した頁ごとに、題の語は頁の見出しの語。
#[test]
fn ptitle_every_page_has_word() {
    let words = [
        ("home", "ホーム"),
        ("ask", "質問"),
        ("map", "地図"),
        ("gaps", "抜けの検査"),
        ("node", "この節点"),
    ];
    for page in PageId::ALL {
        let heading = page.def().heading;
        let word = label(heading);
        assert!(
            !word.starts_with("〔語彙表に無い"),
            "頁 {} の見出しの鍵 {heading} が語彙表に無い",
            page.id()
        );
        assert_eq!(
            doc_title(Some("p"), heading, None),
            format!("p \u{2014} {word}"),
            "頁 {}",
            page.id()
        );
    }
    for (id, want) in words {
        let page = PageId::ALL
            .into_iter()
            .find(|p| p.id() == id)
            .unwrap_or_else(|| panic!("頁 {id} が無い"));
        assert_eq!(label(page.def().heading), want, "頁 {id}");
    }
}

/// (3) kept_subject の組（fixture の電文・見つからない・まだ読んでいない・読めない）。
#[test]
fn ptitle_subject_from_center() {
    let doc = fixture_doc();
    assert_eq!(kept_subject(None, &doc), some(CENTER_TITLE));
    assert_eq!(kept_subject(some("x"), &doc), some(CENTER_TITLE));
    assert_eq!(kept_subject(some("x"), &PageState::NotRead), some("x"));
    assert_eq!(
        kept_subject(some("x"), &PageState::Unread(nodearound::REASON)),
        some("x")
    );
    assert_eq!(kept_subject(some("x"), &PageState::NotFound), None);
    let no_center = edited(|d| d.rows.retain(|r| r.col != 0));
    assert_eq!(kept_subject(some("x"), &no_center), None);
    let blank = edited(|d| {
        for r in d.rows.iter_mut().filter(|r| r.col == 0) {
            r.node.title = " ".to_string();
        }
    });
    assert_eq!(kept_subject(some("x"), &blank), None);
}

/// (4) board.rs と node.rs の DOM の部分の字。
#[test]
fn ptitle_wiring_text() {
    let board = read("src/board.rs");
    for word in ["PageSubject", "doc_title(", "heading", "set_title("] {
        assert!(board.contains(word), "board.rs に {word} が無い");
    }
    assert!(!board.contains("board_title("), "board.rs に board_title( が残る");
    let node = read("src/project/node.rs");
    let at = node.find("mod dom {").expect("node.rs に mod dom { が無い");
    let dom = &node[at..];
    for word in ["PageSubject", "kept_subject(", "context"] {
        assert!(dom.contains(word), "node.rs の DOM の部分に {word} が無い");
    }
}

/// (6) 自分の file の test の名はどれも ptitle_ で始まり、filter の語を含まない。
#[test]
fn ptitle_own_names_clean() {
    let text = read("tests/ptitle.rs");
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
    assert!(names.len() >= 5, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("ptitle_")
            .unwrap_or_else(|| panic!("{name} が ptitle_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
