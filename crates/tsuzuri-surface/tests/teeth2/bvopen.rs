//! 行 g-one-screen-b の歯（接頭辞 bvopen_）: 窓を開く link（home の頁の URL の query の win）は、home の頁では普通の押しで
//! 頁を読み直さずにその窓を開き、問いの id を持てば質問の窓をその問いから出す（判断の記録 ADR-27 決定 (4)）。
//! link の読みは topbar の純粋な関数を host で撃ち、DOM の配線は CARGO_MANIFEST_DIR から読んだ字で見る。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::node::answer_href;
use tsuzuri_surface::topbar::{Win, win_href, win_of_href};
use tsuzuri_surface::widgets::pop::{NEXT_KEYS, next_href};

fn read(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 窓を開く link は窓と問いの id に読み戻り（id の %XX は戻す）、`?` より前と井桁から後は見ず、win の無い link と
/// 知らない窓の名と `?` の無い字は None・空の id は None。
#[test]
fn bvopen_win_of_href() {
    for mode in Mode::ALL {
        for w in Win::ALL {
            assert_eq!(win_of_href(&win_href(w, mode)), Some((w, None)));
        }
        assert_eq!(
            win_of_href(&answer_href("e.2:x", mode)),
            Some((Win::Ask, Some("e.2:x".to_string())))
        );
        assert_eq!(
            win_of_href(&next_href(NEXT_KEYS[0], "t-1", mode)),
            Some((Win::Ask, None))
        );
        assert_eq!(win_of_href(&next_href(NEXT_KEYS[1], "t-1", mode)), None);
    }
    assert_eq!(
        win_of_href("http://127.0.0.1:8120/?mode=expert&win=gaps#x"),
        Some((Win::Gaps, None))
    );
    assert_eq!(
        win_of_href("?win=ask&id=#q"),
        Some((Win::Ask, None)),
        "空の id"
    );
    assert_eq!(win_of_href("?page=node&id=FR1&mode=expert"), None);
    assert_eq!(win_of_href("?win=bogus"), None);
    assert_eq!(win_of_href("win=ask"), None);
    assert_eq!(win_of_href("#win=ask"), None);
}

/// App は問いの id の context を置き、URL の窓を問いの id と一緒に開き、home の頁だけで link の押しを聞く。
/// 押しは窓を開く link の普通の押しだけ既定の遷移を止め、窓の中の link は前の窓に戻る口を持つ。質問の窓は id の問いから出す。
#[test]
fn bvopen_wiring_text() {
    let board = read("src/board.rs");
    for want in [
        "let focus = AskFocus(RwSignal::new(None));",
        "provide_context(focus);",
        "if let Some((w, id)) = win_of_href(&query) {",
        "focus.0.set(id);",
        "if page == PageId::Home {",
        "window_event_listener(ev::click, move |e| open_link(&e, win, focus))",
        "let Some(link) = el.closest(\"a[href]\").ok().flatten() else {",
        "let Some((w, id)) = win_of_href(&href) else {",
        "if !plain_click(e) {",
        "e.prevent_default();",
        "let from_win = el.closest(&format!(\"#{}\", SCRIM)).ok().flatten().is_some();",
        "win.open(w, from_win);",
    ] {
        assert!(board.contains(want), "board.rs に {want} が無い");
    }
    let ask = read("src/askwin.rs");
    for want in [
        "pub struct AskFocus(pub RwSignal<Option<String>>);",
        "let first = use_context::<AskFocus>().and_then(|f| f.0.get_untracked());",
        "let picked = RwSignal::new(first);",
    ] {
        assert!(ask.contains(want), "askwin.rs に {want} が無い");
    }
}
