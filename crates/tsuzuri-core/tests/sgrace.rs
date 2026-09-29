//! 行 c-seat-grace の歯（中核）: 席の card の欄 move_to と grace_until は合図の健康の出力の席の行の欄
//! `move=` と `grace_left=` の写し（grace_until は器の残り秒に組んだ今を足した終わる時刻・行 c-abs-seat）、
//! 欄 refused と pressure は doctor の同じ名の群の行の欄 `refused=` と
//! `pressure=` の写し（器の `-` は Known(None)・読めない字と欄の無い行は Unknown・残りを計算しない）。
//! fixture: tests/fixtures/seat/seat-inputs.json（読むだけ）。器の §20 の形の欄は歯の中で組む。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::{Pressure, Reopens, SeatCard};
use tsuzuri_core::seat::{SeatTexts, anchor, card};

const FIXTURE: &str = "tests/fixtures/seat/seat-inputs.json";

/// 器の行を足して読む 4 組。
const SETS: [&str; 4] = ["run", "wait", "limit", "no-state"];

/// 席の名。
const OWN: &str = "proj-1:0.1";

/// 2026-09-27T13:00:00Z。
const T13: u64 = 1_790_514_000;

#[derive(Deserialize)]
struct Case {
    #[serde(flatten)]
    texts: SeatTexts,
    card: SeatCard,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn case(name: &str) -> Case {
    serde_json::from_str::<BTreeMap<String, Case>>(&read(FIXTURE))
        .expect("fixture の形")
        .remove(name)
        .unwrap_or_else(|| panic!("組 {name} が無い"))
}

/// server と同じ組み方（anchor は doctor の席の行から）。
fn build(target: &str, texts: &SeatTexts, now: u64) -> SeatCard {
    let anchor = texts.doctor.as_deref().and_then(|d| anchor(d, target));
    card(target, anchor.as_deref(), texts, now)
}

/// 合図の健康の出力のうち、席 `target` の行の末に空白と `tail` を足した字（ほかの行はそのまま）。
fn tick_tail(tick: &str, target: &str, tail: &str) -> String {
    let key = format!("target={target} ");
    tick.lines()
        .map(|l| {
            if l.contains(&key) && !tail.is_empty() {
                format!("{l} {tail}\n")
            } else {
                format!("{l}\n")
            }
        })
        .collect()
}

/// doctor の出力のうち、群 `group` の行の ` refused=-` を空白と `tail` に替えた字（群の行が無ければ足す）。
fn group_tail(doctor: &str, group: &str, tail: &str) -> String {
    let head = format!("group={group} ");
    let mut found = false;
    let mut out: String = doctor
        .lines()
        .map(|l| {
            if l.starts_with(&head) {
                found = true;
                let base = l.replace(" refused=-", "");
                format!("{base} {tail}\n")
            } else {
                format!("{l}\n")
            }
        })
        .collect();
    if !found {
        out.push_str(&format!(
            "{head}accounts=acct-3 anchors=1 seat-accounts=acct-3 current=acct-3 next=none {tail}\n"
        ));
    }
    out
}

/// 器の欄の写し（move_to・grace_left の残り秒・refused・pressure の型）。
type Move = Reading<Option<String>>;
type Left = Reading<Option<u64>>;
type Refused = Reading<Option<u64>>;
type Pressed = Reading<Option<Pressure>>;

/// 器の残り秒に今を足した終わる時刻（`-` と読めない字はそのまま）。
fn until(left: &Left, now: u64) -> Left {
    match left {
        Reading::Known(Some(s)) => Reading::Known(Some(now + s)),
        other => other.clone(),
    }
}

fn some(name: &str) -> Move {
    Reading::Known(Some(name.to_string()))
}

fn pr(window: &str, used: u32, cap: u32) -> Pressed {
    Reading::Known(Some(Pressure {
        window: window.to_string(),
        used,
        cap,
    }))
}

#[test]
fn sgrace_copies_move_and_left() {
    let forms: [(&str, Move, Left); 9] = [
        (
            "move=acct-5 grace_left=120",
            some("acct-5"),
            Reading::Known(Some(120)),
        ),
        (
            "move=acct-5 grace_left=0",
            some("acct-5"),
            Reading::Known(Some(0)),
        ),
        (
            "move=acct-5 grace_left=-",
            some("acct-5"),
            Reading::Known(None),
        ),
        (
            "move=- grace_left=-",
            Reading::Known(None),
            Reading::Known(None),
        ),
        (
            "move=unreadable grace_left=unreadable",
            Reading::Unknown,
            Reading::Unknown,
        ),
        ("move=acct-5 grace_left=1.5", some("acct-5"), Reading::Unknown),
        ("move=acct-5 grace_left=-5", some("acct-5"), Reading::Unknown),
        ("move= grace_left=", Reading::Unknown, Reading::Unknown),
        ("", Reading::Unknown, Reading::Unknown),
    ];
    for name in SETS {
        for (pair, move_to, grace_left) in &forms {
            let mut c = case(name);
            let tick = c.texts.tick_status.clone().expect("合図の健康の出力");
            let tail = format!("reopens=- {pair}");
            c.texts.tick_status = Some(tick_tail(&tick, OWN, tail.trim_end()));
            let got = build(OWN, &c.texts, c.card.at);
            let want_until = until(grace_left, c.card.at);
            assert_eq!(
                (&got.move_to, &got.grace_until),
                (move_to, &want_until),
                "組 {name} の {pair:?}"
            );
            let mut want = c.card.clone();
            want.reopens = Reopens::Clear;
            want.move_to = move_to.clone();
            want.grace_until = want_until;
            assert_eq!(got, want, "組 {name} の {pair:?} のほかの欄");
        }
    }
}

#[test]
fn sgrace_left_is_not_computed() {
    // ほかの席の行の move= と grace_left= は写さない。
    let mut run = case("run");
    let tick = run.texts.tick_status.clone().expect("合図の健康の出力");
    assert!(tick.contains("target=proj-2:0.1 "));
    run.texts.tick_status = Some(tick_tail(
        &tick,
        "proj-2:0.1",
        "reopens=- move=acct-5 grace_left=900",
    ));
    let got = build(OWN, &run.texts, run.card.at);
    assert_eq!(
        (&got.move_to, &got.grace_until),
        (&Reading::Unknown, &Reading::Unknown)
    );
    let both = tick_tail(
        &tick_tail(&tick, OWN, "reopens=- move=acct-2 grace_left=1700"),
        "proj-2:0.1",
        "reopens=- move=acct-5 grace_left=900",
    );
    run.texts.tick_status = Some(both);
    let got = build(OWN, &run.texts, run.card.at);
    assert_eq!(
        (&got.move_to, &got.grace_until),
        (&some("acct-2"), &Reading::Known(Some(run.card.at + 1700)))
    );

    // 群の記録の字が在っても無くても、終わる時刻は今に器の字の 1700 を足した値（群の記録から計算しない）。
    for name in SETS {
        let c = case(name);
        let tick = c.texts.tick_status.clone().expect("合図の健康の出力");
        let with = tick_tail(&tick, OWN, "reopens=- move=acct-2 grace_left=1700");
        let records = [
            Vec::new(),
            c.texts.records.clone(),
            vec![
                "account=acct-2\nts=2026-09-27T11:59:00Z\nreason=account-pressed\nprevious=acct-1\n"
                    .to_string(),
            ],
            vec!["account=acct-2\nts=1790510000\nreason=manual\n".to_string()],
        ];
        for recs in records {
            for now in [c.card.at, c.card.at + 60, c.card.at + 3_600] {
                let mut texts = c.texts.clone();
                texts.tick_status = Some(with.clone());
                texts.records = recs.clone();
                let got = build(OWN, &texts, now);
                assert_eq!(
                    got.grace_until,
                    Reading::Known(Some(now + 1700)),
                    "組 {name} の記録 {recs:?} と今 {now}"
                );
                assert_eq!(got.move_to, some("acct-2"), "組 {name}");
            }
        }
    }

    // 中核の src/seat.rs は猶予の rules 行と合図の file と猶予の秒の読みを持たない。
    let src = read("crates/tsuzuri-core/src/seat.rs");
    for word in ["move_grace_s", "move-signal", "grace_secs"] {
        assert!(!src.contains(word), "src/seat.rs に {word} が在る");
    }
}

#[test]
fn sgrace_copies_refused_and_pressure() {
    let forms: [(&str, Refused, Pressed); 10] = [
        (
            "refused=2026-09-27T13:00:00Z pressure=5h:92/85",
            Reading::Known(Some(T13)),
            pr("5h", 92, 85),
        ),
        (
            "refused=- pressure=7d:81/80",
            Reading::Known(None),
            pr("7d", 81, 80),
        ),
        (
            "refused=unreadable pressure=model:100/95",
            Reading::Unknown,
            pr("model", 100, 95),
        ),
        (
            "refused=1790514000 pressure=-",
            Reading::Unknown,
            Reading::Known(None),
        ),
        (
            "refused=2026-09-27 pressure=unmeasured",
            Reading::Unknown,
            Reading::Unknown,
        ),
        ("pressure=unreadable", Reading::Unknown, Reading::Unknown),
        (
            "refused=- pressure=no-rule",
            Reading::Known(None),
            Reading::Unknown,
        ),
        (
            "refused=- pressure=5h:92",
            Reading::Known(None),
            Reading::Unknown,
        ),
        (
            "refused=- pressure=5h:9.5/85",
            Reading::Known(None),
            Reading::Unknown,
        ),
        (
            "refused=- pressure=:92/85",
            Reading::Known(None),
            Reading::Unknown,
        ),
    ];
    for name in SETS {
        let c = case(name);
        assert_eq!(c.card.refused, Reading::Known(None), "組 {name} の期待");
        assert_eq!(c.card.pressure, Reading::Unknown, "組 {name} の期待");
        let doctor = c.texts.doctor.clone().expect("doctor");
        for (tail, refused, pressure) in &forms {
            let mut texts = c.texts.clone();
            texts.doctor = Some(group_tail(&doctor, "g-a", tail));
            let got = build(OWN, &texts, c.card.at);
            assert_eq!(
                (&got.refused, &got.pressure),
                (refused, pressure),
                "組 {name} の {tail:?}"
            );
            let mut want = c.card.clone();
            want.refused = refused.clone();
            want.pressure = pressure.clone();
            assert_eq!(got, want, "組 {name} の {tail:?} のほかの欄");
        }
        // 群 g-b の行を替えても card は変わらない。
        let mut texts = c.texts.clone();
        texts.doctor = Some(group_tail(&doctor, "g-b", forms[0].0));
        assert_eq!(build(OWN, &texts, c.card.at), c.card, "組 {name} の g-b");
    }
    // 群の名が得られない組と doctor の無い組は、読める字でも両方 Unknown。
    for name in ["silent", "unread"] {
        let c = case(name);
        let mut texts = c.texts.clone();
        texts.doctor = texts
            .doctor
            .as_deref()
            .map(|d| group_tail(d, "g-a", forms[0].0));
        let got = build(&c.card.target, &texts, c.card.at);
        assert_eq!(
            (&got.refused, &got.pressure),
            (&Reading::Unknown, &Reading::Unknown),
            "組 {name}"
        );
        assert_eq!(got, c.card, "組 {name}");
    }
}
