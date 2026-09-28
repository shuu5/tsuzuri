//! 口座の 7 日の線の歯（接頭辞 cspk_・設計ノート surface-wave15b 行 c-acct-spark の完了の条件）。
//! log の行は器の AllowanceMeasured の行の形（共通の欄と account・window・endpoint・used_pct・任意の model）で組む。

use std::collections::BTreeMap;

use serde_json::{Value, json};
use tsuzuri_contract::account::{SPARK_SPAN_S, SPARK_STEP_S, Spark, SparkLine, SparkPoint};
use tsuzuri_contract::board::Reading;
use tsuzuri_core::account::host::{HostTexts, MEASURED_EVENT, accounts, accounts_at, spark};
use tsuzuri_core::account::project::doc;

/// 読みの今の時刻（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

/// epoch 秒を RFC 3339 の Z の字にする（log の行の ts）。
fn rfc3339(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem / 60 % 60,
        rem % 60
    )
}

/// 測りの行の JSON（欄を足し替えて読まない行も組む）。
fn event(account: &str, window: &str, ts: &str, used_pct: Value) -> Value {
    json!({
        "schema": 1,
        "ts": ts,
        "kind": MEASURED_EVENT,
        "host": "host-1",
        "actor": "machine",
        "account": account,
        "window": window,
        "endpoint": "usage",
        "used_pct": used_pct,
    })
}

/// 測りの行の 1 行（今から `ago` 秒前・負なら後）。
fn measured(account: &str, window: &str, ago: i64, used_pct: u64) -> String {
    let at = (NOW as i64 - ago) as u64;
    event(account, window, &rfc3339(at), json!(used_pct)).to_string()
}

fn log(lines: &[String]) -> String {
    lines.iter().map(|l| format!("{l}\n")).collect()
}

fn texts(events: Option<String>) -> HostTexts {
    HostTexts {
        events,
        ..HostTexts::default()
    }
}

/// 窓の線（点は (at, 使った割合) の列）。
fn line(window: &str, points: &[(u64, u8)]) -> SparkLine {
    SparkLine {
        window: window.to_string(),
        points: points
            .iter()
            .map(|&(at, used_pct)| SparkPoint { at, used_pct })
            .collect(),
    }
}

/// 最後に測った時刻と点の無い 3 本。
fn bare(measured_at: Option<u64>) -> Reading<Spark> {
    Reading::Known(Spark {
        measured_at,
        lines: vec![
            line("five_hour", &[]),
            line("seven_day", &[]),
            line("seven_day_model", &[]),
        ],
    })
}

#[test]
fn cspk_consts_and_event_word() {
    assert_eq!(SPARK_SPAN_S, 604_800);
    assert_eq!(SPARK_STEP_S, 3_600);
    assert_eq!(SPARK_SPAN_S / SPARK_STEP_S, 168);
    assert_eq!(MEASURED_EVENT, "AllowanceMeasured");
    assert_eq!(rfc3339(NOW), "2026-09-27T12:00:00Z");
    assert_eq!(rfc3339(1_789_905_601), "2026-09-20T12:00:01Z");
}

#[test]
fn cspk_steps_newest_per_hour() {
    let ts = rfc3339(NOW - 200);
    let mut model = event("acct-1", "seven_day_model", &ts, json!(12));
    model["model"] = json!("opus");
    let mut no_pct = event("acct-1", "five_hour", &ts, json!(1));
    no_pct.as_object_mut().expect("欄の表").remove("used_pct");
    let mut unmeasured = event("acct-1", "five_hour", &ts, json!(1));
    unmeasured["kind"] = json!("AllowanceUnmeasured");
    unmeasured["reason"] = json!("no-token");
    let text = log(&[
        measured("acct-1", "five_hour", 3599, 38),
        measured("acct-1", "five_hour", 10, 42),
        measured("acct-1", "five_hour", 100, 40),
        measured("acct-1", "five_hour", 3600, 30),
        measured("acct-1", "five_hour", 8000, 31),
        measured("acct-1", "five_hour", 8000, 33),
        measured("acct-1", "five_hour", 604_799, 5),
        measured("acct-1", "five_hour", 604_800, 6),
        measured("acct-1", "five_hour", -60, 99),
        measured("acct-1", "seven_day", 200, 300),
        model.to_string(),
        // 読まない行。
        measured("acct-1", "seven_day_opus", 200, 1),
        event("acct-1", "five_hour", &ts, json!("12")).to_string(),
        no_pct.to_string(),
        event("acct-1", "five_hour", "bad", json!(1)).to_string(),
        measured("", "five_hour", 200, 1),
        unmeasured.to_string(),
        "AllowanceMeasured {\"account\":\"acct-1\"".to_string(),
        measured("acct-2", "seven_day", 700_000, 50),
    ]);
    let host = texts(Some(text));
    assert_eq!(
        spark(&host, "acct-1", NOW),
        Reading::Known(Spark {
            measured_at: Some(1_790_510_390),
            lines: vec![
                line(
                    "five_hour",
                    &[
                        (1_789_905_601, 5),
                        (1_790_502_400, 33),
                        (1_790_506_800, 30),
                        (1_790_510_390, 42),
                    ]
                ),
                line("seven_day", &[(1_790_510_200, 255)]),
                line("seven_day_model", &[(1_790_510_200, 12)]),
            ],
        })
    );
    assert_eq!(spark(&host, "acct-2", NOW), bare(Some(1_789_810_400)));
    assert_eq!(spark(&host, "acct-3", NOW), bare(None));
    assert_eq!(spark(&texts(None), "acct-1", NOW), Reading::Unknown);
    assert_eq!(spark(&texts(Some(String::new())), "acct-1", NOW), bare(None));
}

#[test]
fn cspk_at_most_168_points() {
    // 今は時の境の 30 分後。
    let now = NOW + 1_800;
    let rows: Vec<String> = (0..=1152u64)
        .rev()
        .map(|k| {
            let at = now - k * 600;
            event("acct-1", "five_hour", &rfc3339(at), json!(k % 100)).to_string()
        })
        .collect();
    assert_eq!(rows.len(), 1153);
    let Reading::Known(got) = spark(&texts(Some(log(&rows))), "acct-1", now) else {
        panic!("線が Unknown");
    };
    assert_eq!(got.measured_at, Some(now));
    let points = &got.lines[0].points;
    assert_eq!(got.lines[0].window, "five_hour");
    assert_eq!(points.len(), 168);
    assert!(points.windows(2).all(|w| w[0].at < w[1].at), "at の古い順");
    assert!(points[0].at > now - SPARK_SPAN_S);
    assert_eq!(points.last().map(|p| p.at), Some(now));
    for p in points {
        assert_eq!((now - p.at) % SPARK_STEP_S, 0, "刻みの中の最も新しい行: {p:?}");
        assert_eq!(u64::from(p.used_pct), (now - p.at) / 600 % 100);
    }
    assert!(got.lines[1].points.is_empty() && got.lines[2].points.is_empty());
}

#[test]
fn cspk_accounts_at_fills_only_spark() {
    let host = HostTexts {
        host_toml: Some(
            "[[account]]\nlabel = \"acct-1\"\n\n[[account]]\nlabel = \"acct-2\"\n\n[[account]]\nlabel = \"acct-3\"\n"
                .to_string(),
        ),
        usage: Some(
            "usage: account=acct-1 five_hour=83% resets=none seven_day=40% resets=none model=opus:12% resets=none\n"
                .to_string(),
        ),
        doctor: Some(
            "doctor: state dir ok\naccount=acct-1 retired=no\naccount=acct-2 retired=no\naccount=acct-3 retired=yes\n"
                .to_string(),
        ),
        events: Some(log(&[
            measured("acct-1", "five_hour", 600, 42),
            measured("acct-3", "seven_day", 90_000, 70),
        ])),
        ..HostTexts::default()
    };
    let Reading::Known(plain) = accounts(&host) else {
        panic!("口座の列が Unknown");
    };
    assert_eq!(plain.len(), 3);
    assert!(plain.iter().all(|r| r.spark == Reading::Unknown));
    let Reading::Known(filled) = accounts_at(&host, NOW) else {
        panic!("口座の列が Unknown");
    };
    assert_eq!(filled.len(), 3);
    for (p, f) in plain.iter().zip(&filled) {
        assert_ne!(f.spark, Reading::Unknown, "{}", f.label);
        assert_eq!(f.spark, spark(&host, &f.label, NOW), "{}", f.label);
        let mut same = f.clone();
        same.spark = Reading::Unknown;
        assert_eq!(&same, p, "線のほかは同じ");
    }
    assert!(filled[2].retired);
    assert_eq!(
        filled[2].spark,
        Reading::Known(Spark {
            measured_at: Some(1_790_420_400),
            lines: vec![
                line("five_hour", &[]),
                line("seven_day", &[(1_790_420_400, 70)]),
                line("seven_day_model", &[]),
            ],
        })
    );
    assert_eq!(filled[1].spark, bare(None));
    assert_eq!(
        doc(&host, &BTreeMap::new(), None, NOW).accounts,
        Reading::Known(filled)
    );
    let no_log = HostTexts {
        events: None,
        ..host.clone()
    };
    assert_eq!(accounts_at(&no_log, NOW), accounts(&no_log));
    let no_toml = HostTexts {
        host_toml: None,
        ..host
    };
    assert_eq!(accounts_at(&no_toml, NOW), Reading::Unknown);
}
