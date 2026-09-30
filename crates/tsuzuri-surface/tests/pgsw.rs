//! 行 g-nav の歯（接頭辞 pgsw_）: 頁の link の押しは文書を読み直さずに URL を積んで頁を切り替える。
//! 押しの決め（frame の switch_url）は host で撃ち、DOM と履歴を撃つ所（board.rs と nodearound.rs・wasm の target の
//! ときだけ組む）の配線は CARGO_MANIFEST_DIR から読んだ fn の本文の字で見る。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::frame::{Mode, PageId, Press, href, switch_url};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} が読めない: {e}", path.display()))
}

/// fn の本文（行頭の `fn name(` か `pub fn name(` から、次の行頭の `fn `・`pub fn `・`#[component]` まで・字下げは見ない）。
fn fn_body<'a>(text: &'a str, name: &str) -> &'a str {
    let head = format!("fn {name}(");
    let mut start = None;
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let t = line.trim_start();
        if start.is_none() {
            if t.starts_with(&head) || t.starts_with(&format!("pub {head}")) {
                start = Some(at);
            }
        } else if t.starts_with("fn ") || t.starts_with("pub fn ") || t.starts_with("#[component]") {
            return &text[start.unwrap_or(0)..at];
        }
        at += line.len();
    }
    let s = start.unwrap_or_else(|| panic!("fn {name} が在る"));
    &text[s..]
}

/// 字の列がそれぞれ 1 度ずつ、この順で在る。
fn once_in_order(body: &str, words: &[&str]) {
    let mut last = 0;
    for w in words {
        assert_eq!(body.matches(w).count(), 1, "{w} が 1 度でない: {body}");
        let i = body.find(w).unwrap_or(0);
        assert!(i >= last, "{w} が前の字の後に無い: {body}");
        last = i;
    }
}

const LEFT: Press = Press {
    button: 0,
    ctrl: false,
    meta: false,
    shift: false,
    alt: false,
};

/// (1) 左の button を修飾の鍵なしで押し、違う頁へ行くときは frame の href の字（読み戻すと行き先の頁と同じ mode）。
#[test]
fn pgsw_plain_left_pushes_href() {
    assert_eq!(
        switch_url(PageId::Map, PageId::Ask, Mode::Beginner, LEFT).as_deref(),
        Some("?page=ask&mode=beginner")
    );
    assert_eq!(
        switch_url(PageId::Node, PageId::Home, Mode::Expert, LEFT).as_deref(),
        Some("?mode=expert")
    );
    assert_eq!(Press::default(), LEFT);
    let mut pairs = 0;
    for now in PageId::ALL {
        for to in PageId::ALL {
            if now == to {
                continue;
            }
            for mode in Mode::ALL {
                let url = switch_url(now, to, mode, LEFT)
                    .unwrap_or_else(|| panic!("{now:?} から {to:?} へ {mode:?} で None"));
                assert_eq!(url, href(to, mode));
                assert_eq!(PageId::from_query(&url), to, "{url}");
                assert_eq!(Mode::from_query(&url), mode, "{url}");
                pairs += 1;
            }
        }
    }
    let n = PageId::ALL.len();
    assert_eq!(pairs, n * (n - 1) * Mode::ALL.len());
}

/// (2) 修飾の鍵つきの押しと、左でない button の押しは browser の既定のまま（None）。
#[test]
fn pgsw_keys_or_button_keep_default() {
    let presses = [
        Press { ctrl: true, ..LEFT },
        Press { meta: true, ..LEFT },
        Press { shift: true, ..LEFT },
        Press { alt: true, ..LEFT },
        Press { button: 1, ..LEFT },
        Press { button: 2, ..LEFT },
    ];
    for p in presses {
        assert_eq!(switch_url(PageId::Map, PageId::Ask, Mode::Beginner, p), None, "{p:?}");
        assert_eq!(switch_url(PageId::Node, PageId::Home, Mode::Beginner, p), None, "{p:?}");
    }
}

/// (3) 今と同じ頁への押しは左の押しでも None（今どおり文書を読み直す）。
#[test]
fn pgsw_same_page_keeps_default() {
    for page in PageId::ALL {
        for mode in Mode::ALL {
            assert_eq!(switch_url(page, page, mode, LEFT), None, "{page:?} {mode:?}");
        }
    }
}

/// (4) board.rs の fn App・fn switch・fn press・fn top の配線の字。
#[test]
fn pgsw_board_wiring_text() {
    let board = read("src/board.rs");
    let app = fn_body(&board, "App");
    for w in [
        "let page = RwSignal::new(PageId::from_query(&query));",
        "{top(page, subject, mode)}",
        "{move || page_view(page.get())}",
        "{move || (page.get() == PageId::Home).then(|| view! { <CoachLayer/> })}",
        "on_cleanup(move || back.remove());",
        "page_view(",
    ] {
        assert_eq!(app.matches(w).count(), 1, "fn App の {w}: {app}");
    }
    let pop = app
        .find("window_event_listener(ev::popstate")
        .expect("fn App に window_event_listener(ev::popstate が在る");
    once_in_order(
        &app[pop..],
        &[
            "Mode::from_query(&now)",
            "go(page, subject, PageId::from_query(&now));",
        ],
    );

    let switch = fn_body(&board, "switch");
    once_in_order(
        switch,
        &[
            "frame::switch_url(",
            "e.prevent_default();",
            "push_state_with_url(",
            "go(page, subject, to);",
        ],
    );

    let press = fn_body(&board, "press");
    for w in [
        "e.button()",
        "e.ctrl_key()",
        "e.meta_key()",
        "e.shift_key()",
        "e.alt_key()",
    ] {
        assert!(press.contains(w), "fn press に {w} が無い: {press}");
    }

    let top = fn_body(&board, "top");
    assert_eq!(top.matches("href=move || frame::href(").count(), 2, "{top}");
    for w in [
        "on:click=move |e| switch(e, PageId::Home, page, subject, mode)",
        "on:click=move |e| switch(e, l.page, page, subject, mode)",
    ] {
        assert_eq!(top.matches(w).count(), 1, "fn top の {w}: {top}");
    }
}

/// (5) 頁を替える前に節点の頁の読みを捨てる（nodearound の forget と board の fn go）。
#[test]
fn pgsw_node_source_forget_text() {
    let around = read("src/project/nodearound.rs");
    let at = around.find("pub use dom::{").expect("pub use dom::{ が在る");
    let uses = &around[at + "pub use dom::{".len()..];
    let uses = &uses[..uses.find("};").expect("pub use dom の並びの終わり")];
    assert!(
        uses.split(',').any(|item| item.trim() == "forget"),
        "pub use dom の並びに forget が無い: {uses}"
    );
    let forget = fn_body(&around, "forget");
    assert!(forget.contains("SOURCE.set(None);"), "{forget}");

    let board = read("src/board.rs");
    let go = fn_body(&board, "go");
    once_in_order(
        go,
        &[
            "if page.get_untracked() == next {",
            "project::nodearound::forget();",
            "subject.set(PageSubject::default());",
            "page.set(next);",
        ],
    );
}
