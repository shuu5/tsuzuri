//! memo と epic の touches の辺の歯（中核・接頭辞 memotch_・設計ノート surface-wave29a 行 c-memo-touch・判断の記録 ADR-44
//! 段 2・ADR-47）。台帳の bead の metadata の欄 touches（字 1 つか字の配列・object か object を JSON にした字）から、問いと同じに
//! memo と epic も touches の辺を組む。契約（task）の touches は辺にしない。premises は問いだけ・source は memo だけのまま。
//! 否定の見本は正しい見本から 1 句だけ替える。節の字は歯の中で組む。
#![cfg(test)]
use tsuzuri_contract::graph::{EdgeType, NodeKind};
use tsuzuri_core::graph::build::TOUCH_KINDS;
use tsuzuri_core::graph::{Graph, Inputs, Verdict, build, check};

/// 節の索引の字（要件 FR1 と判断の記録 ADR-7 の 2 つ）。
const IDX: &str = "FR1\t要件\tsrs.yaml\t00000000\t面は 2 つ\nADR-7\t判断の記録\tadr/ADR-7.yaml\t00000001\t裁定面\n";

/// 走行の event log（読めるが走行 0 本の字）。
const EVENTS: &str = "{\"run\":\"free\",\"kind\":\"Note\"}\n";

/// bead 1 本の字（id・種類の欄・label・metadata の JSON の字）。
fn bead(id: &str, issue_type: &str, labels: &str, metadata: &str) -> String {
    format!(
        r#"{{"id":"{id}","title":"見本","issue_type":"{issue_type}","status":"open","labels":[{labels}],"metadata":{metadata}}}"#
    )
}

fn ledger(beads: &[String]) -> String {
    format!("[{}]", beads.join(","))
}

fn built(ledger: &str) -> Graph {
    build(&Inputs {
        design_index: IDX,
        ledger,
        events: EVENTS,
    })
}

fn edges(g: &Graph, t: EdgeType) -> Vec<(&str, &str)> {
    g.edges
        .iter()
        .filter(|e| e.edge_type == t)
        .map(|e| (e.from.as_str(), e.to.as_str()))
        .collect()
}

/// 正しい見本（memo m は配列・epic e は字 1 つ・問い q は配列・memo s は JSON の字の object）。
fn good() -> Vec<String> {
    vec![
        bead(
            "m",
            "task",
            r#""intake:memo""#,
            r#"{"touches":["FR1","ADR-7"]}"#,
        ),
        bead("e", "epic", "", r#"{"touches":"FR1"}"#),
        bead(
            "q",
            "task",
            r#""intake:question""#,
            r#"{"touches":["ADR-7"]}"#,
        ),
        bead(
            "s",
            "task",
            r#""intake:memo""#,
            r#""{\"touches\":[\"ADR-7\"]}""#,
        ),
    ]
}

#[test]
fn memotch_touch_kinds_are_question_memo_epic() {
    assert_eq!(
        TOUCH_KINDS,
        [NodeKind::Question, NodeKind::Memo, NodeKind::Epic]
    );
}

#[test]
fn memotch_memo_and_epic_touches_make_edges() {
    let g = built(&ledger(&good()));
    assert!(g.unread.is_empty(), "{:?}", g.unread);
    assert_eq!(
        edges(&g, EdgeType::Touches),
        vec![
            ("m", "FR1"),
            ("m", "ADR-7"),
            ("e", "FR1"),
            ("q", "ADR-7"),
            ("s", "ADR-7")
        ]
    );
}

#[test]
fn memotch_contract_touches_make_no_edge() {
    let mut beads = good();
    beads[0] = bead("m", "task", "", r#"{"touches":["FR1","ADR-7"]}"#);
    let g = built(&ledger(&beads));
    assert_eq!(g.beads["m"].kind, NodeKind::Task);
    assert_eq!(g.beads["m"].touches, vec!["FR1", "ADR-7"]);
    assert_eq!(
        edges(&g, EdgeType::Touches),
        vec![("e", "FR1"), ("q", "ADR-7"), ("s", "ADR-7")],
        "label を外した契約の touches は属性のまま辺にしない"
    );
}

#[test]
fn memotch_premises_stay_question_only_and_source_memo_only() {
    let beads = vec![
        bead(
            "m",
            "task",
            r#""intake:memo""#,
            r#"{"premises":["p-1"],"source":"r-1"}"#,
        ),
        bead("e", "epic", "", r#"{"premises":["p-1"],"source":"r-1"}"#),
        bead(
            "q",
            "task",
            r#""intake:question""#,
            r#"{"premises":["p-1"],"source":"r-1"}"#,
        ),
    ];
    let g = built(&ledger(&beads));
    assert_eq!(edges(&g, EdgeType::Premises), vec![("q", "p-1")]);
    assert_eq!(edges(&g, EdgeType::Source), vec![("m", "r-1")]);
    assert!(edges(&g, EdgeType::Touches).is_empty());
}

#[test]
fn memotch_dangling_memo_touch_is_g1_and_g4_stays_question_only() {
    let verdict = |g: &Graph, id: &str| {
        check(g)
            .into_iter()
            .find(|i| i.id == id)
            .map(|i| i.verdict)
            .expect("不変条件")
    };
    let g = built(&ledger(&good()));
    assert_eq!(verdict(&g, "g-1"), Verdict::Pass);
    let mut beads = good();
    beads[0] = bead(
        "m",
        "task",
        r#""intake:memo""#,
        r#"{"touches":["FR1","ADR-77"]}"#,
    );
    let g = built(&ledger(&beads));
    assert_eq!(
        verdict(&g, "g-1"),
        Verdict::Violation(vec!["ADR-77".to_string()])
    );
    let bare = vec![
        bead("m", "task", r#""intake:memo""#, "{}"),
        bead("e", "epic", "", "{}"),
    ];
    let g = built(&ledger(&bare));
    assert_eq!(
        verdict(&g, "g-4"),
        Verdict::Pass,
        "g-4 は open の問いだけを見る"
    );
}
