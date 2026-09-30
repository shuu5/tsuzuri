//! 行 c-abs-time の歯（面・接頭辞 abst_・要件 NFR2）: 札の経過と memo と未反映の年齢は、電文の時刻（段を決めた時刻・
//! 作った時刻）を面の今から引いて出す。CI の語と今日の着地も今で決める。DOM は wasm の target のときだけなので、
//! 1 秒の時計と今の渡し方は src の file の字で見る。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::{Ci, PipelineCard, Reading, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::stats::{LedgerStats, UnreflectedList, UnreflectedRow};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::ledger::{days_since, panel, unref_list};
use tsuzuri_surface::project::pipeline::{
    CI_MARK_S, NO_AGE, age_at, ci_shown, kcard, landed_today,
};
use tsuzuri_surface::view::{Fetched, Screen};

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

/// (4) 札の経過は今から since を引いた字（今より後は 0s）・CI の語は今までの経過が CI_MARK_S 以下の間だけ・今日は since の日本の日。
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

    let ok = |since: u64, now: u64| ci_shown(&card(Stage::Landed, Some(since), Some(Ci::Success)), now);
    assert_eq!(ok(NOW - CI_MARK_S, NOW), Some(Ci::Success));
    assert_eq!(ok(NOW - CI_MARK_S - 1, NOW), None);
    assert_eq!(ok(NOW - 30, NOW + CI_MARK_S), None, "今が進めば語は消える");
    assert_eq!(ok(NOW + 30, NOW), Some(Ci::Success), "今より後の時刻は経過 0");

    let landed = |since: Option<u64>, now: u64| landed_today(&card(Stage::Landed, since, None), now);
    // NOW は日本時間の 21:00 で、その日の 0 時は NOW - 75_600。
    assert!(landed(Some(NOW - 75_600), NOW));
    assert!(!landed(Some(NOW - 75_601), NOW));
    assert!(landed(Some(NOW - 75_600), NOW + 10_799), "日の終わりの 23:59:59 までは今日");
    assert!(!landed(Some(NOW - 75_600), NOW + 10_800), "翌日の 0 時からは昨日");
    assert!(!landed(None, NOW));
}

/// 指標の fixture の組 filled。
fn filled() -> LedgerStats {
    let mut sets: BTreeMap<String, LedgerStats> =
        wire::decode(&read("../../tests/fixtures/surface/ledger-stats.json"))
            .expect("fixture の組が電文として読める");
    sets.remove("filled").expect("fixture の組 filled")
}

fn row(id: &str, created: Option<u64>) -> UnreflectedRow {
    UnreflectedRow {
        id: id.to_string(),
        title: format!("{id} の題"),
        created,
    }
}

/// (5) 年齢は作った時刻から今までの日数（今より後は 0）で、memo は created_p50 から・未反映は created から数える。
#[test]
fn abst_ages_from_created() {
    assert_eq!(days_since(NOW - 86_400, NOW), 1.0);
    assert_eq!(days_since(NOW - 820_800, NOW), 9.5);
    assert_eq!(days_since(NOW + 60, NOW), 0.0);
    assert_eq!(days_since(NOW, NOW), 0.0);

    let screen = Screen::initial();
    let mut s = filled();
    s.memo.created_p50 = Some(NOW - 820_800);
    assert_eq!(panel(&s, &screen, NOW).memo.age, "9.5d");
    s.memo.created_p50 = None;
    assert_eq!(panel(&s, &screen, NOW).memo.age, "―");

    let list = UnreflectedList {
        memos: Reading::Known(vec![
            row("m-1", Some(NOW - 18_000)),
            row("m-2", Some(NOW + 600)),
            row("m-3", None),
        ]),
        rulings: Reading::Known(vec![]),
        requests: Reading::Known(vec![]),
    };
    let fetched = Fetched::Body(wire::encode(&list).expect("電文"));
    let Body::Filled(l) = unref_list(&fetched, NOW) else {
        panic!("一覧が Filled でない");
    };
    let ages: Vec<&str> = l.rows.iter().map(|r| r.age.as_str()).collect();
    assert_eq!(ages, vec!["5h", "0h", "―"]);
}

/// (6) 面の 2 つの file は 1 秒の時計と今を渡し、経過と年齢の秒の欄の字を持たない。
#[test]
fn abst_face_wiring() {
    let pipeline = read("src/project/pipeline.rs");
    let ledger = read("src/project/ledger.rs");
    for (text, needle) in [
        (&pipeline, "let clock = move || tick.get();"),
        (&pipeline, "age_at(since, clock())"),
        (&ledger, "content(f, s, crate::net::now())"),
        (&ledger, "unref_list(&unref.get(), crate::net::now())"),
        (&ledger, "stages(p, crate::net::now())"),
    ] {
        assert_eq!(text.matches(needle).count(), 1, "字 {needle} の数");
    }
    for text in [&pipeline, &ledger] {
        for gone in ["elapsed_s", ".age_s", "age_s:", "age_p50_days"] {
            assert!(!text.contains(gone), "字 {gone} が残る");
        }
    }
}
