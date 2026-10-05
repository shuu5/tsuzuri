//! project board の「戻る」の歯（接頭辞 acctwire_・行 h-wire の完了の条件 (6)(7)・header の (5) は行 g-dead-sweep-b で消した）。
#![cfg(test)]

use std::path::Path;

use crate::common::read;
use tsuzuri_surface::account::windows::ACCOUNT_WIN;
use tsuzuri_surface::frame::{ACCOUNT_URL, BackStep, back_steps};

/// (6) tz-account の窓が在れば前面へ、無ければ ?board=account を新しい窓で開き、どちらの後も自分の窓を閉じる。
#[test]
fn acctwire_back_steps() {
    assert_eq!(back_steps(true), [BackStep::Front, BackStep::CloseSelf]);
    assert_eq!(
        back_steps(false),
        [BackStep::OpenNew("?board=account"), BackStep::CloseSelf]
    );
    assert_eq!(ACCOUNT_URL, "?board=account");
    assert!(
        tsuzuri_surface::account::selects(ACCOUNT_URL),
        "account board の入口が選ぶ"
    );
    assert_eq!(ACCOUNT_WIN, "tz-account");
    // 描画は frame の段の列に従い、window の open は空の URL と tz-account を渡す（wasm の target だけ）。
    let board = read("src/board.rs");
    for word in [
        "frame::back_steps(",
        "open_named(\"\", ACCOUNT_WIN)",
        "BackStep::CloseSelf",
        "me.close()",
        "back: Callback::new(move |()| back_to_board(note))",
        "{topbar::view(bar)}",
    ] {
        assert!(board.contains(word), "board.rs に {word} が無い");
    }
    let lib = read("src/lib.rs");
    assert!(lib.contains("#[cfg(target_arch = \"wasm32\")]\npub mod board;"));
    let frame_src = read("src/frame.rs");
    assert!(!frame_src.contains("web_sys"), "frame は window を撃たない");
}

/// (7) 面の crate の依存の数は変わらない（Cargo.toml は触らない）。
#[test]
fn acctwire_no_new_dependencies() {
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("Cargo.toml");
    assert!(!text.contains("url ="), "{text}");
    assert!(text.contains("\"Location\""), "web-sys の Location の feature");
    assert!(text.contains("\"Window\""), "web-sys の Window の feature");
}
