//! 行 h-sess-axis の歯: account board の session の表の稼働の記録の見出しの欄と目盛の段
//! （C_HH・SAXIS・SAXIS_M・axis_labels と mod dom の axis_view）と、段の class が stylesheet に在ることと、この file の歯の名。
#![cfg(test)]

use crate::common::{AT_OFF, crate_dir};
use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::session::{C_HH, COLUMNS, SAXIS, SAXIS_M, Sort, axis_labels, table};
use tsuzuri_surface::project::seat::{Span, span_ticks};
use tsuzuri_surface::vocab::vocab;

/// fixture の電文の at（2026-09-27 12:00Z・日本時間の 21:00・正時）。
const AT: u64 = 1_790_510_400;

/// UTC の日の変わり目の時点（2026-09-27 00:00Z・日本時間の 09:00）。
const AT_DAY: u64 = 1_790_467_200;

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture() -> AccountDoc {
    wire::decode(&read("../../tests/fixtures/account/acct-doc.json"))
        .expect("fixture が AccountDoc として読める")
}

fn want(v: &[(&str, &str)]) -> Vec<(String, String)> {
    v.iter()
        .map(|(s, l)| (s.to_string(), l.to_string()))
        .collect()
}

/// 空白を除いた字。
fn squeeze(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// (1) 3 つの class の値と 7 つの列、語の鍵 history、stylesheet の 7 つの字。
#[test]
fn sxaxis_classes_in_stylesheet() {
    assert_eq!(C_HH, "c-hh");
    assert_eq!(SAXIS, "saxis");
    assert_eq!(SAXIS_M, "saxis-m");
    assert_eq!(
        COLUMNS,
        [
            "project", "role", "session", "accounts", "stage", "elapsed", "history"
        ]
    );
    let term = vocab().term("history").expect("鍵 history が vocab に無い");
    assert!(!term.label.is_empty(), "鍵 history の語が空");

    let css = read("style.css");
    for needle in [
        ".c-hh {",
        ".c-hh .saxis {",
        ".saxis {",
        ".saxis span {",
        ".saxis-m { display: none; }",
        "@media (max-width: 760px) { .saxis-m { display: block;",
        ".srow.hrow { display: none; }",
    ] {
        assert!(css.contains(needle), "stylesheet に {needle} が無い");
    }
}

/// (2) 目盛の対は span_ticks の順に style の字と時刻、表の at は電文の at。
#[test]
fn sxaxis_labels_follow_seat_ticks() {
    assert_eq!(
        axis_labels(AT, Span::H24),
        want(&[
            ("left:12.50%", "00:00"),
            ("left:37.50%", "06:00"),
            ("left:62.50%", "12:00"),
            ("left:87.50%", "18:00"),
        ])
    );
    assert_eq!(
        axis_labels(AT, Span::H6),
        want(&[
            ("left:0.00%", "15:00"),
            ("left:33.33%", "17:00"),
            ("left:66.67%", "19:00"),
        ])
    );
    assert_eq!(
        axis_labels(AT, Span::H3),
        want(&[
            ("left:0.00%", "18:00"),
            ("left:33.33%", "19:00"),
            ("left:66.67%", "20:00"),
        ])
    );
    for at in [AT_OFF, AT_DAY] {
        for span in Span::ALL {
            let ticks: Vec<(String, String)> = span_ticks(at, span)
                .into_iter()
                .map(|t| (format!("left:{}%", t.left), t.label))
                .collect();
            assert!(!ticks.is_empty(), "at {at} の幅 {}", span.key());
            assert_eq!(axis_labels(at, span), ticks, "at {at} の幅 {}", span.key());
        }
    }

    let doc = fixture();
    assert_eq!(doc.at, AT);
    for sort in Sort::ALL {
        assert_eq!(table(&doc, sort).at, AT, "並べ方 {}", sort.key());
    }
}

/// (3) 見出しの行は 6 つの語の欄と C_HH の欄で閉じ、SAXIS_M の段は見出しの行の外で束より前。
/// (4) axis_view は SAXIS の段に axis_labels の対ごとの span を置く。
#[test]
fn sxaxis_dom_places_both_axes() {
    let src = read("src/account/session.rs");
    let dom = squeeze(&src[src.find("mod dom {").expect("mod dom")..]);
    assert!(dom.contains("COLUMNS[..6]"), "mod dom に COLUMNS[..6] が無い");
    let hs = "{hs(\"history\")}";
    let axis = "{axis_view(at,span)}";
    let chain = format!(
        "<divclass=HROW>{{head}}<divclass=C_HH>{hs}{axis}</div></div><divclass=SAXIS_M>{hs}{axis}</div>{{groups}}"
    );
    assert!(dom.contains(&chain), "mod dom に {chain} が無い");
    assert_eq!(src.matches("class=C_HH").count(), 1);
    assert_eq!(src.matches("class=SAXIS_M").count(), 1);

    let start = src.find("fn axis_view(").expect("fn axis_view が無い");
    let rest = &src[start..];
    let end = [rest.find("\n    fn "), rest.find("\n    pub fn ")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(rest.len());
    let body = squeeze(&rest[..end]);
    for needle in [
        "<divclass=SAXISaria-hidden=\"true\">",
        "move||",
        "axis_labels(at,span.get())",
        "<spanstyle=",
    ] {
        assert!(body.contains(needle), "axis_view に {needle} が無い: {body}");
    }
}

/// 着地済みの行と同じ波のほかの行の verify の filter の語。
const FILTERS: [&str; 78] = [
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
    "aaround_",
    "hruling_",
    "plimit_",
    "bport_",
];

/// (6) この file の歯の名は sxaxis_ で始まり、残りの字はほかの行の filter の語を含まない。
#[test]
fn sxaxis_own_names_clean() {
    let src = read("tests/teeth5/sxaxis.rs");
    let lines: Vec<&str> = src.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1]
                .trim()
                .strip_prefix("fn ")
                .unwrap_or_else(|| panic!("test の属性の次が fn でない: {}", w[1]));
            &rest[..rest.find('(').expect("fn の名の終わり")]
        })
        .collect();
    assert_eq!(names.len(), 4, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("sxaxis_")
            .unwrap_or_else(|| panic!("歯 {name} が sxaxis_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "歯 {name} が filter の語 {word} を含む");
        }
    }
}
