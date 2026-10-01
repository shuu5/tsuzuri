//! 行 g-card-adopt-a の歯: pipeline の札の値の行と詳しく（見本の cardContent の data-run の枝）・札の hover を節点の card に替える値・
//! 次の一手の質問の大きい箱の題（問いの節点の card）・DOM の付け方の字。
//! card の値は host で組み、DOM は wasm の target のときだけなので、2 つの file の字で付け方を見る。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::{NextMove, PipelineBoard, PipelineCard, Reading};
use tsuzuri_contract::graph::GraphDoc;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::stats::{CheckResult, NextCheck, NextStep};
use tsuzuri_contract::wire;
use tsuzuri_surface::mapview::graph::cut;
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::next::{self, Big, big_title};
use tsuzuri_surface::project::pipeline::{
    self, Column, Kcard, RUN_KEY, SOURCE, columns, kcard, node_hover, with_nodes,
};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::widgets::hover::{Card, clip};
use tsuzuri_surface::widgets::nodecard::card_of;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn graph_text() -> String {
    read("../../tests/fixtures/surface/graph-doc.json")
}

fn graph_doc() -> GraphDoc {
    wire::decode(&graph_text()).expect("fixture の電文が読める")
}

fn board_cards() -> Vec<PipelineCard> {
    let board: PipelineBoard =
        wire::decode(&read("../../tests/fixtures/surface/pipeline-board.json"))
            .expect("fixture の板が電文として読める");
    let Reading::Known(cards) = board.cards else {
        panic!("fixture の札が Unknown");
    };
    cards
}

fn next_sets() -> BTreeMap<String, NextStep> {
    wire::decode(&read("../../tests/fixtures/surface/next-step.json"))
        .expect("fixture の組が電文として読める")
}

/// fixture の板の id の札。
fn board_card(id: &str) -> PipelineCard {
    board_cards()
        .into_iter()
        .find(|c| c.contract.as_str() == id)
        .unwrap_or_else(|| panic!("fixture の板に {id} が無い"))
}

fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("id")
}

/// 空の台帳の行で組んだ札（描く今は fixture の板の今）。
fn kc(card: &PipelineCard) -> Kcard {
    kcard(card, &[], NOW)
}

/// x の 30 字と「、」と y の 10 字（41 字）。
fn long1() -> String {
    format!("{}、{}", "x".repeat(30), "y".repeat(10))
}

/// z の 40 字（句切りの字が無い）。
fn long2() -> String {
    "z".repeat(40)
}

/// (1) 札の値の行（回数・段の名・20 字に切った理由）と、理由が 20 字を越えるときの詳しくの行。
#[test]
fn cadopt_run_line_rules() {
    for (id, want) in [
        ("px.5", "↻2 · Questioned · about:write-set"),
        ("px.6", "↻1 · Failed · verify が赤"),
        ("px.7", "↻4 · Stopped · Stopped"),
        ("px.3", "↻1 · Running · Running"),
        ("px.2", "↻2 · Blocked · Blocked"),
        ("px.11", "↻1 · Landed · Landed"),
    ] {
        let k = kc(&board_card(id));
        assert_eq!(k.run_line, want, "{id}");
        assert!(k.run_more.is_empty(), "{id} の詳しく {:?}", k.run_more);
    }

    let mut c = board_card("px.6");
    c.reason = Some(long1());
    let k = kc(&c);
    assert_eq!(k.run_line, format!("↻1 · Failed · {}…", "x".repeat(19)));
    assert_eq!(k.run_line, format!("↻1 · Failed · {}", cut(&long1(), 20)));
    assert_eq!(
        k.run_more,
        vec![format!("{}、", "x".repeat(30)), "y".repeat(10)]
    );

    c.reason = Some(long2());
    let k = kc(&c);
    assert_eq!(k.run_line, format!("↻1 · Failed · {}…", "z".repeat(19)));
    assert_eq!(k.run_more, vec!["z".repeat(34), "z".repeat(6)]);

    // 20 字ちょうどは切らず、詳しくも空。
    c.reason = Some("w".repeat(20));
    let k = kc(&c);
    assert_eq!(k.run_line, format!("↻1 · Failed · {}", "w".repeat(20)));
    assert!(k.run_more.is_empty());

    // 句切りの字の後で片に分け、詰められる片は同じ行に詰める。
    c.reason = Some(format!(
        "{}・{}）{}",
        "a".repeat(10),
        "b".repeat(10),
        "c".repeat(20)
    ));
    let k = kc(&c);
    assert_eq!(
        k.run_more,
        vec![
            format!("{}・{}）", "a".repeat(10), "b".repeat(10)),
            "c".repeat(20)
        ]
    );

    let k = kc(&board_card("px.5"));
    let rows: Vec<String> = k.hover.rows().into_iter().map(|(_, r)| r).collect();
    assert_eq!(
        rows,
        vec![
            "px.5".to_string(),
            "run · Questioned".to_string(),
            "↻2 · acct-4 · 1h".to_string(),
            SOURCE.to_string(),
        ]
    );
}

/// (2) 札の id の節点が電文に在れば、題と種類は節点から・値と出所と詳しくは走行から取る。無ければ札の hover。
#[test]
fn cadopt_node_hover_rules() {
    let doc = graph_doc();

    let mut c3 = board_card("px.3");
    c3.contract = bead("t3-hub.9");
    let k3 = kc(&c3);
    let h3 = node_hover(&doc, &k3);
    assert_eq!(
        h3,
        Card {
            title: "作業中の契約".to_string(),
            kind: "task · beads · in_progress".to_string(),
            value: "↻1 · Running · Running".to_string(),
            src: "fleet/events.jsonl".to_string(),
            more: Vec::new(),
        }
    );

    let mut c5 = board_card("px.5");
    c5.contract = bead("t3-hub.15");
    let k5 = kc(&c5);
    let h5 = node_hover(&doc, &k5);
    assert_eq!(h5.title, "地図の頁の中身を置く");
    assert_eq!(h5.kind, "task · beads · open");
    assert_eq!(h5.value, "↻2 · Questioned · about:write-set");
    assert_eq!(h5.src, SOURCE);
    assert!(h5.more.is_empty());

    for (id, h) in [("t3-hub.9", &h3), ("t3-hub.15", &h5)] {
        let node = card_of(&doc, id).expect("節点の card");
        assert_eq!(h.title, node.title, "{id}");
        assert_eq!(h.kind, node.kind, "{id}");
    }

    // 長い理由の詳しくは走行の詳しく。
    let mut long = c3.clone();
    long.reason = Some(long1());
    let kl = kc(&long);
    let hl = node_hover(&doc, &kl);
    assert_eq!(hl.more, kl.run_more);
    assert_eq!(hl.value, kl.run_line);

    for id in ["px.3", "px.5"] {
        let k = kc(&board_card(id));
        assert_eq!(node_hover(&doc, &k), k.hover, "{id}");
    }
}

/// fixture の板の今（2026-09-27T12:00:00Z・札の since はこの今から経過を引いた時刻）。
const NOW: u64 = 1_790_510_400;

/// fixture の板の px.3 を t3-hub.9 に・px.5 を t3-hub.15 に替えた札の列。
fn swapped_columns() -> Vec<Column> {
    let cards: Vec<PipelineCard> = board_cards()
        .into_iter()
        .map(|mut c| {
            match c.contract.as_str() {
                "px.3" => c.contract = bead("t3-hub.9"),
                "px.5" => c.contract = bead("t3-hub.15"),
                _ => {}
            }
            c
        })
        .collect();
    columns(&cards, &[], NOW)
}

/// (3) グラフの口が読めれば全部の札の hover を節点の card に替え、読めなければ受けた値のまま。
#[test]
fn cadopt_columns_take_node_cards() {
    let doc = graph_doc();
    let cols = swapped_columns();
    let Body::Filled(got) = with_nodes(Body::Filled(cols.clone()), &Fetched::Body(graph_text()))
    else {
        panic!("Filled が Filled で返らない");
    };
    assert_eq!(got.len(), cols.len());
    let mut swapped = 0;
    for (g, c) in got.iter().zip(&cols) {
        assert_eq!(g.lane, c.lane);
        assert_eq!(g.class, c.class);
        assert_eq!(g.cards.len(), c.cards.len());
        for (gk, ck) in g.cards.iter().zip(&c.cards) {
            if ["t3-hub.9", "t3-hub.15"].contains(&ck.id.as_str()) {
                swapped += 1;
                assert_eq!(gk.hover, node_hover(&doc, ck), "{}", ck.id);
                assert_ne!(gk.hover, ck.hover, "{}", ck.id);
                let mut back = gk.clone();
                back.hover = ck.hover.clone();
                assert_eq!(&back, ck, "{} のほかの欄", ck.id);
            } else {
                assert_eq!(gk, ck, "{}", ck.id);
            }
        }
    }
    assert_eq!(swapped, 2);
    unread_keeps(cols);
}

/// グラフの口が読めない・中身の無い値は受けた値のまま。
fn unread_keeps(cols: Vec<Column>) {
    for graph in [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("{}".to_string()),
    ] {
        assert_eq!(
            with_nodes(Body::Filled(cols.clone()), &graph),
            Body::Filled(cols.clone()),
            "{graph:?}"
        );
    }
    let body = Fetched::Body(graph_text());
    assert_eq!(
        with_nodes(Body::Unmeasured(pipeline::REASON), &body),
        Body::Unmeasured(pipeline::REASON)
    );
    assert_eq!(
        with_nodes(Body::Empty(RUN_KEY), &body),
        Body::Empty(RUN_KEY)
    );
}

fn check(kind: NextMove, count: u32, target: &str) -> NextCheck {
    NextCheck {
        kind,
        result: CheckResult::Hit,
        count,
        target: Some(bead(target)),
    }
}

fn big_of(kind: NextMove, target: &str) -> Big {
    next::big(kind, Some(&check(kind, 1, target)))
}

/// (4) 質問の大きい箱だけが、問いの節点が電文に在るとき題と card を持つ。
#[test]
fn cadopt_big_title_rules() {
    let doc = graph_doc();
    let graph = Fetched::Body(graph_text());

    let q = big_of(NextMove::Question, "t3.q10");
    let t = big_title(&q, &graph).expect("t3.q10 の題");
    assert_eq!(t.id, "t3.q10");
    assert_eq!(t.text, "開いた問い");
    let node = card_of(&doc, "t3.q10").expect("t3.q10 の card");
    assert_eq!(t.text, clip(&node.title));
    assert_eq!(t.card, node);
    assert_eq!(t.card.title, "開いた問い");
    assert_eq!(t.card.kind, "question · beads · open");
    assert_eq!(q.what, "t3.q10 · 1 件");
    assert_eq!(q.target.as_deref(), Some("t3.q10"));

    assert!(card_of(&doc, "t3-hub.9").is_some());
    for big in [
        big_of(NextMove::AwaitingEffect, "t3-hub.9"),
        big_of(NextMove::StalledRun, "t3-hub.9"),
    ] {
        assert_eq!(big_title(&big, &graph), None, "{:?}", big.kind);
    }
    other_bigs(graph, q);
}

/// 組の大きい箱は題を持たず、電文が読めなければ質問の大きい箱も題を持たない。
fn other_bigs(graph: Fetched, q: Big) {
    let sets = next_sets();
    for name in ["question", "nothing"] {
        let step = sets
            .get(name)
            .unwrap_or_else(|| panic!("fixture の組 {name}"));
        let big = next::next(step).big;
        assert_eq!(big_title(&big, &graph), None, "組 {name}");
    }
    let asked = next::next(&sets["question"]).big;
    assert_eq!(asked.kind, NextMove::Question);
    assert_eq!(asked.target.as_deref(), Some("nq.4"));

    for g in [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("{}".to_string()),
    ] {
        assert_eq!(big_title(&q, &g), None, "{g:?}");
    }
}

/// 字「mod dom {」より後の字。
fn dom_part(text: &str) -> &str {
    let at = text.find("mod dom {").expect("字 mod dom { が在る");
    &text[at..]
}

/// (5) pipeline の札は節点の card に替えた hover を付け、質問の大きい箱は問いの題の字の節点の頁への link に card を付ける。
#[test]
fn cadopt_dom_wiring() {
    let pipe = read("src/project/pipeline.rs");
    let dom = dom_part(&pipe);
    for want in [
        "with_nodes(content(p, l, now), g)",
        "map::PATH",
        "use:attach=card.hover.clone()",
    ] {
        assert!(
            dom.contains(want),
            "pipeline.rs の mod dom に {want} が無い"
        );
    }

    let next_text = read("src/project/next.rs");
    let dom = dom_part(&next_text);
    for want in [
        "big_title(&next.big, g)",
        "map::PATH",
        "node_href(",
        "<a href=href use:attach=t.card>{t.text}</a>",
    ] {
        assert!(dom.contains(want), "next.rs の mod dom に {want} が無い");
    }
}

/// 着地済みの行と第 3 波から第 7 波のほかの行の verify の filter の語。
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
    "fstop_",
    "nsumw_",
    "fmark_",
    "fserve_",
    "tipx_",
    "hcsess_",
    "hcproj_",
    "sesplit_",
    "mstore_",
    "athr_",
    "qblock_",
];

/// (7) この file の歯の名はどれも cadopt_ で始まり、残りの字は filter の語を含まない。
#[test]
fn cadopt_names_stay_apart() {
    assert_eq!(FILTERS.len(), 92);
    let text = read("tests/cadopt.rs");
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
    assert!(names.len() >= 6, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("cadopt_")
            .unwrap_or_else(|| panic!("{name} が cadopt_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
