//! 席に届いていない裁定の 1 行の歯（接頭辞 lateface_・設計ノート surface-wave23b 行 f-undelivered・要件 FR9）。
//! 純粋な関数は host で撃ち、DOM は wasm の target でしか組めないので src の字の mod dom 以降を読む。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::surface::RulingId;
use tsuzuri_contract::wire;
use tsuzuri_surface::project::ask::{
    PATHS, RECEIPT_WAIT_S, UNRECEIVED_HEAD, UNRECEIVED_PATH, UNRECEIVED_UNKNOWN, late, minute_end,
    unreceived, unreceived_line,
};
use tsuzuri_surface::view::Fetched;

/// fx-u.2 の裁定の分（2026-09-29T10:01Z）の分の終わりの epoch 秒。
const END_1001: u64 = 1_790_676_120;

fn ruling(id: &str) -> RulingId {
    RulingId::new(id).expect("裁定の id")
}

fn rulings(ids: &[&str]) -> Vec<RulingId> {
    ids.iter().map(|id| ruling(id)).collect()
}

/// 口の本文（Known の id の列）。
fn known(ids: &[&str]) -> Fetched {
    Fetched::Body(wire::encode(&Reading::Known(rulings(ids))).expect("電文"))
}

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// 字の mod dom 以降。
fn dom(rel: &str) -> String {
    let text = read(rel);
    let (_, dom) = text.split_once("mod dom {").unwrap_or_else(|| panic!("{rel} に mod dom"));
    dom.to_string()
}

/// (3) minute_end は分の始まりに 60 秒を足した epoch 秒で、読めない id と 1970 年より前は None。
#[test]
fn lateface_minute_end() {
    assert_eq!(minute_end("fx-u.2:20260929T1001Z-1"), Some(END_1001));
    assert_eq!(minute_end("batch:20260929T1001Z-12"), Some(END_1001));
    assert_eq!(minute_end("fx-u.3:20260929T1003Z-2"), Some(END_1001 + 120));
    assert_eq!(minute_end("x:20240229T0000Z-1"), Some(1_709_164_860));
    assert_eq!(minute_end("x:19700101T0000Z-1"), Some(60));
    for bad in [
        "fx-u.2",
        "x:20261329T1001Z-1",
        "x:20260929T2401Z-1",
        "x:20260929T1060Z-1",
        "x:20260929T1001Z",
        "x:20260929T1001Z-",
        "x:20260929T1001Z1",
        "x:2026a929T1001Z-1",
        "x:20260929T1001X-1",
        "x:20260929T1001-1",
        "x:19691231T2359Z-1",
        "x:20230229T0000Z-1",
        "x:20260929T1001Z-a",
    ] {
        assert_eq!(minute_end(bad), None, "{bad}");
    }
}

/// (4) RECEIPT_WAIT_S は 120 で、late は分の終わりから待つ秒が経った id と分の読めない id を元の順に返す。
#[test]
fn lateface_late_after_wait() {
    assert_eq!(RECEIPT_WAIT_S, 120);
    let ids = rulings(&["fx-u.2:20260929T1001Z-1", "plain", "fx-u.3:20260929T1003Z-1"]);
    assert_eq!(late(&ids, END_1001 + 119), rulings(&["plain"]));
    assert_eq!(
        late(&ids, END_1001 + 120),
        rulings(&["fx-u.2:20260929T1001Z-1", "plain"])
    );
    assert_eq!(late(&ids, END_1001 + 240), ids);
    assert_eq!(late(&[], END_1001 + 240), Vec::<RulingId>::new());
}

/// (5) 口の path と字・unreceived は Known の電文を読み・unreceived_line は 1 行か None。
#[test]
fn lateface_line() {
    assert_eq!(UNRECEIVED_PATH, "/api/unreceived");
    assert_eq!(UNRECEIVED_HEAD, "席に届いていない");
    assert_eq!(PATHS.last(), Some(&UNRECEIVED_PATH));
    let now = END_1001 + RECEIPT_WAIT_S;
    assert_eq!(unreceived(&Fetched::NotRead), None);
    assert_eq!(unreceived(&Fetched::Failed), Some(Reading::Unknown));
    assert_eq!(
        unreceived(&known(&["a:20260929T1001Z-1"])),
        Some(Reading::Known(rulings(&["a:20260929T1001Z-1"])))
    );
    assert_eq!(unreceived_line(&Fetched::NotRead, now), None);
    let unknown = Some(UNRECEIVED_UNKNOWN.to_string());
    assert_eq!(unreceived_line(&Fetched::Failed, now), unknown);
    assert_eq!(unreceived_line(&Fetched::Body("[]".to_string()), now), unknown);
    let unread = Fetched::Body(wire::encode(&Reading::<Vec<RulingId>>::Unknown).expect("電文"));
    assert_eq!(unreceived_line(&unread, now), unknown);
    assert_eq!(unreceived_line(&known(&[]), now), None);
    one_and_two_lines(now);
}

/// 1 件と 2 件の 1 行は待ちの秒を過ぎた物だけを数える。
fn one_and_two_lines(now: u64) {
    let one = known(&["fx-u.2:20260929T1001Z-1"]);
    assert_eq!(unreceived_line(&one, now - 1), None);
    assert_eq!(
        unreceived_line(&one, now),
        Some(format!("{UNRECEIVED_HEAD} 1 件: fx-u.2:20260929T1001Z-1"))
    );
    let two = known(&["fx-u.2:20260929T1001Z-1", "fx-u.3:20260929T1003Z-1"]);
    assert_eq!(
        unreceived_line(&two, now + 120),
        Some(format!(
            "{UNRECEIVED_HEAD} 2 件: fx-u.2:20260929T1001Z-1・fx-u.3:20260929T1003Z-1"
        ))
    );
    assert_eq!(
        unreceived_line(&two, now),
        Some(format!("{UNRECEIVED_HEAD} 1 件: fx-u.2:20260929T1001Z-1"))
    );
}

/// (6) ask.rs と next.rs の mod dom は 1 行の口・書き直し・置き場を 1 度ずつ持つ。
#[test]
fn lateface_dom_wiring() {
    let ask = dom("src/project/ask.rs");
    for word in [
        "crate::net::read(UNRECEIVED_PATH)",
        "super::unreceived_line(f, clock.get())",
        "{late_view()}{list}{unknown}",
    ] {
        assert_eq!(ask.matches(word).count(), 1, "ask の mod dom の {word}");
    }
    let next = dom("src/project/next.rs");
    let word = "{ask::late_view()}{body}";
    assert_eq!(next.matches(word).count(), 1, "next の mod dom の {word}");
}
