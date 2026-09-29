//! 節点の更新の時刻の歯（行 c-updated・要件 FR5）。
//! 契約の型の節点に欄 updated（UTC の epoch 秒）が在り、中核の build が bead は台帳の updated_at から、
//! 走行は event log の読める ts の最後の値から読む。設計の索引の節点と notes の定型行から導く節点は無し。
//! 節の字は歯の中で組む。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::graph::{GraphNode, NodeKind};
use tsuzuri_contract::wire;
use tsuzuri_core::graph::{Inputs, build};

/// 節の索引の字（設計の索引の節点の行 1 つ）。
const IDX: &str = "FR1\t要件\tsrs.yaml\t00000000\t面は 2 つ\n";

/// 節の台帳の字（bead 4 本・updated_at は Z の字・+09:00 の字・欄なし・読めない字）。
const LED: &str = r#"[
  {"id":"fx-1","title":"一つ目","issue_type":"task","status":"open","updated_at":"2026-09-27T07:39:16Z","notes":"受け id = fx-r1"},
  {"id":"fx-2","title":"二つ目","issue_type":"task","status":"open","updated_at":"2026-09-27T16:39:16+09:00"},
  {"id":"fx-3","title":"三つ目","issue_type":"task","status":"open"},
  {"id":"fx-4","title":"四つ目","issue_type":"task","status":"open","updated_at":"昨日"}
]"#;

/// 節の器の event の字（4 件・3 件目の ts は読めない・4 件目は ts が無い）。
const EVENTS: &str = r#"{"run":"fx-1-20260927T073916Z","kind":"RunCreated","ts":"2026-09-27T07:39:16Z"}
{"run":"fx-1-20260927T073916Z","kind":"StageChanged","stage":"Running","ts":"2026-09-27T07:40:00Z"}
{"run":"fx-1-20260927T073916Z","kind":"StageChanged","stage":"Gated","ts":"x"}
{"run":"fx-2-20260927T080000Z","kind":"RunCreated"}
"#;

/// 2026-09-27T07:39:16Z の epoch 秒。
const AT_0739: EpochSecs = 1790494756;
/// 2026-09-27T07:40:00Z の epoch 秒。
const AT_0740: EpochSecs = 1790494800;

#[test]
fn cupd_build_times() {
    let g = build(&Inputs {
        design_index: IDX,
        ledger: LED,
        events: EVENTS,
    });
    assert!(g.unread.is_empty(), "3 つの出所は読める: {:?}", g.unread);
    let mut got: Vec<(&str, NodeKind, Option<EpochSecs>)> = g
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n.kind, n.updated))
        .collect();
    got.sort_by(|a, b| a.0.cmp(b.0));
    assert_eq!(
        got,
        vec![
            ("FR1", NodeKind::Req, None),
            ("fx-1", NodeKind::Task, Some(AT_0739)),
            ("fx-1-20260927T073916Z", NodeKind::Run, Some(AT_0740)),
            ("fx-2", NodeKind::Task, Some(AT_0739)),
            ("fx-2-20260927T080000Z", NodeKind::Run, None),
            ("fx-3", NodeKind::Task, None),
            ("fx-4", NodeKind::Task, None),
            ("fx-r1", NodeKind::Receipt, None),
        ]
    );
}

#[test]
fn cupd_wire_default() {
    let old = r#"{"id":"FR1","kind":"要件","file":"srs.yaml","digest":"00000000","title":"面は 2 つ"}"#;
    let n: GraphNode = wire::decode(old).expect("鍵 updated の無い節点を読む");
    assert_eq!(n.updated, None);
    assert_eq!(n.id, "FR1");

    let stamped = GraphNode {
        updated: Some(AT_0739),
        ..n
    };
    let text = wire::encode(&stamped).expect("encode");
    assert!(
        text.contains("\"updated\":1790494756"),
        "鍵 updated と値の組が在る: {text}"
    );
    assert_eq!(
        wire::decode::<GraphNode>(&text).expect("decode"),
        stamped,
        "同じ節点に読み戻す"
    );
}
