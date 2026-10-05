//! 不変条件 g-3・g-7 の歯（接頭辞 g3g7_・行 c-g3g7）: 裁定の書き出し（folio check --emit-rulings の行）を節点へ結び、
//! ruled_by の辺を組み、g-3 と g-7 を 3 値で数える。fixture は tests/fixtures/graph/unruled/ の 4 つの file。
#![cfg(test)]

use std::path::Path;

use crate::common::pair;
use tsuzuri_contract::graph::{EdgeType, GraphNode, NodeKind};
use tsuzuri_core::graph::build::{add_rulings, read_rulings};
use tsuzuri_core::graph::check::{RULED, UNMEASURED, in_ruling_grammar, outside_rulings};
use tsuzuri_core::graph::{Graph, Inputs, RulingForm, Source, Verdict, build, check};

fn read(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/graph/unruled")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

fn index() -> String {
    read("index.tsv")
}

fn ledger() -> String {
    read("ledger.json")
}

fn events() -> String {
    read("events.jsonl")
}

fn rulings() -> String {
    read("rulings.jsonl")
}

/// q7-fx.4 の notes の裁定の行。
const Q4_LINE: &str = "裁定 id = q7-fx.4:20260928T0100Z-1・ADR-7 を発効する";

const Q3: &str = "q7-fx.3:20260928T0300Z-1";
const Q4: &str = "q7-fx.4:20260928T0100Z-1";
const Q5: &str = "q7-fx.5:20260928T0200Z-1";

/// 問いでない bead（memo q7-fx.1）の notes の裁定 id。
const MEMO_RULING: &str = "q7-fx.1:20260928T0700Z-1";

/// folio の裁定 id の文法の外の裁定 id。
const USER_RULING: &str = "user 2026-09-28T01:00Z";

/// q7-fx.4 の裁定の行だけを別の字に替えた台帳。
fn cut() -> String {
    let text = ledger();
    assert!(text.contains(Q4_LINE), "台帳に q7-fx.4 の裁定の行");
    text.replace(Q4_LINE, "見本の字")
}

/// 台帳の q7-fx.1 の notes を替えた字。
fn with_memo(ledger: &str, notes: &str) -> String {
    let from = "\"notes\": \"見本の memo の notes\"";
    assert!(ledger.contains(from), "台帳に q7-fx.1 の notes");
    ledger.replace(from, &format!("\"notes\": \"{}\"", notes.replace('\n', "\\n")))
}

/// 裁定の行 2 つ（文法の外の id と文法の内の id）を持つ memo の台帳。
fn memo() -> String {
    with_memo(
        &ledger(),
        &format!("裁定 id = {USER_RULING}・見本\n裁定 id = {MEMO_RULING}・見本"),
    )
}

/// 文法の外の id の裁定の行だけを持つ memo の台帳。
fn only() -> String {
    with_memo(&ledger(), &format!("裁定 id = {USER_RULING}・見本"))
}

/// 書き出しに node ADR-8 の行を 1 つ足した字。
fn plus_adr8(rulings: &str, ruling: &str, form: &str, bead: &str) -> String {
    format!(
        "{rulings}{{\"ruling\":\"{ruling}\",\"form\":\"{form}\",\"bead\":\"{bead}\",\"node\":\"ADR-8\",\"file\":\"adr/ADR-8.yaml\",\"line\":9,\"field\":\"approval.ruling\"}}\n"
    )
}

fn built(index: &str, ledger: &str, events: &str) -> Graph {
    build(&Inputs {
        design_index: index,
        ledger,
        events,
    })
}

/// 組んで書き出しを結んだグラフ（結べることも確かめる）。
fn joined(index: &str, ledger: &str, rulings: &str) -> Graph {
    let mut g = built(index, ledger, &events());
    assert!(add_rulings(&mut g, rulings), "書き出しを結ぶ");
    g
}

/// fixture の 4 つの字のグラフ G。
fn fixture() -> Graph {
    joined(&index(), &ledger(), &rulings())
}

fn verdict_of(g: &Graph, id: &str) -> Verdict {
    check(g)
        .into_iter()
        .find(|i| i.id == id)
        .unwrap_or_else(|| panic!("不変条件 {id}"))
        .verdict
}

fn violation(ids: &[&str]) -> Verdict {
    Verdict::Violation(ids.iter().map(|s| s.to_string()).collect())
}

/// 型 ruled_by の辺の (元, 先) の列。
fn ruled(g: &Graph) -> Vec<(String, String)> {
    g.edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::RuledBy)
        .map(|e| (e.from.clone(), e.to.clone()))
        .collect()
}

/// 違反の不変条件の id。
fn violated(g: &Graph) -> Vec<&'static str> {
    check(g)
        .into_iter()
        .filter(|i| matches!(i.verdict, Verdict::Violation(_)))
        .map(|i| i.id)
        .collect()
}

#[test]
fn g3g7_join_follows_the_rules() {
    let g = fixture();
    let table = g.rulings.as_ref().expect("結んだ表");
    let keys: Vec<&str> = table.keys().map(String::as_str).collect();
    assert_eq!(keys, vec!["ADR-7", "ADR-8", "P-1", "R-1", "nw#a", "nw#b"]);
    let rows = |id: &str| -> Vec<(String, RulingForm)> {
        table[id]
            .iter()
            .map(|r| (r.ruling.clone(), r.form))
            .collect()
    };
    let row = |r: &str, f: RulingForm| (r.to_string(), f);
    assert_eq!(
        rows("ADR-7"),
        vec![
            row("q7-fx.4", RulingForm::Bead),
            row(Q4, RulingForm::Question)
        ]
    );
    assert_eq!(
        rows("ADR-8"),
        vec![row(
            "q7-fx.1 notes 2026-09-28 10:0x JST",
            RulingForm::NotesTime
        )]
    );
    assert_eq!(rows("P-1"), vec![row("q7-fx.1", RulingForm::Bead)]);
    assert_eq!(
        rows("R-1"),
        vec![
            row("q7-fx.5", RulingForm::Bead),
            row(Q5, RulingForm::Question)
        ]
    );
    let note = vec![
        row("q7-fx.1", RulingForm::Bead),
        row(Q3, RulingForm::Question),
        row("q7-fx.1", RulingForm::Bead),
    ];
    assert_eq!(rows("nw#a"), note);
    assert_eq!(rows("nw#b"), note);
    // 結ばない 5 行はどの鍵の行にも無い。
    let all = read_rulings(&rulings()).expect("書き出し");
    for skipped in &all[9..] {
        assert!(
            table.values().all(|rs| !rs.contains(skipped)),
            "結ばない行 {skipped:?}"
        );
    }

    unexported_node_not_joined();
}

/// 書き出しに無い節点を足して結んでも、結んだ表の鍵は替わらない。
fn unexported_node_not_joined() {
    let mut g = built(&index(), &ledger(), &events());
    let mut copy: GraphNode = g.node("nw#a").expect("nw#a").clone();
    copy.id = "nw-x".to_string();
    copy.kind = NodeKind::Req;
    g.nodes.push(copy);
    assert!(add_rulings(&mut g, &rulings()));
    let keys: Vec<&str> = g
        .rulings
        .as_ref()
        .expect("結んだ表")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, vec!["ADR-7", "ADR-8", "P-1", "R-1", "nw#a", "nw#b"]);
}

#[test]
fn g3g7_edges_to_held_rulings() {
    let plain = built(&index(), &ledger(), &events());
    let g = fixture();
    assert_eq!(g.nodes, plain.nodes, "節点の列は build のまま");
    assert_eq!(&g.edges[..plain.edges.len()], &plain.edges[..]);
    let added: Vec<(String, String)> = g.edges[plain.edges.len()..]
        .iter()
        .map(|e| {
            assert_eq!(e.edge_type, EdgeType::RuledBy);
            (e.from.clone(), e.to.clone())
        })
        .collect();
    assert_eq!(
        added,
        vec![
            pair("ADR-7", "q7-fx.4"),
            pair("ADR-7", Q4),
            pair("ADR-8", "q7-fx.1"),
            pair("P-1", "q7-fx.1"),
            pair("R-1", "q7-fx.5"),
            pair("R-1", Q5),
            pair("nw#a", "q7-fx.1"),
            pair("nw#a", Q3),
            pair("nw#b", "q7-fx.1"),
            pair("nw#b", Q3),
        ]
    );
    assert_eq!(g.node(Q4).expect("Q4").kind, NodeKind::Ruling);
    assert_eq!(g.node("q7-fx.1").expect("q7-fx.1").kind, NodeKind::Memo);
    assert_eq!(verdict_of(&g, "g-1"), Verdict::Pass);
}

#[test]
fn g3g7_fixture_passes() {
    let g = fixture();
    assert!(g.unread.is_empty(), "{:?}", g.unread);
    for inv in check(&g) {
        let want = if inv.id == "g-9" {
            Verdict::Unknown
        } else {
            Verdict::Pass
        };
        assert_eq!(inv.verdict, want, "{}", inv.id);
    }
}

#[test]
fn g3g7_cut_line_names_the_record() {
    let g = joined(&index(), &cut(), &rulings());
    assert!(g.unread.is_empty(), "{:?}", g.unread);
    assert!(g.node(Q4).is_none(), "Q4 の節点が無い");
    assert_eq!(violated(&g), vec!["g-3"]);
    assert_eq!(verdict_of(&g, "g-3"), violation(&["ADR-7"]));
    assert_eq!(verdict_of(&g, "g-7"), Verdict::Pass);
    let edges = ruled(&g);
    assert_eq!(edges.len(), 9);
    assert!(!edges.contains(&pair("ADR-7", Q4)));
    assert!(edges.contains(&pair("ADR-7", "q7-fx.4")));

    let g = joined(&index(), "", &rulings());
    assert_eq!(g.unread, vec![Source::Ledger]);
    assert_eq!(verdict_of(&g, "g-3"), Verdict::Unknown);
    assert_eq!(verdict_of(&g, "g-7"), Verdict::Unknown);
    assert!(ruled(&g).is_empty());
}

#[test]
fn g3g7_missing_and_foreign_ledgers() {
    let g = joined(
        &index(),
        &ledger(),
        &plus_adr8(&rulings(), "q7-fx.9", "bead", "q7-fx.9"),
    );
    assert_eq!(verdict_of(&g, "g-3"), violation(&["ADR-8"]));
    assert!(!ruled(&g).contains(&pair("ADR-8", "q7-fx.9")));

    let far = plus_adr8(&rulings(), "s9-far.2", "bead", "s9-far.2");
    let g = joined(&index(), &ledger(), &far);
    assert_eq!(verdict_of(&g, "g-3"), Verdict::Unknown);
    assert_eq!(verdict_of(&g, "g-7"), Verdict::Pass);
    assert_eq!(ruled(&g).len(), 10);

    let g = joined(&index(), &cut(), &far);
    assert_eq!(verdict_of(&g, "g-3"), violation(&["ADR-7"]));
}

#[test]
fn g3g7_unbound_ruling_is_named() {
    let g = joined(&index(), &memo(), &rulings());
    assert!(g.unread.is_empty(), "{:?}", g.unread);
    assert_eq!(g.count_nodes(NodeKind::Ruling), 6);
    assert_eq!(verdict_of(&g, "g-7"), violation(&[MEMO_RULING]));
    assert_eq!(outside_rulings(&g), None);
    assert_eq!(verdict_of(&g, "g-3"), Verdict::Pass);

    let tied = plus_adr8(&rulings(), MEMO_RULING, "question", "q7-fx.1");
    let g = joined(&index(), &memo(), &tied);
    assert!(ruled(&g).contains(&pair("ADR-8", MEMO_RULING)));
    assert_eq!(verdict_of(&g, "g-7"), Verdict::Unknown);
    assert_eq!(outside_rulings(&g), Some(1));
    assert_eq!(verdict_of(&g, "g-3"), Verdict::Pass);

    only_outside_id();
}

/// 文法の外の id の裁定の行だけを持つ memo の台帳のグラフの g-7 と文法の外の数。
fn only_outside_id() {
    let g = joined(&index(), &only(), &rulings());
    assert_eq!(verdict_of(&g, "g-7"), Verdict::Unknown);
    assert_eq!(outside_rulings(&g), Some(1));

    let plain = built(&index(), &only(), &events());
    assert_eq!(verdict_of(&plain, "g-7"), Verdict::Unknown);
    assert_eq!(outside_rulings(&plain), None);

    let blind = joined("", &only(), &rulings());
    assert_eq!(blind.unread, vec![Source::Design]);
    assert_eq!(verdict_of(&blind, "g-7"), Verdict::Unknown);
    assert_eq!(outside_rulings(&blind), None);
}

#[test]
fn g3g7_unread_material_is_unknown() {
    let plain = built(&index(), &ledger(), &events());
    assert_eq!(plain.rulings, None);
    assert_eq!(verdict_of(&plain, "g-3"), Verdict::Unknown);
    assert_eq!(verdict_of(&plain, "g-7"), Verdict::Unknown);

    let text = rulings();
    let l1 = text.lines().next().expect("1 行目");
    let bad = [
        String::new(),
        " \n\n  \n".to_string(),
        format!("{l1}\n# 注の行\n"),
        format!("{l1}\n[{l1}]\n"),
        l1.replace("\"form\":\"bead\"", "\"form\":\"other\""),
        l1.replace("\"bead\":\"q7-fx.1\",", ""),
        l1.replace("\"file\":\"design-note/nw.yaml\"", "\"file\":9"),
    ];
    for (n, t) in bad.iter().enumerate() {
        assert_ne!(t.as_str(), l1, "{n}: 字を替えた");
        assert_eq!(read_rulings(t), None, "{n}: {t}");
        let mut g = plain.clone();
        assert!(!add_rulings(&mut g, t), "{n}: {t}");
        assert_eq!(g, plain, "{n}: グラフを変えない");
        assert_eq!(verdict_of(&g, "g-3"), Verdict::Unknown, "{n}");
        assert_eq!(verdict_of(&g, "g-7"), Verdict::Unknown, "{n}");
    }

    let g = fixture();
    let rest = |g: &Graph| -> Vec<(&'static str, Verdict)> {
        check(g)
            .into_iter()
            .filter(|i| !RULED.contains(&i.id))
            .map(|i| (i.id, i.verdict))
            .collect()
    };
    assert_eq!(rest(&g).len(), 10);
    assert_eq!(rest(&g), rest(&plain));

    empty_index_joined();
}

/// 空の索引に書き出しを結んだグラフ（表は空・g-3 と g-7 は Unknown）。
fn empty_index_joined() {
    let g = joined("", &ledger(), &rulings());
    assert_eq!(g.rulings, Some(Default::default()));
    assert_eq!(g.unread, vec![Source::Design]);
    assert_eq!(verdict_of(&g, "g-3"), Verdict::Unknown);
    assert_eq!(verdict_of(&g, "g-7"), Verdict::Unknown);
}

#[test]
fn g3g7_forms_and_consts() {
    assert_eq!(UNMEASURED, ["g-9"]);
    assert_eq!(RULED, ["g-3", "g-7"]);
    let rows = read_rulings(&rulings()).expect("書き出し");
    assert_eq!(rows.len(), 14);
    let count = |f: RulingForm| rows.iter().filter(|r| r.form == f).count();
    assert_eq!(count(RulingForm::Question), 4);
    assert_eq!(count(RulingForm::NotesTime), 1);
    assert_eq!(count(RulingForm::Bead), 9);
    assert_eq!(rows[4].ruling, "q7-fx.1 notes 2026-09-28 10:0x JST");
    assert_eq!(rows[4].bead, "q7-fx.1");
    assert_eq!(rows[4].target(), "q7-fx.1");
    assert_eq!(rows[3].target(), Q4);
    assert_eq!(rows[3].node.as_deref(), Some("ADR-7"));
    assert_eq!(rows[0].target(), "q7-fx.1");
    assert_eq!(rows[0].node, None);

    fixture_holds_rows(rows);
}

/// fixture のグラフは書き出しの行を全部持ち、替えた行は持たない。
fn fixture_holds_rows(rows: Vec<tsuzuri_core::graph::RulingRow>) {
    let g = fixture();
    for (n, row) in rows.iter().enumerate() {
        assert!(g.holds(row), "{n}: {row:?}");
    }
    let mut other = rows[3].clone();
    other.ruling = "q7-fx.4:20260928T0100Z-2".to_string();
    assert!(!g.holds(&other));
}

#[test]
fn g3g7_ruling_grammar() {
    for id in [
        Q4,
        "q7-fx.1",
        "q7-fx",
        "s9-far.2.14",
        "q7-fx.1 notes 2026-09-28 10:0x JST",
        "q7-fx.1 notes 20:2x",
        "q7-fx.39 notes 2026-09-18",
        "q7-fx notes 2026-09-12 JST",
        "q7-fx.2 notes 2026-09-13 09:35",
    ] {
        assert!(in_ruling_grammar(id), "文法の内: {id}");
    }
    for id in [
        "",
        USER_RULING,
        "Q7-fx.1",
        "q77-fx.1",
        "fx-q7.1",
        "q7-",
        "q7-fx.",
        "q7-fx.a",
        "q7-fx.4:20260928T0100Z-",
        "q7-fx.4:2026092T0100Z-1",
        "q7-fx.4:20260928T0100Z-1 補足",
        "q7-fx.1 notes",
        "q7-fx.1 notes 2026-09-28 10:0y",
        "q7-fx.1 notes 2026-09-28 JST 補足",
    ] {
        assert!(!in_ruling_grammar(id), "文法の外: {id}");
    }
}
