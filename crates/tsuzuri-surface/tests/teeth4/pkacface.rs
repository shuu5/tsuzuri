//! 区画の席の状態の帯の歯（面・行 c-park-acct・接頭辞 pkac_）: 群の行が区画の行の写し（欄 park が true・
//! 今の口座 -）の席の card は、移動待ちの行と次の移り先の行を出さず、限度・猶予の内・逼迫の行は今のまま。
//! fixture: tests/fixtures/surface/seat-card.json（読むだけ）。群の行は歯の中で区画の行の写しに替える。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::{GroupRow, Reading};
use tsuzuri_contract::seat::{Pressure, SeatCard};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::seat::{
    self, Band, GRACE_NEXT, LIMIT_LINE, MOVE_WAIT, MOVING, NEXT_TARGET, PRESSED,
};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// fixture の組の card を、群の行だけ区画の行の写し（park は引数）に替えた card。
fn card(name: &str, park: bool) -> SeatCard {
    let mut c = wire::decode::<BTreeMap<String, SeatCard>>(&read(
        "../../tests/fixtures/surface/seat-card.json",
    ))
    .expect("fixture の組が電文として読める")
    .remove(name)
    .unwrap_or_else(|| panic!("fixture の組 {name}"));
    c.group = Reading::Known(GroupRow {
        group: "Tier9".into(),
        account: "-".into(),
        candidates: vec!["acct-1".into(), "acct-2".into(), "acct-3".into()],
        next_account: Some("-".into()),
        remaining: Vec::new(),
        park,
    });
    c
}

#[test]
fn pkac_band_park_quiet() {
    // 区画の行の写しの組 wait は帯を出さない。
    assert_eq!(seat::band(&card("wait", true)), None);
    // park を false にすると移動待ちの行と次の移り先の行が出る（歯が見分けを測っている）。
    assert_eq!(
        seat::band(&card("wait", false)),
        Some(Band {
            l1: vec![format!("{MOVE_WAIT} acct-4 → -")],
            l2: Some(format!("{NEXT_TARGET} -")),
        })
    );
    // 逼迫の行は今のまま（今の口座は区画の行の字 -・2 行目は無し）。
    let mut pressed = card("wait", true);
    pressed.pressure = Reading::Known(Some(Pressure {
        window: "5h".into(),
        used: 91,
        cap: 90,
    }));
    assert_eq!(
        seat::band(&pressed),
        Some(Band {
            l1: vec![format!("{PRESSED} - 5h 91% ≥ 90")],
            l2: None,
        })
    );
    // 猶予の内の行と、移る先の口座の 2 行目は今のまま。
    let mut moving = card("wait", true);
    moving.move_to = Reading::Known(Some("acct-5".into()));
    moving.grace_until = Reading::Known(Some(moving.at + 30));
    assert_eq!(
        seat::band(&moving),
        Some(Band {
            l1: vec![format!("{MOVING} 30 秒")],
            l2: Some(format!("{GRACE_NEXT} acct-5")),
        })
    );
    // 限度の行は今のまま。
    assert_eq!(
        seat::band(&card("limit", true)),
        Some(Band {
            l1: vec![LIMIT_LINE.to_string()],
            l2: None,
        })
    );
}
