//! 行 g-card-adopt-b の歯: 問いの頁の問いの card の題・これまでの決定の段の行の題と決定の link に付ける節点の card の値と、
//! DOM の付け方の字・外した要素の出した card を閉じる後始末の決まりと呼ぶ所の字。
//! card の値は host で組み、DOM は wasm の target のときだけなので、file の字で付け方を見る。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::GraphDoc;
use tsuzuri_contract::ledger::{BeadId, LedgerList};
use tsuzuri_contract::question::QuestionList;
use tsuzuri_contract::wire;
use tsuzuri_surface::project::{Body, ask, askpage};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::widgets::hover::{Card, unmount_hides};
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

fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("id")
}

/// 電文にならない読み（まだ読んでいない・読めない・字 {} の本文）。
fn unread() -> [Fetched; 3] {
    [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("{}".to_string()),
    ]
}

/// fixture の問いの一覧の id を `swap` の対で替えた本文。
fn questions(swap: &[(&str, &str)]) -> Fetched {
    let mut list: QuestionList =
        wire::decode(&read("../../tests/fixtures/surface/question-list.json"))
            .expect("fixture の問いの一覧が電文として読める");
    let Reading::Known(cards) = &mut list.cards else {
        panic!("fixture の問いの一覧が Unknown");
    };
    for c in cards.iter_mut() {
        if let Some((_, to)) = swap.iter().find(|(from, _)| c.id.as_str() == *from) {
            c.id = bead(to);
        }
    }
    Fetched::Body(wire::encode(&list).expect("電文の字にできる"))
}

/// fixture の台帳の一覧の id を `swap` の対で替えた本文。
fn ledger(swap: &[(&str, &str)]) -> Fetched {
    let mut list: LedgerList =
        wire::decode(&read("../../tests/fixtures/surface/ledger-questions.json"))
            .expect("fixture の台帳の一覧が電文として読める");
    let Reading::Known(rows) = &mut list.rows else {
        panic!("fixture の台帳の一覧が Unknown");
    };
    for r in rows.iter_mut() {
        if let Some((_, to)) = swap.iter().find(|(from, _)| r.id.as_str() == *from) {
            r.id = bead(to);
        }
    }
    Fetched::Body(wire::encode(&list).expect("電文の字にできる"))
}

/// (1) 問いの card の題の節点の card は、電文に在る問いの id だけを card_of の値で持つ。
#[test]
fn cadq_title_cards_rules() {
    let doc = graph_doc();
    let graph = Fetched::Body(graph_text());

    let cards = ask::listed(&questions(&[("qa.2", "t3.q10")]));
    let ids: Vec<&str> = cards.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, vec!["t3.q10", "qa.10"]);
    let before = cards.clone();

    let got = ask::node_cards(&graph, &cards);
    let keys: Vec<&str> = got.keys().map(String::as_str).collect();
    assert_eq!(keys, vec!["t3.q10"]);
    let node = card_of(&doc, "t3.q10").expect("t3.q10 の card");
    assert_eq!(got["t3.q10"], node);
    assert_eq!(node.title, "開いた問い");
    assert_eq!(node.kind, "question · beads · open");
    assert_eq!(node.value, "t3.q10 要約なし");
    assert_eq!(cards, before, "card の列の値は変えない");

    let plain = ask::listed(&questions(&[]));
    assert_eq!(plain.len(), 2);
    assert!(ask::node_cards(&graph, &plain).is_empty());
    assert!(ask::node_cards(&graph, &[]).is_empty());
    for g in unread() {
        assert!(ask::node_cards(&g, &cards).is_empty(), "{g:?}");
    }
}

/// (2) 段の行の節点の card は、行の問いの id と答えた決定の id のうち電文に在るものだけを card_of の値で持つ。
#[test]
fn cadq_hist_cards_rules() {
    let doc = graph_doc();
    let graph = Fetched::Body(graph_text());

    let Body::Filled(items) = askpage::body(&ledger(&[("lq.2", "t3.q2"), ("lq.10", "t3.q10")]))
    else {
        panic!("替えた台帳の段が Filled でない");
    };
    let rows = askpage::hist_rows(&items, &graph);
    let got: Vec<(&str, Option<&str>)> = rows
        .iter()
        .map(|e| (e.item.id.as_str(), e.ruling.as_deref()))
        .collect();
    assert_eq!(
        got,
        vec![
            ("lq.9", None),
            ("t3.q10", None),
            ("t3.q2", Some("t3.q2#r")),
        ]
    );

    let cards = askpage::hist_cards(&rows, &graph);
    let keys: Vec<&str> = cards.keys().map(String::as_str).collect();
    assert_eq!(keys, vec!["t3.q10", "t3.q2", "t3.q2#r"]);
    for id in keys {
        assert_eq!(Some(&cards[id]), card_of(&doc, id).as_ref(), "{id}");
    }
    let want = |title: &str, kind: &str, id: &str| Card {
        title: title.to_string(),
        kind: kind.to_string(),
        value: format!("{id} 要約なし"),
        src: ".beads/issues.jsonl".to_string(),
        more: Vec::new(),
    };
    assert_eq!(cards["t3.q2"].title, "閉じた問い");
    assert_eq!(cards["t3.q2"].kind, "question · beads · closed");
    assert_eq!(cards["t3.q2#r"].title, "おすすめどおり");
    assert_eq!(cards["t3.q2#r"].kind, "あなたの決定 · beads · 状態なし");
    assert_eq!(
        cards["t3.q10"],
        want("開いた問い", "question · beads · open", "t3.q10")
    );

    let Body::Filled(plain) = askpage::body(&ledger(&[])) else {
        panic!("台帳の段が Filled でない");
    };
    let plain_rows = askpage::hist_rows(&plain, &graph);
    assert_eq!(plain_rows.len(), 3);
    assert!(askpage::hist_cards(&plain_rows, &graph).is_empty());
    assert!(askpage::hist_cards(&[], &graph).is_empty());
    for g in unread() {
        assert!(askpage::hist_cards(&rows, &g).is_empty(), "{g:?}");
    }
}

/// `start` から末尾までの字。
fn after<'a>(text: &'a str, start: &str) -> &'a str {
    let at = text
        .find(start)
        .unwrap_or_else(|| panic!("字 {start} が無い"));
    &text[at..]
}

/// `start` から `end` の前までの字。
fn between<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    let rest = after(text, start);
    let to = rest[start.len()..]
        .find(end)
        .unwrap_or_else(|| panic!("字 {start} の後に {end} が無い"));
    &rest[..to + start.len()]
}

/// `open` で始まる開きの tag（最初の閉じ山括弧まで）。
fn open_tag<'a>(text: &'a str, open: &str) -> &'a str {
    let at = text
        .find(open)
        .unwrap_or_else(|| panic!("字 {open} が無い"));
    let rest = &text[at..];
    let to = rest.find('>').expect("閉じ山括弧");
    &rest[..=to]
}

/// (3) 問いの card の題・段の行の題・決定の link に Option の card を hover の attach_some で付ける。
#[test]
fn cadq_dom_wiring() {
    let ask_text = read("src/project/ask.rs");
    let dom = after(&ask_text, "mod dom {");
    for want in [
        "crate::net::read(map::PATH)",
        "node_cards(g, v)",
        "nodes.with(|m| m.get(&key).cloned())",
        "use crate::widgets::hover::{self, attach_some};",
        "struct Live {",
        "card_view(c, d, Live { nb, node }, on, tick, current, places)",
    ] {
        assert!(dom.contains(want), "ask.rs の mod dom に {want} が無い");
    }
    let head = between(dom, "Part::Head =>", "Part::Summary =>");
    let tag = open_tag(head, "<a class=\"t\"");
    assert!(
        tag.ends_with("href=move || node_href(&id, mode()) use:attach_some=live.node.get()>"),
        "問いの題の a の tag が違う: {tag}"
    );

    let page = read("src/project/askpage.rs");
    let view = after(&page, "pub fn view(");
    for want in [
        "hist_cards(&entries, g)",
        "hist_li(entry, &cards, mode)",
        "use crate::widgets::hover::attach_some;",
        "use:attach_some=cards.get(&rid).cloned()>{ruling_text(&rid)}</a>",
    ] {
        assert!(view.contains(want), "askpage.rs の view に {want} が無い");
    }
    let tag = open_tag(view, "<a class=\"ttl\"");
    assert!(
        tag.ends_with("href=href use:attach_some=card>"),
        "段の行の題の a の tag が違う: {tag}"
    );

    for (name, text) in [("ask.rs", &ask_text), ("askpage.rs", &page)] {
        for bad in ["fn attach_node", "fn attach_some", "#[allow("] {
            assert!(!text.contains(bad), "{name} に {bad} が在る");
        }
    }
}

/// (4) 外した要素の後始末は、その要素の出した card が層の今の card のときだけ閉じる。
#[test]
fn cadq_cleanup_closes() {
    assert!(unmount_hides(3, 3));
    assert!(!unmount_hides(3, 4));
    assert!(!unmount_hides(0, 0));
    assert!(!unmount_hides(0, 5));

    let text = read("src/widgets/hover.rs");
    let dom = after(&text, "mod dom {");

    let attach = between(dom, "pub fn attach(", "pub fn attach_some(");
    let lines: Vec<&str> = attach.lines().map(str::trim).collect();
    let at = lines
        .iter()
        .position(|l| *l == "ctx.show(card.clone(), point(&ev));")
        .expect("attach の pointerenter の show");
    assert_eq!(
        lines.get(at + 1).copied(),
        Some("mine.store(ctx.seq.get_value(), Ordering::Relaxed);")
    );
    assert!(attach.contains("on_cleanup(move || ctx.release(shown.load(Ordering::Relaxed)));"));

    let release = between(dom, "fn release(self, shown", "fn hide(self) {");
    for want in [
        ".try_get_value()",
        ".is_some_and(|now| unmount_hides(shown, now))",
        "self.hide();",
    ] {
        assert!(release.contains(want), "release に {want} が無い");
    }

    let some = between(dom, "pub fn attach_some(", "pub struct Delegate");
    let lines: Vec<&str> = some.lines().map(str::trim).collect();
    let at = lines
        .iter()
        .position(|l| *l == "if let Some(card) = card {")
        .expect("attach_some の if let");
    assert_eq!(lines.get(at + 1).copied(), Some("attach(el, card);"));

    assert!(
        text.contains("pub use dom::{CardLayer, Delegate, HoverCtx, attach, attach_some, delegate};")
    );
}

/// main 7ea0462 の contracts の verify の filter の語を畳んだ語と、後の行 g-gz と同じノートの行 g-card-adopt-c の接頭辞。
const FILTERS: &[&str] = &[
    "aaround_",
    "accept_",
    "acchold_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "afocus_",
    "aord_",
    "apop_",
    "askcard_",
    "athr_",
    "batchpanel_",
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadopt_",
    "cdorm_",
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "cspk_",
    "ctick_",
    "denv_",
    "dnedge_",
    "dnrow_",
    "dnskip_",
    "ecache_",
    "epolq_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "fserve_",
    "fstop_",
    "fxpre_",
    "gapspage_",
    "gbnote_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gjst_",
    "glabel_",
    "gnav_",
    "gpface_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "hacols_",
    "harest_",
    "hasplit_",
    "hbconf_",
    "hbmark_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
    "hdchip_",
    "hfig_",
    "hnunk_",
    "hook_",
    "hruling_",
    "hsblock_",
    "hsderive_",
    "hshist_",
    "hspage_",
    "hspk_",
    "hsym_",
    "iclose_",
    "ilink_",
    "kcli_",
    "klink_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lidle_",
    "lresume_",
    "lsnap_",
    "lspark_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
    "mstore_",
    "mtree_",
    "nact_",
    "nbatch_",
    "ncard_",
    "nextstep_",
    "nodepage_",
    "nstall_",
    "nsum_",
    "nsumw_",
    "ntime_",
    "nxact_",
    "parts_",
    "pclosed_",
    "pfold_",
    "pipe_",
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
    "ptitle_",
    "punmap_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "question_",
    "relay_",
    "rhold_",
    "runsdoc_",
    "rvk_",
    "saxis_",
    "sclosed_",
    "seatblock_",
    "seatcard_",
    "server_",
    "sesplit_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "stcli_",
    "steady_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "topbar_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
    "pgz_",
    "cadl_",
];

/// (6) この file の歯の名はどれも cadq_ で始まり、残りの字は filter の語を含まない。
#[test]
fn cadq_names_stay_apart() {
    assert_eq!(FILTERS.len(), 159);
    let text = read("tests/cadq.rs");
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
    assert!(names.len() >= 5, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("cadq_")
            .unwrap_or_else(|| panic!("{name} が cadq_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
