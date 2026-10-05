//! 設計ノートの行の辺の型 req と depends の歯（行 c-dn-edges・要件 FR2）。
//! folio の索引の設計ノートの行の欄 req と depends の辺の行を、型 Req（行 → 要件）と Depends（行 → 行）の辺に組む。
#![cfg(test)]

use std::path::Path;

use crate::common::FILTER_WORDS;
use serde_json::json;
use tsuzuri_contract::graph::{AroundDoc, EdgeType, Fold};
use tsuzuri_contract::wire;
use tsuzuri_core::graph::build::DESIGN_EDGE_TYPES;
use tsuzuri_core::graph::{Graph, Inputs, Verdict, around, build, check};

/// 節の索引（節点 5・辺 6・型は req と depends）。
const IDX: &str = "# 節点（1 行 = id / 種類 / file / 要約値 8 字 / 題 36 字・タブ区切り）
FR1\t要件\tsrs.yaml\t00000000\t面は 2 つ
FR2\t要件\tsrs.yaml\t00000000\t導出グラフ
ex#a\t設計ノートの行\tdesign-note/ex.yaml\t1a2b3c4d\t便 a — 骨格
ex#b\t設計ノートの行\tdesign-note/ex.yaml\t5e6f7a8b\t便 b — 型
ey#c\t設計ノートの行\tdesign-note/ey.yaml\t9c0d1e2f\t行 c — 札
# 辺（1 行 = 端 / 端 / 型・タブ区切り）
ex#a\tFR1\treq
ex#b\tFR1\treq
ex#b\tFR2\treq
ex#b\tex#a\tdepends
ey#c\tex#b\tdepends
ey#c\tFR2\treq
";

/// 器の event log の 1 行。
const EV: &str = r#"{"schema":1,"ts":"2026-09-28T03:00:00Z","kind":"RunCreated","run":"de.1-20260928T030000Z","bead":"de.1","stage":"Intake"}"#;

/// 節の台帳（epic de と task de.1）。
fn led() -> String {
    json!([
        {"id": "de", "title": "根", "status": "open", "issue_type": "epic"},
        {
            "id": "de.1",
            "title": "t de.1",
            "status": "open",
            "issue_type": "task",
            "acceptance_criteria": "design = contracts/ey.toml#c",
            "dependencies": [{"depends_on_id": "de", "type": "parent-child"}],
        },
    ])
    .to_string()
}

/// 節の索引と台帳と event log のグラフ G。
fn g() -> Graph {
    build(&Inputs {
        design_index: IDX,
        ledger: &led(),
        events: EV,
    })
}

fn triple(from: &str, to: &str, t: EdgeType) -> (String, String, EdgeType) {
    (from.to_string(), to.to_string(), t)
}

#[test]
fn dnedge_types_closed() {
    assert_eq!(EdgeType::ALL.len(), 33);
    assert_eq!(DESIGN_EDGE_TYPES, 19);
    let words: Vec<String> = EdgeType::ALL[..DESIGN_EDGE_TYPES]
        .iter()
        .map(|t| wire::encode(t).expect("型の電文"))
        .collect();
    let want: Vec<String> = [
        "in-article",
        "relations.articles",
        "relations.reqs",
        "relations.rules",
        "relations.sections",
        "amended_by",
        "article",
        "refs",
        "basis",
        "goals",
        "rules",
        "adrs",
        "verify.ac",
        "verifies",
        "figures",
        "produced",
        "amends",
        "req",
        "depends",
    ]
    .iter()
    .map(|w| format!("\"{w}\""))
    .collect();
    assert_eq!(words, want);
    assert_eq!(EdgeType::ALL[17], EdgeType::Req);
    assert_eq!(EdgeType::ALL[18], EdgeType::Depends);
    assert_eq!(EdgeType::ALL[19], EdgeType::ParentChild);
    assert_eq!(wire::decode::<EdgeType>("\"req\"").expect("req"), EdgeType::Req);
    assert_eq!(
        wire::decode::<EdgeType>("\"depends\"").expect("depends"),
        EdgeType::Depends
    );
}

#[test]
fn dnedge_index_builds() {
    let g = g();
    assert!(g.unread.is_empty(), "読めない出所: {:?}", g.unread);
    let design: Vec<(String, String, EdgeType)> = g
        .edges
        .iter()
        .filter(|e| EdgeType::ALL[..DESIGN_EDGE_TYPES].contains(&e.edge_type))
        .map(|e| (e.from.clone(), e.to.clone(), e.edge_type))
        .collect();
    assert_eq!(
        design,
        vec![
            triple("ex#a", "FR1", EdgeType::Req),
            triple("ex#b", "FR1", EdgeType::Req),
            triple("ex#b", "FR2", EdgeType::Req),
            triple("ex#b", "ex#a", EdgeType::Depends),
            triple("ey#c", "ex#b", EdgeType::Depends),
            triple("ey#c", "FR2", EdgeType::Req),
        ]
    );
    assert_eq!(g.skipped.design_edges, 0);
    assert_eq!(g.count_edges(EdgeType::Design), 1);

    let got: Vec<(&str, Verdict)> = check(&g).into_iter().map(|i| (i.id, i.verdict)).collect();
    let want: Vec<(&str, Verdict)> = [
        "g-1", "g-2", "g-3", "g-4", "g-5", "g-6", "g-7", "g-8", "g-9", "g-10", "g-11", "g-12",
    ]
    .into_iter()
    .map(|id| {
        let v = if ["g-3", "g-7", "g-9"].contains(&id) {
            Verdict::Unknown
        } else {
            Verdict::Pass
        };
        (id, v)
    })
    .collect();
    assert_eq!(got, want);
}

#[test]
fn dnedge_around_rows() {
    let g = g();
    let doc: AroundDoc = around(&g, "ex#b", 1, Fold::None).expect("ex#b の近傍");
    let rows: Vec<(&str, i8, Option<EdgeType>)> = doc
        .rows
        .iter()
        .map(|r| (r.node.id.as_str(), r.col, r.edge_type))
        .collect();
    assert_eq!(
        rows,
        vec![
            ("FR1", -1, Some(EdgeType::Req)),
            ("FR2", -1, Some(EdgeType::Req)),
            ("ex#a", -1, Some(EdgeType::Depends)),
            ("ex#b", 0, None),
            ("ey#c", 1, Some(EdgeType::Depends)),
        ]
    );
}

#[test]
fn dnedge_own_names_clean() {
    assert_eq!(FILTER_WORDS.len(), 152, "畳んだ 151 語と pgz_");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth1/dnedge.rs");
    let text = std::fs::read_to_string(&path).expect("tests/teeth1/dnedge.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .filter_map(|w| {
            let rest = w[1].trim().strip_prefix("fn ")?;
            rest.split('(').next()
        })
        .collect();
    assert!(names.len() >= 4, "歯の名: {names:?}");
    for name in names {
        let rest = name
            .strip_prefix("dnedge_")
            .unwrap_or_else(|| panic!("{name} は dnedge_ で始まる"));
        for word in FILTER_WORDS {
            assert!(!rest.contains(word), "{name} は {word} を含む");
        }
    }
}
