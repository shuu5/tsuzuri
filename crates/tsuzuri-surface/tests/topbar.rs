//! 便 g-top-fix の歯: header の最終の記録の短い時刻（見本の hmd と同じ決め方・字は日本時間・行 g-jst）。
//! header の DOM は wasm の target のときだけ組むので、ここでは字を決める純粋な関数だけを pin する。
#![cfg(test)]

use tsuzuri_surface::frame::HEADER;
use tsuzuri_surface::view::clock_short;

#[test]
fn topbar_within_20h_is_hour_and_minute() {
    assert_eq!(clock_short(1_790_513_580, 1_790_513_600), "21:53 JST");
}

#[test]
fn topbar_beyond_20h_adds_month_and_day() {
    assert_eq!(clock_short(1_790_427_180, 1_790_513_600), "09-26 21:53 JST");
}

#[test]
fn topbar_exactly_20h_is_hour_and_minute() {
    assert_eq!(clock_short(1_790_513_580, 1_790_513_580 + 72_000), "21:53 JST");
    assert_eq!(
        clock_short(1_790_513_580, 1_790_513_580 + 72_001),
        "09-27 21:53 JST"
    );
}

#[test]
fn topbar_direction_does_not_matter() {
    assert_eq!(clock_short(1_790_513_600, 1_790_513_580), "21:53 JST");
    assert_eq!(clock_short(1_790_513_580, 1_790_513_580 - 72_000), "21:53 JST");
    assert_eq!(
        clock_short(1_790_513_580, 1_790_513_580 - 72_001),
        "09-27 21:53 JST"
    );
}

#[test]
fn topbar_updated_part_is_a_num_chip_keyed_last_record() {
    let part = HEADER.iter().find(|p| p.part == "updated").unwrap();
    assert_eq!(part.key, "last_record");
    assert_eq!(part.class, "chip num");
}
