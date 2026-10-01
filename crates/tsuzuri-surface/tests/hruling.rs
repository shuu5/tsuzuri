//! 便 g-hist-ruling の歯: 段の行の右に、その問いに答えた決定の頁への link（字は決定の時刻）を置く。
//! グラフの口の answers の辺から問いごとの決定・段の行への添え・link の字の形・view の字の形・歯の名。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::graph::{EdgeType, GraphDoc, GraphEdge};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::{Body, Item, askpage};
use tsuzuri_surface::view::Fetched;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn graph_text() -> String {
    read("../../tests/fixtures/surface/graph-doc.json")
}

fn ledger_text() -> String {
    read("../../tests/fixtures/surface/ledger-questions.json")
}

fn edge(from: &str, to: &str, edge_type: EdgeType) -> GraphEdge {
    GraphEdge {
        from: from.to_string(),
        to: to.to_string(),
        edge_type,
    }
}

/// graph-doc.json の edges の終わりに 4 本を足した電文。
fn built() -> Fetched {
    let mut doc: GraphDoc = wire::decode(&graph_text()).expect("fixture の電文");
    doc.edges.extend([
        edge("lq.9:20260927T1105Z-1", "lq.9", EdgeType::Answers),
        edge("lq.10:20260927T1200Z-1", "lq.10", EdgeType::Answers),
        edge("lq.9:20260927T1300Z-1", "lq.9", EdgeType::Answers),
        edge("lq.4", "lq.2", EdgeType::Touches),
    ]);
    Fetched::Body(wire::encode(&doc).expect("電文"))
}

fn items() -> Vec<Item> {
    match askpage::body(&Fetched::Body(ledger_text())) {
        Body::Filled(items) => items,
        other => panic!("fixture の項が中身にならない: {other:?}"),
    }
}

fn pairs(list: &[(&str, &str)]) -> BTreeMap<String, String> {
    list.iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// answers の辺ごとに鍵 to・値 from（touches は数えない）・読めない本文は空。
#[test]
fn hruling_links_from_answers_edges() {
    assert_eq!(
        askpage::ruling_links(&Fetched::Body(graph_text())),
        pairs(&[("t3.q2", "t3.q2#r")])
    );
    for f in [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("not json".to_string()),
        Fetched::Body(ledger_text()),
    ] {
        assert!(askpage::ruling_links(&f).is_empty(), "{f:?}");
    }
}

/// 同じ問いへの answers の辺が 2 本なら電文の順の最後の from。
#[test]
fn hruling_last_answer_wins() {
    assert_eq!(
        askpage::ruling_links(&built()),
        pairs(&[
            ("t3.q2", "t3.q2#r"),
            ("lq.9", "lq.9:20260927T1300Z-1"),
            ("lq.10", "lq.10:20260927T1200Z-1"),
        ])
    );
}

/// 段の行は項の順のまま、項の写しと答えた決定の id。
#[test]
fn hruling_rows_carry_ruling() {
    let items = items();
    let rows = askpage::hist_rows(&items, &built());
    let ids: Vec<&str> = rows.iter().map(|r| r.item.id.as_str()).collect();
    assert_eq!(ids, vec!["lq.2", "lq.9", "lq.10"]);
    let got: Vec<Item> = rows.iter().map(|r| r.item.clone()).collect();
    assert_eq!(got, items);
    let rulings: Vec<Option<&str>> = rows.iter().map(|r| r.ruling.as_deref()).collect();
    assert_eq!(
        rulings,
        vec![
            None,
            Some("lq.9:20260927T1300Z-1"),
            Some("lq.10:20260927T1200Z-1")
        ]
    );
    for f in [Fetched::NotRead, Fetched::Body(graph_text())] {
        let rows = askpage::hist_rows(&items, &f);
        assert_eq!(rows.len(), 3, "{f:?}");
        for (r, it) in rows.iter().zip(&items) {
            assert_eq!(
                *r,
                askpage::HistEntry {
                    item: it.clone(),
                    ruling: None
                }
            );
        }
    }
}

/// server の形の id は UTC の時分を日本時間にした時分と JST・ほかの形は id のまま（行 g-jst）。
#[test]
fn hruling_link_text_forms() {
    for (id, want) in [
        ("lq.9:20260927T1300Z-1", "22:00 JST"),
        ("t3-hub.52.16:20260927T2233Z-1", "07:33 JST"),
        ("fx.1:20260927T1034Z-12", "19:34 JST"),
        ("a.b:c:20260927T0905Z-2", "18:05 JST"),
    ] {
        assert_eq!(askpage::ruling_text(id), want, "{id}");
    }
    for id in [
        "t3.q2#r",
        "user 2026-09-27T03:19Z",
        "fx.1:20260927T1034Z",
        "fx.1:20260927T1034Z-",
        "fx.1:2026092T1034Z-1",
        "fx.1:20260927T10345Z-1",
        "fx.1:20260927t1034Z-1",
        "fx.1:20260927T1034Z-1x",
        ":20260927T1034Z-1",
        "",
    ] {
        assert_eq!(askpage::ruling_text(id), id, "{id}");
    }
}

/// view の字: 行は hist_rows・グラフの口・題の link を持ち item_view を使わず、決定の link は aside の span の中の a。
#[test]
fn hruling_src_row_links() {
    let text = read("src/project/askpage.rs");
    let start = text.find("pub fn view(").expect("view の宣言");
    let v = &text[start..];
    assert!(v.contains("hist_rows("));
    assert!(v.contains("map::PATH"));
    assert!(v.contains("<a class=\"ttl\""));
    assert!(!v.contains("item_view("));
    let s = v.find("<span class=\"aside\"").expect("aside の span");
    let r = v.find("ruling_text(").expect("ruling_text の呼び");
    assert!(s < r, "ruling_text の呼びが aside の span より前");
    assert!(!v[s..r].contains("</span>"), "link が aside の span の外");
    let a = v[..r].rfind("<a ").expect("決定の link の a");
    assert!(a > s, "決定の link の a が aside の span の外");
    assert!(v[a..r].contains("node_href("), "href が node_href でない");
    assert!(!v[a..r].contains("</a>"), "字が a の外");
}

/// この file の歯の名はどれも hruling_ で始まり、残りの字はほかの行の filter の語を含まない。
#[test]
fn hruling_own_names_clean() {
    let words = [words_head(), words_tail()].concat();
    names_clean(&words);
}

/// ほかの行の filter の語の前半。
fn words_head() -> &'static [&'static str] {
    &[
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
    ]
}

/// ほかの行の filter の語の後半。
fn words_tail() -> &'static [&'static str] {
    &[
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
        "aaround_",
        "sxaxis_",
        "plimit_",
        "bport_",
    ]
}

/// 語の数と、この file の歯の名が hruling_ で始まり残りの字が語を含まないこと。
fn names_clean(words: &[&str]) {
    assert_eq!(words.len(), 78);
    let text = read("tests/hruling.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let mut names = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if *line != "#[test]" {
            continue;
        }
        let next = lines.get(i + 1).expect("test の属性の次の行");
        let name = next
            .strip_prefix("fn ")
            .and_then(|r| r.split('(').next())
            .unwrap_or_else(|| panic!("test の属性の次が fn でない: {next}"));
        names.push(name.to_string());
    }
    assert_eq!(names.len(), 6, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("hruling_")
            .unwrap_or_else(|| panic!("{name} が hruling_ で始まらない"));
        for w in words {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
