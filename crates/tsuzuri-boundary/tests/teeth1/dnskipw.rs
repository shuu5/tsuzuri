//! 種類の読めない節点の行の数を電文に出す歯（行 c-dn-unknown・要件 FR2）。
//! 口 /api/graph の電文と tz graph --design の眺めは、飛ばした節点の行の数を skipped の design_nodes に持つ。
#![cfg(test)]

use tsuzuri_boundary::cli::graph::design_view;
use tsuzuri_boundary::server::board::{Texts, graph};
use tsuzuri_contract::graph::{GraphDoc, GraphSource, SkippedEdges};
use tsuzuri_contract::wire::{decode, encode};

/// 節の索引（FR1 と A-1 と、種類の語が設計の 12 種の外の F-1 の節点の行と、辺の行 3 つ）。
const IDX: &str = "FR1\t要件\tsrs.yaml\t00000000\t面は 2 つ
A-1\t条\tconstitution.yaml\t00000000\t見本の条
F-1\t図\tfigures.yaml\t11111111\t見本の図
FR1\tA-1\tbasis
F-1\tFR1\tbasis
A-1\tFR1\tnot-a-type
";

fn doc() -> GraphDoc {
    graph(&Texts {
        design: IDX.to_string(),
        ledger: "[]".to_string(),
        ..Texts::default()
    })
}

#[test]
fn dnskip_wire_counts() {
    let doc = doc();
    assert!(!doc.unread.contains(&GraphSource::Design), "{:?}", doc.unread);
    let ids: Vec<&str> = doc.nodes.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(ids, vec!["FR1", "A-1"]);
    assert_eq!(doc.edges.len(), 1);
    let want = SkippedEdges {
        design: 2,
        ledger: 0,
        design_nodes: 1,
    };
    assert_eq!(doc.skipped, want);
    assert_eq!(design_view(&doc).skipped, want);
}

#[test]
fn dnskip_wire_old_form() {
    let old: SkippedEdges = decode(r#"{"design":4,"ledger":1}"#).expect("欄の無い電文を読む");
    assert_eq!(
        old,
        SkippedEdges {
            design: 4,
            ledger: 1,
            design_nodes: 0,
        }
    );
    assert_eq!(
        encode(&doc().skipped).expect("電文の字"),
        r#"{"design":2,"ledger":0,"design_nodes":1}"#
    );
}
