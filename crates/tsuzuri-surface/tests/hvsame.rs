//! 同じ点の pointermove の歯（接頭辞 hvsame_）。
//! 本物の Chrome の事件の順（出た点の pointerleave の後に同じ点の pointermove）でも猶予が切れないことを、面の純粋な関数で測る段である。
#![cfg(test)]

use std::fs;
use std::path::Path;

use tsuzuri_boundary::audit::{HoverCard, HoverTarget, exit_path};
use tsuzuri_surface::widgets::hover::{GRACE_MS, Point, Rect, keeps, move_hides};

/// 見本の card（左 116・上 180・幅 200・高さ 100）。
const CARD: Rect = Rect {
    left: 116.0,
    top: 180.0,
    width: 200.0,
    height: 100.0,
};

/// 出た点（card までの距離 16）。
const EXIT: Point = Point { x: 100.0, y: 200.0 };

fn pt(x: f64, y: f64) -> Point {
    Point { x, y }
}

fn elapsed() -> [f64; 4] {
    let grace = f64::from(GRACE_MS);
    [0.0, 10.0, grace, grace + 1.0]
}

#[test]
fn hvsame_same_point_move_keeps_card() {
    for ms in elapsed() {
        assert!(!move_hides(EXIT, EXIT, CARD, ms), "経過 {ms}");
    }
    assert!(!keeps(EXIT, EXIT, CARD, 10.0));
}

#[test]
fn hvsame_other_moves_follow_keeps() {
    let late = f64::from(GRACE_MS) + 1.0;
    for p in [pt(90.0, 200.0), pt(100.0, 300.0), pt(100.0, 201.0), pt(108.0, 200.0)] {
        assert!(move_hides(EXIT, p, CARD, late), "{p:?} 経過 {late}");
    }
    for p in [pt(108.0, 200.0), pt(120.0, 220.0), pt(101.0, 200.0)] {
        assert!(!move_hides(EXIT, p, CARD, 10.0), "{p:?} 経過 10");
    }
    let points = [
        pt(90.0, 200.0),
        pt(100.0, 300.0),
        pt(100.0, 201.0),
        pt(108.0, 200.0),
        pt(120.0, 220.0),
        pt(101.0, 200.0),
    ];
    for p in points {
        for ms in elapsed() {
            assert_eq!(
                move_hides(EXIT, p, CARD, ms),
                !keeps(EXIT, p, CARD, ms),
                "{p:?} 経過 {ms}"
            );
        }
    }
}

#[test]
fn hvsame_runner_path_in_chrome_order() {
    let target = HoverTarget {
        name: "span.ll-spark".to_string(),
        x: 100,
        y: 200,
        bottom: 210,
    };
    let card = HoverCard {
        left: 116.0,
        top: 180.0,
        width: 200.0,
        height: 90.0,
        vw: 1280.0,
        vh: 800.0,
    };
    let path = exit_path(&target, &card);
    assert_eq!(path, Some(((100, 212), (108, 212))));
    let Some(((ex, ey), (px, py))) = path else {
        return;
    };
    let e = pt(f64::from(ex), f64::from(ey));
    let p2 = pt(f64::from(px), f64::from(py));
    let rect = Rect {
        left: 116.0,
        top: 180.0,
        width: 200.0,
        height: 90.0,
    };
    assert!(!move_hides(e, e, rect, 0.0));
    assert!(!move_hides(e, p2, rect, 16.0));
    assert!(!keeps(e, e, rect, 0.0));
}

#[test]
fn hvsame_moved_calls_the_judge() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/widgets/hover.rs");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let from = "fn moved(self, now: Point)";
    let to = "fn hold(self)";
    assert_eq!(text.matches(from).count(), 1, "{from}");
    assert_eq!(text.matches(to).count(), 1, "{to}");
    let start = text.find(from).expect("moved の頭");
    let end = text.find(to).expect("hold の頭");
    assert!(start < end, "moved は hold の前");
    let body = &text[start..end];
    let call = "if move_hides(l.from, now, rect, Date::now() - l.at_ms) {";
    assert_eq!(body.matches(call).count(), 1, "{body}");
    assert_eq!(body.matches("keeps(").count(), 0, "{body}");
}
