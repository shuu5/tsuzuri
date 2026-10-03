//! 行 c-limit-resume の歯（中核）: 席の card の欄 reopens は合図の健康の出力の席の行の欄 `reopens=` の写し。
//! fixture: tests/fixtures/seat/seat-inputs.json（組の名 → 7 つの字と期待の card）と
//! tests/fixtures/account/acct-inputs.json（host の側の字）。どちらも読むだけ。
//! 器の §20 の形の席の行（reopens= と move= と grace_left= を末に足した行）は歯の中で組む。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::{Reopens, SeatCard, SeatState};
use tsuzuri_core::account::host::HostTexts;
use tsuzuri_core::account::project::{ProjectTexts, project_rows};
use tsuzuri_core::seat::{SeatTexts, anchor, card};

const SEAT_FIXTURE: &str = "tests/fixtures/seat/seat-inputs.json";
const ACCT_FIXTURE: &str = "tests/fixtures/account/acct-inputs.json";

/// 2026-09-27T13:00:00Z と 2026-09-28T00:00:00Z。
const T13: u64 = 1_790_514_000;
const T24: u64 = 1_790_553_600;

/// 今（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

#[derive(Deserialize)]
struct Case {
    #[serde(flatten)]
    texts: SeatTexts,
    card: SeatCard,
}

#[derive(Deserialize)]
struct Inputs {
    texts: HostTexts,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn cases() -> BTreeMap<String, Case> {
    serde_json::from_str(&read(SEAT_FIXTURE)).expect("fixture の形")
}

fn case(name: &str) -> Case {
    cases()
        .remove(name)
        .unwrap_or_else(|| panic!("組 {name} が無い"))
}

/// server と同じ組み方（anchor は doctor の席の行から・今は期待の card の at）。
fn built(c: &Case) -> SeatCard {
    let target = &c.card.target;
    let anchor = c.texts.doctor.as_deref().and_then(|d| anchor(d, target));
    card(target, anchor.as_deref(), &c.texts, c.card.at)
}

/// 合図の健康の出力のうち、席 `target` の行の末に空白と `tail` を足した字。
fn with_tail(tick: &str, target: &str, tail: &str) -> String {
    let key = format!("target={target} ");
    tick.lines()
        .map(|l| {
            if l.contains(&key) {
                format!("{l} {tail}\n")
            } else {
                format!("{l}\n")
            }
        })
        .collect()
}

/// §20 の 3 つの欄（reopens の値と move=- と grace_left=-）。
fn tail(value: &str) -> String {
    format!("reopens={value} move=- grace_left=-")
}

/// 組 `name` の席 proj-1:0.1 の行の末に `tail` を足して組んだ card と、期待の card
/// （器の行の move=- grace_left=- の写しは Known(None)）。
fn built_with(name: &str, tail: &str) -> (SeatCard, SeatCard) {
    let mut c = case(name);
    let tick = c.texts.tick_status.as_deref().expect("合図の健康の出力");
    c.texts.tick_status = Some(with_tail(tick, "proj-1:0.1", tail));
    let mut expect = c.card.clone();
    if tail.contains("move=- grace_left=-") {
        expect.move_to = Reading::Known(None);
        expect.grace_until = Reading::Known(None);
    }
    (built(&c), expect)
}

/// (3) 4 つの形の写しと、ほかの欄が変わらないこと。
#[test]
fn lresume_copies_four_forms() {
    let forms = [
        ("2026-09-27T13:00:00Z", Reopens::At(T13)),
        ("2026-09-28T00:00:00Z", Reopens::At(T24)),
        ("-", Reopens::Clear),
        ("unmeasured", Reopens::Unmeasured),
        ("unknown", Reopens::Unknown),
    ];
    for name in ["run", "wait", "limit", "no-state"] {
        for (value, want) in forms {
            let (got, mut expect) = built_with(name, &tail(value));
            assert_eq!(got.reopens, want, "組 {name} の {value}");
            assert_eq!(
                (&got.move_to, &got.grace_until),
                (&Reading::Known(None), &Reading::Known(None)),
                "組 {name} の {value} の move=- grace_left=-"
            );
            expect.reopens = want;
            assert_eq!(got, expect, "組 {name} の {value} のほかの欄");
        }
    }
    let (got, _) = built_with(
        "limit",
        "reopens=2026-09-27T13:00:00Z move=acct-5 grace_left=120",
    );
    assert_eq!(got.reopens, Reopens::At(T13));
    assert_eq!(got.state, SeatState::Limit);
}

/// (4) 読めない字と、欄や出力の無いものは測れていない。
#[test]
fn lresume_unreadable_is_unmeasured() {
    for (name, c) in cases() {
        let got = built(&c);
        assert_eq!(got.reopens, Reopens::Unmeasured, "組 {name}");
        assert_eq!(c.card.reopens, Reopens::Unmeasured, "組 {name} の期待");
    }
    let limit = case("limit");
    assert!(
        limit
            .texts
            .usage
            .as_deref()
            .expect("残量の出力")
            .contains("five_hour=100% resets=2026-09-27T13:00:00Z")
    );
    for value in [
        "",
        "2026-09-27",
        "2026-09-27T13:00:00",
        "2026-09-27T13:00:00+00:00",
        "2026-09-27T13:00:00.5Z",
        "2026-13-01T00:00:00Z",
        "1790514000",
        "UNKNOWN",
        "none",
        "--",
    ] {
        let (got, mut expect) = built_with("limit", &tail(value));
        assert_eq!(got.reopens, Reopens::Unmeasured, "値 {value:?}");
        expect.reopens = Reopens::Unmeasured;
        assert_eq!(got, expect, "値 {value:?}");
    }
    let (got, _) = built_with("limit", "move=- grace_left=-");
    assert_eq!(got.reopens, Reopens::Unmeasured, "欄 reopens の無い行");
    let mut none = case("limit");
    none.texts.tick_status = None;
    assert_eq!(built(&none).reopens, Reopens::Unmeasured, "出力が無い");
}

/// (5) 自分の席の行だけを読む。
#[test]
fn lresume_own_line_only() {
    for name in ["silent", "run"] {
        let mut c = case(name);
        let tick = c.texts.tick_status.as_deref().expect("合図の健康の出力");
        c.texts.tick_status = Some(with_tail(tick, "proj-2:0.1", &tail("2026-09-27T13:00:00Z")));
        assert_eq!(built(&c).reopens, Reopens::Unmeasured, "組 {name}");
    }
    let mut run = case("run");
    let tick = run.texts.tick_status.as_deref().expect("合図の健康の出力");
    let tick = with_tail(tick, "proj-1:0.1", &tail("-"));
    run.texts.tick_status = Some(with_tail(
        &tick,
        "proj-2:0.1",
        &tail("2026-09-27T13:00:00Z"),
    ));
    assert_eq!(built(&run).reopens, Reopens::Clear);
}

/// (6) account board の project の席の card も同じ欄を写す。
#[test]
fn lresume_board_rows_carry() {
    let host: HostTexts = serde_json::from_str::<Inputs>(&read(ACCT_FIXTURE))
        .expect("fixture の形")
        .texts;
    let line = "seat tick status: target=proj-a:0.1 last=1790510390 age=10 healthy=yes heartbeat=on step=5 next=1790510395";
    let reopens_of = |tick: String| {
        let p = ProjectTexts {
            state_dir_known: true,
            tick_status: Some(tick),
            state_log: Some("{\"state\":\"busy\",\"ts\":1790505060}\n".into()),
            ..ProjectTexts::default()
        };
        let projects = BTreeMap::from([("/work/proj-a".to_string(), p)]);
        project_rows(&host, &projects, NOW)
            .into_iter()
            .find_map(|r| match r.seat {
                Reading::Known(c) if c.target == "proj-a:0.1" => Some(c.reopens),
                _ => None,
            })
            .expect("席 proj-a:0.1 の card")
    };
    for (value, want) in [
        ("2026-09-27T13:00:00Z", Reopens::At(T13)),
        ("-", Reopens::Clear),
        ("unknown", Reopens::Unknown),
        ("unmeasured", Reopens::Unmeasured),
    ] {
        assert_eq!(
            reopens_of(format!("{line} {}\n", tail(value))),
            want,
            "値 {value}"
        );
    }
    assert_eq!(reopens_of(format!("{line}\n")), Reopens::Unmeasured);
}
