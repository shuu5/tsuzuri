//! 便 g-ask-around の歯: 問いの card のつながりの段に近傍の図を埋め込む（中心の id を引数で受ける口の path・
//! 開くまで空の path・見つからないも理由の 1 行・通信の module は空の path を読まない・nodearound と ask の字）。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::project::nodearound::{
    NO_NODE, PageState, REASON, center_path, embed_path, embed_reason, id_of, request, state,
    unmeasured_reason,
};
use tsuzuri_surface::project::{NO_CONTENT, NOT_READ};
use tsuzuri_surface::view::Fetched;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// `text` の `from` の最初の所から、その後の最初の `to` まで（`to` が無ければ終わりまで）。
fn span<'a>(text: &'a str, from: &str, to: &str) -> &'a str {
    let start = text
        .find(from)
        .unwrap_or_else(|| panic!("字 {from} が無い"));
    let rest = &text[start..];
    let end = rest[from.len()..]
        .find(to)
        .map_or(rest.len(), |i| from.len() + i);
    &rest[..end]
}

/// `text` の中で `needles` が順に在る（前の字の後に次の字を探す）ことを見て、最後の字の位置を返す。
fn in_order(text: &str, needles: &[&str]) -> usize {
    let mut at = 0;
    for n in needles {
        let i = text[at..]
            .find(n)
            .unwrap_or_else(|| panic!("字 {n} が順に在らない"));
        at += i + n.len();
    }
    at
}

#[test]
fn aaround_center_path_from_param() {
    let cases = [
        ("qa.2", "?page=ask&id=qa.10&mode=beginner", "/api/around?id=qa.2"),
        (
            "qa.2",
            "?page=ask&id=qa.10&k=3&fold=up&mode=expert",
            "/api/around?id=qa.2&k=3&fold=up",
        ),
        (
            "e.2:20260927T0000Z-1",
            "?k=1&fold=both",
            "/api/around?id=e.2%3A20260927T0000Z-1&k=1&fold=both",
        ),
        ("FR1", "?page=ask&k=9&fold=side", "/api/around?id=FR1"),
    ];
    for (id, query, want) in cases {
        assert_eq!(center_path(id, query), want, "{id} {query}");
    }
    for query in [
        "?page=node&id=FR1&k=3&fold=up&mode=expert",
        "?page=node&id=e.2%3A20260927T0000Z-1",
        "?page=node&id=FR1",
    ] {
        let id = id_of(query).expect(query);
        assert_eq!(request(query), Some(center_path(&id, query)), "{query}");
    }
    assert_eq!(request("?page=node"), None);
}

#[test]
fn aaround_lazy_path_until_opened() {
    let cases = [
        (
            "qa.2",
            "?page=ask&id=qa.10&k=3&mode=beginner",
            "/api/around?id=qa.2&k=3",
        ),
        ("qa.10", "?page=ask", "/api/around?id=qa.10"),
    ];
    for (id, query, want) in cases {
        assert_eq!(embed_path(id, query, false), "", "{id} {query}");
        assert_eq!(embed_path(id, query, true), want, "{id} {query}");
    }
}

#[test]
fn aaround_embed_reason_states() {
    assert_eq!(embed_reason(&PageState::NotRead), Some(NOT_READ));
    assert_eq!(embed_reason(&PageState::Unread(REASON)), Some(REASON));
    assert_eq!(embed_reason(&PageState::Unread(NO_CONTENT)), Some(NO_CONTENT));
    assert_eq!(embed_reason(&PageState::NotFound), Some(NO_NODE));
    let text = read("../../tests/fixtures/surface/around-doc.json");
    let doc = state(&Fetched::Body(text), Some(200));
    assert!(matches!(doc, PageState::Doc(_)), "{doc:?}");
    assert_eq!(embed_reason(&doc), None);
    assert_eq!(unmeasured_reason(&PageState::NotFound), None);
    assert!(!NO_NODE.trim().is_empty());
    assert!(!NO_NODE.contains('\n'));
    for other in [REASON, NOT_READ, NO_CONTENT] {
        assert_ne!(NO_NODE, other);
    }
}

#[test]
fn aaround_net_skips_empty_path() {
    let net = read("src/net.rs");
    let body = span(&net, "fn load_watch(", "spawn_local(");
    assert!(
        body.contains("if path.is_empty() {"),
        "load_watch は空の path を読まない: {body}"
    );
}

#[test]
fn aaround_src_embeds_in_nodearound() {
    let src = read("src/project/nodearound.rs");
    let start = src.find("mod dom {").expect("mod dom {");
    let dom = &src[start..];
    for word in ["pub struct Embeds", "pub fn open(self", "embed_path("] {
        assert!(dom.contains(word), "mod dom に {word}");
    }
    let embeds = span(&src, "pub fn embeds(", "fn ");
    assert!(embeds.contains("Owner::current()"), "{embeds}");
    let view = span(&src, "pub fn view(self", "fn ");
    for word in ["state(", "embed_reason(", "unmeasured(", "panel("] {
        assert!(view.contains(word), "Embeds の view に {word}: {view}");
    }
    let uses = span(&src, "pub use dom::{", "};");
    assert!(uses.contains("Embeds"), "{uses}");
    assert!(uses.contains("embeds"), "{uses}");
    assert_eq!(src.matches("layout(&doc").count(), 1);
    assert_eq!(src.matches("svg(&doc").count(), 1);
    assert_eq!(src.matches("<details").count(), 0);
}

#[test]
fn aaround_src_ask_fold_embeds() {
    let src = read("src/project/ask.rs");
    let head = span(&src, "pub fn view() -> AnyView {", "let list = move ||");
    assert!(head.contains("embeds()"), "{head}");
    let arm = span(&src, "Part::Around =>", "fn answer_view(");
    let at = in_order(
        arm,
        &[
            "format!(\"ask:around:{}\"",
            "has_attribute(",
            ".open(",
            "<details",
            "class=\"nb-body\"",
        ],
    );
    let rest = &arm[at..];
    let view = rest.find(".view(").expect("nb-body の後に .view(");
    let close = rest.find("</details>").expect("nb-body の後に </details>");
    assert!(view < close, "図は nb-body の中: {rest}");
    assert_eq!(arm.matches("<details").count(), 1);
    assert!(!arm.contains("class=\"nid\""), "{arm}");
    assert!(!arm.contains("<ul class=\"items\""), "{arm}");
    assert!(!src.contains("mapview::around"));
    assert!(!src.contains("read_path("));
}

/// 着地済みの行と第 3 波から第 5 波の行の verify の filter の語（78）。
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
    "ntime_",
    "ptitle_",
    "ncard_",
    "hsym_",
    "smore_",
    "lcard_",
    "hruling_",
    "sxaxis_",
    "plimit_",
    "bport_",
];

#[test]
fn aaround_own_names_clean() {
    assert_eq!(FILTERS.len(), 78);
    let src = read("tests/aaround.rs");
    let lines: Vec<&str> = src.lines().collect();
    let mut names = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() != "#[test]" {
            continue;
        }
        let next = lines
            .iter()
            .skip(i + 1)
            .find(|l| l.trim_start().starts_with("fn "))
            .expect("test の属性の次の fn");
        let name = next
            .trim_start()
            .trim_start_matches("fn ")
            .split('(')
            .next()
            .unwrap_or_default()
            .to_string();
        names.push(name);
    }
    assert!(names.len() >= 7, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("aaround_")
            .unwrap_or_else(|| panic!("{name} は aaround_ で始まる"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} は {word} を含む");
        }
    }
}
