//! 行 g-help-fig の歯: 注釈の図の記号 `{fig:名}` を見本の figSVG と同じ並びの SVG の字にする（widgets の fig）・
//! 図の記号だけの行を help の fig_of で見分け、DOM の lines_view が li（class figli）に図を描く。

use std::collections::BTreeSet;
use std::path::PathBuf;

use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::fig::{self, NAMES, font_size};
use tsuzuri_surface::widgets::help::{self, fig_of, line};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn svg(name: &str) -> String {
    fig::svg(name).unwrap_or_else(|| panic!("図 {name} が None"))
}

/// stylesheet の class の名（frame の歯と同じ読み: comment を除き `.名` を拾う）。
fn stylesheet_classes() -> BTreeSet<String> {
    let css = read("style.css");
    let mut text = String::new();
    let mut rest = css.as_str();
    while let Some(i) = rest.find("/*") {
        text.push_str(&rest[..i]);
        rest = rest[i..].find("*/").map_or("", |j| &rest[i + j + 2..]);
    }
    text.push_str(rest);
    let chars: Vec<char> = text.chars().collect();
    let mut out = BTreeSet::new();
    for i in 0..chars.len() {
        let starts = chars
            .get(i + 1)
            .is_some_and(|c| c.is_ascii_alphabetic() || *c == '_' || *c == '-');
        if chars[i] == '.' && starts {
            let name: String = chars[i + 1..]
                .iter()
                .take_while(|c| c.is_ascii_alphanumeric() || **c == '-' || **c == '_')
                .collect();
            out.insert(name);
        }
    }
    out
}

/// 字の中の `名="…"` の値の全部（出た順）。
fn attrs(s: &str, name: &str) -> Vec<String> {
    let key = format!(" {name}=\"");
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(i) = rest.find(&key) {
        rest = &rest[i + key.len()..];
        let end = rest.find('"').expect("属性の値の終わり");
        out.push(rest[..end].to_string());
        rest = &rest[end..];
    }
    out
}

/// 最初の要素の開きの tag（`<` から `>` まで）。
fn head_tag(s: &str) -> &str {
    &s[..=s.find('>').expect("tag の終わり")]
}

#[test]
fn hfig_names_and_svg_some() {
    assert!(read("src/widgets/mod.rs").contains("pub mod fig;"));
    assert_eq!(
        NAMES,
        ["back", "move", "next", "reserve", "promo", "tick", "hover"]
    );
    for name in NAMES {
        assert!(fig::svg(name).is_some(), "{name} が None");
    }
    for name in ["", "fig", "Back"] {
        assert_eq!(fig::svg(name), None, "{name:?} が Some");
    }
}

#[test]
fn hfig_svg_head_and_counts() {
    // 名・高さ・rect の数・marker-end の数・stroke-dasharray の数。
    let table = [
        ("back", 104, 4, 4, 0),
        ("move", 92, 4, 3, 1),
        ("next", 58, 5, 4, 0),
        ("reserve", 100, 5, 4, 1),
        ("promo", 64, 4, 3, 1),
        ("tick", 50, 3, 2, 0),
        ("hover", 56, 3, 2, 0),
    ];
    for (name, h, rects, markers, dashes) in table {
        let s = svg(name);
        assert!(s.starts_with("<svg "), "{name}: {s}");
        assert!(s.ends_with("</svg>"), "{name}");
        let head = head_tag(&s);
        assert_eq!(attrs(head, "viewBox"), [format!("0 0 320 {h}")], "{name}");
        assert_eq!(attrs(head, "width"), ["320"], "{name}");
        assert_eq!(attrs(head, "height"), [h.to_string()], "{name}");
        assert_eq!(attrs(head, "role"), ["img"], "{name}");
        assert_eq!(attrs(head, "aria-label"), [format!("図: {name}")], "{name}");
        assert_eq!(s.matches("<rect ").count(), rects, "{name} の rect");
        assert_eq!(attrs(&s, "marker-end").len(), markers, "{name} の marker-end");
        assert_eq!(
            attrs(&s, "stroke-dasharray").len(),
            dashes,
            "{name} の stroke-dasharray"
        );
        assert!(
            s.contains(&format!(
                "<marker id=\"fa-{name}\" viewBox=\"0 0 8 8\" refX=\"7\" refY=\"4\" markerWidth=\"7\" markerHeight=\"7\" orient=\"auto\"><path d=\"M0 0L8 4L0 8z\" fill=\"var(--ink-3)\"/></marker>"
            )),
            "{name} の marker"
        );
        for m in attrs(&s, "marker-end") {
            assert_eq!(m, format!("url(#fa-{name})"));
        }
        // 座標は整数の字（小数点が出るのは font-size と stroke-width だけ）。
        for key in ["x", "y", "width", "height", "d"] {
            for v in attrs(&s, key) {
                assert!(!v.contains('.'), "{name} の {key}={v}");
            }
        }
    }
}

#[test]
fn hfig_line_ends() {
    let d = |name: &str| attrs(&svg(name), "d");
    let tick = d("tick");
    assert!(tick.contains(&"M86 25 L100 25".to_string()), "{tick:?}");
    assert!(tick.contains(&"M206 25 L220 25".to_string()), "{tick:?}");
    let mv = d("move");
    assert!(mv.contains(&"M136 38 L136 60".to_string()), "{mv:?}");
    let hover = d("hover");
    assert!(hover.contains(&"M114 27 L94 27".to_string()), "{hover:?}");
    let promo = d("promo");
    assert!(promo.contains(&"M212 19 L240 33".to_string()), "{promo:?}");
    assert!(
        svg("promo").contains(
            "<text x=\"102\" y=\"16\" font-size=\"9\" text-anchor=\"middle\" fill=\"var(--ink-3)\">昇格</text>"
        ),
        "promo の線の字"
    );
}

#[test]
fn hfig_font_sizes() {
    assert_eq!(font_size("この窓を閉じる", 80), "10.1");
    assert_eq!(font_size("memo", 80), "10.5");
    assert_eq!(font_size("≤ 2×周期 healthy", 106), "9.5");
    assert_eq!(font_size("b 応答なし", 56), "9.0");
    assert_eq!(font_size(&"あ".repeat(11), 56), "8.0");
    let tick = svg("tick");
    let sizes: Vec<String> = tick
        .split("<g ")
        .skip(1)
        .flat_map(|g| attrs(g, "font-size"))
        .collect();
    assert_eq!(sizes, ["10.5", "9.5", "10.4"]);
}

#[test]
fn hfig_classes_and_text() {
    let css = stylesheet_classes();
    for name in NAMES {
        let s = svg(name);
        for value in attrs(&s, "class") {
            for c in value.split_whitespace() {
                assert!(css.contains(c), "{name} の class {c} が style.css に無い");
            }
        }
        assert_eq!(attrs(head_tag(&s), "class"), ["fig"], "{name}");
    }
    let tick = svg("tick");
    let groups: Vec<String> = tick
        .split("<g ")
        .skip(1)
        .map(|g| attrs(&format!(" {}", head_tag(g)), "class").remove(0))
        .collect();
    assert_eq!(groups, ["fb", "fb fb-ok", "fb fb-ng"]);
    assert!(tick.contains(">&gt; 2×周期 stale</text>"), "{tick}");
    assert!(svg("hover").contains(">この節点</text>"));
    assert!(svg("next").contains(">← 左ほど優先 · 先頭の 1 つが大きく出る</text>"));
}

#[test]
fn hfig_fig_of_note_lines() {
    assert_eq!(fig_of(&line("{fig:next}")), Some("next"));
    for s in ["{st:run} 走っている", "a {fig:next}", "{st:run}", "{fig:}"] {
        assert_eq!(fig_of(&line(s)), None, "{s}");
    }
    let mut found = Vec::new();
    for key in vocab().keys() {
        let n = help::note(key).expect("語の注釈");
        for l in n.items.iter().chain(&n.more) {
            if let Some(name) = fig_of(l) {
                found.push((key.to_string(), name.to_string()));
            }
        }
    }
    found.sort();
    let want = [
        ("acct_back", "back"),
        ("candidates", "reserve"),
        ("lg_hover", "hover"),
        ("memo_promo", "promo"),
        ("move_state", "move"),
        ("next", "next"),
        ("next_all", "next"),
        ("tick_health", "tick"),
    ]
    .map(|(k, n)| (k.to_string(), n.to_string()));
    assert_eq!(found, want);
    for (_, name) in &found {
        assert!(fig::svg(name).is_some(), "{name}");
    }
}

#[test]
fn hfig_dom_source() {
    let text = read("src/widgets/help.rs");
    let at = text.find("mod dom {").expect("help.rs に mod dom");
    let dom = &text[at..];
    for s in ["fig_of", "fig::svg", "class=\"figli\""] {
        assert!(dom.contains(s), "DOM の部分に {s} が無い");
    }
}

/// 着地済みと同じ波の歯の名の接頭辞（自分の hfig_ は入れない）。
const FILTER: [&str; 65] = [
    "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
    "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
    "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
    "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_",
    "server_", "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_", "gnav_",
    "mkeys_", "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_", "ticker_",
    "pmore_", "lspark_", "apop_", "brand_", "runsdoc_", "nbatch_",
];

#[test]
fn hfig_test_names() {
    let text = read("tests/helpfig.rs");
    let mut names = Vec::new();
    let mut rest = text.as_str();
    // 属性の行と次の行の `fn `（この字の literal は逆斜線の escape で改行を持たず当たらない）。
    let mark = "\n#[test]\nfn ";
    while let Some(i) = rest.find(mark) {
        let after = &rest[i + mark.len()..];
        let end = after.find('(').expect("fn の名の終わり");
        names.push(after[..end].trim().to_string());
        rest = &after[end..];
    }
    assert!(names.len() >= 8, "{names:?}");
    for name in &names {
        let tail = name
            .strip_prefix("hfig_")
            .unwrap_or_else(|| panic!("{name} が hfig_ で始まらない"));
        for w in FILTER {
            assert!(!tail.contains(w), "{name} が {w} を含む");
        }
    }
}
