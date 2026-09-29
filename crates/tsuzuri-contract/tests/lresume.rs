//! 行 c-limit-resume の歯（契約）: 限度の再開の時刻の閉じた 4 つの値と、席の card の欄 reopens。

use serde_json::{Value, json};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::{Reopens, SeatCard, SeatState};
use tsuzuri_contract::wire;

/// (1) 4 つの値の電文の字・往復・既定の値・読めない電文。
#[test]
fn lresume_wire_four_values() {
    let cases = [
        (Reopens::At(1_790_514_000), r#"{"at":1790514000}"#),
        (Reopens::Clear, r#""clear""#),
        (Reopens::Unmeasured, r#""unmeasured""#),
        (Reopens::Unknown, r#""unknown""#),
    ];
    for (value, want) in cases {
        let text = wire::encode(&value).expect("電文");
        let got: Value = serde_json::from_str(&text).expect("JSON");
        let want_v: Value = serde_json::from_str(want).expect("JSON");
        assert_eq!(got, want_v, "{value:?} の電文");
        assert_eq!(wire::decode::<Reopens>(&text).expect("往復"), value);
    }
    assert_eq!(Reopens::default(), Reopens::Unmeasured);
    for bad in [
        r#""-""#,
        r#""at""#,
        r#""Clear""#,
        r#""limited""#,
        r#"{"at":"2026-09-27T13:00:00Z"}"#,
        r#"{"at":-1}"#,
    ] {
        assert!(wire::decode::<Reopens>(bad).is_err(), "{bad} が読める");
    }
}

fn sample(reopens: Reopens) -> SeatCard {
    SeatCard {
        at: 100,
        target: "t:1".into(),
        state: SeatState::Limit,
        since: Some(90),
        tick_healthy: Reading::Known(true),
        heartbeat: Reading::Known(true),
        tick: Reading::Unknown,
        tick_at: Some(40),
        reopens,
        account: None,
        model: None,
        group: Reading::Unknown,
        usage: Reading::Unknown,
        spans: Reading::Unknown,
        moves: Reading::Unknown,
        move_to: Reading::Unknown,
        grace_left: Reading::Unknown,
        refused: Reading::Unknown,
        pressure: Reading::Unknown,
    }
}

/// (2) 欄 reopens の電文の鍵と値と順・往復・鍵が無い電文の既定の値。
#[test]
fn lresume_card_key_default() {
    let card = sample(Reopens::At(1_790_514_000));
    let text = wire::encode(&card).expect("電文");
    let mut v: Value = serde_json::from_str(&text).expect("JSON");
    assert_eq!(v["reopens"], json!({"at": 1_790_514_000}));
    let at = |k: &str| {
        text.find(&format!("\"{k}\":"))
            .unwrap_or_else(|| panic!("鍵 {k} が無い"))
    };
    assert!(at("tick_at") < at("reopens"));
    assert!(at("reopens") < at("account"));
    assert_eq!(wire::decode::<SeatCard>(&text).expect("往復"), card);

    for r in [Reopens::Clear, Reopens::Unmeasured, Reopens::Unknown] {
        let c = sample(r);
        let t = wire::encode(&c).expect("電文");
        assert_eq!(wire::decode::<SeatCard>(&t).expect("往復"), c);
    }

    v.as_object_mut().expect("object").remove("reopens");
    let bare = serde_json::to_string(&v).expect("字");
    let got: SeatCard = wire::decode(&bare).expect("鍵の無い電文が読める");
    assert_eq!(got, sample(Reopens::Unmeasured));
}
