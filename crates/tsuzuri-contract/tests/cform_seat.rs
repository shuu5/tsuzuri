//! 契約の型の外形の歯（接頭辞 contract_form_）のうち seat の群の型の見本。
//! snapshot は tests/snapshots の群の名の json（seat.json）で、字の比べは標準 library だけで行う。
//! 共通の手は tests/common/mod.rs に 1 つだけ在る。
//! 字が違えば今の字を CARGO_TARGET_TMPDIR の群の名の json に書いて落ちる（見て正しければ snapshot へ写す）。
#![cfg(test)]

mod common;

use common::{AT, Form, distinct, form};
use tsuzuri_contract::board::{GroupRow, QuotaLeft, Reading};
use tsuzuri_contract::seat::{
    AccountMove, Pressure, QuotaUsed, Reopens, SeatCard, SeatSpan, SeatState, TickHealth,
};
use tsuzuri_contract::wire;

fn group_row() -> GroupRow {
    GroupRow {
        group: "Tier1".into(),
        account: "acct-4".into(),
        candidates: vec!["acct-1".into(), "acct-3".into(), "acct-5".into()],
        next_account: Some("acct-5".into()),
        remaining: vec![QuotaLeft {
            window: "5h".into(),
            left_pct: 62,
        }],
        park: false,
    }
}

fn quota_used() -> Vec<QuotaUsed> {
    vec![
        QuotaUsed {
            window: "5h".into(),
            used_pct: 38,
            resets_at: Some(AT + 3_600),
            counted: true,
        },
        QuotaUsed {
            window: "7d-opus".into(),
            used_pct: 90,
            resets_at: None,
            counted: false,
        },
    ]
}

fn seat_spans() -> Vec<SeatSpan> {
    vec![
        SeatSpan {
            from: AT - 7_200,
            to: AT - 3_600,
            state: SeatState::Run,
        },
        SeatSpan {
            from: AT - 3_600,
            to: AT,
            state: SeatState::Wait,
        },
    ]
}

fn account_moves() -> Vec<AccountMove> {
    vec![
        AccountMove {
            at: AT - 86_400,
            from: None,
            to: "acct-3".into(),
        },
        AccountMove {
            at: AT - 3_600,
            from: Some("acct-3".into()),
            to: "acct-4".into(),
        },
    ]
}

fn pressure() -> Pressure {
    Pressure {
        window: "5h".into(),
        used: 92,
        cap: 85,
    }
}

/// seat の群の型の見本（この順が snapshot の file の key の順）。
fn forms() -> Vec<Box<dyn Form>> {
    vec![
        // seat
        form("seat::SeatState", SeatState::ALL.to_vec()),
        form("seat::QuotaUsed", quota_used()),
        form("seat::SeatSpan", seat_spans()),
        form("seat::AccountMove", account_moves()),
        form(
            "seat::Reopens",
            vec![
                Reopens::At(AT + 3_600),
                Reopens::Clear,
                Reopens::Unmeasured,
                Reopens::Unknown,
            ],
        ),
        form("seat::Pressure", vec![pressure()]),
    ]
    .into_iter()
    .chain(card_forms())
    .collect()
}

/// seat の群の型の見本の後半（席の card の型）。
fn card_forms() -> Vec<Box<dyn Form>> {
    vec![
        form(
            "seat::SeatCard",
            vec![
                seat_card(),
                // 器が移さない・断りも逼迫も無い席。
                SeatCard {
                    at: AT,
                    target: "t3:orchestrator".into(),
                    state: SeatState::Run,
                    since: Some(AT - 60),
                    tick_healthy: Reading::Known(true),
                    heartbeat: Reading::Known(true),
                    tick: Reading::Known(TickHealth::Healthy),
                    tick_at: Some(AT - 20),
                    reopens: Reopens::Clear,
                    account: Some("acct-4".into()),
                    model: Some("opus".into()),
                    group: Reading::Known(group_row()),
                    usage: Reading::Known(quota_used()),
                    spans: Reading::Known(seat_spans()),
                    moves: Reading::Known(account_moves()),
                    move_to: Reading::Known(None),
                    grace_until: Reading::Known(None),
                    refused: Reading::Known(None),
                    pressure: Reading::Known(None),
                },
                // 器も口座も読めない席。
                SeatCard {
                    at: AT,
                    target: "t3:orchestrator".into(),
                    state: SeatState::Unknown,
                    since: None,
                    tick_healthy: Reading::Unknown,
                    heartbeat: Reading::Unknown,
                    tick: Reading::Unknown,
                    tick_at: None,
                    reopens: Reopens::Unmeasured,
                    account: None,
                    model: None,
                    group: Reading::Unknown,
                    usage: Reading::Unknown,
                    spans: Reading::Unknown,
                    moves: Reading::Unknown,
                    move_to: Reading::Unknown,
                    grace_until: Reading::Unknown,
                    refused: Reading::Unknown,
                    pressure: Reading::Unknown,
                },
            ],
        ),
    ]
}

/// 器が移す・断りも逼迫も在る席の card の見本。
fn seat_card() -> SeatCard {
    SeatCard {
        at: AT,
        target: "t3:orchestrator".into(),
        state: SeatState::Wait,
        since: Some(AT - 3_600),
        tick_healthy: Reading::Known(true),
        heartbeat: Reading::Known(false),
        tick: Reading::Known(TickHealth::Healthy),
        tick_at: Some(AT - 20),
        reopens: Reopens::At(AT + 3_600),
        account: Some("acct-4".into()),
        model: Some("opus".into()),
        group: Reading::Known(group_row()),
        usage: Reading::Known(quota_used()),
        spans: Reading::Known(seat_spans()),
        moves: Reading::Known(account_moves()),
        move_to: Reading::Known(Some("acct-5".into())),
        grace_until: Reading::Known(Some(AT + 120)),
        refused: Reading::Known(Some(AT - 600)),
        pressure: Reading::Known(Some(pressure())),
    }
}

#[test]
fn contract_form_snapshot_matches() {
    common::snapshot_matches("seat", &forms());
}

#[test]
fn contract_form_roundtrip_all_types() {
    common::roundtrip_all(&forms());
}

#[test]
fn contract_form_closed_lists() {
    assert_eq!(distinct(&SeatState::ALL), 5);
    let seat_words: Vec<String> = SeatState::ALL
        .iter()
        .map(|s| wire::encode(s).expect("語"))
        .collect();
    assert_eq!(
        seat_words,
        [
            "\"run\"",
            "\"wait\"",
            "\"limit\"",
            "\"silent\"",
            "\"unknown\""
        ],
        "席の状態の ALL の順と字"
    );
    assert!(wire::decode::<SeatState>("\"stopped\"").is_err());
}
