//! 行 g-help-sym の歯: 注釈の記号の見本（gi・nxm・thr・tk・band）を実物と同じ class の字にする（help の sym_html）・
//! 語 history の注釈の置き換えの字（`{SPAN}` と `{TICK}`）を URL の query の幅の字に替える（help の note_in）。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use tsuzuri_surface::mapview::band::Band;
use tsuzuri_surface::project::seat::Span;
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::help::{
    Inline, Line, fill, note, note_in, span_words, sym_html, tick_words,
};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn text(s: &str) -> Inline {
    Inline::Text(s.to_string())
}

fn html(sym: &str) -> String {
    sym_html(sym).unwrap_or_else(|| panic!("{sym} が None"))
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

/// 字の中の ` class="…"` の値の全部（出た順）。
fn class_values(s: &str) -> Vec<String> {
    let key = " class=\"";
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(i) = rest.find(key) {
        rest = &rest[i + key.len()..];
        let end = rest.find('"').expect("属性の値の終わり");
        out.push(rest[..end].to_string());
        rest = &rest[end..];
    }
    out
}

/// 片の並びの記号の名の全部。
fn syms(parts: &[Inline]) -> impl Iterator<Item = &str> {
    parts.iter().filter_map(|p| match p {
        Inline::Sym(s) => Some(s.as_str()),
        _ => None,
    })
}

#[test]
fn hsym_span_and_tick_words() {
    let words: Vec<(String, String)> = Span::ALL
        .into_iter()
        .map(|s| (span_words(s), tick_words(s)))
        .collect();
    let want = [("24 時間", "6 時間"), ("6 時間", "1 時間"), ("3 時間", "30 分")]
        .map(|(a, b)| (a.to_string(), b.to_string()));
    assert_eq!(words, want);
    assert_eq!(
        fill("a {SPAN} b {TICK} c {SPAN}", Span::H6),
        "a 6 時間 b 1 時間 c 6 時間"
    );
    assert_eq!(fill("{span} {Tick} {SPAN", Span::H3), "{span} {Tick} {SPAN");
    assert_eq!(fill("", Span::H24), "");
}

#[test]
fn hsym_note_fills_history() {
    assert_eq!(note_in("no-such-key", ""), None);
    let head = |q: &str| note_in("history", q).expect("history の注釈").head;
    let say = |span: &str| vec![text(&format!("直近 {span}の session の状態を帯で見る"))];
    assert_eq!(head(""), say("24 時間"));
    assert_eq!(head("?span=9h"), say("24 時間"));
    assert_eq!(head("?span=6h"), say("6 時間"));
    assert_eq!(head("span=3h&mode=expert"), say("3 時間"));
    let n = note_in("history", "?span=6h").expect("history の注釈");
    assert_eq!(
        n.more.get(4),
        Some(&Line {
            sym: vec![Inline::Sym("strip:ticks".to_string())],
            text: vec![text("目盛 = 1 時間ごと")],
        })
    );
    assert_eq!(
        n.more.get(5),
        Some(&Line {
            sym: vec![text("←")],
            text: vec![text("左端 = 6 時間前")],
        })
    );
    for key in vocab().keys() {
        let term = vocab().term(key).expect("語");
        let plain = !(term.note.contains("{SPAN}")
            || term.note.contains("{TICK}")
            || term.internal.contains("{SPAN}")
            || term.internal.contains("{TICK}"));
        for q in ["span=24h", "?span=6h", "span=3h"] {
            let n = note_in(key, q).expect("語の注釈");
            let lines = n.items.iter().chain(&n.more);
            let all: Vec<&str> = syms(&n.head)
                .chain(lines.flat_map(|l| syms(&l.sym).chain(syms(&l.text))))
                .chain(n.internal.iter().flat_map(|l| syms(l)))
                .collect();
            for s in all {
                assert!(s != "SPAN" && s != "TICK", "{key} の {q} に {s}");
            }
            if plain {
                assert_eq!(Some(n), note(key), "{key} の {q}");
            }
        }
    }
}

#[test]
fn hsym_html_exact_strings() {
    let table = [
        ("gi:ok", "<span class=\"gi ok sm\" aria-hidden=\"true\">✓</span>"),
        ("gi:ng", "<span class=\"gi ng sm\" aria-hidden=\"true\">!</span>"),
        (
            "gi:unknown",
            "<span class=\"gi unknown sm\" aria-hidden=\"true\"></span>",
        ),
        ("nxm:on:c", "<span class=\"nxm on\" aria-hidden=\"true\">c</span>"),
        ("nxm:off:d", "<span class=\"nxm off\" aria-hidden=\"true\">d</span>"),
        ("nxm:na:e", "<span class=\"nxm na\" aria-hidden=\"true\">e</span>"),
        ("nxm:on", "<span class=\"nxm on\" aria-hidden=\"true\">c</span>"),
        ("nxm:off:", "<span class=\"nxm off\" aria-hidden=\"true\">c</span>"),
        ("nxm:na:7", "<span class=\"nxm na\" aria-hidden=\"true\">7</span>"),
        ("thr:85", "<span class=\"thrs\" aria-hidden=\"true\"><i></i>85</span>"),
        ("thr:0", "<span class=\"thrs\" aria-hidden=\"true\"><i></i>0</span>"),
        (
            "tk:healthy",
            "<span class=\"tkhb smpl\"><span class=\"tk tk-healthy\"><span class=\"gi ok sm\" aria-hidden=\"true\">✓</span><b>healthy</b></span></span>",
        ),
        (
            "tk:stale",
            "<span class=\"tkhb smpl\"><span class=\"tk tk-stale\"><span class=\"gi ng sm\" aria-hidden=\"true\">!</span><b>stale</b></span></span>",
        ),
        (
            "tk:absent",
            "<span class=\"tkhb smpl\"><span class=\"tk\"><span class=\"gi unknown sm\" aria-hidden=\"true\"></span><b>absent</b></span></span>",
        ),
        (
            "tk:unreadable",
            "<span class=\"tkhb smpl\"><span class=\"tk\"><span class=\"gi ng sm\" aria-hidden=\"true\">!</span><b>unreadable</b></span></span>",
        ),
        (
            "band:constitution",
            "<span class=\"shape band-constitution fill\" aria-hidden=\"true\"></span>",
        ),
        (
            "band:design-note",
            "<span class=\"shape band-design-note fill\" aria-hidden=\"true\"></span>",
        ),
    ];
    for (sym, want) in table {
        assert_eq!(sym_html(sym).as_deref(), Some(want), "{sym}");
    }
    for b in Band::ALL {
        let want = format!(
            "<span class=\"shape {} fill\" aria-hidden=\"true\"></span>",
            b.class_name()
        );
        assert_eq!(sym_html(&format!("band:{}", b.name())), Some(want));
    }
    for sym in [
        "gi:", "gi:OK", "gi:ok:x", "gi", "nxm:", "nxm:up", "nxm:on:cd", "nxm:on:-", "nxm:on:あ",
        "nxm:on:c:d", "thr:", "thr:8a", "thr:-1", "thr:8.5", "tk:", "tk:fresh", "tk:healthy:x",
        "band:", "band:adr", "band:Beads", "band:x", "st:run", "strip:ticks", "fig:next", "edge:x",
        "hb:ok", "SPAN", "TICK", "", "ok",
    ] {
        assert_eq!(sym_html(sym), None, "{sym:?} が Some");
    }
}

#[test]
fn hsym_vocab_tokens_all_drawn() {
    let heads = ["gi:", "nxm:", "thr:", "tk:", "band:"];
    let mut found = BTreeSet::new();
    for key in vocab().keys() {
        let n = note(key).expect("語の注釈");
        let lines = n.items.iter().chain(&n.more);
        let all: Vec<String> = syms(&n.head)
            .chain(lines.flat_map(|l| syms(&l.sym).chain(syms(&l.text))))
            .chain(n.internal.iter().flat_map(|l| syms(l)))
            .map(str::to_string)
            .collect();
        found.extend(
            all.into_iter()
                .filter(|s| heads.iter().any(|h| s.starts_with(h))),
        );
    }
    let want: BTreeSet<String> = [
        "gi:ok",
        "gi:ng",
        "gi:unknown",
        "nxm:on:c",
        "nxm:off:d",
        "nxm:na:e",
        "thr:85",
        "thr:95",
        "tk:healthy",
        "tk:stale",
        "tk:absent",
        "tk:unreadable",
        "band:constitution",
        "band:beads",
    ]
    .map(str::to_string)
    .into();
    assert_eq!(found, want);
    for s in &found {
        assert!(sym_html(s).is_some(), "{s} が None");
    }
}

#[test]
fn hsym_classes_in_stylesheet() {
    let css = stylesheet_classes();
    let mut names: Vec<String> = [
        "gi:ok",
        "gi:ng",
        "gi:unknown",
        "nxm:on",
        "nxm:off",
        "nxm:na",
        "thr:85",
        "tk:healthy",
        "tk:stale",
        "tk:absent",
        "tk:unreadable",
    ]
    .map(str::to_string)
    .into();
    names.extend(Band::ALL.map(|b| format!("band:{}", b.name())));
    let mut seen = 0;
    for sym in &names {
        for value in class_values(&html(sym)) {
            for c in value.split_whitespace() {
                seen += 1;
                assert!(css.contains(c), "{sym} の class {c} が style.css に無い");
            }
        }
    }
    assert!(seen > names.len(), "集めた class が少ない: {seen}");
}

#[test]
fn hsym_dom_source() {
    let text = read("src/widgets/help.rs");
    let at = text.find("mod dom {").expect("help.rs に mod dom");
    let dom = &text[at..];
    for s in [
        "if let Some(html) = sym_html(sym)",
        "inner_html=html",
        "note_in(open.key",
        "location().search()",
    ] {
        assert!(dom.contains(s), "DOM の部分に {s} が無い");
    }
    assert!(!dom.contains("note(open.key)"), "DOM の部分に note(open.key) が残る");
}

/// 着地済みと同じ波の歯の名の接頭辞（自分の hsym_ は入れない）。
const FILTER: [&str; 80] = [
    "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
    "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
    "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
    "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_",
    "server_", "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_", "gnav_",
    "mkeys_", "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_", "ticker_", "hfig_",
    "pmore_", "lspark_", "apop_", "brand_", "runsdoc_", "nbatch_", "kcli_", "hcard_", "qgate_",
    "nsum_", "ntime_", "ptitle_", "ncard_", "smore_", "lcard_", "aaround_", "hruling_",
    "sxaxis_", "plimit_", "bport_",
];

#[test]
fn hsym_names_stay_apart() {
    let text = read("tests/hsym.rs");
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
    assert!(names.len() >= 7, "{names:?}");
    for name in &names {
        let tail = name
            .strip_prefix("hsym_")
            .unwrap_or_else(|| panic!("{name} が hsym_ で始まらない"));
        for w in FILTER {
            assert!(!tail.contains(w), "{name} が {w} を含む");
        }
    }
}
