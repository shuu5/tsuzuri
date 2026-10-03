//! 行 c-seat-tick の歯（面）: project board の block「orchestrator と口座」の上段の tick の欄に、器の tick の語の
//! 印と class と語と最後の tick からの経過を出す字。
//! fixture: tests/fixtures/surface/seat-card.json（鍵 tick と tick_at が無いので、どの組も tick は Unknown）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::{SeatCard, TickHealth};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::seat::{
    BLANK, NG, OK, age_text, tick_age, tick_class, tick_mark, top,
};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn card(name: &str) -> SeatCard {
    let mut all: BTreeMap<String, SeatCard> =
        wire::decode(&read("../../tests/fixtures/surface/seat-card.json"))
            .expect("fixture の組が電文として読める");
    all.remove(name)
        .unwrap_or_else(|| panic!("fixture の組 {name}"))
}

/// 組の card の tick と tick_at を替えた card。
fn with_tick(name: &str, tick: Reading<TickHealth>, tick_at: Option<u64>) -> SeatCard {
    let mut c = card(name);
    c.tick = tick;
    c.tick_at = tick_at;
    c
}

/// (10) 語ごとの印と class・語が読めない周は健康の真偽から・上段の欄 tick は今の値のまま。
#[test]
fn ctick_sign_and_class_per_word() {
    assert_eq!((BLANK.glyph, BLANK.class), ("", "gi unknown"));
    for (word, sign, class) in [
        (TickHealth::Healthy, OK, "tk tk-healthy"),
        (TickHealth::Stale, NG, "tk tk-stale"),
        (TickHealth::Absent, BLANK, "tk tk-absent"),
        (TickHealth::Unreadable, NG, "tk tk-unreadable"),
    ] {
        let t = top(&with_tick("run", Reading::Known(word), None));
        assert_eq!(tick_mark(&t), Reading::Known(sign), "{}", word.as_str());
        assert_eq!(tick_class(&t), class, "{}", word.as_str());
        assert_eq!(t.tick, Reading::Known(OK), "{}", word.as_str());
        assert_eq!(t.tick_word, Reading::Known(word));
    }
    for (name, sign, class) in [
        ("run", Reading::Known(OK), "tk tk-healthy"),
        ("wait", Reading::Known(NG), "tk tk-stale"),
        ("silent", Reading::Unknown, "tk"),
    ] {
        let t = top(&card(name));
        assert_eq!(t.tick_word, Reading::Unknown, "{name}");
        assert_eq!(tick_mark(&t), sign, "{name}");
        assert_eq!(tick_class(&t), class, "{name}");
        assert_eq!(t.tick, sign, "{name}");
    }
}

/// (11) 経過の字の段。
#[test]
fn ctick_age_text_steps() {
    let got: Vec<String> = [0, 59, 60, 3_599, 3_600, 3_660, 172_799, 172_800, 777_600]
        .into_iter()
        .map(age_text)
        .collect();
    assert_eq!(
        got,
        ["0s", "59s", "1m", "59m", "1h", "1h01", "47h59", "2d", "9d"]
    );
}

/// (12) 経過は今と電文の at の大きい方から tick_at を引き、absent・unreadable・時刻の無い時は出さない。
#[test]
fn ctick_age_from_now_or_at() {
    let at = card("run").at;
    assert_eq!(at, 1_790_510_400);
    let t = top(&with_tick(
        "run",
        Reading::Known(TickHealth::Healthy),
        Some(at - 10),
    ));
    assert_eq!(t.at, at);
    assert_eq!(t.tick_at, Some(at - 10));
    assert_eq!(tick_age(&t, at - 100).as_deref(), Some("10s"));
    assert_eq!(tick_age(&t, at + 50).as_deref(), Some("1m"));
    let t = top(&with_tick(
        "run",
        Reading::Known(TickHealth::Stale),
        Some(at - 7_200),
    ));
    assert_eq!(tick_age(&t, at).as_deref(), Some("2h"));
    let t = top(&with_tick("run", Reading::Unknown, Some(at - 5)));
    assert_eq!(tick_age(&t, at).as_deref(), Some("5s"));
    for (tick, tick_at) in [
        (Reading::Known(TickHealth::Absent), Some(at - 5)),
        (Reading::Known(TickHealth::Unreadable), Some(at - 5)),
        (Reading::Known(TickHealth::Healthy), None),
        (Reading::Unknown, None),
    ] {
        let t = top(&with_tick("run", tick.clone(), tick_at));
        assert_eq!(tick_age(&t, at), None, "{tick:?} {tick_at:?}");
    }
}

/// (14) DOM の top_view の字と stylesheet の規則。
#[test]
fn ctick_dom_text() {
    let dom = read("src/project_dom/seat.rs");
    let start = dom.find("fn top_view(").expect("top_view が在る");
    let len = dom[start..].find("\n}\n").expect("top_view の閉じ");
    let body = &dom[start..start + len];
    for w in [
        "tick_class(&top)",
        "tick_mark(&top)",
        "crate::net::ticker()",
        "tick_age(",
        "clock.get()",
        "<span class=\"num\">",
        "<b>{w.as_str()}</b>",
        "hs(\"tick_health\")",
    ] {
        assert!(body.contains(w), "top_view に {w} が無い");
    }
    // 語の b は tick_health の後・経過は語の後（見本の tkhbHTML の順）。
    let row = body
        .lines()
        .find(|l| l.contains("hs(\"tick_health\")"))
        .expect("tick の欄の行");
    let at = |w: &str| row.find(w).unwrap_or_else(|| panic!("tick の欄に {w} が無い"));
    assert!(at("hs(\"tick_health\")") < at("{word}"));
    assert!(at("{word}") < at("{age}"));
    assert!(!dom.contains("tk tk-healthy"), "DOM に class の字 tk tk-healthy");

    let css = read("style.css");
    for rule in [".tkhb .tk-stale b", ".tkhb .tk-healthy b", ".gi.unknown"] {
        assert!(css.contains(rule), "style.css に {rule} が無い");
    }
}
