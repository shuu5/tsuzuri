//! 本文で名指した id のうち欄にも辺にも無い対を数える床の歯（行 c-g9・不変条件 g-9・要件 FR3）。
//! 中核の `mentioned_ids` が folio の id の文法で字から id を拾い、`unfielded_mentions` が設計の節点の要約の字から
//! 対を数える。g-9 の判定は detect の間は変えない（つねに「まだ分からない」）。節の字は歯の中で組む。
#![cfg(test)]

use serde_json::json;
use tsuzuri_contract::board::Reading;
use tsuzuri_core::graph::build::add_summary;
use tsuzuri_core::graph::check::{UNMEASURED, mentioned_ids, unfielded_mentions};
use tsuzuri_core::graph::{Graph, Inputs, Verdict, build, check};

/// 節の見本の字。
const SAMPLE: &str = "条 P-7.2 と FR3・NFR2（ADR-7）と R-17、D-11 と A-1 と N-2。\
FR30 と P-7.2x と XFR3 と R-1a と ADR-0 と ADR-12 と GOAL2 と AC5 と CON1 と -P-3 と P-4.1.2 と FR3FR4 と fx-hub.5。";

/// 節の索引（節点 7 つと辺 5 本）。
const IDX: &str = "A-1\t条\tconstitution.yaml\t00000000\t条\n\
A-1.1\t規範文\tconstitution.yaml\t00000000\t規範文\n\
R-2\t規則行\trules.yaml\t00000000\t規則行\n\
FR3\t要件\tsrs.yaml\t00000000\t要件\n\
ADR-7\t判断の記録\tadr/ADR-7.yaml\t00000000\t判断の記録\n\
GOAL1\t目的\tsrs.yaml\t00000000\t目的\n\
nt#r1\t設計ノートの行\tnotes/nt.yaml\t00000000\t設計ノートの行\n\
A-1.1\tA-1\tin-article\n\
FR3\tGOAL1\tgoals\n\
FR3\tADR-7\tadrs\n\
R-2\tA-1.1\tarticle\n\
nt#r1\tFR3\treq\n";

/// 節の足す辺 3 本。
const MORE: &str = "A-1\tFR3\trelations.reqs\n\
ADR-7\tA-1.1\tarticle\n\
nt#r1\tR-2\trules\n";

/// 節の台帳（bead fx-1）。
const LED: &str = r#"[{"id":"fx-1","title":"台帳の行","issue_type":"task","status":"open","description":"概要 = R-2 と FR3 を見る"}]"#;

/// 節の要約の字（GOAL1 は本文なし）。
fn sum() -> String {
    [
        ("A-1", "constitution.yaml", Some("R-2 と FR3 を見る"), None),
        ("A-1.1", "constitution.yaml", None, Some("A-1 の枝。ADR-7 に従う")),
        ("R-2", "rules.yaml", None, Some("FR3FR4 と P-9 を数える")),
        ("FR3", "srs.yaml", None, Some("FR3 は GOAL1 と ADR-7 と FR9 を満たす")),
        ("ADR-7", "adr/ADR-7.yaml", Some("FR3 と A-1.1 を決める"), None),
        ("GOAL1", "srs.yaml", None, None),
        ("nt#r1", "notes/nt.yaml", None, Some("R-2 を写す・FR3 を満たす")),
    ]
    .iter()
    .map(|(id, file, plain, eng)| {
        json!({"id": id, "file": file, "line": 1, "plain": plain, "eng": eng}).to_string()
    })
    .collect::<Vec<_>>()
    .join("\n")
}

/// 索引と台帳の字から組み、要約の字を写したグラフと、写せたか。
fn graph(design_index: &str, ledger: &str) -> (Graph, bool) {
    let mut g = build(&Inputs {
        design_index,
        ledger,
        events: "",
    });
    let summary = add_summary(&mut g, &sum());
    (g, summary)
}

fn known(pairs: &[(&str, &str)]) -> Reading<Vec<(String, String)>> {
    Reading::Known(
        pairs
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect(),
    )
}

/// 節の索引と要約の字と台帳から数える 4 つの対。
const PAIRS: [(&str, &str); 4] = [
    ("A-1", "FR3"),
    ("A-1.1", "ADR-7"),
    ("ADR-7", "A-1.1"),
    ("nt#r1", "R-2"),
];

#[test]
fn cg9_scan_grammar() {
    assert_eq!(
        mentioned_ids(SAMPLE),
        vec![
            "P-7.2", "FR3", "NFR2", "ADR-7", "R-17", "D-11", "A-1", "N-2", "FR30", "P-7", "ADR-12",
            "GOAL2", "AC5", "CON1", "P-4.1",
        ]
    );
    assert!(mentioned_ids("").is_empty());
    let bad = ["fr3", "ADR-07", "R-", "FR", "R-2b"].join("と");
    assert!(mentioned_ids(&bad).is_empty(), "{bad} から何も拾わない");
}

#[test]
fn cg9_pairs() {
    let (g, summary) = graph(IDX, LED);
    assert!(summary);
    assert_eq!(unfielded_mentions(&g, summary), known(&PAIRS));

    let (g, summary) = graph(&format!("{IDX}{MORE}"), LED);
    assert!(summary);
    assert_eq!(unfielded_mentions(&g, summary), known(&[]));
}

#[test]
fn cg9_unknown() {
    let (g, _) = graph(IDX, LED);
    assert_eq!(unfielded_mentions(&g, false), Reading::Unknown);

    let (g, summary) = graph("", LED);
    assert_eq!(unfielded_mentions(&g, summary), Reading::Unknown);
    assert_eq!(unfielded_mentions(&g, true), Reading::Unknown);

    let (g, summary) = graph(IDX, "");
    assert!(summary);
    assert_eq!(unfielded_mentions(&g, summary), known(&PAIRS));
}

#[test]
fn cg9_g9_stays_unknown() {
    let (g, _) = graph(IDX, LED);
    let g9 = check(&g)
        .into_iter()
        .find(|i| i.id == "g-9")
        .expect("g-9 が在る");
    assert_eq!(g9.verdict, Verdict::Unknown);
    assert_eq!(UNMEASURED, ["g-9"]);
}
