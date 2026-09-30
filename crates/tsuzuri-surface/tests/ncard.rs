//! 行 g-card-node の歯: 電文の節点 1 つから組む hover の card（widgets の nodecard）と、
//! 地図の圧縮の面の札・条の札・規範文の子と一覧の面の行の題がその card を持ち、a の要素に付けること。
//! card の値は host で組み、DOM は wasm の target のときだけなので、3 つの file の字で付け方を見る。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::graph::GraphDoc;
use tsuzuri_contract::wire;
use tsuzuri_surface::mapview::compact::{Cards, Tag, compact};
use tsuzuri_surface::mapview::list::{Query, listing};
use tsuzuri_surface::widgets::hover::{Card, ELLIPSIS, ROW_CHARS};
use tsuzuri_surface::widgets::nodecard::{card_of, node_card, short_path};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture() -> GraphDoc {
    wire::decode(&read("../../tests/fixtures/surface/graph-doc.json"))
        .expect("fixture が電文として読める")
}

/// id の節点の card（節点が無ければ落ちる）。
fn card(doc: &GraphDoc, id: &str) -> Card {
    let node = doc
        .nodes
        .iter()
        .find(|n| n.id == id)
        .unwrap_or_else(|| panic!("fixture に {id} が無い"));
    node_card(doc, node)
}

/// id の節点の file を無しにする。
fn drop_file(doc: &mut GraphDoc, id: &str) {
    doc.nodes
        .iter_mut()
        .find(|n| n.id == id)
        .unwrap_or_else(|| panic!("fixture に {id} が無い"))
        .file = None;
}

/// id の節点の title を替える。
fn set_title(doc: &mut GraphDoc, id: &str, title: &str) {
    doc.nodes
        .iter_mut()
        .find(|n| n.id == id)
        .unwrap_or_else(|| panic!("fixture に {id} が無い"))
        .title = title.to_string();
}

/// (1) 出所の path は最初の「・」より前の字の、末尾の 2 片に短くする（片が 2 つ以下ならそのまま）。
#[test]
fn ncard_short_path_rules() {
    let text = read("src/widgets/mod.rs");
    for want in [
        "pub mod nodecard;",
        "pub mod fig;",
        "pub mod help;",
        "pub mod hover;",
    ] {
        assert!(text.contains(want), "widgets の mod.rs に {want} が無い");
    }
    for (path, want) in [
        (
            "design-intent/constitution.yaml",
            "design-intent/constitution.yaml",
        ),
        (".beads/issues.jsonl", ".beads/issues.jsonl"),
        ("design-intent/adr/ADR-10.yaml", "…/adr/ADR-10.yaml"),
        (
            "design-intent/design-note/*.yaml・docs/design/*.md",
            "…/design-note/*.yaml",
        ),
        ("<state dir>/fleet/events.jsonl", "…/fleet/events.jsonl"),
        ("a/b/c/d", "…/c/d"),
        ("x・y/z", "x"),
        ("", ""),
    ] {
        assert_eq!(short_path(path), want, "{path:?}");
    }
}

/// (2)(3) fixture の節点の card の 4 行と詳しく。
#[test]
fn ncard_node_card_on_fixture() {
    let doc = fixture();
    assert_eq!(
        card(&doc, "P-1"),
        Card {
            title: "常に守ること".to_string(),
            kind: "条 · constitution · 状態なし".to_string(),
            value: "P-1 要約なし".to_string(),
            src: "design-intent/constitution.yaml".to_string(),
            more: vec!["design-intent/constitution.yaml（行は測れていない）".to_string()],
        }
    );
    assert_eq!(card(&doc, "P-1.2").kind, "条の文 · constitution · 状態なし");
    assert_eq!(
        card(&doc, "ADR-10"),
        Card {
            title: "十番目の判断".to_string(),
            kind: "ADR · ADR · 状態なし".to_string(),
            value: "ADR-10 要約なし".to_string(),
            src: "…/adr/ADR-10.yaml".to_string(),
            more: vec!["design-intent/adr/ADR-10.yaml（行は測れていない）".to_string()],
        }
    );
    assert_eq!(card(&doc, "FR14").kind, "requirement · SRS · 状態なし");
    let task = card(&doc, "t3-hub.9");
    assert_eq!(task.kind, "task · beads · in_progress");
    assert_eq!(task.src, ".beads/issues.jsonl");
    assert!(task.more.is_empty());
    assert_eq!(card(&doc, "t3.q10").kind, "question · beads · open");
    assert_eq!(card(&doc, "t3.p").kind, "全体への指示 · beads · 状態なし");
    let run = card(&doc, "r-1");
    assert_eq!(run.kind, "run · pipeline · Landed");
    assert_eq!(run.src, "…/fleet/events.jsonl");
    assert_eq!(run.more, vec!["<state dir>/fleet/events.jsonl".to_string()]);
    assert_eq!(card(&doc, "r-12").kind, "run · pipeline · 状態なし");

    let long = card(&doc, "R-4");
    assert_eq!(long.title, "abcdefghij abcdefghij abcdefghij abcdefghij");
    let first = &long.rows()[0].1;
    assert_eq!(first, &format!("abcdefghij abcdefghij abcdefghij ab{ELLIPSIS}"));
    assert_eq!(first.chars().count(), ROW_CHARS);

    for id in ["surface-board#g-map", "surface-board#g-frame"] {
        let c = card(&doc, id);
        assert_eq!(c.kind, "design-note の行 · design-note · 状態なし", "{id}");
        assert_eq!(c.src, "contracts/surface-board.toml", "{id}");
        assert_eq!(
            c.more,
            vec!["contracts/surface-board.toml（行は測れていない）".to_string()],
            "{id}"
        );
    }
}

/// (4) 変えた電文での出所・状態・題の規則。
#[test]
fn ncard_rules_on_changed_docs() {
    let base = fixture();

    let mut doc = base.clone();
    drop_file(&mut doc, "P-1");
    let c = card(&doc, "P-1");
    assert_eq!(c.src, "design-intent/constitution.yaml");
    assert!(c.more.is_empty());

    let mut doc = base.clone();
    drop_file(&mut doc, "ADR-2");
    let c = card(&doc, "ADR-2");
    assert_eq!(c.src, "…/adr/ADR-n.yaml");
    assert_eq!(c.more, vec!["design-intent/adr/ADR-n.yaml".to_string()]);

    let mut doc = base.clone();
    drop_file(&mut doc, "surface-board#g-map");
    let c = card(&doc, "surface-board#g-map");
    assert_eq!(c.src, "…/design-note/*.yaml");
    assert_eq!(
        c.more,
        vec!["design-intent/design-note/*.yaml・docs/design/*.md".to_string()]
    );

    let mut doc = base.clone();
    doc.runs.get_mut("r-12").expect("r-12 の属性").stage = Some("Landed".to_string());
    assert_eq!(card(&doc, "r-12").kind, "run · pipeline · Landed");

    let mut doc = base.clone();
    doc.beads.remove("t3-hub.9").expect("t3-hub.9 の属性");
    assert_eq!(card(&doc, "t3-hub.9").kind, "task · beads · 状態なし");

    let mut doc = base.clone();
    doc.beads.get_mut("t3.m1").expect("t3.m1 の属性").status = "closed".to_string();
    assert_eq!(card(&doc, "t3.m1").kind, "memo · beads · closed");

    let mut doc = base.clone();
    set_title(&mut doc, "P-1", " \n\t  \n");
    assert_eq!(card(&doc, "P-1").title, "P-1");

    let mut doc = base.clone();
    set_title(&mut doc, "R-25", " 面の\n\t  crate\n\t  の直接依存 ");
    assert_eq!(card(&doc, "R-25").title, "面の crate の直接依存");
}

/// (5) id から引く card は node_card と同じで、どの節点の card も 2 行目から 4 行目が切られない。
#[test]
fn ncard_card_of_every_node() {
    let doc = fixture();
    assert!(!doc.nodes.is_empty());
    for node in &doc.nodes {
        let c = node_card(&doc, node);
        assert_eq!(card_of(&doc, &node.id), Some(c.clone()), "{}", node.id);
        assert!(!c.title.is_empty(), "{} の題が空", node.id);
        assert!(!c.src.is_empty(), "{} の出所が空", node.id);
        for text in [&c.kind, &c.value, &c.src] {
            assert!(text.chars().count() <= ROW_CHARS, "{} の {text}", node.id);
        }
        let rows = c.rows();
        assert_eq!(rows[1].1, c.kind);
        assert_eq!(rows[2].1, c.value);
        assert_eq!(rows[3].1, c.src);
    }
    assert_eq!(card_of(&doc, ""), None);
    assert_eq!(card_of(&doc, "t3-hub.99"), None);
}

fn check_tag(doc: &GraphDoc, t: &Tag) {
    assert_eq!(Some(t.card.clone()), card_of(doc, &t.id), "札 {}", t.id);
}

/// (6) 圧縮の面の札・条の札・規範文と一覧の面の行は、その id の card を持つ。
#[test]
fn ncard_tags_and_rows_carry_cards() {
    let base = fixture();
    let mut changed = base.clone();
    drop_file(&mut changed, "P-1");
    changed.beads.remove("t3-hub.9").expect("t3-hub.9 の属性");
    for doc in [&base, &changed] {
        let mut seen = 0;
        let mut norms = 0;
        for b in compact(doc) {
            match &b.cards {
                Cards::Unmeasured(_) | Cards::Empty => {}
                Cards::Tags(tags) => tags.iter().for_each(|t| check_tag(doc, t)),
                Cards::Articles { articles, loose } => {
                    for a in articles {
                        check_tag(doc, &a.tag);
                        assert_eq!(a.norm_cards.len(), a.norms.len(), "条 {}", a.tag.id);
                        for (id, c) in a.norms.iter().zip(&a.norm_cards) {
                            assert_eq!(Some(c.clone()), card_of(doc, id), "規範文 {id}");
                            norms += 1;
                        }
                    }
                    loose.iter().for_each(|t| check_tag(doc, t));
                }
                Cards::Lanes(lanes) => lanes
                    .iter()
                    .flat_map(|l| &l.tags)
                    .for_each(|t| check_tag(doc, t)),
                Cards::Notes(notes) => notes
                    .iter()
                    .flat_map(|g| &g.tags)
                    .for_each(|t| check_tag(doc, t)),
            }
            seen += b.ids().len();
        }
        assert!(seen > 0 && norms > 0, "札 {seen}・規範文 {norms}");
        for search in ["", "?sort=state"] {
            let rows = listing(doc, &Query::from_search(search)).rows;
            assert!(!rows.is_empty(), "{search:?} の行が在る");
            for r in rows {
                assert_eq!(Some(r.card.clone()), card_of(doc, &r.id), "行 {}", r.id);
            }
        }
    }
    let p1 = card_of(&changed, "P-1").expect("P-1 の card");
    assert_eq!(p1.src, "design-intent/constitution.yaml");
    let t9 = card_of(&changed, "t3-hub.9").expect("t3-hub.9 の card");
    assert_eq!(t9.kind, "task · beads · 状態なし");
}

/// 字「mod dom {」より後の字。
fn dom_part(text: &str) -> &str {
    let at = text.find("mod dom {").expect("字 mod dom { が在る");
    &text[at..]
}

/// at から始まる tag（次の > まで・> を含む）。
fn tag_from(text: &str, at: usize) -> &str {
    let end = text[at..].find('>').expect("tag の閉じ山括弧");
    &text[at..=at + end]
}

/// before の字の前の最後の <a から始まる tag。
fn anchor_before<'a>(text: &'a str, before: &str) -> &'a str {
    let p = text
        .find(before)
        .unwrap_or_else(|| panic!("字 {before} が無い"));
    let at = text[..p]
        .rfind("<a")
        .unwrap_or_else(|| panic!("{before} の前に <a が無い"));
    tag_from(text, at)
}

/// start の字で始まる tag。
fn anchor_at<'a>(text: &'a str, start: &str) -> &'a str {
    let at = text
        .find(start)
        .unwrap_or_else(|| panic!("字 {start} が無い"));
    tag_from(text, at)
}

/// (7) 4 つの a の開きの tag の末尾に、それぞれの card を付ける use:attach が在る。
#[test]
fn ncard_dom_wiring() {
    let compact_text = read("src/mapview/compact.rs");
    let dom = dom_part(&compact_text);
    for (tag, want) in [
        (
            anchor_at(dom, "<a class=t.class.clone()"),
            "use:attach=t.card.clone()>",
        ),
        (
            anchor_before(dom, "{tag_head(&a.tag)}</a>"),
            "use:attach=a.tag.card.clone()>",
        ),
        (anchor_before(dom, "{id}</a>"), "use:attach=card>"),
    ] {
        assert!(tag.ends_with(want), "{tag} が {want} で終わらない");
    }
    assert!(dom.contains("zip(a.norm_cards"), "規範文の card を zip で組まない");

    let list_text = read("src/mapview/list.rs");
    let tag = anchor_at(dom_part(&list_text), "<a class=\"ttl\"");
    assert!(
        tag.ends_with("use:attach=r.card.clone()>"),
        "{tag} が行の card を付けない"
    );

    let board = read("src/board.rs");
    for want in ["provide_context(HoverCtx::default())", "<CardLayer/>"] {
        assert!(board.contains(want), "board.rs に {want} が無い");
    }
}

/// 着地済みの行と第 3 波から第 5 波の行の verify の filter の語。
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
    "hsym_",
    "smore_",
    "lcard_",
    "aaround_",
    "hruling_",
    "sxaxis_",
    "plimit_",
    "bport_",
];

/// (9) この file の歯の名はどれも ncard_ で始まり、残りの字は filter の語を含まない。
#[test]
fn ncard_names_stay_apart() {
    assert_eq!(FILTERS.len(), 80);
    let text = read("tests/ncard.rs");
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
    assert!(names.len() >= 7, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("ncard_")
            .unwrap_or_else(|| panic!("{name} が ncard_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
