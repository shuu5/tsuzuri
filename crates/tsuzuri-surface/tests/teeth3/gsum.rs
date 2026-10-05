//! 行 g-summary の歯: 電文の節点の概要（plain・eng）と行の番号を、節点の card と
//! 節点の頁の頭と 2 面の概要の箱に写し、見本の字数で切る。値は host で組み、DOM は file の字で見る
//! （一覧の面の行の要約の欄は行 m-map-compact で消した）。
#![cfg(test)]

use crate::common::read;
use tsuzuri_contract::graph::{AroundDoc, AroundRow, GraphDoc, GraphNode};
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::node::{self, NO_SRC, NO_SUMMARY, SUMMARY_MARKED, SUMMARY_NONE};
use tsuzuri_surface::widgets::hover::Card;
use tsuzuri_surface::widgets::nodecard::{NO_GIST, NO_LINE, card_for, gist, node_card};

fn graph_doc() -> GraphDoc {
    wire::decode(&read("../../tests/fixtures/surface/graph-doc.json"))
        .expect("graph-doc.json が電文として読める")
}

fn around_doc() -> AroundDoc {
    wire::decode(&read("../../tests/fixtures/surface/around-doc.json"))
        .expect("around-doc.json が電文として読める")
}

/// id の節点の写し（節点が無ければ落ちる）。
fn node_of(doc: &GraphDoc, id: &str) -> GraphNode {
    doc.nodes
        .iter()
        .find(|n| n.id == id)
        .cloned()
        .unwrap_or_else(|| panic!("fixture に {id} が無い"))
}

/// 近傍の電文の中心の行の可変の参照。
fn center_mut(doc: &mut AroundDoc) -> &mut AroundRow {
    doc.rows
        .iter_mut()
        .find(|r| r.col == 0)
        .expect("fixture の中心の行")
}

fn some(s: &str) -> Option<String> {
    Some(s.to_string())
}

/// 句切りの字で折れる 41 字（x の 30 字と「、」と y の 10 字）。
fn g_long() -> String {
    format!("{}、{}", "x".repeat(30), "y".repeat(10))
}

/// 句切りの字の無い 40 字。
fn g_bare() -> String {
    "z".repeat(40)
}

/// fixture の P-1 に line 12 と概要を置いた節点。
fn p1(plain: Option<String>, eng: Option<String>) -> GraphNode {
    let mut n = node_of(&graph_doc(), "P-1");
    n.line = Some(12);
    n.plain = plain;
    n.eng = eng;
    n
}

/// 節点の card（状態なし）と、rows の 3 行目と 4 行目と node_card の値の行・出所・詳しくが同じ字であること。
fn card_checked(node: &GraphNode) -> Card {
    let c = card_for(node, None);
    let rows = c.rows();
    assert_eq!(rows[2].1, c.value, "{} の値の行", node.id);
    assert_eq!(rows[3].1, c.src, "{} の出所の行", node.id);
    let mut doc = graph_doc();
    doc.nodes.insert(0, node.clone());
    let n = node_card(&doc, node);
    assert_eq!(
        (&n.value, &n.src, &n.more),
        (&c.value, &c.src, &c.more),
        "{} の node_card",
        node.id
    );
    c
}

/// (1) 概要は plain が空でなければ plain・そうでなく eng が空でなければ eng・どちらでもなければ None。
#[test]
fn gsum_gist_pick_rules() {
    for (plain, eng, want) in [
        (some("やさしい"), some("かたい"), Some("やさしい")),
        (None, some("かたい"), Some("かたい")),
        (some(""), some("かたい"), Some("かたい")),
        (some(""), None, None),
        (None, some(""), None),
        (None, None, None),
    ] {
        let n = p1(plain.clone(), eng.clone());
        assert_eq!(gist(&n), want, "{plain:?} {eng:?}");
    }
}

/// (2) 値の行は id と空白と概要を room 字で切った字（room が 6 未満なら id だけ・概要が無ければ要約なし）。
#[test]
fn gsum_card_value_line() {
    let two = "画面は 2 つです。";
    let face = "面は 2 つとする。";
    let cases = [
        (p1(some(two), None), format!("P-1 {two}")),
        (p1(None, some(face)), format!("P-1 {face}")),
        (p1(some(""), some(face)), format!("P-1 {face}")),
        (p1(Some(g_long()), None), format!("P-1 {}…", "x".repeat(29))),
        (p1(None, None), format!("P-1 {NO_GIST}")),
    ];
    for (n, want) in cases {
        assert_eq!(card_checked(&n).value, want, "{:?}", n.plain);
    }
    assert_eq!(
        card_checked(&p1(Some(g_long()), None))
            .value
            .chars()
            .count(),
        34
    );

    for (id, want) in [
        ("a".repeat(27), format!("{} 画面は 2…", "a".repeat(27))),
        ("a".repeat(28), "a".repeat(28)),
        ("b".repeat(40), format!("{}…", "b".repeat(33))),
    ] {
        let mut n = p1(some(two), None);
        n.id = id.clone();
        assert_eq!(card_checked(&n).value, want, "{id}");
    }

    let mut doc = around_doc();
    center_mut(&mut doc).node.plain = some(two);
    let c = node::center(&doc).expect("中心の行");
    assert_eq!(c.node.id, "FR1");
    let card = card_for(&c.node, c.status.as_deref());
    assert_eq!(card.value, format!("FR1 {two}"));
    assert_eq!(card.src, "srs.yaml");
    assert_eq!(card.more, vec![format!("srs.yaml{NO_LINE}")]);
}

/// (3) 出所の行は file と行（file が無ければ帯の path）・出所の全部の字は出所の行と違うときだけ詳しくの最後。
#[test]
fn gsum_card_src_line() {
    assert_eq!(NO_LINE, "（行は測れていない）");
    let doc = graph_doc();
    let with_line = |id: &str, line: Option<u32>| {
        let mut n = node_of(&doc, id);
        n.line = line;
        card_checked(&n)
    };
    for (id, line, src, more) in [
        (
            "P-1",
            Some(12),
            "design-intent/constitution.yaml:12",
            vec![],
        ),
        (
            "P-1",
            None,
            "design-intent/constitution.yaml",
            vec!["design-intent/constitution.yaml（行は測れていない）"],
        ),
        (
            "ADR-10",
            Some(3),
            "…/adr/ADR-10.yaml:3",
            vec!["design-intent/adr/ADR-10.yaml:3"],
        ),
        (
            "ADR-10",
            None,
            "…/adr/ADR-10.yaml",
            vec!["design-intent/adr/ADR-10.yaml（行は測れていない）"],
        ),
        ("t3-hub.9", Some(5), ".beads/issues.jsonl", vec![]),
        (
            "r-1",
            None,
            "…/fleet/events.jsonl",
            vec!["<state dir>/fleet/events.jsonl"],
        ),
    ] {
        let c = with_line(id, line);
        assert_eq!(c.src, src, "{id} {line:?}");
        assert_eq!(c.more, more, "{id} {line:?}");
    }
}

/// (4) 20 字を越える概要は 34 字以下の行に折って詳しくの先に置き、出所の全部の字はその後。
#[test]
fn gsum_card_more_rows() {
    let rows = |plain: String| card_checked(&p1(Some(plain), None)).more;
    assert!(rows("あ".repeat(20)).is_empty());
    assert_eq!(rows("あ".repeat(21)), vec!["あ".repeat(21)]);
    assert_eq!(
        rows(g_long()),
        vec![format!("{}、", "x".repeat(30)), "y".repeat(10)]
    );
    assert_eq!(rows(g_bare()), vec!["z".repeat(34), "z".repeat(6)]);

    let mut n = node_of(&graph_doc(), "ADR-10");
    n.line = None;
    n.plain = Some(g_long());
    assert_eq!(
        card_checked(&n).more,
        vec![
            format!("{}、", "x".repeat(30)),
            "y".repeat(10),
            "design-intent/adr/ADR-10.yaml（行は測れていない）".to_string(),
        ]
    );
}

/// (6) 概要の箱は中心の節点の plain と eng から表示の型の 1 つを切らずに写す（無いか空なら要約なし・
/// 表示の型の側が無ければもう一方を印の class で・行 g-sum-pick で 1 つの箱にした）。
#[test]
fn gsum_node_boxes() {
    let boxes = |plain: Option<String>, eng: Option<String>, mode: Mode| {
        let mut doc = around_doc();
        let c = center_mut(&mut doc);
        c.node.plain = plain;
        c.node.eng = eng;
        node::summary(node::center(&doc).expect("中心の行"), mode)
    };
    let two = "画面は 2 つです。";
    let b = boxes(some(two), None, Mode::Beginner);
    assert_eq!(
        (b.key, b.class, b.text.as_str()),
        ("summary_plain", "sumbox", two)
    );
    let b = boxes(some(two), None, Mode::Expert);
    assert_eq!(
        (b.key, b.class, b.text.as_str()),
        ("summary_plain", SUMMARY_MARKED, two)
    );
    assert_eq!(SUMMARY_NONE, "sumbox none");
    assert_eq!(SUMMARY_MARKED, "sumbox marked");
    assert_eq!(NO_SUMMARY, "要約なし");

    let face = "面は 2 つとする。";
    let b = boxes(None, some(face), Mode::Expert);
    assert_eq!((b.class, b.text.as_str()), ("sumbox", face));

    let long = format!("一つ目です。\n{}", "あ".repeat(200));
    let b = boxes(Some(long.clone()), None, Mode::Beginner);
    assert_eq!(b.text, long);

    for mode in Mode::ALL {
        let b = boxes(some(""), some(""), mode);
        assert_eq!(
            (b.class, b.text.as_str()),
            (SUMMARY_NONE, NO_SUMMARY),
            "{mode:?}"
        );
    }
}

/// (7) 頭の出所は file とコロンと行（行が無ければ file と NO_LINE・file が無ければ出所なし）。
#[test]
fn gsum_node_src_text() {
    let src = |file: Option<String>, line: Option<u32>| {
        let mut doc = around_doc();
        let c = center_mut(&mut doc);
        c.node.file = file;
        c.node.line = line;
        node::head(&doc).expect("頭").src
    };
    let doc = around_doc();
    let fr1 = node::center(&doc).expect("中心の行");
    assert_eq!(fr1.node.file.as_deref(), Some("srs.yaml"));
    assert_eq!(src(some("srs.yaml"), None), "srs.yaml（行は測れていない）");
    assert_eq!(src(some("srs.yaml"), Some(242)), "srs.yaml:242");
    assert_eq!(src(None, Some(3)), "出所なし");
    assert_eq!(NO_SRC, "出所なし");
}

/// 字「mod dom {」より後の字。
fn dom_part(text: &str) -> &str {
    let at = text.find("mod dom {").expect("字 mod dom { が在る");
    &text[at..]
}

/// (8) 節点の頁の概要の箱と出所の行の DOM の字（一覧の面の行の要約の欄は行 m-map-compact で消した・
/// 畳む段はもう一方の概要の 1 つだけ・行 g-sum-pick）。
#[test]
fn gsum_dom_text() {
    let node = read("src/project/node.rs");
    let dom = dom_part(&node);
    for want in [
        "summary_in(c, mode(), &excerpt)",
        "{h2(b.key)}</header><p data-t=",
        "M4 21c1-4 4-6 8-6s7 2 8 6",
        "M8 7l-5 5 5 5M16 7l5 5-5 5M14 4l-4 16",
        "<span>{h.src}</span>",
    ] {
        assert!(dom.contains(want), "node.rs の DOM に {want} が無い");
    }
    assert_eq!(
        node.matches("<details").count(),
        node.matches("<details class=SUMBOX_OTHER prop:open=open on:toggle=toggle>")
            .count(),
        "node.rs の <details はもう一方の概要の畳みだけ"
    );
}

/// 着地済みの行と第 3 波から第 8 波の行の verify の filter の語。
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
    "qgate_",
    "hcard_",
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
    "fstop_",
    "nsum_",
    "nsumw_",
    "fmark_",
    "fserve_",
    "cadopt_",
    "tipx_",
    "hcsess_",
    "hcproj_",
    "sesplit_",
    "mstore_",
    "athr_",
    "qblock_",
    "wsteady_",
    "nstall_",
    "pfold_",
    "uword_",
    "cround_",
    "cgdom_",
    "csled_",
    "lhome_",
    "shb_",
    "ghb_",
    "nact_",
    "aord_",
    "mtree_",
];

/// (11) この file の歯の名はどれも gsum_ で始まり、残りの字は filter の語を含まない。
#[test]
fn gsum_own_names_clean() {
    assert_eq!(FILTERS.len(), 106);
    let text = read("tests/teeth3/gsum.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == "#[test]")
        .map(|(i, _)| {
            lines[i + 1..]
                .iter()
                .find_map(|l| l.strip_prefix("fn "))
                .and_then(|rest| rest.split('(').next())
                .expect("test の属性の後の fn")
        })
        .collect();
    assert!(names.len() >= 8, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("gsum_")
            .unwrap_or_else(|| panic!("{name} が gsum_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
