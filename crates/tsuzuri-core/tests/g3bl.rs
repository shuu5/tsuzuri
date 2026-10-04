//! 器の結びの口の裁定の行の歯（中核・接頭辞 g3bl_・設計ノート surface-wave29a 行 c-g3-bindline・判断の記録 ADR-7 決定 (4)・
//! ADR-44 段 0）。器の結びの口（seat ruling bind）が問いの notes に足す行「<裁定 id> | <問い id> | <発話の ts> | <経路> | <逐語>」を、
//! 導出グラフの台帳の読みと外の台帳の読みが裁定の定型行として読む。土台は tests/fixtures/graph/unruled/ の 4 つの file で、
//! q7-fx.4 の notes の「裁定 id = 」の行を器の形の行に替える。否定の見本は正しい見本から 1 句だけ替える。
#![cfg(test)]
use std::path::Path;

use tsuzuri_contract::graph::{EdgeType, NodeKind, title36};
use tsuzuri_core::graph::build::{BIND_FIELDS, BIND_SEP, add_rulings, bind_line, outside};
use tsuzuri_core::graph::{Graph, Inputs, Verdict, build, check};

/// q7-fx.4 の notes の今の裁定の行（土台の字）。
const Q4_LINE: &str = "裁定 id = q7-fx.4:20260928T0100Z-1・ADR-7 を発効する";

/// q7-fx.4 の裁定 id。
const Q4: &str = "q7-fx.4:20260928T0100Z-1";

/// 器の形の正しい行（q7-fx.4 の notes に置く）。
const GOOD: &str = "q7-fx.4:20260928T0100Z-1 | q7-fx.4 | 2026-09-28T01:00:00Z | chat | \"ADR-7 を発効する\"";

fn read(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/graph/unruled")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

/// q7-fx.4 の裁定の行を `line` に替えた台帳の字（JSON の字の中なので二重引用符を逃がす）。
fn ledger_with(line: &str) -> String {
    let text = read("ledger.json");
    assert!(text.contains(Q4_LINE), "台帳に q7-fx.4 の裁定の行");
    text.replace(Q4_LINE, &line.replace('"', "\\\""))
}

fn built(line: &str) -> Graph {
    build(&Inputs {
        design_index: &read("index.tsv"),
        ledger: &ledger_with(line),
        events: &read("events.jsonl"),
    })
}

fn answers(g: &Graph) -> Vec<(&str, &str)> {
    g.edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::Answers)
        .map(|e| (e.from.as_str(), e.to.as_str()))
        .collect()
}

fn verdict_of(g: &Graph, id: &str) -> Verdict {
    check(g)
        .into_iter()
        .find(|i| i.id == id)
        .map(|i| i.verdict)
        .unwrap_or_else(|| panic!("不変条件 {id}"))
}

/// 否定の見本（正しい行から 1 句だけ替える・理由の字と行の字）。
fn negative_cases() -> Vec<(&'static str, String)> {
    vec![
        (
            "欄が 4 つ",
            "q7-fx.4:20260928T0100Z-1 | q7-fx.4 | 2026-09-28T01:00:00Z | chat".to_string(),
        ),
        (
            "1 つ目の欄が文法の外",
            GOOD.replacen("q7-fx.4:20260928T0100Z-1", "q7-fx.4:2026-09-28-1", 1),
        ),
        (
            "「:」の前が 2 つ目の欄と違う",
            GOOD.replacen("q7-fx.4:20260928T0100Z-1", "q7-fx.9:20260928T0100Z-1", 1),
        ),
        (
            "2 つ目の欄が notes を持つ bead でない",
            GOOD.replace("q7-fx.4", "q7-fx.9"),
        ),
    ]
}

#[test]
fn g3bl_bind_line_makes_ruling_and_answers() {
    assert_eq!((BIND_SEP, BIND_FIELDS), (" | ", 5));
    let g = built(GOOD);
    assert!(g.unread.is_empty(), "{:?}", g.unread);
    let node = g.node(Q4).expect("器の形の行の裁定の節点");
    assert_eq!(node.kind, NodeKind::Ruling);
    assert_eq!(node.title, title36(GOOD));
    assert_eq!(
        (&node.file, &node.digest, node.line, node.updated),
        (&None, &None, None, None)
    );
    assert!(answers(&g).contains(&(Q4, "q7-fx.4")), "{:?}", answers(&g));
    assert_eq!(
        g.nodes.iter().filter(|n| n.id == Q4).count(),
        1,
        "裁定の節点は 1 つ"
    );
}

#[test]
fn g3bl_only_full_bind_lines_count() {
    assert_eq!(bind_line(GOOD, "q7-fx.4"), Some(Q4));
    for (why, line) in negative_cases() {
        assert_eq!(bind_line(&line, "q7-fx.4"), None, "{why}");
        let g = built(&line);
        assert!(g.unread.is_empty(), "{why}: {:?}", g.unread);
        let first = line.split(BIND_SEP).next().unwrap_or_default();
        assert!(g.node(first).is_none(), "{why}: 節点を作らない");
        assert!(g.node(Q4).is_none(), "{why}: Q4 の節点も無い");
        assert!(
            !answers(&g).iter().any(|(_, to)| *to == "q7-fx.4"),
            "{why}: answers の辺を作らない"
        );
    }
}

#[test]
fn g3bl_outside_reads_bind_lines() {
    let good = GOOD.replace("q7-fx.4", "s2-x.4").replace('"', "\\\"");
    let short = "s2-x.5:20260928T0100Z-1 | s2-x.5 | 2026-09-28T01:00:00Z | chat";
    let ledger = format!(
        "[{{\"id\":\"s2-x.4\",\"notes\":\"{good}\"}},{{\"id\":\"s2-x.5\",\"notes\":\"{short}\"}}]"
    );
    let out = outside(&ledger).expect("外の台帳を読む");
    let rulings: Vec<&str> = out.rulings.iter().map(String::as_str).collect();
    assert_eq!(rulings, vec!["s2-x.4:20260928T0100Z-1"]);
    let beads: Vec<&str> = out.beads.iter().map(String::as_str).collect();
    assert_eq!(beads, vec!["s2-x.4", "s2-x.5"]);
}

#[test]
fn g3bl_g3_stops_naming_the_bound_ruling() {
    let mut g = built(GOOD);
    assert!(add_rulings(&mut g, &read("rulings.jsonl")), "書き出しを結ぶ");
    assert_eq!(verdict_of(&g, "g-3"), Verdict::Pass);
    assert_eq!(verdict_of(&g, "g-7"), Verdict::Pass);
    let (_, short) = &negative_cases()[0];
    let mut g = built(short);
    assert!(add_rulings(&mut g, &read("rulings.jsonl")), "書き出しを結ぶ");
    assert_eq!(
        verdict_of(&g, "g-3"),
        Verdict::Violation(vec!["ADR-7".to_string()])
    );
}

#[test]
fn g3bl_typed_prefix_lines_stay_first() {
    let line = format!("{Q4_LINE} | q7-fx.4 | 2026-09-28T01:00:00Z | chat");
    let g = built(&line);
    let node = g.node(Q4).expect("「裁定 id = 」の行の裁定の節点");
    assert_eq!(node.title, title36(&line));
    assert!(answers(&g).contains(&(Q4, "q7-fx.4")));
    assert_eq!(bind_line(Q4_LINE, "q7-fx.4"), None, "頭の字の行は器の形でない");
}
