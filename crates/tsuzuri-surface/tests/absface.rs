//! 行 c-abs-time の歯（面・接頭辞 abst_・要件 NFR2）: 札の経過と memo と未反映の年齢は、電文の時刻（段を決めた時刻・
//! 作った時刻）を面の今から引いて出す。CI の語と直近の着地も今で決める。DOM は wasm の target のときだけなので、
//! 1 秒の時計と今の渡し方は src の file の字で見る。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::board::{Ci, PipelineCard, Reading, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::stats::{UnreflectedList, UnreflectedRow};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::ledger::{days_since, unref_list};
use tsuzuri_surface::project::pipeline::{
    CI_MARK_S, LAND_WINDOW_S, NO_AGE, age_at, ci_shown, kcard, landed_recent,
};
use tsuzuri_surface::view::Fetched;

/// 描く今（2026-09-27T12:00:00Z・日本時間の 21:00）。
const NOW: u64 = 1_790_510_400;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn card(stage: Stage, since: Option<u64>, ci: Option<Ci>) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new("fx-ab.1").expect("id"),
        runs: 1,
        stage,
        reason: None,
        account: None,
        since,
        ci,
    }
}

/// (4) 札の経過は今から since を引いた字（今より後は 0s）・CI の語は今までの経過が CI_MARK_S 以下の間だけ・直近の着地は今までの経過が LAND_WINDOW_S 以下。
#[test]
fn abst_card_from_since() {
    assert_eq!(age_at(Some(NOW - 5), NOW), "5s");
    assert_eq!(age_at(Some(NOW - 7_200), NOW), "2h");
    assert_eq!(age_at(Some(NOW + 30), NOW), "0s");
    assert_eq!(age_at(None, NOW), NO_AGE);

    let k = kcard(&card(Stage::Running, Some(NOW - 125), None), &[], NOW);
    assert_eq!(k.age, "2m");
    assert_eq!(k.since, Some(NOW - 125));
    assert_eq!(k.hover.value, "↻1 · ― · 2m");
    // 同じ札を 1 時間後に組めば経過の字だけが進む。
    let later = kcard(&card(Stage::Running, Some(NOW - 125), None), &[], NOW + 3_600);
    assert_eq!(later.age, "1h");
    ci_and_landed();
}

/// CI の語も直近の着地も今までの経過で決まる。
fn ci_and_landed() {
    let ok = |since: u64, now: u64| ci_shown(&card(Stage::Landed, Some(since), Some(Ci::Success)), now);
    assert_eq!(ok(NOW - CI_MARK_S, NOW), Some(Ci::Success));
    assert_eq!(ok(NOW - CI_MARK_S - 1, NOW), None);
    assert_eq!(ok(NOW - 30, NOW + CI_MARK_S), None, "今が進めば語は消える");
    assert_eq!(ok(NOW + 30, NOW), Some(Ci::Success), "今より後の時刻は経過 0");

    let landed = |since: Option<u64>, now: u64| landed_recent(&card(Stage::Landed, since, None), now);
    assert!(landed(Some(NOW - LAND_WINDOW_S), NOW));
    assert!(!landed(Some(NOW - LAND_WINDOW_S - 1), NOW));
    assert!(landed(Some(NOW - 60), NOW + LAND_WINDOW_S - 60), "今が進んでも範囲の内は残る");
    assert!(!landed(Some(NOW - 60), NOW + LAND_WINDOW_S - 59), "今が進めば列から落ちる");
    assert!(!landed(None, NOW));
}

fn row(id: &str, created: Option<u64>) -> UnreflectedRow {
    UnreflectedRow {
        id: id.to_string(),
        title: format!("{id} の題"),
        created,
    }
}

/// (5) 年齢は作った時刻から今までの日数（今より後は 0）で、未反映は created から数える（memo の段は行 g-ledger-trim で外した）。
#[test]
fn abst_ages_from_created() {
    assert_eq!(days_since(NOW - 86_400, NOW), 1.0);
    assert_eq!(days_since(NOW - 820_800, NOW), 9.5);
    assert_eq!(days_since(NOW + 60, NOW), 0.0);
    assert_eq!(days_since(NOW, NOW), 0.0);

    let list = UnreflectedList {
        memos: Reading::Known(vec![
            row("m-1", Some(NOW - 18_000)),
            row("m-2", Some(NOW + 600)),
            row("m-3", None),
        ]),
        rulings: Reading::Known(vec![]),
        utterances: Reading::Known(vec![]),
        stale: vec![],
    };
    let fetched = Fetched::Body(wire::encode(&list).expect("電文"));
    let Body::Filled(l) = unref_list(&fetched, NOW) else {
        panic!("一覧が Filled でない");
    };
    let ages: Vec<&str> = l.rows.iter().map(|r| r.age.as_str()).collect();
    assert_eq!(ages, vec!["5h", "0h", "―"]);
}

/// (6) 面の 2 つの file は 1 秒の時計と今を渡し、経過と年齢の秒の欄の字を持たない（台帳の一覧の段の今は行 g-list-groups から module ledgerlist が渡す）。
#[test]
fn abst_face_wiring() {
    let pipeline = read("src/project/pipeline.rs");
    let ledger = read("src/project/ledger.rs");
    let list = read("src/ledgerlist.rs");
    for (text, needle) in [
        (&pipeline, "let clock = move || tick.get();"),
        (&pipeline, "age_at(since, clock())"),
        (&ledger, "unref_list(&unref.get(), crate::net::now())"),
        (&list, "content(l, p, b, crate::net::now())"),
    ] {
        assert_eq!(text.matches(needle).count(), 1, "字 {needle} の数");
    }
    for text in [&pipeline, &ledger] {
        for gone in ["elapsed_s", ".age_s", "age_s:", "age_p50_days"] {
            assert!(!text.contains(gone), "字 {gone} が残る");
        }
    }
}
