//! 便 c-q-blocking の歯（面）: 問いの card の止めている task の数の chip（card は電文の列を写す・
//! つながりの段の見出しで関わる所の chip の後・見本の IC.stop の図と語の辞書・歯の名）。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::question::{QuestionCard, QuestionList};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::ask;
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture_cards() -> Vec<QuestionCard> {
    let list: QuestionList =
        wire::decode(&read("../../tests/fixtures/surface/question-list.json")).expect("fixture");
    match list.cards {
        Reading::Known(cards) => cards,
        Reading::Unknown => panic!("fixture の card が Unknown"),
    }
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
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

fn squeeze(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn qblock_card_copies_ids() {
    let mut cards = fixture_cards();
    assert_eq!(cards.len(), 2);
    cards[0].blocking = strs(&["t-3", "t-4"]);
    assert_eq!(ask::card(1, &cards[0]).blocking, strs(&["t-3", "t-4"]));
    assert!(ask::card(2, &cards[1]).blocking.is_empty());
    let text = wire::encode(&QuestionList {
        cards: Reading::Known(cards),
        answerable: true,
    })
    .expect("encode");
    match ask::body(&Fetched::Body(text)) {
        Body::Filled(shown) => {
            assert_eq!(shown.len(), 2);
            assert_eq!(shown[0].blocking, strs(&["t-3", "t-4"]));
            assert!(shown[1].blocking.is_empty());
        }
        other => panic!("中身にならない: {other:?}"),
    }
}

#[test]
fn qblock_chip_after_touches() {
    let src = read("src/project/ask.rs");
    let arm = squeeze(span(&src, "Part::Around =>", "fn answer_view("));
    let touches = arm
        .find("data-term=\"touches\"")
        .expect("touches の chip");
    let close = arm.find("</summary>").expect("</summary>");
    assert!(touches < close, "{arm}");
    let mut at = touches;
    for needle in [
        "<spanclass=\"chipnum\"data-term=\"blocking\">",
        "<spaninner_html=STOP></span>",
        "{label(\"blocking\")}",
        "{card.blocking.len()}",
        "</span>",
    ] {
        let i = arm[at..]
            .find(needle)
            .unwrap_or_else(|| panic!("字 {needle} が順に在らない: {arm}"));
        at += i + needle.len();
    }
    assert!(at <= close, "chip は </summary> の前: {arm}");
    assert_eq!(src.matches("data-term=\"blocking\"").count(), 1);
}

#[test]
fn qblock_vocab_and_icon() {
    let ui = read("../../docs/design/mock3/ui.js");
    let lines: Vec<&str> = ui.lines().filter(|l| l.contains("stop: '<svg")).collect();
    assert_eq!(lines.len(), 1, "{lines:?}");
    let value = lines[0].split('\'').nth(1).expect("単引用符の間の字");
    let rest = value.strip_prefix("<svg").expect("<svg で始まる");
    let want = format!("<svg width=\"12\" height=\"12\"{rest}");
    let src = read("src/project/ask.rs");
    assert_eq!(src.matches("const STOP: &str").count(), 1);
    let decl = span(&src, "const STOP: &str", "\"#;");
    let got = decl
        .split_once("r#\"")
        .map(|(_, v)| v)
        .expect("r# の字");
    assert_eq!(got, want);
    let term = vocab().term("blocking").expect("鍵 blocking");
    assert_eq!(term.label, "止めている task");
}

/// 着地済みの行と第 3 波から第 7 波のほかの行の verify の filter の語（92）。
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
];

#[test]
fn qblock_own_names_clean() {
    assert_eq!(FILTERS.len(), 92);
    let distinct: std::collections::BTreeSet<&&str> = FILTERS.iter().collect();
    assert_eq!(distinct.len(), 92);
    let src = read("tests/qblock.rs");
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
    assert_eq!(names.len(), 4, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("qblock_")
            .unwrap_or_else(|| panic!("{name} は qblock_ で始まる"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} は {word} を含む");
        }
    }
}
