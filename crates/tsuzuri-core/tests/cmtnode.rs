//! 着地の commit の節点の歯（中核・接頭辞 cmtnode_・設計ノート surface-wave29a 行 c-commit-node・判断の記録 ADR-46 決定 (2)・
//! ADR-47・ADR-45 の門 H4）。器の event log の RunDone の detail の札 sha: の 40 字の小文字の 16 進から commit の節点を組み、
//! commit から走行の bead へ landed の辺を組む。着地の commit の節点から landed の辺を受けない着地した契約は床の値
//! unlanded_contracts が名指す。否定の見本は正しい見本から 1 句だけ替える。節の字は歯の中で組む。
#![cfg(test)]
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::{EdgeType, GraphNode, NodeKind};
use tsuzuri_contract::wire;
use tsuzuri_core::graph::build::{SHA_LEN, SHA_TAG, SHA_TITLE, landed_sha};
use tsuzuri_core::graph::check::unlanded_contracts;
use tsuzuri_core::graph::{Graph, Inputs, Source, Verdict, build, check};

/// 着地の commit の名（40 字の小文字の 16 進）。
const SHA_A: &str = "85bb529a8e67fa35652253b1bce533675eba145a";
const SHA_B: &str = "7803a784c0ffee00112233445566778899aabbcc";

/// 節の索引の字（設計の索引の節点の行 1 つ）。
const IDX: &str = "FR1\t要件\tsrs.yaml\t00000000\t面は 2 つ\n";

/// 2026-09-27T07:40:00Z の epoch 秒（RunDone の ts）。
const AT_0740: EpochSecs = 1790494800;

/// 節の台帳の字（契約の bead 6 本と memo 1 本）。fx-1 と fx-2 は landed で閉じ、fx-3 は「着地（便」で閉じ、
/// fx-4 は取り下げで閉じ、fx-5 は open、fx-6 は landed で閉じた memo、fx-7 は着地で閉じた契約。
const LED: &str = r#"[
  {"id":"fx-1","title":"一つ目","issue_type":"task","status":"closed","close_reason":"landed 85bb529a ci=success"},
  {"id":"fx-2","title":"二つ目","issue_type":"task","status":"closed","close_reason":"landed 7803a784 ci=success"},
  {"id":"fx-3","title":"三つ目","issue_type":"task","status":"closed","close_reason":"着地（便 fx-3 の着地）"},
  {"id":"fx-4","title":"四つ目","issue_type":"task","status":"closed","close_reason":"取り下げ 中身は fx-1 の便に載った"},
  {"id":"fx-5","title":"五つ目","issue_type":"task","status":"open"},
  {"id":"fx-6","title":"六つ目","issue_type":"task","status":"closed","labels":["intake:memo"],"close_reason":"landed"},
  {"id":"fx-7","title":"七つ目","issue_type":"task","status":"closed","close_reason":"着地"}
]"#;

/// 走行 1 本の RunCreated の行。
fn created(run: &str) -> String {
    format!(r#"{{"run":"{run}","kind":"RunCreated","ts":"2026-09-27T07:39:16Z"}}"#)
}

/// 走行 1 本の event の行（種類と detail）。
fn event(run: &str, kind: &str, detail: &str) -> String {
    format!(r#"{{"run":"{run}","kind":"{kind}","ts":"2026-09-27T07:40:00Z","detail":"{detail}"}}"#)
}

/// 行を改行でつないだ event log の字。
fn log(lines: &[String]) -> String {
    lines.iter().map(|l| format!("{l}\n")).collect()
}

const RUN_1: &str = "fx-1-20260927T073916Z";
const RUN_2: &str = "fx-2-20260927T080000Z";

/// 正しい見本（fx-1 の走行の RunDone が sha:A と main:B を持つ）。
fn good_events() -> String {
    log(&[
        created(RUN_1),
        event(RUN_1, "RunDone", &format!("sha:{SHA_A} main:{SHA_B}")),
    ])
}

fn built(events: &str) -> Graph {
    build(&Inputs {
        design_index: IDX,
        ledger: LED,
        events,
    })
}

fn commits(g: &Graph) -> Vec<&GraphNode> {
    g.nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Commit)
        .collect()
}

fn landed(g: &Graph) -> Vec<(&str, &str)> {
    g.edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::Landed)
        .map(|e| (e.from.as_str(), e.to.as_str()))
        .collect()
}

#[test]
fn cmtnode_closed_lists_end_with_commit_and_landed() {
    assert_eq!(NodeKind::ALL.len(), 21);
    assert_eq!(NodeKind::ALL[20], NodeKind::Commit);
    assert_eq!(wire::encode(&NodeKind::Commit).expect("語"), "\"commit\"");
    assert_eq!(EdgeType::ALL.len(), 33);
    assert_eq!(EdgeType::ALL[32], EdgeType::Landed);
    assert_eq!(wire::encode(&EdgeType::Landed).expect("語"), "\"landed\"");
    assert_eq!(Source::of(NodeKind::Commit), Some(Source::Runs));
    assert_eq!(Source::Runs.kinds(), &[NodeKind::Run, NodeKind::Commit]);
    assert_eq!((SHA_TAG, SHA_LEN, SHA_TITLE), ("sha:", 40, 8));
}

#[test]
fn cmtnode_run_done_sha_makes_commit_and_landed() {
    let g = built(&good_events());
    assert!(g.unread.is_empty(), "{:?}", g.unread);
    let got = commits(&g);
    assert_eq!(got.len(), 1, "main: の札の名は節点にしない");
    let n = got[0];
    assert_eq!(n.id, SHA_A);
    assert_eq!(n.title, "85bb529a");
    assert_eq!(n.updated, Some(AT_0740));
    assert_eq!(
        (&n.file, &n.digest, n.line, &n.plain, &n.eng),
        (&None, &None, None, &None, &None)
    );
    assert_eq!(landed(&g), vec![(SHA_A, "fx-1")]);
    assert!(g.node(SHA_B).is_none());
}

/// 否定の見本（正しい見本の RunDone の行から 1 句だけ替える・理由の字と event log の字）。
fn negative_cases() -> Vec<(&'static str, String)> {
    let short = &SHA_A[..39];
    let long = format!("{SHA_A}0");
    let upper = SHA_A.to_uppercase();
    let not_hex = format!("{}g", &SHA_A[..39]);
    vec![
        (
            "39 字",
            log(&[
                created(RUN_1),
                event(RUN_1, "RunDone", &format!("sha:{short}")),
            ]),
        ),
        (
            "41 字",
            log(&[
                created(RUN_1),
                event(RUN_1, "RunDone", &format!("sha:{long}")),
            ]),
        ),
        (
            "大文字",
            log(&[
                created(RUN_1),
                event(RUN_1, "RunDone", &format!("sha:{upper}")),
            ]),
        ),
        (
            "16 進でない字",
            log(&[
                created(RUN_1),
                event(RUN_1, "RunDone", &format!("sha:{not_hex}")),
            ]),
        ),
        (
            "札が main: だけ",
            log(&[
                created(RUN_1),
                event(RUN_1, "RunDone", &format!("main:{SHA_A}")),
            ]),
        ),
        (
            "RunDone でない event",
            log(&[
                created(RUN_1),
                event(RUN_1, "RunStage", &format!("sha:{SHA_A}")),
            ]),
        ),
        (
            "RunCreated の無い走行",
            log(&[
                created(RUN_2),
                event(RUN_1, "RunDone", &format!("sha:{SHA_A}")),
            ]),
        ),
    ]
}

#[test]
fn cmtnode_only_full_lower_hex_sha_of_run_done_counts() {
    let short = &SHA_A[..39];
    for (why, events) in negative_cases() {
        let g = built(&events);
        assert!(g.unread.is_empty(), "{why}: {:?}", g.unread);
        assert!(commits(&g).is_empty(), "{why}");
        assert!(landed(&g).is_empty(), "{why}");
    }
    assert_eq!(landed_sha(&format!("sha:{SHA_A}")), Some(SHA_A));
    assert_eq!(landed_sha(&format!("terminal:x,sha:{SHA_A}")), Some(SHA_A));
    assert_eq!(landed_sha(&format!("main:{SHA_A}")), None);
    assert_eq!(landed_sha(&format!("sha:{short}")), None);
}

#[test]
fn cmtnode_same_sha_is_one_node_and_one_edge_per_bead() {
    let events = log(&[
        created(RUN_1),
        created(RUN_2),
        event(RUN_1, "RunDone", &format!("sha:{SHA_A} main:{SHA_A}")),
        event(RUN_1, "RunDone", &format!("sha:{SHA_A} main:{SHA_A}")),
        event(RUN_2, "RunDone", &format!("sha:{SHA_A} main:{SHA_A}")),
        event(RUN_2, "RunDone", &format!("sha:{SHA_B} main:{SHA_B}")),
    ]);
    let g = built(&events);
    let ids: Vec<&str> = commits(&g).iter().map(|n| n.id.as_str()).collect();
    assert_eq!(ids, vec![SHA_A, SHA_B], "event log の順・名ごとに 1 つ");
    assert_eq!(
        landed(&g),
        vec![(SHA_A, "fx-1"), (SHA_A, "fx-2"), (SHA_B, "fx-2")]
    );
}

#[test]
fn cmtnode_run_without_bead_front_has_node_but_no_edge() {
    let events = log(&[
        created("free-run"),
        event("free-run", "RunDone", &format!("sha:{SHA_A}")),
    ]);
    let g = built(&events);
    assert_eq!(commits(&g).len(), 1);
    assert!(landed(&g).is_empty());
}

#[test]
fn cmtnode_unread_events_make_commit_unknown() {
    let g = built("");
    assert_eq!(g.unread, vec![Source::Runs]);
    assert!(g.unknown_kinds().contains(&NodeKind::Commit));
    assert_eq!(g.count_nodes(NodeKind::Commit), 0);
}

#[test]
fn cmtnode_invariants_hold_with_commits() {
    let g = built(&good_events());
    let verdict = |id: &str| {
        check(&g)
            .into_iter()
            .find(|i| i.id == id)
            .map(|i| i.verdict)
            .expect("不変条件")
    };
    assert_eq!(verdict("g-1"), Verdict::Pass);
    assert_eq!(verdict("g-10"), Verdict::Pass);
}

#[test]
fn cmtnode_unlanded_names_landed_contracts_without_commit() {
    let both = log(&[
        created(RUN_1),
        created(RUN_2),
        event(RUN_1, "RunDone", &format!("sha:{SHA_A} main:{SHA_A}")),
        event(RUN_2, "RunDone", &format!("sha:{SHA_B} main:{SHA_B}")),
    ]);
    let g = built(&both);
    assert_eq!(
        unlanded_contracts(&g, LED),
        Reading::Known(vec!["fx-3".to_string(), "fx-7".to_string()]),
        "landed と着地の頭の契約だけ・取り下げと open と memo は数えない"
    );
    let one = log(&[
        created(RUN_1),
        created(RUN_2),
        event(RUN_1, "RunDone", &format!("sha:{SHA_A} main:{SHA_A}")),
        event(RUN_2, "RunDone", "terminal:close:ok"),
    ]);
    let g = built(&one);
    assert_eq!(
        unlanded_contracts(&g, LED),
        Reading::Known(vec![
            "fx-2".to_string(),
            "fx-3".to_string(),
            "fx-7".to_string()
        ]),
        "sha の札の無い RunDone は着地の commit を結ばない"
    );
}

#[test]
fn cmtnode_unlanded_unknown_when_unread() {
    let unread_runs = built("");
    assert_eq!(unlanded_contracts(&unread_runs, LED), Reading::Unknown);
    let unread_ledger = build(&Inputs {
        design_index: IDX,
        ledger: "",
        events: &good_events(),
    });
    assert_eq!(unlanded_contracts(&unread_ledger, LED), Reading::Unknown);
    let g = built(&good_events());
    assert_eq!(unlanded_contracts(&g, "x"), Reading::Unknown);
    assert_eq!(unlanded_contracts(&g, ""), Reading::Unknown);
}
