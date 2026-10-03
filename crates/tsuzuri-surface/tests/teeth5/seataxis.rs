//! 行 g-seat-axis の歯: 稼働の記録の幅ごとの目盛の時刻（span_ticks・Strip の ticks）と、帯の色と縦線の凡例
//! （LEGEND・sample_svg）と、mod dom がそれを描く字と、段の class が stylesheet に在ることと、この file の歯の名。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::seat::SeatCard;
use tsuzuri_contract::wire;
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::seat::{LEGEND, Span, content, sample_svg, span_ticks};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

/// fixture の組の時点（2026-09-27 12:00Z・日本時間の 21:00・正時）。
const AT: u64 = 1_790_510_400;

/// 正時でない時点（2026-09-27 12:40Z・日本時間の 21:40）。
const AT_OFF: u64 = 1_790_512_800;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 目盛の left と label の字の対。
fn pairs(at: u64, span: Span) -> Vec<(String, String)> {
    span_ticks(at, span)
        .into_iter()
        .map(|t| (t.left, t.label))
        .collect()
}

fn want(v: &[(&str, &str)]) -> Vec<(String, String)> {
    v.iter()
        .map(|(l, t)| (l.to_string(), t.to_string()))
        .collect()
}

/// (1) 幅ごとの目盛の間と、正時の時点の目盛（右端の 100.00 は落ちる・6 を越えれば偶数の位置だけ）。
#[test]
fn saxis_ticks_on_the_hour() {
    assert_eq!(Span::H24.tick(), 21_600);
    assert_eq!(Span::H6.tick(), 3_600);
    assert_eq!(Span::H3.tick(), 1_800);
    assert_eq!(
        pairs(AT, Span::H24),
        want(&[
            ("12.50", "00:00"),
            ("37.50", "06:00"),
            ("62.50", "12:00"),
            ("87.50", "18:00"),
        ])
    );
    assert_eq!(
        pairs(AT, Span::H6),
        want(&[("0.00", "15:00"), ("33.33", "17:00"), ("66.67", "19:00")])
    );
    assert_eq!(
        pairs(AT, Span::H3),
        want(&[("0.00", "18:00"), ("33.33", "19:00"), ("66.67", "20:00")])
    );
}

/// (1) 正時でない時点の目盛（92 を越える目盛は落ちる）。
#[test]
fn saxis_ticks_off_the_hour() {
    assert_eq!(
        pairs(AT_OFF, Span::H24),
        want(&[
            ("9.72", "00:00"),
            ("34.72", "06:00"),
            ("59.72", "12:00"),
            ("84.72", "18:00"),
        ])
    );
    assert_eq!(
        pairs(AT_OFF, Span::H6),
        want(&[
            ("5.56", "16:00"),
            ("22.22", "17:00"),
            ("38.89", "18:00"),
            ("55.56", "19:00"),
            ("72.22", "20:00"),
            ("88.89", "21:00"),
        ])
    );
    assert_eq!(
        pairs(AT_OFF, Span::H3),
        want(&[
            ("11.11", "19:00"),
            ("27.78", "19:30"),
            ("44.44", "20:00"),
            ("61.11", "20:30"),
            ("77.78", "21:00"),
        ])
    );
}

/// (2) fixture の 5 組で、3 つの幅の Strip の ticks は span_ticks に電文の at とその幅を渡した値と同じ。
#[test]
fn saxis_strip_carries_ticks() {
    let cards: BTreeMap<String, SeatCard> =
        wire::decode(&read("../../tests/fixtures/surface/seat-card.json"))
            .expect("fixture の組が電文として読める");
    assert_eq!(cards.len(), 5);
    for (name, card) in cards {
        let text = wire::encode(&card).expect("電文");
        let Body::Filled(seat) = content(&Fetched::Body(text), card.at) else {
            panic!("組 {name} が中身を出さない");
        };
        for span in Span::ALL {
            assert_eq!(
                seat.strip(span).ticks,
                span_ticks(card.at, span),
                "組 {name} の幅 {}",
                span.key()
            );
            assert!(!seat.strip(span).ticks.is_empty(), "組 {name}");
        }
    }
}

/// 属性の字（名と等号と引用符で囲んだ値）を持つ。
fn has_attr(svg: &str, name: &str, value: &str) -> bool {
    svg.contains(&format!(" {name}=\"{value}\""))
}

/// 中の要素の開きの字から閉じの字まで。
fn element<'a>(svg: &'a str, tag: &str) -> &'a str {
    let open = format!("<{tag} ");
    let i = svg
        .find(&open)
        .unwrap_or_else(|| panic!("{tag} が無い: {svg}"));
    let rest = &svg[i..];
    &rest[..rest.find("/>").expect("要素の閉じ") + 2]
}

/// (3) 凡例の 7 つの記号の名と語の鍵、見本の svg の属性の字。
#[test]
fn saxis_legend_samples() {
    assert_eq!(
        LEGEND,
        [
            ("run", "lg_run"),
            ("wait", "lg_wait"),
            ("silent", "lg_silent"),
            ("limit", "lg_limit"),
            ("unknown", "lg_unknown"),
            ("acct", "lg_acct"),
            ("now", "lg_now"),
        ]
    );
    for (_, key) in LEGEND {
        assert!(vocab().term(key).is_some(), "鍵 {key} が vocab に無い");
    }
    for name in ["run", "wait", "acct", "now"] {
        let svg = sample_svg(name);
        assert!(svg.starts_with("<svg "), "{svg}");
        assert!(svg.ends_with("</svg>"), "{svg}");
        let head = &svg[..svg.find('>').expect("開きの終わり")];
        assert!(has_attr(head, "class", "ssample"), "{svg}");
        assert!(has_attr(head, "viewBox", "0 0 28 12"), "{svg}");
        assert!(has_attr(head, "width", "28"), "{svg}");
        assert!(has_attr(head, "height", "12"), "{svg}");
        assert!(has_attr(head, "aria-hidden", "true"), "{svg}");
    }
    sample_shapes();
}

/// 見本の svg の中の rect と line の class と位置の属性、記号でない名の見本は図を持たない。
fn sample_shapes() {
    let run = sample_svg("run");
    let r = element(&run, "rect");
    assert!(has_attr(r, "class", "sg-run"), "{r}");
    assert!(has_attr(r, "y", "0"), "{r}");
    assert!(has_attr(r, "height", "12"), "{r}");
    let wait = sample_svg("wait");
    let r = element(&wait, "rect");
    assert!(has_attr(r, "class", "sg-wait"), "{r}");
    assert!(has_attr(r, "y", "6.6"), "{r}");
    assert!(has_attr(r, "height", "5.4"), "{r}");
    let acct = sample_svg("acct");
    let l = element(&acct, "line");
    assert!(has_attr(l, "class", "mk mk-acct"), "{l}");
    assert!(has_attr(l, "x1", "14"), "{l}");
    let now = sample_svg("now");
    let l = element(&now, "line");
    assert!(has_attr(l, "class", "mk mk-now"), "{l}");
    assert!(has_attr(l, "x1", "14"), "{l}");
    let other = sample_svg("spawn");
    assert!(!other.contains("<rect") && !other.contains("<line"), "{other}");
}

/// (4) mod dom（src/project_dom/seat.rs）が目盛と凡例を描く字を持つ。(5) 段の class は stylesheet に在る。
#[test]
fn saxis_src_draws_legend_axis() {
    let dom = read("src/project_dom/seat.rs");
    for needle in [
        "class=\"saxis\"",
        "class=\"sleg\"",
        "class=\"sleg-i\"",
        "sample_svg(",
        "ticks",
        "LEGEND",
    ] {
        assert!(dom.contains(needle), "mod dom に {needle} が無い");
    }
    let css = read("style.css");
    for class in [
        "saxis", "sleg", "sleg-i", "ssample", "sg-run", "sg-wait", "mk", "mk-acct", "mk-now",
    ] {
        let dot = format!(".{class}");
        let found = css.match_indices(&dot).any(|(i, _)| {
            css[i + dot.len()..]
                .chars()
                .next()
                .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        });
        assert!(found, "stylesheet に .{class} の規則が無い");
    }
}

/// 着地済みの行と同じ波の行の verify の filter の語。
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
    "hook_",
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
    "hbproc_",
    "hbconf_",
    "hbroute_",
    "hbpost_",
    "hsblock_",
    "hspage_",
    "hsderive_",
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
    "ilink_",
    "afocus_",
    "nxact_",
    "urpanel_",
    "ticker_",
    "hfig_",
    "pmore_",
    "lspark_",
    "apop_",
    "brand_",
    "runsdoc_",
    "nbatch_",
];

/// (7) この file の歯の名は saxis_ で始まり、残りの字はほかの行の filter の語を含まない。
#[test]
fn saxis_own_names_clean() {
    let src = read("tests/teeth5/seataxis.rs");
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
    assert_eq!(names.len(), 6, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("saxis_")
            .unwrap_or_else(|| panic!("歯 {name} が saxis_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "歯 {name} が filter の語 {word} を含む");
        }
    }
}
