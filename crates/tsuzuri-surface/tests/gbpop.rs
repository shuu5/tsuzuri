//! 吹き出しの設計の行の歯（行 t-graph-bead・要件 FR2・接頭辞 gbpop_）。
//! pointer の行を持つ bead にはその pointer の字を、pointers が空で契約の行の塊が 1 つの bead にはその欄 id を返す。
#![cfg(test)]

use std::collections::BTreeMap;

use tsuzuri_contract::graph::{BeadAttr, GraphDoc, NodeKind, SkippedEdges};
use tsuzuri_surface::widgets::pop::row_pointer;

/// bead 3 本の文書（p は pointer の行・c は contracts が t-one・d は contracts が t-one と t-two）。
fn graph() -> GraphDoc {
    let attr = |pointers: &[&str], contracts: &[&str]| BeadAttr {
        kind: NodeKind::Task,
        status: "open".to_string(),
        labels: Vec::new(),
        pointers: pointers.iter().map(|s| s.to_string()).collect(),
        contracts: contracts.iter().map(|s| s.to_string()).collect(),
        touches: Vec::new(),
    };
    let beads = BTreeMap::from([
        ("p".to_string(), attr(&["design = contracts/nx.toml#a"], &[])),
        ("c".to_string(), attr(&[], &["t-one"])),
        ("d".to_string(), attr(&[], &["t-one", "t-two"])),
    ]);
    GraphDoc {
        nodes: Vec::new(),
        edges: Vec::new(),
        unread: Vec::new(),
        beads,
        runs: BTreeMap::new(),
        invariants: Vec::new(),
        skipped: SkippedEdges {
            design: 0,
            ledger: 0,
            design_nodes: 0,
        },
        retired: None,
    }
}

/// (11) pointer の行の bead は pointer の字・契約の行が 1 つの bead はその欄 id・2 つの bead と台帳に無い bead は無し。
#[test]
fn gbpop_row_pointer_reads_bead_contract() {
    let doc = graph();
    assert_eq!(row_pointer(&doc, "p"), Some("contracts/nx.toml#a".to_string()));
    assert_eq!(row_pointer(&doc, "c"), Some("t-one".to_string()));
    assert_eq!(row_pointer(&doc, "d"), None);
    assert_eq!(row_pointer(&doc, "zz"), None);
}
