//! 行 g-coach の歯（接頭辞 gcoach_）: 初心者の mode の home の頁の最初の案内（見本 mock v3 の ui.js の coach mark）。
//! 済み印は browser の保存に残し（持ち主の裁定 t3-hub.52.16・要件 FR1）、query の coach=1 で出し直す。
//! 始めの決め・段の飛ばし・button の字と点・輪と箱の置き場は host の純粋な関数で撃ち、DOM と保存を撃つ所の配線は source の字で見る。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::{next, seat};
use tsuzuri_surface::widgets::coach::{
    AGAIN, AGAIN_PARAM, BOX_W, COACH_KEY, DONE, GOT, LABEL, NEXT, Rect, SKIP, STEPS, box_at, dots,
    first_found, next_text, ring, starts,
};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} が読めない: {e}", path.display()))
}

/// 字 start から、その後の最初の字 end までの本文（start が無ければ panic）。
fn body<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    let i = text.find(start).unwrap_or_else(|| panic!("{start} が在る"));
    let rest = &text[i..];
    rest.find(end).map_or(rest, |j| &rest[..j])
}

/// (1) 鍵と値・3 段の selector と字・button の字は見本の字。
#[test]
fn gcoach_key_steps_and_words() {
    assert_eq!(COACH_KEY, "tz-coach");
    assert_eq!(DONE, "done");
    assert_eq!(AGAIN_PARAM, "coach");
    assert_eq!(AGAIN, "1");
    let sels: Vec<&str> = STEPS.iter().map(|s| s.sel).collect();
    assert_eq!(sels, ["#next .nxbig", "#orch .big", ".kcard"]);
    assert_eq!(next::BLOCK.id, "next");
    assert_eq!(seat::BLOCK.id, "orch");
    let js = read("../../docs/design/mock3/ui.js");
    let coach = body(&js, "var COACH = [", "];");
    for s in &STEPS {
        let text = format!("text: '{}'", s.text);
        assert!(coach.contains(&text), "見本の COACH に {text} が無い");
    }
    for sel in ["sel: '#next .nxbig'", "sel: '.kcard'"] {
        assert!(coach.contains(sel), "見本の COACH に {sel} が無い");
    }
    for w in [SKIP, NEXT, GOT, LABEL] {
        let quoted = format!("'{w}'");
        assert!(js.contains(&quoted), "見本に {quoted} が無い");
    }
    assert!(js.contains("store('tz-coach', 'done')"), "見本に済み印の書きが無い");
}

/// (2) 始めるのは初心者で、済み印が無いか query の coach=1 のときだけ。
#[test]
fn gcoach_start_rules() {
    assert!(starts(Mode::Beginner, None, ""));
    assert!(starts(Mode::Beginner, Some("later"), ""));
    assert!(!starts(Mode::Beginner, Some(DONE), ""));
    assert!(!starts(Mode::Beginner, Some(DONE), "?coach=0"));
    assert!(starts(Mode::Beginner, Some(DONE), "?coach=1"));
    assert!(starts(Mode::Beginner, Some(DONE), "?mode=beginner&coach=1"));
    assert!(!starts(Mode::Expert, None, ""));
    assert!(!starts(Mode::Expert, Some(DONE), "?coach=1"));
    assert!(!starts(Mode::Expert, None, "?coach=1"));
}

/// (3) 段 from から先で要素の在る最初の段（無ければ None）。
#[test]
fn gcoach_skip_missing_steps() {
    let all = |_: &str| true;
    assert_eq!(first_found(0, all), Some(0));
    assert_eq!(first_found(2, all), Some(2));
    assert_eq!(first_found(3, all), None);
    let cards = |s: &str| s == ".kcard";
    assert_eq!(first_found(0, cards), Some(2));
    let next_only = |s: &str| s == "#next .nxbig";
    assert_eq!(first_found(0, next_only), Some(0));
    assert_eq!(first_found(1, next_only), None);
    assert_eq!(first_found(0, |_: &str| false), None);
}

/// (4) 進む button の字は最後の段だけ分かった・点は今の段だけ真。
#[test]
fn gcoach_button_text_and_dots() {
    assert_eq!(NEXT, "次へ");
    assert_eq!(GOT, "分かった");
    assert_eq!(SKIP, "閉じる");
    assert_eq!(LABEL, "手引き");
    assert_eq!(next_text(0), NEXT);
    assert_eq!(next_text(1), NEXT);
    assert_eq!(next_text(2), GOT);
    assert_eq!(dots(0), [true, false, false]);
    assert_eq!(dots(1), [false, true, false]);
    assert_eq!(dots(2), [false, false, true]);
}

/// (5) 輪は枠の 4px 外（scroll を足す）・箱は要素の下 12px で横は 8px の余白で窓に収める。
#[test]
fn gcoach_ring_and_box_place() {
    let el = Rect {
        left: 100.0,
        top: 50.0,
        width: 300.0,
        height: 40.0,
    };
    assert_eq!(
        ring(el, 0.0, 200.0),
        Rect {
            left: 96.0,
            top: 246.0,
            width: 308.0,
            height: 48.0,
        }
    );
    assert_eq!(BOX_W, 240.0);
    assert_eq!(box_at(el, 0.0, 200.0, 1200.0), (100.0, 302.0));
    let right = Rect { left: 1100.0, ..el };
    assert_eq!(box_at(right, 0.0, 0.0, 1200.0).0, 952.0);
    let out = Rect { left: -20.0, ..el };
    assert_eq!(box_at(out, 0.0, 0.0, 1200.0).0, 8.0);
    assert_eq!(box_at(el, 30.0, 0.0, 1200.0).0, 130.0);
}

/// (6) stylesheet は案内の箱と輪と点の規則を持つ。
#[test]
fn gcoach_stylesheet_classes() {
    let css = read("style.css");
    for w in [".coach {", ".coach-ring {", ".coach .ft .dots i.on"] {
        assert!(css.contains(w), "stylesheet に {w} が無い");
    }
}

const WASM: &str = "#[cfg(target_arch = \"wasm32\")]";

/// (7) 層は home の頁だけ・保存と DOM を撃つ所の字（mod.rs・board.rs・coach.rs）。
#[test]
fn gcoach_wasm_wiring() {
    let module = read("src/widgets/mod.rs");
    assert!(module.contains("pub mod coach;"), "widgets の mod.rs に pub mod coach; が無い");
    let board = read("src/board.rs");
    let app = body(&board, "fn App()", "\n}\n");
    assert!(
        app.contains("{(page == PageId::Home).then(|| view! { <CoachLayer/> })}"),
        "fn App に home の頁だけの案内の層が無い: {app}"
    );

    let src = read("src/widgets/coach.rs");
    let lines: Vec<&str> = src.lines().collect();
    let at = lines
        .windows(2)
        .position(|w| w[0].trim() == WASM && w[1].trim() == "mod dom {")
        .expect("wasm の cfg の行の直後に mod dom");
    let dom = lines[at + 1..].join("\n");
    for w in [
        "store::get(COACH_KEY)",
        "starts(mode, store::get(COACH_KEY).as_deref(), &search)",
        "get_bounding_client_rect()",
        "first_found(from, found).and_then(place)",
        "set_timeout(move || wait(tries - 1, go), WAIT)",
        "class=\"coach-ring\"",
    ] {
        assert!(dom.contains(w), "mod dom に {w} が無い");
    }
    let coach = body(&dom, "class=\"coach\"", ">");
    assert!(coach.contains("role=\"dialog\""), "class coach の要素に role dialog が無い: {coach}");
    let skip = body(&dom, "let skip = move |_| {", "};");
    assert!(
        skip.contains("store::set(COACH_KEY, DONE)"),
        "閉じるに済み印の書きが無い: {skip}"
    );
    let esc = body(&dom, "window_event_listener(ev::keydown", "});");
    assert!(esc.contains("\"Escape\""), "keydown に Escape が無い: {esc}");
    assert!(!esc.contains("store::"), "Esc が保存を撃つ: {esc}");
    for w in ["local_storage", "get_item(", "set_item("] {
        assert!(!src.contains(w), "coach.rs に {w} が在る");
    }
}
