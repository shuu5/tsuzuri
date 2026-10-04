//! 導出グラフの節点と問いの card が 2 つの概要を見出しの形の本文からも読む歯（接頭辞 sumcr_・設計ノート surface-wave27b
//! 行 c-sum-two・判断の記録 ADR-30 決定 (7)）。台帳の字は歯の中の JSON の配列（bd の読み取りの口の形）。
#![cfg(test)]

use serde_json::json;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::GraphNode;
use tsuzuri_core::graph::{Inputs, build};
use tsuzuri_core::question::list;

/// 見出しの形の本文（便の bead の形）。
const HEADS: &str =
    "## 概要\nP-見出しの字\n\n## 技術\nE-見出しの字\n\n## 契約\ndesign = contracts/x.toml#a\n";

/// 定型行の形の本文。
const TYPED: &str = "前置き\n概要 = P-定型の字\n技術 = E-定型の字\n";

fn bead(id: &str, description: &str, labels: &[&str]) -> serde_json::Value {
    json!({
        "id": id,
        "title": format!("題 — {id}"),
        "description": description,
        "status": "open",
        "issue_type": "task",
        "created_at": "2026-10-03T01:00:00Z",
        "updated_at": "2026-10-03T01:00:00Z",
        "labels": labels,
    })
}

fn node<'a>(nodes: &'a [GraphNode], id: &str) -> &'a GraphNode {
    nodes
        .iter()
        .find(|n| n.id == id)
        .unwrap_or_else(|| panic!("節点 {id} が無い"))
}

#[test]
fn sumcr_build_reads_heads() {
    let ledger = json!([
        bead("fx-s.1", HEADS, &[]),
        bead("fx-s.2", TYPED, &[]),
        bead("fx-s.3", "本文だけ\n", &[]),
    ])
    .to_string();
    let g = build(&Inputs {
        design_index: "",
        ledger: &ledger,
        events: "",
    });
    let got = |id: &str| {
        let n = node(&g.nodes, id);
        (n.plain.as_deref(), n.eng.as_deref())
    };
    assert_eq!(got("fx-s.1"), (Some("P-見出しの字"), Some("E-見出しの字")));
    assert_eq!(got("fx-s.2"), (Some("P-定型の字"), Some("E-定型の字")));
    assert_eq!(got("fx-s.3"), (None, None));
}

#[test]
fn sumcr_card_reads_heads() {
    let ledger = json!([
        bead("fx-s.1", HEADS, &["intake:question"]),
        bead("fx-s.2", TYPED, &["intake:question"]),
    ])
    .to_string();
    let Reading::Known(cards) = list(&ledger).cards else {
        panic!("読める台帳が Unknown");
    };
    let got: Vec<(&str, Option<&str>, Option<&str>)> = cards
        .iter()
        .map(|c| (c.id.as_str(), c.plain.as_deref(), c.eng.as_deref()))
        .collect();
    assert_eq!(
        got,
        [
            ("fx-s.1", Some("P-見出しの字"), Some("E-見出しの字")),
            ("fx-s.2", Some("P-定型の字"), Some("E-定型の字")),
        ]
    );
}
