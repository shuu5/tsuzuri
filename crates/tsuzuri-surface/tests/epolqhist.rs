//! 行 e-policy-q の面の歯（接頭辞 epolq_）: block「これまでの決定」は方針の問い（範囲の札の label を持つ）を
//! 出さず数えない（要件 FR8）。方針の問いの label は境界の policy の `create_write` の書きの labels を使う。

use tsuzuri_boundary::server::policy;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadId, LedgerList, LedgerRow, LedgerWrite, QUESTION_LABEL};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::{Body, askpage};
use tsuzuri_surface::view::Fetched;

fn id(s: &str) -> BeadId {
    BeadId::new(s).expect("bead id")
}

/// 境界が方針の問いを作る書きの labels（範囲 `scope`）。
fn policy_labels(scope: &str) -> Vec<String> {
    match policy::create_write(&id("fx-p"), scope, "急がない") {
        LedgerWrite::CreateChild { labels, .. } => labels,
        other => panic!("子を作る書きでない: {other:?}"),
    }
}

fn row(rid: &str, status: &str, labels: Vec<String>) -> LedgerRow {
    LedgerRow {
        id: id(rid),
        kind: "task".into(),
        title: format!("{rid} の題"),
        status: status.into(),
        updated_at: 1_790_584_890,
        parent: Some(id("fx-p")),
        labels,
    }
}

fn fetched(rows: Vec<LedgerRow>) -> Fetched {
    Fetched::Body(
        wire::encode(&LedgerList {
            rows: Reading::Known(rows),
        })
        .expect("電文"),
    )
}

#[test]
fn epolq_hist_skips_policies() {
    let question = || vec![QUESTION_LABEL.to_string()];
    let rows = vec![
        row("fx-p.3", "closed", question()),
        row("fx-p.7", "closed", policy_labels("all")),
        row("fx-p.8", "closed", policy_labels("fx-p.2")),
        row("fx-p.9", "open", policy_labels("all")),
        row("fx-p.10", "closed", question()),
        row("fx-p.2", "open", question()),
    ];
    let ids = |rows: &[LedgerRow]| -> Vec<String> {
        rows.iter().map(|r| r.id.to_string()).collect()
    };
    assert_eq!(ids(&askpage::rulings(&rows)), ["fx-p.3", "fx-p.10"]);

    let list = fetched(rows.clone());
    assert_eq!(askpage::count(&list), Reading::Known(2));
    let Body::Filled(items) = askpage::body(&list) else {
        panic!("中身が在る段が埋まらない");
    };
    let item_ids: Vec<&str> = items.iter().map(|it| it.id.as_str()).collect();
    assert_eq!(item_ids, ["fx-p.3", "fx-p.10"]);
    assert!(!askpage::opens(&list, Some("fx-p.7")));
    assert!(askpage::opens(&list, Some("fx-p.10")));

    let only = fetched(rows[1..3].to_vec());
    assert_eq!(askpage::body(&only), Body::Empty(askpage::EMPTY));
    assert_eq!(askpage::count(&only), Reading::Known(0));
}
