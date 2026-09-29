//! 行 c-abs-time の歯（中核・接頭辞 abst_・要件 NFR2）: 板と指標の電文は server が組む時の今の時刻から作る値を持たず、
//! 台帳も event も変わらない読み直しでは同じ字になる（札は段を決めた時刻・日の 14 本は日本の日の終わり・
//! 時点は台帳の最後の記録の時刻・memo は作った時刻の中央値）。台帳は歯の中で組み、板は fixture の pipeline.json を読む。

use std::path::PathBuf;

use serde_json::{Value, json};
use tsuzuri_contract::board::{PipelineCard, Reading};
use tsuzuri_contract::stats::LedgerStats;
use tsuzuri_contract::wire;
use tsuzuri_core::ledger::stats;
use tsuzuri_core::ledger::stats::{DAY_OFFSET_S, day_end};
use tsuzuri_core::pipeline::board;

fn fixture(rel: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{rel} は JSON: {e}"))
}

/// 指標を数える（台帳が読めなければ落ちる）。
fn known(ledger: &Value, now: u64) -> LedgerStats {
    match stats(&ledger.to_string(), now) {
        Reading::Known(s) => s,
        Reading::Unknown => panic!("台帳が読めない"),
    }
}

/// (1) 札の since は段を決めた最後の event の ts で、板の電文は今を 1 秒・60 秒・3600 秒ずらしても同じ字。
#[test]
fn abst_pipeline_same_across_now() {
    let v = fixture("tests/fixtures/pipeline/pipeline.json");
    let ledger = v["ledger"].to_string();
    let events = v["events"]
        .as_array()
        .expect("events")
        .iter()
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    let now = v["now"].as_u64().expect("now");
    let text = |at: u64| wire::encode(&board(&ledger, &events, at).board).expect("板の電文");
    let base = text(now);
    for shift in [1, 60, 3600] {
        assert_eq!(text(now + shift), base, "今 + {shift} 秒");
    }
    let Reading::Known(cards) = board(&ledger, &events, now).board.cards else {
        panic!("札が Unknown");
    };
    let since = |id: &str| -> Option<u64> {
        cards
            .iter()
            .find(|c: &&PipelineCard| c.contract.as_str() == id)
            .unwrap_or_else(|| panic!("札 {id}"))
            .since
    };
    // fx-pl.1 は RunDone（2026-09-27T00:10:00Z）・fx-pl.16 は RunStopped の前の RunStage（01:15:00Z）で段を決めた。
    assert_eq!(since("fx-pl.1"), Some(1_790_467_800));
    assert_eq!(since("fx-pl.16"), Some(1_790_471_700));
    assert_eq!(since("fx-pl.13"), None);
    assert_eq!(since("fx-pl.14"), None);
}

/// (2) 日の end は日本の日の 23 時 59 分 59 秒で、14 本は 1 日ずつ続き、昨日と今日の本は日本の日で分ける。
#[test]
fn abst_days_end_on_japan_day() {
    assert_eq!(DAY_OFFSET_S, 32_400);
    assert_eq!(day_end(1_790_478_000), 1_790_521_199);
    assert_eq!(day_end(1_790_434_800), 1_790_521_199);
    assert_eq!(day_end(1_790_434_799), 1_790_434_799);

    // 昨日（2026-09-26 JST）の 01:00 に作り 23:30 に閉じた task と、今日の 00:30 に作った task。
    let ledger = json!([
        {"id": "ab.1", "title": "昨日の task", "status": "closed", "issue_type": "task",
         "created_at": "2026-09-26T01:00:00+09:00", "closed_at": "2026-09-26T23:30:00+09:00"},
        {"id": "ab.2", "title": "今日の task", "status": "open", "issue_type": "task",
         "created_at": "2026-09-27T00:30:00+09:00"}
    ]);
    let s = known(&ledger, 1_790_478_000);
    assert_eq!(s.days.len(), 14);
    assert_eq!(s.days[13].end, 1_790_521_199);
    for w in s.days.windows(2) {
        assert_eq!(w[1].end - w[0].end, 86_400);
    }
    let bar = |i: usize| (s.days[i].created, s.days[i].closed, s.days[i].open);
    assert_eq!(bar(12), (1, 1, 0), "昨日の本");
    assert_eq!(bar(13), (1, 0, 1), "今日の本");
    assert_eq!(s.days[12].end, 1_790_434_799);
}

/// 時点と memo の節の台帳（最後の記録は ab.2 の閉じた 2026-09-27T01:00:00Z・open の memo は 3 本）。
fn record_ledger() -> Value {
    json!([
        {"id": "ab.1", "title": "今日の 00:30 JST に作った task", "status": "open", "issue_type": "task",
         "created_at": "2026-09-26T15:30:00Z"},
        {"id": "ab.2", "title": "今日の 10:00 JST に閉じた task", "status": "closed", "issue_type": "task",
         "created_at": "2026-09-26T00:00:00Z", "closed_at": "2026-09-27T01:00:00Z"},
        {"id": "ab.m1", "title": "[memo] 古い", "status": "open", "issue_type": "task",
         "labels": ["intake:memo"], "created_at": "2026-09-18T03:00:00Z"},
        {"id": "ab.m2", "title": "[memo] 中ほど", "status": "open", "issue_type": "task",
         "labels": ["intake:memo"], "created_at": "2026-09-21T03:00:00Z"},
        {"id": "ab.m3", "title": "[memo] 新しい", "status": "open", "issue_type": "task",
         "labels": ["intake:memo"], "created_at": "2026-09-25T03:00:00Z"}
    ])
}

/// (3) 時点は台帳の最後の記録の時刻（今以下）、memo は open の memo の作った時刻の中央値で、同じ日本の日の今では同じ字。
#[test]
fn abst_stats_same_within_day() {
    let ledger = record_ledger();
    let noon = known(&ledger, 1_790_478_000);
    assert_eq!(noon.at, 1_790_470_800);
    assert_eq!(noon.memo.created_p50, Some(1_789_959_600));
    assert_eq!(noon.memo.open, 3);
    let early = known(&ledger, 1_790_436_600);
    assert_eq!(early.at, 1_790_436_600, "今以下の記録の最大");

    let text = |now: u64| wire::encode(&known(&ledger, now)).expect("指標の電文");
    assert_eq!(text(1_790_478_000), text(1_790_517_600));
    assert_eq!(known(&json!([]), 1_790_478_000).at, 0, "記録が無ければ 0");
    assert_eq!(known(&json!([]), 1_790_478_000).memo.created_p50, None);
}
