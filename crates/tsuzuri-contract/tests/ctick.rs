//! 行 c-seat-tick の歯（契約）: 器の管理 tick の健康の閉じた 4 語と、席の card の欄 tick と tick_at。
//! 語の列は 4 語なので snapshot の file を足さず、歯の中の字で pin する。

use serde_json::Value;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::{Reopens, SeatCard, SeatState, TickHealth};
use tsuzuri_contract::wire;

/// (1) 閉じた 4 語の順と字と、4 語のほかの語は読めないこと。
#[test]
fn ctick_words_closed_four() {
    assert_eq!(
        TickHealth::ALL,
        [
            TickHealth::Healthy,
            TickHealth::Stale,
            TickHealth::Absent,
            TickHealth::Unreadable
        ]
    );
    let words: Vec<&str> = TickHealth::ALL.iter().map(|t| t.as_str()).collect();
    assert_eq!(words, ["healthy", "stale", "absent", "unreadable"]);
    let wire_words: Vec<String> = TickHealth::ALL
        .iter()
        .map(|t| wire::encode(t).expect("語"))
        .collect();
    assert_eq!(
        wire_words,
        [
            "\"healthy\"",
            "\"stale\"",
            "\"absent\"",
            "\"unreadable\""
        ]
    );
    for t in TickHealth::ALL {
        let back: TickHealth = wire::decode(&format!("\"{}\"", t.as_str())).expect("読める");
        assert_eq!(back, t);
    }
    for bad in ["past", "no-rule:missing", "Healthy", "unknown"] {
        assert!(
            wire::decode::<TickHealth>(&format!("\"{bad}\"")).is_err(),
            "{bad} が読める"
        );
    }
}

fn sample() -> SeatCard {
    SeatCard {
        at: 100,
        target: "t:1".into(),
        state: SeatState::Run,
        since: Some(90),
        tick_healthy: Reading::Known(true),
        heartbeat: Reading::Known(true),
        tick: Reading::Known(TickHealth::Stale),
        tick_at: Some(40),
        reopens: Reopens::Unmeasured,
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

/// (2) 欄 tick と tick_at の電文の鍵と値・往復・鍵が無い電文の既定の値。
#[test]
fn ctick_card_keys_default() {
    let card = sample();
    let text = wire::encode(&card).expect("電文");
    let mut v: Value = serde_json::from_str(&text).expect("JSON");
    assert_eq!(v["tick"], serde_json::json!({"known": "stale"}));
    assert_eq!(v["tick_at"], serde_json::json!(40));
    // 鍵の順は heartbeat・tick・tick_at・account。
    let at = |k: &str| {
        text.find(&format!("\"{k}\":"))
            .unwrap_or_else(|| panic!("鍵 {k} が無い"))
    };
    assert!(at("heartbeat") < at("tick"));
    assert!(at("tick") < at("tick_at"));
    assert!(at("tick_at") < at("account"));
    assert_eq!(wire::decode::<SeatCard>(&text).expect("往復"), card);

    let obj = v.as_object_mut().expect("object");
    obj.remove("tick");
    obj.remove("tick_at");
    let bare = serde_json::to_string(&v).expect("字");
    let got: SeatCard = wire::decode(&bare).expect("鍵の無い電文が読める");
    let mut want = card.clone();
    want.tick = Reading::Unknown;
    want.tick_at = None;
    assert_eq!(got, want);
}
