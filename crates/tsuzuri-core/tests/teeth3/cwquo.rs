//! 席が自分で開く問う窓の数えの歯（接頭辞 cwquo_・設計ノート surface-wave27a 行 cs-quota・規則の行 R-38）。
//! 台帳の相談の開きの行は歯の中で字に組み、中核の lines::scan で読む。上限の値は rules の R-38 の value の字と照らす。
#![cfg(test)]

use std::path::Path;

use crate::common::win;
use tsuzuri_contract::consult::SeatQuota;
use tsuzuri_core::consult::lines::scan;
use tsuzuri_core::consult::quota::{DAY_MAX, LIVE_MAX, REFUSAL, admit, count};

/// 開きの行（窓・形・起こし手の字・結果の字〔撃ち直しの印を含めてよい〕・時刻）。
fn open(w: &str, form: &str, by: &str, result: &str, at: &str) -> String {
    format!(
        "相談の開き = {w}・形 = {form}・題 = 題なし・起こし手 = {by}・model = fable・念入りさ = xhigh・結果 = {result}・時刻 = {at}"
    )
}

const SEAT: &str = "席";
const CHAT: &str = "持ち主のチャット・発話 = 20261003T0900Z";
const BUTTON: &str = "持ち主の button・頼み = rq-20261003T0900Z-1";

fn notes() -> String {
    [
        open("cw1", "問う", SEAT, "開いた", "20261003T0100Z"),
        open("cw2", "問う", SEAT, "開いた", "20261003T2359Z"),
        open("cw3", "問う", SEAT, "落ちた", "20261003T1000Z"),
        open("cw4", "話す", SEAT, "開いた", "20261003T1000Z"),
        open("cw5", "問う", CHAT, "開いた", "20261003T1000Z"),
        open("cw6", "問う", BUTTON, "開いた", "20261003T1000Z"),
        open(
            "cw1",
            "問う",
            SEAT,
            "開いた・撃ち直し = はい",
            "20261003T1100Z",
        ),
        open("cw7", "問う", SEAT, "開いた", "20261002T2359Z"),
        open("cw8", "問う", SEAT, "開いた", "20261004T0000Z"),
    ]
    .join("\n")
}

#[test]
fn cwquo_counts_seat_ask_only() {
    let lines = scan(&notes());
    assert_eq!(lines.len(), 9);
    assert_eq!(
        count(&lines, "20261003T1200Z", &[]),
        SeatQuota { today: 2, live: 0 }
    );
    assert_eq!(count(&[], "20261003T1200Z", &[]), SeatQuota::default());
}

#[test]
fn cwquo_utc_day_boundary() {
    let lines = scan(&notes());
    assert_eq!(count(&lines, "20261003T0000Z", &[]).today, 2);
    assert_eq!(count(&lines, "20261003T2359Z", &[]).today, 2);
    assert_eq!(count(&lines, "20261002T2359Z", &[]).today, 1);
    assert_eq!(count(&lines, "20261004T0000Z", &[]).today, 1);
    assert_eq!(count(&lines, "20261005T0000Z", &[]).today, 0);
    assert_eq!(count(&lines, "", &[]).today, 0);
}

#[test]
fn cwquo_live_windows() {
    let lines = scan(&notes());
    let now = "20261003T1200Z";
    let live = [
        win("cw1"),
        win("cw4"),
        win("cw5"),
        win("cw6"),
        win("cw3"),
        win("cw9"),
    ];
    assert_eq!(count(&lines, now, &live).live, 1);
    assert_eq!(
        count(&lines, now, &[win("cw1"), win("cw2"), win("cw7")]).live,
        3
    );
    let again_only = scan(&open(
        "cw10",
        "問う",
        SEAT,
        "開いた・撃ち直し = はい",
        "20261003T1000Z",
    ));
    assert_eq!(
        count(&again_only, now, &[win("cw10")]),
        SeatQuota { today: 0, live: 1 }
    );
}

#[test]
fn cwquo_admit_limits() {
    let q = |today, live| SeatQuota { today, live };
    assert_eq!(admit(q(0, 0), false), Ok(()));
    assert_eq!(admit(q(2, 0), false), Ok(()));
    assert_eq!(admit(q(0, 1), false), Err(REFUSAL));
    assert_eq!(admit(q(3, 0), false), Err(REFUSAL));
    assert_eq!(admit(q(9, 9), true), Ok(()));
    assert_eq!(
        REFUSAL,
        "R-38 の上限（同時 1 本・1 日 3 本）で断った・決定の画面で持ち主に問う"
    );
    assert!(REFUSAL.contains(&format!("同時 {LIVE_MAX} 本・1 日 {DAY_MAX} 本")));
}

#[test]
fn cwquo_values_match_rule_r38() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../design-intent/rules.yaml");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
    let rows: Vec<&str> = text.lines().filter(|l| l.contains("id: R-38,")).collect();
    assert_eq!(rows.len(), 1, "R-38 の行");
    let value = rows[0]
        .split("value: \"")
        .nth(1)
        .and_then(|v| v.split('"').next())
        .expect("value");
    assert!(value.contains(&format!("同時に {LIVE_MAX} 本")), "{value}");
    assert!(
        value.contains(&format!("1 日（UTC の日）{DAY_MAX} 本まで")),
        "{value}"
    );
    assert!(value.contains("止まって撃ち直した窓は数えない"), "{value}");
}
